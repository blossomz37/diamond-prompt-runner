# Diamond Prompt Runner

Diamond Prompt Runner is a local-first desktop workspace for authoring, rendering, executing, and organizing AI prompt workflows on disk.

## Repository

GitHub: <https://github.com/blossomz37/diamond-prompt-runner>

## Current Direction

- Product spec: `specifications/SPEC_DIAMOND_RUNNER_v1.md`
- Approved prototype: `Diamond Runner v2 - IDE Pass`
- App direction: Tauri 2 + Svelte 5 desktop app with an IDE-like shell

### Implementation Status

All MVP slices (Plans 1–21) are complete. Post-MVP Plans 22–28 are shipped; some phases in Plans 22 (UX Overhaul) and 27 (Batch Execution) remain in progress.

**Completed slices:**
- Milestone 1 (project create/open, read-only IDE shell, explorer, tabs, inspector)
- Editing (markdown, text, `.tera`, model YAML; dirty state; `project.json` read-only)
- Rendering and Validation (`.tera` preview, `doc("...")` resolution, toggleable bottom panel)
- Single-Block Execution (OpenRouter direct HTTP, keychain credentials, persisted run artifacts)
- Pipeline Runner (sequential execution, stop-on-failure, unsaved-draft guards)
- Run History (project-wide + prompt-scoped, pipeline metadata filtering)
- Pipeline Authoring (create/edit in center pane, no manual JSON editing)
- Export Bundle (selected tabs to `exports/`, unsaved draft blocking)
- Usage Metrics (tokens, cost, word count; persisted in run artifacts)
- Variable Assignment UI (global + project variables, merged template context)
- Workspace Shell Restructure (collapsible sidebar, center-pane pipeline tabs, read-only inspector)
- Settings and Model Presets (project settings UI, auto-preset generator, inline overrides)
- Output Target & JSON Slimdown (`outputTarget: 'document'`, payload cleanup, `document_path` mapping)
- Asset Lifecycle — all 5 phases (pipeline delete, block removal, run delete, document rename/delete, pipeline export)
- Online Research — post-MVP extension (`{# diamond:online #}` directive, web-backed execution, citation tracking)
- Code Audit — backend complete (`types.rs`, `execution.rs`, `credentials.rs`, `history.rs`, `presets.rs`, `variables.rs`, `assets.rs` extracted from `project_store.rs`)
- Code Audit — frontend partial (`assetUtils.ts`, `textSearch.ts`, `validation.svelte.ts`, `CodeEditor.svelte`, `FindBar.svelte` extracted)
- UX Overhaul Phase 1 — sidebar reorganized to 10 sections
- CodeMirror 6 — syntax highlighting for `.tera` + markdown, custom Tera language support, FindBar adapter
- Tera Custom Filters — `extract_section(start="...", end="...")` for slicing document content by markers
- Live Events & Resume — `PipelineProgressEvent` emission, resume-from-block skip logic, continue-run UI

**In progress or partially shipped:**
- UX Overhaul Phases 2–4 (block-level pipeline overrides, help content population)
- Batch Execution (`extract_section` filter done; batch orchestration UI pending)
- CSS Audit findings documented (consolidation not started)
- Code Audit deferred items (App.svelte tab state extraction, WorkspaceShell split evaluation)

**Not started:**
- Duplicate Project / Pipelines (Plan 29)

### Current Layout

- The left sidebar has 10 collapsible sections: Models, Prompts, Blocks, Pipelines, Runs, Documents, Exports, Settings, Help
- Pipeline create/edit opens in the center pane as virtual tabs
- The right inspector is read-only: Project summary, Usage, File Metadata
- The bottom panel shows validation, preview, execution output, and run history for `.tera` tabs
- The editor uses CodeMirror 6 with syntax highlighting for Tera templates and markdown

### Execution

- Direct Rust HTTP to OpenRouter (no SDK)
- API key resolved from native OS keychain first, then `OPENROUTER_API_KEY` env var fallback
- Strict mode fails on unresolved variables and invalid/missing `doc("...")` references
- Run artifacts persisted under `runs/` with `artifactVersion`, usage metrics, variable tracking, and `document_path` references
- Pipelines: sequential execution, stop-on-first-failure, resume-from-block support, live progress events
- Online: `{# diamond:online #}` directive enables web-backed execution with citation tracking
- Custom Tera filter: `extract_section(start="...", end="...")` for slicing document content

### Variables

- Global variables persist to `app_data_dir/global-variables.json` and are shared across all projects
- Project variables persist in `project.json` and override globals with the same name
- Template context merges global → project → built-in variables

### Asset Lifecycle

- Pipelines: two-click inline delete from sidebar; export as bundle with referenced `.tera` templates
- Prompt blocks: manifest removal from sidebar (underlying `.tera` preserved on disk)
- Run artifacts: two-click inline delete from Run History; refreshes usage summary
- Documents: inline rename and delete from Explorer tree; rename auto-reopens in tab
- All deletions are guarded and fail cleanly without corrupting `project.json`

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

## License

This project is licensed under the [MIT License](LICENSE).

