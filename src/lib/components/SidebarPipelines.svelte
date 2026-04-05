<script lang="ts">
  // ──────────────────────────────────────────────
  // File:        SidebarPipelines.svelte
  // Description: Slim clickable pipeline list for the sidebar; editing happens in the center pane
  // Version:     2.0.0
  // Created:     2026-04-04
  // Modified:    2026-04-05
  // Author:      Diamond Runner
  // ──────────────────────────────────────────────
  import type { ProjectPipeline, ProjectPromptBlock } from '$lib/types/project';

  interface Props {
    pipelines: ProjectPipeline[];
    promptBlocks: ProjectPromptBlock[];
    activePipelineId: string | null;
    pipelineAuthoringLoading: boolean;
    onSelectPipeline: (pipeline: ProjectPipeline) => void;
    onNewPipeline: () => void;
  }

  let {
    pipelines,
    promptBlocks,
    activePipelineId,
    pipelineAuthoringLoading,
    onSelectPipeline,
    onNewPipeline
  }: Props = $props();
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
    <div class="pipeline-list">
      {#each pipelines as pipeline (pipeline.pipelineId)}
        <button
          type="button"
          class="pipeline-item"
          class:active={activePipelineId === pipeline.pipelineId}
          onclick={() => onSelectPipeline(pipeline)}
        >
          <span class="pipeline-name">{pipeline.name}</span>
          <span class="pipeline-meta">{pipeline.blocks.length} blocks</span>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .sidebar-pipelines {
    display: grid;
    gap: 0.25rem;
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

  .pipeline-list {
    display: grid;
    gap: 0.08rem;
  }

  .pipeline-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    min-height: 2rem;
    padding: 0.25rem 0.55rem;
    border-radius: 12px;
    background: transparent;
    border: none;
    color: var(--text-dim);
    text-align: left;
    cursor: pointer;
  }

  .pipeline-item:hover {
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
  }

  .pipeline-item.active {
    background: rgba(139, 177, 255, 0.14);
    color: var(--text);
  }

  .pipeline-name {
    font-size: 0.84rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pipeline-meta {
    font-size: 0.72rem;
    color: var(--text-soft);
    flex-shrink: 0;
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
</style>
