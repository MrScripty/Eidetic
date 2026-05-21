interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

export type DesktopUnlisten = () => void;

interface TauriEvent<T = unknown> {
  payload: T;
}

interface TauriEventApi {
  listen<T>(event: string, handler: (event: TauriEvent<T>) => void): Promise<DesktopUnlisten>;
}

interface TauriGlobal {
  core?: TauriCore;
  event?: TauriEventApi;
}

declare global {
  interface Window {
    __TAURI__?: TauriGlobal;
  }
}

interface DesktopCommandError {
  kind?: string;
  message?: string;
}

function tauriCore(): TauriCore | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.__TAURI__?.core ?? null;
}

function tauriEventApi(): TauriEventApi | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.__TAURI__?.event ?? null;
}

export function hasDesktopEventTransport(): boolean {
  return tauriEventApi() !== null;
}

export async function listenDesktop<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<DesktopUnlisten> {
  const eventApi = tauriEventApi();
  if (!eventApi) {
    throw new Error('desktop event transport is unavailable');
  }

  return eventApi.listen<T>(event, ({ payload }) => {
    handler(payload);
  });
}

export async function invokeDesktop<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const core = tauriCore();
  if (!core) {
    throw new Error('desktop transport is unavailable');
  }

  try {
    return await core.invoke<T>(command, args);
  } catch (error) {
    const commandError = error as DesktopCommandError;
    if (typeof commandError?.message === 'string') {
      throw new Error(commandError.message);
    }
    if (typeof error === 'string') {
      throw new Error(error);
    }
    throw error;
  }
}
