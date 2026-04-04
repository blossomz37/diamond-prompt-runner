---
created: 2026-04-03 11:50 PM
modified: 2026-04-04 12:00 AM
---
# Plan To Reach The Run History Slice

## Summary

Pipeline Runner and prompt creation are complete. The next milestone after those slices is broader run-history browsing across the workspace.

This slice should prove the first project-wide history view without inventing a separate history screen:
- persist enough metadata on each run artifact to tell whether it came from a standalone block run or a pipeline step
- reuse the existing inspector as the project-level history surface
- keep the bottom panel focused on prompt-scoped validation, latest execution state, and prompt-scoped history
- allow the user to filter persisted runs by block or by pipeline from the current shell
- keep persisted run artifacts under `runs/` as the source of truth for history browsing

This slice does not include pipeline summary artifacts, history deletion, full-text history search, or export tooling.

Sequence around this milestone:
- completed small follow-up: unavailable-recents `Locate` flow
- completed milestone: Single-Block Execution Slice
- completed milestone: Pipeline Runner Slice
- completed follow-up: explorer-side prompt creation
- active milestone: Run History Slice
- later: Exports and import/export workflows
- later: scoped online-research extension

## Delivered So Far

### Backend history contract

- Extend persisted run records with optional pipeline metadata so block runs created by pipeline execution can be grouped later.
- Add a project-wide run-history reader that scans `runs/` and returns the normalized history list sorted by completion time.
- Keep the existing per-prompt history reader intact by reusing the shared run-history parsing path.
- Continue treating the per-block run artifact as the persisted unit rather than inventing a parallel history store.

### Frontend history browsing

- Keep prompt-scoped history in the bottom panel for active `.tera` tabs.
- Add a project-wide run-history section to the inspector instead of creating a new history route or modal.
- Allow filtering persisted runs by block or pipeline from the inspector.
- Let users open any persisted JSON run artifact directly from the inspector history list.

## Scope

Included in this slice:
- optional pipeline metadata on persisted run artifacts
- project-wide run-history listing from `runs/`
- inspector-based browsing of persisted runs
- block and pipeline filters in the inspector history section
- backend and frontend tests for project-wide history metadata and filtering behavior

Explicitly out of scope for this slice:
- deleting or mutating run artifacts from the UI
- dedicated history screens outside the current shell
- pipeline summary artifacts separate from per-block run records
- export tooling
- full-text history search or analytics

## Follow-Up Checklist

- [x] Extend persisted run artifacts with optional pipeline metadata.
- [x] Add a project-wide run-history command on the Tauri bridge.
- [x] Reuse the inspector as the project-level persisted history surface.
- [x] Add block and pipeline filters for project-wide history.
- [x] Keep prompt-scoped bottom-panel history intact.
- [x] Add backend and frontend coverage for the new history path.
- [ ] Add export-oriented slices after the run-history behavior settles.

## Verification

- Project-wide history must remain derived from persisted run artifacts under `runs/`.
- Pipeline-originated block runs must retain enough metadata to filter by pipeline later.
- Prompt-scoped bottom-panel history must remain available for the active `.tera` tab.
- This slice must pass:
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- The inspector is the correct place for broader project-level history in the current IDE shell.
- Persisted per-block run artifacts remain the history source of truth, including during pipeline execution.