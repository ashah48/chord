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

Run the backend on a dedicated NixOS machine or VM reachable by SSH from
the Mac. Use a fresh checkout with your dad's own account data. Chord stores
message history on the backend and uses fixed ports. If the host already runs
your Chord instance, give his instance its own VM.

### Start the backend on NixOS

Run these commands as the backend owner on a NixOS machine with Git and SSH
access from the Mac.

```sh
git clone --branch v0.3.0-macos-preview https://github.com/maxlang7/chord.git "$HOME/chord"
cd "$HOME/chord"
nix-shell shell.nix --run 'bash scripts/bootstrap.sh'
nix-shell shell.nix --run 'python3 scripts/launch-client.py --backend-only'
```

Keep the last command running. The first startup can take several minutes.
If startup fails, check `private/desktop-runtime.log` and
`private/desktop-api.log` on NixOS. Keep the backend's `private` directory on
that machine.

### Connect the Mac

Download [connect-macos-backend.sh](../packaging/connect-macos-backend.sh)
from the repository using **Download raw file**. Quit Chord. In Mac Terminal,
replace `dad@chord-host` with your SSH login and run the script.

```sh
sh "$HOME/Downloads/connect-macos-backend.sh" dad@chord-host
```

The script assumes the backend checkout is at `~/chord` on NixOS. On the
first run it copies the API key over SSH into
`~/Library/Application Support/Chord/config.json`. It leaves an existing
configuration alone. It then opens an SSH tunnel on the Mac's loopback
interface, so the Chord API can stay private on the backend.

Keep the tunnel terminal open and launch Chord from Applications. To check
the tunnel from another terminal, run this command. Expect `401` because
the probe omits the API key.

```sh
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:4789/api/status
```

Run the same script after a reboot or whenever you close the tunnel. If you
switch to a different backend, move the existing Mac config aside first so
the script fetches the new key.

### Connect his accounts

- **Slack.** Open **Accounts → Slack → Connect** and follow Chord's sign-in
  prompts for his workspace. The bridge uses a Slack web session. If Chord
  asks for session credentials, enter them in its sign-in dialog. Workspace
  policy may prevent the bridge from connecting. Check for **Connected**,
  then open the Slack inbox.
- **WhatsApp, if he uses it.** Open **Accounts → WhatsApp → Connect**.
  On his phone, open WhatsApp's **Linked devices → Link a device** and scan
  Chord's QR code. Check for **Connected** in Chord.
- **iMessage.** Use Apple's Messages app on the Mac for iMessage. Chord's
  iPhone integration uses BlueFerry over Bluetooth on Linux and cannot
  access the Mac's Messages history.

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
