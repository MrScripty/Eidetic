# Dependency Governance

Eidetic follows the dependency and release standards in
`developer-tooling/Coding-Standards`. This document records repository-specific
policy, tool ownership, and tracked exceptions.

## Required Checks

Run the dependency gate before release and on dependency-changing PRs:

```bash
scripts/check-dependencies.sh
```

The gate owns these checks:

- Rust security advisories: `cargo audit`
- Rust license, advisory, source, and ban policy: `cargo deny check`
- Rust unused dependency heuristic: `cargo machete --with-metadata`
- Rust duplicate versions: `cargo tree --duplicates`
- Node high/critical advisories: `npm audit --audit-level=high`
- Node duplicate/invalid install state: `npm ls`

`cargo tree --duplicates` is a warning gate because duplicate transitive crates
are often unavoidable in desktop stacks. The output is written to
`target/dependency-checks/cargo-duplicates.txt` for review.

Set `EIDETIC_REQUIRE_DEPENDENCY_TOOLS=1` in CI and release jobs. With that
setting, missing `cargo audit`, `cargo deny`, or `cargo machete` is a failure
instead of a local warning.

## External Pumas Dependency

`eidetic-server` depends on `pumas-library` through a workspace path dependency:

```toml
pumas-library = { path = "../../ai-systems/Pumas-Library/rust/crates/pumas-core" }
```

The path is intentional while Pumas is developed as a sibling owned repository,
but it must be reproducible for CI and new developer machines.

Use the preparation script before running Cargo commands on a machine that does
not already have the sibling checkout:

```bash
EIDETIC_FETCH_PUMAS=1 scripts/prepare-pumas-dependency.sh
```

Default source contract:

- Repository: `https://github.com/MrScripty/Pumas-Library.git`
- Revision: `ad31e391dcd94c204b3fcd748cc98d27b3281e65`
- Required package: `pumas-library`
- Required local path: `../../ai-systems/Pumas-Library/rust/crates/pumas-core`

Override the source with `EIDETIC_PUMAS_LIBRARY_REPOSITORY`,
`EIDETIC_PUMAS_LIBRARY_REF`, or `EIDETIC_PUMAS_LIBRARY_ROOT` when CI uses a
private mirror or an already provisioned checkout.

## Tracked Exceptions

- Duplicate Rust crates are currently reviewed as warnings. The Bevy, Tauri,
  and Pumas dependency trees intentionally bring overlapping versions of common
  crates such as `serde`, `syn`, `bitflags`, and graphics stack dependencies.
  Consolidation should happen during dependency upgrade work, not as incidental
  feature work.
- Pumas remains a path dependency until the sibling repository has an agreed
  publication or git-dependency contract. CI must provision the exact expected
  checkout before running Cargo.
- `cookie 0.6.0` remains a low-severity finding through SvelteKit's declared
  `^0.6.0` dependency. The static desktop deployment has no Node HTTP cookie
  serializer or server load/form endpoints; application input does not supply
  cookie names, paths or domains. Do not override the producer's dependency
  contract to make the audit count disappear. The frontend dependency owner must
  select a supported parent update before adding server-side cookie handling or
  accepting untrusted cookie metadata. See the exact finding and follow-up in
  [PR29's dependency audit](reports/pr29-dependency-audit.md).

The 2026-10-14 PR29 audit repair maps all 14 high and 3 critical baseline rows to
their actual consumers and upstream advisories. It uses same-family patches,
SvelteKit 2.70.3 for a reachable development-server header issue, and a coordinated
Vitest/coverage 4.1.11 update that removes vulnerable Tinypool 1.x. Node remains
24.12.0 and Vite remains on 6.x; no forced transitive major overrides are used.
The CI frontend job now runs the existing high/critical audit policy and the
static build in addition to lint, typechecking and tests. Runtime-capable Svelte
dependencies are reviewed even though this app's manifest labels them as dev
dependencies. Audit counts include inherited parent rows; `fixAvailable: false`
does not prove that upstream has no patched release.

The current local-inference consumer uses Pumas 0.8.0-rc.1's HTTP advertisement /
admission-fence schema 1, typed model-operation contract 1 and serving RPC DTOs.
The `ad31e391` pin is intentional when integrating main `59793c19`: main's
`a94fd920` setup revision is Pumas 0.7.0 and does not expose the discovery module
or typed `/v1/model-operations` contract required by this consumer. The older
`8444b50d` documentation revision is also superseded. Keep the preparation script,
this source contract and the lockfile on the same tested producer revision;
native UI acceptance from PR30 does not qualify model inference.
Build `pumas-rpc` with its default `inference-plugins` feature; an inference-disabled
binary is explicitly refused. The path dependency and `Cargo.lock` package version
match the setup pin. See [local acceptance](pumas-local-inference.md) for runtime
setup; no application model is downloaded during build or test.

The 2026-10-14 authoring integration carries the exact reviewed frontend repair
from PR29 commit `be03aa1f2b7e6e19da632fac3b2b7eb89ec12c73`, without importing
its Pumas/model lifecycle changes. The security-relevant CI, package manifest,
lockfile and historical audit report match that source. The repair uses compatible
patches, SvelteKit 2.70.3 and coordinated Vitest/coverage 4.1.11, retaining Node
24.12.0 and Vite 6.x. The audit report's previous execution evidence belongs to
PR29; each new authoring integration must run its own checks and source CI.
