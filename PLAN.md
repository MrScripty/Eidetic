# Eidetic — AI-Assisted Writing Platform

## Project Vision

Eidetic is a cross-platform, AI-assisted book and movie script writing platform built in Rust.
It uses CEF (Chromium Embedded Framework) with Svelte 5 for the UI, local AI inference via
llama.cpp, and Pumas-Library for model management. The platform targets feature parity with
Celtx for script/book writing while adding AI-powered assistance throughout the creative process.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        EIDETIC APPLICATION                          │
│                                                                     │
│  ┌───────────────────────┐      ┌────────────────────────────────┐ │
│  │   eidetic-core crate  │      │     CEF Browser Process        │ │
│  │   (reusable library)  │      │                                │ │
│  │                       │      │  ┌──────────────────────────┐  │ │
│  │ • Project management  │ IPC  │  │   Svelte 5 SPA (UI)     │  │ │
│  │ • Document engine     │<────>│  │                          │  │ │
│  │ • Script formatting   │      │  │ • Script editor          │  │ │
│  │ • AI inference        │      │  │ • Beat sheet / outliner  │  │ │
│  │ • Export pipeline     │      │  │ • Character manager      │  │ │
│  │ • File I/O            │      │  │ • Scene navigator        │  │ │
│  │ • State management    │      │  │ • AI assistant panel     │  │ │
│  │                       │      │  │ • Pantograph dark theme  │  │ │
│  └───────┬───────────────┘      │  └──────────────────────────┘  │ │
│          │                      └────────────────────────────────┘ │
│          │                                                         │
│  ┌───────▼───────────────┐      ┌────────────────────────────────┐ │
│  │  pumas-library crate  │      │     llama-cpp-2 crate          │ │
│  │                       │      │                                │ │
│  │ • Model discovery     │─────>│ • GGUF model loading           │ │
│  │ • Download + cache    │      │ • Token streaming              │ │
│  │ • Metadata / search   │      │ • GPU offloading (CUDA/Metal)  │ │
│  │ • Health monitoring   │      │ • Context management           │ │
│  └───────────────────────┘      └────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### Separation of Concerns

The backend (`eidetic-core`) is a standalone Rust library crate with **zero UI dependencies**. It
exposes a public API that any frontend can consume — the CEF/Svelte UI is one such consumer. This
means `eidetic-core` could also be used headlessly, integrated into other tools, or driven by
a different UI in the future.

The frontend (Svelte 5 SPA) is a pure web application that communicates with the backend
exclusively through a typed IPC message protocol, following the pattern established in Pentimento.

---

## Platform Targets

| Platform | Priority | GPU Backend | Notes |
|---|---|---|---|
| Linux x86_64 | Primary | Vulkan | Must build and test in CI |
| Windows x86_64 | Secondary | Vulkan / CUDA | Must build and test in CI |
| macOS ARM (Apple Silicon) | Best-effort | Metal | Must compile; testing optional |

Per Coding-Standards/CROSS-PLATFORM-STANDARDS.md, platform-specific behavior hides behind
abstraction interfaces. No scattered `#[cfg]` in business logic — use a strategy + factory pattern
with per-platform implementation files.

---

## Cargo Workspace Structure

```
Eidetic/
├── Cargo.toml                  # Workspace root
├── justfile                    # Build orchestration (like Pentimento)
├── .editorconfig               # From Coding-Standards template
├── lefthook.yml                # Pre-commit hooks from Coding-Standards
├── LICENSE                     # MIT (existing)
├── .gitignore                  # Rust + Node ignores (existing, extend)
├── PLAN.md                     # This document
│
├── crates/
│   ├── app/                    # Desktop application binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # Entry point, CEF process detection
│   │       ├── window.rs       # Window creation and management
│   │       ├── embedded_ui.rs  # rust-embed asset serving
│   │       └── platform/       # Platform-specific impls
│   │           ├── mod.rs
│   │           ├── linux.rs
│   │           ├── windows.rs
│   │           └── macos.rs
│   │
│   ├── core/                   # eidetic-core: reusable library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Public API surface
│   │       ├── project/        # Project file management
│   │       │   ├── mod.rs
│   │       │   ├── project.rs  # Project struct, metadata, save/load
│   │       │   ├── document.rs # Document types (script, novel, beat sheet)
│   │       │   └── storage.rs  # File I/O, project format
│   │       ├── script/         # Script engine
│   │       │   ├── mod.rs
│   │       │   ├── format.rs   # Script formatting rules per type
│   │       │   ├── element.rs  # Script elements (scene heading, action, dialogue, etc.)
│   │       │   ├── parser.rs   # Script text parsing
│   │       │   └── template.rs # Script templates (feature film, TV, stageplay, etc.)
│   │       ├── novel/          # Novel/prose engine
│   │       │   ├── mod.rs
│   │       │   ├── chapter.rs  # Chapter management
│   │       │   └── format.rs   # Prose formatting
│   │       ├── story/          # Story development tools
│   │       │   ├── mod.rs
│   │       │   ├── beat.rs     # Beat sheet / beat cards
│   │       │   ├── outline.rs  # Story outline / structure
│   │       │   └── character.rs # Character catalog
│   │       ├── scene/          # Scene management
│   │       │   ├── mod.rs
│   │       │   ├── navigator.rs # Scene listing, ordering, summaries
│   │       │   ├── breakdown.rs # Scene element tagging
│   │       │   └── timeline.rs  # Dramatic days, continuity
│   │       ├── ai/             # AI integration layer
│   │       │   ├── mod.rs
│   │       │   ├── engine.rs   # llama.cpp wrapper, model loading, inference
│   │       │   ├── session.rs  # Chat session management, context windows
│   │       │   ├── prompt.rs   # Prompt templates for writing tasks
│   │       │   └── stream.rs   # Token streaming abstraction
│   │       ├── export/         # Export pipeline
│   │       │   ├── mod.rs
│   │       │   ├── pdf.rs      # PDF export (screenplay format)
│   │       │   ├── fountain.rs # Fountain format (.fountain)
│   │       │   ├── fdx.rs      # Final Draft XML (.fdx)
│   │       │   ├── plain.rs    # Plain text export
│   │       │   └── html.rs     # HTML export
│   │       ├── import/         # Import pipeline
│   │       │   ├── mod.rs
│   │       │   ├── fountain.rs # Fountain import
│   │       │   ├── fdx.rs      # Final Draft import
│   │       │   └── plain.rs    # Plain text import
│   │       ├── revision/       # Version control and revision tracking
│   │       │   ├── mod.rs
│   │       │   ├── draft.rs    # Draft/version management
│   │       │   ├── history.rs  # Edit history, undo/redo
│   │       │   └── diff.rs     # Revision diff and color-coded tracking
│   │       ├── analytics/      # Script insights and statistics
│   │       │   ├── mod.rs
│   │       │   ├── stats.rs    # Word count, page count, dialogue distribution
│   │       │   └── goals.rs    # Writing goals and progress tracking
│   │       └── error.rs        # Unified error types
│   │
│   ├── ipc/                    # Typed IPC message protocol
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Re-exports
│   │       ├── messages.rs     # BackendToUi / UiToBackend enums
│   │       ├── commands/       # Command message types
│   │       │   ├── mod.rs
│   │       │   ├── project.rs  # Project commands (open, save, new, close)
│   │       │   ├── document.rs # Document editing commands
│   │       │   ├── script.rs   # Script-specific commands
│   │       │   ├── story.rs    # Beat sheet, outline commands
│   │       │   ├── ai.rs       # AI inference commands
│   │       │   └── export.rs   # Export/import commands
│   │       ├── types/          # Shared data types
│   │       │   ├── mod.rs
│   │       │   ├── project.rs  # Project metadata types
│   │       │   ├── document.rs # Document content types
│   │       │   ├── script.rs   # Script element types
│   │       │   ├── character.rs # Character types
│   │       │   ├── scene.rs    # Scene types
│   │       │   └── ai.rs       # AI response types
│   │       ├── input.rs        # Keyboard/mouse event types
│   │       └── error.rs        # IPC error types
│   │
│   ├── webview/                # CEF webview abstraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # Webview trait abstraction
│   │       ├── error.rs
│   │       ├── cef.rs          # CEF backend implementation
│   │       └── platform/
│   │           ├── mod.rs
│   │           ├── linux.rs    # Linux CEF specifics
│   │           ├── windows.rs  # Windows CEF specifics
│   │           └── macos.rs    # macOS CEF specifics (app bundle handling)
│   │
│   └── cef-helper/             # CEF subprocess helper binary
│       ├── Cargo.toml
│       └── src/
│           └── main.rs         # Minimal CEF subprocess entry point
│
├── ui/                         # Svelte 5 frontend
│   ├── package.json
│   ├── svelte.config.js        # SvelteKit static adapter
│   ├── vite.config.ts          # Vite build config
│   ├── tailwind.config.cjs     # Tailwind with Pantograph theme
│   ├── postcss.config.cjs      # PostCSS + autoprefixer
│   ├── tsconfig.json
│   ├── src/
│   │   ├── app.html            # SPA shell
│   │   ├── styles.css          # Global styles (Pantograph theme tokens)
│   │   ├── routes/
│   │   │   ├── +layout.svelte  # Root layout
│   │   │   ├── +layout.js      # SSR disabled, no prerender
│   │   │   └── +page.svelte    # Main app page
│   │   ├── lib/
│   │   │   ├── bridge.ts       # IPC bridge (mirrors Pentimento pattern)
│   │   │   ├── types.ts        # TypeScript types mirroring Rust IPC types
│   │   │   ├── stores/         # Svelte 5 state management
│   │   │   │   ├── project.svelte.ts
│   │   │   │   ├── editor.svelte.ts
│   │   │   │   ├── ai.svelte.ts
│   │   │   │   └── ui.svelte.ts
│   │   │   └── utils/          # Shared utilities
│   │   │       ├── formatting.ts # Script element formatting helpers
│   │   │       └── keyboard.ts   # Keyboard shortcut handling
│   │   └── components/
│   │       ├── layout/         # App shell layout
│   │       │   ├── AppShell.svelte
│   │       │   ├── Sidebar.svelte
│   │       │   ├── TopBar.svelte
│   │       │   └── StatusBar.svelte
│   │       ├── editor/         # Core editing components
│   │       │   ├── ScriptEditor.svelte       # Main script editing surface
│   │       │   ├── ProseEditor.svelte        # Novel/prose editing
│   │       │   ├── ElementToolbar.svelte     # Script element type selector
│   │       │   ├── SceneNavigator.svelte     # Scene list / jump-to
│   │       │   ├── TitlePage.svelte          # Title page editor
│   │       │   └── DualDialogue.svelte       # Side-by-side dialogue
│   │       ├── story/          # Story development tools
│   │       │   ├── BeatSheet.svelte          # Visual beat canvas
│   │       │   ├── BeatCard.svelte           # Individual beat card
│   │       │   ├── OutlinePanel.svelte       # Outline tree view
│   │       │   └── IndexCards.svelte         # Index card view
│   │       ├── character/      # Character management
│   │       │   ├── CharacterCatalog.svelte   # Character list
│   │       │   ├── CharacterProfile.svelte   # Character detail editor
│   │       │   └── CharacterInsights.svelte  # Dialogue stats per character
│   │       ├── scene/          # Scene management
│   │       │   ├── SceneBreakdown.svelte     # Element tagging
│   │       │   ├── SceneSummary.svelte       # Scene summary editor
│   │       │   └── Timeline.svelte           # Dramatic day tracking
│   │       ├── ai/             # AI assistant UI
│   │       │   ├── AiPanel.svelte            # AI chat/assistant sidebar
│   │       │   ├── AiSuggestions.svelte      # Inline AI suggestions
│   │       │   ├── ModelSelector.svelte      # Model picker (via Pumas-Library)
│   │       │   └── PromptTemplates.svelte    # Quick prompt templates
│   │       ├── project/        # Project management
│   │       │   ├── ProjectBrowser.svelte     # Project open/create
│   │       │   ├── ProjectSettings.svelte    # Project metadata editor
│   │       │   └── FileTree.svelte           # Project file navigator
│   │       ├── export/         # Export UI
│   │       │   ├── ExportDialog.svelte       # Export format selection
│   │       │   └── PrintPreview.svelte       # PDF preview
│   │       ├── revision/       # Revision tracking UI
│   │       │   ├── RevisionPanel.svelte      # Revision color/mode selector
│   │       │   ├── DraftManager.svelte       # Draft version list
│   │       │   └── DiffView.svelte           # Side-by-side diff
│   │       ├── analytics/      # Writing analytics
│   │       │   ├── StatsPanel.svelte         # Word/page counts, charts
│   │       │   └── GoalsWidget.svelte        # Writing goal progress
│   │       └── shared/         # Reusable UI primitives
│   │           ├── Button.svelte
│   │           ├── Modal.svelte
│   │           ├── Dropdown.svelte
│   │           ├── TabBar.svelte
│   │           ├── SplitPane.svelte
│   │           └── ContextMenu.svelte
│   └── static/
│       └── fonts/              # JetBrains Mono (from Pantograph theme)
│
├── standards/                  # Copied from Coding-Standards repo
│   ├── CODING-STANDARDS.md
│   ├── TESTING-STANDARDS.md
│   ├── COMMIT-STANDARDS.md
│   ├── ARCHITECTURE-PATTERNS.md
│   ├── DOCUMENTATION-STANDARDS.md
│   ├── SECURITY-STANDARDS.md
│   ├── TOOLING-STANDARDS.md
│   ├── DEPENDENCY-STANDARDS.md
│   ├── CONCURRENCY-STANDARDS.md
│   ├── CROSS-PLATFORM-STANDARDS.md
│   ├── INTEROP-STANDARDS.md
│   ├── .editorconfig
│   ├── lefthook.yml
│   └── README-TEMPLATE.md
│
└── dist/                       # Build output (gitignored)
    └── ui/                     # Compiled Svelte SPA
```

---

## Crate Dependency Graph

```
eidetic-app (binary)
├── eidetic-core (library)
│   ├── eidetic-ipc (types only — no runtime deps)
│   ├── pumas-library (model management)
│   ├── llama-cpp-2 (AI inference)
│   ├── serde / serde_json (serialization)
│   ├── tokio (async runtime)
│   ├── sqlx or rusqlite (project database)
│   ├── printpdf or typst (PDF export)
│   └── thiserror (error handling)
├── eidetic-webview (CEF abstraction)
│   ├── cef = "143" (tauri-apps/cef-rs)
│   ├── eidetic-ipc
│   └── rust-embed (UI asset bundling)
├── eidetic-ipc (shared types)
│   ├── serde / serde_json
│   └── thiserror
└── platform dependencies
    ├── [linux] gtk, gdk
    ├── [windows] windows-sys
    └── [macos] cocoa, objc
```

### Dependency Justification (per DEPENDENCY-STANDARDS.md)

| Dependency | Purpose | Justification |
|---|---|---|
| `cef` (143+) | CEF Rust bindings | Core requirement; >200 lines equivalent |
| `llama-cpp-2` | llama.cpp bindings | Core AI requirement; actively maintained |
| `pumas-library` | Model management | Core requirement; project ecosystem crate |
| `serde` + `serde_json` | JSON serialization | IPC protocol; ecosystem standard |
| `tokio` | Async runtime | AI inference streaming, file I/O |
| `thiserror` | Error types | Reduces boilerplate; <20 transitive deps |
| `rust-embed` | Asset bundling | UI embedding; established pattern from Pentimento |
| `printpdf` or `typst` | PDF generation | Export pipeline; evaluate both |

---

## IPC Message Protocol

Following Pentimento's pattern, all messages use serde's internally-tagged JSON format.

### BackendToUi (Events from Rust to Svelte)

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum BackendToUi {
    // Lifecycle
    Initialize { project: ProjectInfo, settings: AppSettings },
    Error { code: String, message: String },

    // Project
    ProjectOpened(ProjectInfo),
    ProjectSaved,
    DocumentChanged(DocumentContent),
    DocumentListUpdated(Vec<DocumentMeta>),

    // Script editing
    FormatApplied { element_type: ScriptElementType },
    AutoComplete { suggestions: Vec<String> },
    CursorContext { element_type: ScriptElementType, scene_number: Option<u32> },

    // Story development
    BeatSheetUpdated(BeatSheet),
    OutlineUpdated(Outline),
    CharacterListUpdated(Vec<CharacterSummary>),
    CharacterDetailLoaded(CharacterProfile),

    // Scene management
    SceneListUpdated(Vec<SceneSummary>),
    BreakdownUpdated(SceneBreakdown),

    // AI
    AiTokenStream { session_id: String, token: String },
    AiResponseComplete { session_id: String },
    AiModelStatus { model_id: String, status: ModelStatus },
    AiModelsAvailable(Vec<ModelInfo>),

    // Analytics
    StatsUpdated(ScriptStats),
    GoalProgress { words_today: u32, target: u32 },

    // Revision
    DraftListUpdated(Vec<DraftMeta>),
    RevisionApplied { revision_color: String },

    // Export
    ExportProgress { percent: f32 },
    ExportComplete { path: String },
}
```

### UiToBackend (Commands from Svelte to Rust)

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum UiToBackend {
    // Project
    NewProject { name: String, template: ProjectTemplate },
    OpenProject { path: String },
    SaveProject,
    CloseProject,

    // Document editing
    OpenDocument { doc_id: String },
    UpdateContent { doc_id: String, content: String, cursor_pos: usize },
    SetElementType { element_type: ScriptElementType },
    RequestAutoComplete { prefix: String, element_type: ScriptElementType },

    // Story development
    CreateBeat { title: String, content: String },
    UpdateBeat { beat_id: String, title: Option<String>, content: Option<String> },
    ReorderBeats { beat_ids: Vec<String> },
    DeleteBeat { beat_id: String },
    BeatToScene { beat_id: String },

    // Character management
    CreateCharacter { name: String },
    UpdateCharacter { char_id: String, profile: CharacterProfile },
    DeleteCharacter { char_id: String },

    // Scene management
    UpdateSceneSummary { scene_id: String, summary: String },
    TagSceneElement { scene_id: String, element: BreakdownElement },
    SetDramaticDay { scene_id: String, day: u32 },

    // AI
    AiStartSession { model_id: String, system_prompt: Option<String> },
    AiSendMessage { session_id: String, message: String },
    AiCancelGeneration { session_id: String },
    AiLoadModel { model_id: String },
    AiUnloadModel { model_id: String },
    AiRefreshModels,

    // Export
    ExportDocument { doc_id: String, format: ExportFormat, options: ExportOptions },
    ImportDocument { path: String, format: ImportFormat },

    // Revision
    CreateDraft { name: String },
    SwitchDraft { draft_id: String },
    StartRevision { color: String },
    StopRevision,

    // Analytics
    SetWritingGoal { daily_words: u32, deadline: Option<String> },

    // Settings
    UpdateSettings(AppSettings),
}
```

### IPC Transport (following Pentimento's CEF pattern)

**UI -> Backend:** Svelte calls `console.log("__EIDETIC_IPC__:" + json)`. The CEF
`DisplayHandler` intercepts console messages with this prefix, deserializes the JSON into
`UiToBackend`, and sends it through an `mpsc` channel.

**Backend -> UI:** Rust serializes `BackendToUi` to JSON and injects it via CEF's
`frame.execute_javascript()` calling `window.__EIDETIC_RECEIVE__(json)`.

**TypeScript bridge:** A `bridge.ts` module (mirroring Pentimento's pattern) provides typed
wrapper methods and a subscriber pattern for incoming messages.

---

## Frontend Theme (Pantograph)

The UI uses the Pantograph dark theme with the following design tokens:

```css
:root {
    --color-bg-inner: #0a0a0a;
    --color-bg-outer: #262626;
    --color-accent: #3b82f6;
    --color-grid: rgba(255, 255, 255, 0.05);
    --color-ruler: #404040;
}
```

- **Font:** JetBrains Mono (monospace) for UI chrome; a serif/proportional font option for
  script/prose editing (user-configurable)
- **Color palette:** Tailwind CSS defaults (neutral-900, neutral-800, neutral-700, blue-500, etc.)
- **Backdrop blur:** Semi-transparent panels with `backdrop-filter: blur()`
- **Component styling:** Tailwind utility classes, consistent with Pantograph component patterns

The Tailwind config extends Pantograph's approach but adds writing-specific tokens:

```javascript
// tailwind.config.cjs (additions to Pantograph base)
module.exports = {
    content: ['./src/**/*.{svelte,ts}'],
    theme: {
        extend: {
            fontFamily: {
                mono: ['JetBrains Mono', 'ui-monospace', /* ... */],
                script: ['Courier Prime', 'Courier New', 'monospace'],
                prose: ['Georgia', 'Cambria', 'serif'],
            },
        },
    },
};
```

---

## Script Formatting Engine

The core script engine supports industry-standard formats:

### Supported Script Types

| Type | Format Standard | Key Elements |
|---|---|---|
| Screenplay (Film) | Hollywood standard | Scene heading, action, character, dialogue, parenthetical, transition |
| TV Script (1-hour) | Network standard | Act breaks, teasers, tags |
| TV Script (half-hour) | Sitcom format | Same + possible multi-camera format |
| Stageplay | Theater standard | Stage directions, character, dialogue |
| Multi-column A/V | Documentary/corporate | Audio column, video column |
| Novel/Prose | Standard manuscript | Chapters, paragraphs, formatting |

### Script Element Types

```rust
pub enum ScriptElementType {
    SceneHeading,       // INT./EXT. LOCATION - TIME
    Action,             // Scene description
    Character,          // Character name (centered, uppercase)
    Dialogue,           // Character speech
    Parenthetical,      // Acting direction within dialogue
    Transition,         // CUT TO:, FADE IN:, etc.
    Shot,               // Camera direction
    DualDialogue,       // Two characters speaking simultaneously
    ActBreak,           // TV act break
    TitlePage,          // Title page elements
    Note,               // Writer's note (non-printing)
}
```

### Auto-Formatting Rules

Each element type has defined formatting rules (margins, caps, alignment) applied automatically.
The engine follows Fountain format internally for storage, with real-time visual formatting
in the Svelte editor.

---

## AI Integration Architecture

### Model Management (via Pumas-Library)

Pumas-Library provides:
- Model discovery and catalog (SQLite FTS5 full-text search)
- HuggingFace download with progress tracking
- SHA256 + BLAKE3 dual-hash verification
- Cross-process library discovery via SQLite registry
- JSON-RPC 2.0 IPC for multi-process model sharing

Eidetic integrates as a Pumas-Library client. On startup, it discovers a running Pumas-Library
primary instance or initializes one. The model selector UI queries available models through
this integration.

### Inference Engine (via llama-cpp-2)

```
Pumas-Library ──(provides GGUF path)──> llama-cpp-2 ──(loads model)──> LlamaModel
                                                                           │
                                              ┌────────────────────────────┘
                                              ▼
                                        LlamaContext (per session)
                                              │
                                    ┌─────────┴─────────┐
                                    ▼                   ▼
                              Chat Session        Completion Session
                              (dialogue AI)       (inline suggestions)
```

### AI Features

| Feature | Description | Prompt Strategy |
|---|---|---|
| **Writing assistant** | Chat-based creative guidance | System prompt with project context |
| **Scene generation** | Generate scene drafts from beats | Beat content + character profiles as context |
| **Dialogue polish** | Rewrite/improve dialogue | Character voice description + surrounding context |
| **Character voice** | Maintain character consistency | Character profile + dialogue samples |
| **Plot suggestions** | Story direction brainstorming | Outline + beat sheet as context |
| **Auto-complete** | Inline text completion | Recent text as prompt, element-type-aware |
| **Format conversion** | Adapt between script types | Source text + target format rules |
| **Summarization** | Scene/chapter summaries | Full text of scene/chapter |

### GPU Acceleration

| Platform | Backend | Feature Flag |
|---|---|---|
| Linux | Vulkan | `llama-cpp-2/vulkan` |
| Windows | Vulkan or CUDA | `llama-cpp-2/vulkan` or `llama-cpp-2/cuda` |
| macOS (Apple Silicon) | Metal | `llama-cpp-2/metal` |

The `eidetic-core` AI engine detects available hardware at startup and selects the appropriate
backend. GPU layer offloading is configured based on available VRAM.

---

## CEF Integration (following Pentimento)

### Architecture

- **Offscreen rendering:** CEF renders the Svelte UI to a pixel buffer, not a native window.
  Eidetic composites this into its own window. This follows Pentimento's recommended CEF mode.
- **Separate helper binary:** `eidetic-cef-helper` handles CEF subprocess roles (renderer, GPU,
  utility) to avoid GTK initialization conflicts.
- **Manual message pump:** CEF's event loop is driven from the main application loop.
- **Sandbox disabled:** Simplified deployment model.

### CEF Crate

Use `cef = "143"` from `tauri-apps/cef-rs`:
- Automated CEF binary download via `download-cef` / `export-cef-dir`
- CMake builds `libcef_dll_wrapper` automatically
- Cross-platform bundling tools included

### Asset Serving

**Production:** The Svelte SPA is compiled to `dist/ui/`, then embedded into the Rust binary
via `rust-embed`. The `embedded_ui.rs` module inlines CSS and JS into a single HTML string
and loads it directly into CEF via `load_string()`. No HTTP server needed.

**Development:** When `EIDETIC_DEV` env var is set, CEF navigates to `http://localhost:5173`
(Vite dev server) for hot-reload during development.

---

## Feature Roadmap

### Phase 1: Foundation

- [ ] Cargo workspace scaffolding (all crates, Cargo.toml files)
- [ ] CEF integration (window, offscreen rendering, IPC transport)
- [ ] Svelte 5 SPA skeleton with Pantograph theme
- [ ] IPC message protocol (bridge.ts ↔ Rust types)
- [ ] Project management (new, open, save, close)
- [ ] Basic text editor (plain rich text, no script formatting yet)
- [ ] Build pipeline (justfile, Vite, cargo build)
- [ ] Cross-platform CI (Linux + Windows builds)

### Phase 2: Script Engine

- [ ] Script element types and auto-formatting
- [ ] Screenplay template with full element support
- [ ] Scene heading auto-complete (INT./EXT., locations, times)
- [ ] Character name auto-complete
- [ ] Scene navigator (list, jump-to, reorder)
- [ ] Title page editor
- [ ] Dual dialogue support
- [ ] TV script template (act breaks, teasers)
- [ ] Stageplay template

### Phase 3: Story Development

- [ ] Beat sheet (visual canvas, drag-and-drop, color coding)
- [ ] Beat-to-scene export
- [ ] Story outline (tree structure)
- [ ] Character catalog (profiles, metadata, images)
- [ ] Scene breakdown (element tagging: props, wardrobe, locations, etc.)
- [ ] Dramatic day tracking
- [ ] Scene summaries

### Phase 4: AI Integration

- [ ] Pumas-Library integration (model discovery, download, selection)
- [ ] llama-cpp-2 inference engine (model loading, context management)
- [ ] AI chat panel (assistant sidebar)
- [ ] Token streaming to UI
- [ ] Writing assistant prompts (scene gen, dialogue polish, plot suggestions)
- [ ] Inline auto-complete (element-type-aware)
- [ ] Character voice consistency checking
- [ ] AI-generated scene summaries

### Phase 5: Export & Import

- [ ] Fountain format (.fountain) export/import
- [ ] PDF export with screenplay formatting
- [ ] Final Draft XML (.fdx) export/import
- [ ] Plain text export
- [ ] HTML export
- [ ] PDF customization (watermarks, headers/footers, revision colors)
- [ ] Print preview

### Phase 6: Revision & Analytics

- [ ] Draft/version management (create, switch, compare)
- [ ] Revision mode with industry-standard color sequence
- [ ] Edit history (undo/redo stack, persistent history)
- [ ] Revision diff view
- [ ] Script statistics (word/page count, action vs. dialogue ratio)
- [ ] Dialogue distribution per character
- [ ] Writing goals (daily word targets, deadline tracking)
- [ ] Settings frequency analysis

### Phase 7: Novel/Prose

- [ ] Prose editor (rich text, chapters, sections)
- [ ] Chapter management (reorder, split, merge)
- [ ] Manuscript formatting options
- [ ] Format conversion (screenplay ↔ novel)
- [ ] Distraction-free writing mode
- [ ] Writing session timer

### Phase 8: Polish & Platform

- [ ] macOS app bundle packaging
- [ ] Windows installer / portable build
- [ ] Linux AppImage or Flatpak
- [ ] Keyboard shortcut system (customizable)
- [ ] Accessibility (screen reader support, high contrast)
- [ ] Localization framework
- [ ] Performance optimization (large scripts, long novels)

---

## Build System

### justfile Recipes

| Recipe | Description |
|---|---|
| `setup` | Install npm dependencies, download CEF binaries, add cross-compilation targets |
| `dev` | Run Vite dev server + Rust app concurrently (hot-reload) |
| `build-ui` | Compile Svelte SPA to `dist/ui/` |
| `build-rust` | `cargo build --release` |
| `build` | `build-ui` then `build-rust` (sequential) |
| `build-windows` | Cross-compile for `x86_64-pc-windows-gnu` |
| `run` | Execute release binary |
| `test` | `cargo test --workspace` |
| `check-ui` | `svelte-check --tsconfig ./tsconfig.json` |
| `check-rust` | `cargo check --workspace` |
| `check` | Both `check-ui` and `check-rust` |
| `lint` | `cargo clippy --workspace -- -D warnings` |
| `fmt` | `cargo fmt --all` |
| `clean` | Remove `target/`, `dist/`, `node_modules/` |

### Vite Configuration

Following Pentimento, output filenames are **deterministic** (no content hashes) so
`embedded_ui.rs` can reliably inline assets:

```typescript
rollupOptions: {
    output: {
        entryFileNames: 'assets/[name].js',
        chunkFileNames: 'assets/[name].js',
        assetFileNames: 'assets/[name].[ext]',
    },
},
```

---

## Key Design Decisions

### 1. Fountain as Internal Format

Scripts are stored in Fountain format internally. Fountain is a plain-text markup language for
screenwriting that is human-readable, diff-friendly, and widely supported. The visual formatting
is applied at render time in the Svelte editor.

**Rationale:** Fountain is the de facto open standard for screenplay interchange. Storing in
Fountain means the project files remain usable outside Eidetic, and version control diffs are
meaningful.

### 2. SQLite for Project Storage

Each project is a directory containing:
- `project.db` — SQLite database with metadata, characters, scenes, beats, settings
- `scripts/` — Fountain files for each script document
- `prose/` — Markdown files for novel chapters
- `assets/` — Images, reference materials

**Rationale:** SQLite provides ACID transactions, full-text search (FTS5), and is a single-file
database. The script content itself stays in plain text files for readability and version control.

### 3. No Real-Time Collaboration (Phase 1-8)

Unlike Celtx's cloud-based collaboration, Eidetic is a local-first application. Real-time
co-writing is explicitly out of scope for the initial roadmap. Future collaboration could
leverage the Fire-Moss P2P library from the MrScripty ecosystem.

### 4. Backend-Owned Data

Per Coding-Standards/ARCHITECTURE-PATTERNS.md, the Rust backend is the single source of truth.
The Svelte frontend displays data received via IPC and sends commands back. It does not
maintain its own copy of project state beyond transient UI state (scroll position, focus,
panel sizes).

### 5. Rust 2024 Edition

Following Pentimento, the workspace uses `edition = "2024"` for the latest Rust features.

---

## Coding Standards Integration

The `standards/` directory contains all 11 documents from MrScripty/Coding-Standards. Key
standards applied to Eidetic:

| Standard | Application |
|---|---|
| **File size limit** | 500 lines max per .rs file |
| **Layered architecture** | core (domain) → ipc (contract) → webview (presentation) → app (orchestration) |
| **Commit format** | `<type>(<scope>): <description>` — feat, fix, refactor, etc. |
| **Testing** | Unit tests mirror source structure; property-based tests for serialization roundtrips |
| **Cross-platform** | Strategy + factory pattern; per-platform files; no scattered cfg |
| **Security** | Validate at boundaries (IPC messages, file paths, user input) |
| **Concurrency** | Message passing over shared state; tokio for async; bounded queues for AI streaming |
| **Interop** | CEF ↔ Rust boundary validated; TypeScript types mirror Rust enums |
| **Dependencies** | Each dep justified; `cargo audit` in CI; lockfile committed |

---

## Open Questions

1. **Script editor implementation:** Build a custom contenteditable-based editor in Svelte, or
   integrate an existing rich text editor (ProseMirror, TipTap, CodeMirror)? ProseMirror with a
   custom schema for script elements is likely the most robust approach.

2. **PDF generation:** `printpdf` (low-level), `typst` (high-level typesetting), or
   `headless-chrome` (render HTML to PDF via CEF)? Typst offers the best balance of control
   and ergonomics for screenplay formatting.

3. **Project file format:** Directory-based (described above) vs. single-file archive (zip/tar)?
   Directory-based is more version-control-friendly but single-file is more portable.

4. **AI prompt architecture:** Fixed prompt templates vs. user-customizable system prompts?
   Start with curated templates, allow advanced users to customize.
