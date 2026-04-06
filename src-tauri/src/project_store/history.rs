use std::{
    fs,
    path::{Component, Path},
};

use crate::types::*;
use super::{diff_path, preview_text, sanitize_relative_path, validate_project};
use super::execution::PersistedRunRecord;

// ── Public API ────────────────────────────────────────────────────────────────

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

pub fn delete_run(root_path: &Path, run_path: &str) -> StoreResult<()> {
    let (root_path, _) = validate_project(root_path)?;
    let safe_relative = sanitize_relative_path(run_path)?;
    // Restrict deletions to the runs/ directory only.
    let first_component = safe_relative.components().next();
    if first_component != Some(Component::Normal("runs".as_ref())) {
        return Err(ProjectStoreError::message(
            "Run artifact path must be within the runs/ directory.",
        ));
    }
    let full_path = root_path.join(&safe_relative);
    if !full_path.is_file() {
        return Err(ProjectStoreError::message(format!(
            "Run artifact not found: {run_path}"
        )));
    }
    fs::remove_file(full_path)?;
    Ok(())
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

// ── Private helpers ──────────────────────────────────────────────────────────

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
