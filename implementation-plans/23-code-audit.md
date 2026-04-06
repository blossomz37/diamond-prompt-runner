---
source: "/Users/carlo/.claude/plans/merry-snacking-stallman.md"
created: 2026/04/05
---

# Code Splitting Audit — Diamond Runner

## Context
This is a read-only audit and staged implementation plan. No refactoring has been performed.
The goal is safe, incremental code splitting that preserves current behavior while improving maintainability, testability, and future development velocity.

---

## Audit Summary

- **Total first-party source files scanned:** ~35 (src/, src-tauri/src/)
- **Files over 300 lines:** 11
- **Strong split candidates:** 3 (project_store.rs, App.svelte, AssetViewer.svelte)
- **Moderate candidates:** 3 (WorkspaceShell.svelte, PipelineEditorTab.svelte, ProjectBrowser.svelte)
- **Reviewed but not recommended for splitting:** 5 (ValidationPanel.svelte, PromptBlockEditorTab.svelte, SidebarModels.svelte, ExplorerTree.svelte, App.test.ts)

---

## Priority Candidates

### `src-tauri/src/project_store.rs`
- **Line count:** 4,483
- **Recommendation:** Split
- **Priority:** High
- **Why:** At 15× the 300-line threshold, this file has 6 distinct responsibility clusters with no forced coupling between them. The test module alone is ~980 lines. Each cluster is a natural module boundary in Rust.
- **Current responsibilities:**
  1. Type/struct/enum definitions (30+ public, 3 private) — lines 1–462
  2. Project lifecycle (create, open, rename, recent projects) — lines 464–916
  3. Prompt block and pipeline CRUD — lines 521–1114
  4. Asset access (list, read, write, export) — lines 917–1244
  5. Template validation and execution engine — lines 1274–1817 (the most complex cluster)
  6. Credential management — lines 1370–1411
  7. Variable management (global, project, workspace) — lines 608–636 + helpers
  8. Run history and usage analytics — lines 1413–1500
  9. Model preset management — lines 705–759
  10. Private helpers (45 functions) and tests (~980 lines)
- **Recommended target structure:**
  ```
  src-tauri/src/
  ├── main.rs               (unchanged — thin IPC dispatcher)
  ├── project_store.rs      (retained as re-export hub + project lifecycle core)
  ├── types.rs              (all public structs, enums, error type)
  ├── execution.rs          (execute_prompt_block_with_transport, execute_pipeline_with_transport, validate_project_template, all execution helpers)
  ├── assets.rs             (list_project_assets, read_project_asset, write_project_asset, export_project_assets, build_metadata, classify helpers)
  ├── variables.rs          (get/set global/project/workspace variables + YAML/JSON helpers)
  ├── credentials.rs        (get_execution_credential_status, save/clear_execution_api_key)
  ├── history.rs            (list_prompt_run_history, list_project_run_history, get_project_usage_summary, delete_run)
  └── presets.rs            (list_model_presets, create/delete/set_default_model_preset, set_block_model_preset)
  ```
- **Suggested extracted units:**
  - `types.rs` — all serializable structs and enums currently at lines 53–462
  - `execution.rs` — validate_project_template, execute_prompt_block, execute_pipeline, execute_prompt_block_with_transport (180 lines), execute_pipeline_with_transport (92 lines), prepare_template_context (88 lines), preprocess_doc_references (52 lines)
  - `assets.rs` — list_project_assets, read_project_asset, write_project_asset, export_project_assets, build_metadata (86 lines with 8-branch asset-kind match)
  - `variables.rs` — 4 public + 6 private functions for variable resolution
  - `credentials.rs` — 3 public functions, keychain + env fallback logic
  - `history.rs` — 4 public functions for run artifacts
  - `presets.rs` — 6 functions for model preset CRUD
- **Key risks:**
  - Types used across all modules must be in `types.rs` and imported correctly — doing this first eliminates all cross-module compile errors in later stages
  - `project_store.rs` retains core manifest helpers (`validate_project`, `read_manifest`, `write_manifest`, `sanitize_relative_path`) that all modules depend on — these cannot be moved until a shared utilities module exists
  - The 45 private helper functions span multiple concerns; some (path helpers, manifest IO) must remain in a shared location
  - `main.rs` currently imports `project_store::*` — after splitting, it will need targeted imports from each submodule, or `project_store.rs` re-exports everything
- **Notes on sequencing:** Types first, then execution (highest value for testability), then leaf modules (credentials, history, presets), then assets, then variables last (most interwoven with manifest helpers).

---

### `src/App.svelte`
- **Line count:** 1,026
- **Recommendation:** Split
- **Priority:** High
- **Why:** This is a God object orchestrating ~12 distinct feature areas from a single component. The script block alone is 943 lines. Several functions are pure utilities with no reactive dependency that have been left inline by accident.
- **Current responsibilities:**
  1. Mode routing (browser vs. workspace)
  2. Project lifecycle (create, open, recent, locate)
  3. Tab management (open, close, select, reload, save, draft tracking)
  4. Template validation with debounce and request deduplication
  5. Prompt execution and history loading
  6. Pipeline authoring (create, update) and execution
  7. Asset mutation (delete doc, rename doc, delete run, export)
  8. Variable management (global, project)
  9. Credential management
  10. Model preset management
  11. Workspace state hydration (parallel API calls on project open)
- **Recommended target structure:**
  ```
  src/
  ├── App.svelte                           (orchestrator, reduced to ~400 lines)
  ├── lib/
  │   ├── utils/
  │   │   └── assetUtils.ts                (findAssetNode, latestStepForPath)
  │   └── stores/
  │       ├── tabs.svelte.ts               (tabs[], activePath, loadingPath, tab lifecycle functions)
  │       └── validation.svelte.ts         (validationResult, debounce logic, runTemplateValidation)
  ```
- **Suggested extracted units:**
  - `assetUtils.ts` — `findAssetNode` (lines 761–770, pure recursive search) and `latestStepForPath` (lines 518–529, pure array search): no reactive dependencies, already called by reference
  - `tabs.svelte.ts` — tabs state, openAssetPath, handleSelectTab, handleCloseTab, updateTab, handleDraftChange, handleSaveTab, handleReloadTab; these form a coherent lifecycle with no dependency on project or execution state beyond what can be passed in
  - `validation.svelte.ts` — validationResult, validationLoading, validationTimer, validationRequestId, runTemplateValidation, the `$effect` debounce block; this is already logically isolated and the deduplication pattern is worth testing in isolation
- **Key risks:**
  - `withBusy` pattern (lines 208–219) is used ~10 times and tightly wraps the `busy` and `errorMessage` top-level state — do not extract until busy/error ownership is clear
  - `enterWorkspace` (lines 122–153) does 7 parallel API calls to hydrate all workspace state — this cannot be split until each state slice has a home
  - Tab state is read by WorkspaceShell (40+ props) — moving tabs to a store requires WorkspaceShell to read the store directly instead of receiving props, which is a larger refactor
  - Suggest extracting utilities first (zero risk) before touching state
- **Notes on sequencing:** Pure utilities first (assetUtils.ts), then validation store (self-contained), then tabs store (requires WorkspaceShell prop audit).

---

### `src/lib/components/AssetViewer.svelte`
- **Line count:** 621
- **Recommendation:** Split
- **Priority:** High
- **Why:** The find/replace system (state + DOM refs + functions + UI) is a complete sub-feature embedded inline. It could stand alone as `FindBar.svelte`. Additionally, two pure text utilities (`getMatches`, `replaceAll`) have no reactive dependencies and belong in a shared module.
- **Current responsibilities:**
  1. Text/JSON/Markdown editor with draft tracking
  2. Find/replace system with case sensitivity, match cycling, DOM selection
  3. Markdown preview rendering via `marked`
  4. Execution status display and run action
  5. Save/reload actions
  6. Keyboard shortcut handling (Cmd+F, Cmd+H, Escape, Enter)
- **Recommended target structure:**
  ```
  src/lib/
  ├── components/
  │   ├── AssetViewer.svelte     (reduced ~350 lines — editor, header, callbacks)
  │   └── FindBar.svelte         (find/replace UI + internal state: showFind, showReplace, findText, replaceText, caseSensitive, matchIndex, editorEl ref)
  └── utils/
      └── textSearch.ts          (getMatches(), escapeRegex() — pure functions)
  ```
- **Suggested extracted units:**
  - `textSearch.ts` — `getMatches(text, needle, caseSensitive)` (lines 33–44) and the regex escape logic from `replaceAll` (lines 81–89); both are pure with no imports
  - `FindBar.svelte` — the find/replace state machine and its UI (lines 21–31 state, 33–109 logic, 236–276 template, find-bar styles)
- **Key risks:**
  - `FindBar` needs a reference to the outer `textarea` DOM element (`editorEl`) to call `.setSelectionRange()` — must be passed as a prop or resolved via a callback. The cleanest pattern is passing the editor ref as a prop to FindBar and letting FindBar call selectMatch internally.
  - `openFind`/`openFindReplace`/`closeFind` are currently triggered by keyboard events in `handleEditorKeydown` inside AssetViewer — after extraction, FindBar will need to expose an API (exported functions or event) to open/close from the parent
  - The `{#if showFind}` block in the template must remain wired to FindBar's internal state — using `bind:showFind` or a callback works cleanly in Svelte 5
- **Notes on sequencing:** Extract `textSearch.ts` first (zero risk, no UI touch), then extract `FindBar.svelte` (medium risk — requires passing editor ref and coordinating keyboard triggers).

---

### `src/lib/components/WorkspaceShell.svelte`
- **Line count:** 928
- **Recommendation:** Monitor
- **Priority:** Medium
- **Why:** Large but mostly coherent wiring code. The script block is only 255 lines; the remaining 670 lines are HTML layout and CSS. The sidebar section HTML (10 collapsible sections, lines 280–476) is repetitive but uses different child components with different prop shapes, making factoring into a loop difficult without over-abstracting. The CSS is substantial but functional and local.
- **Current responsibilities:**
  - IDE shell layout (3-column grid + bottom panel)
  - 10 sidebar section collapse toggles
  - Tab bar rendering and routing to editor/virtual tabs
  - Pipeline editor virtual tab lifecycle
  - Prompt block editor virtual tab lifecycle
  - Bottom validation panel toggle
  - Responsive layout breakpoints
- **Recommended target structure (if splitting is pursued):**
  - `lib/stores/workspaceSidebar.svelte.ts` — extract 10 boolean sidebar collapse states (low complexity, enables sidebar state persistence later)
  - The pipeline/block editor virtual tab state (lines 185–254) could move to `lib/stores/virtualTabs.svelte.ts`
- **Key risks:** The 40+ prop interface between App and WorkspaceShell is the dominant coupling concern. Splitting internal state to stores is low-risk, but the prop surface itself would remain. Do not split until App.svelte tab state is resolved.
- **Notes on sequencing:** Defer until App.svelte tabs store is done. Sidebar store extraction is low-risk if pursued independently.

---

### `src/lib/components/PipelineEditorTab.svelte`
- **Line count:** 554
- **Recommendation:** Monitor
- **Priority:** Medium
- **Why:** Has clear sub-sections (export operation, delete operation, block ordering, view/edit mode) but they are all tightly woven through the `editing` boolean toggle. Splitting now risks fragile prop chains for little real testability gain — none of these sub-features are independently testable without the parent state.
- **Recommended target structure (if splitting is pursued):**
  - `PipelineBlockManager.svelte` — dropdown + add/remove/reorder logic (lines 235–270)
  - `PipelineStepsView.svelte` — read-only pipeline steps list (lines 288–302)
  - Extract `blockLabel()` to shared utils
- **Key risks:** `editing` toggle cascades through nearly every conditional in the template. Export and delete both reset `editing` differently. Extraction without resolving the mode-toggle ownership will create prop drilling.
- **Notes on sequencing:** Best split after any general pattern for "two-step confirmation" exists in the codebase (currently duplicated across PipelineEditorTab, PromptBlockEditorTab, SidebarModels, ExplorerTree).

---

### `src/lib/components/ProjectBrowser.svelte`
- **Line count:** 525
- **Recommendation:** Monitor
- **Priority:** Medium
- **Why:** Contains two visually and logically distinct panels — credential management and project creation/recent projects — inside one component. The credential UI is especially self-contained and reusable. However, the component currently has minimal internal state (all driven by props from App), so splitting does not reduce App complexity.
- **Suggested extracted units:**
  - `CredentialsPanel.svelte` — credential status display, input, save/clear actions (lines 75–118 template + lines 48–62 helpers `credentialTone`, `credentialLabel`)
  - `RecentProjectsList.svelte` — recents list with locate/open/remove actions (lines 178–224)
- **Key risks:** `busy` prop gates everything; every extracted child needs it. Splitting adds prop drilling without reducing complexity until App.svelte also refactors credential state.
- **Notes on sequencing:** `CredentialsPanel` is safe to extract as a pure display + callback component. `RecentProjectsList` is straightforward. Do these after App.svelte credential state is reviewed.

---

### `src/lib/components/ValidationPanel.svelte`
- **Line count:** 565
- **Recommendation:** Leave as-is
- **Priority:** Low
- **Why:** This is a pure display component — zero internal state. All data flows in via props. The file is long because the output is information-dense (validation results, execution metrics with 10+ conditional fields, run history list). Splitting would fragment a coherent display into artificial pieces without improving testability or maintainability.

---

### `src/lib/components/PromptBlockEditorTab.svelte`
- **Line count:** 373
- **Recommendation:** Monitor
- **Priority:** Low
- **Why:** Just over threshold. The conditional output-strategy field and the two-step delete are the only structural candidates for extraction, and neither is complex enough to justify a new file at this time. The file's current size is appropriate for its feature scope.

---

### `src/lib/components/SidebarModels.svelte`
- **Line count:** 334
- **Recommendation:** Leave as-is
- **Priority:** Low
- **Why:** Barely over threshold. All five responsibilities are directly related to model preset management. The per-item delete confirmation pattern is slightly novel (string rather than boolean) but is localized and understandable.

---

### `src/lib/components/ExplorerTree.svelte`
- **Line count:** 320
- **Recommendation:** Leave as-is
- **Priority:** Low
- **Why:** The recursive `{@render renderNode}` snippet pattern is the core of this component. Converting it to a recursive child component in Svelte 5 is possible but changes the recursion model entirely and requires careful testing. The current structure is cohesive — all state (openDirs, hover, rename, delete confirm) serves a single tree interaction model. Splitting is high-risk for low maintainability gain.

---

### `src/App.test.ts`
- **Line count:** 1,310
- **Recommendation:** Out of scope (test file)
- **Why:** Excluded per audit criteria. Test files are expected to be long; length reflects coverage breadth. No structural split recommended unless test organization becomes a maintenance problem.

---

## Staged Refactor Plan

### `src-tauri/src/project_store.rs`

**Stage 1: Extract types to `types.rs`**
- Purpose: Give all public structs and enums their own module, eliminating cross-module type duplication risk in later stages.
- Exact changes:
  - Create `src-tauri/src/types.rs`
  - Move lines 53–462 (all public enums and structs: `ProjectStoreError`, `ProjectSummary`, `ProjectManifest`, `PromptBlock`, `Pipeline`, `PromptExecutionResult`, `PipelineExecutionResult`, `TemplateValidationResult`, `ProjectAssetNode`, `AssetContent`, `AssetMetadata`, `UsageMetrics`, `OnlineRunMetadata`, `ExecutionCredentialStatus`, all serializable types)
  - In `project_store.rs`: replace definitions with `mod types; pub use types::*;`
  - In `main.rs`: no changes needed if `project_store::*` re-exports everything
- Validation:
  - `cargo test --manifest-path src-tauri/Cargo.toml` — all existing tests must pass
  - `cargo build` — zero new warnings
  - `npm run tauri:dev` — app loads, project opens, prompt executes (smoke test)

**Stage 2: Extract execution engine to `execution.rs`**
- Purpose: Isolate the most complex and test-critical code cluster. After this stage, the execution engine can be unit-tested in isolation.
- Exact changes:
  - Create `src-tauri/src/execution.rs`
  - Move: `validate_project_template`, `execute_prompt_block`, `execute_pipeline` (public), and `execute_prompt_block_with_transport`, `execute_pipeline_with_transport`, `prepare_template_context`, `preprocess_doc_references`, `call_openrouter_with_retry` (private helpers)
  - These functions depend on: `types.rs` (already extracted), `validate_project` + `read_manifest` + `write_manifest` + `sanitize_relative_path` (stay in project_store.rs as shared utilities)
  - In `project_store.rs`: `mod execution; pub use execution::{validate_project_template, execute_prompt_block, execute_pipeline};`
- Validation:
  - `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass (execution tests are in the test module; ensure they still resolve function paths)
  - Manual: run a prompt from AssetViewer, verify result appears in ValidationPanel
  - Manual: run a pipeline, verify all steps complete and result shows
  - `npm run typecheck` (frontend unaffected but confirms nothing broken)

**Stage 3: Extract credentials to `credentials.rs`**
- Purpose: Isolate keychain interaction — the most OS-sensitive code — into a testable, replaceable module.
- Exact changes:
  - Create `src-tauri/src/credentials.rs`
  - Move: `get_execution_credential_status`, `save_execution_api_key`, `clear_execution_api_key` (all 3 public functions, lines 1370–1411)
  - These functions have no dependency on manifest helpers — they only use `keyring` crate and env vars
  - In `project_store.rs`: `mod credentials; pub use credentials::*;`
- Validation:
  - `cargo test` — all tests pass
  - Manual: open ProjectBrowser, verify credential status badge shows correctly (keychain/env/missing)
  - Manual: save an API key, reload app, verify it persists and status shows "Keychain"
  - Manual: clear API key, verify status reverts

**Stage 4: Extract leaf modules (history, presets, variables)**
- Purpose: Three small, self-contained responsibility clusters each become their own module.
- Exact changes (one sub-step each, can be done in sequence or separately):
  - `history.rs` — `list_prompt_run_history`, `list_project_run_history`, `get_project_usage_summary`, `delete_run`
  - `presets.rs` — `list_model_presets`, `create_model_preset`, `delete_model_preset`, `set_default_model_preset`, `set_block_model_preset`
  - `variables.rs` — `get_global_variables`, `set_global_variables`, `set_project_variables`, and private helpers `read_global_variables_store`, `write_global_variables_store`, `read_workspace_variables_yaml`, `write_workspace_variables_yaml`
  - Each: add `mod X; pub use X::*;` to `project_store.rs`
- Validation after each sub-step:
  - `cargo test` — all pass
  - `cargo build` — zero warnings
  - For history: manual — open Runs sidebar, verify entries appear; delete a run, verify it disappears
  - For presets: manual — create a preset, set as default, delete a preset
  - For variables: manual — set a global variable, open a project, set a project variable, run a prompt that references both via `{{ var_name }}`

**Stage 5: Extract asset management to `assets.rs`**
- Purpose: Consolidate the filesystem read/write/list/export cluster. `build_metadata` (86 lines, 8-branch match on AssetKind) is the main complexity reduction target.
- Exact changes:
  - Create `src-tauri/src/assets.rs`
  - Move: `list_project_assets`, `read_project_asset`, `write_project_asset`, `export_project_assets`, `build_metadata` and its private helpers (`classify_asset_kind`, `read_asset_view`, path utilities specific to asset resolution)
  - These functions depend on `validate_project`, `read_manifest` (stay in project_store.rs) and `types.rs` (already extracted)
  - In `project_store.rs`: `mod assets; pub use assets::*;`
- Validation:
  - `cargo test` — all pass
  - Manual: open a project, navigate the explorer tree (list_project_assets), open a document (read_project_asset), edit and save (write_project_asset), export (export_project_assets)
  - Verify metadata panel shows correct file info for different asset types (.tera, .md, .yaml)

---

### `src/App.svelte`

**Stage 1: Extract pure utilities to `lib/utils/assetUtils.ts`**
- Purpose: Zero-risk extraction of two pure functions that have no reactive dependencies. Establishes the utils directory pattern.
- Exact changes:
  - Create `src/lib/utils/assetUtils.ts`
  - Move `findAssetNode` (App.svelte lines 761–770) — recursive search through `ProjectAssetNode[]`
  - Move `latestStepForPath` (App.svelte lines 518–529) — find last pipeline step for a path
  - Export both functions from the module
  - In `App.svelte`: import and call them identically to current usage
- Validation:
  - `npm run typecheck` — zero errors
  - `npm run lint` — zero new warnings
  - `npm run test` — all App.test.ts tests pass (these functions are called indirectly through handlers that are tested)
  - Manual: open a project, click a file in the explorer tree (findAssetNode path); run a pipeline and verify step results show correctly (latestStepForPath path)

**Stage 2: Extract validation state to `lib/stores/validation.svelte.ts`**
- Purpose: The debounced validation system (state, effect, request deduplication) is a complete sub-feature. Extracting it makes the debounce logic independently testable and removes ~30 lines from App.svelte's script block.
- Exact changes:
  - Create `src/lib/stores/validation.svelte.ts` using Svelte 5 runes (`$state`, `$effect`)
  - Move: `validationResult`, `validationLoading`, `validationTimer`, `validationRequestId` state
  - Move: `runTemplateValidation` function and the `$effect` debounce block (App.svelte lines 160–190)
  - The store needs the active tab path and workspace project path as inputs — expose these as settable state or pass via a `setContext` function
  - In `App.svelte`: instantiate the store, wire `activeTab` into it, read `validationResult`/`validationLoading` from it
- Validation:
  - `npm run typecheck` — zero errors
  - `npm run test` — all App.test.ts validation-related tests pass
  - Manual: open a .tera file, edit it, wait for debounced validation to fire, verify ValidationPanel updates
  - Manual: rapidly type in a template — verify only one validation call fires (deduplication working)
  - Manual: switch tabs quickly — verify validation state resets cleanly

**Stage 3 (deferred): Tab state store**
- Purpose: Extract `tabs`, `activePath`, `loadingPath` and all tab lifecycle functions to `lib/stores/tabs.svelte.ts`
- Prerequisite: WorkspaceShell must be audited for prop binding impact first. WorkspaceShell receives ~15 tab-related props. Moving tabs to a store means WorkspaceShell reads the store directly, which changes the data flow model.
- Flag: Do not begin Stage 3 until the WorkspaceShell prop surface has been mapped and there is a clear decision on whether WorkspaceShell should use stores or continue with prop drilling.

---

### `src/lib/components/AssetViewer.svelte`

**Stage 1: Extract pure text search utilities to `lib/utils/textSearch.ts`**
- Purpose: `getMatches` (lines 33–44) and the regex escape logic in `replaceAll` (lines 81–89) are pure functions with no imports or reactive dependencies. Zero-risk extraction.
- Exact changes:
  - Create `src/lib/utils/textSearch.ts`
  - Export `getMatches(text: string, needle: string, caseSensitive: boolean): Array<{start: number, end: number}>`
  - Export `escapeRegex(str: string): string` (the regex escape used inside replaceAll)
  - In `AssetViewer.svelte`: import and call identically
- Validation:
  - `npm run typecheck` — zero errors
  - `npm run test` — add 2–3 unit tests for getMatches (empty, single match, multiple matches, case sensitivity)
  - Manual: open a .tera file, use Cmd+F, verify find highlights work correctly

**Stage 2: Extract `FindBar.svelte`**
- Purpose: The find/replace system (state, functions, UI, styles) is a cohesive sub-feature. Extracting it reduces AssetViewer from ~621 to ~350 lines and makes the find/replace feature independently maintainable.
- Exact changes:
  - Create `src/lib/components/FindBar.svelte`
  - Move to FindBar:
    - State: `showFind`, `showReplace`, `findText`, `replaceText`, `caseSensitive`, `matchIndex`, `findInputEl`
    - Functions: `findNext`, `findPrev`, `replaceCurrent`, `replaceAll`, `openFind`, `openFindReplace`, `closeFind`, `selectMatch`
    - Imports: `getMatches`, `escapeRegex` from textSearch.ts
    - Template: lines 236–276 (the find bar HTML)
    - Styles: find-bar related CSS rules
  - FindBar props:
    - `editorEl: HTMLTextAreaElement | null` — needed for DOM selection
    - `content: string` — the current text (for getMatches)
    - `onContentChange: (newContent: string) => void` — for replaceAll to update draft
  - FindBar exports (for parent to trigger): expose `open(withReplace: boolean)` and `close()` as component methods (Svelte 5 `$bindable` or a returned object from a function)
  - In `AssetViewer.svelte`:
    - Replace find bar state/functions with a `let findBar` ref to the FindBar component
    - `handleEditorKeydown`: call `findBar.open(false)` for Cmd+F, `findBar.open(true)` for Cmd+H
    - `{#if showFind}` block replaced by `<FindBar bind:this={findBar} ... />`
- Validation:
  - `npm run typecheck` — zero errors
  - `npm run lint` — zero warnings
  - `npm run test` — all tests pass (App.test.ts exercises AssetViewer indirectly)
  - Manual: open a .tera file → Cmd+F opens find bar, match count shows, Prev/Next cycles matches
  - Manual: Cmd+H opens replace panel, Replace replaces current, Replace All replaces all
  - Manual: Escape closes the bar, editor regains focus
  - Manual: case-sensitive toggle changes match count
  - Manual: switch tabs, reopen find — state resets correctly (no stale matches from previous file)

---

## Final Recommendation

**First file to split: `src-tauri/src/project_store.rs` — Stage 1 (types.rs)**

**Why this is the best first move:**
- Highest ROI: 4,483 lines with 6 distinct modules is the clearest structural problem in the codebase
- Stage 1 (types extraction) is the lowest-risk entry point — it is purely additive, requires no logic changes, and every later stage depends on it
- Rust's module system is explicit and compiler-checked — if the extraction is wrong, it fails to compile, making validation automatic
- This sets the foundation for extracting `execution.rs` (Stage 2), which is where the most complex and hard-to-test code lives

**Second move: `src/lib/utils/textSearch.ts` + `src/lib/utils/assetUtils.ts`**
- Both are zero-risk, zero-coupling pure utility extractions
- They establish the `lib/utils/` directory pattern for future helpers
- Together they take <30 minutes and validate the extraction workflow on the frontend

**Third move: `src/lib/components/FindBar.svelte`**
- High value: reduces AssetViewer by ~35%, isolates a complete sub-feature
- Medium risk: requires the editor ref prop pattern to be done correctly
- Do Stage 1 (textSearch.ts) first so FindBar already imports from the right place

**Files that should wait:**
- `App.svelte` tab store (Stage 3) — requires WorkspaceShell prop audit first
- `WorkspaceShell.svelte` — defer until App.svelte tab state is resolved
- `PipelineEditorTab.svelte` and `ProjectBrowser.svelte` — monitor; only split if the two-step confirmation pattern gets formalized or credential state moves

**Prerequisites before starting:**
- No prerequisites for project_store.rs Stage 1 or frontend utils extraction
- Confirm Rust module structure: `mod types; pub use types::*;` pattern works in this Tauri setup (it will, but verify with a single compile step)
- For FindBar: verify Svelte 5 component method exposure pattern used elsewhere in the codebase, or confirm `bind:this` + exported functions approach works as expected
