import type { PipelineActivityItem } from '$lib/types/project';

const MAX_PIPELINE_ACTIVITY = 6;

function createPipelineActivityStore() {
  let items = $state<PipelineActivityItem[]>([]);

  function push(
    level: PipelineActivityItem['level'],
    message: string,
    detail: string | null = null,
    pipelineId: string | null = null
  ): void {
    items = [
      {
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        pipelineId,
        level,
        message,
        detail,
        timestamp: new Date().toISOString()
      },
      ...items
    ].slice(0, MAX_PIPELINE_ACTIVITY);
  }

  function reset(): void {
    items = [];
  }

  return {
    get items() { return items; },
    push,
    reset
  };
}

export const pipelineActivityStore = createPipelineActivityStore();