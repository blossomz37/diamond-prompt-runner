# Scripts

Utility scripts for development, testing, release automation, and quality assurance in Diamond Prompt Runner.

## Overview

| Script | Type | Purpose |
|--------|------|---------|
| `generate-updater-json.mjs` | Node.js | Generate Tauri updater metadata for releases |
| `probe-openrouter-online.mjs` | Node.js | Test OpenRouter online/web-search capabilities |
| `tauri-before-dev.mjs` | Node.js | Pre-dev setup: ensure Vite dev server is ready |
| `test-batch-pipeline.sh` | Bash | Batch test pipeline execution across multiple runs |
| `test-headless.mjs` | Node.js | Headless CLI testing of pipeline execution |
| `wcag-audit.mjs` | Node.js | WCAG 2.1 accessibility contrast ratio audit |

---

## Scripts

### `generate-updater-json.mjs`

**Purpose:** Generate `latest.json` updater metadata for the Tauri auto-updater mechanism.

**Usage:**
```bash
npm run updater:json
```

**What it does:**
- Reads the app signature (`.app.tar.gz.sig`) from the macOS build bundle
- Extracts version from `tauri.conf.json`
- Constructs a `latest.json` file with version, signature, download URL, and publish date
- Outputs to `src-tauri/target/release/bundle/latest.json`

**Prerequisites:**
- Must run after `cargo build --release` bundles have been created
- Requires a signed macOS `.app.tar.gz` with a `.sig` file
- Uses the Tauri signing key to produce valid signatures

**Environment:**
- Reads `src-tauri/tauri.conf.json` for version and product name
- Reads signature from `src-tauri/target/release/bundle/macos/`
- Publishes to GitHub releases at `https://github.com/{user}/diamond-prompt-runner/releases/`

**Related:** Release workflows, Tauri auto-updater configuration

---

### `probe-openrouter-online.mjs`

**Purpose:** Test and validate OpenRouter's online/web-search capabilities.

**Usage:**
```bash
# Default test
node scripts/probe-openrouter-online.mjs

# With custom prompt
node scripts/probe-openrouter-online.mjs "What recent AI news should I know about?"

# With environment overrides
OPENROUTER_ONLINE_MODEL="openai/gpt-5:online" \
OPENROUTER_WEB_MAX_RESULTS="5" \
OPENROUTER_SEARCH_CONTEXT_SIZE="large" \
node scripts/probe-openrouter-online.mjs
```

**What it does:**
- Sends a prompt to OpenRouter with web-search plugins enabled
- Validates that the model supports online research (`:online` variant)
- Checks for web search requests and citations in the response
- Reports model, web search request count, and citation count

**Environment variables:**
- `OPENROUTER_API_KEY` (required) — OpenRouter API key
- `OPENROUTER_ONLINE_MODEL` (default: `openai/gpt-5.4:online`) — Model to test
- `OPENROUTER_WEB_MAX_RESULTS` (default: `3`) — Number of web results per search
- `OPENROUTER_SEARCH_CONTEXT_SIZE` (default: `medium`) — Context size: `small`, `medium`, `large`
- `OPENROUTER_INCLUDE_DOMAINS` (optional) — Comma-separated domains to prioritize

**Exit codes:**
- `0` — Success; web search evidence found
- `1` — Missing API key, empty response, or no web search evidence

**Use cases:**
- Validate OpenRouter credentials and online feature access
- Test web-search configuration before production runs
- Verify citation tracking in response metadata

**Related:** Online research feature, prompt execution with web search

---

### `tauri-before-dev.mjs`

**Purpose:** Pre-development setup hook that ensures the Vite dev server is running and ready before Tauri connects to it.

**Usage:**
```bash
# Typically run as part of the build script
npm run tauri:dev
# (calls `tauri-before-dev.mjs` via tauri.conf.json before launch)
```

**What it does:**
- Checks if the Vite dev server is already running on the specified port (default: 1420)
- If not running, spawns a new Vite dev server with `npm run dev`
- Waits for the server to be HTTP-ready with exponential backoff (up to 60 attempts, 500ms apart)
- If port is busy and unreachable, exits with a helpful error message
- Avoids spawning duplicate dev servers on subsequent runs

**Environment:**
- `TAURI_DEV_URL` (default: `http://localhost:1420`) — Dev server URL

**Why it exists:**
- `npm run tauri:dev` reuses an existing Vite dev server instead of launching a duplicate
- Improves dev workflow by avoiding "port in use" errors when rebuilding the Tauri binary
- Gracefully handles cold starts and server recovery

**Related:** Vite configuration, Tauri development workflow

---

### `test-batch-pipeline.sh`

**Purpose:** Batch-execute a headless pipeline across multiple runs with variable payloads.

**Usage:**
```bash
# Run chapters 1–5 of the batch-production pipeline
./scripts/test-batch-pipeline.sh 1 5

# Run only chapter 3
./scripts/test-batch-pipeline.sh 3 3

# Default: chapter 1
./scripts/test-batch-pipeline.sh
```

**What it does:**
- Iterates through a range of chapters (start to end, inclusive)
- For each iteration, invokes the headless CLI with a payload containing the chapter number
- Passes the payload as JSON: `{"chapter": "N"}`
- Halts the batch if any chapter fails
- Reports completion status

**Parameters:**
1. `START_CH` (default: 1) — First chapter number to process
2. `END_CH` (default: 1) — Last chapter number to process

**Example execution:**
```bash
# Produces chapters 1–3 in the "Neon & Nightmares" project
./scripts/test-batch-pipeline.sh 1 3
```

**Output:** For each chapter:
```
>>> PRODUCING CHAPTER 1 <<<
[CLI output from headless run]
```

**Exit codes:**
- `0` — All chapters completed successfully
- `1` — Pipeline failed; batch halted

**Prerequisites:**
- Rust binary must be compiled: `cargo build --release`
- Project and pipeline must exist and be valid

**Related:** Headless execution, batch processing, pipeline payloads

---

### `test-sample-project-2-batch.sh`

**Purpose:** Execute `docs/sample-projects/sample-project-2` five times and write `flash-fiction-NN.md` outputs into that sample project's `documents/` folder.

**Usage:**
```bash
# Default: runs counts 01-05
./scripts/test-sample-project-2-batch.sh

# Custom range
./scripts/test-sample-project-2-batch.sh 2 4
```

**What it does:**
- Invokes the headless CLI against `docs/sample-projects/sample-project-2`
- Runs the `flash-fiction-batch` pipeline once per loop
- Passes a padded `count` payload (`01`, `02`, `03`, `04`, `05`)
- Causes the prompt block to write:
  - `documents/flash-fiction-01.md`
  - `documents/flash-fiction-02.md`
  - `documents/flash-fiction-03.md`
  - `documents/flash-fiction-04.md`
  - `documents/flash-fiction-05.md`

**Parameters:**
1. `START_COUNT` (default: `1`)
2. `END_COUNT` (default: `5`)

**Prerequisites:**
- Rust binary must be compilable locally
- OpenRouter API key must be available in the app keychain or via `OPENROUTER_API_KEY`
- The batch script auto-loads `../.env` if present

**Related:** Headless execution, payload-driven output filenames, sample project validation

---

### `test-headless.mjs`

**Purpose:** Execute a pipeline in headless (CLI) mode without the desktop UI, useful for testing and automation.

**Usage:**
```bash
# Default: run batch-production pipeline in Neon & Nightmares with chapter 3
node scripts/test-headless.mjs

# Custom project, pipeline, and payload
node scripts/test-headless.mjs "path/to/project" "pipeline-id" '{"chapter": "7", "tone": "dark"}'
```

**What it does:**
- Spawns the compiled Rust binary with CLI arguments
- Passes project path, pipeline ID, and a JSON payload as arguments
- Streams backend output to stdout with color-coded prefixes
- Reports success or failure with appropriate exit code

**Parameters:**
1. `projectPath` (default: `"Sample Projects/Neon & Nightmares"`) — Path to Diamond project
2. `pipelineId` (default: `"batch-production"`) — Pipeline ID to execute
3. `payload` (default: `{"chapter": "3"}`) — JSON object with runtime variables

**Example:**
```bash
node scripts/test-headless.mjs \
  "my-project" \
  "story-pipeline" \
  '{"chapter": "5", "setting": "dystopian"}'
```

**Output:**
```
Starting headless test for pipeline 'batch-production' in 'Sample Projects/Neon & Nightmares'...
[Backend] [CLI output]
Success: Headless pipeline execution completed.
```

**Exit codes:**
- `0` — Pipeline execution succeeded
- Non-zero — Pipeline execution failed (reported in console)

**Prerequisites:**
- Rust binary must be compiled: `npm run tauri:dev` or `cargo build --debug`
- Project must exist at the specified path
- `test-headless.mjs` auto-loads `../.env` if present before reading `OPENROUTER_API_KEY`

---

### `normalize-markdown-boundaries.mjs`

**Purpose:** Convert a heading-based markdown document into nested bracket-boundary sections that are easier to target with `doc | extract_section`.

**Usage:**
```bash
node scripts/normalize-markdown-boundaries.mjs input.md
node scripts/normalize-markdown-boundaries.mjs input.md output.md
```

**What it does:**
- Reads markdown headings (`#` through `######`)
- Normalizes each heading into a tag like `[act_1]`
- Wraps each heading block in matching closing tags like `[/act_1]`
- Preserves the original body content under each heading

**Example:**
```bash
node scripts/normalize-markdown-boundaries.mjs \
  docs/sample-projects/sample-project-3/documents/Story_Dossier_Worksheet.md \
  docs/sample-projects/sample-project-3/documents/Story_Dossier_Worksheet.boundaries.md
```

**Related:** CLI interface, pipeline execution, batch processing, test-batch-pipeline.sh

---

### `wcag-audit.mjs`

**Purpose:** Audit the app's CSS color contrast ratios against WCAG 2.1 Level AA and AAA standards.

**Usage:**
```bash
node scripts/wcag-audit.mjs
```

**What it does:**
- Calculates luminance using the standard sRGB to linear color space conversion
- Computes contrast ratios between foreground and background colors
- Grades each color pair against WCAG 2.1 criteria for normal and large text
- Audits all dark theme color combinations
- Identifies failing (red ✗), passing AA (yellow ~), and passing AAA (green ✓) pairs

**Output:**
```
=== DARK THEME ===

  ✓ --text on base               11.24:1  AAA
  ✓ --text on panel              10.58:1  AAA
  ~ --text-dim on base            5.12:1  AA
  ✗ --text-soft on base           3.87:1  FAIL
  ...
```

**Contrast thresholds:**
- **AAA (normal text):** ≥ 7.0:1
- **AA (normal text):** ≥ 4.5:1
- **AA (large text ≥18px or 14px bold):** ≥ 3.0:1
- **FAIL:** < minimum threshold

**Color formats supported:**
- Hex: `#f3f5ff`
- RGB: `rgb(243, 245, 255)`
- RGBA: `rgba(243, 245, 255, 0.82)`

**How it works:**
- Flattens semi-transparent colors onto known opaque backgrounds
- Uses the WCAG 2.1 relative luminance formula
- Computes contrast as `(lighter + 0.05) / (darker + 0.05)`

**Integration:**
- Audit pairs defined in the script can be updated to reflect your design system
- Add new color pairs as you introduce new semantic colors

**Related:** Accessibility, WCAG compliance, CSS color system, dark theme

---

## Quick Reference

### Running scripts

```bash
# Auto-updater
npm run updater:json

# Online research validate
OPENROUTER_API_KEY="sk-..." node scripts/probe-openrouter-online.mjs

# Headless pipeline
node scripts/test-headless.mjs "project-name" "pipeline-id" '{"var": "value"}'

# Batch pipeline
./scripts/test-batch-pipeline.sh 1 10

# Accessibility audit
node scripts/wcag-audit.mjs
```

### Environment setup

```bash
# Set OpenRouter API key for online tests
export OPENROUTER_API_KEY="sk-or-v1-..."

# Set Tauri dev URL (optional, defaults to localhost:1420)
export TAURI_DEV_URL="http://localhost:1420"
```

---

## Maintenance

- **Node scripts:** Run with `node` (no package.json script aliases)
- **Bash scripts:** Make executable: `chmod +x scripts/*.sh`
- **Rust CLI:** Ensure the binary is compiled in debug or release mode before running headless tests
- **Color audit:** Update `wcag-audit.mjs` when adding new CSS custom properties

---

## Dev Utilities

The `scripts/dev/` folder contains one-off or exploratory helper scripts that are not part of normal day-to-day workflows.

- `scripts/dev/migrate.rs` — ad hoc Rust migration helper for extracting inline pipeline entries into per-file pipeline JSON files.
- `scripts/dev/test-marked.js` — tiny smoke test for local markdown parsing behavior with `marked`.

These are intentionally kept out of the repository root to reduce confusion for new contributors.
