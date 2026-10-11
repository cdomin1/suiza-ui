# Native SUIZA Canvas documents

New documents save as `.suiza-ui`. File > Open and Open Recent read these files directly; users do not need to unzip them. Existing JSON files remain readable and continue saving as JSON. Save As can create a native copy. Documents can also be dropped onto the editor or supplied as a command-line filename.

Version 1 is a ZIP container with stored entries:

- `manifest.json`: format identifier `suiza-ui`, version `1`, document entry name.
- `document.json`: editable pages, frames, layers, vectors, typography and prototype connections.
- `assets/image-N.bin`: original embedded image bytes; each corresponding layer uses an `image_asset` reference instead of a numeric JSON byte array.
- `preview.png`: first-frame preview, at most 512 pixels on its longest side. Preview generation is optional and does not prevent saving a valid document.

The loader checks version, geometry, image dimensions and archive size. Entries are read in memory and never extracted. Failed opens preserve the current document; failed writes preserve the previous saved file and dirty state. The existing five-second session recovery remains independent of manual document saves.

Fonts are referenced by family and style rather than embedded. Bundled Inter is available on every platform. Missing installed font families are reported when opening a document, and their original names are retained while fallback faces render the content.

Native format disk tests verify edits through save, tab close and reopen, plus damaged-file handling and embedded-image round trips. Live macOS verification also confirms Command-S saves a native package, closing its tab and reopening through File Open restores the inserted editable group and children. Finder open-file event handling and Windows installer associations remain unfinished; Linux bundles include MIME metadata for `.suiza-ui`, with installation/registration still required on the destination machine.
