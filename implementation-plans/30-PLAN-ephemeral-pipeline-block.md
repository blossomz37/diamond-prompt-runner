## Plan: Ephemeral Pipeline Block Selection

Add a per-run block-selection mode for pipelines so the author can run only part of a pipeline without duplicating or mutating the saved pipeline definition. The recommended approach is runtime-only: keep pipeline JSON unchanged, add ephemeral selection state in the pipeline editor, and pass an optional selected block list through the Tauri bridge to Rust so execution, progress counts, resume behavior, and run artifacts stay consistent.

**Steps**
1. Backend execution contract: extend the pipeline execution call chain to accept an optional selected block ID list. This means updating the Tauri command in `src-tauri/src/main.rs`, the public execution function in `src-tauri/src/project_store/execution.rs`, and the transport-backed helper in the same file. This step blocks the frontend wiring because the UI needs a stable runtime contract.
2. Backend filtering behavior: in `src-tauri/src/project_store/execution.rs`, derive an ordered `blocks_to_run` list from the saved pipeline’s `ordered_blocks`, filtered by the selected IDs while preserving canonical pipeline order. Validate that the selection is not empty and that selected IDs belong to the pipeline. Use this filtered list for execution, progress events, and resume indexing.
3. Resume semantics: define and implement the interaction between `resume_from_block_id` and selected blocks. Recommended rule: the resume block must be present in the selected subset; otherwise return a clear error. This keeps behavior explicit and avoids silent mismatches.
4. Bridge update: extend `src/lib/tauri.ts` so `executePipeline` accepts `selectedBlockIds?: string[]` and forwards them to the Tauri command. This can run in parallel with step 2 once the backend contract is decided.
5. Pipeline editor UI: update `src/lib/components/PipelineEditorTab.svelte` to add an ephemeral run mode for existing pipelines. Recommended UX: default “Run all” behavior plus an opt-in “Run selected” mode that reveals checkboxes beside the read-only step list. All blocks should start selected when entering this mode.
6. Run action wiring: update `src/lib/components/WorkspaceShell.svelte` and `src/App.svelte` so the run callback can carry selected block IDs from the editor to `handleRunPipeline`, and from there to the Tauri bridge. Keep selection state local to the editor and reset it when switching pipelines.
7. Guardrails in the UI: in `src/lib/components/PipelineEditorTab.svelte`, disable the run action when no blocks are selected, surface a concise message when resume is incompatible with the current selection, and keep existing dirty-prompt guards in `src/App.svelte` unchanged so subset runs still require saved `.tera` files.
8. Backend tests: add Rust tests beside the existing project store tests for subset execution, ordered filtering, empty selection rejection, invalid block rejection, and resume-with-subset behavior. These depend on steps 1–3.
9. Frontend verification: add or extend the relevant UI test coverage in `src/App.test.ts` only if there is already a practical mocking path for the pipeline editor flow; otherwise rely on focused manual verification for the first slice and keep automated coverage backend-heavy.
10. Manual validation: verify in the sample projects that a full run still behaves identically, that a subset run skips deselected editorial steps, that progress counts reflect only selected steps, and that “Continue Pipeline” behaves correctly when used with a selected subset.

**Relevant files**
- `src-tauri/src/main.rs` — extend the `execute_pipeline` Tauri command signature and forward selected block IDs.
- `src-tauri/src/project_store/execution.rs` — filter `ordered_blocks` into a runtime subset, validate selection, and keep progress/resume logic coherent.
- `src/lib/tauri.ts` — extend the `executePipeline` bridge wrapper.
- `src/App.svelte` — thread optional selected block IDs through `handleRunPipeline` while preserving existing dirty-file checks and run history refresh.
- `src/lib/components/WorkspaceShell.svelte` — pass the enriched run callback into `PipelineEditorTab`.
- `src/lib/components/PipelineEditorTab.svelte` — add the per-run selection UI, local selection state, and run-mode behavior.
- `src/App.test.ts` — optional UI-level coverage if the editor flow is practical to mock.

**Verification**
1. Run `cargo test --manifest-path src-tauri/Cargo.toml` and confirm new subset-execution tests pass.
2. Run `npm run typecheck`.
3. Run focused ESLint on the changed frontend files if repository-wide lint remains noisy.
4. In the app, open a pipeline such as “Full Chapter Production,” switch to the subset mode, deselect one or more editorial blocks, and confirm only selected steps execute.
5. Trigger a failure in a selected subset, then use “Continue Pipeline” and verify resume only works when the failed block remains selected.
6. Confirm a normal “Run Pipeline” with no subset mode change still executes all steps and preserves current behavior.

**Decisions**
- Included scope: ephemeral, per-run block selection for pipelines.
- Excluded scope: persisted muted/skipped block defaults in pipeline JSON.
- Excluded scope: portable pipeline packaging/export-import.
- Recommended contract: backend-enforced subset filtering, not frontend-only emulation, so progress, resume behavior, and run artifacts stay truthful.
- Assumption: “the easy one” means the runtime-only subset toggle, not schema-backed mute defaults.

**Further Considerations**
1. Recommended UX label: use “Run all” and “Run selected” instead of “mute,” because the action is execution-scoped and clearer for authors.
2. Recommended future follow-up: if this proves useful, the deferred version should add persisted defaults per pipeline rather than creating many near-duplicate pipelines.
3. Recommended run-artifact follow-up: capture the selected subset in pipeline execution metadata later if auditability becomes important.