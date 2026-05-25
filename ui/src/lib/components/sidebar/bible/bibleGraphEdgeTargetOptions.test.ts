import { describe, expect, it } from 'vitest';

import { bibleGraphEdgeTargetOptions } from './bibleGraphEdgeTargetOptions.js';

describe('bibleGraphEdgeTargetOptions', () => {
  it('builds selectable target options from durable graph nodes', () => {
    expect(
      bibleGraphEdgeTargetOptions(
        [
          {
            id: 'node.character.ada',
            parent_id: 'canonical.characters',
            schema_key: 'character',
            name: 'Ada',
            system_owned: false,
            sort_order: 1,
          },
          {
            id: 'node.place.beach',
            parent_id: 'canonical.places',
            schema_key: 'location',
            name: 'Beach',
            system_owned: false,
            sort_order: 2,
          },
        ],
        'node.character.ada',
      ),
    ).toEqual([
      {
        nodeId: 'node.place.beach',
        label: 'Beach',
        schemaKey: 'location',
      },
    ]);
  });

  it('builds selectable target options from render graph nodes', () => {
    expect(
      bibleGraphEdgeTargetOptions(
        [
          {
            node_id: 'node.prop.lantern',
            parent_id: 'canonical.objects',
            schema_key: 'prop',
            label: 'Lantern',
            system_owned: false,
            sort_order: 2,
            depth: 1,
            position: { x: 0, y: 0, z: 0 },
          },
          {
            node_id: 'node.character.ada',
            parent_id: 'canonical.characters',
            schema_key: 'character',
            label: 'Ada',
            system_owned: false,
            sort_order: 1,
            depth: 1,
            position: { x: 1, y: 0, z: 0 },
          },
        ],
        'node.character.ada',
      ),
    ).toEqual([
      {
        nodeId: 'node.prop.lantern',
        label: 'Lantern',
        schemaKey: 'prop',
      },
    ]);
  });
});
