import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ONLINE_PROMPT_DIRECTIVE } from '$lib/promptExecution';
import App from './App.svelte';
import type {
  AssetContent,
  CreatedPromptBlockResult,
  ExecutionCredentialStatus,
  ExportBundleResult,
  PipelineExecutionResult,
  ProjectAssetNode,
  ProjectPipeline,
  ProjectPromptBlock,
  ProjectRunHistoryEntry,
  PromptExecutionResult,
  PromptRunHistoryEntry,
  ProjectSummary,
  RecentProjectEntry,
  SavedPipelineResult,
  TemplateValidationResult
} from '$lib/types/project';

const tauri = vi.hoisted(() => ({
  clearExecutionApiKey: vi.fn(),
  createPipeline: vi.fn(),
  createProject: vi.fn(),
  createPromptBlock: vi.fn(),
  executePipeline: vi.fn(),
  exportProjectAssets: vi.fn(),
  getRecentProjects: vi.fn(),
  getExecutionCredentialStatus: vi.fn(),
  listProjectPipelines: vi.fn(),
  listProjectPromptBlocks: vi.fn(),
  listProjectRunHistory: vi.fn(),
  listPromptRunHistory: vi.fn(),
  listProjectAssets: vi.fn(),
  locateRecentProject: vi.fn(),
  openProject: vi.fn(),
  pickDirectory: vi.fn(),
  removeRecentProject: vi.fn(),
  readProjectAsset: vi.fn(),
  executePromptBlock: vi.fn(),
  saveExecutionApiKey: vi.fn(),
  updatePipeline: vi.fn(),
  validateProjectTemplate: vi.fn(),
  writeProjectAsset: vi.fn()
}));

vi.mock('$lib/tauri', () => tauri);

const offlineRunMetadata = {
  enabled: false,
  webSearchRequests: 0,
  citationCount: 0
};

const emptyUsageMetrics = {
  promptTokens: null,
  completionTokens: null,
  totalTokens: null,
  cost: null,
  outputWordCount: null
};

const summary: ProjectSummary = {
  rootPath: '/tmp/story-lab',
  projectId: 'project-1',
  projectName: 'Story Lab',
  defaultModelPreset: 'models/default.yaml',
  updatedAt: '2026-04-03T20:10:00Z',
  counts: {
    documents: 1,
    prompts: 1,
    models: 1,
    runs: 0,
    exports: 0
  }
};

const recents: RecentProjectEntry[] = [
  {
    ...summary,
    lastOpenedAt: '2026-04-03T20:11:00Z',
    lastKnownValid: true
  }
];

const assetNodes: ProjectAssetNode[] = [
  {
    name: 'project.json',
    path: 'project.json',
    kind: 'manifest',
    isDirectory: false,
    children: []
  },
  {
    name: 'documents',
    path: 'documents',
    kind: 'directory',
    isDirectory: true,
    children: [
      {
        name: 'context.md',
        path: 'documents/context.md',
        kind: 'markdown',
        isDirectory: false,
        children: []
      }
    ]
  },
  {
    name: 'prompts',
    path: 'prompts',
    kind: 'directory',
    isDirectory: true,
    children: [
      {
        name: 'brief-review.tera',
        path: 'prompts/brief-review.tera',
        kind: 'tera',
        isDirectory: false,
        children: []
      }
    ]
  },
  {
    name: 'models',
    path: 'models',
    kind: 'directory',
    isDirectory: true,
    children: [
      {
        name: 'default.yaml',
        path: 'models/default.yaml',
        kind: 'yaml',
        isDirectory: false,
        children: []
      }
    ]
  }
];

const assetContent: AssetContent = {
  path: 'documents/context.md',
  kind: 'markdown',
  view: 'text',
  content: '# Context\n\nA small fixture document.',
  isEditable: true,
  metadata: {
    kind: 'markdown',
    path: 'documents/context.md',
    name: 'context.md',
    sizeBytes: 32,
    modifiedAt: '2026-04-03T20:12:00Z',
    details: [
      { label: 'Lines', value: '3' },
      { label: 'Words', value: '5' }
    ]
  },
  parsedJson: null
};

const yamlAssetContent: AssetContent = {
  path: 'models/default.yaml',
  kind: 'yaml',
  view: 'text',
  content: 'model: openai/gpt-5.4\ntemperature: 0.7\nmax_completion_tokens: 12000\n',
  isEditable: true,
  metadata: {
    kind: 'yaml',
    path: 'models/default.yaml',
    name: 'default.yaml',
    sizeBytes: 72,
    modifiedAt: '2026-04-03T20:12:00Z',
    details: [
      { label: 'Model', value: 'openai/gpt-5.4' },
      { label: 'Temperature', value: '0.7' },
      { label: 'Max Tokens', value: '12000' }
    ]
  },
  parsedJson: null
};

const teraAssetContent: AssetContent = {
  path: 'prompts/brief-review.tera',
  kind: 'tera',
  view: 'text',
  content:
    'You are reviewing the current project context.\n\nContext:\n{{ doc("context.md") }}\n\nRespond with a short summary and one next action.\n',
  isEditable: true,
  metadata: {
    kind: 'tera',
    path: 'prompts/brief-review.tera',
    name: 'brief-review.tera',
    sizeBytes: 126,
    modifiedAt: '2026-04-03T20:12:00Z',
    details: [
      { label: 'Lines', value: '6' },
      { label: 'Linked Blocks', value: 'Brief Review' }
    ]
  },
  parsedJson: null
};

const validationResult: TemplateValidationResult = {
  path: 'prompts/brief-review.tera',
  status: 'valid',
  preview:
    'You are reviewing the current project context.\n\nContext:\n# Product Context\n\nDiamond Prompt Runner stores prompts, documents, and model presets as local project files.\n\nRespond with a short summary and one next action.',
  warnings: [],
  errors: [],
  contextSummary: [
    { label: 'Project', value: 'Story Lab' },
    { label: 'Model ID', value: 'openai/gpt-5.4' }
  ]
};

const executionResult: PromptExecutionResult = {
  runId: 'run-1',
  path: 'prompts/brief-review.tera',
  blockId: 'brief-review',
  blockName: 'Brief Review',
  pipelineId: null,
  pipelineName: null,
  modelPreset: 'models/default.yaml',
  modelId: 'openai/gpt-5.4-nano',
  status: 'success',
  renderedPrompt: 'Rendered prompt body',
  output: 'Execution output from the provider.',
  error: null,
  runPath: 'runs/run-1.json',
  startedAt: '2026-04-03T20:13:00Z',
  completedAt: '2026-04-03T20:13:05Z',
  online: offlineRunMetadata,
  usage: emptyUsageMetrics
};

const missingCredentialStatus: ExecutionCredentialStatus = {
  source: 'missing',
  hasStoredKey: false
};

const keychainCredentialStatus: ExecutionCredentialStatus = {
  source: 'keychain',
  hasStoredKey: true
};

const runHistory: PromptRunHistoryEntry[] = [
  {
    runId: 'run-2',
    path: 'prompts/brief-review.tera',
    blockId: 'brief-review',
    blockName: 'Brief Review',
    pipelineId: null,
    pipelineName: null,
    modelId: 'openai/gpt-5.4-nano',
    status: 'success',
    runPath: 'runs/run-2.json',
    startedAt: '2026-04-03T20:20:00Z',
    completedAt: '2026-04-03T20:20:04Z',
    outputPreview: 'Earlier persisted output.',
    error: null,
    online: offlineRunMetadata,
    usage: emptyUsageMetrics
  }
];

const projectRunHistory: ProjectRunHistoryEntry[] = [
  {
    runId: 'run-pipeline',
    path: 'prompts/brief-review.tera',
    blockId: 'brief-review',
    blockName: 'Brief Review',
    pipelineId: 'review-pipeline',
    pipelineName: 'Review Pipeline',
    modelId: 'openai/gpt-5.4-nano',
    status: 'success',
    runPath: 'runs/run-pipeline.json',
    startedAt: '2026-04-03T20:21:00Z',
    completedAt: '2026-04-03T20:21:08Z',
    outputPreview: 'Pipeline output preview.',
    error: null,
    online: offlineRunMetadata,
    usage: emptyUsageMetrics
  },
  {
    runId: 'run-standalone',
    path: 'prompts/other.tera',
    blockId: 'other-block',
    blockName: 'Other Prompt',
    pipelineId: null,
    pipelineName: null,
    modelId: 'openai/gpt-5.4',
    status: 'failed',
    runPath: 'runs/run-standalone.json',
    startedAt: '2026-04-03T20:10:00Z',
    completedAt: '2026-04-03T20:10:02Z',
    outputPreview: null,
    error: 'Provider timeout',
    online: offlineRunMetadata,
    usage: emptyUsageMetrics
  }
];

const pipelines: ProjectPipeline[] = [
  {
    pipelineId: 'review-pipeline',
    name: 'Review Pipeline',
    executionMode: 'sequential',
    blocks: [
      {
        blockId: 'brief-review',
        name: 'Brief Review',
        templateSource: 'prompts/brief-review.tera',
        modelPreset: 'models/default.yaml'
      }
    ]
  }
];

const promptBlocks: ProjectPromptBlock[] = [
  {
    blockId: 'brief-review',
    name: 'Brief Review',
    templateSource: 'prompts/brief-review.tera',
    modelPreset: 'models/default.yaml'
  },
  {
    blockId: 'scene-draft',
    name: 'Scene Draft',
    templateSource: 'prompts/scene-draft.tera',
    modelPreset: 'models/default.yaml'
  }
];

const pipelineExecutionResult: PipelineExecutionResult = {
  pipelineId: 'review-pipeline',
  pipelineName: 'Review Pipeline',
  status: 'success',
  startedAt: '2026-04-03T20:21:00Z',
  completedAt: '2026-04-03T20:21:08Z',
  error: null,
  steps: [executionResult]
};

const createdPromptResult: CreatedPromptBlockResult = {
  summary: {
    ...summary,
    updatedAt: '2026-04-03T20:22:00Z',
    counts: {
      ...summary.counts,
      prompts: 2
    }
  },
  path: 'prompts/scene-draft.tera'
};

const createdPipelineResult: SavedPipelineResult = {
  summary: {
    ...summary,
    updatedAt: '2026-04-03T20:23:00Z'
  },
  pipelineId: 'draft-pipeline'
};

const updatedPipelineResult: SavedPipelineResult = {
  summary: {
    ...summary,
    updatedAt: '2026-04-03T20:24:00Z'
  },
  pipelineId: 'review-pipeline'
};

const exportBundleResult: ExportBundleResult = {
  summary: {
    ...summary,
    updatedAt: '2026-04-04T00:26:00Z',
    counts: {
      ...summary.counts,
      exports: 1
    }
  },
  bundleName: 'Session Export',
  bundlePath: 'exports/session-export',
  exportedPaths: ['documents/context.md', 'prompts/brief-review.tera']
};

const createdPromptAssetContent: AssetContent = {
  path: 'prompts/scene-draft.tera',
  kind: 'tera',
  view: 'text',
  content:
    'Project: {{ project.name }}\nDate: {{ current_date }}\n\nWrite the instructions for this prompt block here.\n',
  isEditable: true,
  metadata: {
    kind: 'tera',
    path: 'prompts/scene-draft.tera',
    name: 'scene-draft.tera',
    sizeBytes: 101,
    modifiedAt: '2026-04-03T20:22:00Z',
    details: [
      { label: 'Lines', value: '4' },
      { label: 'Linked Blocks', value: 'Scene Draft' }
    ]
  },
  parsedJson: null
};

const runAssetContent: AssetContent = {
  path: 'runs/run-2.json',
  kind: 'json',
  view: 'json',
  content: JSON.stringify({ runId: 'run-2', output: 'Earlier persisted output.' }, null, 2),
  isEditable: false,
  metadata: {
    kind: 'json',
    path: 'runs/run-2.json',
    name: 'run-2.json',
    sizeBytes: 64,
    modifiedAt: '2026-04-03T20:20:05Z',
    details: [{ label: 'Status', value: 'success' }]
  },
  parsedJson: {
    runId: 'run-2',
    output: 'Earlier persisted output.'
  }
};

async function openExplorerAsset(folderName: string, assetName: string): Promise<void> {
  const explorer = screen.getByTestId('explorer-tree');
  const folderButton = within(explorer)
    .getAllByRole('button')
    .find((button) => button.textContent?.trim().endsWith(folderName));
  expect(folderButton).toBeTruthy();
  await fireEvent.click(folderButton!);

  await waitFor(() => {
    const assetButton = within(screen.getByTestId('explorer-tree'))
      .queryAllByRole('button')
      .find((button) => button.textContent?.trim().endsWith(assetName));
    expect(assetButton).toBeTruthy();
  });

  const assetButton = within(screen.getByTestId('explorer-tree'))
    .getAllByRole('button')
    .find((button) => button.textContent?.trim().endsWith(assetName));
  expect(assetButton).toBeTruthy();
  await fireEvent.click(assetButton!);
}

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    tauri.getRecentProjects.mockResolvedValue(recents);
    tauri.exportProjectAssets.mockResolvedValue(exportBundleResult);
    tauri.getExecutionCredentialStatus.mockResolvedValue(missingCredentialStatus);
    tauri.listProjectPipelines.mockResolvedValue(pipelines);
    tauri.listProjectPromptBlocks.mockResolvedValue(promptBlocks);
    tauri.listProjectRunHistory.mockResolvedValue(projectRunHistory);
    tauri.listPromptRunHistory.mockResolvedValue(runHistory);
    tauri.pickDirectory.mockResolvedValue('/tmp');
    tauri.createPipeline.mockResolvedValue(createdPipelineResult);
    tauri.createProject.mockResolvedValue(summary);
    tauri.createPromptBlock.mockResolvedValue(createdPromptResult);
    tauri.locateRecentProject.mockResolvedValue(summary);
    tauri.openProject.mockResolvedValue(summary);
    tauri.removeRecentProject.mockResolvedValue(undefined);
    tauri.executePipeline.mockResolvedValue(pipelineExecutionResult);
    tauri.listProjectAssets.mockResolvedValue(assetNodes);
    tauri.readProjectAsset.mockImplementation(async (_rootPath: string, relativePath: string) => {
      if (relativePath === 'models/default.yaml') {
        return yamlAssetContent;
      }

      if (relativePath === 'prompts/brief-review.tera') {
        return teraAssetContent;
      }

      if (relativePath === 'prompts/scene-draft.tera') {
        return createdPromptAssetContent;
      }

      if (relativePath === 'runs/run-2.json') {
        return runAssetContent;
      }

      return assetContent;
    });
    tauri.validateProjectTemplate.mockResolvedValue(validationResult);
    tauri.executePromptBlock.mockResolvedValue(executionResult);
    tauri.saveExecutionApiKey.mockResolvedValue(keychainCredentialStatus);
    tauri.clearExecutionApiKey.mockResolvedValue(missingCredentialStatus);
    tauri.updatePipeline.mockResolvedValue(updatedPipelineResult);
    tauri.writeProjectAsset.mockImplementation(async (_rootPath: string, relativePath: string, content: string) => {
      if (relativePath === 'models/default.yaml') {
        return {
          ...yamlAssetContent,
          content
        };
      }

      if (relativePath === 'prompts/brief-review.tera') {
        return {
          ...teraAssetContent,
          content
        };
      }

      return {
        ...assetContent,
        content,
        metadata: {
          ...assetContent.metadata,
          sizeBytes: content.length
        }
      };
    });
  });

  it('renders recents on load', async () => {
    render(App);

    expect(await screen.findByText('Reopen quickly')).toBeInTheDocument();
    expect(screen.getByText('Story Lab')).toBeInTheDocument();
  });

  it('opens an existing project and reuses a tab for the same asset', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('documents', 'context.md');
    await waitFor(() => expect(screen.getAllByText('context.md')).not.toHaveLength(0));

    const contextButton = within(screen.getByTestId('explorer-tree'))
      .getAllByRole('button')
      .find((button) => button.textContent?.trim().endsWith('context.md'));
    expect(contextButton).toBeTruthy();
    await fireEvent.click(contextButton!);

    expect(tauri.readProjectAsset).toHaveBeenCalledTimes(1);
    expect(screen.getByText('Lines')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
  });

  it('creates a new prompt from the explorer and opens it in a tab', async () => {
    tauri.listProjectAssets
      .mockResolvedValueOnce(assetNodes)
      .mockResolvedValueOnce([
        ...assetNodes.slice(0, 2),
        {
          name: 'prompts',
          path: 'prompts',
          kind: 'directory',
          isDirectory: true,
          children: [
            assetNodes[2].children[0],
            {
              name: 'scene-draft.tera',
              path: 'prompts/scene-draft.tera',
              kind: 'tera',
              isDirectory: false,
              children: []
            }
          ]
        },
        assetNodes[3]
      ]);

    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await fireEvent.click(screen.getByRole('button', { name: 'New Prompt' }));
    await fireEvent.input(screen.getByRole('textbox', { name: 'Prompt name' }), {
      target: { value: 'Scene Draft' }
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() =>
      expect(tauri.createPromptBlock).toHaveBeenCalledWith('/tmp/story-lab', 'Scene Draft')
    );

    expect(await screen.findAllByText('scene-draft.tera')).not.toHaveLength(0);
    expect(screen.getByTestId('asset-editor')).toHaveValue(createdPromptAssetContent.content);
  });

  it('removes an unavailable recent project', async () => {
    tauri.getRecentProjects
      .mockResolvedValueOnce([
        {
          ...summary,
          rootPath: '/tmp/missing-project',
          lastOpenedAt: '2026-04-03T20:11:00Z',
          lastKnownValid: false
        }
      ])
      .mockResolvedValueOnce([]);

    render(App);

    expect(await screen.findByText('Unavailable')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Remove' }));

    await waitFor(() => expect(screen.getByText('No recent projects yet.')).toBeInTheDocument());
    expect(tauri.removeRecentProject).toHaveBeenCalledWith('/tmp/missing-project');
  });

  it('locates an unavailable recent project and opens the rebound workspace', async () => {
    tauri.getRecentProjects
      .mockResolvedValueOnce([
        {
          ...summary,
          rootPath: '/tmp/missing-project',
          lastOpenedAt: '2026-04-03T20:11:00Z',
          lastKnownValid: false
        }
      ])
      .mockResolvedValueOnce([
        {
          ...summary,
          rootPath: '/tmp/story-lab-relocated',
          lastOpenedAt: '2026-04-03T20:12:00Z',
          lastKnownValid: true
        }
      ]);

    tauri.locateRecentProject.mockResolvedValueOnce({
      ...summary,
      rootPath: '/tmp/story-lab-relocated'
    });
    tauri.pickDirectory.mockResolvedValueOnce('/tmp/story-lab-relocated');

    render(App);

    expect(await screen.findByText('Unavailable')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Locate' }));

    await waitFor(() =>
      expect(tauri.locateRecentProject).toHaveBeenCalledWith(
        '/tmp/missing-project',
        '/tmp/story-lab-relocated'
      )
    );

    expect(await screen.findByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument();
  });

  it('shows a locate mismatch error when the selected folder is the wrong project', async () => {
    tauri.getRecentProjects.mockResolvedValueOnce([
      {
        ...summary,
        rootPath: '/tmp/missing-project',
        lastOpenedAt: '2026-04-03T20:11:00Z',
        lastKnownValid: false
      }
    ]);
    tauri.locateRecentProject.mockRejectedValueOnce(
      new Error('The selected folder is a valid Diamond project, but it does not match the missing recent project.')
    );
    tauri.pickDirectory.mockResolvedValueOnce('/tmp/other-project');

    render(App);

    expect(await screen.findByText('Unavailable')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Locate' }));

    expect(
      await screen.findByText(
        'The selected folder is a valid Diamond project, but it does not match the missing recent project.'
      )
    ).toBeInTheDocument();
    expect(screen.getByText('Reopen quickly')).toBeInTheDocument();
  });

  it('runs a manifest pipeline from the inspector', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    expect(screen.getByText('Review Pipeline')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Run Review Pipeline' }));

    await waitFor(() =>
      expect(tauri.executePipeline).toHaveBeenCalledWith('/tmp/story-lab', 'review-pipeline')
    );

    expect(await screen.findByText('Pipeline complete')).toBeInTheDocument();
    expect(screen.getByText('1 / 1 blocks completed')).toBeInTheDocument();
  });

  it('creates a pipeline from the inspector authoring controls', async () => {
    tauri.listProjectPipelines
      .mockResolvedValueOnce(pipelines)
      .mockResolvedValueOnce([
        ...pipelines,
        {
          pipelineId: 'draft-pipeline',
          name: 'Draft Pipeline',
          executionMode: 'sequential',
          blocks: [promptBlocks[0], promptBlocks[1]]
        }
      ]);

    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await fireEvent.click(screen.getByRole('button', { name: 'New Pipeline' }));
    await fireEvent.input(screen.getByRole('textbox', { name: 'Pipeline name' }), {
      target: { value: 'Draft Pipeline' }
    });

    const pipelineSelect = screen.getByRole('combobox', { name: 'Available prompt blocks' });
    await fireEvent.change(pipelineSelect, { target: { value: 'brief-review' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add Block' }));
    await fireEvent.change(pipelineSelect, { target: { value: 'scene-draft' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add Block' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Save Pipeline' }));

    await waitFor(() =>
      expect(tauri.createPipeline).toHaveBeenCalledWith('/tmp/story-lab', 'Draft Pipeline', [
        'brief-review',
        'scene-draft'
      ])
    );

    expect(await screen.findByText('Draft Pipeline')).toBeInTheDocument();
  });

  it('edits a pipeline from the inspector authoring controls', async () => {
    tauri.listProjectPipelines
      .mockResolvedValueOnce(pipelines)
      .mockResolvedValueOnce([
        {
          pipelineId: 'review-pipeline',
          name: 'Revised Review Pipeline',
          executionMode: 'sequential',
          blocks: [promptBlocks[0], promptBlocks[1]]
        }
      ]);

    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await fireEvent.click(screen.getAllByRole('button', { name: 'Edit' })[0]);
    const editNameInput = screen.getByRole('textbox', { name: 'Pipeline name for Review Pipeline' });
    await fireEvent.input(editNameInput, {
      target: { value: 'Revised Review Pipeline' }
    });

    const editSelect = screen.getByRole('combobox', {
      name: 'Available prompt blocks for Review Pipeline'
    });
    await fireEvent.change(editSelect, { target: { value: 'scene-draft' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add Block' }));
    await fireEvent.click(screen.getByRole('button', { name: 'Save Changes' }));

    await waitFor(() =>
      expect(tauri.updatePipeline).toHaveBeenCalledWith(
        '/tmp/story-lab',
        'review-pipeline',
        'Revised Review Pipeline',
        ['brief-review', 'scene-draft']
      )
    );

    expect(await screen.findByText('Revised Review Pipeline')).toBeInTheDocument();
  });

  it('filters project run history by pipeline in the inspector', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    expect(await screen.findByText('Pipeline output preview.')).toBeInTheDocument();
    expect(screen.getByText('Provider timeout')).toBeInTheDocument();

    const historyFilter = screen.getByRole('combobox');

    await fireEvent.change(historyFilter, {
      target: { value: 'pipeline:review-pipeline' }
    });

    expect(historyFilter).toHaveValue('pipeline:review-pipeline');
    expect(screen.getByText('Pipeline output preview.')).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByText('Provider timeout')).not.toBeInTheDocument());
  });

  it('blocks pipeline runs when a related prompt tab has unsaved changes', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    const editor = await screen.findByTestId('asset-editor');
    await fireEvent.input(editor, { target: { value: `${teraAssetContent.content}\n# unsaved` } });
    await fireEvent.click(screen.getByRole('button', { name: 'Run Review Pipeline' }));

    expect(
      await screen.findByText(
        'Save prompt changes before running Review Pipeline. Pipeline runs use the saved files on disk.'
      )
    ).toBeInTheDocument();
    expect(tauri.executePipeline).not.toHaveBeenCalled();
  });

  it('edits and saves an editable asset', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('documents', 'context.md');

    const editor = await screen.findByTestId('asset-editor');
    await fireEvent.input(editor, { target: { value: '# Context\n\nUpdated document.' } });

    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() =>
      expect(tauri.writeProjectAsset).toHaveBeenCalledWith(
        '/tmp/story-lab',
        'documents/context.md',
        '# Context\n\nUpdated document.'
      )
    );

    expect(screen.getByText('Saved')).toBeInTheDocument();
  });

  it('exports selected open tabs into a derived bundle from the inspector', async () => {
    tauri.listProjectAssets
      .mockResolvedValueOnce(assetNodes)
      .mockResolvedValueOnce(assetNodes)
      .mockResolvedValueOnce([
        ...assetNodes,
        {
          name: 'exports',
          path: 'exports',
          kind: 'directory',
          isDirectory: true,
          children: [
            {
              name: 'session-export',
              path: 'exports/session-export',
              kind: 'directory',
              isDirectory: true,
              children: []
            }
          ]
        }
      ]);

    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('documents', 'context.md');
    await openExplorerAsset('prompts', 'brief-review.tera');

    await fireEvent.input(screen.getByRole('textbox', { name: 'Export bundle name' }), {
      target: { value: 'Session Export' }
    });

    const exportsSection = screen.getByText('Exports').closest('section');
    expect(exportsSection).toBeTruthy();
    const documentExportItem = within(exportsSection!).getByText('documents/context.md').closest('label');
    expect(documentExportItem).toBeTruthy();
    const documentCheckbox = within(documentExportItem!).getByRole('checkbox') as HTMLInputElement;
    if (!documentCheckbox.checked) {
      await fireEvent.click(documentExportItem!);
    }

    const promptExportItem = within(exportsSection!).getByText('prompts/brief-review.tera').closest('label');
    expect(promptExportItem).toBeTruthy();
    const promptCheckbox = within(promptExportItem!).getByRole('checkbox') as HTMLInputElement;
    if (!promptCheckbox.checked) {
      await fireEvent.click(promptExportItem!);
    }
    await fireEvent.click(screen.getByRole('button', { name: 'Export Bundle' }));

    await waitFor(() =>
      expect(tauri.exportProjectAssets).toHaveBeenCalledWith('/tmp/story-lab', 'Session Export', [
        'documents/context.md',
        'prompts/brief-review.tera'
      ])
    );

    expect(await screen.findByText('Last export: exports/session-export')).toBeInTheDocument();
  });

  it('opens yaml presets in the editor instead of read-only mode', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('models', 'default.yaml');

    const editor = await screen.findByTestId('asset-editor');
    expect(editor).toHaveValue(yamlAssetContent.content);
    expect(screen.queryByText('Read-only View')).not.toBeInTheDocument();
  });

  it('shows template preview and validation state in the bottom panel', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    await waitFor(() =>
      expect(tauri.validateProjectTemplate).toHaveBeenCalledWith(
        '/tmp/story-lab',
        'prompts/brief-review.tera',
        teraAssetContent.content
      )
    );

    expect(await screen.findByText('valid')).toBeInTheDocument();
    expect(screen.getByText('Preview')).toBeInTheDocument();
    expect(screen.getByText(/Diamond Prompt Runner stores prompts/)).toBeInTheDocument();
  });

  it('clears validation result when switching away from a tera tab', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');
    await waitFor(() => expect(screen.findByText('valid')).toBeTruthy());

    await openExplorerAsset('documents', 'context.md');

    await waitFor(() =>
      expect(
        screen.getByText('Template preview and validation are available for `.tera` prompt templates in this slice.')
      ).toBeInTheDocument()
    );
    expect(screen.queryByText('valid')).not.toBeInTheDocument();
  });

  it('shows loading state while validation is in flight', async () => {
    let resolveValidation: ((result: TemplateValidationResult) => void) | null = null;
    tauri.validateProjectTemplate.mockReturnValueOnce(
      new Promise<TemplateValidationResult>((resolve) => {
        resolveValidation = resolve;
      })
    );

    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    await waitFor(() =>
      expect(
        screen.getByText('Refreshing validation from the current draft…')
      ).toBeInTheDocument()
    );

    resolveValidation!(validationResult);
    await waitFor(() => expect(screen.getByText('valid')).toBeInTheDocument());
  });

  it('re-runs validation when the draft content changes', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    await waitFor(() =>
      expect(tauri.validateProjectTemplate).toHaveBeenCalledWith(
        '/tmp/story-lab',
        'prompts/brief-review.tera',
        teraAssetContent.content
      )
    );

    const editor = await screen.findByTestId('asset-editor');
    const updatedContent = 'Updated template content.';
    await fireEvent.input(editor, { target: { value: updatedContent } });

    await waitFor(() =>
      expect(tauri.validateProjectTemplate).toHaveBeenCalledWith(
        '/tmp/story-lab',
        'prompts/brief-review.tera',
        updatedContent
      )
    );
  });

  it('shows the online research affordance when the prompt opts in locally', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    const editor = await screen.findByTestId('asset-editor');
    await fireEvent.input(editor, {
      target: { value: `${ONLINE_PROMPT_DIRECTIVE}\n${teraAssetContent.content}` }
    });

    expect(await screen.findByText('Online enabled')).toBeInTheDocument();
    expect(
      await screen.findByText(/This prompt will use web-backed execution because its first non-empty line is/)
    ).toBeInTheDocument();
  });

  it('runs the active tera prompt and shows the latest execution output', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    await fireEvent.click(await screen.findByRole('button', { name: 'Run' }));

    await waitFor(() =>
      expect(tauri.executePromptBlock).toHaveBeenCalledWith(
        '/tmp/story-lab',
        'prompts/brief-review.tera',
        teraAssetContent.content
      )
    );

    expect(await screen.findByText('Execution output from the provider.')).toBeInTheDocument();
    expect(screen.getByText('runs/run-1.json')).toBeInTheDocument();
  });

  it('shows execution failure in the bottom panel', async () => {
    tauri.executePromptBlock.mockRejectedValueOnce(
      new Error('Missing OpenRouter API key. Save one in the app or set OPENROUTER_API_KEY.')
    );

    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    await fireEvent.click(await screen.findByRole('button', { name: 'Run' }));

    expect(await screen.findByText('Run failed')).toBeInTheDocument();
    expect(
      screen.getByText('Missing OpenRouter API key. Save one in the app or set OPENROUTER_API_KEY.')
    ).toBeInTheDocument();
  });

  it('stores an execution api key in app credential storage', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    const input = await screen.findByLabelText('OpenRouter API key');
    await fireEvent.input(input, { target: { value: 'sk-test-123' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Save key' }));

    await waitFor(() => expect(tauri.saveExecutionApiKey).toHaveBeenCalledWith('sk-test-123'));

    expect(await screen.findByText('Stored in the native keychain for this app.')).toBeInTheDocument();
    expect(input).toHaveValue('');
  });

  it('shows prompt run history and opens a persisted run artifact', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    await openExplorerAsset('prompts', 'brief-review.tera');

    const historyPreview = await screen.findByText('Earlier persisted output.');
    const historyItem = historyPreview.closest('article');
    expect(historyItem).not.toBeNull();
    await fireEvent.click(within(historyItem!).getByRole('button', { name: 'Open artifact' }));

    await waitFor(() =>
      expect(tauri.readProjectAsset).toHaveBeenCalledWith('/tmp/story-lab', 'runs/run-2.json')
    );

    expect(await screen.findByText('Structured View')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'run-2.json', level: 2 })).toBeInTheDocument();
  });
});
