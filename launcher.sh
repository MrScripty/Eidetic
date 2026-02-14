#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

usage() {
    echo "Usage: $0 [--build] [--run]"
    echo "  --build   Build the Rust server and SvelteKit UI"
    echo "  --run     Run the Eidetic server"
    exit 1
}

do_build=false
do_run=false

if [[ $# -eq 0 ]]; then
    usage
fi

while [[ $# -gt 0 ]]; do
    case "$1" in
        --build) do_build=true; shift ;;
        --run)   do_run=true; shift ;;
        *)       echo "Unknown option: $1"; usage ;;
    esac
done

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
