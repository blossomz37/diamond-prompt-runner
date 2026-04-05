<script lang="ts">
  import type { ProjectAssetNode } from '$lib/types/project';
  import ExplorerTree from './ExplorerTree.svelte';

  interface Props {
    nodes: ProjectAssetNode[];
    activePath: string | null;
    onSelectPath: (node: ProjectAssetNode) => void | Promise<void>;
  }

  let { nodes, activePath, onSelectPath }: Props = $props();

  const helpNodes = $derived.by(() => {
    const helpDir = nodes.find((n) => n.isDirectory && n.name === 'help');
    return helpDir ? helpDir.children : [];
  });
</script>

<div class="sidebar-help">
  {#if helpNodes.length === 0}
    <p class="empty">No help files yet.</p>
  {:else}
    <ExplorerTree
      nodes={helpNodes}
      {activePath}
      {onSelectPath}
    />
  {/if}
</div>

<style>
  .sidebar-help {
    display: grid;
    gap: 0.25rem;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }
</style>
