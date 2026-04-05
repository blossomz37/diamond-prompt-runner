# Pipeline Architecture Redesign (File-First)

## 1. Motivation
Currently, pipelines are stored as an array inside the `project.json` manifest. While this keeps the project definition centralized, it reduces modularity. Moving pipelines to their own dedicated folder aligns perfectly with Diamond's "file-first" architecture, making them easier to export, share, version control, and template independently of the overarching project config.

## 2. Updated Directory Structure
The standard Diamond project layout will be expanded to include a `pipelines/` directory:

```text
MyProject/
├── project.json       (manifest: variables, defaults, prompt blocks)
├── documents/         (editable markdown docs)
├── prompts/           (tera templates)
├── pipelines/         <-- [NEW] (standalone pipeline json definitions)
├── models/            (yaml presets)
├── runs/              (history)
└── exports/           (exports/bundles)
```

## 3. Storage Changes
### `project.json` Updates
The `pipelines` array will be **removed** from `project.json`. 
`project.json` will continue to track `promptBlocks` (the individual templates/nodes), `variables`, and `uiPreferences`.

### `pipelines/` Directory
Each pipeline will map 1:1 to a file inside the `pipelines/` folder (e.g., `series_components.json`).
The pipeline ID is derived natively from the filename or kept inside the JSON schema.

**Example: `pipelines/series_components.json`**
```json
{
  "pipelineId": "series_components",
  "name": "Series Components Workflow",
  "orderedBlocks": [
    "01_data_analysis",
    "02_trope_analysis",
    "03_series_premise"
  ],
  "executionMode": "sequential"
}
```

## 4. Backend Implementation Changes (Rust / Tauri)
1. **Load Phase (`project_store.rs`)**
   - The backend will now scan the `pipelines/` directory upon opening a project.
   - It will parse all `*.json` files and aggregate them into the backend's memory state to send up to the Svelte frontend during project load.
2. **Save Phase (`project_store.rs`)**
   - When a user creates or edits a pipeline, the backend will serialize that specific pipeline directly to `pipelines/<pipeline_id>.json` rather than re-saving the entire `project.json`.
3. **Exporting**
   - Exporting a pipeline natively becomes as trivial as copying the targeted `.json` file from the `pipelines/` folder, alongside the referenced `promptBlocks` (`.tera` files).

## 5. Required Documentation Updates
Once approved and implemented, the following authoritative tracking docs must be updated to align with this new structure:
- `specifications/SPEC_DIAMOND_RUNNER_v1.md` (Update the architecture map and data models)
- `README.md`
- `AGENTS.md` 
