#!/bin/bash
set -euo pipefail
scratch=$(mktemp -d)
HOME="$scratch" sh scripts/start-macos.sh &
backend=$!
trap 'kill "$backend" 2>/dev/null || true; wait "$backend" || true' EXIT
data="$scratch/Library/Application Support/Chord/local"
for ((attempt=0; attempt<90; attempt++)); do
    kill -0 "$backend"
    if [[ $(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:4789/api/status || true) = 401 ]]; then
        node --input-type=module - "$data/config.json" <<'JS'
import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const config = JSON.parse(await readFile(process.argv[2]));
const headers = {Authorization: 'Bearer ' + config.apiKey};
const response = await fetch('http://127.0.0.1:4789/api/status', {headers});
assert.equal(response.status, 200);
const status = await response.json();
assert.equal(status.userID, '@chord:chord.local');
assert.deepEqual(status.networks.map(network => network.id).sort(), ['slack', 'whatsapp']);
assert.deepEqual(status.mail, []);
const inbox = await fetch('http://127.0.0.1:4789/api/inbox?source=iphone&fresh=true', {headers});
assert.equal(inbox.status, 200);
assert.deepEqual((await inbox.json()).errors, []);
JS
        for port in 4811 4812; do
            ready=no
            for ((probe=0; probe<30; probe++)); do
                if nc -z 127.0.0.1 "$port"; then ready=yes; break; fi
                kill -0 "$backend"
                sleep 1
            done
            test "$ready" = yes
        done
        exit 0
    fi
    sleep 1
done
exit 1
