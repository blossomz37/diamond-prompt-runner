import { fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App.svelte';
import type { AssetContent, ProjectAssetNode, ProjectSummary, RecentProjectEntry } from '$lib/types/project';

const tauri = vi.hoisted(() => ({
  createProject: vi.fn(),
  getRecentProjects: vi.fn(),
  listProjectAssets: vi.fn(),
  openProject: vi.fn(),
  pickDirectory: vi.fn(),
  removeRecentProject: vi.fn(),
  readProjectAsset: vi.fn()
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
  }
];

const assetContent: AssetContent = {
  path: 'documents/context.md',
  kind: 'markdown',
  view: 'text',
  content: '# Context\n\nA small fixture document.',
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

describe('App', () => {
  beforeEach(() => {
    tauri.getRecentProjects.mockResolvedValue(recents);
    tauri.pickDirectory.mockResolvedValue('/tmp');
    tauri.createProject.mockResolvedValue(summary);
    tauri.openProject.mockResolvedValue(summary);
    tauri.removeRecentProject.mockResolvedValue(undefined);
    tauri.listProjectAssets.mockResolvedValue(assetNodes);
    tauri.readProjectAsset.mockResolvedValue(assetContent);
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
});
