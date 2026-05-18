import type { Timeline } from './types.js';

export type ReferenceId = string;

export type ReferenceType =
  | 'CharacterBible'
  | 'StyleGuide'
  | 'WorldBuilding'
  | 'PreviousEpisode'
  | { Custom: string };

export interface ReferenceDocument {
  id: ReferenceId;
  name: string;
  content: string;
  doc_type: ReferenceType;
}

export interface Project {
  name: string;
  premise: string;
  timeline: Timeline;
  references?: ReferenceDocument[];
}
