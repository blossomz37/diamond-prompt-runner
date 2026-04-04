<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        SidebarExports.svelte
  // Description: Collapsible sidebar section for building and triggering export bundles
  // Version:     1.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type { ExportBundleResult, WorkspaceTab } from '$lib/types/project';

  interface Props {
    openTabs: WorkspaceTab[];
    activePath: string | null;
    onExportAssets: (bundleName: string, relativePaths: string[]) => Promise<ExportBundleResult>;
    exportLoading: boolean;
  }

  let {
    openTabs,
    activePath,
    onExportAssets,
    exportLoading
  }: Props = $props();

  let exportBundleName = $state('Export Bundle');
  let selectedExportPaths = $state<string[]>([]);
  let lastExportPath = $state<string | null>(null);

  const exportableTabs = $derived.by(() =>
    openTabs.filter((tab) => ['manifest', 'markdown', 'text', 'tera', 'yaml', 'json'].includes(tab.kind))
  );

  $effect(() => {
    const available = exportableTabs.map((tab) => tab.path);
    const nextSelectedPaths = selectedExportPaths.filter((path) => available.includes(path));

    if (
      nextSelectedPaths.length !== selectedExportPaths.length ||
      nextSelectedPaths.some((path, index) => path !== selectedExportPaths[index])
    ) {
      selectedExportPaths = nextSelectedPaths;
      return;
    }

    if (nextSelectedPaths.length === 0 && activePath && available.includes(activePath)) {
      selectedExportPaths = [activePath];
    }
  });

  function toggleExportPath(path: string): void {
    if (selectedExportPaths.includes(path)) {
      selectedExportPaths = selectedExportPaths.filter((item) => item !== path);
      return;
    }

    selectedExportPaths = [...selectedExportPaths, path];
  }

  async function handleExportSubmit(): Promise<void> {
    const result = await onExportAssets(exportBundleName.trim(), selectedExportPaths);
    exportBundleName = 'Export Bundle';
    lastExportPath = result.bundlePath;
  }
</script>

<div class="sidebar-exports">
  {#if exportableTabs.length === 0}
    <p class="empty">Open one or more supported assets in tabs to export them as a derived bundle.</p>
  {:else}
    <form class="export-form" onsubmit={(event) => { event.preventDefault(); void handleExportSubmit(); }}>
      <input
        type="text"
        bind:value={exportBundleName}
        aria-label="Export bundle name"
        placeholder="Export bundle name"
        disabled={exportLoading}
      />
      <div class="export-list">
        {#each exportableTabs as tab (tab.path)}
          <label class="export-item">
            <input
              type="checkbox"
              checked={selectedExportPaths.includes(tab.path)}
              onclick={() => toggleExportPath(tab.path)}
              disabled={exportLoading}
            />
            <span>{tab.title}</span>
            <small>{tab.path}</small>
          </label>
        {/each}
      </div>
      <div class="export-actions">
        <button
          type="submit"
          class="mini-action primary"
          disabled={exportLoading || !exportBundleName.trim() || selectedExportPaths.length === 0}
        >
          {exportLoading ? 'Exporting…' : 'Export Bundle'}
        </button>
        {#if lastExportPath}
          <p class="meta">Last export: {lastExportPath}</p>
        {/if}
      </div>
    </form>
  {/if}
</div>

<style>
  .sidebar-exports {
    display: grid;
    gap: 0.55rem;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .export-form {
    display: grid;
    gap: 0.6rem;
  }

  .export-form input[type="text"] {
    min-height: 2rem;
    border-radius: 8px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.35rem 0.55rem;
    font-size: 0.82rem;
  }

  .export-list {
    display: grid;
    gap: 0.4rem;
  }

  .export-item {
    display: grid;
    gap: 0.15rem;
    grid-template-columns: auto 1fr;
    column-gap: 0.45rem;
    align-items: start;
    font-size: 0.82rem;
  }

  .export-item span,
  .export-item small {
    grid-column: 2;
  }

  .export-item small {
    color: var(--text-soft);
    word-break: break-word;
    font-size: 0.76rem;
  }

  .export-actions {
    display: grid;
    gap: 0.4rem;
  }

  .meta {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.78rem;
  }

  .mini-action {
    min-height: 2rem;
    padding: 0.35rem 0.65rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    font-size: 0.82rem;
  }

  .mini-action.primary {
    background: rgba(139, 177, 255, 0.14);
    border-color: rgba(139, 177, 255, 0.28);
  }
</style>
