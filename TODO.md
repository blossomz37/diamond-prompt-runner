---
created: 2026-04-03 12:57 PM
modified: 2026-04-04 12:00 AM
---

# Rolling TODO List
Oldest to Newest

## 2026-04-03
- [ ] ONGOING: Implementation Plans. `implementation-plans/`
- [x] Spare parts files from YFD Runner. `workshop-parts/`
- [ ] MVP Specifications. `specifications/`
- [x] Figma Prototype. Selected `Diamond Runner v2 - IDE Pass`.
- [x] Prototype direction approved. IDE shell is the selected UI model.
- [x] Create a user-facing readme.
- [x] Create a gitignore. Ignore system files for macOS, Windows, and Linux.
- [x] Milestone 1 implementation: scaffold Tauri + Svelte + Vite app in-repo.
- [x] Milestone 1 implementation: build project create/open flows and project validation.
- [x] Milestone 1 implementation: build read-only IDE shell with explorer, tabs, inspector, and bottom panel placeholder.
- [x] Milestone 1 implementation: add `fixtures/sample-project/` and milestone smoke tests.
- [x] Milestone 1 manual validation: completed all scenarios in `implementation-plans/06-REFERENCE-milestone-1-manual-test.md` and all passed.
- [x] Follow-up after Milestone 1: add a `Locate` flow for unavailable recent projects that rebinds a moved project only when the selected folder validates and matches the stored `project_id`.
- [x] Editing Slice: add save support for editable project files after shell and navigation stability.
- [x] Editing Slice: support markdown and text document editing with visible dirty state and deterministic save or reload behavior.
- [x] Editing Slice: support `.tera` template editing and editable model YAML while keeping `project.json` read-only.
- [x] Rendering and Validation Slice: port Tauri-side Tera template validation patterns into Diamond prompt editing.
- [x] Rendering and Validation Slice: show template preview, warnings, and validation state in the bottom panel for `.tera` tabs.
- [x] Rendering and Validation Slice: add backend tests for invalid Tera syntax, missing context variables, and invalid `doc("...")` reference paths.
- [x] Rendering and Validation Slice: add frontend tests for tab switching, loading and reset states, and validation refresh from draft edits.
- [x] Rendering and Validation Slice: fix YAML model preset read fault-tolerance and make bottom panel toggleable per spec.
- [x] Fix: explorer directory nodes now expand/collapse on click; carets update state correctly.
- [x] Fix: bottom validation panel is now contained within its grid row; summary and preview columns are both scrollable within bounds.
- [x] Fix: bottom panel preview now expands to accommodate full preview text instead of clipping at a fixed row height.
- [x] Rendering and Validation Slice: add a slice sign-off checklist and manual validation notes.
- [x] Single-Block Execution Slice: create active implementation plan and lock first-pass execution scope.
- [x] Single-Block Execution Slice: add backend-only execution foundation with strict render, direct OpenRouter HTTP, and persisted run artifacts.
- [x] Single-Block Execution Slice: wire frontend run controls and bottom-panel execution output for active `.tera` tabs.
- [x] Single-Block Execution Slice: add app-level OpenRouter key storage via native keychain with env-var fallback.
- [x] Single-Block Execution Slice: keep direct Rust HTTP for the first full execution slice instead of switching to the SDK mid-slice.
- [x] Single-Block Execution Slice: add prompt-scoped browsing for persisted run artifacts and open the run JSON in a tab.
- [x] Single-Block Execution Slice: lock strict execution failures for unresolved variables and invalid or missing `doc("...")` references.
- [x] Single-Block Execution Slice: lock the typed on-disk run artifact contract under `runs/` and roll it into the umbrella roadmap.
- [x] Single-Block Execution Slice: seed curated model preset defaults from the workshop routing set while keeping `aion-2.0` out of the default project starter pack.
- [x] Pipeline Runner Slice: create the active implementation plan and add manifest-driven sequential pipeline runs to the inspector.
- [x] Pipeline Runner Slice: stop pipeline execution on the first failing block while reusing the existing per-block run artifact contract under `runs/`.
- [x] Prompt Authoring: add an explorer-side create flow for new `.tera` prompt blocks and manifest entries.
- [x] Run History Slice: expose project-wide persisted run history in the inspector while keeping prompt-scoped bottom-panel history in place.
- [x] Run History Slice: retain pipeline metadata on persisted block runs so history can filter by pipeline.
- [ ] Post-MVP online extension: support scoped `:online` behavior for prompts that require live web research.
- [ ] Post-MVP online extension: add tests for scoped online-research prompt behavior.
- [ ] Post-MVP online extension: add Help-section notes for online-research prompt behavior, constraints, and setup.
- [ ] Post-MVP online extension: decide whether `:online` support belongs in the MVP spec or as a later extension.
- [ ] Test GitHub app tools directly for repo workflows in this workspace.
- [ ] Decide whether to create custom Diamond-specific skills for implementation, fixtures, and prompt-pack porting.
- [ ] Keep README and AGENTS aligned with the approved tool and commit workflow recommendations.
