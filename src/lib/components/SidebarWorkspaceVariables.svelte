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
  /* All variable panel styles now in app.css */
</style>
