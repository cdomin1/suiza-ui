# Local asset libraries

The left pane has Layers and Assets views. Assets offers searchable, virtualized previews of the current document's frames/groups, six original Suiza starter assets, and imported local libraries. Import runs on a worker thread so parsing/decompression does not block the editor. Imported libraries and conversion notes are saved with preferences. Removing a library removes its registration, not the source file.

## Supported imports

- `.suiza-ui` and legacy document JSON.
- `.sketch` ZIP documents, including deflated entries. Source files remain unchanged. Pages, artboards/symbol masters, rectangles/ovals, cubic paths, solid fills, first-run typography and embedded bitmaps are converted to editable layers. Symbol instances are expanded using available local/foreign masters and direct text overrides. Nested groups are flattened to the editor's existing grouping model.
- Figma bridge JSON (`suiza-library`, version 1). The local development plugin in `tools/figma-library-export` exports the current selection/page. Native `.fig` binary files and Figma URLs are not direct import formats. No hosted backend or API token is needed.

Clicking an asset inserts an independent editable group at the active frame's center. Insertions clear prototype IDs/locks, choose unique group names and support undo. These are independent copies, not linked instances or overrides.

## Conversion limits

Sketch import notes identify unsupported rotation/flips, transparency, gradients/patterns, multiple fills/borders, shadows/blur, masks, Boolean shape groups, nested hierarchy, mixed text runs, unresolved external symbols and non-text symbol overrides. Unsupported classes are skipped; supported approximations are explicitly reported. Auto layout/resizing constraints, shared-style links, variables and prototype connections are not imported. Invalid geometry, missing images, unsafe references, excessive nesting and oversized archives reject the import without replacing the editor document or adding a partial library. Frames larger than 8192 points are omitted with a note. Large kits should be imported as smaller component documents.

The Figma bridge preserves simple layers and rasterizes complex appearance; see its README for exact limitations. Official-kit visual fidelity is not yet verified. Tests use independently authored fixtures; a local Sketch sample is also exercised without copying it into the repository.

## Official platform kits

Suiza-UI links to publisher pages instead of bundling or redistributing their assets:

- Apple iOS/iPadOS and macOS: https://developer.apple.com/design/resources/
- Material 3: https://m3.material.io/get-started

Apple's official Sketch download presents its Design Resources license. Its displayed terms restrict use to Apple-platform UI mockups and prohibit embedding the resources in software and unauthorized redistribution. The agreement was left unaccepted during development. Publisher assets and proprietary fonts are excluded from Suiza-UI's source and distribution; users obtain their own copies under applicable publisher terms. The original Suiza starter assets are not Apple/Google kits.
