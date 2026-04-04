<script lang="ts">
  import type {
    AssetMetadata,
    PipelineExecutionResult,
    ProjectPipeline,
    ProjectSummary
  } from '$lib/types/project';

  interface Props {
    summary: ProjectSummary;
    metadata: AssetMetadata | null;
    pipelines: ProjectPipeline[];
    pipelineExecution: PipelineExecutionResult | null;
    pipelineLoading: boolean;
    onRunPipeline: (pipelineId: string) => void | Promise<void>;
  }

  let {
    summary,
    metadata,
    pipelines,
    pipelineExecution,
    pipelineLoading,
    onRunPipeline
  }: Props = $props();
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

  <section class="section">
    <p class="eyebrow">Pipelines</p>
    {#if pipelines.length === 0}
      <p class="empty">No manifest pipelines are defined for this project yet.</p>
    {:else}
      {#each pipelines as pipeline (pipeline.pipelineId)}
        <article class="pipeline-card">
          <div class="pipeline-head">
            <div>
              <h3>{pipeline.name}</h3>
              <p class="meta">{pipeline.executionMode} · {pipeline.blocks.length} blocks</p>
            </div>
            <button
              type="button"
              class="pipeline-run"
              aria-label={`Run ${pipeline.name}`}
              onclick={() => onRunPipeline(pipeline.pipelineId)}
              disabled={pipelineLoading}
            >
              {pipelineLoading && pipelineExecution?.pipelineId === pipeline.pipelineId ? 'Running…' : 'Run pipeline'}
            </button>
          </div>

          <ol class="pipeline-steps">
            {#each pipeline.blocks as block (block.blockId)}
              <li>
                <strong>{block.name}</strong>
                <span>{block.modelPreset}</span>
              </li>
            {/each}
          </ol>

          {#if pipelineExecution?.pipelineId === pipeline.pipelineId}
            <div class="pipeline-status">
              <p class:failed={pipelineExecution.status === 'failed'} class="meta strong">
                {pipelineExecution.status === 'success' ? 'Pipeline complete' : 'Pipeline failed'}
              </p>
              <p class="meta">
                {pipelineExecution.steps.length} / {pipeline.blocks.length} blocks completed
              </p>
              {#if pipelineExecution.error}
                <p class="meta failed">{pipelineExecution.error}</p>
              {/if}
            </div>
          {/if}
        </article>
      {/each}
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

  .pipeline-card {
    display: grid;
    gap: 0.75rem;
    padding: 0.85rem;
    border-radius: 14px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
  }

  .pipeline-head {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .pipeline-run {
    min-height: 2.2rem;
    padding: 0.5rem 0.8rem;
    border-radius: 12px;
    border: 1px solid rgba(153, 227, 190, 0.28);
    color: var(--text);
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
  }

  .pipeline-steps {
    display: grid;
    gap: 0.55rem;
    margin: 0;
    padding-left: 1.1rem;
    color: var(--text-dim);
  }

  .pipeline-steps li {
    display: grid;
    gap: 0.15rem;
  }

  .pipeline-steps span,
  .meta {
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  .pipeline-status {
    display: grid;
    gap: 0.35rem;
  }

  .strong {
    color: var(--text);
  }

  .failed {
    color: var(--danger);
  }
</style>
