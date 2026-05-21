/** Compatibility barrel for frontend API contracts. Prefer importing from focused modules. */

export type {
  ArcProgression,
  ArcId,
  ArcType,
  Color,
  ProgressionIssue,
  Severity,
  StoryArc,
} from './storyArcTypes.js';
export { colorToHex } from './storyArcTypes.js';

export type {
  BeatType,
  CharacterId,
  ContentStatus,
  EpisodeStructure,
  NodeArc,
  NodeContent,
  NodeId,
  Relationship,
  RelationshipId,
  RelationshipType,
  SegmentType,
  StoryLevel,
  StoryNode,
  StructureSegment,
  TimeRange,
  Timeline,
  TimelineGap,
  Track,
  TrackId,
} from './timelineTypes.js';
export {
  bestText,
  childLevel,
  formatTime,
  mainTimelinePanelHeightPx,
  PANEL,
  parentLevel,
  TIMELINE,
  timelineTrackRowsHeightPx,
} from './timelineTypes.js';

export type {
  ChangeEventId,
  CommandEnvelope,
  CommandId,
  CommandOutcome,
  FieldValue,
  ObjectFieldCommandResponse,
  ObjectFieldProjection,
  ObjectKind,
  ProjectionEnvelope,
  SetObjectFieldCommand,
} from './projectionTypes.js';

export * from './bibleGraphTypes.js';
export * from './scriptTypes.js';
export * from './timelineCommandTypes.js';
export * from './timelineRenderTypes.js';

export type { ChildPlan, ChildProposal } from './childPlanningTypes.js';
export type { Project, ReferenceDocument, ReferenceId, ReferenceType } from './projectTypes.js';

export type { AiConfig, AiStatus, BackendType, ModelEntry, ModelListResponse } from './aiTypes.js';

export type { ServerMessage } from './serverEventTypes.js';
