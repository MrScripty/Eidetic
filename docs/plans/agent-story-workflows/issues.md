# Issues

- **Implemented, M1; independent narrow re-review accepted:** Provider, manifest and executor
  failures now persist terminal outcomes; tool intent precedes execution.
  Initial eight-test checkpoint passed; review added three fault-injection
  regressions preserving execution outcomes through call/result write failures.
  All 11 harness tests passed independent re-review. Cooperative cancellation is supported at
  provider/executor boundaries; process-crash recovery and UI cancellation
  transport remain later integration work.
- **Open, M2:** Persistent fictional-time authoring and independent flashback
  mapping remain open; bounded field resolution is implemented in M2a.
- **Open, M3:** Embeddings use a direct OpenAI-shaped HTTP client rather than
  Pumas; the in-memory vector store has no model/revision identity checks.
- **Open, M3/M5:** Confirm installed runtime/model and reproducible Pumas source
  path before live inference and complete application acceptance.
- **Implemented, M2a; independent narrow re-review accepted:** Explicit per-request fictional
  time resolves sparse canonical fields before prompts; omitted time withholds
  affected fields. Persistent clip/world-time authoring and batch mapping remain
  open. Original canonical values/history remain unchanged.
- **Resolved, CI remediation:** Baseline renderer lint, Windows dependency and
  icon/test portability, and traceability runner repairs are merged into main.
  Main post-merge CI passed at `72b8485025701fc18220c6522562e874704b6286`.
  The composed M2a head still requires its own hosted checks and bot review.
- **Implemented, M3a; qualification pending:** Late embeddings cannot republish
  deleted/replaced sources, stale queries cannot cross index lifetimes, and
  configured endpoint/model/dimension mismatches are excluded from ranking.
  Exact source snapshots are checked on publication and retrieval. Immutable
  weight/revision identity, Pumas transport and re-indexing saved references
  remain open. Strict returned-model validation can make nonconforming embedding
  endpoints unavailable; it does not silently assume their identity.
