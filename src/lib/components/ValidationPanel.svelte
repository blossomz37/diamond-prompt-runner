<script lang="ts">
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
    credentialDraft: string;
    credentialLoading: boolean;
    onCredentialInput: (value: string) => void;
    onSaveCredential: () => void | Promise<void>;
    onClearCredential: () => void | Promise<void>;
    recentRuns: PromptRunHistoryEntry[];
    recentRunsLoading: boolean;
    onOpenRunPath: (path: string) => void | Promise<void>;
    onExecute: (path: string) => void | Promise<void>;
  }

  let {
    tab,
    validation,
    loading,
    execution,
    executionLoading,
    credentialState,
    credentialDraft,
    credentialLoading,
    onCredentialInput,
    onSaveCredential,
    onClearCredential,
    recentRuns,
    recentRunsLoading,
    onOpenRunPath,
    onExecute
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

  function credentialTone(source: ExecutionCredentialStatus['source']): string {
    switch (source) {
      case 'keychain':
        return 'good';
      case 'environment':
        return 'warn';
      default:
        return 'bad';
    }
  }

  function credentialLabel(source: ExecutionCredentialStatus['source']): string {
    switch (source) {
      case 'keychain':
        return 'keychain';
      case 'environment':
        return 'env fallback';
      default:
        return 'missing';
    }
  }

  function credentialHelp(status: ExecutionCredentialStatus): string {
    switch (status.source) {
      case 'keychain':
        return 'Stored in the native keychain for this app.';
      case 'environment':
        return 'No stored key is saved yet. Runs currently rely on OPENROUTER_API_KEY from the environment.';
      default:
        return 'No OpenRouter key is available yet. Save one here or set OPENROUTER_API_KEY.';
    }
  }

  const onlineDraftEnabled = $derived(
    tab?.kind === 'tera' ? promptUsesOnlineResearch(tab.draftContent) : false
  );
</script>

{#if !tab}
  <p class="empty">Open a project asset to inspect validation and preview state here.</p>
{:else if tab.kind !== 'tera'}
  <p class="empty">Template preview and validation are available for `.tera` prompt templates in this slice.</p>
{:else}
  <div class="panel-grid">
    <div class="summary">
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
        <p class="empty">Refreshing validation from the current draft…</p>
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
        <p class="empty">No validation state yet.</p>
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

        <div class="credential-card">
          <div class="heading">
            <div>
              <p class="eyebrow">Credentials</p>
              <h3>OpenRouter API Key</h3>
            </div>
            <span class={`badge ${credentialTone(credentialState.source)}`}>
              {credentialLabel(credentialState.source)}
            </span>
          </div>

          <p class="empty">{credentialHelp(credentialState)}</p>

          <label class="secret-field">
            <span>OpenRouter API key</span>
            <input
              type="password"
              value={credentialDraft}
              placeholder="sk-or-v1-..."
              autocomplete="off"
              spellcheck="false"
              oninput={(event) => onCredentialInput((event.currentTarget as HTMLInputElement).value)}
            />
          </label>

          <div class="credential-actions">
            <button
              type="button"
              class="primary"
              onclick={() => onSaveCredential()}
              disabled={credentialLoading || credentialDraft.trim().length === 0}
            >{credentialLoading ? 'Saving…' : 'Save key'}</button>
            <button
              type="button"
              onclick={() => onClearCredential()}
              disabled={credentialLoading || !credentialState.hasStoredKey}
            >Clear stored key</button>
          </div>
        </div>

         {#if executionLoading}
          <div class="run-action state-loading">
            <p class="empty">Assembling and running prompt…</p>
          </div>
        {:else if execution}
          <div class="run-action state-done">
            <button
              type="primary"
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
            <div>
              <dt>Run File</dt>
              <dd>{execution.runPath || 'Not persisted'}</dd>
            </div>
          </dl>
        {:else}
          <div class="run-action state-ready">
            <p class="empty">Context is ready. Assemble references and execute via OpenRouter.</p>
            <button
              type="primary"
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
      <p class="eyebrow">Preview</p>
      {#if loading}
        <p class="empty">Waiting for preview…</p>
      {:else if validation?.preview}
        <pre>{validation.preview}</pre>
      {:else if validation}
        <p class="empty">No preview available for the current draft.</p>
      {:else}
        <p class="empty">Open a `.tera` prompt to generate a preview.</p>
      {/if}

      <div class="output-panel">
        <p class="eyebrow">Latest Run</p>
        {#if executionLoading}
          <p class="empty">Waiting for provider response…</p>
        {:else if execution?.output}
          <pre>{execution.output}</pre>
        {:else if execution?.error}
          <p class="empty">The latest execution failed before producing output.</p>
        {:else}
          <p class="empty">Run the active `.tera` prompt to inspect output here.</p>
        {/if}

        <div class="history-card">
          <div class="heading">
            <div>
              <p class="eyebrow">Recent Runs</p>
              <h3>{tab.title}</h3>
            </div>
          </div>

          {#if recentRunsLoading}
            <p class="empty">Loading persisted runs…</p>
          {:else if recentRuns.length > 0}
            <div class="history-list">
              {#each recentRuns as item (item.runId)}
                <article class="history-item">
                  <div class="history-head">
                    <div>
                      <strong>{item.runId}</strong>
                      <p class="empty">{item.completedAt}</p>
                    </div>
                    <span class={`badge ${executionTone(item.status)}`}>{item.status}</span>
                  </div>
                  <p class="empty">{item.modelId}</p>
                  {#if item.usage.totalTokens != null || item.usage.cost != null}
                    <p class="empty">
                      {#if item.usage.totalTokens != null}{item.usage.totalTokens.toLocaleString()} tokens{/if}{#if item.usage.totalTokens != null && item.usage.cost != null} · {/if}{#if item.usage.cost != null}${item.usage.cost.toFixed(4)}{/if}{#if item.usage.outputWordCount != null} · {item.usage.outputWordCount.toLocaleString()} words{/if}
                    </p>
                  {/if}
                  {#if item.online.enabled}
                    <p class="empty">
                      Online research: {item.online.webSearchRequests} web request{item.online.webSearchRequests === 1 ? '' : 's'} / {item.online.citationCount} citation{item.online.citationCount === 1 ? '' : 's'}
                    </p>
                  {/if}
                  {#if item.outputPreview}
                    <p>{item.outputPreview}</p>
                  {:else if item.error}
                    <p>{item.error}</p>
                  {/if}
                  <button type="button" onclick={() => onOpenRunPath(item.runPath)}>Open artifact</button>
                </article>
              {/each}
            </div>
          {:else}
            <p class="empty">No persisted runs yet for this prompt.</p>
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

  .credential-card {
    display: grid;
    gap: 0.75rem;
    padding: 0.8rem;
    border-radius: 14px;
    border: 1px solid rgba(157, 180, 255, 0.12);
    background: rgba(255, 255, 255, 0.03);
  }

  .history-item {
    padding: 0.8rem;
    border-radius: 14px;
    border: 1px solid rgba(157, 180, 255, 0.12);
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

  .eyebrow {
    margin: 0;
    color: var(--accent);
    font-size: 0.72rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  h3,
  h4,
  p,
  pre,
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
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    border: 1px solid var(--panel-border);
    font-size: 0.8rem;
    text-transform: capitalize;
  }

  .badge.good {
    color: var(--success);
    border-color: rgba(153, 227, 190, 0.3);
  }

  .badge.warn {
    color: #ffd57f;
    border-color: rgba(255, 213, 127, 0.3);
  }

  .badge.bad {
    color: var(--danger);
    border-color: rgba(255, 141, 161, 0.3);
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

  .secret-field {
    display: grid;
    gap: 0.4rem;
    color: var(--text-soft);
    font-size: 0.8rem;
  }

  .secret-field input {
    width: 100%;
    border-radius: 12px;
    border: 1px solid var(--panel-border);
    background: rgba(5, 8, 15, 0.9);
    color: var(--text);
    padding: 0.7rem 0.8rem;
    font: inherit;
  }

  .secret-field input:focus {
    outline: 1px solid rgba(139, 177, 255, 0.34);
    border-color: rgba(139, 177, 255, 0.34);
  }

  .credential-actions {
    display: flex;
    flex-wrap: wrap;
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
    border-color: rgba(157, 180, 255, 0.12);
  }

  .run-action-btn {
    font-size: 0.95rem;
    padding: 0.6rem 1.2rem;
  }

  button {
    border-radius: 999px;
    border: 1px solid var(--panel-border);
    background: rgba(255, 255, 255, 0.04);
    color: var(--text);
    padding: 0.45rem 0.9rem;
    font: inherit;
    cursor: pointer;
  }

  button.primary {
    background: linear-gradient(135deg, rgba(153, 227, 190, 0.22), rgba(49, 134, 96, 0.28));
    border-color: rgba(153, 227, 190, 0.3);
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

  pre {
    padding: 0.8rem;
    border-radius: 14px;
    background: rgba(5, 8, 15, 0.9);
    border: 1px solid rgba(157, 180, 255, 0.12);
    white-space: pre-wrap;
    word-break: break-word;
  }

  @media (max-width: 980px) {
    .panel-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
