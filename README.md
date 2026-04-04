# Diamond Prompt Runner

Diamond Prompt Runner is a local-first desktop workspace for authoring, rendering, executing, and organizing AI prompt workflows on disk.

## Repository

GitHub: <https://github.com/blossomz37/diamond-prompt-runner>

## Current Direction

- Product spec: `specifications/SPEC_DIAMOND_RUNNER_v1.md`
- Approved prototype: `Diamond Runner v2 - IDE Pass`
- App direction: Tauri + Svelte desktop app with an IDE-like shell
- Current implementation status: Milestone 1 and the Editing Slice are complete, and the active milestone is the Rendering and Validation Slice
- Current Rendering and Validation behavior:
	- markdown, text, `.tera`, and model YAML assets are editable in the workspace
	- `project.json` remains read-only
	- `.tera` tabs validate against the current draft content
	- Diamond `doc("...")` references are resolved from `documents/` during preview validation
	- the bottom panel shows validation status, warnings or errors, context summary, and preview for `.tera` tabs
	- the bottom panel is toggleable via a chevron button in the pane header and auto-sizes to fit preview content
	- explorer directory nodes expand and collapse on click
- OpenRouter integration remains deferred to the later execution slice; provider calls, API-key storage, and SDK wiring are not yet in scope

Active implementation plan: `implementation-plans/08-PLAN-rendering-and-validation-slice.md`

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
