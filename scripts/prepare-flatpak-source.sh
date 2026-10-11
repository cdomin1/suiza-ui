#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p dist
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT HUP INT TERM
git archive HEAD | tar -x -C "$stage"
mkdir -p "$stage/.cargo"
# Dependencies are downloaded before entering Flatpak's offline build sandbox.
(cd "$stage" && cargo vendor --locked cargo-vendor > .cargo/config.toml)
tar -czf dist/flatpak-source.tar.gz -C "$stage" .
