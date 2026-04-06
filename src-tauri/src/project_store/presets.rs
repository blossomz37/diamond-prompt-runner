use std::{
    fs,
    path::Path,
};

use crate::types::*;
use super::{summarize_project, timestamp, validate_project, write_manifest};
use super::execution::load_model_id_from_preset;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn list_model_presets(root_path: &Path) -> StoreResult<Vec<ModelPresetSummary>> {
    let (root_path, _manifest) = validate_project(root_path)?;
    let models_dir = root_path.join("models");
    if !models_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut presets: Vec<ModelPresetSummary> = Vec::new();
    for entry in fs::read_dir(&models_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let relative_path = format!("models/{}", filename);
        let model_id = load_model_id_from_preset(&root_path, &relative_path)
            .unwrap_or_else(|_| "unknown".to_string());
        presets.push(ModelPresetSummary {
            path: relative_path,
            filename,
            model_id,
        });
    }
    presets.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(presets)
}

pub fn set_default_model_preset(
    root_path: &Path,
    preset_path: &str,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    if !root_path.join(preset_path).is_file() {
        return Err(ProjectStoreError::message(format!(
            "Preset file not found: {}",
            preset_path
        )));
    }
    manifest.default_model_preset = preset_path.to_string();
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    summarize_project(&root_path, &manifest)
}

pub fn create_model_preset(
    root_path: &Path,
    filename: &str,
    model_id: &str,
) -> StoreResult<ModelPresetSummary> {
    let (root_path, _manifest) = validate_project(root_path)?;

    let safe_filename = if filename.ends_with(".yaml") {
        filename.to_string()
    } else {
        format!("{}.yaml", filename)
    };

    let preset_path = root_path.join("models").join(&safe_filename);
    if preset_path.exists() {
        return Err(ProjectStoreError::message(format!(
            "Preset already exists: models/{}",
            safe_filename
        )));
    }

    let content = format!(
        "# Custom model preset.\nmodel: {}\ntemperature: 1\nmax_completion_tokens: 16000\n",
        model_id
    );
    fs::write(&preset_path, &content)?;

    Ok(ModelPresetSummary {
        path: format!("models/{}", safe_filename),
        filename: safe_filename,
        model_id: model_id.to_string(),
    })
}

pub fn delete_model_preset(root_path: &Path, preset_path: &str) -> StoreResult<()> {
    let (root_path, manifest) = validate_project(root_path)?;
    if manifest.default_model_preset == preset_path {
        return Err(ProjectStoreError::message(
            "Cannot delete the current default model preset. Change the default first.",
        ));
    }
    let full_path = root_path.join(preset_path);
    if !full_path.is_file() {
        return Err(ProjectStoreError::message(format!(
            "Preset file not found: {}",
            preset_path
        )));
    }
    fs::remove_file(full_path)?;
    Ok(())
}

pub fn set_block_model_preset(
    root_path: &Path,
    block_id: &str,
    preset_path: Option<&str>,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;

    // Validate preset exists when setting (not clearing)
    if let Some(path) = preset_path {
        if !root_path.join(path).is_file() {
            return Err(ProjectStoreError::message(format!(
                "Preset file not found: {}",
                path
            )));
        }
    }

    let block = manifest
        .prompt_blocks
        .iter_mut()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| {
            ProjectStoreError::message(format!("Prompt block not found: {}", block_id))
        })?;

    block.model_preset = preset_path.map(|p| p.to_string());
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    summarize_project(&root_path, &manifest)
}
