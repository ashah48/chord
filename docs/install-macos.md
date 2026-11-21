# Install Chord on a Mac

Requires macOS 15 Sequoia or later. Open **Apple menu → About This Mac** and
choose the disk image that matches your Mac.

| About This Mac shows | Disk image |
| --- | --- |
| Chip: Apple M1, M2, M3, or another Apple M-series chip | `Chord-macOS-arm64.dmg` |
| Processor: Intel | `Chord-macOS-x86_64.dmg` |

1. Download the matching file from [Chord's GitHub releases](https://github.com/maxlang7/chord/releases). If there is no disk image, see the build instructions below.
2. Double-click the downloaded disk image. Drag **Chord** onto **Applications**.
3. Open **Applications → Chord**. With no backend configuration, Chord opens with sample conversations.
4. If macOS cannot verify the developer, confirm the download came from the official release before deciding whether to allow it. These preview builds are ad hoc signed and have no Apple notarization. [Apple's instructions for allowing an individual app](https://support.apple.com/en-us/102445)

## Live accounts

The disk image contains the desktop client. Live accounts need a separate Chord
API accessible on the Mac at `127.0.0.1:4789`, with an API key that matches the
backend. With that backend running, put a JSON file containing the matching
`apiKey` at `~/Library/Application Support/Chord/config.json` and reopen Chord.
Keep that file private. The existing backend bootstrap and phone bridge target
Linux, so installing the disk image alone gives you demo mode. There is no
supported one-click Mac setup for live accounts yet.

## Build or verify the preview

An assistant can use `sw_vers -productVersion` to check the OS and
`sysctl -n hw.optional.arm64` to detect Apple silicon. A result of `1` means
Apple silicon, including when Terminal runs under Rosetta. Otherwise use the
Intel image on an Intel Mac.

A repository maintainer can run **Actions → macOS disk images → Run workflow**
and download either DMG from the successful run's artifacts. To build on a Mac
with stable Rust and Apple's command-line tools, run these from the repository
root in a native Terminal session.

```sh
export MACOSX_DEPLOYMENT_TARGET=15.0
cargo build --locked --release --bin chord
bash packaging/macos-dmg.sh target/release/chord "$(uname -m)" dist
```

Open the installed app and check that sample conversations appear. To test
without opening a window, run the following from Terminal. The file should
contain a rendered inbox.

```sh
scratch=$(mktemp -d)
/Applications/Chord.app/Contents/MacOS/Chord --demo --snapshot "$scratch/inbox.txt"
test -s "$scratch/inbox.txt"
```
