import type { BibleRecallProjection } from '$lib/bibleRecallTypes.js';
import type { ScriptRecallSelection } from '$lib/propagationProposalTypes.js';
import type { BibleGraphEdgeKind } from '$lib/bibleGraphTypes.js';

function copyKind(kind: BibleGraphEdgeKind): BibleGraphEdgeKind {
  return typeof kind === 'string' ? kind : { ...kind };
}

export function buildScriptRecallSelection(
  packet: BibleRecallProjection | null,
  fieldIds: string[],
): ScriptRecallSelection | null {
  if (fieldIds.length === 0) return null;
  if (!packet || packet.request.story_time_ms !== null || fieldIds.length > 8) {
    throw new Error('Select up to eight baseline facts recalled at unspecified story time.');
  }
  if (new Set(fieldIds).size !== fieldIds.length) throw new Error('Duplicate fact selection.');
  const facts: ScriptRecallSelection['facts'] = [];
  const nodeIds = new Set<string>();
  for (const id of fieldIds) {
    const node = packet.nodes.find((node) =>
      node.fields.some((field) => field.source.kind === 'baseline' && field.source.field_id === id),
    );
    const field = node?.fields.find(
      (field) => field.source.kind === 'baseline' && field.source.field_id === id,
    );
    if (!node || !field || field.source.kind !== 'baseline') {
      throw new Error(
        'Timed, unresolved, omitted or unavailable facts cannot be selected. Recall again.',
      );
    }
    facts.push({
      node_id: node.node_id,
      part_key: field.part_key,
      field_key: field.field_key,
      field_id: field.source.field_id,
      revision_event_id: field.source.revision_event_id,
    });
    nodeIds.add(node.node_id);
  }
  const paths = packet.paths
    .filter((path) => nodeIds.has(path.neighbor_node_id))
    .map((path) => ({
      neighbor_node_id: path.neighbor_node_id,
      relationship: {
        revision_event_id: path.relationship.revision_event_id,
        edge: { ...path.relationship.edge, edge_kind: copyKind(path.relationship.edge.edge_kind) },
      },
    }));
  if (paths.length) nodeIds.add(packet.request.anchor_node_id);
  const names = packet.nodes
    .filter((node) => nodeIds.has(node.node_id))
    .map((node) => {
      if (!node.name_revision_event_id)
        throw new Error('Entity name source is unknown. Recall again.');
      return {
        node_id: node.node_id,
        name: node.name,
        revision_event_id: node.name_revision_event_id,
      };
    })
    .sort((a, b) => a.node_id.localeCompare(b.node_id));
  return {
    query: { ...packet.request, edge_kinds: packet.request.edge_kinds.map(copyKind) },
    facts,
    names,
    paths,
  };
}
