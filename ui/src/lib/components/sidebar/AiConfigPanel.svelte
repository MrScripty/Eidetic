<script lang="ts">
  import type { AiConfig, BackendType } from '$lib/aiTypes.js';
  import { updateAiConfig } from '$lib/api.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import { refreshAiStatus } from '$lib/stores/aiStatus.svelte.js';

  const BACKEND_BASE_URLS: Record<BackendType, string> = {
    llama_cpp: 'http://localhost:8080/v1',
    ollama: 'http://localhost:11434',
    open_router: 'https://openrouter.ai/api/v1',
  };

  let config = $state<AiConfig>({
    backend_type: 'llama_cpp',
    model: 'auto',
    temperature: 0.7,
    max_tokens: 4096,
    base_url: BACKEND_BASE_URLS.llama_cpp,
    api_key: null,
  });

  let saving = $state(false);
  let statusMessage = $state('');
  let isLocalBackend = $derived(
    config.backend_type === 'llama_cpp' || config.backend_type === 'ollama',
  );

  async function checkStatus() {
    await refreshAiStatus();
  }

  async function handleSave() {
    saving = true;
    statusMessage = '';

    try {
      await updateAiConfig(config);
      await checkStatus();
      statusMessage = editorState.aiStatus?.connected ? 'Connected' : 'Connection failed';
    } catch {
      statusMessage = 'Failed to save config';
    }

    saving = false;
  }

  function handleBackendChange(e: Event) {
    const value = (e.target as HTMLSelectElement).value as BackendType;
    config.backend_type = value;
    config.base_url = BACKEND_BASE_URLS[value];
    if (value !== 'open_router') {
      config.api_key = null;
    }
  }

  function modelPlaceholder(backendType: BackendType): string {
    if (backendType === 'open_router') {
      return 'openai/gpt-4.1-mini';
    }
    return 'auto (detect available model)';
  }

  // Check status on mount.
  $effect(() => {
    void checkStatus();
  });
</script>

<div class="ai-config">
  <div class="status-row">
    <span
      class="status-dot"
      class:connected={editorState.aiStatus?.connected}
      class:disconnected={editorState.aiStatus && !editorState.aiStatus.connected}
    ></span>
    <span class="status-text">
      {#if editorState.aiStatus?.connected}
        Connected - {editorState.aiStatus.model || config.model}
      {:else if editorState.aiStatus?.error}
        {editorState.aiStatus.error}
      {:else}
        Checking...
      {/if}
    </span>
  </div>

  <label class="field">
    <span class="field-label">Backend</span>
    <select value={config.backend_type} onchange={handleBackendChange}>
      <option value="llama_cpp">llama.cpp (Local)</option>
      <option value="ollama">Ollama (Local)</option>
      <option value="open_router">OpenRouter (Cloud)</option>
    </select>
  </label>

  <label class="field">
    <span class="field-label">Model</span>
    <input
      type="text"
      bind:value={config.model}
      placeholder={modelPlaceholder(config.backend_type)}
    />
  </label>

  {#if isLocalBackend}
    <label class="field">
      <span class="field-label">Base URL</span>
      <input
        type="text"
        bind:value={config.base_url}
        placeholder={BACKEND_BASE_URLS[config.backend_type]}
      />
    </label>
  {:else}
    <label class="field">
      <span class="field-label">API Key</span>
      <input type="password" bind:value={config.api_key} placeholder="sk-or-..." />
    </label>
  {/if}

  <label class="field">
    <span class="field-label"
      >Temperature <span class="field-value">{config.temperature.toFixed(1)}</span></span
    >
    <input type="range" min="0" max="2" step="0.1" bind:value={config.temperature} />
  </label>

  <label class="field">
    <span class="field-label">Max Tokens</span>
    <input type="number" bind:value={config.max_tokens} min="256" max="32768" step="256" />
  </label>

  <button class="save-btn" onclick={handleSave} disabled={saving}>
    {saving ? 'Saving...' : 'Save & Connect'}
  </button>

  {#if statusMessage}
    <div class="save-message" class:success={statusMessage === 'Connected'}>
      {statusMessage}
    </div>
  {/if}
</div>

<style>
  .ai-config {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--color-bg-surface);
    border-radius: 6px;
    border: 1px solid var(--color-border-subtle);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-muted);
    flex-shrink: 0;
  }

  .status-dot.connected {
    background: var(--color-success);
  }

  .status-dot.disconnected {
    background: var(--color-danger);
  }

  .status-text {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .field-value {
    font-size: 0.7rem;
    color: var(--color-text-secondary);
    text-transform: none;
    letter-spacing: normal;
  }

  .field select,
  .field input[type='text'],
  .field input[type='password'],
  .field input[type='number'] {
    padding: 6px 8px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    font-size: 0.8rem;
    font-family: inherit;
  }

  .field input[type='range'] {
    width: 100%;
    accent-color: var(--color-accent);
  }

  .field select:focus,
  .field input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .save-btn {
    padding: 8px 16px;
    background: var(--color-accent);
    color: var(--color-text-on-dark);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-family: inherit;
    transition: opacity 0.15s;
  }

  .save-btn:hover:not(:disabled) {
    opacity: 0.9;
  }

  .save-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .save-message {
    font-size: 0.75rem;
    color: var(--color-danger-light);
    text-align: center;
  }

  .save-message.success {
    color: var(--color-success);
  }
</style>
