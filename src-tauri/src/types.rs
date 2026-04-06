use std::collections::BTreeMap;

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

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
    pub(crate) fn message(message: impl Into<String>) -> Self {
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
    #[serde(default)]
    pub help: usize,
    #[serde(default)]
    pub variables: usize,
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
pub struct PipelineProgressEvent {
    pub pipeline_id: String,
    pub total_blocks: usize,
    pub completed_blocks: usize,
    pub current_block_name: String,
    pub status: String,
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
    pub output_target: String,
    pub output_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPromptBlockSummary {
    pub block_id: String,
    pub name: String,
    pub template_source: String,
    pub model_preset: String,
    pub output_target: String,
    pub output_filename: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_filename: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pipeline {
    pub pipeline_id: String,
    pub name: String,
    pub ordered_blocks: Vec<String>,
    pub execution_mode: String,
}
