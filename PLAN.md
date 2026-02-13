# Eidetic — AI-Driven Script Writing Platform (Alternative Plan)

## Vision

Eidetic is an AI-driven script writing tool focused on **30-minute TV episodes**. Rather than
chasing feature parity with Celtx or Final Draft, Eidetic reimagines script writing as a
**timeline-based, node-graph-driven, AI-collaborative** workflow — closer to a nonlinear editor
(NLE) for text than a traditional word processor.

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
| **Editor model** | Traditional document editor (ProseMirror-style) | Timeline + node graph + clip-based text decomposition |
| **AI role** | Assistant panel, suggestions, auto-complete (Phase 4) | Core from day one; generates and reacts to edits live |
| **Story structure** | Beat sheets, outlines, character catalog (Phase 3) | Story arcs, characters, beats, scenes as **graph nodes on a timeline** |
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
   about narrative as clips on a timeline with node-graph connections. This is a novel UX that
   makes AI integration natural — nodes carry context, clips carry text, and the AI can
   operate on the gaps.

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
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │  │
│  │  │ Script View  │  │  Node Graph  │  │  Timeline (NLE-style)   │ │  │
│  │  │ (generated   │  │  (arcs,      │  │  (clips, tracks,        │ │  │
│  │  │  script +    │  │   beats,     │  │   beats as regions,     │ │  │
│  │  │  inline      │  │   scenes,    │  │   scene boundaries)     │ │  │
│  │  │  editing)    │  │   characters)│  │                          │ │  │
│  │  └─────────────┘  └──────────────┘  └──────────────────────────┘ │  │
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
│  │                │    │  • Timeline engine (clips, tracks)   │   │      │
│  │  Serves:       │    │  • Character model                  │   │      │
│  │  • Static UI   │    │  • Script generation + formatting   │   │      │
│  │  • REST API    │    │  • AI orchestration                 │   │      │
│  │  • WS streams  │    │  • Project persistence              │   │      │
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

- **`ui/`** — Svelte 5 SPA. Renders the three-panel layout (script view, node graph, timeline).
  Communicates with the server via REST (commands) and WebSocket (streaming updates).

---

## Core Concepts

### 1. The Story Graph

Story structure is modeled as a directed graph, not a linear outline. Nodes represent:

- **Story Arcs** — High-level narrative threads (A-plot, B-plot, C-runner)
- **Characters** — Persistent entities with profiles, voice descriptions, relationships
- **Beats** — Narrative turning points ("inciting incident", "midpoint reversal", etc.)
- **Scenes** — Concrete dramatic units with location, characters, and purpose

Edges represent relationships:
- Arc → Beat (this beat advances this arc)
- Beat → Scene (this scene dramatizes this beat)
- Character → Scene (this character appears in this scene)
- Beat → Beat (causal dependency, "this must happen before that")
- Arc → Arc (thematic connection, subplot weaving)

```rust
pub struct StoryGraph {
    pub arcs: Vec<Arc>,
    pub characters: Vec<Character>,
    pub beats: Vec<Beat>,
    pub scenes: Vec<Scene>,
    pub edges: Vec<Edge>,
}

pub struct Arc {
    pub id: ArcId,
    pub name: String,
    pub description: String,
    pub arc_type: ArcType, // APlot, BPlot, CRunner
}

pub struct Beat {
    pub id: BeatId,
    pub name: String,
    pub description: String,
    pub beat_type: BeatType, // Teaser, ActBreak, Climax, Tag, Custom
    pub time_position: TimePosition,
}

pub struct Scene {
    pub id: SceneId,
    pub heading: String, // INT./EXT. LOCATION - TIME
    pub description: String,
    pub character_ids: Vec<CharacterId>,
    pub time_range: TimeRange,
}

pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub edge_type: EdgeType,
}
```

### 2. The Timeline

The timeline represents the **runtime** of the episode (approximately 22 minutes for a 30-min
TV episode, or 30 pages at 1 page/minute). It has:

- **Time ruler** — Measured in minutes/seconds or page equivalents
- **Tracks** — Multiple horizontal lanes (like audio/video tracks in an NLE)
  - Arc tracks (one per story arc, showing which beats are active)
  - Scene track (scene boundaries and durations)
  - Script track(s) (text "clips" representing generated or written script content)
- **Regions/Clips** — Rectangular blocks on tracks representing content with a time range
- **Markers** — Point-in-time markers for act breaks, commercial breaks, etc.

Users can:
- Drag clip edges to adjust duration (the AI regenerates to fit the new timing)
- Split clips to decompose a section for finer editing
- Rearrange clips to reorder scenes
- Layer multiple script tracks (e.g., "draft 1" vs "draft 2" for A/B comparison)
- Snap beats to standard TV structure positions (cold open, act 1, act 2, etc.)

```rust
pub struct Timeline {
    pub total_duration: Duration,  // e.g., 22 minutes
    pub tracks: Vec<Track>,
    pub markers: Vec<Marker>,
}

pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub track_type: TrackType, // Arc, Scene, Script, Beat
    pub clips: Vec<Clip>,
}

pub struct Clip {
    pub id: ClipId,
    pub time_range: TimeRange,
    pub content: ClipContent,
    pub locked: bool,
}

pub enum ClipContent {
    ScriptText { text: String, generation_status: GenStatus },
    BeatRegion { beat_id: BeatId },
    SceneRegion { scene_id: SceneId },
    ArcRegion { arc_id: ArcId },
}

pub struct TimeRange {
    pub start: Duration,
    pub end: Duration,
}
```

### 3. The Script View

Above the timeline and node graph, the user sees the **generated script** — a formatted,
editable view of the episode's screenplay. This is the "output" that the timeline and graph
produce.

Key behaviors:
- **Live generation:** As the user defines beats, scenes, and arcs in the graph/timeline,
  the AI generates script text to fill the corresponding time slots
- **Inline editing:** The user can type directly into the script. Edits are tracked and
  the AI treats user-written text as canonical (it won't overwrite it)
- **User-written beats:** The user can write certain scenes/beats themselves. The AI fills
  in the transitions and connecting tissue between user-written sections
- **Consistency reactions:** When the user edits the script, the AI can update downstream
  content to remain consistent (e.g., if a character's name changes in an edit, the AI
  updates subsequent references)
- **Contextual awareness:** The AI uses the full story graph (arcs, characters, beats),
  surrounding script text, and any RAG-retrieved reference material to generate coherent
  output

### 4. AI Integration (Core, Not Afterthought)

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
│   │       ├── graph/          # Story graph data model
│   │       │   ├── mod.rs
│   │       │   ├── arc.rs      # Story arcs
│   │       │   ├── beat.rs     # Narrative beats
│   │       │   ├── scene.rs    # Scenes
│   │       │   ├── character.rs # Characters and relationships
│   │       │   └── edge.rs     # Graph edges and traversal
│   │       ├── timeline/       # Timeline engine
│   │       │   ├── mod.rs
│   │       │   ├── track.rs    # Tracks and track types
│   │       │   ├── clip.rs     # Clips (text clips, beat regions, etc.)
│   │       │   ├── marker.rs   # Act break markers, etc.
│   │       │   └── timing.rs   # Duration math, page-to-time conversion
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
│           │   ├── graph.rs    # Story graph operations
│           │   ├── timeline.rs # Timeline operations
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
│   │   │       ├── graph.svelte.ts
│   │   │       ├── timeline.svelte.ts
│   │   │       └── script.svelte.ts
│   │   └── components/
│   │       ├── layout/
│   │       │   ├── AppShell.svelte       # Three-panel layout
│   │       │   └── PanelResizer.svelte   # Draggable panel boundaries
│   │       ├── script/
│   │       │   ├── ScriptView.svelte     # Formatted script display + editing
│   │       │   ├── ScriptElement.svelte  # Individual script element renderer
│   │       │   └── InlineEditor.svelte   # Contenteditable inline editing
│   │       ├── graph/
│   │       │   ├── NodeGraph.svelte      # Canvas-based node graph
│   │       │   ├── ArcNode.svelte        # Story arc node
│   │       │   ├── BeatNode.svelte       # Beat node
│   │       │   ├── SceneNode.svelte      # Scene node
│   │       │   ├── CharacterNode.svelte  # Character node
│   │       │   └── EdgeRenderer.svelte   # Edge drawing between nodes
│   │       ├── timeline/
│   │       │   ├── Timeline.svelte       # Main timeline component
│   │       │   ├── Track.svelte          # Single track lane
│   │       │   ├── Clip.svelte           # Draggable/resizable clip
│   │       │   ├── TimeRuler.svelte      # Time ruler header
│   │       │   ├── Playhead.svelte       # Current position indicator
│   │       │   └── Marker.svelte         # Act break / marker
│   │       ├── character/
│   │       │   ├── CharacterPanel.svelte # Character detail editor
│   │       │   └── CharacterList.svelte  # Character sidebar list
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

# Story Graph
GET    /api/graph                      # Full story graph
POST   /api/graph/arcs                 # Create arc
PUT    /api/graph/arcs/:id             # Update arc
DELETE /api/graph/arcs/:id             # Delete arc
POST   /api/graph/beats                # Create beat
PUT    /api/graph/beats/:id            # Update beat
DELETE /api/graph/beats/:id            # Delete beat
POST   /api/graph/scenes               # Create scene
PUT    /api/graph/scenes/:id           # Update scene
DELETE /api/graph/scenes/:id           # Delete scene
POST   /api/graph/characters           # Create character
PUT    /api/graph/characters/:id       # Update character
DELETE /api/graph/characters/:id       # Delete character
POST   /api/graph/edges                # Create edge
DELETE /api/graph/edges/:id            # Delete edge

# Timeline
GET    /api/timeline                   # Full timeline state
PUT    /api/timeline/clips/:id         # Update clip (resize, move)
POST   /api/timeline/clips/:id/split   # Split a clip
POST   /api/timeline/tracks            # Add track
DELETE /api/timeline/tracks/:id        # Remove track
PUT    /api/timeline/markers/:id       # Update marker position

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
{ type: "graph_changed", data: { graph } }
{ type: "timeline_changed", data: { timeline } }

// Client -> Server
{ type: "subscribe", data: { channels: ["script", "generation", "graph"] } }
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
1. User defines beats and scenes in the graph
2. Beats are placed on the timeline with time allocations
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

### Phase 1: Skeleton + Server + Story Graph

- [ ] Cargo workspace with `core` and `server` crates
- [ ] `eidetic-core`: Story graph data model (arcs, beats, scenes, characters, edges)
- [ ] `eidetic-core`: Timeline data model (tracks, clips, markers, time ranges)
- [ ] `eidetic-core`: Basic script element types (scene heading, action, character, dialogue,
      parenthetical, transition) — 30-min TV format only
- [ ] `eidetic-server`: axum HTTP server serving static files
- [ ] `eidetic-server`: REST API for graph CRUD and timeline operations
- [ ] `eidetic-server`: WebSocket endpoint for streaming updates
- [ ] Svelte 5 SPA skeleton with Pantograph dark theme
- [ ] Three-panel layout: script view (top), node graph (middle-left), timeline (bottom)
- [ ] Node graph: render arcs, beats, scenes, characters as draggable nodes with edges
- [ ] Timeline: tracks, clips, time ruler, act break markers
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

### 5. Graph + Timeline = Script

The script is a **derived artifact** of the story graph and timeline, not the primary data
structure. This is the fundamental difference from Plan A (and from every existing
screenwriting tool). The graph holds "what happens and why." The timeline holds "when and for
how long." The script is the textual rendering of those two structures.

---

## Open Questions

1. **Node graph library:** Build a custom canvas-based graph renderer in Svelte, or use
   an existing library (e.g., Svelvet, xyflow/react-flow ported to Svelte)? A custom
   solution offers more control but more work.

2. **Timeline library:** Same question. NLE-style timelines are complex UI. Consider
   adapting an open-source timeline component or building from scratch with HTML5 Canvas.

3. **AI model requirements:** What model size is needed for acceptable script generation?
   7B models are fast but may produce weak dialogue. 13B+ is better but requires more
   VRAM. The pluggable backend means users can choose local vs. API.

4. **Conflict resolution:** When the AI suggests consistency updates that conflict with
   other user edits, how should conflicts be presented? Inline diff markers?
   Side-by-side comparison?

5. **Collaboration (future):** The REST+WS architecture naturally supports multi-user
   access. Should we design the API with multi-tenancy in mind from the start, or keep
   it single-user and add collaboration later?
