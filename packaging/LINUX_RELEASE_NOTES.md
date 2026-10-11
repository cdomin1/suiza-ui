SUIZA Canvas **Alpha 0.1.0-alpha.3** for **x86_64**.

- **AppImage:** download, run `chmod +x SUIZA-Canvas-*.AppImage`, then launch it. Built on Ubuntu 22.04; requires a working X11/Wayland desktop and OpenGL drivers. If FUSE is unavailable, launch with `--appimage-extract-and-run`.
- **Flatpak:** add Flathub if needed, then run `flatpak install --user ./SUIZA-Canvas-*.flatpak`. Launch with `flatpak run io.github.cdomin1.SuizaCanvas`. The Freedesktop 25.08 runtime is downloaded separately. The sandbox has home-directory access for saving documents, importing local assets and CSV image paths; no network permission is granted.
- **Documents:** new files use `.sza`; existing `.suiza-ui` and JSON documents remain readable.
- **Checksums:** verify downloads with `sha256sum -c SHA256SUMS` from the directory containing both packages.

This is an Alpha development prerelease. Automated Linux tests, package construction/install checks and window-launch smoke tests for both formats run before publication. These checks do not replace interactive testing across Linux distributions. Backend sharing, AI and a plugin scripting runtime remain deferred.
