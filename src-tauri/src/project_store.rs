use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use chrono::{SecondsFormat, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
// tera used via execution submodule
use uuid::Uuid;

pub use crate::types::*;

pub(crate) mod assets;
pub use assets::{
    audit_project_asset, convert_project_asset, list_project_assets, read_project_asset, write_project_asset,
    export_project_assets, delete_document, rename_document, trash_prompt,
};
pub(crate) use assets::classify_asset;

pub(crate) mod credentials;
pub use credentials::{get_execution_credential_status, save_execution_api_key, clear_execution_api_key};

pub(crate) mod execution;
pub use execution::{validate_project_template, execute_prompt_block, execute_pipeline};

#[cfg(test)]
use serde_json::Value;
#[cfg(test)]
use execution::{
    PersistedRunRecord, extract_usage_metrics,
    execute_prompt_block_with_transport,
};

pub(crate) mod history;
pub use history::{list_prompt_run_history, list_project_run_history, delete_run, get_project_usage_summary};

pub(crate) mod presets;
pub use presets::{list_model_presets, set_default_model_preset, create_model_preset, delete_model_preset, set_block_model_preset};

pub(crate) mod variables;
pub use variables::{get_global_variables, set_global_variables, set_project_variables};
pub(crate) use variables::{read_global_variables_store, read_workspace_variables_yaml, write_workspace_variables_yaml};

const PROJECT_DIRS: [&str; 8] = ["documents", "prompts", "pipelines", "models", "runs", "exports", "help", "variables"];
const RECENTS_FILE_NAME: &str = "recent-projects.json";
const SEEDED_MODEL_PRESETS: [(&str, &str); 5] = [
    (
        "default.yaml",
        "# Default fallback model used when no stronger override applies.\nmodel: openai/gpt-5.4\nreasoning:\n  effort: high\ntemperature: 1\nmax_completion_tokens: 128000\n",
    ),
    (
        "gpt-5.4.yaml",
        "# Strong default drafting and final-pass model.\nmodel: openai/gpt-5.4\nreasoning:\n  effort: high\ntemperature: 1\nmax_completion_tokens: 128000\n",
    ),
    (
        "gpt-5.4-nano.yaml",
        "# Cheap, rule-following model reserved primarily for test runs.\nmodel: openai/gpt-5.4-nano\ntemperature: 0.3\nmax_completion_tokens: 8000\n",
    ),
    (
        "claude-sonnet-4.6.yaml",
        "# Strong controlled prose and planning candidate from the workshop presets.\nmodel: anthropic/claude-sonnet-4.6\ntemperature: 0.9\nmax_completion_tokens: 8000\n",
    ),
    (
        "gpt-5.2-think.yaml",
        "# Higher-effort planning model carried forward from the workshop routing.\nmodel: openai/gpt-5.2\nreasoning:\n  effort: high\ntemperature: 1\nmax_completion_tokens: 65000\n",
    ),
];
// Execution-specific constants (URL, retries, online) moved to execution.rs
// Credential constants moved to credentials.rs
const DEFAULT_PROMPT_TEMPLATE: &str = "Project: {{ project.name }}\nDate: {{ current_date }}\n\nWrite the instructions for this prompt block here.\n";

#[derive(Debug, Default, Serialize, Deserialize)]
struct RecentsStore {
    projects: Vec<RecentProjectEntry>,
}

// GlobalVariablesStore moved to variables.rs

pub fn create_project(parent_path: &Path, project_name: &str, app_data_dir: &Path) -> StoreResult<ProjectSummary> {
    if !parent_path.is_dir() {
        return Err(ProjectStoreError::message("The selected parent folder does not exist."));
    }

    let trimmed_name = project_name.trim();
    if trimmed_name.is_empty() {
        return Err(ProjectStoreError::message("Project name cannot be empty."));
    }

    if trimmed_name.contains('/') || trimmed_name.contains('\\') {
        return Err(ProjectStoreError::message("Project name cannot contain path separators."));
    }

    let root_path = parent_path.join(trimmed_name);
    if root_path.exists() {
        return Err(ProjectStoreError::message("A folder with that project name already exists."));
    }

    fs::create_dir_all(&root_path)?;
    for directory in PROJECT_DIRS {
        fs::create_dir_all(root_path.join(directory))?;
    }

    write_seeded_model_presets(&root_path)?;
    write_seeded_help_files(&root_path)?;
    write_workspace_variables_yaml(&root_path, &BTreeMap::new())?;

    let now = timestamp();
    let manifest = ProjectManifest {
        project_id: Uuid::new_v4().to_string(),
        project_name: trimmed_name.to_string(),
        created_at: now.clone(),
        updated_at: now,
        default_model_preset: "models/default.yaml".to_string(),
        variables: BTreeMap::new(),
        prompt_blocks: Vec::new(),
        pipelines: Vec::new(),
        ui_preferences: json!({
            "theme": "ide-pass",
            "density": "dense"
        }),
    };

    write_manifest(&root_path, &manifest)?;
    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;
    Ok(summary)
}

pub fn open_project(root_path: &Path, app_data_dir: &Path) -> StoreResult<ProjectSummary> {
    let (path, manifest) = validate_project(root_path)?;
    let summary = summarize_project(&path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;
    Ok(summary)
}

pub fn create_prompt_block(
    root_path: &Path,
    prompt_name: &str,
    app_data_dir: &Path,
) -> StoreResult<CreatedPromptBlockResult> {
    let trimmed_name = prompt_name.trim();
    if trimmed_name.is_empty() {
        return Err(ProjectStoreError::message("Prompt name cannot be empty."));
    }

    let (root_path, mut manifest) = validate_project(root_path)?;
    let prompt_slug = unique_prompt_slug(&root_path, &manifest, trimmed_name);
    let prompt_path = format!("prompts/{prompt_slug}.tera");
    let full_path = root_path.join(&prompt_path);

    fs::write(&full_path, DEFAULT_PROMPT_TEMPLATE)?;

    manifest.prompt_blocks.push(PromptBlock {
        block_id: prompt_slug.clone(),
        name: trimmed_name.to_string(),
        template_source: prompt_path.clone(),
        input_bindings: Vec::new(),
        model_preset: None,
        output_target: "replace_document".to_string(),
        output_filename: None,
    });
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(CreatedPromptBlockResult {
        summary,
        path: prompt_path,
    })
}

pub fn register_prompt_block(
    root_path: &Path,
    template_source: &str,
    app_data_dir: &Path,
) -> StoreResult<CreatedPromptBlockResult> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    let safe_template = sanitize_relative_path(template_source)?;
    let safe_template_string = safe_template.to_string_lossy().replace('\\', "/");

    if !safe_template_string.starts_with("prompts/") || !safe_template_string.ends_with(".tera") {
        return Err(ProjectStoreError::message(
            "Only existing `.tera` files inside `prompts/` can be registered as prompt blocks.",
        ));
    }

    let full_path = root_path.join(&safe_template_string);
    if !full_path.is_file() {
        return Err(ProjectStoreError::message(format!(
            "Prompt template `{safe_template_string}` was not found."
        )));
    }

    if manifest
        .prompt_blocks
        .iter()
        .any(|block| block.template_source == safe_template_string)
    {
        return Err(ProjectStoreError::message(format!(
            "Prompt template `{safe_template_string}` is already registered as a prompt block."
        )));
    }

    let file_stem = full_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| ProjectStoreError::message("Prompt template filename is invalid."))?;
    let base_block_id = slugify_prompt_name(file_stem);
    let mut block_id = base_block_id.clone();
    let mut suffix = 2usize;

    while manifest.prompt_blocks.iter().any(|block| block.block_id == block_id) {
        block_id = format!("{base_block_id}-{suffix}");
        suffix += 1;
    }

    let display_name = file_stem
        .split(['-', '_'])
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    manifest.prompt_blocks.push(PromptBlock {
        block_id,
        name: if display_name.is_empty() {
            file_stem.to_string()
        } else {
            display_name
        },
        template_source: safe_template_string.clone(),
        input_bindings: Vec::new(),
        model_preset: None,
        output_target: "replace_document".to_string(),
        output_filename: None,
    });
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(CreatedPromptBlockResult {
        summary,
        path: safe_template_string,
    })
}

pub fn delete_prompt_block(
    root_path: &Path,
    block_id: &str,
    app_data_dir: &Path,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    let before_len = manifest.prompt_blocks.len();
    manifest.prompt_blocks.retain(|b| b.block_id != block_id);
    if manifest.prompt_blocks.len() == before_len {
        return Err(ProjectStoreError::message(format!(
            "Prompt block `{block_id}` was not found."
        )));
    }
    // Remove the deleted block from any pipeline that references it.
    for pipeline in &mut manifest.pipelines {
        pipeline.ordered_blocks.retain(|id| id != block_id);
    }
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;
    Ok(summary)
}

pub fn get_recent_projects(app_data_dir: &Path) -> StoreResult<Vec<RecentProjectEntry>> {
    let recents = read_recents_store(app_data_dir)?;

    Ok(recents
        .projects
        .into_iter()
        .map(|mut project| {
            project.last_known_valid = validate_project(Path::new(&project.summary.root_path)).is_ok();
            project
        })
        .collect())
}

pub fn remove_recent_project(app_data_dir: &Path, root_path: &Path) -> StoreResult<()> {
    fs::create_dir_all(app_data_dir)?;
    let mut store = read_recents_store(app_data_dir)?;
    let target = root_path.to_string_lossy().to_string();

    store.projects.retain(|project| project.summary.root_path != target);

    let store_path = app_data_dir.join(RECENTS_FILE_NAME);
    fs::write(store_path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}

// get_global_variables, set_global_variables, set_project_variables → variables.rs

// list_model_presets, set_default_model_preset, create_model_preset, delete_model_preset, set_block_model_preset → presets.rs

pub fn set_block_output_target(
    root_path: &Path,
    block_id: &str,
    target: &str,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;

    let valid_targets = ["replace_document", "append_document", "run_artifact", "document", "both", "history_only"];
    if !valid_targets.contains(&target) {
        return Err(ProjectStoreError::message(format!(
            "Invalid output target '{}'",
            target
        )));
    }

    let block = manifest
        .prompt_blocks
        .iter_mut()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| {
            ProjectStoreError::message(format!("Prompt block not found: {}", block_id))
        })?;

    block.output_target = target.to_string();
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    summarize_project(&root_path, &manifest)
}

pub fn set_block_output_filename(
    root_path: &Path,
    block_id: &str,
    filename: Option<&str>,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;

    let block = manifest
        .prompt_blocks
        .iter_mut()
        .find(|b| b.block_id == block_id)
        .ok_or_else(|| {
            ProjectStoreError::message(format!("Prompt block not found: {}", block_id))
        })?;

    block.output_filename = match filename {
        Some(name) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                None
            } else {
                // Ensure the filename ends with .md
                let safe_name = if trimmed.ends_with(".md") {
                    trimmed.to_string()
                } else {
                    format!("{trimmed}.md")
                };
                // Reject path separators to prevent directory traversal
                if safe_name.contains('/') || safe_name.contains('\\') {
                    return Err(ProjectStoreError::message(
                        "Output filename cannot contain path separators.",
                    ));
                }
                Some(safe_name)
            }
        }
        None => None,
    };
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    summarize_project(&root_path, &manifest)
}

pub fn rename_project(
    root_path: &Path,
    new_name: &str,
    app_data_dir: &Path,
) -> StoreResult<ProjectSummary> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::message("Project name cannot be empty."));
    }
    let (root_path, mut manifest) = validate_project(root_path)?;
    manifest.project_name = trimmed.to_string();
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    // Also update the recents store so the project browser shows the new name
    fs::create_dir_all(app_data_dir)?;
    let mut recents = read_recents_store(app_data_dir)?;
    let root_str = root_path.to_string_lossy().to_string();
    if let Some(entry) = recents
        .projects
        .iter_mut()
        .find(|p| p.summary.root_path == root_str)
    {
        entry.summary.project_name = trimmed.to_string();
        let store_path = app_data_dir.join(RECENTS_FILE_NAME);
        fs::write(store_path, serde_json::to_string_pretty(&recents)?)?;
    }

    summarize_project(&root_path, &manifest)
}

pub fn locate_recent_project(
    app_data_dir: &Path,
    previous_root_path: &Path,
    candidate_root_path: &Path,
) -> StoreResult<ProjectSummary> {
    fs::create_dir_all(app_data_dir)?;
    let mut store = read_recents_store(app_data_dir)?;
    let previous_root = previous_root_path.to_string_lossy().to_string();

    let Some(index) = store
        .projects
        .iter()
        .position(|project| project.summary.root_path == previous_root)
    else {
        return Err(ProjectStoreError::message(
            "The selected recent project entry no longer exists.",
        ));
    };

    let expected_project_id = store.projects[index].summary.project_id.clone();
    let (validated_root, manifest) = validate_project(candidate_root_path)?;

    if manifest.project_id != expected_project_id {
        return Err(ProjectStoreError::message(
            "The selected folder is a valid Diamond project, but it does not match the missing recent project.",
        ));
    }

    let summary = summarize_project(&validated_root, &manifest)?;
    store.projects.remove(index);
    store.projects.insert(
        0,
        RecentProjectEntry {
            summary: summary.clone(),
            last_opened_at: timestamp(),
            last_known_valid: true,
        },
    );
    store.projects.truncate(8);

    let store_path = app_data_dir.join(RECENTS_FILE_NAME);
    fs::write(store_path, serde_json::to_string_pretty(&store)?)?;
    Ok(summary)
}

// list_project_assets, read_project_asset, write_project_asset → assets.rs
// export_project_assets, delete_document, rename_document → assets.rs

pub fn list_project_pipelines(root_path: &Path) -> StoreResult<Vec<ProjectPipelineSummary>> {
    let (root_path, manifest) = validate_project(root_path)?;
    let mut pipelines_list = vec![];
    let pipelines_dir = root_path.join("pipelines");

    if pipelines_dir.is_dir() {
        for entry in std::fs::read_dir(pipelines_dir)?.flatten() {
            if entry.path().is_file() && entry.path().extension().is_some_and(|e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if let Ok(pipeline) = serde_json::from_str::<Pipeline>(&content) {
                        pipelines_list.push(summarize_pipeline(&manifest, &pipeline));
                    }
                }
            }
        }
    }
    
    pipelines_list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(pipelines_list)
}

pub fn list_project_prompt_blocks(root_path: &Path) -> StoreResult<Vec<ProjectPromptBlockSummary>> {
    let (_, manifest) = validate_project(root_path)?;

    Ok(manifest
        .prompt_blocks
        .iter()
        .map(|block| ProjectPromptBlockSummary {
            block_id: block.block_id.clone(),
            name: block.name.clone(),
            template_source: block.template_source.clone(),
            model_preset: block
                .model_preset
                .clone()
                .unwrap_or_else(|| manifest.default_model_preset.clone()),
            output_target: block.output_target.clone(),
            output_filename: block.output_filename.clone(),
        })
        .collect())
}

pub fn create_pipeline(
    root_path: &Path,
    pipeline_name: &str,
    ordered_block_ids: &[String],
    app_data_dir: &Path,
) -> StoreResult<SavedPipelineResult> {
    let trimmed_name = pipeline_name.trim();
    if trimmed_name.is_empty() {
        return Err(ProjectStoreError::message("Pipeline name cannot be empty."));
    }

    let (root_path, mut manifest) = validate_project(root_path)?;
    validate_pipeline_block_ids(&manifest, ordered_block_ids, None)?;

    let pipeline_id = unique_pipeline_slug(&root_path, trimmed_name);
    let pipeline = Pipeline {
        pipeline_id: pipeline_id.clone(),
        name: trimmed_name.to_string(),
        ordered_blocks: ordered_block_ids.to_vec(),
        execution_mode: "sequential".to_string(),
    };
    let pipeline_path = root_path.join("pipelines").join(format!("{}.json", pipeline_id));
    std::fs::write(pipeline_path, serde_json::to_string_pretty(&pipeline).unwrap_or_default())?;

    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(SavedPipelineResult { summary, pipeline_id })
}

pub fn update_pipeline(
    root_path: &Path,
    pipeline_id: &str,
    pipeline_name: &str,
    ordered_block_ids: &[String],
    app_data_dir: &Path,
) -> StoreResult<SavedPipelineResult> {
    let trimmed_name = pipeline_name.trim();
    if trimmed_name.is_empty() {
        return Err(ProjectStoreError::message("Pipeline name cannot be empty."));
    }

    let (root_path, mut manifest) = validate_project(root_path)?;
    validate_pipeline_block_ids(&manifest, ordered_block_ids, Some(pipeline_id))?;

    let pipeline_path = root_path.join("pipelines").join(format!("{}.json", pipeline_id));
    if !pipeline_path.exists() {
        return Err(ProjectStoreError::message(format!(
            "Pipeline `{pipeline_id}` was not found."
        )));
    }

    let mut pipeline: Pipeline = serde_json::from_str(&std::fs::read_to_string(&pipeline_path)?)?;
    pipeline.name = trimmed_name.to_string();
    pipeline.ordered_blocks = ordered_block_ids.to_vec();
    pipeline.execution_mode = "sequential".to_string();
    std::fs::write(&pipeline_path, serde_json::to_string_pretty(&pipeline)?)?;

    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(SavedPipelineResult {
        summary,
        pipeline_id: pipeline_id.to_string(),
    })
}

pub fn duplicate_pipeline(
    root_path: &Path,
    pipeline_id: &str,
    app_data_dir: &Path,
) -> StoreResult<SavedPipelineResult> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    let source_path = root_path.join("pipelines").join(format!("{}.json", pipeline_id));
    if !source_path.exists() {
        return Err(ProjectStoreError::message(format!(
            "Pipeline `{pipeline_id}` was not found."
        )));
    }
    let source: Pipeline = serde_json::from_str(&std::fs::read_to_string(&source_path)?)?;
    let new_name = format!("Copy of {}", source.name);
    let new_id = unique_pipeline_slug(&root_path, &new_name);
    let new_pipeline = Pipeline {
        pipeline_id: new_id.clone(),
        name: new_name,
        ordered_blocks: source.ordered_blocks,
        execution_mode: source.execution_mode,
    };
    let new_path = root_path.join("pipelines").join(format!("{}.json", new_id));
    std::fs::write(new_path, serde_json::to_string_pretty(&new_pipeline)?)?;

    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(SavedPipelineResult {
        summary,
        pipeline_id: new_id,
    })
}

pub fn delete_pipeline(
    root_path: &Path,
    pipeline_id: &str,
    app_data_dir: &Path,
) -> StoreResult<ProjectSummary> {
    let (root_path, mut manifest) = validate_project(root_path)?;
    let pipeline_path = root_path.join("pipelines").join(format!("{}.json", pipeline_id));
    if pipeline_path.exists() {
        std::fs::remove_file(pipeline_path)?;
    } else {
        return Err(ProjectStoreError::message(format!(
            "Pipeline `{pipeline_id}` was not found."
        )));
    }
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;
    Ok(summary)
}

// export_project_assets, read_project_asset, write_project_asset → assets.rs

// validate_project_template, execute_prompt_block, execute_pipeline → execution.rs
// get_execution_credential_status, save_execution_api_key, clear_execution_api_key → credentials.rs
// list_prompt_run_history, list_project_run_history, delete_run, get_project_usage_summary → history.rs


fn summarize_pipeline(manifest: &ProjectManifest, pipeline: &Pipeline) -> ProjectPipelineSummary {
    let blocks = pipeline
        .ordered_blocks
        .iter()
        .map(|block_id| {
            if let Some(block) = manifest.prompt_blocks.iter().find(|block| &block.block_id == block_id) {
                ProjectPipelineBlockSummary {
                    block_id: block.block_id.clone(),
                    name: block.name.clone(),
                    template_source: block.template_source.clone(),
                    model_preset: block
                        .model_preset
                        .clone()
                        .unwrap_or_else(|| manifest.default_model_preset.clone()),
                    output_target: block.output_target.clone(),
                    output_filename: block.output_filename.clone(),
                }
            } else {
                ProjectPipelineBlockSummary {
                    block_id: block_id.clone(),
                    name: block_id.clone(),
                    template_source: String::new(),
                    model_preset: manifest.default_model_preset.clone(),
                    output_target: "replace_document".to_string(),
                    output_filename: None,
                }
            }
        })
        .collect();

    ProjectPipelineSummary {
        pipeline_id: pipeline.pipeline_id.clone(),
        name: pipeline.name.clone(),
        execution_mode: pipeline.execution_mode.clone(),
        blocks,
    }
}

// build_tree_node, display_name_for_asset, display_name_for_run_record, is_hidden_ui_entry → assets.rs

pub(crate) fn summarize_project(root_path: &Path, manifest: &ProjectManifest) -> StoreResult<ProjectSummary> {
    let counts = ProjectCounts {
        documents: count_files(root_path.join("documents"))?,
        prompts: count_files(root_path.join("prompts"))?,
        models: count_files(root_path.join("models"))?,
        runs: count_files(root_path.join("runs"))?,
        exports: count_files(root_path.join("exports"))?,
        help: count_files(root_path.join("help"))?,
        variables: count_files(root_path.join("variables"))?,
    };

    // Read from YAML file first; fall back to project.json manifest
    let yaml_vars = read_workspace_variables_yaml(root_path);
    let variables = if !yaml_vars.is_empty() {
        yaml_vars
    } else {
        manifest
            .variables
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect()
    };

    Ok(ProjectSummary {
        root_path: root_path.to_string_lossy().to_string(),
        project_id: manifest.project_id.clone(),
        project_name: manifest.project_name.clone(),
        default_model_preset: manifest.default_model_preset.clone(),
        updated_at: manifest.updated_at.clone(),
        counts,
        variables,
    })
}

pub(crate) fn validate_project(root_path: &Path) -> StoreResult<(PathBuf, ProjectManifest)> {
    if !root_path.exists() || !root_path.is_dir() {
        return Err(ProjectStoreError::message("The selected project folder does not exist."));
    }

    let root_path = root_path.canonicalize()?;
    for directory in PROJECT_DIRS {
        let directory_path = root_path.join(directory);
        if !directory_path.is_dir() {
            // Auto-create directories that were added after initial project creation
            if directory == "help" || directory == "variables" || directory == "pipelines" {
                fs::create_dir_all(&directory_path)?;
            } else {
                return Err(ProjectStoreError::message(format!(
                    "Project folder is missing required directory `{directory}`."
                )));
            }
        }
    }

    let manifest_path = root_path.join("project.json");
    if !manifest_path.is_file() {
        return Err(ProjectStoreError::message("Project folder is missing `project.json`."));
    }

    let mut manifest = read_manifest(&manifest_path)?;

    // Legacy Pipeline Migration
    if !manifest.pipelines.is_empty() {
        for pipeline in manifest.pipelines.drain(..) {
            let pipeline_path = root_path.join("pipelines").join(format!("{}.json", pipeline.pipeline_id));
            std::fs::write(pipeline_path, serde_json::to_string_pretty(&pipeline).unwrap_or_default())?;
        }
        write_manifest(&root_path, &manifest)?;
    }
    if manifest.project_id.trim().is_empty()
        || manifest.project_name.trim().is_empty()
        || manifest.default_model_preset.trim().is_empty()
    {
        return Err(ProjectStoreError::message(
            "Project manifest is missing required milestone-1 fields.",
        ));
    }

    if !root_path.join(&manifest.default_model_preset).is_file() {
        return Err(ProjectStoreError::message(
            "The manifest default model preset does not exist on disk.",
        ));
    }

    Ok((root_path, manifest))
}

// build_metadata → assets.rs

pub(crate) fn sanitize_relative_path(relative_path: &str) -> StoreResult<PathBuf> {
    let path = Path::new(relative_path);

    if path.is_absolute() {
        return Err(ProjectStoreError::message("Absolute paths are not allowed."));
    }

    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(ProjectStoreError::message("Parent directory traversal is not allowed."));
        }
    }

    Ok(path.to_path_buf())
}

pub(crate) fn update_recent_projects(app_data_dir: &Path, summary: &ProjectSummary) -> StoreResult<()> {
    fs::create_dir_all(app_data_dir)?;
    let mut store = read_recents_store(app_data_dir)?;
    let now = timestamp();

    store.projects.retain(|project| project.summary.root_path != summary.root_path);
    store.projects.insert(
        0,
        RecentProjectEntry {
            summary: summary.clone(),
            last_opened_at: now,
            last_known_valid: true,
        },
    );
    store.projects.truncate(8);

    let store_path = app_data_dir.join(RECENTS_FILE_NAME);
    fs::write(store_path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}

fn write_seeded_model_presets(root_path: &Path) -> StoreResult<()> {
    for (file_name, content) in SEEDED_MODEL_PRESETS {
        fs::write(root_path.join("models").join(file_name), content)?;
    }

    Ok(())
}

fn write_seeded_help_files(root_path: &Path) -> StoreResult<()> {
    let help_dir = root_path.join("help");
    fs::write(
        help_dir.join("Tera Template Syntax.md"),
        r#"# Tera Template Syntax

Diamond Runner uses the Tera template engine (similar to Jinja2) for prompt files.
This guide covers everything available inside `.tera` prompt templates.

---

## Delimiters

| Syntax       | Purpose                          |
|--------------|----------------------------------|
| `{{ }}`      | Output a value (expression)      |
| `{% %}`      | Control flow (if, for, set, etc.)|
| `{# #}`      | Comment (ignored at render time) |

---

## Available Context Variables

These are injected automatically when Diamond renders a template:

| Variable                 | Type   | Description                                |
|--------------------------|--------|--------------------------------------------|
| `project.name`           | string | The project name from project.json         |
| `project.id`             | string | The project UUID                           |
| `project.default_model_preset` | string | Path to default model preset        |
| `model_id`               | string | The model ID that will run this block      |
| `current_date`           | string | Today's date as `YYYY-MM-DD`               |
| `now_iso`                | string | Current UTC timestamp in ISO 8601          |

### User-Defined Variables

Any variable you set in **Variables** (global or project) becomes a top-level template variable.

```
{# If you set genre = "fantasy" in Variables: #}
Genre: {{ genre }}
```

Global variables are available via `global_variables.name`, and project variables via `variables.name`. But both are also promoted to top-level names, so `{{ genre }}` works directly. Project variables override globals of the same name.

---

## Expressions: `{{ }}`

Output any variable or computed value:

```
Project: {{ project.name }}
Date: {{ current_date }}
Model: {{ model_id }}
```

### Dot Notation and Brackets

```
{{ project.name }}
{{ project["name"] }}
```

### String Concatenation

Use `~` to join strings (not `+`):

```
{{ "Chapter " ~ chapter_number ~ ": " ~ chapter_title }}
```

---

## Inlining Documents: `doc()`

The `doc()` function reads a file from your project's `documents/` folder and inlines its full content:

```
{{ doc("worldbuilding.md") }}
```

This resolves to `documents/worldbuilding.md` in your project. Use it to inject reference material, character sheets, outlines, or any supporting document into a prompt.

### Subdirectories

Organize documents in subfolders and reference them with paths:

```
{{ doc("characters/protagonist.md") }}
{{ doc("worldbuilding/magic-system.md") }}
{{ doc("outline/act-1.md") }}
```

### Multiple Documents

Combine several documents in a single prompt:

```
## World Context
{{ doc("worldbuilding.md") }}

## Character Sheet
{{ doc("characters/elena.md") }}

## Scene Outline
{{ doc("outline/chapter-5.md") }}

Now write Chapter 5 based on the above context.
```

### What Happens When a Document Is Missing

- **Validation (preview):** Shows a warning but still renders: `[Missing document: filename.md]`
- **Execution (run):** Fails with an error — all `doc()` references must resolve

---

## Filters

Transform values with the `|` pipe. Filters chain left to right:

```
{{ name | upper }}
{{ biography | truncate(length=200) }}
{{ title | lower | replace(from=" ", to="-") }}
```

### Useful Filters

| Filter      | Example                                      | Result                |
|-------------|----------------------------------------------|-----------------------|
| `upper`     | `{{ "hello" \| upper }}`                     | `HELLO`               |
| `lower`     | `{{ "HELLO" \| lower }}`                     | `hello`               |
| `title`     | `{{ "dark forest" \| title }}`               | `Dark Forest`         |
| `trim`      | `{{ value \| trim }}`                        | strips whitespace     |
| `length`    | `{{ items \| length }}`                      | count of items        |
| `default`   | `{{ tone \| default(value="neutral") }}`     | fallback if undefined |
| `replace`   | `{{ text \| replace(from="X", to="Y") }}`   | find and replace      |
| `truncate`  | `{{ text \| truncate(length=100) }}`         | cut with ellipsis     |
| `wordcount` | `{{ text \| wordcount }}`                    | number of words       |
| `join`      | `{{ tags \| join(sep=", ") }}`               | array to string       |

---

## Control Flow: `{% %}`

### If / Elif / Else

```
{% if genre == "fantasy" %}
Include magic system rules.
{% elif genre == "thriller" %}
Include pacing guidelines.
{% else %}
Use general fiction defaults.
{% endif %}
```

### Checking if a Variable Exists

```
{% if character_name is defined %}
Focus character: {{ character_name }}
{% endif %}
```

### For Loops

Useful for iterating over lists if you set array-valued variables:

```
{% for chapter in chapters %}
{{ loop.index }}. {{ chapter }}
{% endfor %}
```

Loop variables:

| Variable       | Description                     |
|----------------|---------------------------------|
| `loop.index`   | Current iteration (1-based)     |
| `loop.index0`  | Current iteration (0-based)     |
| `loop.first`   | `true` on first iteration       |
| `loop.last`    | `true` on last iteration        |

Empty fallback:

```
{% for item in items %}
  {{ item }}
{% else %}
  No items provided.
{% endfor %}
```

---

## Assignments

Set local variables inside a template:

```
{% set word_target = 2000 %}
{% set scene_label = "Chapter " ~ chapter_number ~ ", Scene " ~ scene_number %}

Write {{ word_target }} words for {{ scene_label }}.
```

---

## Comments

Comments are stripped from the rendered output:

```
{# This note is for the author, not sent to the LLM #}
{# TODO: add character backstory doc reference #}
```

---

## Whitespace Control

Add `-` inside delimiters to trim surrounding whitespace:

```
{% set x = 42 -%}
Value: {{ x }}
```

Without `-%}`, there would be a blank line before `Value:`.

---

## Raw Blocks

Prevent Tera from parsing content (useful if you need literal `{{ }}` in output):

```
{% raw %}
Use {{ variable }} syntax in your templates.
{% endraw %}
```

---

## Typical Prompt Pattern

A complete prompt template pulling together variables, documents, and control flow:

```
{# Scene drafting prompt — Chapter {{ chapter_number }} #}
Project: {{ project.name }}
Date: {{ current_date }}
Target: {{ word_count | default(value="2000") }} words

## World Context
{{ doc("worldbuilding.md") }}

## Character Sheet
{{ doc("characters/protagonist.md") }}

{% if outline is defined %}
## Scene Outline
{{ outline }}
{% endif %}

## Instructions
Write Chapter {{ chapter_number }}, Scene {{ scene_number }}.
Genre: {{ genre | default(value="literary fiction") }}
POV: {{ pov | default(value="third person limited") }}
Tone: {{ tone | default(value="measured, observational") }}
```
"#,
    )?;
    fs::write(
        help_dir.join("Prompt Blocks.md"),
        "# Prompt Blocks\n\nA prompt block registers a `.tera` template as an executable unit with an optional model override and output strategy.\n",
    )?;
    fs::write(
        help_dir.join("Pipelines.md"),
        "# Pipelines\n\nPipelines chain multiple prompt blocks into a sequential workflow, executing each block in order.\n",
    )?;
    fs::write(
        help_dir.join("Variables.md"),
        "# Variables\n\nGlobal variables apply across all projects. Project variables are scoped to a single workspace and override globals of the same name.\n",
    )?;
    fs::write(
        help_dir.join("Model Presets.md"),
        "# Model Presets\n\nYAML files under `models/` define model configurations. The project default preset is set in Settings; individual blocks can override it.\n",
    )?;
    fs::write(
        help_dir.join("Export Bundles.md"),
        "# Export Bundles\n\nPackage selected project files into a zip bundle under `exports/` for sharing or archiving.\n",
    )?;
    fs::write(
        help_dir.join("Keyboard Shortcuts.md"),
        "# Keyboard Shortcuts\n\nDocument your preferred keyboard shortcuts and navigation patterns here.\n",
    )?;
    Ok(())
}

fn read_recents_store(app_data_dir: &Path) -> StoreResult<RecentsStore> {
    let store_path = app_data_dir.join(RECENTS_FILE_NAME);
    if !store_path.exists() {
        return Ok(RecentsStore::default());
    }

    let content = fs::read_to_string(store_path)?;
    Ok(serde_json::from_str(&content)?)
}

// read_global_variables_store, write_global_variables_store → variables.rs
// read_workspace_variables_yaml, write_workspace_variables_yaml → variables.rs

fn read_manifest(manifest_path: &Path) -> StoreResult<ProjectManifest> {
    let content = fs::read_to_string(manifest_path)?;
    Ok(serde_json::from_str(&content)?)
}

pub(crate) fn write_manifest(root_path: &Path, manifest: &ProjectManifest) -> StoreResult<()> {
    let manifest_path = root_path.join("project.json");
    fs::write(manifest_path, serde_json::to_string_pretty(manifest)?)?;
    Ok(())
}

fn count_files(directory: PathBuf) -> StoreResult<usize> {
    let mut total = 0usize;
    for entry in fs::read_dir(directory)?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            total += count_files(path)?;
        } else {
            total += 1;
        }
    }
    Ok(total)
}

fn unique_prompt_slug(root_path: &Path, manifest: &ProjectManifest, prompt_name: &str) -> String {
    let base_slug = slugify_prompt_name(prompt_name);
    let mut candidate = base_slug.clone();
    let mut suffix = 2usize;

    loop {
        let prompt_path = format!("prompts/{candidate}.tera");
        let block_taken = manifest.prompt_blocks.iter().any(|block| block.block_id == candidate);
        let path_taken = manifest.prompt_blocks.iter().any(|block| block.template_source == prompt_path)
            || root_path.join(&prompt_path).exists();

        if !block_taken && !path_taken {
            return candidate;
        }

        candidate = format!("{base_slug}-{suffix}");
        suffix += 1;
    }
}

fn unique_pipeline_slug(root_path: &Path, pipeline_name: &str) -> String {
    let base_slug = slugify_prompt_name(pipeline_name);
    let mut candidate = base_slug.clone();
    let mut suffix = 2usize;

    loop {
        let pipeline_path = root_path.join("pipelines").join(format!("{}.json", candidate));
        if !pipeline_path.exists() {
            return candidate;
        }

        candidate = format!("{base_slug}-{suffix}");
        suffix += 1;
    }
}

pub(crate) fn unique_export_slug(root_path: &Path, export_name: &str) -> String {
    let base_slug = slugify_prompt_name(export_name);
    let mut candidate = base_slug.clone();
    let mut suffix = 2usize;

    loop {
        let export_root = root_path.join("exports").join(&candidate);
        if !export_root.exists() {
            return candidate;
        }

        candidate = format!("{base_slug}-{suffix}");
        suffix += 1;
    }
}

fn validate_pipeline_block_ids(
    manifest: &ProjectManifest,
    ordered_block_ids: &[String],
    pipeline_id: Option<&str>,
) -> StoreResult<()> {
    let _ = pipeline_id;

    if ordered_block_ids.is_empty() {
        return Err(ProjectStoreError::message(
            "Pipelines must contain at least one prompt block.",
        ));
    }

    let mut seen = Vec::<&str>::new();
    for block_id in ordered_block_ids {
        let trimmed = block_id.trim();
        if trimmed.is_empty() {
            return Err(ProjectStoreError::message(
                "Pipeline blocks cannot be empty.",
            ));
        }

        if seen.contains(&trimmed) {
            return Err(ProjectStoreError::message(format!(
                "Pipeline block `{trimmed}` is duplicated."
            )));
        }

        if !manifest.prompt_blocks.iter().any(|block| block.block_id == trimmed) {
            return Err(ProjectStoreError::message(format!(
                "Pipeline block `{trimmed}` was not found in the project prompt blocks."
            )));
        }

        seen.push(trimmed);
    }

    Ok(())
}

pub(crate) fn slugify_prompt_name(prompt_name: &str) -> String {
    let lower = prompt_name.trim().to_lowercase();
    let regex = Regex::new(r"[^a-z0-9]+").unwrap();
    let collapsed = regex.replace_all(&lower, "-");
    let slug = collapsed.trim_matches('-');
    if slug.is_empty() {
        "prompt".to_string()
    } else {
        slug.to_string()
    }
}

// classify_asset, is_exportable_kind, is_editable_kind, yaml_value_to_string → assets.rs

pub(crate) fn diff_path(root_path: &Path, path: &Path) -> StoreResult<String> {
    path.strip_prefix(root_path)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| ProjectStoreError::message("Asset path escaped the project root."))
}

pub(crate) fn detail(label: &str, value: &str) -> MetadataField {
    MetadataField {
        label: label.to_string(),
        value: value.to_string(),
    }
}


pub(crate) fn preview_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let preview = trimmed.chars().take(max_chars).collect::<String>();
    format!("{}...", preview.trim_end())
}

// Credential helpers moved to credentials.rs

pub(crate) fn default_model_id(root_path: &Path, manifest: &ProjectManifest) -> Option<String> {
    let model_path = root_path.join(&manifest.default_model_preset);
    let content = fs::read_to_string(model_path).ok()?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    yaml.get("model").and_then(|value| value.as_str()).map(|value| value.to_string())
}

pub(crate) fn is_missing_context_warning(message: &str) -> bool {
    message.contains("not found in context") || message.contains("Variable `")
}

pub(crate) fn is_identifier_like(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

pub(crate) fn timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn creates_project_structure_and_recents() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Story Lab", &app_data).unwrap();

        assert_eq!(summary.project_name, "Story Lab");
        assert!(temp.path().join("Story Lab").join("project.json").is_file());
        assert!(temp.path().join("Story Lab").join("models").join("default.yaml").is_file());
        assert!(temp.path().join("Story Lab").join("models").join("gpt-5.4.yaml").is_file());
        assert!(temp.path().join("Story Lab").join("models").join("gpt-5.4-nano.yaml").is_file());
        assert!(temp.path().join("Story Lab").join("models").join("claude-sonnet-4.6.yaml").is_file());
        assert!(temp.path().join("Story Lab").join("models").join("gpt-5.2-think.yaml").is_file());
        assert_eq!(summary.counts.models, 5);

        let recents = get_recent_projects(&app_data).unwrap();
        assert_eq!(recents.len(), 1);
        assert_eq!(recents[0].summary.project_name, "Story Lab");

        remove_recent_project(&app_data, Path::new(&summary.root_path)).unwrap();
        assert!(get_recent_projects(&app_data).unwrap().is_empty());
    }

    #[test]
    fn rejects_invalid_project_folder() {
        let temp = tempdir().unwrap();
        let invalid_path = temp.path().join("not-a-project");
        fs::create_dir_all(&invalid_path).unwrap();

        let error = open_project(&invalid_path, temp.path()).unwrap_err();
        assert!(error.to_string().contains("missing required directory"));
    }

    #[test]
    fn creates_prompt_block_file_and_manifest_entry() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PromptCreate", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let created = create_prompt_block(&root, "Scene Draft", &app_data).unwrap();

        assert_eq!(created.path, "prompts/scene-draft.tera");
        assert!(root.join(&created.path).is_file());
        assert!(fs::read_to_string(root.join(&created.path))
            .unwrap()
            .contains("{{ project.name }}"));

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        assert_eq!(manifest.prompt_blocks.len(), 1);
        assert_eq!(manifest.prompt_blocks[0].name, "Scene Draft");
        assert_eq!(manifest.prompt_blocks[0].template_source, "prompts/scene-draft.tera");

        let recents = get_recent_projects(&app_data).unwrap();
        assert_eq!(recents[0].summary.counts.prompts, 1);
    }

    #[test]
    fn locates_moved_recent_project_when_project_id_matches() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Story Lab", &app_data).unwrap();
        let original_root = PathBuf::from(&summary.root_path);
        let relocated_root = temp.path().join("Story Lab Moved");

        fs::rename(&original_root, &relocated_root).unwrap();

        let relocated = locate_recent_project(&app_data, &original_root, &relocated_root).unwrap();

        assert_eq!(relocated.project_id, summary.project_id);
        assert_eq!(relocated.root_path, relocated_root.canonicalize().unwrap().to_string_lossy());

        let recents = get_recent_projects(&app_data).unwrap();
        assert_eq!(recents.len(), 1);
        assert_eq!(recents[0].summary.root_path, relocated.root_path);
        assert!(recents[0].last_known_valid);
    }

    #[test]
    fn rejects_locate_when_project_id_does_not_match_missing_recent() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Story Lab", &app_data).unwrap();
        let original_root = PathBuf::from(&summary.root_path);
        let candidate_summary = create_project(temp.path(), "Other Lab", &app_data).unwrap();
        let candidate_root = PathBuf::from(&candidate_summary.root_path);

        fs::remove_dir_all(&original_root).unwrap();

        let error = locate_recent_project(&app_data, &original_root, &candidate_root).unwrap_err();

        assert!(error.to_string().contains("does not match the missing recent project"));
    }

    #[test]
    fn lists_and_reads_assets_from_fixture_style_project() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Fixture", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("documents").join("context.md"), "# Context\n\nFixture text.").unwrap();
        fs::write(root.join("prompts").join("review.tera"), "{{ doc(\"context.md\") }}").unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "block-1".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        fs::write(root.join("runs").join(".gitkeep"), "").unwrap();
        fs::write(root.join("exports").join(".gitkeep"), "").unwrap();

        let assets = list_project_assets(&root).unwrap();
        assert_eq!(assets[0].path, "project.json");
        assert!(assets
            .iter()
            .find(|node| node.path == "documents")
            .unwrap()
            .children
            .iter()
            .any(|child| child.path == "documents/context.md"));
        assert!(assets
            .iter()
            .find(|node| node.path == "runs")
            .unwrap()
            .children
            .is_empty());
        assert!(assets
            .iter()
            .find(|node| node.path == "exports")
            .unwrap()
            .children
            .is_empty());

        fs::write(
            root.join("runs").join("run-1.json"),
            serde_json::to_string_pretty(&json!({
                "runId": "run-1",
                "path": "prompts/review.tera",
                "blockName": "Review",
                "modelId": "openai/gpt-5.4",
                "status": "success",
                "output": "Execution output.",
                "error": null,
                "startedAt": "2026-04-03T20:00:00Z",
                "completedAt": "2026-04-03T20:00:01Z"
            }))
            .unwrap(),
        )
        .unwrap();

        let assets = list_project_assets(&root).unwrap();
        let run_node = assets
            .iter()
            .find(|node| node.path == "runs")
            .unwrap()
            .children
            .iter()
            .find(|child| child.path == "runs/run-1.json")
            .unwrap();
        assert_eq!(run_node.name, "Review");

        let content = read_project_asset(&root, "prompts/review.tera").unwrap();
        assert_eq!(content.kind, AssetKind::Tera);
        assert_eq!(content.view, AssetView::Text);
        assert!(content.is_editable);
        assert!(content
            .metadata
            .details
            .iter()
            .any(|detail| detail.value.contains("Review")));

        let run_content = read_project_asset(&root, "runs/run-1.json").unwrap();
        assert_eq!(run_content.metadata.name, "Review");
    }

    #[test]
    fn writes_supported_text_assets_and_rejects_read_only_assets() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Editable", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("documents").join("notes.md"), "# Notes\n").unwrap();
        let updated = write_project_asset(&root, "documents/notes.md", "# Notes\n\nUpdated.\n").unwrap();
        assert!(updated.is_editable);
        assert_eq!(
            fs::read_to_string(root.join("documents").join("notes.md")).unwrap(),
            "# Notes\n\nUpdated.\n"
        );

        let yaml_updated = write_project_asset(
            &root,
            "models/default.yaml",
            "model: openai/gpt-5.4-nano\ntemperature: 0.2\nmax_completion_tokens: 4000\n",
        )
        .unwrap();
        assert!(yaml_updated.is_editable);
        assert!(fs::read_to_string(root.join("models").join("default.yaml"))
            .unwrap()
            .contains("openai/gpt-5.4-nano"));

        let read_only_error = write_project_asset(&root, "project.json", "{}").unwrap_err();
        assert!(read_only_error.to_string().contains("read-only"));
    }

    #[test]
    fn validates_templates_with_preview_and_missing_document_warnings() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Validate", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("documents").join("context.md"), "Document body.").unwrap();

        let result = validate_project_template(
            &root,
            "prompts/review.tera",
            "Context:\n{{ doc(\"context.md\") }}\nModel: {{ model_id }}",
            &app_data,
        )
        .unwrap();
        assert_eq!(result.status, ValidationStatus::Valid);
        assert!(result.preview.unwrap().contains("Document body."));

        let warning_result =
            validate_project_template(&root, "prompts/review.tera", "{{ doc(\"missing.md\") }}", &app_data)
                .unwrap();
        assert_eq!(warning_result.status, ValidationStatus::Warnings);
        assert!(warning_result.warnings[0].contains("missing.md"));
    }

    #[test]
    fn validates_invalid_tera_syntax_as_error() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Errors", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let result = validate_project_template(
            &root,
            "prompts/broken.tera",
            "{% if unclosed_block %}Hello",
            &app_data,
        )
        .unwrap();

        assert_eq!(result.status, ValidationStatus::Invalid);
        assert!(!result.errors.is_empty());
        assert!(result.preview.is_none());
    }

    #[test]
    fn validates_undefined_variable_as_warning() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Warn", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let result =
            validate_project_template(&root, "prompts/review.tera", "{{ my_custom_var }}", &app_data)
                .unwrap();

        assert_eq!(result.status, ValidationStatus::Warnings);
        assert!(result.warnings.iter().any(|w| w.contains("my_custom_var")));
        assert!(result.errors.is_empty());
    }

    #[test]
    fn validates_traversal_doc_reference_as_warning() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Traversal", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let result = validate_project_template(
            &root,
            "prompts/review.tera",
            "{{ doc(\"../../../etc/passwd\") }}",
            &app_data,
        )
        .unwrap();

        assert_eq!(result.status, ValidationStatus::Warnings);
        assert!(result.warnings.iter().any(|w| w.contains("invalid")));
    }

    #[test]
    fn reads_malformed_yaml_asset_without_error() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "BadYaml", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(
            root.join("models").join("default.yaml"),
            "this: is: not: valid: yaml: [",
        )
        .unwrap();

        let result = read_project_asset(&root, "models/default.yaml");
        assert!(result.is_ok());

        let asset = result.unwrap();
        assert!(asset.metadata.details.iter().any(|d| d.value == "Invalid YAML"));
    }

    #[test]
    fn executes_prompt_block_and_persists_run_record() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Execute", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("documents").join("context.md"), "Doc body.").unwrap();
        fs::write(
            root.join("prompts").join("review.tera"),
            "Context:\n{{ doc(\"context.md\") }}\nTone: {{ tone }}\nModel: {{ model_id }}",
        )
        .unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.variables.insert("tone".to_string(), json!("precise"));
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "block-1".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, payload: Value| {
            assert_eq!(payload["model"], json!("openai/gpt-5.4"));
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": "Execution output."
                        }
                    }
                ],
                "usage": {
                    "prompt_tokens": 42,
                    "completion_tokens": 10,
                    "total_tokens": 52,
                    "cost": 0.00123
                }
            }))
        };

        let result = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/review.tera",
            "Context:\n{{ doc(\"context.md\") }}\nTone: {{ tone }}\nModel: {{ model_id }}",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.block_id.as_deref(), Some("block-1"));
        assert_eq!(result.output.as_deref(), Some("Execution output."));
        assert!(root.join(&result.run_path).is_file());

        let persisted = serde_json::from_str::<PersistedRunRecord>(
            &fs::read_to_string(root.join(&result.run_path)).unwrap(),
        )
        .unwrap();
        assert_eq!(persisted.artifact_version, 4);
        assert_eq!(persisted.block_id.as_deref(), Some("block-1"));
        assert_eq!(persisted.model_preset, "models/default.yaml");
        assert!(!persisted.online.enabled);

        assert_eq!(result.usage.prompt_tokens, Some(42));
        assert_eq!(result.usage.completion_tokens, Some(10));
        assert_eq!(result.usage.total_tokens, Some(52));
        assert!((result.usage.cost.unwrap() - 0.00123).abs() < f64::EPSILON);
        assert_eq!(result.usage.output_word_count, Some(2));
        assert_eq!(result.usage.retry_count, None);
        assert_eq!(persisted.usage.prompt_tokens, Some(42));
        assert_eq!(persisted.usage.completion_tokens, Some(10));
        assert_eq!(persisted.usage.retry_count, None);
    }

    #[test]
    fn retries_on_empty_response_and_records_retry_count() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Retry", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);
        let manifest = read_manifest(&root.join("project.json")).unwrap();

        let mut call_count = 0u32;
        let mut transport = |_api_key: &str, _payload: Value| {
            call_count += 1;
            if call_count < 3 {
                // Return empty content for the first two calls.
                Ok(json!({ "choices": [{ "message": { "content": "" } }] }))
            } else {
                Ok(json!({
                    "choices": [{ "message": { "content": "Retry success." } }],
                    "usage": { "prompt_tokens": 5, "completion_tokens": 2, "total_tokens": 7 }
                }))
            }
        };

        let result = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/review.tera",
            "Hello world",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.output.as_deref(), Some("Retry success."));
        assert_eq!(result.usage.retry_count, Some(2));
        assert_eq!(call_count, 3);
    }

    #[test]
    fn execution_enables_online_payload_and_persists_online_metadata() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "OnlineResearch", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(
            root.join("prompts").join("research.tera"),
            "{# diamond:online #}\nResearch the current market for {{ project.name }}.",
        )
        .unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "research-block".to_string(),
            name: "Research".to_string(),
            template_source: "prompts/research.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, payload: Value| {
            assert_eq!(payload["model"], json!("openai/gpt-5.4:online"));
            assert_eq!(payload["plugins"][0]["id"], json!("web"));
            assert_eq!(payload["plugins"][0]["max_results"], json!(3));
            assert_eq!(payload["web_search_options"]["search_context_size"], json!("medium"));
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": "Online output.",
                            "annotations": [
                                { "type": "url_citation" },
                                { "type": "url_citation" }
                            ]
                        }
                    }
                ],
                "usage": {
                    "server_tool_use": {
                        "web_search_requests": 2
                    }
                }
            }))
        };

        let result = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/research.tera",
            "{# diamond:online #}\nResearch the current market for {{ project.name }}.",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.model_id, "openai/gpt-5.4:online");
        assert!(result.online.enabled);
        assert_eq!(result.online.web_search_requests, 2);
        assert_eq!(result.online.citation_count, 2);

        let persisted = serde_json::from_str::<PersistedRunRecord>(
            &fs::read_to_string(root.join(&result.run_path)).unwrap(),
        )
        .unwrap();
        assert!(persisted.online.enabled);
        assert_eq!(persisted.online.web_search_requests, 2);
        assert_eq!(persisted.online.citation_count, 2);

        let history = list_prompt_run_history(&root, "prompts/research.tera").unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].online.enabled);
        assert_eq!(history[0].online.web_search_requests, 2);
        assert_eq!(history[0].online.citation_count, 2);
    }

    #[test]
    fn execution_fails_on_missing_document_reference() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "ExecuteFail", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);
        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, _payload: Value| {
            unreachable!("transport should not run for invalid execution input")
        };

        let error = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/review.tera",
            "{{ doc(\"missing.md\") }}",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap_err();

        assert!(error.to_string().contains("could not be resolved"));
    }

    #[test]
    fn execution_fails_on_unresolved_required_variable() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "ExecuteVarFail", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);
        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, _payload: Value| {
            unreachable!("transport should not run when required variables are missing")
        };

        let error = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/review.tera",
            "Tone: {{ missing_tone }}",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("Execution requires all referenced variables to resolve"));
        assert!(error.to_string().contains("missing_tone"));
    }

    #[test]
    fn execution_allows_optional_variable_when_template_guards_it() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "ExecuteVarGuard", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);
        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, payload: Value| {
            assert!(payload["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("Fallback tone"));
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": "Guarded output."
                        }
                    }
                ]
            }))
        };

        let result = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/review.tera",
            "{% if missing_tone is defined %}{{ missing_tone }}{% else %}Fallback tone{% endif %}",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.output.as_deref(), Some("Guarded output."));
    }

    #[test]
    fn execution_uses_block_model_override_when_present() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Override", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(
            root.join("models").join("review.yaml"),
            "model: openai/gpt-5.4-nano\ntemperature: 0.2\nmax_completion_tokens: 4000\n",
        )
        .unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "block-1".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: Some("models/review.yaml".to_string()),
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, payload: Value| {
            assert_eq!(payload["model"], json!("openai/gpt-5.4-nano"));
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": "Override output."
                        }
                    }
                ]
            }))
        };

        let result = execute_prompt_block_with_transport(
            &root,
            &manifest,
            "prompts/review.tera",
            "Hello world",
            None,
            None,
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.model_preset, "models/review.yaml");
        assert_eq!(result.model_id, "openai/gpt-5.4-nano");
    }

    // Credential tests moved to credentials.rs

    #[test]
    fn lists_prompt_run_history_for_matching_prompt() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "History", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(
            root.join("runs").join("run-old.json"),
            serde_json::to_string_pretty(&json!({
                "runId": "run-old",
                "path": "prompts/review.tera",
                "blockName": "Review",
                "modelId": "openai/gpt-5.4",
                "status": "success",
                "output": "Earlier output",
                "error": null,
                "startedAt": "2026-04-03T20:00:00Z",
                "completedAt": "2026-04-03T20:00:01Z"
            }))
            .unwrap(),
        )
        .unwrap();

        fs::write(
            root.join("runs").join("run-new.json"),
            serde_json::to_string_pretty(&json!({
                "runId": "run-new",
                "path": "prompts/review.tera",
                "blockName": "Review",
                "modelId": "openai/gpt-5.4-nano",
                "status": "failed",
                "output": null,
                "error": "Provider timeout",
                "startedAt": "2026-04-03T21:00:00Z",
                "completedAt": "2026-04-03T21:00:02Z"
            }))
            .unwrap(),
        )
        .unwrap();

        fs::write(
            root.join("runs").join("run-other.json"),
            serde_json::to_string_pretty(&json!({
                "runId": "run-other",
                "path": "prompts/other.tera",
                "blockName": "Other",
                "modelId": "openai/gpt-5.4",
                "status": "success",
                "output": "Other output",
                "error": null,
                "startedAt": "2026-04-03T22:00:00Z",
                "completedAt": "2026-04-03T22:00:03Z"
            }))
            .unwrap(),
        )
        .unwrap();

        let history = list_prompt_run_history(&root, "prompts/review.tera").unwrap();

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].run_id, "run-new");
        assert_eq!(history[1].run_id, "run-old");
        assert_eq!(history[0].run_path, "runs/run-new.json");
    }

    #[test]
    fn lists_project_run_history_with_pipeline_metadata() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "ProjectHistory", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(
            root.join("runs").join("run-pipeline.json"),
            serde_json::to_string_pretty(&json!({
                "runId": "run-pipeline",
                "path": "prompts/review.tera",
                "blockId": "brief-review",
                "blockName": "Brief Review",
                "pipelineId": "review-pipeline",
                "pipelineName": "Review Pipeline",
                "modelId": "openai/gpt-5.4",
                "status": "success",
                "output": "Pipeline output",
                "error": null,
                "startedAt": "2026-04-03T21:00:00Z",
                "completedAt": "2026-04-03T21:00:01Z"
            }))
            .unwrap(),
        )
        .unwrap();

        fs::write(
            root.join("runs").join("run-standalone.json"),
            serde_json::to_string_pretty(&json!({
                "runId": "run-standalone",
                "path": "prompts/other.tera",
                "blockId": "other-block",
                "blockName": "Other",
                "pipelineId": null,
                "pipelineName": null,
                "modelId": "openai/gpt-5.4-nano",
                "status": "failed",
                "output": null,
                "error": "Provider timeout",
                "startedAt": "2026-04-03T20:00:00Z",
                "completedAt": "2026-04-03T20:00:02Z"
            }))
            .unwrap(),
        )
        .unwrap();

        let history = list_project_run_history(&root).unwrap();

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].run_id, "run-pipeline");
        assert_eq!(history[0].pipeline_id.as_deref(), Some("review-pipeline"));
        assert_eq!(history[0].pipeline_name.as_deref(), Some("Review Pipeline"));
        assert_eq!(history[0].block_id.as_deref(), Some("brief-review"));
        assert_eq!(history[1].run_id, "run-standalone");
        assert_eq!(history[1].pipeline_id, None);
    }

    #[test]
    fn lists_manifest_pipelines_with_resolved_block_metadata() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Pipelines", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "block-1".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        let pipeline = Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["block-1".to_string()],
            execution_mode: "sequential".to_string(),
        };
        std::fs::create_dir_all(root.join("pipelines")).unwrap();
        std::fs::write(root.join("pipelines").join("pipeline-1.json"), serde_json::to_string(&pipeline).unwrap()).unwrap();
        write_manifest(&root, &manifest).unwrap();

        let pipelines = list_project_pipelines(&root).unwrap();

        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].name, "Review Pipeline");
        assert_eq!(pipelines[0].blocks[0].name, "Review");
        assert_eq!(pipelines[0].blocks[0].model_preset, "models/default.yaml");
    }

    #[test]
    fn lists_project_prompt_blocks_with_resolved_model_presets() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PromptBlocks", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let prompt_blocks = list_project_prompt_blocks(&root).unwrap();

        assert_eq!(prompt_blocks.len(), 1);
        assert_eq!(prompt_blocks[0].block_id, "review");
        assert_eq!(prompt_blocks[0].model_preset, "models/default.yaml");
    }

    #[test]
    fn registers_existing_prompt_template_as_prompt_block() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PromptRegistration", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let prompt_path = root.join("prompts").join("plan-chapter.tera");
        fs::write(&prompt_path, "Write the plan.").unwrap();

        let created = register_prompt_block(&root, "prompts/plan-chapter.tera", &app_data).unwrap();
        assert_eq!(created.path, "prompts/plan-chapter.tera");

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        assert_eq!(manifest.prompt_blocks.len(), 1);
        assert_eq!(manifest.prompt_blocks[0].block_id, "plan-chapter");
        assert_eq!(manifest.prompt_blocks[0].name, "Plan Chapter");
        assert_eq!(manifest.prompt_blocks[0].template_source, "prompts/plan-chapter.tera");
    }

    #[test]
    fn rejects_registering_prompt_template_twice() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PromptRegistration", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let prompt_path = root.join("prompts").join("plan-chapter.tera");
        fs::write(&prompt_path, "Write the plan.").unwrap();

        register_prompt_block(&root, "prompts/plan-chapter.tera", &app_data).unwrap();
        let error = register_prompt_block(&root, "prompts/plan-chapter.tera", &app_data).unwrap_err();
        assert!(error.to_string().contains("already registered"));
    }

    #[test]
    fn creates_and_updates_pipeline_from_manifest_authoring_api() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PipelineAuthoring", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "outline".to_string(),
            name: "Outline".to_string(),
            template_source: "prompts/outline.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let created = create_pipeline(
            &root,
            "Draft Pipeline",
            &["review".to_string()],
            &app_data,
        )
        .unwrap();

        assert_eq!(created.pipeline_id, "draft-pipeline");

        let updated = update_pipeline(
            &root,
            "draft-pipeline",
            "Draft Pipeline Revised",
            &["review".to_string(), "outline".to_string()],
            &app_data,
        )
        .unwrap();

        assert_eq!(updated.pipeline_id, "draft-pipeline");

        let pipelines = list_project_pipelines(&root).unwrap();
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].name, "Draft Pipeline Revised");
        assert_eq!(pipelines[0].blocks.len(), 2);
        assert_eq!(pipelines[0].execution_mode, "sequential");
    }

    #[test]
    fn rejects_pipeline_authoring_with_duplicate_blocks() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PipelineValidation", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let error = create_pipeline(
            &root,
            "Invalid Pipeline",
            &["review".to_string(), "review".to_string()],
            &app_data,
        )
        .unwrap_err();

        assert!(error.to_string().contains("duplicated"));
    }

    #[test]
    fn duplicates_pipeline_with_new_id_and_name() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "DuplicatePipeline", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        write_manifest(&root, &manifest).unwrap();

        let created = create_pipeline(
            &root,
            "Draft Pipeline",
            &["review".to_string()],
            &app_data,
        )
        .unwrap();

        let dup = duplicate_pipeline(&root, &created.pipeline_id, &app_data).unwrap();

        assert_eq!(dup.pipeline_id, "copy-of-draft-pipeline");
        assert_ne!(dup.pipeline_id, created.pipeline_id);

        let pipelines = list_project_pipelines(&root).unwrap();
        assert_eq!(pipelines.len(), 2);

        let copy = pipelines.iter().find(|p| p.pipeline_id == dup.pipeline_id).unwrap();
        assert_eq!(copy.name, "Copy of Draft Pipeline");
        assert_eq!(copy.blocks.len(), 1);
        assert_eq!(copy.blocks[0].block_id, "review");
    }

    #[test]
    fn exports_selected_assets_into_a_bundle_directory() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "Exports", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("documents").join("context.md"), "Doc body").unwrap();
        fs::write(root.join("prompts").join("review.tera"), "Review doc").unwrap();

        let result = export_project_assets(
            &root,
            "Session Export",
            &["documents/context.md".to_string(), "prompts/review.tera".to_string()],
            &app_data,
        )
        .unwrap();

        assert_eq!(result.bundle_path, "exports/session-export");
        assert!(root.join("exports").join("session-export").join("documents").join("context.md").is_file());
        assert!(root.join("exports").join("session-export").join("prompts").join("review.tera").is_file());
        assert!(root.join("exports").join("session-export").join("export.json").is_file());
    }

    #[test]
    fn rejects_export_bundle_without_any_selected_assets() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "ExportsValidation", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let error = export_project_assets(&root, "Empty Export", &[], &app_data).unwrap_err();

        assert!(error.to_string().contains("Select at least one project asset"));
    }

    #[test]
    fn executes_pipeline_sequentially_and_stops_on_failure() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PipelineRun", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("documents").join("context.md"), "Doc body.").unwrap();
        fs::write(root.join("prompts").join("review.tera"), "Review {{ doc(\"context.md\") }}").unwrap();
        fs::write(root.join("prompts").join("outline.tera"), "Outline {{ missing_value }}").unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "outline".to_string(),
            name: "Outline".to_string(),
            template_source: "prompts/outline.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        manifest.variables.insert("tone".to_string(), json!("precise"));
        let pipeline = Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["review".to_string(), "outline".to_string()],
            execution_mode: "sequential".to_string(),
        };
        std::fs::write(root.join("pipelines").join("pipeline-1.json"), serde_json::to_string(&pipeline).unwrap()).unwrap();
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut calls = 0usize;
        let mut transport = |_api_key: &str, _payload: Value| {
            calls += 1;
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": format!("Step output {calls}")
                        }
                    }
                ]
            }))
        };

        let result = crate::project_store::execution::execute_pipeline_with_transport(
            &root,
            &manifest,
            "pipeline-1",
            None,
            "test-key",
            &mut transport,
            &app_data,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(result.status, ExecutionStatus::Failed);
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].block_name, "Review");
        assert!(result.error.unwrap().contains("Outline"));
        assert_eq!(calls, 1);
    }

    #[test]
    fn extracts_usage_metrics_from_response_with_all_fields() {
        let response = json!({
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 200,
                "total_tokens": 300,
                "cost": 0.0042
            }
        });
        let metrics = extract_usage_metrics(&response, Some("Hello world output here"));
        assert_eq!(metrics.prompt_tokens, Some(100));
        assert_eq!(metrics.completion_tokens, Some(200));
        assert_eq!(metrics.total_tokens, Some(300));
        assert!((metrics.cost.unwrap() - 0.0042).abs() < f64::EPSILON);
        assert_eq!(metrics.output_word_count, Some(4));
    }

    #[test]
    fn executes_selected_pipeline_blocks_in_pipeline_order() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PipelineSubset", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("prompts").join("plan.tera"), "Plan").unwrap();
        fs::write(root.join("prompts").join("draft.tera"), "Draft").unwrap();
        fs::write(root.join("prompts").join("polish.tera"), "Polish").unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "plan".to_string(),
            name: "Plan".to_string(),
            template_source: "prompts/plan.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "draft".to_string(),
            name: "Draft".to_string(),
            template_source: "prompts/draft.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "polish".to_string(),
            name: "Polish".to_string(),
            template_source: "prompts/polish.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        let pipeline = Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["plan".to_string(), "draft".to_string(), "polish".to_string()],
            execution_mode: "sequential".to_string(),
        };
        std::fs::write(
            root.join("pipelines").join("pipeline-1.json"),
            serde_json::to_string(&pipeline).unwrap(),
        )
        .unwrap();
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut calls = Vec::new();
        let mut transport = |_api_key: &str, payload: Value| {
            let prompt = payload["messages"][0]["content"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            calls.push(prompt.clone());
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": prompt
                        }
                    }
                ]
            }))
        };

        let result = crate::project_store::execution::execute_pipeline_with_transport(
            &root,
            &manifest,
            "pipeline-1",
            None,
            "test-key",
            &mut transport,
            &app_data,
            None,
            Some(vec!["polish".to_string(), "plan".to_string()]),
            None,
            None,
        )
        .unwrap();

        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.steps.len(), 2);
        assert_eq!(result.steps[0].block_id.as_deref(), Some("plan"));
        assert_eq!(result.steps[1].block_id.as_deref(), Some("polish"));
        assert_eq!(calls, vec!["Plan".to_string(), "Polish".to_string()]);
    }

    #[test]
    fn rejects_selected_pipeline_blocks_that_are_not_in_pipeline() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PipelineSubsetInvalid", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("prompts").join("review.tera"), "Review").unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        let pipeline = Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["review".to_string()],
            execution_mode: "sequential".to_string(),
        };
        std::fs::write(
            root.join("pipelines").join("pipeline-1.json"),
            serde_json::to_string(&pipeline).unwrap(),
        )
        .unwrap();
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, _payload: Value| {
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": "Review"
                        }
                    }
                ]
            }))
        };

        let error = crate::project_store::execution::execute_pipeline_with_transport(
            &root,
            &manifest,
            "pipeline-1",
            None,
            "test-key",
            &mut transport,
            &app_data,
            None,
            Some(vec!["missing".to_string()]),
            None,
            None,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("does not contain selected block(s): missing"));
    }

    #[test]
    fn rejects_resume_block_that_is_not_selected_for_subset_run() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PipelineSubsetResume", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        fs::write(root.join("prompts").join("review.tera"), "Review").unwrap();
        fs::write(root.join("prompts").join("outline.tera"), "Outline").unwrap();

        let mut manifest = read_manifest(&root.join("project.json")).unwrap();
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "review".to_string(),
            name: "Review".to_string(),
            template_source: "prompts/review.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "outline".to_string(),
            name: "Outline".to_string(),
            template_source: "prompts/outline.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
            output_filename: None,
        });
        let pipeline = Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["review".to_string(), "outline".to_string()],
            execution_mode: "sequential".to_string(),
        };
        std::fs::write(
            root.join("pipelines").join("pipeline-1.json"),
            serde_json::to_string(&pipeline).unwrap(),
        )
        .unwrap();
        write_manifest(&root, &manifest).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        let mut transport = |_api_key: &str, _payload: Value| {
            Ok(json!({
                "choices": [
                    {
                        "message": {
                            "content": "Review"
                        }
                    }
                ]
            }))
        };

        let error = crate::project_store::execution::execute_pipeline_with_transport(
            &root,
            &manifest,
            "pipeline-1",
            None,
            "test-key",
            &mut transport,
            &app_data,
            Some("outline".to_string()),
            Some(vec!["review".to_string()]),
            None,
            None,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("Resume block `outline` is not selected for this run"));
    }

    #[test]
    fn extracts_usage_metrics_gracefully_when_fields_missing() {
        let response = json!({
            "choices": [{ "message": { "content": "ok" } }]
        });
        let metrics = extract_usage_metrics(&response, None);
        assert_eq!(metrics.prompt_tokens, None);
        assert_eq!(metrics.completion_tokens, None);
        assert_eq!(metrics.total_tokens, None);
        assert!(metrics.cost.is_none());
        assert!(metrics.output_word_count.is_none());
    }

    #[test]
    fn reads_empty_global_variables_when_file_missing() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let vars = get_global_variables(&app_data).unwrap();
        assert!(vars.is_empty());
    }

    #[test]
    fn saves_and_reloads_global_variables() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        fs::create_dir_all(&app_data).unwrap();

        let mut vars = BTreeMap::new();
        vars.insert("tone".to_string(), "precise".to_string());
        vars.insert("pov".to_string(), "third-limited".to_string());

        let returned = set_global_variables(&app_data, vars.clone()).unwrap();
        assert_eq!(returned, vars);

        let reloaded = get_global_variables(&app_data).unwrap();
        assert_eq!(reloaded["tone"], "precise");
        assert_eq!(reloaded["pov"], "third-limited");
    }

    #[test]
    fn global_variables_are_available_in_template_context() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "GlobalVarTest", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut global_vars = BTreeMap::new();
        global_vars.insert("tone".to_string(), "lyrical".to_string());
        set_global_variables(&app_data, global_vars).unwrap();

        let result = validate_project_template(
            &root,
            "prompts/test.tera",
            "Tone: {{ tone }}",
            &app_data,
        )
        .unwrap();

        assert_eq!(result.status, ValidationStatus::Valid);
        assert!(result.preview.unwrap().contains("lyrical"));
    }

    #[test]
    fn project_variables_override_globals_with_same_name() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "OverrideTest", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut global_vars = BTreeMap::new();
        global_vars.insert("tone".to_string(), "global-tone".to_string());
        set_global_variables(&app_data, global_vars).unwrap();

        let mut project_vars = BTreeMap::new();
        project_vars.insert("tone".to_string(), "project-tone".to_string());
        set_project_variables(&root, project_vars).unwrap();

        let result = validate_project_template(
            &root,
            "prompts/test.tera",
            "Tone: {{ tone }}",
            &app_data,
        )
        .unwrap();

        assert_eq!(result.status, ValidationStatus::Valid);
        assert!(result.preview.unwrap().contains("project-tone"));
    }

    #[test]
    fn set_project_variables_persists_to_manifest() {
        let temp = tempdir().unwrap();
        let app_data = temp.path().join("app-data");
        let summary = create_project(temp.path(), "PersistVars", &app_data).unwrap();
        let root = PathBuf::from(&summary.root_path);

        let mut vars = BTreeMap::new();
        vars.insert("chapter".to_string(), "12".to_string());
        vars.insert("word_target".to_string(), "5000".to_string());

        set_project_variables(&root, vars).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        assert_eq!(manifest.variables["chapter"], Value::String("12".to_string()));
        assert_eq!(manifest.variables["word_target"], Value::String("5000".to_string()));
    }
    #[test]
    fn executes_neon_and_nightmares_sample_project() {
        let root = std::path::PathBuf::from("../Sample Projects/Neon & Nightmares");
        
        // This validates if the project even parses!
        let manifest = read_manifest(&root.join("project.json")).unwrap();
        
        let temp_app_data = tempdir().unwrap();
        let app_data = temp_app_data.path().join("app-data");
        std::fs::create_dir_all(&app_data).unwrap();

        let mut call_count = 0;
        let mut transport = |_api_key: &str, _payload: Value| {
            call_count += 1;
            Ok(json!({
                "choices": [{ "message": { "content": format!("Simulated response for step {}", call_count) } }],
                "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
            }))
        };

        let mut payload = BTreeMap::new();
        payload.insert("chapter".to_string(), "2".to_string());

        // Prove the batch-production pipeline successfully extracts logic and outputs the files
        let result = crate::project_store::execution::execute_pipeline_with_transport(
            &root,
            &manifest,
            "batch-production",
            Some(payload),
            "dummy_key",
            &mut transport,
            &app_data,
            None,
            None,
            None,
            None,
        ).expect("Pipeline failed to cleanly execute in Neon & Nightmares sample folder");

        assert_eq!(result.steps.len(), 7);
        assert_eq!(call_count, 7);
        
        // We know that block 0 output target makes "chapter-2-01-plan.md"
        let output_path = root.join("documents").join("chapter-2-01-plan.md");
        assert!(output_path.exists(), "The pipeline did not write the markdown document correctly!");
        
        // We can safely clean up the test generated files from the sample directory
        let _ = std::fs::remove_file(output_path);
        let _ = std::fs::remove_file(root.join("documents").join("forensic-trope-audit-output.md"));
    }
}
