#!/usr/bin/env bash
# Build a native wheel, then install it into the selected Python environment.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PYTHON_BIN="${PYTHON_BIN:-python}"

finish() {
    local status=$?
    trap - EXIT

    if [[ "${NO_PAUSE:-0}" != "1" && -t 0 && -t 1 ]]; then
        if [[ "$status" -eq 0 ]]; then
            printf '\nBuild and installation completed successfully.\n'
        else
            printf '\nBuild or installation failed with exit code %s.\n' "$status" >&2
        fi
        read -r -p "Press Enter to close..."
    fi

    exit "$status"
}

trap finish EXIT

NO_PAUSE=1 "$ROOT_DIR/scripts/build-wheel.sh"

PYTHON_PATH="$(command -v "$PYTHON_BIN")"
WHEEL_PATH="$(find "$ROOT_DIR/dist" -maxdepth 1 -type f -name 'dirdb_rust-*.whl' -printf '%T@ %p\n' | sort -nr | head -n 1 | cut -d' ' -f2-)"

if [[ -z "$WHEEL_PATH" ]]; then
    printf 'No DirDB wheel was generated.\n' >&2
    exit 1
fi

"$PYTHON_PATH" -m pip install --no-deps --force-reinstall "$WHEEL_PATH"
printf 'Installed: %s\n' "$WHEEL_PATH"
