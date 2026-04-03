# Diamond Workspace Migration Map From `yfd-runner`

## Summary

Set up the new Diamond workspace as a Tauri + Svelte app with `workshop-parts/` as a staging area for reusable engine logic and prompt assets. Treat `yfd-runner` as source material, not as the new app architecture.

The migration should preserve four things from `yfd-runner`:
- OpenRouter request plumbing
- local JSON persistence patterns
- usage/cost accounting
- prompt-pack assets

It should explicitly discard the current novel-pipeline execution model as a runtime dependency.

## New Workspace Shape

Use this baseline layout in the new Diamond repo:

- `workshop-parts/source-python/`
  Raw copied files from `yfd-runner` for reference.
- `workshop-parts/starter-assets/models/`
  OpenRouter model preset YAML files copied from `yfd-runner/models/`.
- `workshop-parts/starter-assets/prompt-packs/yfd-novel/`
  Prompt templates copied from `yfd-runner/templates/`.
- `src-tauri/src/openrouter/`
  Rust client, config loading, payload building, retry handling.
- `src-tauri/src/project_store/`
  Local project persistence, atomic writes, ids, timestamps.
- `src-tauri/src/metrics/`
  Call accounting, cumulative usage summaries.
- `src-tauri/src/template_engine/`
  Tera rendering plus Diamond document/variable resolution.
- `src-tauri/src/export/`
  Document bundle/export assembly.
- `src/lib/types/`
  Shared TypeScript types for projects, documents, prompt blocks, runs, and metrics.

Adopt these public project-level interfaces in the new app:
- `project.json` as the workspace manifest
- `documents/` for editable markdown/text assets
- `prompts/` for user prompt blocks or pipeline definitions
- `models/` for OpenRouter preset files
- `runs/` for execution history and outputs
- `exports/` for zipped or compiled output bundles

## Migration Map

| Old source | Copy into `workshop-parts/` | New Diamond target | Exact change needed |
|---|---|---|---|
| `yfd-runner/api.py` | `source-python/api.py` | `src-tauri/src/openrouter/client.rs` and `config.rs` | Port model preset loading, payload assembly, retry loop, response-text extraction, and error mapping. Remove `run_id`/chapter-specific logic. Replace Python env loading with Tauri-side config/env resolution. |
| `yfd-runner/models/*.yaml` | `starter-assets/models/*.yaml` | `models/*.yaml` | Keep format. Rename only if desired for product clarity. Treat as Diamond model presets selectable by prompt block or project default. |
| `yfd-runner/state.py` | `source-python/state.py` | `src-tauri/src/project_store/store.rs` and `ids.rs` | Port atomic JSON write, load/save helpers, timestamp generation, and id generation. Remove worksheet parsing, chapter buckets, and story-specific fields. Replace run schema with `project.json`, document records, prompt block records, and run records. |
| `yfd-runner/metrics.py` | `source-python/metrics.py` | `src-tauri/src/metrics/store.rs` | Port usage extraction, word counting, per-call recording, and cumulative summaries. Replace `chapter` and `step` fields with `block_id`, `pipeline_id`, `run_id`, and `document_id` as appropriate. |
| `yfd-runner/validator.py` | `source-python/validator.py` | `src-tauri/src/template_engine/validation.rs` | Keep placeholder detection, word count, and basic completion checks. Remove `check_cascade_response` and prose-specific assumptions unless a prompt pack opts into them. |
| `yfd-runner/renderer.py` | `source-python/renderer.py` | `src-tauri/src/template_engine/render.rs` | Rebuild around Tera. Keep the idea of template lookup and render-to-file debug output. Replace worksheet/chapter context building with Diamond context assembly from project docs, variables, prior block outputs, and selected documents. |
| `yfd-runner/templates/*.j2` | `starter-assets/prompt-packs/yfd-novel/*.j2` | `prompt-packs/yfd-novel/*.tera` | Convert syntax where needed, then treat these as a bundled domain-specific pack for story workflows. Do not make them the core engine assumptions. |
| `yfd-runner/manuscript.py` | `source-python/manuscript.py` | `src-tauri/src/export/assembler.rs` | Generalize from “final chapters to manuscript” into “selected documents/outputs to export bundle or compiled doc.” |
| `yfd-runner/runner.py` | `source-python/runner.py` | `src-tauri/src/execution/orchestrator.rs` | Use only as behavioral reference for execute -> validate -> save -> metric-record flow. Do not port the fixed `cascade` and `plan/draft/...` step order. |
| `yfd-runner/config.yaml` | `source-python/config.yaml` | none as-is | Use only as a reference for how presets and per-step overrides worked. Replace with Diamond project settings and model preset assignment rules. |
| `yfd-runner/ui.py` | `source-python/ui.py` | none by default | Keep only as an archival example of a minimal local inspector. Do not port into the Diamond runtime. |
| `yfd-runner/user-commands.md` | `source-python/user-commands.md` | none as-is | Use only to recover CLI affordances for later developer tooling. |
| `yfd-runner/test/` | `source-python/test/` | `src-tauri` unit tests and integration tests | Use as behavior reference for persistence, metrics, and validation edge cases. Rewrite tests in Rust for the new modules. |

## Implementation Changes

### Engine rules
- The Diamond engine must be generic: prompt blocks, pipelines, and document references are first-class.
- No runtime code may assume `chapter`, `worksheet`, `cascade`, `summary`, or fixed step names.
- All domain logic from YFD lives in the bundled `yfd-novel` prompt pack, not in the core engine.

### Template/render rules
- Tera becomes the runtime renderer.
- Document references must resolve from Diamond project documents, not from hardcoded runner state.
- Unresolved document references should warn in the UI and fail execution only when a block requires strict resolution.

### Persistence rules
- `project.json` stores project metadata, variables, prompt blocks, pipeline definitions, and defaults.
- Each run record stores selected block, resolved model preset, rendered prompt snapshot, output text, status, timestamps, and usage metrics.
- Outputs are stored as standalone documents or run artifacts, not as chapter slots in a single run JSON.

### Prompt-pack rules
- The YFD prompt templates ship as an example pack under `prompt-packs/yfd-novel`.
- Their current assumptions about `worksheet`, `plan`, `draft`, and prior chapters remain inside that pack.
- If those templates are meant to execute in Diamond v1, the pack must provide its own schema and context builder.

## Test Plan

- Project persistence:
  - create a new project manifest
  - write and reload without corruption
  - verify atomic-write behavior on interrupted temp-file replacement
- Model preset loading:
  - load bundled YAML presets
  - apply preset overrides
  - reject malformed preset files clearly
- OpenRouter execution:
  - build payload from a selected block and model preset
  - retry on request failure
  - extract text from both string and array-shaped content responses
- Metrics:
  - record prompt/completion tokens and cost
  - aggregate cumulative totals across runs
- Template rendering:
  - render a block with plain variables
  - render a block with document references
  - fail or warn correctly on unresolved document references
- YFD prompt-pack smoke tests:
  - render each migrated `.tera` file with a fixture context
  - confirm no core-engine assumptions leak into runtime outside the pack
- Export:
  - assemble multiple generated documents into an export bundle
  - preserve stable filenames and document order

## Assumptions And Defaults

- Target stack is literal Diamond stack: Tauri + Svelte.
- `workshop-parts/` is a starter area, not just an archive.
- Python files are copied for reference and porting, but the production Diamond engine is implemented in Rust.
- The `yfd-runner` novel workflow is treated as a bundled prompt pack, not as the Diamond core.
- YAML remains the model preset format unless a later decision replaces it.
