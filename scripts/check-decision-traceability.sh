#!/usr/bin/env bash
set -euo pipefail

ADR_DIR="${ADR_DIR:-docs/adr}"
BASE_REF="${TRACEABILITY_BASE_REF:-origin/main}"
SOURCE_ROOTS="${TRACEABILITY_SOURCE_ROOTS:-crates/core/src crates/server/src crates/bevy_bible_graph/src crates/bevy_timeline/src src-tauri/src ui/src}"
HOST_FACING_DIRS="${TRACEABILITY_HOST_FACING_DIRS:-src-tauri/src/commands,src-tauri/src/projections,crates/server/src,crates/core/src/contracts}"
STRUCTURED_PRODUCER_DIRS="${TRACEABILITY_STRUCTURED_PRODUCER_DIRS:-src-tauri/capabilities,src-tauri/gen,src-tauri/gen/schemas}"

required_headers=(
  "## Purpose"
  "## Contents"
  "## Problem"
  "## Constraints"
  "## Decision"
  "## Alternatives Rejected"
  "## Invariants"
  "## Revisit Triggers"
  "## Dependencies"
  "## Related ADRs"
  "## Usage Examples"
)

host_facing_headers=(
  "## API Consumer Contract"
)

structured_producer_headers=(
  "## Structured Producer Contract"
)

banned_placeholders=(
  "Source file used by modules in this directory."
  "Subdirectory containing related implementation details."
  "Keep files in this directory scoped to a single responsibility boundary."
  "import { value } from './module';"
)

extract_section_body() {
  local header="$1"
  local file="$2"

  awk -v header="$header" '
    $0 == header { in_section = 1; next }
    in_section && /^## / { exit }
    in_section { print }
  ' "$file"
}

dir_list_contains() {
  local dir_path="${1#./}"
  local list="$2"
  local item normalized

  [ -n "$list" ] || return 1

  IFS=',' read -ra items <<< "$list"
  for item in "${items[@]}"; do
    normalized="${item#./}"
    normalized="${normalized%/}"
    if [ "$normalized" = "$dir_path" ]; then
      return 0
    fi
  done

  return 1
}

is_under_source_root() {
  local file="$1"
  local source_root

  for source_root in $SOURCE_ROOTS; do
    if [[ "$file" == "$source_root/"* ]]; then
      return 0
    fi
  done

  return 1
}

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "Not inside a git repository."
  exit 1
fi

diff_range="${TRACEABILITY_DIFF_RANGE:-}"
if [ -z "$diff_range" ]; then
  if git rev-parse --verify "$BASE_REF" >/dev/null 2>&1; then
    diff_range="$BASE_REF...HEAD"
  elif git rev-parse --verify "origin/master" >/dev/null 2>&1; then
    diff_range="origin/master...HEAD"
  elif git rev-parse --verify "main" >/dev/null 2>&1; then
    diff_range="main...HEAD"
  elif git rev-parse --verify "master" >/dev/null 2>&1; then
    diff_range="master...HEAD"
  elif git rev-parse --verify "HEAD~1" >/dev/null 2>&1; then
    diff_range="HEAD~1...HEAD"
  else
    echo "Skipping decision traceability check: unable to resolve diff base."
    exit 0
  fi
fi

mapfile -t changed_files < <(git diff --name-only --diff-filter=ACMR "$diff_range")
if [ "${#changed_files[@]}" -eq 0 ]; then
  echo "No changed files detected for decision traceability check."
  exit 0
fi

declare -A changed_lookup=()
for file in "${changed_files[@]}"; do
  changed_lookup["$file"]=1
done

adr_changed=false
for file in "${changed_files[@]}"; do
  if [[ "$file" == "$ADR_DIR/"*.md ]]; then
    adr_changed=true
    break
  fi
done

declare -A changed_dirs=()
for file in "${changed_files[@]}"; do
  is_under_source_root "$file" || continue
  dir_path="$(dirname "$file")"
  changed_dirs["$dir_path"]=1
done

if [ "${#changed_dirs[@]}" -eq 0 ]; then
  echo "No source directory changes detected for decision traceability check."
  exit 0
fi

failures=0
for module_dir in "${!changed_dirs[@]}"; do
  readme_path="$module_dir/README.md"
  required_headers_for_dir=("${required_headers[@]}")

  if dir_list_contains "$module_dir" "$HOST_FACING_DIRS"; then
    required_headers_for_dir+=("${host_facing_headers[@]}")
  fi
  if dir_list_contains "$module_dir" "$STRUCTURED_PRODUCER_DIRS"; then
    required_headers_for_dir+=("${structured_producer_headers[@]}")
  fi

  if [ ! -f "$readme_path" ]; then
    echo "Missing README.md for changed directory: $module_dir"
    failures=$((failures + 1))
    continue
  fi

  missing_header=false
  for header in "${required_headers_for_dir[@]}"; do
    if ! rg -F -x -q "$header" "$readme_path"; then
      echo "Missing required heading in $readme_path: $header"
      missing_header=true
    fi
  done
  if [ "$missing_header" = true ]; then
    failures=$((failures + 1))
  fi

  none_format_invalid=false
  for header in "${required_headers_for_dir[@]}"; do
    section_body="$(extract_section_body "$header" "$readme_path")"
    if printf '%s\n' "$section_body" | rg -i -q '\bnone\b'; then
      if ! printf '%s\n' "$section_body" | rg -i -q 'reason:'; then
        echo "Section with None is missing Reason in $readme_path: $header"
        none_format_invalid=true
      fi
      if ! printf '%s\n' "$section_body" | rg -i -q 'revisit trigger:'; then
        echo "Section with None is missing Revisit trigger in $readme_path: $header"
        none_format_invalid=true
      fi
    fi
  done
  if [ "$none_format_invalid" = true ]; then
    failures=$((failures + 1))
  fi

  placeholder_found=false
  for phrase in "${banned_placeholders[@]}"; do
    if rg -F -q "$phrase" "$readme_path"; then
      echo "Banned placeholder phrase in $readme_path: $phrase"
      placeholder_found=true
    fi
  done
  if [ "$placeholder_found" = true ]; then
    failures=$((failures + 1))
  fi

  readme_changed=false
  if [ "${changed_lookup["$readme_path"]+set}" = "set" ]; then
    readme_changed=true
  fi

  if [ "$readme_changed" = false ] && [ "$adr_changed" = false ]; then
    echo "Changed directory without decision traceability update: $module_dir"
    echo "Update $readme_path or add/update an ADR under $ADR_DIR/."
    failures=$((failures + 1))
  fi
done

if [ "$failures" -gt 0 ]; then
  echo "Decision traceability check failed ($failures issue(s))."
  exit 1
fi

echo "Decision traceability check passed."
