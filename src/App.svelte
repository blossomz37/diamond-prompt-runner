<script lang="ts">
  import { onMount } from 'svelte';
  import ProjectBrowser from '$lib/components/ProjectBrowser.svelte';
  import WorkspaceShell from '$lib/components/WorkspaceShell.svelte';
  import {
    auditProjectAsset,
    clearExecutionApiKey,
    convertProjectAsset,
    createModelPreset,
    createPipeline,
    createProject,
    createPromptBlock,
    deleteModelPreset,
    executePipeline,
    executePromptBlock,
    exportProjectAssets,
    getExecutionCredentialStatus,
    getGlobalVariables,
    getRecentProjects,
    listModelPresets,
    listProjectPromptBlocks,
    listProjectRunHistory,
    listProjectPipelines,
    listPromptRunHistory,
    listProjectAssets,
    locateRecentProject,
    openProject,
    pickDirectory,
    readProjectAsset,
    registerPromptBlock,
    removeRecentProject,
    renameProject,
    saveExecutionApiKey,
    setBlockModelPreset,
    setBlockOutputFilename,
    setBlockOutputTarget,
    setDefaultModelPreset,
    setGlobalVariables,
    setProjectVariables,
    updatePipeline,
    writeProjectAsset,
    deletePipeline,
    duplicatePipeline,
    deletePromptBlock,
    deleteRun,
    deleteDocument,
    trashPrompt,
    renameDocument,
    cancelPipeline,
    onPipelineProgress
  } from '$lib/tauri';
  import {
    GLOBAL_VARIABLES_PATH,
    WORKSPACE_VARIABLES_PATH
  } from '$lib/types/project';
  import type {
    AssetContent,
    AssetConversionAuditResult,
    AssetConversionResult,
    AssetMetadata,
    ExecutionCredentialStatus,
    CreatedPromptBlockResult,
    ExportBundleResult,
    ModelPresetSummary,
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
    WorkspaceTab,
    PipelineProgressEvent
  } from '$lib/types/project';
  import { createValidationStore } from '$lib/stores/validation.svelte';
  import { pipelineActivityStore } from '$lib/stores/pipelineActivity.svelte';
  import '$lib/stores/theme.svelte'; // side-effect: applies saved theme on load
  import { check } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { ask } from '@tauri-apps/plugin-dialog';

  let mode = $state<'browser' | 'workspace'>('browser');
  let recentProjects = $state<RecentProjectEntry[]>([]);
  let projectName = $state('');
  let parentPath = $state('');
  let busy = $state(false);
  let errorMessage = $state<string | null>(null);
  type WorkspaceFeedback =
    | {
        kind: 'audit';
        result: AssetConversionAuditResult;
      }
    | {
        kind: 'notice';
        title: string;
        detail: string;
      };

  let feedback = $state<WorkspaceFeedback | null>(null);
  let workspace = $state<ProjectSummary | null>(null);
  let assetNodes = $state<ProjectAssetNode[]>([]);
  let projectPipelines = $state<ProjectPipeline[]>([]);
  let projectPromptBlocks = $state<ProjectPromptBlock[]>([]);
  let tabs = $state<WorkspaceTab[]>([]);
  let activePath = $state<string | null>(null);
  let loadingPath = $state<string | null>(null);
  const validation = createValidationStore(
    () => tabs,
    () => activePath,
    () => workspace?.rootPath ?? null
  );
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
  let activePipelineProgress = $state<PipelineProgressEvent | null>(null);
  let activePipelineRunLabel = $state<string | null>(null);
  let pipelineAuthoringLoading = $state(false);
  let exportLoading = $state(false);
  let promptCreationLoading = $state(false);
  let modelPresets = $state<ModelPresetSummary[]>([]);
  let executionCredentialStatus = $state<ExecutionCredentialStatus>({
    source: 'missing',
    hasStoredKey: false
  });
  let executionCredentialDraft = $state('');
  let executionCredentialLoading = $state(false);
  let executionCredentialError = $state<string | null>(null);
  let executionHistoryRequestId = 0;
  let updateAvailable = $state(false);
  let updateVersion = $state<string | null>(null);
  let updateInstalling = $state(false);
  let updateChecking = $state(false);
  let updateNotice = $state<{ tone: 'info' | 'error'; text: string } | null>(null);

  async function checkForUpdate(): Promise<void> {
    updateChecking = true;
    updateNotice = null;
    try {
      const update = await check();
      if (update) {
        updateAvailable = true;
        updateVersion = update.version;
        updateNotice = { tone: 'info', text: `Update available: v${update.version}. Click the download arrow to install.` };
      } else {
        updateAvailable = false;
        updateVersion = null;
        updateNotice = { tone: 'info', text: 'You are on the latest version.' };
      }
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      updateNotice = { tone: 'error', text: `Update check failed: ${detail}` };
      console.error('[updater] check failed', err);
    } finally {
      updateChecking = false;
    }
  }

  async function handleInstallUpdate(): Promise<void> {
    if (updateInstalling) return;
    try {
      const update = await check();
      if (!update) {
        updateAvailable = false;
        updateVersion = null;
        updateNotice = { tone: 'info', text: 'You are on the latest version.' };
        return;
      }
      const confirmed = await ask(
        `Diamond Prompt Runner v${update.version} is ready to install. The app will restart after the update finishes. Install now?`,
        {
          title: 'Update available',
          kind: 'info',
          okLabel: 'Update now',
          cancelLabel: 'Later'
        }
      );
      if (!confirmed) {
        updateNotice = { tone: 'info', text: `Update v${update.version} deferred. Click the arrow when you're ready.` };
        return;
      }
      updateInstalling = true;
      updateNotice = { tone: 'info', text: 'Downloading update…' };
      await update.downloadAndInstall();
      updateNotice = { tone: 'info', text: 'Update installed. Relaunching…' };
      await relaunch();
    } catch (err) {
      const detail = err instanceof Error ? err.message : String(err);
      updateNotice = { tone: 'error', text: `Update failed: ${detail}` };
      console.error('[updater] install failed', err);
      updateInstalling = false;
    }
  }

  function summarizeProjectRunHistory(entries: ProjectRunHistoryEntry[]): ProjectUsageSummary {
    return entries.reduce(
      (summary, entry) => {
        summary.totalRuns += 1;

        if (entry.status === 'success') {
          summary.successfulRuns += 1;
        } else {
          summary.failedRuns += 1;
        }

        summary.totalPromptTokens += entry.usage.promptTokens ?? 0;
        summary.totalCompletionTokens += entry.usage.completionTokens ?? 0;
        summary.totalTokens += entry.usage.totalTokens ?? 0;
        summary.totalCost += entry.usage.cost ?? 0;
        summary.totalOutputWords += entry.usage.outputWordCount ?? 0;
        summary.totalRetries += entry.usage.retryCount ?? 0;
        return summary;
      },
      {
        totalRuns: 0,
        successfulRuns: 0,
        failedRuns: 0,
        totalPromptTokens: 0,
        totalCompletionTokens: 0,
        totalTokens: 0,
        totalCost: 0,
        totalOutputWords: 0,
        totalRetries: 0
      }
    );
  }

  function describePipelinePayload(payload?: Record<string, string>): string | null {
    if (!payload) {
      return null;
    }

    const entries = Object.entries(payload).filter(([, value]) => value.trim().length > 0);
    if (entries.length === 0) {
      return null;
    }

    if (entries.length === 1) {
      const [name, value] = entries[0];
      return `${name} ${value}`;
    }

    return entries.map(([name, value]) => `${name}=${value}`).join(', ');
  }

  function formatPipelineRunLabel(pipelineName: string, payload?: Record<string, string>): string {
    const payloadLabel = describePipelinePayload(payload);
    return payloadLabel ? `${pipelineName} · ${payloadLabel}` : pipelineName;
  }

  function serializeVariables(variables: Record<string, string>, scope: 'global' | 'workspace'): string {
    const header =
      scope === 'global'
        ? '# App-level global variables\n# Saved outside the project workspace.\n'
        : '# Workspace variables\n# Saved with the current project.\n';

    const entries = Object.entries(variables)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, value]) => `${name}: ${JSON.stringify(value)}`)
      .join('\n');

    return entries ? `${header}\n${entries}\n` : `${header}\n`;
  }

  function parseVariables(content: string): Record<string, string> {
    const next: Record<string, string> = {};

    for (const rawLine of content.split(/\r?\n/)) {
      const line = rawLine.trim();
      if (!line || line.startsWith('#')) {
        continue;
      }

      const separator = line.indexOf(':');
      if (separator <= 0) {
        throw new Error(`Invalid variable entry: ${line}`);
      }

      const name = line.slice(0, separator).trim();
      const rawValue = line.slice(separator + 1).trim();

      if (!name) {
        throw new Error(`Variable name is required: ${line}`);
      }

      if (!rawValue) {
        next[name] = '';
        continue;
      }

      if (rawValue.startsWith('"')) {
        const parsed = JSON.parse(rawValue);
        next[name] = typeof parsed === 'string' ? parsed : String(parsed);
        continue;
      }

      next[name] = rawValue;
    }

    return next;
  }

  function clearFeedback(): void {
    feedback = null;
  }

  function buildVariablesMetadata(
    path: string,
    name: string,
    content: string,
    details: { label: string; value: string }[]
  ): AssetMetadata {
    return {
      kind: 'yaml',
      path,
      name,
      sizeBytes: content.length,
      modifiedAt: new Date().toISOString(),
      details
    };
  }

  function buildGlobalVariablesAsset(): AssetContent {
    const content = serializeVariables(globalVariables, 'global');

    return {
      path: GLOBAL_VARIABLES_PATH,
      kind: 'yaml',
      view: 'text',
      content,
      isEditable: true,
      metadata: buildVariablesMetadata(GLOBAL_VARIABLES_PATH, 'global-variables.yaml', content, [
        { label: 'Scope', value: 'App-level' },
        { label: 'Storage', value: 'App data' },
        { label: 'Variables', value: String(Object.keys(globalVariables).length) }
      ]),
      parsedJson: null
    };
  }

  function buildWorkspaceVariablesAsset(): AssetContent {
    const variables = workspace?.variables ?? {};
    const content = serializeVariables(variables, 'workspace');

    return {
      path: WORKSPACE_VARIABLES_PATH,
      kind: 'yaml',
      view: 'text',
      content,
      isEditable: true,
      metadata: buildVariablesMetadata(WORKSPACE_VARIABLES_PATH, 'workspace-variables.yaml', content, [
        { label: 'Scope', value: 'Project-backed' },
        { label: 'Stored At', value: 'variables/workspace-variables.yaml' },
        { label: 'Variables', value: String(Object.keys(variables).length) }
      ]),
      parsedJson: null
    };
  }

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

      // Fire-and-forget update check on startup
      void checkForUpdate();

      await onPipelineProgress((event) => {
        const previous = activePipelineProgress;

        if (
          previous &&
          previous.pipelineId === event.pipelineId &&
          event.completedBlocks > previous.completedBlocks
        ) {
          pipelineActivityStore.push(
            'success',
            `Completed ${previous.currentBlockName}`,
            `${event.completedBlocks}/${event.totalBlocks} · ${activePipelineRunLabel ?? 'Pipeline run'}`,
            event.pipelineId
          );
        }

        pipelineActivityStore.push(
          'info',
          `Starting ${event.currentBlockName}`,
          `${event.completedBlocks + 1}/${event.totalBlocks} · ${activePipelineRunLabel ?? 'Pipeline run'}`,
          event.pipelineId
        );

        activePipelineProgress = event;
      });
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : typeof error === 'string' ? error : 'Failed to load app state.';
    }
  });

  async function enterWorkspace(summary: ProjectSummary): Promise<void> {
    workspace = summary;
    const [nodes, pipelines, promptBlocks, runHistory, presets] = await Promise.all([
      listProjectAssets(summary.rootPath),
      listProjectPipelines(summary.rootPath),
      listProjectPromptBlocks(summary.rootPath),
      listProjectRunHistory(summary.rootPath),
      listModelPresets(summary.rootPath)
    ]);
    assetNodes = nodes;
    projectPipelines = pipelines;
    projectPromptBlocks = promptBlocks;
    projectRunHistory = runHistory;
    projectUsageSummary = summarizeProjectRunHistory(runHistory);
    modelPresets = presets;
    tabs = [];
    activePath = null;
    loadingPath = null;
    validation.reset();
    executionResult = null;
    executionLoading = false;
    executionHistory = [];
    executionHistoryLoading = false;
    projectRunHistoryLoading = false;
    pipelineExecutionResult = null;
    activePipelineProgress = null;
    pipelineActivityStore.reset();
    pipelineExecutionLoading = false;
    pipelineAuthoringLoading = false;
    exportLoading = false;
    mode = 'workspace';
  }

  function handleCloseProject(): void {
    mode = 'browser';
    workspace = null;
    errorMessage = null;
  }

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
      let asset: AssetContent;

      if (path === GLOBAL_VARIABLES_PATH) {
        asset = buildGlobalVariablesAsset();
      } else if (path === WORKSPACE_VARIABLES_PATH) {
        asset = buildWorkspaceVariablesAsset();
      } else {
        asset = await readProjectAsset(workspace.rootPath, path);
      }

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

  function openAssetFromContent(asset: AssetContent, title?: string): void {
    const nextTab: WorkspaceTab = {
      ...asset,
      title: title ?? asset.metadata.name ?? asset.path.split('/').pop() ?? asset.path,
      savedContent: asset.content,
      draftContent: asset.content,
      isSaving: false
    };

    const existing = tabs.find((tab) => tab.path === asset.path);
    tabs = existing
      ? tabs.map((tab) => (tab.path === asset.path ? nextTab : tab))
      : [...tabs, nextTab];
    activePath = asset.path;
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

  function updateTab(path: string, mutate: (tab: WorkspaceTab) => WorkspaceTab): void {
    tabs = tabs.map((tab) => (tab.path === path ? mutate(tab) : tab));
  }

  function handleDraftChange(path: string, content: string): void {
    updateTab(path, (tab) => ({
      ...tab,
      draftContent: content
    }));
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
      let asset: AssetContent;

      if (path === GLOBAL_VARIABLES_PATH) {
        globalVariables = await setGlobalVariables(parseVariables(tab.draftContent));
        asset = buildGlobalVariablesAsset();
      } else if (path === WORKSPACE_VARIABLES_PATH) {
        workspace = await setProjectVariables(workspace.rootPath, parseVariables(tab.draftContent));
        assetNodes = await listProjectAssets(workspace.rootPath);
        asset = buildWorkspaceVariablesAsset();
      } else {
        asset = await writeProjectAsset(workspace.rootPath, path, tab.draftContent);
      }

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
      let asset: AssetContent;

      if (path === GLOBAL_VARIABLES_PATH) {
        asset = buildGlobalVariablesAsset();
      } else if (path === WORKSPACE_VARIABLES_PATH) {
        asset = buildWorkspaceVariablesAsset();
      } else {
        asset = await readProjectAsset(workspace.rootPath, path);
      }

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
      void refreshProjectRunHistory(workspace.rootPath);
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
        outputTarget: 'run_artifact',
        documentPath: null,
        variables: {},
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

  async function handleRunPipeline(
    pipelineId: string,
    payload?: Record<string, string>,
    resumeFromBlockId?: string,
    selectedBlockIds?: string[]
  ): Promise<void> {
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
    const runLabel = formatPipelineRunLabel(pipeline.name, payload);
    activePipelineRunLabel = runLabel;
    pipelineActivityStore.push(
      'info',
      resumeFromBlockId ? `Resuming ${runLabel}` : `Started ${runLabel}`,
      selectedBlockIds?.length
        ? `${selectedBlockIds.length} selected blocks`
        : `${pipeline.blocks.length} blocks queued`,
      pipeline.pipelineId
    );

    try {
      const result = await executePipeline(
        workspace.rootPath,
        pipelineId,
        payload,
        resumeFromBlockId,
        selectedBlockIds
      );
      
      let mergedResult = result;
      if (resumeFromBlockId && pipelineExecutionResult) {
        mergedResult = {
            ...result,
            startedAt: pipelineExecutionResult.startedAt,
            steps: [...pipelineExecutionResult.steps, ...result.steps]
        };
      }
      pipelineExecutionResult = mergedResult;

      if (activePipelineProgress?.pipelineId === pipeline.pipelineId) {
        pipelineActivityStore.push(
          'success',
          `Completed ${activePipelineProgress.currentBlockName}`,
          `${activePipelineProgress.totalBlocks}/${activePipelineProgress.totalBlocks} · ${runLabel}`,
          pipeline.pipelineId
        );
      }

      pipelineActivityStore.push(
        result.status === 'success' ? 'success' : 'error',
        result.status === 'success' ? `Finished ${runLabel}` : `Failed ${runLabel}`,
        result.error ?? `${result.steps.length} steps recorded`,
        pipeline.pipelineId
      );

      void refreshProjectRunHistory(workspace.rootPath);
    } catch (error) {
      errorMessage = typeof error === 'string' ? error : 'Pipeline failed to execute.';
      pipelineActivityStore.push(
        'error',
        `Failed ${runLabel}`,
        typeof error === 'string' ? error : 'Pipeline failed to execute.',
        pipeline.pipelineId
      );
    } finally {
      pipelineExecutionLoading = false;
      activePipelineProgress = null;
      activePipelineRunLabel = null;
    }
  }

  async function handleCancelPipeline(): Promise<void> {
    try {
      await cancelPipeline();
    } catch (error) {
      errorMessage = typeof error === 'string' ? error : 'Failed to cancel pipeline.';
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
      const history = await listProjectRunHistory(rootPath);
      projectRunHistory = history;
      projectUsageSummary = summarizeProjectRunHistory(history);
    } catch {
      projectRunHistory = [];
      projectUsageSummary = summarizeProjectRunHistory([]);
    } finally {
      projectRunHistoryLoading = false;
    }
  }

  async function handleRenameProject(newName: string): Promise<void> {
    if (!workspace) return;
    workspace = await renameProject(workspace.rootPath, newName);
  }

  async function handleSetDefaultPreset(presetPath: string): Promise<void> {
    if (!workspace) return;
    workspace = await setDefaultModelPreset(workspace.rootPath, presetPath);
  }

  async function handleCreatePreset(filename: string, modelId: string): Promise<void> {
    if (!workspace) return;
    await createModelPreset(workspace.rootPath, filename, modelId);
    modelPresets = await listModelPresets(workspace.rootPath);
    assetNodes = await listProjectAssets(workspace.rootPath);
  }

  async function handleDeletePreset(presetPath: string): Promise<void> {
    if (!workspace) return;
    await deleteModelPreset(workspace.rootPath, presetPath);
    modelPresets = await listModelPresets(workspace.rootPath);
    // Close the tab if the deleted preset was open
    const existingTab = tabs.find((t) => t.path === presetPath);
    if (existingTab) {
      tabs = tabs.filter((t) => t.path !== presetPath);
      if (activePath === presetPath) {
        activePath = tabs.length > 0 ? tabs[tabs.length - 1].path : null;
      }
    }
    assetNodes = await listProjectAssets(workspace.rootPath);
  }

  async function handleSetBlockPreset(blockId: string, presetPath: string | null): Promise<void> {
    if (!workspace) return;
    workspace = await setBlockModelPreset(workspace.rootPath, blockId, presetPath);
    // Refresh prompt blocks list to see the update
    projectPromptBlocks = await listProjectPromptBlocks(workspace.rootPath);
  }

  async function handleSetBlockOutputTarget(blockId: string, target: string): Promise<void> {
    if (!workspace) return;
    workspace = await setBlockOutputTarget(workspace.rootPath, blockId, target);
    projectPromptBlocks = await listProjectPromptBlocks(workspace.rootPath);
  }

  async function handleSetBlockOutputFilename(blockId: string, filename: string | null): Promise<void> {
    if (!workspace) return;
    workspace = await setBlockOutputFilename(workspace.rootPath, blockId, filename);
    projectPromptBlocks = await listProjectPromptBlocks(workspace.rootPath);
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

  async function handleDeletePipeline(pipelineId: string): Promise<void> {
    if (!workspace) return;
    const updated = await deletePipeline(workspace.rootPath, pipelineId);
    workspace = updated;
    await refreshPipelineAuthoringState(workspace.rootPath);
  }

  async function handleDuplicatePipeline(pipelineId: string): Promise<SavedPipelineResult> {
    if (!workspace) throw new Error('No workspace open.');
    const result = await duplicatePipeline(workspace.rootPath, pipelineId);
    workspace = result.summary;
    recentProjects = await getRecentProjects();
    await refreshPipelineAuthoringState(result.summary.rootPath);
    return result;
  }

  async function handleRegisterPromptBlock(templateSource: string): Promise<ProjectPromptBlock> {
    if (!workspace) {
      throw new Error('No workspace open.');
    }

    const result = await registerPromptBlock(workspace.rootPath, templateSource);
    workspace = result.summary;
    recentProjects = await getRecentProjects();

    const [nodes, pipelines, blocks] = await Promise.all([
      listProjectAssets(result.summary.rootPath),
      listProjectPipelines(result.summary.rootPath),
      listProjectPromptBlocks(result.summary.rootPath)
    ]);

    assetNodes = nodes;
    projectPipelines = pipelines;
    projectPromptBlocks = blocks;

    const createdBlock = blocks.find((block) => block.templateSource === result.path);
    if (!createdBlock) {
      throw new Error('Prompt block was registered but could not be loaded back into the workspace.');
    }

    return createdBlock;
  }

  async function handleDeletePromptBlock(blockId: string): Promise<void> {
    if (!workspace) return;
    const updated = await deletePromptBlock(workspace.rootPath, blockId);
    workspace = updated;
    // Refresh both asset tree and block/pipeline lists since pipeline steps may have changed.
    const [nodes, pipelines, blocks] = await Promise.all([
      listProjectAssets(workspace.rootPath),
      listProjectPipelines(workspace.rootPath),
      listProjectPromptBlocks(workspace.rootPath)
    ]);
    assetNodes = nodes;
    projectPipelines = pipelines;
    projectPromptBlocks = blocks;
  }

  async function handleDeleteDocument(relativePath: string): Promise<void> {
    if (!workspace) return;
    await deleteDocument(workspace.rootPath, relativePath);
    // Close any open tab pointing to the deleted file before refreshing the tree.
    tabs = tabs.filter((tab) => tab.path !== relativePath);
    if (activePath === relativePath) activePath = tabs[0]?.path ?? null;
    assetNodes = await listProjectAssets(workspace.rootPath);
  }

  async function handleTrashPrompt(relativePath: string): Promise<void> {
    if (!workspace) return;
    const updated = await trashPrompt(workspace.rootPath, relativePath);
    workspace = updated;
    tabs = tabs.filter((tab) => tab.path !== relativePath);
    if (activePath === relativePath) activePath = tabs[0]?.path ?? null;
    recentProjects = await getRecentProjects();
    const [nodes, pipelines, blocks] = await Promise.all([
      listProjectAssets(workspace.rootPath),
      listProjectPipelines(workspace.rootPath),
      listProjectPromptBlocks(workspace.rootPath)
    ]);
    assetNodes = nodes;
    projectPipelines = pipelines;
    projectPromptBlocks = blocks;
  }

  async function handleRenameDocument(relativePath: string, newName: string): Promise<void> {
    if (!workspace) return;
    const newPath = await renameDocument(workspace.rootPath, relativePath, newName);
    // Close the stale tab so the renamed file opens fresh on next selection.
    tabs = tabs.filter((tab) => tab.path !== relativePath);
    if (activePath === relativePath) activePath = tabs[0]?.path ?? null;
    assetNodes = await listProjectAssets(workspace.rootPath);
    // Open the renamed file automatically so the author stays in context.
    await openAssetPath(newPath);
  }

  async function handleDeleteRun(runPath: string): Promise<void> {
    if (!workspace) return;
    // Optimistic removal for instant feedback.
    projectRunHistory = projectRunHistory.filter((item) => item.runPath !== runPath);
    projectUsageSummary = summarizeProjectRunHistory(projectRunHistory);
    try {
      await deleteRun(workspace.rootPath, runPath);
    } catch (error) {
      // On failure, refetch the full history to restore consistent state.
      errorMessage = error instanceof Error ? error.message : 'Failed to delete run artifact.';
      const history = await listProjectRunHistory(workspace.rootPath);
      projectRunHistory = history;
      projectUsageSummary = summarizeProjectRunHistory(history);
    }
  }

  async function handleConvertAsset(path: string): Promise<void> {
    if (!workspace) {
      return;
    }

    const tab = tabs.find((candidate) => candidate.path === path);
    if (tab && tab.isEditable && tab.draftContent !== tab.savedContent) {
      errorMessage = `Save changes to ${tab.title} before converting. Conversion uses the saved file on disk.`;
      return;
    }

    errorMessage = null;
    feedback = null;

    try {
      const result: AssetConversionResult = await convertProjectAsset(workspace.rootPath, path);
      assetNodes = await listProjectAssets(workspace.rootPath);
      openAssetFromContent(result.asset);
      feedback = {
        kind: 'notice',
        title: `Created ${result.targetPath}`,
        detail: result.assessment
      };
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : `Failed to convert ${path}.`;
    }
  }

  async function handleAuditAsset(path: string): Promise<void> {
    if (!workspace) {
      return;
    }

    const tab = tabs.find((candidate) => candidate.path === path);
    if (tab && tab.isEditable && tab.draftContent !== tab.savedContent) {
      errorMessage = `Save changes to ${tab.title} before auditing. The audit uses the saved file on disk.`;
      feedback = null;
      return;
    }

    errorMessage = null;
    feedback = null;

    try {
      const result: AssetConversionAuditResult = await auditProjectAsset(workspace.rootPath, path);
      feedback = {
        kind: 'audit',
        result
      };
    } catch (error) {
      feedback = null;
      errorMessage = error instanceof Error ? error.message : `Failed to audit ${path}.`;
    }
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
    executionCredentialError = null;
    errorMessage = null;

    try {
      executionCredentialStatus = await saveExecutionApiKey(apiKey);
      executionCredentialDraft = '';
    } catch (error) {
      executionCredentialError = error instanceof Error ? error.message : 'Failed to save execution API key.';
    } finally {
      executionCredentialLoading = false;
    }
  }

  async function handleClearExecutionCredential(): Promise<void> {
    if (executionCredentialLoading) {
      return;
    }

    executionCredentialLoading = true;
    executionCredentialError = null;

    try {
      executionCredentialStatus = await clearExecutionApiKey();
    } catch (error) {
      executionCredentialError = error instanceof Error ? error.message : 'Failed to clear execution API key.';
    } finally {
      executionCredentialLoading = false;
    }
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
    credentialState={executionCredentialStatus}
    credentialDraft={executionCredentialDraft}
    credentialLoading={executionCredentialLoading}
    credentialError={executionCredentialError}
    onCredentialInput={(value) => (executionCredentialDraft = value)}
    onSaveCredential={handleSaveExecutionCredential}
    onClearCredential={handleClearExecutionCredential}
    {updateAvailable}
    {updateVersion}
    {updateInstalling}
    {updateChecking}
    {updateNotice}
    onInstallUpdate={handleInstallUpdate}
    onCheckForUpdate={checkForUpdate}
  />
{:else}
  {#if errorMessage}
    <p class="error-banner">{errorMessage}</p>
  {/if}

  {#if feedback}
    <section class:status-panel={feedback.kind === 'notice'} class:audit-panel={feedback.kind === 'audit'}>
      <div class="feedback-head">
        <div>
          {#if feedback.kind === 'audit'}
            <p class="feedback-label">Conversion Audit</p>
            <h2>{feedback.result.status}</h2>
          {:else}
            <p class="feedback-label">Update</p>
            <h2>{feedback.title}</h2>
          {/if}
        </div>
        <button type="button" class="feedback-dismiss" onclick={clearFeedback} aria-label="Dismiss message">×</button>
      </div>

      {#if feedback.kind === 'audit'}
        <p class="feedback-text">{feedback.result.assessment}</p>
        <p class="feedback-meta">Target: {feedback.result.targetPath}</p>
        {#if feedback.result.warnings.length > 0}
          <ul class="feedback-list">
            {#each feedback.result.warnings as warning (warning)}
              <li>{warning}</li>
            {/each}
          </ul>
        {:else}
          <p class="feedback-text">No warnings.</p>
        {/if}
      {:else}
        <p class="feedback-text">{feedback.detail}</p>
      {/if}
    </section>
  {/if}

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
    {activePipelineProgress}
    {pipelineAuthoringLoading}
    {projectRunHistory}
    {projectRunHistoryLoading}
    {projectUsageSummary}
    {globalVariables}
    {modelPresets}
    onSelectAsset={handleSelectAsset}
    onOpenPath={openAssetPath}
    onSelectTab={(path) => { activePath = path; errorMessage = null; clearFeedback(); }}
    onCloseTab={(path) => { tabs = tabs.filter(t => t.path !== path); if (activePath === path) activePath = tabs[0]?.path ?? null; }}
    onDraftChange={handleDraftChange}
    onSaveTab={handleSaveTab}
    onReloadTab={handleReloadTab}
    onRunTab={handleExecuteTab}
    onRunPipeline={handleRunPipeline}
    onCancelPipeline={handleCancelPipeline}
    onCreatePipeline={handleCreatePipeline}
    onUpdatePipeline={handleUpdatePipeline}
    onExportAssets={handleExportAssets}
    {exportLoading}
    onCreatePrompt={handleCreatePrompt}
    {promptCreationLoading}
    credentialState={executionCredentialStatus}
    historyItems={executionHistory}
    historyLoading={executionHistoryLoading}
    onOpenRunPath={handleOpenRunPath}
    onCloseProject={handleCloseProject}
    validationResult={validation.result}
    validationLoading={validation.loading}
    {executionResult}
    {executionLoading}
    onRenameProject={handleRenameProject}
    onSetDefaultPreset={handleSetDefaultPreset}
    onCreatePreset={handleCreatePreset}
    onDeletePreset={handleDeletePreset}
    onSetBlockPreset={handleSetBlockPreset}
    onSetBlockOutputTarget={handleSetBlockOutputTarget}
    onSetBlockOutputFilename={handleSetBlockOutputFilename}
    onDeletePipeline={handleDeletePipeline}
    onDuplicatePipeline={handleDuplicatePipeline}
    onRegisterPromptBlock={handleRegisterPromptBlock}
    onDeletePromptBlock={handleDeletePromptBlock}
    onDeleteRun={handleDeleteRun}
    onDeleteDocument={handleDeleteDocument}
    onDeletePrompt={handleTrashPrompt}
    onRenameDocument={handleRenameDocument}
    onAuditAsset={handleAuditAsset}
    onConvertAsset={handleConvertAsset}
    onOpenHelpFile={handleSelectAsset}
    {updateAvailable}
    {updateVersion}
    {updateInstalling}
    {updateChecking}
    onInstallUpdate={handleInstallUpdate}
    onCheckForUpdate={checkForUpdate}
  />
{/if}

<style>
  .status-panel,
  .audit-panel {
    margin: 0 0 0.75rem;
    padding: 0.85rem 0.95rem;
    border-radius: 14px;
    color: var(--text);
  }

  .status-panel {
    border: 1px solid rgba(93, 167, 214, 0.35);
    background: rgba(33, 70, 92, 0.35);
  }

  .audit-panel {
    border: 1px solid rgba(208, 176, 88, 0.35);
    background: rgba(78, 62, 26, 0.3);
  }

  .feedback-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }

  .feedback-head h2 {
    margin: 0;
    font-size: 1rem;
    text-transform: capitalize;
  }

  .feedback-label {
    margin: 0 0 0.2rem;
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-dim);
  }

  .feedback-dismiss {
    flex: 0 0 auto;
    min-width: 2rem;
    min-height: 2rem;
    border-radius: 999px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: transparent;
    color: var(--text-dim);
    font-size: 1.1rem;
    line-height: 1;
  }

  .feedback-dismiss:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.06);
  }

  .feedback-text,
  .feedback-meta {
    margin: 0.6rem 0 0;
  }

  .feedback-meta {
    color: var(--text-dim);
    font-size: 0.92rem;
  }

  .feedback-list {
    margin: 0.7rem 0 0;
    padding-left: 1.2rem;
  }

  .feedback-list li + li {
    margin-top: 0.35rem;
  }
</style>
