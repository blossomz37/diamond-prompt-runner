<script lang="ts">
  import type { ProjectAssetNode } from '$lib/types/project';

  interface Props {
    nodes: ProjectAssetNode[];
    activePath: string | null;
    onSelectPath: (node: ProjectAssetNode) => void | Promise<void>;
  }

  let { nodes, activePath, onSelectPath }: Props = $props();

  let openDirs = $state<string[]>([]);

  function toggleDir(path: string): void {
    if (openDirs.includes(path)) {
      openDirs = openDirs.filter((entry) => entry !== path);
    } else {
      openDirs = [...openDirs, path];
    }
  }

  function iconFor(node: ProjectAssetNode): string {
    if (node.isDirectory) {
      return openDirs.includes(node.path) ? '▾' : '▸';
    }

    switch (node.kind) {
      case 'manifest':
        return '{}';
      case 'markdown':
        return 'MD';
      case 'tera':
        return 'TE';
      case 'yaml':
        return 'YM';
      case 'text':
        return 'TX';
      default:
        return '•';
    }
  }
</script>

<div class="tree" data-testid="explorer-tree">
  {#snippet renderNode(node: ProjectAssetNode, depth: number)}
    <button
      class:active={activePath === node.path}
      class="tree-node"
      data-depth={depth}
      type="button"
      onclick={() => node.isDirectory ? toggleDir(node.path) : onSelectPath(node)}
    >
      <span class="icon">{iconFor(node)}</span>
      <span>{node.name}</span>
    </button>

    {#if node.children.length > 0 && openDirs.includes(node.path)}
      <div class="children">
        {#each node.children as child (child.path)}
          {@render renderNode(child, depth + 1)}
        {/each}
      </div>
    {/if}
  {/snippet}

  {#each nodes as node (node.path)}
    {@render renderNode(node, 0)}
  {/each}
</div>

<style>
  .tree {
    display: grid;
    gap: 0.08rem;
  }

  .tree-node {
    display: grid;
    grid-template-columns: 1.9rem minmax(0, 1fr);
    gap: 0.45rem;
    align-items: center;
    min-height: 2rem;
    padding: 0.25rem 0.45rem;
    padding-left: calc(0.55rem + attr(data-depth number) * 0.9rem);
    border-radius: 12px;
    background: transparent;
    color: var(--text-dim);
    text-align: left;
  }

  .tree-node:hover {
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
  }

  .tree-node.active {
    background: rgba(139, 177, 255, 0.14);
    color: var(--text);
  }

  .icon {
    display: inline-flex;
    justify-content: center;
    font-size: 0.74rem;
    color: var(--accent-strong);
  }

  .children {
    display: grid;
  }

  .tree-node[data-depth='1'] {
    padding-left: 1.35rem;
  }

  .tree-node[data-depth='2'] {
    padding-left: 2.25rem;
  }

  .tree-node[data-depth='3'] {
    padding-left: 3.15rem;
  }

  .tree-node[data-depth='4'] {
    padding-left: 4.05rem;
  }
</style>
