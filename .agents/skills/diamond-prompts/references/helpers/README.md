# Helpers

These helper files are illustrative patterns, not turnkey scripts.

Do not tell a user to run them unchanged on another machine without first mapping:
- project path
- pipeline id
- payload shape
- generated output files
- backup behavior
- local build assumptions

Use these helpers as adaptation templates.

## Included Helpers

### `normalize-markdown-boundaries.mjs`
Portable markdown-wrapper utility. Good for converting heading-based source docs into explicit bracket boundaries before selective retrieval.

### `test-headless.mjs`
Single-run headless CLI test harness. Good for validating one project, one pipeline, and one payload.

### `run-range-template.sh`
Illustrative batch runner template for count-based loops, chapter ranges, or ordered section sequences.

### `backup-and-clear-output-template.sh`
Illustrative cleanup helper for experiment-heavy workflows where generated outputs should be backed up before being cleared.
