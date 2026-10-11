# SUIZA Canvas plugin API v1

Open **Edit → Plugins…**, choose Install plugin, review the permission and choose Allow and install. Commands run only when clicked. Installation persists locally; Uninstall removes an extension. Changes participate in document Undo and normal saving.

API v1 is a deliberately limited **declarative command API**, not a JavaScript runtime. Third parties distribute JSON manifests. There is no network, shell, arbitrary filesystem access, automatic execution, or access to unselected layers. No backend is needed. Custom plugin panels, scripting and marketplace distribution remain future work.

See [the working example](../examples/plugins/blue-labels.json). Required manifest fields: `api_version` (1), `id` (ASCII identifier), `name`, `author`, `permissions` (exactly `["selection.write"]`), and `commands`. Each command has `name` and an `operations` array.

Supported operations, applied to selected unlocked layers:

| type | parameters | behavior |
| --- | --- | --- |
| rename | prefix, suffix | Change layer label, preserve text content |
| fill | color: [r,g,b] | Set RGB color |
| move | x, y | Offset position in frame coordinates |
| text_size | size | Set text size; skip other layer types |

Commands apply to a copy and validate the entire document before committing. Errors leave the current document intact. Duplicate selected indices are deduplicated. Limits: 1 MB per manifest, 50 commands, 100 operations per command, movement ±8192, text sizes 1–512. Unknown fields, versions and capabilities are rejected. Plugin author names are self-declared, not signatures or verification.
