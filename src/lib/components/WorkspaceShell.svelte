<script lang="ts">
  import AssetViewer from '$lib/components/AssetViewer.svelte';
  import ExplorerTree from '$lib/components/ExplorerTree.svelte';
  import InspectorPanel from '$lib/components/InspectorPanel.svelte';
  import type { ProjectAssetNode, ProjectSummary, WorkspaceTab } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    nodes: ProjectAssetNode[];
    tabs: WorkspaceTab[];
    activePath: string | null;
    loadingPath: string | null;
    errorMessage: string | null;
    onSelectAsset: (node: ProjectAssetNode) => void | Promise<void>;
    onSelectTab: (path: string) => void;
    onCloseTab: (path: string) => void;
    onDraftChange: (path: string, content: string) => void;
    onSaveTab: (path: string) => void | Promise<void>;
    onReloadTab: (path: string) => void | Promise<void>;
  }

  let {
    summary,
    nodes,
    tabs,
    activePath,
    loadingPath,
    errorMessage,
    onSelectAsset,
    onSelectTab,
    onCloseTab,
    onDraftChange,
    onSaveTab,
    onReloadTab
  }: Props = $props();

  const activeTab = $derived(tabs.find((tab) => tab.path === activePath) ?? null);
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

  <div class="shell-grid">
    <aside class="explorer panel">
      <div class="pane-head">
        <p class="eyebrow">Explorer</p>
        <span>{nodes.length} root nodes</span>
      </div>
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
        />
      </div>
    </main>

    <aside class="inspector panel">
      <InspectorPanel summary={summary} metadata={activeTab?.metadata ?? null} />
    </aside>

    <section class="bottom panel">
      <div class="pane-head">
        <p class="eyebrow">Bottom Panel</p>
        <span>Placeholder</span>
      </div>
      <p>
        Preview, validation, run history, and logs will land in later slices. Milestone 1 keeps this
        area intentionally inactive.
      </p>
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
    grid-template-rows: minmax(0, 1fr) 10rem;
    gap: 0.85rem;
    min-height: 0;
    flex: 1;
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
      grid-template-rows: minmax(0, 1fr) minmax(16rem, auto) 10rem;
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
      grid-template-rows: minmax(16rem, auto) minmax(24rem, auto) minmax(16rem, auto) 10rem;
    }

    .inspector,
    .bottom {
      grid-column: 1;
    }
  }
</style>
