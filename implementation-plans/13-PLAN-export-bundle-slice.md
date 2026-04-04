---
created: 2026-04-04 12:25 AM
modified: 2026-04-04 8:44 PM
---
# Plan To Reach The Export Bundle Slice

## Summary

Pipeline authoring is complete. The next narrow MVP slice is export support for existing project artifacts.

This slice should prove the first export workflow without inventing a separate export screen:
- export selected project assets from the current shell into a derived bundle under `exports/`
- preserve file-first project truth by copying selected files rather than moving or rewriting source artifacts
- support the existing asset types the shell already knows how to open: documents, prompt templates, project/config JSON, and persisted run artifacts
- keep exports derived and inspectable on disk from the explorer

This slice does not include zip packaging, external file-picker destinations, import flows, or manuscript assembly.

## Scope

Included in this slice:
- backend command to export selected relative paths into a unique bundle directory under `exports/`
- bundle manifest file describing the export name and included asset paths
- inspector-based selection of open tabs for export
- guards against exporting unsaved editable drafts
- backend and frontend coverage for the export flow

Explicitly out of scope for this slice:
- zip archive generation
- import workflows
- manuscript compilation or bundle post-processing
- exporting unopened assets from the explorer tree

## Verification

- Export bundles must be written under `exports/` and leave source assets untouched.
- Exported bundles must be derived from saved files on disk, not unsaved editor drafts.
- This slice must pass:
  - `npm run typecheck`
  - `npm run lint`
  - `npm run test`
  - `cargo test --manifest-path src-tauri/Cargo.toml`

## Status

Completed.

Delivered in this slice:
- backend export bundle creation under `exports/<bundle-slug>/`
- persisted `export.json` metadata for each bundle
- inspector-driven export selection from currently open tabs
- unsaved-draft guardrails for editable assets
- frontend and backend automated coverage for the export workflow