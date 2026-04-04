# Output Target Document Generation & Run History JSON Slimdown

This plan addresses two related requests to overhaul output handling in Diamond Runner:

1. Honoring the `outputTarget` settings by automatically generating markdown files in the `documents/` folder when a run succeeds.
2. Slimming down the `runs/*.json` artifacts to act as lightweight orchestration ledgers rather than massive payload dumps, preserving metadata (variables, targets, paths) instead of raw text copies.

## Completed Changes

### Backend Execution Updates (Tauri + Rust)
- **JSON Schema Overhaul:** 
  - Purged the `rendered_prompt` and raw `response` payload from `PersistedRunRecord`, slashing the file size by over 80%.
  - Added `output_target: String` to dynamically reflect the selected extraction target.
  - Added `document_path: Option<String>` to serve as a pointer for text that gets persisted to the file system.
  - Added `variables: BTreeMap<String, String>` to permanently lock resolving context parameters into the run trace.
- **Output Routing:**
  - Evaluates `outputTarget` on the prompt block structure.
  - For `"document"` and `"both"` target types, materializes a `.md` artifact at `documents/{slugified-block-name}.md`.
  - For strictly `"document"` mode, the JSON artifact entirely drops the duplicated text, pointing instead solely to the destination document path to maintain ultra-lightweight logging.

### Frontend Updates (Svelte Bridge + Types)
- Exposes `outputTarget`, `documentPath`, and `variables` through the `PromptExecutionResult` bridge.
- Validates strict types against `ExecutionStatus` enums.
- Updates the **Validation Panel**, adding conditional readout components that intercept Document targets and render beautiful variable metric states seamlessly.

## Design Decisions
- Always slugifying the block name for document paths enables deterministic referencing in subsequent blocks (i.e. `{{ doc("genre-research.md") }}`) instead of trying to map unpredictable UUIDs or timestamps.
