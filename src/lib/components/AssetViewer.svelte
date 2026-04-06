<script lang="ts">
  import { marked } from 'marked';
  import { ONLINE_PROMPT_DIRECTIVE, promptUsesOnlineResearch } from '$lib/promptExecution';
  import {
    GLOBAL_VARIABLES_PATH,
    WORKSPACE_VARIABLES_PATH,
    type ExecutionCredentialStatus,
    type PromptExecutionResult,
    type PromptRunHistoryEntry,
    type TemplateValidationResult,
    type WorkspaceTab
  } from '$lib/types/project';
  import FindBar from '$lib/components/FindBar.svelte';
  import CodeEditor, { type CodeEditorApi } from '$lib/components/CodeEditor.svelte';

  // Disable raw HTML passthrough for safety
  marked.use({ renderer: { html: () => '' } });

  interface Props {
    tab: WorkspaceTab | null;
    onDraftChange: (path: string, content: string) => void;
    onSave: (path: string) => void | Promise<void>;
    onReload: (path: string) => void | Promise<void>;
    onExecute: (path: string) => void | Promise<void>;
    validation: TemplateValidationResult | null;
    validationLoading: boolean;
    execution: PromptExecutionResult | null;
    executionLoading: boolean;
    historyItems: PromptRunHistoryEntry[];
    historyLoading: boolean;
    credentialState: ExecutionCredentialStatus;
    onOpenRunPath: (path: string) => void | Promise<void>;
    defaultModelPreset: string;
    onSetDefaultPreset: (presetPath: string) => void | Promise<void>;
    onDeletePreset: (presetPath: string) => void | Promise<void>;
  }

  let {
    tab,
    onDraftChange,
    onSave,
    onReload,
    onExecute,
    validation,
    validationLoading,
    execution,
    executionLoading,
    historyItems,
    historyLoading,
    credentialState,
    onOpenRunPath,
    defaultModelPreset,
    onSetDefaultPreset,
    onDeletePreset
  }: Props = $props();

  let previewMode = $state(false);
  let writingView = $state(false);
  let editorEl: HTMLTextAreaElement | CodeEditorApi | undefined = $state();
  let findBar: FindBar | undefined = $state();
  let currentTabPath = '';
  let deletePresetConfirm = $state(false);
  let modelActionLoading = $state(false);
  let variableFormEntries = $state<VariableEntry[]>([]);
  let variableParseError = $state<string | null>(null);
  let lastVariableDraft = $state('');

  type VariableEntry = {
    id: string;
    name: string;
    value: string;
  };

  // Reset preview mode and close find bar when switching tabs
  $effect(() => {
    if (tab?.path && tab.path !== currentTabPath) {
      currentTabPath = tab.path;
      previewMode = false;
      writingView = false;
      deletePresetConfirm = false;
      findBar?.close();
    }
  });

  $effect(() => {
    if (!tab || !isVariableTab) {
      variableFormEntries = [];
      variableParseError = null;
      lastVariableDraft = '';
      return;
    }

    if (tab.draftContent === lastVariableDraft) {
      return;
    }

    try {
      variableFormEntries = parseVariableEntries(tab.draftContent);
      variableParseError = null;
    } catch (error) {
      variableFormEntries = [];
      variableParseError = error instanceof Error ? error.message : 'Variable content could not be parsed.';
    }

    lastVariableDraft = tab.draftContent;
  });

  function handleEditorKeydown(event: KeyboardEvent): void {
    const mod = event.metaKey || event.ctrlKey;
    if (mod && event.key === 'f') {
      event.preventDefault();
      findBar?.open(false);
    } else if (mod && event.key === 'h') {
      event.preventDefault();
      findBar?.open(true);
    }
  }

  const isMarkdown = $derived(tab?.kind === 'markdown');
  const isPreviewable = $derived(tab?.kind === 'markdown' || tab?.kind === 'tera');

  const renderedMarkdown = $derived(
    tab?.kind === 'markdown' && previewMode ? marked.parse(tab!.draftContent ?? tab!.content) : ''
  );

  const renderedPromptPreview = $derived(
    tab?.kind === 'tera' && previewMode && validation?.preview ? marked.parse(validation.preview) : ''
  );

  const onlineEnabled = $derived(
    tab?.kind === 'tera' ? promptUsesOnlineResearch(tab.draftContent) : false
  );

  const isModelPreset = $derived(tab?.path.startsWith('models/') ?? false);
  const isDefaultModelPreset = $derived(tab?.path === defaultModelPreset);
  const isGlobalVariablesTab = $derived(tab?.path === GLOBAL_VARIABLES_PATH);
  const isWorkspaceVariablesTab = $derived(tab?.path === WORKSPACE_VARIABLES_PATH);
  const isVariableTab = $derived(isGlobalVariablesTab || isWorkspaceVariablesTab);

  const renderedExecutionOutput = $derived(
    execution?.output ? marked.parse(execution.output) : ''
  );

  function variableHeader(scope: 'global' | 'workspace'): string {
    return scope === 'global'
      ? '# App-level global variables\n# Saved outside the project workspace.\n'
      : '# Workspace variables\n# Saved with the current project.\n';
  }

  function parseVariableEntries(content: string): VariableEntry[] {
    const entries: VariableEntry[] = [];
    let index = 0;

    for (const rawLine of content.split(/\r?\n/)) {
      const line = rawLine.trim();
      if (!line || line.startsWith('#')) {
        continue;
      }

      const separator = line.indexOf(':');
      if (separator <= 0) {
        throw new Error(`Invalid variable entry: ${line}`);
      }

      const name = line.slice(0, separator).trim();
      const rawValue = line.slice(separator + 1).trim();
      const parsedValue = rawValue.startsWith('"') ? JSON.parse(rawValue) : rawValue;

      entries.push({
        id: `${name}-${index}`,
        name,
        value: typeof parsedValue === 'string' ? parsedValue : String(parsedValue)
      });
      index += 1;
    }

    return entries;
  }

  function serializeVariableEntries(entries: VariableEntry[], scope: 'global' | 'workspace'): string {
    const body = entries
      .filter((entry) => entry.name.trim().length > 0)
      .map((entry) => `${entry.name.trim()}: ${JSON.stringify(entry.value)}`)
      .join('\n');

    const header = variableHeader(scope);
    return body ? `${header}\n${body}\n` : `${header}\n`;
  }

  function updateVariableEntry(entryId: string, field: 'name' | 'value', nextValue: string): void {
    if (!tab || !isVariableTab) {
      return;
    }

    const scope = isGlobalVariablesTab ? 'global' : 'workspace';
    const nextEntries = variableFormEntries.map((entry) =>
      entry.id === entryId ? { ...entry, [field]: nextValue } : entry
    );

    variableFormEntries = nextEntries;
    onDraftChange(tab.path, serializeVariableEntries(nextEntries, scope));
  }

  function addVariableEntry(): void {
    if (!tab || !isVariableTab) {
      return;
    }

    const nextEntries = [
      ...variableFormEntries,
      { id: `new-${variableFormEntries.length}`, name: '', value: '' }
    ];
    variableFormEntries = nextEntries;
    lastVariableDraft = tab.draftContent;
  }

  function removeVariableEntry(entryId: string): void {
    if (!tab || !isVariableTab) {
      return;
    }

    const scope = isGlobalVariablesTab ? 'global' : 'workspace';
    const nextEntries = variableFormEntries.filter((entry) => entry.id !== entryId);
    variableFormEntries = nextEntries;
    onDraftChange(tab.path, serializeVariableEntries(nextEntries, scope));
  }

  async function handleSetDefaultModelPreset(): Promise<void> {
    if (!tab || modelActionLoading || isDefaultModelPreset) {
      return;
    }

    modelActionLoading = true;
    try {
      await onSetDefaultPreset(tab.path);
    } finally {
      modelActionLoading = false;
    }
  }

  async function handleDeleteModelPreset(): Promise<void> {
    if (!tab || modelActionLoading || isDefaultModelPreset) {
      return;
    }

    if (!deletePresetConfirm) {
      deletePresetConfirm = true;
      return;
    }

    modelActionLoading = true;
    try {
      await onDeletePreset(tab.path);
    } finally {
      modelActionLoading = false;
      deletePresetConfirm = false;
    }
  }
</script>

{#if !tab}
  <section class="blank-state">
    <p class="eyebrow">Workspace</p>
    <h2>Open a project asset</h2>
    <p>Select `project.json`, a document, a `.tera` prompt, or a model preset from the explorer.</p>
  </section>
{:else if tab.view === 'json'}
  <section class="viewer">
    <header>
      <p class="eyebrow">Structured View</p>
      <h2>{tab.title}</h2>
    </header>
    <pre>{JSON.stringify(tab.parsedJson, null, 2)}</pre>
  </section>
{:else if tab.view === 'text' && tab.isEditable}
  <section class="viewer editable">
    <header>
      <div>
        <p class="eyebrow">
          {#if isGlobalVariablesTab}
            App Variables
          {:else if isWorkspaceVariablesTab}
            Workspace Variables
          {:else if previewMode && isMarkdown}
            Preview
          {:else if isModelPreset}
            Model Preset
          {:else}
            Editing
          {/if}
        </p>
        <h2>{tab.title}</h2>
      </div>
      <div class="actions">
        {#if isModelPreset}
          {#if isDefaultModelPreset}
            <span class="online-chip">Default preset</span>
          {:else}
            <button
              type="button"
              class="ghost"
              onclick={() => void handleSetDefaultModelPreset()}
              disabled={modelActionLoading}
            >
              {modelActionLoading ? 'Updating…' : 'Set As Default'}
            </button>
            <button
              type="button"
              class="ghost danger"
              onclick={() => void handleDeleteModelPreset()}
              disabled={modelActionLoading}
            >
              {deletePresetConfirm ? 'Confirm Delete?' : 'Delete Preset'}
            </button>
          {/if}
        {/if}
        {#if isPreviewable}
          <button
            type="button"
            class="ghost toggle-preview"
            class:active={previewMode}
            onclick={() => previewMode = !previewMode}
          >
            {previewMode ? 'Edit' : 'Preview'}
          </button>
        {/if}
        {#if tab.kind === 'tera'}
          {#if !previewMode}
            <button
              type="button"
              class="ghost toggle-preview"
              class:active={writingView}
              onclick={() => writingView = !writingView}
            >
              {writingView ? 'Expand Tera Blocks' : 'Fold Tera Blocks'}
            </button>
          {/if}
          {#if onlineEnabled}
            <span class="online-chip" title={`Opted into online research via ${ONLINE_PROMPT_DIRECTIVE}`}>
              Online enabled
            </span>
          {/if}
          <span class:failed={execution?.status === 'failed'} class="run-status">
            {#if executionLoading}
              Running…
            {:else if execution?.status === 'success'}
              Last run saved
            {:else if execution?.status === 'failed'}
              Run failed
            {:else}
              No run yet
            {/if}
          </span>
          <button type="button" class="primary run" onclick={() => onExecute(tab.path)} disabled={tab.isSaving || executionLoading}>
            {executionLoading ? 'Running…' : 'Run'}
          </button>
        {/if}
        <span class:dirty={tab.draftContent !== tab.savedContent} class="status">
          {tab.draftContent === tab.savedContent ? 'Saved' : 'Unsaved changes'}
        </span>
        <button type="button" class="ghost" onclick={() => onReload(tab.path)} disabled={tab.isSaving}>
          {tab.draftContent === tab.savedContent ? 'Reload From Disk' : 'Discard And Reload'}
        </button>
        <button
          type="button"
          class="primary"
          onclick={() => onSave(tab.path)}
          disabled={tab.isSaving || tab.draftContent === tab.savedContent}
        >
          {tab.isSaving ? 'Saving…' : 'Save'}
        </button>
      </div>
    </header>
    {#if previewMode && tab.kind === 'markdown'}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked with raw HTML disabled -->
      <div class="markdown-preview">{@html renderedMarkdown}</div>
    {:else if previewMode && tab.kind === 'tera'}
      {#if validationLoading}
        <div class="preview-state">
          <p class="eyebrow">Rendered Preview</p>
          <p>Refreshing preview from the current draft…</p>
        </div>
      {:else if validation?.preview}
        <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked with raw HTML disabled -->
        <div class="markdown-preview">{@html renderedPromptPreview}</div>
      {:else}
        <div class="preview-state">
          <p class="eyebrow">Rendered Preview</p>
          <p>No prompt preview is available for the current draft.</p>
        </div>
      {/if}
    {:else if isVariableTab}
      <section class="variable-editor">
        <div class="variable-header">
          <div>
            <p class="eyebrow">Variable Form</p>
            <h3>{variableFormEntries.length} {variableFormEntries.length === 1 ? 'variable' : 'variables'}</h3>
          </div>
          <button type="button" class="ghost" onclick={addVariableEntry}>Add Variable</button>
        </div>

        {#if variableParseError}
          <div class="preview-state">
            <p>{variableParseError}</p>
            <p>Reload this tab to restore the last saved variable form.</p>
          </div>
        {:else if variableFormEntries.length === 0}
          <div class="preview-state">
            <p>No variables yet. Add one to start defining reusable context.</p>
          </div>
        {:else}
          <div class="variable-list">
            {#each variableFormEntries as entry, index (entry.id)}
              <article class="variable-card">
                <div class="variable-card-head">
                  <p class="eyebrow">Variable {index + 1}</p>
                  <button type="button" class="ghost danger remove-variable" onclick={() => removeVariableEntry(entry.id)}>
                    Remove
                  </button>
                </div>
                <label class="variable-field">
                  <span>Name</span>
                  <input
                    type="text"
                    value={entry.name}
                    placeholder="Variable name"
                    aria-label={`Variable name ${index + 1}`}
                    oninput={(event) => updateVariableEntry(entry.id, 'name', (event.currentTarget as HTMLInputElement).value)}
                  />
                </label>
                <label class="variable-field">
                  <span>Value</span>
                  <textarea
                    rows="4"
                    placeholder="Variable value"
                    aria-label={`Variable value ${index + 1}`}
                    value={entry.value}
                    oninput={(event) => updateVariableEntry(entry.id, 'value', (event.currentTarget as HTMLTextAreaElement).value)}
                  ></textarea>
                </label>
              </article>
            {/each}
          </div>
        {/if}
      </section>
    {:else}
      <FindBar
        bind:this={findBar}
        editorEl={editorEl}
        content={tab.draftContent}
        onContentChange={(newContent) => onDraftChange(tab.path, newContent)}
      />
      <div class:writing-view={tab.kind === 'tera' && writingView}>
        {#key tab.path}
          <CodeEditor
            bind:api={editorEl as CodeEditorApi}
            value={tab.draftContent}
            kind={tab.kind}
            foldTeraBlocks={tab.kind === 'tera' && writingView}
            onContentChange={(newContent) => onDraftChange(tab.path, newContent)}
            onkeydown={handleEditorKeydown}
          />
        {/key}
      </div>

      {#if tab.kind === 'tera'}
        <section class="execution-section">
          <div class="execution-header">
            <div>
              <p class="eyebrow">Latest Run</p>
              <h3>{tab.title}</h3>
            </div>
            <span class:failed={execution?.status === 'failed'} class="run-status">
              {#if executionLoading}
                Running…
              {:else if execution?.status === 'success'}
                Last run saved
              {:else if execution?.status === 'failed'}
                Run failed
              {:else if credentialState.hasStoredKey}
                Ready to run
              {:else}
                Add API key in Settings
              {/if}
            </span>
          </div>

          {#if executionLoading}
            <div class="preview-state">
              <p>Waiting for provider response…</p>
            </div>
          {:else if execution?.documentPath && execution?.output == null}
            <div class="preview-state">
              <p>The latest run wrote output directly to {execution.documentPath}.</p>
            </div>
          {:else if execution?.output}
            <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked with raw HTML disabled -->
            <div class="markdown-preview latest-run">{@html renderedExecutionOutput}</div>
          {:else if execution?.error}
            <div class="preview-state">
              <p>{execution.error}</p>
            </div>
          {:else}
            <div class="preview-state">
              <p>Run this prompt to inspect the latest output here.</p>
            </div>
          {/if}

          <div class="history-card">
            <div class="execution-header">
              <div>
                <p class="eyebrow">Recent Runs</p>
                <h3>{historyItems.length}</h3>
              </div>
            </div>

            {#if historyLoading}
              <p class="history-empty">Loading persisted runs…</p>
            {:else if historyItems.length === 0}
              <p class="history-empty">No persisted runs yet for this prompt.</p>
            {:else}
              <div class="history-list">
                {#each historyItems as item (item.runId)}
                  <article class="history-item">
                    <div class="history-head">
                      <strong>{item.runId}</strong>
                      <span class:failed={item.status === 'failed'} class="run-status">{item.status}</span>
                    </div>
                    <p class="history-meta">{item.modelId}</p>
                    {#if item.usage.totalTokens != null || item.usage.cost != null || item.usage.outputWordCount != null}
                      <p class="history-meta">
                        {#if item.usage.totalTokens != null}{item.usage.totalTokens.toLocaleString()} tokens{/if}{#if item.usage.totalTokens != null && item.usage.cost != null} · {/if}{#if item.usage.cost != null}${item.usage.cost.toFixed(4)}{/if}{#if item.usage.outputWordCount != null} · {item.usage.outputWordCount.toLocaleString()} words{/if}
                      </p>
                    {/if}
                    {#if item.outputPreview}
                      <p class="history-preview">{item.outputPreview}</p>
                    {:else if item.error}
                      <p class="history-preview">{item.error}</p>
                    {/if}
                    <button type="button" class="ghost history-open" onclick={() => onOpenRunPath(item.runPath)}>
                      Open Artifact
                    </button>
                  </article>
                {/each}
              </div>
            {/if}
          </div>
        </section>
      {/if}
    {/if}
  </section>
{:else if tab.view === 'text'}
  <section class="viewer">
    <header>
      <p class="eyebrow">{isMarkdown ? 'Preview' : 'Read-only View'}</p>
      <h2>{tab.title}</h2>
    </header>
    {#if isMarkdown}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked with raw HTML disabled -->
      <div class="markdown-preview">{@html marked.parse(tab.content)}</div>
    {:else}
      <pre>{tab.content}</pre>
    {/if}
  </section>
{:else}
  <section class="blank-state">
    <p class="eyebrow">Unsupported</p>
    <h2>{tab.title}</h2>
    <p>{tab.content}</p>
  </section>
{/if}

<style>
  .blank-state,
  .viewer {
    display: grid;
    gap: 0.8rem;
    height: 100%;
    min-height: 0;
    padding: 1rem 1.15rem;
  }

  h2 {
    margin: 0;
    font-size: 1.1rem;
  }

  header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
  }

  .actions {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .status {
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .run-status {
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .online-chip {
    padding: 0.3rem 0.6rem;
    border-radius: 999px;
    border: 1px solid var(--success-border);
    background: var(--success-bg);
    color: var(--success);
    font-size: 0.78rem;
  }

  .run-status.failed {
    color: var(--danger);
  }

  .status.dirty {
    color: var(--accent-strong);
  }

  .ghost,
  .primary {
    min-height: 2.35rem;
    padding: 0.55rem 0.8rem;
    border-radius: 12px;
    border: 1px solid var(--panel-border);
    color: var(--text);
  }

  .ghost {
    background: var(--bg-ghost);
  }

  .ghost.danger {
    color: var(--danger);
  }

  .primary {
    background: var(--gradient-accent);
    border-color: rgba(139, 177, 255, 0.34);
  }

  .primary.run {
    background: var(--gradient-success);
    border-color: var(--success-border);
  }

  p {
    margin: 0;
    color: var(--text-dim);
  }

  pre {
    margin: 0;
    padding: 1rem;
    min-height: 0;
    overflow: auto;
    border-radius: 18px;
    background: rgba(5, 8, 15, 0.78);
    border: 1px solid var(--border-faint);
    color: #dbe5ff;
    white-space: pre-wrap;
    word-break: break-word;
  }


  .editable {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .editable header {
    flex: 0 0 auto;
  }

  .toggle-preview {
    font-size: 0.82rem;
    padding: 0.35rem 0.7rem;
    min-height: unset;
  }

  .toggle-preview.active {
    background: rgba(132, 173, 255, 0.18);
    border-color: rgba(139, 177, 255, 0.34);
  }

  .preview-state {
    display: grid;
    gap: 0.45rem;
    min-height: 0;
    padding: 1rem;
    border-radius: 18px;
    background: rgba(5, 8, 15, 0.78);
    border: 1px solid var(--border-faint);
  }

  .writing-view {
    min-height: 0;
  }

  .execution-section {
    display: grid;
    gap: 0.9rem;
    padding-top: 0.9rem;
    border-top: 1px solid var(--border-faint);
  }

  .execution-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
  }

  .execution-header h3 {
    margin: 0;
    font-size: 0.96rem;
  }

  .latest-run {
    padding: 1rem;
    border-radius: 18px;
    background: rgba(5, 8, 15, 0.78);
    border: 1px solid var(--border-faint);
  }

  .history-card {
    display: grid;
    gap: 0.7rem;
  }

  .history-list {
    display: grid;
    gap: 0.55rem;
  }

  .history-item {
    display: grid;
    gap: 0.35rem;
    padding: 0.8rem;
    border-radius: 14px;
    background: rgba(5, 8, 15, 0.62);
    border: 1px solid var(--border-faint);
  }

  .history-head {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .history-meta,
  .history-empty,
  .history-preview {
    margin: 0;
    color: var(--text-dim);
  }

  .history-preview {
    color: var(--text-soft);
  }

  .history-open {
    justify-self: start;
  }

  .variable-editor {
    display: grid;
    gap: 0.9rem;
  }

  .variable-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
  }

  .variable-header h3 {
    margin: 0;
    font-size: 0.96rem;
  }

  .variable-list {
    display: grid;
    gap: 0.8rem;
  }

  .variable-card {
    display: grid;
    gap: 0.75rem;
    padding: 0.95rem;
    border-radius: 18px;
    background: rgba(5, 8, 15, 0.62);
    border: 1px solid var(--border-faint);
  }

  .variable-card-head {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
  }

  .variable-field {
    display: grid;
    gap: 0.35rem;
  }

  .variable-field span {
    color: var(--text-soft);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .variable-field input,
  .variable-field textarea {
    width: 100%;
    min-width: 0;
    padding: 0.7rem 0.8rem;
    border-radius: 12px;
    border: 1px solid var(--panel-border);
    background: rgba(255, 255, 255, 0.03);
    color: var(--text);
  }

  .variable-field textarea {
    min-height: 8rem;
    resize: vertical;
    font: inherit;
    line-height: 1.45;
  }

  .remove-variable {
    min-height: 0;
  }


</style>
