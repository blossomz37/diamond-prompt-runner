<script lang="ts">
  import type { ModelPresetSummary, ProjectPromptBlock } from '$lib/types/project';

  interface Props {
    promptBlocks: ProjectPromptBlock[];
    modelPresets: ModelPresetSummary[];
    onDeletePromptBlock: (blockId: string) => Promise<void>;
    onSetBlockPreset: (blockId: string, presetPath: string | null) => Promise<void>;
    onSetBlockOutputTarget: (blockId: string, target: string) => Promise<void>;
    onOpenTemplate: (templateSource: string) => void;
  }

  let {
    promptBlocks,
    modelPresets,
    onDeletePromptBlock,
    onSetBlockPreset,
    onSetBlockOutputTarget,
    onOpenTemplate
  }: Props = $props();

  let deleteConfirm = $state<string | null>(null);
  let deleteLoading = $state(false);
  let expandedBlock = $state<string | null>(null);
  let presetSaving = $state(false);
  let targetSaving = $state(false);

  async function handleDelete(blockId: string): Promise<void> {
    if (deleteConfirm !== blockId) {
      deleteConfirm = blockId;
      return;
    }
    deleteLoading = true;
    try {
      await onDeletePromptBlock(blockId);
      deleteConfirm = null;
    } finally {
      deleteLoading = false;
    }
  }

  function toggleExpand(blockId: string): void {
    expandedBlock = expandedBlock === blockId ? null : blockId;
  }

  async function handlePresetChange(event: Event, blockId: string): Promise<void> {
    const select = event.target as HTMLSelectElement;
    presetSaving = true;
    try {
      await onSetBlockPreset(blockId, select.value || null);
    } finally {
      presetSaving = false;
    }
  }

  async function handleTargetChange(event: Event, blockId: string): Promise<void> {
    const select = event.target as HTMLSelectElement;
    targetSaving = true;
    try {
      await onSetBlockOutputTarget(blockId, select.value);
    } finally {
      targetSaving = false;
    }
  }
</script>

<div class="sidebar-blocks">
  {#if promptBlocks.length === 0}
    <p class="empty">No prompt blocks registered yet.</p>
  {:else}
    <div class="block-list">
      {#each promptBlocks as block (block.blockId)}
        <article class="block-card">
          <div class="block-head">
            <button type="button" class="block-name-btn" onclick={() => toggleExpand(block.blockId)}>
              <span class="toggle">{expandedBlock === block.blockId ? '▾' : '▸'}</span>
              <strong>{block.name}</strong>
            </button>
            <div class="block-actions">
              <button
                type="button"
                class="mini-link"
                onclick={() => onOpenTemplate(block.templateSource)}
              >Open</button>
              <button
                type="button"
                class="mini-link"
                class:danger={deleteConfirm === block.blockId}
                onclick={() => handleDelete(block.blockId)}
                disabled={deleteLoading}
                aria-label={deleteConfirm === block.blockId ? `Confirm remove ${block.name}` : `Remove ${block.name}`}
              >{deleteConfirm === block.blockId ? 'Confirm?' : 'Remove'}</button>
            </div>
          </div>

          <p class="block-meta">
            {block.templateSource} · {block.modelPreset || 'project default'} · {block.outputTarget}
          </p>

          {#if expandedBlock === block.blockId}
            <div class="block-settings">
              <div class="setting-row">
                <label for="block-preset-{block.blockId}">Model Override</label>
                <select
                  id="block-preset-{block.blockId}"
                  class="setting-select"
                  value={block.modelPreset}
                  onchange={(e) => handlePresetChange(e, block.blockId)}
                  disabled={presetSaving}
                >
                  <option value="">(Use project default)</option>
                  {#each modelPresets as preset (preset.path)}
                    <option value={preset.path}>{preset.filename} — {preset.modelId}</option>
                  {/each}
                </select>
              </div>
              <div class="setting-row">
                <label for="block-target-{block.blockId}">Output Strategy</label>
                <select
                  id="block-target-{block.blockId}"
                  class="setting-select"
                  value={block.outputTarget || 'history_only'}
                  onchange={(e) => handleTargetChange(e, block.blockId)}
                  disabled={targetSaving}
                >
                  <option value="history_only">Run History JSON only</option>
                  <option value="document">Extracted Text Document</option>
                  <option value="both">Both (Document + Full Run History)</option>
                </select>
              </div>
            </div>
          {/if}
        </article>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar-blocks {
    display: grid;
    gap: 0.55rem;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .block-list {
    display: grid;
    gap: 0.4rem;
  }

  .block-card {
    display: grid;
    gap: 0.35rem;
    padding: 0.55rem 0.65rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
  }

  .block-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.4rem;
  }

  .block-name-btn {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    background: none;
    border: none;
    color: var(--text);
    cursor: pointer;
    padding: 0;
    font-size: 0.86rem;
    text-align: left;
    min-width: 0;
  }

  .block-name-btn strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .toggle {
    color: var(--text-soft);
    font-size: 0.78rem;
    flex-shrink: 0;
  }

  .block-actions {
    display: flex;
    gap: 0.35rem;
    flex-shrink: 0;
  }

  .block-meta {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.76rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .block-settings {
    display: grid;
    gap: 0.5rem;
    padding-top: 0.35rem;
    border-top: 1px solid rgba(157, 180, 255, 0.08);
  }

  .setting-row {
    display: grid;
    gap: 0.25rem;
  }

  .setting-row label {
    font-size: 0.72rem;
    color: var(--text-soft);
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .setting-select {
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

  .setting-select:focus {
    border-color: rgba(139, 177, 255, 0.35);
  }

  .mini-link {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 10.5px;
    padding: 2px 4px;
    border-radius: 4px;
  }

  .mini-link:hover:not(:disabled) {
    color: var(--accent);
  }

  .mini-link.danger {
    color: var(--text-dim);
  }

  .mini-link.danger:hover:not(:disabled) {
    color: var(--danger);
  }
</style>
