---
name: "diamond-prompts"
description: "Help user adapt prompt workflows to Diamond Prompt Runner using Tera prompts, variables, documents, models, and pipelines."
---

# SKILL: Diamond Prompt Runner Prompt Authoring

## Purpose
Use this skill to adapt a writing or prompt workflow into Diamond Prompt Runner project files.

Diamond projects are file-first:
- prompts are `.tera`
- source material usually lives in `documents/`
- model presets are YAML in `models/`
- pipelines are ordered prompt-block sequences
- structured execution context must be exposed deliberately

## Use This Skill When
- a user wants to convert an existing prompt workflow into Diamond files
- a prompt needs looping, counters, per-step output files, or batch generation
- a workflow needs selective document injection instead of whole-document context
- a large markdown source needs to be normalized into parseable sections
- a fiction pipeline needs planning, drafting, editing, summaries, or continuity control

## Non-Negotiable Engine Truths
1. Raw YAML or Markdown in `documents/` does not automatically become a Tera variable.
2. Small execution-critical values belong in project or workspace variables.
3. Large author-facing material belongs in documents and should be pulled in intentionally.
4. There is no general-purpose include system for arbitrary structured docs; the working pattern is `doc(...)` plus section extraction.
5. Prompts are not runnable by themselves; `project.json` is the execution manifest, and payload keys must line up with the variable names the prompt actually reads.

## Default Working Rules
- Put small structured lists in variables.
- Put rich source material in documents.
- Normalize large markdown into explicit wrappers before selective retrieval.
- Normalize to the deepest level you expect to retrieve later.
- Require stable output headers for reusable generated files.
- For cascade builds, lock canon explicitly and preserve wrapper structure exactly.
- Use repo-relative paths in skill guidance and examples. Do not assume a machine-specific workspace path.

## Execution Workflow
1. Identify the user's assets: prompt text, source docs, loop variables, outputs, and model needs.
2. Decide file roles:
   - variables for small structured values
   - documents for rich source text
   - prompts for Tera logic and formatting contracts
   - models for provider presets
   - pipelines for execution order
   - `project.json` for the runnable manifest, including `projectId`, `variables`, and `promptBlocks`
3. If a large markdown document will be queried selectively, normalize it into wrappers first.
4. Author the prompt so it pulls only the needed slices and emits a stable, reusable output shape.
5. Configure filenames and loop variables so reruns produce predictable artifacts.
6. If the user is authoring the project on disk before creating it in the Diamond UI, make sure `project.json` exists and includes a UUID-shaped `projectId`, for example `"projectId": "3ba62a39-f7fa-4d95-8fe9-3dfa614c2442"`.
7. Validate the workflow against a sample run or a sample-project pattern before calling it done.

## Reference Map
Open only the note you need:

- `references/core-patterns.md`
  Core engine rules, file roles, normalization guidance, rolling context, headers, canon-lock, and append-vs-rebuild tradeoffs.

- `references/sample-projects.md`
  Concrete patterns drawn from `docs/sample-projects/sample-project-2`, `sample-project-3`, and `sample-project-4`.

- `references/checklist.md`
  A short build/review checklist for turning a workflow into a Diamond project.

- `references/helpers/`
  Illustrative utility scripts and templates for normalization, headless testing, range runs, and cleanup planning. These are examples to adapt, not drop-in scripts to run unchanged on another machine.

## Deliverable Standard
When you help a user build a Diamond workflow, prefer delivering:
- the recommended file layout
- the specific files to create or edit
- the Tera pattern that makes the workflow work
- the variable or document structure needed for the prompt
- any normalization step needed for large markdown sources

## Most Important Warning
Do not tell the user that a raw structured document will be directly available as a prompt variable unless that structure is actually being loaded from variables or deliberately extracted from document text.
