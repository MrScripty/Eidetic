# Story Bible Worldbuilding Plan

This note captures the planning direction for expanding Eidetic's story bible from a screenplay-continuity entity list into a broader world-development system.

## Current Model

The current story bible stores `Entity` records with these categories:

```text
Character
Location
Prop
Theme
Event
```

Each entity has a name, tagline, description, category-specific details, timeline snapshots, node references, relations, color, and lock state.

This is a good baseline for screenplay continuity, but it is too narrow for deeper world development.

## Target Direction

The bible should support both:

- screenplay continuity: who, where, what object, what state, what changed,
- world development: societies, organizations, belief systems, rules, history, motifs, and relationships that shape the story.

The story bible should be treated as one of the core backend-owned canonical domains used by AI planning and generation.

The target model should be composable rather than only a larger fixed enum. Users should be able to read, write, add, and restructure individual pieces of the bible without loading or replacing an entire entity.

## Proposed Data Model

The story bible should become a weakly typed graph with composable nodes:

```text
StoryBible
  canonical roots
  nodes
  edges
  snapshots
  assets
```

Core concepts:

- `BibleNode`: a thing in the story/world.
- `BibleNode` children: traits, details, notes, sub-details, and arbitrary
  user-created facets attached to any node.
- `BibleEdge`: a relationship between nodes.
- `BibleSchema`: a weak category/template hint for default names, colors,
  creation behavior, and AI prompting. Schemas do not limit what children a node
  may have.
- `BibleSnapshot`: a time-specific state or override for a node or edge.

In short:

```text
Nodes are things, traits, and details.
Edges are relationships between things.
Snapshots are time-specific versions of nodes or edges.
Schemas describe what a type usually means, not what it may contain.
```

Fixed field/part records should be phased out as canonical worldbuilding state.
Canonical character traits such as motivation, appearance, voice, and backstory
should be represented as ordinary child nodes when they are useful, not as
required fields on a character record.

## Canonical Graph

The graph should be composable, but every project should still start with a canonical backbone. The canonical graph gives the UI predictable navigation and gives AI/context retrieval stable indexing anchors.

Canonical roots should be normal `BibleNode` records marked as system-owned:

```text
BibleNode
  id: canon.characters
  type_id: CanonSection
  title: Characters
  system: true
```

Expected canonical roots:

```text
World
  Cultures
  Organizations
  Locations
  History
  Rules

Story
  Characters
  Relationships
  Events
  Themes
  Motifs
  SetPieces

Production
  Objects
  Continuity
  References
```

The same roots can also be exposed as a flatter index when useful:

```text
Characters
Locations
Objects
Organizations
Cultures
Events
Themes
WorldRules
Motifs
Relationships
SetPieces
References
```

Canonical roots should be:

- created for every project,
- ordinary graph nodes with stable IDs,
- hidden or collapsed when the user does not need them,
- protected from casual deletion,
- usable as default parents for new nodes,
- treated as indexing hints rather than hard ontology limits.

Examples:

```text
Characters
  Sarah
  Marcus

Cultures
  Glass Coast

Organizations
  Ministry of Weather

Rules
  Weather cannot be predicted, only negotiated
```

Custom roots and custom schemas should still be allowed. The canonical graph is the default organization layer, not the maximum allowed structure.

## Bible Nodes

A node is the stable identity of a bible entry.

Example shape:

```text
BibleNode
  id
  type_id
  title
  summary
  parent_id
  aliases
  tags
  importance
  node_refs
  locked
```

`parent_id` supports tree-style world organization:

```text
Empire of Lume
  Ministry of Weather
    Storm Audit Office
```

Parent/child structure should be easy to browse and edit, but the bible should not be limited to trees. Richer relationships belong in edges.

## Bible Detail Nodes

Detail nodes let the system read and write only the specific facet it needs
while keeping the bible graph open-ended.

Example shape:

```text
BibleNode
  id
  parent_id
  schema_key: detail | custom schema key
  title
  body/content
  sort_order
```

Examples:

```text
Character node: Sarah
  Identity              <- detail node
  Traits                <- detail node
    Stubborn            <- nested detail node
  Voice                 <- detail node
  Goals                 <- detail node
  Secrets               <- detail node
  CurrentState          <- detail node

Culture node: Glass Coast
  Values
  Taboos
  Rituals
  Language
  History
  PowerStructure

Organization node: Ministry of Weather
  Mission
  Hierarchy
  Resources
  PublicFace
  Secrets
```

This enables precise operations such as:

```text
read node Sarah > Voice
write node Sarah > CurrentState
add node Glass Coast > Rituals
read node Ministry of Weather > Hierarchy
```

The UI and AI should not need to fetch or rewrite a whole bible entry when only
one detail node is relevant. Users may add arbitrary detail nodes at any depth;
canonical traits are optional scaffold nodes, not hard fields.

## Bible Edges

Edges represent graph relationships between nodes.

Example shape:

```text
BibleEdge
  id
  from_node_id
  to_node_id
  kind
  label
  fields
  snapshots
```

Example edge kinds:

```text
child_of
member_of
located_in
owns
created_by
opposes
allied_with
governs
believes
caused_by
symbolizes
constrains
```

This allows the same node to participate in multiple structures:

```text
Sarah -> member_of -> Ministry of Weather
Sarah -> opposes -> Storm Audit Office
Sarah -> owns -> Glass Compass
Glass Compass -> symbolizes -> Forbidden Navigation
```

## State Over Time And History

The bible graph needs both story-time state and edit-history state.

Story-time state answers:

```text
What is true at 12:30 in the story?
Where is this character during this sequence?
What does the audience know at this point?
What is the current state of this location, object, organization, or relationship?
```

Edit-history state answers:

```text
Who changed this fact?
Which script edit introduced this fact?
What did the bible graph look like before and after that change?
Which later script segments depend on this change?
```

`BibleSnapshot` handles story-time state. A separate traceable change history should handle edit-history state.

Every bible node, part, edge, snapshot, and semantic claim should be traceable to the change event that created or modified it.

Example:

```text
User edits final script:
  "Rain lashes the windows."

Semantic analysis proposes:
  Beach House weather = rainy during Scene 12

User accepts:
  Add timed bible snapshot for Beach House

History stores:
  script edit event
  semantic claim event
  bible snapshot creation event
  affected script segment review/regeneration events
```

This lets the app inspect state before and after the edit:

```text
bible_graph_state_at(event_before_rain_change)
bible_graph_state_at(event_after_rain_change)
script_state_at(event_before_rain_change)
script_state_at(event_after_rain_change)
```

Graph state should be stored as sparse per-object revisions, not repeated full-graph snapshots.

Each changed object gets its own revision:

```text
bible_node revision
bible_part revision
bible_edge revision
bible_snapshot revision
semantic_claim revision
script_segment revision
```

If a change only edits one location's weather snapshot, only that snapshot object and the associated semantic claim need new revisions. Characters, unrelated places, culture notes, relationship edges, and other unchanged objects should not be copied.

Full graph state at an event is reconstructed by reading the latest revision for each relevant object at or before that event. For performance, the system can later add periodic checkpoints or materialized indexes, but those are cache artifacts. The canonical history should remain delta-based.

AI-driven graph updates should be reviewable before they become accepted state. A script edit can produce proposed semantic claims, proposed bible field changes, proposed snapshots, and proposed regeneration work. The user should be able to inspect each proposed change, adjust the scope or value, reject it, or add instructions before propagation continues.

Example staged flow:

```text
user_script_edit
  -> semantic_claim_proposed
  -> bible_snapshot_proposed
  -> bible_snapshot_accepted
  -> affected_script_segments_marked
  -> regeneration_proposed
  -> regeneration_accepted
```

The graph history should allow before/after review at each stage:

```text
show bible node before proposed snapshot
show bible node after proposed snapshot
show script before regeneration
show script after regeneration
show dependency path from original user edit to final script change
```

Undo/redo should apply to accepted change events by reversing or replaying only the object revisions attached to those events. Rejecting or undoing an AI bible update should leave the original user script edit intact unless the user explicitly undoes that edit too.

The graph should support semantic dependency edges for impact analysis:

```text
script_segment -> depends_on -> bible_part
script_segment -> depends_on -> bible_snapshot
script_segment -> mentions -> bible_node
semantic_claim -> about -> bible_node
change_event -> caused -> change_event
change_event -> invalidated -> script_segment
```

These semantic relations are separate from story-world relations. They explain generation and revision history rather than in-world narrative facts.

## Storage Model

The bible graph should be saved in the existing per-project SQLite database. SQLite is a good fit for local-first project data: it is portable, transactional, inspectable, and already used by Eidetic for project persistence.

The graph maps cleanly to relational tables. JSON should not be the primary storage format for facts that need to be queried, indexed, diffed, or partially updated.

Use SQLite tables for the canonical graph:

```text
bible_schema_versions
bible_type_defs
bible_field_defs
bible_nodes
bible_node_aliases
bible_node_tags
bible_parts
bible_part_fields
bible_edges
bible_edge_fields
bible_snapshots
bible_snapshot_fields
change_events
object_revisions
object_revision_fields
semantic_claims
semantic_claim_values
semantic_claim_scopes
semantic_dependencies
```

Recommended table shape:

```sql
bible_schema_versions (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  version INTEGER NOT NULL,
  system INTEGER NOT NULL DEFAULT 0
);

bible_type_defs (
  id TEXT PRIMARY KEY,
  schema_version_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  name TEXT NOT NULL,
  parent_type_id TEXT,
  system INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (schema_version_id) REFERENCES bible_schema_versions(id)
);

bible_field_defs (
  id TEXT PRIMARY KEY,
  type_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  value_type TEXT NOT NULL,
  required INTEGER NOT NULL DEFAULT 0,
  repeatable INTEGER NOT NULL DEFAULT 0,
  indexed INTEGER NOT NULL DEFAULT 0,
  sort_order INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (type_id) REFERENCES bible_type_defs(id)
);

bible_nodes (
  id TEXT PRIMARY KEY,
  type_id TEXT NOT NULL,
  parent_id TEXT,
  title TEXT NOT NULL,
  summary TEXT NOT NULL DEFAULT '',
  importance INTEGER NOT NULL DEFAULT 0,
  system INTEGER NOT NULL DEFAULT 0,
  locked INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (parent_id) REFERENCES bible_nodes(id)
);

bible_node_aliases (
  node_id TEXT NOT NULL,
  alias TEXT NOT NULL,
  PRIMARY KEY (node_id, alias),
  FOREIGN KEY (node_id) REFERENCES bible_nodes(id) ON DELETE CASCADE
);

bible_node_tags (
  node_id TEXT NOT NULL,
  tag TEXT NOT NULL,
  PRIMARY KEY (node_id, tag),
  FOREIGN KEY (node_id) REFERENCES bible_nodes(id) ON DELETE CASCADE
);

bible_parts (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (node_id, kind),
  FOREIGN KEY (node_id) REFERENCES bible_nodes(id) ON DELETE CASCADE
);

bible_part_fields (
  part_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  value_type TEXT NOT NULL,
  value_text TEXT,
  value_number REAL,
  value_integer INTEGER,
  value_bool INTEGER,
  ref_kind TEXT,
  ref_id TEXT,
  asset_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (part_id, field_key, sort_order),
  FOREIGN KEY (part_id) REFERENCES bible_parts(id) ON DELETE CASCADE
);

bible_edges (
  id TEXT PRIMARY KEY,
  from_node_id TEXT NOT NULL,
  to_node_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  label TEXT NOT NULL DEFAULT '',
  locked INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (from_node_id) REFERENCES bible_nodes(id),
  FOREIGN KEY (to_node_id) REFERENCES bible_nodes(id)
);

bible_edge_fields (
  edge_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  value_type TEXT NOT NULL,
  value_text TEXT,
  value_number REAL,
  value_integer INTEGER,
  value_bool INTEGER,
  ref_kind TEXT,
  ref_id TEXT,
  asset_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (edge_id, field_key, sort_order),
  FOREIGN KEY (edge_id) REFERENCES bible_edges(id) ON DELETE CASCADE
);

bible_snapshots (
  id TEXT PRIMARY KEY,
  target_kind TEXT NOT NULL,
  node_id TEXT,
  part_kind TEXT,
  edge_id TEXT,
  at_ms INTEGER NOT NULL,
  description TEXT NOT NULL
);

bible_snapshot_fields (
  snapshot_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  value_type TEXT NOT NULL,
  value_text TEXT,
  value_number REAL,
  value_integer INTEGER,
  value_bool INTEGER,
  ref_kind TEXT,
  ref_id TEXT,
  asset_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (snapshot_id, field_key, sort_order),
  FOREIGN KEY (snapshot_id) REFERENCES bible_snapshots(id) ON DELETE CASCADE
);

change_events (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  author TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT ''
);

object_revisions (
  id TEXT PRIMARY KEY,
  object_kind TEXT NOT NULL,
  object_id TEXT NOT NULL,
  change_event_id TEXT NOT NULL,
  base_revision_id TEXT,
  operation TEXT NOT NULL,
  created_at TEXT NOT NULL
);

object_revision_fields (
  revision_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  value_type TEXT NOT NULL,
  old_value_text TEXT,
  old_value_number REAL,
  old_value_integer INTEGER,
  old_value_bool INTEGER,
  new_value_text TEXT,
  new_value_number REAL,
  new_value_integer INTEGER,
  new_value_bool INTEGER,
  ref_kind TEXT,
  ref_id TEXT,
  asset_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (revision_id, field_key, sort_order),
  FOREIGN KEY (revision_id) REFERENCES object_revisions(id) ON DELETE CASCADE
);

semantic_claims (
  id TEXT PRIMARY KEY,
  change_event_id TEXT NOT NULL,
  subject_kind TEXT NOT NULL,
  subject_id TEXT NOT NULL,
  predicate TEXT NOT NULL,
  confidence REAL NOT NULL
);

semantic_claim_values (
  claim_id TEXT NOT NULL,
  value_type TEXT NOT NULL,
  value_text TEXT,
  value_number REAL,
  value_integer INTEGER,
  value_bool INTEGER,
  ref_kind TEXT,
  ref_id TEXT,
  asset_id TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (claim_id, sort_order),
  FOREIGN KEY (claim_id) REFERENCES semantic_claims(id) ON DELETE CASCADE
);

semantic_claim_scopes (
  claim_id TEXT NOT NULL,
  scope_kind TEXT NOT NULL,
  scope_id TEXT,
  start_ms INTEGER,
  end_ms INTEGER,
  PRIMARY KEY (claim_id, scope_kind, scope_id, start_ms, end_ms),
  FOREIGN KEY (claim_id) REFERENCES semantic_claims(id) ON DELETE CASCADE
);

semantic_dependencies (
  source_kind TEXT NOT NULL,
  source_id TEXT NOT NULL,
  target_kind TEXT NOT NULL,
  target_id TEXT NOT NULL,
  relation TEXT NOT NULL,
  confidence REAL NOT NULL DEFAULT 1.0,
  PRIMARY KEY (source_kind, source_id, target_kind, target_id, relation)
);
```

JSON can still be used as an escape hatch for data that is rarely queried and does not need partial updates, such as plugin-private display preferences or imported reference metadata. It should not be used for canonical bible facts, semantic claims, dependency edges, or revision history.

Granular part reads/writes become direct SQL operations:

```sql
SELECT field_key, value_type, value_text, value_number, value_integer, value_bool, ref_kind, ref_id, asset_id
FROM bible_part_fields
WHERE part_id = ?;
```

```sql
INSERT INTO bible_part_fields
  (part_id, field_key, value_type, value_text, sort_order)
VALUES
  (?, ?, 'text', ?, 0)
ON CONFLICT(part_id, field_key, sort_order)
DO UPDATE SET value_text = excluded.value_text,
              value_type = excluded.value_type;
```

Reading only one trait does not require loading the whole part:

```sql
SELECT value_text
FROM bible_part_fields
WHERE part_id = ? AND field_key = 'weather';
```

Finding all rainy locations is an indexed relational query:

```sql
SELECT p.node_id
FROM bible_parts
JOIN bible_part_fields f ON f.part_id = p.id
WHERE p.kind = 'location_state'
  AND f.field_key = 'weather'
  AND f.value_text = 'rainy';
```

## Asset Storage

The bible should support media and reference materials such as images, audio clips, URLs, documents, and generated assets.

Large binary assets should live on disk beside the project database, not inside the main graph tables:

```text
project.db
assets/
  images/
  audio/
  video/
  documents/
  generated/
```

SQLite should store asset metadata and references:

```sql
bible_assets (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  mime_type TEXT,
  local_path TEXT,
  source_url TEXT,
  content_hash TEXT
);

bible_asset_fields (
  asset_id TEXT NOT NULL,
  field_key TEXT NOT NULL,
  value_type TEXT NOT NULL,
  value_text TEXT,
  value_number REAL,
  value_integer INTEGER,
  value_bool INTEGER,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (asset_id, field_key, sort_order),
  FOREIGN KEY (asset_id) REFERENCES bible_assets(id) ON DELETE CASCADE
);

bible_asset_refs (
  asset_id TEXT NOT NULL,
  target_kind TEXT NOT NULL,
  node_id TEXT,
  part_kind TEXT,
  edge_id TEXT,
  snapshot_id TEXT,
  role TEXT NOT NULL DEFAULT ''
);
```

Examples:

```text
Culture: Glass Coast
  image reference: costume board
  audio reference: dialect sample
  URL reference: research source
  document reference: mythology notes
```

Small generated thumbnails or previews may be stored as files or, where useful, small SQLite BLOBs. Full-size images, audio, video, and documents should default to files so project databases stay manageable.

Recommended principle:

```text
SQLite stores canonical graph data, structured parts, metadata, and links.
The filesystem stores large binary assets.
Asset reference tables connect media to nodes, parts, edges, and snapshots.
```

## Default Schemas

The app should ship with strong default schemas, but users should be able to add custom types or attach additional parts.

Recommended built-in node types:

```text
Character
Location
Object
Organization
Culture
Event
Theme
WorldRule
Motif
RelationshipDynamic
SetPiece
```

`Prop` should likely become `Object`. Props are production-facing; objects are story/world-facing and can include weapons, artifacts, documents, vehicles, tools, technology, and symbolic items.

These should be default schemas, not hard limits.

## Default Schema Intent

`Character`
People or person-like agents. Should track traits, voice, goals, needs, flaws, secrets, wounds, values, social role, and current state.

`Location`
Physical places. Should track geography, scene-heading name, atmosphere, rules of access, history, social meaning, and recurring visual/tonal qualities.

`Object`
Meaningful items. Should track owner, origin, function, significance, constraints, transformations, and symbolic value.

`Organization`
Groups with agency: families, governments, companies, schools, gangs, religions, teams, military units, guilds, studios, institutions.

`Culture`
Shared norms, customs, language, taboos, rituals, aesthetics, class markers, moral assumptions, historical memory, and internal contradictions.

`Event`
Timeline events, backstory, historical incidents, public turning points, personal turning points, and offscreen causes.

`Theme`
Abstract ideas the story tests. Should track dramatic argument, counterargument, recurring situations, and how it manifests in scenes.

`WorldRule`
Rules governing the setting: magic systems, technology constraints, laws, economics, politics, social rules, genre rules, supernatural limits, or procedural rules.

`Motif`
Recurring images, jokes, phrases, sounds, symbols, visual language, or repeated story patterns.

`RelationshipDynamic`
A first-class relationship entry between entities. This should track power balance, emotional temperature, trust, conflict, dependency, secrets, and how the dynamic changes over time.

`SetPiece`
Major memorable sequences or centerpiece moments. Useful for action, horror, comedy, mystery reveals, musical numbers, elaborate cons, battles, or big emotional confrontations.

## Composability Requirements

The bible should support:

- adding a new node,
- adding a child node under another node,
- adding an edge between existing nodes,
- adding a new part to an existing node,
- reading one node part,
- writing one node part,
- reading only edges of a specific kind,
- reading the local graph around a node,
- adding a custom schema/type,
- adding custom fields to a part without changing core code.

Possible API shape:

```text
GET    /bible/nodes/{id}
POST   /bible/nodes
PATCH  /bible/nodes/{id}
GET    /bible/nodes/{id}/children
GET    /bible/nodes/{id}/parts/{kind}
PUT    /bible/nodes/{id}/parts/{kind}
PATCH  /bible/nodes/{id}/parts/{kind}
POST   /bible/nodes/{id}/parts
GET    /bible/nodes/{id}/edges
POST   /bible/edges
PATCH  /bible/edges/{id}
GET    /bible/graph?center={id}&depth=2
```

The important point is granular read/write behavior. A character voice edit should not require replacing the entire character. A culture-taboo query should not require loading every culture detail.

## Cross-Cutting Fields

Most bible nodes should share:

- `name`
- `type_id`
- `tagline`
- `description`
- `aliases`
- `tags`
- `status`
- `importance`
- `node_refs`
- `relations`
- `snapshots`
- `locked`

Some of these may remain direct node fields, while deeper structured information should move into parts.

The current snapshot model should remain central. Worldbuilding entries also change over story time: a culture can be revealed differently, an organization can fracture, a law can be broken, an object can change owners, and a relationship can transform.

## Relationship Model

The current relation model is a simple labeled edge:

```text
source entity -> label -> target entity
```

That should remain useful for lightweight links, but important relationships need their own structured entries.

Example relationship dynamic fields:

- participants
- relationship type
- current status
- power balance
- emotional state
- conflict
- trust level
- secrets
- public vs private understanding
- history
- snapshot changes over time

This matters because relationships are often the story engine, not just metadata.

## AI Context Behavior

Bible context should be selected by relevance, not dumped wholesale.

Priority order should roughly be:

1. entries directly referenced by the target node,
2. active snapshots for those entries at the target time,
3. relationship dynamics between referenced entries,
4. world rules that constrain the node,
5. organizations/cultures/locations active in the node,
6. motifs/themes connected to the active arc,
7. nearby or recently changed entries.

The prompt should distinguish between:

- facts that must not be contradicted,
- current-state context,
- optional flavor,
- thematic guidance,
- unresolved questions.

The graph model should let AI request context by slice:

```text
node summary
specific part
active snapshot
local edges
ancestor/child tree
relationship path between two nodes
world rules constraining this scene
```

This is especially important for large projects where the full bible may be much larger than the context window.

## Implementation Notes

No backwards compatibility with the current `StoryBible { entities }` schema is required. Implement the graph model as the new backend-owned canonical model rather than layering it beside the current entity model.

Standards compliance requirements:

- The bible graph is backend-owned canonical state. UI and Bevy graph views consume projections and submit commands only.
- `Entity`, `EntityCategory`, `EntityDetails`, and old entity snapshot APIs should be deleted when graph equivalents become active.
- Graph commands must validate node IDs, schema IDs, field types, edge endpoints, snapshot ranges, asset refs, and URL refs at the backend boundary.
- Canonical facts, claims, dependencies, locks, revisions, and queryable fields must be relational rows, not JSON blobs.
- JSON is allowed only for non-canonical metadata that does not need indexing, partial updates, dependency tracing, or undo/redo.
- Asset files must live under validated project asset roots; SQLite stores metadata, hashes, refs, and provenance.
- AI extraction must create reviewable semantic claim and graph-change proposals. It must not commit bible state directly.
- Graph updates must write transactional change events and sparse object revisions.
- Tests must cover graph invariants, schema validation, per-field updates, snapshot scoping, dependency lookup, undo/redo, recovery, and idempotency.
- Schema/UI implementation must be decomposed into focused modules and components rather than expanding fixed entity forms.
- Any touched `src/` story, route, store, or bible UI directories must keep README contracts current.

Recommended implementation path:

1. Replace `StoryBible { entities }` with graph-shaped bible types.
2. Replace fixed `EntityCategory` / `EntityDetails` with schemas, nodes, parts, edges, snapshots, and assets.
3. Use `Object` as the built-in story/world type instead of `Prop`.
4. Store all relationships as `BibleEdge` records.
5. Store structured category-specific data as composable `BiblePart` records.
6. Allow snapshots to target nodes, parts, or edges.
7. Add granular route operations for node, part, edge, and local graph reads/writes.
8. Rebuild the bible UI around schemas, canonical roots, nodes, parts, and graph navigation.
9. Add persistence tests for the new graph schema.
10. Update AI extraction prompts to discover organizations, cultures, world rules, motifs, and relationship dynamics.
11. Update context packing so large worldbuilding data does not overwhelm scene/beat prompts.

## Open Design Questions

- Should `RelationshipDynamic` be a node schema, an edge schema, or both?
- Should `WorldRule` include strictness levels such as hard rule, soft rule, rumor, belief, and exception?
- Should `Culture` and `Organization` share some fields for hierarchy, membership, values, and internal factions?
- Should `SetPiece` live in the bible, the timeline, or both?
- Should a project have separate bibles for story, world, production, and research, or one unified bible with filters?
- Should custom schemas be project-local only, or should they be reusable presets?
- Should parent/child be stored as `parent_id`, as a `child_of` edge, or both with one canonical source?
- How should schema validation work for custom parts while still allowing freeform worldbuilding?

## Current Implementation Touchpoints

Expected implementation areas:

- `crates/core/src/story/bible.rs`
- `crates/server/src/persistence.rs`
- `crates/server/src/routes/story.rs`
- `crates/server/src/routes/ai.rs`
- `crates/server/src/routes/timeline.rs`
- `ui/src/lib/types.ts`
- `ui/src/lib/components/sidebar/bible/`
- `ui/src/lib/components/relationship/`
- AI prompt formatting and extraction prompts

The expansion should be done as a coordinated schema and UI change, not as isolated enum additions.
