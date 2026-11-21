#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    printf 'Usage: sh %s user@chord-host\n' "$0" >&2
    exit 2
fi

backend=$1
config_dir="$HOME/Library/Application Support/Chord"
config="$config_dir/config.json"
umask 077
mkdir -p "$config_dir"

if [ ! -f "$config" ]; then
    # Read the key from the default checkout without copying backend credentials.
    key_json=$(ssh -T -- "$backend" /bin/sh <<'REMOTE'
set -eu
cd "$HOME/chord"
nix-shell -p python3 --run 'python3 -' <<'PYTHON'
import json
from pathlib import Path

key = json.loads(Path("private/config.json").read_text())["apiKey"]
if not isinstance(key, str) or not key:
    raise SystemExit("Missing Chord API key")
print(json.dumps({"apiKey": key}))
PYTHON
REMOTE
)
    test -n "$key_json"
    (set -C; printf '%s\n' "$key_json" > "$config")
fi

printf '%s\n' 'Leave this terminal open while using Chord. Press Ctrl+C to disconnect.'
exec ssh -NT -o ExitOnForwardFailure=yes -L 127.0.0.1:4789:127.0.0.1:4789 -- "$backend"
