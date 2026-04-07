<script lang="ts">
  import { getMatches, escapeRegex } from '$lib/utils/textSearch';
  import type { CodeEditorApi } from '$lib/components/CodeEditor.svelte';

  interface Props {
    editorEl: HTMLTextAreaElement | CodeEditorApi | undefined;
    content: string;
    onContentChange: (newContent: string) => void;
  }

  let { editorEl, content, onContentChange }: Props = $props();

  let showFind = $state(false);
  let showReplace = $state(false);
  let findText = $state('');
  let replaceText = $state('');
  let caseSensitive = $state(false);
  let matchIndex = $state(0);
  let findInputEl: HTMLInputElement | undefined = $state();

  const matches = $derived(findText ? getMatches(content, findText, caseSensitive) : []);

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
    if (matches.length === 0) return;
    const pos = matches[matchIndex];
    const updated = content.substring(0, pos) + replaceText + content.substring(pos + findText.length);
    onContentChange(updated);
    if (matchIndex >= matches.length - 1) matchIndex = Math.max(0, matches.length - 2);
    queueMicrotask(() => selectMatch(matchIndex));
  }

  function replaceAll(): void {
    if (matches.length === 0 || !findText) return;
    const flags = caseSensitive ? 'g' : 'gi';
    const escaped = escapeRegex(findText);
    const updated = content.replace(new RegExp(escaped, flags), replaceText);
    onContentChange(updated);
    matchIndex = 0;
  }

  // Jump to first match when search text or case sensitivity changes
  $effect(() => {
    if (matches.length > 0) {
      matchIndex = 0;
      selectMatch(0);
    }
  });

  export function open(withReplace: boolean): void {
    showFind = true;
    showReplace = withReplace;
    queueMicrotask(() => findInputEl?.focus());
  }

  export function close(): void {
    showFind = false;
    showReplace = false;
    findText = '';
    replaceText = '';
    matchIndex = 0;
  }

  export function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault();
      close();
      editorEl?.focus();
    } else if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      findNext();
    } else if (event.key === 'Enter' && event.shiftKey) {
      event.preventDefault();
      findPrev();
    }
  }
</script>

{#if showFind}
  <div class="find-bar">
    <div class="find-row">
      <input
        bind:this={findInputEl}
        type="text"
        bind:value={findText}
        placeholder="Find"
        class="find-input"
        onkeydown={handleKeydown}
      />
      <span class="match-count">
        {matches.length > 0 ? `${matchIndex + 1} of ${matches.length}` : findText ? 'No matches' : ''}
      </span>
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
      <button type="button" class="find-btn close-btn" onclick={close} title="Close (Escape)">✕</button>
    </div>
    {#if showReplace}
      <div class="find-row">
        <input
          type="text"
          bind:value={replaceText}
          placeholder="Replace"
          class="find-input"
          onkeydown={handleKeydown}
        />
        <button type="button" class="find-btn replace-btn" onclick={replaceCurrent} disabled={matches.length === 0} title="Replace">Replace</button>
        <button type="button" class="find-btn replace-btn" onclick={replaceAll} disabled={matches.length === 0} title="Replace All">All</button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .find-bar {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 0.5rem 0.6rem;
    border-radius: 12px;
    background: var(--panel);
    border: 1px solid var(--border-subtle);
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
    border: 1px solid var(--border-subtle);
    background: var(--input-bg);
    color: var(--text);
    padding: 0.25rem 0.5rem;
    font-size: 0.82rem;
    outline: none;
  }

  .find-input:focus {
    border-color: var(--border-focus);
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
    border-color: var(--border-focus);
    color: var(--accent);
  }

  .close-btn {
    margin-left: auto;
  }

  .replace-btn {
    font-size: 0.78rem;
    padding: 0.2rem 0.55rem;
  }
</style>
