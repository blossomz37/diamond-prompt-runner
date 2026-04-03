<script lang="ts">
  import type { WorkspaceTab } from '$lib/types/project';

  interface Props {
    tab: WorkspaceTab | null;
    onDraftChange: (path: string, content: string) => void;
    onSave: (path: string) => void | Promise<void>;
    onReload: (path: string) => void | Promise<void>;
  }

  let { tab, onDraftChange, onSave, onReload }: Props = $props();
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
        <p class="eyebrow">Editing</p>
        <h2>{tab.title}</h2>
      </div>
      <div class="actions">
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
    <textarea
      data-testid="asset-editor"
      value={tab.draftContent}
      spellcheck="false"
      oninput={(event) => onDraftChange(tab.path, (event.currentTarget as HTMLTextAreaElement).value)}
    ></textarea>
  </section>
{:else if tab.view === 'text'}
  <section class="viewer">
    <header>
      <p class="eyebrow">Read-only View</p>
      <h2>{tab.title}</h2>
    </header>
    <pre>{tab.content}</pre>
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
    min-height: 14rem;
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
</style>
