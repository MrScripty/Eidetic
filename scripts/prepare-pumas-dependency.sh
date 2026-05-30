#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_PUMAS_ROOT="$(cd "$PROJECT_ROOT/../.." && pwd)/ai-systems/Pumas-Library"
PUMAS_ROOT="${EIDETIC_PUMAS_LIBRARY_ROOT:-$DEFAULT_PUMAS_ROOT}"
PUMAS_REPOSITORY="${EIDETIC_PUMAS_LIBRARY_REPOSITORY:-https://github.com/MrScripty/Pumas-Library.git}"
PUMAS_REF="${EIDETIC_PUMAS_LIBRARY_REF:-8444b50df28c3e2bd8db58fb3645fa4dd8664b27}"
PUMAS_PACKAGE_PATH="$PUMAS_ROOT/rust/crates/pumas-core"

log() {
  printf '[pumas] %s\n' "$*"
}

die() {
  log "error: $*"
  exit 1
}

fetch_pumas() {
  local parent_dir

  [[ "${EIDETIC_FETCH_PUMAS:-0}" == "1" ]] \
    || die "missing Pumas checkout at $PUMAS_ROOT; set EIDETIC_FETCH_PUMAS=1 to clone it"

  command -v git >/dev/null 2>&1 || die "git is required to fetch Pumas"

  parent_dir="$(dirname "$PUMAS_ROOT")"
  mkdir -p "$parent_dir"

  log "cloning $PUMAS_REPOSITORY into $PUMAS_ROOT"
  git clone "$PUMAS_REPOSITORY" "$PUMAS_ROOT"
  git -C "$PUMAS_ROOT" checkout "$PUMAS_REF"
}

verify_pumas() {
  [[ -f "$PUMAS_ROOT/rust/Cargo.toml" ]] \
    || die "Pumas Rust workspace manifest is missing at $PUMAS_ROOT/rust/Cargo.toml"
  [[ -f "$PUMAS_PACKAGE_PATH/Cargo.toml" ]] \
    || die "pumas-core package manifest is missing at $PUMAS_PACKAGE_PATH/Cargo.toml"

  if ! grep -q '^name = "pumas-library"$' "$PUMAS_PACKAGE_PATH/Cargo.toml"; then
    die "expected package name pumas-library in $PUMAS_PACKAGE_PATH/Cargo.toml"
  fi

  if [[ -d "$PUMAS_ROOT/.git" ]]; then
    local actual_ref
    actual_ref="$(git -C "$PUMAS_ROOT" rev-parse HEAD)"
    if [[ "$actual_ref" != "$PUMAS_REF" ]]; then
      die "Pumas checkout is at $actual_ref; expected $PUMAS_REF"
    fi
  fi

  log "ready at $PUMAS_PACKAGE_PATH"
}

if [[ ! -d "$PUMAS_ROOT" ]]; then
  fetch_pumas
fi

verify_pumas
