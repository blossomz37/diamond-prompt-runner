<script lang="ts">
  import type { AssetMetadata, ProjectSummary } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    metadata: AssetMetadata | null;
  }

  let { summary, metadata }: Props = $props();
</script>

<aside class="inspector">
  <section class="section">
    <p class="eyebrow">Project</p>
    <h3>{summary.projectName}</h3>
    <dl>
      <div>
        <dt>Root</dt>
        <dd>{summary.rootPath}</dd>
      </div>
      <div>
        <dt>Default preset</dt>
        <dd>{summary.defaultModelPreset}</dd>
      </div>
      <div>
        <dt>Assets</dt>
        <dd>{summary.counts.documents} docs / {summary.counts.prompts} prompts / {summary.counts.models} models</dd>
      </div>
    </dl>
  </section>

  <section class="section">
    <p class="eyebrow">Inspector</p>
    {#if metadata}
      <h3>{metadata.name}</h3>
      <dl>
        <div>
          <dt>Kind</dt>
          <dd>{metadata.kind}</dd>
        </div>
        <div>
          <dt>Path</dt>
          <dd>{metadata.path}</dd>
        </div>
        <div>
          <dt>Size</dt>
          <dd>{metadata.sizeBytes === null ? '—' : `${metadata.sizeBytes} bytes`}</dd>
        </div>
        <div>
          <dt>Modified</dt>
          <dd>{metadata.modifiedAt ?? '—'}</dd>
        </div>
        {#each metadata.details as detail (detail.label)}
          <div>
            <dt>{detail.label}</dt>
            <dd>{detail.value}</dd>
          </div>
        {/each}
      </dl>
    {:else}
      <p class="empty">Open a file from the explorer to inspect file metadata here.</p>
    {/if}
  </section>
</aside>

<style>
  .inspector {
    display: grid;
    gap: 0.85rem;
    padding: 0.8rem;
    height: 100%;
    overflow: auto;
  }

  .section {
    display: grid;
    gap: 0.75rem;
    padding: 0.95rem;
    border-radius: 18px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(157, 180, 255, 0.14);
  }

  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
  }

  dl {
    display: grid;
    gap: 0.7rem;
    margin: 0;
  }

  dt {
    margin: 0 0 0.18rem;
    color: var(--text-soft);
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  dd {
    margin: 0;
    color: var(--text-dim);
    word-break: break-word;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
  }
</style>
