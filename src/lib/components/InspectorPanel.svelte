<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        InspectorPanel.svelte
  // Description: Read-only right sidebar showing project summary, usage, file metadata, and run history
  // Version:     2.0.0
  // Created:     2026-04-03
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type {
    AssetMetadata,
    ProjectSummary,
    ProjectUsageSummary
  } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    metadata: AssetMetadata | null;
    usageSummary: ProjectUsageSummary | null;
  }

  let {
    summary,
    metadata,
    usageSummary
  }: Props = $props();
</script>

<aside class="inspector">
  {#if !metadata}
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

    {#if usageSummary && usageSummary.totalRuns > 0}
      <section class="section">
        <p class="eyebrow">Usage</p>
        <dl>
          <div>
            <dt>Runs</dt>
            <dd>{usageSummary.successfulRuns} succeeded · {usageSummary.failedRuns} failed</dd>
          </div>
          <div>
            <dt>Total Tokens</dt>
            <dd>{usageSummary.totalTokens.toLocaleString()}</dd>
          </div>
          {#if usageSummary.totalCost > 0}
            <div>
              <dt>Total Cost</dt>
              <dd>${usageSummary.totalCost.toFixed(4)}</dd>
            </div>
          {/if}
          {#if usageSummary.totalOutputWords > 0}
            <div>
              <dt>Output Words</dt>
              <dd>{usageSummary.totalOutputWords.toLocaleString()}</dd>
            </div>
          {/if}
          {#if usageSummary.totalRetries > 0}
            <div>
              <dt>Total Retries</dt>
              <dd>{usageSummary.totalRetries}</dd>
            </div>
          {/if}
        </dl>
      </section>
    {/if}
  {:else}
    <section class="section">
      <p class="eyebrow">File Info</p>
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
    </section>
  {/if}
</aside>

<style>
  .inspector {
    display: grid;
    align-content: start;
    gap: 1.2rem;
    padding: 0.95rem 0.75rem;
    overflow-y: auto;
    font-size: 0.86rem;
    color: var(--text-dim);
  }

  .section {
    display: grid;
    gap: 0.55rem;
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
</style>
