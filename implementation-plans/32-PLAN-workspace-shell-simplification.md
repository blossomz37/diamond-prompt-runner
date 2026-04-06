## Plan: Workspace-First Shell Simplification

Restructure the shell so the sidebar behaves like a file navigator instead of an inline control surface. The recommended approach is to keep project-backed assets as real files, convert app-level Global Variables into a virtual editor tab, remove the bottom panel entirely, and move pipeline execution visibility into the topbar plus existing editor/inspector surfaces. This preserves the repo’s file-first model while matching the requested “everything opens in the workspace” UX.

**Steps**
1. Phase 1, shell inventory and boundaries: formalize which sidebar sections remain true asset lists versus special workflow panels. Treat Prompts, Prompt Blocks, Pipelines, Runs, Documents, Exports, Models, Workspace Variables, and Help as project/workspace navigation surfaces. Treat Global Variables as the only virtual asset because they are app-level state rather than project files. This decision blocks the UI refactor because the shell needs a consistent item model.
2. Phase 2, prompt-block sidebar structure: update the Prompt Blocks section so Registered Templates render first, Available Templates render second, and both sections have explicit labels plus independent collapsed state that defaults closed. This work is isolated to the prompt-block sidebar and can run in parallel with Phase 3.
3. Phase 3, Models as file navigation: replace the current in-sidebar model-management form in `/Users/carlo/diamond-runner/src/lib/components/SidebarModels.svelte` with a file-list-style view sourced from the existing asset tree under `models/`. Selecting a preset should open it in the main workspace editor, and create/delete/default-preset actions should move either into the editor tab, inspector, or a dedicated model-file toolbar rather than living in the sidebar. This blocks any follow-up cleanup in the App shell because callbacks and props will change.
4. Phase 4, Variables as workspace/editor assets: refactor Workspace Variables to open and edit through the main workspace rather than inline sidebar forms. Recommended implementation: surface the existing project variable source as a file-like asset, preferably the existing project-backed variable file if one already exists in the asset tree; otherwise add a workspace-variable virtual tab only if necessary. In the same phase, add a Global Variables virtual tab that opens from the sidebar and edits app-level variables in the editor pane using the same visual/editor pattern as a file. This phase depends on Phase 1 because the project-vs-virtual boundary must be explicit.
5. Phase 5, shared sidebar item model: normalize the sidebar components so Models and Variables follow the same interaction pattern as other sidebar sections: list entries only, no inline form editing, open item into the workspace, and rely on the inspector/editor for detail and mutation. This can be implemented incrementally across `/Users/carlo/diamond-runner/src/lib/components/SidebarModels.svelte`, `/Users/carlo/diamond-runner/src/lib/components/SidebarGlobalVariables.svelte`, `/Users/carlo/diamond-runner/src/lib/components/SidebarWorkspaceVariables.svelte`, and `/Users/carlo/diamond-runner/src/lib/components/WorkspaceShell.svelte`.
6. Phase 6, bottom-panel removal: remove the bottom panel from `/Users/carlo/diamond-runner/src/lib/components/WorkspaceShell.svelte` and delete or retire its remaining responsibilities from `/Users/carlo/diamond-runner/src/lib/components/ValidationPanel.svelte`. Prompt validation should stay in the inspector, prompt preview should stay in the editor pane, and run output/history should move into either the workspace tab area, the inspector, or dedicated run/history views already present in the shell. This phase depends on verifying that no unique information remains exclusive to the bottom panel.
7. Phase 7, topbar execution status: replace the current topbar count badges in `/Users/carlo/diamond-runner/src/lib/components/WorkspaceShell.svelte` with pipeline execution status. Recommended behavior: when a pipeline is active, show live progress using existing `activePipelineProgress` data; when nothing is running, show a compact last-run summary based on `pipelineExecutionResult`, with status and success/failure state clearly visible. This can run in parallel with Phase 6 once the bottom-panel dependency on run state is resolved.
8. Phase 8, App and bridge cleanup: update `/Users/carlo/diamond-runner/src/App.svelte` and the sidebar component props so removed inline-edit flows no longer pass stale callbacks into the shell. Keep backend changes minimal unless the variable/file model requires new read/write commands for virtual tabs or project-backed variable assets.
9. Phase 9, automated coverage updates: adjust `/Users/carlo/diamond-runner/src/App.test.ts` to reflect the new shell model. Prioritize tests for: Prompt Blocks collapsed sections, prompt-template registration still working after the reorder, Models opening in the main workspace, Variables opening as file-like items, bottom-panel removal, and topbar active/last pipeline status. Existing unrelated stale tests should be updated only where directly impacted by this UX refactor.
10. Phase 10, manual validation: verify the shell end to end in the app. Specifically confirm that sidebar sections feel like navigation only, prompt preview/validation remain available after removing the bottom panel, model presets and variables open in the editor correctly, and active pipeline progress remains visible without opening a dedicated run view.

**Relevant files**
- `/Users/carlo/diamond-runner/src/lib/components/WorkspaceShell.svelte` — central shell layout, topbar, sidebar section composition, bottom-panel removal, and prop threading.
- `/Users/carlo/diamond-runner/src/App.svelte` — state orchestration, virtual-tab/file-opening flow, pipeline status state, and callback cleanup.
- `/Users/carlo/diamond-runner/src/lib/components/SidebarPromptBlocks.svelte` — reorder Registered vs Available templates and add local collapsed subsections.
- `/Users/carlo/diamond-runner/src/lib/components/SidebarModels.svelte` — convert from sidebar management UI to file-list navigation behavior.
- `/Users/carlo/diamond-runner/src/lib/components/SidebarGlobalVariables.svelte` — replace inline editing with virtual-file navigation entry behavior.
- `/Users/carlo/diamond-runner/src/lib/components/SidebarWorkspaceVariables.svelte` — replace inline editing with file-like navigation behavior for workspace/project variables.
- `/Users/carlo/diamond-runner/src/lib/components/ValidationPanel.svelte` — retire or repurpose bottom-panel-only execution/history UI.
- `/Users/carlo/diamond-runner/src/lib/components/InspectorPanel.svelte` — absorb any remaining validation/run detail that should live in the inspector after bottom-panel removal.
- `/Users/carlo/diamond-runner/src/lib/components/AssetViewer.svelte` — preserve prompt preview/editor behavior after shell simplification.
- `/Users/carlo/diamond-runner/src/lib/components/SidebarPipelines.svelte` and `/Users/carlo/diamond-runner/src/lib/components/PipelineEditorTab.svelte` — reference current live-progress UI and last-run semantics while moving topbar status.
- `/Users/carlo/diamond-runner/src/lib/types/project.ts` — confirm whether any virtual-asset or topbar status types need extension.
- `/Users/carlo/diamond-runner/src/App.test.ts` — update shell-level behavior tests.

**Verification**
1. Run `npm run typecheck`.
2. Run focused frontend tests covering shell/sidebar behavior, at minimum `src/App.test.ts`, after updating affected assertions.
3. If backend/bridge commands change for variable-file handling, run `cargo test --manifest-path src-tauri/Cargo.toml`.
4. In the app, confirm Prompt Blocks shows Registered Templates above Available Templates and both groups start collapsed.
5. In the app, confirm Models entries open in the main editor and no longer require inline sidebar editing.
6. In the app, confirm Workspace Variables and Global Variables open as file-like editor surfaces rather than inline sidebar forms.
7. In the app, confirm the bottom panel is gone and no validation/preview/run information is lost.
8. In the app, trigger a pipeline run and confirm the topbar shows live progress; after completion, confirm it switches to a last-run success/failure summary.

**Decisions**
- Included scope: Prompt Blocks section restructure, Models/Variables file-like sidebar behavior, bottom-panel removal, and topbar pipeline-status replacement.
- Included scope: Global Variables as a virtual editor tab, because they are app-level state and not a project file.
- Excluded scope: Trash recovery/restore UI.
- Excluded scope: changing the underlying project/app storage model unless required to support the new editor surfaces.
- Excluded scope: the user-edited Ghost Draft prompt files under `/Users/carlo/diamond-runner/Sample Projects/NPE Ghost Draft/prompts/`.
- Recommended implementation bias: prefer reusing the existing asset tree and workspace tab system rather than adding more sidebar-specific forms.

**Further Considerations**
1. Recommended UX rule: only actual navigable items should appear in the sidebar body; create/delete actions should live in the workspace view or inspector for the selected item, not inline in the list.
2. Recommended variable strategy: keep Workspace Variables project-backed and Global Variables virtual, but give them the same editor chrome so they feel identical to authors.
3. Recommended rollout order: ship Prompt Blocks reordering and bottom-panel removal only after Models/Variables file-like navigation is in place, otherwise the shell temporarily loses too many editing affordances.