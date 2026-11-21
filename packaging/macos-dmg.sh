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
