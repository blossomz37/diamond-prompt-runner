<script lang="ts">
  import type { ProjectSummary } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    onSetProjectVariables: (variables: Record<string, string>) => Promise<void>;
  }

  let { summary, onSetProjectVariables }: Props = $props();

  let newName = $state('');
  let newValue = $state('');
  let saving = $state(false);

  const projectVariables = $derived(summary.variables);

  async function addVariable(): Promise<void> {
    const name = newName.trim();
    const value = newValue.trim();
    if (!name || saving) return;
    saving = true;
    try {
      await onSetProjectVariables({ ...projectVariables, [name]: value });
      newName = '';
      newValue = '';
    } finally {
      saving = false;
    }
  }

  async function removeVariable(name: string): Promise<void> {
    if (saving) return;
    saving = true;
    try {
      const next = { ...projectVariables };
      delete next[name];
      await onSetProjectVariables(next);
    } finally {
      saving = false;
    }
  }
</script>

<div class="vars-section">
  {#if Object.keys(projectVariables).length > 0}
    <ul class="vars-list">
      {#each Object.entries(projectVariables) as [name, value] (name)}
        <li class="var-row">
          <span class="var-name">{name}</span>
          <span class="var-value">{value}</span>
          <button
            type="button"
            class="var-remove"
            onclick={() => removeVariable(name)}
            disabled={saving}
            aria-label="Remove {name}"
          >✕</button>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="vars-empty">No workspace variables yet.</p>
  {/if}
  <div class="vars-add-form">
    <input
      type="text"
      bind:value={newName}
      placeholder="name"
      aria-label="New workspace variable name"
      disabled={saving}
    />
    <input
      type="text"
      bind:value={newValue}
      placeholder="value"
      aria-label="New workspace variable value"
      disabled={saving}
    />
    <button
      type="button"
      class="mini-action"
      onclick={addVariable}
      disabled={saving || !newName.trim()}
    >+ Add</button>
  </div>
</div>

<style>
  .vars-section {
    display: grid;
    gap: 0.55rem;
  }

  .vars-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }

  .var-row {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 6px;
    align-items: center;
    padding: 5px 7px;
    border-radius: 8px;
    background: rgba(7, 11, 20, 0.6);
    border: 1px solid var(--panel-border);
  }

  .var-name {
    font-size: 11.5px;
    color: var(--accent);
    font-family: 'SF Mono', 'Fira Code', ui-monospace, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .var-value {
    font-size: 11.5px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .var-remove {
    background: none;
    border: none;
    color: var(--text-soft);
    cursor: pointer;
    font-size: 11px;
    padding: 2px 4px;
    border-radius: 4px;
    opacity: 0.6;
    line-height: 1;
  }

  .var-remove:hover:not(:disabled) {
    color: var(--danger);
    opacity: 1;
  }

  .vars-empty {
    font-size: 11px;
    color: var(--text-soft);
    font-style: italic;
    margin: 0;
  }

  .vars-add-form {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 6px;
    align-items: center;
  }

  .vars-add-form input {
    min-height: 2rem;
    border-radius: 8px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.3rem 0.5rem;
    font-size: 11.5px;
    outline: none;
    width: 100%;
  }

  .vars-add-form input::placeholder {
    color: var(--text-soft);
  }

  .vars-add-form input:focus {
    border-color: rgba(139, 177, 255, 0.35);
  }

  .mini-action {
    min-height: 2rem;
    padding: 0.35rem 0.65rem;
    border-radius: 10px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    font-size: 0.82rem;
  }
</style>
