#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

usage() {
    echo "Usage: $0 [--install] [--build] [--run]"
    echo "  --install Install system and Python dependencies"
    echo "  --build   Build the Rust server and SvelteKit UI"
    echo "  --run     Run the Eidetic server"
    exit 1
}

do_install=false
do_build=false
do_run=false

if [[ $# -eq 0 ]]; then
    usage
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        --install) do_install=true; shift ;;
        --build)   do_build=true; shift ;;
        --run)     do_run=true; shift ;;
        *)         echo "Unknown option: $1"; usage ;;
    esac
done

# ── Install dependencies ──

if $do_install; then
    echo "==> Checking dependencies..."

    # --- System packages (apt) ---
    APT_PKGS=()
    for pkg in python3-dev python3-venv; do
        if ! dpkg -s "$pkg" >/dev/null 2>&1; then
            APT_PKGS+=("$pkg")
        else
            echo "    [ok] $pkg"
        fi
    done
    if [[ ${#APT_PKGS[@]} -gt 0 ]]; then
        echo "==> Installing system packages: ${APT_PKGS[*]}"
        sudo apt-get update -qq && sudo apt-get install -y -qq "${APT_PKGS[@]}"
    fi

    # --- Node.js ---
    if command -v node >/dev/null 2>&1; then
        echo "    [ok] node $(node --version)"
    else
        echo "ERROR: Node.js is not installed. Install it via nvm or your package manager."
        exit 1
    fi

    # --- Rust toolchain ---
    if command -v cargo >/dev/null 2>&1; then
        echo "    [ok] cargo $(cargo --version | awk '{print $2}')"
    else
        echo "ERROR: Rust/cargo is not installed. Install via https://rustup.rs"
        exit 1
    fi

    # --- Python venv + pip packages ---
    VENV_DIR="$SCRIPT_DIR/.venv"
    if [[ ! -d "$VENV_DIR" ]]; then
        echo "==> Creating Python venv at $VENV_DIR"
        python3 -m venv "$VENV_DIR"
    else
        echo "    [ok] Python venv"
    fi
    source "$VENV_DIR/bin/activate"

    PIP_PKGS=(torch transformers safetensors accelerate)
    MISSING_PKGS=()
    for pkg in "${PIP_PKGS[@]}"; do
        if ! python3 -c "import $pkg" 2>/dev/null; then
            MISSING_PKGS+=("$pkg")
        else
            echo "    [ok] $pkg"
        fi
    done
    if [[ ${#MISSING_PKGS[@]} -gt 0 ]]; then
        echo "==> Installing Python packages: ${MISSING_PKGS[*]}"
        pip install --quiet "${MISSING_PKGS[@]}"
    fi

    # --- npm dependencies ---
    if [[ -d "$SCRIPT_DIR/ui/node_modules" ]]; then
        echo "    [ok] ui/node_modules"
    else
        echo "==> Installing UI npm packages..."
        (cd "$SCRIPT_DIR/ui" && npm install)
    fi

    echo "==> All dependencies installed."
fi

# ── Build ──

# Activate venv if present (needed for PyO3 to find torch/transformers).
VENV_DIR="$SCRIPT_DIR/.venv"
if [[ -d "$VENV_DIR" ]]; then
    source "$VENV_DIR/bin/activate"
    export PYO3_PYTHON="$VENV_DIR/bin/python3"
fi

if $do_build; then
    echo "==> Building UI..."
    (cd ui && npm install && npm run build)

    echo "==> Building server..."
    cargo build --release -p eidetic-server
    echo "==> Build complete."
fi

if $do_run; then
    SERVER_URL="http://127.0.0.1:3000"

    echo "==> Starting Eidetic server..."
    cargo run --release -p eidetic-server &
    SERVER_PID=$!
    trap "kill $SERVER_PID 2>/dev/null" EXIT

    echo "==> Waiting for server to be ready..."
    for i in $(seq 1 30); do
        if curl -sf "$SERVER_URL" >/dev/null 2>&1; then
            echo "==> Server is ready at $SERVER_URL"
            xdg-open "$SERVER_URL" 2>/dev/null || open "$SERVER_URL" 2>/dev/null || true
            break
        fi
        sleep 0.5
    done

    wait $SERVER_PID
fi
