<script lang="ts">
  interface Props {
    globalVariables: Record<string, string>;
    projectVariables: Record<string, string>;
    onSetGlobalVariables: (variables: Record<string, string>) => Promise<void>;
  }

  let {
    globalVariables,
    projectVariables,
    onSetGlobalVariables
  }: Props = $props();

  let newName = $state('');
  let newValue = $state('');
  let saving = $state(false);

  async function addVariable(): Promise<void> {
    const name = newName.trim();
    const value = newValue.trim();
    if (!name || saving) return;
    saving = true;
    try {
      await onSetGlobalVariables({ ...globalVariables, [name]: value });
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
      const next = { ...globalVariables };
      delete next[name];
      await onSetGlobalVariables(next);
    } finally {
      saving = false;
    }
  }
</script>

<div class="vars-section">
  {#if Object.keys(globalVariables).length > 0}
    <ul class="vars-list">
      {#each Object.entries(globalVariables) as [name, value] (name)}
        {@const overridden = Object.prototype.hasOwnProperty.call(projectVariables, name)}
        <li class="var-row" class:overridden>
          <span class="var-name">{name}</span>
          <span class="var-value">{overridden ? '(overridden)' : value}</span>
          {#if !overridden}
            <button
              type="button"
              class="var-remove"
              onclick={() => removeVariable(name)}
              disabled={saving}
              aria-label="Remove {name}"
            >✕</button>
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <p class="vars-empty">No global variables yet.</p>
  {/if}
  <div class="vars-add-form">
    <input
      type="text"
      bind:value={newName}
      placeholder="name"
      aria-label="New global variable name"
      disabled={saving}
    />
    <input
      type="text"
      bind:value={newValue}
      placeholder="value"
      aria-label="New global variable value"
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
  /* Variable panel base styles in app.css; local override only */
  .var-row.overridden {
    opacity: 0.4;
  }
</style>
