---
created: 2026-04-03 12:57 PM
modified: 2026-04-06
---

# Rolling TODO List
Oldest to Newest


## Started: 2026-04-03
- [x] ONGOING: Implementation Plans. `implementation-plans/`
- [x] ONGOING: Keep README and AGENTS aligned with the approved tool and commit workflow recommendations.
- [x] Spare parts files from YFD Runner. `workshop-parts/`
- [x] MVP Specifications. `specifications/`
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
- [x] Pipeline Authoring Slice: add an active implementation plan and replace manual `project.json` edits with structured in-app pipeline editing.
- [x] Export Bundle Slice: export selected open tabs into derived bundles under `exports/` and block unsaved editable drafts from being exported.
- [x] Usage Metrics Slice: extract and display prompt tokens, completion tokens, total tokens, cost, and output word count from OpenRouter responses.
- [x] Usage Metrics Slice: persist usage metrics in run artifacts and surface them in execution metadata and run history UI.
- [x] Variable Assignment UI: add global and project variable editing with in-app UI, backend persistence, and template context injection.
- [x] Workspace Shell Restructure: move Pipelines, Variables, Exports to collapsible left sidebar sections; pipeline authoring opens in center pane; strip Inspector to read-only.
- [x] Settings and Model Presets Slice: add project settings UI, automatic model preset generator, and enable inline dynamic overrides for model choices.
- [x] Output Target & JSON Slimdown Slice: implement document artifact extraction loop (`outputTarget: 'document'`); purge payload bloat, extract `variables`, and map `document_path` on JSON records.
- [x] Asset Lifecycle Slice (all 5 phases): pipeline deletion, prompt block manifest removal (preserving .tera), run artifact deletion, document rename/delete from ExplorerTree, and pipeline export via UI.


## Post-MVP
- [x] Post-MVP online extension: add the active implementation plan and require the standalone OpenRouter probe to pass before app behavior is hardcoded.
- [x] Post-MVP online extension: support scoped `:online` behavior for prompts that require live web research.
- [x] Post-MVP online extension: add tests for scoped online-research prompt behavior.
- [x] Post-MVP online extension: add Help-section notes for online-research prompt behavior, constraints, and setup.
- [x] Post-MVP online extension: decide whether `:online` support belongs in the MVP spec or as a later extension. Decision: post-MVP extension, documented in SPEC_DIAMOND_RUNNER_v1.md §3 Post-MVP Extensions.
- [x] Code Audit (Plan 23): extract `types.rs`, `execution.rs`, `credentials.rs`, `history.rs`, `presets.rs`, `variables.rs`, `assets.rs` from `project_store.rs`; extract `assetUtils.ts`, `textSearch.ts`, `validation.svelte.ts`, `FindBar.svelte` on frontend.
- [x] UX Overhaul Phase 1 (Plan 22): sidebar reorganized to 10 sections (Models, Prompts, Blocks, Pipelines, Runs, Documents, Exports, Settings, Help); new sidebar components created.
- [x] CSS Audit (Plan 24): documented duplication families across `.eyebrow`, `.empty`, `.mini-action`, `.primary` button gradients, and variable panel triplet.
- [x] CodeMirror 6 (Plan 26): `CodeEditor.svelte` with syntax highlighting for `.tera` and markdown, custom Tera language support (`teraLanguage.ts`), FindBar adapter pattern.
- [x] Tera Custom Filters (Plan 27 Phase 1): `extract_section` filter for slicing document content by start/end markers.
- [x] Live Events & Resume (Plan 28): `PipelineProgressEvent` emission, `resume_from_block_id` skip logic, continue-run UI in `PipelineEditorTab`.
- [ ] UX Overhaul Phases 2–4 (Plan 22): block-level pipeline overrides, progress bar polish, help content population.
- [ ] Batch Execution (Plan 27 Phases 2–3): batch orchestration backend and batch execution UI.
- [ ] CSS consolidation: extract shared patterns (`.eyebrow`, `.empty`, `.mini-action`, `.primary`) to `app.css` per Plan 24 findings.
- [ ] Code Audit deferred: extract App.svelte tab state to `lib/stores/tabs.svelte.ts` (requires WorkspaceShell prop audit first).
- [ ] Code Audit deferred: evaluate WorkspaceShell.svelte split after App.svelte tab state is resolved.
- [ ] Code Audit deferred: evaluate PipelineEditorTab.svelte split if a shared two-step confirmation pattern is formalized.
- [ ] Code Audit deferred: evaluate ProjectBrowser.svelte split (`CredentialsPanel`, `RecentProjectsList`) after App.svelte credential state is reviewed.
- [ ] Duplicate Project and Pipelines (Plan 29).
- [ ] Decide whether to create custom Diamond-specific skills for implementation, fixtures, and prompt-pack porting.


## Release
- [ ] Generate app icons: `npm run tauri icon path/to/source.png` → commit `src-tauri/icons/` → update `"icon"` array in `tauri.conf.json`.
- [ ] Set repository secrets for installer signing: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (optional for dev-only releases).
- [ ] Sync version number across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` before tagging.
- [ ] Cut first release: bump version → commit → `git tag v0.x.0 && git push origin v0.x.0` → review draft release → publish.

## Other
- [ ] Test GitHub app tools directly for repo workflows in this workspace.


