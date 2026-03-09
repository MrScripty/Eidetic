#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_NAME="$(basename "$0")"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_BIN="eidetic-server"
UI_DIR="$PROJECT_ROOT/ui"
UI_BUILD_DIR="$PROJECT_ROOT/dist/ui"
VENV_DIR="$PROJECT_ROOT/.venv"
RELEASE_BIN_PATH="$PROJECT_ROOT/target/release/${APP_BIN}"
LAUNCHER_STATE_ROOT="${EIDETIC_LAUNCHER_STATE_ROOT:-$PROJECT_ROOT/.launcher-state}"
PYTHON_PACKAGES=(torch transformers safetensors accelerate protobuf sentencepiece)
DEPENDENCIES=(node npm cargo python3 python_venv ui_deps git_hooks)

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

check_python3() { command -v python3 >/dev/null 2>&1; }
install_python3() { die "python3 is required. Install Python 3 and rerun --install."; }

python_package_available() {
  local python_bin="$1"
  local package="$2"
  local module_name="$package"

  case "$package" in
    protobuf)
      module_name="google.protobuf"
      ;;
  esac

  "$python_bin" -c "import importlib.util, sys; sys.exit(0 if importlib.util.find_spec('$module_name') else 1)" >/dev/null 2>&1
}

check_python_venv() {
  local package

  [[ -x "$VENV_DIR/bin/python3" ]] || return 1
  for package in "${PYTHON_PACKAGES[@]}"; do
    python_package_available "$VENV_DIR/bin/python3" "$package" || return 1
  done
}

install_python_venv() {
  local missing_packages=()
  local package

  check_python3 || install_python3

  if [[ ! -x "$VENV_DIR/bin/python3" ]]; then
    log "[install] python_venv missing; creating $VENV_DIR"
    python3 -m venv "$VENV_DIR"
  fi

  "$VENV_DIR/bin/python3" -m pip install --quiet --upgrade pip

  for package in "${PYTHON_PACKAGES[@]}"; do
    if ! python_package_available "$VENV_DIR/bin/python3" "$package"; then
      missing_packages+=("$package")
    fi
  done

  if ((${#missing_packages[@]} > 0)); then
    "$VENV_DIR/bin/python3" -m pip install --quiet "${missing_packages[@]}"
  fi
}

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

activate_python_env() {
  if [[ -x "$VENV_DIR/bin/python3" ]]; then
    export PYO3_PYTHON="$VENV_DIR/bin/python3"
  fi
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
  activate_python_env

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
  activate_python_env
  prepare_persistent_state "dev"

  exec cargo run -p "$APP_BIN" -- "${run_args[@]}"
}

run_release() {
  local run_args=("$@")

  ensure_ui_assets
  activate_python_env
  prepare_persistent_state "release"

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
  activate_python_env
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
