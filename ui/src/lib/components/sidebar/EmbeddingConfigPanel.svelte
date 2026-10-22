<script lang="ts">
  import type { EmbeddingConfig } from '$lib/aiTypes.js';
  import PumasSelection from './PumasSelection.svelte';
  let {
    embedding = $bindable<EmbeddingConfig>({
      provider: 'disabled',
      base_url: '',
      model: '',
      profile: '',
      revision: '',
      api_key: null,
    }),
  } = $props<{ embedding?: EmbeddingConfig }>();
</script>

<fieldset>
  <legend>Reference embeddings</legend>
  <label
    >Provider
    <select bind:value={embedding.provider}>
      <option value="disabled">Disabled</option>
      <option value="pumas">Pumas local inference</option>
      <option value="open_ai_compatible">OpenAI compatible endpoint</option>
    </select>
  </label>
  {#if embedding.provider !== 'disabled'}
    <label
      >Embedding endpoint
      <input
        type="text"
        bind:value={embedding.base_url}
        placeholder={embedding.provider === 'pumas'
          ? 'http://127.0.0.1:8080'
          : 'https://provider.example/v1'}
      />
    </label>
    {#if embedding.provider === 'pumas'}
      <PumasSelection
        endpoint={embedding.base_url}
        bind:model={embedding.model}
        bind:profile={embedding.profile}
      />
    {:else}
      <label>Embedding model <input type="text" bind:value={embedding.model} /></label>
      <label>Model revision <input type="text" bind:value={embedding.revision} /></label>
      <label>Embedding API key <input type="password" bind:value={embedding.api_key} /></label>
    {/if}
  {/if}
</fieldset>

<style>
  fieldset {
    margin: 0;
    padding: 8px;
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  legend,
  label {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  input,
  select {
    padding: 6px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
  }
</style>
