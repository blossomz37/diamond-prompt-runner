<script lang="ts">
  import type { ProjectAssetNode } from '$lib/types/project';

  interface Props {
    nodes: ProjectAssetNode[];
    activePath: string | null;
    defaultModelPreset: string;
    onCreatePreset: (filename: string, modelId: string) => Promise<void>;
    onOpenModel: (path: string, title?: string) => void | Promise<void>;
  }

  let {
    nodes,
    activePath,
    defaultModelPreset,
    onCreatePreset,
    onOpenModel
  }: Props = $props();

  let createOpen = $state(false);
  let newPresetModelId = $state('');
  let newPresetFilename = $state('');
  let creating = $state(false);

  const modelNodes = $derived.by(() => {
    const modelsDir = nodes.find((node) => node.path === 'models' && node.isDirectory);
    return (modelsDir?.children ?? []).filter((node) => !node.isDirectory);
  });

  async function handleCreatePresetSubmit(): Promise<void> {
    const modelId = newPresetModelId.trim();
    const filename = newPresetFilename.trim() || modelId.replace(/\//g, '-');

    if (!modelId || creating) {
      return;
    }

    creating = true;
    try {
      await onCreatePreset(filename, modelId);
      newPresetModelId = '';
      newPresetFilename = '';
      createOpen = false;
    } finally {
      creating = false;
    }
  }
</script>

<div class="models-section">
  <div class="header-row">
    <span>{modelNodes.length} presets</span>
    <button type="button" class="mini-action" onclick={() => (createOpen = !createOpen)}>
      {createOpen ? 'Close' : 'New Preset'}
    </button>
  </div>

  {#if createOpen}
    <form class="create-form" onsubmit={(event) => { event.preventDefault(); void handleCreatePresetSubmit(); }}>
      <input
        type="text"
        bind:value={newPresetModelId}
        placeholder="model ID"
        aria-label="Model preset model ID"
        disabled={creating}
      />
      <input
        type="text"
        bind:value={newPresetFilename}
        placeholder="filename (optional)"
        aria-label="Model preset filename"
        disabled={creating}
      />
      <button type="submit" class="mini-action primary" disabled={creating || !newPresetModelId.trim()}>
        {creating ? 'Creating…' : 'Create'}
      </button>
    </form>
  {/if}

  {#if modelNodes.length === 0}
    <p class="models-empty">No presets found.</p>
  {:else}
    <div class="model-list">
      {#each modelNodes as node (node.path)}
        <button
          type="button"
          class="model-item"
          class:active={activePath === node.path}
          onclick={() => onOpenModel(node.path, node.name)}
        >
          <span class="model-name">{node.name}</span>
          {#if node.path === defaultModelPreset}
            <span class="model-badge">Default</span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .models-section {
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
    gap: 0.45rem;
  }

  .create-form input {
    min-width: 0;
    min-height: 2.2rem;
    border-radius: 12px;
    padding: 0.55rem 0.75rem;
  }

  .models-empty {
    color: var(--text-soft);
    margin: 0;
  }

  .model-list {
    display: grid;
    gap: 0.08rem;
  }

  .model-item {
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

  .model-item:hover {
    background: var(--bg-ghost);
    color: var(--text);
  }

  .model-item.active {
    background: var(--bg-active);
    color: var(--text);
  }

  .model-name {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-badge {
    padding: 0.18rem 0.45rem;
    border-radius: 999px;
    background: rgba(132, 173, 255, 0.14);
    border: 1px solid rgba(139, 177, 255, 0.28);
    color: var(--accent);
    font-size: 0.68rem;
    flex-shrink: 0;
  }
</style>
