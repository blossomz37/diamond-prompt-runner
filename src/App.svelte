<script lang="ts">
  import { onMount } from 'svelte';
  import ProjectBrowser from '$lib/components/ProjectBrowser.svelte';
  import WorkspaceShell from '$lib/components/WorkspaceShell.svelte';
  import {
    createProject,
    getRecentProjects,
    listProjectAssets,
    openProject,
    pickDirectory,
    removeRecentProject,
    readProjectAsset
  } from '$lib/tauri';
  import type {
    ProjectAssetNode,
    ProjectSummary,
    RecentProjectEntry,
    WorkspaceTab
  } from '$lib/types/project';

  let mode = $state<'browser' | 'workspace'>('browser');
  let recentProjects = $state<RecentProjectEntry[]>([]);
  let projectName = $state('');
  let parentPath = $state('');
  let busy = $state(false);
  let errorMessage = $state<string | null>(null);
  let workspace = $state<ProjectSummary | null>(null);
  let assetNodes = $state<ProjectAssetNode[]>([]);
  let tabs = $state<WorkspaceTab[]>([]);
  let activePath = $state<string | null>(null);
  let loadingPath = $state<string | null>(null);

  onMount(async () => {
    recentProjects = await getRecentProjects();
  });

  async function enterWorkspace(summary: ProjectSummary): Promise<void> {
    workspace = summary;
    assetNodes = await listProjectAssets(summary.rootPath);
    tabs = [];
    activePath = null;
    loadingPath = null;
    mode = 'workspace';
  }

  async function withBusy(work: () => Promise<void>): Promise<void> {
    busy = true;
    errorMessage = null;

    try {
      await work();
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'The operation failed.';
    } finally {
      busy = false;
    }
  }

  async function handlePickParent(): Promise<void> {
    const path = await pickDirectory('Choose parent folder for new Diamond project');
    if (path) {
      parentPath = path;
    }
  }

  async function handleCreateProject(): Promise<void> {
    await withBusy(async () => {
      const summary = await createProject(parentPath, projectName);
      recentProjects = await getRecentProjects();
      await enterWorkspace(summary);
    });
  }

  async function handleOpenExisting(): Promise<void> {
    const selected = await pickDirectory('Choose an existing Diamond project folder');
    if (!selected) {
      return;
    }

    await withBusy(async () => {
      const summary = await openProject(selected);
      recentProjects = await getRecentProjects();
      await enterWorkspace(summary);
    });
  }

  async function handleOpenRecent(project: ProjectSummary): Promise<void> {
    await withBusy(async () => {
      const summary = await openProject(project.rootPath);
      recentProjects = await getRecentProjects();
      await enterWorkspace(summary);
    });
  }

  async function handleRemoveRecent(rootPath: string): Promise<void> {
    await withBusy(async () => {
      await removeRecentProject(rootPath);
      recentProjects = await getRecentProjects();
    });
  }

  async function handleSelectAsset(node: ProjectAssetNode): Promise<void> {
    if (!workspace || node.isDirectory) {
      return;
    }

    const existing = tabs.find((tab) => tab.path === node.path);
    if (existing) {
      activePath = existing.path;
      return;
    }

    loadingPath = node.path;
    errorMessage = null;

    try {
      const asset = await readProjectAsset(workspace.rootPath, node.path);
      const nextTab: WorkspaceTab = {
        ...asset,
        title: node.name
      };

      tabs = [...tabs, nextTab];
      activePath = node.path;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : `Failed to open ${node.path}.`;
    } finally {
      loadingPath = null;
    }
  }

  function handleSelectTab(path: string): void {
    activePath = path;
  }

  function handleCloseTab(path: string): void {
    const index = tabs.findIndex((tab) => tab.path === path);
    if (index === -1) {
      return;
    }

    const nextTabs = tabs.filter((tab) => tab.path !== path);
    tabs = nextTabs;

    if (activePath !== path) {
      return;
    }

    const fallback = nextTabs[index] ?? nextTabs[index - 1] ?? null;
    activePath = fallback?.path ?? null;
  }
</script>

{#if mode === 'browser' || !workspace}
  <ProjectBrowser
    {recentProjects}
    {projectName}
    {parentPath}
    {busy}
    {errorMessage}
    onProjectNameInput={(value: string) => (projectName = value)}
    onPickParent={handlePickParent}
    onCreateProject={handleCreateProject}
    onOpenExisting={handleOpenExisting}
    onOpenRecent={handleOpenRecent}
    onRemoveRecent={handleRemoveRecent}
  />
{:else}
  <WorkspaceShell
    summary={workspace}
    nodes={assetNodes}
    {tabs}
    {activePath}
    {loadingPath}
    {errorMessage}
    onSelectAsset={handleSelectAsset}
    onSelectTab={handleSelectTab}
    onCloseTab={handleCloseTab}
  />
{/if}
