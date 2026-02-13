# Eidetic — AI-Driven Script Writing Platform (Alternative Plan)

## Vision

Eidetic is an AI-driven script writing tool focused on **30-minute TV episodes**. Rather than
chasing feature parity with Celtx or Final Draft, Eidetic reimagines script writing as a
**timeline-based, AI-collaborative** workflow — closer to a nonlinear editor (NLE) for text
than a traditional word processor.

The core Rust library (`eidetic-core`) provides all application logic as a reusable module with
zero UI dependencies. The UI is a Svelte 5 SPA served via a lightweight local HTTP server, with
a clear path to deploying as a full web application (potentially with WASM for the core).

---

## How This Differs from Plan A

| Dimension | Plan A (Existing) | Plan B (This Plan) |
|---|---|---|
| **Scope** | Feature parity with Celtx; film, TV, stage, novel | 30-min TV episodes only |
| **UI hosting** | CEF (Chromium Embedded Framework), offscreen rendering | Lightweight local HTTP server (e.g., `axum`) |
| **Deployment** | Desktop-only binary with embedded assets | Local server now; web deployment via WASM later |
| **Editor model** | Traditional document editor (ProseMirror-style) | Timeline with markers + arc overlays + clip-based text decomposition |
| **AI role** | Assistant panel, suggestions, auto-complete (Phase 4) | Core from day one; generates and reacts to edits live |
| **Story structure** | Beat sheets, outlines, character catalog (Phase 3) | Story arcs, characters, beats, scenes as **markers on a timeline with relationship arcs** |
| **Format support** | Fountain, FDX, PDF, HTML, plain text | Minimal — native project format only, PDF export later |
| **Script types** | Screenplay, TV (1hr/30min), stage, multi-col A/V, novel | 30-min TV episode only |
| **Phase count** | 8 phases, script engine before AI | 5 phases, AI integrated from Phase 1 |
| **Complexity** | CEF subprocess management, IPC console.log interception, platform-specific windowing | Standard HTTP API, WebSocket for streaming, no native windowing |
| **WASM potential** | None (CEF is native-only) | Explicit goal — `eidetic-core` compiles to WASM |

### Why These Changes

1. **CEF is heavyweight.** It requires downloading ~300MB of Chromium binaries, managing
   subprocess helpers, handling offscreen rendering, and dealing with platform-specific
   GTK/Cocoa/Win32 windowing. A local `axum` server serving static files is trivially
   cross-platform and eliminates an entire crate (`eidetic-webview`) plus the `cef-helper`
   binary.

2. **Narrow scope enables faster iteration.** Supporting every script type and format before
   touching AI means the unique value proposition (AI-assisted writing) arrives last. By
   targeting only 30-min TV episodes, we can ship a usable tool sooner and expand later.

3. **Timeline/NLE metaphor is the differentiator.** No existing script writing tool thinks
   about narrative as clips on a timeline with markers and relationship arcs drawn between
   them. This is a novel UX that makes AI integration natural — markers carry story context,
   clips carry text, arcs show how beats and characters connect, and the AI can operate on
   the gaps.

4. **Web deployment opens doors.** WASM compilation of the core means Eidetic could run
   entirely in-browser, or be served as a collaborative web app. CEF locks you into native
   desktop forever.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          BROWSER (any)                                  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                     Svelte 5 SPA                                  │  │
│  │                                                                   │  │
│  │  ┌───────────────────────────────────────────────────────────┐    │  │
│  │  │  Script View (top)                                        │    │  │
│  │  │  Generated + user-edited screenplay, inline editing       │    │  │
│  │  └───────────────────────────────────────────────────────────┘    │  │
│  │  ┌───────────────────────────────────────────────────────────┐    │  │
│  │  │  Annotated Timeline (bottom)                              │    │  │
│  │  │                                                           │    │  │
│  │  │   Relationship arcs ──╮    ╭──── drawn as curves          │    │  │
│  │  │    ╭───────╮  ╭───╮  │    │  ╭───────╮                   │    │  │
│  │  │  ──┤Beat A ├──┤B  ├──╯ ╭──╯──┤Beat C ├──── Markers       │    │  │
│  │  │  ──┴───────┴──┴───┴────┴─────┴───────┴──── on timeline   │    │  │
│  │  │  ══[  Scene 1 clip  ]══[  Scene 2 clip  ]══ Script clips  │    │  │
│  │  │  ──────────────────────────────────────────── Time ruler   │    │  │
│  │  └───────────────────────────────────────────────────────────┘    │  │
│  │                                                                   │  │
│  │  REST + WebSocket ────────────────────────────────────────────┐   │  │
│  └──────────────────────────────────────────────────────────────┬┘   │  │
│                                                                  │      │
└──────────────────────────────────────────────────────────────────┼──────┘
                                                                   │
┌──────────────────────────────────────────────────────────────────┼──────┐
│                        LOCAL SERVER (Rust)                        │      │
│                                                                  │      │
│  ┌───────────────┐     ┌─────────────────────────────────────┐   │      │
│  │  axum / tower  │◄───┤        eidetic-core (library)       │   │      │
│  │  HTTP + WS     │    │                                     │   │      │
│  │  server        │    │  • Story graph (arcs, beats, scenes)│   │      │
│  │                │    │  • Timeline engine (clips, tracks,   │   │      │
│  │  Serves:       │    │    markers, relationship arcs)       │   │      │
│  │  • Static UI   │    │  • Character model                  │   │      │
│  │  • REST API    │    │  • Script generation + formatting   │   │      │
│  │  • WS streams  │    │  • AI orchestration                 │   │      │
│  └───────────────┘     └──────────────┬──────────────────────┘   │      │
│                                       │                          │      │
│                          ┌────────────┴────────────┐             │      │
│                          │    AI Backend(s)         │             │      │
│                          │                          │             │      │
│                          │  • llama.cpp (local)     │             │      │
│                          │  • OpenAI/Anthropic API  │             │      │
│                          │  • (pluggable)           │             │      │
│                          └─────────────────────────┘             │      │
└──────────────────────────────────────────────────────────────────────────┘

Future: eidetic-core ──(compile to WASM)──> runs entirely in browser
```

### Separation of Concerns

- **`eidetic-core`** — Pure Rust library. No IO, no networking, no UI. Defines the data model
  (story graph, timeline, clips, characters), script generation logic, AI prompt construction,
  and all domain operations. Designed to compile to WASM.

- **`eidetic-server`** — Thin binary that wraps `eidetic-core` with an `axum` HTTP/WebSocket
  server. Serves the compiled Svelte SPA as static files. Handles file I/O, AI backend
  communication, and exposes the core API over REST + WebSocket.

- **`ui/`** — Svelte 5 SPA. Renders the two-panel layout (script view above, annotated timeline
  below). Communicates with the server via REST (commands) and WebSocket (streaming updates).

---

## Core Concepts

### 1. The Annotated Timeline

The timeline is the central workspace. It represents the **runtime** of the episode (~22
minutes for a 30-min TV episode, ~22 pages at 1 page/minute). Rather than having a separate
node graph panel, story structure elements live **directly on the timeline** as markers, with
**relationship arcs drawn as curves above the tracks** to show how story elements connect.

The timeline has three visual layers, stacked vertically:

```
    Relationship Arcs (curves drawn between markers)
    ╭─────────────────────╮        ╭──────╮
    │                     │        │      │
────┼─────────────────────┼────────┼──────┼──────────────────────
    ▼                     ▼        ▼      ▼
    Markers (story elements pinned to time positions)
  [Beat:       [Beat:     [Beat:  [Beat:        [Beat:
   Inciting     Midpoint   B-plot  Climax        Tag]
   Incident]    Reversal]  Payoff] Resolution]

────────────────────────────────────────────────────────────────
    Tracks (clips representing content with time ranges)
  ═══[ Scene 1 clip ]═══[ Scene 2 clip ]═══[ Scene 3 clip ]═══  Script track
  ───[ A-plot ──────────────────────── ]───[ A-plot ───── ]───  Arc track
  ───────────[ B-plot ─────── ]────────────────────────────────  Arc track
  ▏ Cold Open ▕▏  Act One     ▕▏  Act Two      ▕▏Act Three▕▏Tag▕  Structure
────────────────────────────────────────────────────────────────
  0:00       2:30         9:30         16:30     21:30  22:00   Time ruler
```

**Markers** are the story graph made spatial. Each marker is pinned to a time position on the
timeline and represents a story element:

- **Beat markers** — Narrative turning points (inciting incident, midpoint, climax, etc.)
- **Character markers** — Character entrances, exits, or key moments
- **Scene boundary markers** — Where one scene ends and another begins
- **Structural markers** — Act breaks, commercial breaks, cold open/tag boundaries

**Relationship arcs** are curves drawn in the space above the tracks, connecting markers to
show how story elements relate to each other:

- Beat → Beat arcs (causal chain: "this must happen before that")
- Arc → Beat arcs (this beat advances this story arc), color-coded per arc
- Character → Beat arcs (this character drives this beat)
- Any marker → any marker (user-defined thematic or structural links)

The arcs are drawn as bezier curves between their source and target markers. Color-coding
distinguishes arc types (e.g., A-plot arcs in blue, B-plot in green, character arcs in amber).
The user can toggle arc visibility by type to reduce visual clutter.

**Tracks** sit below the markers and carry content with time ranges (like an NLE):

- **Script tracks** — Text "clips" containing generated or user-written script content
- **Arc tracks** — One per story arc, showing which time ranges that arc occupies
- **Structure track** — Read-only visual of the act structure (cold open, acts, tag)

```rust
/// The central data structure: timeline with markers, arcs, and tracks.
pub struct Timeline {
    pub total_duration: Duration,      // ~22 minutes for 30-min TV
    pub markers: Vec<Marker>,          // Story elements pinned to time
    pub relationships: Vec<Relationship>, // Arcs drawn between markers
    pub tracks: Vec<Track>,            // Content tracks (clips)
    pub structure_markers: Vec<StructureMarker>, // Act breaks, commercials
}

/// A story element pinned to a position on the timeline.
pub struct Marker {
    pub id: MarkerId,
    pub time_position: Duration,       // Where on the timeline
    pub marker_type: MarkerType,
    pub name: String,
    pub description: String,           // User-provided context for AI
    pub color: Option<Color>,          // Visual color override
}

pub enum MarkerType {
    Beat {
        beat_type: BeatType,           // Inciting, Midpoint, Climax, etc.
        arc_id: Option<ArcId>,         // Which story arc this advances
    },
    CharacterMoment {
        character_id: CharacterId,
        moment_type: MomentType,       // Entrance, Exit, Revelation, etc.
    },
    SceneBoundary {
        heading: String,               // INT./EXT. LOCATION - TIME
        character_ids: Vec<CharacterId>,
    },
}

/// A visual arc drawn between two markers to show their relationship.
pub struct Relationship {
    pub id: RelationshipId,
    pub from_marker: MarkerId,
    pub to_marker: MarkerId,
    pub relationship_type: RelationshipType,
    pub color: Color,                  // Derived from arc color or type
}

pub enum RelationshipType {
    Causal,          // "this causes that" (beat → beat)
    AdvancesArc {    // "this beat advances this arc"
        arc_id: ArcId,
    },
    CharacterDrives, // "this character drives this beat"
    Thematic,        // User-defined thematic link
}

/// A named story arc (A-plot, B-plot, etc.) — not on the timeline itself,
/// but referenced by markers and relationships.
pub struct StoryArc {
    pub id: ArcId,
    pub name: String,
    pub description: String,
    pub arc_type: ArcType,             // APlot, BPlot, CRunner
    pub color: Color,                  // Color for all related arcs/markers
}

/// A character — also not directly on the timeline, but referenced by markers.
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub description: String,
    pub voice_notes: String,           // How this character speaks (for AI)
    pub color: Color,
}

/// A content track containing clips.
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub track_type: TrackType,
    pub clips: Vec<Clip>,
}

pub enum TrackType {
    Script,  // Text clips (generated or user-written)
    Arc {    // Visual region showing arc presence
        arc_id: ArcId,
    },
}

/// A clip on a track — a block of content with a time range.
pub struct Clip {
    pub id: ClipId,
    pub time_range: TimeRange,
    pub content: ClipContent,
    pub locked: bool,                  // If true, AI won't regenerate
}

pub enum ClipContent {
    ScriptText {
        text: String,
        generation_status: GenStatus,  // Empty, Generating, Generated, UserWritten
    },
    ArcPresence,                       // Just a colored region (no text)
}

pub struct TimeRange {
    pub start: Duration,
    pub end: Duration,
}
```

**User interactions on the timeline:**

- **Add marker:** Click on the timeline to place a new beat, character moment, or scene boundary
- **Connect markers:** Drag from one marker to another to create a relationship arc
- **Move markers:** Drag markers left/right to adjust their time position
- **Edit markers:** Click a marker to open a popover for editing name, description, type
- **Drag clip edges:** Adjust duration (AI regenerates to fit the new timing)
- **Split clips:** Razor tool to decompose a clip for finer editing
- **Rearrange clips:** Drag clips to reorder scenes
- **Toggle arc visibility:** Filter which relationship arcs are shown by type or story arc
- **Snap to structure:** Markers snap to act break positions when dragged near them

### 2. The Script View

Above the timeline, the user sees the **generated script** — a formatted, editable view of
the episode's screenplay. This is the "output" that the timeline produces.

Key behaviors:
- **Live generation:** As the user places markers and defines beats, scenes, and arcs on
  the timeline, the AI generates script text to fill the corresponding time slots
- **Inline editing:** The user can type directly into the script. Edits are tracked and
  the AI treats user-written text as canonical (it won't overwrite it)
- **User-written beats:** The user can write certain scenes/beats themselves. The AI fills
  in the transitions and connecting tissue between user-written sections
- **Consistency reactions:** When the user edits the script, the AI can update downstream
  content to remain consistent (e.g., if a character's name changes in an edit, the AI
  updates subsequent references)
- **Contextual awareness:** The AI uses the full timeline structure (markers, relationships,
  arcs, characters), surrounding script text, and any RAG-retrieved reference material to
  generate coherent output

### 3. AI Integration (Core, Not Afterthought)

Unlike Plan A where AI is Phase 4, here AI is integral from Phase 1. The AI:

- **Generates script from structure:** Given beats, scenes, characters, and timing
  constraints, produces formatted screenplay text
- **Reacts to user edits:** When the user modifies script text, the AI can propagate
  consistency changes to surrounding generated sections
- **Fills gaps:** User writes beat A and beat C; AI generates beat B to bridge them
- **Respects constraints:** Generated text must fit within the time allocation of its
  clip on the timeline (approximately 1 page per minute of screen time)
- **Uses RAG:** Reference materials, style guides, character bibles, and previous
  episodes can be indexed and retrieved for context

### 4. Characters and Story Arcs (Sidebar Entities)

Characters and story arcs are not placed directly on the timeline — they are **persistent
entities** managed in a sidebar panel. They are _referenced by_ markers and relationships on
the timeline.

- **Characters** have a name, description, voice notes (how they speak), and a color. When a
  character marker or scene boundary references a character, that character's color appears on
  the marker. The AI uses voice notes to maintain consistent dialogue.

- **Story arcs** have a name, description, type (A-plot, B-plot, C-runner), and a color.
  Beats that advance an arc inherit its color. Arc tracks on the timeline show colored regions
  where each arc is active. Relationship arcs connecting beats to their parent story arc use
  the arc's color.

The sidebar provides list views for managing characters and arcs, with detail panels for
editing their properties. These entities are the "vocabulary" that the timeline markers
reference.

```rust
pub trait AiBackend: Send + Sync {
    /// Generate script text for a given context
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateStream>;

    /// React to a user edit and suggest consistency updates
    async fn react_to_edit(&self, edit: EditContext) -> Result<Vec<ConsistencyUpdate>>;

    /// Summarize a section for use as context elsewhere
    async fn summarize(&self, text: &str) -> Result<String>;
}

pub struct GenerateRequest {
    pub scene: Scene,
    pub beat: Beat,
    pub characters: Vec<Character>,
    pub surrounding_context: SurroundingContext,
    pub time_budget: Duration,        // target screen time
    pub user_written_anchors: Vec<Anchor>, // text the user wrote that must be preserved
    pub style_notes: Option<String>,
    pub rag_context: Vec<RagChunk>,
}
```

---

## Cargo Workspace Structure

```
Eidetic/
├── Cargo.toml                  # Workspace root
├── PLAN.md                     # This document
├── LICENSE                     # MIT (existing)
├── .gitignore                  # Rust + Node ignores
│
├── crates/
│   ├── core/                   # eidetic-core: reusable library (WASM-compatible)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Public API
│   │       ├── timeline/       # Timeline engine (central data model)
│   │       │   ├── mod.rs
│   │       │   ├── marker.rs   # Markers (beats, character moments, scene boundaries)
│   │       │   ├── relationship.rs # Relationship arcs between markers
│   │       │   ├── track.rs    # Tracks and track types
│   │       │   ├── clip.rs     # Clips (text clips, arc regions)
│   │       │   └── timing.rs   # Duration math, page-to-time conversion
│   │       ├── story/          # Story entities (referenced by timeline)
│   │       │   ├── mod.rs
│   │       │   ├── arc.rs      # Story arcs (A-plot, B-plot, etc.)
│   │       │   └── character.rs # Characters, voice notes, relationships
│   │       ├── script/         # Script formatting
│   │       │   ├── mod.rs
│   │       │   ├── element.rs  # Script elements (scene heading, dialogue, etc.)
│   │       │   ├── format.rs   # 30-min TV episode formatting rules
│   │       │   └── merge.rs    # Merging user edits with AI generation
│   │       ├── ai/             # AI orchestration (backend-agnostic)
│   │       │   ├── mod.rs
│   │       │   ├── backend.rs  # AiBackend trait definition
│   │       │   ├── prompt.rs   # Prompt construction from graph + timeline
│   │       │   ├── context.rs  # Context window management, RAG integration
│   │       │   └── consistency.rs # Edit reaction and consistency engine
│   │       ├── project/        # Project model (no I/O — just data structures)
│   │       │   ├── mod.rs
│   │       │   └── project.rs  # Project struct, metadata
│   │       └── error.rs        # Error types
│   │
│   └── server/                 # eidetic-server: HTTP/WS server binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         # Entry point, axum server setup
│           ├── routes/         # REST API routes
│           │   ├── mod.rs
│           │   ├── project.rs  # Project CRUD
│           │   ├── timeline.rs # Timeline operations (markers, relationships, clips)
│           │   ├── story.rs    # Story entity CRUD (arcs, characters)
│           │   ├── script.rs   # Script content operations
│           │   └── ai.rs       # AI generation endpoints
│           ├── ws.rs           # WebSocket handler (streaming updates)
│           ├── ai_backends/    # Concrete AI backend implementations
│           │   ├── mod.rs
│           │   ├── llama.rs    # llama.cpp local inference
│           │   └── api.rs      # OpenAI/Anthropic API client
│           ├── persistence.rs  # File I/O, project save/load
│           └── static_files.rs # Serve compiled Svelte SPA
│
├── ui/                         # Svelte 5 frontend
│   ├── package.json
│   ├── svelte.config.js
│   ├── vite.config.ts
│   ├── tailwind.config.cjs
│   ├── tsconfig.json
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css             # Global styles, Pantograph dark theme
│   │   ├── routes/
│   │   │   ├── +layout.svelte
│   │   │   └── +page.svelte    # Main app layout (three-panel)
│   │   ├── lib/
│   │   │   ├── api.ts          # REST API client
│   │   │   ├── ws.ts           # WebSocket client for streaming
│   │   │   ├── types.ts        # TypeScript types mirroring Rust models
│   │   │   └── stores/
│   │   │       ├── project.svelte.ts
│   │   │       ├── timeline.svelte.ts   # Markers, relationships, tracks, clips
│   │   │       ├── story.svelte.ts      # Arcs, characters
│   │   │       └── script.svelte.ts
│   │   └── components/
│   │       ├── layout/
│   │       │   ├── AppShell.svelte       # Two-panel layout (script + timeline)
│   │       │   ├── PanelResizer.svelte   # Draggable panel boundary
│   │       │   └── Sidebar.svelte        # Collapsible sidebar (arcs, characters)
│   │       ├── script/
│   │       │   ├── ScriptView.svelte     # Formatted script display + editing
│   │       │   ├── ScriptElement.svelte  # Individual script element renderer
│   │       │   └── InlineEditor.svelte   # Contenteditable inline editing
│   │       ├── timeline/
│   │       │   ├── Timeline.svelte       # Main timeline component (all layers)
│   │       │   ├── MarkerLayer.svelte    # Renders markers above tracks
│   │       │   ├── Marker.svelte         # Individual marker (beat, scene, etc.)
│   │       │   ├── MarkerPopover.svelte  # Edit popover for a marker
│   │       │   ├── ArcLayer.svelte       # Renders relationship curves above markers
│   │       │   ├── RelationshipArc.svelte # Single bezier curve between markers
│   │       │   ├── Track.svelte          # Single track lane
│   │       │   ├── Clip.svelte           # Draggable/resizable clip
│   │       │   ├── TimeRuler.svelte      # Time ruler at bottom
│   │       │   └── StructureBar.svelte   # Act structure visual (cold open, acts, tag)
│   │       ├── sidebar/
│   │       │   ├── ArcList.svelte        # Story arc list + editor
│   │       │   ├── ArcDetail.svelte      # Arc detail panel
│   │       │   ├── CharacterList.svelte  # Character list + editor
│   │       │   └── CharacterDetail.svelte # Character detail panel
│   │       └── shared/
│   │           ├── Button.svelte
│   │           ├── Modal.svelte
│   │           ├── ContextMenu.svelte
│   │           └── Tooltip.svelte
│   └── static/
│       └── fonts/
│
└── dist/                       # Build output (gitignored)
    └── ui/
```

---

## API Design (REST + WebSocket)

### REST Endpoints

```
# Project
POST   /api/project                    # Create new project
GET    /api/project                    # Get current project
PUT    /api/project                    # Update project metadata

# Story entities (sidebar)
GET    /api/arcs                       # List story arcs
POST   /api/arcs                       # Create arc
PUT    /api/arcs/:id                   # Update arc
DELETE /api/arcs/:id                   # Delete arc
GET    /api/characters                 # List characters
POST   /api/characters                 # Create character
PUT    /api/characters/:id             # Update character
DELETE /api/characters/:id             # Delete character

# Timeline (markers, relationships, tracks, clips)
GET    /api/timeline                   # Full timeline state
POST   /api/timeline/markers           # Create marker (beat, scene boundary, etc.)
PUT    /api/timeline/markers/:id       # Update marker (position, type, description)
DELETE /api/timeline/markers/:id       # Delete marker
POST   /api/timeline/relationships     # Create relationship arc between markers
DELETE /api/timeline/relationships/:id # Delete relationship arc
PUT    /api/timeline/clips/:id         # Update clip (resize, move)
POST   /api/timeline/clips/:id/split   # Split a clip
POST   /api/timeline/tracks            # Add track
DELETE /api/timeline/tracks/:id        # Remove track

# Script
GET    /api/script                     # Full generated script
PUT    /api/script/sections/:id        # User edit to a script section
POST   /api/script/lock/:id            # Lock a section (mark as user-written)
POST   /api/script/unlock/:id          # Unlock a section (allow AI regeneration)

# AI
POST   /api/ai/generate               # Generate script for a clip/scene
POST   /api/ai/react                   # React to user edit, get consistency suggestions
POST   /api/ai/summarize              # Summarize a section
GET    /api/ai/status                  # AI backend status
PUT    /api/ai/config                  # Configure AI backend
```

### WebSocket Events

```typescript
// Server -> Client
{ type: "script_updated", data: { section_id, text, status } }
{ type: "generation_progress", data: { clip_id, tokens_generated, token } }
{ type: "generation_complete", data: { clip_id, text } }
{ type: "consistency_suggestion", data: { section_id, original, suggested, reason } }
{ type: "timeline_changed", data: { timeline } }  // markers, relationships, clips

// Client -> Server
{ type: "subscribe", data: { channels: ["script", "generation", "timeline"] } }
{ type: "user_edit", data: { section_id, text, cursor_pos } }
{ type: "accept_suggestion", data: { suggestion_id } }
{ type: "reject_suggestion", data: { suggestion_id } }
```

---

## TV Episode Structure (30-Minute Format)

The timeline comes pre-configured with standard 30-minute TV episode structure:

```
 0:00  ─── COLD OPEN / TEASER ──────────────────  ~2 min
 2:00  ─── MAIN TITLES ─────────────────────────  ~0:30
 2:30  ─── ACT ONE ─────────────────────────────  ~7 min
 9:30  ─── COMMERCIAL BREAK ────────────────────
 9:30  ─── ACT TWO ─────────────────────────────  ~7 min
16:30  ─── COMMERCIAL BREAK ────────────────────
16:30  ─── ACT THREE ──────────────────────────  ~5 min
21:30  ─── TAG / STINGER ──────────────────────  ~0:30
22:00  ─── END
```

This maps to approximately 22 pages of script (1 page ≈ 1 minute). The markers for act breaks
and commercial breaks are pre-placed but adjustable.

---

## AI Prompt Strategy

### Generation Pipeline

```
1. User places beat and scene boundary markers on the timeline
2. User connects markers with relationship arcs and assigns them to story arcs
3. For each clip that needs generation:
   a. Gather context:
      - The beat description and scene description
      - Character profiles for all characters in the scene
      - Preceding script text (user-written or previously generated)
      - Following script text (if any user-written anchors exist ahead)
      - Story arc context (what this beat is advancing)
      - RAG-retrieved reference material
   b. Construct prompt with:
      - System prompt: "You are a TV screenwriter. Write in standard screenplay format."
      - Time/page budget: "This scene should be approximately N pages."
      - Character voices: Descriptions of how each character speaks
      - Constraints: "The scene must end with [beat description]"
      - Anchors: "The following user-written text must appear verbatim: ..."
   c. Stream generation token-by-token to the UI
   d. Format the output as proper script elements
```

### Edit Reaction Pipeline

```
1. User edits script text in the Script View
2. The edit is diffed against the previous version
3. The diff + surrounding context is sent to the AI
4. The AI identifies what changed semantically:
   - Character name change? → Update all subsequent references
   - Plot point changed? → Flag downstream scenes for regeneration
   - Dialogue tone shift? → Adjust character voice in adjacent scenes
5. Suggestions are presented to the user (not auto-applied)
6. User accepts/rejects each suggestion
```

---

## Phase Roadmap

### Phase 1: Skeleton + Server + Annotated Timeline

- [ ] Cargo workspace with `core` and `server` crates
- [ ] `eidetic-core`: Timeline data model (markers, relationships, tracks, clips, time ranges)
- [ ] `eidetic-core`: Story entity models (arcs, characters)
- [ ] `eidetic-core`: Basic script element types (scene heading, action, character, dialogue,
      parenthetical, transition) — 30-min TV format only
- [ ] `eidetic-server`: axum HTTP server serving static files
- [ ] `eidetic-server`: REST API for timeline operations, story entity CRUD
- [ ] `eidetic-server`: WebSocket endpoint for streaming updates
- [ ] Svelte 5 SPA skeleton with Pantograph dark theme
- [ ] Two-panel layout: script view (top), annotated timeline (bottom)
- [ ] Collapsible sidebar for story arcs and characters
- [ ] Timeline: marker layer (draggable markers), relationship arc layer (bezier curves),
      track layer (clips), time ruler, act break structure bar
- [ ] Marker interactions: add, move, edit popover, connect (drag to create relationship)
- [ ] Wire up REST API to Svelte stores

### Phase 2: Script Generation + AI Core

- [ ] `eidetic-core`: AI backend trait and prompt construction
- [ ] `eidetic-server`: llama.cpp backend implementation (local inference)
- [ ] `eidetic-server`: API backend implementation (OpenAI/Anthropic)
- [ ] Generate script text from beat + scene + character context
- [ ] Stream tokens to the UI via WebSocket
- [ ] Display generated script in the Script View with proper formatting
- [ ] Timeline clip status indicators (empty, generating, generated, user-written)
- [ ] Basic context windowing (fit relevant context into model context length)

### Phase 3: Interactive Editing + Consistency

- [ ] Inline script editing (contenteditable in Script View)
- [ ] User edit tracking (diff user changes vs generated text)
- [ ] Lock/unlock sections (mark as user-written vs AI-regeneratable)
- [ ] Edit reaction pipeline: AI detects semantic changes and suggests updates
- [ ] Consistency suggestion UI (accept/reject per suggestion)
- [ ] Timeline clip splitting (decompose a clip for finer editing)
- [ ] Timeline clip resizing (adjust time allocation, trigger regeneration)
- [ ] "Fill gaps" mode: user writes beats A and C, AI generates B

### Phase 4: RAG + Context Enrichment

- [ ] Reference material indexing (character bibles, style guides, previous episodes)
- [ ] RAG retrieval for generation context
- [ ] Character voice consistency (learn from user-written dialogue samples)
- [ ] Episode-level consistency checking (continuity across scenes)
- [ ] Story arc progression tracking (does the A-plot advance sufficiently?)

### Phase 5: Polish + Export

- [ ] PDF export (standard TV screenplay format)
- [ ] Project save/load (JSON-based project file)
- [ ] Undo/redo for graph, timeline, and script edits
- [ ] Keyboard shortcuts
- [ ] WASM compilation of `eidetic-core` (proof of concept)
- [ ] Performance optimization for large scripts

---

## Key Design Decisions

### 1. Local HTTP Server, Not CEF

The Svelte UI runs in any browser. During development, Vite's dev server handles hot-reload.
In production, the Rust binary serves the compiled SPA on `localhost:PORT` and opens the
default browser. This eliminates hundreds of megabytes of CEF dependencies, platform-specific
windowing code, and subprocess management.

**Trade-off:** We lose the ability to create a "native-feeling" desktop window. The app runs
in a browser tab. This is acceptable for Phase 1-5 and can be revisited later (Tauri is a
lighter alternative to CEF if native windowing is ever needed).

### 2. WASM-Compatible Core

`eidetic-core` has no I/O, no filesystem access, no networking. All side effects are handled
by the server binary. This means the core can compile to WASM and run in the browser for a
fully client-side deployment.

**Constraint:** This means `eidetic-core` cannot directly call llama.cpp or make HTTP requests.
AI backends are injected as trait implementations by the server.

### 3. 30-Minute TV Episodes Only

Rather than supporting every script format (film, TV 1-hour, stage, multi-column A/V, novel),
we support exactly one: 30-minute TV episodes. This constrains:
- The timeline is fixed at ~22 minutes
- Act structure follows the standard 3-act + cold open + tag format
- Script formatting follows TV script conventions
- AI prompts are tuned for episodic TV writing

**Expansion path:** Additional formats can be added as "timeline templates" later, but the
core data model (graph + timeline + clips) is format-agnostic.

### 4. AI-First, Not AI-Later

The AI is not a bolt-on feature. The entire workflow assumes AI-generated content is the
default, with user edits as refinements. This inverts the traditional screenwriting software
model (user writes everything, AI assists occasionally).

**The user's workflow:**
1. Define story arcs, characters, and beats
2. Place beats on the timeline
3. AI generates a complete first draft
4. User reads, edits, rewrites sections they want to control
5. AI adjusts surrounding content for consistency
6. Iterate until satisfied

### 5. Annotated Timeline = Script

The script is a **derived artifact** of the annotated timeline, not the primary data
structure. This is the fundamental difference from Plan A (and from every existing
screenwriting tool). Markers hold "what happens and why." Tracks hold "when and for how long."
Relationship arcs show how story elements connect. The script is the textual rendering of all
of these, unified in a single timeline workspace.

---

## Open Questions

1. **Timeline rendering approach:** Build the annotated timeline (markers, relationship arcs,
   tracks, clips) with HTML/CSS/SVG, or use HTML5 Canvas for the whole thing? SVG is easier
   to make interactive (click handlers, drag, popovers) but Canvas performs better at scale.
   A hybrid approach (Canvas for tracks/clips, SVG overlay for markers/arcs) may be ideal.

2. **Relationship arc drawing:** Bezier curves between markers need to avoid visual clutter
   when many arcs exist. Options: bundled edges, arc grouping by story arc, fade on hover,
   or a filter panel to show/hide by type. Need to prototype to find the right balance.

3. **AI model requirements:** What model size is needed for acceptable script generation?
   7B models are fast but may produce weak dialogue. 13B+ is better but requires more
   VRAM. The pluggable backend means users can choose local vs. API.

4. **Conflict resolution:** When the AI suggests consistency updates that conflict with
   other user edits, how should conflicts be presented? Inline diff markers?
   Side-by-side comparison?

5. **Collaboration (future):** The REST+WS architecture naturally supports multi-user
   access. Should we design the API with multi-tenancy in mind from the start, or keep
   it single-user and add collaboration later?
