import type { Project, StoryArc, Entity, EntityCategory, EntityDetails, EntitySnapshot, EntityRelation, ExtractionResult, AiStatus, AiConfig, TimelineGap, ReferenceDocument, ArcProgression, Timeline, ArcTrack, BeatClip, BeatContent, InferredScene, Relationship, ArcType, RelationshipType, ReferenceType, Color } from './types.js';

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		headers: { 'Content-Type': 'application/json' },
		...options,
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({}));
		throw new Error((body as Record<string, string>).error || `HTTP ${res.status}`);
	}
	return res.json() as Promise<T>;
}

// --- Project ---

export function createProject(name: string, template: string): Promise<Project> {
	return request('/project', {
		method: 'POST',
		body: JSON.stringify({ name, template }),
	});
}

export function getProject(): Promise<Project> {
	return request('/project');
}

export function updateProject(updates: { name?: string }): Promise<Project> {
	return request('/project', {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

// --- Story arcs ---

export function listArcs(): Promise<StoryArc[]> {
	return request('/arcs');
}

export function createArc(name: string, arc_type: ArcType, color?: [number, number, number]): Promise<StoryArc> {
	return request('/arcs', {
		method: 'POST',
		body: JSON.stringify({ name, arc_type, color }),
	});
}

export function updateArc(id: string, updates: Partial<Omit<StoryArc, 'id'>>): Promise<StoryArc> {
	return request(`/arcs/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteArc(id: string): Promise<{ deleted: boolean }> {
	return request(`/arcs/${id}`, { method: 'DELETE' });
}

export function getArcProgression(): Promise<ArcProgression[]> {
	return request('/arcs/progression');
}

// --- Bible entities ---

export function listEntities(): Promise<Entity[]> {
	return request('/bible/entities');
}

export function getEntity(id: string): Promise<Entity> {
	return request(`/bible/entities/${id}`);
}

export function createEntity(data: {
	name: string;
	category: EntityCategory;
	tagline?: string;
	description?: string;
	details?: EntityDetails;
	color?: Color;
}): Promise<Entity> {
	return request('/bible/entities', {
		method: 'POST',
		body: JSON.stringify(data),
	});
}

export function updateEntity(id: string, updates: Partial<Omit<Entity, 'id'>>): Promise<Entity> {
	return request(`/bible/entities/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteEntity(id: string): Promise<{ deleted: boolean }> {
	return request(`/bible/entities/${id}`, { method: 'DELETE' });
}

// --- Entity snapshots ---

export function addSnapshot(entityId: string, snapshot: Omit<EntitySnapshot, 'source_clip_id'> & { source_clip_id?: string | null }): Promise<Entity> {
	return request(`/bible/entities/${entityId}/snapshots`, {
		method: 'POST',
		body: JSON.stringify(snapshot),
	});
}

export function updateSnapshot(entityId: string, idx: number, snapshot: Partial<EntitySnapshot>): Promise<Entity> {
	return request(`/bible/entities/${entityId}/snapshots/${idx}`, {
		method: 'PUT',
		body: JSON.stringify(snapshot),
	});
}

export function deleteSnapshot(entityId: string, idx: number): Promise<Entity> {
	return request(`/bible/entities/${entityId}/snapshots/${idx}`, { method: 'DELETE' });
}

// --- Entity relations ---

export function addRelation(entityId: string, relation: EntityRelation): Promise<Entity> {
	return request(`/bible/entities/${entityId}/relations`, {
		method: 'POST',
		body: JSON.stringify(relation),
	});
}

export function deleteRelation(entityId: string, idx: number): Promise<Entity> {
	return request(`/bible/entities/${entityId}/relations/${idx}`, { method: 'DELETE' });
}

// --- Entity clip refs ---

export function addClipRef(entityId: string, clipId: string): Promise<Entity> {
	return request(`/bible/entities/${entityId}/clip-refs`, {
		method: 'POST',
		body: JSON.stringify({ clip_id: clipId }),
	});
}

export function removeClipRef(entityId: string, clipId: string): Promise<Entity> {
	return request(`/bible/entities/${entityId}/clip-refs/${clipId}`, { method: 'DELETE' });
}

// --- Entity extraction ---

export function extractEntities(clipId: string): Promise<ExtractionResult> {
	return request('/ai/extract', {
		method: 'POST',
		body: JSON.stringify({ clip_id: clipId }),
	});
}

export function commitExtraction(
	clipId: string,
	result: ExtractionResult,
	acceptedEntities: boolean[],
	acceptedSnapshots: boolean[],
): Promise<{ new_entity_count: number; snapshot_count: number }> {
	return request('/ai/extract/commit', {
		method: 'POST',
		body: JSON.stringify({
			clip_id: clipId,
			result,
			accepted_entities: acceptedEntities,
			accepted_snapshots: acceptedSnapshots,
		}),
	});
}

// --- Bible resolve at time ---

export function resolveEntitiesAtTime(timeMs: number): Promise<Entity[]> {
	return request(`/bible/at?time_ms=${timeMs}`);
}

// --- Timeline ---

export function getTimeline(): Promise<Timeline> {
	return request('/timeline');
}

export function createClip(trackId: string, name: string, beatType: string, startMs: number, endMs: number): Promise<BeatClip> {
	return request('/timeline/clips', {
		method: 'POST',
		body: JSON.stringify({ track_id: trackId, name, beat_type: beatType, start_ms: startMs, end_ms: endMs }),
	});
}

export function updateClip(id: string, updates: { name?: string; start_ms?: number; end_ms?: number }): Promise<BeatClip> {
	return request(`/timeline/clips/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteClip(id: string): Promise<{ deleted: boolean }> {
	return request(`/timeline/clips/${id}`, { method: 'DELETE' });
}

export function splitClip(id: string, atMs: number): Promise<{ left: BeatClip; right: BeatClip }> {
	return request(`/timeline/clips/${id}/split`, {
		method: 'POST',
		body: JSON.stringify({ at_ms: atMs }),
	});
}

// --- Beats ---

export function getBeat(id: string): Promise<BeatContent> {
	return request(`/beats/${id}`);
}

export function updateBeatNotes(id: string, notes: string): Promise<BeatContent> {
	return request(`/beats/${id}/notes`, {
		method: 'PUT',
		body: JSON.stringify({ notes }),
	});
}

export function updateBeatScript(id: string, script: string): Promise<BeatContent> {
	return request(`/beats/${id}/script`, {
		method: 'PUT',
		body: JSON.stringify({ script }),
	});
}

export function lockBeat(id: string): Promise<BeatContent> {
	return request(`/beats/${id}/lock`, { method: 'POST' });
}

export function unlockBeat(id: string): Promise<BeatContent> {
	return request(`/beats/${id}/unlock`, { method: 'POST' });
}

// --- Scenes ---

export function getScenes(): Promise<InferredScene[]> {
	return request('/scenes');
}

// --- Relationships ---

export function createRelationship(fromClip: string, toClip: string, type_: RelationshipType): Promise<Relationship> {
	return request('/timeline/relationships', {
		method: 'POST',
		body: JSON.stringify({ from_clip: fromClip, to_clip: toClip, relationship_type: type_ }),
	});
}

export function deleteRelationship(id: string): Promise<{ deleted: boolean }> {
	return request(`/timeline/relationships/${id}`, { method: 'DELETE' });
}

// --- Tracks ---

export function addTrack(arcId: string): Promise<ArcTrack> {
	return request('/timeline/tracks', {
		method: 'POST',
		body: JSON.stringify({ arc_id: arcId }),
	});
}

export function removeTrack(trackId: string): Promise<ArcTrack> {
	return request(`/timeline/tracks/${trackId}`, { method: 'DELETE' });
}

// --- Gaps ---

export function getGaps(): Promise<TimelineGap[]> {
	return request('/timeline/gaps');
}

export function fillGap(trackId: string, startMs: number, endMs: number): Promise<unknown> {
	return request('/timeline/gaps/fill', {
		method: 'POST',
		body: JSON.stringify({ track_id: trackId, start_ms: startMs, end_ms: endMs }),
	});
}

export function closeGap(trackId: string, gapEndMs: number): Promise<Timeline> {
	return request(`/timeline/tracks/${trackId}/close-gap`, {
		method: 'POST',
		body: JSON.stringify({ gap_end_ms: gapEndMs }),
	});
}

export function closeAllGaps(trackId: string): Promise<Timeline> {
	return request(`/timeline/tracks/${trackId}/close-all-gaps`, {
		method: 'POST',
	});
}

// --- References ---

export function listReferences(): Promise<ReferenceDocument[]> {
	return request('/references');
}

export function uploadReference(name: string, content: string, docType: ReferenceType): Promise<ReferenceDocument> {
	return request('/references', {
		method: 'POST',
		body: JSON.stringify({ name, content, doc_type: docType }),
	});
}

export function deleteReference(id: string): Promise<{ deleted: boolean }> {
	return request(`/references/${id}`, { method: 'DELETE' });
}

// --- AI ---

export function generateScript(clipId: string): Promise<{ status: string; clip_id: string }> {
	return request('/ai/generate', {
		method: 'POST',
		body: JSON.stringify({ clip_id: clipId }),
	});
}

export function getAiStatus(): Promise<AiStatus> {
	return request('/ai/status');
}

export function updateAiConfig(updates: Partial<AiConfig>): Promise<AiConfig> {
	return request('/ai/config', {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function reactToEdit(clipId: string): Promise<{ status: string }> {
	return request('/ai/react', {
		method: 'POST',
		body: JSON.stringify({ clip_id: clipId }),
	});
}

// --- Export ---

export async function exportPdf(): Promise<Blob> {
	const res = await fetch(`${BASE}/export/pdf`, { method: 'POST' });
	if (!res.ok) {
		const err = await res.json();
		throw new Error(err.error || 'Export failed');
	}
	return res.blob();
}

// --- Undo/Redo ---

export function undo(): Promise<Project> {
	return request('/project/undo', { method: 'POST' });
}

export function redo(): Promise<Project> {
	return request('/project/redo', { method: 'POST' });
}

// --- Persistence ---

export function saveProject(path?: string): Promise<{ saved?: string; error?: string }> {
	return request('/project/save', {
		method: 'POST',
		body: JSON.stringify({ path }),
	});
}

export function loadProject(path: string): Promise<Project> {
	return request('/project/load', {
		method: 'POST',
		body: JSON.stringify({ path }),
	});
}

export function listProjects(): Promise<{ name: string; path: string; modified: string }[]> {
	return request('/project/list');
}
