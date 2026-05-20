#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_NAME="$(basename "$0")"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_BIN="eidetic-server"
UI_DIR="$PROJECT_ROOT/ui"
UI_BUILD_DIR="$PROJECT_ROOT/dist/ui"
RELEASE_BIN_PATH="$PROJECT_ROOT/target/release/${APP_BIN}"
LAUNCHER_STATE_ROOT="${EIDETIC_LAUNCHER_STATE_ROOT:-$PROJECT_ROOT/.launcher-state}"
APP_URL="http://127.0.0.1:3000"
DEPENDENCIES=(node npm cargo ui_deps git_hooks)

usage() {
  cat <<EOF
Eidetic launcher for install, build, test, and run workflows.

Usage:
  ./${SCRIPT_NAME} --help
  ./${SCRIPT_NAME} --install
  ./${SCRIPT_NAME} --build
  ./${SCRIPT_NAME} --build-release
  ./${SCRIPT_NAME} --run [-- <app args...>]
  ./${SCRIPT_NAME} --run-release [-- <app args...>]
  ./${SCRIPT_NAME} --test

Examples:
  ./${SCRIPT_NAME} --install
  ./${SCRIPT_NAME} --build
  ./${SCRIPT_NAME} --build-release
  ./${SCRIPT_NAME} --run
  ./${SCRIPT_NAME} --run -- --log-level debug
  ./${SCRIPT_NAME} --run-release
  ./${SCRIPT_NAME} --test

Managed state:
  EIDETIC_LAUNCHER_ISOLATE_STATE=0   Use host XDG state/data directories.
  EIDETIC_LAUNCHER_STATE_ROOT=<dir>  Override the launcher-managed state root.

Exit codes:
  0 success
  1 operation failed
  2 usage error
  3 missing dependency for runtime
  4 missing release artifact
EOF
}

log() {
  printf '[launcher] %s\n' "$*"
}

die() {
  log "error: $*"
  exit 1
}

die_usage() {
  log "usage error: $*"
  usage
  exit 2
}

check_node() { command -v node >/dev/null 2>&1; }
install_node() { die "Node.js is required. Install it and rerun --install."; }

check_npm() { command -v npm >/dev/null 2>&1; }
install_npm() { die "npm is required. Install Node.js/npm and rerun --install."; }

check_cargo() { command -v cargo >/dev/null 2>&1; }
install_cargo() { die "cargo is required. Install Rust via rustup and rerun --install."; }

check_ui_deps() {
  [[ -d "$UI_DIR/node_modules" ]]
}

install_ui_deps() {
  check_npm || install_npm
  (cd "$UI_DIR" && npm install)
}

check_git_hooks() {
  [[ -x "$PROJECT_ROOT/.git/hooks/pre-commit" ]] \
    && [[ -x "$PROJECT_ROOT/.git/hooks/prepare-commit-msg" ]] \
    && [[ -e "$PROJECT_ROOT/node_modules/lefthook/bin/index.js" ]]
}

install_git_hooks() {
  mkdir -p "$PROJECT_ROOT/node_modules" "$PROJECT_ROOT/node_modules/.bin"
  ln -sfn "$UI_DIR/node_modules/lefthook" "$PROJECT_ROOT/node_modules/lefthook"
  if [[ -x "$UI_DIR/node_modules/.bin/lefthook" ]]; then
    ln -sfn "$UI_DIR/node_modules/.bin/lefthook" "$PROJECT_ROOT/node_modules/.bin/lefthook"
  fi
  (cd "$UI_DIR" && npx lefthook install -f)
}

check_dep() { "check_$1"; }
install_dep() { "install_$1"; }

install_dependencies() {
  local dependency

  for dependency in "${DEPENDENCIES[@]}"; do
    if check_dep "$dependency"; then
      log "[ok] $dependency already satisfied"
      continue
    fi

    log "[install] $dependency missing; installing"
    install_dep "$dependency"

    if check_dep "$dependency"; then
      log "[done] $dependency installed"
    else
      log "[error] $dependency install failed verification"
      exit 1
    fi
  done
}

ensure_dependencies() {
  local dependency

  for dependency in "$@"; do
    if ! check_dep "$dependency"; then
      log "missing dependency: $dependency"
      log "run ./${SCRIPT_NAME} --install first"
      exit 3
    fi
  done
}

use_isolated_state() {
  [[ "${EIDETIC_LAUNCHER_ISOLATE_STATE:-1}" != "0" ]]
}

setup_managed_state_env() {
  local mode="$1"
  local state_dir="$2"

  if ! use_isolated_state; then
    log "[state] using host state directories"
    return
  fi

  mkdir -p "$state_dir"/xdg-cache "$state_dir"/xdg-config "$state_dir"/xdg-data "$state_dir"/xdg-state
  export XDG_CACHE_HOME="$state_dir/xdg-cache"
  export XDG_CONFIG_HOME="$state_dir/xdg-config"
  export XDG_DATA_HOME="$state_dir/xdg-data"
  export XDG_STATE_HOME="$state_dir/xdg-state"
  log "[state] using isolated ${mode} state at $state_dir"
}

prepare_persistent_state() {
  local mode="$1"

  mkdir -p "$LAUNCHER_STATE_ROOT"
  setup_managed_state_env "$mode" "$LAUNCHER_STATE_ROOT/$mode"
}

prepare_temp_state() {
  local mode="$1"
  local temp_dir

  mkdir -p "$LAUNCHER_STATE_ROOT"
  temp_dir="$(mktemp -d "$LAUNCHER_STATE_ROOT/${mode}.XXXXXX")"
  setup_managed_state_env "$mode" "$temp_dir"
  printf '%s\n' "$temp_dir"
}

open_app_when_ready() {
  local url="$1"
  local attempts=60

  if [[ "${EIDETIC_OPEN_BROWSER:-1}" == "0" ]]; then
    log "[ui] browser auto-open disabled; open $url"
    return
  fi

  if ! command -v xdg-open >/dev/null 2>&1; then
    log "[ui] xdg-open not found; open $url"
    return
  fi

  (
    log "[ui] waiting for $url"

    if ! command -v curl >/dev/null 2>&1; then
      sleep 2
      log "[ui] opening $url"
      if ! xdg-open "$url" >/dev/null 2>&1; then
        log "[ui] browser open failed; open $url"
      fi
      exit 0
    fi

    while ((attempts > 0)); do
      if curl -fsS "$url" >/dev/null 2>&1; then
        log "[ui] opening $url"
        if ! xdg-open "$url" >/dev/null 2>&1; then
          log "[ui] browser open failed; open $url"
        fi
        exit 0
      fi
      attempts=$((attempts - 1))
      sleep 0.25
    done
    log "[ui] server did not become ready; open $url"
  ) &
}

ensure_ui_assets() {
  [[ -f "$UI_BUILD_DIR/index.html" ]] || die "UI assets are missing. Run ./${SCRIPT_NAME} --build or --build-release first."
}

build_ui_assets() {
  ensure_dependencies node npm ui_deps
  log "[build] building UI assets"
  (cd "$UI_DIR" && npm run build)
}

build_server() {
  local mode="$1"

  ensure_dependencies cargo

  case "$mode" in
    dev)
      log "[build] compiling dev server binary"
      cargo build -p "$APP_BIN"
      ;;
    release)
      log "[build] compiling release server binary"
      cargo build --release -p "$APP_BIN"
      ;;
    *)
      die_usage "invalid build mode: $mode"
      ;;
  esac
}

run_dev() {
  local run_args=("$@")

  ensure_dependencies cargo
  ensure_ui_assets
  prepare_persistent_state "dev"
  open_app_when_ready "$APP_URL"

  exec cargo run -p "$APP_BIN" -- "${run_args[@]}"
}

run_release() {
  local run_args=("$@")

  ensure_ui_assets
  prepare_persistent_state "release"
  open_app_when_ready "$APP_URL"

  if [[ ! -x "$RELEASE_BIN_PATH" ]]; then
    log "missing release binary: $RELEASE_BIN_PATH"
    log "run ./${SCRIPT_NAME} --build-release first"
    exit 4
  fi

  exec "$RELEASE_BIN_PATH" "${run_args[@]}"
}

run_tests() {
  local state_dir

  ensure_dependencies cargo node npm ui_deps
  state_dir="$(prepare_temp_state "test")"
  trap "rm -rf '$state_dir'" EXIT

  log "[test] running Rust tests"
  cargo test --workspace --all-targets

  log "[test] running frontend lint"
  (cd "$UI_DIR" && npm run lint)

  log "[test] running frontend format check"
  (cd "$UI_DIR" && npm run format:check)

  log "[test] running frontend typecheck"
  (cd "$UI_DIR" && npm run check)

  log "[test] running frontend tests"
  (cd "$UI_DIR" && npm run test)

  trap - EXIT
  rm -rf "$state_dir"
}

main() {
  local action=""
  local run_args=()

  while (($#)); do
    case "$1" in
      --help)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="help"
        shift
        ;;
      --install)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="install"
        shift
        ;;
      --build)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="build"
        shift
        ;;
      --build-release)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="build-release"
        shift
        ;;
      --run)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="run"
        shift
        ;;
      --run-release)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="run-release"
        shift
        ;;
      --test)
        [[ -z "$action" ]] || die_usage "only one action flag is allowed"
        action="test"
        shift
        ;;
      --)
        [[ "$action" == "run" || "$action" == "run-release" ]] \
          || die_usage "-- is only valid with --run or --run-release"
        shift
        run_args=("$@")
        break
        ;;
      *)
        die_usage "unknown argument: $1"
        ;;
    esac
  done

  [[ -n "$action" ]] || die_usage "one action flag is required"

  case "$action" in
    help)
      usage
      ;;
    install)
      install_dependencies
      ;;
    build)
      build_ui_assets
      build_server "dev"
      ;;
    build-release)
      build_ui_assets
      build_server "release"
      ;;
    run)
      run_dev "${run_args[@]}"
      ;;
    run-release)
      run_release "${run_args[@]}"
      ;;
    test)
      ((${#run_args[@]} == 0)) || die_usage "--test does not accept app args"
      run_tests
      ;;
    *)
      die_usage "invalid action: $action"
      ;;
  esac
}

main "$@"
