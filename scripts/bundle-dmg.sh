#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { printf 'Build DMGs on macOS.\n' >&2; exit 1; }
version=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
scripts/bundle-macos.sh release
arch=$(uname -m)
stage=$(mktemp -d "${TMPDIR:-/tmp}/suiza-dmg.XXXXXX")
trap 'rm -rf "$stage"' EXIT HUP INT TERM
ditto "dist/SUIZA Canvas.app" "$stage/SUIZA Canvas.app"
cp LICENSE "$stage/LICENSE.txt"
ln -s /Applications "$stage/Applications"
cat > "$stage/Read Me.txt" <<EOF
SUIZA Canvas $version (Alpha) — macOS $arch

Drag SUIZA Canvas into Applications to install.

This Alpha build is ad-hoc signed and is not Apple notarized.
EOF
output="dist/SUIZA-Canvas-${version}-macOS-${arch}.dmg"
hdiutil create -volname "SUIZA Canvas $version" -srcfolder "$stage" -format UDZO -ov "$output"
hdiutil verify "$output"
(cd dist && shasum -a 256 "$(basename "$output")") > "$output.sha256"
printf 'Created %s\n' "$output"
