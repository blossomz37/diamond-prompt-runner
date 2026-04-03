---
created: 2026-04-03 11:58 AM
modified: 2026-04-03 1:38 PM
---
# Rewrite `SPEC_DIAMOND_RUNNER_v1.md` As Diamond Runner v2

## Filename Changes: 
- `SPEC_DIAMOND_RUNNER_v1.md` = `spec/SPEC_DIAMOND_RUNNER_v0.md`
- `SPEC_DIAMOND_RUNNER_v2.md`= `spec/SPEC_DIAMOND_RUNNER_v1.md`

## Summary

Replace the current file with a full v2 implementation spec for the new Diamond workspace. The rewrite should be a complete replacement, not an edit pass over the current wording.

The new document should define Diamond as:
- a Tauri + Svelte desktop app
- a generic local-first prompt runner with projects, documents, variables, prompt blocks, pipelines, runs, and exports
- a core engine that is workflow-agnostic
- optionally bundled with domain-specific prompt packs such as a YFD novel pack, but not architecturally shaped around them

Keep the same file path for now, but change the internal title/versioning to v2 and remove wording that assumes this is still an early mockup brief.

Approved prototype direction:
- `Diamond Runner v2 - IDE Pass`
- shared IDE shell with explorer, tabs, editor, inspector, and bottom panel

## Required Rewrite Structure

### 1. Product definition
Open with a short authoritative definition of Diamond Runner:
- local-first desktop workspace
- Tauri backend, Svelte frontend
- Tera-based prompt templating
- OpenRouter as the only model provider in v2
- dense IDE-like UI direction with minimal padding, subtle radii, and a near-black navy or royal-purple default palette
- project-based organization on disk

This section should replace the current loose bullet list and remove all references that imply the product is still just being handed to an AI for mockups.

### 2. Core concepts
Add a section that defines the runtime nouns clearly:
- Project
- Document
- Variable
- Document Reference
- Prompt Block
- Pipeline
- Run
- Model Preset
- Export Bundle
- Prompt Pack

This is the most important addition. The current spec gestures at these ideas but never pins them down.

### 3. MVP scope
Define v2 MVP as:
- project creation/opening
- document editing
- template/prompt block editing
- Tera rendering with variables and document references
- manual single-block execution
- linear pipeline execution
- local run history
- OpenRouter execution with preset selection
- import/export of markdown, Tera templates, and workflow/config JSON
- export of project artifacts

Explicitly defer:
- node graph editor
- WYSIWYG editor
- advanced OpenRouter feature toggles
- multi-run live concurrency UI
- toast/polish items unless you want them as non-blocking UX details

This section should resolve the current contradiction where some features are both “in” and “deferred.”

### 4. Architecture and storage
Add an implementation-grade section that defines on-disk structure and ownership:

- `project.json`
- `documents/`
- `prompts/`
- `models/`
- `runs/`
- `exports/`

The spec should describe behavior, not code symbols, but it must be concrete enough that an implementer can create the filesystem contract without guessing.

Required interface definitions:
- `project.json` contains project metadata, defaults, variables, prompt blocks, pipeline definitions, and UI preferences
- documents are first-class files, not hidden fields inside runner state
- each run stores rendered prompt snapshot, selected preset, output text, timestamps, status, and usage metrics
- exports are derived artifacts, not the source of truth

### 5. Rendering and execution model
Add a section that defines exactly how prompts run:
- prompt blocks render through Tera
- variables and document references are resolved before execution
- unresolved references produce warnings and block execution only when strict mode is enabled
- manual execution runs one prompt block
- pipeline execution runs a linear ordered series of blocks
- outputs can be stored as run artifacts and optionally written back into project documents when configured

Do not define graph execution as part of MVP behavior. Mention it only as future expansion.

### 6. OpenRouter and model presets
Replace the current vague AI section with concrete MVP behavior:
- OpenRouter only
- model presets stored locally
- per-project default preset
- per-block preset override
- retries, timeout handling, and usage/cost tracking
- no advanced feature toggles in MVP beyond what fits naturally into preset configuration

This section should be written so the current `yfd-runner` OpenRouter pieces can be ported into Diamond without changing product intent.

### 7. UI surfaces
Keep a concise screen list, not a full mockup screenplay.

Required MVP screens:
- Project Browser
- Project Workspace
- Document Editor
- Prompt Block Editor
- Pipeline Runner
- Run History / Output Inspector
- Settings / Model Presets

For each screen, specify purpose and key controls in 2-4 short bullets. Do not include layout prose longer than needed to prevent ambiguity.

The approved shell direction should also be made explicit:
- the primary surface is an IDE-like shell, not a dashboard-first workspace
- a shared explorer, tabs, center editor, right inspector, and bottom panel should anchor the UI
- documents, prompts, workflows, presets, runs, and exports are asset types within one workspace
- preview and warnings should default to toggleable panes rather than always-visible split cards
- Settings should include version visibility and a `Check for Updates` action

### 8. Prompt packs
Add a section explaining that Diamond Core is generic and prompt packs are optional layered content.

Specify:
- a prompt pack can bundle templates, starter documents, variables, pipelines, and model defaults
- YFD can later exist as a bundled or imported pack
- prompt packs must not introduce core-engine assumptions

This is the clean place to absorb useful `yfd-runner` assets without making Diamond itself a novel-writing app.

### 9. Non-goals and future work
Close with a short section listing explicitly deferred items:
- graph editor
- WYSIWYG markdown hybrid editor
- advanced OpenRouter controls UI
- collaborative/multi-user features
- remote execution
- branch/merge semantics if not needed in MVP

## Public Interfaces And Types The Spec Must Define

The rewrite should explicitly define these interfaces in product terms:

- Project manifest:
  - project id
  - project name
  - created/updated timestamps
  - default model preset
  - variables
  - prompt blocks
  - pipelines
  - UI preferences
- Prompt block:
  - block id
  - name
  - template/document source
  - input bindings
  - model preset selection
  - output target
- Pipeline:
  - pipeline id
  - ordered block list
  - execution mode: manual or sequential
- Run record:
  - run id
  - block or pipeline id
  - rendered prompt snapshot
  - resolved preset
  - output
  - status
  - timestamps
  - usage metrics
- Document reference behavior:
  - how `{{ some_document.md }}` or equivalent references resolve
  - what happens on missing documents
  - whether strict mode is on by default

Defaults to lock in the spec:
- Tera is the template engine
- strict resolution is enabled by default for execution, with warnings visible before run
- YAML model presets are supported in MVP
- runs are stored locally per project
- outputs are plain text or markdown by default
- the UI may show short model labels while preserving full provider IDs in config/runtime
- the approved visual direction is dense, low-padding, subtle-radius, and near-black navy or royal purple

## Acceptance Checks For The Rewrite

The rewritten v2 spec is complete only if it satisfies all of these:

- A new engineer could scaffold the desktop app and filesystem model without inventing missing nouns.
- The document no longer mentions Tauri/Svelte/Tera as speculative or optional.
- The MVP list is internally consistent and does not contradict the deferred list.
- The document makes Diamond Core generic and moves YFD-like workflows into prompt packs or examples.
- The UI section is concise but enough to derive the initial app shell.
- The execution section clearly distinguishes single-block runs from linear pipeline runs.
- The storage section clearly defines where project truth lives on disk.
- The spec no longer reads like a mockup handoff note.

## Assumptions And Defaults

- Keep the file path as `SPEC_DIAMOND_RUNNER_v1.md`, but rewrite the title/version in the content to “Diamond Prompt Runner v2” or “Diamond Runner v2”.
- Target audience is implementers, not just mockup generators.
- Diamond Core Only is the scope; YFD remains an example prompt pack, not the architecture.
- Keep a concise prioritized screen list in the document.
- No code or file edits are performed in this step; this is the rewrite spec for the document itself.
