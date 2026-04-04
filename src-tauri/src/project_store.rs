use std::{
    collections::BTreeMap,
    env,
    fs,
    path::{Component, Path, PathBuf},
};

use chrono::{SecondsFormat, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tera::{Context, Tera};
use thiserror::Error;
use uuid::Uuid;

const PROJECT_DIRS: [&str; 5] = ["documents", "prompts", "models", "runs", "exports"];
const RECENTS_FILE_NAME: &str = "recent-projects.json";
const GLOBAL_VARIABLES_FILE_NAME: &str = "global-variables.json";
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
const OPENROUTER_CHAT_COMPLETIONS_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const OPENROUTER_API_KEY_ENV: &str = "OPENROUTER_API_KEY";
const OPENROUTER_KEYCHAIN_SERVICE: &str = "com.blossomz37.diamondrunner";
const OPENROUTER_KEYCHAIN_ACCOUNT: &str = "openrouter-api-key";
const PERSISTED_RUN_RECORD_VERSION: u32 = 4;
const MAX_EXECUTION_RETRIES: u32 = 2;
const ONLINE_PROMPT_DIRECTIVE: &str = "diamond:online";
const DEFAULT_ONLINE_WEB_MAX_RESULTS: u32 = 3;
const DEFAULT_ONLINE_SEARCH_CONTEXT_SIZE: &str = "medium";
const DEFAULT_PROMPT_TEMPLATE: &str = "Project: {{ project.name }}\nDate: {{ current_date }}\n\nWrite the instructions for this prompt block here.\n";

#[derive(Debug, Error)]
pub enum ProjectStoreError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

impl ProjectStoreError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type StoreResult<T> = Result<T, ProjectStoreError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCounts {
    pub documents: usize,
    pub prompts: usize,
    pub models: usize,
    pub runs: usize,
    pub exports: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub root_path: String,
    pub project_id: String,
    pub project_name: String,
    pub default_model_preset: String,
    pub updated_at: String,
    pub counts: ProjectCounts,
    // String-typed project variables only. Complex values set via direct file edit are omitted.
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedPromptBlockResult {
    pub summary: ProjectSummary,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPresetSummary {
    pub path: String,
    pub filename: String,
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPipelineResult {
    pub summary: ProjectSummary,
    pub pipeline_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBundleResult {
    pub summary: ProjectSummary,
    pub bundle_name: String,
    pub bundle_path: String,
    pub exported_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUsageSummary {
    pub total_runs: u32,
    pub successful_runs: u32,
    pub failed_runs: u32,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub total_output_words: u64,
    pub total_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProjectEntry {
    #[serde(flatten)]
    pub summary: ProjectSummary,
    pub last_opened_at: String,
    pub last_known_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAssetNode {
    pub name: String,
    pub path: String,
    pub kind: AssetKind,
    pub is_directory: bool,
    pub children: Vec<ProjectAssetNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataField {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMetadata {
    pub kind: AssetKind,
    pub path: String,
    pub name: String,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub details: Vec<MetadataField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetContent {
    pub path: String,
    pub kind: AssetKind,
    pub view: AssetView,
    pub content: String,
    pub is_editable: bool,
    pub metadata: AssetMetadata,
    pub parsed_json: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateValidationResult {
    pub path: String,
    pub status: ValidationStatus,
    pub preview: Option<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub context_summary: Vec<MetadataField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnlineRunMetadata {
    pub enabled: bool,
    pub web_search_requests: u32,
    pub citation_count: u32,
}

impl Default for OnlineRunMetadata {
    fn default() -> Self {
        Self {
            enabled: false,
            web_search_requests: 0,
            citation_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetrics {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
    pub cost: Option<f64>,
    pub output_word_count: Option<u32>,
    #[serde(default)]
    pub retry_count: Option<u32>,
}

impl Default for UsageMetrics {
    fn default() -> Self {
        Self {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
            cost: None,
            output_word_count: None,
            retry_count: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptExecutionResult {
    pub run_id: String,
    pub path: String,
    pub block_id: Option<String>,
    pub block_name: String,
    #[serde(default)]
    pub pipeline_id: Option<String>,
    #[serde(default)]
    pub pipeline_name: Option<String>,
    pub model_preset: String,
    pub model_id: String,
    pub status: ExecutionStatus,
    pub output_target: String,
    pub document_path: Option<String>,
    pub variables: std::collections::BTreeMap<String, String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub run_path: String,
    pub started_at: String,
    pub completed_at: String,
    #[serde(default)]
    pub online: OnlineRunMetadata,
    #[serde(default)]
    pub usage: UsageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PromptRunHistoryEntry {
    pub run_id: String,
    pub path: String,
    pub block_id: Option<String>,
    pub block_name: String,
    pub pipeline_id: Option<String>,
    pub pipeline_name: Option<String>,
    pub model_id: String,
    pub status: ExecutionStatus,
    pub run_path: String,
    pub started_at: String,
    pub completed_at: String,
    pub output_preview: Option<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub online: OnlineRunMetadata,
    #[serde(default)]
    pub usage: UsageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRunHistoryEntry {
    pub run_id: String,
    pub path: String,
    pub block_id: Option<String>,
    pub block_name: String,
    pub pipeline_id: Option<String>,
    pub pipeline_name: Option<String>,
    pub model_id: String,
    pub status: ExecutionStatus,
    pub run_path: String,
    pub started_at: String,
    pub completed_at: String,
    pub output_preview: Option<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub online: OnlineRunMetadata,
    #[serde(default)]
    pub usage: UsageMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPipelineBlockSummary {
    pub block_id: String,
    pub name: String,
    pub template_source: String,
    pub model_preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPromptBlockSummary {
    pub block_id: String,
    pub name: String,
    pub template_source: String,
    pub model_preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPipelineSummary {
    pub pipeline_id: String,
    pub name: String,
    pub execution_mode: String,
    pub blocks: Vec<ProjectPipelineBlockSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineExecutionResult {
    pub pipeline_id: String,
    pub pipeline_name: String,
    pub status: ExecutionStatus,
    pub started_at: String,
    pub completed_at: String,
    pub error: Option<String>,
    pub steps: Vec<PromptExecutionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionCredentialStatus {
    pub source: CredentialSource,
    pub has_stored_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CredentialSource {
    Keychain,
    Environment,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Valid,
    Warnings,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AssetView {
    Text,
    Json,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Manifest,
    Directory,
    Markdown,
    Text,
    Tera,
    Yaml,
    Json,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub project_id: String,
    pub project_name: String,
    pub created_at: String,
    pub updated_at: String,
    pub default_model_preset: String,
    pub variables: BTreeMap<String, Value>,
    pub prompt_blocks: Vec<PromptBlock>,
    pub pipelines: Vec<Pipeline>,
    pub ui_preferences: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptBlock {
    pub block_id: String,
    pub name: String,
    pub template_source: String,
    pub input_bindings: Vec<Value>,
    pub model_preset: Option<String>,
    pub output_target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pipeline {
    pub pipeline_id: String,
    pub name: String,
    pub ordered_blocks: Vec<String>,
    pub execution_mode: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RecentsStore {
    projects: Vec<RecentProjectEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct GlobalVariablesStore {
    #[serde(default)]
    variables: BTreeMap<String, String>,
}

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
        output_target: "run_artifact".to_string(),
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
    manifest.variables = variables
        .into_iter()
        .map(|(k, v)| (k, Value::String(v)))
        .collect();
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;
    summarize_project(&root_path, &manifest)
}

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

pub fn list_project_pipelines(root_path: &Path) -> StoreResult<Vec<ProjectPipelineSummary>> {
    let (_, manifest) = validate_project(root_path)?;

    Ok(manifest
        .pipelines
        .iter()
        .map(|pipeline| summarize_pipeline(&manifest, pipeline))
        .collect())
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

    let pipeline_id = unique_pipeline_slug(&manifest, trimmed_name);
    manifest.pipelines.push(Pipeline {
        pipeline_id: pipeline_id.clone(),
        name: trimmed_name.to_string(),
        ordered_blocks: ordered_block_ids.to_vec(),
        execution_mode: "sequential".to_string(),
    });
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

    let Some(pipeline) = manifest
        .pipelines
        .iter_mut()
        .find(|pipeline| pipeline.pipeline_id == pipeline_id)
    else {
        return Err(ProjectStoreError::message(format!(
            "Pipeline `{pipeline_id}` was not found."
        )));
    };

    pipeline.name = trimmed_name.to_string();
    pipeline.ordered_blocks = ordered_block_ids.to_vec();
    pipeline.execution_mode = "sequential".to_string();
    manifest.updated_at = timestamp();
    write_manifest(&root_path, &manifest)?;

    let summary = summarize_project(&root_path, &manifest)?;
    update_recent_projects(app_data_dir, &summary)?;

    Ok(SavedPipelineResult {
        summary,
        pipeline_id: pipeline_id.to_string(),
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

pub fn validate_project_template(
    root_path: &Path,
    relative_path: &str,
    content: &str,
    app_data_dir: &Path,
) -> StoreResult<TemplateValidationResult> {
    let (root_path, manifest) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(relative_path)?;
    let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");

    if classify_asset(&safe_relative_string, false) != AssetKind::Tera {
        return Err(ProjectStoreError::message(
            "Template validation is only available for `.tera` prompt files.",
        ));
    }

    let prepared = prepare_template_context(&root_path, &manifest, content, false, None, Some(app_data_dir))?;
    let model_id = prepared.model_id.clone();
    let mut warnings = prepared.warnings;

    let mut errors = Vec::new();
    let mut tera = Tera::default();
    tera.autoescape_on(Vec::new());
    if let Err(error) = tera.add_raw_template("active", &prepared.content) {
        errors.push(error.to_string());
        return Ok(build_validation_result(
            safe_relative_string,
            None,
            warnings,
            errors,
            &manifest,
            &model_id,
        ));
    }

    let preview = match tera.render("active", &prepared.context) {
        Ok(rendered) => Some(rendered),
        Err(error) => {
            let message = flatten_error_chain(&error);
            if is_missing_context_warning(&message) {
                warnings.push(message);
            } else {
                errors.push(message);
            }
            None
        }
    };

    Ok(build_validation_result(
        safe_relative_string,
        preview,
        warnings,
        errors,
        &manifest,
        &model_id,
    ))
}

pub fn execute_prompt_block(
    root_path: &Path,
    relative_path: &str,
    content: &str,
    app_data_dir: &Path,
) -> StoreResult<PromptExecutionResult> {
    let api_key = load_execution_api_key()?;
    let (root_path, manifest) = validate_project(root_path)?;
    let mut transport = |api_key: &str, payload: Value| {
        post_openrouter_chat_completion(OPENROUTER_CHAT_COMPLETIONS_URL, api_key, &payload)
    };

    execute_prompt_block_with_transport(
        &root_path,
        &manifest,
        relative_path,
        content,
        None,
        &api_key,
        &mut transport,
        app_data_dir,
    )
}

pub fn execute_pipeline(
    root_path: &Path,
    pipeline_id: &str,
    app_data_dir: &Path,
) -> StoreResult<PipelineExecutionResult> {
    let api_key = load_execution_api_key()?;
    let (root_path, manifest) = validate_project(root_path)?;
    let mut transport = |api_key: &str, payload: Value| {
        post_openrouter_chat_completion(OPENROUTER_CHAT_COMPLETIONS_URL, api_key, &payload)
    };

    execute_pipeline_with_transport(&root_path, &manifest, pipeline_id, &api_key, &mut transport, app_data_dir)
}

pub fn get_execution_credential_status() -> StoreResult<ExecutionCredentialStatus> {
    // Keychain access can fail in unsigned dev builds or sandboxed contexts.
    // Treat errors as "no stored key" so the app can still load.
    let has_stored_key = match load_stored_openrouter_api_key() {
        Ok(key) => key.is_some(),
        Err(error) => {
            eprintln!("[diamond] keychain probe failed (non-fatal): {error}");
            false
        }
    };
    Ok(build_execution_credential_status(
        has_stored_key,
        load_environment_api_key().is_some(),
    ))
}

pub fn save_execution_api_key(api_key: &str) -> StoreResult<ExecutionCredentialStatus> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(ProjectStoreError::message("API key cannot be empty."));
    }

    openrouter_keyring_entry()?
        .set_password(trimmed)
        .map_err(keyring_error)?;
    get_execution_credential_status()
}

pub fn clear_execution_api_key() -> StoreResult<ExecutionCredentialStatus> {
    match openrouter_keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => get_execution_credential_status(),
        Err(error) => Err(keyring_error(error)),
    }
}

pub fn list_prompt_run_history(root_path: &Path, relative_path: &str) -> StoreResult<Vec<PromptRunHistoryEntry>> {
    let (root_path, _) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(relative_path)?;
    let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");

    let entries = read_run_history_entries(&root_path, Some(&safe_relative_string))?;
    Ok(entries
        .into_iter()
        .map(|entry| PromptRunHistoryEntry {
            run_id: entry.run_id,
            path: entry.path,
            block_id: entry.block_id,
            block_name: entry.block_name,
            pipeline_id: entry.pipeline_id,
            pipeline_name: entry.pipeline_name,
            model_id: entry.model_id,
            status: entry.status,
            run_path: entry.run_path,
            started_at: entry.started_at,
            completed_at: entry.completed_at,
            output_preview: entry.output_preview,
            error: entry.error,
            online: entry.online,
            usage: entry.usage,
        })
        .collect())
}

pub fn list_project_run_history(root_path: &Path) -> StoreResult<Vec<ProjectRunHistoryEntry>> {
    let (root_path, _) = validate_project(root_path)?;
    read_run_history_entries(&root_path, None)
}

pub fn get_project_usage_summary(root_path: &Path) -> StoreResult<ProjectUsageSummary> {
    let (root_path, _) = validate_project(root_path)?;
    let entries = read_run_history_entries(&root_path, None)?;
    let mut summary = ProjectUsageSummary {
        total_runs: 0,
        successful_runs: 0,
        failed_runs: 0,
        total_prompt_tokens: 0,
        total_completion_tokens: 0,
        total_tokens: 0,
        total_cost: 0.0,
        total_output_words: 0,
        total_retries: 0,
    };
    for entry in &entries {
        summary.total_runs += 1;
        match entry.status {
            ExecutionStatus::Success => summary.successful_runs += 1,
            ExecutionStatus::Failed => summary.failed_runs += 1,
        }
        summary.total_prompt_tokens += entry.usage.prompt_tokens.unwrap_or(0) as u64;
        summary.total_completion_tokens += entry.usage.completion_tokens.unwrap_or(0) as u64;
        summary.total_tokens += entry.usage.total_tokens.unwrap_or(0) as u64;
        summary.total_cost += entry.usage.cost.unwrap_or(0.0);
        summary.total_output_words += entry.usage.output_word_count.unwrap_or(0) as u64;
        summary.total_retries += entry.usage.retry_count.unwrap_or(0);
    }
    Ok(summary)
}

fn read_run_history_entries(
    root_path: &Path,
    path_filter: Option<&str>,
) -> StoreResult<Vec<ProjectRunHistoryEntry>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(root_path.join("runs"))?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let record = match serde_json::from_str::<PersistedRunRecord>(&content) {
            Ok(record) => record,
            Err(_) => continue,
        };

        if let Some(filter) = path_filter {
            if record.path != filter {
                continue;
            }
        }

        let run_path = diff_path(&root_path, &path)?;
        entries.push(ProjectRunHistoryEntry {
            run_id: record.run_id,
            path: record.path,
            block_id: record.block_id,
            block_name: record.block_name,
            pipeline_id: record.pipeline_id,
            pipeline_name: record.pipeline_name,
            model_id: record.model_id,
            status: record.status,
            run_path,
            started_at: record.started_at,
            completed_at: record.completed_at,
            output_preview: record.output.as_deref().map(|value| preview_text(value, 180)),
            error: record.error,
            online: record.online,
            usage: record.usage,
        });
    }

    entries.sort_by(|left, right| right.completed_at.cmp(&left.completed_at));
    Ok(entries)
}

fn execute_prompt_block_with_transport<F>(
    root_path: &Path,
    manifest: &ProjectManifest,
    relative_path: &str,
    content: &str,
    pipeline_context: Option<&PipelineExecutionContext>,
    api_key: &str,
    transport: &mut F,
    app_data_dir: &Path,
) -> StoreResult<PromptExecutionResult>
where
    F: FnMut(&str, Value) -> StoreResult<Value>,
{
    let started_at = timestamp();
    let safe_relative = sanitize_relative_path(relative_path)?;
    let safe_relative_string = safe_relative.to_string_lossy().replace('\\', "/");

    if classify_asset(&safe_relative_string, false) != AssetKind::Tera {
        return Err(ProjectStoreError::message(
            "Prompt execution is only available for `.tera` prompt files.",
        ));
    }

    let linked_block = manifest
        .prompt_blocks
        .iter()
        .find(|block| block.template_source == safe_relative_string);
    let model_preset = linked_block
        .and_then(|block| block.model_preset.clone())
        .unwrap_or_else(|| manifest.default_model_preset.clone());
    let base_model_id = load_model_id_from_preset(&root_path, &model_preset)?;
    let online_enabled = prompt_uses_online_research(content);
    let model_id = execution_model_id(&base_model_id, online_enabled);

    let prepared = prepare_template_context(
        &root_path,
        &manifest,
        content,
        true,
        Some(model_id.clone()),
        Some(app_data_dir),
    )?;

    if !prepared.errors.is_empty() {
        return Err(ProjectStoreError::message(prepared.errors.join("\n")));
    }

    let rendered_prompt = render_template_for_execution(&prepared.content, &prepared.context)?;
    let model_config = load_model_preset_config(&root_path, &model_preset)?;
    let payload = build_openrouter_payload(model_config, &rendered_prompt, &model_id, online_enabled);

    let mut retry_count = 0u32;
    let (response, output) = loop {
        match transport(api_key, payload.clone()) {
            Ok(response) => {
                let output = extract_completion_text(&response);
                if !output.trim().is_empty() {
                    break (response, output);
                }
                if retry_count < MAX_EXECUTION_RETRIES {
                    retry_count += 1;
                    continue;
                }
                return Err(ProjectStoreError::message(
                    "OpenRouter returned an empty response body.",
                ));
            }
            Err(err) => {
                if retry_count < MAX_EXECUTION_RETRIES {
                    retry_count += 1;
                    continue;
                }
                return Err(err);
            }
        }
    };

    let run_id = format!("run-{}", Uuid::new_v4());
    let run_path = format!("runs/{run_id}.json");
    let completed_at = timestamp();
    let block_id = linked_block.as_ref().map(|block| block.block_id.clone());
    let block_name = linked_block
        .as_ref()
        .map(|block| block.name.clone())
        .unwrap_or_else(|| {
            Path::new(&safe_relative_string)
                .file_stem()
                .and_then(|name| name.to_str())
                .unwrap_or("Prompt")
                .to_string()
        });
        
    let output_target = linked_block
        .map(|block| block.output_target.clone())
        .unwrap_or_else(|| "run_artifact".to_string());

    let trimmed_output = output.trim().to_string();
    let mut document_path = None;
    let mut final_output_for_json = Some(trimmed_output.clone());

    if output_target == "document" || output_target == "both" {
        let block_slug = slugify_prompt_name(&block_name);
        let doc_filename = format!("{block_slug}.md");
        let doc_relative_path = format!("documents/{doc_filename}");
        let doc_absolute_path = root_path.join("documents").join(&doc_filename);
        
        if let Some(parent) = doc_absolute_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        
        if let Err(e) = fs::write(&doc_absolute_path, &trimmed_output) {
            return Err(ProjectStoreError::message(format!("Failed to write document output: {e}")));
        }
        document_path = Some(doc_relative_path);
        
        if output_target == "document" {
            final_output_for_json = None;
        }
    }

    let result = PromptExecutionResult {
        run_id: run_id.clone(),
        path: safe_relative_string.clone(),
        block_id,
        block_name,
        pipeline_id: pipeline_context.map(|context| context.pipeline_id.clone()),
        pipeline_name: pipeline_context.map(|context| context.pipeline_name.clone()),
        model_preset,
        model_id,
        status: ExecutionStatus::Success,
        output_target,
        document_path,
        variables: prepared.variables,
        output: final_output_for_json,
        error: None,
        run_path: run_path.clone(),
        started_at,
        completed_at,
        online: extract_online_run_metadata(&response, online_enabled),
        usage: {
            let mut metrics = extract_usage_metrics(&response, Some(&trimmed_output));
            metrics.retry_count = if retry_count > 0 { Some(retry_count) } else { None };
            metrics
        },
    };

    persist_run_record(&root_path, &result)?;
    Ok(result)
}

fn execute_pipeline_with_transport<F>(
    root_path: &Path,
    manifest: &ProjectManifest,
    pipeline_id: &str,
    api_key: &str,
    transport: &mut F,
    app_data_dir: &Path,
) -> StoreResult<PipelineExecutionResult>
where
    F: FnMut(&str, Value) -> StoreResult<Value>,
{
    let pipeline = manifest
        .pipelines
        .iter()
        .find(|pipeline| pipeline.pipeline_id == pipeline_id)
        .ok_or_else(|| ProjectStoreError::message(format!("Pipeline `{pipeline_id}` was not found.")))?;

    if pipeline.execution_mode != "sequential" {
        return Err(ProjectStoreError::message(format!(
            "Pipeline `{}` must use sequential execution mode.",
            pipeline.name
        )));
    }

    if pipeline.ordered_blocks.is_empty() {
        return Err(ProjectStoreError::message(format!(
            "Pipeline `{}` has no blocks to run.",
            pipeline.name
        )));
    }

    let started_at = timestamp();
    let mut steps = Vec::new();
    let mut status = ExecutionStatus::Success;
    let mut error = None;
    let pipeline_context = PipelineExecutionContext {
        pipeline_id: pipeline.pipeline_id.clone(),
        pipeline_name: pipeline.name.clone(),
    };

    for block_id in &pipeline.ordered_blocks {
        let block = manifest
            .prompt_blocks
            .iter()
            .find(|block| &block.block_id == block_id)
            .ok_or_else(|| {
                ProjectStoreError::message(format!(
                    "Pipeline `{}` references missing block `{block_id}`.",
                    pipeline.name
                ))
            })?;

        let template_path = sanitize_relative_path(&block.template_source)?;
        let relative_path = template_path.to_string_lossy().replace('\\', "/");
        let full_path = root_path.join(&template_path);
        if !full_path.is_file() {
            return Err(ProjectStoreError::message(format!(
                "Pipeline block `{}` is missing template `{}` on disk.",
                block.name, block.template_source
            )));
        }

        let content = fs::read_to_string(&full_path)?;
        match execute_prompt_block_with_transport(
            root_path,
            manifest,
            &relative_path,
            &content,
            Some(&pipeline_context),
            api_key,
            transport,
            app_data_dir,
        ) {
            Ok(result) => steps.push(result),
            Err(step_error) => {
                status = ExecutionStatus::Failed;
                error = Some(format!("{}: {}", block.name, step_error));
                break;
            }
        }
    }

    Ok(PipelineExecutionResult {
        pipeline_id: pipeline.pipeline_id.clone(),
        pipeline_name: pipeline.name.clone(),
        status,
        started_at,
        completed_at: timestamp(),
        error,
        steps,
    })
}

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
                }
            } else {
                ProjectPipelineBlockSummary {
                    block_id: block_id.clone(),
                    name: block_id.clone(),
                    template_source: String::new(),
                    model_preset: manifest.default_model_preset.clone(),
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

fn summarize_project(root_path: &Path, manifest: &ProjectManifest) -> StoreResult<ProjectSummary> {
    let counts = ProjectCounts {
        documents: count_files(root_path.join("documents"))?,
        prompts: count_files(root_path.join("prompts"))?,
        models: count_files(root_path.join("models"))?,
        runs: count_files(root_path.join("runs"))?,
        exports: count_files(root_path.join("exports"))?,
    };

    let variables = manifest
        .variables
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
        .collect();

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

fn validate_project(root_path: &Path) -> StoreResult<(PathBuf, ProjectManifest)> {
    if !root_path.exists() || !root_path.is_dir() {
        return Err(ProjectStoreError::message("The selected project folder does not exist."));
    }

    let root_path = root_path.canonicalize()?;
    for directory in PROJECT_DIRS {
        let directory_path = root_path.join(directory);
        if !directory_path.is_dir() {
            return Err(ProjectStoreError::message(format!(
                "Project folder is missing required directory `{directory}`."
            )));
        }
    }

    let manifest_path = root_path.join("project.json");
    if !manifest_path.is_file() {
        return Err(ProjectStoreError::message("Project folder is missing `project.json`."));
    }

    let manifest = read_manifest(&manifest_path)?;
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

fn sanitize_relative_path(relative_path: &str) -> StoreResult<PathBuf> {
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

fn update_recent_projects(app_data_dir: &Path, summary: &ProjectSummary) -> StoreResult<()> {
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

fn read_recents_store(app_data_dir: &Path) -> StoreResult<RecentsStore> {
    let store_path = app_data_dir.join(RECENTS_FILE_NAME);
    if !store_path.exists() {
        return Ok(RecentsStore::default());
    }

    let content = fs::read_to_string(store_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_global_variables_store(app_data_dir: &Path) -> BTreeMap<String, String> {
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

fn write_global_variables_store(app_data_dir: &Path, variables: &BTreeMap<String, String>) -> StoreResult<()> {
    let store = GlobalVariablesStore { variables: variables.clone() };
    let store_path = app_data_dir.join(GLOBAL_VARIABLES_FILE_NAME);
    fs::write(store_path, serde_json::to_string_pretty(&store)?)?;
    Ok(())
}

fn read_manifest(manifest_path: &Path) -> StoreResult<ProjectManifest> {
    let content = fs::read_to_string(manifest_path)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_manifest(root_path: &Path, manifest: &ProjectManifest) -> StoreResult<()> {
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

fn unique_pipeline_slug(manifest: &ProjectManifest, pipeline_name: &str) -> String {
    let base_slug = slugify_prompt_name(pipeline_name);
    let mut candidate = base_slug.clone();
    let mut suffix = 2usize;

    loop {
        let taken = manifest
            .pipelines
            .iter()
            .any(|pipeline| pipeline.pipeline_id == candidate);

        if !taken {
            return candidate;
        }

        candidate = format!("{base_slug}-{suffix}");
        suffix += 1;
    }
}

fn unique_export_slug(root_path: &Path, export_name: &str) -> String {
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

fn slugify_prompt_name(prompt_name: &str) -> String {
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

fn classify_asset(relative_path: &str, is_directory: bool) -> AssetKind {
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

fn diff_path(root_path: &Path, path: &Path) -> StoreResult<String> {
    path.strip_prefix(root_path)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .map_err(|_| ProjectStoreError::message("Asset path escaped the project root."))
}

fn detail(label: &str, value: &str) -> MetadataField {
    MetadataField {
        label: label.to_string(),
        value: value.to_string(),
    }
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

fn build_validation_result(
    path: String,
    preview: Option<String>,
    warnings: Vec<String>,
    errors: Vec<String>,
    manifest: &ProjectManifest,
    model_id: &str,
) -> TemplateValidationResult {
    let status = if !errors.is_empty() {
        ValidationStatus::Invalid
    } else if !warnings.is_empty() {
        ValidationStatus::Warnings
    } else {
        ValidationStatus::Valid
    };

    TemplateValidationResult {
        path,
        status,
        preview,
        warnings,
        errors,
        context_summary: vec![
            detail("Project", &manifest.project_name),
            detail("Project ID", &manifest.project_id),
            detail("Default Preset", &manifest.default_model_preset),
            detail("Model ID", model_id),
            detail("Variables", &manifest.variables.len().to_string()),
        ],
    }
}

struct PreparedTemplateContext {
    content: String,
    context: Context,
    model_id: String,
    variables: std::collections::BTreeMap<String, String>,
    warnings: Vec<String>,
    errors: Vec<String>,
}

fn prepare_template_context(
    root_path: &Path,
    manifest: &ProjectManifest,
    content: &str,
    strict_doc_references: bool,
    model_id_override: Option<String>,
    app_data_dir: Option<&Path>,
) -> StoreResult<PreparedTemplateContext> {
    let model_id = model_id_override
        .or_else(|| default_model_id(root_path, manifest))
        .unwrap_or_else(|| "Unknown".to_string());
    let doc_refs = preprocess_doc_references(root_path, content, strict_doc_references)?;

    let mut context = Context::new();
    context.insert(
        "project",
        &json!({
            "id": manifest.project_id,
            "name": manifest.project_name,
            "default_model_preset": manifest.default_model_preset,
            "updated_at": manifest.updated_at,
        }),
    );
    context.insert("model_id", &model_id);
    context.insert("now_iso", &timestamp());
    context.insert("current_date", &Utc::now().format("%Y-%m-%d").to_string());

    let mut resolved_variables = std::collections::BTreeMap::new();

    // Global variables — lowest priority; project variables override these.
    if let Some(app_data_dir) = app_data_dir {
        let global_vars = read_global_variables_store(app_data_dir);
        context.insert("global_variables", &global_vars);
        for (name, value) in &global_vars {
            if is_identifier_like(name) {
                context.insert(name, value);
                resolved_variables.insert(name.clone(), value.clone());
            }
        }
    }

    // Project variables — override globals with the same name.
    context.insert("variables", &manifest.variables);
    for (name, value) in &manifest.variables {
        if is_identifier_like(name) {
            context.insert(name, value);
            
            let stringified = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            resolved_variables.insert(name.clone(), stringified);
        }
    }

    let PreparedDocReferences {
        content,
        warnings,
        errors,
        bindings,
    } = doc_refs;

    for (name, value) in bindings {
        context.insert(&name, &value);
    }

    Ok(PreparedTemplateContext {
        content,
        context,
        model_id,
        variables: resolved_variables,
        warnings,
        errors,
    })
}

#[derive(Debug)]
struct PreparedDocReferences {
    content: String,
    warnings: Vec<String>,
    errors: Vec<String>,
    bindings: BTreeMap<String, String>,
}

fn preprocess_doc_references(
    root_path: &Path,
    content: &str,
    strict: bool,
) -> StoreResult<PreparedDocReferences> {
    let regex = Regex::new(r#"\{\{\s*doc\(\s*"([^"]+)"\s*\)\s*\}\}"#)
        .map_err(|error| ProjectStoreError::message(error.to_string()))?;
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut bindings = BTreeMap::new();
    let mut index = 0usize;

    let rendered = regex
        .replace_all(content, |captures: &regex::Captures<'_>| {
            let requested_path = captures.get(1).map(|capture| capture.as_str()).unwrap_or_default();
            let binding_name = format!("diamond_doc_ref_{index}");
            index += 1;

            let replacement = match sanitize_relative_path(requested_path) {
                Ok(safe_path) => {
                    let document_path = root_path.join("documents").join(&safe_path);
                    match fs::read_to_string(&document_path) {
                        Ok(document_content) => document_content,
                        Err(_) => {
                            let message = format!(
                                "Document reference `{requested_path}` could not be resolved from `documents/`."
                            );
                            if strict {
                                errors.push(message);
                            } else {
                                warnings.push(message);
                            }
                            format!("[Missing document: {requested_path}]")
                        }
                    }
                }
                Err(_) => {
                    let message = format!(
                        "Document reference `{requested_path}` is invalid and could not be resolved."
                    );
                    if strict {
                        errors.push(message);
                    } else {
                        warnings.push(message);
                    }
                    format!("[Invalid document reference: {requested_path}]")
                }
            };

            bindings.insert(binding_name.clone(), replacement);
            format!("{{{{ {binding_name} }}}}")
        })
        .to_string();

    Ok(PreparedDocReferences {
        content: rendered,
        warnings,
        errors,
        bindings,
    })
}

fn render_template_for_execution(content: &str, context: &Context) -> StoreResult<String> {
    let mut tera = Tera::default();
    tera.autoescape_on(Vec::new());
    tera.add_raw_template("active", content)
        .map_err(|error| ProjectStoreError::message(error.to_string()))?;

    tera.render("active", context)
        .map_err(|error| ProjectStoreError::message(execution_render_error_message(&flatten_error_chain(&error))))
}

fn execution_render_error_message(message: &str) -> String {
    if is_missing_context_warning(message) {
        format!(
            "Execution requires all referenced variables to resolve unless the template guards them with `is defined` or a default.\n{message}"
        )
    } else {
        message.to_string()
    }
}

fn flatten_error_chain(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(e) = cause {
        message.push('\n');
        message.push_str(&e.to_string());
        cause = e.source();
    }
    message
}

fn load_model_preset_config(root_path: &Path, preset_path: &str) -> StoreResult<serde_json::Map<String, Value>> {
    let content = fs::read_to_string(root_path.join(preset_path))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let json_value = serde_json::to_value(yaml)?;
    match json_value {
        Value::Object(object) => Ok(object),
        _ => Err(ProjectStoreError::message("Model preset must be a YAML mapping.")),
    }
}

fn load_model_id_from_preset(root_path: &Path, preset_path: &str) -> StoreResult<String> {
    let content = fs::read_to_string(root_path.join(preset_path))?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    yaml.get("model")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| ProjectStoreError::message("Model preset is missing `model`."))
}

fn prompt_uses_online_research(content: &str) -> bool {
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(is_online_directive_line)
        .unwrap_or(false)
}

fn is_online_directive_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("{#") || !trimmed.ends_with("#}") {
        return false;
    }

    let inner = trimmed
        .trim_start_matches("{#")
        .trim_end_matches("#}")
        .trim();
    inner.eq_ignore_ascii_case(ONLINE_PROMPT_DIRECTIVE)
}

fn execution_model_id(base_model_id: &str, online_enabled: bool) -> String {
    if !online_enabled || base_model_id.ends_with(":online") {
        return base_model_id.to_string();
    }

    format!("{base_model_id}:online")
}

fn build_openrouter_payload(
    mut model_config: serde_json::Map<String, Value>,
    prompt: &str,
    model_id: &str,
    online_enabled: bool,
) -> Value {
    model_config.insert("model".to_string(), Value::String(model_id.to_string()));
    model_config.insert(
        "messages".to_string(),
        json!([
            {
                "role": "user",
                "content": prompt,
            }
        ]),
    );

    if online_enabled {
        model_config.insert(
            "plugins".to_string(),
            json!([
                {
                    "id": "web",
                    "max_results": DEFAULT_ONLINE_WEB_MAX_RESULTS,
                }
            ]),
        );
        model_config.insert(
            "web_search_options".to_string(),
            json!({
                "search_context_size": DEFAULT_ONLINE_SEARCH_CONTEXT_SIZE,
            }),
        );
    }

    Value::Object(model_config)
}

fn extract_online_run_metadata(response: &Value, online_enabled: bool) -> OnlineRunMetadata {
    let web_search_requests = response
        .get("usage")
        .and_then(|usage| usage.get("server_tool_use"))
        .and_then(|tools| tools.get("web_search_requests"))
        .and_then(|value| value.as_u64())
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0);
    let citation_count = response
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("annotations"))
        .and_then(|annotations| annotations.as_array())
        .map(|annotations| annotations.len() as u32)
        .unwrap_or(0);

    OnlineRunMetadata {
        enabled: online_enabled,
        web_search_requests,
        citation_count,
    }
}

fn extract_usage_metrics(response: &Value, output: Option<&str>) -> UsageMetrics {
    let usage = response.get("usage");
    let prompt_tokens = usage
        .and_then(|u| u.get("prompt_tokens"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    let completion_tokens = usage
        .and_then(|u| u.get("completion_tokens"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    let total_tokens = usage
        .and_then(|u| u.get("total_tokens"))
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok());
    let cost = usage
        .and_then(|u| u.get("cost"))
        .and_then(|v| v.as_f64());
    let output_word_count = output.map(|text| text.split_whitespace().count() as u32);

    UsageMetrics {
        prompt_tokens,
        completion_tokens,
        total_tokens,
        cost,
        output_word_count,
        retry_count: None,
    }
}

fn post_openrouter_chat_completion(url: &str, api_key: &str, payload: &Value) -> StoreResult<Value> {
    let response = reqwest::blocking::Client::new()
        .post(url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(payload)
        .send()?;

    let status = response.status();
    let body = response.text()?;
    let json_body: Value = serde_json::from_str(&body).map_err(|_| {
        ProjectStoreError::message(format!("OpenRouter returned non-JSON response: {body}"))
    })?;

    if !status.is_success() {
        let message = json_body
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(|value| value.as_str())
            .unwrap_or(&body);
        return Err(ProjectStoreError::message(format!(
            "OpenRouter error {}: {message}",
            status.as_u16()
        )));
    }

    Ok(json_body)
}

fn extract_completion_text(response: &Value) -> String {
    response
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .map(|content| match content {
            Value::String(text) => text.clone(),
            Value::Array(parts) => parts
                .iter()
                .filter_map(|part| {
                    part.get("text")
                        .and_then(|value| value.as_str())
                        .map(|text| text.to_string())
                })
                .collect::<Vec<_>>()
                .join("\n"),
            other => other.to_string(),
        })
        .unwrap_or_default()
}

fn persist_run_record(root_path: &Path, result: &PromptExecutionResult) -> StoreResult<()> {
    let run_record = PersistedRunRecord::from_result(result);
    fs::write(root_path.join(&result.run_path), serde_json::to_string_pretty(&run_record)?)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedRunRecord {
    #[serde(default = "persisted_run_record_version")]
    artifact_version: u32,
    run_id: String,
    path: String,
    #[serde(default)]
    block_id: Option<String>,
    block_name: String,
    #[serde(default)]
    pipeline_id: Option<String>,
    #[serde(default)]
    pipeline_name: Option<String>,
    #[serde(default)]
    model_preset: String,
    model_id: String,
    status: ExecutionStatus,
    output_target: String,
    #[serde(default)]
    document_path: Option<String>,
    #[serde(default)]
    variables: std::collections::BTreeMap<String, String>,
    output: Option<String>,
    error: Option<String>,
    started_at: String,
    completed_at: String,
    #[serde(default)]
    online: OnlineRunMetadata,
    #[serde(default)]
    usage: UsageMetrics,
}

impl PersistedRunRecord {
    fn from_result(result: &PromptExecutionResult) -> Self {
        Self {
            artifact_version: PERSISTED_RUN_RECORD_VERSION,
            run_id: result.run_id.clone(),
            path: result.path.clone(),
            block_id: result.block_id.clone(),
            block_name: result.block_name.clone(),
            pipeline_id: result.pipeline_id.clone(),
            pipeline_name: result.pipeline_name.clone(),
            model_preset: result.model_preset.clone(),
            model_id: result.model_id.clone(),
            status: result.status.clone(),
            output_target: result.output_target.clone(),
            document_path: result.document_path.clone(),
            variables: result.variables.clone(),
            output: result.output.clone(),
            error: result.error.clone(),
            started_at: result.started_at.clone(),
            completed_at: result.completed_at.clone(),
            online: result.online.clone(),
            usage: result.usage.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct PipelineExecutionContext {
    pipeline_id: String,
    pipeline_name: String,
}

fn persisted_run_record_version() -> u32 {
    PERSISTED_RUN_RECORD_VERSION
}

fn preview_text(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let preview = trimmed.chars().take(max_chars).collect::<String>();
    format!("{}...", preview.trim_end())
}

fn load_execution_api_key() -> StoreResult<String> {
    let stored_key = load_stored_openrouter_api_key()?;
    let environment_key = load_environment_api_key();

    select_openrouter_api_key(stored_key, environment_key).ok_or_else(|| {
        ProjectStoreError::message(format!(
            "Missing OpenRouter API key. Save one in the app or set {OPENROUTER_API_KEY_ENV}."
        ))
    })
}

fn load_stored_openrouter_api_key() -> StoreResult<Option<String>> {
    match openrouter_keyring_entry()?.get_password() {
        Ok(password) if password.trim().is_empty() => Ok(None),
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(keyring_error(error)),
    }
}

fn load_environment_api_key() -> Option<String> {
    match env::var(OPENROUTER_API_KEY_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

fn select_openrouter_api_key(
    stored_key: Option<String>,
    environment_key: Option<String>,
) -> Option<String> {
    stored_key
        .filter(|value| !value.trim().is_empty())
        .or_else(|| environment_key.filter(|value| !value.trim().is_empty()))
}

fn build_execution_credential_status(
    has_stored_key: bool,
    has_environment_key: bool,
) -> ExecutionCredentialStatus {
    let source = if has_stored_key {
        CredentialSource::Keychain
    } else if has_environment_key {
        CredentialSource::Environment
    } else {
        CredentialSource::Missing
    };

    ExecutionCredentialStatus {
        source,
        has_stored_key,
    }
}

fn openrouter_keyring_entry() -> StoreResult<keyring::Entry> {
    keyring::Entry::new(OPENROUTER_KEYCHAIN_SERVICE, OPENROUTER_KEYCHAIN_ACCOUNT)
        .map_err(keyring_error)
}

fn keyring_error(error: keyring::Error) -> ProjectStoreError {
    ProjectStoreError::message(format!("Credential storage failed: {error}"))
}

fn default_model_id(root_path: &Path, manifest: &ProjectManifest) -> Option<String> {
    let model_path = root_path.join(&manifest.default_model_preset);
    let content = fs::read_to_string(model_path).ok()?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    yaml.get("model").and_then(|value| value.as_str()).map(|value| value.to_string())
}

fn is_missing_context_warning(message: &str) -> bool {
    message.contains("not found in context") || message.contains("Variable `")
}

fn is_identifier_like(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn timestamp() -> String {
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
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.block_id.as_deref(), Some("block-1"));
        assert_eq!(result.output.as_deref(), Some("Execution output."));
        assert!(result.rendered_prompt.contains("Doc body."));
        assert!(root.join(&result.run_path).is_file());

        let persisted = serde_json::from_str::<PersistedRunRecord>(
            &fs::read_to_string(root.join(&result.run_path)).unwrap(),
        )
        .unwrap();
        assert_eq!(persisted.artifact_version, 4);
        assert_eq!(persisted.block_id.as_deref(), Some("block-1"));
        assert_eq!(persisted.model_preset, "models/default.yaml");
        assert!(persisted.rendered_prompt.contains("Doc body."));
        assert!(!persisted.online.enabled);
        assert!(persisted.response.get("choices").is_some());

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
            "test-key",
            &mut transport,
            &app_data,
        )
        .unwrap();

        assert_eq!(result.model_preset, "models/review.yaml");
        assert_eq!(result.model_id, "openai/gpt-5.4-nano");
    }

    #[test]
    fn prefers_stored_execution_api_key_over_environment() {
        let selected = select_openrouter_api_key(
            Some("stored-key".to_string()),
            Some("env-key".to_string()),
        );

        assert_eq!(selected.as_deref(), Some("stored-key"));
    }

    #[test]
    fn falls_back_to_environment_execution_api_key() {
        let selected = select_openrouter_api_key(None, Some("env-key".to_string()));

        assert_eq!(selected.as_deref(), Some("env-key"));
    }

    #[test]
    fn reports_missing_execution_credentials_when_nothing_is_available() {
        let status = build_execution_credential_status(false, false);

        assert_eq!(status.source, CredentialSource::Missing);
        assert!(!status.has_stored_key);
    }

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
        });
        manifest.pipelines.push(Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["block-1".to_string()],
            execution_mode: "sequential".to_string(),
        });
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
        });
        write_manifest(&root, &manifest).unwrap();

        let prompt_blocks = list_project_prompt_blocks(&root).unwrap();

        assert_eq!(prompt_blocks.len(), 1);
        assert_eq!(prompt_blocks[0].block_id, "review");
        assert_eq!(prompt_blocks[0].model_preset, "models/default.yaml");
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
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "outline".to_string(),
            name: "Outline".to_string(),
            template_source: "prompts/outline.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
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

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        assert_eq!(manifest.pipelines.len(), 1);
        assert_eq!(manifest.pipelines[0].name, "Draft Pipeline Revised");
        assert_eq!(manifest.pipelines[0].ordered_blocks, vec!["review", "outline"]);
        assert_eq!(manifest.pipelines[0].execution_mode, "sequential");
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
        });
        manifest.prompt_blocks.push(PromptBlock {
            block_id: "outline".to_string(),
            name: "Outline".to_string(),
            template_source: "prompts/outline.tera".to_string(),
            input_bindings: Vec::new(),
            model_preset: None,
            output_target: "run_artifact".to_string(),
        });
        manifest.variables.insert("tone".to_string(), json!("precise"));
        manifest.pipelines.push(Pipeline {
            pipeline_id: "pipeline-1".to_string(),
            name: "Review Pipeline".to_string(),
            ordered_blocks: vec!["review".to_string(), "outline".to_string()],
            execution_mode: "sequential".to_string(),
        });
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

        let result = execute_pipeline_with_transport(
            &root,
            &manifest,
            "pipeline-1",
            "test-key",
            &mut transport,
            &app_data,
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
        set_project_variables(&root, project_vars, &app_data).unwrap();

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

        set_project_variables(&root, vars, &app_data).unwrap();

        let manifest = read_manifest(&root.join("project.json")).unwrap();
        assert_eq!(manifest.variables["chapter"], Value::String("12".to_string()));
        assert_eq!(manifest.variables["word_target"], Value::String("5000".to_string()));
    }
}
