import type { Project, StoryArc, Character, AiStatus, AiConfig, TimelineGap } from './types.js';

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const res = await fetch(`${BASE}${path}`, {
		headers: { 'Content-Type': 'application/json' },
		...options,
	});
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

export function createArc(name: string, arc_type: string, color?: [number, number, number]): Promise<StoryArc> {
	return request('/arcs', {
		method: 'POST',
		body: JSON.stringify({ name, arc_type, color }),
	});
}

export function updateArc(id: string, updates: Record<string, unknown>): Promise<StoryArc> {
	return request(`/arcs/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteArc(id: string): Promise<{ deleted: boolean }> {
	return request(`/arcs/${id}`, { method: 'DELETE' });
}

// --- Characters ---

export function listCharacters(): Promise<Character[]> {
	return request('/characters');
}

export function createCharacter(name: string, color?: [number, number, number]): Promise<Character> {
	return request('/characters', {
		method: 'POST',
		body: JSON.stringify({ name, color }),
	});
}

export function updateCharacter(id: string, updates: Record<string, unknown>): Promise<Character> {
	return request(`/characters/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteCharacter(id: string): Promise<{ deleted: boolean }> {
	return request(`/characters/${id}`, { method: 'DELETE' });
}

// --- Timeline ---

export function getTimeline() {
	return request('/timeline');
}

export function createClip(trackId: string, name: string, beatType: string, startMs: number, endMs: number) {
	return request('/timeline/clips', {
		method: 'POST',
		body: JSON.stringify({ track_id: trackId, name, beat_type: beatType, start_ms: startMs, end_ms: endMs }),
	});
}

export function updateClip(id: string, updates: { name?: string; start_ms?: number; end_ms?: number }) {
	return request(`/timeline/clips/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteClip(id: string) {
	return request(`/timeline/clips/${id}`, { method: 'DELETE' });
}

export function splitClip(id: string, atMs: number) {
	return request(`/timeline/clips/${id}/split`, {
		method: 'POST',
		body: JSON.stringify({ at_ms: atMs }),
	});
}

// --- Beats ---

export function getBeat(id: string) {
	return request(`/beats/${id}`);
}

export function updateBeatNotes(id: string, notes: string) {
	return request(`/beats/${id}/notes`, {
		method: 'PUT',
		body: JSON.stringify({ notes }),
	});
}

export function updateBeatScript(id: string, script: string) {
	return request(`/beats/${id}/script`, {
		method: 'PUT',
		body: JSON.stringify({ script }),
	});
}

export function lockBeat(id: string) {
	return request(`/beats/${id}/lock`, { method: 'POST' });
}

export function unlockBeat(id: string) {
	return request(`/beats/${id}/unlock`, { method: 'POST' });
}

// --- Scenes ---

export function getScenes() {
	return request('/scenes');
}

// --- Relationships ---

export function createRelationship(fromClip: string, toClip: string, type_: string) {
	return request('/timeline/relationships', {
		method: 'POST',
		body: JSON.stringify({ from_clip: fromClip, to_clip: toClip, relationship_type: type_ }),
	});
}

export function deleteRelationship(id: string) {
	return request(`/timeline/relationships/${id}`, { method: 'DELETE' });
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
