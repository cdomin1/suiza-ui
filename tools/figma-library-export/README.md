# Local Figma library bridge

This development plugin exports a local Suiza library JSON, without network calls, API keys or modifications to the Figma source. It is not a published Figma marketplace plugin. Export only documents/assets you are authorized to use. Official kits remain subject to their publishers' terms; do not redistribute them with Suiza-UI.

1. In Figma's desktop app, use Plugins → Development → Import plugin from manifest and choose this folder's `manifest.json`.
2. Open your own copy of a kit, then select the components to export. With no selection, the exporter finds components on the current page; if none exist, it uses top-level page nodes.
3. Run Export library to Suiza-UI and choose Export library.
4. In Suiza-UI, switch the left pane to Assets → Import and choose the downloaded `suiza-library.json`.

Simple rectangles, ellipses, solid fills, uniform text styles, component contents and supported SVG Bézier paths remain editable. Auto layout becomes the currently computed geometry; instances become independent contents. Complex paints/borders, effects, transformed nodes, unsupported vector commands and nested clipping render as PNG layers and are listed in Import notes. Mixed text style runs also render as PNG. Missing fonts use Suiza's fallback while retaining the family name. Hidden nodes are omitted. Component links, constraints, responsive layout rules, variables and interactions are not exported.

The bridge uses only the current page; select individual variants for large component sets. Output has a 256 MB importer limit. Preview/image export is limited to 2048 pixels per longest side. Live Figma-host testing and full official Material-kit verification remain outstanding; independent API fixtures verify geometry, styles, fallbacks and import compatibility.

Run `node test.cjs` to verify the exporter without Figma. Set `SUIZA_BRIDGE_FIXTURE` to generate an independent JSON fixture for the Rust importer test.
