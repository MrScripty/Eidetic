import { describe, expect, it } from 'vitest';

import { toggleGraphWorkspaceEdgeKindFilter } from './graphWorkspaceEdgeKindFilters.js';

describe('graph workspace edge kind filters', () => {
  it('adds an edge kind to the selected projection filters', () => {
    expect(toggleGraphWorkspaceEdgeKindFilter(['references'], 'located_in')).toEqual([
      'references',
      'located_in',
    ]);
  });

  it('removes an existing edge kind from the selected projection filters', () => {
    expect(toggleGraphWorkspaceEdgeKindFilter(['references', 'located_in'], 'references')).toEqual([
      'located_in',
    ]);
  });
});
