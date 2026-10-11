#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
profile="${1:-release}"
case "$profile" in
  release) cargo build --release --locked ;;
  debug) cargo build --locked ;;
  *) printf 'Usage: %s [release|debug]\n' "$0" >&2; exit 2 ;;
esac
bundle="dist/SUIZA Canvas.app"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
# Replace the executable with a new inode: overwriting a previously launched
# Mach-O can leave macOS with cached code-signing pages from the old build.
cp "target/$profile/suiza" "$bundle/Contents/MacOS/Suiza.new"
rm -f "$bundle/Contents/MacOS/Suiza"
mv -f "$bundle/Contents/MacOS/Suiza.new" "$bundle/Contents/MacOS/Suiza"
cp assets/suiza.icns "$bundle/Contents/Resources/Suiza.icns"
mkdir -p "$bundle/Contents/Resources/Licenses"
cp assets/fonts/*LICENSE*.txt "$bundle/Contents/Resources/Licenses/"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>SUIZA Canvas</string>
<key>CFBundleDisplayName</key><string>SUIZA Canvas</string>
<key>CFBundleExecutable</key><string>Suiza</string>
<key>CFBundleIdentifier</key><string>local.suiza.studio</string>
<key>CFBundleIconFile</key><string>Suiza.icns</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
# Local builds use an ad-hoc signature; distribution signing is separate.
codesign --force --sign - "$bundle"
codesign --verify --deep --strict "$bundle"
printf 'Created %s\n' "$bundle"
