---
created: 2026-04-03 3:34 PM
modified: 2026-04-03 5:05 PM
---
# Plan To Reach The Editing Slice

## Summary

Milestone 1 is complete. The next real milestone is the Editing Slice.

Status update:
- the Editing Slice is implemented and manually validated against the sample project
- `project.json` remains read-only
- markdown, text, `.tera`, and model YAML assets are editable in the workspace
- the next active milestone is the Rendering and Validation Slice
- the active follow-on plan is `implementation-plans/08-PLAN-rendering-and-validation-slice.md`

This slice adds local file editing on top of the existing read-and-navigate shell without pulling validation, preview, or provider execution into scope. The goal is to make the workspace useful for direct project authoring while keeping Diamond file-first and keeping milestone boundaries explicit.

Sequence around this milestone:
- completed small post-milestone follow-up: unavailable-recents `Locate` flow
- next real milestone: Editing Slice
- after editing: Rendering and Validation Slice
- after validation: Single-Block Execution Slice
- later: Pipelines, History, and Exports
- later: scoped online-research extension

## Implementation Changes

### Editing behavior

- Add save support after shell and navigation stability from Milestone 1.
- Make markdown and plain-text documents editable in the center pane.
- Make file-backed `.tera` prompt templates editable in the center pane.
- Model YAML editing landed in this slice after manual validation feedback, while `project.json` stayed read-only.
- Keep `project.json` read-only in this slice unless a later plan explicitly promotes manifest editing into scope.

### File and tab behavior

- Edits write back to disk as normal project-file updates; no database layer or hidden draft store is introduced.
- Tab dirty state appears only after an actual content change from the last saved disk version.
- Save behavior is explicit and deterministic.
- Reopening a saved file shows the persisted content from disk.
- Reload or discard behavior is deterministic when the current tab contents differ from the on-disk file.
- Import and export workflows remain out of scope except ordinary file open and save behavior inside the project.

### Interface and backend additions

- Extend the early Tauri/backend surface from read-only access to include save support for editable text assets.
- Keep the supported editable asset set narrow in this slice:
  - markdown
  - plain text
  - `.tera`
  - YAML model presets
- Preserve existing read-only behavior for unsupported or deferred asset types.
- Do not add template validation, preview rendering, run execution, API-key storage, or provider settings in this slice.

## Implementation Tools

- `Svelte MCP` and `Context7`
  - primary guidance sources for Svelte 5 patterns, component state, filesystem-facing UI, Tauri integration, and test setup
- `Playwright MCP`
  - smoke checks for edit flows, save and reload behavior, dirty-state UX, and later preview or validation panels
- `Figma MCP`
  - use only if the editor or preview shell needs layout iteration or if the editing slice materially changes the approved IDE-shell interaction
- `GitHub` tools
  - repo hygiene, commit, PR, release, and publish workflows once the slice is implemented
- `Memory MCP`
  - use only for durable repo facts such as approved milestone ordering; not for product or runtime behavior
- `Notion`
  - optional planning and search helper if related project material lives there; not required for implementation

Cautions to carry forward:
- do not use `SQLite MCP` to justify database-first architecture
- do not use `workshop-parts/` as runtime truth
- do not pull OpenRouter wiring, API-key storage, or provider execution into the editing slice

## Test Plan

- Open an editable markdown or text document, change it, save it, and confirm the on-disk file updates.
- Open an editable `.tera` prompt, change it, save it, and confirm the on-disk file updates.
- Dirty state appears only after actual modification.
- Saving clears dirty state only when the persisted write succeeds.
- Closing and reopening a saved tab shows the persisted disk content.
- Reload or discard behavior is deterministic when the file on disk differs from the open tab contents.
- `project.json` and unsupported asset types remain read-only.
- No prompt validation, preview, or provider execution UI is exposed in this milestone.

Validation-specific tests stay out of this slice except whatever is required to safely support saving and reload behavior.

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- This file records the completed Editing Slice.
- The unavailable-recents `Locate` flow was the small follow-up after Milestone 1, and it is now complete.
- Model YAML editing is now part of the completed Editing Slice.
- `TODO.md` should stay concise and reminder-oriented rather than duplicating this full plan.
