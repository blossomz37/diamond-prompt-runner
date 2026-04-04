---
description: "Use when editing Rust backend code in src-tauri/, including Tauri commands, project_store.rs, filesystem access, validation, execution, credential handling, run history, or OpenRouter integration. Covers backend boundaries and test patterns."
name: "Rust Backend"
applyTo: "src-tauri/**"
---
# Rust Backend Guidelines

- Keep filesystem access, credential lookup, run persistence, and provider HTTP calls in Rust under `src-tauri/`.
- Register new command surfaces in `src-tauri/src/main.rs` and expose matching frontend bridge functions in `src/lib/tauri.ts`.
- Prefer extending `src-tauri/src/project_store.rs` for current project, validation, execution, and run-history logic unless a new module is clearly warranted.
- Keep command boundaries narrow and return user-facing `Result<T, String>` values from the Tauri command layer.
- Preserve Diamond’s file-first model: project truth lives in `project.json`, `documents/`, `prompts/`, `models/`, `runs/`, and `exports/`.
- Do not move secrets into project files. App-level credentials belong in native keychain storage, with `OPENROUTER_API_KEY` only as fallback.
- Preserve the current execution contract for this slice: direct Rust HTTP to OpenRouter, strict execution-time handling, and persisted JSON run artifacts under `runs/`.
- Preview validation and execution are intentionally different: warnings may be acceptable in preview, but required execution context should fail fast.
- Add backend tests beside the Rust code in `src-tauri/src/project_store.rs` and use `tempfile` to build disposable project fixtures.
- Validate backend changes with `cargo test --manifest-path src-tauri/Cargo.toml`, plus the relevant frontend checks if the Tauri boundary changed.

See `AGENTS.md` for repo-wide operating rules and `implementation-plans/09-PLAN-single-block-execution-slice.md` for the active backend slice.