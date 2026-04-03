---
created: 2026-04-03 2:30 PM
modified: 2026-04-03 2:34 PM
---

# Plan To Reach Milestone 1: Read-and-Navigate IDE Shell

## Summary
- Target milestone: a desktop shell that can create and open real local projects, show the project filesystem in an explorer, open supported assets in tabs, and show asset metadata in a right-side inspector.
- Stack is locked to `npm` + `Tauri 2` + `Svelte 5` + `Vite` + `TypeScript`, in this repo as a single app. Do not use `SvelteKit`, a database layer, or a monorepo split for milestone 1.
- Milestone 1 is explicitly read-only after project creation/open: no editing, save, template validation, rendering, execution, imports/exports, prompt-pack runtime work, or `:online` behavior.

## Implementation Changes
1. Align the repo to the milestone before code starts.
- Update `TODO.md` to add a milestone-1 checklist and move later slices such as validation, execution, and online support out of the immediate path.
- Keep `README.md`, the v2 spec, and the active roadmap consistent that milestone 1 is the read-and-navigate shell only.

2. Scaffold the application and baseline tooling.
- Create `src/`, `src-tauri/`, and `src/lib/types/`.
- Wire `npm` scripts for `dev`, `tauri dev`, `typecheck`, `lint`, and `test`.
- Use `svelte-check`, `eslint`, `vitest`, and `cargo test` as the required validation baseline.
- Keep styling lightweight and local: plain CSS with shared variables for the IDE shell; do not add Tailwind or a heavy component framework.

3. Build the backend filesystem slice first.
- Implement project creation, project validation/open, asset indexing, asset reading, and recent-project tracking as Tauri commands before building the UI shell.
- Treat a valid project as a folder containing `project.json`, `documents/`, `prompts/`, `models/`, `runs/`, and `exports/`.
- New project creation should generate the full folder structure plus a starter `models/default.yaml`, and set `default_model_preset` in `project.json` to that file so the manifest is valid from day one.
- Keep `documents/`, `prompts/`, `runs/`, and `exports/` empty on create; use the committed fixture, not new-project generation, to exercise markdown and `.tera` viewing.

4. Commit one canonical sample project fixture.
- Add `fixtures/sample-project/` with a valid `project.json`, one markdown document, one file-backed `.tera` prompt, one YAML model preset, and empty `runs/` and `exports/`.
- Make the fixture the primary smoke-test target for explorer, tabs, inspector, screenshots, and regression checks.
- Keep prompt metadata in `project.json`, with `prompt_blocks[*].template_source` pointing to the file under `prompts/`.

5. Build the milestone-1 UI in two app states, without a router.
- Project Browser state: recent projects list, `New Project`, and `Open Existing Project`.
- Workspace state: fixed five-region shell with left explorer, top tab bar, center read-only viewer, right inspector, and bottom placeholder panel.
- Transition from browser to workspace only after a successful create/open result from the backend.
- Defer browser rename/archive actions; they are not required for this milestone.

6. Keep the viewer intentionally simple and read-only.
- Use lightweight read-only panes, not Monaco or CodeMirror, for milestone 1.
- Support opening `.md`, `.txt`, `.tera`, `.yaml`/`.yml`, and `project.json`.
- Render markdown and text as readable text content; render `.tera` and YAML as raw code/text; render `project.json` as a structured JSON viewer.
- For unsupported files, show a non-failing placeholder tab instead of adding more viewers.

7. Define explorer and inspector behavior tightly.
- Explorer shows `project.json` plus grouped top-level asset directories, even when a directory is empty.
- Tab identity is by project-relative path; reopening an already open asset should focus the existing tab.
- Inspector always shows common metadata: asset kind, project-relative path, size, modified time.
- Add type-specific inspector details: project name/id/default model for `project.json`, line and word counts for documents, linked block names for `.tera` files referenced by the manifest, and parsed model id/basic fields for YAML presets when parsing succeeds.

8. Add recent-project persistence as app state, not project state.
- Store recents in a local JSON file under the Tauri app-data directory.
- Persist path, cached project name, last-opened timestamp, and last-known validity.
- Update recents only after successful create/open.
- If a recent path no longer exists, keep it visible but marked unavailable instead of deleting it silently.

## Public Interfaces And Types
- `ProjectManifest` should include the v2-required milestone fields: `project_id`, `project_name`, `created_at`, `updated_at`, `default_model_preset`, `variables`, `prompt_blocks`, `pipelines`, and `ui_preferences`.
- Add frontend/shared types for `ProjectSummary`, `RecentProjectEntry`, `AssetKind`, `ProjectAssetNode`, `AssetMetadata`, `AssetContent`, and `WorkspaceTab`.
- Expose Tauri commands for `create_project`, `open_project`, `list_project_assets`, `read_project_asset`, and `get_recent_projects`.
- `create_project` should return the same normalized project summary shape as `open_project` so the frontend can use one success path.
- `list_project_assets` should return the explorer tree plus basic counts needed by the browser/workspace chrome.
- `read_project_asset` should return raw content plus metadata, not editor-state concerns like dirty tracking or save conflicts.

## Test Plan
- Backend unit tests: create a project, validate/open a valid project, reject invalid folders, list assets, read supported files, and persist recents correctly.
- Fixture tests: confirm `fixtures/sample-project/` passes validation and exposes one markdown file, one `.tera` file, one YAML preset, and a valid manifest.
- Frontend tests: browser-to-workspace transition, explorer rendering, tab reuse by path, read-only viewer rendering for each supported type, and inspector updates by asset type.
- Validation gates for completion: `npm run typecheck`, `npm run lint`, `npm run test`, and `cargo test`.
- Manual acceptance in `tauri dev`: create a new project, open the sample fixture, open each supported asset type in tabs, confirm inspector metadata changes correctly, and confirm there is no edit/save/run behavior exposed.

## Assumptions And Defaults
- Milestone 1 ships the shell and filesystem contract only; editing begins in the next slice.
- `project.json` remains the source of truth for prompt-block metadata even though prompt bodies are file-backed under `prompts/`.
- New blank projects get a starter model preset file but no starter document or prompt template.
- The app starts in the Project Browser and does not implement session restore, multi-window behavior, or quick-open yet.
- `workshop-parts/` stays untouched during this milestone except as later reference material; no runtime migration work is needed to finish milestone 1.
