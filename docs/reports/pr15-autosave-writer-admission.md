# PR15 public edit/autosave writer admission

The shared history writer now reserves SQLite write ownership before its
transaction-local command-signature read. A real public edit/autosave regression
establishes why this is required: the deferred reader could become a stale WAL
snapshot after autosave committed, then fail immediately at command INSERT with
SQLITE_BUSY_SNAPSHOT(517). This is a transaction ordering defect, not a timeout
or presumed Windows flake.

## Exact source and preserved failure

- Preserved PR15 head: `c6dc3a716bcb5db61994004bb5a49fc989837580`, tree
  `8b879508a1e222515b4f086c33b6ab0ad8a77772`.
- [Failed CI37526522571](https://github.com/MrScripty/Eidetic/actions/runs/37526522571),
  Windows job112484801596: the original delayed-regeneration test failed at538
  with `Internal("database is locked")`; other481 server tests passed.
- Original test blob `9a9db5c392babd75437b43f3715a1025700a3364` matches merged main.
  Existing canonical generation tests remain byte-for-byte unchanged; the new
  regression is appended. No failed CI rerun or review request was made.
- Frozen reproduction `b3a95a9e856d949d142b62e6d82cc4d600d18f8c`, tree
  `1bfdbe06013ba40b137de7edd6d31eace61e61d0`, local branch
  `test/pr15-autosave-snapshot-regression`.
- Tested repair `d159bd3ae6f0ad4314504f681a707ceb6657db0f`, tree
  `40067a4e1de990d4f165d3d35e773413c8b84bcb`.
- Author and committer: `MrScripty <TheEnvironmentGuy@protonmail.com>`.

## Deterministic actual-service ordering

Test-only probes are scoped by command UUID and database path, removed through
registrations, and never hold their registry mutex while a worker pauses. The
fixture executes successful first generation through the production HTTP client,
starts delayed regeneration, and pauses the actual public edit immediately after
its transaction-local command-signature SELECT. The real autosave loop is then
admitted through its existing session gate and production two-second debounce.
It commits through `persistence::save_project`, rather than an injected SQL write.
Releasing the edit observes code517 at the first `INSERT INTO commands` and the
same public service error as CI. Reaching that barrier rules out earlier
schema/open failure in this reproduction.

The Windows log does not include an extended code. The forced Rust ordering
proves this application defect; it does not retroactively identify the precise
statement or extended code of the historical Windows failure. Its original
failure and subsequent disconnected synthetic HTTP worker remain preserved.

## Repair and ownership

`history_store::record_change_with` first checks immutable committed replay using
a read-only operation, then begins IMMEDIATE for fresh work. It repeats signature
validation after writer admission, so a caller that initially missed another
in-flight command cannot duplicate it or apply a conflicting payload. Existing
mutation callbacks still validate canonical revisions and locks under that writer
and roll back command/history rows on refusal. Exact replay continues to work on
query_only connections while another connection owns the writer.

No application mutex, session/doc lock order, busy timeout, retry loop or error
suppression is added. SQL write ownership is acquired before the authority read;
the existing autosave session gate continues to precede persistence. Public edits
do not acquire that gate while owning SQLite, avoiding a new inverted lock order.
The repair covers all callers of the existing shared history mechanism without
parallel state or a screenplay-specific transaction implementation.

The forced regression now observes writer_reserved=true, INSERT success and a
saved manual edit. Autosave completes after that write. Late generation refuses
the changed target with unchanged history, and canonical preview still reads the
exact manual text. Separate barriers force two callers to observe the same fresh
command ID missing before admission: equal payload replays once, conflicting
payload refuses, and the mutation callback runs once.

## Validation and limits

Local full server485/core119/UI459/production HTTP-driver39/no-download5 pass.
The originally failing test and all name cases pass. Strict server all-target
Clippy (`-D warnings`), frontend typecheck0/0/lint/format/build, rustfmt, diff and
decision-traceability gates pass. Locked all-features Cargo metadata passes the
ONNX no-download union guard; no dependency or bypass setting changes.

Exact named logs and SHA256 receipts are in the companion JSON and
`/workspace/scratch/pr15-autosave-repair/`. Local workspace/native build remains
blocked by missing glib-2.0.pc; ordinary PR15 hosted CI owns the platform-wide
checks. Publishing this tested successor triggers a new exact-head CI; it is not
a blind retry of the preserved failure. Prior native screenshots stay pinned to
their original source and are not relabelled as post-repair qualification.
No merge or CodeRabbit request; parent retains review cadence and merge.
