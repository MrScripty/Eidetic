<script lang="ts">
	import type { AiConfig, BackendType, ModelEntry } from '$lib/types.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { getAiStatus, updateAiConfig, getDiffusionStatus, loadDiffusionModel, unloadDiffusionModel, listModels } from '$lib/api.js';

	let config = $state<AiConfig>({
		backend_type: 'ollama',
		model: 'auto',
		temperature: 0.7,
		max_tokens: 4096,
		base_url: 'http://localhost:11434',
		api_key: null,
	});

	let saving = $state(false);
	let statusMessage = $state('');

	// Diffusion LLM local state
	let diffusionModelPath = $state('');
	let diffusionDevice = $state<'cuda' | 'cpu'>('cuda');
	let diffusionLoading = $state(false);
	let diffusionMessage = $state('');

	// Model library browser
	let libraryModels = $state<ModelEntry[]>([]);
	let libraryAvailable = $state(false);
	let showModelPicker = $state(false);
	let modelSearch = $state('');

	let filteredModels = $derived(
		modelSearch
			? libraryModels.filter(m =>
				m.name.toLowerCase().includes(modelSearch.toLowerCase()) ||
				m.id.toLowerCase().includes(modelSearch.toLowerCase())
			)
			: libraryModels
	);

	async function checkStatus() {
		try {
			editorState.aiStatus = await getAiStatus();
		} catch {
			editorState.aiStatus = { backend: config.backend_type, connected: false, error: 'Failed to reach server' };
		}
	}

	async function checkDiffusionStatus() {
		try {
			editorState.diffusionStatus = await getDiffusionStatus();
			if (editorState.diffusionStatus?.model_path && !diffusionModelPath) {
				diffusionModelPath = editorState.diffusionStatus.model_path;
			}
			if (editorState.diffusionStatus?.device) {
				diffusionDevice = editorState.diffusionStatus.device as 'cuda' | 'cpu';
			}
		} catch {
			editorState.diffusionStatus = null;
		}
	}

	async function loadLibraryModels() {
		try {
			const result = await listModels({ model_type: 'llm', limit: 200 });
			libraryModels = result.models;
			libraryAvailable = true;
		} catch {
			libraryAvailable = false;
		}
	}

	function selectModel(model: ModelEntry) {
		diffusionModelPath = model.path;
		showModelPicker = false;
		modelSearch = '';
	}

	function formatSize(bytes: number | null): string {
		if (bytes == null) return '';
		if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
		if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(0)} MB`;
		return `${bytes} B`;
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
		if (value === 'ollama') {
			config.base_url = 'http://localhost:11434';
		} else {
			config.base_url = 'https://openrouter.ai/api/v1';
		}
	}

	async function handleLoadDiffusion() {
		if (!diffusionModelPath.trim()) {
			diffusionMessage = 'Model path is required';
			return;
		}
		diffusionLoading = true;
		diffusionMessage = '';
		try {
			await loadDiffusionModel(diffusionModelPath.trim(), diffusionDevice);
			await checkDiffusionStatus();
			diffusionMessage = editorState.diffusionStatus?.model_loaded ? 'Model loaded' : 'Load failed';
		} catch (e) {
			diffusionMessage = e instanceof Error ? e.message : 'Load failed';
		}
		diffusionLoading = false;
	}

	async function handleUnloadDiffusion() {
		diffusionLoading = true;
		diffusionMessage = '';
		try {
			await unloadDiffusionModel();
			await checkDiffusionStatus();
			diffusionMessage = 'Model unloaded';
		} catch (e) {
			diffusionMessage = e instanceof Error ? e.message : 'Unload failed';
		}
		diffusionLoading = false;
	}

	// Check status on mount
	$effect(() => {
		checkStatus();
		checkDiffusionStatus();
		loadLibraryModels();
		const interval = setInterval(() => {
			checkStatus();
			checkDiffusionStatus();
		}, 30_000);
		return () => clearInterval(interval);
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
				Connected — {editorState.aiStatus.model || config.model}
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
			<option value="ollama">Ollama (Local)</option>
			<option value="open_router">OpenRouter (Cloud)</option>
		</select>
	</label>

	<label class="field">
		<span class="field-label">Model</span>
		<input type="text" bind:value={config.model} placeholder="auto (detect loaded model)" />
	</label>

	{#if config.backend_type === 'ollama'}
		<label class="field">
			<span class="field-label">Base URL</span>
			<input type="text" bind:value={config.base_url} placeholder="http://localhost:11434" />
		</label>
	{:else}
		<label class="field">
			<span class="field-label">API Key</span>
			<input type="password" bind:value={config.api_key} placeholder="sk-or-..." />
		</label>
	{/if}

	<label class="field">
		<span class="field-label">Temperature <span class="field-value">{config.temperature.toFixed(1)}</span></span>
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

	<!-- ── Diffusion LLM ── -->
	<div class="section-divider"></div>

	<div class="section-header">Diffusion LLM</div>

	<div class="status-row">
		<span
			class="status-dot"
			class:connected={editorState.diffusionStatus?.model_loaded}
			class:disconnected={editorState.diffusionStatus && !editorState.diffusionStatus.model_loaded}
		></span>
		<span class="status-text">
			{#if editorState.diffusionStatus?.model_loaded}
				Loaded — {editorState.diffusionStatus.model_path}
			{:else}
				No model loaded
			{/if}
		</span>
	</div>

	<div class="field">
		<span class="field-label">
			Model Path
			{#if libraryAvailable}
				<button class="browse-btn" onclick={() => { showModelPicker = !showModelPicker; }} disabled={diffusionLoading}>
					{showModelPicker ? 'Close' : 'Browse'}
				</button>
			{/if}
		</span>
		<input
			type="text"
			bind:value={diffusionModelPath}
			placeholder="/path/to/model or HuggingFace ID"
			disabled={diffusionLoading}
		/>
	</div>

	{#if showModelPicker}
		<div class="model-picker">
			<input
				type="text"
				class="model-search"
				bind:value={modelSearch}
				placeholder="Search models..."
			/>
			<div class="model-list">
				{#each filteredModels as model (model.id)}
					<button class="model-item" onclick={() => selectModel(model)}>
						<span class="model-name">{model.name}</span>
						<span class="model-meta">
							{model.model_type}
							{#if model.size_bytes}
								 &middot; {formatSize(model.size_bytes)}
							{/if}
						</span>
					</button>
				{:else}
					<div class="model-empty">No models found</div>
				{/each}
			</div>
		</div>
	{/if}

	<label class="field">
		<span class="field-label">Device</span>
		<select bind:value={diffusionDevice} disabled={diffusionLoading}>
			<option value="cuda">CUDA (GPU)</option>
			<option value="cpu">CPU</option>
		</select>
	</label>

	<div class="diffusion-actions">
		{#if editorState.diffusionStatus?.model_loaded}
			<button class="save-btn unload-btn" onclick={handleUnloadDiffusion} disabled={diffusionLoading}>
				{diffusionLoading ? 'Working...' : 'Unload Model'}
			</button>
			<button class="save-btn" onclick={handleLoadDiffusion} disabled={diffusionLoading}>
				{diffusionLoading ? 'Working...' : 'Reload'}
			</button>
		{:else}
			<button class="save-btn" onclick={handleLoadDiffusion} disabled={diffusionLoading}>
				{diffusionLoading ? 'Loading...' : 'Load Model'}
			</button>
		{/if}
	</div>

	{#if diffusionMessage}
		<div class="save-message" class:success={diffusionMessage === 'Model loaded' || diffusionMessage === 'Model unloaded'}>
			{diffusionMessage}
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
	.field input[type="text"],
	.field input[type="password"],
	.field input[type="number"] {
		padding: 6px 8px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-size: 0.8rem;
		font-family: inherit;
	}

	.field input[type="range"] {
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

	.section-divider {
		border-top: 1px solid var(--color-border-subtle);
		margin: 4px 0;
	}

	.section-header {
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.diffusion-actions {
		display: flex;
		gap: 8px;
	}

	.diffusion-actions .save-btn {
		flex: 1;
	}

	.unload-btn {
		background: var(--color-bg-surface) !important;
		color: var(--color-text-secondary) !important;
		border: 1px solid var(--color-border-default) !important;
	}

	.unload-btn:hover:not(:disabled) {
		border-color: var(--color-danger-light) !important;
		color: var(--color-danger-light) !important;
	}

	.browse-btn {
		background: none;
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		color: var(--color-accent);
		font-size: 0.65rem;
		padding: 1px 6px;
		cursor: pointer;
		text-transform: none;
		letter-spacing: normal;
		font-family: inherit;
	}

	.browse-btn:hover:not(:disabled) {
		background: var(--color-bg-surface);
	}

	.model-picker {
		display: flex;
		flex-direction: column;
		gap: 4px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 6px;
		padding: 6px;
	}

	.model-search {
		padding: 4px 8px;
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		font-size: 0.75rem;
		font-family: inherit;
	}

	.model-search:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.model-list {
		max-height: 200px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.model-item {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 1px;
		padding: 5px 8px;
		background: none;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		text-align: left;
		font-family: inherit;
		color: var(--color-text-primary);
	}

	.model-item:hover {
		background: var(--color-bg-hover);
	}

	.model-name {
		font-size: 0.75rem;
		font-weight: 500;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		width: 100%;
	}

	.model-meta {
		font-size: 0.65rem;
		color: var(--color-text-muted);
	}

	.model-empty {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-align: center;
		padding: 12px;
	}
</style>
