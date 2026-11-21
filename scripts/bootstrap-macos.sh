#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
umask 077

if [[ $(uname -s) != Darwin ]]; then
    echo 'Run this setup on macOS.' >&2
    exit 1
fi

bin="$PWD/.venv/bin"
python3 -m venv .venv
"$bin/python" -m pip install 'matrix-synapse==1.150.0'
git submodule update --init vendor/pickle vendor/ai-bridge
patch="$PWD/patches/pickle-local-history.patch"
if ! git -C vendor/pickle apply --reverse --check "$patch"; then
    git -C vendor/pickle apply "$patch"
fi
npm ci
(cd vendor/pickle && pnpm install --frozen-lockfile &&
    pnpm --filter @beeper/pickle --filter @beeper/pickle-state-file build)

sources=$(mktemp -d)
build_bridge() {
    local name=$1 tag=$2 revision=$3
    local source="$sources/$name"
    git clone --depth 1 --branch "$tag" "https://github.com/mautrix/$name.git" "$source"
    (
        cd "$source"
        test "$(git rev-parse HEAD)" = "$revision"
        if [[ $name = slack ]]; then
            git apply "$OLDPWD/patches/slack-login-version-data.patch"
        fi
        go build -tags goolm -o "$bin/mautrix-$name" "./cmd/mautrix-$name"
    )
}
build_bridge slack v0.2604.0 afdcad0911a7b73689c0837890e127434d832d46
build_bridge whatsapp v0.2610.0 1067291a89f4949887b328104005abe4f7163ddf
