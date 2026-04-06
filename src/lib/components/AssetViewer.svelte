<script lang="ts">
  import { marked } from 'marked';
  import { ONLINE_PROMPT_DIRECTIVE, promptUsesOnlineResearch } from '$lib/promptExecution';
  import type { PromptExecutionResult, WorkspaceTab } from '$lib/types/project';
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
    execution: PromptExecutionResult | null;
    executionLoading: boolean;
  }

  let { tab, onDraftChange, onSave, onReload, onExecute, execution, executionLoading }: Props = $props();

  let previewMode = $state(false);
  let editorEl: HTMLTextAreaElement | CodeEditorApi | undefined = $state();
  let findBar: FindBar | undefined = $state();
  let currentTabPath = '';

  // Reset preview mode and close find bar when switching tabs
  $effect(() => {
    if (tab?.path && tab.path !== currentTabPath) {
      currentTabPath = tab.path;
      previewMode = false;
      findBar?.close();
    }
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

  const isMarkdown = $derived(tab?.kind === 'markdown' || tab?.kind === 'tera');

  const renderedMarkdown = $derived(
    isMarkdown && previewMode ? marked.parse(tab!.draftContent ?? tab!.content) : ''
  );

  const onlineEnabled = $derived(
    tab?.kind === 'tera' ? promptUsesOnlineResearch(tab.draftContent) : false
  );
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
        <p class="eyebrow">{previewMode && isMarkdown ? 'Preview' : 'Editing'}</p>
        <h2>{tab.title}</h2>
      </div>
      <div class="actions">
        {#if isMarkdown}
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
    {#if previewMode && isMarkdown}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked with raw HTML disabled -->
      <div class="markdown-preview">{@html renderedMarkdown}</div>
    {:else}
      <FindBar
        bind:this={findBar}
        editorEl={editorEl}
        content={tab.draftContent}
        onContentChange={(newContent) => onDraftChange(tab.path, newContent)}
      />
      {#key tab.path}
        <CodeEditor
          bind:api={editorEl as CodeEditorApi}
          value={tab.draftContent}
          kind={tab.kind}
          onContentChange={(newContent) => onDraftChange(tab.path, newContent)}
          onkeydown={handleEditorKeydown}
        />
      {/key}
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


</style>
