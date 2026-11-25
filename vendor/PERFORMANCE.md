# Pickle performance checks

Measured locally on an Intel i5-1135G7, using Go's native test runner. Results
below are medians of three runs, five iterations per run. These are synthetic
benchmarks, not measurements of the running app or a production homeserver.

| Workload | Before | After |
| --- | ---: | ---: |
| Fetch 50 messages, with 2 ms simulated latency per HTTP request | 129 ms | 44 ms |
| Process an unstored state event, with 1,000 members cached | 2.27 ms | 0.00063 ms |
| Snapshot writes per unstored state event | 1 | 0 |
| Bytes allocated per unstored state event | 149,129 | 16 |

History lookups previously waited for each message's edit request sequentially.
They now use at most four workers. Decryption, edit cache updates and message
conversion remain sequential. Repeated IDs share a lookup, and page deduplication
uses an index instead of repeatedly scanning the page. Thread replies, cached
edits, pagination, sorting and fallback after failed edit requests are covered by
regression tests. Cancellation stops workers from starting further requests.

The history benchmark still makes the same 51 HTTP requests for 50 distinct
unedited messages. It does not cache negative edit results or skip freshness
checks. Concurrency trades some transient allocations for latency: measured
allocations per page increased from about 910 KB to 1,120 KB in the native HTTP
benchmark. Exact improvements depend on network latency and server limits.

The state store previously saved its full snapshot after every UpdateState call,
including state events the underlying store ignores. It now dispatches through
the existing persisting setters. Relevant changes are still written immediately,
using the same storage format. No encryption algorithms or key storage changed.

From the Chord root, inside `nix-shell`:

```sh
cd vendor/pickle/packages/pickle/native
go test -p 3 -tags goolm ./...
go test -race -p 3 -tags goolm ./internal/core
go test -p 3 -tags goolm ./internal/core -run '^$' -bench BenchmarkChord -benchtime=5x -count=3
```

Build and check the actual JavaScript/WebAssembly path from the Chord root:

```sh
cd vendor/pickle
pnpm --filter @beeper/pickle --filter @beeper/pickle-state-file build
pnpm --filter @beeper/pickle exec vitest run
node scripts/chord-history-smoke.mjs
```

The smoke check uses a fake fetch implementation and a synthetic account. It
confirms the compiled WASM returns messages in order, retains the cursor and
limits concurrent edit lookups to four, without accessing real accounts.
