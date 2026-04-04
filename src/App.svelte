<script lang="ts">
  import { onMount } from 'svelte';
  import ProjectBrowser from '$lib/components/ProjectBrowser.svelte';
  import WorkspaceShell from '$lib/components/WorkspaceShell.svelte';
  import {
    clearExecutionApiKey,
    createPipeline,
    createProject,
    createPromptBlock,
    executePipeline,
    executePromptBlock,
    exportProjectAssets,
    getExecutionCredentialStatus,
    getGlobalVariables,
    getRecentProjects,
    getProjectUsageSummary,
    listProjectPromptBlocks,
    listProjectRunHistory,
    listProjectPipelines,
    listPromptRunHistory,
    listProjectAssets,
    locateRecentProject,
    openProject,
    pickDirectory,
    removeRecentProject,
    readProjectAsset,
    saveExecutionApiKey,
    setGlobalVariables,
    setProjectVariables,
    updatePipeline,
    validateProjectTemplate,
    writeProjectAsset
  } from '$lib/tauri';
  import type {
    ExecutionCredentialStatus,
    CreatedPromptBlockResult,
    ExportBundleResult,
    PipelineExecutionResult,
    ProjectAssetNode,
    ProjectPipeline,
    ProjectPromptBlock,
    ProjectRunHistoryEntry,
    ProjectUsageSummary,
    PromptExecutionResult,
    PromptRunHistoryEntry,
    ProjectSummary,
    RecentProjectEntry,
    SavedPipelineResult,
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
  let projectPipelines = $state<ProjectPipeline[]>([]);
  let projectPromptBlocks = $state<ProjectPromptBlock[]>([]);
  let tabs = $state<WorkspaceTab[]>([]);
  let activePath = $state<string | null>(null);
  let loadingPath = $state<string | null>(null);
  let validationResult = $state<TemplateValidationResult | null>(null);
  let validationLoading = $state(false);
  let executionResult = $state<PromptExecutionResult | null>(null);
  let executionLoading = $state(false);
  let executionHistory = $state<PromptRunHistoryEntry[]>([]);
  let executionHistoryLoading = $state(false);
  let projectRunHistory = $state<ProjectRunHistoryEntry[]>([]);
  let projectRunHistoryLoading = $state(false);
  let projectUsageSummary = $state<ProjectUsageSummary | null>(null);
  let globalVariables = $state<Record<string, string>>({});
  let pipelineExecutionResult = $state<PipelineExecutionResult | null>(null);
  let pipelineExecutionLoading = $state(false);
  let pipelineAuthoringLoading = $state(false);
  let exportLoading = $state(false);
  let promptCreationLoading = $state(false);
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
      const [projects, credentialStatus, globals] = await Promise.all([
        getRecentProjects(),
        getExecutionCredentialStatus(),
        getGlobalVariables()
      ]);
      recentProjects = projects;
      executionCredentialStatus = credentialStatus;
      globalVariables = globals;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : typeof error === 'string' ? error : 'Failed to load app state.';
    }
  });

  async function enterWorkspace(summary: ProjectSummary): Promise<void> {
    workspace = summary;
    const [nodes, pipelines, promptBlocks, runHistory, usageSummary] = await Promise.all([
      listProjectAssets(summary.rootPath),
      listProjectPipelines(summary.rootPath),
      listProjectPromptBlocks(summary.rootPath),
      listProjectRunHistory(summary.rootPath),
      getProjectUsageSummary(summary.rootPath)
    ]);
    assetNodes = nodes;
    projectPipelines = pipelines;
    projectPromptBlocks = promptBlocks;
    projectRunHistory = runHistory;
    projectUsageSummary = usageSummary;
    tabs = [];
    activePath = null;
    loadingPath = null;
    validationResult = null;
    validationLoading = false;
    executionResult = null;
    executionLoading = false;
    executionHistory = [];
    executionHistoryLoading = false;
    projectRunHistoryLoading = false;
    pipelineExecutionResult = null;
    pipelineExecutionLoading = false;
    pipelineAuthoringLoading = false;
    exportLoading = false;
    mode = 'workspace';
  }

  function handleCloseProject(): void {
    mode = 'browser';
    workspace = null;
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

  async function handleLocateRecent(project: RecentProjectEntry): Promise<void> {
    const selected = await pickDirectory(`Locate ${project.projectName}`);
    if (!selected) {
      return;
    }

    await withBusy(async () => {
      const summary = await locateRecentProject(project.rootPath, selected);
      recentProjects = await getRecentProjects();
      await enterWorkspace(summary);
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
        title: title ?? asset.metadata.name ?? path.split('/').pop() ?? path,
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

  async function handleCreatePrompt(promptName: string): Promise<void> {
    if (!workspace || promptCreationLoading) {
      return;
    }

    promptCreationLoading = true;
    errorMessage = null;

    try {
      const created: CreatedPromptBlockResult = await createPromptBlock(workspace.rootPath, promptName);
      workspace = created.summary;
      recentProjects = await getRecentProjects();
      const [nodes, pipelines, promptBlocks] = await Promise.all([
        listProjectAssets(created.summary.rootPath),
        listProjectPipelines(created.summary.rootPath),
        listProjectPromptBlocks(created.summary.rootPath)
      ]);
      assetNodes = nodes;
      projectPipelines = pipelines;
      projectPromptBlocks = promptBlocks;
      await openAssetPath(created.path);
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to create prompt.';
      throw error;
    } finally {
      promptCreationLoading = false;
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
      await refreshProjectRunHistory(workspace.rootPath);
      assetNodes = await listProjectAssets(workspace.rootPath);
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
        pipelineId: null,
        pipelineName: null,
        modelPreset: '',
        modelId: '',
        status: 'failed',
        renderedPrompt: '',
        output: null,
        error: message,
        runPath: '',
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
        online: {
          enabled: false,
          webSearchRequests: 0,
          citationCount: 0
        },
        usage: {
          promptTokens: null,
          completionTokens: null,
          totalTokens: null,
          cost: null,
          outputWordCount: null,
          retryCount: null
        }
      };
    } finally {
      executionLoading = false;
    }
  }

  function latestStepForPath(
    steps: PromptExecutionResult[],
    path: string
  ): PromptExecutionResult | null {
    for (let index = steps.length - 1; index >= 0; index -= 1) {
      if (steps[index]?.path === path) {
        return steps[index];
      }
    }

    return null;
  }

  async function handleRunPipeline(pipelineId: string): Promise<void> {
    if (!workspace || pipelineExecutionLoading) {
      return;
    }

    const pipeline = projectPipelines.find((candidate) => candidate.pipelineId === pipelineId);
    if (!pipeline) {
      return;
    }

    const dirtyPrompt = tabs.find(
      (tab) =>
        tab.kind === 'tera' &&
        tab.draftContent !== tab.savedContent &&
        pipeline.blocks.some((block) => block.templateSource === tab.path)
    );

    if (dirtyPrompt) {
      errorMessage = `Save prompt changes before running ${pipeline.name}. Pipeline runs use the saved files on disk.`;
      return;
    }

    pipelineExecutionLoading = true;
    errorMessage = null;

    try {
      const result = await executePipeline(workspace.rootPath, pipelineId);
      pipelineExecutionResult = result;
      await refreshProjectRunHistory(workspace.rootPath);

      if (result.steps.length > 0) {
        executionResult = result.steps[result.steps.length - 1] ?? null;
        assetNodes = await listProjectAssets(workspace.rootPath);

        if (activePath) {
          const activeStep = latestStepForPath(result.steps, activePath);
          if (activeStep) {
            executionResult = activeStep;
            await refreshExecutionHistory(workspace.rootPath, activeStep.path);
          }
        }

        workspace = {
          ...workspace,
          counts: {
            ...workspace.counts,
            runs: workspace.counts.runs + result.steps.length
          }
        };
      }
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : `Failed to run ${pipeline.name}.`;
    } finally {
      pipelineExecutionLoading = false;
    }
  }

  async function handleCreatePipeline(
    pipelineName: string,
    orderedBlockIds: string[]
  ): Promise<SavedPipelineResult> {
    if (!workspace || pipelineAuthoringLoading) {
      throw new Error('Pipeline authoring is not available right now.');
    }

    pipelineAuthoringLoading = true;
    errorMessage = null;

    try {
      const result = await createPipeline(workspace.rootPath, pipelineName, orderedBlockIds);
      workspace = result.summary;
      recentProjects = await getRecentProjects();
      await refreshPipelineAuthoringState(result.summary.rootPath);
      return result;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to create pipeline.';
      throw error;
    } finally {
      pipelineAuthoringLoading = false;
    }
  }

  async function handleUpdatePipeline(
    pipelineId: string,
    pipelineName: string,
    orderedBlockIds: string[]
  ): Promise<SavedPipelineResult> {
    if (!workspace || pipelineAuthoringLoading) {
      throw new Error('Pipeline authoring is not available right now.');
    }

    pipelineAuthoringLoading = true;
    errorMessage = null;

    try {
      const result = await updatePipeline(
        workspace.rootPath,
        pipelineId,
        pipelineName,
        orderedBlockIds
      );
      workspace = result.summary;
      recentProjects = await getRecentProjects();
      await refreshPipelineAuthoringState(result.summary.rootPath);
      return result;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to update pipeline.';
      throw error;
    } finally {
      pipelineAuthoringLoading = false;
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

  async function refreshProjectRunHistory(rootPath: string): Promise<void> {
    projectRunHistoryLoading = true;

    try {
      [projectRunHistory, projectUsageSummary] = await Promise.all([
        listProjectRunHistory(rootPath),
        getProjectUsageSummary(rootPath)
      ]);
    } catch {
      projectRunHistory = [];
      projectUsageSummary = null;
    } finally {
      projectRunHistoryLoading = false;
    }
  }

  async function handleSetGlobalVariables(variables: Record<string, string>): Promise<void> {
    globalVariables = await setGlobalVariables(variables);
  }

  async function handleSetProjectVariables(variables: Record<string, string>): Promise<void> {
    if (!workspace) return;
    workspace = await setProjectVariables(workspace.rootPath, variables);
  }

  async function refreshPipelineAuthoringState(rootPath: string): Promise<void> {
    const [pipelines, promptBlocks] = await Promise.all([
      listProjectPipelines(rootPath),
      listProjectPromptBlocks(rootPath)
    ]);

    projectPipelines = pipelines;
    projectPromptBlocks = promptBlocks;
  }

  async function handleExportAssets(
    bundleName: string,
    relativePaths: string[]
  ): Promise<ExportBundleResult> {
    if (!workspace || exportLoading) {
      throw new Error('Export is not available right now.');
    }

    const dirtyTab = tabs.find(
      (tab) =>
        relativePaths.includes(tab.path) &&
        tab.isEditable &&
        tab.draftContent !== tab.savedContent
    );

    if (dirtyTab) {
      const error = new Error(
        `Save changes to ${dirtyTab.title} before exporting. Export bundles use the saved files on disk.`
      );
      errorMessage = error.message;
      throw error;
    }

    exportLoading = true;
    errorMessage = null;

    try {
      const result = await exportProjectAssets(workspace.rootPath, bundleName, relativePaths);
      workspace = result.summary;
      recentProjects = await getRecentProjects();
      assetNodes = await listProjectAssets(result.summary.rootPath);
      return result;
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : 'Failed to export project assets.';
      throw error;
    } finally {
      exportLoading = false;
    }
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
    onLocateRecent={handleLocateRecent}
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
    pipelines={projectPipelines}
    promptBlocks={projectPromptBlocks}
    pipelineExecution={pipelineExecutionResult}
    pipelineLoading={pipelineExecutionLoading}
    {pipelineAuthoringLoading}
    {projectRunHistory}
    {projectRunHistoryLoading}
    {projectUsageSummary}
    {globalVariables}
    onSetGlobalVariables={handleSetGlobalVariables}
    onSetProjectVariables={handleSetProjectVariables}
    onSelectAsset={handleSelectAsset}
    onSelectTab={handleSelectTab}
    onCloseTab={handleCloseTab}
    onDraftChange={handleDraftChange}
    onSaveTab={handleSaveTab}
    onReloadTab={handleReloadTab}
    onRunTab={handleExecuteTab}
    onRunPipeline={handleRunPipeline}
    onCreatePipeline={handleCreatePipeline}
    onUpdatePipeline={handleUpdatePipeline}
    onExportAssets={handleExportAssets}
    {exportLoading}
    onCreatePrompt={handleCreatePrompt}
    {promptCreationLoading}
    credentialState={executionCredentialStatus}
    credentialDraft={executionCredentialDraft}
    credentialLoading={executionCredentialLoading}
    onExecutionCredentialInput={(value: string) => (executionCredentialDraft = value)}
    onSaveExecutionCredential={handleSaveExecutionCredential}
    onClearExecutionCredential={handleClearExecutionCredential}
    historyItems={executionHistory}
    historyLoading={executionHistoryLoading}
    onOpenRunPath={handleOpenRunPath}
    onCloseProject={handleCloseProject}
    {validationResult}
    {validationLoading}
    {executionResult}
    {executionLoading}
  />
{/if}
