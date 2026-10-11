# Linux packages

The Linux packages workflow builds on Linux runners when a `v*` tag is pushed. It publishes a GitHub prerelease only after both packaging jobs succeed. Manual runs produce downloadable workflow artifacts without creating a release. The initial target is x86_64.

Installation instructions are in [the release notes](../packaging/LINUX_RELEASE_NOTES.md). The Flatpak bundle uses the standard Freedesktop 25.08 runtime from Flathub, which is installed separately. It is not a Flathub store submission.

To build locally on Linux, install the desktop development dependencies listed in `.github/workflows/linux-release.yml` and a current stable Rust toolchain. For AppImage, obtain the official linuxdeploy and appimagetool AppImages, set `LINUXDEPLOY` and `APPIMAGETOOL` to their executable paths, then run `scripts/bundle-appimage.sh 0.1.0`. Dependencies are bundled with linuxdeploy; graphics drivers remain supplied by the host.

For Flatpak, install flatpak-builder, add Flathub and install the Freedesktop Platform, SDK and rust-stable extension for 25.08. Run `scripts/bundle-flatpak.sh 0.1.0`. Source is taken from the current Git commit and Cargo dependencies are vendored into a temporary source archive before the offline sandbox build. No local documents are included.

The workflow currently downloads official continuous packaging-tool builds and logs their SHA-256 hashes; tools are not yet pinned for byte-for-byte reproducibility. Application dependencies are locked by Cargo.lock. Future distribution work includes ARM64 builds and broader desktop testing.
