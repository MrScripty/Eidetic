<script lang="ts">
  import {
    notesPromptQA,
    armContextHold,
    releaseContext,
    contextReceipt,
  } from './notesPrompt.svelte.js';
  import {
    factQA,
    replayRequest,
    replayDecision,
    relationshipABA,
    readImpacts,
    receipt,
    armSavePause,
    releaseSaveAcknowledgement,
    newerFact,
    failDetailRefresh,
    armInitialDetailFailure,
    holdDetailRecovery,
    releaseDetailRecovery,
  } from './manualFacts.svelte.js';
</script>

<aside class="qa" aria-label="Qualifier-only saved edit controls">
  <strong>QA context delay · actual public context read · no fabricated prompt</strong>
  <button onclick={armContextHold}>QA hold next real context response</button>
  <button onclick={releaseContext} disabled={!notesPromptQA.held}
    >QA release real context response</button
  >
  <label
    >QA context receipt<textarea aria-label="QA context receipt" readonly value={contextReceipt()}
    ></textarea></label
  >
  <strong>QA saved-edit fixture · public commands · SYNTHETIC HTTP replies</strong>
  <button onclick={replayRequest} disabled={factQA.busy || !factQA.requestId}
    >QA replay exact analysis</button
  >
  <button onclick={replayDecision} disabled={factQA.busy || !factQA.decisionId}
    >QA replay exact decision</button
  >
  <button onclick={relationshipABA} disabled={factQA.busy}>QA relationship edit and restore</button>
  <button onclick={readImpacts} disabled={factQA.busy}>QA read canonical impacts</button>
  <strong>QA transport faults: held Save acknowledgement and failed/held detail reads</strong>
  <button onclick={armSavePause}>QA hold next Bible Save acknowledgement</button>
  <button onclick={newerFact} disabled={factQA.busy}>QA write newer Green fact</button>
  <button onclick={releaseSaveAcknowledgement} disabled={!factQA.saveAcknowledgementHeld}
    >QA release Bible Save acknowledgement</button
  >
  <button onclick={failDetailRefresh} disabled={factQA.busy}>QA fail Bible detail refresh</button>
  <button onclick={armInitialDetailFailure}>QA arm initial Bible detail failure</button>
  <button onclick={holdDetailRecovery}>QA hold Bible detail recovery</button>
  <button onclick={releaseDetailRecovery}>QA release Bible detail recovery</button>
  <label
    >QA saved-edit receipt<textarea aria-label="QA saved-edit receipt" readonly value={receipt()}
    ></textarea></label
  >
  {#if factQA.error}<p role="alert">{factQA.error}</p>{/if}
</aside>

<style>
  .qa {
    position: fixed;
    bottom: 8px;
    right: 8px;
    z-index: 100;
    width: 570px;
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
