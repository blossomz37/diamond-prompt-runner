---
created: 2026-04-03 2:16 PM
modified: 2026-04-04 12:20 AM
---
# Diamond Runner: Planning, Setup, and Engineering Roadmap

## Summary

Use a single-app-repo Tauri + Svelte architecture in this repo and build the product as vertical slices, not as disconnected screens. The first real milestone is a read-and-navigate shell: create and open real local projects, load the explorer from disk, open `.md`, `.tera`, and model YAML files in tabs, and show metadata in the inspector. Editing, rendering, and execution come in later slices.

This is the next active implementation plan for the workspace.

Current slice status:
- Milestone 1 is complete
- the Editing Slice is complete
- the Rendering and Validation Slice is complete
- the Single-Block Execution Slice is complete
- the Pipeline Runner Slice is complete
- the Run History Slice is complete
- the Pipeline Authoring Slice is complete

Next active milestone plan after the Run History Slice:
- `implementation-plans/12-PLAN-pipeline-authoring-slice.md`

Sequence after Milestone 1:
- completed small follow-up: unavailable-recents `Locate` flow
- completed milestone: Editing Slice
- completed milestone: Rendering and Validation Slice
- completed milestone: Single-Block Execution Slice
- completed milestone: Pipeline Runner Slice
- completed milestone: Run History Slice
- completed milestone: Pipeline Authoring Slice
- later exports
- later scoped online-research extension

## Implementation Plan

### Phase 0: Lock Planning Inputs

- Treat `AGENTS.md`, `README.md`, and `specifications/SPEC_DIAMOND_RUNNER_v1.md` as implementation authority.
- Treat `implementation-plans/01-PLAN-migration-setup.md` and `implementation-plans/02-PLAN-specv1-to-specv2.md` as sequencing and reference inputs, not runtime truth.
- Treat `workshop-parts/` as reference and starter assets only.
- Keep scoped `:online` support out of the first implementation roadmap; design it as a post-MVP extension point with later tests and Help docs.

### Phase 1: Repo and App Bootstrap

- Scaffold the app directly in this repo as a single Tauri + Svelte application.
- Add the standard app entrypoints:
  - frontend app under `src/`
  - Tauri backend under `src-tauri/`
  - shared frontend type definitions under `src/lib/types/`
- Add one committed sample project fixture under `fixtures/sample-project/`.
- Add the minimum validation and tooling baseline:
  - `npm` scripts for dev, typecheck, lint, and test
  - Rust test command coverage via `cargo test`
  - frontend type validation via `svelte-check`
- Do not add database infrastructure. Keep the architecture file-first from the start.

### Phase 2: Project Filesystem Slice

- Implement real local project create and open flows in milestone 1.
- Support both:
  - create new project folder with `project.json` and required subdirectories
  - open existing local project folder and validate its shape
- Lock the initial on-disk contract for milestone 1:
  - `project.json`
  - `documents/`
  - `prompts/`
  - `models/`
  - `runs/`
  - `exports/`
- Use file-backed `.tera` prompt sources from the start. Do not store prompt bodies inline in the manifest for the first implementation.
- Include one sample project fixture that exercises:
  - at least one markdown document
  - at least one `.tera` prompt
  - at least one model YAML preset
  - basic manifest metadata

### Phase 3: IDE Shell Milestone

- Build the approved IDE-shell structure as the first user-facing milestone:
  - left explorer
  - top tabs
  - center editor or viewer
  - right inspector
  - bottom utility panel placeholder
- Milestone 1 done criteria:
  - create project
  - open project
  - list project assets in explorer
  - open supported assets in tabs
  - show metadata in inspector
  - no editing required yet
  - no execution required yet
- Supported first-slice asset types:
  - markdown and text documents
  - `.tera` prompt templates
  - model preset YAML
  - `project.json` in read-only structured form if opened directly
- The bottom panel can be present but inactive or placeholder-backed in milestone 1.

### Phase 4: Editing Slice

- Add save support after shell and navigation are stable.
- Editing scope for this phase:
  - markdown and text document editing
  - `.tera` template editing
  - model YAML editing
- Preserve file-first behavior:
  - edits write back to disk
  - tab dirty state is visible
  - save and reload behavior is deterministic
- Keep import and export out of this slice except where it is naturally part of file open and save.

### Phase 5: Rendering and Validation Slice

- Port the Tauri-side Tera validation pattern before execution work.
- Add backend validation commands for prompt templates using sample render context, returning per-template validation results.
- Define the first Diamond render context for validation and preview around Diamond nouns, not tracker or chat nouns:
  - project metadata
  - variables
  - selected document references
  - model id
  - local clock fields if needed
- Do not port tracker-specific `scope` or chat transcript assumptions.
- Add preview and warnings in the bottom panel or toggleable inspector area, consistent with the spec’s toggle-first behavior.
- Keep this slice execution-free: preview validation can warn on unresolved references, but execution-time strict blocking remains part of the later execution slice.

### Phase 6: Single-Block Execution Slice

- Add OpenRouter settings only after shell, files, and validation are working.
- Keep the first full execution slice on direct Rust HTTP to OpenRouter and defer any SDK evaluation until after the slice is complete.
- Initial execution capabilities:
  - per-project default preset
  - per-block preset override
  - curated starter presets seeded into `models/` for new projects
  - API-key storage with local desktop-safe storage strategy
  - single-block render and execute
  - persisted run record in `runs/`
  - raw provider response persistence so usage, token, and cost data remain available in the artifact when returned by OpenRouter
- Execution strictness rules for this slice:
  - invalid or missing `doc("...")` references fail before provider transport
  - unresolved variables fail unless the template explicitly guards them with `is defined` or a defaulting pattern
- Do not ship advanced provider controls in this slice.
- Treat any future online-research behavior as an extension point, not part of the initial runtime path.

### Phase 7: Pipelines, History, and Exports

- Add linear pipeline execution after single-block execution is stable.
- Persist run records and artifacts to `runs/`.
- Add broader run history browsing and output inspection beyond the current prompt-scoped bottom-panel history.
- Add import and export support for:
  - markdown documents
  - `.tera` templates
  - workflow and config JSON
  - selected run artifacts
- Keep exports file-based and derived; never make them the source of truth.

### Phase 8: Post-MVP Online Research Extension

- Add scoped online-research prompt behavior only after the local-first execution path is stable.
- Scope it to selected prompts or presets, not a global toggle-first design.
- Validate it with explicit tests and then document it in the in-app Help section.
- Do not rely on deprecated `:online` behavior as the product baseline without a fresh design review at implementation time.

## Important Interfaces And Decisions

- Repo shape:
  - single app repo rooted here
  - no monorepo split in phase 1
- First milestone project flow:
  - create new local project
  - open existing local project
- Prompt storage:
  - file-backed `.tera` under `prompts/`
- Fixture strategy:
  - commit one sample project fixture for development, screenshots, and tests
- Validation baseline:
  - frontend type and lint checks
  - targeted Rust and TypeScript tests per slice
- Tauri and backend interface for early slices should minimally cover:
  - create project
  - open and validate project
  - list assets
  - read asset content
  - stat asset metadata
  - later: save asset, validate template, run block
- Keep the first slice read-and-navigate only. Editing, preview, and execution are separate milestones.

## Test Plan

- Bootstrap and tooling:
  - `npm install` succeeds
  - `npm` typecheck, lint, and test commands are wired
  - `cargo test` runs for backend modules
- Project filesystem:
  - create a valid new project structure
  - open and validate an existing valid project
  - reject invalid project folders cleanly
- Shell and navigation:
  - explorer lists documents, prompts, models, runs, and exports
  - tabs open correct asset content
  - inspector updates by asset type
- Fixtures:
  - sample project loads without manual repair
  - fixture covers markdown, `.tera`, and YAML model cases
- Later validation slice:
  - valid `.tera` templates pass
  - invalid templates return useful per-template errors
  - unresolved document references warn correctly
- Later execution slice:
  - OpenRouter settings persist correctly
  - single-block runs create typed run records with version, prompt snapshot, output or error state, and raw provider response payload
- Post-MVP online slice:
  - scoped online-enabled prompts use the intended runtime path
  - non-online prompts remain unaffected
  - Help text matches real behavior and constraints

## Assumptions And Defaults

- Stack is Tauri + Svelte in this repo, not a separate bootstrap repo.
- Diamond stays file-first; no database-first detour from sibling repos.
- `workshop-parts/` remains reference and starter material, not implementation authority.
- The approved UX direction is `Diamond Runner v2 - IDE Pass`.
- Scoped online-research support is intentionally post-MVP for now.
