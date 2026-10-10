<script lang="ts">
  import { getPumasCatalog, loadPumasModel, unloadPumasModel } from '$lib/api.js';
  import type { PumasCatalog } from '$lib/aiTypes.js';

  let {
    endpoint,
    model = $bindable(''),
    profile = $bindable(''),
  } = $props<{
    endpoint: string;
    model: string;
    profile?: string;
  }>();
  let catalog = $state<PumasCatalog | null>(null);
  let message = $state('');
  let busy = $state(false);
  const selectedProfile = $derived(
    catalog?.profiles.snapshot.profiles.find((p) => p.profile_id === profile),
  );

  async function refresh() {
    busy = true;
    try {
      catalog = await getPumasCatalog(endpoint);
      message = 'Choose the installed model and its Pumas runtime profile, then Load.';
    } catch (error) {
      message = String(error);
    } finally {
      busy = false;
    }
  }
  async function lifecycle(action: 'load' | 'unload') {
    if (!selectedProfile) return;
    busy = true;
    try {
      if (action === 'load') {
        await loadPumasModel(
          endpoint,
          model,
          profile,
          selectedProfile.provider,
          selectedProfile.device.mode,
        );
      } else {
        await unloadPumasModel(endpoint, model, profile, selectedProfile.provider);
      }
      message =
        action === 'load' ? 'Model loaded through Pumas' : 'Selected model unloaded through Pumas';
    } catch (error) {
      message = String(error);
    } finally {
      busy = false;
    }
  }
</script>

<div class="selection">
  <button type="button" onclick={refresh} disabled={busy}>List Pumas models and profiles</button>
  <label
    >Model
    <select bind:value={model}>
      <option value="">Choose model</option>
      {#each catalog?.models.models ?? [] as entry (entry.id)}
        <option value={entry.id}>{entry.official_name} ({entry.model_type})</option>
      {/each}
    </select>
  </label>
  <label
    >Runtime profile
    <select bind:value={profile}>
      <option value="">Choose profile</option>
      {#each catalog?.profiles.snapshot.profiles ?? [] as entry (entry.profile_id)}
        <option value={entry.profile_id}>{entry.profile_id} ({entry.provider})</option>
      {/each}
    </select>
  </label>
  <div>
    <button
      type="button"
      onclick={() => lifecycle('load')}
      disabled={busy || !model || !selectedProfile}>Load</button
    >
    <button
      type="button"
      onclick={() => lifecycle('unload')}
      disabled={busy || !model || !selectedProfile}>Unload</button
    >
  </div>
  {#if message}<p role="status">{message}</p>{/if}
</div>

<style>
  .selection,
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 0.75rem;
  }
  select,
  button {
    padding: 5px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
  }
  p {
    margin: 0;
    overflow-wrap: anywhere;
    color: var(--color-text-secondary);
  }
</style>
