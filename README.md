# Diamond Prompt Runner

Diamond Prompt Runner is a local-first desktop workspace for authoring, rendering, executing, and organizing AI prompt workflows on disk.

## Repository

GitHub: <https://github.com/blossomz37/diamond-prompt-runner>

## Current Direction

- Product spec: `specifications/SPEC_DIAMOND_RUNNER_v1.md`
- Approved prototype: `Diamond Runner v2 - IDE Pass`
- App direction: Tauri + Svelte desktop app with an IDE-like shell
- Current implementation status: Milestone 1, the Editing Slice, the Rendering and Validation Slice, the Single-Block Execution Slice, the Pipeline Runner Slice, the Run History Slice, the Pipeline Authoring Slice, the Export Bundle Slice, the Usage Metrics Slice, the Variable Assignment UI, the Workspace Shell Restructure, the Settings and Model Presets Slice, the Output Target & JSON Slimdown Slice, and the Asset Lifecycle Slice (Phases 1–3: pipeline deletion, prompt block deletion, run history deletion) are complete
- Current Rendering and Validation behavior:
	- markdown, text, `.tera`, and model YAML assets are editable in the workspace
	- new prompt blocks can be created directly from the explorer without hand-editing `project.json`
	- `project.json` remains read-only
	- `.tera` tabs validate against the current draft content
	- Diamond `doc("...")` references are resolved from `documents/` during preview validation
	- the bottom panel shows validation status, warnings or errors, context summary, and preview for `.tera` tabs
	- the bottom panel is toggleable via a chevron button in the pane header and auto-sizes to fit preview content
	- explorer directory nodes expand and collapse on click
- Current execution-slice status:
	- backend-only `execute_prompt_block` support exists in the Tauri layer
	- the first full execution slice stays on direct Rust HTTP to OpenRouter
	- execution now resolves the OpenRouter API key from native app keychain storage first, then falls back to `OPENROUTER_API_KEY`
	- execution fails strictly on invalid or missing `doc("...")` references and on unresolved variables unless the template guards them explicitly
	- run artifacts are persisted under `runs/` as typed JSON records with `artifactVersion`, prompt and model metadata, a `document_path` reference if writing to the `documents/` pool directly, timestamps, and full `variables` injection tracking
	- new projects now seed a small curated preset set inferred from `workshop-parts/openrouter/models/` and model presets can now dynamically generate YAML skeletons purely from entering a known model string ID in settings
	- active editable `.tera` tabs now expose a `Run` action in the editor
	- the bottom panel shows latest execution metadata and output alongside validation and preview
	- the bottom panel also exposes minimal save or clear controls for the app-level OpenRouter key
	- the bottom panel now lists recent persisted runs for the active prompt and can open the JSON artifact in a tab
	- the left sidebar lists and runs manifest-defined pipelines; pipeline create and edit open in the center pane
	- manifest-defined pipelines can be created and edited without hand-editing `project.json`
	- pipeline runs stop on the first failing block and reuse the existing per-block run artifact contract under `runs/`
	- pipeline runs are guarded against unsaved related `.tera` drafts so the user does not accidentally run stale saved prompt files
	- the inspector shows project-wide persisted run history sourced from `runs/` and can filter those artifacts by block or pipeline
	- pipeline-originated block runs now retain pipeline metadata in their persisted artifacts so project history can group and inspect them later
	- the left sidebar can export selected open tabs into a derived bundle under `exports/` and writes bundle metadata alongside the copied artifacts
	- export bundles are derived from saved files on disk and block attempts to export unsaved editable drafts
	- prompts whose first non-empty line is `{# diamond:online #}` now opt into OpenRouter web-backed execution without changing standard prompt runs
	- online-enabled runs now persist audit metadata for whether online mode was enabled, how many web-search requests were issued, and how many citations were returned
	- execution results now extract and display usage metrics from the provider response: prompt tokens, completion tokens, total tokens, estimated cost, and output word count
	- persisted run artifacts and run history entries carry usage metrics extracted from the raw OpenRouter response
- Current variable and layout status:
	- global variables persist to `app_data_dir/global-variables.json` and are shared across all projects
	- project variables persist in `project.json` and override globals with the same name
	- template context merges global → project → built-in variables
	- the left sidebar now has collapsible sections: Explorer, Pipelines, Variables, and Exports
	- pipeline create and edit open in the center pane as virtual tabs
	- the right inspector is read-only: Project summary, Usage, File Metadata, and Run History

- Current asset lifecycle status:
	- pipelines can be deleted from the Pipelines sidebar with a two-click inline confirmation; deletion removes the pipeline from `project.json` without touching any prompt block templates
	- prompt blocks can be removed from the manifest via the "Registered Blocks" list in the Explorer sidebar; the underlying `.tera` file is preserved on disk
	- individual run artifacts can be deleted from the Run History inspector with the same two-click inline confirmation; deletion removes the JSON file from `runs/` and refreshes the usage summary
	- all three deletions are guarded: they fail cleanly if the target is not found and do not corrupt `project.json`

Active implementation plan: `implementation-plans/21-PLAN-asset-lifecycle-management.md`

## Recommended Agent Setup

Recommended plugins and tool surfaces for this repo:
- Figma for prototype iteration and later design-to-code work
- GitHub for commit, PR, release, and publish workflows
- Svelte docs and Context7 for implementation guidance
- Playwright for UI smoke tests and regression checks once the app shell exists
- Notion only if planning or spec material for this project is kept there

Recommended custom-instruction themes:
- treat `AGENTS.md`, `README.md`, and `specifications/SPEC_DIAMOND_RUNNER_v1.md` as authority
- keep Diamond file-first
- prefer vertical slices over screen-by-screen implementation
- treat `workshop-parts/` as reference material, not runtime truth
- keep local commits milestone-based and pushes explicit

See `implementation-plans/04-PLAN-agent-tooling-and-workflows.md` for the detailed tooling audit and workflow guidance.
