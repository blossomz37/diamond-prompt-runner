<script lang="ts">
  import type {
    ModelPresetSummary,
    ProjectPromptBlock
  } from '$lib/types/project';

  interface Props {
    block: ProjectPromptBlock;
    modelPresets: ModelPresetSummary[];
    onSetBlockPreset: (blockId: string, presetPath: string | null) => Promise<void>;
    onSetBlockOutputTarget: (blockId: string, target: string) => Promise<void>;
    onSetBlockOutputFilename: (blockId: string, filename: string | null) => Promise<void>;
    onDeletePromptBlock: (blockId: string) => Promise<void>;
    onOpenTemplate: (path: string) => void;
    onClose: () => void;
  }

  let {
    block,
    modelPresets,
    onSetBlockPreset,
    onSetBlockOutputTarget,
    onSetBlockOutputFilename,
    onDeletePromptBlock,
    onOpenTemplate,
    onClose
  }: Props = $props();

  let saving = $state(false);
  let deleteConfirm = $state(false);
  let error = $state('');
  // eslint-disable-next-line svelte/prefer-writable-derived -- needs to be writable for bind:value
  let filenameInput = $state('');

  // Keep filenameInput in sync when the block prop refreshes (e.g. after save)
  $effect(() => {
    filenameInput = block.outputFilename ?? '';
  });

  const showsDocument = $derived(
    block.outputTarget === 'replace_document' || block.outputTarget === 'append_document'
    || block.outputTarget === 'document' || block.outputTarget === 'both'
  );

  function slugify(name: string): string {
    return name.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'prompt';
  }

  const resolvedFilename = $derived(
    block.outputFilename || `${slugify(block.name)}.md`
  );

  async function handlePresetChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    const value = select.value;
    saving = true;
    error = '';
    try {
      await onSetBlockPreset(block.blockId, value || null);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleOutputTargetChange(event: Event): Promise<void> {
    const select = event.target as HTMLSelectElement;
    saving = true;
    error = '';
    try {
      await onSetBlockOutputTarget(block.blockId, select.value);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleFilenameBlur(): Promise<void> {
    const trimmed = filenameInput.trim();
    const newValue = trimmed || null;
    if (newValue === (block.outputFilename ?? null)) return;
    saving = true;
    error = '';
    try {
      await onSetBlockOutputFilename(block.blockId, newValue);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function handleDelete(): Promise<void> {
    if (!deleteConfirm) {
      deleteConfirm = true;
      return;
    }
    saving = true;
    error = '';
    try {
      await onDeletePromptBlock(block.blockId);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
      deleteConfirm = false;
    }
  }
</script>

<div class="block-editor">
  <header class="editor-header">
    <p class="eyebrow">Prompt Block</p>
    <h2>{block.name}</h2>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <div class="editor-form">
    <!-- Template Source -->
    <div class="field">
      <span>Template Source</span>
      <div class="template-row">
        <code class="template-path">{block.templateSource}</code>
        <button
          type="button"
          class="action-btn"
          onclick={() => onOpenTemplate(block.templateSource)}
        >Open Template</button>
      </div>
    </div>

    <!-- Model Preset Override -->
    <div class="field">
      <span>Model Preset Override</span>
      <select
        class="field-select"
        value={block.modelPreset}
        onchange={handlePresetChange}
        disabled={saving}
        aria-label="Model preset override"
      >
        <option value="">Use project default</option>
        {#each modelPresets as preset (preset.path)}
          <option value={preset.path}>{preset.filename} — {preset.modelId}</option>
        {/each}
      </select>
      <p class="field-hint">
        {block.modelPreset
          ? `Overrides the project default for this block.`
          : `This block uses the project default preset.`}
      </p>
    </div>

    <!-- Output Target -->
    <div class="field">
      <span>Output Strategy</span>
      <select
        class="field-select"
        value={block.outputTarget}
        onchange={handleOutputTargetChange}
        disabled={saving}
        aria-label="Output strategy"
      >
        <option value="replace_document">Create / Replace Document</option>
        <option value="append_document">Append to Document</option>
        <option value="run_artifact">Run History Only</option>
      </select>
      <p class="field-hint">
        {#if block.outputTarget === 'run_artifact' || block.outputTarget === 'history_only'}
          Output is saved to run history only. Other blocks cannot reference it.
        {:else if block.outputTarget === 'append_document'}
          Output is appended to <code>documents/{resolvedFilename}</code>. Use <code>&lbrace;&lbrace; doc("{resolvedFilename}") &rbrace;&rbrace;</code> in other prompts to reference it.
        {:else}
          Output is written to <code>documents/{resolvedFilename}</code>. Use <code>&lbrace;&lbrace; doc("{resolvedFilename}") &rbrace;&rbrace;</code> in other prompts to reference it.
        {/if}
      </p>
    </div>

    {#if showsDocument}
      <div class="field">
        <span>Output Filename</span>
        <input
          type="text"
          class="field-input"
          placeholder={`${slugify(block.name)}.md`}
          bind:value={filenameInput}
          onblur={handleFilenameBlur}
          disabled={saving}
          aria-label="Output filename"
        />
        <p class="field-hint">
          Leave blank to auto-generate from the block name. The file is always written to <code>documents/</code>.
        </p>
      </div>
    {/if}

    <!-- Danger Zone -->
    <div class="danger-zone">
      <span>Danger Zone</span>
      <button
        type="button"
        class="action-btn danger"
        onclick={handleDelete}
        disabled={saving}
      >
        {deleteConfirm ? 'Confirm Delete?' : 'Delete Block'}
      </button>
    </div>
  </div>
</div>

<style>
  .block-editor {
    display: grid;
    gap: 1.5rem;
    padding: 1.5rem 2rem;
    max-width: 52rem;
    overflow-y: auto;
  }

  .editor-header {
    display: grid;
    gap: 0.25rem;
  }

  h2 {
    margin: 0;
    font-size: 1.15rem;
  }

  .error {
    font-size: 0.82rem;
    color: var(--danger);
    margin: 0;
    padding: 0.5rem 0.7rem;
    background: rgba(255, 80, 80, 0.08);
    border-radius: 8px;
  }

  .editor-form {
    display: grid;
    gap: 1.25rem;
  }

  .field {
    display: grid;
    gap: 0.4rem;
  }

  .field span,
  .danger-zone span {
    color: var(--text-dim);
    font-size: 0.82rem;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  .field-select,
  .field-input {
    min-height: 2.4rem;
    border-radius: 10px;
    padding: 0.5rem 0.7rem;
    font-size: 0.88rem;
  }

  .field-select {
    cursor: pointer;
  }

  .field-input {
    font-family: var(--mono, monospace);
  }

  .field-input::placeholder {
    opacity: 0.6;
  }

  .field-hint {
    margin: 0;
    font-size: 0.76rem;
    color: var(--text-soft);
  }

  .field-hint code {
    font-size: 0.74rem;
    background: rgba(255, 255, 255, 0.06);
    padding: 0.1rem 0.35rem;
    border-radius: 4px;
  }

  .template-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .template-path {
    font-size: 0.85rem;
    color: var(--text);
    background: var(--bg-ghost);
    padding: 0.35rem 0.6rem;
    border-radius: 8px;
    border: 1px solid rgba(157, 180, 255, 0.1);
  }

  .danger-zone {
    display: grid;
    gap: 0.5rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(255, 141, 161, 0.15);
  }
</style>
