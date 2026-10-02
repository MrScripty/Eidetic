# CI baseline repair — 2026-10-02

## Scope and evidence

The harness PR's exact head `52a856048a06c5b036a17677b58901e497914446`
failed three hosted jobs in [run 37067693144](https://github.com/MrScripty/Eidetic/actions/runs/37067693144).
The frontend job passed. This repair is separate from harness and temporal
features and starts at main `ab5d75ce138c1b8e25cb5effc8f8800c0c6f9841`.

Comparison of Git blob identities proves that Cargo.toml, Cargo.lock,
.github/workflows/ci.yml, scripts/check-decision-traceability.sh and
crates/bevy_bible_graph/src/workspace.rs are unchanged between main and that
PR head. The diagnosed failing source/dependency/workflow inputs therefore
precede the harness patch. This comparison is not a claim that the entire main
branch was separately rebuilt on both operating systems.

## Repairs

1. **Traceability:** the hosted runner lacked `rg`, so the script incorrectly
   reported every README heading missing. Install ripgrep explicitly in its job
   and fail once with an actionable dependency error before checking documents
   when it is unavailable. Preserve all existing documentation checks.
2. **Linux clippy:** derive `Default` on
   `BibleGraphWorkspaceTimelinePresentationMode`, marking the existing
   `CameraAnchoredPanel` variant as default. No behavior, wire representation or
   renderer architecture changes.
3. **Windows compilation:** the lockfile selected `windows` 0.57 for
   gpu-allocator 0.27 while wgpu-hal 27.0.4 passed `windows` 0.58 Direct3D12 types
   across their shared boundary. Align only gpu-allocator's dependency edge to
   already locked 0.58. Keep sysinfo's separate 0.57 requirement intact. The
   [upstream allocator manifest](https://github.com/Traverse-Research/gpu-allocator/blob/0.27.0/Cargo.toml)
   explicitly supports >=0.53, <=0.58. This is a deliberate lockfile graph repair,
   not a renderer upgrade or feature removal.

## Verification and remaining gate

- Cargo's locked Windows dependency resolution accepts the repaired lockfile:
  gpu-allocator and wgpu-hal both resolve windows 0.58; sysinfo remains on 0.57.
- Shell syntax and scoped Rust formatting checks pass.
- Missing-ripgrep and normal traceability behavior are checked locally.
- Full native Linux/Windows compilation is intentionally left to hosted CI
  because local disk cannot support the native renderer build. No claim of
  cross-platform build success follows merely from dependency resolution.
- Independent review and fresh hosted checks must qualify the exact published
  repair before merge. Do not waive tests or treat these initial failure points
  as proof that later stages cannot reveal further problems.

Local baseline reproduction additionally confirms the original lock resolves
gpu-allocator against windows 0.57. The original traceability script under a
PATH without ripgrep produces the same false missing-heading errors, while the
repaired script exits once with the dependency diagnostic. With ripgrep present,
the unchanged documentation gate passes for the repair write set.

Independent review accepted this narrow repair with no blocking findings on
2026-10-02. Native hosted compilation remains required; local dependency
resolution and source review do not replace that gate. Reviewed patch SHA-256:
`9409a921b766f649c5d64fa4e4cebed8d0277334b3f91bb17a3bc498137e191d`.

## Subsequent Windows resource gate

At repair head `deff093a9ddbd04cbabd0dfe49a4a512226f0bcb`, Windows passed the
previous wgpu-hal type mismatch and reached Tauri's build script, which requires
`icons/icon.ico`. Main and the repair both contained only the existing 1×1 PNG
placeholder, despite an empty bundle icon list. Added an ICO container around
that existing placeholder. Independent review found its inherited PNG IDAT CRC
was invalid: Pillow accepted the image but Tauri's actual ico 0.5.0 decoder
rejected it. Repaired only the four checksum bytes in icon.png and regenerated
the ICO from that corrected PNG, preserving all image data and identical RGBA
pixels. The strict ico 0.5.0 entry decoder is now required for local verification.
This is packaging repair, not new artwork or a release-quality icon claim.
Hosted Windows compilation must still qualify the repaired input.

Independent review accepted the corrected packaging on 2026-10-02, verifying
all chunk checksums, unchanged image data and RGBA pixels, and successful decode
with ico 0.5.0. Accepted patch SHA-256:
`90bd8a7330b0c4a1c22bf428f1834f782a3ca8ecb3d2399bff91672b6b3ebc9b`.
