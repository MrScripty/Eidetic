/**
 * CRDT document layer — Y.js integration.
 *
 * Provides a single shared Y.Doc instance with helpers to access per-node
 * Y.Text fields. The sync provider wires binary WebSocket frames directly
 * to/from the server's Y.Doc manager.
 */
import * as Y from 'yjs';

/** Shared Y.Doc — mirrors the server's authoritative CRDT state. */
export const ydoc = new Y.Doc();

/** Root Y.Map keyed by node UUID strings. */
const nodesMap = ydoc.getMap('nodes');

/**
 * Get the Y.Text for a node's "notes" field.
 * Creates the node map and text if they don't exist yet.
 */
export function getNodeNotes(nodeId: string): Y.Text {
	return getOrCreateTextField(nodeId, 'notes');
}

/**
 * Get the Y.Text for a node's "content" field (script / outline).
 * Creates the node map and text if they don't exist yet.
 */
export function getNodeContent(nodeId: string): Y.Text {
	return getOrCreateTextField(nodeId, 'content');
}

/**
 * Read a node's notes as a plain string (non-reactive snapshot).
 */
export function readNodeNotes(nodeId: string): string {
	const nodeMap = nodesMap.get(nodeId) as Y.Map<Y.Text> | undefined;
	if (!nodeMap) return '';
	const text = nodeMap.get('notes');
	return text ? text.toString() : '';
}

/**
 * Read a node's content as a plain string (non-reactive snapshot).
 */
export function readNodeContent(nodeId: string): string {
	const nodeMap = nodesMap.get(nodeId) as Y.Map<Y.Text> | undefined;
	if (!nodeMap) return '';
	const text = nodeMap.get('content');
	return text ? text.toString() : '';
}

// ─── Internal ─────────────────────────────────────────────────────

function getOrCreateTextField(nodeId: string, field: string): Y.Text {
	let nodeMap = nodesMap.get(nodeId) as Y.Map<Y.Text> | undefined;
	if (!nodeMap) {
		nodeMap = new Y.Map<Y.Text>();
		nodesMap.set(nodeId, nodeMap);
	}
	let text = nodeMap.get(field);
	if (!text) {
		text = new Y.Text();
		nodeMap.set(field, text);
	}
	return text;
}
