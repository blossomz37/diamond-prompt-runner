---
created: 2026-04-03 11:59 PM
modified: 2026-04-04 12:20 AM
---
# Plan To Reach The Pipeline Authoring Slice

## Summary

Pipeline execution, prompt creation, and project-wide run history are now in place. The next product gap is authoring and editing pipelines from inside the app rather than by hand-editing `project.json`.

This slice should complete the first in-app pipeline workflow:
- create a new linear pipeline from the current shell
- edit pipeline name and ordered block membership
- reorder and remove blocks within an existing pipeline
- validate pipeline references before saving
- keep `project.json` read-only as a raw file while exposing a structured pipeline editor in the UI

This slice does not include graph execution, pipeline cancellation, pipeline-level summary artifacts, or branch logic.

Sequence around this milestone:
- completed small follow-up: unavailable-recents `Locate` flow
- completed milestone: Single-Block Execution Slice
- completed milestone: Pipeline Runner Slice
- completed follow-up: explorer-side prompt creation
- completed milestone: Run History Slice
- completed milestone: Pipeline Authoring Slice
- later: Exports and import/export workflows
- later: scoped online-research extension

## Delivered So Far

### Product state entering this slice

- Prompt blocks can already be created from the explorer and saved as `.tera` files plus manifest entries.
- Manifest-defined pipelines can already be listed and run sequentially from the inspector.
- Project-wide run history can already show persisted artifacts and filter them by block or pipeline.
- The remaining gap is that pipelines still require manual manifest edits outside the app.

### Goal for this slice

- Add the first structured pipeline editor without introducing a separate settings screen or raw manifest editing requirement.
- Reuse the existing shell and inspector patterns where practical.
- Keep filesystem writes and manifest validation in Rust, with the frontend acting as the orchestration layer.

## Scope

Included in this slice:
- create a pipeline from the app
- rename a pipeline
- add and remove blocks from a pipeline
- reorder blocks in a linear pipeline
- save pipeline edits back into `project.json` through the backend
- validate missing or duplicate block references before persisting
- refresh inspector pipeline listings after edits
- backend and frontend tests for pipeline authoring behavior

Explicitly out of scope for this slice:
- graph or branching pipelines
- pipeline cancellation controls
- pipeline-level output artifacts separate from per-block run records
- direct raw `project.json` editing in the shell
- export tooling

## Candidate UX Direction

- Keep the pipeline list in the inspector.
- Add one structured authoring mode in the inspector or an adjacent shell panel rather than creating a new route.
- Allow choosing from existing prompt blocks already defined in the manifest.
- Use simple move-up and move-down ordering controls for the first slice instead of drag-and-drop.
- Keep the existing `Run pipeline` affordance intact after authoring changes.

## Follow-Up Checklist

- [x] Add backend commands for creating and updating pipeline definitions.
- [x] Validate ordered block references against current prompt blocks before saving.
- [x] Add inspector-level UI for pipeline create and edit flows.
- [x] Support ordered block add, remove, and reorder actions.
- [x] Refresh pipeline listings and related shell state after pipeline edits.
- [x] Add backend and frontend coverage for pipeline authoring.
- [x] Update README, roadmap docs, and TODO after the slice lands.

## Verification

- Pipeline authoring must preserve Diamond's file-first model by writing through `project.json` in Rust.
- The app must not require manual manifest edits to create a basic linear pipeline.
- Newly edited pipelines must remain runnable through the existing sequential pipeline execution path.
- This slice must pass:
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- `project.json` stays read-only as a raw file even if structured pipeline editing is added elsewhere in the shell.
- The first authoring pass targets linear pipelines only and defers any drag-and-drop or graph UI.