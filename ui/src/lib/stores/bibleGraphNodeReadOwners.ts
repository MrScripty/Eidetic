/** Opaque async ownership tokens intentionally do not participate in Svelte effects. */
export function createBibleGraphNodeReadOwners() {
  return {
    requests: new Map<string, object>(),
    inspectors: new Map<string, Set<object>>(),
    createInspectors: () => new Set<object>(),
  };
}
