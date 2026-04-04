# Plan 18: Settings & Model Presets UI

## Context

The Settings / Model Presets surface is the first thing a user needs after opening a project — "Which model am I running against?" Currently:

- Five presets are seeded into `models/` on project creation
- The project default is `models/default.yaml`, set in `project.json → default_model_preset`
- Per-block overrides exist in the manifest (`prompt_blocks[].model_preset`)
- Users can edit preset YAML files directly in the workspace editor
- **No UI exists** to: list available presets, change the project default, create a new preset, set or clear the per-block override, rename the project, or manage the OpenRouter API key outside the bottom panel

---

## Resolved Decisions

- **Sidebar vs tab**: Settings is a collapsible left-sidebar section (like Variables). Compact enough to fit.
- **Per-block preset override**: Allowed in the Inspector as a small scoped exception. The Inspector becomes contextual — it doesn't show Project Summary and Run History at the same time.
- **Inspector context rules**:
  - No file active → Project Summary + Usage
  - File active → File Metadata + Block Preset Override (if `.tera` prompt)
  - Run History → always available but collapses when file-level detail is prominent
- **New preset creation**: Auto-generate YAML from a model ID with sensible defaults. User can edit afterward.
- **Project rename**: Bundled into this Settings section.
- **App version / update check**: Out of scope for this slice.

---

## Proposed Changes

### Left Sidebar: New "Settings" Section

Add a **Settings** collapsible section to the left sidebar (below Exports):

1. **Project Name** — editable text field showing the current project name. Saving updates `project.json → project_name`.

2. **Project Default Preset** — dropdown listing all `.yaml` files in `models/`, with the current default highlighted. Changing writes `default_model_preset` in `project.json`.

3. **Available Presets** — compact list of preset files:
   - Filename + model ID
   - "Edit" → opens YAML in center pane
   - "Delete" → removes file (blocked if it's the current default)

4. **New Preset** — model ID input + "Create" button. Auto-generates YAML with sensible defaults, opens in editor.

5. **API Key Status** — indicator showing whether an OpenRouter key is configured.

### Inspector: Contextual Layout

Make the Inspector context-aware:

- **No file active**: Project Summary, Usage Summary
- **File active**: File Metadata, per-block Preset Override dropdown (for `.tera` prompts linked to a manifest block)
- **Run History**: always available as a collapsible section, but doesn't crowd the file-level detail

### Backend: New Tauri Commands

| Command | Signature | Purpose |
|---------|-----------|---------|
| `list_model_presets` | `(root_path) → Vec<ModelPresetSummary>` | Scan `models/` for `.yaml` files |
| `set_default_model_preset` | `(root_path, preset_path) → ProjectSummary` | Update project default |
| `create_model_preset` | `(root_path, filename, model_id) → ModelPresetSummary` | Generate YAML from model ID |
| `delete_model_preset` | `(root_path, preset_path) → ()` | Remove preset file |
| `set_block_model_preset` | `(root_path, block_id, preset_path | null) → ProjectSummary` | Set/clear per-block override |
| `rename_project` | `(root_path, new_name) → ProjectSummary` | Update `project_name` in manifest |

---

## Critical Files

| File | Change |
|------|--------|
| `src-tauri/src/project_store.rs` | Add 6 new command implementations |
| `src-tauri/src/main.rs` | Register 6 new Tauri commands |
| `src/lib/tauri.ts` | Add bridge functions |
| `src/lib/types/project.ts` | Add `ModelPresetSummary` type |
| `src/lib/components/SidebarSettings.svelte` | **[NEW]** Settings sidebar section |
| `src/lib/components/WorkspaceShell.svelte` | Add Settings section, wire callbacks |
| `src/lib/components/InspectorPanel.svelte` | Contextual layout (Project vs File view) |
| `src/App.svelte` | Wire preset list state and callbacks |
| `src/App.test.ts` | Tests for presets, rename, contextual inspector |

---

## Verification Plan

### Automated Tests
- `npm run test -- --run` — new tests
- `npm run typecheck` — 0 new errors
- `npm run lint` — clean
- `cargo test --manifest-path src-tauri/Cargo.toml` — backend tests for all 6 new commands

### Manual Verification
1. Open project → expand Settings → see 5 presets with model IDs
2. Change default preset → Inspector "Project" section updates
3. Create new preset → appears in list, opens in editor
4. Delete non-default preset → removed from list
5. Attempt to delete default → error
6. Rename project → name updates everywhere
7. Open `.tera` file → Inspector shows file metadata + block preset override
8. Close file → Inspector reverts to Project Summary + Usage
