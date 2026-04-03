# Milestone 1 Manual Test Reference

Use this document to run and manually verify the milestone-1 read-and-navigate shell.

This is a runtime check for the desktop app. It complements, but does not replace, the automated checks.

## 1. Prerequisites

From the repo root:

```bash
cd /Users/carlo/diamond-runner
```

If dependencies are not installed yet:

```bash
npm install
```

## 2. Automated Baseline

Run these first. If any of them fail, stop and fix that before manual UI testing.

```bash
npm run typecheck
npm run lint
npm run test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected result:
- all commands exit successfully
- frontend tests pass
- Rust project-store tests pass

## 3. Launch The Desktop App

Run the desktop shell:

```bash
npm run tauri:dev
```

Expected result:
- the Tauri desktop window opens
- the app starts on the Project Browser screen
- you see controls for `Create Project` and `Open Existing Project`

## 4. Manual Scenarios

### Scenario A: Open the committed sample project

Use `Open Existing Project` and select:

```text
/Users/carlo/diamond-runner/fixtures/sample-project
```

Verify:
- the workspace opens successfully
- the explorer shows `project.json`, `documents/`, `prompts/`, `models/`, `runs/`, and `exports/`
- `runs/` and `exports/` appear even though they are effectively empty
- the top bar shows the sample project name and model preset

Open these files from the explorer and verify each opens in a tab:
- `project.json`
- `documents/context.md`
- `prompts/brief-review.tera`
- `models/default.yaml`

For each tab, verify:
- clicking the same file again re-focuses the existing tab instead of duplicating it
- the center pane is read-only
- the right inspector updates with kind, path, size, modified time

Type-specific inspector checks:
- `project.json`: project id, default preset, prompt block count, pipeline count
- `context.md`: line count and word count
- `brief-review.tera`: line count and linked block name
- `default.yaml`: model, temperature, and max tokens

### Scenario B: Create a new blank project

In the Project Browser:
1. Choose a parent folder you control, such as your Desktop or `/tmp`
2. Enter a project name like `MilestoneOneManualTest`
3. Click `Create Project`

Verify on disk:
- a new folder is created at the chosen location
- it contains:

```text
project.json
documents/
prompts/
models/
runs/
exports/
```

- `models/default.yaml` exists
- `project.json` points `defaultModelPreset` at `models/default.yaml`

Verify in the app:
- the new project opens automatically
- the explorer shows the required folders
- `project.json` opens and renders as structured JSON
- empty directories do not crash the explorer

### Scenario C: Recent projects behavior

After opening the sample project and creating a new project:
- return to the Project Browser by restarting the app
- confirm both projects appear in recents
- confirm the sample project can be reopened from recents

Optional unavailable-path check:
1. Create a temporary project
2. Close the app
3. Delete that project folder from Finder or the command line
4. Relaunch the app

Verify:
- the recent entry remains visible
- it is marked unavailable rather than silently disappearing

## 5. What Should Not Exist Yet

These are out of scope for milestone 1. If any appear, treat that as a regression in scope discipline.

Verify there is no working UI for:
- editing or saving files
- prompt rendering or validation
- OpenRouter API key entry
- block execution or pipeline execution
- run history browsing beyond the placeholder bottom panel
- import/export workflows beyond direct filesystem viewing

The bottom panel should exist, but only as a placeholder.

## 6. Useful Cleanup Commands

If you created a temporary manual-test project and want to remove it:

```bash
rm -rf /path/to/MilestoneOneManualTest
```

If you want to restart from a clean dev session:

```bash
pkill -f "tauri dev" || true
npm run tauri:dev
```

## 7. Pass Criteria

Milestone 1 manual testing passes if all of the following are true:
- the desktop window launches from `npm run tauri:dev`
- the sample fixture opens correctly
- the new-project flow creates a valid on-disk workspace
- supported asset types open in tabs without duplication
- the inspector updates correctly per asset type
- empty directories and recents do not break the shell
- there is no editable or execution-related behavior exposed yet
