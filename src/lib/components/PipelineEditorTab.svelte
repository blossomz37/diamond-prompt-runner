<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        PipelineEditorTab.svelte
  // Description: Center-pane pipeline viewer/editor with run, export, and delete actions
  // Version:     2.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-05
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type {
    ExportBundleResult,
    PipelineExecutionResult,
    ProjectPipeline,
    ProjectPromptBlock,
    SavedPipelineResult
  } from '$lib/types/project';

  interface Props {
    /** null for create mode, pipeline object for edit mode */
    existingPipeline: ProjectPipeline | null;
    promptBlocks: ProjectPromptBlock[];
    loading: boolean;
    pipelineExecution: PipelineExecutionResult | null;
    pipelineLoading: boolean;
    onSave: (name: string, orderedBlockIds: string[], existingPipelineId: string | null) => Promise<SavedPipelineResult>;
    onCancel: () => void;
    onRunPipeline: (pipelineId: string) => void | Promise<void>;
    onDeletePipeline: (pipelineId: string) => Promise<void>;
    onExportPipeline: (bundleName: string, relativePaths: string[]) => Promise<ExportBundleResult>;
  }

  let {
    existingPipeline,
    promptBlocks,
    loading,
    pipelineExecution,
    pipelineLoading,
    onSave,
    onCancel,
    onRunPipeline,
    onDeletePipeline,
    onExportPipeline
  }: Props = $props();

  let pipelineName = $state('');
  let pipelineBlocks = $state<string[]>([]);
  let blockChoice = $state('');
  let editing = $state(false);

  // Sync editor state when the target pipeline changes.
  $effect(() => {
    pipelineName = existingPipeline?.name ?? '';
    pipelineBlocks = existingPipeline?.blocks.map((b) => b.blockId) ?? [];
    blockChoice = '';
    // Start in edit mode when creating a new pipeline
    editing = existingPipeline === null;
  });

  const isEdit = $derived(existingPipeline !== null);

  $effect(() => {
    if (!blockChoice || pipelineBlocks.includes(blockChoice)) {
      blockChoice =
        promptBlocks.find((block) => !pipelineBlocks.includes(block.blockId))?.blockId ?? '';
    }
  });

  function blockLabel(blockId: string): string {
    const block = promptBlocks.find((item) => item.blockId === blockId);
    return block ? `${block.name} (${block.blockId})` : blockId;
  }

  function addBlock(): void {
    if (!blockChoice || pipelineBlocks.includes(blockChoice)) return;
    pipelineBlocks = [...pipelineBlocks, blockChoice];
  }

  function removeBlock(index: number): void {
    pipelineBlocks = pipelineBlocks.filter((_, i) => i !== index);
  }

  function moveBlock(index: number, direction: -1 | 1): void {
    const nextIndex = index + direction;
    if (nextIndex < 0 || nextIndex >= pipelineBlocks.length) return;
    const next = [...pipelineBlocks];
    [next[index], next[nextIndex]] = [next[nextIndex], next[index]];
    pipelineBlocks = next;
  }

  async function handleSubmit(): Promise<void> {
    await onSave(
      pipelineName.trim(),
      pipelineBlocks,
      existingPipeline?.pipelineId ?? null
    );
    editing = false;
  }

  // ── Export ────────────────────────────────────
  let exportLoading = $state(false);
  let exportError = $state('');
  let exportSuccess = $state('');

  async function handleExport(): Promise<void> {
    if (!existingPipeline || exportLoading) return;
    exportLoading = true;
    exportError = '';
    exportSuccess = '';
    const slug = existingPipeline.name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    const bundleName = `pipeline-${slug}`;
    const paths = existingPipeline.blocks.map((b) => b.templateSource).filter(Boolean);
    try {
      await onExportPipeline(bundleName, paths);
      exportSuccess = `Exported to exports/${bundleName}/`;
    } catch (e) {
      exportError = e instanceof Error ? e.message : 'Export failed.';
    } finally {
      exportLoading = false;
    }
  }

  // ── Delete ────────────────────────────────────
  let deleteConfirm = $state(false);
  let deleteLoading = $state(false);

  async function handleDelete(): Promise<void> {
    if (!existingPipeline) return;
    if (!deleteConfirm) {
      deleteConfirm = true;
      return;
    }
    deleteLoading = true;
    try {
      await onDeletePipeline(existingPipeline.pipelineId);
    } finally {
      deleteLoading = false;
      deleteConfirm = false;
    }
  }

  // Reset confirm when the target pipeline changes
  $effect(() => {
    if (existingPipeline) deleteConfirm = false;
  });

  const thisExecution = $derived(
    existingPipeline && pipelineExecution?.pipelineId === existingPipeline.pipelineId
      ? pipelineExecution
      : null
  );
</script>

<div class="pipeline-editor">
  <header class="editor-header">
    <div class="header-top">
      <div>
        <p class="eyebrow">{isEdit ? 'Pipeline' : 'New Pipeline'}</p>
        <h2>{isEdit ? existingPipeline!.name : 'Create Pipeline'}</h2>
        {#if isEdit}
          <p class="meta">{existingPipeline!.executionMode} · {existingPipeline!.blocks.length} blocks</p>
        {/if}
      </div>
      {#if isEdit}
        <div class="header-actions">
          <button
            type="button"
            class="action-btn"
            onclick={() => { editing = !editing; }}
            disabled={pipelineLoading || loading || deleteLoading}
          >
            {editing ? 'Cancel Edit' : 'Edit'}
          </button>
          <button
            type="button"
            class="action-btn"
            onclick={handleExport}
            disabled={pipelineLoading || loading || deleteLoading || exportLoading}
          >
            {exportLoading ? '…' : 'Export'}
          </button>
          <button
            type="button"
            class="action-btn"
            class:danger={deleteConfirm}
            onclick={handleDelete}
            disabled={pipelineLoading || loading || deleteLoading}
          >
            {deleteConfirm ? 'Confirm Delete?' : 'Delete'}
          </button>
          <button
            type="button"
            class="action-btn run"
            onclick={() => onRunPipeline(existingPipeline!.pipelineId)}
            disabled={pipelineLoading || loading || deleteLoading || editing}
          >
            {pipelineLoading && thisExecution ? 'Running…' : 'Run Pipeline'}
          </button>
        </div>
      {/if}
    </div>
    {#if exportError}
      <p class="meta failed">{exportError}</p>
    {/if}
    {#if exportSuccess}
      <p class="meta success">{exportSuccess}</p>
    {/if}
    {#if thisExecution}
      <div class="pipeline-status">
        <p class:failed={thisExecution.status === 'failed'} class="meta strong">
          {thisExecution.status === 'success' ? 'Pipeline complete' : 'Pipeline failed'}
        </p>
        <p class="meta">
          {thisExecution.steps.length} / {existingPipeline!.blocks.length} blocks completed
        </p>
        {#if thisExecution.error}
          <p class="meta failed">{thisExecution.error}</p>
        {/if}
      </div>
    {/if}
  </header>

  {#if editing}
  <form class="editor-form" onsubmit={(event) => { event.preventDefault(); void handleSubmit(); }}>
    <label class="field">
      <span>Pipeline Name</span>
      <input
        type="text"
        bind:value={pipelineName}
        placeholder="e.g. Full Draft Pipeline"
        aria-label="Pipeline name"
        disabled={loading}
      />
    </label>

    <div class="field">
      <span>Prompt Blocks</span>
      <div class="block-picker">
        <select bind:value={blockChoice} disabled={loading} aria-label="Available prompt blocks">
          <option value="">Select a prompt block</option>
          {#each promptBlocks.filter((block) => !pipelineBlocks.includes(block.blockId)) as block (block.blockId)}
            <option value={block.blockId}>{block.name}</option>
          {/each}
        </select>
        <button
          type="button"
          class="action-btn"
          onclick={addBlock}
          disabled={loading || !blockChoice}
        >
          Add Block
        </button>
      </div>
    </div>

    {#if pipelineBlocks.length === 0}
      <p class="empty-state">Add one or more prompt blocks to define the pipeline order.</p>
    {:else}
      <ol class="block-list">
        {#each pipelineBlocks as blockId, index (blockId)}
          <li class="block-item">
            <span class="block-label">{blockLabel(blockId)}</span>
            <div class="block-actions">
              <button type="button" class="action-btn sm" onclick={() => moveBlock(index, -1)} disabled={index === 0 || loading}>↑</button>
              <button type="button" class="action-btn sm" onclick={() => moveBlock(index, 1)} disabled={index === pipelineBlocks.length - 1 || loading}>↓</button>
              <button type="button" class="action-btn sm danger" onclick={() => removeBlock(index)} disabled={loading}>Remove</button>
            </div>
          </li>
        {/each}
      </ol>
    {/if}

    <div class="form-actions">
      <button
        type="submit"
        class="action-btn primary"
        disabled={loading || !pipelineName.trim() || pipelineBlocks.length === 0}
      >
        {#if loading}
          Saving…
        {:else}
          {isEdit ? 'Save Changes' : 'Save Pipeline'}
        {/if}
      </button>
      <button type="button" class="action-btn" onclick={() => { if (isEdit) editing = false; else onCancel(); }} disabled={loading}>Cancel</button>
    </div>
  </form>
  {:else if existingPipeline}
    <div class="view-steps">
      <h3>Pipeline Steps</h3>
      <ol class="step-list">
        {#each existingPipeline.blocks as block, index (block.blockId)}
          <li class="step-item">
            <span class="step-number">{index + 1}</span>
            <div class="step-info">
              <span class="step-name">{block.name}</span>
              <span class="step-preset">{block.modelPreset}</span>
            </div>
          </li>
        {/each}
      </ol>
    </div>
  {/if}
</div>

<style>
  .pipeline-editor {
    display: grid;
    gap: 1.5rem;
    padding: 1.5rem 2rem;
    max-width: 52rem;
    overflow-y: auto;
  }

  .editor-header {
    display: grid;
    gap: 0.25rem;
  }

  .header-top {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
  }

  .header-actions {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    flex-shrink: 0;
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

  .success {
    color: var(--accent, #99e3be);
  }

  .pipeline-status {
    display: grid;
    gap: 0.25rem;
    margin-top: 0.5rem;
    padding: 0.6rem 0.85rem;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(157, 180, 255, 0.1);
  }

  h2 {
    margin: 0;
    font-size: 1.15rem;
  }

  .editor-form {
    display: grid;
    gap: 1.25rem;
  }

  .field {
    display: grid;
    gap: 0.4rem;
  }

  .field span {
    color: var(--text-dim);
    font-size: 0.82rem;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  .field input,
  .field select {
    min-height: 2.4rem;
    border-radius: 10px;
    padding: 0.5rem 0.7rem;
    font-size: 0.88rem;
  }

  .block-picker {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.6rem;
  }

  .empty-state {
    font-style: italic;
  }

  .block-list {
    display: grid;
    gap: 0.5rem;
    margin: 0;
    padding: 0;
    list-style: none;
    counter-reset: step;
  }

  .block-item {
    counter-increment: step;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 0.85rem;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid var(--border-faint);
  }

  .block-label::before {
    content: counter(step) ". ";
    color: var(--text-soft);
  }

  .block-actions {
    display: flex;
    gap: 0.35rem;
    flex-shrink: 0;
  }

  .form-actions {
    display: flex;
    gap: 0.6rem;
    padding-top: 0.5rem;
  }

  /* ── View mode ──────────────────────────────── */
  .view-steps h3 {
    margin: 0 0 0.65rem;
    font-size: 0.88rem;
    color: var(--text-dim);
    letter-spacing: 0.04em;
  }

  .step-list {
    display: grid;
    gap: 0.35rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .step-item {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.55rem 0.85rem;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(157, 180, 255, 0.1);
  }

  .step-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 50%;
    background: var(--bg-active);
    color: var(--text-dim);
    font-size: 0.72rem;
    font-weight: 600;
    flex-shrink: 0;
  }

  .step-info {
    display: grid;
    gap: 0.1rem;
  }

  .step-name {
    font-size: 0.84rem;
    color: var(--text);
  }

  .step-preset {
    font-size: 0.72rem;
    color: var(--text-soft);
  }
</style>
