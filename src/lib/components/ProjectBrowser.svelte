<script lang="ts">
  import type { ProjectSummary, RecentProjectEntry } from '$lib/types/project';

  interface Props {
    recentProjects: RecentProjectEntry[];
    projectName: string;
    parentPath: string;
    busy: boolean;
    errorMessage: string | null;
    onProjectNameInput: (value: string) => void;
    onPickParent: () => void | Promise<void>;
    onCreateProject: () => void | Promise<void>;
    onOpenExisting: () => void | Promise<void>;
    onOpenRecent: (project: ProjectSummary) => void | Promise<void>;
    onLocateRecent: (project: RecentProjectEntry) => void | Promise<void>;
    onRemoveRecent: (rootPath: string) => void | Promise<void>;
  }

  let {
    recentProjects,
    projectName,
    parentPath,
    busy,
    errorMessage,
    onProjectNameInput,
    onPickParent,
    onCreateProject,
    onOpenExisting,
    onOpenRecent,
    onLocateRecent,
    onRemoveRecent
  }: Props = $props();
</script>

<section class="browser-shell">
  <div class="hero panel">
    <div>
      <p class="eyebrow">Milestone 1</p>
      <h1>Diamond Prompt Runner</h1>
      <p class="summary">
        Create or open a project, browse the workspace tree, inspect metadata, and open supported
        files in read-only tabs.
      </p>
    </div>
    <div class="hero-note">
      <span class="pill">Tauri 2</span>
      <span class="pill">Svelte 5</span>
      <span class="pill">Read-only shell</span>
    </div>
  </div>

  <div class="browser-grid">
    <section class="panel form-card">
      <header>
        <p class="eyebrow">New Project</p>
        <h2>Start on disk</h2>
      </header>

      <label class="field">
        <span>Name</span>
        <input
          data-testid="project-name-input"
          type="text"
          value={projectName}
          placeholder="Example: Story Lab"
          oninput={(event) => onProjectNameInput((event.currentTarget as HTMLInputElement).value)}
        />
      </label>

      <label class="field">
        <span>Location</span>
        <button class="secondary" type="button" onclick={onPickParent} disabled={busy}>
          {parentPath ? 'Change Parent Folder' : 'Choose Parent Folder'}
        </button>
        <p class="path">{parentPath || 'No folder selected yet.'}</p>
      </label>

      <div class="actions">
        <button
          class="primary"
          type="button"
          onclick={onCreateProject}
          disabled={busy || !projectName.trim() || !parentPath}
        >
          Create Project
        </button>
        <button class="secondary" type="button" onclick={onOpenExisting} disabled={busy}>
          Open Existing Project
        </button>
      </div>

      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}
    </section>

    <section class="panel recents-card">
      <header>
        <p class="eyebrow">Recent Projects</p>
        <h2>Reopen quickly</h2>
      </header>

      {#if recentProjects.length === 0}
        <div class="empty-state">
          <p>No recent projects yet.</p>
          <p class="hint">Create one or open an existing workspace to seed this list.</p>
        </div>
      {:else}
        <div class="recent-list">
          {#each recentProjects as project (project.rootPath)}
            <div class:invalid={!project.lastKnownValid} class="recent-item">
              <div class="recent-heading">
                <strong>{project.projectName}</strong>
                <span class:warning={!project.lastKnownValid}>
                  {project.lastKnownValid ? 'Available' : 'Unavailable'}
                </span>
              </div>
              <p>{project.rootPath}</p>
              <div class="meta-row">
                <span>{project.counts.documents} docs</span>
                <span>{project.counts.prompts} prompts</span>
                <span>{project.defaultModelPreset}</span>
              </div>
              <div class="recent-actions">
                {#if !project.lastKnownValid}
                  <button
                    class="recent-locate"
                    type="button"
                    onclick={() => onLocateRecent(project)}
                    disabled={busy}
                  >
                    Locate
                  </button>
                {/if}
                <button
                  class="recent-open"
                  type="button"
                  onclick={() => onOpenRecent(project)}
                  disabled={busy || !project.lastKnownValid}
                >
                  Open
                </button>
                <button
                  class="recent-remove"
                  type="button"
                  onclick={() => onRemoveRecent(project.rootPath)}
                  disabled={busy}
                >
                  Remove
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  </div>
</section>

<style>
  .browser-shell {
    display: grid;
    gap: 1rem;
    min-height: 100vh;
    padding: 1rem;
  }

  .panel {
    background: linear-gradient(180deg, rgba(17, 23, 40, 0.92), rgba(10, 14, 25, 0.94));
    border: 1px solid var(--panel-border);
    border-radius: 22px;
    box-shadow: var(--shadow);
  }

  .hero {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    padding: 1.3rem 1.4rem;
  }

  .hero h1,
  .form-card h2,
  .recents-card h2 {
    margin: 0;
    font-size: 1.55rem;
  }

  .eyebrow {
    margin: 0 0 0.35rem;
    color: var(--accent);
    font-size: 0.74rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .summary {
    margin: 0.65rem 0 0;
    max-width: 52rem;
    color: var(--text-dim);
  }

  .hero-note {
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .pill {
    padding: 0.38rem 0.72rem;
    border-radius: 999px;
    background: rgba(139, 177, 255, 0.12);
    border: 1px solid rgba(139, 177, 255, 0.24);
    color: var(--accent-strong);
    font-size: 0.82rem;
  }

  .browser-grid {
    display: grid;
    grid-template-columns: minmax(20rem, 26rem) minmax(0, 1fr);
    gap: 1rem;
    min-height: 0;
  }

  .form-card,
  .recents-card {
    display: grid;
    gap: 1rem;
    padding: 1.15rem;
  }

  .field {
    display: grid;
    gap: 0.45rem;
  }

  .field button {
    justify-self: start;
  }

  .field span {
    color: var(--text-dim);
    font-size: 0.86rem;
  }

  input,
  .secondary,
  .primary {
    min-height: 0;
    border-radius: 6px;
    border: 1px solid var(--panel-border);
  }

  input {
    padding: 0.3rem 0.55rem;
    background: rgba(7, 11, 20, 0.82);
    color: var(--text);
  }

  .actions {
    display: grid;
    grid-auto-flow: column;
    grid-auto-columns: max-content;
    gap: 0.5rem;
  }

  .primary,
  .secondary {
    padding: 0.3rem 0.65rem;
    color: var(--text);
  }

  .primary {
    background: linear-gradient(135deg, rgba(132, 173, 255, 0.28), rgba(85, 113, 204, 0.36));
    border-color: rgba(139, 177, 255, 0.34);
  }

  .secondary {
    background: rgba(255, 255, 255, 0.02);
  }

  .path,
  .hint,
  .recent-item p {
    margin: 0;
    color: var(--text-soft);
    word-break: break-word;
  }

  .error {
    margin: 0;
    color: var(--danger);
  }

  .empty-state {
    display: grid;
    place-items: center;
    min-height: 18rem;
    border: 1px dashed var(--panel-border);
    border-radius: 18px;
    color: var(--text-dim);
    text-align: center;
  }

  .recent-list {
    display: grid;
    gap: 0.75rem;
    align-content: start;
  }

  .recent-item {
    display: grid;
    gap: 0.55rem;
    padding: 0.9rem;
    border-radius: 16px;
    text-align: left;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(150, 176, 255, 0.16);
  }

  .recent-item.invalid {
    border-style: dashed;
  }

  .recent-heading,
  .meta-row {
    display: flex;
    justify-content: space-between;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .warning {
    color: var(--danger);
  }

  .meta-row {
    color: var(--text-dim);
    font-size: 0.82rem;
  }

  .recent-actions {
    display: flex;
    gap: 0.55rem;
    justify-content: flex-end;
  }

  .recent-open,
  .recent-locate,
  .recent-remove {
    min-height: 2.2rem;
    padding: 0.5rem 0.8rem;
    border-radius: 12px;
    border: 1px solid var(--panel-border);
    color: var(--text);
  }

  .recent-open {
    background: rgba(139, 177, 255, 0.12);
  }

  .recent-locate {
    background: rgba(153, 227, 190, 0.12);
    border-color: rgba(153, 227, 190, 0.28);
  }

  .recent-remove {
    background: rgba(255, 255, 255, 0.02);
    color: var(--text-dim);
  }

  @media (max-width: 980px) {
    .browser-grid {
      grid-template-columns: 1fr;
    }

    .hero {
      grid-template-columns: 1fr;
      flex-direction: column;
    }

    .hero-note {
      justify-content: flex-start;
    }
  }
</style>
