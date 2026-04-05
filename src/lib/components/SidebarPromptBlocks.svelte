<script lang="ts">
  import type { ProjectPromptBlock } from '$lib/types/project';

  interface Props {
    promptBlocks: ProjectPromptBlock[];
    activeBlockId: string | null;
    onSelectBlock: (block: ProjectPromptBlock) => void;
  }

  let {
    promptBlocks,
    activeBlockId,
    onSelectBlock
  }: Props = $props();
</script>

<div class="sidebar-blocks">
  {#if promptBlocks.length === 0}
    <p class="empty">No prompt blocks registered yet.</p>
  {:else}
    <div class="block-list">
      {#each promptBlocks as block (block.blockId)}
        <button
          type="button"
          class="block-item"
          class:active={activeBlockId === block.blockId}
          onclick={() => onSelectBlock(block)}
        >
          <span class="block-name">{block.name}</span>
          <span class="block-meta">{block.templateSource.split('/').pop()}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar-blocks {
    display: grid;
    gap: 0.25rem;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .block-list {
    display: grid;
    gap: 0.08rem;
  }

  .block-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    min-height: 2rem;
    padding: 0.25rem 0.55rem;
    border-radius: 12px;
    background: transparent;
    border: none;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
  }

  .block-item:hover {
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
  }

  .block-item.active {
    background: rgba(139, 177, 255, 0.14);
    color: var(--text);
  }

  .block-name {
    font-size: 0.84rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .block-meta {
    font-size: 0.72rem;
    color: var(--text-soft);
    flex-shrink: 0;
  }
</style>
