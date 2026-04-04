<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        InspectorPanel.svelte
  // Description: Read-only right sidebar showing project summary, usage, file metadata, and run history
  // Version:     2.0.0
  // Created:     2026-04-03
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type {
    AssetMetadata,
    ModelPresetSummary,
    ProjectPromptBlock,
    ProjectRunHistoryEntry,
    ProjectSummary,
    ProjectUsageSummary
  } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    metadata: AssetMetadata | null;
    runHistory: ProjectRunHistoryEntry[];
    runHistoryLoading: boolean;
    usageSummary: ProjectUsageSummary | null;
    activePromptBlock: ProjectPromptBlock | null;
    modelPresets: ModelPresetSummary[];
    onOpenRunPath: (path: string) => void | Promise<void>;
    onSetBlockPreset: (blockId: string, presetPath: string | null) => Promise<void>;
  }

  let {
    summary,
    metadata,
    runHistory,
    runHistoryLoading,
    usageSummary,
    activePromptBlock,
    modelPresets,
    onOpenRunPath,
    onSetBlockPreset
  }: Props = $props();

  let historyFilter = $state('all');

  const blockFilters = $derived.by(() => {
    return runHistory.reduce<ProjectRunHistoryEntry[]>((items, item) => {
      if (!item.blockId || items.some((candidate) => candidate.blockId === item.blockId)) {
        return items;
      }

      items.push(item);
      return items;
    }, []);
  });

  const pipelineFilters = $derived.by(() => {
    return runHistory.reduce<ProjectRunHistoryEntry[]>((items, item) => {
      if (!item.pipelineId || items.some((candidate) => candidate.pipelineId === item.pipelineId)) {
        return items;
      }

      items.push(item);
      return items;
    }, []);
  });

  const filteredHistory = $derived.by(() => {
    if (historyFilter === 'all') {
      return runHistory;
    }

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

  let prevMetadataPath = $state<string | null>(null);
  let runHistoryOpen = $state(true);

  $effect(() => {
    const currentPath = metadata?.path ?? null;
    if (currentPath !== prevMetadataPath) {
      runHistoryOpen = (currentPath === null);
      prevMetadataPath = currentPath;
    }
  });

  let presetSaving = $state(false);
  let presetError = $state('');

  async function handlePresetChange(event: Event): Promise<void> {
    if (!activePromptBlock) return;
    const select = event.target as HTMLSelectElement;
    const value = select.value || null;
    
    presetSaving = true;
    presetError = '';
    try {
      await onSetBlockPreset(activePromptBlock.blockId, value);
    } catch (e) {
      presetError = String(e);
    } finally {
      presetSaving = false;
    }
  }
</script>

<aside class="inspector">
  {#if !metadata}
    <section class="section">
      <p class="eyebrow">Project</p>
      <h3>{summary.projectName}</h3>
      <dl>
        <div>
          <dt>Root</dt>
          <dd>{summary.rootPath}</dd>
        </div>
        <div>
          <dt>Default preset</dt>
          <dd>{summary.defaultModelPreset}</dd>
        </div>
        <div>
          <dt>Assets</dt>
          <dd>{summary.counts.documents} docs / {summary.counts.prompts} prompts / {summary.counts.models} models</dd>
        </div>
      </dl>
    </section>

    {#if usageSummary && usageSummary.totalRuns > 0}
      <section class="section">
        <p class="eyebrow">Usage</p>
        <dl>
          <div>
            <dt>Runs</dt>
            <dd>{usageSummary.successfulRuns} succeeded · {usageSummary.failedRuns} failed</dd>
          </div>
          <div>
            <dt>Total Tokens</dt>
            <dd>{usageSummary.totalTokens.toLocaleString()}</dd>
          </div>
          {#if usageSummary.totalCost > 0}
            <div>
              <dt>Total Cost</dt>
              <dd>${usageSummary.totalCost.toFixed(4)}</dd>
            </div>
          {/if}
          {#if usageSummary.totalOutputWords > 0}
            <div>
              <dt>Output Words</dt>
              <dd>{usageSummary.totalOutputWords.toLocaleString()}</dd>
            </div>
          {/if}
          {#if usageSummary.totalRetries > 0}
            <div>
              <dt>Total Retries</dt>
              <dd>{usageSummary.totalRetries}</dd>
            </div>
          {/if}
        </dl>
      </section>
    {/if}
  {:else}
    <section class="section">
      <p class="eyebrow">File Info</p>
      <h3>{metadata.name}</h3>
      <dl>
        <div>
          <dt>Kind</dt>
          <dd>{metadata.kind}</dd>
        </div>
        <div>
          <dt>Path</dt>
          <dd>{metadata.path}</dd>
        </div>
        <div>
          <dt>Size</dt>
          <dd>{metadata.sizeBytes === null ? '—' : `${metadata.sizeBytes} bytes`}</dd>
        </div>
        <div>
          <dt>Modified</dt>
          <dd>{metadata.modifiedAt ?? '—'}</dd>
        </div>
        {#each metadata.details as detail (detail.label)}
          <div>
            <dt>{detail.label}</dt>
            <dd>{detail.value}</dd>
          </div>
        {/each}
      </dl>
    </section>

    {#if activePromptBlock}
      <section class="section preset-section">
        <p class="eyebrow">Block Settings</p>
        <div class="preset-override">
          <label for="block-preset-select">Model Override</label>
          <select
            id="block-preset-select"
            class="preset-select"
            value={activePromptBlock.modelPreset}
            onchange={handlePresetChange}
            disabled={presetSaving}
          >
            <option value="">(Use project default)</option>
            {#each modelPresets as preset (preset.path)}
              <option value={preset.path}>{preset.filename} — {preset.modelId}</option>
            {/each}
          </select>
          {#if presetError}
            <p class="preset-error">{presetError}</p>
          {/if}
        </div>
      </section>
    {/if}
  {/if}

  <section class="section">
    <div class="history-heading">
      <button type="button" class="history-toggle" onclick={() => (runHistoryOpen = !runHistoryOpen)}>
        <p class="eyebrow">Run History</p>
        <span class="toggle">{runHistoryOpen ? '▾' : '▸'}</span>
      </button>
      {#if runHistoryOpen}
        <label class="history-filter">
          <span>Filter</span>
          <select bind:value={historyFilter}>
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

    {#if runHistoryOpen}
      {#if runHistoryLoading}
        <p class="empty">Loading persisted project runs…</p>
      {:else if filteredHistory.length === 0}
        <p class="empty">No matching run artifacts yet.</p>
      {:else}
        <div class="history-list">
          {#each filteredHistory as item (item.runId)}
            <article class="history-item">
              <div class="history-item-head">
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
                <p>{item.outputPreview}</p>
              {:else if item.error}
                <p class="failed">{item.error}</p>
              {/if}
              <button type="button" class="history-open" onclick={() => onOpenRunPath(item.runPath)}>Open artifact</button>
            </article>
          {/each}
        </div>
      {/if}
    {/if}
  </section>
</aside>

<style>
  .inspector {
    display: grid;
    align-content: start;
    gap: 1.2rem;
    padding: 0.95rem 0.75rem;
    overflow-y: auto;
    font-size: 0.86rem;
    color: var(--text-dim);
  }

  .section {
    display: grid;
    gap: 0.55rem;
  }

  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
  }

  dl {
    display: grid;
    gap: 0.7rem;
    margin: 0;
  }

  dt {
    margin: 0 0 0.18rem;
    color: var(--text-soft);
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  dd {
    margin: 0;
    color: var(--text-dim);
    word-break: break-word;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
  }

  .meta {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .strong {
    color: var(--text);
  }

  .failed {
    color: var(--danger);
  }

  .history-heading,
  .history-item-head {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .history-filter {
    display: grid;
    gap: 0.25rem;
    min-width: 10rem;
  }

  .history-filter span {
    color: var(--text-soft);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .history-filter select,
  .history-open {
    min-height: 2rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.35rem 0.55rem;
  }

  .history-list {
    display: grid;
    gap: 0.75rem;
  }

  .history-item {
    display: grid;
    gap: 0.45rem;
    padding: 0.85rem;
    border-radius: 14px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
  }

  .history-toggle {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    text-align: left;
  }

  .history-toggle:hover .eyebrow {
    color: var(--text);
  }

  .history-toggle .toggle {
    font-size: 11px;
    color: var(--text-soft);
  }

  .preset-override {
    display: grid;
    gap: 0.4rem;
  }

  .preset-override label {
    font-size: 0.72rem;
    color: var(--text-soft);
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .preset-select {
    width: 100%;
    min-height: 2rem;
    border-radius: 8px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.3rem 0.5rem;
    font-size: 11.5px;
    outline: none;
    cursor: pointer;
  }

  .preset-select:focus {
    border-color: rgba(139, 177, 255, 0.35);
  }

  .preset-error {
    font-size: 11px;
    color: var(--danger);
    margin: 0;
  }
</style>
