<script lang="ts">
  import type {
    AssetMetadata,
    ExportBundleResult,
    PipelineExecutionResult,
    ProjectPipeline,
    ProjectPromptBlock,
    ProjectRunHistoryEntry,
    ProjectSummary,
    SavedPipelineResult,
    WorkspaceTab
  } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    metadata: AssetMetadata | null;
    openTabs: WorkspaceTab[];
    activePath: string | null;
    pipelines: ProjectPipeline[];
    promptBlocks: ProjectPromptBlock[];
    pipelineExecution: PipelineExecutionResult | null;
    pipelineLoading: boolean;
    pipelineAuthoringLoading: boolean;
    onRunPipeline: (pipelineId: string) => void | Promise<void>;
    onCreatePipeline: (name: string, orderedBlockIds: string[]) => Promise<SavedPipelineResult>;
    onUpdatePipeline: (
      pipelineId: string,
      name: string,
      orderedBlockIds: string[]
    ) => Promise<SavedPipelineResult>;
    onExportAssets: (bundleName: string, relativePaths: string[]) => Promise<ExportBundleResult>;
    exportLoading: boolean;
    runHistory: ProjectRunHistoryEntry[];
    runHistoryLoading: boolean;
    onOpenRunPath: (path: string) => void | Promise<void>;
  }

  let {
    summary,
    metadata,
    openTabs,
    activePath,
    pipelines,
    promptBlocks,
    pipelineExecution,
    pipelineLoading,
    pipelineAuthoringLoading,
    onRunPipeline,
    onCreatePipeline,
    onUpdatePipeline,
    onExportAssets,
    exportLoading,
    runHistory,
    runHistoryLoading,
    onOpenRunPath
  }: Props = $props();

  let createPipelineOpen = $state(false);
  let createPipelineName = $state('');
  let createPipelineBlocks = $state<string[]>([]);
  let createBlockChoice = $state('');
  let editingPipelineId = $state<string | null>(null);
  let editingPipelineName = $state('');
  let editingPipelineBlocks = $state<string[]>([]);
  let editingBlockChoice = $state('');
  let exportBundleName = $state('Export Bundle');
  let selectedExportPaths = $state<string[]>([]);
  let lastExportPath = $state<string | null>(null);
  let historyFilter = $state('all');

  const defaultPromptBlockChoice = $derived(promptBlocks[0]?.blockId ?? '');

  $effect(() => {
    if (!createBlockChoice || createPipelineBlocks.includes(createBlockChoice)) {
      createBlockChoice =
        promptBlocks.find((block) => !createPipelineBlocks.includes(block.blockId))?.blockId ?? '';
    }
  });

  $effect(() => {
    if (!editingBlockChoice || editingPipelineBlocks.includes(editingBlockChoice)) {
      editingBlockChoice =
        promptBlocks.find((block) => !editingPipelineBlocks.includes(block.blockId))?.blockId ?? '';
    }
  });

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

  function blockLabel(blockId: string): string {
    const block = promptBlocks.find((item) => item.blockId === blockId);
    return block ? `${block.name} (${block.blockId})` : blockId;
  }

  function startCreatePipeline(): void {
    createPipelineOpen = true;
    createPipelineName = '';
    createPipelineBlocks = [];
    createBlockChoice = defaultPromptBlockChoice;
  }

  function cancelCreatePipeline(): void {
    createPipelineOpen = false;
    createPipelineName = '';
    createPipelineBlocks = [];
    createBlockChoice = defaultPromptBlockChoice;
  }

  function startEditPipeline(pipeline: ProjectPipeline): void {
    editingPipelineId = pipeline.pipelineId;
    editingPipelineName = pipeline.name;
    editingPipelineBlocks = pipeline.blocks.map((block) => block.blockId);
    editingBlockChoice = promptBlocks.find(
      (block) => !pipeline.blocks.some((pipelineBlock) => pipelineBlock.blockId === block.blockId)
    )?.blockId ?? '';
  }

  function cancelEditPipeline(): void {
    editingPipelineId = null;
    editingPipelineName = '';
    editingPipelineBlocks = [];
    editingBlockChoice = '';
  }

  function addCreateBlock(): void {
    if (!createBlockChoice || createPipelineBlocks.includes(createBlockChoice)) {
      return;
    }

    createPipelineBlocks = [...createPipelineBlocks, createBlockChoice];
  }

  function addEditingBlock(): void {
    if (!editingBlockChoice || editingPipelineBlocks.includes(editingBlockChoice)) {
      return;
    }

    editingPipelineBlocks = [...editingPipelineBlocks, editingBlockChoice];
  }

  function removeCreateBlock(index: number): void {
    createPipelineBlocks = createPipelineBlocks.filter((_, itemIndex) => itemIndex !== index);
  }

  function removeEditingBlock(index: number): void {
    editingPipelineBlocks = editingPipelineBlocks.filter((_, itemIndex) => itemIndex !== index);
  }

  function moveCreateBlock(index: number, direction: -1 | 1): void {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= createPipelineBlocks.length) {
      return;
    }

    const next = [...createPipelineBlocks];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    createPipelineBlocks = next;
  }

  function moveEditingBlock(index: number, direction: -1 | 1): void {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= editingPipelineBlocks.length) {
      return;
    }

    const next = [...editingPipelineBlocks];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    editingPipelineBlocks = next;
  }

  async function handleCreatePipelineSubmit(): Promise<void> {
    await onCreatePipeline(createPipelineName.trim(), createPipelineBlocks);
    cancelCreatePipeline();
  }

  async function handleUpdatePipelineSubmit(): Promise<void> {
    if (!editingPipelineId) {
      return;
    }

    await onUpdatePipeline(editingPipelineId, editingPipelineName.trim(), editingPipelineBlocks);
    cancelEditPipeline();
  }

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

<aside class="inspector">
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

  <section class="section">
    <p class="eyebrow">Inspector</p>
    {#if metadata}
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
    {:else}
      <p class="empty">Open a file from the explorer to inspect file metadata here.</p>
    {/if}
  </section>

  <section class="section">
    <p class="eyebrow">Exports</p>
    {#if exportableTabs.length === 0}
      <p class="empty">Open one or more supported assets in tabs to export them as a derived bundle.</p>
    {:else}
      <form class="pipeline-form" onsubmit={(event) => { event.preventDefault(); void handleExportSubmit(); }}>
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
        <div class="pipeline-form-actions">
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
  </section>

  <section class="section">
    <div class="section-head">
      <p class="eyebrow">Pipelines</p>
      <button
        type="button"
        class="mini-action"
        onclick={() => (createPipelineOpen ? cancelCreatePipeline() : startCreatePipeline())}
        disabled={pipelineAuthoringLoading || promptBlocks.length === 0}
      >
        {createPipelineOpen ? 'Close' : 'New Pipeline'}
      </button>
    </div>

    {#if promptBlocks.length === 0}
      <p class="empty">Create at least one prompt block before authoring a pipeline.</p>
    {:else if createPipelineOpen}
      <form class="pipeline-form" onsubmit={(event) => { event.preventDefault(); void handleCreatePipelineSubmit(); }}>
        <input
          type="text"
          bind:value={createPipelineName}
          placeholder="Pipeline name"
          aria-label="Pipeline name"
          disabled={pipelineAuthoringLoading}
        />
        <div class="pipeline-picker-row">
          <select bind:value={createBlockChoice} disabled={pipelineAuthoringLoading} aria-label="Available prompt blocks">
            <option value="">Select a prompt block</option>
            {#each promptBlocks.filter((block) => !createPipelineBlocks.includes(block.blockId)) as block (block.blockId)}
              <option value={block.blockId}>{block.name}</option>
            {/each}
          </select>
          <button
            type="button"
            class="mini-action"
            onclick={addCreateBlock}
            disabled={pipelineAuthoringLoading || !createBlockChoice}
          >
            Add Block
          </button>
        </div>

        {#if createPipelineBlocks.length === 0}
          <p class="empty">Add one or more prompt blocks to define the pipeline order.</p>
        {:else}
          <ol class="pipeline-editor-list">
            {#each createPipelineBlocks as blockId, index (blockId)}
              <li>
                <span>{blockLabel(blockId)}</span>
                <div class="pipeline-step-actions">
                  <button type="button" class="mini-action" onclick={() => moveCreateBlock(index, -1)} disabled={index === 0 || pipelineAuthoringLoading}>↑</button>
                  <button type="button" class="mini-action" onclick={() => moveCreateBlock(index, 1)} disabled={index === createPipelineBlocks.length - 1 || pipelineAuthoringLoading}>↓</button>
                  <button type="button" class="mini-action" onclick={() => removeCreateBlock(index)} disabled={pipelineAuthoringLoading}>Remove</button>
                </div>
              </li>
            {/each}
          </ol>
        {/if}

        <div class="pipeline-form-actions">
          <button type="submit" class="mini-action primary" disabled={pipelineAuthoringLoading || !createPipelineName.trim() || createPipelineBlocks.length === 0}>
            {pipelineAuthoringLoading ? 'Saving…' : 'Save Pipeline'}
          </button>
          <button type="button" class="mini-action" onclick={cancelCreatePipeline} disabled={pipelineAuthoringLoading}>Cancel</button>
        </div>
      </form>
    {/if}

    {#if pipelines.length === 0}
      <p class="empty">No manifest pipelines are defined for this project yet.</p>
    {:else}
      {#each pipelines as pipeline (pipeline.pipelineId)}
        <article class="pipeline-card">
          {#if editingPipelineId === pipeline.pipelineId}
            <form class="pipeline-form" onsubmit={(event) => { event.preventDefault(); void handleUpdatePipelineSubmit(); }}>
              <input
                type="text"
                bind:value={editingPipelineName}
                aria-label={`Pipeline name for ${pipeline.name}`}
                disabled={pipelineAuthoringLoading}
              />
              <div class="pipeline-picker-row">
                <select bind:value={editingBlockChoice} disabled={pipelineAuthoringLoading} aria-label={`Available prompt blocks for ${pipeline.name}`}>
                  <option value="">Select a prompt block</option>
                  {#each promptBlocks.filter((block) => !editingPipelineBlocks.includes(block.blockId)) as block (block.blockId)}
                    <option value={block.blockId}>{block.name}</option>
                  {/each}
                </select>
                <button
                  type="button"
                  class="mini-action"
                  onclick={addEditingBlock}
                  disabled={pipelineAuthoringLoading || !editingBlockChoice}
                >
                  Add Block
                </button>
              </div>

              <ol class="pipeline-editor-list">
                {#each editingPipelineBlocks as blockId, index (blockId)}
                  <li>
                    <span>{blockLabel(blockId)}</span>
                    <div class="pipeline-step-actions">
                      <button type="button" class="mini-action" onclick={() => moveEditingBlock(index, -1)} disabled={index === 0 || pipelineAuthoringLoading}>↑</button>
                      <button type="button" class="mini-action" onclick={() => moveEditingBlock(index, 1)} disabled={index === editingPipelineBlocks.length - 1 || pipelineAuthoringLoading}>↓</button>
                      <button type="button" class="mini-action" onclick={() => removeEditingBlock(index)} disabled={pipelineAuthoringLoading}>Remove</button>
                    </div>
                  </li>
                {/each}
              </ol>

              <div class="pipeline-form-actions">
                <button type="submit" class="mini-action primary" disabled={pipelineAuthoringLoading || !editingPipelineName.trim() || editingPipelineBlocks.length === 0}>
                  {pipelineAuthoringLoading ? 'Saving…' : 'Save Changes'}
                </button>
                <button type="button" class="mini-action" onclick={cancelEditPipeline} disabled={pipelineAuthoringLoading}>Cancel</button>
              </div>
            </form>
          {:else}
            <div class="pipeline-head">
              <div>
                <h3>{pipeline.name}</h3>
                <p class="meta">{pipeline.executionMode} · {pipeline.blocks.length} blocks</p>
              </div>
              <div class="pipeline-card-actions">
                <button
                  type="button"
                  class="mini-action"
                  onclick={() => startEditPipeline(pipeline)}
                  disabled={pipelineLoading || pipelineAuthoringLoading}
                >
                  Edit
                </button>
                <button
                  type="button"
                  class="pipeline-run"
                  aria-label={`Run ${pipeline.name}`}
                  onclick={() => onRunPipeline(pipeline.pipelineId)}
                  disabled={pipelineLoading || pipelineAuthoringLoading}
                >
                  {pipelineLoading && pipelineExecution?.pipelineId === pipeline.pipelineId ? 'Running…' : 'Run pipeline'}
                </button>
              </div>
            </div>

            <ol class="pipeline-steps">
              {#each pipeline.blocks as block (block.blockId)}
                <li>
                  <strong>{block.name}</strong>
                  <span>{block.modelPreset}</span>
                </li>
              {/each}
            </ol>

            {#if pipelineExecution?.pipelineId === pipeline.pipelineId}
              <div class="pipeline-status">
                <p class:failed={pipelineExecution.status === 'failed'} class="meta strong">
                  {pipelineExecution.status === 'success' ? 'Pipeline complete' : 'Pipeline failed'}
                </p>
                <p class="meta">
                  {pipelineExecution.steps.length} / {pipeline.blocks.length} blocks completed
                </p>
                {#if pipelineExecution.error}
                  <p class="meta failed">{pipelineExecution.error}</p>
                {/if}
              </div>
            {/if}
          {/if}
        </article>
      {/each}
    {/if}
  </section>

  <section class="section">
    <div class="history-heading">
      <div>
        <p class="eyebrow">Run History</p>
        <h3>Project Runs</h3>
      </div>
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
    </div>

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
  </section>
</aside>

<style>
  .inspector {
    display: grid;
    gap: 0.85rem;
    padding: 0.8rem;
    height: 100%;
    overflow: auto;
  }

  .section {
    display: grid;
    gap: 0.75rem;
    padding: 0.95rem;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(157, 180, 255, 0.14);
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

  .pipeline-card {
    display: grid;
    gap: 0.75rem;
    padding: 0.85rem;
    border-radius: 14px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
  }

  .pipeline-head {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .section-head,
  .pipeline-card-actions {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .mini-action {
    min-height: 2rem;
    padding: 0.35rem 0.6rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
  }

  .mini-action.primary {
    border-color: rgba(153, 227, 190, 0.28);
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
  }

  .pipeline-run {
    min-height: 2.2rem;
    padding: 0.5rem 0.8rem;
    border-radius: 12px;
    border: 1px solid rgba(153, 227, 190, 0.28);
    color: var(--text);
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
  }

  .pipeline-form,
  .pipeline-picker-row,
  .pipeline-form-actions {
    display: grid;
    gap: 0.6rem;
  }

  .pipeline-form input,
  .pipeline-picker-row select {
    min-height: 2.2rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.45rem 0.6rem;
  }

  .pipeline-editor-list {
    display: grid;
    gap: 0.55rem;
    margin: 0;
    padding-left: 1.2rem;
  }

  .pipeline-editor-list li {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .pipeline-step-actions {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .export-list {
    display: grid;
    gap: 0.55rem;
  }

  .export-item {
    display: grid;
    gap: 0.15rem;
    grid-template-columns: auto 1fr;
    column-gap: 0.55rem;
    align-items: start;
  }

  .export-item span,
  .export-item small {
    grid-column: 2;
  }

  .export-item small {
    color: var(--text-soft);
    word-break: break-word;
  }

  .pipeline-steps {
    display: grid;
    gap: 0.55rem;
    margin: 0;
    padding-left: 1.1rem;
    color: var(--text-dim);
  }

  .pipeline-steps li {
    display: grid;
    gap: 0.15rem;
  }

  .pipeline-steps span,
  .meta {
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .pipeline-status {
    display: grid;
    gap: 0.35rem;
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

  .strong {
    color: var(--text);
  }

  .failed {
    color: var(--danger);
  }
</style>
