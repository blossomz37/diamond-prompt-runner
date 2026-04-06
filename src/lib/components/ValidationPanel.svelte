<script lang="ts">
  import { marked } from 'marked';
  import { ONLINE_PROMPT_DIRECTIVE, promptUsesOnlineResearch } from '$lib/promptExecution';
  import type {
    ExecutionCredentialStatus,
    PromptExecutionResult,
    PromptRunHistoryEntry,
    TemplateValidationResult,
    WorkspaceTab
  } from '$lib/types/project';

  interface Props {
    tab: WorkspaceTab | null;
    validation: TemplateValidationResult | null;
    loading: boolean;
    execution: PromptExecutionResult | null;
    executionLoading: boolean;
    credentialState: ExecutionCredentialStatus;
    recentRuns: PromptRunHistoryEntry[];
    recentRunsLoading: boolean;
    onOpenRunPath: (path: string) => void | Promise<void>;
    onExecute: (path: string) => void | Promise<void>;
    showValidation?: boolean;
    showPreview?: boolean;
  }

  let {
    tab,
    validation,
    loading,
    execution,
    executionLoading,
    credentialState,
    recentRuns,
    recentRunsLoading,
    onOpenRunPath,
    onExecute,
    showValidation = true,
    showPreview = true
  }: Props = $props();

  function tone(status: TemplateValidationResult['status']): string {
    switch (status) {
      case 'valid':
        return 'good';
      case 'warnings':
        return 'warn';
      default:
        return 'bad';
    }
  }

  function executionTone(status: PromptExecutionResult['status']): string {
    return status === 'success' ? 'good' : 'bad';
  }



  const onlineDraftEnabled = $derived(
    tab?.kind === 'tera' ? promptUsesOnlineResearch(tab.draftContent) : false
  );

  const parsedPreview = $derived(
    validation?.preview ? marked.parse(validation.preview) : ''
  );

  const parsedOutput = $derived(
    execution?.output ? marked.parse(execution.output) : ''
  );
</script>

{#if !tab}
  <p class="empty-state">Open a project asset to inspect validation and preview state here.</p>
{:else if tab.kind !== 'tera'}
  <p class="empty-state">Template preview and validation are available for `.tera` prompt templates in this slice.</p>
{:else}
  <div class="panel-grid">
    <div class="summary">
      {#if showValidation}
        <div class="heading">
          <div>
            <p class="eyebrow">Validation</p>
            <h3>{tab.title}</h3>
          </div>
          {#if validation}
            <span class={`badge ${tone(validation.status)}`}>{validation.status}</span>
          {/if}
        </div>

        {#if loading}
          <p class="empty-state">Refreshing validation from the current draft…</p>
        {:else if validation}
          {#if onlineDraftEnabled}
            <div class="messages info">
              <h4>Online Research Mode</h4>
              <p>This prompt opts into web-backed execution via the <code>{ONLINE_PROMPT_DIRECTIVE}</code> directive on its first non-empty line. During a run, OpenRouter will issue live web search requests and inject citations into the context before generating the response.</p>
              <p><strong>Constraints:</strong> Online runs consume additional tokens for search context and cost more than standard runs. Citation count and quality depend on the model and search results — they are not guaranteed. The <code>:online</code> directive routes execution through OpenRouter's online feature; it does not call a separate search API.</p>
              <p><strong>Setup:</strong> No additional configuration is required beyond a valid OpenRouter API key. Any model preset works; OpenRouter selects a compatible online-capable variant automatically.</p>
              <p><strong>To disable:</strong> Remove or comment out the directive line, or move it below the first non-empty line.</p>
            </div>
          {/if}

          {#if validation.errors.length > 0}
            <div class="messages">
              <h4>Errors</h4>
              {#each validation.errors as error (error)}
                <p>{error}</p>
              {/each}
            </div>
          {/if}

          {#if validation.warnings.length > 0}
            <div class="messages warnings">
              <h4>Warnings</h4>
              {#each validation.warnings as warning (warning)}
                <p>{warning}</p>
              {/each}
            </div>
          {/if}

          <dl class="context-list">
            {#each validation.contextSummary as item (item.label)}
              <div>
                <dt>{item.label}</dt>
                <dd>{item.value}</dd>
              </div>
            {/each}
          </dl>
        {:else}
          <p class="empty-state">No validation state yet.</p>
        {/if}
      {/if}

      <div class="execution-card">
        <div class="heading">
          <div>
            <p class="eyebrow">Execution</p>
            <h3>{tab.title}</h3>
          </div>
          {#if execution}
            <span class={`badge ${executionTone(execution.status)}`}>{execution.status}</span>
          {/if}
        </div>


         {#if executionLoading}
          <div class="run-action state-loading">
            <p class="empty-state">Assembling and running prompt…</p>
          </div>
        {:else if execution}
          <div class="run-action state-done">
            <button
              type="button"
              class="primary run-action-btn"
              onclick={() => onExecute(tab!.path)}
            >
              Re-Assemble & Run
            </button>
          </div>

          {#if execution.error}
            <div class="messages">
              <h4>Execution Error</h4>
              <p>{execution.error}</p>
              <p class="meta">If you see decoding or OpenRouter errors, the provider may be experiencing network instability or returning corrupted data. Please wait a moment and try assembling the run again.</p>
            </div>
          {/if}

          <dl class="context-list">
            <div>
              <dt>Model</dt>
              <dd>{execution.modelId || 'Unknown'}</dd>
            </div>
            {#if execution.usage.promptTokens != null}
              <div>
                <dt>Prompt Tokens</dt>
                <dd>{execution.usage.promptTokens.toLocaleString()}</dd>
              </div>
            {/if}
            {#if execution.usage.completionTokens != null}
              <div>
                <dt>Completion Tokens</dt>
                <dd>{execution.usage.completionTokens.toLocaleString()}</dd>
              </div>
            {/if}
            {#if execution.usage.totalTokens != null}
              <div>
                <dt>Total Tokens</dt>
                <dd>{execution.usage.totalTokens.toLocaleString()}</dd>
              </div>
            {/if}
            {#if execution.usage.cost != null}
              <div>
                <dt>Cost</dt>
                <dd>${execution.usage.cost.toFixed(4)}</dd>
              </div>
            {/if}
            {#if execution.usage.outputWordCount != null}
              <div>
                <dt>Output Words</dt>
                <dd>{execution.usage.outputWordCount.toLocaleString()}</dd>
              </div>
            {/if}
            {#if execution.usage.retryCount != null && execution.usage.retryCount > 0}
              <div>
                <dt>Retries</dt>
                <dd>{execution.usage.retryCount}</dd>
              </div>
            {/if}
            <div>
              <dt>Online</dt>
              <dd>{execution.online.enabled ? 'Enabled' : 'Disabled'}</dd>
            </div>
            {#if execution.online.enabled}
              <div>
                <dt>Web Search Requests</dt>
                <dd>{execution.online.webSearchRequests}</dd>
              </div>
              <div>
                <dt>Citations</dt>
                <dd>{execution.online.citationCount}</dd>
              </div>
            {/if}
            {#if execution.outputTarget === 'document' || execution.outputTarget === 'both'}
              <div>
                <dt>Output Document</dt>
                <dd>{execution.documentPath || 'None'}</dd>
              </div>
            {/if}
            <div>
              <dt>Variables</dt>
              <dd>
                {#if Object.keys(execution.variables).length > 0}
                  {Object.entries(execution.variables).map(([k, v]) => `${k}=${v}`).join(', ')}
                {:else}
                  None
                {/if}
              </dd>
            </div>
            <div>
              <dt>Run File</dt>
              <dd>{execution.runPath || 'Not persisted'}</dd>
            </div>
          </dl>
        {:else}
          <div class="run-action state-ready">
            <p class="empty-state">Context is ready. Assemble references and execute via OpenRouter.</p>
            <button
              type="button"
              class="primary run-action-btn"
              onclick={() => onExecute(tab!.path)}
              disabled={!credentialState.hasStoredKey}
            >
              Assemble & Run Prompt
            </button>
          </div>
        {/if}
      </div>
    </div>

    <div class="preview">
      {#if showPreview}
        <p class="eyebrow">Preview</p>
        {#if loading}
          <p class="empty-state">Waiting for preview…</p>
        {:else if validation?.preview}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="markdown-preview">{@html parsedPreview}</div>
        {:else if validation}
          <p class="empty-state">No preview available for the current draft.</p>
        {:else}
          <p class="empty-state">Open a `.tera` prompt to generate a preview.</p>
        {/if}
      {/if}

      <div class="output-panel">
        <p class="eyebrow">Latest Run</p>
        {#if executionLoading}
          <p class="empty-state">Waiting for provider response…</p>
        {:else if execution?.documentPath && execution?.output == null}
          <div class="messages info">
            <h4>Saved to Workspace</h4>
            <p>The output was successfully generated and written directly to <code>{execution.documentPath}</code>.</p>
          </div>
        {:else if execution?.output}
          {#if execution?.documentPath}
            <div class="messages info" style="margin-bottom: 0.8rem;">
              <h4>Saved to Workspace</h4>
              <p>The output below was also written to <code>{execution.documentPath}</code>.</p>
            </div>
          {/if}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="markdown-preview">{@html parsedOutput}</div>
        {:else if execution?.error}
          <p class="empty-state">The latest execution failed before producing output.</p>
        {:else}
          <p class="empty-state">Run the active `.tera` prompt to inspect output here.</p>
        {/if}

        <div class="history-card">
          <div class="heading">
            <div>
              <p class="eyebrow">Recent Runs</p>
              <h3>{tab.title}</h3>
            </div>
          </div>

          {#if recentRunsLoading}
            <p class="empty-state">Loading persisted runs…</p>
          {:else if recentRuns.length > 0}
            <div class="history-list">
              {#each recentRuns as item (item.runId)}
                <article class="history-item">
                  <div class="history-head">
                    <div>
                      <strong>{item.runId}</strong>
                      <p class="empty-state">{item.completedAt}</p>
                    </div>
                    <span class={`badge ${executionTone(item.status)}`}>{item.status}</span>
                  </div>
                  <p class="empty-state">{item.modelId}</p>
                  {#if item.usage.totalTokens != null || item.usage.cost != null}
                    <p class="empty-state">
                      {#if item.usage.totalTokens != null}{item.usage.totalTokens.toLocaleString()} tokens{/if}{#if item.usage.totalTokens != null && item.usage.cost != null} · {/if}{#if item.usage.cost != null}${item.usage.cost.toFixed(4)}{/if}{#if item.usage.outputWordCount != null} · {item.usage.outputWordCount.toLocaleString()} words{/if}
                    </p>
                  {/if}
                  {#if item.online.enabled}
                    <p class="empty-state">
                      Online research: {item.online.webSearchRequests} web request{item.online.webSearchRequests === 1 ? '' : 's'} / {item.online.citationCount} citation{item.online.citationCount === 1 ? '' : 's'}
                    </p>
                  {/if}
                  {#if item.outputPreview}
                    <div class="markdown-preview">{@html marked.parse(item.outputPreview)}</div>
                  {:else if item.error}
                    <p>{item.error}</p>
                  {/if}
                  <button type="button" onclick={() => onOpenRunPath(item.runPath)}>Open artifact</button>
                </article>
              {/each}
            </div>
          {:else}
            <p class="empty-state">No persisted runs yet for this prompt.</p>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .panel-grid {
    display: grid;
    grid-template-columns: minmax(18rem, 24rem) minmax(0, 1fr);
    gap: 1rem;
    min-height: 0;
    overflow: hidden;
  }

  .summary,
  .preview {
    display: grid;
    gap: 0.75rem;
    min-height: 0;
    align-content: start;
  }

  .execution-card,
  .output-panel {
    display: grid;
    gap: 0.75rem;
  }

  .history-card,
  .history-list,
  .history-item {
    display: grid;
    gap: 0.75rem;
  }



  .history-item {
    padding: 0.8rem;
    border-radius: 14px;
    border: 1px solid var(--border-faint);
    background: rgba(255, 255, 255, 0.03);
  }

  .history-head {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
  }

  strong {
    font-size: 0.92rem;
  }

  .heading {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
  }

  h3,
  h4,
  p,
  dl {
    margin: 0;
  }

  h3 {
    font-size: 1rem;
  }

  h4 {
    font-size: 0.8rem;
    color: var(--text-soft);
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  .badge {
    font-size: 0.8rem;
    text-transform: capitalize;
  }

  .messages {
    display: grid;
    gap: 0.4rem;
    padding: 0.7rem;
    border-radius: 14px;
    background: rgba(255, 141, 161, 0.08);
    border: 1px solid rgba(255, 141, 161, 0.16);
  }

  .messages.warnings {
    background: rgba(255, 213, 127, 0.08);
    border-color: rgba(255, 213, 127, 0.16);
  }

  .messages.info {
    background: rgba(153, 227, 190, 0.08);
    border-color: rgba(153, 227, 190, 0.16);
  }

  .context-list {
    display: grid;
    gap: 0.6rem;
  }



  .run-action {
    display: grid;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 12px;
    background: rgba(49, 134, 96, 0.05);
    border: 1px solid rgba(153, 227, 190, 0.15);
    align-items: start;
    justify-items: start;
  }

  .run-action.state-loading,
  .run-action.state-done {
    background: rgba(255, 255, 255, 0.03);
    border-color: var(--border-faint);
  }

  .run-action-btn {
    font-size: 0.95rem;
    padding: 0.6rem 1.2rem;
  }

  button {
    border-radius: 999px;
    border: 1px solid var(--panel-border);
    background: var(--bg-ghost);
    color: var(--text);
    padding: 0.45rem 0.9rem;
    font: inherit;
    cursor: pointer;
  }

  button.primary {
    background: var(--gradient-success);
    border-color: var(--success-border);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  dt {
    color: var(--text-soft);
    font-size: 0.76rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
  }

  dd,
  .empty {
    color: var(--text-dim);
  }

  @media (max-width: 980px) {
    .panel-grid {
      grid-template-columns: 1fr;
    }
  }

  .panel-grid:has(.summary:only-child),
  .panel-grid:has(.preview:empty) {
    grid-template-columns: 1fr;
  }
</style>
