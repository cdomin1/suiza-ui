# Suiza-UI

Suiza-UI is the interface design app in the Suiza family of creative apps.

A local Rust desktop design editor inspired by Sketch's compact inspector and Figma's document tabs. The app uses egui/eframe and saves editable documents as JSON.

## Build and run

```sh
cargo run --locked
cargo test --locked
cargo fmt --check
```

On macOS, `scripts/bundle-macos.sh debug` or `scripts/bundle-macos.sh release` builds `dist/Suiza-UI.app`. The bundle uses the white knife icon on #EB5E34. Icon assets are maintained in `assets/`; regenerate platform sizes with `python3 scripts/generate-icons.py` (requires Pillow).

## Current functionality

- Independent document tabs, undo/redo, recent documents and startup templates.
- Pages, custom-sized artboards, layer visibility/locking, groups, shape masks and layer context menus.
- Resizable layer pane; layer labels can be renamed separately from text content; rows can be reordered.
- Rectangles, ellipses, embedded images, text, cubic Bézier paths and compound contours.
- Pen and node editing, Boolean union/subtract/intersect/difference, shape conversion and text outlines.
- Installed system fonts, searchable face previews, In Use / All Fonts, text size/style/spacing/alignment and advanced Text commands.
- Color and saved swatches, corner styles, shadows and five effects.
- PNG, JPG, TIFF, WebP, PDF and SVG export. Text, images and raster effects are embedded as raster content in SVG; converted text outlines and cubic paths remain vector geometry.
- Light/Dark/System appearance, trackpad zoom and scroll, macOS global menus and in-window menus elsewhere.

Use V to select, R for rectangles, O for ellipses, T for text and P for the pen. Double-click canvas text to edit characters; double-click a path to edit nodes. Enter or Escape ends path editing. Double-click a layer name to rename it. Command on macOS, Super on Linux and Ctrl on Windows are the primary command modifiers.

The vendored `egui-winit` patch retains Linux's physical Super key so Linux shortcuts can use Super consistently. Upstream otherwise exposes this key only on macOS.

## Verification and limits

The request-by-request verification record is in [docs/REQUEST_CHECKLIST.md](docs/REQUEST_CHECKLIST.md). The documentation-driven workflow audit and remaining gaps are in [docs/SKETCH_FUNCTIONALITY_AUDIT.md](docs/SKETCH_FUNCTIONALITY_AUDIT.md).

This is an evolving editor, not full Sketch feature parity. Nested group hierarchies, variable artboard dimensions, symbols/components, rich per-range text, text on path, variable font axes, and several advanced vector tools remain incomplete. Native macOS testing is in progress; a build matrix exists for macOS, Linux and Windows, but platform behavior still needs verification on those systems. Installed font files are not embedded in documents.

GitHub publication is held until the requested work and verification are complete.

Application UI typography uses bundled Inter Regular and Bold from the official Inter 4.1 release, with its SIL Open Font License in `assets/fonts/Inter-LICENSE.txt`. Document fonts are selected separately.

## Checkpoint status

This repository captures work in progress, not a completed release. See [the request checklist](docs/REQUEST_CHECKLIST.md) and [next-phase scope](docs/NEXT_PHASE.md). The Assets library browser and native `.suiza-ui` package format remain planned; current documents save as JSON. Hosted sharing, collaboration, and AI features need later work.

## License

Suiza-UI application code is available under [MIT](LICENSE). Third-party fonts and vendored dependencies retain their own licenses; see `assets/fonts/` and `vendor/`.
