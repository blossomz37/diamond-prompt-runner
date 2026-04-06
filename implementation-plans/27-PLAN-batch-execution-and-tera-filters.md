# 27-PLAN-batch-execution-and-tera-filters

## Mission Statement
Evolve the Diamond Runner pipeline architecture to support complex automation tasks, specifically batch looping over text chunks like chapters, without introducing visual graph-programming complexity. This is achieved by:
1. **Custom Tera Filters:** Creating a custom Tera filter (`extract_section`) to dynamically parse subsections of documents during prompt execution.
2. **Batch Orchestration:** Building parameterized "Batch Execution" for pipelines, allowing a single linear pipeline to run recursively across an array of payloads (e.g. `chapter 1`, `chapter 2`...).

This approach adheres to the primary Diamond specification: pipelines remain purely linear and deterministically executed. The looping behavior is isolated entirely in batch queueing, avoiding runaway infinite states.

---

## Phases

### Phase 1: Custom Tera Filtering (Backend)
- Locate Tera initialization inside `src-tauri/src/project_store/execution.rs` (or related file).
- Implement an `extract_section` (or similar regex/string isolation) function in Rust.
  - *Example Template usage:* `{{ doc("Outline.md") | extract_section(start="CHAPTER 5", end="CHAPTER 6") }}`
- Provide resilient error handling: return clear markdown tags or empty strings instead of crashing if the target string boundaries aren't found.
- Write isolated Rust unit tests for the filter to ensure stability.

### Phase 2: Backend Batch Architecture (Backend)
- Extend the Pipeline execution functions in `execution.rs` and the payload structs in `types.rs`.
- Allow the execution function to accept an optional array/list of "Runtime Variables" (e.g., `Vec<Value>`).
- Implement loop dispatching: if a payload array is passed, iterate through the payloads, injecting them into the project variables state *for the duration of each pipeline run*.

### Phase 3: UI Controls for Batch Execution (Frontend)
- Update Svelte components (likely `PipelineEditorTab.svelte` or `SidebarPipelines.svelte`).
- Expose a "Batch Run" UI mechanic. This could be a textarea taking JSON array payloads or a visual payload generator.
- Hook the payload array up to the Tauri IPC bridge, sending the batch payloads down to the Rust execution runner.

---

## Sanity Checks, Linting, and Validation
- **After Phase 1 & 2 (Rust Backend):**
  - Run `cargo test --manifest-path src-tauri/Cargo.toml`
  - Ensure existing prompt validation tests (`project_store.rs`) still pass stringently.
- **After Phase 3 (Svelte Frontend):**
  - Run `npm run typecheck`
  - Run `npm run lint`
  - Run `npm run test`
- **Final Integration:**
  - Build smoke-test loop utilizing `npm run tauri:dev` with an actual demo dataset to confirm no race conditions occur during fast artifact generations.

---

## Smart Commits Strategy
1. `feat(backend): implement extract_section tera filter with tests` *(After Phase 1)*
2. `feat(backend): add payload parameterization to pipeline runner` *(After Phase 2)*
3. `feat(frontend): implement UI for pipeline batch variables and routing` *(After Phase 3)*

---

## Evaluations for Scope Creep and Code Bloat
* **Risk (Bloat):** Transforming Diamond into a DAG graph node system or injecting recursive node dependencies like n8n.
* **Mitigation:** Strictly enforce pipelines to remain 1D sequential arrays in `types.rs`.
* **Risk (Scope Creep):** Over-engineering the UI to have complex data-table editors for "Batch" payloads.
* **Mitigation:** Start simply with JSON entry for batch payloads on the frontend, allowing authors to manually define `[{"chapter": 1}, {"chapter": 2}]`.
* **Risk (Bloat):** Creating overly complex file-locking mechanisms causing hanging execution.
* **Mitigation:** Ensure batch runs are executed synchronously in the backend loop (Run 1 finishes writing, THEN Run 2 starts) to avoid asynchronous file corruption.
