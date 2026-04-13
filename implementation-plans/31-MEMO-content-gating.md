## Memo: Content Gating With Tera

Date: 2026-04-06

## Decision

For prompt-level token reduction and chapter-aware prompt behavior, prefer Tera content gating inside prompt templates over adding new pipeline execution infrastructure.

This means:

- Keep the pipeline runner simple.
- Use Tera conditionals to include or exclude expensive context blocks at render time.
- Treat this as content shaping, not orchestration.

This is the preferred short-term solution when the goal is to reduce tokens, avoid unnecessary document injections, or tailor prompt weight by chapter or mode.

## Why This Decision Was Made

The alternative would be adding more execution-layer logic for per-step conditions, skip rules, or richer runtime policies. That would increase product and code complexity across:

- pipeline schema
- pipeline editor UI
- execution logic
- progress and resume behavior
- run history semantics
- test surface area

For many real authoring cases, the problem is narrower: some context is only useful after a certain chapter, or only useful in certain modes. Tera already solves that cleanly.

## What Tera Content Gating Is Good For

Use content gating when you want to:

- exclude large docs for early chapters
- include context only after a threshold like `chapter >= 2`
- swap between light and heavy prompt variants
- include optional notes only when a variable is enabled
- reduce token load without changing pipeline structure

Examples:

```tera
{% if chapter | int >= 2 %}
{{ "World_Bible.md" | doc }}
{% endif %}
```

```tera
{% if do_editorial_pass == "true" %}
{{ "Editorial_Guide.md" | doc }}
{% endif %}
```

```tera
{% set chapter_num = chapter | int %}
{% set include_heavy_context = chapter_num >= 3 %}

{% if include_heavy_context %}
{{ "Relationship_Notes.md" | doc }}
{% endif %}
```

## What Tera Content Gating Is Not

Tera content gating does not skip execution of a prompt block.

The block still runs.
The pipeline still invokes it.
The app still records a run artifact.

Tera only changes the rendered prompt payload for a block that is already being executed.

So this memo recommends Tera gating for prompt size and prompt shape, not for execution routing.

## Decision Boundary

Use Tera content gating when the question is:

- "Should this chunk of context appear in the rendered prompt?"

Do not use Tera content gating as the primary solution when the question is:

- "Should this block run at all?"
- "Should this block loop differently from another block?"
- "Should this block start at chapter 2 while another starts at chapter 1?"

Those are execution-layer concerns and should remain outside this memo.

## Prompting Shape Guidelines

Preferred shape:

1. Compute a small set of booleans near the top.
2. Use those booleans to control expensive `doc()` insertions.
3. Keep the instruction body stable whenever possible.
4. Gate context sections, not just random lines.

Preferred pattern:

```tera
{% set chapter_num = chapter | int %}
{% set include_prev_summary = chapter_num > 1 %}
{% set include_world_bible = chapter_num >= 2 %}
```

Then:

```tera
{% if include_world_bible %}
# World Context
{{ "World_Bible.md" | doc }}
{% endif %}
```

This reads better and scales better than repeating `chapter | int` checks all over the prompt.

## Example: Repetition Audit Prompt

Target prompt:

- `Sample Projects/Neon & Nightmares 2/prompts/03-repetition-audit.tera`

Current shape:

```tera
{% set prev_chapter = chapter | int - 1 %}
{% set _prev_draft = "chapter-" ~ prev_chapter ~ "-06-final.md" %}
{% set _curr_draft = "chapter-" ~ chapter ~ "-02-draft.md" %}
<system>
You are an eagle-eyed developmental continuity editor specializing in Urban Fantasy.
</system>

<context>
# The Previous Chapter
{{ _prev_draft | doc }}

# The New Draft
{{ _curr_draft | doc }}
</context>
```

### Problem

For chapter 1, there is no meaningful previous chapter draft. Pulling a previous chapter file is either wasteful, fragile, or conceptually wrong.

### Recommended gated shape

```tera
{% set chapter_num = chapter | int %}
{% set prev_chapter = chapter_num - 1 %}
{% set include_prev_draft = chapter_num > 1 %}
{% set prev_draft = "chapter-" ~ prev_chapter ~ "-06-final.md" %}
{% set curr_draft = "chapter-" ~ chapter ~ "-02-draft.md" %}

<system>
You are an eagle-eyed developmental continuity editor specializing in Urban Fantasy.
</system>

<context>
{% if include_prev_draft %}
# The Previous Chapter
{{ prev_draft | doc }}
{% endif %}

# The New Draft
{{ curr_draft | doc }}
</context>

<instruction>
{% if include_prev_draft %}
Review the new draft against the previous chapter specifically for narrative redundancy.
{% else %}
Review the new draft for internal repetition only. Because this is chapter 1, do not assume there is a previous chapter to compare against.
{% endif %}

1. **Emotional Repetition:** Did the characters restate feelings they already worked through?
2. **Dialogue Loops:** Are there repeated arguments or conversational beats?
3. **Action Redundancy:** Did we see similar physical blocking or scene setups?
4. **Trope Spam:** Are we leaning too heavily on one trope cluster?

Produce an itemized "Repetition Audit Report". If there are issues, recommend precise cuts and substitutions. If it flows perfectly, output "CLEAN PASS".
</instruction>
```

### Why this shape is better

- Chapter 1 no longer pays to inject nonexistent or unnecessary prior-chapter context.
- The instruction stays semantically correct for both chapter 1 and later chapters.
- The gating is explicit and easy to maintain.
- The expensive `doc()` call is only used when it adds value.

## Recommended Guardrails

- Prefer gating whole sections over interleaving tiny conditional fragments.
- Give gated booleans descriptive names like `include_prev_draft`.
- When chapter 1 changes the logic, update the instruction copy as well as the context.
- Avoid hiding essential continuity context for later steps unless the prompt is intentionally lightweight.

## Recommendation Going Forward

Use Tera content gating as the default short-term pattern for token control inside prompts.

Examples of high-value uses:

- chapter 1 skips previous-chapter docs
- early chapters skip large world docs
- optional editorial passes only inject their guide when enabled
- later-stage prompts load heavier support material than early-stage prompts

Keep execution-layer changes reserved for genuine orchestration problems rather than prompt-size optimization.