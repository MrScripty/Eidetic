# Script Generation Model

This note captures a needed correction to Eidetic's script model.

## Current Issue

The current implementation treats Beat-level timeline nodes as the place where generated script text lives.

That is probably the wrong domain model.

Timeline nodes/tracks are better understood as temporal context chunks. They represent story, world, character, structural, and planning context that applies over a span of time.

The final screenplay should not be represented as a clip on the timeline. It should be an independent generated artifact that is produced from the overlapping context at each point in time.

## Corrected Mental Model

At any given time, the prompt/instructions for generating screenplay text are the combined context of all active nodes intersecting that point in time.

```text
time T
  active Premise context
  active Act context
  active Sequence context
  active Scene context
  active Beat context
  active story-bible snapshots
  active arc context
  active references/rules
  previous generated script context
  => generated script segment at T
```

The timeline is therefore a layered context model, not the final script document.

## Nodes As Context Chunks

Timeline nodes should store:

- structural intent,
- outline/notes,
- instructions,
- constraints,
- relevant story-bible links,
- arc tags,
- emotional/story function metadata,
- generation hints,
- lock/pinning state for their own context.

Nodes may have text, but that text is context/instructional text. It is not necessarily the final screenplay.

## Script As Generated Artifact

The generated script should be represented separately from timeline nodes.

Possible model:

```text
ScriptDocument
  id
  name
  version
  segments
  created_at
  updated_at
```

```text
ScriptSegment
  id
  time_range
  source_context_hash
  text
  status
  locked
  generated_from_node_ids
  generated_from_bible_snapshot_ids
```

The script exists in time, but it is not itself a timeline clip.

It is an end-product assembled from the context layers.

## Context Query

Generation should start with a context query:

```text
context_at(time_range)
  timeline nodes intersecting range
  story-bible nodes referenced by those timeline nodes
  active story-bible snapshots at range start/end
  arcs active in range
  world rules active in range
  adjacent generated script segments
  references selected for the range
```

For a point in time:

```text
context_at(T) = all context nodes where node.start <= T < node.end
```

For a segment:

```text
context_for_range(start, end) =
  intersecting context nodes
  plus preceding/following continuity window
```

## Script Generation Flow

The high-level flow should become:

1. Select a script time range or generation region.
2. Gather intersecting context nodes and active bible/world state.
3. Build a prompt from layered context.
4. Generate screenplay text into a `ScriptSegment`.
5. Store provenance: which context nodes and snapshots contributed.
6. Render the script document from ordered script segments.
7. Allow user edits on script segments without mutating timeline context nodes.
8. Optionally propagate edits back into context nodes as suggestions, not automatic overwrites.

## Editing Model

User edits should apply to the script artifact, not directly to timeline nodes.

Editing a generated script segment should:

- update the `ScriptSegment.text`,
- mark the segment as user-edited or locked if needed,
- preserve provenance to the context used for generation,
- trigger consistency analysis against downstream script/context,
- optionally suggest updates to timeline context or bible state.

This avoids conflating:

```text
story planning context
```

with:

```text
final screenplay prose/dialogue
```

Manual edits must become first-class state. They are not incidental text changes inside a generated blob.

The script should preserve authorship and edit protection at a finer level than a whole segment:

```text
ScriptSegment
  blocks

ScriptBlock
  id
  kind
  spans

ScriptSpan
  id
  text
  author: ai | user
  lock_state
  pinned_facts
  source_context_ids
```

Lock states should support:

```text
Unlocked
  AI may replace freely.

UserEdited
  AI should preserve wording unless explicitly asked to revise.

HardLocked
  AI cannot change this range.

PinnedFact
  Exact wording may change, but the fact must remain true.
```

Regeneration should operate as a constrained patch over script blocks/spans, not as blind full replacement.

Regeneration input should include:

- current script segment text,
- protected spans,
- pinned facts,
- relevant timeline/bible/reference context,
- accepted semantic edits,
- instruction to revise only unlocked ranges.

Regeneration output should be a patch:

```text
replace block
insert after block
delete unlocked block
leave protected span unchanged
```

User-authored or hard-locked spans should survive regeneration unless the user explicitly permits replacement.

## Semantic Edit Analysis

When the user edits generated script, the system should analyze whether the edit implies a change to story/world state.

Example:

```text
Bible says: Beach House weather = sunny
User script edit: Rain lashes the windows.
```

The system should extract a structured claim:

```text
entity: Beach House
property: weather
old/context value: sunny
new/script value: rainy
time_scope: current segment | scene | sequence | global
confidence: high
```

The app should not automatically update the bible. It should present interpretations:

```text
Local script detail only
Update this scene/sequence context
Add timed bible snapshot
Change global bible fact
Mark as intentional contradiction
```

Accepted interpretations become semantic change events and may update the bible graph, timeline context, script segment metadata, or all three.

## Semantic Dependencies

To know what a manual edit affects, script segments need semantic dependency tracking.

A script segment may depend on:

```text
timeline_node
bible_node
bible_part
bible_edge
bible_snapshot
story_arc
reference_document
previous_script_segment
semantic_claim
```

When a fact changes, the system should find affected script segments by:

- source/provenance links,
- direct entity mentions,
- bible graph edges,
- timeline time overlap,
- story arc overlap,
- downstream causal/story relationships,
- extracted semantic claims.

Affected segments should be marked with review states rather than automatically rewritten:

```text
Stale
NeedsReview
RegenerateSuggested
ContradictionPossible
```

This creates the desired loop:

```text
context -> generate script -> user edits script -> analyze edit
  -> update context or mark exception -> identify affected script
  -> selectively regenerate or review
```

Propagation should not be an opaque automatic rewrite. It should be a staged change transaction:

```text
1. user edits script
2. AI extracts semantic claims
3. AI proposes bible/timeline/script changes
4. app shows before/after diff for each proposed change
5. user accepts, edits, rejects, or gives refinement instructions
6. accepted changes commit as traceable events
7. downstream script segments are marked stale or regenerated as additional proposed changes
```

The user should be able to stop the process at any stage. If the AI proposes the wrong bible update, the user can reject it, edit the proposed trait/scope, or provide instruction before propagation continues.

## Traceable Change History

Every meaningful edit should be represented as a traceable change event.

This includes:

- user text edits,
- AI generation,
- AI regeneration,
- accepted semantic edit interpretations,
- bible graph changes,
- timeline context changes,
- propagation/regeneration actions.

Example event shape:

```text
ChangeEvent
  id
  parent_event_ids
  author
  timestamp
  kind
  reason
  affected_objects
  before_refs
  after_refs
  semantic_claims
```

The system should be able to answer:

```text
Why did this line change?
Which edit caused this later regeneration?
What did the script look like before this change?
What did the bible graph look like before this change?
Which facts were introduced by this edit?
Which later segments depend on that fact?
```

State over time should be queryable for both script and bible:

```text
script_state_at(change_event_id)
bible_graph_state_at(change_event_id)
diff_script(before_event, after_event)
diff_bible_graph(before_event, after_event)
```

This state should be reconstructed from sparse per-object revisions. A change event should only write revisions for objects that actually changed: one script block, one script segment, one bible part, one semantic claim, one dependency edge, etc.

The system should not store a full copy of the script or full bible graph for every event. Whole-document or whole-graph snapshots may exist later as optional checkpoints for fast loading, but they should be derived cache data. Backend-owned event history plus object-level revisions should remain the canonical durable history.

The history model should support branching or drafts later, but the first requirement is linear traceability across script and bible changes.

## Reviewable Propagation And Undo

AI propagation should be modeled as a chain of small events rather than one large event.

Example:

```text
event_100 user_script_edit
event_101 semantic_claim_proposed        parent: event_100
event_102 bible_snapshot_proposed        parent: event_101
event_103 bible_snapshot_accepted        parent: event_102
event_104 script_segment_regen_proposed  parent: event_103
event_105 script_segment_regen_accepted  parent: event_104
```

Each proposed event can be previewed before it becomes accepted project state. The preview should show:

- the script text before and after,
- the bible graph fields before and after,
- which semantic claim caused the change,
- which script segments are affected,
- which dependency edges explain the impact.

Undo and redo should operate at the same event-chain level:

```text
undo event_105
  restores previous script segment revision

undo event_103
  restores previous bible snapshot/field revision
  marks dependent script segments stale again

redo event_103
  reapplies the accepted bible change
  reopens or replays dependent propagation proposals
```

Because revisions store typed old and new field values, undo does not require restoring a full database snapshot. It applies the inverse of the accepted object revisions for that event. Redo reapplies the same revisions if the base revision still matches, or asks for review if later edits conflict.

The user should also be able to branch from a previous event. This allows a workflow like:

```text
inspect change chain
go back to before AI bible update
add refinement instruction
rerun semantic analysis/propagation
compare new branch against old branch
accept the preferred result
```

Branching can be implemented later, but the event model should be designed so it is not blocked.

## Display Model

The formatted screenplay view should render the script document, not Beat node content.

The script renderer should consume:

```text
ScriptDocument.segments ordered by time_range.start
```

not:

```text
Timeline.nodes where level == Beat
```

The timeline can still show generation coverage:

- which ranges have generated script,
- which ranges are stale relative to context changes,
- which ranges are locked,
- which script segments need regeneration,
- which context nodes contributed to a selected script segment.

## Staleness And Provenance

Each generated script segment should record a context hash or revision set.

When a contributing timeline node, bible node, edge, part, snapshot, or reference changes, affected script segments can be marked stale.

Possible states:

```text
Draft
Generated
UserEdited
Locked
Stale
Regenerating
NeedsReview
```

This lets the app regenerate only affected script ranges instead of treating every beat as a script owner.

## Relationship To Current Beat Content

No backwards compatibility with the current Beat-owned script content is required.

The target model should replace the current behavior cleanly:

- remove final screenplay text from `StoryNode.content.content`,
- keep timeline node text scoped to context, notes, instructions, or constraints,
- store generated screenplay text only in `ScriptDocument` / `ScriptSegment`,
- use timeline node IDs only as generation sources/provenance.

## Persistence Implications

The project database should gain script artifact tables.

Possible tables:

```sql
script_documents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  version INTEGER NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

script_segments (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  start_ms INTEGER NOT NULL,
  end_ms INTEGER NOT NULL,
  text TEXT NOT NULL,
  status TEXT NOT NULL,
  locked INTEGER NOT NULL DEFAULT 0,
  source_context_hash TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (document_id) REFERENCES script_documents(id)
);

script_segment_sources (
  segment_id TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_id TEXT NOT NULL,
  PRIMARY KEY (segment_id, source_kind, source_id)
);

change_events (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  author TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  reason TEXT NOT NULL DEFAULT ''
);

change_event_parents (
  event_id TEXT NOT NULL,
  parent_event_id TEXT NOT NULL,
  PRIMARY KEY (event_id, parent_event_id)
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
```

JSON should not be used for canonical script or semantic-history facts that need indexing, partial updates, or impact analysis. Flexible metadata can still use JSON as an escape hatch, but the core history model should be relational rows: changed field names, typed old values, typed new values, scopes, refs, and asset links.

Source kinds might include:

```text
timeline_node
bible_node
bible_part
bible_edge
bible_snapshot
story_arc
reference_document
previous_script_segment
```

## Open Questions

- What is the default script segmentation unit: beat, scene, arbitrary range, or generated chunk?
- Should one project support multiple script documents or drafts?
- Should script segments be CRDT-backed?
- How should manual script edits feed back into the timeline context model?
- How should generated script handle overlaps where context changes mid-segment?
- Should final export use script segments directly or reparse the rendered document?
- Should locked script segments block regeneration even if source context changes?

## Implementation Direction

Standards compliance requirements:

- The generated screenplay is backend-owned canonical state in `ScriptDocument` tables. Timeline nodes, Svelte stores, Y.Doc buffers, and Bevy projections are not canonical script stores.
- Manual script edits must enter through validated script commands that target stable document, segment, block, or span IDs.
- Regeneration must produce reviewable `ScriptPatchProposal` records and must preserve hard-locked/user-authored spans unless the user explicitly permits replacement.
- Script changes, semantic claims, bible updates, and propagation effects must be traceable through transactional change events and sparse object revisions.
- Y.Doc, if retained, is an active editing transport/cache only. Accepted Y.Doc changes must become explicit script commands/events.
- Script blocks, spans, locks, claims, dependencies, and revisions must be stored as queryable relational rows rather than canonical JSON blobs.
- Export must consume an `ExportProjection` from `ScriptDocument`, not Beat node content.
- AI routes must not mutate script state directly; they emit proposals that are accepted or rejected through the command layer.
- Tests must cover patch conflict handling, locked span preservation, semantic claim proposal flow, propagation review, undo/redo, export projection, restart recovery, and command idempotency.
- Any touched `src/` script, editor, export, AI, or route directories must keep README contracts current.

Recommended path:

1. Define `ScriptDocument` and `ScriptSegment` as the only final-script storage model.
2. Remove script ownership from timeline nodes.
3. Add persistence for script documents and segments.
4. Add a context query that gathers all active timeline/bible/reference context for a time range.
5. Generate script into script segments instead of node content.
6. Render formatted screenplay from script segments.
7. Add staleness/provenance tracking.
8. Delete the Beat-based script display once the segment renderer exists.
