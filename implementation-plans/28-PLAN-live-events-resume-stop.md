# Pipeline Live Events & Resume functionality

Currently, `diamond-runner`'s execution engine operates as a synchronous loop without sending progress indicators to the UI. If a large pipeline takes 15 minutes, the frontend blocks with a single static "Running..." view.

Furthermore, if a pipeline crashes midway (e.g., due to an OpenRouter timeout, or a syntax compilation error on Block 3), hitting "Run Pipeline" will start over from Block 1, generating redundant files and wasting API credits.

## Proposed Changes

### 1. `src-tauri/src/project_store/execution.rs` (Backend Engine)
We will refactor the inner loop of `execute_pipeline` to accept an optional anonymous closure: `&mut dyn FnMut(PipelineProgressEvent)`.

The `PipelineProgressEvent` struct will look like:
```rust
pub struct PipelineProgressEvent {
    pub pipeline_id: String,
    pub total_blocks: usize,
    pub completed_blocks: usize,
    pub current_block_name: String,
    pub status: String,
}
```

Since the engine decoupled from Tauri itself (for headless CLI support), we will inject the logic for the Tauri AppHandle event broadcast in `main.rs`. 

#### [MODIFY] `src-tauri/src/project_store/execution.rs`
- Introduce `PipelineProgressEvent`.
- Update `execute_pipeline` parameter signature to handle `resume_from_block_id: Option<String>` and `mut on_progress: Option<&mut dyn FnMut(PipelineProgressEvent)>`.
- Use `resume_from_block_id` to iterate the blocks starting from the index of the matching ID.
- Emit a progress event before dialing the OpenRouter API.

#### [MODIFY] `src-tauri/src/main.rs`
- Update `execute_pipeline` Tauri command to accept `resume_from_block_id: Option<String>`.
- Hook into `on_progress`:
  ```rust
  let mut callback = |progress: project_store::PipelineProgressEvent| {
      let _ = app.emit("pipeline-progress", progress);
  };
  ```
- Make a similar update for `run-pipeline` in the headless CLI match block, substituting the Tauri `app.emit` with `println!` bounds so the user natively gets logs streaming to standard output.

---

### 2. Svelte Frontend Architecture
We'll update `App.svelte` and `WorkspaceShell.svelte` to listen to the `pipeline-progress` event out of Tauri and push the active status into the `PipelineEditorTab.svelte` layout.

#### [MODIFY] `src/lib/types/project.ts`
- Introduce `PipelineProgressEvent` to the shared Domain Type mappings.

#### [MODIFY] `src/lib/tauri.ts`
- Export an `onPipelineProgress(cb)` listener hook wrapper around `@tauri-apps/api/event`.
- Update `executePipeline()` signature with an optional `resumeFromBlockId` string.

#### [MODIFY] `src/App.svelte`
- Establish the `onPipelineProgress` listener.
- Set a global `$state` for `activePipelineProgress`.
- Update `handleRunPipeline()` to concatenate the results from `pipelineExecutionResult` if a resume block is provided, effectively merging the failed run history seamlessly with the successful resumed steps into the UI pane.

#### [MODIFY] `src/lib/components/PipelineEditorTab.svelte`
- If `thisExecution` contains status `'failed'` and `thisExecution.steps.length > 0`, calculate the pending block ID and display a new action button: **Continue Run ({unresolved steps} remaining)**.
- Re-architect the Pipeline Status header with dynamic updates reading off the new `activePipelineProgress` prop.
- Introduce an animated `[⟳] Generating Output: {current_block_name}` notification below the normal metrics!

## Open Questions
> [!NOTE] 
> Is there a specific place inside the center Pipeline pane you want the visual indicator? I'm planning to place it right next to or directly taking over the `{X} / {Total} blocks completed` line while it is running.

## Verification Plan

### Automated / Manual Test
- Intentionally typo a template like `03-repetition-audit` or throw a `ProjectStoreError::message("Simulated failure")` into the execution path to force it to fail on the third block.
- Verify that a **"Continue Pipeline"** button spawns on the UI.
- Verify clicking Continue resumes execution strictly starting from Block 3 and maintains the previous Block 1 and Block 2 artifacts on disk without hitting OpenRouter.
