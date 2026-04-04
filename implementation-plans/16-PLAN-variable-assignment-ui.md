# Plan: Variable Assignment UI (Global + Project)

## Context

Variables are currently set by hand-editing `project.json` — no in-app UI exists. The user wants a friendly way to assign variables with two scopes:

- **Global**: shared across all projects (stored in `app_data_dir/global-variables.json`, same location as `recent-projects.json`)
- **Project**: per-project (already in `project.json → variables`)

Template context merge order (lowest → highest priority):
```
global-variables.json   ← baseline
project.json > variables ← overrides globals with same name
built-ins (project.name, current_date, model_id…) ← always present
```

The user also wants Figma MCP set up first to design the Variables panel UI before it's built in code.

---

## Phase 0: Figma MCP Setup (prerequisite — do this before coding)

1. Get a Figma personal access token: figma.com → Settings → Personal access tokens → Generate new token
2. Add to `/Users/carlo/.claude/.mcp.json` inside `"mcpServers"`:
   ```json
   "figma": {
     "command": "npx",
     "args": ["-y", "@figma/mcp"],
     "env": {
       "FIGMA_ACCESS_TOKEN": "your-token-here"
     }
   }
   ```
3. Restart Claude Code (Cmd+Shift+P → Reload Window, or quit and reopen)
4. After restart, use Figma MCP to create a Variables panel design in a Diamond Runner Figma file — two sections (Global / Project), add/remove/edit rows, consistent with the dark InspectorPanel style

---

## Phase 1: Backend — `src-tauri/src/project_store.rs`

### New constant
```rust
const GLOBAL_VARIABLES_FILE_NAME: &str = "global-variables.json";
```

### New private type (internal store format)
```rust
#[derive(Debug, Default, Serialize, Deserialize)]
struct GlobalVariablesStore {
    #[serde(default)]
    variables: BTreeMap<String, String>,
}
```

### New public result type (for Tauri serialization)
No new struct needed — `BTreeMap<String, String>` serializes cleanly as a JSON object.

### New private helpers
```rust
fn read_global_variables_store(app_data_dir: &Path) -> BTreeMap<String, String>
// Returns empty map if file doesn't exist — never errors
fn write_global_variables_store(app_data_dir: &Path, vars: &BTreeMap<String, String>) -> StoreResult<()>
```

### New public functions
```rust
pub fn get_global_variables(app_data_dir: &Path) -> StoreResult<BTreeMap<String, String>>
pub fn set_global_variables(app_data_dir: &Path, variables: BTreeMap<String, String>) -> StoreResult<BTreeMap<String, String>>
pub fn set_project_variables(root_path: &Path, variables: BTreeMap<String, String>, app_data_dir: &Path) -> StoreResult<ProjectSummary>
```

`set_project_variables` serializes incoming `String` values as `serde_json::Value::String` and writes back `project.json` via the existing `write_manifest` helper. Non-string values set manually in `project.json` are left untouched (merge, not replace).

Actually simpler: replace the entire variables map. UI-set values are always strings. Document in comments that complex values (numbers, objects) must be set via direct file edit.

### Update `prepare_template_context`
Add `app_data_dir: Option<&Path>` as final parameter. When `Some`, read global variables and insert them into context *before* project variables:

```rust
// 1. Insert global variables (lowest priority)
if let Some(app_data_dir) = app_data_dir {
    let global_vars = read_global_variables_store(app_data_dir);
    context.insert("global_variables", &global_vars);
    for (name, value) in &global_vars {
        if is_identifier_like(name) {
            context.insert(name, value);
        }
    }
}

// 2. Insert project variables (overrides globals — existing logic unchanged)
context.insert("variables", &manifest.variables);
for (name, value) in &manifest.variables {
    if is_identifier_like(name) {
        context.insert(name, value);
    }
}
```

### Update callers of `prepare_template_context`
- `validate_project_template(root_path, relative_path, content, app_data_dir)` — add `app_data_dir` param, pass `Some(app_data_dir)` through
- `execute_prompt_block_with_transport(…, app_data_dir)` — add param, pass `Some(app_data_dir)` through
- Internal test calls — pass `None` (global variables are skipped, tests unchanged)

### Update public execution/validation functions
```rust
pub fn validate_project_template(root_path: &Path, relative_path: &str, content: &str, app_data_dir: &Path)
pub fn execute_prompt_block(root_path: &Path, relative_path: &str, content: &str, app_data_dir: &Path)
pub fn execute_pipeline(root_path: &Path, pipeline_id: &str, app_data_dir: &Path)
```

---

## Phase 2: `src-tauri/src/main.rs`

### Update existing commands to use `app: tauri::AppHandle`
`validate_project_template` and `execute_prompt_block` and `execute_pipeline` currently don't take `app`. Add it:
```rust
#[tauri::command]
fn execute_prompt_block(app: tauri::AppHandle, root_path: String, ...) -> Result<...>
```
Pass `app_data_dir(&app)?` through to the store function.

### New commands
```rust
#[tauri::command]
fn get_global_variables(app: tauri::AppHandle) -> Result<BTreeMap<String, String>, String>

#[tauri::command]
fn set_global_variables(app: tauri::AppHandle, variables: BTreeMap<String, String>) -> Result<BTreeMap<String, String>, String>

#[tauri::command]
fn set_project_variables(app: tauri::AppHandle, root_path: String, variables: BTreeMap<String, String>) -> Result<ProjectSummary, String>
```

Register all three in `invoke_handler`.

---

## Phase 3: Frontend

### `src/lib/types/project.ts`
No new types — variables are `Record<string, string>` (plain JS object). Already compatible.

### `src/lib/tauri.ts`
Add 3 wrappers:
```ts
export async function getGlobalVariables(): Promise<Record<string, string>>
export async function setGlobalVariables(variables: Record<string, string>): Promise<Record<string, string>>
export async function setProjectVariables(rootPath: string, variables: Record<string, string>): Promise<ProjectSummary>
```

### `src/App.svelte`
- Import the 3 new functions
- Add state: `let globalVariables = $state<Record<string, string>>({});`
- Fetch on `onMount` alongside `getRecentProjects`
- Add handlers: `handleSetGlobalVariables`, `handleSetProjectVariables`
- `handleSetProjectVariables` updates both `globalVariables` state and refreshes workspace summary
- Pass `globalVariables`, `onSetGlobalVariables`, `onSetProjectVariables` props to `WorkspaceShell`

### `src/lib/components/WorkspaceShell.svelte`
Thread `globalVariables`, `onSetGlobalVariables`, `onSetProjectVariables` through Props → InspectorPanel.

### `src/lib/components/InspectorPanel.svelte`
New **Variables** section (add between the Usage section and the Inspector section):

```
┌─────────────────────────────────────────┐
│ VARIABLES                               │
│                                         │
│ Global                                  │
│ ┌──────────────┬──────────────────────┐ │
│ │ tone         │ literary fiction  ✕  │ │
│ │ pov          │ third-limited     ✕  │ │
│ └──────────────┴──────────────────────┘ │
│ [name ________] [value ______________]  │
│                               [+ Add]   │
│                                         │
│ Project                                 │
│ ┌──────────────┬──────────────────────┐ │
│ │ chapter      │ 12                ✕  │ │
│ │ word_target  │ 5000              ✕  │ │
│ └──────────────┴──────────────────────┘ │
│ [name ________] [value ______________]  │
│                               [+ Add]   │
└─────────────────────────────────────────┘
```

State (Svelte 5 `$state`):
```ts
let newGlobalName = $state('');
let newGlobalValue = $state('');
let newProjectName = $state('');
let newProjectValue = $state('');
let variablesSaving = $state(false);
```

Add/remove pattern mirrors the existing pipeline form: inline input pair + "Add" button, existing entries shown as rows with a remove `×` button. No separate "edit" mode — remove and re-add.

---

## Phase 4: Figma Design

After Figma MCP is configured, use it to:
1. Create a Diamond Runner component library file or add a frame to an existing file
2. Design the Variables panel section matching the InspectorPanel dark style (CSS vars: `--bg: rgba(6,10,18,0.88)`, `--accent: #8bb1ff`, border `rgba(157,180,255,0.14)`)
3. Show both Global and Project subsections with add/remove rows
4. Use the design as reference during coding Phase 3

---

## Critical Files

| File | Change |
|------|--------|
| `src-tauri/src/project_store.rs` | New global vars store, update prepare_template_context, update public fn signatures |
| `src-tauri/src/main.rs` | Add app handle to 3 commands, register 3 new commands |
| `src/lib/tauri.ts` | 3 new wrappers |
| `src/lib/types/project.ts` | None (variables as Record<string,string> is already valid TS) |
| `src/App.svelte` | globalVariables state, fetch on mount, 2 new handlers |
| `src/lib/components/WorkspaceShell.svelte` | Thread 3 new props |
| `src/lib/components/InspectorPanel.svelte` | New Variables section |
| `src/App.test.ts` | Mock getGlobalVariables, add fixture, mock setGlobalVariables/setProjectVariables |
| `/Users/carlo/.claude/.mcp.json` | Add Figma MCP entry (prerequisite) |

---

## Tests to Add

**Rust (project_store.rs):**
- `reads_empty_global_variables_when_file_missing` — returns empty map
- `saves_and_reloads_global_variables` — round-trip
- `global_variables_are_available_in_template_context`
- `project_variables_override_globals_with_same_name`
- `set_project_variables_persists_to_manifest`

**Frontend (App.test.ts):**
- Mock `getGlobalVariables` returns `{}` in `beforeEach`
- Mock `setGlobalVariables` and `setProjectVariables`
- Test: adding a global variable calls `setGlobalVariables`
- Test: adding a project variable calls `setProjectVariables`

---

## Verification

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # 33 + 5 new = 38 pass
npm run test -- --run                               # 23 + 2 new = 25 pass
npm run typecheck                                   # 0 errors
npm run lint                                        # clean
```

Manual smoke test:
1. Open a project → Variables section shows empty Global + empty Project
2. Add a global variable `tone = precise` → close project → open another project → template preview shows `{{ tone }}` resolves to `precise`
3. Add project variable `tone = literary` → template preview now shows `literary` (project overrides global)
4. Remove project variable → preview reverts to global `precise`
