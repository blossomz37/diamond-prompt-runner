export type AssetKind =
  | 'manifest'
  | 'directory'
  | 'markdown'
  | 'text'
  | 'tera'
  | 'yaml'
  | 'json'
  | 'unknown';

export interface ProjectCounts {
  documents: number;
  prompts: number;
  models: number;
  runs: number;
  exports: number;
}

export interface ProjectSummary {
  rootPath: string;
  projectId: string;
  projectName: string;
  defaultModelPreset: string;
  updatedAt: string;
  counts: ProjectCounts;
}

export interface RecentProjectEntry extends ProjectSummary {
  lastOpenedAt: string;
  lastKnownValid: boolean;
}

export interface ProjectAssetNode {
  name: string;
  path: string;
  kind: AssetKind;
  isDirectory: boolean;
  children: ProjectAssetNode[];
}

export interface MetadataField {
  label: string;
  value: string;
}

export interface AssetMetadata {
  kind: AssetKind;
  path: string;
  name: string;
  sizeBytes: number | null;
  modifiedAt: string | null;
  details: MetadataField[];
}

export interface AssetContent {
  path: string;
  kind: AssetKind;
  view: 'text' | 'json' | 'placeholder';
  content: string;
  isEditable: boolean;
  metadata: AssetMetadata;
  parsedJson: unknown | null;
}

export interface WorkspaceTab extends AssetContent {
  title: string;
  savedContent: string;
  draftContent: string;
  isSaving: boolean;
}

export interface TemplateValidationResult {
  path: string;
  status: 'valid' | 'warnings' | 'invalid';
  preview: string | null;
  warnings: string[];
  errors: string[];
  contextSummary: MetadataField[];
}

export interface PromptExecutionResult {
  runId: string;
  path: string;
  blockId: string | null;
  blockName: string;
  modelPreset: string;
  modelId: string;
  status: 'success' | 'failed';
  renderedPrompt: string;
  output: string | null;
  error: string | null;
  runPath: string;
  startedAt: string;
  completedAt: string;
}

export interface PromptRunHistoryEntry {
  runId: string;
  path: string;
  blockName: string;
  modelId: string;
  status: 'success' | 'failed';
  runPath: string;
  startedAt: string;
  completedAt: string;
  outputPreview: string | null;
  error: string | null;
}

export type ExecutionCredentialSource = 'keychain' | 'environment' | 'missing';

export interface ExecutionCredentialStatus {
  source: ExecutionCredentialSource;
  hasStoredKey: boolean;
}
