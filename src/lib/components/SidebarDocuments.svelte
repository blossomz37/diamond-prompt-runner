<script lang="ts">
  import type { ProjectAssetNode } from '$lib/types/project';
  import ExplorerTree from './ExplorerTree.svelte';

  interface Props {
    nodes: ProjectAssetNode[];
    activePath: string | null;
    onSelectPath: (node: ProjectAssetNode) => void | Promise<void>;
    onDeleteDocument?: (relativePath: string) => Promise<void>;
    onRenameDocument?: (relativePath: string, newName: string) => Promise<void>;
  }

  let { nodes, activePath, onSelectPath, onDeleteDocument, onRenameDocument }: Props = $props();

  const documentNodes = $derived.by(() => {
    const docsDir = nodes.find((n) => n.isDirectory && n.name === 'documents');
    return docsDir ? docsDir.children : [];
  });
</script>

<div class="sidebar-documents">
  {#if documentNodes.length === 0}
    <p class="empty">No documents yet.</p>
  {:else}
    <ExplorerTree
      nodes={documentNodes}
      {activePath}
      {onSelectPath}
      {onDeleteDocument}
      {onRenameDocument}
    />
  {/if}
</div>

<style>
  .sidebar-documents {
    display: grid;
    gap: 0.25rem;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }
</style>
