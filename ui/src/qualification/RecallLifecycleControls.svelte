<script lang="ts">
  import {
    lifecycleQA,
    lifecycleReceipt,
    releaseHeldCompletion,
  } from './recallLifecycle.svelte.js';
</script>

<aside class="qa" aria-label="Qualifier-only recall lifecycle controls">
  <strong>QA lifecycle fixture · actual reads · synthetic error</strong>
  <p>Inspector hosts: {lifecycleQA.hosts} · completion {lifecycleQA.completions}</p>
  <button
    onclick={() => (lifecycleQA.holdNext = true)}
    disabled={lifecycleQA.holdNext || lifecycleQA.held}>QA hold next recall completion</button
  >
  <button
    onclick={() => (lifecycleQA.inspectorsVisible = false)}
    disabled={!lifecycleQA.inspectorsVisible}>QA dispose all recall inspectors</button
  >
  <button
    onclick={() => (lifecycleQA.inspectorsVisible = true)}
    disabled={lifecycleQA.inspectorsVisible}>QA remount recall inspectors</button
  >
  <button onclick={() => releaseHeldCompletion(false)} disabled={!lifecycleQA.held}
    >QA release actual read success</button
  >
  <button onclick={() => releaseHeldCompletion(true)} disabled={!lifecycleQA.held}
    >QA release labelled synthetic error</button
  >
  <label
    >QA lifecycle receipt<textarea
      aria-label="QA lifecycle receipt"
      readonly
      value={lifecycleReceipt()}
    ></textarea></label
  >
</aside>

<style>
  .qa {
    position: fixed;
    bottom: 8px;
    right: 8px;
    z-index: 100;
    width: 650px;
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
  p {
    margin: 2px;
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
