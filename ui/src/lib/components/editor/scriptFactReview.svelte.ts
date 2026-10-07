import { SvelteMap } from 'svelte/reactivity';
import type { RequestScriptFactProposalCommand } from '$lib/scriptFactTypes.js';
import {
  applyRequestScriptFactProposalCommand,
  applyScriptFactDecisionCommand,
  getPropagationProposalSessionEpoch,
} from '$lib/stores/propagationProposalProjection.svelte.js';

// Retry preserves command/proposal identity; changing the saved evidence retires it.
export function createScriptFactReview(getOwner: () => string) {
  const admittedEpoch = getPropagationProposalSessionEpoch();
  const state = $state({ pending: false, error: '', retry: false });
  let intent: {
    owner: string;
    commandId: string;
    payload: RequestScriptFactProposalCommand;
  } | null = null;
  const decisions = new SvelteMap<string, string>();
  async function analyze(payload: Omit<RequestScriptFactProposalCommand, 'proposal_id'>) {
    if (state.pending || getPropagationProposalSessionEpoch() !== admittedEpoch) return;
    const owner = getOwner();
    if (!intent || intent.owner !== owner)
      intent = {
        owner,
        commandId: crypto.randomUUID(),
        payload: { ...payload, proposal_id: `script.fact.${crypto.randomUUID()}` },
      };
    const request = intent;
    state.pending = true;
    state.error = '';
    try {
      await applyRequestScriptFactProposalCommand(request.payload, request.commandId);
      if (getOwner() !== owner)
        throw new Error(
          'Saved evidence changed during analysis. Review the stored proposal before accepting.',
        );
      intent = null;
      state.retry = false;
    } catch (error) {
      state.error = error instanceof Error ? error.message : 'Fact analysis failed';
      state.retry = getOwner() === owner;
    } finally {
      state.pending = false;
    }
  }
  async function decide(proposalId: string, accept: boolean) {
    if (state.pending || getPropagationProposalSessionEpoch() !== admittedEpoch) return;
    const key = JSON.stringify([admittedEpoch, proposalId, accept]);
    const commandId = decisions.get(key) ?? crypto.randomUUID();
    decisions.set(key, commandId);
    state.pending = true;
    state.error = '';
    try {
      await applyScriptFactDecisionCommand(proposalId, accept, commandId);
    } catch (error) {
      if (getPropagationProposalSessionEpoch() === admittedEpoch)
        state.error = error instanceof Error ? error.message : 'Fact review failed';
    } finally {
      state.pending = false;
    }
  }
  function observe() {
    if (intent && intent.owner !== getOwner()) {
      state.retry = false;
      if (!state.pending) state.error = '';
    }
  }
  return { state, analyze, decide, observe };
}
