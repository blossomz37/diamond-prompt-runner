# 22-PLAN-ux-overhaul-sidebar-streaming-help

## Goal
Reorganize the workspace shell to follow the author's natural workflow, add pipeline-level block overrides, deliver execution progress feedback via a progress bar, and ship an in-app help section. This plan captures four interrelated UX improvements surfaced in [UX-Eval.md](../specifications/UX-Eval.md).

## Rationale
The current sidebar order (Explorer → Pipelines → Variables → Exports → Settings) is backwards relative to the actual workflow: set up models → define variables → write prompts → configure blocks → assemble pipelines → run → review → export. The pipeline data model is also too flat — a block can only be used once with its default settings, and there's no per-step override when the same template needs different models or output targets. Execution gives no progress feedback during long runs. And there's no discoverable help for Tera syntax, online search, or pipeline assembly.

## Authority reference
- sidebar discussion: `specifications/UX-Eval.md`
- product spec: `specifications/SPEC_DIAMOND_RUNNER_v1.md`
- current data model: `src/lib/types/project.ts`, `src-tauri/src/project_store.rs`
- current sidebar: `src/lib/components/WorkspaceShell.svelte`

---

## Phase 1 — Sidebar Reorganization (structural, no new features)

**Goal:** Reorder and rename sidebar sections to mirror the authoring workflow.

New section order, top to bottom:

| # | Section | Content | Notes |
|---|---------|---------|-------|
| 1 | **Models** | List of model presets, create/edit/delete, set project default | Extracted from current Settings |
| 2 | **Variables** | Global + project variable editing | Existing `SidebarVariables` |
| 3 | **Prompts** | `.tera` template file browser + New Prompt action | Extracted from current Explorer |
| 4 | **Prompt Blocks** | Registered blocks list + create/edit per block | Extracted from current Explorer bottom zone |
| 5 | **Pipelines** | Pipeline list + create/edit/run/delete | Existing `SidebarPipelines` + run button |
| 6 | **Runs** | Project-wide run history with pipeline filter | Moved from right Inspector panel |
| 7 | **Documents** | Document file browser (markdown, text) | Remaining Explorer content |
| 8 | **Exports** | Bundle export + Reveal in Finder + Copy Path | Existing `SidebarExports` plus new local-path actions |
| 9 | **Settings** | Project name, API key status, UI prefs | Slimmed-down Settings |
| 10 | **Help** | In-app reference docs (opens as center-pane tab) | New — see Phase 4 |

### Work items
- [ ] Create `SidebarModels.svelte`: extract model preset list and create/delete/edit from `SidebarSettings`
- [ ] Create `SidebarPrompts.svelte`: list `.tera` files from project, include the New Prompt form (moved from Explorer)
- [ ] Create `SidebarPromptBlocks.svelte`: registered blocks list with inline create/edit (name, template picker, model preset, output target)
- [ ] Create `SidebarRuns.svelte`: project-wide run history (extracted from `InspectorPanel`)
- [ ] Create `SidebarDocuments.svelte`: document-specific subtree of the explorer
- [ ] Create `SidebarHelp.svelte`: list of help topics that open in center pane
- [ ] Update `WorkspaceShell.svelte`: reorder sections, update all toggle states and default-open/closed
- [ ] Slim down `SidebarSettings.svelte`: remove model preset management (now in Models), keep project name + API key status
- [ ] Slim down `InspectorPanel.svelte`: remove run history (now in sidebar), keep File Metadata + Project Summary + Usage
- [ ] Slim down `ExplorerTree` / Explorer section: scope to the full project tree or remove it entirely if Prompts + Documents sections replace it

### Checkpoint 1
- [ ] All sidebar sections render in the new order
- [ ] Existing functionality is preserved (no regressions)
- [ ] `npm run typecheck` and `npm run lint` pass
- **Stop, test manually, commit**

---

## Phase 2 — Prompt Blocks Sidebar & Pipeline-Level Overrides

**Goal:** Prompt blocks become a first-class sidebar section with full CRUD. Pipelines can override model preset, output target, and step-level variables per block reference.

### 2a — Prompt Block Sidebar (frontend)
- [ ] `SidebarPromptBlocks.svelte` supports: create block (name + template picker + model preset + output target), inline edit, delete (existing command)
- [ ] Clicking a block opens its `.tera` template in center pane for editing
- [ ] Block list shows template name, assigned model, and output target at a glance

### 2b — Pipeline-Level Block Overrides (data model)

**Rust manifest change:** `Pipeline.ordered_blocks` changes from `Vec<String>` to `Vec<PipelineStep>`:

```rust
pub struct PipelineStep {
    pub block_id: String,
    pub model_preset: Option<String>,    // override block default
    pub output_target: Option<String>,   // override block default
    pub variables: Option<HashMap<String, String>>,  // step-level bindings
}
```

**Frontend type change:** `ProjectPipeline.blocks` changes from `ProjectPipelineBlock[]` to `ProjectPipelineStep[]`:

```typescript
export interface ProjectPipelineStep {
  blockId: string;
  modelPreset?: string | null;   // null = use block default
  outputTarget?: string | null;
  variables?: Record<string, string>;
}
```

- [ ] Add `PipelineStep` struct to `project_store.rs`
- [ ] Migrate `Pipeline` from `ordered_blocks: Vec<String>` to `steps: Vec<PipelineStep>`
- [ ] Add backward-compatible deserialization: if an existing `ordered_blocks` is `Vec<String>`, auto-convert to `Vec<PipelineStep>` with all overrides as `None`
- [ ] Update `execute_pipeline_with_transport` to merge step overrides with block defaults at runtime
- [ ] Update `ProjectPipelineStep` type in `project.ts`
- [ ] Update `PipelineEditorTab.svelte` to show optional override fields per step
- [ ] Update `SidebarPipelines.svelte` to display resolved model + output per step
- [ ] Update `fixtures/sample-project/project.json` to use the new format

### Checkpoint 2
- [ ] A pipeline can reference the same block ID twice with different model overrides
- [ ] Existing pipelines with no overrides still load and run correctly
- [ ] `cargo test` and `npm run test` pass
- **Stop, test manually, commit**

---

## Phase 3 — Execution Progress Bar

**Goal:** Replace the blind spinner with a deterministic progress bar during pipeline execution. For single-block runs, show an indeterminate bar. For pipelines, show `step N of M` with a percentage.

### Approach
Use Tauri's event system to emit progress events from Rust during pipeline execution. This does **not** require switching to streaming SSE from OpenRouter — it tracks which block in the pipeline is currently executing.

### 3a — Backend progress events (Rust)
- [ ] Define a `PipelineProgress` payload: `{ pipeline_id, current_step, total_steps, current_block_name, status: "running" | "completed" | "failed" }`
- [ ] In `execute_pipeline_with_transport`, emit a `pipeline:progress` Tauri event before each block starts and after each block completes
- [ ] For single-block execution, emit `block:progress` with `{ status: "running" }` at start and `{ status: "completed" | "failed" }` at end

### 3b — Frontend progress UI (Svelte)
- [ ] Create `ProgressBar.svelte`: determinate mode (percentage + label) and indeterminate mode (animated bar)
- [ ] Listen for `pipeline:progress` events in `WorkspaceShell.svelte` (via Tauri event listener)
- [ ] Display the progress bar in the bottom panel during active runs
- [ ] Show "Running block 2 of 5: Edit Style" label alongside the bar
- [ ] Clear the progress bar when execution completes or fails
- [ ] For single-block runs, show indeterminate bar with block name

### Checkpoint 3
- [ ] Pipeline run shows progress bar advancing through steps
- [ ] Single-block run shows indeterminate bar
- [ ] Progress bar disappears on completion/failure
- [ ] No regressions to execution results or run artifacts
- **Stop, test manually, commit**

---

## Phase 4 — Help Section

**Goal:** Ship a discoverable in-app help reference so users can learn Tera syntax, online search, pipeline assembly, and output targets without leaving the workspace.

### Work items
- [ ] Create `src/lib/help/` directory with static help content as TypeScript string constants (no separate file I/O needed)
- [ ] Topics to cover:
  - **Tera Template Basics** — variables `{{ var }}`, conditionals `{% if %}`, the `doc("file.md")` function, filters
  - **Online Research** — how `{# diamond:online #}` works, what models support it, constraints
  - **Prompt Blocks** — what they are, how to configure model and output target
  - **Pipelines** — how to assemble, per-step overrides, execution behavior
  - **Variables** — global vs project, how they appear in templates
  - **Output Targets** — `run_artifact` vs `document`, what each produces
  - **Keyboard Shortcuts** (if any exist)
- [ ] `SidebarHelp.svelte` lists topics; clicking one opens a read-only help tab in center pane
- [ ] Help tabs use the existing `text` view mode in `AssetViewer` (or a simple rendered-markdown viewer)

### Checkpoint 4
- [ ] Every help topic opens and displays correctly in center pane
- [ ] Help section is visible and accessible in the sidebar
- [ ] No build or type errors
- **Stop, test manually, commit**

---

## Phase 5 — Local File Actions & Polish

**Goal:** Add Reveal in Finder and Copy Path to sidebar items. Polish any rough edges from the reorg.

### Work items
- [ ] Add `reveal_in_finder(path)` Tauri command (Rust: `open::that` or `Command::new("open")` on macOS)
- [ ] Add `copy_to_clipboard(text)` Tauri command (or use the Tauri clipboard plugin)
- [ ] Add "Reveal in Finder" and "Copy Path" context actions to: Documents, Prompts, Runs, Exports
- [ ] Remove or repurpose the right Inspector panel if all its content has moved to the sidebar and center pane
- [ ] Verify all tab opening still works from the new sidebar sections
- [ ] Update `AGENTS.md` architecture section to reflect new sidebar layout
- [ ] Update `README.md` with current feature list
- [ ] Update `specifications/SPEC_DIAMOND_RUNNER_v1.md` if the pipeline data model change is spec-impacting

### Checkpoint 5
- [ ] Reveal in Finder opens the correct folder/file on macOS
- [ ] Copy Path puts the correct path on the clipboard
- [ ] All authority docs are updated
- [ ] Full validation: `npm run typecheck`, `npm run lint`, `npm run test`, `cargo test`
- **Stop, final commit, push to GitHub**

---

## Out of Scope (future plans)
- **Token streaming** — real-time display of LLM output as it arrives (requires async SSE reader + Tauri event stream). Tracked separately.
- **Loop/logic conditions on prompt blocks** — conditional execution, iteration, branching within pipelines. Design phase needed before implementation.
- **Graph execution mode** — non-linear pipeline topologies.
- **Prompt pack import/export** — sharing workflow templates between projects.

## Success Criteria
- [ ] Sidebar sections follow the authoring workflow order
- [ ] Prompts and Prompt Blocks are separate, clearly distinct sections
- [ ] A pipeline can use the same prompt block twice with different model/output overrides
- [ ] Pipeline execution shows a progress bar with step-level feedback
- [ ] In-app help is accessible from the sidebar
- [ ] Reveal in Finder and Copy Path work on all relevant assets
- [ ] All authority docs reflect the new layout and data model
