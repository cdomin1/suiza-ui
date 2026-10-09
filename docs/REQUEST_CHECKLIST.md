# Suiza request and verification checklist

The user authorized a GitHub checkpoint of the current implementation on 2026-10-09, before the full backlog is complete. Remaining work below is still outstanding.

| Request | Current evidence | Remaining verification / work |
|---|---|---|
| App name Suiza; white knife on #EB5E34 | Bundle, SVG, PNG, ICNS, ICO updated | Remove stale preview; check Dock icon |
| Padding, hierarchy, aligned controls throughout | Shared pane and field sizes; layout tests at 3 window sizes | Inspect all updated panels in light and dark |
| Clean top area, no name/open/save strip, one tool capsule | Implemented in app | Verify tabs and platform menu layout |
| macOS native File menu; Linux/Windows in-window menus | Shared command model | Native submenu organization and platform builds |
| Document tabs | Independent document/view/history test | Unsaved close and keyboard navigation |
| Startup recents/templates | Implemented | Verify saved recent opens and search |
| Color picker 10–15% smaller and swatches | Implemented | Inspect picker editing and swatch persistence |
| PNG/JPG/TIFF/WebP/PDF/SVG; no HEIC | Six valid-format tests | Export from pane and frame context menu |
| Drop/inner shadows and all five effects | Rendered pixel tests | Inspect controls, disabled effects, undo |
| Text selection and I-beam | Canvas character selection verified | Edit, copy/paste, undo and installed-font editing |
| Functioning compact type pane | Font, weight, size, tracking, leading and align implemented | Add advanced typography and inspect final UI |
| Installed system fonts on all platforms | fontdb discovery and face rendering test | Native installed-font chooser, missing-font fallback |
| Text/image/shape right-click menus | Canvas menu verified; shared sidebar commands added | Sidebar and image commands in app |
| Frame right-click menu | Frame clipboard, duplicate, rename, content lock/hide and export added | Verify frame selection and menu in app |
| Group objects and shape masks | Explicit group-row selection; pointer regression verifies whole-group drag versus independent child drag; group/mask coverage tests | Native refreshed-build QA, group resize and mask commands |
| Corner styles and individual values | Six styles and geometry tests | Inspect icon/field spacing; conversion fidelity |
| Trackpad pinch zoom and two-finger pan | Anchor-preserving transforms implemented | Input regression tests and native hardware check |
| Light/Dark/System; dark contrast | Theme text/icon fixes implemented | Inspect rebuilt dark UI and system switching |
| Command / Super / Ctrl shortcuts | Shared shortcuts plus Linux Super event patch | Expanded shortcuts, focus behavior, native tests |
| Accurate Bézier pen and node editing | Cubic splitting and tangent tests | App pen/handle editing, undo, bounds and export |
| Boolean operations and compound paths | Coverage tests with cubic outputs | Complex curved operands, holes and selection UX |
| Convert text to outlines | Glyph-hole and rendered-area test | Shape new typography first, verify native operation |
| Expanded global Text menu | Advanced commands, actual shaping and shared native submenus implemented | Native typography menu and font preview QA |
| Publish a new GitHub repository | Public `cdomin1/suiza-ui` checkpoint authorized; MIT license added | Release backlog remains independent of this checkpoint |

Latest pass (2026-10-09): 30 tests passed. Rebuilt and launched macOS Suiza. Verified keyboard layer rename opens, changes the tree label, and leaves text content intact. Frame inspector is visible with position, dimensions, corners, clip content, background and export. Regression drags rectangles, ellipses, images, text and cubic paths onto the pasteboard and back.

Additional requests: resizable left pane, drag reorder, distinct text-layer labels, In Use / All Fonts with actual face previews, anchored inspector popovers, padded multi-resolution icons, custom frame dimensions. Implementation exists; live font preview, icon optical sizing, persisted sidebar width and full interaction QA remain pending. See SKETCH_FUNCTIONALITY_AUDIT.md for explicit remaining feature gaps. GitHub checkpoint publication is in progress under the user’s updated authorization.

Latest group-selection pass (2026-10-09): 35 tests pass. Group rows are selectable and accessible; selecting a group uses member bounds and moves its unlocked members together. Clicking a child clears group selection and moves that child independently. Group selection persists per document tab. macOS debug bundle rebuilt and relaunched. Native UI verification confirms Navigation group row selects the Group inspector and clicking atelier selects the independent Text inspector. Bundle packaging now replaces the executable inode and signs/verifies the completed bundle, resolving the macOS CODESIGNING Invalid Page launch failure.

UI refinement pass (2026-10-09): Prototype/Preview/Share toolbar moved to canvas top center; Prototype and Share popovers anchor below their buttons. Color, font, toolbar and keyboard-help popovers use outside-click/Escape dismissal with no close button; native dropdown/context menus retain their existing dismissal. Narrow canvases move drawing tools to a second row to avoid overlap. Template preview/title opens Atelier; only the blank card has Create document. All 35 tests pass and signed Suiza-UI.app rebuilt. Native screenshot confirmed toolbar and Share anchor; latest welcome-card wording awaits a running-app restart.

Inter UI pass: bundled Regular/Bold fonts and license; shared cross-platform UI definitions retain separate document font defaults. All 35 tests pass. Current checkpoint publication is now authorized independently of unfinished release work.
