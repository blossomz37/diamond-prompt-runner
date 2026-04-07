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
    ProjectUsageSummary,
    TemplateValidationResult,
    WorkspaceTab
  } from '$lib/types/project';
  import { buildYamlPromptPaths } from '$lib/utils/yamlPromptPaths';

  interface Props {
    summary: ProjectSummary;
    tab: WorkspaceTab | null;
    metadata: AssetMetadata | null;
    usageSummary: ProjectUsageSummary | null;
    validation: TemplateValidationResult | null;
    validationLoading: boolean;
  }

  let {
    summary,
    tab,
    metadata,
    usageSummary,
    validation,
    validationLoading
  }: Props = $props();

  const yamlPromptPaths = $derived.by(() => {
    if (!tab || tab.kind !== 'yaml' || !tab.path.startsWith('documents/')) {
      return null;
    }

    return buildYamlPromptPaths(tab.path, tab.draftContent);
  });

  const branchPromptText = $derived(yamlPromptPaths?.branches.join('\n') ?? '');
  const leafPromptText = $derived(yamlPromptPaths?.leaves.join('\n') ?? '');

  function rowsFor(text: string): number {
    const lineCount = text ? text.split('\n').length : 1;
    return Math.min(Math.max(lineCount, 2), 8);
  }
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
    {#if tab?.kind === 'tera'}
      <section class="section">
        <div class="section-head">
          <p class="eyebrow">Validation</p>
          {#if validation}
            <span class={`badge ${validation.status}`}>{validation.status}</span>
          {/if}
        </div>

        {#if validationLoading}
          <p class="hint">Refreshing validation from the current draft…</p>
        {:else if validation}
          {#if validation.errors.length > 0}
            <div class="message error">
              {#each validation.errors as error (error)}
                <p>{error}</p>
              {/each}
            </div>
          {/if}

          {#if validation.warnings.length > 0}
            <div class="message warning">
              {#each validation.warnings as warning (warning)}
                <p>{warning}</p>
              {/each}
            </div>
          {/if}

          {#if validation.contextSummary.length > 0}
            <dl>
              {#each validation.contextSummary as item (item.label)}
                <div>
                  <dt>{item.label}</dt>
                  <dd>{item.value}</dd>
                </div>
              {/each}
            </dl>
          {/if}
        {:else}
          <p class="hint">No validation state yet.</p>
        {/if}
      </section>
    {/if}

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

    {#if yamlPromptPaths}
      <section class="section">
        <p class="eyebrow">Prompt Paths</p>
        <dl>
          <div>
            <dt>Alias</dt>
            <dd>{yamlPromptPaths.alias}</dd>
          </div>
        </dl>

        {#if yamlPromptPaths.error}
          <div class="message error">
            <p>{yamlPromptPaths.error}</p>
          </div>
        {:else}
          <div class="prompt-path-group">
            <label class="prompt-path-label" for="yaml-branch-paths">Branches</label>
            <textarea
              id="yaml-branch-paths"
              class="prompt-paths"
              readonly
              aria-label="Branch prompt paths"
              rows={rowsFor(branchPromptText)}
              value={branchPromptText || 'No branch objects found in this YAML document.'}
            ></textarea>
          </div>

          <div class="prompt-path-group">
            <label class="prompt-path-label" for="yaml-leaf-paths">Leaves</label>
            <textarea
              id="yaml-leaf-paths"
              class="prompt-paths"
              readonly
              aria-label="Leaf prompt paths"
              rows={rowsFor(leafPromptText)}
              value={leafPromptText || 'No scalar values found in this YAML document.'}
            ></textarea>
          </div>
        {/if}
      </section>
    {/if}
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

  .section-head {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    align-items: center;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
  }

  .badge {
    border-radius: 999px;
    padding: 0.18rem 0.5rem;
    font-size: 0.72rem;
    text-transform: capitalize;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .badge.valid {
    color: var(--success);
    border-color: var(--success-border);
    background: var(--success-bg);
  }

  .badge.warnings {
    color: #f4c56a;
    border-color: rgba(244, 197, 106, 0.25);
    background: rgba(244, 197, 106, 0.08);
  }

  .badge.invalid {
    color: var(--danger);
    border-color: rgba(255, 141, 161, 0.18);
    background: var(--danger-bg);
  }

  .message {
    display: grid;
    gap: 0.45rem;
    padding: 0.65rem;
    border-radius: 12px;
    border: 1px solid var(--border-faint);
  }

  .message.error {
    background: var(--danger-bg);
    border-color: rgba(255, 141, 161, 0.16);
  }

  .message.warning {
    background: rgba(244, 197, 106, 0.08);
    border-color: rgba(244, 197, 106, 0.16);
  }

  .hint {
    margin: 0;
    color: var(--text-soft);
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

  .prompt-path-group {
    display: grid;
    gap: 0.35rem;
  }

  .prompt-path-label {
    color: var(--text-soft);
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .prompt-paths {
    width: 100%;
    resize: vertical;
    border-radius: 12px;
    border: 1px solid var(--border-faint);
    background: var(--input-bg);
    color: var(--text-dim);
    padding: 0.65rem 0.75rem;
    font: inherit;
    line-height: 1.5;
  }
</style>
