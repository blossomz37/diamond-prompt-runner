<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        WorkspaceShell.svelte
  // Description: IDE shell layout with collapsible sidebar sections and center-pane editing
  // Version:     2.0.0
  // Created:     2026-04-03
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import AssetViewer from '$lib/components/AssetViewer.svelte';
  import InspectorPanel from '$lib/components/InspectorPanel.svelte';
  import PipelineEditorTab from '$lib/components/PipelineEditorTab.svelte';
  import PromptBlockEditorTab from '$lib/components/PromptBlockEditorTab.svelte';
  import SidebarDocuments from '$lib/components/SidebarDocuments.svelte';
  import SidebarExports from '$lib/components/SidebarExports.svelte';
  import SidebarHelp from '$lib/components/SidebarHelp.svelte';
  import SidebarModels from '$lib/components/SidebarModels.svelte';
  import SidebarPipelines from '$lib/components/SidebarPipelines.svelte';
  import SidebarPromptBlocks from '$lib/components/SidebarPromptBlocks.svelte';
  import SidebarPrompts from '$lib/components/SidebarPrompts.svelte';
  import SidebarRuns from '$lib/components/SidebarRuns.svelte';
  import SidebarSettings from '$lib/components/SidebarSettings.svelte';
  import SidebarGlobalVariables from '$lib/components/SidebarGlobalVariables.svelte';
  import SidebarWorkspaceVariables from '$lib/components/SidebarWorkspaceVariables.svelte';
  import { pipelineActivityStore } from '$lib/stores/pipelineActivity.svelte';
  import type {
    ExportBundleResult,
    ExecutionCredentialStatus,
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
    SavedPipelineResult,
    TemplateValidationResult,
    WorkspaceTab,
    PipelineProgressEvent
  } from '$lib/types/project';

  export interface Props {
    summary: ProjectSummary;
    nodes: ProjectAssetNode[];
    tabs: WorkspaceTab[];
    activePath: string | null;
    loadingPath: string | null;
    errorMessage: string | null;
    pipelines: ProjectPipeline[];
    promptBlocks: ProjectPromptBlock[];
    pipelineExecution: PipelineExecutionResult | null;
    pipelineLoading: boolean;
    activePipelineProgress: PipelineProgressEvent | null;
    pipelineAuthoringLoading: boolean;
    projectRunHistory: ProjectRunHistoryEntry[];
    projectRunHistoryLoading: boolean;
    projectUsageSummary: ProjectUsageSummary | null;
    globalVariables: Record<string, string>;
    onSelectAsset: (node: ProjectAssetNode) => void | Promise<void>;
    onOpenPath: (path: string, title?: string) => void | Promise<void>;
    onSelectTab: (path: string) => void;
    onCloseTab: (path: string) => void;
    onDraftChange: (path: string, content: string) => void;
    onSaveTab: (path: string) => void | Promise<void>;
    onReloadTab: (path: string) => void | Promise<void>;
    onRunTab: (path: string) => void | Promise<void>;
    onRunPipeline: (
      pipelineId: string,
      payload?: Record<string, string>,
      resumeFromBlockId?: string,
      selectedBlockIds?: string[]
    ) => void | Promise<void>;
    onCancelPipeline: () => void | Promise<void>;
    onCreatePipeline: (name: string, orderedBlockIds: string[]) => Promise<SavedPipelineResult>;
    onUpdatePipeline: (
      pipelineId: string,
      name: string,
      orderedBlockIds: string[]
    ) => Promise<SavedPipelineResult>;
    onExportAssets: (bundleName: string, relativePaths: string[]) => Promise<ExportBundleResult>;
    exportLoading: boolean;
    onCreatePrompt: (name: string) => void | Promise<void>;
    promptCreationLoading: boolean;
    historyItems: PromptRunHistoryEntry[];
    historyLoading: boolean;
    onOpenRunPath: (path: string) => void | Promise<void>;
    onCloseProject: () => void;
    validationResult: TemplateValidationResult | null;
    validationLoading: boolean;
    executionResult: PromptExecutionResult | null;
    executionLoading: boolean;
    modelPresets: ModelPresetSummary[];
    onRenameProject: (newName: string) => Promise<void>;
    onSetDefaultPreset: (presetPath: string) => Promise<void>;
    onCreatePreset: (filename: string, modelId: string) => Promise<void>;
    onDeletePreset: (presetPath: string) => Promise<void>;
    onSetBlockPreset: (blockId: string, presetPath: string | null) => Promise<void>;
    onSetBlockOutputTarget: (blockId: string, target: string) => Promise<void>;
    onSetBlockOutputFilename: (blockId: string, filename: string | null) => Promise<void>;
    onDeletePipeline: (pipelineId: string) => Promise<void>;
    onDuplicatePipeline: (pipelineId: string) => Promise<SavedPipelineResult>;
    onRegisterPromptBlock: (templateSource: string) => Promise<ProjectPromptBlock>;
    onDeletePromptBlock: (blockId: string) => Promise<void>;
    onDeleteRun: (runPath: string) => Promise<void>;
    onDeleteDocument: (relativePath: string) => Promise<void>;
    onDeletePrompt: (relativePath: string) => Promise<void>;
    onRenameDocument: (relativePath: string, newName: string) => Promise<void>;
    onAuditAsset: (path: string) => Promise<void>;
    onConvertAsset: (path: string) => Promise<void>;
    credentialState: ExecutionCredentialStatus | null;
    onOpenHelpFile: (node: ProjectAssetNode) => void;
    updateAvailable: boolean;
    updateVersion: string | null;
    updateInstalling: boolean;
    onInstallUpdate: () => void | Promise<void>;
  }

  let {
    summary,
    nodes,
    tabs,
    activePath,
    loadingPath,
    errorMessage,
    pipelines,
    promptBlocks,
    pipelineExecution,
    pipelineLoading,
    activePipelineProgress,
    pipelineAuthoringLoading,
    projectRunHistory,
    projectRunHistoryLoading,
    projectUsageSummary,
    globalVariables,
    onSelectAsset,
    onOpenPath,
    onSelectTab,
    onCloseTab,
    onDraftChange,
    onSaveTab,
    onReloadTab,
    onRunTab,
    onRunPipeline,
    onCancelPipeline,
    onCreatePipeline,
    onUpdatePipeline,
    onExportAssets,
    exportLoading,
    onCreatePrompt,
    promptCreationLoading,
    credentialState,
    historyItems,
    historyLoading,
    onOpenRunPath,
    onCloseProject,
    validationResult,
    validationLoading,
    executionResult,
    executionLoading,
    modelPresets,
    onRenameProject,
    onSetDefaultPreset,
    onCreatePreset,
    onDeletePreset,
    onSetBlockPreset,
    onSetBlockOutputTarget,
    onSetBlockOutputFilename,
    onDeletePipeline,
    onDuplicatePipeline,
    onRegisterPromptBlock,
    onDeletePromptBlock,
    onDeleteRun,
    onDeleteDocument,
    onDeletePrompt,
    onRenameDocument,
    onAuditAsset,
    onConvertAsset,
    onOpenHelpFile,
    updateAvailable,
    updateVersion,
    updateInstalling,
    onInstallUpdate
  }: Props = $props();

  const activeTab = $derived(tabs.find((tab) => tab.path === activePath) ?? null);
  const activeExecution = $derived(
    activeTab && executionResult?.path === activeTab.path ? executionResult : null
  );

  const pipelineStatusLabel = $derived.by(() => {
    if (activePipelineProgress) {
      return `Running ${activePipelineProgress.completedBlocks}/${activePipelineProgress.totalBlocks}`;
    }

    if (pipelineExecution) {
      return pipelineExecution.status === 'success' ? 'Last pipeline succeeded' : 'Last pipeline failed';
    }

    return 'No pipeline runs yet';
  });

  const pipelineStatusDetail = $derived.by(() => {
    if (activePipelineProgress) {
      return `${activePipelineProgress.currentBlockName} · ${activePipelineProgress.status}`;
    }

    if (pipelineExecution) {
      return `${pipelineExecution.pipelineName} · ${pipelineExecution.steps.length} steps`;
    }

    return 'Use the Pipelines section to run a workflow.';
  });

  const visiblePipelineActivity = $derived(pipelineActivityStore.items.slice(0, 3));

  const pipelineStatusSegments = $derived.by(() => {
    const segments: string[] = [pipelineStatusLabel, pipelineStatusDetail];

    for (const item of visiblePipelineActivity) {
      segments.push(item.message);
      if (item.detail) {
        segments.push(item.detail);
      }
    }

    return segments.filter((segment) => segment.trim().length > 0);
  });

  // Sidebar section collapse states (workflow order)
  let modelsOpen = $state(false);
  let globalVarsOpen = $state(false);
  let workspaceVarsOpen = $state(false);
  let promptsOpen = $state(false);
  let promptBlocksOpen = $state(false);
  let pipelinesOpen = $state(false);
  let runsOpen = $state(false);
  let documentsOpen = $state(false);
  let exportsOpen = $state(false);
  let settingsOpen = $state(false);
  let helpOpen = $state(false);

  // Inspector panel
  let inspectorOpen = $state(true);

  // Pipeline editor virtual tab state
  let pipelineEditorActive = $state(false);
  let pipelineEditorTarget = $state<ProjectPipeline | null>(null);

  const pipelineEditorTitle = $derived(
    pipelineEditorTarget ? `Pipeline: ${pipelineEditorTarget.name}` : 'New Pipeline'
  );

  // Keep pipelineEditorTarget in sync when pipelines list refreshes
  $effect(() => {
    if (pipelineEditorTarget) {
      const fresh = pipelines.find((p) => p.pipelineId === pipelineEditorTarget!.pipelineId);
      if (fresh) pipelineEditorTarget = fresh;
      else { pipelineEditorActive = false; pipelineEditorTarget = null; }
    }
  });

  function openPipelineEditor(pipeline: ProjectPipeline | null): void {
    pipelineEditorTarget = pipeline;
    pipelineEditorActive = true;
    blockEditorActive = false;
  }

  function closePipelineEditor(): void {
    pipelineEditorActive = false;
    pipelineEditorTarget = null;
  }

  // Block editor virtual tab state
  let blockEditorActive = $state(false);
  let blockEditorTarget = $state<ProjectPromptBlock | null>(null);

  const blockEditorTitle = $derived(
    blockEditorTarget ? `Block: ${blockEditorTarget.name}` : 'Prompt Block'
  );

  // Keep blockEditorTarget in sync when promptBlocks list refreshes
  $effect(() => {
    if (blockEditorTarget) {
      const fresh = promptBlocks.find((b) => b.blockId === blockEditorTarget!.blockId);
      if (fresh) blockEditorTarget = fresh;
      else { blockEditorActive = false; blockEditorTarget = null; }
    }
  });

  function openBlockEditor(block: ProjectPromptBlock): void {
    blockEditorTarget = block;
    blockEditorActive = true;
    pipelineEditorActive = false;
  }

  function closeBlockEditor(): void {
    blockEditorActive = false;
    blockEditorTarget = null;
  }

  async function handlePipelineSave(
    name: string,
    orderedBlockIds: string[],
    existingPipelineId: string | null
  ): Promise<SavedPipelineResult> {
    let result: SavedPipelineResult;
    if (existingPipelineId) {
      result = await onUpdatePipeline(existingPipelineId, name, orderedBlockIds);
    } else {
      result = await onCreatePipeline(name, orderedBlockIds);
    }
    closePipelineEditor();
    return result;
  }

  async function handlePipelineDuplicate(pipelineId: string): Promise<void> {
    const result = await onDuplicatePipeline(pipelineId);
    // Open the newly created duplicate in the editor
    const fresh = pipelines.find((p) => p.pipelineId === result.pipelineId);
    if (fresh) openPipelineEditor(fresh);
  }
</script>

<section class="workspace-shell">
  <header class="topbar panel">
    <div class="topbar-left">
      <button type="button" class="back-btn" onclick={onCloseProject} aria-label="Back to projects">←</button>
      <div>
        <p class="eyebrow">Workspace</p>
        <h1>{summary.projectName}</h1>
      </div>
    </div>
    <div class="status-strip">
      <div class="status-cluster">
        <div class="pipeline-status-inline" class:failed={pipelineExecution?.status === 'failed'} class:running={activePipelineProgress !== null} aria-live="polite">
          {#if pipelineLoading}
            <span class="inline-spinner" aria-label="Pipeline running"></span>
          {/if}
          <span class="pipeline-inline-label">Pipeline Status</span>
          <div class="pipeline-inline-track">
            {#each pipelineStatusSegments as segment, index (`${index}-${segment}`)}
              {#if index > 0}
                <span class="pipeline-inline-separator" aria-hidden="true">|</span>
              {/if}
              <span class="pipeline-inline-segment">{segment}</span>
            {/each}
          </div>
        </div>
        {#if updateAvailable}
          <button
            type="button"
            class="pane-toggle update-badge"
            onclick={onInstallUpdate}
            disabled={updateInstalling}
            title={updateInstalling ? 'Installing update…' : `Update available: ${updateVersion}`}
            aria-label={updateInstalling ? 'Installing update' : `Update to ${updateVersion}`}
          >
            {#if updateInstalling}
              <span class="inline-spinner" aria-label="Installing"></span>
            {:else}
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path d="M8 2v8M5 7l3 3 3-3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M3 12h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            {/if}
          </button>
        {/if}
        <button
          type="button"
          class="pane-toggle"
          onclick={() => (inspectorOpen = !inspectorOpen)}
          aria-label={inspectorOpen ? 'Collapse inspector' : 'Expand inspector'}
        >
          Inspector {inspectorOpen ? '▾' : '▸'}
        </button>
      </div>
    </div>
  </header>

  {#if errorMessage}
    <p class="error-banner">{errorMessage}</p>
  {/if}

  <div class="shell-grid" class:inspector-closed={!inspectorOpen}>
    <aside class="sidebar panel">
      <!-- 1. Models -->
      <div class="sidebar-section" class:collapsed={!modelsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (modelsOpen = !modelsOpen)}>
          <span>Models</span>
          <span class="toggle">{modelsOpen ? '▾' : '▸'}</span>
        </button>
        {#if modelsOpen}
          <div class="sidebar-body">
            <SidebarModels
              {nodes}
              activePath={blockEditorActive || pipelineEditorActive ? null : activePath}
              defaultModelPreset={summary.defaultModelPreset}
              {onCreatePreset}
              onOpenModel={onOpenPath}
            />
          </div>
        {/if}
      </div>

      <!-- 2a. Global Variables -->
      <div class="sidebar-section" class:collapsed={!globalVarsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (globalVarsOpen = !globalVarsOpen)}>
          <span>Global Variables</span>
          <span class="toggle">{globalVarsOpen ? '▾' : '▸'}</span>
        </button>
        {#if globalVarsOpen}
          <div class="sidebar-body">
            <SidebarGlobalVariables
              activePath={blockEditorActive || pipelineEditorActive ? null : activePath}
              variableCount={Object.keys(globalVariables).length}
              onOpenGlobalVariables={onOpenPath}
            />
          </div>
        {/if}
      </div>

      <!-- 2b. Workspace Variables -->
      <div class="sidebar-section" class:collapsed={!workspaceVarsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (workspaceVarsOpen = !workspaceVarsOpen)}>
          <span>Workspace Variables</span>
          <span class="toggle">{workspaceVarsOpen ? '▾' : '▸'}</span>
        </button>
        {#if workspaceVarsOpen}
          <div class="sidebar-body">
            <SidebarWorkspaceVariables
              activePath={blockEditorActive || pipelineEditorActive ? null : activePath}
              variableCount={Object.keys(summary.variables).length}
              onOpenWorkspaceVariables={onOpenPath}
            />
          </div>
        {/if}
      </div>

      <!-- 3. Prompts -->
      <div class="sidebar-section" class:collapsed={!promptsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (promptsOpen = !promptsOpen)}>
          <span>Prompts</span>
          <span class="toggle">{promptsOpen ? '▾' : '▸'}</span>
        </button>
        {#if promptsOpen}
          <div class="sidebar-body">
            <SidebarPrompts
              {nodes}
              activePath={pipelineEditorActive ? null : activePath}
              onSelectPath={onSelectAsset}
              {onCreatePrompt}
              onDeletePrompt={onDeletePrompt}
              {promptCreationLoading}
            />
          </div>
        {/if}
      </div>

      <!-- 4. Prompt Blocks -->
      <div class="sidebar-section" class:collapsed={!promptBlocksOpen}>
        <button type="button" class="sidebar-header" onclick={() => (promptBlocksOpen = !promptBlocksOpen)}>
          <span>Prompt Blocks</span>
          <span class="toggle">{promptBlocksOpen ? '▾' : '▸'}</span>
        </button>
        {#if promptBlocksOpen}
          <div class="sidebar-body">
            <SidebarPromptBlocks
              {nodes}
              {promptBlocks}
              activeBlockId={blockEditorActive ? blockEditorTarget?.blockId ?? null : null}
              onSelectBlock={openBlockEditor}
              onRegisterPromptBlock={async (templateSource) => {
                const block = await onRegisterPromptBlock(templateSource);
                openBlockEditor(block);
              }}
            />
          </div>
        {/if}
      </div>

      <!-- 5. Pipelines -->
      <div class="sidebar-section" class:collapsed={!pipelinesOpen}>
        <button type="button" class="sidebar-header" onclick={() => (pipelinesOpen = !pipelinesOpen)}>
          <span>Pipelines</span>
          <span class="toggle">{pipelinesOpen ? '▾' : '▸'}</span>
        </button>
        {#if pipelinesOpen}
          <div class="sidebar-body">
            <SidebarPipelines
              {pipelines}
              {promptBlocks}
              activePipelineId={pipelineEditorActive ? pipelineEditorTarget?.pipelineId ?? null : null}
              {pipelineAuthoringLoading}
              onSelectPipeline={(pipeline) => openPipelineEditor(pipeline)}
              onNewPipeline={() => openPipelineEditor(null)}
            />
          </div>
        {/if}
      </div>

      <!-- 6. Runs -->
      <div class="sidebar-section" class:collapsed={!runsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (runsOpen = !runsOpen)}>
          <span>Runs</span>
          <span class="toggle">{runsOpen ? '▾' : '▸'}</span>
        </button>
        {#if runsOpen}
          <div class="sidebar-body">
            <SidebarRuns
              runHistory={projectRunHistory}
              runHistoryLoading={projectRunHistoryLoading}
              {onOpenRunPath}
              {onDeleteRun}
            />
          </div>
        {/if}
      </div>

      <!-- 7. Documents -->
      <div class="sidebar-section" class:collapsed={!documentsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (documentsOpen = !documentsOpen)}>
          <span>Documents</span>
          <span class="toggle">{documentsOpen ? '▾' : '▸'}</span>
        </button>
        {#if documentsOpen}
          <div class="sidebar-body">
            <SidebarDocuments
              {nodes}
              activePath={pipelineEditorActive ? null : activePath}
              onSelectPath={onSelectAsset}
              {onDeleteDocument}
              {onRenameDocument}
            />
          </div>
        {/if}
      </div>

      <!-- 8. Exports -->
      <div class="sidebar-section" class:collapsed={!exportsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (exportsOpen = !exportsOpen)}>
          <span>Exports</span>
          <span class="toggle">{exportsOpen ? '▾' : '▸'}</span>
        </button>
        {#if exportsOpen}
          <div class="sidebar-body">
            <SidebarExports
              openTabs={tabs}
              {activePath}
              {onExportAssets}
              {exportLoading}
            />
          </div>
        {/if}
      </div>

      <!-- 9. Settings -->
      <div class="sidebar-section" class:collapsed={!settingsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (settingsOpen = !settingsOpen)}>
          <span>Settings</span>
          <span class="toggle">{settingsOpen ? '▾' : '▸'}</span>
        </button>
        {#if settingsOpen}
          <div class="sidebar-body">
            <SidebarSettings
              {summary}
              {onRenameProject}
            />
          </div>
        {/if}
      </div>

      <!-- 10. Help -->
      <div class="sidebar-section" class:collapsed={!helpOpen}>
        <button type="button" class="sidebar-header" onclick={() => (helpOpen = !helpOpen)}>
          <span>Help</span>
          <span class="toggle">{helpOpen ? '▾' : '▸'}</span>
        </button>
        {#if helpOpen}
          <div class="sidebar-body">
            <SidebarHelp
              {nodes}
              activePath={blockEditorActive || pipelineEditorActive ? null : activePath}
              onSelectPath={onOpenHelpFile}
            />
          </div>
        {/if}
      </div>
    </aside>

    <main class="editor panel">
      <div class="tabs" data-testid="workspace-tabs">
        {#if tabs.length === 0 && !pipelineEditorActive && !blockEditorActive}
          <div class="empty-tab">No tabs open</div>
        {:else}
          {#each tabs as tab (tab.path)}
            <div class:active={tab.path === activePath && !pipelineEditorActive && !blockEditorActive} class="tab">
              <button type="button" onclick={() => { pipelineEditorActive = false; blockEditorActive = false; onSelectTab(tab.path); }}>
                {tab.title}{tab.draftContent !== tab.savedContent ? ' *' : ''}
              </button>
              <button
                class="close"
                type="button"
                aria-label={`Close ${tab.title}`}
                onclick={() => onCloseTab(tab.path)}
              >
                ×
              </button>
            </div>
          {/each}
          {#if pipelineEditorActive}
            <div class="tab active">
              <button type="button">{pipelineEditorTitle}</button>
              <button
                class="close"
                type="button"
                aria-label="Close pipeline editor"
                onclick={closePipelineEditor}
              >
                ×
              </button>
            </div>
          {/if}
          {#if blockEditorActive}
            <div class="tab active">
              <button type="button">{blockEditorTitle}</button>
              <button
                class="close"
                type="button"
                aria-label="Close block editor"
                onclick={closeBlockEditor}
              >
                ×
              </button>
            </div>
          {/if}
        {/if}
      </div>

      {#if loadingPath}
        <div class="loading">Loading {loadingPath}…</div>
      {/if}

      <div class="viewer-wrap">
        {#if blockEditorActive && blockEditorTarget}
          <PromptBlockEditorTab
            block={blockEditorTarget}
            {modelPresets}
            {onSetBlockPreset}
            {onSetBlockOutputTarget}
            {onSetBlockOutputFilename}
            {onDeletePromptBlock}
            onOpenTemplate={(path) => {
              blockEditorActive = false;
              onSelectAsset({ name: path.split('/').pop() ?? path, path, kind: 'tera', isDirectory: false, children: [] });
            }}
            onClose={closeBlockEditor}
          />
        {:else if pipelineEditorActive}
          <PipelineEditorTab
            existingPipeline={pipelineEditorTarget}
            {promptBlocks}
            loading={pipelineAuthoringLoading}
            pipelineExecution={pipelineExecution}
            {pipelineLoading}
            {activePipelineProgress}
            onSave={handlePipelineSave}
            onCancel={closePipelineEditor}
            onRunPipeline={onRunPipeline}
            {onCancelPipeline}
            {onDeletePipeline}
            onDuplicatePipeline={handlePipelineDuplicate}
            onExportPipeline={onExportAssets}
          />
        {:else}
          <AssetViewer
            tab={activeTab}
            onDraftChange={onDraftChange}
            onSave={onSaveTab}
            onReload={onReloadTab}
            onExecute={onRunTab}
            validation={validationResult}
            validationLoading={validationLoading}
            execution={activeExecution}
            executionLoading={executionLoading && activeTab?.kind === 'tera'}
            historyItems={historyItems}
            historyLoading={historyLoading}
            credentialState={credentialState ?? { source: 'missing', hasStoredKey: false }}
            {onOpenRunPath}
            defaultModelPreset={summary.defaultModelPreset}
            {onSetDefaultPreset}
            onDeletePreset={onDeletePreset}
            onAuditAsset={onAuditAsset}
            onConvertAsset={onConvertAsset}
          />
        {/if}
      </div>
    </main>

    {#if inspectorOpen}
      <aside class="inspector panel">
        <InspectorPanel
          summary={summary}
          tab={activeTab}
          metadata={activeTab?.metadata ?? null}
          usageSummary={projectUsageSummary}
          validation={validationResult}
          validationLoading={validationLoading}
        />
      </aside>
    {/if}
  </div>
</section>

<style>
  .workspace-shell {
    display: grid;
    grid-template-rows: auto auto 1fr;
    gap: 0.85rem;
    min-height: 100vh;
    padding: 0.85rem;
  }

  .panel {
    background: linear-gradient(180deg, rgba(15, 22, 38, 0.94), rgba(7, 11, 20, 0.96));
    border: 1px solid var(--panel-border);
    border-radius: 20px;
    box-shadow: var(--shadow);
    min-height: 0;
  }

  .topbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.95rem 1.1rem;
  }

  .topbar-left {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .back-btn {
    background: none;
    border: 1px solid var(--border, #444);
    border-radius: 4px;
    color: inherit;
    font-size: 1.1rem;
    padding: 0.2rem 0.5rem;
    cursor: pointer;
    line-height: 1;
  }

  .back-btn:hover {
    background: var(--hover-bg, var(--bg-hover));
  }

  h1 {
    margin: 0.2rem 0 0;
    font-size: 1.15rem;
  }

  .status-strip {
    display: flex;
    justify-content: flex-end;
    align-items: flex-start;
    min-width: min(42rem, 58vw);
  }

  .status-cluster {
    display: grid;
    justify-items: end;
    gap: 0.45rem;
    min-width: min(44rem, 58vw);
  }

  .pipeline-status-inline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: min(52rem, 58vw);
    min-height: 2.4rem;
    padding: 0.45rem 0.75rem;
    border-radius: 999px;
    background: var(--bg-ghost);
    border: 1px solid var(--border-faint);
    color: var(--text-dim);
    overflow: hidden;
  }

  .pipeline-inline-label {
    flex: 0 0 auto;
    font-size: 0.68rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--accent);
    font-weight: 700;
  }

  .pipeline-inline-track {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    min-width: 0;
    overflow-x: auto;
    overflow-y: hidden;
    white-space: nowrap;
    scrollbar-width: none;
  }

  .pipeline-inline-track::-webkit-scrollbar {
    display: none;
  }

  .pipeline-inline-segment {
    flex: 0 0 auto;
    font-size: 0.8rem;
    color: var(--text-soft);
  }

  .pipeline-inline-segment:first-of-type {
    color: var(--text);
    font-weight: 700;
  }

  .pipeline-inline-separator {
    flex: 0 0 auto;
    color: rgba(157, 180, 255, 0.38);
  }

  .inline-spinner {
    width: 0.85rem;
    height: 0.85rem;
    border-radius: 999px;
    border: 2px solid rgba(157, 180, 255, 0.2);
    border-top-color: rgba(157, 180, 255, 0.9);
    animation: shell-spin 0.85s linear infinite;
    flex-shrink: 0;
  }

  .pipeline-status-inline.running {
    border-color: rgba(139, 177, 255, 0.34);
    background: rgba(132, 173, 255, 0.1);
  }

  .pipeline-status-inline.failed {
    border-color: rgba(255, 141, 161, 0.2);
    background: rgba(255, 141, 161, 0.08);
  }

  .shell-grid {
    display: grid;
    grid-template-columns: minmax(15rem, 18rem) minmax(0, 1fr) minmax(17rem, 21rem);
    gap: 0.85rem;
    min-height: 0;
    flex: 1;
  }

  .shell-grid.inspector-closed {
    grid-template-columns: minmax(15rem, 18rem) minmax(0, 1fr);
  }

  .sidebar,
  .inspector,
  .editor {
    min-height: 0;
  }

  /* ── Sidebar collapsible sections ── */

  .sidebar {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    padding: 0;
  }

  .sidebar-section {
    border-bottom: 1px solid var(--panel-border);
  }

  .sidebar-section:last-child {
    border-bottom: none;
  }

  .sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background: none;
    border: none;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    font-weight: 600;
    padding: 0.65rem 0.75rem;
    cursor: pointer;
    border-radius: 0;
    user-select: none;
  }

  .sidebar-header:hover {
    background: rgba(139, 177, 255, 0.06);
  }

  .toggle {
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .sidebar-body {
    padding: 0 0.75rem 0.75rem;
  }

  .sidebar-section.collapsed .sidebar-body {
    display: none;
  }

  /* ── Explorer innards ── */

  .pane-toggle {
    background: transparent;
    color: var(--text-dim);
    font-size: 0.85rem;
    padding: 0.15rem 0.35rem;
    border-radius: 6px;
    border: 1px solid transparent;
    cursor: pointer;
    line-height: 1;
  }

  .pane-toggle:hover {
    border-color: rgba(157, 180, 255, 0.2);
    color: var(--text);
  }

  .update-badge {
    color: var(--accent, #60a5fa);
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
  }

  /* ── Editor ── */

  .editor {
    display: grid;
    grid-template-rows: auto minmax(0, auto) minmax(0, 1fr);
    overflow: hidden;
  }

  .tabs {
    display: flex;
    gap: 0.35rem;
    overflow-x: auto;
    padding: 0.7rem 0.7rem 0;
    border-bottom: 1px solid var(--border-faint);
  }

  .tab,
  .empty-tab {
    display: inline-flex;
    gap: 0.35rem;
    align-items: center;
    border-radius: 14px 14px 0 0;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid transparent;
    border-bottom: 0;
    padding: 0.35rem 0.4rem 0.35rem 0.65rem;
    color: var(--text-dim);
  }

  .tab.active {
    border-color: rgba(157, 180, 255, 0.2);
    background: rgba(139, 177, 255, 0.1);
    color: var(--text);
  }

  .tab > button,
  .close {
    background: transparent;
    color: inherit;
  }

  .loading,
  .error-banner {
    margin: 0;
    padding: 0.6rem 0.8rem;
    border-radius: 14px;
    background: rgba(139, 177, 255, 0.12);
    color: var(--text);
  }

  .error-banner {
    background: var(--danger-bg);
    color: var(--danger);
  }

  .viewer-wrap {
    display: grid;
    min-height: 0;
    overflow-y: auto;
  }

  @media (max-width: 1160px) {
    .shell-grid {
      grid-template-columns: minmax(14rem, 16rem) minmax(0, 1fr);
    }

    .inspector {
      grid-column: 1 / span 2;
    }
  }

  @media (max-width: 820px) {
    .topbar,
    .shell-grid {
      grid-template-columns: 1fr;
    }

    .shell-grid {
      grid-template-rows: minmax(16rem, auto) minmax(24rem, auto) minmax(16rem, auto);
    }

    .inspector {
      grid-column: 1;
    }

    .status-strip {
      width: 100%;
      min-width: 0;
    }

    .status-cluster,
    .pipeline-status-inline {
      width: 100%;
      min-width: 0;
    }

    .status-cluster {
      justify-items: stretch;
    }
  }

  @keyframes shell-spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
