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
  help: number;
}

export interface ProjectSummary {
  rootPath: string;
  projectId: string;
  projectName: string;
  defaultModelPreset: string;
  updatedAt: string;
  counts: ProjectCounts;
  variables: Record<string, string>;
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

export interface OnlineRunMetadata {
  enabled: boolean;
  webSearchRequests: number;
  citationCount: number;
}

export interface ProjectUsageSummary {
  totalRuns: number;
  successfulRuns: number;
  failedRuns: number;
  totalPromptTokens: number;
  totalCompletionTokens: number;
  totalTokens: number;
  totalCost: number;
  totalOutputWords: number;
  totalRetries: number;
}

export interface UsageMetrics {
  promptTokens: number | null;
  completionTokens: number | null;
  totalTokens: number | null;
  cost: number | null;
  outputWordCount: number | null;
  retryCount: number | null;
}

export interface PromptExecutionResult {
  runId: string;
  path: string;
  blockId: string | null;
  blockName: string;
  pipelineId: string | null;
  pipelineName: string | null;
  modelPreset: string;
  modelId: string;
  status: 'success' | 'failed';
  outputTarget: string;
  documentPath: string | null;
  variables: Record<string, string>;
  output: string | null;
  error: string | null;
  runPath: string;
  startedAt: string;
  completedAt: string;
  online: OnlineRunMetadata;
  usage: UsageMetrics;
}

export interface ProjectRunHistoryEntry {
  runId: string;
  path: string;
  blockId: string | null;
  blockName: string;
  pipelineId: string | null;
  pipelineName: string | null;
  modelId: string;
  status: 'success' | 'failed';
  runPath: string;
  startedAt: string;
  completedAt: string;
  outputPreview: string | null;
  error: string | null;
  online: OnlineRunMetadata;
  usage: UsageMetrics;
}

export type PromptRunHistoryEntry = ProjectRunHistoryEntry;

export interface ProjectPipelineBlock {
  blockId: string;
  name: string;
  templateSource: string;
  modelPreset: string;
  outputTarget: string;
}

export type ProjectPromptBlock = ProjectPipelineBlock;

export interface ProjectPipeline {
  pipelineId: string;
  name: string;
  executionMode: string;
  blocks: ProjectPipelineBlock[];
}

export interface SavedPipelineResult {
  summary: ProjectSummary;
  pipelineId: string;
}

export interface ExportBundleResult {
  summary: ProjectSummary;
  bundleName: string;
  bundlePath: string;
  exportedPaths: string[];
}

export interface PipelineExecutionResult {
  pipelineId: string;
  pipelineName: string;
  status: 'success' | 'failed';
  startedAt: string;
  completedAt: string;
  error: string | null;
  steps: PromptExecutionResult[];
}

export interface CreatedPromptBlockResult {
  summary: ProjectSummary;
  path: string;
}

export type ExecutionCredentialSource = 'keychain' | 'environment' | 'missing';

export interface ExecutionCredentialStatus {
  source: ExecutionCredentialSource;
  hasStoredKey: boolean;
}

export interface ModelPresetSummary {
  path: string;
  filename: string;
  modelId: string;
}
