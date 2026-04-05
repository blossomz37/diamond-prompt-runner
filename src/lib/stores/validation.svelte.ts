import { validateProjectTemplate } from '$lib/tauri';
import type { TemplateValidationResult, WorkspaceTab } from '$lib/types/project';

export function createValidationStore(
  getTabs: () => WorkspaceTab[],
  getActivePath: () => string | null,
  getRootPath: () => string | null
) {
  let result = $state<TemplateValidationResult | null>(null);
  let loading = $state(false);
  let requestId = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;

  async function run(rootPath: string, path: string, content: string): Promise<void> {
    const thisRequestId = ++requestId;

    try {
      const validationResult = await validateProjectTemplate(rootPath, path, content);
      if (thisRequestId !== requestId) return;
      result = validationResult;
    } catch (error) {
      if (thisRequestId !== requestId) return;
      result = {
        path,
        status: 'invalid',
        preview: null,
        warnings: [],
        errors: [error instanceof Error ? error.message : 'Template validation failed.'],
        contextSummary: []
      };
    } finally {
      if (thisRequestId === requestId) {
        loading = false;
      }
    }
  }

  $effect(() => {
    const currentTab = getTabs().find((tab) => tab.path === getActivePath()) ?? null;
    const rootPath = getRootPath();

    if (timer) {
      clearTimeout(timer);
      timer = null;
    }

    if (!rootPath || !currentTab || currentTab.kind !== 'tera') {
      requestId += 1;
      loading = false;
      result = null;
      return;
    }

    loading = true;
    const path = currentTab.path;
    const content = currentTab.draftContent;

    timer = setTimeout(() => {
      void run(rootPath, path, content);
    }, 250);

    return () => {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    };
  });

  function reset(): void {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    requestId += 1;
    result = null;
    loading = false;
  }

  return {
    get result() { return result; },
    get loading() { return loading; },
    reset
  };
}
