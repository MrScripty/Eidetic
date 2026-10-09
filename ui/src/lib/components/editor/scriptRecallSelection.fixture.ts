import type { BibleRecallProjection } from '$lib/bibleRecallTypes.js';

export function packet(): BibleRecallProjection {
  return {
    request: {
      anchor_node_id: 'Mara',
      story_time_ms: null,
      direction: 'both',
      edge_kinds: [],
      neighbor_limit: 8,
    },
    nodes: [
      {
        node_id: 'Mara',
        name: 'Mara',
        schema_key: 'character',
        name_revision_event_id: 'mara.name',
        fields: [],
        unresolved_timed_fields: [],
        omitted_fields: 0,
      },
      {
        node_id: 'House',
        name: 'Beach House',
        schema_key: 'location',
        name_revision_event_id: 'house.name',
        fields: Array.from({ length: 9 }, (_, index) => ({
          part_key: 'profile',
          field_key: `fact${index}`,
          value: { type: 'text', value: `Exact fact ${index} — 雨\n\n` },
          source: {
            kind: 'baseline',
            field_id: `house.${index}`,
            revision_event_id: `revision.${index}`,
          },
        })),
        unresolved_timed_fields: [{ part_key: 'profile', field_key: 'unresolved' }],
        omitted_fields: 1,
      },
    ],
    paths: [
      {
        neighbor_node_id: 'House',
        relationship: {
          revision_event_id: 'path.revision',
          edge: {
            edge_id: 'Mara.House',
            from_node_id: 'Mara',
            to_node_id: 'House',
            edge_kind: 'located_in',
            label: 'untimed home',
            directed: true,
          },
        },
      },
    ],
    omitted_neighbors: 0,
    omitted_edges: 0,
    relationships_untimed: true,
  };
}
