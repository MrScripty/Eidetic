import type {
	Project, StoryArc, Entity, EntityCategory, EntityDetails, EntitySnapshot,
	EntityRelation, ExtractionResult, AiStatus, AiConfig, TimelineGap,
	ReferenceDocument, ArcProgression, Timeline, Track, StoryNode,
	NodeContent, ChildPlan, ChildProposal, Relationship, ArcType,
	RelationshipType, ReferenceType, Color, StoryLevel,
} from './types.js';

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

export function updateProject(updates: { name?: string; premise?: string }): Promise<Project> {
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

export function addSnapshot(entityId: string, snapshot: Omit<EntitySnapshot, 'source_node_id'> & { source_node_id?: string | null }): Promise<Entity> {
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

// --- Entity node refs ---

export function addNodeRef(entityId: string, nodeId: string): Promise<Entity> {
	return request(`/bible/entities/${entityId}/node-refs`, {
		method: 'POST',
		body: JSON.stringify({ node_id: nodeId }),
	});
}

export function removeNodeRef(entityId: string, nodeId: string): Promise<Entity> {
	return request(`/bible/entities/${entityId}/node-refs/${nodeId}`, { method: 'DELETE' });
}

// --- Entity extraction ---

export function extractEntities(nodeId: string): Promise<ExtractionResult> {
	return request('/ai/extract', {
		method: 'POST',
		body: JSON.stringify({ node_id: nodeId }),
	});
}

export function commitExtraction(
	nodeId: string,
	result: ExtractionResult,
	acceptedEntities: boolean[],
	acceptedSnapshots: boolean[],
): Promise<{ new_entity_count: number; snapshot_count: number }> {
	return request('/ai/extract/commit', {
		method: 'POST',
		body: JSON.stringify({
			node_id: nodeId,
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

// --- Nodes ---

export function createNode(data: {
	parent_id?: string | null;
	level: StoryLevel;
	name: string;
	beat_type?: string | null;
	start_ms: number;
	end_ms: number;
}): Promise<StoryNode> {
	return request('/timeline/nodes', {
		method: 'POST',
		body: JSON.stringify(data),
	});
}

export function updateNode(id: string, updates: { name?: string; start_ms?: number; end_ms?: number }): Promise<StoryNode> {
	return request(`/timeline/nodes/${id}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function deleteNode(id: string): Promise<{ deleted: boolean }> {
	return request(`/timeline/nodes/${id}`, { method: 'DELETE' });
}

export function splitNode(id: string, atMs: number): Promise<{ left: StoryNode; right: StoryNode }> {
	return request(`/timeline/nodes/${id}/split`, {
		method: 'POST',
		body: JSON.stringify({ at_ms: atMs }),
	});
}

export function resizeNode(id: string, startMs: number, endMs: number): Promise<StoryNode> {
	return request(`/timeline/nodes/${id}/resize`, {
		method: 'POST',
		body: JSON.stringify({ start_ms: startMs, end_ms: endMs }),
	});
}

export function getNodeChildren(id: string): Promise<StoryNode[]> {
	return request(`/timeline/nodes/${id}/children`);
}

export function applyChildren(nodeId: string, children: ChildProposal[]): Promise<{ ok: boolean; children: StoryNode[] }> {
	return request(`/timeline/nodes/${nodeId}/apply-children`, {
		method: 'POST',
		body: JSON.stringify({ children }),
	});
}

// --- Node content ---

export function getNodeContent(id: string): Promise<NodeContent> {
	return request(`/nodes/${id}/content`);
}

export function updateNodeNotes(id: string, notes: string): Promise<NodeContent> {
	return request(`/nodes/${id}/notes`, {
		method: 'PUT',
		body: JSON.stringify({ notes }),
	});
}

export function updateNodeScript(id: string, script: string): Promise<NodeContent> {
	return request(`/nodes/${id}/script`, {
		method: 'PUT',
		body: JSON.stringify({ script }),
	});
}

export function lockNode(id: string): Promise<NodeContent> {
	return request(`/nodes/${id}/lock`, { method: 'POST' });
}

export function unlockNode(id: string): Promise<NodeContent> {
	return request(`/nodes/${id}/unlock`, { method: 'POST' });
}

// --- Relationships ---

export function createRelationship(fromNode: string, toNode: string, type_: RelationshipType): Promise<Relationship> {
	return request('/timeline/relationships', {
		method: 'POST',
		body: JSON.stringify({ from_node: fromNode, to_node: toNode, relationship_type: type_ }),
	});
}

export function deleteRelationship(id: string): Promise<{ deleted: boolean }> {
	return request(`/timeline/relationships/${id}`, { method: 'DELETE' });
}

// --- Tracks ---

export function createTrack(level: StoryLevel, label: string): Promise<Track> {
	return request('/timeline/tracks', {
		method: 'POST',
		body: JSON.stringify({ level, label }),
	});
}

export function updateTrack(trackId: string, updates: { label?: string; collapsed?: boolean }): Promise<Track> {
	return request(`/timeline/tracks/${trackId}`, {
		method: 'PUT',
		body: JSON.stringify(updates),
	});
}

export function removeTrack(trackId: string): Promise<{ deleted: boolean }> {
	return request(`/timeline/tracks/${trackId}`, { method: 'DELETE' });
}

// --- Node-Arc tagging ---

export function tagNodeWithArc(nodeId: string, arcId: string): Promise<{ ok: boolean }> {
	return request('/timeline/node-arcs', {
		method: 'POST',
		body: JSON.stringify({ node_id: nodeId, arc_id: arcId }),
	});
}

export function untagNodeFromArc(nodeId: string, arcId: string): Promise<{ ok: boolean }> {
	return request(`/timeline/node-arcs/${nodeId}/${arcId}`, { method: 'DELETE' });
}

// --- Gaps ---

export function getGaps(level?: StoryLevel): Promise<TimelineGap[]> {
	const query = level ? `?level=${level}` : '';
	return request(`/timeline/gaps${query}`);
}

export function fillGap(level: StoryLevel, startMs: number, endMs: number): Promise<unknown> {
	return request('/timeline/gaps/fill', {
		method: 'POST',
		body: JSON.stringify({ level, start_ms: startMs, end_ms: endMs }),
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

export function generateContent(nodeId: string): Promise<{ status: string; node_id: string }> {
	return request('/ai/generate', {
		method: 'POST',
		body: JSON.stringify({ node_id: nodeId }),
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

export function reactToEdit(nodeId: string): Promise<{ status: string }> {
	return request('/ai/react', {
		method: 'POST',
		body: JSON.stringify({ node_id: nodeId }),
	});
}

export function getAiContext(nodeId: string): Promise<{ system: string; user: string }> {
	return request(`/ai/context/${nodeId}`);
}

export function generateChildren(nodeId: string): Promise<ChildPlan> {
	return request('/ai/generate-children', {
		method: 'POST',
		body: JSON.stringify({ node_id: nodeId }),
	});
}

export function generateBatch(parentNodeId: string): Promise<{ status: string; parent_node_id: string; child_count: number }> {
	return request('/ai/generate-batch', {
		method: 'POST',
		body: JSON.stringify({ parent_node_id: parentNodeId }),
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
