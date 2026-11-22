#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
npm ci
(
  cd vendor/pickle
  pnpm install --frozen-lockfile
  pnpm --filter @beeper/pickle --filter @beeper/pickle-state-file build
)
cargo build --release --locked
printf '%s\n' 'Built Chord. Launch with ./scripts/launch.sh.'
