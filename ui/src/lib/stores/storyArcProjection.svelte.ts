import { createStoryArc, deleteStoryArc, setStoryArcMetadata } from '$lib/commandApi.js';
import { getStoryArcListProjection } from '$lib/projectionApi.js';
import type { CommandId, ProjectionEnvelope } from '$lib/types.js';
import type {
  CreateStoryArcCommand,
  DeleteStoryArcCommand,
  SetStoryArcMetadataCommand,
  StoryArcCommandResponse,
  StoryArcListProjection,
} from '$lib/storyArcTypes.js';

export const storyArcProjectionState = $state<{
  projection: ProjectionEnvelope<StoryArcListProjection> | null;
  pending: boolean;
  error?: string;
}>({
  projection: null,
  pending: false,
  error: undefined,
});

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function cacheProjection(projection: ProjectionEnvelope<StoryArcListProjection>): void {
  storyArcProjectionState.projection = projection;
}

export function getCachedStoryArcListProjection(): ProjectionEnvelope<StoryArcListProjection> | null {
  return storyArcProjectionState.projection;
}

export async function refreshStoryArcListProjection(): Promise<
  ProjectionEnvelope<StoryArcListProjection>
> {
  storyArcProjectionState.pending = true;
  storyArcProjectionState.error = undefined;

  try {
    const projection = await getStoryArcListProjection();
    cacheProjection(projection);
    return projection;
  } catch (error) {
    storyArcProjectionState.error = errorMessage(error, 'Failed to load story arc projection');
    throw error;
  } finally {
    storyArcProjectionState.pending = false;
  }
}

export async function applyCreateStoryArcCommand(
  payload: CreateStoryArcCommand,
  commandId?: CommandId,
): Promise<StoryArcCommandResponse> {
  storyArcProjectionState.pending = true;
  storyArcProjectionState.error = undefined;

  try {
    const response = await createStoryArc(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    storyArcProjectionState.error = errorMessage(error, 'Failed to create story arc');
    throw error;
  } finally {
    storyArcProjectionState.pending = false;
  }
}

export async function applySetStoryArcMetadataCommand(
  payload: SetStoryArcMetadataCommand,
  commandId?: CommandId,
): Promise<StoryArcCommandResponse> {
  storyArcProjectionState.pending = true;
  storyArcProjectionState.error = undefined;

  try {
    const response = await setStoryArcMetadata(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    storyArcProjectionState.error = errorMessage(error, 'Failed to update story arc');
    throw error;
  } finally {
    storyArcProjectionState.pending = false;
  }
}

export async function applyDeleteStoryArcCommand(
  payload: DeleteStoryArcCommand,
  commandId?: CommandId,
): Promise<StoryArcCommandResponse> {
  storyArcProjectionState.pending = true;
  storyArcProjectionState.error = undefined;

  try {
    const response = await deleteStoryArc(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    storyArcProjectionState.error = errorMessage(error, 'Failed to delete story arc');
    throw error;
  } finally {
    storyArcProjectionState.pending = false;
  }
}

export function clearStoryArcListProjection(): void {
  storyArcProjectionState.projection = null;
  storyArcProjectionState.pending = false;
  storyArcProjectionState.error = undefined;
}
