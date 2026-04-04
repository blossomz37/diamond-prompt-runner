---
created: 2026-04-04 9:30 AM
modified: 2026-04-04 9:30 AM
status: complete
---
# Plan: Usage Metrics Extraction Slice

## Summary

The execution pipeline already persists the full raw OpenRouter response in every run artifact, but the structured usage fields — tokens, cost, and output word count — were not extracted or surfaced. This slice closes the spec gap for "track retries, execution status, tokens, cost, and timestamps" (Spec §3, §4.5) by extracting usage metrics from the raw response and threading them through the full data path from execution result to persisted artifact to run history to the UI.

## Scope

Included in this slice:
- define a `UsageMetrics` struct carrying `prompt_tokens`, `completion_tokens`, `total_tokens`, `cost`, and `output_word_count` — all optional to handle provider responses that omit fields
- add an `extract_usage_metrics()` function that pulls token and cost data from the raw OpenRouter response and computes output word count from the completion text
- thread the `usage` field through `PromptExecutionResult`, `PersistedRunRecord`, `PromptRunHistoryEntry`, and `ProjectRunHistoryEntry`
- display usage metrics (tokens, cost, word count) in the bottom-panel execution metadata and in run history item summaries
- add backend tests for usage extraction with full and missing fields
- bump `PERSISTED_RUN_RECORD_VERSION` to 3 with backward-compatible `#[serde(default)]` deserialization

Explicitly out of scope for this slice:
- retry count tracking (requires an execution retry loop, not just extraction)
- cost aggregation dashboards or project-level usage summaries
- provider-specific cost calculation formulas beyond what OpenRouter returns in `usage.cost`
- any changes to the execution transport itself

## Spec Items Addressed

- §3 "track retries, execution status, tokens, cost, and timestamps" — partially DONE; tokens, cost, timestamps, and status are now fully tracked; retry count remains deferred
- §4.5 "Usage Metrics" — prompt tokens, completion tokens, estimated cost, and output word count are now extracted and persisted; retry count deferred

## Technical Approach

### Backend (`src-tauri/src/project_store.rs`)

1. **`UsageMetrics` struct** — `Serialize`, `Deserialize`, `PartialEq`, `Default`. All fields are `Option` because OpenRouter may omit fields depending on the model.
2. **`extract_usage_metrics(response, output)`** — reads `response.usage.prompt_tokens`, `.completion_tokens`, `.total_tokens`, `.cost` from the JSON response. Computes `output_word_count` from `output.split_whitespace().count()`.
3. **Struct threading** — `usage: UsageMetrics` added to `PromptExecutionResult`, `PersistedRunRecord`, `PromptRunHistoryEntry`, `ProjectRunHistoryEntry` with `#[serde(default)]` for backward compatibility with v2 artifacts.
4. **Wiring** — `extract_usage_metrics()` called in `execute_prompt_block_with_transport()` right after the response is received, alongside `extract_online_run_metadata()`.
5. **Artifact version** — bumped from 2 to 3. Older artifacts still load cleanly because all new fields have serde defaults.

### Frontend (`src/lib/types/project.ts`, `ValidationPanel.svelte`, `App.svelte`)

1. **`UsageMetrics` interface** — mirrors the backend struct with `null` instead of `Option`.
2. **`usage` field** — added to `PromptExecutionResult` and `ProjectRunHistoryEntry`.
3. **Execution metadata display** — shows Prompt Tokens, Completion Tokens, Total Tokens, Cost (formatted `$X.XXXX`), and Output Words in the bottom-panel dl when values are available.
4. **History item display** — compact summary line: `{tokens} tokens · ${cost} · {words} words` below the model ID for each run history entry.
5. **Error fallback** — `App.svelte` catch block provides `usage` with all-null fields.

### Tests

- **Backend**: Updated the existing `executes_prompt_block_and_persists_run_record` test to include `usage` in mock transport response and assert on extracted metrics. Added two new unit tests: `extracts_usage_metrics_from_response_with_all_fields` and `extracts_usage_metrics_gracefully_when_fields_missing`.
- **Frontend**: Updated all test fixtures with `usage: emptyUsageMetrics`. Existing 23 tests pass with the expanded type shapes.

## Verification

- `cargo test` — 32/32 passed
- `npm run test` — 23/23 passed
- `npm run typecheck` — 0 errors
- `npm run lint` — clean
- Backward compatibility: old v2 run artifacts deserialize without error due to `#[serde(default)]`

## Follow-Up Items

- [ ] Retry count tracking — requires an execution-level retry loop, not just response extraction
- [ ] Project-level usage aggregation — total tokens and cost across all runs
- [ ] Cost estimation for models where OpenRouter does not return `usage.cost`
