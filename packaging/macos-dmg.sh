#!/bin/bash
set -euo pipefail

binary=$1
arch=$2
output=$3
version=$("$binary" --version)
version=${version#chord }
work=$(mktemp -d)
stage="$work/image"
app="$stage/Chord.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
install -m 755 "$binary" "$app/Contents/MacOS/chord-client"
install -m 755 packaging/macos-launcher "$app/Contents/MacOS/Chord"
cp LICENSE assets/*-LICENSE.txt assets/macos/Chord.icns "$app/Contents/Resources/"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>Chord</string>
  <key>CFBundleIconFile</key><string>Chord</string>
  <key>CFBundleIdentifier</key><string>com.github.maxlang7.chord</string>
  <key>CFBundleName</key><string>Chord</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>15.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
codesign --force --sign - "$app/Contents/MacOS/chord-client"
codesign --force --sign - "$app"
codesign --verify --deep --strict "$app"
"$app/Contents/MacOS/Chord" --demo --snapshot "$work/demo.txt"
test -s "$work/demo.txt"
mkdir -p "$output"
ln -s /Applications "$stage/Applications"
hdiutil create -volname Chord -format UDZO -srcfolder "$stage" -ov -quiet \
    "$output/Chord-macOS-$arch.dmg"
hdiutil verify "$output/Chord-macOS-$arch.dmg"

# Exercise the same signed bundle from a separate Applications directory.
# The temporary HOME keeps the test away from existing profiles and settings.
installed="$work/Applications/Chord.app"
mkdir -p "$work/Applications" "$work/home"
ditto "$app" "$installed"
python3 - "$installed/Contents/MacOS/Chord" "$work/home" <<'PYTHON'
import json
import os
import subprocess
import sys
from pathlib import Path

binary, home = sys.argv[1:]
env = dict(os.environ, HOME=home)
subprocess.run([binary, "--demo", "--appearance-smoke-test", "write"],
               env=env, check=True, timeout=30)
settings = Path(home) / "Library/Application Support/Chord/ui.json"
if json.loads(settings.read_text())["theme"] != "macos":
    raise RuntimeError("Chord did not save its appearance settings")
subprocess.run([binary, "--demo", "--appearance-smoke-test", "check"],
               env=env, check=True, timeout=30)
PYTHON
