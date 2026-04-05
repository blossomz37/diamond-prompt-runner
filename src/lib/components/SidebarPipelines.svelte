<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        SidebarPipelines.svelte
  // Description: Collapsible sidebar section showing pipeline list with Run and Edit access
  // Version:     1.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-04
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type {
    PipelineExecutionResult,
    ProjectPipeline,
    ProjectPromptBlock
  } from '$lib/types/project';

  interface Props {
    pipelines: ProjectPipeline[];
    promptBlocks: ProjectPromptBlock[];
    pipelineExecution: PipelineExecutionResult | null;
    pipelineLoading: boolean;
    pipelineAuthoringLoading: boolean;
    onRunPipeline: (pipelineId: string) => void | Promise<void>;
    onEditPipeline: (pipeline: ProjectPipeline) => void;
    onNewPipeline: () => void;
    onDeletePipeline: (pipelineId: string) => Promise<void>;
  }

  let {
    pipelines,
    promptBlocks,
    pipelineExecution,
    pipelineLoading,
    pipelineAuthoringLoading,
    onRunPipeline,
    onEditPipeline,
    onNewPipeline,
    onDeletePipeline
  }: Props = $props();

  let deletePipelineConfirm = $state<string | null>(null);
  let deletePipelineLoading = $state(false);

  async function handleDeletePipeline(pipelineId: string): Promise<void> {
    if (deletePipelineConfirm !== pipelineId) {
      deletePipelineConfirm = pipelineId;
      return;
    }
    deletePipelineLoading = true;
    try {
      await onDeletePipeline(pipelineId);
      deletePipelineConfirm = null;
    } finally {
      deletePipelineLoading = false;
    }
  }
</script>

<div class="sidebar-pipelines">
  <div class="header-row">
    <button
      type="button"
      class="mini-action"
      onclick={onNewPipeline}
      disabled={pipelineAuthoringLoading || promptBlocks.length === 0}
    >
      New Pipeline
    </button>
  </div>

  {#if promptBlocks.length === 0}
    <p class="empty">Create at least one prompt block before authoring a pipeline.</p>
  {:else if pipelines.length === 0}
    <p class="empty">No pipelines defined yet.</p>
  {:else}
    {#each pipelines as pipeline (pipeline.pipelineId)}
      <article class="pipeline-card">
        <div class="pipeline-head">
          <div>
            <h4>{pipeline.name}</h4>
            <p class="meta">{pipeline.executionMode} · {pipeline.blocks.length} blocks</p>
          </div>
          <div class="pipeline-card-actions">
            <button
              type="button"
              class="mini-action"
              onclick={() => onEditPipeline(pipeline)}
              disabled={pipelineLoading || pipelineAuthoringLoading || deletePipelineLoading}
            >
              Edit
            </button>
            <button
              type="button"
              class="mini-action"
              class:danger={deletePipelineConfirm === pipeline.pipelineId}
              aria-label={deletePipelineConfirm === pipeline.pipelineId ? `Confirm delete ${pipeline.name}` : `Delete ${pipeline.name}`}
              onclick={() => handleDeletePipeline(pipeline.pipelineId)}
              disabled={pipelineLoading || pipelineAuthoringLoading || deletePipelineLoading}
            >
              {deletePipelineConfirm === pipeline.pipelineId ? 'Confirm?' : 'Delete'}
            </button>
            <button
              type="button"
              class="pipeline-run"
              aria-label="Run {pipeline.name}"
              onclick={() => { deletePipelineConfirm = null; onRunPipeline(pipeline.pipelineId); }}
              disabled={pipelineLoading || pipelineAuthoringLoading || deletePipelineLoading}
            >
              {pipelineLoading && pipelineExecution?.pipelineId === pipeline.pipelineId ? 'Running…' : 'Run'}
            </button>
          </div>
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
</div>

<style>
  .sidebar-pipelines {
    display: grid;
    gap: 0.65rem;
  }

  .header-row {
    display: flex;
    justify-content: flex-end;
  }

  .empty {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.82rem;
  }

  h4 {
    margin: 0;
    font-size: 0.88rem;
  }

  .pipeline-card {
    display: grid;
    gap: 0.65rem;
    padding: 0.7rem;
    border-radius: 12px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
  }

  .pipeline-head {
    display: flex;
    justify-content: space-between;
    gap: 0.55rem;
    align-items: flex-start;
  }

  .pipeline-card-actions {
    display: flex;
    gap: 0.35rem;
    align-items: center;
    flex-shrink: 0;
  }

  .pipeline-steps {
    display: grid;
    gap: 0.35rem;
    margin: 0;
    padding-left: 1.1rem;
    color: var(--text-dim);
    font-size: 0.82rem;
  }

  .pipeline-steps li {
    display: grid;
    gap: 0.1rem;
  }

  .pipeline-steps span {
    color: var(--text-soft);
    font-size: 0.78rem;
  }

  .pipeline-status {
    display: grid;
    gap: 0.25rem;
  }

  .meta {
    margin: 0;
    color: var(--text-soft);
    font-size: 0.78rem;
  }

  .strong {
    color: var(--text);
  }

  .failed {
    color: var(--danger);
  }

  .mini-action {
    min-height: 0;
    padding: 0.3rem 0.55rem;
    border-radius: 8px;
    border: 1px solid rgba(157, 180, 255, 0.16);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    font-size: 0.78rem;
  }

  .pipeline-run {
    min-height: 0;
    padding: 0.3rem 0.55rem;
    border-radius: 8px;
    border: 1px solid rgba(153, 227, 190, 0.28);
    color: var(--text);
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
    font-size: 0.78rem;
  }

  .mini-action.danger {
    border-color: rgba(255, 100, 100, 0.35);
    color: var(--danger);
  }
</style>
