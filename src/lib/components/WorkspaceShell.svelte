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
  import ExplorerTree from '$lib/components/ExplorerTree.svelte';
  import InspectorPanel from '$lib/components/InspectorPanel.svelte';
  import PipelineEditorTab from '$lib/components/PipelineEditorTab.svelte';
  import SidebarExports from '$lib/components/SidebarExports.svelte';
  import SidebarPipelines from '$lib/components/SidebarPipelines.svelte';
  import SidebarSettings from '$lib/components/SidebarSettings.svelte';
  import SidebarVariables from '$lib/components/SidebarVariables.svelte';
  import ValidationPanel from '$lib/components/ValidationPanel.svelte';
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
    SavedPipelineResult,
    ProjectSummary,
    TemplateValidationResult,
    WorkspaceTab
  } from '$lib/types/project';

  interface Props {
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
    pipelineAuthoringLoading: boolean;
    projectRunHistory: ProjectRunHistoryEntry[];
    projectRunHistoryLoading: boolean;
    projectUsageSummary: ProjectUsageSummary | null;
    globalVariables: Record<string, string>;
    onSetGlobalVariables: (variables: Record<string, string>) => Promise<void>;
    onSetProjectVariables: (variables: Record<string, string>) => Promise<void>;
    onSelectAsset: (node: ProjectAssetNode) => void | Promise<void>;
    onSelectTab: (path: string) => void;
    onCloseTab: (path: string) => void;
    onDraftChange: (path: string, content: string) => void;
    onSaveTab: (path: string) => void | Promise<void>;
    onReloadTab: (path: string) => void | Promise<void>;
    onRunTab: (path: string) => void | Promise<void>;
    onRunPipeline: (pipelineId: string) => void | Promise<void>;
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
    onOpenPresetFile: (presetPath: string) => void;
    onSetBlockPreset: (blockId: string, presetPath: string | null) => Promise<void>;
    onSetBlockOutputTarget: (blockId: string, target: string) => Promise<void>;
    onDeletePipeline: (pipelineId: string) => Promise<void>;
    onDeletePromptBlock: (blockId: string) => Promise<void>;
    onDeleteRun: (runPath: string) => Promise<void>;
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
    pipelineAuthoringLoading,
    projectRunHistory,
    projectRunHistoryLoading,
    projectUsageSummary,
    globalVariables,
    onSetGlobalVariables,
    onSetProjectVariables,
    onSelectAsset,
    onSelectTab,
    onCloseTab,
    onDraftChange,
    onSaveTab,
    onReloadTab,
    onRunTab,
    onRunPipeline,
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
    onOpenPresetFile,
    onSetBlockPreset,
    onSetBlockOutputTarget,
    onDeletePipeline,
    onDeletePromptBlock,
    onDeleteRun
  }: Props = $props();

  let deleteBlockConfirm = $state<string | null>(null);
  let deleteBlockLoading = $state(false);

  async function handleDeletePromptBlock(blockId: string): Promise<void> {
    if (deleteBlockConfirm !== blockId) {
      deleteBlockConfirm = blockId;
      return;
    }
    deleteBlockLoading = true;
    try {
      await onDeletePromptBlock(blockId);
      deleteBlockConfirm = null;
    } finally {
      deleteBlockLoading = false;
    }
  }

  const activeTab = $derived(tabs.find((tab) => tab.path === activePath) ?? null);
  const activePromptBlock = $derived(
    activeTab && activeTab.kind === 'tera'
      ? promptBlocks.find((b) => b.templateSource === activeTab.path) ?? null
      : null
  );
  const activeExecution = $derived(
    activeTab && executionResult?.path === activeTab.path ? executionResult : null
  );

  // Sidebar section collapse states
  let explorerOpen = $state(true);
  let pipelinesOpen = $state(false);
  let variablesOpen = $state(false);
  let exportsOpen = $state(false);
  let settingsOpen = $state(false);

  // Bottom panel
  let bottomOpen = $state(true);

  // Explorer prompt creation
  let createPromptOpen = $state(false);
  let newPromptName = $state('');

  // Pipeline editor virtual tab state
  let pipelineEditorActive = $state(false);
  let pipelineEditorTarget = $state<ProjectPipeline | null>(null);

  const pipelineEditorTitle = $derived(
    pipelineEditorTarget ? `Pipeline: ${pipelineEditorTarget.name}` : 'New Pipeline'
  );

  async function handleCreatePromptSubmit(): Promise<void> {
    const trimmed = newPromptName.trim();
    if (!trimmed) return;

    try {
      await onCreatePrompt(trimmed);
      newPromptName = '';
      createPromptOpen = false;
    } catch {
      // Keep the form open so the user can correct and retry.
    }
  }

  function openPipelineEditor(pipeline: ProjectPipeline | null): void {
    pipelineEditorTarget = pipeline;
    pipelineEditorActive = true;
  }

  function closePipelineEditor(): void {
    pipelineEditorActive = false;
    pipelineEditorTarget = null;
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
    <div class="counts">
      <span>{summary.counts.documents} docs</span>
      <span>{summary.counts.prompts} prompts</span>
      <span>{summary.counts.models} models</span>
      <span>{summary.defaultModelPreset}</span>
    </div>
  </header>

  {#if errorMessage}
    <p class="error-banner">{errorMessage}</p>
  {/if}

  <div class="shell-grid" class:bottom-closed={!bottomOpen}>
    <aside class="sidebar panel">
      <!-- Explorer section -->
      <div class="sidebar-section" class:collapsed={!explorerOpen}>
        <button type="button" class="sidebar-header" onclick={() => (explorerOpen = !explorerOpen)}>
          <span>Explorer</span>
          <span class="toggle">{explorerOpen ? '▾' : '▸'}</span>
        </button>
        {#if explorerOpen}
          <div class="sidebar-body">
            <div class="pane-head">
              <div class="pane-actions">
                <span>{nodes.length} root nodes</span>
                <button type="button" class="mini-action" onclick={() => (createPromptOpen = !createPromptOpen)}>
                  {createPromptOpen ? 'Close' : 'New Prompt'}
                </button>
              </div>
            </div>
            {#if createPromptOpen}
              <form class="create-form" onsubmit={(event) => { event.preventDefault(); void handleCreatePromptSubmit(); }}>
                <input
                  type="text"
                  bind:value={newPromptName}
                  placeholder="Prompt name"
                  aria-label="Prompt name"
                  disabled={promptCreationLoading}
                />
                <button type="submit" class="mini-action primary" disabled={promptCreationLoading || !newPromptName.trim()}>
                  {promptCreationLoading ? 'Creating…' : 'Create'}
                </button>
              </form>
            {/if}
            <ExplorerTree nodes={nodes} activePath={pipelineEditorActive ? null : activePath} onSelectPath={onSelectAsset} />

            {#if promptBlocks.length > 0}
              <div class="block-list">
                <p class="block-list-label">Registered Blocks</p>
                {#each promptBlocks as block (block.blockId)}
                  <div class="block-row">
                    <span class="block-name">{block.name}</span>
                    <button
                      type="button"
                      class="mini-action block-delete"
                      class:danger={deleteBlockConfirm === block.blockId}
                      onclick={() => handleDeletePromptBlock(block.blockId)}
                      disabled={deleteBlockLoading}
                      aria-label={deleteBlockConfirm === block.blockId ? `Confirm remove ${block.name}` : `Remove ${block.name}`}
                    >
                      {deleteBlockConfirm === block.blockId ? 'Confirm?' : 'Remove'}
                    </button>
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Pipelines section -->
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
              {pipelineExecution}
              {pipelineLoading}
              {pipelineAuthoringLoading}
              onRunPipeline={onRunPipeline}
              onEditPipeline={(pipeline) => openPipelineEditor(pipeline)}
              onNewPipeline={() => openPipelineEditor(null)}
              {onDeletePipeline}
            />
          </div>
        {/if}
      </div>

      <!-- Variables section -->
      <div class="sidebar-section" class:collapsed={!variablesOpen}>
        <button type="button" class="sidebar-header" onclick={() => (variablesOpen = !variablesOpen)}>
          <span>Variables</span>
          <span class="toggle">{variablesOpen ? '▾' : '▸'}</span>
        </button>
        {#if variablesOpen}
          <div class="sidebar-body">
            <SidebarVariables
              {summary}
              {globalVariables}
              {onSetGlobalVariables}
              {onSetProjectVariables}
            />
          </div>
        {/if}
      </div>

      <!-- Exports section -->
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

      <!-- Settings section -->
      <div class="sidebar-section" class:collapsed={!settingsOpen}>
        <button type="button" class="sidebar-header" onclick={() => (settingsOpen = !settingsOpen)}>
          <span>Settings</span>
          <span class="toggle">{settingsOpen ? '▾' : '▸'}</span>
        </button>
        {#if settingsOpen}
          <div class="sidebar-body">
            <SidebarSettings
              {summary}
              presets={modelPresets}
              {onRenameProject}
              {onSetDefaultPreset}
              {onCreatePreset}
              {onDeletePreset}
              {onOpenPresetFile}
            />
          </div>
        {/if}
      </div>
    </aside>

    <main class="editor panel">
      <div class="tabs" data-testid="workspace-tabs">
        {#if tabs.length === 0 && !pipelineEditorActive}
          <div class="empty-tab">No tabs open</div>
        {:else}
          {#each tabs as tab (tab.path)}
            <div class:active={tab.path === activePath && !pipelineEditorActive} class="tab">
              <button type="button" onclick={() => { pipelineEditorActive = false; onSelectTab(tab.path); }}>
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
        {/if}
      </div>

      {#if loadingPath}
        <div class="loading">Loading {loadingPath}…</div>
      {/if}

      <div class="viewer-wrap">
        {#if pipelineEditorActive}
          <PipelineEditorTab
            existingPipeline={pipelineEditorTarget}
            {promptBlocks}
            loading={pipelineAuthoringLoading}
            onSave={handlePipelineSave}
            onCancel={closePipelineEditor}
          />
        {:else}
          <AssetViewer
            tab={activeTab}
            onDraftChange={onDraftChange}
            onSave={onSaveTab}
            onReload={onReloadTab}
            onExecute={onRunTab}
            execution={activeExecution}
            executionLoading={executionLoading && activeTab?.kind === 'tera'}
          />
        {/if}
      </div>
    </main>

    <aside class="inspector panel">
      <InspectorPanel
        summary={summary}
        metadata={activeTab?.metadata ?? null}
        runHistory={projectRunHistory}
        runHistoryLoading={projectRunHistoryLoading}
        usageSummary={projectUsageSummary}
        onOpenRunPath={onOpenRunPath}
        onDeleteRun={onDeleteRun}
        activePromptBlock={activePromptBlock}
        modelPresets={modelPresets}
        onSetBlockPreset={onSetBlockPreset}
        onSetBlockOutputTarget={onSetBlockOutputTarget}
      />
    </aside>

    <section class="bottom panel">
      <div class="pane-head">
        <p class="eyebrow">Bottom Panel</p>
        <div class="pane-controls">
          <span>{activeTab?.kind === 'tera' ? 'Validation + Run' : 'Preview'}</span>
          <button
            type="button"
            class="pane-toggle"
            onclick={() => (bottomOpen = !bottomOpen)}
            aria-label={bottomOpen ? 'Collapse bottom panel' : 'Expand bottom panel'}
          >{bottomOpen ? '▾' : '▸'}</button>
        </div>
      </div>
      {#if bottomOpen}
        <ValidationPanel
          tab={activeTab}
          validation={validationResult}
          loading={validationLoading}
          execution={activeExecution}
          executionLoading={executionLoading && activeTab?.kind === 'tera'}
          credentialState={credentialState}
          recentRuns={historyItems}
          recentRunsLoading={historyLoading}
          onOpenRunPath={onOpenRunPath}
          onExecute={onRunTab}
        />
      {/if}
    </section>
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
    background: var(--hover-bg, rgba(255, 255, 255, 0.08));
  }

  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  h1 {
    margin: 0.2rem 0 0;
    font-size: 1.15rem;
  }

  .counts {
    display: flex;
    gap: 0.55rem;
    flex-wrap: wrap;
    align-items: center;
    color: var(--text-dim);
    font-size: 0.84rem;
  }

  .counts span {
    padding: 0.28rem 0.55rem;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(157, 180, 255, 0.12);
  }

  .shell-grid {
    display: grid;
    grid-template-columns: minmax(15rem, 18rem) minmax(0, 1fr) minmax(17rem, 21rem);
    grid-template-rows: minmax(0, 1fr) minmax(10rem, auto);
    gap: 0.85rem;
    min-height: 0;
    flex: 1;
  }

  .shell-grid.bottom-closed {
    grid-template-rows: minmax(0, 1fr) auto;
  }

  .sidebar,
  .inspector,
  .editor,
  .bottom {
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

  .pane-head {
    display: flex;
    justify-content: space-between;
    gap: 0.65rem;
    align-items: center;
    margin-bottom: 0.75rem;
    color: var(--text-dim);
    font-size: 0.8rem;
  }

  .pane-controls {
    display: flex;
    align-items: center;
    gap: 0.55rem;
  }

  .pane-actions {
    display: flex;
    align-items: center;
    gap: 0.55rem;
  }

  .mini-action {
    min-height: 2rem;
    padding: 0.35rem 0.65rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
  }

  .mini-action.primary {
    background: rgba(139, 177, 255, 0.14);
    border-color: rgba(139, 177, 255, 0.28);
  }

  .mini-action.danger {
    border-color: rgba(255, 100, 100, 0.35);
    color: var(--danger);
  }

  .block-list {
    display: grid;
    gap: 0.3rem;
    margin-top: 0.65rem;
    padding-top: 0.65rem;
    border-top: 1px solid rgba(157, 180, 255, 0.08);
  }

  .block-list-label {
    margin: 0 0 0.25rem;
    color: var(--text-soft);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .block-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }

  .block-name {
    font-size: 0.82rem;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .block-delete {
    flex-shrink: 0;
    min-height: 0;
    padding: 0.2rem 0.45rem;
    font-size: 0.75rem;
  }

  .create-form {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .create-form input {
    min-width: 0;
    min-height: 2.2rem;
    border-radius: 12px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.55rem 0.75rem;
  }

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
    border-bottom: 1px solid rgba(157, 180, 255, 0.12);
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
    background: rgba(255, 141, 161, 0.12);
    color: var(--danger);
  }

  .viewer-wrap {
    display: grid;
    min-height: 0;
    overflow: hidden;
  }

  /* ── Bottom panel ── */

  .bottom {
    grid-column: 1 / span 3;
    display: grid;
    grid-template-rows: auto auto;
    gap: 0.45rem;
    padding: 0.8rem 1rem;
    color: var(--text-dim);
  }

  .bottom p {
    margin: 0;
  }

  @media (max-width: 1160px) {
    .shell-grid {
      grid-template-columns: minmax(14rem, 16rem) minmax(0, 1fr);
      grid-template-rows: minmax(0, 1fr) minmax(16rem, auto) minmax(10rem, auto);
    }

    .shell-grid.bottom-closed {
      grid-template-rows: minmax(0, 1fr) minmax(16rem, auto) auto;
    }

    .inspector {
      grid-column: 1 / span 2;
    }

    .bottom {
      grid-column: 1 / span 2;
    }
  }

  @media (max-width: 820px) {
    .topbar,
    .shell-grid {
      grid-template-columns: 1fr;
    }

    .shell-grid {
      grid-template-rows: minmax(16rem, auto) minmax(24rem, auto) minmax(16rem, auto) minmax(10rem, auto);
    }

    .shell-grid.bottom-closed {
      grid-template-rows: minmax(16rem, auto) minmax(24rem, auto) minmax(16rem, auto) auto;
    }

    .inspector,
    .bottom {
      grid-column: 1;
    }
  }
</style>
