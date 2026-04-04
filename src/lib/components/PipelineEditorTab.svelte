<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        PipelineEditorTab.svelte
  // Description: Center-pane pipeline create/edit form opened as a virtual tab
  // Version:     1.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type {
    ProjectPipeline,
    ProjectPromptBlock,
    SavedPipelineResult
  } from '$lib/types/project';

  interface Props {
    /** null for create mode, pipeline object for edit mode */
    existingPipeline: ProjectPipeline | null;
    promptBlocks: ProjectPromptBlock[];
    loading: boolean;
    onSave: (name: string, orderedBlockIds: string[], existingPipelineId: string | null) => Promise<SavedPipelineResult>;
    onCancel: () => void;
  }

  let {
    existingPipeline,
    promptBlocks,
    loading,
    onSave,
    onCancel
  }: Props = $props();

  let pipelineName = $state('');
  let pipelineBlocks = $state<string[]>([]);
  let blockChoice = $state('');

  // Sync editor state when the target pipeline changes.
  $effect(() => {
    pipelineName = existingPipeline?.name ?? '';
    pipelineBlocks = existingPipeline?.blocks.map((b) => b.blockId) ?? [];
    blockChoice = '';
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
  }
</script>

<div class="pipeline-editor">
  <header class="editor-header">
    <p class="eyebrow">{isEdit ? 'Edit Pipeline' : 'New Pipeline'}</p>
    <h2>{isEdit ? existingPipeline!.name : 'Create Pipeline'}</h2>
  </header>

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
      <p class="empty">Add one or more prompt blocks to define the pipeline order.</p>
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
      <button type="button" class="action-btn" onclick={onCancel} disabled={loading}>Cancel</button>
    </div>
  </form>
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

  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
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
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.5rem 0.7rem;
    font-size: 0.88rem;
  }

  .field input:focus,
  .field select:focus {
    border-color: rgba(139, 177, 255, 0.35);
    outline: none;
  }

  .block-picker {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.6rem;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
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
    border: 1px solid rgba(157, 180, 255, 0.12);
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

  .action-btn {
    min-height: 2.2rem;
    padding: 0.4rem 0.75rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    font-size: 0.82rem;
    cursor: pointer;
  }

  .action-btn.sm {
    min-height: 0;
    padding: 0.25rem 0.5rem;
    font-size: 0.78rem;
    border-radius: 8px;
  }

  .action-btn.primary {
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
    border-color: rgba(153, 227, 190, 0.28);
  }

  .action-btn.danger {
    border-color: rgba(255, 141, 161, 0.2);
  }

  .action-btn.danger:hover:not(:disabled) {
    background: rgba(255, 141, 161, 0.12);
    color: var(--danger);
  }

  .action-btn:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .form-actions {
    display: flex;
    gap: 0.6rem;
    padding-top: 0.5rem;
  }
</style>
