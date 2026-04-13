# Core Patterns

## Engine Reality
- Raw files in `documents/` are text sources, not automatic variables.
- Reliable structured execution-time values come from:
  - `project.json` variables
  - `variables/workspace-variables.yaml`
  - system variables such as `count`, `current_date`, and `project.*`
- The practical component pattern is:
  - `doc(...)`
  - normalized wrappers
  - `extract_section(...)`

## File Roles
- `project.json`
  Prompt blocks, pipelines, filename templates, defaults, and small project-level variables.

  This is the execution manifest for a Diamond project. It defines what the runner can execute and how payload values map into prompt behavior.

  The most important areas are:
  - `projectId`
  - `variables`
  - `promptBlocks`
  - `defaultModelPreset`

  Typical `projectId` shape:

  ```json
  "projectId": "3ba62a39-f7fa-4d95-8fe9-3dfa614c2442"
  ```

  Use a UUID-shaped value. This matters most when a user is authoring a project on disk before creating it in the Diamond UI.

- `variables/workspace-variables.yaml`
  Author-editable structured values that should be available to prompts.

- `documents/*.md` or `documents/*.txt`
  Rich source material, generated outputs, summaries, reports, dossiers, outlines, and reference prose.

- `prompts/*.tera`
  Prompt logic, selection logic, formatting contracts, and injected context blocks.

- `models/*.yaml`
  Model/provider presets and task routing.

## `project.json` Structure Matters
The runner executes prompt blocks from `project.json`, not from loose prompt files alone.

That means:
- a `.tera` file is not enough by itself
- a prompt becomes runnable when a prompt block references it
- payload values only matter if the prompt block and prompt logic actually use them

Typical shape:

```json
{
  "projectId": "3ba62a39-f7fa-4d95-8fe9-3dfa614c2442",
  "projectName": "my-project",
  "defaultModelPreset": "models/default.yaml",
  "variables": {
    "count": "01"
  },
  "promptBlocks": [
    {
      "blockId": "flash-fiction",
      "name": "Flash Fiction",
      "templateSource": "prompts/flash-fiction.tera",
      "inputBindings": [],
      "modelPreset": "models/gpt-5.4-nano.yaml",
      "outputTarget": "replace_document",
      "outputFilename": "flash-fiction-{{ count | default(value=\"01\") }}.md"
    }
  ]
}
```

Read it this way:
- `variables` expose small structured values to Tera
- `promptBlocks` tell the runner which prompt file to execute
- `outputFilename` often depends on payload variables such as `count` or `chapter`
- pipeline payloads must line up with the variable names the prompt actually reads

Example:
- if the payload sends `{"count": "03"}`
- and the prompt selects from `count`
- and `outputFilename` also uses `count`
- then one run can both choose the third item and write `flash-fiction-03.md`

## Pattern 1: Small Lists Live In Variables
For a loopable list such as flash-fiction prompt options, expose the list through variables and select the active item from `count`.

Example:

```tera
{% set active_count = count | default(value="01") | int %}
{% set current_prompt_option = prompt_options[active_count - 1] %}
```

Use this for prompt options, chapter numbers, beat selectors, or other short structured lists.

## Pattern 2: Normalize Large Markdown Before Selective Retrieval
If a document will be queried in pieces, normalize headings into explicit wrappers first.

Example shape:

```text
[authorial_style_and_voice]
## authorial_style_and_voice

[core_prose_style_directives]
### core_prose_style_directives
...
[/core_prose_style_directives]

[/authorial_style_and_voice]
```

Choose wrapper depth based on the smallest unit you expect to retrieve later, not just the current build step.

## Pattern 3: Extract Only What The Step Needs
Once a document is normalized, slice narrowly.

```tera
{% set dossier = "Story_Dossier_Worksheet.boundaries.md" | doc %}
{% set voice_rules = dossier | extract_section(start="[authorial_style_and_voice]", end="[/authorial_style_and_voice]") | trim %}
```

When generating from a wrapped shell, explicitly require the model to preserve all nested wrappers exactly as given.

## Pattern 4: Use Rolling Context Windows
For long-form fiction pipelines:
- distant history -> summaries
- recent history -> full chapters or drafts

Do not keep sending the entire manuscript forever.

## Pattern 5: Number Injected Context Blocks
Wrap prior chapters or summaries with explicit labels.

```tera
[Summary Chapter {{ i }}]
{{ summary_path | doc }}
[/Summary Chapter {{ i }}]

[Chapter {{ i }}]
{{ chapter_path | doc }}
[/Chapter {{ i }}]
```

This makes context boundaries clearer to the model.

## Pattern 6: Require Stable Output Headers
Reusable generated files should start with a predictable `##` heading.

Examples:
- `## Chapter 4 Plan`
- `## Chapter 4 Summary`
- `## Chapter 4`
- `## Chapter 4 Repetition Report`

## Pattern 7: Audit Prompts Need A Method
For repetition/style/craft audits, require a checking procedure, not just a goal.

Typical requirements:
- scan for candidates first
- allow a "no issues" result only after a real scan
- specify the issue format

## Pattern 8: Cascade Prompts Need Canon Lock
When building a dossier section-by-section:
- treat author seed facts as canon
- treat the built dossier so far as canon
- if canon and inference conflict, canon wins
- do not rename characters, roles, or core setup facts unless upstream canon changes them

## Pattern 9: Tiny Canon Seeds May Need Repeating
If a seed document is tiny but foundational, resending it every cascade step can improve fidelity.

Use this judgment:
- large source docs -> summarize, slice, or use once
- tiny canon seeds -> often worth resending

## Pattern 10: Append vs Rebuild Is A Workflow Choice
- `append_document` is fast for dossier cascades
- rerunning a step can duplicate content unless the target is cleared first
- `replace_document` is safer when reruns are common

Pick the pattern based on correction frequency, not preference alone.

## Model Routing Guidance
- analytical planning, audits, and structured editing -> stronger analytical model
- long prose drafting/finalization -> stronger prose model
- summaries and cheap transforms -> smaller fast model

If reasoning should be disabled, omit the `reasoning` block from the model YAML.
