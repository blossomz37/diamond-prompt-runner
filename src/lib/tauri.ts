import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AssetContent,
  ProjectAssetNode,
  ProjectSummary,
  PromptExecutionResult,
  RecentProjectEntry,
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

export async function createProject(parentPath: string, projectName: string): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('create_project', { parentPath, projectName });
}

export async function openProject(rootPath: string): Promise<ProjectSummary> {
  return invoke<ProjectSummary>('open_project', { rootPath });
}

export async function listProjectAssets(rootPath: string): Promise<ProjectAssetNode[]> {
  return invoke<ProjectAssetNode[]>('list_project_assets', { rootPath });
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
