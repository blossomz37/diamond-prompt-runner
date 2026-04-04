---
created: 2026-04-03 7:15 PM
modified: 2026-04-03 7:15 PM
---
# Plan To Reach The Pipeline Runner Slice

## Summary

Single-Block Execution is complete. The next active milestone is the Pipeline Runner Slice.

This slice should prove the end-to-end path for one manifest-defined linear pipeline:
- list pipelines already defined in `project.json`
- run a sequential pipeline from the existing shell
- execute each block from the saved on-disk prompt files
- stop on the first execution failure
- reuse the existing per-block run artifact contract under `runs/`
- surface the latest pipeline status in the current workspace shell

This slice does not include pipeline editing, pipeline cancellation, workspace-wide run-history screens, or export tooling.

Sequence around this milestone:
- completed small follow-up: unavailable-recents `Locate` flow
- completed milestone: Single-Block Execution Slice
- active milestone: Pipeline Runner Slice
- later: Run History / Output Inspector expansion
- later: Exports and import/export workflows
- later: scoped online-research extension

## Delivered So Far

### Backend pipeline execution

- Add `list_project_pipelines` so the frontend can inspect manifest-defined pipelines without parsing `project.json` in the browser layer.
- Add `execute_pipeline` for sequential manifest pipeline runs.
- Reuse the existing block execution path and OpenRouter transport contract for each pipeline step.
- Stop the pipeline on the first failing block and return the completed steps plus the failure summary.
- Keep per-block persisted run artifacts under `runs/` as the first saved history surface for pipeline work.

### Frontend pipeline runner UI

- Surface manifest pipelines in the inspector rather than creating a separate pipeline screen.
- Add one `Run pipeline` action per pipeline from the existing shell.
- Show the latest pipeline status and completed-step count in the inspector.
- Guard against running pipelines from stale saved files when related `.tera` tabs have unsaved drafts.
- Add a small explorer-side `New Prompt` flow so prompt authoring does not require manual file creation outside the app.

## Scope

Included in this slice:
- manifest-driven pipeline listing
- sequential pipeline execution
- saved-file execution semantics for pipeline runs
- inspector-level run controls and latest pipeline status
- dedicated prompt creation from the explorer that creates the `.tera` file and matching prompt-block manifest entry
- backend and frontend tests for pipeline success and stop-on-failure behavior

Explicitly out of scope for this slice:
- pipeline editing UI
- pipeline cancellation
- pipeline-level persisted summary artifacts
- workspace-wide run-history browsing beyond the current prompt-scoped history and latest inspector status
- exports and import/export tooling

## Follow-Up Checklist

- [x] Add manifest pipeline listing to the Tauri bridge.
- [x] Execute linear pipelines sequentially from saved prompt files.
- [x] Stop further execution after the first failing block.
- [x] Surface pipeline run controls and latest pipeline status in the existing shell.
- [x] Guard against pipeline runs when related prompt tabs have unsaved drafts.
- [x] Add an in-app prompt creation flow so new prompt blocks no longer require manual filesystem edits.
- [ ] Expand run-history UX beyond prompt-scoped bottom-panel browsing.
- [ ] Add export-oriented slices after the pipeline runner behavior settles.

## Verification

- Pipeline runs must reuse the existing strict execution rules from Single-Block Execution.
- Pipeline runs must execute blocks in listed order and stop on the first failure.
- Pipeline runs must continue to pass:
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- Saved prompt files on disk are the source of truth for pipeline runs in this slice.
- The existing per-block run artifact contract remains the persisted history unit for the first pipeline slice.