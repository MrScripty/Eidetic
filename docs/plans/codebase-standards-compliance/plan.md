# Plan: Codebase Standards Compliance

## Objective

Bring Eidetic into compliance with the coding standards in
`developer-tooling/Coding-Standards` without treating raw line count as a
violation. The plan should close hard gate failures first, then add missing
governance, documentation, release, cross-platform, accessibility, dependency,
and lifecycle controls.

## Scope

### In Scope

- CI, hooks, PR template, and decision traceability.
- Current Rust and frontend verification failures.
- Documentation layout and source directory README coverage.
- Dependency, release, and toolchain governance.
- Cross-platform build and test support.
- Async task lifecycle and GUI test behavior.
- Frontend accessibility and TypeScript strictness.
- Complection review for large or dense files where independent concerns are
  coupled.

### Out of Scope

- Feature redesigns unrelated to standards compliance.
- Splitting files only because they are long.
- Replacing working architecture without a clearer ownership boundary.
- Broad UI redesign beyond accessibility and testability fixes.

## Inputs

### Problem

The standards audit found several hard compliance gaps: missing CI and PR
traceability, failing clippy, a headless test failure, incomplete documentation
coverage, weak release/dependency governance, incomplete cross-platform checks,
abort-only async task shutdown, and frontend accessibility/type-safety gaps.

The earlier audit also flagged large files as if line count itself were a
standards violation. That interpretation was wrong. The current coding standard
defines simplicity as reduced entanglement between concepts, not reduced lines
of code. Large files are review signals only; they become refactor candidates
when independent concerns are complected.

### Constraints

- Preserve existing user changes and avoid unrelated refactors.
- Keep fixes scoped to the standard being addressed in each milestone.
- Do not split modules unless the new boundary reduces reasoning load.
- Every new gate must be runnable locally through existing project tooling or a
  documented command.
- CI must be able to run in a headless environment.

### Assumptions

- GitHub Actions is the intended CI platform.
- The project should support at least Linux and Windows checks.
- Tauri/Svelte and Rust workspace boundaries remain the primary architecture.
- Existing `launcher.sh` remains the repo entry point for common local actions.
- The external local `pumas-library` path dependency is intentional, but still
  needs reproducible setup documentation or a controlled replacement.

### Dependencies

- Coding standards under `developer-tooling/Coding-Standards`.
- Existing `launcher.sh`, `lefthook.yml`, Cargo workspace, and `ui/package.json`.
- Rust tooling: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo check`.
- Frontend tooling: `eslint`, `prettier`, `svelte-check`, `vitest`.
- Optional audit tools selected during implementation: `cargo audit`,
  `cargo deny`, `cargo machete`, npm audit, and license checks.

### Risks

| Risk | Impact | Mitigation |
| ---- | ------ | ---------- |
| New CI exposes additional platform failures. | High | Add CI in stages and treat first failures as discovery work. |
| Headless GUI tests stay brittle. | High | Separate pure renderer/state tests from display-required smoke tests. |
| Documentation work becomes generic filler. | Medium | Use module-specific decision rationale and reject inventory-only READMEs. |
| Dependency cleanup changes runtime behavior. | Medium | Start with audit/report gates before version consolidation. |
| Refactoring large files creates artificial boundaries. | Medium | Require a complection review before any extraction. |

### Simplicity / Complection Review

- Independent concepts in this change:
  tooling gates, release governance, documentation traceability, dependency
  policy, cross-platform support, async lifecycle, frontend accessibility, and
  module ownership.
- Concepts intentionally coupled:
  local launcher commands and CI jobs should verify the same behavior where
  possible; module READMEs and source directories should stay coupled by
  decision traceability.
- Concepts accidentally coupled or at risk:
  native Bevy rendering tests are coupled to an available display; task handles
  are coupled to abort-only cleanup; frontend actions are coupled to unlabeled
  controls; external local dependency setup is coupled to one developer machine.
- Boundary that owns each policy/state/lifecycle decision:
  CI owns required verification, `launcher.sh` owns local entry points,
  module READMEs own local design rationale, task supervisors own background
  task shutdown, and platform modules/tests own OS-specific behavior.
- Future change that should not require touching this area:
  adding a feature should not require editing CI policy, release policy, or
  unrelated module documentation unless it changes those contracts.

## Definition of Done

- Required CI exists and passes on supported targets.
- Local hooks and launcher commands align with CI gates.
- Clippy passes with warnings denied.
- Headless test execution is reliable.
- Source directory README coverage and plan layout match documentation
  standards.
- Dependency, license, duplicate, and vulnerability checks are documented and
  automated.
- Release metadata, changelog, toolchain pins, release smoke, checksums, and
  SBOM expectations are represented in tooling.
- Cross-platform test/build issues are gated or explicitly isolated.
- Async task ownership includes cancellation, shutdown, and panic inspection.
- Frontend controls meet accessibility standards and exported TypeScript APIs
  have explicit return types.
- Any file extraction is justified by reduced complection, not by line count.

## Milestones

### Milestone 1: Establish Hard Verification Baseline

**Goal:** Make the existing required local checks pass or explicitly isolate
environment-dependent failures.

**Tasks:**

- [x] Fix current clippy failures.
- [x] Decide how headless environments should run native Bevy window tests.
- [x] Make `./launcher.sh --test` pass in a headless environment or split
  display-required smoke tests behind an explicit command.
- [x] Align `launcher.sh` hook checks with the configured `lefthook.yml` hook
  names.
- [ ] Make `--run` behavior comply with launcher lifecycle requirements or
  document why the wrapper must own the UI dev server child process.

**Verification:**

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo check --workspace --all-targets`
- `./launcher.sh --test`

**Status:** In progress. Clippy baseline, headless default tests, and hook-name
alignment are complete. `--run` exec behavior remains open.

### Milestone 2: Add CI and Traceability Gates

**Goal:** Move required verification from developer convention into enforced
repository policy.

**Tasks:**

- [ ] Add `.github/workflows/ci.yml` with Linux and Windows jobs.
- [ ] Configure matrix jobs with fail-fast disabled.
- [ ] Add PR template from the standards template.
- [ ] Add decision-traceability script under `scripts/`.
- [ ] Wire decision traceability, lint, format, typecheck, and tests into CI.
- [ ] Keep local `lefthook.yml` aligned with CI-critical gates.

**Verification:**

- CI runs on a branch and reports all required jobs.
- `scripts/check-decision-traceability.sh` catches missing README/ADR updates
  for changed source directories.
- `lefthook install` produces the expected hooks.

**Status:** Not started.

### Milestone 3: Documentation Layout and Module READMEs

**Goal:** Make documentation traceability match the standards without adding
filler.

**Tasks:**

- [ ] Move root-level plan artifacts into `docs/plans/<slug>/`.
- [ ] Add required README files to source directories that lack them.
- [ ] Include purpose, constraints, decisions, invariants, dependencies, and
  revisit triggers in each README.
- [ ] Add API consumer or structured producer contract sections where modules
  expose host-facing behavior or structured outputs.
- [ ] Remove or document empty source directories.

**Verification:**

- Decision traceability script passes.
- README content describes design rationale, not just file inventory.

**Status:** Not started.

### Milestone 4: Dependency and Release Governance

**Goal:** Make dependency and release standards enforceable.

**Tasks:**

- [ ] Add `rust-toolchain.toml` and a Node version pin.
- [ ] Add `CHANGELOG.md` with an `Unreleased` section.
- [ ] Add `publish = false` to non-publishable crates.
- [ ] Replace broad dependency features such as `tokio/full` with narrower
  feature sets where practical.
- [ ] Document or control the external `pumas-library` path dependency.
- [ ] Add vulnerability, license, unused dependency, and duplicate dependency
  checks.
- [ ] Add release workflow expectations for tag-triggered builds, smoke tests,
  checksums, and SBOM output.

**Verification:**

- Dependency audit commands pass or produce tracked exceptions.
- Release workflow can build artifacts in a dry run or documented equivalent.
- Package metadata passes release-standard review.

**Status:** Not started.

### Milestone 5: Cross-Platform and Async Lifecycle

**Goal:** Remove platform and background-task behavior that depends on unstated
runtime assumptions.

**Tasks:**

- [ ] Gate Unix-only tests or move platform-specific behavior behind a platform
  module.
- [ ] Add Windows compile/test coverage in CI.
- [ ] Add explicit cancellation channels or tokens for background tasks.
- [ ] Replace abort-only cleanup with shutdown paths that await task completion
  or inspect join errors.
- [ ] Bound polling loops and document lifecycle ownership for Tauri bridges.

**Verification:**

- Linux and Windows CI jobs pass.
- Task lifecycle tests cover cancellation and shutdown behavior.
- Logs expose background task panics or join failures.

**Status:** Not started.

### Milestone 6: Frontend Accessibility and Type Safety

**Goal:** Enforce frontend standards through code and lint configuration.

**Tasks:**

- [ ] Add or enable Svelte-compatible accessibility lint rules.
- [ ] Add `type="button"` to non-submit buttons.
- [ ] Add labels or accessible names to form controls and icon-only controls.
- [ ] Re-enable `no-explicit-any` or limit exceptions to narrow documented
  cases.
- [ ] Add explicit return types for exported TypeScript APIs.
- [ ] Consider stricter TypeScript compiler options such as
  `noUnusedParameters`, `noImplicitReturns`, and `exactOptionalPropertyTypes`.

**Verification:**

- `cd ui && npm run lint`
- `cd ui && npm run format:check`
- `cd ui && npm run typecheck`
- Frontend tests cover any behavior touched while adding accessible names.

**Status:** Not started.

### Milestone 7: Complection Review of Dense Files

**Goal:** Decide whether dense files should stay together or be split based on
reasoning boundaries, not size.

**Tasks:**

- [ ] Review `crates/bevy_bible_graph/src/native_render.rs` by concern:
  app/window lifecycle, camera navigation, hit testing, text editor behavior,
  projection rebuild, material cache, and command emission.
- [ ] Review `crates/server/src/affect_store.rs` by concern:
  schema setup, transaction invariants, row mapping, revision generation, and
  enum/target encoding.
- [ ] Review large test files for fixture builders or scenario grouping that
  would clarify invariants.
- [ ] Extract only when the new boundary lets readers ignore an independent
  concern safely.
- [ ] Record keep-together decisions in the relevant module README when a large
  file remains coherent.

**Verification:**

- Each extraction has a stated ownership boundary.
- No extraction is justified solely by line count.
- Existing behavior tests continue to pass.

**Status:** Not started.

## Execution Notes

- 2026-05-30: Initial plan created from standards audit. Large-file findings are
  reframed as complection review candidates rather than line-count violations.
- 2026-05-30: Rust clippy baseline completed. The cleanup exposed additional
  warnings after `eidetic-core` passed, including Bevy ECS system signature
  lint noise, copy/clone idioms, a timeline startup enum with a large config
  variant, and several server-side iterator/type-complexity issues.
- 2026-05-30: Headless `./launcher.sh --test` completed by splitting native
  Bevy/Winit display creation checks into ignored smoke tests while keeping
  pure control/resource behavior covered in default tests. This uncovered a
  remaining launcher standards gap: development `--run` supervises a Vite child
  server and therefore does not currently `exec` the app process.

## Commit Cadence Notes

- Commit each milestone after its verification commands pass.
- Keep documentation-only moves separate from behavior changes when practical.
- Follow conventional commit format from `COMMIT-STANDARDS.md`.

## Re-Plan Triggers

- CI exposes platform failures that require product or support-policy decisions.
- The external `pumas-library` dependency cannot be made reproducible without a
  repository ownership decision.
- Headless GUI test support requires a different renderer or test architecture.
- Complection review shows that an apparent extraction would increase reasoning
  load.

## Completion Summary

### Completed

- Plan drafted.

### Deviations

- None.

### Follow-Ups

- Execute milestones in order, starting with hard verification failures.

### Verification Summary

- Not yet run for implementation; this is a planning artifact.

### Traceability Links

- Module README updated: N/A.
- ADR added/updated: N/A.
- PR notes completed per `templates/PULL_REQUEST_TEMPLATE.md`: N/A.
