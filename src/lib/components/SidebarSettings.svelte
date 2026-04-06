<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        SidebarSettings.svelte
  // Description: Collapsible sidebar section for project name and API key status
  // Version:     2.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type { ProjectSummary } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    onRenameProject: (newName: string) => Promise<void>;
  }

  let {
    summary,
    onRenameProject
  }: Props = $props();

  let editingName = $state(false);
  let nameInput = $state('');
  let saving = $state(false);
  let error = $state('');

  function startRename(): void {
    nameInput = summary.projectName;
    editingName = true;
    error = '';
  }

  async function saveRename(): Promise<void> {
    const trimmed = nameInput.trim();
    if (!trimmed || trimmed === summary.projectName) {
      editingName = false;
      return;
    }
    saving = true;
    error = '';
    try {
      await onRenameProject(trimmed);
      editingName = false;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function cancelRename(): void {
    editingName = false;
    error = '';
  }

  function handleRenameKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter') saveRename();
    if (event.key === 'Escape') cancelRename();
  }
</script>

<div class="settings-section">
  {#if error}
    <p class="settings-error">{error}</p>
  {/if}

  <!-- Project Name -->
  <div class="settings-group">
    <p class="settings-heading">Project Name</p>
    {#if editingName}
      <div class="rename-form">
        <input
          type="text"
          bind:value={nameInput}
          onkeydown={handleRenameKeydown}
          disabled={saving}
          aria-label="Project name"
        />
        <button class="mini-action" onclick={saveRename} disabled={saving || !nameInput.trim()}>Save</button>
        <button class="mini-action secondary" onclick={cancelRename} disabled={saving}>Cancel</button>
      </div>
    {:else}
      <div class="name-display">
        <span class="name-value">{summary.projectName}</span>
        <button class="mini-action" onclick={startRename}>Rename</button>
      </div>
    {/if}
  </div>
</div>

<style>
  .settings-section {
    display: grid;
    gap: 1rem;
  }

  .settings-group {
    display: grid;
    gap: 0.4rem;
  }

  .settings-heading {
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



  .settings-error {
    font-size: 11px;
    color: var(--danger);
    margin: 0;
    padding: 4px 7px;
    background: rgba(255, 80, 80, 0.08);
    border-radius: 6px;
  }

  .name-display {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .name-value {
    font-size: 12px;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rename-form {
    display: grid;
    grid-template-columns: 1fr auto auto;
    gap: 6px;
    align-items: center;
  }

  .rename-form input {
    font-size: 11.5px;
    width: 100%;
  }
</style>
