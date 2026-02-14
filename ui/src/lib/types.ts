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
	| { CharacterDrives: { character_id: CharacterId } }
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

export interface Character {
	id: CharacterId;
	name: string;
	description: string;
	voice_notes: string;
	color: Color;
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

// --- Project ---

export interface Project {
	name: string;
	timeline: Timeline;
	arcs: StoryArc[];
	characters: Character[];
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
} as const;

export const PANEL = {
	/** Minimum height of the beat editor panel (px). */
	MIN_EDITOR_HEIGHT_PX: 150,
	/** Minimum height of the timeline panel (px). */
	MIN_TIMELINE_HEIGHT_PX: 200,
	/** Sidebar width (px). */
	SIDEBAR_WIDTH_PX: 280,
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
