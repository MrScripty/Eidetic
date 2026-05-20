type RefreshTask = () => Promise<unknown>;

interface RefreshQueueEntry {
  task: RefreshTask;
  scheduled: boolean;
  running: boolean;
  rerunRequested: boolean;
  waiters: Array<() => void>;
}

const refreshQueue = new Map<string, RefreshQueueEntry>();

export function requestProjectionRefresh(key: string, task: RefreshTask): Promise<void> {
  const entry = refreshQueue.get(key) ?? {
    task,
    scheduled: false,
    running: false,
    rerunRequested: false,
    waiters: [],
  };
  entry.task = task;
  refreshQueue.set(key, entry);
  const done = new Promise<void>((resolve) => {
    entry.waiters.push(resolve);
  });

  if (entry.running) {
    entry.rerunRequested = true;
    return done;
  }

  if (entry.scheduled) return done;

  entry.scheduled = true;
  queueMicrotask(() => {
    void runRefresh(key);
  });

  return done;
}

export function clearProjectionRefreshQueue(): void {
  for (const entry of refreshQueue.values()) {
    for (const resolve of entry.waiters.splice(0)) {
      resolve();
    }
  }
  refreshQueue.clear();
}

async function runRefresh(key: string): Promise<void> {
  const entry = refreshQueue.get(key);
  if (!entry) return;

  do {
    entry.scheduled = false;
    entry.running = true;
    entry.rerunRequested = false;
    try {
      await entry.task();
    } catch {
      // Projection stores keep the last confirmed data and expose their own bounded error state.
    } finally {
      entry.running = false;
    }
  } while (entry.rerunRequested);

  for (const resolve of entry.waiters.splice(0)) {
    resolve();
  }
  refreshQueue.delete(key);
}
