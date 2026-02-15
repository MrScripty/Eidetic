/** Mirrors Rust types from eidetic-core for type-safe API communication. */

// --- IDs (all UUIDs as strings) ---

export type TrackId = string;
export type ClipId = string;
export type ArcId = string;
export type CharacterId = string;
export type RelationshipId = string;

// --- Timeline ---

export interface Timeline {
	total_duration_ms: number;
	tracks: ArcTrack[];
	relationships: Relationship[];
	structure: EpisodeStructure;
}

export interface ArcTrack {
	id: TrackId;
	arc_id: ArcId;
	clips: BeatClip[];
}

export interface BeatClip {
	id: ClipId;
	time_range: TimeRange;
	beat_type: BeatType;
	name: string;
	content: BeatContent;
	locked: boolean;
}

export interface TimeRange {
	start_ms: number;
	end_ms: number;
}

export type BeatType =
	| 'Setup'
	| 'Complication'
	| 'Escalation'
	| 'Climax'
	| 'Resolution'
	| 'Payoff'
	| 'Callback'
	| { Custom: string };

export interface BeatContent {
	beat_notes: string;
	generated_script: string | null;
	user_refined_script: string | null;
	status: ContentStatus;
	/** Compact structured recap of scene end state for continuity. */
	scene_recap?: string | null;
}

export type ContentStatus =
	| 'Empty'
	| 'NotesOnly'
	| 'Generating'
	| 'Generated'
	| 'UserRefined'
	| 'UserWritten';

// --- Relationships ---

export interface Relationship {
	id: RelationshipId;
	from_clip: ClipId;
	to_clip: ClipId;
	relationship_type: RelationshipType;
}

export type RelationshipType =
	| 'Causal'
	| { Convergence: { arc_ids: ArcId[] } }
	| { EntityDrives: { entity_id: string } }
	| 'Thematic';

// --- Episode structure ---

export interface EpisodeStructure {
	template_name: string;
	segments: StructureSegment[];
}

export interface StructureSegment {
	segment_type: SegmentType;
	time_range: TimeRange;
	label: string;
}

export type SegmentType = 'ColdOpen' | 'MainTitles' | 'Act' | 'CommercialBreak' | 'Tag';

// --- Story entities ---

export interface StoryArc {
	id: ArcId;
	name: string;
	description: string;
	arc_type: ArcType;
	color: Color;
}

export type ArcType = 'APlot' | 'BPlot' | 'CRunner' | { Custom: string };

export interface Color {
	r: number;
	g: number;
	b: number;
}

// --- Story Bible ---

export type EntityId = string;

export type EntityCategory = 'Character' | 'Location' | 'Prop' | 'Theme' | 'Event';

export interface Entity {
	id: EntityId;
	category: EntityCategory;
	name: string;
	tagline: string;
	description: string;
	details: EntityDetails;
	snapshots: EntitySnapshot[];
	clip_refs: ClipId[];
	relations: EntityRelation[];
	color: Color;
	locked: boolean;
}

export type EntityDetails =
	| { type: 'Character'; traits: string[]; voice_notes: string; character_relations: [EntityId, string][]; audience_knowledge: string }
	| { type: 'Location'; int_ext: string; scene_heading_name: string; atmosphere: string }
	| { type: 'Prop'; owner_entity_id: EntityId | null; significance: string }
	| { type: 'Theme'; manifestation: string }
	| { type: 'Event'; timeline_ms: number | null; is_backstory: boolean; involved_entity_ids: EntityId[] };

export interface EntitySnapshot {
	at_ms: number;
	source_clip_id: ClipId | null;
	description: string;
	state_overrides?: SnapshotOverrides | null;
}

export interface SnapshotOverrides {
	traits?: string[];
	audience_knowledge?: string;
	emotional_state?: string;
	atmosphere?: string;
	owner_entity_id?: EntityId | null;
	significance?: string;
	custom?: [string, string][];
	/** Where this entity is located at this point in the timeline. */
	location?: string;
}

export interface EntityRelation {
	target_entity_id: EntityId;
	label: string;
}

export interface StoryBible {
	entities: Entity[];
}

export interface ExtractionResult {
	new_entities: SuggestedEntity[];
	snapshot_suggestions: SuggestedSnapshot[];
	/** Names of all entities (existing or new) present in the scene. */
	entities_present: string[];
}

export interface SuggestedEntity {
	name: string;
	category: EntityCategory;
	tagline: string;
	description: string;
}

export interface SuggestedSnapshot {
	entity_name: string;
	description: string;
	emotional_state?: string;
	audience_knowledge?: string;
	/** Where this entity is located. */
	location?: string;
}

// --- Scenes (inferred) ---

export interface InferredScene {
	time_range: TimeRange;
	active_arcs: ArcId[];
	contributing_clips: ClipId[];
}

// --- Timeline gaps ---

export interface TimelineGap {
	track_id: TrackId;
	arc_id: ArcId;
	time_range: TimeRange;
	preceding_clip_id: ClipId | null;
	following_clip_id: ClipId | null;
}

// --- Reference Documents ---

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

// --- Project ---

export interface Project {
	name: string;
	timeline: Timeline;
	arcs: StoryArc[];
	bible: StoryBible;
	references?: ReferenceDocument[];
}

// --- UI constants ---

export const TIMELINE = {
	/** Total episode duration in ms (22 min). */
	DURATION_MS: 1_320_000,
	/** Minimum clip width in pixels. */
	MIN_CLIP_WIDTH_PX: 20,
	/** Pixels per millisecond at default zoom (1x). */
	DEFAULT_PX_PER_MS: 0.08,
	/** Track lane height in pixels. */
	TRACK_HEIGHT_PX: 48,
	/** Gap between tracks. */
	TRACK_GAP_PX: 4,
	/** Height of the structure bar. */
	STRUCTURE_BAR_HEIGHT_PX: 32,
	/** Height of the time ruler. */
	TIME_RULER_HEIGHT_PX: 28,
	/** Height of the character progression track. */
	CHARACTER_TRACK_HEIGHT_PX: 40,
} as const;

export const PANEL = {
	/** Minimum height of the beat editor panel (px). */
	MIN_EDITOR_HEIGHT_PX: 150,
	/** Minimum height of the timeline panel (px). */
	MIN_TIMELINE_HEIGHT_PX: 200,
	/** Minimum height of the script panel (px). */
	MIN_SCRIPT_HEIGHT_PX: 120,
	/** Sidebar width (px). */
	SIDEBAR_WIDTH_PX: 280,
	/** Sidebar expanded width when viewing entity detail (px). */
	SIDEBAR_EXPANDED_WIDTH_PX: 420,
	/** Minimum width of the relationship panel (px). */
	MIN_RELATIONSHIP_WIDTH_PX: 240,
	/** Default width of the relationship panel (px). */
	DEFAULT_RELATIONSHIP_WIDTH_PX: 320,
	/** Maximum width of the relationship panel (px). */
	MAX_RELATIONSHIP_WIDTH_PX: 600,
} as const;

// --- AI Configuration ---

export type BackendType = 'ollama' | 'open_router';

export interface AiConfig {
	backend_type: BackendType;
	model: string;
	temperature: number;
	max_tokens: number;
	base_url: string;
	api_key: string | null;
}

export interface AiStatus {
	backend: BackendType;
	model?: string;
	connected: boolean;
	message?: string;
	error?: string;
}

// --- Consistency ---

export interface ConsistencySuggestion {
	source_clip_id: ClipId;
	target_clip_id: ClipId;
	original_text: string;
	suggested_text: string;
	reason: string;
}

// --- Arc Progression ---

export type Severity = 'Warning' | 'Error';

export interface ProgressionIssue {
	severity: Severity;
	message: string;
}

export interface ArcProgression {
	arc_id: string;
	arc_name: string;
	beat_count: number;
	has_setup: boolean;
	has_resolution: boolean;
	coverage_percent: number;
	longest_gap_ms: number;
	issues: ProgressionIssue[];
}

// --- WebSocket Messages ---

export type ServerMessage =
	| { type: 'timeline_changed' }
	| { type: 'scenes_changed' }
	| { type: 'story_changed' }
	| { type: 'beat_updated'; clip_id: string }
	| { type: 'generation_progress'; clip_id: string; token: string; tokens_generated: number }
	| { type: 'generation_complete'; clip_id: string }
	| { type: 'generation_error'; clip_id: string; error: string }
	| {
			type: 'consistency_suggestion';
			source_clip_id: string;
			target_clip_id: string;
			original_text: string;
			suggested_text: string;
			reason: string;
	  }
	| { type: 'consistency_complete'; source_clip_id: string; suggestion_count: number }
	| { type: 'undo_redo_changed'; can_undo: boolean; can_redo: boolean }
	| { type: 'project_mutated' }
	| { type: 'bible_changed' }
	| { type: 'entity_extraction_complete'; clip_id: string; new_entity_count: number; snapshot_count: number };

// --- Helpers ---

export function colorToHex(c: Color): string {
	const r = c.r.toString(16).padStart(2, '0');
	const g = c.g.toString(16).padStart(2, '0');
	const b = c.b.toString(16).padStart(2, '0');
	return `#${r}${g}${b}`;
}

export function formatTime(ms: number): string {
	const totalSeconds = Math.floor(ms / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, '0')}`;
}
