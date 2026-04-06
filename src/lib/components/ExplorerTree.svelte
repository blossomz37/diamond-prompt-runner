<script lang="ts">
  import type { ProjectAssetNode } from '$lib/types/project';

  interface Props {
    nodes: ProjectAssetNode[];
    activePath: string | null;
    onSelectPath: (node: ProjectAssetNode) => void | Promise<void>;
    onDeleteDocument?: (relativePath: string) => Promise<void>;
    onRenameDocument?: (relativePath: string, newName: string) => Promise<void>;
  }

  let { nodes, activePath, onSelectPath, onDeleteDocument, onRenameDocument }: Props = $props();

  let openDirs = $state<string[]>([]);
  let hoveredPath = $state<string | null>(null);
  let renamingPath = $state<string | null>(null);
  let renameValue = $state('');
  let deleteDocConfirm = $state<string | null>(null);
  let deleteDocLoading = $state(false);

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

  function isDocumentNode(node: ProjectAssetNode): boolean {
    return (
      !node.isDirectory &&
      (node.kind === 'markdown' || node.kind === 'text') &&
      node.path.startsWith('documents/')
    );
  }

  function startRename(node: ProjectAssetNode): void {
    renamingPath = node.path;
    renameValue = node.name;
    deleteDocConfirm = null;
  }

  function cancelRename(): void {
    renamingPath = null;
    renameValue = '';
  }

  async function confirmRename(relativePath: string): Promise<void> {
    if (!renameValue.trim() || !onRenameDocument) return;
    await onRenameDocument(relativePath, renameValue.trim());
    renamingPath = null;
    renameValue = '';
  }

  function handleRenameKeydown(event: KeyboardEvent, relativePath: string): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      void confirmRename(relativePath);
    } else if (event.key === 'Escape') {
      cancelRename();
    }
  }

  async function handleDeleteDoc(relativePath: string): Promise<void> {
    if (!onDeleteDocument) return;
    if (deleteDocConfirm !== relativePath) {
      deleteDocConfirm = relativePath;
      return;
    }
    deleteDocLoading = true;
    try {
      await onDeleteDocument(relativePath);
      deleteDocConfirm = null;
    } finally {
      deleteDocLoading = false;
    }
  }
</script>

<div class="tree" data-testid="explorer-tree">
  {#snippet renderNode(node: ProjectAssetNode, depth: number)}
    {#if renamingPath === node.path}
      <div class="tree-node rename-row" data-depth={depth}>
        <span class="icon">{iconFor(node)}</span>
        <input
          class="rename-input"
          type="text"
          bind:value={renameValue}
          onkeydown={(e) => handleRenameKeydown(e, node.path)}
          onblur={cancelRename}

          aria-label="Rename document"
        />
        <button
          type="button"
          class="doc-action confirm"
          onmousedown={(e) => { e.preventDefault(); void confirmRename(node.path); }}
          aria-label="Confirm rename"
          title="Confirm"
        >✓</button>
        <button
          type="button"
          class="doc-action cancel"
          onmousedown={(e) => { e.preventDefault(); cancelRename(); }}
          aria-label="Cancel rename"
          title="Cancel"
        >✕</button>
      </div>
    {:else}
      <div
        class="node-row"
        role="group"
        onmouseenter={() => { hoveredPath = node.path; }}
        onmouseleave={() => { if (hoveredPath === node.path) hoveredPath = null; }}
      >
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

        {#if isDocumentNode(node) && (hoveredPath === node.path || deleteDocConfirm === node.path)}
          <div class="doc-actions">
            {#if onRenameDocument}
              <button
                type="button"
                class="doc-action rename"
                onclick={() => startRename(node)}
                aria-label="Rename {node.name}"
                title="Rename"
              >✎</button>
            {/if}
            {#if onDeleteDocument}
              <button
                type="button"
                class="doc-action delete"
                class:danger={deleteDocConfirm === node.path}
                onclick={() => handleDeleteDoc(node.path)}
                disabled={deleteDocLoading}
                aria-label={deleteDocConfirm === node.path ? 'Confirm delete' : 'Delete {node.name}'}
                title={deleteDocConfirm === node.path ? 'Confirm delete' : 'Delete'}
              >{deleteDocConfirm === node.path ? '?' : '✕'}</button>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

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

  .node-row {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    position: relative;
  }

  .tree-node {
    display: grid;
    grid-template-columns: 1.9rem minmax(0, 1fr);
    gap: 0.45rem;
    align-items: center;
    flex: 1;
    min-height: 2rem;
    padding: 0.25rem 0.45rem;
    padding-left: calc(0.55rem + attr(data-depth number) * 0.9rem);
    border-radius: 12px;
    background: transparent;
    color: var(--text-dim);
    text-align: left;
    min-width: 0;
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

  /* Document action buttons */
  .doc-actions {
    display: flex;
    gap: 0.15rem;
    flex-shrink: 0;
    padding-right: 0.3rem;
  }

  .doc-action {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.5rem;
    height: 1.5rem;
    border-radius: 6px;
    border: 1px solid rgba(157, 180, 255, 0.14);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-soft);
    font-size: 0.72rem;
    cursor: pointer;
    line-height: 1;
  }

  .doc-action:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.08);
  }

  .doc-action.danger {
    border-color: rgba(255, 100, 100, 0.35);
    color: var(--danger);
  }

  .doc-action.confirm {
    border-color: rgba(153, 227, 190, 0.3);
    color: #8de8b5;
  }

  /* Rename row */
  .rename-row {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    min-height: 2rem;
    padding: 0.2rem 0.45rem;
    padding-left: calc(0.55rem + attr(data-depth number) * 0.9rem);
    border-radius: 12px;
    background: rgba(139, 177, 255, 0.08);
    border: 1px solid rgba(139, 177, 255, 0.2);
  }

  .rename-row[data-depth='1'] { padding-left: 1.35rem; }
  .rename-row[data-depth='2'] { padding-left: 2.25rem; }
  .rename-row[data-depth='3'] { padding-left: 3.15rem; }
  .rename-row[data-depth='4'] { padding-left: 4.05rem; }

  .rename-input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: none;
    color: var(--text);
    font-size: 0.86rem;
    outline: none;
    padding: 0;
  }
</style>
