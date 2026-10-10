#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --release --locked
bundle=dist/suiza-linux
mkdir -p "$bundle/bin" "$bundle/share/applications"
cp target/release/suiza "$bundle/bin/suiza"
mkdir -p "$bundle/share/licenses/suiza"
cp assets/fonts/*LICENSE*.txt "$bundle/share/licenses/suiza/"
cp packaging/suiza.desktop "$bundle/share/applications/suiza.desktop"
mkdir -p "$bundle/share/mime/packages"
cp packaging/suiza-ui.xml "$bundle/share/mime/packages/suiza-ui.xml"
for size in 16 24 32 48 64 128 256 512 1024; do
    folder="$bundle/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$folder"
    cp "assets/icons/$size/suiza.png" "$folder/suiza.png"
done
printf 'Created %s\n' "$bundle"
