<script lang="ts">
  import type { ProjectRunHistoryEntry } from '$lib/types/project';

  interface Props {
    runHistory: ProjectRunHistoryEntry[];
    runHistoryLoading: boolean;
    onOpenRunPath: (path: string) => void | Promise<void>;
    onDeleteRun: (runPath: string) => Promise<void>;
  }

  let {
    runHistory,
    runHistoryLoading,
    onOpenRunPath,
    onDeleteRun
  }: Props = $props();

  let deleteConfirm = $state<string | null>(null);
  let deleteLoading = $state(false);
  let historyFilter = $state('all');

  async function handleDeleteRun(runPath: string): Promise<void> {
    if (deleteConfirm !== runPath) {
      deleteConfirm = runPath;
      return;
    }
    deleteLoading = true;
    try {
      await onDeleteRun(runPath);
      deleteConfirm = null;
    } finally {
      deleteLoading = false;
    }
  }

  const blockFilters = $derived.by(() => {
    return runHistory.reduce<ProjectRunHistoryEntry[]>((items, item) => {
      if (!item.blockId || items.some((c) => c.blockId === item.blockId)) return items;
      items.push(item);
      return items;
    }, []);
  });

  const pipelineFilters = $derived.by(() => {
    return runHistory.reduce<ProjectRunHistoryEntry[]>((items, item) => {
      if (!item.pipelineId || items.some((c) => c.pipelineId === item.pipelineId)) return items;
      items.push(item);
      return items;
    }, []);
  });

  const filteredHistory = $derived.by(() => {
    if (historyFilter === 'all') return runHistory;
    if (historyFilter.startsWith('block:')) {
      const blockId = historyFilter.slice('block:'.length);
      return runHistory.filter((item) => item.blockId === blockId);
    }
    if (historyFilter.startsWith('pipeline:')) {
      const pipelineId = historyFilter.slice('pipeline:'.length);
      return runHistory.filter((item) => item.pipelineId === pipelineId);
    }
    return runHistory;
  });
</script>

<div class="sidebar-runs">
  <div class="header-row">
    <span>{runHistory.length} runs</span>
    {#if runHistory.length > 0}
      <label class="filter-label">
        <select bind:value={historyFilter} class="filter-select">
          <option value="all">All runs</option>
          {#if blockFilters.length > 0}
            {#each blockFilters as item (item.blockId)}
              <option value={`block:${item.blockId}`}>Block: {item.blockName}</option>
            {/each}
          {/if}
          {#if pipelineFilters.length > 0}
            {#each pipelineFilters as item (item.pipelineId)}
              <option value={`pipeline:${item.pipelineId}`}>Pipeline: {item.pipelineName}</option>
            {/each}
          {/if}
        </select>
      </label>
    {/if}
  </div>

  {#if runHistoryLoading}
    <p class="empty-state">Loading runs…</p>
  {:else if filteredHistory.length === 0}
    <p class="empty-state">No matching run artifacts yet.</p>
  {:else}
    <div class="run-list">
      {#each filteredHistory as item (item.runId)}
        <article class="run-card">
          <div class="run-head">
            <div>
              <strong>{item.blockName}</strong>
              <p class="meta">{item.completedAt}</p>
            </div>
            <span class:failed={item.status === 'failed'} class="meta strong">{item.status}</span>
          </div>
          {#if item.pipelineName}
            <p class="meta">Pipeline: {item.pipelineName}</p>
          {/if}
          <p class="meta">{item.modelId}{item.usage.retryCount != null && item.usage.retryCount > 0 ? ` · ${item.usage.retryCount} retr${item.usage.retryCount === 1 ? 'y' : 'ies'}` : ''}</p>
          {#if item.outputPreview}
            <p class="preview">{item.outputPreview}</p>
          {:else if item.error}
            <p class="failed">{item.error}</p>
          {/if}
          <div class="run-actions">
            <button type="button" class="action-btn" onclick={() => { deleteConfirm = null; onOpenRunPath(item.runPath); }}>Open</button>
            <button
              type="button"
              class="action-btn"
              class:danger={deleteConfirm === item.runPath}
              onclick={() => handleDeleteRun(item.runPath)}
              disabled={deleteLoading}
              aria-label={deleteConfirm === item.runPath ? 'Confirm delete run' : 'Delete run'}
            >{deleteConfirm === item.runPath ? 'Confirm?' : 'Delete'}</button>
          </div>
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar-runs {
    display: grid;
    gap: 0.55rem;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-dim);
    font-size: 0.8rem;
  }

  .filter-select {
    font-size: 11px;
  }

  .run-list {
    display: grid;
    gap: 0.55rem;
  }

  .run-card {
    display: grid;
    gap: 0.35rem;
    padding: 0.65rem;
    border-radius: 12px;
    border: 1px solid var(--border-faint);
    background: rgba(255, 255, 255, 0.03);
  }

  .run-head {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    align-items: flex-start;
  }

  .meta {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.78rem;
  }

  .strong {
    color: var(--text);
  }

  .failed {
    color: var(--danger);
  }

  .preview {
    margin: 0;
    color: var(--text-dim);
    font-size: 0.82rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .run-actions {
    display: flex;
    gap: 0.4rem;
  }

  .action-btn {
    min-height: 2rem;
    border-radius: 8px;
    background: var(--input-bg);
    padding: 0.3rem 0.55rem;
    font-size: 0.78rem;
  }

  .action-btn.danger {
    border-color: rgba(255, 100, 100, 0.35);
    color: var(--danger);
  }
</style>
