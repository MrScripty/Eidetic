/** Mirrors Rust types from eidetic-core for type-safe API communication. */

// --- IDs (all UUIDs as strings) ---

export type TrackId = string;
export type NodeId = string;
export type ArcId = string;
export type CharacterId = string;
export type RelationshipId = string;

// --- Story hierarchy levels ---

export type StoryLevel = 'Premise' | 'Act' | 'Sequence' | 'Scene' | 'Beat';

// --- Timeline ---

export interface Timeline {
	total_duration_ms: number;
	tracks: Track[];
	nodes: StoryNode[];
	node_arcs: NodeArc[];
	relationships: Relationship[];
	structure: EpisodeStructure;
}

export interface Track {
	id: TrackId;
	level: StoryLevel;
	label: string;
	sort_order: number;
	collapsed: boolean;
}

export interface StoryNode {
	id: NodeId;
	parent_id: NodeId | null;
	level: StoryLevel;
	sort_order: number;
	time_range: TimeRange;
	name: string;
	content: NodeContent;
	beat_type: BeatType | null;
	locked: boolean;
}

export interface NodeArc {
	node_id: NodeId;
	arc_id: ArcId;
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

export interface NodeContent {
	notes: string;
	/** Script/outline text. Replaces the old generated_text + user_refined_text split. */
	content: string;
	status: ContentStatus;
	scene_recap?: string | null;
}

export type ContentStatus =
	| 'Empty'
	| 'NotesOnly'
	| 'Generating'
	| 'HasContent';

// --- Relationships ---

export interface Relationship {
	id: RelationshipId;
	from_node: NodeId;
	to_node: NodeId;
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
	parent_arc_id: ArcId | null;
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
	node_refs: NodeId[];
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
	source_node_id: NodeId | null;
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
	location?: string;
}

// --- Child Planning (replaces Beat Planning) ---

export interface ChildProposal {
	name: string;
	beat_type: BeatType | null;
	outline: string;
	weight: number;
	characters?: string[];
	location?: string | null;
	props?: string[];
}

export interface ChildPlan {
	parent_node_id: NodeId;
	target_child_level: StoryLevel;
	children: ChildProposal[];
}

// --- Timeline gaps ---

export interface TimelineGap {
	level: StoryLevel;
	time_range: TimeRange;
	preceding_node_id: NodeId | null;
	following_node_id: NodeId | null;
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
	premise: string;
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
	/** Width of track label column. */
	LABEL_WIDTH_PX: 80,
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
	source_node_id: NodeId;
	target_node_id: NodeId;
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
	| { type: 'hierarchy_changed' }
	| { type: 'story_changed' }
	| { type: 'node_updated'; node_id: string }
	| { type: 'generation_context'; node_id: string; system_prompt: string; user_prompt: string }
	| { type: 'generation_progress'; node_id: string; token: string; tokens_generated: number }
	| { type: 'generation_complete'; node_id: string }
	| { type: 'generation_error'; node_id: string; error: string }
	| {
			type: 'consistency_suggestion';
			source_node_id: string;
			target_node_id: string;
			original_text: string;
			suggested_text: string;
			reason: string;
	  }
	| { type: 'consistency_complete'; source_node_id: string; suggestion_count: number }
	| { type: 'undo_redo_changed'; can_undo: boolean; can_redo: boolean }
	| { type: 'project_mutated' }
	| { type: 'bible_changed' }
	| { type: 'entity_extraction_complete'; node_id: string; new_entity_count: number; snapshot_count: number }
	| { type: 'diffusion_progress'; node_id: string; step: number; total_steps: number }
	| { type: 'diffusion_complete'; node_id: string }
	| { type: 'diffusion_error'; node_id: string; error: string };

// --- Diffusion ---

export interface DiffuseRequest {
	node_id: string;
	anchor_ranges: { start: number; end: number }[];
	mask_budget: number;
}

export interface DiffusionStatus {
	model_loaded: boolean;
	model_path: string | null;
	device: string;
}

// --- Model Library ---

export interface ModelEntry {
	id: string;
	name: string;
	path: string;
	model_type: string;
	size_bytes: number | null;
	tags: string[];
}

export interface ModelListResponse {
	models: ModelEntry[];
	total_count: number;
}

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

/** Get the child level for a given story level. */
export function childLevel(level: StoryLevel): StoryLevel | null {
	switch (level) {
		case 'Premise': return 'Act';
		case 'Act': return 'Sequence';
		case 'Sequence': return 'Scene';
		case 'Scene': return 'Beat';
		case 'Beat': return null;
	}
}

/** Get the parent level for a given story level. */
export function parentLevel(level: StoryLevel): StoryLevel | null {
	switch (level) {
		case 'Premise': return null;
		case 'Act': return 'Premise';
		case 'Sequence': return 'Act';
		case 'Scene': return 'Sequence';
		case 'Beat': return 'Scene';
	}
}

/** Get the best display text for a node (content > notes). */
export function bestText(node: StoryNode): string {
	return node.content.content || node.content.notes;
}
