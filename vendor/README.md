# Included dependencies

These libraries are tracked directly in Chord. A regular clone contains their
source, including Chord's changes. Bootstrap builds them without fetching Git
submodules or applying patches. Package managers still download build dependencies.

## Pickle

- Upstream: https://github.com/beeper/pickle
- Imported revision: `b662c4ad6eb86987bb65084615d1d0d0728e56fd`
- License: see `pickle/LICENSE`.
- Chord changes preserve thread replies in message history, find encrypted media
  replacements, and use the included ai-bridge source through a Go module
  replacement. Regression tests are included in the native core package.
- History edit lookups use four concurrent requests with duplicate lookups
  removed. State events only write snapshots when they affect stored state.
  See `PERFORMANCE.md` for measurements and verification commands.

## ai-bridge

- Upstream: https://github.com/beeper/ai-bridge
- Imported revision: `930c6228cb30fab20bb4617da1825676e0e4c439`
- Source is unchanged from that revision.

Keep these revisions and the description of local changes current when updating
the included source. Build outputs and installed packages are not included.
