#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/target/dependency-checks"
REQUIRE_TOOLS="${EIDETIC_REQUIRE_DEPENDENCY_TOOLS:-0}"

log() {
  printf '[dependency-check] %s\n' "$*"
}

tool_missing() {
  local tool="$1"
  local install_hint="$2"

  if [[ "$REQUIRE_TOOLS" == "1" ]]; then
    log "missing required tool: $tool"
    log "install: $install_hint"
    exit 1
  fi

  log "warning: $tool not installed; skipped. Install with: $install_hint"
}

run_optional_tool() {
  local tool="$1"
  local install_hint="$2"
  shift 2

  if command -v "$tool" >/dev/null 2>&1; then
    "$@"
  else
    tool_missing "$tool" "$install_hint"
  fi
}

mkdir -p "$OUTPUT_DIR"

log "checking external Pumas dependency"
"$PROJECT_ROOT/scripts/prepare-pumas-dependency.sh"

log "checking Rust security advisories"
run_optional_tool cargo-audit "cargo install cargo-audit --locked" cargo audit

log "checking Rust licenses, bans, sources, and advisories"
run_optional_tool cargo-deny "cargo install cargo-deny --locked" cargo deny check

log "checking Rust unused dependencies"
run_optional_tool cargo-machete "cargo install cargo-machete --locked" cargo machete --with-metadata

log "recording Rust duplicate dependencies"
if cargo tree --duplicates >"$OUTPUT_DIR/cargo-duplicates.txt"; then
  if [[ -s "$OUTPUT_DIR/cargo-duplicates.txt" ]]; then
    log "warning: duplicate Rust crates recorded at $OUTPUT_DIR/cargo-duplicates.txt"
  else
    log "no duplicate Rust crates detected"
  fi
else
  log "warning: cargo duplicate scan failed; see $OUTPUT_DIR/cargo-duplicates.txt"
fi

log "checking Node security advisories"
(cd "$PROJECT_ROOT/ui" && npm audit --audit-level=high)

log "checking Node dependency install state"
(cd "$PROJECT_ROOT/ui" && npm ls)

log "dependency checks completed"
