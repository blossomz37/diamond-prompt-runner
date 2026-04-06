use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tera::{Context, Tera};
use uuid::Uuid;

use crate::types::*;
use super::{
    classify_asset, default_model_id, detail, is_identifier_like, is_missing_context_warning,
    read_global_variables_store,
    read_workspace_variables_yaml, sanitize_relative_path,
    slugify_prompt_name, timestamp, validate_project,
};
use super::credentials::{
    load_environment_api_key, load_stored_openrouter_api_key, select_openrouter_api_key,
    OPENROUTER_API_KEY_ENV,
};

const OPENROUTER_CHAT_COMPLETIONS_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const PERSISTED_RUN_RECORD_VERSION: u32 = 4;
const MAX_EXECUTION_RETRIES: u32 = 2;
const ONLINE_PROMPT_DIRECTIVE: &str = "diamond:online";
const DEFAULT_ONLINE_WEB_MAX_RESULTS: u32 = 3;
const DEFAULT_ONLINE_SEARCH_CONTEXT_SIZE: &str = "medium";

// ── Public API ────────────────────────────────────────────────────────────────

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

    let prepared = prepare_template_context(&root_path, &manifest, content, false, None, None, Some(app_data_dir))?;
    let model_id = prepared.model_id.clone();
    let mut warnings = prepared.warnings;

    let mut errors = Vec::new();
    let mut tera = build_tera(&root_path);
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
    payload: Option<BTreeMap<String, String>>,
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
        payload,
        &api_key,
        &mut transport,
        app_data_dir,
    )
}

pub fn execute_pipeline(
    root_path: &Path,
    pipeline_id: &str,
    payload: Option<BTreeMap<String, String>>,
    app_data_dir: &Path,
    resume_from_block_id: Option<String>,
    on_progress: Option<&mut dyn FnMut(PipelineProgressEvent)>,
    abort_signal: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> StoreResult<PipelineExecutionResult> {
    let api_key = load_execution_api_key()?;
    let (root_path, manifest) = validate_project(root_path)?;
    let mut transport = |api_key: &str, payload: Value| {
        post_openrouter_chat_completion(OPENROUTER_CHAT_COMPLETIONS_URL, api_key, &payload)
    };

    execute_pipeline_with_transport(
        &root_path,
        &manifest,
        pipeline_id,
        payload,
        &api_key,
        &mut transport,
        app_data_dir,
        resume_from_block_id,
        on_progress,
        abort_signal,
    )
}

// ── Private helpers ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) struct PipelineExecutionContext {
    pub(crate) pipeline_id: String,
    pub(crate) pipeline_name: String,
}

pub(crate) fn execute_prompt_block_with_transport<F>(
    root_path: &Path,
    manifest: &ProjectManifest,
    relative_path: &str,
    content: &str,
    pipeline_context: Option<&PipelineExecutionContext>,
    payload: Option<BTreeMap<String, String>>,
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
    let base_model_id = load_model_id_from_preset(root_path, &model_preset)?;
    let online_enabled = prompt_uses_online_research(content);
    let model_id = execution_model_id(&base_model_id, online_enabled);

    let prepared = prepare_template_context(
        root_path,
        manifest,
        content,
        true,
        Some(model_id.clone()),
        payload,
        Some(app_data_dir),
    )?;

    if !prepared.errors.is_empty() {
        return Err(ProjectStoreError::message(prepared.errors.join("\n")));
    }

    let rendered_prompt = render_template_for_execution(&prepared.content, &prepared.context, root_path)?;
    let model_config = load_model_preset_config(root_path, &model_preset)?;
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
    let output_filename_override = linked_block.as_ref().and_then(|block| block.output_filename.clone());

    let output_target = linked_block
        .map(|block| block.output_target.clone())
        .unwrap_or_else(|| "run_artifact".to_string());

    let trimmed_output = output.trim().to_string();
    let mut document_path = None;
    let mut final_output_for_json = Some(trimmed_output.clone());

    let writes_document = matches!(
        output_target.as_str(),
        "replace_document" | "append_document" | "document" | "both"
    );

    if writes_document {
        let doc_filename = match &output_filename_override {
            Some(name) => {
                if name.contains("{{") || name.contains("{%") {
                    render_template_for_execution(name, &prepared.context, root_path).unwrap_or_else(|_| name.clone())
                } else {
                    name.clone()
                }
            },
            None => {
                let block_slug = slugify_prompt_name(&block_name);
                format!("{block_slug}.md")
            }
        };
        let doc_relative_path = format!("documents/{doc_filename}");
        let doc_absolute_path = root_path.join("documents").join(&doc_filename);

        if let Some(parent) = doc_absolute_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let write_result = if output_target == "append_document" {
            // Append: read existing content and add new output
            let existing = if doc_absolute_path.is_file() {
                fs::read_to_string(&doc_absolute_path).unwrap_or_default()
            } else {
                String::new()
            };
            let combined = if existing.is_empty() {
                trimmed_output.clone()
            } else {
                format!("{existing}\n\n---\n\n{trimmed_output}")
            };
            fs::write(&doc_absolute_path, &combined)
        } else {
            // Replace (replace_document, document, both)
            fs::write(&doc_absolute_path, &trimmed_output)
        };

        if let Err(e) = write_result {
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

    persist_run_record(root_path, &result)?;
    Ok(result)
}

pub(crate) fn execute_pipeline_with_transport<F>(
    root_path: &Path,
    manifest: &ProjectManifest,
    pipeline_id: &str,
    payload: Option<BTreeMap<String, String>>,
    api_key: &str,
    transport: &mut F,
    app_data_dir: &Path,
    resume_from_block_id: Option<String>,
    mut on_progress: Option<&mut dyn FnMut(PipelineProgressEvent)>,
    abort_signal: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
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

    let mut starting_index = 0;
    if let Some(block_id) = &resume_from_block_id {
        if let Some(pos) = pipeline.ordered_blocks.iter().position(|id| id == block_id) {
            starting_index = pos;
        }
    }

    let total_blocks = pipeline.ordered_blocks.len();
    for i in starting_index..total_blocks {
        let block_id = &pipeline.ordered_blocks[i];
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

        if let Some(abort) = &abort_signal {
            if abort.load(std::sync::atomic::Ordering::Relaxed) {
                return Ok(PipelineExecutionResult {
                    pipeline_id: pipeline.pipeline_id.clone(),
                    pipeline_name: pipeline.name.clone(),
                    status: ExecutionStatus::Failed,
                    started_at,
                    completed_at: timestamp(),
                    error: Some("Pipeline execution cancelled by user.".to_string()),
                    steps,
                });
            }
        }

        if let Some(ref mut callback) = on_progress {
            callback(PipelineProgressEvent {
                pipeline_id: pipeline.pipeline_id.clone(),
                current_block_name: block.name.clone(),
                completed_blocks: i,
                total_blocks: total_blocks,
                status: "running".to_string(),
            });
        }

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
            payload.clone(),
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
    payload: Option<BTreeMap<String, String>>,
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
    // Read from YAML file first; fall back to project.json manifest.
    let yaml_vars = read_workspace_variables_yaml(root_path);
    if !yaml_vars.is_empty() {
        context.insert("variables", &yaml_vars);
        for (name, value) in &yaml_vars {
            if is_identifier_like(name) {
                context.insert(name, value);
                resolved_variables.insert(name.clone(), value.clone());
            }
        }
    } else {
        context.insert("variables", &manifest.variables);
        for (name, value) in &manifest.variables {
            if is_identifier_like(name) {
                context.insert(name, value);

                let stringified = match value {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => value.to_string(),
                };
                resolved_variables.insert(name.clone(), stringified);
            }
        }
    }

    if let Some(payload_vars) = payload {
        for (name, value) in &payload_vars {
            if is_identifier_like(name) {
                context.insert(name, value);
                resolved_variables.insert(name.clone(), value.clone());
            }
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
    let regex = Regex::new(r#"doc\(\s*"([^"]+)"\s*\)"#)
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
            binding_name
        })
        .to_string();

    Ok(PreparedDocReferences {
        content: rendered,
        warnings,
        errors,
        bindings,
    })
}

fn render_template_for_execution(content: &str, context: &Context, root_path: &Path) -> StoreResult<String> {
    let mut tera = build_tera(root_path);
    tera.add_raw_template("active", content)
        .map_err(|error| ProjectStoreError::message(flatten_error_chain(&error)))?;

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

pub(crate) fn load_model_id_from_preset(root_path: &Path, preset_path: &str) -> StoreResult<String> {
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

pub(crate) fn extract_usage_metrics(response: &Value, output: Option<&str>) -> UsageMetrics {
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
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new());

    let response = client
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

fn extract_section_filter(value: &Value, args: &std::collections::HashMap<String, Value>) -> tera::Result<Value> {
    let input = match value.as_str() {
        Some(s) => s,
        None => return Err(tera::Error::msg("extract_section filter requires a string input")),
    };

    let start_marker = args.get("start").and_then(|v| v.as_str());
    let end_marker = args.get("end").and_then(|v| v.as_str());

    if start_marker.is_none() && end_marker.is_none() {
        return Err(tera::Error::msg("extract_section requires at least a `start` or `end` argument"));
    }

    let mut start_idx = 0;
    if let Some(start_str) = start_marker {
        if let Some(idx) = input.find(start_str) {
            start_idx = idx + start_str.len();
        } else {
            return Ok(Value::String(format!("[extract_section warning: start marker '{}' not found]", start_str)));
        }
    }

    let slice_from_start = &input[start_idx..];

    let mut end_idx = slice_from_start.len();
    if let Some(end_str) = end_marker {
        if let Some(idx) = slice_from_start.find(&end_str) {
            end_idx = idx;
        }
    }

    Ok(Value::String(slice_from_start[..end_idx].trim().to_string()))
}

fn build_tera(root_path: &Path) -> Tera {
    let mut tera = Tera::default();
    tera.autoescape_on(Vec::new());
    tera.register_filter("extract_section", extract_section_filter);
    
    let doc_root = root_path.to_path_buf();
    tera.register_filter("doc", move |value: &Value, _: &std::collections::HashMap<String, Value>| -> tera::Result<Value> {
        let filename = match value.as_str() {
            Some(s) => s,
            None => return Err(tera::Error::msg("doc filter requires a string input")),
        };
        
        let path = doc_root.join("documents").join(filename);
        match fs::read_to_string(&path) {
            Ok(content) => Ok(Value::String(content)),
            Err(_) => Ok(Value::String(format!("[Missing document: {}]", filename))),
        }
    });
    
    tera
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
pub(crate) struct PersistedRunRecord {
    #[serde(default = "persisted_run_record_version")]
    pub(crate) artifact_version: u32,
    pub(crate) run_id: String,
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) block_id: Option<String>,
    pub(crate) block_name: String,
    #[serde(default)]
    pub(crate) pipeline_id: Option<String>,
    #[serde(default)]
    pub(crate) pipeline_name: Option<String>,
    #[serde(default)]
    pub(crate) model_preset: String,
    pub(crate) model_id: String,
    pub(crate) status: ExecutionStatus,
    #[serde(default)]
    pub(crate) output_target: String,
    #[serde(default)]
    pub(crate) document_path: Option<String>,
    #[serde(default)]
    pub(crate) variables: std::collections::BTreeMap<String, String>,
    pub(crate) output: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) started_at: String,
    pub(crate) completed_at: String,
    #[serde(default)]
    pub(crate) online: OnlineRunMetadata,
    #[serde(default)]
    pub(crate) usage: UsageMetrics,
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

fn persisted_run_record_version() -> u32 {
    PERSISTED_RUN_RECORD_VERSION
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
