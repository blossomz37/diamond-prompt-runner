# Diamond Workflow Checklist

Use this before closing a Diamond prompt-authoring task.

## Structure
- Is each piece of information in the right place: variables, documents, prompts, models, pipelines?
- Are repo-relative paths used in guidance and examples?
- If a large markdown source is queried selectively, has it been normalized first?
- If the user is authoring the project on disk, does `project.json` exist and include a UUID-shaped `projectId`?
- Does `project.json` contain the prompt block and variables needed for the payload the user plans to send?

## Prompt Design
- Does the prompt pull only the slices it needs?
- Does it state the output contract explicitly?
- If using wrappers, does it tell the model to preserve them exactly?
- If continuity matters, does it lock canon clearly?

## Looping And Files
- Are loop variables such as `count` or chapter number handled explicitly?
- Are output filenames deterministic and reusable?
- If using append behavior, is the rerun story acceptable?
- Do the payload keys match the variable names actually used by the prompt block and prompt template?

## Model Choice
- Is the chosen model matched to the task type?
- If reasoning should be off, is the `reasoning` block omitted from the model preset?

## Validation
- Is there a representative sample run or sample-project analogy?
- Would the user be able to edit the author-facing content without editing engine-only config?
- If helper scripts are being shared, are they clearly marked as illustrative and mapped to the user's real paths, pipeline ids, payloads, and cleanup rules?
