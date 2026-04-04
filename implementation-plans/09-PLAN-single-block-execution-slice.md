---
created: 2026-04-03 8:00 PM
modified: 2026-04-03 11:20 PM
---
# Plan To Reach The Single-Block Execution Slice

## Summary

Rendering and Validation is complete. The Single-Block Execution Slice is now complete.

This slice should prove the end-to-end path for one prompt block:
- render one `.tera` prompt from the current draft content
- apply execution-time strictness rather than preview-time warning behavior
- call the selected provider for one run
- persist the run artifact under `runs/`
- surface execution state and result in the existing workspace shell

This slice does not include pipelines, run-history browsing, prompt-pack work, or scoped `:online` behavior.

Sequence around this milestone:
- completed small follow-up: unavailable-recents `Locate` flow
- completed milestone: Rendering and Validation Slice
- completed milestone: Single-Block Execution Slice
- next active milestone: Pipeline Runner Slice
- later: broader run history and exports
- later: scoped online-research extension

## Delivered So Far

### Backend execution foundation

- Add a backend-only `execute_prompt_block` path for active `.tera` prompt content.
- Reuse the current Tera rendering flow with stricter execution-time handling than preview validation.
- Resolve model presets from either the linked prompt block override or the manifest default preset.
- Call OpenRouter through direct HTTP from Rust for the first pass.
- Read the API key from `OPENROUTER_API_KEY` as the current temporary app-level credential source.
- Persist one JSON run artifact under `runs/` with run id, prompt path, rendered prompt, model info, timestamps, output, and raw response payload.
- Add backend tests for successful execution, strict document-reference failure, and block-level model override selection.

### Frontend execution wiring

- Add a `Run` action for active editable `.tera` tabs in the asset viewer.
- Execute from the current draft buffer so runs do not require a prior save.
- Surface in-flight, success, and failure state in the editor header.
- Show latest execution metadata and output in the existing bottom panel beside validation and preview.
- Add frontend tests for successful run output display and execution failure display.

### App-level credential storage

- Resolve the OpenRouter API key from native app keychain storage before falling back to `OPENROUTER_API_KEY`.
- Add Tauri commands to read credential status plus save or clear the stored key without placing secrets in project files.
- Expose minimal key save or clear controls in the bottom-panel execution area for active `.tera` tabs.
- Add backend unit coverage for credential selection priority and frontend tests for saving the key from the UI.

### Prompt-scoped run history browsing

- List persisted run artifacts from `runs/` for the active `.tera` prompt.
- Surface recent run summaries in the bottom panel without introducing a separate history screen.
- Open a selected run artifact JSON in a regular editor tab.
- Add backend coverage for prompt-scoped run listing and frontend coverage for opening a persisted run artifact.

### Execution strictness and artifact contract

- Treat invalid or missing `doc("...")` references as hard execution failures before any provider call is attempted.
- Treat unresolved variables as hard execution failures unless the template explicitly guards them with `is defined` or a defaulting pattern.
- Keep preview validation more permissive than execution so the UI can warn without blocking exploratory editing.
- Persist typed run records under `runs/` with `artifactVersion`, prompt path, block metadata, model metadata, rendered prompt snapshot, timestamps, output or error fields, and the raw provider response payload.

### Seeded model preset defaults

- Seed new projects with a small curated starter set inferred from `workshop-parts/openrouter/models/`.
- Keep `models/default.yaml` pointed at the strong general-purpose `openai/gpt-5.4` preset.
- Include `gpt-5.4-nano`, `claude-sonnet-4.6`, and `gpt-5.2-think` as explicit alternates in the starter pack.
- Keep `aion-2.0` out of the seeded defaults for now because the workshop notes treat it as an experimental later candidate rather than a standard route.

## Scope

Included in this slice:
- one manual run action for an active `.tera` prompt tab
- strict render-time validation for execution
- one provider integration path for OpenRouter-backed execution
- local run persistence under `runs/`
- basic execution status and result display in the existing shell

Explicitly out of scope for this slice:
- pipeline execution
- broad run-history workspace UX beyond prompt-scoped browsing in the bottom panel
- prompt-pack-specific execution logic
- broad settings UI beyond what is required to prove a single-block run
- scoped `:online` behavior

## Decisions To Lock Before Code

### 1. API-Key Storage Strategy

Decision needed:
- choose a local secure storage approach for the OpenRouter API key that does not place secrets inside project folders

Current recommendation:
- prefer native secure storage for secrets over project-file storage
- keep project truth file-first while treating credentials as app-level state

Current first-pass implementation default:
- use native app keychain storage first and fall back to `OPENROUTER_API_KEY` when no stored key exists

Implementation implication:
- execution code should depend on an app-level credential lookup rather than reading secrets from `project.json` or model YAML files

### 2. OpenRouter Integration Shape

Decision needed:
- choose between the official TypeScript SDK and direct HTTP requests through the Tauri boundary

Current recommendation:
- keep the first implementation backend-oriented and narrow; avoid inventing a larger provider abstraction until one real execution path exists

Current first-pass implementation default:
- use direct HTTP from the Rust backend for OpenRouter chat completions

Locked decision:
- keep direct Rust HTTP for the first full execution slice and defer SDK evaluation until after the slice is complete

Implementation implication:
- the first slice should optimize for a stable single-provider path rather than a generalized provider framework

### 3. Execution-Time Strictness Rules

Decision needed:
- define how execution differs from preview when variables or `doc("...")` references are unresolved

Locked default for planning:
- execution should be stricter than preview validation
- unresolved required context should fail execution rather than degrade to a warning-only result

Locked implementation rules:
- invalid or missing `doc("...")` references fail execution before provider transport runs
- unresolved variables fail execution when referenced directly in the rendered template
- variables remain optional only when the template explicitly guards them with `is defined` checks or a defaulting pattern

Implementation implication:
- reuse the current rendering path where possible, but separate preview behavior from execution behavior explicitly in backend code

### 4. Run Persistence Contract

Decision needed:
- define the minimum artifact written under `runs/` for one execution

Locked minimum contract for planning:
- run id
- source prompt path
- rendered prompt snapshot
- model identifier used
- timestamp
- completion status
- output text or error payload

Locked first-pass on-disk contract:
- `artifactVersion`
- `runId`
- `path`
- `blockId`
- `blockName`
- `modelPreset`
- `modelId`
- `status`
- `renderedPrompt`
- `output`
- `error`
- `startedAt`
- `completedAt`
- `response`

Implementation implication:
- the first execution slice should prove run persistence on disk before any richer run-history UI is built

## Recommended Implementation Order

### Unit 1: Backend Execution Foundation

Add the minimum backend path to:
- render the active `.tera` draft with execution-time strictness
- load the selected model preset
- execute one request
- persist one run artifact under `runs/`

Likely integration points:
- `src-tauri/src/project_store.rs` for shared project and asset helpers
- `src-tauri/src/main.rs` for Tauri command registration
- a new backend module for execution orchestration if the logic would otherwise overload `project_store.rs`

### Unit 2: Frontend Execution Wiring

Add the minimum workspace wiring to:
- trigger a run from an active `.tera` tab
- show in-flight, success, and error states
- display the latest run result in the bottom panel without replacing the validation surface entirely

Likely integration points:
- `src/App.svelte` for request orchestration and state
- `src/lib/components/WorkspaceShell.svelte` for shell-level status placement
- `src/lib/components/AssetViewer.svelte` for a run action on `.tera` tabs
- `src/lib/types/project.ts` for execution and run-result types

## Follow-Up Checklist

- [x] Implement the first backend-only execution foundation for a single `.tera` prompt tab.
- [x] Wire frontend run controls and bottom-panel execution output for active `.tera` tabs.
- [x] Finalize API-key storage approach for local desktop execution with native keychain storage and env-var fallback.
- [x] Decide to keep direct HTTP for the first full slice instead of switching to the SDK mid-slice.
- [x] Add prompt-scoped run history browsing and open-artifact support in the bottom panel.
- [x] Define exact strict-failure rules for unresolved variables and document references during execution.
- [x] Define the on-disk structure for first-pass run artifacts under `runs/`.
- [x] Add the execution-slice implementation plan to the umbrella roadmap once decisions are locked.
- [x] Review curated model preset defaults and choose the seeded starter set for new projects.

## Verification

- Planning docs must agree that Rendering and Validation is complete and Single-Block Execution is the next active milestone.
- The first execution slice must remain backend-first and single-block only.
- Execution work must continue to pass:
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Assumptions And Defaults

- `implementation-plans/03-PLAN-app-bootstrap-and-shell.md` remains the umbrella roadmap.
- This file becomes the active detailed plan once the Rendering and Validation slice is closed.
- Preview validation behavior stays more permissive than execution behavior.
- Project truth remains file-first; secrets do not become project-file state by default.