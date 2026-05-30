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
- Revision: `8444b50df28c3e2bd8db58fb3645fa4dd8664b27`
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
- npm currently reports low-severity `cookie <0.7.0` advisories through
  `@sveltejs/kit`. The compatible `npm audit fix` path has already been applied;
  npm's remaining `--force` suggestion would install `@sveltejs/kit@0.0.30`, so
  it is rejected as a breaking downgrade. The dependency gate blocks
  high/critical npm advisories and leaves this low advisory tracked here until a
  compatible SvelteKit release resolves it.
