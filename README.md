# Diamond Prompt Runner

Diamond Prompt Runner is a local-first desktop workspace for authoring, rendering, executing, and organizing AI prompt workflows on disk.

## Repository

GitHub: <https://github.com/blossomz37/diamond-prompt-runner>

## Current Direction

- Product spec: `specifications/SPEC_DIAMOND_RUNNER_v1.md`
- Approved prototype: `Diamond Runner v2 - IDE Pass`
- App direction: Tauri + Svelte desktop app with an IDE-like shell
- Current implementation status: Milestone 1, the Editing Slice, the Rendering and Validation Slice, and the Single-Block Execution Slice are complete; the Pipeline Runner Slice is now active with manifest-driven sequential pipeline runs and inspector-level pipeline controls in place
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
	- run artifacts are persisted under `runs/` as typed JSON records with `artifactVersion`, prompt and model metadata, rendered prompt snapshot, output or error fields, timestamps, and raw provider response payload
	- new projects now seed a small curated preset set inferred from `workshop-parts/openrouter/models/`: `default`, `gpt-5.4`, `gpt-5.4-nano`, `claude-sonnet-4.6`, and `gpt-5.2-think`
	- active editable `.tera` tabs now expose a `Run` action in the editor
	- the bottom panel shows latest execution metadata and output alongside validation and preview
	- the bottom panel also exposes minimal save or clear controls for the app-level OpenRouter key
	- the bottom panel now lists recent persisted runs for the active prompt and can open the JSON artifact in a tab
	- manifest-defined pipelines can now be listed and run sequentially from the inspector
	- pipeline runs stop on the first failing block and reuse the existing per-block run artifact contract under `runs/`
	- pipeline runs are guarded against unsaved related `.tera` drafts so the user does not accidentally run stale saved prompt files

Active implementation plan: `implementation-plans/10-PLAN-pipeline-runner-slice.md`

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
