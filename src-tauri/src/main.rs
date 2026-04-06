mod project_store;
mod types;
// execution is a submodule of project_store — declared there

use std::path::PathBuf;

use project_store::{
    AssetContent, CreatedPromptBlockResult, ExecutionCredentialStatus, ExportBundleResult,
    ModelPresetSummary, PipelineExecutionResult, ProjectAssetNode, ProjectPipelineSummary,
    ProjectPromptBlockSummary, ProjectRunHistoryEntry, ProjectSummary, ProjectUsageSummary,
    PromptExecutionResult, PromptRunHistoryEntry, RecentProjectEntry, SavedPipelineResult,
    TemplateValidationResult,
};
use std::collections::BTreeMap;
use tauri::Manager;

fn app_data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_project(
    app: tauri::AppHandle,
    parent_path: String,
    project_name: String,
) -> Result<ProjectSummary, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::create_project(PathBuf::from(parent_path).as_path(), &project_name, &app_data_dir)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_project(app: tauri::AppHandle, root_path: String) -> Result<ProjectSummary, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::open_project(PathBuf::from(root_path).as_path(), &app_data_dir)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_prompt_block(
    app: tauri::AppHandle,
    root_path: String,
    prompt_name: String,
) -> Result<CreatedPromptBlockResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::create_prompt_block(PathBuf::from(root_path).as_path(), &prompt_name, &app_data_dir)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_recent_projects(app: tauri::AppHandle) -> Result<Vec<RecentProjectEntry>, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::get_recent_projects(&app_data_dir).map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_recent_project(app: tauri::AppHandle, root_path: String) -> Result<(), String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::remove_recent_project(&app_data_dir, PathBuf::from(root_path).as_path())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn locate_recent_project(
    app: tauri::AppHandle,
    previous_root_path: String,
    candidate_root_path: String,
) -> Result<ProjectSummary, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::locate_recent_project(
        &app_data_dir,
        PathBuf::from(previous_root_path).as_path(),
        PathBuf::from(candidate_root_path).as_path(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_project_assets(root_path: String) -> Result<Vec<ProjectAssetNode>, String> {
    project_store::list_project_assets(PathBuf::from(root_path).as_path()).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_project_pipelines(root_path: String) -> Result<Vec<ProjectPipelineSummary>, String> {
    project_store::list_project_pipelines(PathBuf::from(root_path).as_path())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_project_prompt_blocks(root_path: String) -> Result<Vec<ProjectPromptBlockSummary>, String> {
    project_store::list_project_prompt_blocks(PathBuf::from(root_path).as_path())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_pipeline(
    app: tauri::AppHandle,
    root_path: String,
    pipeline_name: String,
    ordered_block_ids: Vec<String>,
) -> Result<SavedPipelineResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::create_pipeline(
        PathBuf::from(root_path).as_path(),
        &pipeline_name,
        &ordered_block_ids,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn update_pipeline(
    app: tauri::AppHandle,
    root_path: String,
    pipeline_id: String,
    pipeline_name: String,
    ordered_block_ids: Vec<String>,
) -> Result<SavedPipelineResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::update_pipeline(
        PathBuf::from(root_path).as_path(),
        &pipeline_id,
        &pipeline_name,
        &ordered_block_ids,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_project_assets(
    app: tauri::AppHandle,
    root_path: String,
    bundle_name: String,
    relative_paths: Vec<String>,
) -> Result<ExportBundleResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::export_project_assets(
        PathBuf::from(root_path).as_path(),
        &bundle_name,
        &relative_paths,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn read_project_asset(root_path: String, relative_path: String) -> Result<AssetContent, String> {
    project_store::read_project_asset(PathBuf::from(root_path).as_path(), &relative_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn write_project_asset(
    root_path: String,
    relative_path: String,
    content: String,
) -> Result<AssetContent, String> {
    project_store::write_project_asset(PathBuf::from(root_path).as_path(), &relative_path, &content)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn validate_project_template(
    app: tauri::AppHandle,
    root_path: String,
    relative_path: String,
    content: String,
) -> Result<TemplateValidationResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::validate_project_template(
        PathBuf::from(root_path).as_path(),
        &relative_path,
        &content,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn execute_prompt_block(
    app: tauri::AppHandle,
    root_path: String,
    relative_path: String,
    content: String,
    payload: Option<BTreeMap<String, String>>,
) -> Result<PromptExecutionResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::execute_prompt_block(
        PathBuf::from(root_path).as_path(),
        &relative_path,
        &content,
        payload,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn execute_pipeline(
    app: tauri::AppHandle,
    root_path: String,
    pipeline_id: String,
    payload: Option<BTreeMap<String, String>>,
) -> Result<PipelineExecutionResult, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::execute_pipeline(
        PathBuf::from(root_path).as_path(),
        &pipeline_id,
        payload,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_global_variables(app: tauri::AppHandle) -> Result<BTreeMap<String, String>, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::get_global_variables(&app_data_dir).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_global_variables(
    app: tauri::AppHandle,
    variables: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::set_global_variables(&app_data_dir, variables).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_project_variables(
    root_path: String,
    variables: BTreeMap<String, String>,
) -> Result<ProjectSummary, String> {
    project_store::set_project_variables(
        PathBuf::from(root_path).as_path(),
        variables,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_execution_credential_status() -> Result<ExecutionCredentialStatus, String> {
    project_store::get_execution_credential_status().map_err(|error| error.to_string())
}

#[tauri::command]
fn save_execution_api_key(api_key: String) -> Result<ExecutionCredentialStatus, String> {
    project_store::save_execution_api_key(&api_key).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_execution_api_key() -> Result<ExecutionCredentialStatus, String> {
    project_store::clear_execution_api_key().map_err(|error| error.to_string())
}

#[tauri::command]
fn list_prompt_run_history(
    root_path: String,
    relative_path: String,
) -> Result<Vec<PromptRunHistoryEntry>, String> {
    project_store::list_prompt_run_history(PathBuf::from(root_path).as_path(), &relative_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_project_run_history(root_path: String) -> Result<Vec<ProjectRunHistoryEntry>, String> {
    project_store::list_project_run_history(PathBuf::from(root_path).as_path())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn get_project_usage_summary(root_path: String) -> Result<ProjectUsageSummary, String> {
    project_store::get_project_usage_summary(PathBuf::from(root_path).as_path())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_model_presets(root_path: String) -> Result<Vec<ModelPresetSummary>, String> {
    project_store::list_model_presets(PathBuf::from(root_path).as_path())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_default_model_preset(
    root_path: String,
    preset_path: String,
) -> Result<ProjectSummary, String> {
    project_store::set_default_model_preset(
        PathBuf::from(root_path).as_path(),
        &preset_path,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_model_preset(
    root_path: String,
    filename: String,
    model_id: String,
) -> Result<ModelPresetSummary, String> {
    project_store::create_model_preset(
        PathBuf::from(root_path).as_path(),
        &filename,
        &model_id,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_model_preset(root_path: String, preset_path: String) -> Result<(), String> {
    project_store::delete_model_preset(PathBuf::from(root_path).as_path(), &preset_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_block_model_preset(
    root_path: String,
    block_id: String,
    preset_path: Option<String>,
) -> Result<ProjectSummary, String> {
    project_store::set_block_model_preset(
        PathBuf::from(root_path).as_path(),
        &block_id,
        preset_path.as_deref(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_block_output_target(
    root_path: String,
    block_id: String,
    target: String,
) -> Result<ProjectSummary, String> {
    project_store::set_block_output_target(
        PathBuf::from(root_path).as_path(),
        &block_id,
        &target,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_block_output_filename(
    root_path: String,
    block_id: String,
    filename: Option<String>,
) -> Result<ProjectSummary, String> {
    project_store::set_block_output_filename(
        PathBuf::from(root_path).as_path(),
        &block_id,
        filename.as_deref(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn rename_project(
    app: tauri::AppHandle,
    root_path: String,
    new_name: String,
) -> Result<ProjectSummary, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::rename_project(
        PathBuf::from(root_path).as_path(),
        &new_name,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_pipeline(
    app: tauri::AppHandle,
    root_path: String,
    pipeline_id: String,
) -> Result<ProjectSummary, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::delete_pipeline(
        PathBuf::from(root_path).as_path(),
        &pipeline_id,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_prompt_block(
    app: tauri::AppHandle,
    root_path: String,
    block_id: String,
) -> Result<ProjectSummary, String> {
    let app_data_dir = app_data_dir(&app)?;
    project_store::delete_prompt_block(
        PathBuf::from(root_path).as_path(),
        &block_id,
        &app_data_dir,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_run(root_path: String, run_path: String) -> Result<(), String> {
    project_store::delete_run(PathBuf::from(root_path).as_path(), &run_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_document(root_path: String, relative_path: String) -> Result<(), String> {
    project_store::delete_document(PathBuf::from(root_path).as_path(), &relative_path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn rename_document(
    root_path: String,
    old_path: String,
    new_name: String,
) -> Result<String, String> {
    project_store::rename_document(PathBuf::from(root_path).as_path(), &old_path, &new_name)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_in_os(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer").arg(&path).spawn().map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            create_project,
            open_project,
            create_prompt_block,
            get_recent_projects,
            remove_recent_project,
            locate_recent_project,
            list_project_assets,
            list_project_pipelines,
            list_project_prompt_blocks,
            create_pipeline,
            update_pipeline,
            export_project_assets,
            read_project_asset,
            write_project_asset,
            validate_project_template,
            execute_prompt_block,
            execute_pipeline,
            get_execution_credential_status,
            save_execution_api_key,
            clear_execution_api_key,
            list_prompt_run_history,
            list_project_run_history,
            get_project_usage_summary,
            get_global_variables,
            set_global_variables,
            set_project_variables,
            list_model_presets,
            set_default_model_preset,
            create_model_preset,
            delete_model_preset,
            set_block_model_preset,
            set_block_output_target,
            set_block_output_filename,
            rename_project,
            delete_pipeline,
            delete_prompt_block,
            delete_run,
            delete_document,
            rename_document,
            open_in_os
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 4 && args[1] == "cli" && args[2] == "run-pipeline" {
        let project_path = std::path::PathBuf::from(&args[3]);
        let pipeline_id = if args.len() > 4 { &args[4] } else { "" };
        
        if pipeline_id.is_empty() {
             eprintln!("Pipeline ID missing");
             std::process::exit(1);
        }

        let payload = if args.len() > 5 {
            let p: BTreeMap<String, String> = serde_json::from_str(&args[5]).expect("Invalid payload JSON");
            Some(p)
        } else {
            None
        };

        let app_data = std::env::temp_dir().join("diamond-runner-headless");
        std::fs::create_dir_all(&app_data).unwrap();

        match project_store::execution::execute_pipeline(&project_path, pipeline_id, payload, &app_data) {
            Ok(result) => {
                if result.status == project_store::ExecutionStatus::Failed {
                    eprintln!("CLI Execute Pipeline FAILED internally:");
                    if let Some(err) = result.error {
                        eprintln!("Error: {}", err);
                    }
                    std::process::exit(1);
                } else {
                    println!("CLI Execute Pipeline SUCCESS!");
                    println!("Blocks Completed: {}", result.steps.len());
                    std::process::exit(0);
                }
            }
            Err(e) => {
                eprintln!("CLI Execute Pipeline FATAL: {}", e);
                std::process::exit(1);
            }
        }
    }

    run();
}
