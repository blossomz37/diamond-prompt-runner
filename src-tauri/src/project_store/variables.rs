use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::*;
use super::{summarize_project, timestamp, validate_project, write_manifest};

const WORKSPACE_VARIABLES_FILE: &str = "variables/workspace-variables.yaml";
const GLOBAL_VARIABLES_FILE_NAME: &str = "global-variables.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct GlobalVariablesStore {
    #[serde(default)]
    variables: BTreeMap<String, String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

pub fn get_global_variables(app_data_dir: &Path) -> StoreResult<BTreeMap<String, String>> {
    Ok(read_global_variables_store(app_data_dir))
}

pub fn set_global_variables(
    app_data_dir: &Path,
    variables: BTreeMap<String, String>,
) -> StoreResult<BTreeMap<String, String>> {
    fs::create_dir_all(app_data_dir)?;
    write_global_variables_store(app_data_dir, &variables)?;
    Ok(variables)
}

pub fn set_project_variables(
    root_path: &Path,
    variables: BTreeMap<String, String>,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    // Write to YAML file (source of truth)
    write_workspace_variables_yaml(&root_path, &variables)?;
    // Sync to project.json for backward compatibility
    manifest.variables = variables
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    summarize_project(&root_path, &manifest)
}

// ── Helpers (crate-visible for execution.rs and project_store) ───────────────

pub(crate) fn read_global_variables_store(app_data_dir: &Path) -> BTreeMap<String, String> {
    let store_path = app_data_dir.join(GLOBAL_VARIABLES_FILE_NAME);
    if !store_path.exists() {
        return BTreeMap::new();
    }
    fs::read_to_string(store_path)
        .ok()
        .and_then(|content| serde_json::from_str::<GlobalVariablesStore>(&content).ok())
        .map(|store| store.variables)
        .unwrap_or_default()
}

pub(crate) fn read_workspace_variables_yaml(root_path: &Path) -> BTreeMap<String, String> {
    let yaml_path = root_path.join(WORKSPACE_VARIABLES_FILE);
    if !yaml_path.exists() {
        return BTreeMap::new();
    }
    fs::read_to_string(yaml_path)
        .ok()
        .and_then(|content| serde_yaml::from_str::<BTreeMap<String, String>>(&content).ok())
        .unwrap_or_default()
}

pub(crate) fn write_workspace_variables_yaml(root_path: &Path, variables: &BTreeMap<String, String>) -> StoreResult<()> {
    let yaml_path = root_path.join(WORKSPACE_VARIABLES_FILE);
    let content = if variables.is_empty() {
        "# Workspace variables — editable here or via the sidebar\n".to_string()
    } else {
        format!(
            "# Workspace variables — editable here or via the sidebar\n{}",
            serde_yaml::to_string(variables).unwrap_or_default()
        )
    };
    fs::write(yaml_path, content)?;
    Ok(())
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn write_global_variables_store(app_data_dir: &Path, variables: &BTreeMap<String, String>) -> StoreResult<()> {
    let store = GlobalVariablesStore { variables: variables.clone() };
    let store_path = app_data_dir.join(GLOBAL_VARIABLES_FILE_NAME);
    fs::write(store_path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}
