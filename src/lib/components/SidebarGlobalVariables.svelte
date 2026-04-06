<script lang="ts">
  import { GLOBAL_VARIABLES_PATH } from '$lib/types/project';

  interface Props {
    activePath: string | null;
    variableCount: number;
    onOpenGlobalVariables: (path: string, title?: string) => void | Promise<void>;
  }

  let {
    activePath,
    variableCount,
    onOpenGlobalVariables
  }: Props = $props();
</script>

<div class="vars-section">
  <button
    type="button"
    class="var-item"
    class:active={activePath === GLOBAL_VARIABLES_PATH}
    onclick={() => onOpenGlobalVariables(GLOBAL_VARIABLES_PATH, 'global-variables.yaml')}
  >
    <span class="var-name">global-variables.yaml</span>
    <span class="var-meta">{variableCount} vars</span>
  </button>
</div>

<style>
  .var-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    min-height: 2rem;
    padding: 0.25rem 0.55rem;
    border-radius: 12px;
    background: transparent;
    border: none;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
  }

  .var-item:hover {
    background: var(--bg-ghost);
    color: var(--text);
  }

  .var-item.active {
    background: var(--bg-active);
    color: var(--text);
  }

  .var-meta {
    color: var(--text-soft);
    font-size: 0.72rem;
    flex-shrink: 0;
  }
</style>
