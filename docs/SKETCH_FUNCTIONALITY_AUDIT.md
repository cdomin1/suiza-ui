# Sketch workflow audit for Suiza

Reviewed official documentation on 2026-10-09. This is a functional acceptance list, not a claim of full Sketch feature parity.

Sources:
- [Layer List](https://www.sketch.com/docs/interface-and-settings/the-mac-app-interface/the-layer-list/)
- [Renaming layers](https://www.sketch.com/docs/designing/layer-basics/renaming-layers/)
- [Vector editing](https://www.sketch.com/docs/designing/vector-editing/)
- [Text](https://www.sketch.com/docs/designing/text/)
- [Mac shortcuts](https://www.sketch.com/docs/shortcuts/mac/)
- [Apple app icons](https://developer.apple.com/design/human-interface-guidelines/app-icons/)
- [Microsoft icon resources](https://learn.microsoft.com/en-us/windows/win32/menurc/about-icons)

## Layer organization

Acceptance: rename a layer from its row, context menu or Edit menu; retain the original text content when renaming a text layer; renamed names remain searchable and survive save/load. Reorder objects through the layer tree, including movement between frames. Selected objects can move outside frame boundaries and remain visible/selectable. Groups move and duplicate together; locks block edits. Preserve undo for each operation.

Implemented this pass: distinct optional layer labels, double-click rename, context/Edit Rename command with platform modifier + R, drag reorder, press-origin drag selection, pasteboard drawing/hit-testing, resizable left pane.

Still to verify in the running app: inline/name focus behavior, dragging between collapsed and expanded frame trees, group selection affordances, undo after reordering, saved pane width. Nested groups and moving layers across pages require more model work.

## Fonts and typography

Acceptance: document-specific In Use tab, searchable All Fonts tab, actual face previews, installed fonts on each platform, same shaping in canvas/export/outlines, direct text selection and editing. Popover aligns with the inspector edge.

Implemented: virtualized face-preview browser, installed font discovery and registration, anchored font browser, advanced Text menu, real ligature/kerning/OpenType shaping, font-family persistence and fallback. Font listing renders only visible rows to avoid loading every installed font simultaneously.

Remaining: native visual verification, rich text with per-range styling, variable font axes, text on path, fully resolved mixed-direction Unicode paragraphs, font-weight inventory beyond regular/bold/italic. These must not be described as implemented.

## Vector editing

Acceptance: pen creates lines and cubic curves; close or finish returns to selection; selected paths drag normally. Editing exposes independent/mirrored/tangent handles, numeric points, insertion and deletion. Boolean operations and compound holes survive SVG export. Text outlines retain glyph contours.

Implemented: cubic paths, de Casteljau insertion, tangent editing, compound contours, four Boolean operations, glyph outlines, reverse all contours, closure exits editing. Regression reproduces rapid drags onto the pasteboard and back for all object kinds.

Remaining: native pen/handle verification, multi-node marquee selection, joining/splitting paths, segment bending, stroke outlines and additional stroke cap/join options. Non-round corner conversion now preserves each style, with smooth/inside-arc conversion sampled at 64 steps per corner; this is an approximation and should be assessed at high zoom.

## UI and platform packaging

Acceptance: equal pane padding and aligned field heights, quiet separators, light/dark contrast, consistent anchored popovers, clean toolbar, startup without redundant header divider. Icon silhouette should match neighboring apps at Dock/taskbar sizes.

Implemented: startup divider removed, thumbnail clipping/centering, shared vector pen icon, common layer action sizing, 96px master icon inset at 1024px, macOS ICNS and multi-resolution Windows ICO, Linux PNG assets 16–1024px, shared saved swatches. Platform sizing assets are generated reproducibly by scripts/generate-icons.py.

Remaining: rebuild/restart after preserving the user's unsaved document, inspect live icon and panels, actual Linux/Windows packaging and input QA. Having all icon representations does not by itself prove each platform installer uses them.
