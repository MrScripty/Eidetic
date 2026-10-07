<script lang="ts">
  import {
    selectedQA,
    mutateSelectedFact,
    replayStaleSelection,
    selectedReceipt,
  } from './recalledFacts.svelte.js';
</script>

<aside class="qa" aria-label="Qualifier-only recalled fact controls">
  <strong>QA selected-fact fixture · public commands · synthetic provider replies</strong>
  <button onclick={mutateSelectedFact} disabled={selectedQA.busy}
    >QA mutate selected Keeper fact</button
  >
  <button onclick={replayStaleSelection} disabled={selectedQA.busy || !selectedQA.capturedRequest}
    >QA replay stale selection</button
  >
  <label
    >QA selected-fact receipt<textarea
      aria-label="QA selected-fact receipt"
      readonly
      value={selectedReceipt()}
    ></textarea></label
  >
  {#if selectedQA.error}<p role="alert">{selectedQA.error}</p>{/if}
</aside>

<style>
  .qa {
    position: fixed;
    bottom: 8px;
    right: 8px;
    z-index: 100;
    width: 550px;
    padding: 8px;
    border: 2px solid #eab308;
    background: #121826;
    color: #fff;
    font-size: 11px;
  }
  button {
    margin: 3px;
    padding: 4px;
    border: 1px solid #aab;
    background: #273248;
    color: white;
  }
  button:disabled {
    opacity: 0.4;
  }
  label {
    display: block;
  }
  textarea {
    height: 40px;
    width: 100%;
    color: #fff;
    background: #121826;
    font-size: 10px;
  }
</style>
