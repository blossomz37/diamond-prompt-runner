<script lang="ts">
  import { onMount } from 'svelte';
  import ProjectBrowser from '$lib/components/ProjectBrowser.svelte';
  import WorkspaceShell from '$lib/components/WorkspaceShell.svelte';
  import {
    clearExecutionApiKey,
    createProject,
    executePromptBlock,
    getExecutionCredentialStatus,
    getRecentProjects,
    listPromptRunHistory,
    listProjectAssets,
    openProject,
    pickDirectory,
    removeRecentProject,
    readProjectAsset,
    saveExecutionApiKey,
    validateProjectTemplate,
    writeProjectAsset
  } from '$lib/tauri';
  import type {
    ExecutionCredentialStatus,
    ProjectAssetNode,
    PromptExecutionResult,
    PromptRunHistoryEntry,
    ProjectSummary,
    RecentProjectEntry,
    TemplateValidationResult,
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
  let validationResult = $state<TemplateValidationResult | null>(null);
  let validationLoading = $state(false);
  let executionResult = $state<PromptExecutionResult | null>(null);
  let executionLoading = $state(false);
  let executionHistory = $state<PromptRunHistoryEntry[]>([]);
  let executionHistoryLoading = $state(false);
  let executionCredentialStatus = $state<ExecutionCredentialStatus>({
    source: 'missing',
    hasStoredKey: false
  });
  let executionCredentialDraft = $state('');
  let executionCredentialLoading = $state(false);
  let validationTimer: ReturnType<typeof setTimeout> | null = null;
  let validationRequestId = 0;
  let executionHistoryRequestId = 0;

  onMount(async () => {
    try {
      const [projects, credentialStatus] = await Promise.all([
        getRecentProjects(),
        getExecutionCredentialStatus()
      ]);
      recentProjects = projects;
      executionCredentialStatus = credentialStatus;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to load app state.';
    }
  });

  async function enterWorkspace(summary: ProjectSummary): Promise<void> {
    workspace = summary;
    assetNodes = await listProjectAssets(summary.rootPath);
    tabs = [];
    activePath = null;
    loadingPath = null;
    validationResult = null;
    validationLoading = false;
    executionResult = null;
    executionLoading = false;
    executionHistory = [];
    executionHistoryLoading = false;
    mode = 'workspace';
  }

  $effect(() => {
    const currentTab = tabs.find((tab) => tab.path === activePath) ?? null;
    const rootPath = workspace?.rootPath ?? null;

    if (validationTimer) {
      clearTimeout(validationTimer);
      validationTimer = null;
    }

    if (!rootPath || !currentTab || currentTab.kind !== 'tera') {
      validationRequestId += 1;
      validationLoading = false;
      validationResult = null;
      return;
    }

    validationLoading = true;
    const path = currentTab.path;
    const content = currentTab.draftContent;

    validationTimer = setTimeout(() => {
      void runTemplateValidation(rootPath, path, content);
    }, 250);

    return () => {
      if (validationTimer) {
        clearTimeout(validationTimer);
        validationTimer = null;
      }
    };
  });

  $effect(() => {
    const currentTab = tabs.find((tab) => tab.path === activePath) ?? null;
    const rootPath = workspace?.rootPath ?? null;

    if (!rootPath || !currentTab || currentTab.kind !== 'tera') {
      executionHistoryRequestId += 1;
      executionHistoryLoading = false;
      executionHistory = [];
      return;
    }

    const requestId = ++executionHistoryRequestId;
    executionHistoryLoading = true;
    void loadExecutionHistory(rootPath, currentTab.path, requestId);
  });

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

    await openAssetPath(node.path, node.name);
  }

  async function openAssetPath(path: string, title?: string): Promise<void> {
    if (!workspace) {
      return;
    }

    const existing = tabs.find((tab) => tab.path === path);
    if (existing) {
      activePath = existing.path;
      return;
    }

    loadingPath = path;
    errorMessage = null;

    try {
      const asset = await readProjectAsset(workspace.rootPath, path);
      const nextTab: WorkspaceTab = {
        ...asset,
        title: title ?? path.split('/').pop() ?? path,
        savedContent: asset.content,
        draftContent: asset.content,
        isSaving: false
      };

      tabs = [...tabs, nextTab];
      activePath = path;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : `Failed to open ${path}.`;
    } finally {
      loadingPath = null;
    }
  }

  function handleSelectTab(path: string): void {
    activePath = path;
  }

  function updateTab(path: string, mutate: (tab: WorkspaceTab) => WorkspaceTab): void {
    tabs = tabs.map((tab) => (tab.path === path ? mutate(tab) : tab));
  }

  function handleDraftChange(path: string, content: string): void {
    updateTab(path, (tab) => ({
      ...tab,
      draftContent: content
    }));
  }

  async function runTemplateValidation(
    rootPath: string,
    path: string,
    content: string
  ): Promise<void> {
    const requestId = ++validationRequestId;

    try {
      const result = await validateProjectTemplate(rootPath, path, content);
      if (requestId !== validationRequestId) {
        return;
      }

      validationResult = result;
    } catch (error) {
      if (requestId !== validationRequestId) {
        return;
      }

      validationResult = {
        path,
        status: 'invalid',
        preview: null,
        warnings: [],
        errors: [error instanceof Error ? error.message : 'Template validation failed.'],
        contextSummary: []
      };
    } finally {
      if (requestId === validationRequestId) {
        validationLoading = false;
      }
    }
  }

  async function handleSaveTab(path: string): Promise<void> {
    if (!workspace) {
      return;
    }

    const tab = tabs.find((candidate) => candidate.path === path);
    if (!tab || !tab.isEditable || tab.draftContent === tab.savedContent || tab.isSaving) {
      return;
    }

    updateTab(path, (current) => ({ ...current, isSaving: true }));
    errorMessage = null;

    try {
      const asset = await writeProjectAsset(workspace.rootPath, path, tab.draftContent);
      updateTab(path, (current) => ({
        ...current,
        ...asset,
        title: current.title,
        savedContent: asset.content,
        draftContent: asset.content,
        isSaving: false
      }));
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : `Failed to save ${path}.`;
      updateTab(path, (current) => ({ ...current, isSaving: false }));
    }
  }

  async function handleReloadTab(path: string): Promise<void> {
    if (!workspace) {
      return;
    }

    const tab = tabs.find((candidate) => candidate.path === path);
    if (!tab || tab.isSaving) {
      return;
    }

    updateTab(path, (current) => ({ ...current, isSaving: true }));
    errorMessage = null;

    try {
      const asset = await readProjectAsset(workspace.rootPath, path);
      updateTab(path, (current) => ({
        ...current,
        ...asset,
        title: current.title,
        savedContent: asset.content,
        draftContent: asset.content,
        isSaving: false
      }));
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : `Failed to reload ${path}.`;
      updateTab(path, (current) => ({ ...current, isSaving: false }));
    }
  }

  async function handleExecuteTab(path: string): Promise<void> {
    if (!workspace) {
      return;
    }

    const tab = tabs.find((candidate) => candidate.path === path);
    if (!tab || tab.kind !== 'tera' || executionLoading) {
      return;
    }

    executionLoading = true;
    errorMessage = null;

    try {
      executionResult = await executePromptBlock(workspace.rootPath, path, tab.draftContent);
      await refreshExecutionHistory(workspace.rootPath, path);
      workspace = {
        ...workspace,
        counts: {
          ...workspace.counts,
          runs: workspace.counts.runs + 1
        }
      };
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Prompt execution failed.';
      executionResult = {
        runId: '',
        path,
        blockId: null,
        blockName: tab.title,
        modelPreset: '',
        modelId: '',
        status: 'failed',
        renderedPrompt: '',
        output: null,
        error: message,
        runPath: '',
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString()
      };
    } finally {
      executionLoading = false;
    }
  }

  async function loadExecutionHistory(
    rootPath: string,
    path: string,
    requestId: number
  ): Promise<void> {
    try {
      const history = await listPromptRunHistory(rootPath, path);
      if (requestId !== executionHistoryRequestId) {
        return;
      }

      executionHistory = history;
    } catch {
      if (requestId !== executionHistoryRequestId) {
        return;
      }

      executionHistory = [];
    } finally {
      if (requestId === executionHistoryRequestId) {
        executionHistoryLoading = false;
      }
    }
  }

  async function refreshExecutionHistory(rootPath: string, path: string): Promise<void> {
    const requestId = ++executionHistoryRequestId;
    executionHistoryLoading = true;
    await loadExecutionHistory(rootPath, path, requestId);
  }

  async function handleOpenRunPath(path: string): Promise<void> {
    await openAssetPath(path);
  }

  async function handleSaveExecutionCredential(): Promise<void> {
    if (executionCredentialLoading) {
      return;
    }

    const apiKey = executionCredentialDraft.trim();
    if (!apiKey) {
      return;
    }

    executionCredentialLoading = true;
    errorMessage = null;

    try {
      executionCredentialStatus = await saveExecutionApiKey(apiKey);
      executionCredentialDraft = '';
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to save execution API key.';
    } finally {
      executionCredentialLoading = false;
    }
  }

  async function handleClearExecutionCredential(): Promise<void> {
    if (executionCredentialLoading) {
      return;
    }

    executionCredentialLoading = true;
    errorMessage = null;

    try {
      executionCredentialStatus = await clearExecutionApiKey();
      executionCredentialDraft = '';
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to clear execution API key.';
    } finally {
      executionCredentialLoading = false;
    }
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
    onDraftChange={handleDraftChange}
    onSaveTab={handleSaveTab}
    onReloadTab={handleReloadTab}
    onRunTab={handleExecuteTab}
    credentialState={executionCredentialStatus}
    credentialDraft={executionCredentialDraft}
    credentialLoading={executionCredentialLoading}
    onExecutionCredentialInput={(value: string) => (executionCredentialDraft = value)}
    onSaveExecutionCredential={handleSaveExecutionCredential}
    onClearExecutionCredential={handleClearExecutionCredential}
    historyItems={executionHistory}
    historyLoading={executionHistoryLoading}
    onOpenRunPath={handleOpenRunPath}
    {validationResult}
    {validationLoading}
    {executionResult}
    {executionLoading}
  />
{/if}
