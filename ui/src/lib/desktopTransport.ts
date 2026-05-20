interface TauriCore {
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

interface TauriGlobal {
  core?: TauriCore;
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

export function hasDesktopTransport(): boolean {
  return tauriCore() !== null;
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
