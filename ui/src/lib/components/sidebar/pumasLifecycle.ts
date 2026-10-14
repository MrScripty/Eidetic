import type { PumasUnserveResult } from '$lib/aiTypes.js';

export function pumasUnloadMessage(result: PumasUnserveResult): string {
  if (!result.success || typeof result.error === 'string') {
    throw new Error(result.error ?? 'Pumas could not unload the selected model');
  }
  return result.unloaded
    ? 'Selected model unloaded through Pumas'
    : 'Selected model was already not served by Pumas';
}
