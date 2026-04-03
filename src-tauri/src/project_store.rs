use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;
use uuid::Uuid;

const PROJECT_DIRS: [&str; 5] = ["documents", "prompts", "models", "runs", "exports"];
const RECENTS_FILE_NAME: &str = "recent-projects.json";
const DEFAULT_MODEL_PRESET_CONTENT: &str = "# Default fallback model used when no stronger override applies.\nmodel: openai/gpt-5.4\ntemperature: 0.7\nmax_completion_tokens: 12000\n";

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
    pub metadata: AssetMetadata,
    pub parsed_json: Option<Value>,
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

    fs::write(root_path.join("models").join("default.yaml"), DEFAULT_MODEL_PRESET_CONTENT)?;

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
        metadata,
        parsed_json,
    })
}

fn build_tree_node(root_path: &Path, full_path: &Path, relative_path: String) -> StoreResult<ProjectAssetNode> {
    let metadata = fs::metadata(full_path)?;
    let is_directory = metadata.is_dir();
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
        name: full_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        path: relative_path.clone(),
        kind: classify_asset(&relative_path, is_directory),
        is_directory,
        children,
    })
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

    Ok(ProjectSummary {
        root_path: root_path.to_string_lossy().to_string(),
        project_id: manifest.project_id.clone(),
        project_name: manifest.project_name.clone(),
        default_model_preset: manifest.default_model_preset.clone(),
        updated_at: manifest.updated_at.clone(),
        counts,
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
            let yaml: serde_yaml::Value = serde_yaml::from_str(content)?;
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
        _ => {
            details.push(detail("Status", "Unsupported in milestone 1"));
            details.push(detail("Project Root", &root_path.to_string_lossy()));
        }
    }

    Ok(AssetMetadata {
        kind: kind.clone(),
        path: relative_path.to_string(),
        name: full_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
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

fn read_recents_store(app_data_dir: &Path) -> StoreResult<RecentsStore> {
    let store_path = app_data_dir.join(RECENTS_FILE_NAME);
    if !store_path.exists() {
        return Ok(RecentsStore::default());
    }

    let content = fs::read_to_string(store_path)?;
    Ok(serde_json::from_str(&content)?)
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

        let content = read_project_asset(&root, "prompts/review.tera").unwrap();
        assert_eq!(content.kind, AssetKind::Tera);
        assert_eq!(content.view, AssetView::Text);
        assert!(content
            .metadata
            .details
            .iter()
            .any(|detail| detail.value.contains("Review")));
    }
}
