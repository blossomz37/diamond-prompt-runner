import type { ProjectAssetNode, PromptExecutionResult } from '$lib/types/project';

export function findAssetNode(nodes: ProjectAssetNode[], path: string): ProjectAssetNode | null {
  for (const node of nodes) {
    if (node.path === path) return node;
    if (node.isDirectory && node.children.length > 0) {
      const found = findAssetNode(node.children, path);
      if (found) return found;
    }
  }
  return null;
}

export function latestStepForPath(
  steps: PromptExecutionResult[],
  path: string
): PromptExecutionResult | null {
  for (let index = steps.length - 1; index >= 0; index -= 1) {
    if (steps[index]?.path === path) {
      return steps[index];
    }
  }
  return null;
}
