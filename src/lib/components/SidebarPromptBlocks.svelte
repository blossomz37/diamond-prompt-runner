<script lang="ts">
  import type { ProjectAssetNode, ProjectPromptBlock } from '$lib/types/project';

  interface Props {
    nodes: ProjectAssetNode[];
    promptBlocks: ProjectPromptBlock[];
    activeBlockId: string | null;
    onSelectBlock: (block: ProjectPromptBlock) => void;
    onRegisterPromptBlock: (templateSource: string) => Promise<void>;
  }

  let {
    nodes,
    promptBlocks,
    activeBlockId,
    onSelectBlock,
    onRegisterPromptBlock
  }: Props = $props();

  let registeringPath = $state<string | null>(null);

  const promptTemplates = $derived.by(() => {
    const promptsDir = nodes.find((node) => node.path === 'prompts' && node.isDirectory);
    return (promptsDir?.children ?? []).filter((node) => !node.isDirectory && node.kind === 'tera');
  });

  const unregisteredTemplates = $derived.by(() => {
    const registered = new Set(promptBlocks.map((block) => block.templateSource));
    return promptTemplates.filter((node) => !registered.has(node.path));
  });

  async function handleRegister(templateSource: string): Promise<void> {
    if (registeringPath) {
      return;
    }

    registeringPath = templateSource;
    try {
      await onRegisterPromptBlock(templateSource);
    } finally {
      registeringPath = null;
    }
  }
</script>

<div class="sidebar-blocks">
  {#if unregisteredTemplates.length > 0}
    <section class="register-list">
      <p class="section-label">Available Templates</p>
      {#each unregisteredTemplates as node (node.path)}
        <div class="register-item">
          <span class="block-name">{node.name}</span>
          <button
            type="button"
            class="register-btn"
            onclick={() => void handleRegister(node.path)}
            disabled={registeringPath !== null}
          >
            {registeringPath === node.path ? 'Registering…' : 'Register'}
          </button>
        </div>
      {/each}
    </section>
  {/if}

  {#if promptBlocks.length === 0}
    <p class="empty-state">No prompt blocks registered yet.</p>
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
    gap: 0.55rem;
  }

  .register-list {
    display: grid;
    gap: 0.35rem;
    padding-bottom: 0.35rem;
    border-bottom: 1px solid var(--border-faint);
  }

  .section-label {
    margin: 0;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--text-soft);
  }

  .register-item {
    display: flex;
    justify-content: space-between;
    gap: 0.45rem;
    align-items: center;
  }

  .register-btn {
    min-height: 0;
    padding: 0.28rem 0.55rem;
    border-radius: 8px;
    font-size: 0.74rem;
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
    background: var(--bg-ghost);
    color: var(--text);
  }

  .block-item.active {
    background: var(--bg-active);
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
