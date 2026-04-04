<script lang="ts">
  import AssetViewer from '$lib/components/AssetViewer.svelte';
  import ExplorerTree from '$lib/components/ExplorerTree.svelte';
  import InspectorPanel from '$lib/components/InspectorPanel.svelte';
  import ValidationPanel from '$lib/components/ValidationPanel.svelte';
  import type {
    ExecutionCredentialStatus,
    PipelineExecutionResult,
    ProjectAssetNode,
    ProjectPipeline,
    PromptExecutionResult,
    PromptRunHistoryEntry,
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
    pipelineExecution: PipelineExecutionResult | null;
    pipelineLoading: boolean;
    onSelectAsset: (node: ProjectAssetNode) => void | Promise<void>;
    onSelectTab: (path: string) => void;
    onCloseTab: (path: string) => void;
    onDraftChange: (path: string, content: string) => void;
    onSaveTab: (path: string) => void | Promise<void>;
    onReloadTab: (path: string) => void | Promise<void>;
    onRunTab: (path: string) => void | Promise<void>;
    onRunPipeline: (pipelineId: string) => void | Promise<void>;
    onCreatePrompt: (name: string) => void | Promise<void>;
    promptCreationLoading: boolean;
    credentialState: ExecutionCredentialStatus;
    credentialDraft: string;
    credentialLoading: boolean;
    onExecutionCredentialInput: (value: string) => void;
    onSaveExecutionCredential: () => void | Promise<void>;
    onClearExecutionCredential: () => void | Promise<void>;
    historyItems: PromptRunHistoryEntry[];
    historyLoading: boolean;
    onOpenRunPath: (path: string) => void | Promise<void>;
    validationResult: TemplateValidationResult | null;
    validationLoading: boolean;
    executionResult: PromptExecutionResult | null;
    executionLoading: boolean;
  }

  let {
    summary,
    nodes,
    tabs,
    activePath,
    loadingPath,
    errorMessage,
    pipelines,
    pipelineExecution,
    pipelineLoading,
    onSelectAsset,
    onSelectTab,
    onCloseTab,
    onDraftChange,
    onSaveTab,
    onReloadTab,
    onRunTab,
    onRunPipeline,
    onCreatePrompt,
    promptCreationLoading,
    credentialState,
    credentialDraft,
    credentialLoading,
    onExecutionCredentialInput,
    onSaveExecutionCredential,
    onClearExecutionCredential,
    historyItems,
    historyLoading,
    onOpenRunPath,
    validationResult,
    validationLoading,
    executionResult,
    executionLoading
  }: Props = $props();

  const activeTab = $derived(tabs.find((tab) => tab.path === activePath) ?? null);
  const activeExecution = $derived(
    activeTab && executionResult?.path === activeTab.path ? executionResult : null
  );

  let bottomOpen = $state(true);
  let createPromptOpen = $state(false);
  let newPromptName = $state('');

  async function handleCreatePromptSubmit(): Promise<void> {
    const trimmed = newPromptName.trim();
    if (!trimmed) {
      return;
    }

    try {
      await onCreatePrompt(trimmed);
      newPromptName = '';
      createPromptOpen = false;
    } catch {
      // Keep the form open so the user can correct and retry.
    }
  }
</script>

<section class="workspace-shell">
  <header class="topbar panel">
    <div>
      <p class="eyebrow">Workspace</p>
      <h1>{summary.projectName}</h1>
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
    <aside class="explorer panel">
      <div class="pane-head">
        <p class="eyebrow">Explorer</p>
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
      <ExplorerTree nodes={nodes} activePath={activePath} onSelectPath={onSelectAsset} />
    </aside>

    <main class="editor panel">
      <div class="tabs" data-testid="workspace-tabs">
        {#if tabs.length === 0}
          <div class="empty-tab">No tabs open</div>
        {:else}
          {#each tabs as tab (tab.path)}
            <div class:active={tab.path === activePath} class="tab">
              <button type="button" onclick={() => onSelectTab(tab.path)}>
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
        {/if}
      </div>

      {#if loadingPath}
        <div class="loading">Loading {loadingPath}…</div>
      {/if}

      <div class="viewer-wrap">
        <AssetViewer
          tab={activeTab}
          onDraftChange={onDraftChange}
          onSave={onSaveTab}
          onReload={onReloadTab}
          onExecute={onRunTab}
          execution={activeExecution}
          executionLoading={executionLoading && activeTab?.kind === 'tera'}
        />
      </div>
    </main>

    <aside class="inspector panel">
      <InspectorPanel
        summary={summary}
        metadata={activeTab?.metadata ?? null}
        {pipelines}
        pipelineExecution={pipelineExecution}
        pipelineLoading={pipelineLoading}
        onRunPipeline={onRunPipeline}
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
          credentialDraft={credentialDraft}
          credentialLoading={credentialLoading}
          onCredentialInput={onExecutionCredentialInput}
          onSaveCredential={onSaveExecutionCredential}
          onClearCredential={onClearExecutionCredential}
          recentRuns={historyItems}
          recentRunsLoading={historyLoading}
          onOpenRunPath={onOpenRunPath}
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
    gap: 1rem;
    padding: 0.95rem 1.1rem;
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

  .explorer,
  .inspector,
  .editor,
  .bottom {
    min-height: 0;
  }

  .explorer {
    padding: 0.75rem;
    overflow: auto;
  }

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
