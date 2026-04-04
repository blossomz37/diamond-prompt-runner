---
created: 2026-04-03 5:05 PM
modified: 2026-04-03 8:00 PM
---
# Plan To Reach The Rendering And Validation Slice

## Summary

Milestone 1 and the Editing Slice are complete. The active milestone is the Rendering and Validation Slice.

Current status in the app:
- `.tera` prompt tabs validate against the current draft content
- Diamond `doc("...")` references are preprocessed and resolved from `documents/`
- the bottom panel shows validation status, warnings or errors, context summary, and preview for active `.tera` tabs
- markdown, text, `.tera`, and model YAML assets are editable
- `project.json` remains read-only

This slice should finish validation and preview behavior for prompt authoring without pulling execution, API-key storage, or provider settings into scope.

Sequence around this milestone:
- small post-milestone follow-up: unavailable-recents `Locate` flow
- completed milestone: Editing Slice
- active milestone: Rendering and Validation Slice
- after validation: Single-Block Execution Slice
- later: Pipelines, History, and Exports
- later: scoped online-research extension

## Delivered So Far

### Backend validation surface

- Add a Tauri command for template validation from the current draft buffer rather than from disk only.
- Build a Diamond-oriented validation context around:
  - project metadata
  - project variables
  - model id
  - local clock fields
- Preprocess Diamond `doc("...")` references so project documents can be resolved during validation preview.
- Return validation status, warnings, errors, preview, and context summary to the frontend.

### Workspace behavior

- Trigger validation for the active `.tera` tab.
- Validate from the current draft content so unsaved edits are reflected immediately.
- Debounce validation requests and ignore stale responses.
- Show preview and validation state in the bottom panel while keeping non-`.tera` tabs on simple informational empty states.
- Bottom panel is toggleable via a chevron button in the pane header (open by default, aligned with the spec default).
- Bottom panel auto-sizes to fit full preview content rather than clipping at a fixed row height.
- Explorer directory nodes expand and collapse on click; caret icon reflects open/closed state.

## Follow-Up Checklist

### Refinements Deferred Out Of Slice Close

- [x] Add tests for invalid Tera syntax so parser failures stay distinct from render-time failures.
- [x] Add tests for missing context variables and confirm they stay warnings in preview mode.
- [x] Add tests for invalid or traversal-style `doc("...")` references.
- [x] Fix YAML metadata parsing to tolerate malformed model preset files on asset read instead of propagating a hard error.
- [ ] Decide whether validation responses need more structured issue data than plain strings before execution work begins.

- [x] Add tests for validation state reset when switching away from `.tera` tabs.
- [x] Add tests for loading states and validation refresh after draft edits.
- [x] Reconcile the current always-visible bottom-panel presentation with the spec requirement that preview and warning panes be toggleable by default.
- [ ] Confirm whether the bottom panel should preserve the last good preview while a refreshed validation request is in flight.
- [ ] Confirm whether context summary should include referenced documents or warning counts, not only project-level fields.

### Slice Close Notes

- [x] Add manual validation notes for the rendering and validation slice.
- [x] Confirm the authority docs and roadmap stay aligned when the slice is considered complete.
- [x] Keep execution-specific strict blocking semantics out of this slice; document them in the later execution plan instead.

## Slice Sign-Off

Rendering and Validation is complete.

Shipped behavior confirmed at slice close:
- `.tera` tabs validate from the current draft buffer rather than disk-only state
- preview validation resolves Diamond `doc("...")` references from `documents/`
- parser failures, unresolved variables, missing documents, and traversal-style document references are handled without crashing validation
- markdown, text, `.tera`, and model YAML assets are editable while `project.json` remains read-only
- explorer directories expand and collapse in the workspace shell
- the bottom panel is toggleable and grows to fit preview content instead of clipping at a fixed height

Manual validation notes:
- current automated gates remain green for this slice: `npm run typecheck`, `npm run lint`, `npm run test`, and `cargo test --manifest-path src-tauri/Cargo.toml`
- no execution-time strict blocking rules were introduced in this slice; preview validation remains intentionally more permissive than future execution behavior
- remaining unchecked items in this file are refinement decisions for future UX and data-shape work, not blockers for moving into the execution milestone

## Review Notes From Current Audit

- The broad TODO items for validation are now implemented in code and should be treated as landed work.
- Backend test depth is now complete: parser errors, render-time warnings (undefined variables, traversal paths, missing documents), and YAML fault tolerance are all covered.
- Frontend state-transition and loading tests are in place.
- The bottom panel is now toggleable via a chevron button in the pane header, aligned with the spec default.
- The remaining open decisions are minor: structured issue data format, preview persistence during in-flight refresh, and context summary enrichment.

## Test Plan

- Backend:
  - valid `.tera` templates produce preview output
  - invalid `.tera` templates return parser or render errors without crashing validation
  - unresolved `doc("...")` references warn correctly
  - invalid variable references return preview-mode warnings as intended
- Frontend:
  - opening a `.tera` tab shows validation output in the bottom panel
  - editing a `.tera` draft refreshes validation output
  - switching to a non-`.tera` tab clears validation-specific UI
  - stale validation responses do not overwrite newer draft results
- Validation gates:
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- This file is the active detailed plan for the current milestone.
- Execution-time strict mode is still deferred even though preview validation now exists.
- `project.json` remains read-only unless a later plan explicitly promotes manifest editing into scope.