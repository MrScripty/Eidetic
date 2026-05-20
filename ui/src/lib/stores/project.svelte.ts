export interface ActiveProjectMetadata {
  name: string;
}

/** Frontend-owned active project session metadata. Durable project data is projection-backed. */
export const projectState = $state<{ current: ActiveProjectMetadata | null }>({ current: null });
