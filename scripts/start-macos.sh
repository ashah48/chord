#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
umask 077
export PATH="$PWD/.venv/bin:$PATH"
export CHORD_DATA_DIR="$HOME/Library/Application Support/Chord/local"
export CHORD_MACOS_LOCAL=1
if [ "${1:-}" = "--messages" ]; then
    export CHORD_MACOS_MESSAGES=1
fi
export PORT=4789
exec python3 scripts/start-macos.py
