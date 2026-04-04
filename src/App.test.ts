import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App.svelte';
import type {
  AssetContent,
  ProjectAssetNode,
  ProjectSummary,
  RecentProjectEntry,
  TemplateValidationResult
} from '$lib/types/project';

const tauri = vi.hoisted(() => ({
  createProject: vi.fn(),
  getRecentProjects: vi.fn(),
  listProjectAssets: vi.fn(),
  openProject: vi.fn(),
  pickDirectory: vi.fn(),
  removeRecentProject: vi.fn(),
  readProjectAsset: vi.fn(),
  validateProjectTemplate: vi.fn(),
  writeProjectAsset: vi.fn()
}));

vi.mock('$lib/tauri', () => tauri);

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

describe('App', () => {
  beforeEach(() => {
    tauri.getRecentProjects.mockResolvedValue(recents);
    tauri.pickDirectory.mockResolvedValue('/tmp');
    tauri.createProject.mockResolvedValue(summary);
    tauri.openProject.mockResolvedValue(summary);
    tauri.removeRecentProject.mockResolvedValue(undefined);
    tauri.listProjectAssets.mockResolvedValue(assetNodes);
    tauri.readProjectAsset.mockImplementation(async (_rootPath: string, relativePath: string) => {
      if (relativePath === 'models/default.yaml') {
        return yamlAssetContent;
      }

      if (relativePath === 'prompts/brief-review.tera') {
        return teraAssetContent;
      }

      return assetContent;
    });
    tauri.validateProjectTemplate.mockResolvedValue(validationResult);
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

    const explorer = screen.getByTestId('explorer-tree');
    await fireEvent.click(within(explorer).getByText('documents'));
    await fireEvent.click(within(explorer).getByText('context.md'));
    await waitFor(() => expect(screen.getAllByText('context.md')).not.toHaveLength(0));

    await fireEvent.click(within(explorer).getByText('context.md'));

    expect(tauri.readProjectAsset).toHaveBeenCalledTimes(1);
    expect(screen.getByText('Lines')).toBeInTheDocument();
    expect(screen.getByText('3')).toBeInTheDocument();
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

  it('edits and saves an editable asset', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    const explorer = screen.getByTestId('explorer-tree');
    await fireEvent.click(within(explorer).getByText('documents'));
    await fireEvent.click(within(explorer).getByText('context.md'));

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

  it('opens yaml presets in the editor instead of read-only mode', async () => {
    render(App);

    await fireEvent.click(await screen.findByText('Open Existing Project'));

    await waitFor(() =>
      expect(screen.getByRole('heading', { name: 'Story Lab', level: 1 })).toBeInTheDocument()
    );

    const explorer = screen.getByTestId('explorer-tree');
    await fireEvent.click(within(explorer).getByText('models'));
    await fireEvent.click(within(explorer).getByText('default.yaml'));

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

    const explorer = screen.getByTestId('explorer-tree');
    await fireEvent.click(within(explorer).getByText('prompts'));
    await fireEvent.click(within(explorer).getByText('brief-review.tera'));

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

    const explorer = screen.getByTestId('explorer-tree');

    await fireEvent.click(within(explorer).getByText('prompts'));
    await fireEvent.click(within(explorer).getByText('brief-review.tera'));
    await waitFor(() => expect(screen.findByText('valid')).toBeTruthy());

    await fireEvent.click(within(explorer).getByText('documents'));
    await fireEvent.click(within(explorer).getByText('context.md'));

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

    const explorer = screen.getByTestId('explorer-tree');
    await fireEvent.click(within(explorer).getByText('prompts'));
    await fireEvent.click(within(explorer).getByText('brief-review.tera'));

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

    const explorer = screen.getByTestId('explorer-tree');
    await fireEvent.click(within(explorer).getByText('prompts'));
    await fireEvent.click(within(explorer).getByText('brief-review.tera'));

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
});
