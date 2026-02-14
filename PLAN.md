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
| **Editor model** | Traditional document editor (ProseMirror-style) | Arc tracks with beat clips (NLE-style) + contextual beat editor above |
| **AI role** | Assistant panel, suggestions, auto-complete (Phase 4) | Core from day one; generates and reacts to edits live |
| **Story structure** | Beat sheets, outlines, character catalog (Phase 3) | Story arcs as tracks, beats as clips, scenes inferred from vertical overlap, edge-bundled relationship curves |
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
│  │  │  Beat Editor (top)                                        │    │  │
│  │  │  Beat notes → generated script → user-refined script      │    │  │
│  │  └───────────────────────────────────────────────────────────┘    │  │
│  │  ┌───────────────────────────────────────────────────────────┐    │  │
│  │  │  Timeline (bottom)                                        │    │  │
│  │  │                                                           │    │  │
│  │  │   Edge-bundled relationship curves                        │    │  │
│  │  │  ──╭────────────────────────╮──╭──────╮────────────────   │    │  │
│  │  │  ══[Setup]══[Complication]══[Escalation]══[Climax]═════   │    │  │
│  │  │  ═════[Setup]═══[Complication]══════════[Payoff]═══════   │    │  │
│  │  │  ══[Beat]═══════════[Beat]════════════════[Callback]═══   │    │  │
│  │  │  ──▏Cold Open▕▏ Act 1 ▕▏   Act 2   ▕▏Act 3▕▏Tag▕─────   │    │  │
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
│  │  server        │    │  • Story arcs, characters            │   │      │
│  │                │    │  • Timeline engine (arc tracks,      │   │      │
│  │  Serves:       │    │    beat clips, relationships)        │   │      │
│  │  • Static UI   │    │  • Scene inference from overlap      │   │      │
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
  (arc tracks, beat clips, relationships, characters, scene inference), script generation
  logic, AI prompt construction, and all domain operations. Designed to compile to WASM.

- **`eidetic-server`** — Thin binary that wraps `eidetic-core` with an `axum` HTTP/WebSocket
  server. Serves the compiled Svelte SPA as static files. Handles file I/O, AI backend
  communication, and exposes the core API over REST + WebSocket.

- **`ui/`** — Svelte 5 SPA. Renders the two-panel layout (beat editor above, timeline below)
  with a collapsible sidebar for arcs and characters. Communicates with the server via REST
  (commands) and WebSocket (streaming updates).

---

## Core Concepts

### 1. The Annotated Timeline

The timeline is the central workspace. It represents the **runtime** of the episode (~22
minutes for a 30-min TV episode, ~22 pages at 1 page/minute). The primary organizing unit is
**arc tracks** — each story arc (A-plot, B-plot, C-runner) gets its own track, and the beats
within each arc are represented as **clips** on that track. This mirrors how TV writers
"break" a story: arcs interleave across the episode, and the timeline makes that structure
visible and directly manipulable.

**Relationship arcs** (hierarchical edge-bundled curves) are drawn above the tracks to show
how beats across different arcs connect — causal links, character connections, thematic ties.

The timeline has three visual layers, stacked vertically:

```
    Relationship Arcs (edge-bundled curves between beat clips)
    ╭──────────────────────────╮       ╭──────╮
    │    bundles together      │       │      │
────┼──────────────────────────┼───────┼──────┼──────────────────
    │                          │       │      │
    Arc Tracks (one track per story arc, beats as clips)
  ══[Setup]════[Complication]══════[Escalation]═══[Climax]══[Resolution]══  A-Plot
  ═════[Setup]═════[Complication]══════════════[Payoff]═══════════════════  B-Plot
  ═══[Beat]═══════════════[Beat]═══════════════════════[Callback]════════  C-Runner

────────────────────────────────────────────────────────────────
    Structure bar (act breaks, commercial breaks)
  ▏ Cold Open ▕▏  Act One     ▕▏  Act Two      ▕▏Act Three▕▏Tag▕
────────────────────────────────────────────────────────────────
  0:00       2:30         9:30         16:30     21:30  22:00   Time ruler
```

**Arc tracks** are the primary content layer. Each track belongs to a story arc and contains
beat clips — the narrative turning points of that arc:

- **A-plot track** — The main story. Clips represent its beats: setup, complication,
  escalation, climax, resolution.
- **B-plot track** — The secondary story. Typically fewer beats, interleaving with the A-plot.
- **C-runner track** — A running gag or minor thread. Usually 3 beats: setup, callback, payoff.
- **Additional tracks** — Users can add more tracks for subplots, thematic threads, or any
  other narrative layer.

Beat clips on arc tracks behave like NLE clips:

- **Drag to move** — Reposition beats on the timeline
- **Drag edges to resize** — Adjust how much screen time a beat occupies
- **Split** — Razor tool to decompose a beat into finer sub-beats
- **Gaps are meaningful** — Time ranges where no clip exists on a track mean that arc is not
  on screen during that period

**Scenes emerge from vertical overlap.** Rather than explicitly defining scenes, scenes are
**inferred** from the timeline. A vertical slice through the timeline at any point reveals
which arcs are active — that combination defines a scene. Scene boundaries appear wherever the
active combination of arcs changes. For example, if the A-plot and B-plot clips overlap at a
time position, that's a scene that serves both arcs simultaneously.

**Relationship arcs** are edge-bundled curves drawn above the tracks, connecting beat clips
across different arc tracks to show how story elements relate:

- Beat → Beat (causal: "this must happen before that")
- Beat → Beat across arcs (convergence: "these two arcs intersect here")
- Character → Beat (this character drives this beat)
- Any clip → any clip (user-defined thematic or structural links)

The curves use **hierarchical edge bundling** — related connections share a common path
("bundle") and splay apart near their endpoints. Color-coding distinguishes connection types
(e.g., A-plot connections in blue, B-plot in green, character links in amber). Users can
toggle visibility by type or arc to reduce clutter.

**Structure bar** sits below the tracks as a read-only reference showing the standard act
structure (cold open, acts, commercial breaks, tag). Beat clips snap to act break positions
when dragged near them.

```rust
/// The central data structure: timeline with arc tracks and structure.
pub struct Timeline {
    pub total_duration: Duration,      // ~22 minutes for 30-min TV
    pub tracks: Vec<ArcTrack>,         // One track per story arc
    pub relationships: Vec<Relationship>, // Edge-bundled curves between clips
    pub structure: EpisodeStructure,   // Act breaks, commercial breaks
}

/// A track belonging to a story arc, containing beat clips.
pub struct ArcTrack {
    pub id: TrackId,
    pub arc_id: ArcId,                 // Which story arc this track represents
    pub clips: Vec<BeatClip>,          // Beat clips on this track
}

/// A beat clip on an arc track — a narrative turning point with a time range.
pub struct BeatClip {
    pub id: ClipId,
    pub time_range: TimeRange,
    pub beat_type: BeatType,           // Setup, Complication, Climax, etc.
    pub name: String,
    pub content: BeatContent,          // What the user has written for this beat
    pub locked: bool,                  // If true, AI won't regenerate
}

/// The content of a beat clip, progressing through stages.
pub struct BeatContent {
    pub beat_notes: String,            // User's markdown description of what happens
    pub generated_script: Option<String>, // AI-generated screenplay text
    pub user_refined_script: Option<String>, // User's edits to the generated script
    pub status: ContentStatus,
}

pub enum ContentStatus {
    Empty,           // No content yet
    NotesOnly,       // User has written beat notes, no script generated
    Generating,      // AI is currently generating script
    Generated,       // AI has generated script from beat notes
    UserRefined,     // User has edited the generated script
    UserWritten,     // User wrote the script directly (no AI generation)
}

/// A visual connection drawn between two beat clips.
pub struct Relationship {
    pub id: RelationshipId,
    pub from_clip: ClipId,
    pub to_clip: ClipId,
    pub relationship_type: RelationshipType,
    pub color: Color,                  // Derived from arc color or type
}

pub enum RelationshipType {
    Causal,          // "this causes that" (beat → beat)
    Convergence {    // "these arcs intersect at this point"
        arc_ids: Vec<ArcId>,
    },
    CharacterDrives { // "this character drives this beat"
        character_id: CharacterId,
    },
    Thematic,        // User-defined thematic link
}

/// A named story arc (A-plot, B-plot, etc.) with its own track.
pub struct StoryArc {
    pub id: ArcId,
    pub name: String,
    pub description: String,
    pub arc_type: ArcType,             // APlot, BPlot, CRunner
    pub color: Color,                  // Color for the track and its clips
}

/// A character — referenced by beat clips and relationships.
pub struct Character {
    pub id: CharacterId,
    pub name: String,
    pub description: String,
    pub voice_notes: String,           // How this character speaks (for AI)
    pub color: Color,
}

/// The episode's act structure (pre-placed, adjustable).
pub struct EpisodeStructure {
    pub template_name: String,         // e.g., "Standard 30-Min Comedy"
    pub segments: Vec<StructureSegment>,
}

pub struct StructureSegment {
    pub segment_type: SegmentType,     // ColdOpen, Act, CommercialBreak, Tag
    pub time_range: TimeRange,
    pub label: String,
}

pub struct TimeRange {
    pub start: Duration,
    pub end: Duration,
}
```

### 2. The Default Template

A new project starts with a pre-configured timeline reflecting standard 30-minute TV episode
structure. The user sees arc tracks already in place with standard beat clips positioned at
conventional timing:

```text
A-Plot:  ═[Setup]══════════[Complication]══════════[Escalation]════[Climax]══[Resolution]═
B-Plot:  ══════[Setup]══════════[Complication]═══════════════[Payoff]══════════════════════
C-Runner:═[Beat]════════════════════[Beat]═══════════════════════════[Callback]════════════
         |Cold Open|   Act 1    |         Act 2          |    Act 3     |Tag|
         0:00     2:30         9:30                     16:30         21:30 22:00
```

All clips are empty (no content written yet) but positioned at conventional timing. The user
can immediately start clicking clips and writing beat notes, or rearrange the structure first.

**Subgenre presets** offer different starting configurations:

- **Multi-cam sitcom** (Seinfeld-style) — More interleaving, shorter beats, rapid A/B cutting
- **Single-cam dramedy** (Scrubs-style) — Longer, more flowing scenes, fewer cuts between arcs
- **Animated comedy** (Bob's Burgers-style) — Flexible act structure, C-runner emphasis

The template is a starting point — every clip, track, and structural marker is fully
adjustable.

### 3. The Beat Editor

Above the timeline, the user sees the **beat editor** — a contextual panel that displays and
edits the content of the currently selected beat clip. This is where the writing happens.

The beat editor progresses through three stages for each beat:

**Stage 1: Beat Notes (Markdown)**
When a clip is first selected, the editor shows a markdown writing area. The user describes
what happens in this beat — the **turn** (what changes from beginning to end):

> *Jake finds the letter hidden in the couch cushions. He reads it and realizes his roommate
> has been lying about the landlord. He's furious but decides to set a trap instead of
> confronting him directly.*

Beat notes capture: who's present, what happens, what changes emotionally. This is the raw
material the AI uses for generation.

**Stage 2: Generated Script**
When the user triggers generation (or it runs automatically), the AI produces formatted
screenplay text from the beat notes, character profiles, and surrounding context. The
generated script appears in the editor, formatted as proper screenplay elements (scene
headings, action, dialogue, parentheticals).

**Stage 3: User-Refined Script**
The user can edit the generated script directly. Edits are tracked — the AI treats
user-written text as canonical and won't overwrite it. The user can also bypass generation
entirely and write the script for a beat from scratch.

Key behaviors:

- **Clip selection drives the editor:** Click a beat clip on the timeline to load its content
- **Live generation:** The AI generates script text from beat notes + character info + context
- **Lock/unlock:** Mark a beat as user-written (locked) so the AI won't regenerate it
- **Fill gaps:** Write beats A and C yourself; the AI generates beat B to bridge them
- **Consistency reactions:** When the user edits a beat's script, the AI can suggest updates
  to downstream beats (presented as a side-by-side diff for accept/reject)

### 4. Scene Inference

Scenes are not explicitly created by the user — they **emerge from the timeline**. A scene is
defined by the vertical overlap of beat clips across arc tracks at a given time position.

```
A-Plot:  ═══════[Complication]═══════════
B-Plot:  ═══[Setup]═════════════════════
                   ↑
                   This overlap = a scene serving both A and B plots
```

When the AI generates script for overlapping beats, it weaves both arcs into a single scene —
characters from both storylines interact, and the dialogue serves dual purposes. This is one
of the key craft skills in TV writing, and the timeline makes it visually explicit.

Scene boundaries appear wherever the active combination of arcs changes. The system derives:

- **Scene headings** (INT./EXT. LOCATION - TIME) from the beat notes or user input
- **Character presence** from which arcs are active and which characters are assigned to them
- **Scene duration** from the overlapping time range

### 5. AI Integration (Core, Not Afterthought)

Unlike Plan A where AI is Phase 4, here AI is integral from Phase 1. The AI:

- **Generates script from beat notes:** Given the user's beat description, characters, arc
  context, and timing constraints, produces formatted screenplay text
- **Weaves overlapping arcs:** When beats from different arcs overlap on the timeline, the
  AI generates scenes that serve multiple storylines simultaneously
- **Reacts to user edits:** When the user modifies script text, the AI can propagate
  consistency changes to surrounding beats (presented as side-by-side diffs)
- **Fills gaps:** User writes beats A and C; AI generates beat B to bridge them
- **Respects constraints:** Generated text must fit within the time allocation of its
  clip on the timeline (approximately 1 page per minute of screen time)
- **Uses RAG:** Reference materials, style guides, character bibles, and previous
  episodes can be indexed and retrieved for context

### 6. Characters and Story Arcs (Sidebar Entities)

Characters and story arcs are **persistent entities** managed in a collapsible sidebar panel.

- **Story arcs** are the primary structural element. Each arc has a name, description, type
  (A-plot, B-plot, C-runner), and a color. Creating an arc automatically creates a
  corresponding track on the timeline. The arc's color is used for its track, clips, and
  relationship curves.

- **Characters** have a name, description, voice notes (how they speak), and a color. Beat
  clips can reference characters who appear in that beat. The AI uses voice notes to maintain
  consistent dialogue across the episode.

The sidebar provides list views for managing characters and arcs, with detail panels for
editing their properties.

```rust
pub trait AiBackend: Send + Sync {
    /// Generate script text for a beat clip given its notes and context
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateStream>;

    /// React to a user edit and suggest consistency updates
    async fn react_to_edit(&self, edit: EditContext) -> Result<Vec<ConsistencyUpdate>>;

    /// Summarize a section for use as context elsewhere
    async fn summarize(&self, text: &str) -> Result<String>;
}

pub struct GenerateRequest {
    pub beat_clip: BeatClip,           // The beat to generate for
    pub arc: StoryArc,                 // Which arc this beat belongs to
    pub overlapping_beats: Vec<(BeatClip, StoryArc)>, // Beats on other arcs at same time
    pub characters: Vec<Character>,    // Characters present in this beat
    pub surrounding_context: SurroundingContext, // Adjacent beat scripts
    pub time_budget: Duration,         // Target screen time for this beat
    pub user_written_anchors: Vec<Anchor>, // Text the user wrote that must be preserved
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
│   │       │   ├── track.rs    # Arc tracks
│   │       │   ├── clip.rs     # Beat clips (content, status, time ranges)
│   │       │   ├── relationship.rs # Edge-bundled relationships between clips
│   │       │   ├── scene.rs    # Scene inference from vertical clip overlap
│   │       │   ├── structure.rs # Episode structure (act breaks, templates)
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
│   │   │   └── +page.svelte    # Main app layout
│   │   ├── lib/
│   │   │   ├── api.ts          # REST API client
│   │   │   ├── ws.ts           # WebSocket client for streaming
│   │   │   ├── types.ts        # TypeScript types mirroring Rust models
│   │   │   └── stores/
│   │   │       ├── project.svelte.ts
│   │   │       ├── timeline.svelte.ts   # Arc tracks, beat clips, relationships
│   │   │       ├── story.svelte.ts      # Arcs, characters
│   │   │       └── editor.svelte.ts     # Beat editor state (selected clip, content)
│   │   └── components/
│   │       ├── layout/
│   │       │   ├── AppShell.svelte       # Two-panel layout (editor + timeline)
│   │       │   ├── PanelResizer.svelte   # Draggable panel boundary
│   │       │   └── Sidebar.svelte        # Collapsible sidebar (arcs, characters)
│   │       ├── editor/
│   │       │   ├── BeatEditor.svelte     # Contextual editor for selected beat clip
│   │       │   ├── BeatNotes.svelte      # Markdown editor for beat description
│   │       │   ├── ScriptView.svelte     # Generated/refined screenplay display
│   │       │   └── DiffView.svelte       # Side-by-side diff for consistency suggestions
│   │       ├── timeline/
│   │       │   ├── Timeline.svelte       # Main timeline component (all layers)
│   │       │   ├── ArcTrack.svelte       # Single arc track lane
│   │       │   ├── BeatClip.svelte       # Draggable/resizable beat clip
│   │       │   ├── RelationshipLayer.svelte # Edge-bundled curves above tracks
│   │       │   ├── RelationshipArc.svelte # Single bundled curve
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

# Timeline (arc tracks, beat clips, relationships)
GET    /api/timeline                   # Full timeline state (all tracks, clips, relationships)
POST   /api/timeline/tracks            # Add arc track (creates track for an arc)
DELETE /api/timeline/tracks/:id        # Remove arc track
POST   /api/timeline/clips             # Create beat clip on a track
PUT    /api/timeline/clips/:id         # Update beat clip (resize, move, rename)
DELETE /api/timeline/clips/:id         # Delete beat clip
POST   /api/timeline/clips/:id/split   # Split a beat clip into sub-beats
POST   /api/timeline/relationships     # Create relationship between clips
DELETE /api/timeline/relationships/:id # Delete relationship

# Beat content (editor)
GET    /api/beats/:id                  # Get beat content (notes + script)
PUT    /api/beats/:id/notes            # Update beat notes (markdown)
PUT    /api/beats/:id/script           # User edit to generated/refined script
POST   /api/beats/:id/lock             # Lock beat (mark as user-written)
POST   /api/beats/:id/unlock           # Unlock beat (allow AI regeneration)

# Scenes (inferred, read-only)
GET    /api/scenes                     # List inferred scenes from clip overlap

# AI
POST   /api/ai/generate               # Generate script for a beat clip
POST   /api/ai/react                   # React to user edit, get consistency suggestions
POST   /api/ai/summarize              # Summarize a section
GET    /api/ai/status                  # AI backend status
PUT    /api/ai/config                  # Configure AI backend
```

### WebSocket Events

```typescript
// Server -> Client
{ type: "beat_updated", data: { clip_id, notes, script, status } }
{ type: "generation_progress", data: { clip_id, tokens_generated, token } }
{ type: "generation_complete", data: { clip_id, script_text } }
{ type: "consistency_suggestion", data: { clip_id, original, suggested, reason } }
{ type: "timeline_changed", data: { tracks, relationships } }
{ type: "scenes_changed", data: { scenes } }  // re-inferred from clip overlap

// Client -> Server
{ type: "subscribe", data: { channels: ["beats", "generation", "timeline", "scenes"] } }
{ type: "beat_notes_edit", data: { clip_id, notes, cursor_pos } }
{ type: "beat_script_edit", data: { clip_id, script_text, cursor_pos } }
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

```text
1. User writes beat notes (markdown) for a beat clip on the timeline
2. User connects beat clips with relationship arcs across arc tracks
3. For each beat clip that needs script generation:
   a. Gather context:
      - The beat notes (user's markdown description of what happens)
      - The story arc this beat belongs to (name, description, type)
      - Overlapping beat clips from other arc tracks (for scene weaving)
      - Character profiles for all characters present in this beat
      - Preceding beat scripts (user-written or previously generated)
      - Following beat scripts (if any user-written anchors exist ahead)
      - Relationship arcs connecting this beat to others (causal, thematic)
      - RAG-retrieved reference material
   b. Construct prompt with:
      - System prompt: "You are a TV screenwriter. Write in standard screenplay format."
      - Time/page budget: "This beat should be approximately N pages."
      - Character voices: Descriptions of how each character speaks
      - Scene weaving: "This scene also advances [B-plot]: [B-plot beat notes]"
      - Constraints: "The beat must accomplish: [beat notes summary]"
      - Anchors: "The following user-written text must appear verbatim: ..."
   c. Stream generation token-by-token to the beat editor via WebSocket
   d. Format the output as proper script elements
```

### Edit Reaction Pipeline

```text
1. User edits a beat's script in the beat editor
2. The edit is diffed against the previous generated version
3. The diff + surrounding context is sent to the AI
4. The AI identifies what changed semantically:
   - Character name change? → Update all subsequent references in other beats
   - Plot point changed? → Flag downstream beats for regeneration
   - Dialogue tone shift? → Adjust character voice in adjacent beats
5. Suggestions are presented as side-by-side diffs (not auto-applied)
6. User accepts/rejects each suggestion
```

---

## Multi-Agent Implementation Plan

Work is divided across four parallel agents. Each agent owns a vertical slice of the
codebase and can work independently between **sync points** — integration milestones
where agents merge, test cross-boundary contracts, and align on the next sprint.

### Agent Definitions

| Agent | Scope | Crate / Directory | Key Expertise |
| --- | --- | --- | --- |
| **A1 — Core Engine** | Data model, timeline logic, scene inference, script elements | `crates/core/` | Rust, domain modeling, algorithms |
| **A2 — Server** | HTTP/WS server, persistence, AI backend wiring, static hosting | `crates/server/` | Rust, axum, networking, file I/O |
| **A3 — Frontend** | Svelte 5 SPA, timeline visualization, beat editor, sidebar | `ui/` | Svelte 5, TypeScript, D3, CSS |
| **A4 — AI/ML** | Prompt engineering, generation pipeline, consistency engine, RAG | `crates/core/src/ai/` + `crates/server/src/ai_backends/` | LLM integration, prompt design |

### Dependency Graph

```
            ┌──────────────────────────────────────┐
            │       A1 — Core Engine               │
            │  (data model, timeline, inference)    │
            └──────────┬───────────┬───────────────┘
                       │           │
              depends  │           │  depends
                       ▼           ▼
            ┌──────────────┐   ┌──────────────────┐
            │  A2 — Server │   │  A4 — AI/ML      │
            │  (wraps core │   │  (trait in core,  │
            │   with HTTP) │   │   backends in     │
            └──────┬───────┘   │   server)         │
                   │           └────────┬──────────┘
                   │                    │
              REST/WS API         AI endpoints
                   │                    │
                   ▼                    ▼
            ┌──────────────────────────────────────┐
            │       A3 — Frontend                  │
            │  (consumes API, renders everything)   │
            └──────────────────────────────────────┘
```

**Critical path:** A1 defines types → A2 + A4 consume them → A3 consumes A2's API.
A3 can start immediately with mock data and hardcoded types, converging with real
API at each sync point.

---

### Sprint 1 — Foundation (parallel kickoff)

> **Goal:** Compilable workspace, core types, server skeleton, UI shell, AI research.
> All agents start simultaneously. A3 works against mock data.

**A1 — Core Engine**

- [ ] Cargo workspace setup (`Cargo.toml` with `core` and `server` members)
- [ ] Timeline data model: `Timeline`, `ArcTrack`, `BeatClip`, `TimeRange`
- [ ] Story entity models: `StoryArc`, `Character`
- [ ] `EpisodeStructure` and `StructureSegment` types
- [ ] Default 30-min TV template (cold open, 3 acts, tag) with standard timings
- [ ] `Relationship` and `RelationshipType` types
- [ ] ID types (`TrackId`, `ClipId`, `ArcId`, `CharacterId`, `RelationshipId`)
- [ ] Unit tests for core data model construction and validation

**A2 — Server**

- [ ] `eidetic-server` crate skeleton with `axum` dependency
- [ ] Static file serving (serve compiled Svelte SPA from `dist/ui/`)
- [ ] Application state management (`Arc<Mutex<...>>` or similar)
- [ ] Stub REST routes returning hardcoded JSON (matching A1 types via `serde`)
- [ ] WebSocket endpoint scaffold (connect, subscribe, echo)
- [ ] CORS configuration for dev mode (Vite on different port)

**A3 — Frontend**

- [ ] Svelte 5 SPA scaffold (`ui/` with Vite, Tailwind, TypeScript)
- [ ] Pantograph dark theme: CSS custom properties, global styles
- [ ] `AppShell.svelte`: two-panel layout (beat editor top, timeline bottom)
- [ ] `PanelResizer.svelte`: draggable divider between panels
- [ ] `Sidebar.svelte`: collapsible sidebar shell (arcs, characters tabs)
- [ ] TypeScript types mirroring Rust models (`types.ts`)
- [ ] Mock stores with hardcoded data (`timeline.svelte.ts`, `story.svelte.ts`)
- [ ] `Timeline.svelte`: container with placeholder tracks and time ruler
- [ ] `TimeRuler.svelte`: time axis rendering (0:00 – 22:00)
- [ ] `StructureBar.svelte`: act structure visualization

**A4 — AI/ML**

- [ ] Research and document prompt templates for beat → script generation
- [ ] Define `AiBackend` trait signature in `crates/core/src/ai/backend.rs`
- [ ] Define `GenerateRequest`, `GenerateStream`, `ConsistencyUpdate` types
- [ ] Design context window packing strategy (what fits, priority order)
- [ ] Prototype prompt with a sample beat → script (offline, manual testing)

#### ✦ Sync Point 1 — "Hello World"
> **Criteria:** `cargo build` succeeds. `npm run dev` shows the two-panel layout.
> Server serves stub API and static files. Types are aligned across Rust and TypeScript.

---

### Sprint 2 — Timeline Interactivity + API Wiring

> **Goal:** Interactive timeline with real data flowing through the server.
> A3 drops mocks and connects to A2's real API backed by A1's types.

**A1 — Core Engine**

- [ ] Timeline operations: add/remove tracks, add/move/resize/delete clips
- [ ] Clip split logic (razor tool: one clip → two clips at a time point)
- [ ] Relationship CRUD (create, delete, query relationships for a clip)
- [ ] Scene inference engine: compute scenes from vertical clip overlap
- [ ] Subgenre presets (multi-cam, single-cam, animated) as template variants
- [ ] `BeatContent` and `ContentStatus` progression logic
- [ ] Project model (`Project` struct aggregating timeline + arcs + characters)

**A2 — Server**

- [ ] REST API: project CRUD (`POST/GET/PUT /api/project`)
- [ ] REST API: story entity CRUD (`/api/arcs`, `/api/characters`)
- [ ] REST API: timeline operations (`/api/timeline/*`)
- [ ] REST API: beat content (`/api/beats/:id`, `/api/beats/:id/notes`)
- [ ] REST API: scene list (`GET /api/scenes`)
- [ ] WebSocket: broadcast `timeline_changed` and `scenes_changed` events
- [ ] In-memory project state (no persistence to disk yet)

**A3 — Frontend**

- [ ] `api.ts`: REST client matching all server endpoints
- [ ] `ws.ts`: WebSocket client with reconnection and event dispatch
- [ ] Replace mock stores with live API-backed Svelte stores
- [ ] `ArcTrack.svelte`: render beat clips on a horizontal track lane
- [ ] `BeatClip.svelte`: draggable + resizable clip (pointer events + transforms)
- [ ] Beat clip interactions: add (click empty space), move (drag), resize (edge drag)
- [ ] Clip split interaction (razor cursor mode)
- [ ] `RelationshipLayer.svelte`: D3 hierarchical edge-bundled curves above tracks
- [ ] `RelationshipArc.svelte`: single bezier curve with color coding
- [ ] Drag-to-connect: draw a relationship arc between two clips
- [ ] `ArcList.svelte` + `ArcDetail.svelte`: sidebar arc management
- [ ] `CharacterList.svelte` + `CharacterDetail.svelte`: sidebar character management
- [ ] `BeatEditor.svelte`: contextual panel for selected clip
- [ ] `BeatNotes.svelte`: markdown editor for beat descriptions
- [ ] Default template rendering on new project (A-plot, B-plot, C-runner pre-placed)
- [ ] Subgenre preset picker (modal on project creation)

**A4 — AI/ML**

- [ ] Prompt construction module (`crates/core/src/ai/prompt.rs`)
- [ ] Context assembly: gather beat notes, arc info, characters, surrounding scripts
- [ ] Overlapping beat detection (query timeline for beats at same time position)
- [ ] Scene weaving prompt: multi-arc scene generation instructions
- [ ] Context window budgeting (`crates/core/src/ai/context.rs`)

#### ✦ Sync Point 2 — "Interactive Timeline"
> **Criteria:** User can create a project, manage arcs/characters in the sidebar,
> manipulate beat clips on the timeline (drag, resize, split, connect), write beat
> notes in the editor, and see inferred scenes update in real time. All data flows
> through REST/WS to the core engine and back.

---

### Sprint 3 — AI Generation Pipeline

> **Goal:** Beat notes produce AI-generated screenplay. Streaming tokens appear
> in the beat editor. Scene weaving merges overlapping arcs into unified scenes.

**A1 — Core Engine**

- [ ] Script element types: `SceneHeading`, `Action`, `Character`, `Dialogue`,
      `Parenthetical`, `Transition` — 30-min TV format only
- [ ] Script formatting rules (margins, caps, spacing for TV format)
- [ ] Script merge logic: combine user-written anchors with AI-generated text
- [ ] Time-to-page conversion (1 page ≈ 1 minute, clip duration → page budget)

**A2 — Server**

- [ ] llama.cpp backend implementation (`ai_backends/llama.rs`)
  - [ ] Model loading and inference (up to 80B parameters)
  - [ ] Streaming token output via channel
- [ ] OpenRouter backend implementation (`ai_backends/api.rs`)
  - [ ] HTTP client for OpenRouter API
  - [ ] Streaming SSE token parsing
- [ ] `POST /api/ai/generate`: assemble `GenerateRequest`, call backend, stream response
- [ ] `GET /api/ai/status`: backend health and model info
- [ ] `PUT /api/ai/config`: switch backends, set model, adjust parameters
- [ ] WebSocket: stream `generation_progress` and `generation_complete` events

**A3 — Frontend**

- [ ] `ScriptView.svelte`: formatted screenplay display (scene headings, dialogue, etc.)
- [ ] Streaming generation UX: tokens appear incrementally in the beat editor
- [ ] Beat editor stage progression UI (notes → generating → generated → refined)
- [ ] Beat clip status indicators (color/icon for empty, notes, generating, generated, written)
- [ ] "Generate" button on beat editor (trigger generation for selected clip)
- [ ] AI status indicator in UI (backend connected, model loaded, generating)
- [ ] AI configuration panel (backend selection, model, parameters)

**A4 — AI/ML**

- [ ] End-to-end generation pipeline: beat notes → prompt → LLM → formatted script
- [ ] Scene weaving: unified prompt for overlapping beats across arcs
- [ ] Character voice injection: embed voice notes into generation prompts
- [ ] Time budget enforcement: instruct model to produce ~N pages for clip duration
- [ ] User-written anchor preservation: integrate locked text into generation
- [ ] Output parsing: extract script elements from raw LLM output
- [ ] Quality validation: basic checks on generated output (length, format, characters)

#### ✦ Sync Point 3 — "AI Writes"
> **Criteria:** User writes beat notes, clicks generate, and sees formatted screenplay
> stream into the beat editor. Overlapping beats produce woven multi-arc scenes.
> Both llama.cpp (local) and OpenRouter (API) backends work.

---

### Sprint 4 — Interactive Editing + Consistency

> **Goal:** User can edit generated script inline. AI reacts to edits with
> consistency suggestions. Gap-filling and beat splitting work end-to-end.

**A1 — Core Engine**

- [ ] Edit tracking: diff user changes against generated text
- [ ] Consistency engine (`crates/core/src/ai/consistency.rs`)
  - [ ] Semantic change detection (character name, plot point, tone shift)
  - [ ] Downstream beat identification (which beats are affected by an edit)
  - [ ] Suggestion generation (proposed updates + reasoning)
- [ ] Beat locking/unlocking logic (mark as user-written, exclude from regeneration)
- [ ] Gap detection: identify time ranges with missing content between user-written beats

**A2 — Server**

- [ ] `PUT /api/beats/:id/script`: accept user edits, trigger consistency check
- [ ] `POST /api/beats/:id/lock` / `POST /api/beats/:id/unlock`
- [ ] `POST /api/ai/react`: edit reaction endpoint (diff → suggestions)
- [ ] WebSocket: stream `consistency_suggestion` events
- [ ] Persistence: project save/load to JSON files (`persistence.rs`)

**A3 — Frontend**

- [ ] Inline script editing in the beat editor (contenteditable or textarea)
- [ ] `DiffView.svelte`: side-by-side diff for consistency suggestions
- [ ] Accept/reject UI per suggestion
- [ ] Lock/unlock toggle on beat clips (visual indicator + click handler)
- [ ] Beat clip splitting UX (razor tool cursor, click to split)
- [ ] Beat clip resize with regeneration prompt ("Regenerate for new duration?")
- [ ] "Fill gaps" mode: highlight empty beats between user-written ones, offer generation

**A4 — AI/ML**

- [ ] Edit reaction pipeline: diff → semantic analysis → downstream impact → suggestions
- [ ] Character name propagation (rename across all beats)
- [ ] Plot point change detection and downstream flagging
- [ ] Dialogue tone consistency checking
- [ ] Gap-fill generation: produce bridging content between user-written beats A and C

#### ✦ Sync Point 4 — "Collaborative Editing"
> **Criteria:** User can edit generated scripts inline, lock beats, split beats, and
> resize beats. AI suggests consistency updates as side-by-side diffs. Gap filling
> generates bridging content. Projects save and load from disk.

---

### Sprint 5 — RAG, Polish, Export

> **Goal:** Reference materials enhance generation. PDF export produces
> industry-standard scripts. Core compiles to WASM.

**A1 — Core Engine**

- [ ] WASM compilation target (`wasm32-unknown-unknown`) — proof of concept
- [ ] Undo/redo command stack (timeline, beat content, relationship operations)
- [ ] Performance: optimize scene inference for large timelines
- [ ] Story arc progression analysis (does each arc advance sufficiently?)

**A2 — Server**

- [ ] PDF export endpoint (`POST /api/export/pdf`) — standard TV screenplay format
- [ ] Reference material ingestion endpoint (upload character bibles, style guides)
- [ ] Embedding generation for RAG (local or API-based)
- [ ] Vector storage for RAG chunks (in-memory or SQLite-backed)
- [ ] Iroh P2P integration scaffold (real-time collaboration foundation)

**A3 — Frontend**

- [ ] PDF export UI (button + progress + download)
- [ ] Undo/redo controls (Ctrl+Z/Ctrl+Y, toolbar buttons)
- [ ] Keyboard shortcuts (navigation, clip manipulation, editor commands)
- [ ] Reference material upload UI
- [ ] Performance: virtualized timeline rendering for large projects
- [ ] Polish: animations, transitions, loading states, error handling

**A4 — AI/ML**

- [ ] RAG pipeline: index reference materials → retrieve relevant chunks → inject into prompts
- [ ] Character voice learning: analyze user-written dialogue to refine voice profiles
- [ ] Episode-level consistency checking (continuity across all beats and scenes)
- [ ] Story arc progression tracking (flag arcs that stall or lack resolution)
- [ ] Prompt optimization based on generation quality feedback

#### ✦ Sync Point 5 — "Release Candidate"
> **Criteria:** Full workflow end-to-end. RAG enhances generation quality. PDF export
> produces properly formatted TV scripts. Undo/redo works across all operations.
> `eidetic-core` compiles to WASM. Project files save/load reliably.

---

### Agent Coordination Rules

1. **Shared contract first.** Before each sprint, agents agree on the types and API
   signatures they'll consume. A1 defines Rust types, A2 defines REST/WS contracts,
   A3 defines TypeScript mirrors. Changes to shared contracts require a mini-sync.

2. **Feature flags over branches.** All agents commit to `main`. Incomplete features
   are gated behind flags or stub implementations, not long-lived branches.

3. **Mock → Real progression.** A3 starts every sprint with mocks for new endpoints
   and swaps them for real API calls as A2 delivers. Mocks must match the agreed contract.

4. **Vertical slices at sync points.** Each sync point is a demo: one user-facing
   workflow working end-to-end, not a collection of isolated modules.

5. **A1 never blocks.** Core types are defined first in each sprint. If A1 is still
   refining logic, A2/A3/A4 work against the type signatures (which are stable early).

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
core data model (arc tracks + beat clips + relationships) is format-agnostic.

### 4. AI-First, Not AI-Later

The AI is not a bolt-on feature. The entire workflow assumes AI-generated content is the
default, with user edits as refinements. This inverts the traditional screenwriting software
model (user writes everything, AI assists occasionally).

**The user's workflow:**

1. Start from a template (standard 30-min TV, or a subgenre preset)
2. Define story arcs and characters in the sidebar
3. Adjust beat clips on the arc tracks — move, resize, add, remove
4. Click a beat clip and write beat notes describing what happens (the "turn")
5. Connect beats across arcs with relationship arcs to show causal/thematic links
6. AI generates screenplay for each beat, weaving overlapping arcs into unified scenes
7. User reads, edits, rewrites beats they want to control (lock as user-written)
8. AI suggests consistency updates to downstream beats (side-by-side diff)
9. Iterate until satisfied

### 5. Timeline Drives the Script

The script is a **derived artifact** of the timeline, not the primary data structure. This is
the fundamental difference from Plan A (and from every existing screenwriting tool). Arc
tracks hold "which storylines and when." Beat clips hold "what happens and why." Relationship
arcs show how beats connect across storylines. Scenes emerge from vertical overlap. The script
is the textual rendering of all of these, unified in a single timeline workspace.

---

## Resolved Questions

1. **Timeline rendering approach:** Undecided between HTML/CSS/SVG and HTML5 Canvas — both
   have merits. SVG is easier for interactivity (click handlers, drag, popovers) while Canvas
   performs better at scale. A hybrid approach remains a possibility. **To be resolved during
   prototyping.**

2. **Relationship arc drawing:** **Hierarchical edge bundling** — bezier curves that
   magnetically bundle along shared paths where relationships are similar, then splay out as
   they approach their individual endpoints. Inspired by Danny Holten's hierarchical edge
   bundling technique combined with Sankey-like flow aesthetics. Implementation will use D3's
   edge bundling algorithms. Reference examples:
   - [Hierarchical Edge Bundling](https://observablehq.com/@d3/hierarchical-edge-bundling/2)
   - [Radial Cluster](https://observablehq.com/@d3/radial-cluster/2)
   - [Sankey Diagram](https://observablehq.com/@d3/sankey/2)

3. **AI model requirements:**
   - **Local inference:** Support models up to 80B parameters via llama.cpp. Target models
     include Qwen3-30B-A3B-Instruct, famino-12B, and abliterated/Gutenberg-tuned models.
   - **API inference:** **OpenRouter** integration for easy model switching across providers
     (OpenAI, Anthropic, open-source hosted models, etc.) via a single API.

4. **Conflict resolution:** **Side-by-side diff.** When AI consistency suggestions conflict
   with user edits, present both versions in a side-by-side diff view for the user to
   choose between.

5. **Collaboration:** **Iroh P2P.** Collaboration will use [Iroh](https://iroh.computer/),
   a peer-to-peer networking library, for real-time multi-user collaboration without
   requiring a central server.
