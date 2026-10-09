# Explicit one-hop Bible recall

## Admission and ownership

Branch `feat/explicit-bible-recall` starts from verified merged main
`8000f29e96517b1862f9cb091150a315c1f351ca`, tree
`f096352611b570f9c31617fcc4a97887874fac0d`. PR17 and PR18 source, qualifier
and evidence branches remain separate. Parent owns review, PRs and merges.

Integration preserves normal ancestry from accepted main
`c4587c11911af355c2446d0befa5ac1cbde8f3e3` (parents8000f29 and aa142712),
retaining consumed arc memory alongside this explicit recall read. Conflicting
README additions are both retained; the application merges without conflicts.

The authoring plan requests relevance-based slices and relationship paths
(`story-bible-worldbuilding/plan.md`, AI Context Behavior). Current
ReadBibleNeighborhood calls the renderer query: selected IDs are required,
default candidates are ordered prefixes, and edges are read between already
selected nodes. Depth controls hierarchy descendants for a focused root, not
relationship expansion. Generation shares this bounded renderer selection.
Existing consumed field/name/relationship receipts are complete for their
supplied inputs, but do not establish retrieval of omitted connected entities.
The renderer deliberately preserves the default graph with selection; its
behavior must remain unchanged.

Research: Puma, Knowledge Graphs and Agentic Memory, research edition
2 October 2026, chapter 6 (bounded typed graph expansion and coverage), chapter
11 (compact exact-source evidence and explicit omissions). Chapter 13's
pre-resolver Eidetic audit predates current temporal resolution. No embedding
dependency is needed for exact-ID relationship lookup.

## Exact read contract

- Input: exact anchor BibleGraphNodeId, optional fictional story_time_ms,
  direction incoming/outgoing/both (default both), optional edge-kind filter
  (empty means all stored kinds), neighbor_limit 1..8 (default8). One hop only.
  No implicit alias resolution, fictional-time inference or renderer defaults.
- The anchor must be live. Read only live canonical edges incident to it and
  live endpoints. Directed edges obey requested direction. Undirected edges
  are traversable from either endpoint; retain their actual stored orientation
  and directed=false. Deduplicate self/cyclic paths; never expand a neighbor.
- Order: anchor first; candidate neighbors in stable ID order; connecting edges
  in stable edge-ID order. Return at most8 distinct neighbors and32 connecting
  edges. Counts describe matching live candidates omitted by each cap.
- Output: normalized request, canonical anchor/neighbor identities, resolved
  field values or unresolved field identities, typed connecting edges and an
  exact why-recalled path from the anchor. Edge temporal validity is explicitly
  untimed/unknown, not inferred from field resolution or connectedness.
- Reuse the existing temporal resolver. Unspecified time withholds timed
  fields; future-only, cleared and conflicting values retain current resolver
  semantics. Conflicts refuse the read explicitly; they never choose a winner.
- Carry actual owned name/field/edge revisions through existing lineage readers.
  Timed values also identify the original snapshot and snapshot-field IDs and
  their actual canonical update events. Resolve and capture every source and
  the projection envelope in one SQLite read snapshot; never assemble versions
  from separate reads or fabricate absent history.
- Bound returned evidence to32KiB by omitting whole field records, with explicit
  omitted-field counts. Retain identities and path/source metadata. An identity
  packet that cannot fit refuses explicitly rather than cutting identifiers,
  values or temporal qualifiers. No opaque pagination or historical lease.
- One shared domain projection serves native inspection and the read tool.
  Returned material is recalled evidence, not an accepted world update or a
  generation receipt. The read writes no canonical state, history, dependency,
  selected context link, pending proposal or screenplay.
- Native action: Recall related story facts in the selected Bible entity detail.
  Explicit optional story time and direction/kind controls; readable related
  names, values, path labels, untimed warning, unresolved states and omissions.
  Work with graph visualization closed and Bible/timeline/screenplay visible.
- Frontend projection ownership retains project-session, selected anchor,
  request identity and version guards. Bible mutations invalidate recalled
  evidence and pending requests; refreshing is explicit. Late success/error
  cannot replace a newer request, a changed anchor or a cleared session.

## Qualification and exclusions

Deterministic tests cover205+nodes, linked endpoints beyond the default prefix,
no unrelated filler, edge kinds/direction/undirected edges/self loops/cycles,
node/edge/evidence caps, temporal boundaries/conflicts/withheld future fields,
exact provenance and atomic source/version reads, deletion and late frontend
responses. Verify generation context and all saved screenplay, drafts and
pending proposals remain unchanged. Hosted native qualification uses existing
standard dependency/no-download admission and original unedited screenshots.
No synthetic responses are needed for this read-only native demonstration;
no real-model quality claim follows. No embeddings, model downloads, inference,
automatic generation-context change, project-switch feature, new persistence,
renderer selection change or main merge. Publish separate tested source and
evidence milestones for parent review.
