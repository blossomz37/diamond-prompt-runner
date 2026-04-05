# 21-PLAN-asset-lifecycle-management

## Goal
Close the loop on Asset Lifecycle Management (CRUD operations) within the Diamond Prompt Runner IDE shell. The author should never be forced to drop back into their file system or manually edit `project.json` for essential destructive actions (Delete, Rename) or sharing actions (Export).

## Rationale
Currently, the "file-first" architecture relies on users manually manipulating files for deletions and renaming. Asking non-technical authors to edit raw JSON exposes the project to corruption risks, creates orphaned prompt templates, and severely disrupts writing immersion. 

## Phase 1: Pipeline Deletion
**Goal:** Safely remove pipelines from the workspace via UI.
- **Backend (Rust):**
  - Create `delete_pipeline(project_id, pipeline_id)` Tauri command.
  - Rust cleanly removes the pipeline entry from the `pipelines` array in `project.json` without formatting issues.
- **Frontend (Svelte):**
  - Add a "Delete" (trash icon or context menu) onto the pipeline entries in `SidebarPipelines.svelte`.
  - Wire up a confirmation modal: "Are you sure you want to delete this pipeline? (This does not delete the prompt blocks)."
  - Refetch / optimistic-update the local Project store upon success.

## Phase 2: Prompt Block Lifecycle
**Goal:** Safely remove prompt blocks from the workspace via UI without destroying underlying templates.
- **Backend (Rust):**
  - Create `delete_prompt_block(project_id, block_id)` Tauri command.
  - Logic: Remove block from `prompt_blocks` array in `project.json`.
  - **Important:** Do *not* delete the referenced `.tera` file, as it may be used by other blocks or kept for later use. Deleting physical templates should remain an explicit file browser action.
- **Frontend (Svelte):**
  - Add standard "Delete" actions onto Prompt Blocks in the IDE shell.
  - Display confirmation: "Remove this prompt block? (The underlying .tera template file will remain untouched)."

## Phase 3: Run History Management
**Goal:** Allow users to reclaim disk space and clear out junk generations.
- **Backend (Rust):**
  - Create a `delete_run(project_id, run_id)` Tauri command.
  - Rust physically deletes the run artifact JSON file stored in the `runs/` folder.
- **Frontend (Svelte):**
  - Add a trash icon to individual run entries in the `InspectorPanel` under the Run History tab.

## Phase 4: Document CRUD
**Goal:** Manage project document assets within the sidebar.
- **Backend (Rust):**
  - Implement `rename_document(project_id, old_path, new_path)` and `delete_document(project_id, path)`.
  - Ensure logic only allows deletions/renames confined within the `documents/` safe boundary.
- **Frontend (Svelte):**
  - Enhance `ExplorerTree.svelte` with a context menu (Right Click) or dropdown action menu next to document entries.
  - Allow renaming and deleting inline without leaving the app.

## Phase 5: Pipeline & Asset Export
**Goal:** Allow a user to export a pipeline cleanly through the UI.
- **Backend/Frontend Integration:**
  - Build upon the `13-PLAN-export-bundle-slice`.
  - Hook an "Export" button on the Pipeline UI that gathers the pipeline JSON object and its associated `.tera` templates into an organized export package.
  
## Success Criteria
- [ ] A user can delete a pipeline without opening `project.json`.
- [ ] A user can delete a prompt block without accidentally destroying the underlying `.tera` template file.
- [ ] A user can delete old runs to free up history clutter.
- [ ] Documents can be renamed and deleted natively from the Explorer Tree.
- [ ] The risk of JSON corruption from manual editing is effectively reduced to zero for daily use cases.
