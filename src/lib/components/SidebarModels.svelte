<script lang="ts">
  import type { ModelPresetSummary, ProjectSummary } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    presets: ModelPresetSummary[];
    onSetDefaultPreset: (presetPath: string) => Promise<void>;
    onCreatePreset: (filename: string, modelId: string) => Promise<void>;
    onDeletePreset: (presetPath: string) => Promise<void>;
    onOpenPresetFile: (presetPath: string) => void;
  }

  let {
    summary,
    presets,
    onSetDefaultPreset,
    onCreatePreset,
    onDeletePreset,
    onOpenPresetFile
  }: Props = $props();

  let saving = $state(false);
  let error = $state('');
  let newPresetModelId = $state('');
  let newPresetFilename = $state('');
  let deleteConfirm = $state<string | null>(null);

  async function handleDefaultChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const value = select.value;
    if (value === summary.defaultModelPreset) return;
    saving = true;
    error = '';
    try {
      await onSetDefaultPreset(value);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleCreatePreset(): Promise<void> {
    const modelId = newPresetModelId.trim();
    const filename = newPresetFilename.trim() || modelId.replace(/\//g, '-');
    if (!modelId || saving) return;
    saving = true;
    error = '';
    try {
      await onCreatePreset(filename, modelId);
      newPresetModelId = '';
      newPresetFilename = '';
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleDeletePreset(presetPath: string): Promise<void> {
    if (deleteConfirm !== presetPath) {
      deleteConfirm = presetPath;
      return;
    }
    saving = true;
    error = '';
    try {
      await onDeletePreset(presetPath);
      deleteConfirm = null;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function handleCreateKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter') handleCreatePreset();
  }
</script>

<div class="models-section">
  {#if error}
    <p class="models-error">{error}</p>
  {/if}

  <div class="models-group">
    <p class="models-heading">Default Preset</p>
    <select
      class="preset-select"
      value={summary.defaultModelPreset}
      onchange={handleDefaultChange}
      disabled={saving}
      aria-label="Default model preset"
    >
      {#each presets as preset (preset.path)}
        <option value={preset.path}>{preset.filename} — {preset.modelId}</option>
      {/each}
    </select>
  </div>

  <div class="models-group">
    <p class="models-heading">Available Presets <span>{presets.length}</span></p>
    {#if presets.length > 0}
      <ul class="preset-list">
        {#each presets as preset (preset.path)}
          <li class="preset-row">
            <div class="preset-info">
              <span class="preset-filename">{preset.filename}</span>
              <span class="preset-model">{preset.modelId}</span>
            </div>
            <div class="preset-actions">
              <button
                class="mini-link"
                onclick={() => onOpenPresetFile(preset.path)}
                aria-label="Edit {preset.filename}"
              >Edit</button>
              {#if preset.path !== summary.defaultModelPreset}
                <button
                  class="mini-link danger"
                  onclick={() => handleDeletePreset(preset.path)}
                  disabled={saving}
                  aria-label="Delete {preset.filename}"
                >{deleteConfirm === preset.path ? 'Confirm?' : 'Delete'}</button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="models-empty">No presets found.</p>
    {/if}
  </div>

  <div class="models-group">
    <p class="models-heading">New Preset</p>
    <div class="new-preset-form">
      <input
        type="text"
        bind:value={newPresetModelId}
        placeholder="model ID (e.g. anthropic/claude-opus-4)"
        onkeydown={handleCreateKeydown}
        disabled={saving}
        aria-label="New preset model ID"
      />
      <input
        type="text"
        bind:value={newPresetFilename}
        placeholder="filename (optional)"
        onkeydown={handleCreateKeydown}
        disabled={saving}
        aria-label="New preset filename"
      />
      <button
        class="mini-action"
        onclick={handleCreatePreset}
        disabled={saving || !newPresetModelId.trim()}
      >+ Create</button>
    </div>
  </div>
</div>

<style>
  .models-section {
    display: grid;
    gap: 1rem;
  }

  .models-group {
    display: grid;
    gap: 0.4rem;
  }

  .models-heading {
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

  .models-heading span {
    font-weight: 400;
    color: var(--text-soft);
    font-size: 10px;
    letter-spacing: 0;
  }

  .models-error {
    font-size: 11px;
    color: var(--danger);
    margin: 0;
    padding: 4px 7px;
    background: rgba(255, 80, 80, 0.08);
    border-radius: 6px;
  }

  .models-empty {
    font-size: 11px;
    color: var(--text-soft);
    font-style: italic;
    margin: 0;
  }

  .preset-select {
    width: 100%;
    min-height: 2rem;
    border-radius: 8px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
    padding: 0.3rem 0.5rem;
    font-size: 11.5px;
    outline: none;
    cursor: pointer;
  }

  .preset-select:focus {
    border-color: rgba(139, 177, 255, 0.35);
  }

  .preset-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 4px;
  }

  .preset-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 5px 7px;
    border-radius: 8px;
    background: rgba(7, 11, 20, 0.6);
    border: 1px solid var(--panel-border);
    gap: 6px;
  }

  .preset-info {
    display: grid;
    gap: 2px;
    overflow: hidden;
    min-width: 0;
  }

  .preset-filename {
    font-size: 11.5px;
    color: var(--accent);
    font-family: 'SF Mono', 'Fira Code', ui-monospace, monospace;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preset-model {
    font-size: 10px;
    color: var(--text-soft);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preset-actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }

  .mini-link {
    background: none;
    border: none;
    color: var(--text-dim);
    cursor: pointer;
    font-size: 10.5px;
    padding: 2px 4px;
    border-radius: 4px;
  }

  .mini-link:hover:not(:disabled) {
    color: var(--accent);
  }

  .mini-link.danger:hover:not(:disabled) {
    color: var(--danger);
  }

  .new-preset-form {
    display: grid;
    gap: 6px;
  }

  .new-preset-form input {
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

  .new-preset-form input::placeholder {
    color: var(--text-soft);
  }

  .new-preset-form input:focus {
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
    cursor: pointer;
    white-space: nowrap;
  }

  .mini-action:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
  }
</style>
