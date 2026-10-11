#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
version=${1:-0.1.0}
app_id=io.github.cdomin1.SuizaCanvas
scripts/prepare-flatpak-source.sh
flatpak-builder --user --force-clean --repo=dist/flatpak-repo dist/flatpak-build packaging/flatpak/$app_id.json
flatpak build-bundle --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo dist/flatpak-repo "dist/SUIZA-Canvas-${version}-x86_64.flatpak" "$app_id"
