# Prototype and Share

Implemented on 2026-10-09. The backend is explicitly deferred by the user.

A separate floating pill offers **Prototype**, **Preview**, and **Share**. The native macOS Prototype menu and in-window menu on other platforms expose the same commands. Preview uses Command+Enter on macOS, Super+Enter on Linux, and Ctrl+Enter on Windows. Edit interactions uses the same platform modifier with Shift+P.

Select a frame or layer, open Prototype, and drag its connection handle to a destination frame, or choose the destination in the interaction panel. Define named flows and their home screens. Connections are saved in the document using stable IDs, preserving them when names and tree order change. Each source supports multiple interactions. Supported triggers are tap/click, hover, drag, keyboard key and time delay. Supported actions are transition, overlay, previous artboard, close overlay, HTTP(S) URL and scroll to Y. Transition animations include dissolve and directional slides, with configurable duration. Overlay position is configurable.

Preview opens the same self-contained browser player used by HTML export. Export includes only screens reachable from the chosen home and embeds rendered screens and interactions, with no external dependencies. The design rendering matches frame export, including embedded images and installed-font rendering. Screen pixels are embedded; source fonts do not need to be installed on the viewer’s device.

Share offers Design Review, Development, Presentation, User Testing and Custom presets. Development includes dimensions, coordinates, color and typography inspection and copying basic CSS values. Review notes are local to the viewer’s browser and can be downloaded as JSON. They are **not shared/synchronized comments**. Full screen requires a viewer click because browser security policies prevent automatic full-screen entry.

This is an initial Adobe XD-inspired workflow, not complete XD parity. Auto-animate, component states, media, voice/gamepad inputs, recording, hosted URLs, password/private access and synchronized review remain future work. Copied objects currently receive new IDs; outgoing interactions are not automatically copied to duplicates. Connections to deleted screens must be reassigned. A deleted flow home produces a clear export error.

References:
- [Adobe — Create interactive prototypes](https://helpx.adobe.com/xd/desktop/prototype/create-prototypes.html)
- [Adobe — Share designs and prototypes](https://helpx.adobe.com/xd/desktop/share-export-and-review/share-designs-prototypes.html)

Validation: regression tests cover stable IDs across rename/reorder and JSON round-trip, reachable-screen export, embedded images, safe user-string serialization, deleted-home errors and preset behavior. Browser checks confirmed hotspot navigation, history/back, specification inspection and local note creation. Native UI refresh remains pending while the currently running app contains unsaved edits.
