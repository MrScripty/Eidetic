interface Context {
  system: string;
  user: string;
}

interface Options {
  selectedNodeId: () => string | null;
  fetchContext: (nodeId: string) => Promise<Context>;
  setContext: (context: Context | null) => void;
  setLoading: (loading: boolean) => void;
}

export function createContextRequestLifecycle(options: Options) {
  let requestId = 0;
  let contextNodeId: string | null = null;
  let contextRevision = -1;

  function invalidate() {
    requestId += 1;
    contextNodeId = null;
    options.setContext(null);
    options.setLoading(false);
  }

  async function load(nodeId: string) {
    const id = ++requestId;
    const current = () => id === requestId && options.selectedNodeId() === nodeId;
    contextNodeId = nodeId;
    options.setContext(null);
    options.setLoading(true);
    try {
      const context = await options.fetchContext(nodeId);
      if (current()) options.setContext(context);
    } catch {
      if (current()) options.setContext(null);
    } finally {
      if (current()) options.setLoading(false);
    }
  }

  function update(nodeId: string | null, notes: string | undefined, revision: number) {
    if (!nodeId || !notes?.trim()) {
      invalidate();
      return;
    }
    if (nodeId === contextNodeId && revision === contextRevision) return;
    contextRevision = revision;
    void load(nodeId);
  }

  return { update, load, invalidate };
}
