#!/usr/bin/env bash
# Build a native wheel with Git for Windows Bash, using the selected Python ABI.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PYTHON_BIN="${PYTHON_BIN:-python}"

finish() {
    local status=$?
    trap - EXIT

    if [[ "${NO_PAUSE:-0}" != "1" && -t 0 && -t 1 ]]; then
        if [[ "$status" -eq 0 ]]; then
            printf '\nBuild completed successfully.\n'
        else
            printf '\nBuild failed with exit code %s.\n' "$status" >&2
        fi
        read -r -p "Press Enter to close..."
    fi

    exit "$status"
}

trap finish EXIT

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        printf 'Required command not found: %s\n' "$1" >&2
        exit 1
    }
}

require_command "$PYTHON_BIN"
require_command cargo
require_command uvx

PYTHON_PATH="$(command -v "$PYTHON_BIN")"
if command -v cygpath >/dev/null 2>&1; then
    PYTHON_PATH="$(cygpath -w "$PYTHON_PATH")"
fi

cd "$ROOT_DIR"
mkdir -p dist

uvx --from maturin maturin build \
    --release \
    --out dist \
    --interpreter "$PYTHON_PATH"

printf 'Built wheel(s):\n'
find dist -maxdepth 1 -type f -name 'dirdb_rust-*.whl' -print
