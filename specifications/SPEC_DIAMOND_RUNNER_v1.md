---
title: Diamond Prompt Runner v2
created: 2024-04-03 10:48 AM
modified: 2026-04-03 1:38 PM
status: draft-v2
---

# Diamond Prompt Runner v2

## 1. Product Definition

Diamond Prompt Runner is a local-first desktop workspace for authoring, rendering, executing, and organizing AI prompt workflows on disk.

v2 is defined as:

- a Tauri desktop app with a Svelte frontend
- a generic prompt runner rather than a single-purpose writing tool
- a project-based workspace that stores documents, prompt blocks, pipelines, runs, model presets, and exports locally
- a Tera-based prompt templating system
- an OpenRouter-only execution backend for v2
- a dense, IDE-like interface with minimal padding, subtle radii, and a near-black navy or royal-purple default theme

Diamond Core is workflow-agnostic. Domain-specific workflows such as a novel-writing system may be shipped later as prompt packs, but they must not define the architecture of the core product.

## 2. Core Concepts

### Project

A project is the top-level local workspace. It owns project metadata, defaults, documents, prompt blocks, pipelines, run history, model presets, and exports.

### Document

A document is a first-class project file, typically stored as markdown or plain text. Documents may be edited directly by the user, referenced by prompt blocks, generated from run output, or included in exports.

### Variable

A variable is a named project value available during template rendering. Variables are scalar values or short structured values stored in the project manifest. Variables are not hidden documents.

### Document Reference

A document reference is a template-time instruction that resolves a project document into rendered prompt content.

v2 default document reference form:

```tera
{{ doc("Genre_Conventions.md") }}
```

Rules:

- the path is project-relative to `documents/`
- the referenced document content is inserted as text during rendering
- missing references are surfaced as warnings in editors and previews
- execution runs in strict mode by default, so unresolved references block execution

### Prompt Block

A prompt block is a named executable unit that renders a template with project context and optional block-specific inputs, then sends the rendered prompt to OpenRouter.

Prompt blocks are the fundamental execution unit in Diamond.

### Pipeline

A pipeline is an ordered list of prompt blocks executed sequentially. Each block receives project context plus any declared upstream outputs.

v2 pipelines are linear only. Graph execution is deferred.

### Run

A run is a persisted execution record for either a single prompt block or a pipeline. A run stores the rendered prompt snapshot, resolved model preset, status, output, timestamps, and usage metrics.

### Model Preset

A model preset is a locally stored YAML configuration that describes an OpenRouter model and its default execution options.

### Export Bundle

An export bundle is a derived artifact produced from project documents, generated outputs, or selected run artifacts. Exports are not the source of truth.

### Prompt Pack

A prompt pack is an optional content bundle layered on top of Diamond Core. It may include templates, starter documents, variables, pipelines, and model defaults for a specific use case.

Prompt packs may accelerate setup, but they must not add engine assumptions that only apply to one domain.

## 3. MVP Scope

### Included In v2 MVP

- create, open, rename, and browse local projects
- edit project documents
- create and edit prompt blocks
- create and edit linear pipelines
- render Tera templates with variables and document references
- run a single prompt block manually
- run a pipeline sequentially
- store local run history per project
- select a project default model preset
- override a model preset per prompt block
- track retries, execution status, tokens, cost, and timestamps
- import and export markdown project documents
- import and export Tera templates
- import and export workflow and AI config JSON artifacts
- export selected project artifacts as files or a bundle

### Early Implementation Sequencing Note

The first implementation milestone is narrower than the full MVP. It is a read-and-navigate shell:

- create a new local project
- open an existing local project
- browse project assets in the explorer
- open supported assets in read-only tabs
- inspect metadata in a sidebar

Editing, template validation, rendering, OpenRouter execution, and run history UX land in later slices.

### Explicitly Deferred

- node graph editor
- WYSIWYG or hybrid markdown editor
- advanced OpenRouter controls UI for structured output, online mode, healing, and similar provider-specific toggles
- multi-run live concurrency UI
- collaborative or multi-user features
- remote execution
- branch and merge semantics
- notification polish such as toast systems as a required MVP dependency

### Post-MVP Extensions

Features that are implemented and functional but are scoped as extensions beyond the MVP boundary:

- **Online Research (`:online`)** — Prompts whose first non-empty line is `{# diamond:online #}` opt into web-backed execution via OpenRouter's online feature. This is a post-MVP extension: it requires no architectural changes to the core, adds no new project data model fields, and is fully backward-compatible. Online run metadata (`enabled`, `web_search_requests`, `citation_count`) is persisted in run artifacts with serde defaults so existing artifacts are unaffected. The feature is documented in the ValidationPanel inline when the directive is detected.

## 4. Public Interfaces And Project Data Model

### 4.1 Project Manifest

Each project is defined by a `project.json` manifest.

Required project manifest fields:

- `project_id`: stable machine id
- `project_name`: human-readable name
- `created_at`
- `updated_at`
- `default_model_preset`
- `variables`
- `prompt_blocks`
- `pipelines`
- `ui_preferences`

Recommended additional fields:

- `description`
- `default_output_format`
- `installed_prompt_packs`
- `tags`

### 4.2 Prompt Block

Each prompt block definition must include:

- `block_id`
- `name`
- `template_source`
- `input_bindings`
- `model_preset`
- `output_target`

Field meanings:

- `template_source`: the file or inline template used for rendering
- `input_bindings`: declared inputs pulled from variables, documents, or upstream block outputs
- `model_preset`: block-specific override; falls back to the project default when omitted
- `output_target`: run artifact only, project document writeback, or both

### 4.3 Pipeline

Each pipeline definition must include:

- `pipeline_id`
- `name`
- `ordered_blocks`
- `execution_mode`

v2 allowed execution modes:

- `manual`
- `sequential`

`sequential` means the pipeline executes blocks in listed order, one block at a time, without graph branching.

### 4.4 Run Record

Each run record must include:

- `run_id`
- `target_type`: `block` or `pipeline`
- `target_id`
- `rendered_prompt_snapshot`
- `resolved_model_preset`
- `output`
- `status`
- `started_at`
- `completed_at`
- `usage_metrics`

Recommended status values:

- `pending`
- `running`
- `completed`
- `failed`
- `cancelled`

### 4.5 Usage Metrics

Each run record must persist:

- prompt tokens
- completion tokens
- estimated or provider-reported cost
- output word count
- retry count
- error summary when applicable

## 5. Architecture And Storage

Diamond projects are stored locally on disk as folders. The following layout is the v2 contract:

```text
MyProject/
├── project.json
├── documents/
├── prompts/
├── models/
├── runs/
└── exports/
```

### `project.json`

Source of truth for project metadata, defaults, variables, prompt block definitions, pipeline definitions, installed prompt packs, and UI preferences.

### `documents/`

Contains editable user-facing project documents. These are first-class files and may be authored directly, generated by prompt blocks, or referenced during rendering.

Markdown import and export are required MVP behaviors for document files.

### `prompts/`

Contains prompt templates or block-owned source files. Inline prompt blocks are allowed, but file-backed prompt sources are preferred for reuse and versionability.

Tera template import and export are required MVP behaviors for prompt source files.

### `models/`

Contains YAML model presets available to the project. Projects may inherit bundled presets and also add project-local presets.

Projects may also export machine-readable AI config JSON derived from local preset and workflow state when the user needs to hand configuration to other tools.

### `runs/`

Contains persisted run records and any attached run artifacts. Run records are append-only history objects and are not the primary source of project truth.

### `exports/`

Contains generated export bundles or derived compiled outputs. Exports may be deleted and regenerated without data loss.

The export surface must support markdown documents, Tera templates, workflow JSON, preset/config JSON, and selected run artifacts.

## 6. Rendering And Execution Model

### 6.1 Rendering

Diamond v2 uses Tera for prompt rendering.

Rendering inputs may come from:

- project variables
- project documents
- explicit block input bindings
- upstream pipeline outputs when declared

Rendering behavior:

- previews surface unresolved references as warnings
- execution uses strict resolution by default
- strict resolution blocks execution on missing required variables or documents
- rendered prompt snapshots are saved into the run record before the model call

### 6.2 Manual Single-Block Execution

Manual execution runs one prompt block against the current project context.

The system must:

1. resolve inputs
2. render the prompt
3. save a run record in `pending` or `running` state
4. execute against OpenRouter
5. persist output, metrics, and final status
6. optionally write the output back to a target document when configured

### 6.3 Pipeline Execution

Pipeline execution runs a linear ordered series of prompt blocks.

Rules:

- blocks execute sequentially in listed order
- each block may consume earlier block outputs only through declared bindings
- pipeline failure stops further execution unless a future non-MVP recovery mode is added
- each block execution in a pipeline is still persisted as part of run history

### 6.4 Output Behavior

Default output formats in v2 are plain text or markdown.

Each prompt block may store output as:

- run artifact only
- document writeback only
- both run artifact and document writeback

Document writeback must be explicitly configured. Run artifact persistence is always on.

Users must also be able to exchange project assets as files without executing them, including markdown documents, Tera templates, workflow JSON, and AI config JSON.

## 7. OpenRouter And Model Presets

Diamond v2 uses OpenRouter as its only model provider.

### MVP behavior

- model presets are stored locally as YAML
- each project declares one default model preset
- each prompt block may override the project default
- the execution layer records retries, timeout failures, provider errors, and usage metrics
- provider-specific advanced toggles are not a required part of the MVP UI
- the UI may display short model labels such as `claude-3.7-sonnet` while preserving the full provider-backed model identifier in config and runtime state

Implementation sequencing note:

- the first shell milestone does not include provider calls, API-key storage, or SDK wiring
- when the execution slice begins, the official OpenRouter TypeScript SDK should be evaluated first for any TypeScript-owned integration surface without forcing milestone-1 architecture changes

### Model preset rules

Each preset must support:

- `model`
- `temperature`
- `max_completion_tokens`
- optional reasoning settings
- optional provider-supported parameters that can be passed through safely

Preset precedence:

1. block-level preset override
2. project default preset
3. bundled app fallback preset

## 8. UI Surfaces

The v2 MVP must ship with these screens.

UI direction for all screens:

- the primary interaction model is an IDE-style shell rather than a dashboard-first app
- the shell is built around a left explorer, top tabs, central editor, right inspector, and bottom utility panel
- documents, prompt templates, workflows, presets, runs, and exports are asset types inside one shared workspace
- controls should use short labels, minimal padding, subtle corners, and very little explanatory copy in the main surface
- previews, warnings, logs, and output panes should be toggleable panels rather than permanent side-by-side cards unless the user explicitly opens a split view

### 8.1 Project Browser

Purpose:

- create and open local projects
- display recent projects and metadata

Key controls:

- new project
- open existing project
- rename or archive project entry
- inspect compact project metadata such as file count, prompt count, and default model label

### 8.2 Project Workspace

Purpose:

- serve as the main project shell
- provide one shared environment for browsing and editing documents, prompt templates, workflows, presets, runs, and exports

Key controls:

- project explorer and quick-open navigation
- tabbed editing across multiple open assets
- inspector sidebar for metadata, bindings, presets, and targets
- bottom panel for preview, warnings, logs, runs, and problems
- quick-run actions for selected block or pipeline

### 8.3 Document Editor

Purpose:

- edit markdown or text documents stored in the project
- preview document content used by prompt blocks when requested

Key controls:

- save
- rename
- duplicate
- import markdown
- export markdown
- open linked references
- toggle preview or problems pane

### 8.4 Prompt Block Editor

Purpose:

- create and edit prompt blocks
- preview rendered prompts before execution when requested

Key controls:

- edit template source
- bind variables and documents
- select model preset
- import Tera template
- export Tera template
- toggle preview, warnings, and last-run panes
- run block

### 8.5 Pipeline Runner

Purpose:

- define and execute ordered linear workflows
- inspect block order and pipeline status

Key controls:

- reorder blocks
- import workflow JSON
- export workflow JSON
- run pipeline
- stop or cancel active pipeline

### 8.6 Run History / Output Inspector

Purpose:

- inspect prior block and pipeline runs
- compare rendered prompt snapshots with outputs

Key controls:

- filter by block or pipeline
- inspect metrics and errors
- export selected run artifacts
- promote or copy output into a document when applicable

### 8.7 Settings / Model Presets

Purpose:

- manage project defaults and available model presets
- configure execution behavior visible in MVP

Key controls:

- set project default preset
- add or edit local preset YAML
- inspect retry and timeout settings
- display app version
- check for updates
- import or export preset/config JSON

## 9. Prompt Packs

Prompt packs are optional layered content bundles on top of Diamond Core.

A prompt pack may include:

- prompt templates
- starter documents
- variables
- pipelines
- default model presets
- example outputs

Rules:

- prompt packs may accelerate project setup but must not redefine core engine behavior
- prompt packs must use the same project, block, pipeline, and run interfaces as Diamond Core
- prompt packs may be bundled with the app or imported into a project

YFD is explicitly a candidate prompt pack, not the Diamond architecture. Useful assets from a prior writing runner may be repackaged into a `yfd-novel` prompt pack later.

## 10. Non-Goals And Future Work

The following items are out of v2 MVP scope:

- visual node graph editor
- WYSIWYG markdown editor
- advanced OpenRouter controls UI
- collaborative and multi-user features
- remote execution services
- branch and merge workflows
- non-linear pipeline graphs
- live orchestration dashboards for many simultaneous active runs

These may be explored in later versions once the local project model, rendering model, and linear execution model are stable.

## 11. Acceptance Checks

The v2 spec is complete only if all of the following are true:

- a new engineer can scaffold the Tauri app, Svelte shell, and on-disk project layout without inventing missing nouns
- the document treats Tauri, Svelte, Tera, and OpenRouter as concrete product choices for v2
- the MVP feature list does not contradict the deferred list
- Diamond Core is generic and does not assume a single content domain
- prompt packs are clearly separated from core engine behavior
- the execution model clearly distinguishes single-block runs from linear pipeline runs
- the storage model clearly defines where project truth lives on disk
- the document reads as an implementation spec rather than a mockup handoff note

## 12. Defaults And Assumptions

Locked defaults for v2:

- Tera is the template engine
- strict resolution is enabled by default for execution
- preview surfaces warnings before execution
- the default shell is IDE-like, dense, low-padding, and uses subtle radii
- the default palette is near-black navy or royal purple rather than warm brown
- YAML model presets are supported in MVP
- the UI may show short model labels while preserving full provider identifiers in config files
- runs are stored locally per project
- outputs default to plain text or markdown
- preview, warnings, and output panes are toggleable instead of permanently split by default
- Diamond Core is the product scope of this spec
- YFD-like workflows belong in prompt packs, not in the core architecture

This document is intended for implementers. Mockups may be derived from it later, but the document itself is the source of product and architecture intent for Diamond Prompt Runner v2.
