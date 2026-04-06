use std::{
    fs,
    path::{Component, Path},
};

use chrono::{SecondsFormat, Utc};
use serde_json::{json, Value};

use crate::types::*;
use super::{
    detail, diff_path, sanitize_relative_path, summarize_project,
    timestamp, validate_project, update_recent_projects, unique_export_slug, write_manifest,
};
use super::execution::PersistedRunRecord;

// ── Constants ────────────────────────────────────────────────────────────────

const PROJECT_DIRS: [&str; 7] = ["documents", "prompts", "models", "runs", "exports", "help", "variables"];

// ── Public API ────────────────────────────────────────────────────────────────

pub fn list_project_assets(root_path: &Path) -> StoreResult<Vec<ProjectAssetNode>> {
    let (root_path, _) = validate_project(root_path)?;
    let mut nodes = Vec::with_capacity(PROJECT_DIRS.len() + 1);

    nodes.push(ProjectAssetNode {
        name: "project.json".to_string(),
        path: "project.json".to_string(),
        kind: AssetKind::Manifest,
        is_directory: false,
        children: Vec::new(),
    });

    for directory in PROJECT_DIRS {
        nodes.push(build_tree_node(&root_path, &root_path.join(directory), directory.to_string())?);
    }

    Ok(nodes)
}

pub fn read_project_asset(root_path: &Path, relative_path: &str) -> StoreResult<AssetContent> {
    let (root_path, manifest) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(relative_path)?;
    let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");
    let full_path = root_path.join(&safe_relative);

    if !full_path.exists() {
        return Err(ProjectStoreError::message(format!(
            "Asset `{relative_path}` does not exist."
        )));
    }

    if full_path.is_dir() {
        return Err(ProjectStoreError::message("Directories cannot be opened as tabs."));
    }

    let kind = classify_asset(&safe_relative_string, false);
    let content = fs::read_to_string(&full_path).unwrap_or_else(|_| String::from("Binary or unreadable file."));
    let is_editable = is_editable_kind(&kind);
    let metadata = build_metadata(
        &root_path,
        &manifest,
        &safe_relative_string,
        &full_path,
        &content,
        &kind,
    )?;

    let (view, parsed_json) = match kind {
        AssetKind::Manifest | AssetKind::Json => {
            let value = serde_json::from_str::<Value>(&content).ok();
            (AssetView::Json, value)
        }
        AssetKind::Markdown | AssetKind::Text | AssetKind::Tera | AssetKind::Yaml => (AssetView::Text, None),
        _ => (
            AssetView::Placeholder,
            None,
        ),
    };

    let placeholder_content = if view == AssetView::Placeholder {
        format!("`{relative_path}` is not supported in the milestone-1 viewer.")
    } else {
        content
    };

    Ok(AssetContent {
        path: safe_relative_string,
        kind,
        view,
        content: placeholder_content,
        is_editable,
        metadata,
        parsed_json,
    })
}

pub fn write_project_asset(root_path: &Path, relative_path: &str, content: &str) -> StoreResult<AssetContent> {
    let (root_path, _) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(relative_path)?;
    let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");
    let full_path = root_path.join(&safe_relative);

    if !full_path.exists() {
        return Err(ProjectStoreError::message(format!(
            "Asset `{relative_path}` does not exist."
        )));
    }

    if full_path.is_dir() {
        return Err(ProjectStoreError::message("Directories cannot be saved as files."));
    }

    let kind = classify_asset(&safe_relative_string, false);
    if !is_editable_kind(&kind) {
        return Err(ProjectStoreError::message(
            "This asset type is read-only in the editing slice.",
        ));
    }

    fs::write(&full_path, content)?;
    read_project_asset(&root_path, &safe_relative_string)
}

pub fn export_project_assets(
    root_path: &Path,
    bundle_name: &str,
    relative_paths: &[String],
    app_data_dir: &Path,
) -> StoreResult<ExportBundleResult> {
    let trimmed_name = bundle_name.trim();
    if trimmed_name.is_empty() {
        return Err(ProjectStoreError::message("Export name cannot be empty."));
    }

    if relative_paths.is_empty() {
        return Err(ProjectStoreError::message(
            "Select at least one project asset to export.",
        ));
    }

    let (root_path, manifest) = validate_project(root_path)?;
    let bundle_slug = unique_export_slug(&root_path, trimmed_name);
    let bundle_path = format!("exports/{bundle_slug}");
    let bundle_root = root_path.join(&bundle_path);
    fs::create_dir_all(&bundle_root)?;

    let mut exported_paths = Vec::new();
    for relative_path in relative_paths {
        let safe_relative = sanitize_relative_path(relative_path)?;
        let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");
        let kind = classify_asset(&safe_relative_string, false);

        if !is_exportable_kind(&kind) {
            return Err(ProjectStoreError::message(format!(
                "Asset `{safe_relative_string}` cannot be exported in this slice."
            )));
        }

        let source_path = root_path.join(&safe_relative);
        if !source_path.is_file() {
            return Err(ProjectStoreError::message(format!(
                "Asset `{safe_relative_string}` was not found on disk."
            )));
        }

        let destination_path = bundle_root.join(&safe_relative);
        if let Some(parent) = destination_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source_path, &destination_path)?;
        exported_paths.push(safe_relative_string);
    }

    let export_manifest = json!({
        "bundleName": trimmed_name,
        "bundlePath": bundle_path,
        "exportedAt": timestamp(),
        "projectName": manifest.project_name,
        "paths": exported_paths,
    });
    fs::write(
        bundle_root.join("export.json"),
        serde_json::to_string_pretty(&export_manifest)?,
    )?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(ExportBundleResult {
        summary,
        bundle_name: trimmed_name.to_string(),
        bundle_path,
        exported_paths: relative_paths
            .iter()
            .map(|path| path.replace('\\', "/"))
            .collect(),
    })
}

pub fn delete_document(root_path: &Path, relative_path: &str) -> StoreResult<()> {
    let (root_path, _) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(relative_path)?;
    let first_component = safe_relative.components().next();
    if first_component != Some(Component::Normal("documents".as_ref())) {
        return Err(ProjectStoreError::message(
            "Document path must be within the documents/ directory.",
        ));
    }
    let full_path = root_path.join(&safe_relative);
    if !full_path.is_file() {
        return Err(ProjectStoreError::message(format!(
            "Document not found: {relative_path}"
        )));
    }
    fs::remove_file(full_path)?;
    Ok(())
}

pub fn trash_prompt(root_path: &Path, relative_path: &str, app_data_dir: &Path) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(relative_path)?;
    let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");

    if safe_relative.components().next() != Some(Component::Normal("prompts".as_ref())) {
        return Err(ProjectStoreError::message(
            "Prompt path must be within the prompts/ directory.",
        ));
    }

    if safe_relative.extension().and_then(|value| value.to_str()) != Some("tera") {
        return Err(ProjectStoreError::message(
            "Only .tera prompt templates can be moved to Trash.",
        ));
    }

    let full_path = root_path.join(&safe_relative);
    if !full_path.is_file() {
        return Err(ProjectStoreError::message(format!(
            "Prompt template not found: {relative_path}"
        )));
    }

    let trash_path = unique_trash_path(&root_path, &safe_relative)?;
    if let Some(parent) = trash_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&full_path, &trash_path)?;

    let removed_block_ids = manifest
        .prompt_blocks
        .iter()
        .filter(|block| block.template_source == safe_relative_string)
        .map(|block| block.block_id.clone())
        .collect::<Vec<_>>();

    if !removed_block_ids.is_empty() {
        manifest
            .prompt_blocks
            .retain(|block| block.template_source != safe_relative_string);

        for pipeline in &mut manifest.pipelines {
            pipeline
                .ordered_blocks
                .retain(|block_id| !removed_block_ids.iter().any(|removed_id| removed_id == block_id));
        }
    }

    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;
    Ok(summary)
}

pub fn rename_document(root_path: &Path, old_path: &str, new_name: &str) -> StoreResult<String> {
    let (root_path, _) = validate_project(root_path)?;
    let safe_old = sanitize_relative_path(old_path)?;
    let trimmed_name = new_name.trim();

    let first_component = safe_old.components().next();
    if first_component != Some(Component::Normal("documents".as_ref())) {
        return Err(ProjectStoreError::message(
            "Document path must be within the documents/ directory.",
        ));
    }

    if trimmed_name.is_empty() {
        return Err(ProjectStoreError::message("New document name cannot be empty."));
    }
    if trimmed_name.contains('/') || trimmed_name.contains('\\') {
        return Err(ProjectStoreError::message(
            "New document name cannot contain path separators.",
        ));
    }

    let old_full = root_path.join(&safe_old);
    if !old_full.is_file() {
        return Err(ProjectStoreError::message(format!(
            "Document not found: {old_path}"
        )));
    }

    let new_full = old_full.with_file_name(trimmed_name);
    if new_full.exists() {
        return Err(ProjectStoreError::message(format!(
            "A document named '{trimmed_name}' already exists."
        )));
    }

    fs::rename(&old_full, &new_full)?;
    Ok(format!("documents/{trimmed_name}"))
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn build_tree_node(root_path: &Path, full_path: &Path, relative_path: String) -> StoreResult<ProjectAssetNode> {
    let metadata = fs::metadata(full_path)?;
    let is_directory = metadata.is_dir();
    let name = display_name_for_asset(full_path, &relative_path, is_directory);
    let mut children = Vec::new();

    if is_directory {
        let mut entries = fs::read_dir(full_path)?
            .flatten()
            .filter(|entry| !is_hidden_ui_entry(entry.path().as_path()))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        entries.sort();

        for entry_path in entries {
            let child_relative = diff_path(root_path, &entry_path)?;
            children.push(build_tree_node(root_path, &entry_path, child_relative)?);
        }
    }

    Ok(ProjectAssetNode {
        name,
        path: relative_path.clone(),
        kind: classify_asset(&relative_path, is_directory),
        is_directory,
        children,
    })
}

fn display_name_for_asset(full_path: &Path, relative_path: &str, is_directory: bool) -> String {
    if !is_directory {
        if let Some(name) = display_name_for_run_record(full_path, relative_path) {
            return name;
        }
    }

    full_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn display_name_for_run_record(full_path: &Path, relative_path: &str) -> Option<String> {
    if !relative_path.starts_with("runs/") || !relative_path.ends_with(".json") {
        return None;
    }

    let content = fs::read_to_string(full_path).ok()?;
    let record = serde_json::from_str::<PersistedRunRecord>(&content).ok()?;
    let block_name = record.block_name.trim();
    if block_name.is_empty() {
        None
    } else {
        Some(block_name.to_string())
    }
}

fn is_hidden_ui_entry(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

fn unique_trash_path(root_path: &Path, relative_path: &Path) -> StoreResult<std::path::PathBuf> {
    let trash_root = root_path.join("Trash");
    let candidate = trash_root.join(relative_path);
    if !candidate.exists() {
        return Ok(candidate);
    }

    let stem = candidate
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ProjectStoreError::message("Prompt filename is invalid."))?;
    let extension = candidate.extension().and_then(|value| value.to_str()).unwrap_or("");

    let mut suffix = 2usize;
    loop {
        let next_name = if extension.is_empty() {
            format!("{stem}-{suffix}")
        } else {
            format!("{stem}-{suffix}.{extension}")
        };
        let next_candidate = candidate.with_file_name(next_name);
        if !next_candidate.exists() {
            return Ok(next_candidate);
        }
        suffix += 1;
    }
}

fn build_metadata(
    root_path: &Path,
    manifest: &ProjectManifest,
    relative_path: &str,
    full_path: &Path,
    content: &str,
    kind: &AssetKind,
) -> StoreResult<AssetMetadata> {
    let metadata = fs::metadata(full_path)?;
    let modified_at = metadata
        .modified()
        .ok()
        .map(|time| chrono::DateTime::<Utc>::from(time).to_rfc3339_opts(SecondsFormat::Secs, true));

    let mut details = Vec::new();

    match kind {
        AssetKind::Manifest => {
            details.push(detail("Project ID", &manifest.project_id));
            details.push(detail("Default Preset", &manifest.default_model_preset));
            details.push(detail("Prompt Blocks", &manifest.prompt_blocks.len().to_string()));
            details.push(detail("Pipelines", &manifest.pipelines.len().to_string()));
        }
        AssetKind::Markdown | AssetKind::Text => {
            details.push(detail("Lines", &content.lines().count().to_string()));
            details.push(detail("Words", &content.split_whitespace().count().to_string()));
        }
        AssetKind::Tera => {
            let linked_blocks = manifest
                .prompt_blocks
                .iter()
                .filter(|block| block.template_source == relative_path)
                .map(|block| block.name.clone())
                .collect::<Vec<_>>();
            let linked_blocks_value = if linked_blocks.is_empty() {
                "None".to_string()
            } else {
                linked_blocks.join(", ")
            };
            details.push(detail("Lines", &content.lines().count().to_string()));
            details.push(detail("Linked Blocks", &linked_blocks_value));
        }
        AssetKind::Yaml => {
            match serde_yaml::from_str::<serde_yaml::Value>(content) {
                Ok(yaml) => {
                    details.push(detail(
                        "Model",
                        yaml.get("model")
                            .and_then(|value| value.as_str())
                            .unwrap_or("Unknown"),
                    ));
                    details.push(detail(
                        "Temperature",
                        &yaml.get("temperature")
                            .map(yaml_value_to_string)
                            .unwrap_or_else(|| "—".to_string()),
                    ));
                    details.push(detail(
                        "Max Tokens",
                        &yaml.get("max_completion_tokens")
                            .map(yaml_value_to_string)
                            .unwrap_or_else(|| "—".to_string()),
                    ));
                }
                Err(_) => {
                    details.push(detail("Model", "Invalid YAML"));
                    details.push(detail("Temperature", "—"));
                    details.push(detail("Max Tokens", "—"));
                }
            }
        }
        _ => {
            details.push(detail("Status", "Unsupported in milestone 1"));
            details.push(detail("Project Root", &root_path.to_string_lossy()));
        }
    }

    Ok(AssetMetadata {
        kind: kind.clone(),
        path: relative_path.to_string(),
        name: display_name_for_asset(full_path, relative_path, false),
        size_bytes: Some(metadata.len()),
        modified_at,
        details,
    })
}

pub(crate) fn classify_asset(relative_path: &str, is_directory: bool) -> AssetKind {
    if is_directory {
        return AssetKind::Directory;
    }

    if relative_path == "project.json" {
        return AssetKind::Manifest;
    }

    match Path::new(relative_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "md" | "markdown" => AssetKind::Markdown,
        "txt" => AssetKind::Text,
        "tera" => AssetKind::Tera,
        "yaml" | "yml" => AssetKind::Yaml,
        "json" => AssetKind::Json,
        _ => AssetKind::Unknown,
    }
}

fn is_exportable_kind(kind: &AssetKind) -> bool {
    matches!(
        kind,
        AssetKind::Manifest
            | AssetKind::Markdown
            | AssetKind::Text
            | AssetKind::Tera
            | AssetKind::Yaml
            | AssetKind::Json
    )
}

fn is_editable_kind(kind: &AssetKind) -> bool {
    matches!(kind, AssetKind::Markdown | AssetKind::Text | AssetKind::Tera | AssetKind::Yaml)
}

fn yaml_value_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Bool(value) => value.to_string(),
        serde_yaml::Value::Number(value) => value.to_string(),
        serde_yaml::Value::String(value) => value.clone(),
        other => serde_yaml::to_string(other)
            .unwrap_or_else(|_| "—".to_string())
            .trim()
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn trashes_prompt_and_unregisters_dependent_blocks() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = Path::new(&summary.root_path);

        let prompt_path = root_path.join("prompts/review.tera");
        fs::write(&prompt_path, "Review prompt").unwrap();

        let mut manifest = super::super::read_manifest(&root_path.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "replace_document".to_string(),
            output_filename: None,
        });
        manifest.pipelines.push(Pipeline {
            pipeline_id: "line-edit".to_string(),
            name: "Line Edit".to_string(),
            execution_mode: "sequential".to_string(),
            ordered_blocks: vec!["review".to_string()],
        });
        super::super::write_manifest(root_path, &manifest).unwrap();

        let updated = trash_prompt(root_path, "prompts/review.tera", &app_data_dir).unwrap();

        assert_eq!(updated.counts.prompts, 0);
        assert!(!prompt_path.exists());
        assert!(root_path.join("Trash/prompts/review.tera").is_file());

        let updated_manifest = super::super::read_manifest(&root_path.join("project.json")).unwrap();
        assert!(updated_manifest.prompt_blocks.is_empty());
        assert!(!updated_manifest
            .pipelines
            .iter()
            .any(|pipeline| pipeline.ordered_blocks.iter().any(|block_id| block_id == "review")));
    }
}
