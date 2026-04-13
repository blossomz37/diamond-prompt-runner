# Local Web App Development Notes

Status: Post-MVP candidate

## Prompt

Question: If I were to copy these files to a new repo:

- all Svelte component logic, UI state, props
- all TypeScript types in `src/lib/types/project.ts`
- the function signatures in `src/lib/tauri.ts` (only implementations change)
- YAML preset files and `project.json` schema, just stored differently

What would be the implementation plan to make a local web app that used browser storage instead of local markdown files, so it could load imports and exports and handle project backups, but not sync automatically with local files?

## Plan

### Implementation Plan: Diamond Prompt Runner as a Local Web App (Browser Storage)

### Overview

This port replaces the Tauri + Rust backend with a pure browser stack. The Svelte components, TypeScript types, and all function signatures in `tauri.ts` transfer directly; only implementations change.

The filesystem becomes IndexedDB + File System Access API (for user-triggered import/export), with `localStorage` for lightweight app state like recent projects and API key state.

## 1. Project Bootstrap

- Create a new Vite + Svelte + TypeScript project (no Tauri).
- Copy `src/lib/types/project.ts` verbatim.
- Copy all Svelte components from `src/lib/components/` and `src/App.svelte`.
- Stub `src/lib/tauri.ts` with the same exported function signatures (implementations replaced in later steps).
- Remove all `@tauri-apps/*` imports; keep the rest of the dependency surface.

## 2. Storage Layer (`src/lib/storage/`)

Define a single storage adapter with two areas.

IndexedDB (using a small wrapper like `idb`):

- `projects` store: keyed by `projectId`; holds full project JSON blob (manifest + serialized asset tree as path -> content entries)
- `runs` store: keyed by `runId`; holds `PromptExecutionResult` records
- `global-variables` store: holds global variables record
- `recent-projects` store: holds `RecentProjectEntry[]`

`localStorage`:

- `openrouter_api_key` (or `sessionStorage` if persistence is undesired)
- last-opened `projectId`

Adapter API should expose typed `get`, `set`, `list`, and `delete` helpers for the bridge layer.

The concept of `rootPath` becomes `projectId`. Relative asset paths remain unchanged (for example, `documents/brief.md`), preserving virtual filesystem shape.

## 3. Bridge Layer: Replacing `tauri.ts`

Reimplement each function in `tauri.ts` against the storage adapter.

### Project lifecycle

Functions:

- `createProject`
- `openProject`
- `getRecentProjects`
- `removeRecentProject`
- `locateRecentProject`
- `renameProject`

Behavior:

- `createProject`: generate UUID, write skeleton project blob, add to recents
- `openProject`: read blob, return `ProjectSummary`
- `getRecentProjects` / `removeRecentProject`: manage recents in IndexedDB
- `locateRecentProject`: no-op or projectId lookup (path relocation no longer applies)

### Asset CRUD

Functions:

- `listProjectAssets`
- `readProjectAsset`
- `writeProjectAsset`
- `deleteDocument`
- `renameDocument`
- `trashPrompt`

Behavior:

- project blob stores flat map: `relativePath -> { content, kind, modifiedAt }`
- `listProjectAssets` builds in-memory `ProjectAssetNode` tree from map
- read/write mutate map and persist blob

### Prompts and pipelines

Functions:

- `createPromptBlock`
- `registerPromptBlock`
- `deletePromptBlock`
- `listProjectPromptBlocks`
- `createPipeline`
- `updatePipeline`
- `duplicatePipeline`
- `deletePipeline`
- `listProjectPipelines`

Behavior:

- state lives in project blob manifest (same shape as `project.json`)
- operations mutate and persist blob

### Model presets

Functions:

- `listModelPresets`
- `createModelPreset`
- `deleteModelPreset`
- `setDefaultModelPreset`
- `setBlockModelPreset`
- `setBlockOutputTarget`
- `setBlockOutputFilename`

Behavior:

- YAML preset files stored in asset map under `models/`
- parse and stringify with `js-yaml`

### Variables

Functions:

- `getGlobalVariables`
- `setGlobalVariables`
- `setProjectVariables`

Behavior:

- global variables in IndexedDB `global-variables`
- project variables embedded in manifest

### Template validation

Function:

- `validateProjectTemplate`

Behavior:

- port Tera validation behavior to JS templating engine such as `nunjucks` or `eta`
- run validation in browser
- note: this is the highest-risk parity area because Tera syntax is not a strict 1:1 match

### Execution

Functions:

- `executePromptBlock`
- `executePipeline`
- `cancelPipeline`
- `onPipelineProgress`

Behavior:

- call OpenRouter directly with browser `fetch`
- verify CORS support first (or plan explicit proxy fallback)
- replace Tauri `listen` channel with event emitter, EventTarget, or Svelte store
- persist runs to IndexedDB `runs` store and link to project

### Credential management

Functions:

- `getExecutionCredentialStatus`
- `saveExecutionApiKey`
- `clearExecutionApiKey`

Behavior:

- store key in `localStorage` or `sessionStorage`
- status values: `stored` or `missing`

### Run history

Functions:

- `listPromptRunHistory`
- `listProjectRunHistory`
- `getProjectUsageSummary`
- `deleteRun`

Behavior:

- query IndexedDB `runs` by `projectId` and optional path
- compute usage summary client-side

### Export bundle

Functions:

- `exportProjectAssets`
- `auditProjectAsset`
- `convertProjectAsset`

Behavior:

- `exportProjectAssets`: build ZIP with `JSZip` and download using `URL.createObjectURL`
- audit and convert logic ports mostly unchanged (classification logic, not filesystem-bound)

### Replace `pickDirectory`

- remove Tauri dialog call
- use `<input type="file" webkitdirectory>` for import workflows
- use File System Access API `showDirectoryPicker` for open-from-disk workflows

## 4. Import / Export Workflows

- Import project from disk: `showDirectoryPicker()` -> read recursively -> build asset map -> store new project blob in IndexedDB
- Export project to disk: serialize project blob to ZIP -> trigger browser download
- Project backup: serialize full project blob to JSON -> download as `.diamond-backup.json`; restore via file input
- Export bundle (`exportProjectAssets`): same ZIP approach scoped to selected paths

## 5. What Changes vs. What Does Not

| Area | Status |
| --- | --- |
| All Svelte components | Copy as-is; minimal structural change expected |
| `src/lib/types/project.ts` | Copy as-is |
| `tauri.ts` function signatures | Keep signatures, rewrite implementations |
| YAML preset structure | Keep format; parse with `js-yaml` |
| `project.json` schema | Keep schema; store in IndexedDB blob |
| Tera template engine | Replace with JS equivalent (`nunjucks` or `eta`) |
| Tauri event system | Replace with Svelte store, EventTarget, or emitter |
| Native keychain | Replace with `localStorage` or `sessionStorage` |
| Filesystem access | Replace with File System Access API + `JSZip` |
| OpenRouter HTTP calls | Move from Rust to browser `fetch` |

## 6. New Dependencies (Additions Only)

- `idb` for typed IndexedDB access
- `js-yaml` for YAML presets
- `jszip` for ZIP export bundles
- `nunjucks` or `eta` for template validation behavior

## 7. Sequencing

1. Bootstrap Vite + Svelte project; copy types/components; stub bridge.
2. Implement storage adapter and IndexedDB schema.
3. Implement project lifecycle bridge functions.
4. Implement asset CRUD bridge functions; confirm explorer rendering works.
5. Implement prompt block and pipeline management.
6. Implement model preset and variable management.
7. Implement import/export workflows.
8. Implement execution bridge (`fetch` to OpenRouter + run persistence).
9. Implement template validation bridge with chosen JS engine.
10. Run end-to-end smoke test: create project -> add block -> run -> view history -> export backup.