import type { BibleGraphEdgeKind } from '$lib/bibleGraphTypes.js';

export interface GraphWorkspaceEdgeKindFilter {
  kind: BibleGraphEdgeKind;
  label: string;
}

export const graphWorkspaceEdgeKindFilters: GraphWorkspaceEdgeKindFilter[] = [
  { kind: 'references', label: 'References' },
  { kind: 'located_in', label: 'Located In' },
  { kind: 'owns', label: 'Owns' },
  { kind: 'member_of', label: 'Member Of' },
  { kind: 'conflicts_with', label: 'Conflicts' },
  { kind: 'supports_theme', label: 'Theme' },
];

export function toggleGraphWorkspaceEdgeKindFilter(
  selectedKinds: BibleGraphEdgeKind[],
  kind: BibleGraphEdgeKind,
): BibleGraphEdgeKind[] {
  if (selectedKinds.includes(kind)) {
    return selectedKinds.filter((selectedKind) => selectedKind !== kind);
  }
  return [...selectedKinds, kind];
}
