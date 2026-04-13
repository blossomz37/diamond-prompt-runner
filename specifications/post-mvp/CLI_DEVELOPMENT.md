# CLI Development Notes

Status: Post-MVP candidate

## Prompt 1

Evaluate how I could run this app via CLI.

## Current CLI Options

1. Full desktop app (development)

```bash
npm install           # first time only
npm run tauri:dev     # launches the Tauri desktop window
```

This compiles the Rust backend (~5-10 min cold, ~15-30 sec incremental) and opens the full GUI. It is the primary intended runtime.

2. Frontend-only dev server (no Rust, no window)

```bash
npm run dev
```

Starts a Vite server at http://localhost:1420. Useful for frontend-only work, but Tauri commands will not work.

3. Headless CLI pipeline runner (no GUI)

The Rust binary has a built-in CLI mode. Once built, you can invoke it directly:

```bash
# Build first (debug)
cargo build --manifest-path src-tauri/Cargo.toml

# Then run headlessly
./src-tauri/target/debug/diamond-runner cli run-pipeline <project-path> <pipeline-id> '<json-payload>'
```

Argument structure (main.rs:614):

```bash
diamond-runner cli run-pipeline <project_path> <pipeline_id> [json_payload]
```

- `cli` and `run-pipeline` are literal keywords.
- `<project_path>` is the path to a Diamond project directory (with `project.json`).
- `<pipeline_id>` is the ID of the pipeline to run.
- `[json_payload]` is an optional JSON object, for example `{"chapter": "3"}`.

Example (using the sample project):

```bash
./src-tauri/target/debug/diamond-runner cli run-pipeline \
  "fixtures/sample-project" \
  "review-pipeline" \
  '{}'
```

There are two convenience scripts wrapping this:

- `scripts/test-headless.mjs`: Node wrapper

```bash
node scripts/test-headless.mjs <project-path> <pipeline-id> '<json>'
```

- `scripts/test-batch-pipeline.sh`: shell loop for running a chapter range

```bash
./scripts/test-batch-pipeline.sh 1 5
```

4. Utility scripts

```bash
npm run probe:online     # Test OpenRouter connectivity (needs OPENROUTER_API_KEY env var)
npm run test             # Run Vitest frontend tests (headless, no window)
npm run typecheck        # TypeScript/Svelte type checking
npm run lint             # ESLint
cargo test --manifest-path src-tauri/Cargo.toml   # Rust backend tests
```

5. Key notes

- OpenRouter API key is required for any pipeline execution. The headless CLI reads it from the OS keychain first, then falls back to the `OPENROUTER_API_KEY` environment variable.
- There is no interactive REPL or pure CLI prompt runner. The CLI mode only supports `run-pipeline`. If you want to run single blocks headlessly, that is not currently exposed as a CLI subcommand (only through the GUI).
- The headless runner writes run artifacts to a temp dir (`/tmp/diamond-runner-headless`) rather than the project's `runs/` folder.

---

## Prompt 2

How could I extend this to run the app completely from CLI and have all files save in the workspace rather than exterior agent directories like `~/tmp/`? Specify tasks related to:

- refactoring
- repurposing
- creating new script

## Plan: Extending Diamond Prompt Runner for Full CLI Operation with Workspace-Local Files

## Context: What Happens Today

The existing CLI mode (`main.rs:612-651`) handles exactly one subcommand, `cli run-pipeline`, and hardcodes `app_data` to:

```rust
std::env::temp_dir().join("diamond-runner-headless")
```

That temp dir is used for three things:

- credential store (`credentials.json`)
- global variables store
- recent-projects list

Run artifacts already write correctly to `<project>/runs/` via `persist_run_record`.

So the files-in-wrong-place problem is specifically about credentials and global variables leaking into `/tmp`. Everything else is already workspace-local.

## 1. Refactoring (Rust backend, `src-tauri/src/`)

### A. Replace the hardcoded temp `app_data_dir` with a workspace-local path

In `main.rs`, replace:

```rust
let app_data = std::env::temp_dir().join("diamond-runner-headless");
```

with a path under the project root (already available as `project_path`).

Decision: use a hidden `.diamond/` subfolder within the project root as the CLI app-data dir. This keeps it workspace-local without polluting top-level project layout.

### B. Extract a `cli_app_data_dir(project_path: &Path) -> PathBuf` helper

Create a helper returning:

```rust
project_path.join(".diamond")
```

Call it from each CLI subcommand. This keeps the convention explicit and testable without affecting the GUI path (which still uses Tauri's platform `app_data_dir`).

### C. Extract a `run_cli(args: &[String])` function from `main()`

Move inline CLI handling in `main()` to:

```rust
fn run_cli(args: &[String]) -> Result<(), String>
```

This makes CLI behavior easier to test and extend.

### D. Expand subcommand handling to a dispatch match

Replace a single hardcoded check such as:

```rust
if args[1] == "cli" && args[2] == "run-pipeline"
```

with:

```rust
match args[2].as_str() {
    // subcommands...
}
```

This is the minimal structural change needed to add more CLI subcommands cleanly.

## 2. Repurposing (existing scripts/assets)

### A. Repurpose `scripts/test-headless.mjs` as a general CLI harness

Current behavior is hardcoded to `run-pipeline`. Repurpose it by:

- accepting subcommand as the first argument
- supporting JSON output mode via `--json`
- standardizing exit codes (`0` success, `1` failure) for CI use

### B. Repurpose `scripts/test-batch-pipeline.sh` as a generic batch runner

Current behavior is hardcoded to `batch-production` and `chapter`. Repurpose it to:

- accept project path and pipeline ID as parameters
- accept variable key/value input
- loop over a numeric range for any variable key
- use `.diamond/` credential path conventions instead of temp-dir assumptions

### C. Repurpose `scripts/probe-openrouter-online.mjs` as pre-flight connectivity check

Keep its online capability probing, but add automation-focused behavior:

- support `--quiet` mode (exit-code only)
- run as a gate before pipeline execution in scripts/CI

## 3. Creating a New Script

### `scripts/diamond-cli.sh`: Unified CLI entry point

Create a thin shell wrapper that:

1. detects release (`src-tauri/target/release/diamond-runner`) or debug (`src-tauri/target/debug/diamond-runner`) binary, preferring release
2. falls back to `cargo run --manifest-path src-tauri/Cargo.toml --` if no binary exists
3. provides a help message for supported subcommands
4. passes all args through unchanged (`"$@"`)

Supported subcommands to wire through (requires Rust refactors above):

| Subcommand | Args | Notes |
| --- | --- | --- |
| `run-pipeline` | `<project> <pipeline-id> [json-payload]` | Already works |
| `run-block` | `<project> <relative-block-path> [json-payload]` | Wire to `execute_prompt_block` |
| `list-pipelines` | `<project>` | Wire to `list_project_pipelines` |
| `list-blocks` | `<project>` | Wire to `list_project_prompt_blocks` |
| `list-runs` | `<project>` | Wire to `list_project_run_history` |
| `set-key` | `<project> <api-key>` | Call `save_execution_api_key` against `.diamond/` |
| `validate` | `<project> <relative-block-path>` | Wire to `validate_project_template` |

Example usage:

```bash
./scripts/diamond-cli.sh run-pipeline ./my-project my-pipeline '{"var": "val"}'
./scripts/diamond-cli.sh set-key ./my-project sk-or-v1-...
./scripts/diamond-cli.sh list-runs ./my-project
```

Output behavior:

- plain text by default
- JSON when `--json` is provided, so output can be piped to `jq` or consumed by scripts

## Key Principle

The GUI runtime path stays unchanged.

Tauri desktop continues using platform-resolved `app_data_dir` for desktop credentials and recents. Only CLI path resolution changes, from `/tmp/diamond-runner-headless` to `<project>/.diamond/`, keeping the two runtime paths fully separate.