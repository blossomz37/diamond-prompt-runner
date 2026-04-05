<script lang="ts">
  import { marked } from 'marked';
  import { ONLINE_PROMPT_DIRECTIVE, promptUsesOnlineResearch } from '$lib/promptExecution';
  import type { PromptExecutionResult, WorkspaceTab } from '$lib/types/project';

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

  // Find / Replace state
  let showFind = $state(false);
  let showReplace = $state(false);
  let findText = $state('');
  let replaceText = $state('');
  let caseSensitive = $state(false);
  let matchIndex = $state(0);
  let editorEl: HTMLTextAreaElement | undefined = $state();
  let findInputEl: HTMLInputElement | undefined = $state();

  function getMatches(content: string, query: string): number[] {
    if (!query) return [];
    const hay = caseSensitive ? content : content.toLowerCase();
    const needle = caseSensitive ? query : query.toLowerCase();
    const positions: number[] = [];
    let idx = 0;
    while ((idx = hay.indexOf(needle, idx)) !== -1) {
      positions.push(idx);
      idx += 1;
    }
    return positions;
  }

  const matches = $derived(
    tab && findText ? getMatches(tab.draftContent ?? tab.content, findText) : []
  );

  function selectMatch(index: number): void {
    if (!editorEl || matches.length === 0) return;
    const pos = matches[index];
    editorEl.focus();
    editorEl.setSelectionRange(pos, pos + findText.length);
  }

  function findNext(): void {
    if (matches.length === 0) return;
    matchIndex = (matchIndex + 1) % matches.length;
    selectMatch(matchIndex);
  }

  function findPrev(): void {
    if (matches.length === 0) return;
    matchIndex = (matchIndex - 1 + matches.length) % matches.length;
    selectMatch(matchIndex);
  }

  function replaceCurrent(): void {
    if (!tab || matches.length === 0) return;
    const pos = matches[matchIndex];
    const content = tab.draftContent;
    const updated = content.substring(0, pos) + replaceText + content.substring(pos + findText.length);
    onDraftChange(tab.path, updated);
    // After replacement, stay at same index (or clamp)
    if (matchIndex >= matches.length - 1) matchIndex = Math.max(0, matches.length - 2);
    // Queue selection after Svelte updates the textarea
    queueMicrotask(() => selectMatch(matchIndex));
  }

  function replaceAll(): void {
    if (!tab || matches.length === 0 || !findText) return;
    const content = tab.draftContent;
    const flags = caseSensitive ? 'g' : 'gi';
    const escaped = findText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const updated = content.replace(new RegExp(escaped, flags), replaceText);
    onDraftChange(tab.path, updated);
    matchIndex = 0;
  }

  function openFind(): void {
    showFind = true;
    showReplace = false;
    queueMicrotask(() => findInputEl?.focus());
  }

  function openFindReplace(): void {
    showFind = true;
    showReplace = true;
    queueMicrotask(() => findInputEl?.focus());
  }

  function closeFind(): void {
    showFind = false;
    showReplace = false;
    findText = '';
    replaceText = '';
    matchIndex = 0;
  }

  // Jump to first match when search text changes
  $effect(() => {
    if (matches.length > 0) {
      matchIndex = 0;
      selectMatch(0);
    }
  });

  // Reset preview mode when switching tabs
  $effect(() => {
    if (tab?.path) {
      previewMode = false;
      closeFind();
    }
  });

  function handleEditorKeydown(event: KeyboardEvent): void {
    const mod = event.metaKey || event.ctrlKey;
    if (mod && event.key === 'f') {
      event.preventDefault();
      openFind();
    } else if (mod && event.key === 'h') {
      event.preventDefault();
      openFindReplace();
    }
  }

  function handleFindKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      closeFind();
      editorEl?.focus();
    } else if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      findNext();
    } else if (event.key === 'Enter' && event.shiftKey) {
      event.preventDefault();
      findPrev();
    }
  }

  const isMarkdown = $derived(tab?.kind === 'markdown');

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
      {#if showFind}
        <div class="find-bar">
          <div class="find-row">
            <input
              bind:this={findInputEl}
              type="text"
              bind:value={findText}
              placeholder="Find"
              class="find-input"
              onkeydown={handleFindKeydown}
            />
            <span class="match-count">{matches.length > 0 ? `${matchIndex + 1} of ${matches.length}` : findText ? 'No matches' : ''}</span>
            <button type="button" class="find-btn" onclick={findPrev} disabled={matches.length === 0} title="Previous (Shift+Enter)">▲</button>
            <button type="button" class="find-btn" onclick={findNext} disabled={matches.length === 0} title="Next (Enter)">▼</button>
            <button
              type="button"
              class="find-btn case-btn"
              class:active={caseSensitive}
              onclick={() => caseSensitive = !caseSensitive}
              title="Match case"
            >Aa</button>
            {#if !showReplace}
              <button type="button" class="find-btn" onclick={() => showReplace = true} title="Show Replace">⇄</button>
            {/if}
            <button type="button" class="find-btn close-btn" onclick={closeFind} title="Close (Escape)">✕</button>
          </div>
          {#if showReplace}
            <div class="find-row">
              <input
                type="text"
                bind:value={replaceText}
                placeholder="Replace"
                class="find-input"
                onkeydown={handleFindKeydown}
              />
              <button type="button" class="find-btn replace-btn" onclick={replaceCurrent} disabled={matches.length === 0} title="Replace">Replace</button>
              <button type="button" class="find-btn replace-btn" onclick={replaceAll} disabled={matches.length === 0} title="Replace All">All</button>
            </div>
          {/if}
        </div>
      {/if}
      <textarea
        bind:this={editorEl}
        data-testid="asset-editor"
        value={tab.draftContent}
        spellcheck="false"
        oninput={(event) => onDraftChange(tab.path, (event.currentTarget as HTMLTextAreaElement).value)}
        onkeydown={handleEditorKeydown}
      ></textarea>
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

  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
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
    border: 1px solid rgba(153, 227, 190, 0.3);
    background: rgba(153, 227, 190, 0.12);
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
    background: rgba(255, 255, 255, 0.04);
  }

  .primary {
    background: linear-gradient(135deg, rgba(132, 173, 255, 0.28), rgba(85, 113, 204, 0.36));
    border-color: rgba(139, 177, 255, 0.34);
  }

  .primary.run {
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
    border-color: rgba(153, 227, 190, 0.3);
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
    border: 1px solid rgba(157, 180, 255, 0.12);
    color: #dbe5ff;
    white-space: pre-wrap;
    word-break: break-word;
  }

  textarea {
    width: 100%;
    min-height: 30rem;
    flex: 1 1 auto;
    resize: none;
    border-radius: 18px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(5, 8, 15, 0.9);
    color: #dbe5ff;
    padding: 1rem;
    line-height: 1.55;
  }

  .editable {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .editable header {
    flex: 0 0 auto;
  }

  .find-bar {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 0.5rem 0.6rem;
    border-radius: 12px;
    background: rgba(15, 20, 35, 0.92);
    border: 1px solid rgba(157, 180, 255, 0.18);
  }

  .find-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .find-input {
    flex: 1;
    min-height: 1.8rem;
    border-radius: 6px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(7, 11, 20, 0.9);
    color: var(--text);
    padding: 0.25rem 0.5rem;
    font-size: 0.82rem;
    outline: none;
  }

  .find-input:focus {
    border-color: rgba(139, 177, 255, 0.4);
  }

  .match-count {
    font-size: 0.75rem;
    color: var(--text-soft);
    min-width: 5.5em;
    text-align: center;
    white-space: nowrap;
  }

  .find-btn {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(157, 180, 255, 0.14);
    color: var(--text-soft);
    border-radius: 6px;
    padding: 0.2rem 0.45rem;
    font-size: 0.78rem;
    cursor: pointer;
    min-height: 1.8rem;
    line-height: 1;
  }

  .find-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .find-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }

  .case-btn.active {
    background: rgba(132, 173, 255, 0.2);
    border-color: rgba(139, 177, 255, 0.35);
    color: var(--accent);
  }

  .close-btn {
    margin-left: auto;
  }

  .replace-btn {
    font-size: 0.78rem;
    padding: 0.2rem 0.55rem;
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

  .markdown-preview {
    padding: 1rem 1.25rem;
    min-height: 0;
    overflow: auto;
    border-radius: 18px;
    background: rgba(5, 8, 15, 0.78);
    border: 1px solid rgba(157, 180, 255, 0.12);
    color: #dbe5ff;
    line-height: 1.65;
    flex: 1 1 auto;
  }

  .markdown-preview :global(h1),
  .markdown-preview :global(h2),
  .markdown-preview :global(h3),
  .markdown-preview :global(h4) {
    margin: 1.2em 0 0.4em;
    color: var(--text);
  }

  .markdown-preview :global(h1) { font-size: 1.4rem; }
  .markdown-preview :global(h2) { font-size: 1.2rem; }
  .markdown-preview :global(h3) { font-size: 1.05rem; }

  .markdown-preview :global(p) {
    margin: 0.5em 0;
    color: #dbe5ff;
  }

  .markdown-preview :global(ul),
  .markdown-preview :global(ol) {
    margin: 0.5em 0;
    padding-left: 1.6em;
  }

  .markdown-preview :global(li) {
    margin: 0.25em 0;
  }

  .markdown-preview :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 0.8em 0;
    font-size: 0.92em;
  }

  .markdown-preview :global(th),
  .markdown-preview :global(td) {
    padding: 0.45em 0.7em;
    border: 1px solid rgba(157, 180, 255, 0.18);
    text-align: left;
  }

  .markdown-preview :global(th) {
    background: rgba(132, 173, 255, 0.1);
    font-weight: 600;
  }

  .markdown-preview :global(tr:nth-child(even)) {
    background: rgba(255, 255, 255, 0.02);
  }

  .markdown-preview :global(code) {
    padding: 0.15em 0.4em;
    border-radius: 4px;
    background: rgba(132, 173, 255, 0.12);
    font-size: 0.9em;
  }

  .markdown-preview :global(pre) {
    margin: 0.8em 0;
    padding: 0.8rem 1rem;
    border-radius: 12px;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(157, 180, 255, 0.1);
    overflow-x: auto;
  }

  .markdown-preview :global(pre code) {
    padding: 0;
    background: none;
  }

  .markdown-preview :global(blockquote) {
    margin: 0.8em 0;
    padding: 0.4em 1em;
    border-left: 3px solid rgba(132, 173, 255, 0.35);
    color: var(--text-soft);
  }

  .markdown-preview :global(hr) {
    border: none;
    border-top: 1px solid rgba(157, 180, 255, 0.15);
    margin: 1.2em 0;
  }
</style>
