# Issues

- **Implemented, M1; independent review pending:** Provider, manifest and executor
  failures now persist terminal outcomes; tool intent precedes execution.
  Initial eight-test checkpoint passed; review added three fault-injection
  regressions preserving execution outcomes through call/result write failures.
  Narrow re-review pending. Cooperative cancellation is supported at
  provider/executor boundaries; process-crash recovery and UI cancellation
  transport remain later integration work.
- **Open, M2:** AI context includes all populated timed snapshots and does not
  resolve active facts. Snapshot `at_ms` has no independent flashback mapping.
- **Open, M3:** Embeddings use a direct OpenAI-shaped HTTP client rather than
  Pumas; the in-memory vector store has no model/revision identity checks.
- **Open, M3/M5:** Confirm installed runtime/model and reproducible Pumas source
  path before live inference and complete application acceptance.
