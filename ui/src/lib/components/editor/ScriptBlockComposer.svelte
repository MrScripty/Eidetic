<script lang="ts">
  import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
  import { getSessionScriptBlockCreationDraft } from '$lib/stores/scriptBlockCreationSession.svelte.js';

  let { source }: { source: SelectedNodeEditorNode | null } = $props();
  let composer = $derived(getSessionScriptBlockCreationDraft());
  let draft = $derived(composer.state);
</script>

<section class="composer" aria-label="Write screenplay">
  {#if draft.writing}
    <p>Writing for <strong>{draft.target?.name}</strong>. Placement follows this timeline clip.</p>
    <label
      >Block type
      <select bind:value={draft.kind} disabled={draft.saving || draft.uncertain}>
        <option value="action">Action</option>
        <option value="scene_heading">Scene heading</option>
        <option value="character">Character</option>
        <option value="dialogue">Dialogue</option>
        <option value="parenthetical">Parenthetical</option>
        <option value="transition">Transition</option>
        <option value="shot">Shot</option>
        <option value="note">Note</option>
      </select>
    </label>
    <label for="new-screenplay-text">New screenplay text</label>
    <textarea
      id="new-screenplay-text"
      bind:value={draft.text}
      rows="8"
      disabled={draft.saving || draft.uncertain}
    ></textarea>
    {#if draft.error}
      <p role="alert">
        {draft.error}
        {draft.uncertain
          ? 'The save may have completed. Retry to confirm it before changing this text.'
          : 'Your draft is still here.'}
      </p>
    {/if}
    <div class="actions">
      <button
        type="button"
        onclick={composer.save}
        disabled={draft.saving || (!draft.uncertain && !draft.text.trim())}
        >{draft.saving
          ? 'Saving…'
          : draft.uncertain
            ? 'Retry same save'
            : 'Save screenplay'}</button
      >
      <button type="button" onclick={composer.cancel} disabled={draft.saving || draft.uncertain}
        >Cancel</button
      >
      {#if draft.placementRefused && source?.node_id === draft.target?.node_id}
        <button
          type="button"
          onclick={() => source && composer.useCurrentPlacement(source)}
          disabled={draft.saving || draft.uncertain}>Use current placement and save</button
        >
      {/if}
    </div>
  {:else}
    <button type="button" onclick={() => source && composer.begin(source)} disabled={!source}
      >Write screenplay</button
    >
    {#if source}<p>For {source.name}. Add your own text without replacing existing blocks.</p>
    {:else}<p>Select a timeline clip to start writing.</p>{/if}
  {/if}
</section>

<style>
  .composer {
    break-inside: avoid;
    padding: 8px;
    margin-bottom: 12px;
    border: 1px solid var(--color-border-subtle);
  }
  p,
  label {
    font-size: 0.8rem;
  }
  label {
    display: block;
    margin-bottom: 4px;
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    font-family: 'Courier New', monospace;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
  }
  select {
    margin-left: 6px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 6px;
  }
</style>
