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

pub fn convert_project_asset(root_path: &Path, relative_path: &str) -> StoreResult<AssetConversionResult> {
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
        return Err(ProjectStoreError::message("Directories cannot be converted."));
    }

    let source_kind = classify_asset(&safe_relative_string, false);
    let content = fs::read_to_string(&full_path)?;
    let audit = assess_asset_conversion(&safe_relative, &safe_relative_string, &source_kind, &content);
    if audit.status != AssetConversionAuditStatus::Convertible {
        let detail = if audit.warnings.is_empty() {
            audit.assessment.clone()
        } else {
            format!("{} {}", audit.assessment, audit.warnings.join(" "))
        };
        return Err(ProjectStoreError::message(detail));
    }

    let prepared = prepare_asset_conversion(&safe_relative, &source_kind, &content)?;
    let target_full_path = root_path.join(&prepared.target_path);

    if target_full_path.exists() {
        return Err(ProjectStoreError::message(format!(
            "A converted copy already exists at `{}`.",
            prepared.target_path
        )));
    }

    if let Some(parent) = target_full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target_full_path, &prepared.converted_content)?;

    let asset = read_project_asset(&root_path, &prepared.target_path)?;

    Ok(AssetConversionResult {
        source_path: safe_relative_string,
        target_path: prepared.target_path,
        source_kind,
        target_kind: prepared.target_kind,
        assessment: prepared.assessment,
        warnings: prepared.warnings,
        asset,
    })
}

pub fn audit_project_asset(root_path: &Path, relative_path: &str) -> StoreResult<AssetConversionAuditResult> {
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
        return Err(ProjectStoreError::message("Directories cannot be audited for conversion."));
    }

    let source_kind = classify_asset(&safe_relative_string, false);
    let content = fs::read_to_string(&full_path)?;
    let audit = assess_asset_conversion(&safe_relative, &safe_relative_string, &source_kind, &content);

    Ok(AssetConversionAuditResult {
        source_path: safe_relative_string,
        target_path: audit.target_path,
        source_kind,
        target_kind: audit.target_kind,
        status: audit.status,
        assessment: audit.assessment,
        warnings: audit.warnings,
    })
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

#[derive(Debug, Clone)]
struct PreparedAssetConversion {
    target_path: String,
    target_kind: AssetKind,
    converted_content: String,
    assessment: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct PreparedAssetAudit {
    target_path: String,
    target_kind: AssetKind,
    status: AssetConversionAuditStatus,
    assessment: String,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct StructuredEntry {
    key: String,
    value: StructuredValue,
}

#[derive(Debug, Clone)]
enum StructuredValue {
    Section(Vec<StructuredEntry>),
    Scalar(String),
}

fn assess_asset_conversion(
    relative_path: &Path,
    safe_relative_string: &str,
    source_kind: &AssetKind,
    content: &str,
) -> PreparedAssetAudit {
    match source_kind {
        AssetKind::Markdown => assess_markdown_conversion(relative_path, content),
        AssetKind::Yaml => assess_yaml_conversion(relative_path, content),
        _ => PreparedAssetAudit {
            target_path: relative_path.to_string_lossy().replace('\\', "/"),
            target_kind: source_kind.clone(),
            status: AssetConversionAuditStatus::NotSafelyConvertible,
            assessment: format!(
                "Asset `{safe_relative_string}` is not a supported Markdown/YAML conversion candidate."
            ),
            warnings: Vec::new(),
        },
    }
}

fn prepare_asset_conversion(
    relative_path: &Path,
    source_kind: &AssetKind,
    content: &str,
) -> StoreResult<PreparedAssetConversion> {
    match source_kind {
        AssetKind::Markdown => Ok(PreparedAssetConversion {
            target_path: target_path_with_extension(relative_path, "yaml")?,
            target_kind: AssetKind::Yaml,
            converted_content: convert_markdown_to_yaml(content)?,
            assessment:
                "Structured Markdown headings and `**field:** value` lines were mapped into YAML."
                    .to_string(),
            warnings: Vec::new(),
        }),
        AssetKind::Yaml => Ok(PreparedAssetConversion {
            target_path: target_path_with_extension(relative_path, "md")?,
            target_kind: AssetKind::Markdown,
            converted_content: convert_yaml_to_markdown(content)?,
            assessment: "YAML mappings and scalar values were rendered as structured Markdown."
                .to_string(),
            warnings: Vec::new(),
        }),
        _ => Err(ProjectStoreError::message(
            "Only Markdown and YAML assets can be converted.",
        )),
    }
}

fn assess_markdown_conversion(relative_path: &Path, content: &str) -> PreparedAssetAudit {
    let target_path = target_path_with_extension(relative_path, "yaml")
        .unwrap_or_else(|_| relative_path.to_string_lossy().replace('\\', "/"));

    let mut warnings = Vec::new();
    let mut fatal_issues = Vec::new();
    let normalized = content.replace("\r\n", "\n");
    let mut lines = normalized.lines().peekable();
    let mut heading_path: Vec<String> = Vec::new();
    let mut last_field: Option<String> = None;
    let mut field_count = 0usize;

    if matches!(lines.peek(), Some(line) if line.trim() == "---") {
        lines.next();
        let mut frontmatter_lines = Vec::new();
        let mut found_closing = false;

        for line in lines.by_ref() {
            if line.trim() == "---" {
                found_closing = true;
                break;
            }
            frontmatter_lines.push(line);
        }

        if !found_closing {
            fatal_issues.push("Frontmatter was opened with `---` but never closed.".to_string());
        } else if !frontmatter_lines.join("\n").trim().is_empty()
            && serde_yaml::from_str::<serde_yaml::Value>(&frontmatter_lines.join("\n")).is_err()
        {
            warnings.push("Frontmatter is present but could not be parsed as YAML.".to_string());
        }
    }

    for raw_line in lines {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(level) = parse_any_heading_level(trimmed) {
            if level < 2 {
                warnings.push(format!(
                    "Heading `{trimmed}` uses `#` and would need to be remapped before safe conversion."
                ));
                last_field = None;
                continue;
            }

            let heading = trimmed[level..].trim();
            let depth = level - 2;
            if depth > heading_path.len() {
                fatal_issues.push(format!(
                    "Heading `{heading}` skips a nesting level and cannot be mapped safely."
                ));
                last_field = None;
                continue;
            }

            heading_path.truncate(depth);
            heading_path.push(heading.to_string());
            last_field = None;
            continue;
        }

        if let Some((field, _value)) = parse_markdown_field(trimmed) {
            field_count += 1;
            last_field = Some(field.to_string());
            continue;
        }

        if last_field.is_some() {
            continue;
        }

        if is_markdown_list_item(trimmed) {
            warnings.push(format!(
                "List item `{trimmed}` is not yet normalized into a stable YAML array shape."
            ));
        } else if is_markdown_table_row(trimmed) {
            warnings.push(format!(
                "Table row `{trimmed}` would need explicit table-to-object mapping before safe conversion."
            ));
        } else {
            warnings.push(format!(
                "Line `{trimmed}` is freeform prose and would need a named field before safe conversion."
            ));
        }
    }

    let (status, assessment) = if !fatal_issues.is_empty() {
        (
            AssetConversionAuditStatus::NotSafelyConvertible,
            "The Markdown contains structural issues that would make automatic YAML conversion unsafe."
                .to_string(),
        )
    } else if field_count == 0 {
        (
            AssetConversionAuditStatus::NotSafelyConvertible,
            "No structured `**field:** value` definitions were found, so Diamond cannot safely convert this Markdown yet."
                .to_string(),
        )
    } else if warnings.is_empty() {
        (
            AssetConversionAuditStatus::Convertible,
            "The Markdown matches Diamond's current structured worksheet rules and is safe to convert."
                .to_string(),
        )
    } else {
        (
            AssetConversionAuditStatus::PartiallyConvertible,
            "The Markdown contains extractable structure, but some lines would need manual cleanup or richer parser support before safe conversion."
                .to_string(),
        )
    };

    let mut combined_warnings = fatal_issues;
    combined_warnings.extend(warnings);

    PreparedAssetAudit {
        target_path,
        target_kind: AssetKind::Yaml,
        status,
        assessment,
        warnings: combined_warnings,
    }
}

fn assess_yaml_conversion(relative_path: &Path, content: &str) -> PreparedAssetAudit {
    let target_path = target_path_with_extension(relative_path, "md")
        .unwrap_or_else(|_| relative_path.to_string_lossy().replace('\\', "/"));

    let yaml = match serde_yaml::from_str::<serde_yaml::Value>(content) {
        Ok(yaml) => yaml,
        Err(error) => {
            return PreparedAssetAudit {
                target_path,
                target_kind: AssetKind::Markdown,
                status: AssetConversionAuditStatus::NotSafelyConvertible,
                assessment: "The YAML could not be parsed, so Diamond cannot assess a safe Markdown conversion."
                    .to_string(),
                warnings: vec![error.to_string()],
            };
        }
    };

    let root = match yaml.as_mapping() {
        Some(root) => root,
        None => {
            return PreparedAssetAudit {
                target_path,
                target_kind: AssetKind::Markdown,
                status: AssetConversionAuditStatus::NotSafelyConvertible,
                assessment: "Only YAML objects can be converted into structured Markdown safely."
                    .to_string(),
                warnings: Vec::new(),
            };
        }
    };

    let mut warnings = Vec::new();
    let mut scalar_count = 0usize;
    inspect_yaml_markdown_shape(&yaml, 0, &mut scalar_count, &mut warnings);

    let (status, assessment) = if scalar_count == 0 {
        (
            AssetConversionAuditStatus::NotSafelyConvertible,
            "The YAML does not expose any scalar fields that Diamond can render into stable Markdown definitions."
                .to_string(),
        )
    } else if warnings.is_empty() && !root.is_empty() {
        (
            AssetConversionAuditStatus::Convertible,
            "The YAML matches Diamond's current structured Markdown renderer and is safe to convert."
                .to_string(),
        )
    } else {
        (
            AssetConversionAuditStatus::PartiallyConvertible,
            "The YAML contains useful structure, but some shapes would need manual cleanup or richer renderer support before safe conversion."
                .to_string(),
        )
    };

    PreparedAssetAudit {
        target_path,
        target_kind: AssetKind::Markdown,
        status,
        assessment,
        warnings,
    }
}

fn target_path_with_extension(relative_path: &Path, extension: &str) -> StoreResult<String> {
    let file_name = relative_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| ProjectStoreError::message("Asset filename is invalid."))?;

    if file_name.starts_with('.') {
        return Err(ProjectStoreError::message(
            "Hidden files are not supported for Markdown/YAML conversion.",
        ));
    }

    Ok(relative_path.with_extension(extension).to_string_lossy().replace('\\', "/"))
}

fn convert_markdown_to_yaml(content: &str) -> StoreResult<String> {
    let (frontmatter, entries) = parse_structured_markdown(content)?;
    let mut root = serde_yaml::Mapping::new();

    if let Some(frontmatter) = frontmatter {
        root.insert(
            serde_yaml::Value::String("frontmatter".to_string()),
            frontmatter,
        );
    }

    for entry in &entries {
        root.insert(
            serde_yaml::Value::String(entry.key.clone()),
            structured_value_to_yaml(&entry.value),
        );
    }

    let serialized = serde_yaml::to_string(&serde_yaml::Value::Mapping(root))?;
    Ok(serialized.trim_start_matches("---\n").to_string())
}

fn convert_yaml_to_markdown(content: &str) -> StoreResult<String> {
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(content)?;
    validate_yaml_markdown_shape(&yaml, 0)?;

    let root = yaml
        .as_mapping()
        .ok_or_else(|| ProjectStoreError::message("Only YAML objects can be converted into structured Markdown."))?;

    let mut sections = Vec::new();
    if let Some(frontmatter) = root.get(serde_yaml::Value::String("frontmatter".to_string())) {
        sections.push("---".to_string());
        sections.push(serde_yaml::to_string(frontmatter)?.trim_end().to_string());
        sections.push("---".to_string());
    }

    let mut body = Vec::new();
    for (key, value) in root {
        let key = yaml_string_key(key)?;
        if key == "frontmatter" {
            continue;
        }
        render_yaml_entry_as_markdown(key, value, 2, &mut body)?;
    }

    if body.is_empty() && sections.is_empty() {
        return Err(ProjectStoreError::message(
            "YAML must contain at least one mapping entry to convert into Markdown.",
        ));
    }

    sections.extend(body);
    Ok(format!("{}\n", sections.join("\n\n").trim()))
}

fn parse_structured_markdown(
    content: &str,
) -> StoreResult<(Option<serde_yaml::Value>, Vec<StructuredEntry>)> {
    let normalized = content.replace("\r\n", "\n");
    let mut lines = normalized.lines().peekable();
    let mut frontmatter = None;

    if matches!(lines.peek(), Some(line) if line.trim() == "---") {
        lines.next();
        let mut frontmatter_lines = Vec::new();
        let mut found_closing = false;

        for line in lines.by_ref() {
            if line.trim() == "---" {
                found_closing = true;
                break;
            }
            frontmatter_lines.push(line);
        }

        if !found_closing {
            return Err(ProjectStoreError::message(
                "Markdown frontmatter block was opened with `---` but never closed.",
            ));
        }

        let frontmatter_text = frontmatter_lines.join("\n");
        if !frontmatter_text.trim().is_empty() {
            frontmatter = Some(serde_yaml::from_str::<serde_yaml::Value>(&frontmatter_text)?);
        }
    }

    let mut root_entries = Vec::new();
    let mut heading_path: Vec<String> = Vec::new();
    let mut last_field: Option<(Vec<String>, String)> = None;
    let mut field_count = 0usize;

    for raw_line in lines {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "---" {
            last_field = None;
            continue;
        }

        if let Some((level, heading)) = parse_markdown_heading(trimmed) {
            if level < 2 {
                return Err(ProjectStoreError::message(
                    "Structured Markdown conversion expects section headings to start at `##`.",
                ));
            }

            let depth = level - 2;
            if depth > heading_path.len() {
                return Err(ProjectStoreError::message(format!(
                    "Heading `{heading}` skips a level and cannot be converted safely."
                )));
            }

            heading_path.truncate(depth);
            heading_path.push(heading.to_string());
            ensure_section_path(&mut root_entries, &heading_path)?;
            last_field = None;
            continue;
        }

        if let Some((field, value)) = parse_markdown_field(trimmed) {
            insert_scalar_entry(&mut root_entries, &heading_path, field, value.to_string())?;
            last_field = Some((heading_path.clone(), field.to_string()));
            field_count += 1;
            continue;
        }

        if let Some((path, field)) = &last_field {
            append_scalar_entry(&mut root_entries, path, field, trimmed)?;
            continue;
        }

        return Err(ProjectStoreError::message(format!(
            "Markdown line `{trimmed}` is not a supported heading or `**field:** value` entry."
        )));
    }

    if field_count == 0 && frontmatter.is_none() {
        return Err(ProjectStoreError::message(
            "Markdown file does not contain any structured fields that can be converted to YAML.",
        ));
    }

    Ok((frontmatter, root_entries))
}

fn parse_markdown_heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|character| *character == '#').count();
    if !(2..=6).contains(&level) {
        return None;
    }

    let remainder = line[level..].trim();
    if remainder.is_empty() {
        None
    } else {
        Some((level, remainder))
    }
}

fn parse_any_heading_level(line: &str) -> Option<usize> {
    let level = line.chars().take_while(|character| *character == '#').count();
    if (1..=6).contains(&level) {
        Some(level)
    } else {
        None
    }
}

fn parse_markdown_field(line: &str) -> Option<(&str, &str)> {
    if !line.starts_with("**") {
        return None;
    }

    let remainder = &line[2..];
    let marker = remainder.find(":**")?;
    let field = remainder[..marker].trim();
    if field.is_empty() {
        return None;
    }

    let value = remainder[marker + 3..].trim();
    Some((field, value))
}

fn ensure_section_path<'a>(
    entries: &'a mut Vec<StructuredEntry>,
    path: &[String],
) -> StoreResult<&'a mut Vec<StructuredEntry>> {
    if path.is_empty() {
        return Ok(entries);
    }

    let key = &path[0];
    let position = entries.iter().position(|entry| entry.key == *key);
    let index = match position {
        Some(index) => index,
        None => {
            entries.push(StructuredEntry {
                key: key.clone(),
                value: StructuredValue::Section(Vec::new()),
            });
            entries.len() - 1
        }
    };

    match &mut entries[index].value {
        StructuredValue::Section(children) => ensure_section_path(children, &path[1..]),
        StructuredValue::Scalar(_) => Err(ProjectStoreError::message(format!(
            "Section `{key}` conflicts with an existing field and cannot be converted safely."
        ))),
    }
}

fn insert_scalar_entry(
    entries: &mut Vec<StructuredEntry>,
    path: &[String],
    field: &str,
    value: String,
) -> StoreResult<()> {
    let target = ensure_section_path(entries, path)?;
    if target.iter().any(|entry| entry.key == field) {
        return Err(ProjectStoreError::message(format!(
            "Field `{field}` appears more than once in the same section."
        )));
    }

    target.push(StructuredEntry {
        key: field.to_string(),
        value: StructuredValue::Scalar(value),
    });
    Ok(())
}

fn append_scalar_entry(
    entries: &mut Vec<StructuredEntry>,
    path: &[String],
    field: &str,
    line: &str,
) -> StoreResult<()> {
    let target = ensure_section_path(entries, path)?;
    let entry = target
        .iter_mut()
        .find(|entry| entry.key == field)
        .ok_or_else(|| ProjectStoreError::message(format!(
            "Continuation text was found for `{field}` before the field was created."
        )))?;

    match &mut entry.value {
        StructuredValue::Scalar(existing) => {
            if existing.is_empty() {
                existing.push_str(line);
            } else {
                existing.push('\n');
                existing.push_str(line);
            }
            Ok(())
        }
        StructuredValue::Section(_) => Err(ProjectStoreError::message(format!(
            "Continuation text cannot be attached to section `{field}`."
        ))),
    }
}

fn structured_value_to_yaml(value: &StructuredValue) -> serde_yaml::Value {
    match value {
        StructuredValue::Scalar(text) => serde_yaml::Value::String(text.clone()),
        StructuredValue::Section(entries) => {
            let mut mapping = serde_yaml::Mapping::new();
            for entry in entries {
                mapping.insert(
                    serde_yaml::Value::String(entry.key.clone()),
                    structured_value_to_yaml(&entry.value),
                );
            }
            serde_yaml::Value::Mapping(mapping)
        }
    }
}

fn validate_yaml_markdown_shape(value: &serde_yaml::Value, depth: usize) -> StoreResult<()> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            for (key, child) in mapping {
                if key.as_str().is_none() {
                    return Err(ProjectStoreError::message(
                        "YAML keys must be strings to convert into Markdown.",
                    ));
                }

                if depth >= 5 {
                    return Err(ProjectStoreError::message(
                        "YAML nesting is too deep to render as readable Markdown headings.",
                    ));
                }

                validate_yaml_markdown_shape(child, depth + 1)?;
            }
            Ok(())
        }
        serde_yaml::Value::Sequence(sequence) => {
            if sequence.iter().all(is_yaml_scalar) {
                Ok(())
            } else {
                Err(ProjectStoreError::message(
                    "Only scalar YAML lists can be converted into Markdown safely.",
                ))
            }
        }
        _ => Ok(()),
    }
}

fn inspect_yaml_markdown_shape(
    value: &serde_yaml::Value,
    depth: usize,
    scalar_count: &mut usize,
    warnings: &mut Vec<String>,
) {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            if depth >= 5 {
                warnings.push(
                    "YAML nesting is deeper than Diamond's current Markdown heading renderer supports."
                        .to_string(),
                );
            }

            for (key, child) in mapping {
                if key.as_str().is_none() {
                    warnings.push("YAML keys must be strings for stable Markdown conversion.".to_string());
                }
                inspect_yaml_markdown_shape(child, depth + 1, scalar_count, warnings);
            }
        }
        serde_yaml::Value::Sequence(sequence) => {
            if sequence.iter().all(is_yaml_scalar) {
                *scalar_count += sequence.len();
            } else {
                warnings.push(
                    "Only scalar YAML lists convert cleanly today; nested list items would be lossy."
                        .to_string(),
                );
            }
        }
        serde_yaml::Value::Null
        | serde_yaml::Value::Bool(_)
        | serde_yaml::Value::Number(_)
        | serde_yaml::Value::String(_) => {
            *scalar_count += 1;
        }
        serde_yaml::Value::Tagged(tagged) => inspect_yaml_markdown_shape(&tagged.value, depth, scalar_count, warnings),
    }
}

fn is_markdown_list_item(line: &str) -> bool {
    matches!(line.chars().next(), Some('-' | '*' | '+')) && line.chars().nth(1) == Some(' ')
}

fn is_markdown_table_row(line: &str) -> bool {
    line.contains('|')
}

fn is_yaml_scalar(value: &serde_yaml::Value) -> bool {
    matches!(
        value,
        serde_yaml::Value::Null
            | serde_yaml::Value::Bool(_)
            | serde_yaml::Value::Number(_)
            | serde_yaml::Value::String(_)
    )
}

fn render_yaml_entry_as_markdown(
    key: &str,
    value: &serde_yaml::Value,
    heading_level: usize,
    body: &mut Vec<String>,
) -> StoreResult<()> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            body.push(format!("{} {}", "#".repeat(heading_level), key));
            for (child_key, child_value) in mapping {
                let child_key = yaml_string_key(child_key)?;
                render_yaml_entry_as_markdown(child_key, child_value, heading_level + 1, body)?;
            }
            Ok(())
        }
        _ => {
            body.push(render_markdown_field(key, &yaml_value_to_markdown_text(value)?));
            Ok(())
        }
    }
}

fn render_markdown_field(key: &str, value: &str) -> String {
    if let Some((first_line, remaining)) = value.split_once('\n') {
        format!("**{key}:** {first_line}\n{remaining}")
    } else {
        format!("**{key}:** {value}")
    }
}

fn yaml_value_to_markdown_text(value: &serde_yaml::Value) -> StoreResult<String> {
    match value {
        serde_yaml::Value::Null => Ok(String::new()),
        serde_yaml::Value::Bool(boolean) => Ok(boolean.to_string()),
        serde_yaml::Value::Number(number) => Ok(number.to_string()),
        serde_yaml::Value::String(text) => Ok(text.clone()),
        serde_yaml::Value::Sequence(sequence) => Ok(sequence
            .iter()
            .map(|entry| format!("- {}", yaml_scalar_to_string(entry)))
            .collect::<Vec<_>>()
            .join("\n")),
        serde_yaml::Value::Mapping(_) => Err(ProjectStoreError::message(
            "Nested YAML objects must be rendered as headings, not inline field values.",
        )),
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_markdown_text(&tagged.value),
    }
}

fn yaml_scalar_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::Null => String::new(),
        serde_yaml::Value::Bool(boolean) => boolean.to_string(),
        serde_yaml::Value::Number(number) => number.to_string(),
        serde_yaml::Value::String(text) => text.clone(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    }
}

fn yaml_string_key(value: &serde_yaml::Value) -> StoreResult<&str> {
    value
        .as_str()
        .ok_or_else(|| ProjectStoreError::message("YAML keys must be strings to convert into Markdown."))
}

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
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn converts_structured_markdown_copy_to_yaml() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = PathBuf::from(&summary.root_path);

        fs::write(
            root_path.join("documents/worksheet.md"),
            "---\nfile: worksheet.md\n---\n\n## section_1\n\n### required_data\n\n**series_position:** Standalone.\n**target_chapter_count:** 28\n",
        )
        .unwrap();

        let result = convert_project_asset(&root_path, "documents/worksheet.md").unwrap();

        assert_eq!(result.target_path, "documents/worksheet.yaml");
        assert_eq!(result.target_kind, AssetKind::Yaml);
        assert!(root_path.join("documents/worksheet.yaml").is_file());
        assert!(result.asset.content.contains("frontmatter:"));
        assert!(result.asset.content.contains("series_position: Standalone."));
    }

    #[test]
    fn converts_yaml_copy_to_markdown() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = PathBuf::from(&summary.root_path);

        fs::write(
            root_path.join("documents/worksheet.yaml"),
            "frontmatter:\n  file: worksheet.yaml\nsection_1:\n  required_data:\n    series_position: Standalone.\n    target_chapter_count: 28\n",
        )
        .unwrap();

        let result = convert_project_asset(&root_path, "documents/worksheet.yaml").unwrap();

        assert_eq!(result.target_path, "documents/worksheet.md");
        assert_eq!(result.target_kind, AssetKind::Markdown);
        assert!(root_path.join("documents/worksheet.md").is_file());
        assert!(result.asset.content.contains("## section_1"));
        assert!(result.asset.content.contains("**series_position:** Standalone."));
    }

    #[test]
    fn rejects_unstructured_markdown_conversion() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = PathBuf::from(&summary.root_path);

        fs::write(
            root_path.join("documents/freeform.md"),
            "# Freeform\n\nThis paragraph has no structured fields.\n",
        )
        .unwrap();

        let error = convert_project_asset(&root_path, "documents/freeform.md").unwrap_err();

        assert!(error
            .to_string()
            .contains("cannot safely convert this Markdown"));
    }

    #[test]
    fn audits_structured_markdown_as_convertible() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = PathBuf::from(&summary.root_path);

        fs::write(
            root_path.join("documents/worksheet.md"),
            "## section_1\n\n### required_data\n\n**series_position:** Standalone.\n",
        )
        .unwrap();

        let audit = audit_project_asset(&root_path, "documents/worksheet.md").unwrap();

        assert_eq!(audit.status, AssetConversionAuditStatus::Convertible);
        assert!(audit.warnings.is_empty());
    }

    #[test]
    fn audits_markdown_with_title_and_list_as_partially_convertible() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = PathBuf::from(&summary.root_path);

        fs::write(
            root_path.join("documents/worksheet.md"),
            "# Worksheet\n\n## section_1\n\n### required_data\n\n**series_position:** Standalone.\n- fake engagement\n",
        )
        .unwrap();

        let audit = audit_project_asset(&root_path, "documents/worksheet.md").unwrap();

        assert_eq!(audit.status, AssetConversionAuditStatus::PartiallyConvertible);
        assert!(!audit.warnings.is_empty());
    }

    #[test]
    fn audits_freeform_markdown_as_not_safely_convertible() {
        let temp = tempdir().unwrap();
        let app_data_dir = temp.path().join("app-data");
        let summary = super::super::create_project(temp.path(), "Story Lab", &app_data_dir).unwrap();
        let root_path = PathBuf::from(&summary.root_path);

        fs::write(
            root_path.join("documents/freeform.md"),
            "# Freeform\n\nThis paragraph has no structured fields.\n",
        )
        .unwrap();

        let audit = audit_project_asset(&root_path, "documents/freeform.md").unwrap();

        assert_eq!(audit.status, AssetConversionAuditStatus::NotSafelyConvertible);
    }

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
