<script lang="ts">
  import type { ProjectAssetNode } from '$lib/types/project';

  interface Props {
    nodes: ProjectAssetNode[];
    activePath: string | null;
    onSelectPath: (node: ProjectAssetNode) => void | Promise<void>;
    onCreatePrompt: (name: string) => void | Promise<void>;
    onDeletePrompt: (relativePath: string) => Promise<void>;
    promptCreationLoading: boolean;
  }

  let {
    nodes,
    activePath,
    onSelectPath,
    onCreatePrompt,
    onDeletePrompt,
    promptCreationLoading
  }: Props = $props();

  let createOpen = $state(false);
  let newPromptName = $state('');
  let hoveredPath = $state<string | null>(null);
  let deletePromptConfirm = $state<string | null>(null);
  let deletePromptLoading = $state(false);

  const promptNodes = $derived.by(() => {
    const prompts = nodes.find((n) => n.name === 'prompts' && n.isDirectory);
    return prompts?.children ?? [];
  });

  async function handleCreateSubmit(): Promise<void> {
    const trimmed = newPromptName.trim();
    if (!trimmed) return;
    try {
      await onCreatePrompt(trimmed);
      newPromptName = '';
      createOpen = false;
    } catch {
      // Keep form open for retry
    }
  }

  async function handleDeletePrompt(relativePath: string): Promise<void> {
    if (deletePromptConfirm !== relativePath) {
      deletePromptConfirm = relativePath;
      return;
    }

    deletePromptLoading = true;
    try {
      await onDeletePrompt(relativePath);
      deletePromptConfirm = null;
    } finally {
      deletePromptLoading = false;
    }
  }
</script>

<div class="sidebar-prompts">
  <div class="header-row">
    <span>{promptNodes.length} templates</span>
    <button type="button" class="mini-action" onclick={() => (createOpen = !createOpen)}>
      {createOpen ? 'Close' : 'New Prompt'}
    </button>
  </div>

  {#if createOpen}
    <form class="create-form" onsubmit={(event) => { event.preventDefault(); void handleCreateSubmit(); }}>
      <input
        type="text"
        bind:value={newPromptName}
        placeholder="Prompt name"
        aria-label="Prompt name"
        disabled={promptCreationLoading}
      />
      <button type="submit" class="mini-action primary" disabled={promptCreationLoading || !newPromptName.trim()}>
        {promptCreationLoading ? 'Creating…' : 'Create'}
      </button>
    </form>
  {/if}

  {#if promptNodes.length === 0}
    <p class="empty-state">No .tera templates yet.</p>
  {:else}
    <div class="prompt-list">
      {#each promptNodes as node (node.path)}
        <div
          class="prompt-row"
          role="presentation"
          onmouseenter={() => { hoveredPath = node.path; }}
          onmouseleave={() => {
            if (hoveredPath === node.path) hoveredPath = null;
          }}
        >
          <button
            type="button"
            class="prompt-item"
            class:active={activePath === node.path}
            onclick={() => onSelectPath(node)}
          >
            <span class="icon">TE</span>
            <span class="name">{node.name}</span>
          </button>
          {#if hoveredPath === node.path || deletePromptConfirm === node.path}
            <div class="prompt-actions">
              <button
                type="button"
                class="prompt-action delete"
                class:danger={deletePromptConfirm === node.path}
                onclick={() => handleDeletePrompt(node.path)}
                disabled={deletePromptLoading}
                aria-label={deletePromptConfirm === node.path ? `Confirm delete ${node.name}` : `Delete ${node.name}`}
                title={deletePromptConfirm === node.path ? 'Confirm move to Trash' : 'Move to Trash'}
              >{deletePromptConfirm === node.path ? '?' : '✕'}</button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar-prompts {
    display: grid;
    gap: 0.55rem;
  }

  .header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--text-dim);
    font-size: 0.8rem;
  }

  .create-form {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 0.5rem;
  }

  .create-form input {
    min-width: 0;
    min-height: 2.2rem;
    border-radius: 12px;
    padding: 0.55rem 0.75rem;
  }

  .prompt-list {
    display: grid;
    gap: 0.08rem;
  }

  .prompt-row {
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }

  .prompt-item {
    display: grid;
    grid-template-columns: 1.9rem minmax(0, 1fr);
    gap: 0.45rem;
    align-items: center;
    min-height: 2rem;
    padding: 0.25rem 0.45rem;
    border-radius: 12px;
    background: transparent;
    color: var(--text-dim);
    text-align: left;
    border: none;
    cursor: pointer;
    min-width: 0;
  }

  .prompt-actions {
    display: flex;
    gap: 0.15rem;
    flex-shrink: 0;
    padding-right: 0.3rem;
  }

  .prompt-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 6px;
    border: 1px solid rgba(157, 180, 255, 0.14);
    background: var(--bg-ghost);
    color: var(--text-soft);
    font-size: 0.72rem;
    cursor: pointer;
    line-height: 1;
  }

  .prompt-action:hover {
    color: var(--text);
    background: var(--bg-hover);
  }

  .prompt-action.danger {
    border-color: rgba(255, 100, 100, 0.35);
    color: var(--danger);
  }

  .prompt-item:hover {
    background: var(--bg-ghost);
    color: var(--text);
  }

  .prompt-item.active {
    background: var(--bg-active);
    color: var(--text);
  }

  .icon {
    display: inline-flex;
    justify-content: center;
    font-size: 0.74rem;
    color: var(--accent-strong);
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 0.86rem;
  }

  .mini-action.primary {
    background: var(--bg-active);
    border-color: rgba(139, 177, 255, 0.28);
  }
</style>
