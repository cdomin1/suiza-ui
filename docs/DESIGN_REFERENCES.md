# Visual direction from supplied Sketch screenshots

SUIZA uses the references for interaction and visual language, with its own branding and cross-platform command structure.

- Quiet light gray canvas; compact white sidebars with fine separators.
- Floating white toolbar capsules, restrained shadows, uniform thin vector icons.
- Separate document, page, and object selection states.
- Artboards at the tree root, folder groups, indented typed objects, disclosure arrows, visibility and locks at the row edge.
- Sidebar search pinned to the bottom; long names elide without expanding the pane.
- Inspector changes with selection. Use compact gray pill fields and consistent section spacing.
- Purple object selections and thin canvas outlines with small square handles.
- Rounded command menus with grouped actions and keyboard hints; adapt commands to implemented cross-platform features.

Implemented in the current pass: pages, fixed-size artboards, one-level organizational folders, layer visibility/locks, shared vector icon drawings, floating toolbar capsules, platform-specific menus, legacy JSON migration, and sidebar layout regression coverage.

References reserved for future implementation: variable artboard sizes and device presets, graphic presets, shape-picker expansion (lines/arrows/triangles/stars/polygons), nested group transforms and symbols/components. These reference screenshots are not a claim that those features are implemented.

The latest pass adds compact typography, color and swatch controls, shadows, five effects, six export formats, and a startup document chooser. macOS uses the native global File menu; Windows/Linux retain in-window commands. Figma informs the compact workspace/recent-document header, while Sketch informs floating canvas tools and inspector fields.
