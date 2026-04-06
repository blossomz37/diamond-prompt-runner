import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AssetContent,
  CreatedPromptBlockResult,
  ExecutionCredentialStatus,
  ExportBundleResult,
  ModelPresetSummary,
  PipelineExecutionResult,
  PipelineProgressEvent,
  ProjectAssetNode,
  ProjectPipeline,
  ProjectPromptBlock,
  ProjectRunHistoryEntry,
  ProjectSummary,
  ProjectUsageSummary,
  PromptExecutionResult,
  PromptRunHistoryEntry,
  RecentProjectEntry,
  SavedPipelineResult,
  TemplateValidationResult
} from './types/project';

export async function pickDirectory(title: string): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    title
  });

  if (Array.isArray(result)) {
    return result[0] ?? null;
  }

  return result;
}

export async function getRecentProjects(): Promise<RecentProjectEntry[]> {
  return invoke<RecentProjectEntry[]>('get_recent_projects');
}

export async function removeRecentProject(rootPath: string): Promise<void> {
  return invoke('remove_recent_project', { rootPath });
}

export async function locateRecentProject(
  previousRootPath: string,
  candidateRootPath: string
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('locate_recent_project', {
    previousRootPath,
    candidateRootPath
  });
}

export async function createProject(parentPath: string, projectName: string): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('create_project', { parentPath, projectName });
}

export async function createPromptBlock(
  rootPath: string,
  promptName: string
): Promise<CreatedPromptBlockResult> {
  return invoke<CreatedPromptBlockResult>('create_prompt_block', { rootPath, promptName });
}

export async function openProject(rootPath: string): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('open_project', { rootPath });
}

export async function listProjectAssets(rootPath: string): Promise<ProjectAssetNode[]> {
  return invoke<ProjectAssetNode[]>('list_project_assets', { rootPath });
}

export async function listProjectPipelines(rootPath: string): Promise<ProjectPipeline[]> {
  return invoke<ProjectPipeline[]>('list_project_pipelines', { rootPath });
}

export async function listProjectPromptBlocks(rootPath: string): Promise<ProjectPromptBlock[]> {
  return invoke<ProjectPromptBlock[]>('list_project_prompt_blocks', { rootPath });
}

export async function createPipeline(
  rootPath: string,
  pipelineName: string,
  orderedBlockIds: string[]
): Promise<SavedPipelineResult> {
  return invoke<SavedPipelineResult>('create_pipeline', {
    rootPath,
    pipelineName,
    orderedBlockIds
  });
}

export async function updatePipeline(
  rootPath: string,
  pipelineId: string,
  pipelineName: string,
  orderedBlockIds: string[]
): Promise<SavedPipelineResult> {
  return invoke<SavedPipelineResult>('update_pipeline', {
    rootPath,
    pipelineId,
    pipelineName,
    orderedBlockIds
  });
}

export async function duplicatePipeline(
  rootPath: string,
  pipelineId: string
): Promise<SavedPipelineResult> {
  return invoke<SavedPipelineResult>('duplicate_pipeline', {
    rootPath,
    pipelineId
  });
}

export async function exportProjectAssets(
  rootPath: string,
  bundleName: string,
  relativePaths: string[]
): Promise<ExportBundleResult> {
  return invoke<ExportBundleResult>('export_project_assets', {
    rootPath,
    bundleName,
    relativePaths
  });
}

export async function readProjectAsset(rootPath: string, relativePath: string): Promise<AssetContent> {
  return invoke<AssetContent>('read_project_asset', { rootPath, relativePath });
}

export async function writeProjectAsset(
  rootPath: string,
  relativePath: string,
  content: string
): Promise<AssetContent> {
  return invoke<AssetContent>('write_project_asset', { rootPath, relativePath, content });
}

export async function validateProjectTemplate(
  rootPath: string,
  relativePath: string,
  content: string
): Promise<TemplateValidationResult> {
  return invoke<TemplateValidationResult>('validate_project_template', {
    rootPath,
    relativePath,
    content
  });
}

export async function executePromptBlock(
  rootPath: string,
  relativePath: string,
  content: string
): Promise<PromptExecutionResult> {
  return invoke<PromptExecutionResult>('execute_prompt_block', {
    rootPath,
    relativePath,
    content
  });
}

export async function executePipeline(
  root_path: string,
  pipeline_id: string,
  payload?: Record<string, string>,
  resumeFromBlockId?: string,
  selectedBlockIds?: string[]
): Promise<PipelineExecutionResult> {
  return invoke<PipelineExecutionResult>('execute_pipeline', {
    rootPath: root_path,
    pipelineId: pipeline_id,
    payload,
    resumeFromBlockId,
    selectedBlockIds
  });
}

export async function cancelPipeline(): Promise<void> {
  return invoke('cancel_pipeline');
}

export async function onPipelineProgress(
  callback: (event: PipelineProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<PipelineProgressEvent>('pipeline-progress', (event) => {
    callback(event.payload);
  });
}

export async function getExecutionCredentialStatus(): Promise<ExecutionCredentialStatus> {
  return invoke<ExecutionCredentialStatus>('get_execution_credential_status');
}

export async function saveExecutionApiKey(apiKey: string): Promise<ExecutionCredentialStatus> {
  return invoke<ExecutionCredentialStatus>('save_execution_api_key', { apiKey });
}

export async function clearExecutionApiKey(): Promise<ExecutionCredentialStatus> {
  return invoke<ExecutionCredentialStatus>('clear_execution_api_key');
}

export async function listPromptRunHistory(
  rootPath: string,
  relativePath: string
): Promise<PromptRunHistoryEntry[]> {
  return invoke<PromptRunHistoryEntry[]>('list_prompt_run_history', {
    rootPath,
    relativePath
  });
}

export async function listProjectRunHistory(rootPath: string): Promise<ProjectRunHistoryEntry[]> {
  return invoke<ProjectRunHistoryEntry[]>('list_project_run_history', {
    rootPath
  });
}

export async function getProjectUsageSummary(rootPath: string): Promise<ProjectUsageSummary> {
  return invoke<ProjectUsageSummary>('get_project_usage_summary', { rootPath });
}

export async function getGlobalVariables(): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('get_global_variables');
}

export async function setGlobalVariables(
  variables: Record<string, string>
): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('set_global_variables', { variables });
}

export async function setProjectVariables(
  rootPath: string,
  variables: Record<string, string>
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('set_project_variables', { rootPath, variables });
}

export async function listModelPresets(rootPath: string): Promise<ModelPresetSummary[]> {
  return invoke<ModelPresetSummary[]>('list_model_presets', { rootPath });
}

export async function setDefaultModelPreset(
  rootPath: string,
  presetPath: string
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('set_default_model_preset', { rootPath, presetPath });
}

export async function createModelPreset(
  rootPath: string,
  filename: string,
  modelId: string
): Promise<ModelPresetSummary> {
  return invoke<ModelPresetSummary>('create_model_preset', { rootPath, filename, modelId });
}

export async function deleteModelPreset(
  rootPath: string,
  presetPath: string
): Promise<void> {
  return invoke('delete_model_preset', { rootPath, presetPath });
}

export async function setBlockModelPreset(
  rootPath: string,
  blockId: string,
  presetPath: string | null
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('set_block_model_preset', { rootPath, blockId, presetPath });
}

export async function setBlockOutputTarget(
  rootPath: string,
  blockId: string,
  target: string
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('set_block_output_target', { rootPath, blockId, target });
}

export async function setBlockOutputFilename(
  rootPath: string,
  blockId: string,
  filename: string | null
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('set_block_output_filename', { rootPath, blockId, filename });
}

export async function renameProject(
  rootPath: string,
  newName: string
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('rename_project', { rootPath, newName });
}

export async function deletePipeline(
  rootPath: string,
  pipelineId: string
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('delete_pipeline', { rootPath, pipelineId });
}

export async function deletePromptBlock(
  rootPath: string,
  blockId: string
): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('delete_prompt_block', { rootPath, blockId });
}

export async function deleteRun(rootPath: string, runPath: string): Promise<void> {
  return invoke('delete_run', { rootPath, runPath });
}

export async function deleteDocument(rootPath: string, relativePath: string): Promise<void> {
  return invoke('delete_document', { rootPath, relativePath });
}

export async function renameDocument(
  rootPath: string,
  oldPath: string,
  newName: string
): Promise<string> {
  return invoke<string>('rename_document', { rootPath, oldPath, newName });
}
