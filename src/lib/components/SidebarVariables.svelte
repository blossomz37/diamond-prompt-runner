<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        SidebarVariables.svelte
  // Description: Collapsible sidebar section for editing global and project variables
  // Version:     1.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type { ProjectSummary } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    globalVariables: Record<string, string>;
    onSetGlobalVariables: (variables: Record<string, string>) => Promise<void>;
    onSetProjectVariables: (variables: Record<string, string>) => Promise<void>;
  }

  let {
    summary,
    globalVariables,
    onSetGlobalVariables,
    onSetProjectVariables
  }: Props = $props();

  let newGlobalName = $state('');
  let newGlobalValue = $state('');
  let newProjectName = $state('');
  let newProjectValue = $state('');
  let variablesSaving = $state(false);

  const projectVariables = $derived(summary.variables);

  async function addGlobalVariable(): Promise<void> {
    const name = newGlobalName.trim();
    const value = newGlobalValue.trim();
    if (!name || variablesSaving) return;
    variablesSaving = true;
    try {
      await onSetGlobalVariables({ ...globalVariables, [name]: value });
      newGlobalName = '';
      newGlobalValue = '';
    } finally {
      variablesSaving = false;
    }
  }

  async function removeGlobalVariable(name: string): Promise<void> {
    if (variablesSaving) return;
    variablesSaving = true;
    try {
      const next = { ...globalVariables };
      delete next[name];
      await onSetGlobalVariables(next);
    } finally {
      variablesSaving = false;
    }
  }

  async function addProjectVariable(): Promise<void> {
    const name = newProjectName.trim();
    const value = newProjectValue.trim();
    if (!name || variablesSaving) return;
    variablesSaving = true;
    try {
      await onSetProjectVariables({ ...projectVariables, [name]: value });
      newProjectName = '';
      newProjectValue = '';
    } finally {
      variablesSaving = false;
    }
  }

  async function removeProjectVariable(name: string): Promise<void> {
    if (variablesSaving) return;
    variablesSaving = true;
    try {
      const next = { ...projectVariables };
      delete next[name];
      await onSetProjectVariables(next);
    } finally {
      variablesSaving = false;
    }
  }
</script>

<div class="vars-section">
  <div class="vars-scope">
    <p class="vars-scope-heading">Global <span>all projects</span></p>
    {#if Object.keys(globalVariables).length > 0}
      <ul class="vars-list">
        {#each Object.entries(globalVariables) as [name, value] (name)}
          <li class="var-row {Object.prototype.hasOwnProperty.call(projectVariables, name) ? 'overridden' : ''}">
            <span class="var-name">{name}</span>
            <span class="var-value">{Object.prototype.hasOwnProperty.call(projectVariables, name) ? '(overridden)' : value}</span>
            {#if !Object.prototype.hasOwnProperty.call(projectVariables, name)}
              <button
                type="button"
                class="var-remove"
                onclick={() => removeGlobalVariable(name)}
                disabled={variablesSaving}
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
        bind:value={newGlobalName}
        placeholder="name"
        aria-label="New global variable name"
        disabled={variablesSaving}
      />
      <input
        type="text"
        bind:value={newGlobalValue}
        placeholder="value"
        aria-label="New global variable value"
        disabled={variablesSaving}
      />
      <button
        type="button"
        class="mini-action"
        onclick={addGlobalVariable}
        disabled={variablesSaving || !newGlobalName.trim()}
      >+ Add</button>
    </div>
  </div>

  <div class="vars-scope">
    <p class="vars-scope-heading">Project <span>{summary.projectName}</span></p>
    {#if Object.keys(projectVariables).length > 0}
      <ul class="vars-list">
        {#each Object.entries(projectVariables) as [name, value] (name)}
          <li class="var-row">
            <span class="var-name">{name}</span>
            <span class="var-value">{value}</span>
            <button
              type="button"
              class="var-remove"
              onclick={() => removeProjectVariable(name)}
              disabled={variablesSaving}
              aria-label="Remove {name}"
            >✕</button>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="vars-empty">No project variables yet.</p>
    {/if}
    <div class="vars-add-form">
      <input
        type="text"
        bind:value={newProjectName}
        placeholder="name"
        aria-label="New project variable name"
        disabled={variablesSaving}
      />
      <input
        type="text"
        bind:value={newProjectValue}
        placeholder="value"
        aria-label="New project variable value"
        disabled={variablesSaving}
      />
      <button
        type="button"
        class="mini-action"
        onclick={addProjectVariable}
        disabled={variablesSaving || !newProjectName.trim()}
      >+ Add</button>
    </div>
  </div>
</div>

<style>
  .vars-section {
    display: grid;
    gap: 1rem;
  }

  .vars-scope {
    display: grid;
    gap: 0.55rem;
  }

  .vars-scope-heading {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.04em;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--panel-border);
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin: 0;
  }

  .vars-scope-heading span {
    font-weight: 400;
    color: var(--text-soft);
    font-size: 10px;
    letter-spacing: 0;
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

  .var-row.overridden {
    opacity: 0.4;
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
