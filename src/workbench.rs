//! Offline creative workflows. All generators return independent frames before mutation.
use crate::{Artboard, Document, Kind, Layer};
use ab_glyph::{Font, ScaleFont};
use eframe::egui::{self, Color32};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    pub name: String,
    pub created: u64,
    pub snapshot: String,
}
const MAX_HISTORY: usize = 20;
const MAX_HISTORY_BYTES: usize = 64 * 1024 * 1024;
pub fn checkpoint(doc: &mut Document, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 120 {
        return Err("Use a checkpoint name of 1–120 characters".into());
    }
    let mut snapshot = doc.clone();
    snapshot.checkpoints.clear();
    let snapshot = serde_json::to_string(&snapshot).map_err(|e| e.to_string())?;
    if snapshot.len() > MAX_HISTORY_BYTES {
        return Err("This document is too large for an embedded checkpoint".into());
    }
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    doc.checkpoints.push(Checkpoint {
        name: name.into(),
        created,
        snapshot,
    });
    while doc.checkpoints.len() > MAX_HISTORY
        || doc
            .checkpoints
            .iter()
            .map(|c| c.snapshot.len())
            .sum::<usize>()
            > MAX_HISTORY_BYTES
    {
        doc.checkpoints.remove(0);
    }
    Ok(())
}
pub fn restore(doc: &Document, index: usize) -> Result<Document, String> {
    let point = doc
        .checkpoints
        .get(index)
        .ok_or("Checkpoint no longer exists")?;
    let mut restored = Document::from_json(&point.snapshot)?;
    restored.checkpoints = doc.checkpoints.clone();
    checkpoint(&mut restored, "Before restoring checkpoint")?;
    // The recovery checkpoint must hold the current document, not the restored one.
    let mut current = doc.clone();
    current.checkpoints.clear();
    restored.checkpoints.last_mut().unwrap().snapshot =
        serde_json::to_string(&current).map_err(|e| e.to_string())?;
    validate_history(&restored.checkpoints)?;
    Ok(restored)
}
pub fn validate_history(points: &[Checkpoint]) -> Result<(), String> {
    if points.len() > MAX_HISTORY
        || points.iter().map(|p| p.snapshot.len()).sum::<usize>() > MAX_HISTORY_BYTES
    {
        return Err("Checkpoint history exceeds its limit".into());
    }
    if points
        .iter()
        .any(|p| p.name.trim().is_empty() || p.name.chars().count() > 120)
    {
        return Err("Invalid checkpoint name".into());
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Rule {
    Scale,
    Fixed,
    Left,
    Right,
    Center,
    Stretch,
    Cover,
}
impl Rule {
    const ALL: [Self; 7] = [
        Self::Scale,
        Self::Fixed,
        Self::Left,
        Self::Right,
        Self::Center,
        Self::Stretch,
        Self::Cover,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Scale => "Scale proportionally",
            Self::Fixed => "Keep position and size",
            Self::Left => "Anchor left",
            Self::Right => "Anchor right",
            Self::Center => "Center",
            Self::Stretch => "Reflow / stretch box",
            Self::Cover => "Cover frame (image)",
        }
    }
}
fn independent(board: &mut Artboard) {
    board.prototype_id.clear();
    for l in &mut board.layers {
        l.prototype_id.clear();
    }
}
pub fn adapt(
    source: &Artboard,
    size: [f32; 2],
    rules: &BTreeMap<usize, Rule>,
) -> Result<Artboard, String> {
    if size
        .iter()
        .any(|s| !s.is_finite() || *s < 1.0 || *s > 8192.0)
    {
        return Err("Choose dimensions between 1 and 8192 pixels".into());
    }
    let mut board = source.clone();
    independent(&mut board);
    board.name = format!("{} · {} × {}", source.name, size[0], size[1]);
    board.frame.width = size[0];
    board.frame.height = size[1];
    let sx = size[0] / source.frame.width;
    let sy = size[1] / source.frame.height;
    let scale = sx.min(sy);
    let dx = (size[0] - source.frame.width * scale) / 2.0;
    let dy = (size[1] - source.frame.height * scale) / 2.0;
    for (i, l) in board.layers.iter_mut().enumerate() {
        match rules.get(&i).copied().unwrap_or(Rule::Scale) {
            Rule::Scale => {
                l.x = l.x * scale + dx;
                l.y = l.y * scale + dy;
                l.width *= scale;
                l.height *= scale;
                l.radius = (l.radius * scale).min(200.0);
                if let Some(c) = &mut l.corners {
                    for r in c {
                        *r = (*r * scale).min(200.0);
                    }
                }
                if l.kind == Kind::Text {
                    l.typography.size = Some(
                        l.typography
                            .size
                            .unwrap_or((source.layers[i].height * 0.65).min(48.0))
                            * scale,
                    );
                    l.typography.tracking *= scale;
                    if let Some(h) = &mut l.typography.line_height {
                        *h *= scale;
                    }
                }
            }
            Rule::Fixed => {}
            Rule::Left => l.y += (size[1] - source.frame.height) / 2.0,
            Rule::Right => {
                l.x += size[0] - source.frame.width;
                l.y += (size[1] - source.frame.height) / 2.0;
            }
            Rule::Center => {
                l.x += (size[0] - source.frame.width) / 2.0;
                l.y += (size[1] - source.frame.height) / 2.0;
            }
            Rule::Stretch => {
                l.width = (l.width + size[0] - source.frame.width).max(1.0);
                l.height = (l.height + size[1] - source.frame.height).max(1.0);
            }
            Rule::Cover => {
                if l.kind != Kind::Image {
                    return Err("Cover frame is only available for image layers".into());
                }
                let data = l.image_data.as_ref().ok_or("Image has no pixels")?;
                let image = image::load_from_memory(data).map_err(|e| e.to_string())?;
                let w = size[0].round() as u32;
                let h = size[1].round() as u32;
                if w as u64 * h as u64 > 16_000_000 {
                    return Err("Image crop exceeds 16 megapixels".into());
                }
                let cropped = image.resize_to_fill(w, h, image::imageops::FilterType::Lanczos3);
                let mut bytes = Cursor::new(Vec::new());
                cropped
                    .write_to(&mut bytes, image::ImageFormat::Png)
                    .map_err(|e| e.to_string())?;
                l.image_data = Some(bytes.into_inner());
                l.x = 0.0;
                l.y = 0.0;
                l.width = size[0];
                l.height = size[1];
            }
        }
    }
    validate_boards(std::slice::from_ref(&board))?;
    Ok(board)
}
fn validate_boards(boards: &[Artboard]) -> Result<(), String> {
    if boards.is_empty() || boards.len() > 100 {
        return Err("Generate between 1 and 100 frames at a time".into());
    }
    let image_bytes: usize = boards
        .iter()
        .flat_map(|b| &b.layers)
        .filter_map(|l| l.image_data.as_ref())
        .map(Vec::len)
        .sum();
    if image_bytes > 128 * 1024 * 1024 {
        return Err("Generated image data exceeds 128 MB; use fewer rows or smaller images".into());
    }
    Document {
        checkpoints: vec![],
        prototype: Default::default(),
        name: "Validation".into(),
        pages: vec![crate::Page {
            name: "Generated".into(),
            artboards: boards.to_vec(),
        }],
    }
    .validate()
}

#[derive(Clone, Serialize)]
pub struct Finding {
    pub page: usize,
    pub board: usize,
    pub layer: Option<usize>,
    pub message: String,
}
fn luminance(rgb: [u8; 3]) -> f64 {
    let linear = |c: u8| {
        let n = c as f64 / 255.0;
        if n <= 0.04045 {
            n / 12.92
        } else {
            ((n + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
}
pub fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
    let a = luminance(a);
    let b = luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
pub fn checks(doc: &Document) -> Vec<Finding> {
    let missing = crate::fonts::missing_families(doc);
    let mut found = vec![];
    for (pi, page) in doc.pages.iter().enumerate() {
        for (bi, b) in page.artboards.iter().enumerate() {
            let mut add = |layer, message: String| {
                found.push(Finding {
                    page: pi,
                    board: bi,
                    layer,
                    message,
                })
            };
            for (i, l) in b.layers.iter().enumerate().filter(|(_, l)| l.visible) {
                let name = l.label.as_deref().unwrap_or(&l.name);
                if l.x < 0.0
                    || l.y < 0.0
                    || l.x + l.width > b.frame.width + 0.1
                    || l.y + l.height > b.frame.height + 0.1
                {
                    add(
                        Some(i),
                        format!(
                            "{name}: extends beyond frame{}",
                            if b.frame.clip {
                                " and will be clipped"
                            } else {
                                ""
                            }
                        ),
                    );
                }
                if l.kind == Kind::Text {
                    let mut background = b.frame.background;
                    // Use the topmost earlier opaque solid rectangle containing the entire text box.
                    let mut uncertain = false;
                    for behind in &b.layers[..i] {
                        if !behind.visible {
                            continue;
                        }
                        let overlaps = behind.x < l.x + l.width
                            && behind.x + behind.width > l.x
                            && behind.y < l.y + l.height
                            && behind.y + behind.height > l.y;
                        if overlaps {
                            if behind.kind == Kind::Rectangle
                                && behind.path.is_none()
                                && behind.x <= l.x
                                && behind.y <= l.y
                                && behind.x + behind.width >= l.x + l.width
                                && behind.y + behind.height >= l.y + l.height
                                && behind.effects.is_empty()
                            {
                                background = behind.color;
                                uncertain = false;
                            } else {
                                uncertain = true;
                            }
                        }
                    }
                    if !uncertain {
                        let ratio = contrast(l.color, background);
                        let size = l.typography.size.unwrap_or((l.height * 0.65).min(48.0));
                        let threshold = if size >= 24.0 || (l.typography.bold && size >= 18.66) {
                            3.0
                        } else {
                            4.5
                        };
                        if ratio < threshold {
                            add(Some(i),format!("{name}: text contrast {ratio:.2}:1 (target {threshold}:1, solid-background estimate)"));
                        }
                    } else {
                        add(
                            Some(i),
                            format!(
                                "{name}: contrast needs manual review over overlapping artwork"
                            ),
                        );
                    }
                    if let Ok(layout) = crate::text::layout(l) {
                        let font = crate::fonts::render_font(&l.typography);
                        let scaled = font.as_scaled(layout.size);
                        if layout.glyphs.iter().any(|g| {
                            g.x < -0.5
                                || g.x + scaled.h_advance(g.id) > l.width + 0.5
                                || g.y + scaled.descent().abs() > l.height + 0.5
                        }) {
                            add(
                                Some(i),
                                format!("{name}: text may be clipped; enlarge its box"),
                            );
                        }
                    }
                    if l.typography
                        .family
                        .as_ref()
                        .is_some_and(|f| missing.contains(f))
                    {
                        add(Some(i), format!("{name}: requested font is not installed"));
                    }
                }
                if doc
                    .prototype
                    .interactions
                    .iter()
                    .any(|link| link.source == l.prototype_id && !l.prototype_id.is_empty())
                    && (l.width < 24.0 || l.height < 24.0)
                {
                    add(
                        Some(i),
                        format!("{name}: interaction target is smaller than 24 × 24"),
                    );
                }
            }
            // Report spacing outliers only for a meaningful repeated horizontal/vertical sequence.
            for horizontal in [true, false] {
                let mut items: Vec<_> = b
                    .layers
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.visible && l.group.is_some())
                    .collect();
                items.sort_by(|a, b| {
                    let f = |l: &Layer| if horizontal { l.x } else { l.y };
                    f(a.1).total_cmp(&f(b.1))
                });
                let mut gaps = vec![];
                for pair in items.windows(2) {
                    let (a, b) = (pair[0], pair[1]);
                    if a.1.group != b.1.group {
                        continue;
                    }
                    let aligned = if horizontal {
                        (a.1.y - b.1.y).abs() < 1.0
                    } else {
                        (a.1.x - b.1.x).abs() < 1.0
                    };
                    let gap = if horizontal {
                        b.1.x - a.1.x - a.1.width
                    } else {
                        b.1.y - a.1.y - a.1.height
                    };
                    if aligned && gap >= 0.0 {
                        gaps.push((b.0, gap));
                    }
                }
                if gaps.len() >= 3 {
                    let mut values: Vec<_> = gaps.iter().map(|(_, g)| *g).collect();
                    values.sort_by(f32::total_cmp);
                    let median = values[values.len() / 2];
                    for (i, gap) in gaps {
                        if (gap - median).abs() > 2.0 {
                            add(Some(i),format!("Spacing differs from repeated sequence: {gap:.1} px, median {median:.1} px"));
                        }
                    }
                }
            }
        }
    }
    found
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stress {
    Long,
    Empty,
    Translation,
    Crop,
}
pub fn stress(source: &Artboard, mode: Stress, replacement: &str) -> Result<Artboard, String> {
    if replacement.len() > 16_384 {
        return Err("Stress text exceeds 16 KB".into());
    }
    let mut b = source.clone();
    independent(&mut b);
    b.name = format!(
        "{} · {}",
        source.name,
        match mode {
            Stress::Long => "Long copy",
            Stress::Empty => "Empty state",
            Stress::Translation => "Translation",
            Stress::Crop => "Image crops",
        }
    );
    for l in &mut b.layers {
        if l.kind == Kind::Text {
            match mode {
                Stress::Long => l.name = format!("{} — {}", l.name, replacement),
                Stress::Empty => l.name.clear(),
                Stress::Translation => l.name = replacement.into(),
                Stress::Crop => {}
            }
        }
        if mode == Stress::Crop && l.kind == Kind::Image {
            if let Some(data) = &l.image_data {
                let img = image::load_from_memory(data).map_err(|e| e.to_string())?;
                let w = l.width.round().max(1.0) as u32;
                let h = l.height.round().max(1.0) as u32;
                if w as u64 * h as u64 > 16_000_000 {
                    return Err("Image crop exceeds 16 megapixels".into());
                }
                let crop = img.resize_to_fill(w, h, image::imageops::FilterType::Lanczos3);
                let mut out = Cursor::new(Vec::new());
                crop.write_to(&mut out, image::ImageFormat::Png)
                    .map_err(|e| e.to_string())?;
                l.image_data = Some(out.into_inner());
            }
        }
    }
    validate_boards(std::slice::from_ref(&b))?;
    Ok(b)
}

pub type Row = BTreeMap<String, String>;
fn csv(text: &str) -> Result<Vec<Row>, String> {
    let mut records = vec![];
    let mut record = vec![];
    let mut field = String::new();
    let mut quoted = false;
    let mut closed = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' if field.is_empty() && !closed => quoted = true,
            ',' => {
                record.push(std::mem::take(&mut field));
                closed = false;
            }
            '\n' => {
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
                closed = false;
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            _ => {
                if closed {
                    return Err("Unexpected characters after quoted CSV field".into());
                }
                field.push(c);
            }
        }
    }
    if quoted {
        return Err("Unclosed CSV quote".into());
    }
    if !record.is_empty() || !field.is_empty() || closed {
        record.push(field);
        records.push(record);
    }
    let headers = records.first().ok_or("CSV is empty")?.clone();
    if headers.iter().any(|h| h.trim().is_empty())
        || headers
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != headers.len()
    {
        return Err("CSV needs nonempty, unique headers".into());
    }
    records
        .into_iter()
        .skip(1)
        .map(|r| {
            if r.len() != headers.len() {
                Err("CSV row has a different number of columns".into())
            } else {
                Ok(headers.iter().cloned().zip(r).collect())
            }
        })
        .collect()
}
pub fn parse_rows(text: &str, json: bool) -> Result<Vec<Row>, String> {
    if text.len() > 4 * 1024 * 1024 {
        return Err("Data file exceeds 4 MB".into());
    }
    let rows = if json {
        let v: Vec<BTreeMap<String, serde_json::Value>> =
            serde_json::from_str(text).map_err(|e| e.to_string())?;
        v.into_iter()
            .map(|r| {
                r.into_iter()
                    .map(|(k, v)| {
                        let value =
                            match v {
                                serde_json::Value::String(s) => s,
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                serde_json::Value::Null => String::new(),
                                _ => return Err(
                                    "JSON columns must contain strings, numbers, booleans or null"
                                        .into(),
                                ),
                            };
                        Ok((k, value))
                    })
                    .collect()
            })
            .collect::<Result<Vec<Row>, String>>()?
    } else {
        csv(text)?
    };
    if rows.is_empty() || rows.len() > 100 {
        return Err("Import between 1 and 100 data rows".into());
    }
    Ok(rows)
}
#[derive(Clone, Debug)]
pub struct Mapping {
    pub layer: usize,
    pub column: String,
}
pub fn batch(
    source: &Artboard,
    rows: &[Row],
    mappings: &[Mapping],
    base: &Path,
) -> Result<Vec<Artboard>, String> {
    if mappings.is_empty() {
        return Err("Map at least one text or image layer to a data column".into());
    }
    let source_bytes: usize = source
        .layers
        .iter()
        .filter_map(|l| l.image_data.as_ref())
        .map(Vec::len)
        .sum();
    if source_bytes.saturating_mul(rows.len()) > 128 * 1024 * 1024 {
        return Err("Batch image data exceeds 128 MB; use fewer rows or smaller images".into());
    }
    let mut result = vec![];
    for (index, row) in rows.iter().enumerate() {
        let mut b = source.clone();
        independent(&mut b);
        b.name = format!("{} · Row {}", source.name, index + 1);
        for mapping in mappings {
            let value = row
                .get(&mapping.column)
                .ok_or_else(|| format!("Row {} has no '{}' column", index + 1, mapping.column))?;
            let layer = b
                .layers
                .get_mut(mapping.layer)
                .ok_or("Mapped layer was removed")?;
            match layer.kind {
                Kind::Text => layer.name = value.clone(),
                Kind::Image => {
                    let path = base.join(value);
                    let file = std::fs::File::open(&path)
                        .map_err(|e| format!("Cannot read mapped image {}: {e}", path.display()))?;
                    if file.metadata().map_err(|e| e.to_string())?.len() > 32_000_000 {
                        return Err("Mapped image exceeds 32 MB".into());
                    }
                    let mut data = vec![];
                    file.take(32_000_001)
                        .read_to_end(&mut data)
                        .map_err(|e| e.to_string())?;
                    image::load_from_memory(&data)
                        .map_err(|e| format!("Invalid mapped image: {e}"))?;
                    layer.image_data = Some(data);
                }
                _ => return Err("Only text and image layers can be mapped".into()),
            }
        }
        result.push(b);
    }
    validate_boards(&result)?;
    Ok(result)
}

pub fn delivery(doc: &Document, path: &Path, notes: &str) -> Result<(), String> {
    if notes.len() > 64 * 1024 {
        return Err("Design notes exceed 64 KB".into());
    }
    doc.validate()?;
    static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let temp = std::env::temp_dir().join(format!(
        "suiza-delivery-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&temp).map_err(|e| e.to_string())?;
    let outcome = (|| {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        let mut add = |name: &str, bytes: &[u8]| -> Result<(), String> {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            zip.write_all(bytes).map_err(|e| e.to_string())
        };
        add(
            "project.suiza-ui",
            &crate::document_file::encode(doc, Path::new("project.suiza-ui"))?,
        )?;
        add("design-notes.md", notes.as_bytes())?;
        add(
            "design-checks.json",
            &serde_json::to_vec_pretty(&checks(doc)).map_err(|e| e.to_string())?,
        )?;
        let mut manifest = vec![];
        for (pi, page) in doc.pages.iter().enumerate() {
            for (bi, board) in page.artboards.iter().enumerate() {
                let stem = format!("page-{}-frame-{}", pi + 1, bi + 1);
                for format in ["SVG", "PNG"] {
                    let ext = format.to_lowercase();
                    let file = temp.join(format!("{stem}.{ext}"));
                    crate::export_board(board, &file, format, 1)?;
                    add(
                        &format!("frames/{stem}.{ext}"),
                        &std::fs::read(file).map_err(|e| e.to_string())?,
                    )?;
                }
                manifest.push(serde_json::json!({"page":page.name,"frame":board.name,"file":stem,"width":board.frame.width,"height":board.frame.height}));
                for (li, l) in board.layers.iter().enumerate() {
                    if let Some(bytes) = &l.image_data {
                        let ext = match image::guess_format(bytes).map_err(|e| e.to_string())? {
                            image::ImageFormat::Png => "png",
                            image::ImageFormat::Jpeg => "jpg",
                            image::ImageFormat::WebP => "webp",
                            image::ImageFormat::Tiff => "tiff",
                            _ => "bin",
                        };
                        add(&format!("assets/{stem}-layer-{}.{ext}", li + 1), bytes)?;
                    }
                }
            }
        }
        let mut prototype = doc.clone();
        crate::prototype::ensure_ids(&mut prototype);
        if prototype.prototype.flows.is_empty() {
            prototype.prototype.flows.push(crate::prototype::Flow {
                name: "All screens".into(),
                home: prototype.pages[0].artboards[0].prototype_id.clone(),
            });
        }
        // An offline prototype per existing flow; detached frames remain available as exports.
        for i in 0..prototype.prototype.flows.len() {
            let file = temp.join(format!("flow-{i}.html"));
            crate::prototype::export_html(
                &prototype,
                i,
                &crate::prototype::ShareSettings::default(),
                &file,
            )?;
            add(
                &format!("prototypes/flow-{}.html", i + 1),
                &std::fs::read(file).map_err(|e| e.to_string())?,
            )?;
        }
        add(
            "manifest.json",
            &serde_json::to_vec_pretty(
                &serde_json::json!({"format":"suiza-form-delivery","version":1,"frames":manifest}),
            )
            .map_err(|e| e.to_string())?,
        )?;
        add("README.txt",b"SUIZA Canvas offline delivery\nOpen project.suiza-ui in SUIZA Canvas. View frames/ for PNG and SVG exports, prototypes/ for self-contained browser prototypes, design-notes.md and design-checks.json for review. Original embedded images are in assets/. Fonts are referenced, not redistributed; install the licensed fonts to edit with matching typography. No account or server is required.\n")?;
        drop(add);
        let bytes = zip.finish().map_err(|e| e.to_string())?.into_inner();
        if bytes.len() > 256 * 1024 * 1024 {
            return Err("Delivery package exceeds 256 MB".into());
        }
        crate::write_document_atomic(path, &bytes)
    })();
    let _ = std::fs::remove_dir_all(&temp);
    outcome
}

pub enum Command {
    Insert(Vec<Artboard>),
    Select(Finding),
    SavePoint(String),
    Restore(usize),
    Deliver(PathBuf, String),
}
pub struct Workbench {
    pub open: bool,
    tab: usize,
    source: Option<Artboard>,
    targets: Vec<bool>,
    rules: BTreeMap<usize, Rule>,
    generated: Vec<Artboard>,
    stress_mode: Stress,
    stress_text: String,
    rows: Vec<Row>,
    mappings: Vec<Mapping>,
    data_base: PathBuf,
    data_name: String,
    findings: Vec<Finding>,
    findings_document: Option<Document>,
    checkpoint_name: String,
    history_index: Option<usize>,
    history_preview: Option<Document>,
    notes: String,
    error: String,
}
impl Default for Workbench {
    fn default() -> Self {
        Self {
            open: false,
            tab: 0,
            source: None,
            targets: vec![false; crate::CANVAS_PRESETS[2].len()],
            rules: BTreeMap::new(),
            generated: vec![],
            stress_mode: Stress::Long,
            stress_text: "A considerably longer headline to test wrapping and layout resilience"
                .into(),
            rows: vec![],
            mappings: vec![],
            data_base: PathBuf::new(),
            data_name: String::new(),
            findings: vec![],
            findings_document: None,
            checkpoint_name: String::new(),
            history_index: None,
            history_preview: None,
            notes: String::new(),
            error: String::new(),
        }
    }
}
fn preview(ui: &mut egui::Ui, board: &Artboard, width: f32) {
    let doc = Document {
        checkpoints: vec![],
        prototype: Default::default(),
        name: board.name.clone(),
        pages: vec![crate::Page {
            name: "Preview".into(),
            artboards: vec![board.clone()],
        }],
    };
    crate::welcome_document_card(
        ui,
        &doc,
        &board.name,
        &format!("{} × {}", board.frame.width, board.frame.height),
        width,
    );
}
fn age(created: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let seconds = now.saturating_sub(created);
    if seconds < 60 {
        "just now".into()
    } else if seconds < 3600 {
        format!("{} min ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}
impl Workbench {
    pub fn clear_history_selection(&mut self) {
        self.history_index = None;
        self.history_preview = None;
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        doc: &Document,
        page: usize,
        board: usize,
    ) -> Option<Command> {
        if !self.open {
            return None;
        }
        let source = &doc.pages[page].artboards[board];
        if self.source.as_ref() != Some(source) {
            self.source = Some(source.clone());
            self.generated.clear();
            self.rules.clear();
            self.mappings.clear();
            self.error.clear();
        }
        let settings_before = format!(
            "{:?}{:?}{:?}{}{:?}",
            self.targets, self.rules, self.stress_mode, self.stress_text, self.mappings
        );
        if self
            .findings_document
            .as_ref()
            .is_some_and(|previous| previous != doc)
        {
            self.findings.clear();
            self.findings_document = None;
        }
        self.findings.retain(|f| {
            doc.pages
                .get(f.page)
                .and_then(|p| p.artboards.get(f.board))
                .is_some_and(|b| f.layer.is_none_or(|i| i < b.layers.len()))
        });
        let mut command = None;
        let mut close = false;
        egui::Window::new("Creative Workbench").title_bar(false).resizable(true).default_width(930.0).max_width((ctx.screen_rect().width()-32.0).max(360.0)).frame(egui::Frame::window(&ctx.style()).inner_margin(20)).show(ctx,|ui|{
            ui.horizontal(|ui|{ui.label(egui::RichText::new("Creative Workbench").size(20.0).strong());ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{close=crate::close_icon_button(ui,28.0).clicked();});});
            ui.label(egui::RichText::new(format!("Source: {} / {}",doc.pages[page].name,source.name)).color(crate::secondary_text_color(ui)));
            ui.horizontal_wrapped(|ui|{for(i,label)in["Adapt","Check","Stress test","History","Batch","Deliver"].iter().enumerate(){if ui.selectable_label(self.tab==i,*label).clicked(){self.tab=i;self.error.clear();self.generated.clear();}}});ui.separator();
            egui::ScrollArea::vertical().max_height((ctx.screen_rect().height()-200.0).clamp(180.0,620.0)).show(ui,|ui|{
                match self.tab {
                    0=>{
                        ui.label("Choose output sizes and how each layer adapts. Originals stay intact; generated frames remain editable.");
                        ui.horizontal_wrapped(|ui|{for(i,(name,_))in crate::CANVAS_PRESETS[2].iter().enumerate(){ui.checkbox(&mut self.targets[i],*name);}});
                        ui.collapsing("Layer adaptation rules",|ui|{for(i,l)in source.layers.iter().enumerate(){ui.horizontal(|ui|{ui.add_sized([(ui.available_width()*0.4).min(230.0),22.0],egui::Label::new(l.label.as_deref().unwrap_or(&l.name)).truncate());let rule=self.rules.entry(i).or_insert(Rule::Scale);egui::ComboBox::from_id_salt(("adapt_rule",i)).selected_text(rule.label()).show_ui(ui,|ui|{for choice in Rule::ALL{if choice!=Rule::Cover || l.kind==Kind::Image{ui.selectable_value(rule,choice,choice.label());}}});});}});
                        if ui.button("Generate preview").clicked(){let result=self.targets.iter().enumerate().filter(|(_,on)|**on).map(|(i,_)|adapt(source,crate::CANVAS_PRESETS[2][i].1,&self.rules)).collect::<Result<Vec<_>,_>>();match result{Ok(boards)if !boards.is_empty()=>self.generated=boards,Ok(_)=>self.error="Select at least one target size".into(),Err(e)=>self.error=e}}
                    },
                    1=>{
                        ui.label("Checks cover text contrast on solid backgrounds, clipping, missing fonts, repeated spacing and prototype targets. These are review aids, not accessibility certification.");
                        if ui.button("Scan document").clicked(){self.findings=checks(doc);self.findings_document=Some(doc.clone());self.error.clear();}
                        ui.label(format!("{} findings",self.findings.len()));
                        for finding in &self.findings {let label=format!("{} / {}: {}",doc.pages[finding.page].name,doc.pages[finding.page].artboards[finding.board].name,finding.message);if ui.add(egui::Button::new(label).wrap()).clicked(){command=Some(Command::Select(finding.clone()));close=true;}}
                    },
                    2=>{
                        ui.label("Try difficult content in a separate editable frame. Use your own translated copy; no network translation is performed.");
                        ui.horizontal_wrapped(|ui|{for(mode,label)in[(Stress::Long,"Long headlines"),(Stress::Empty,"Empty text"),(Stress::Translation,"Translated copy"),(Stress::Crop,"Center image crops")]{ui.selectable_value(&mut self.stress_mode,mode,label);}});
                        if self.stress_mode==Stress::Long || self.stress_mode==Stress::Translation{ui.add(egui::TextEdit::multiline(&mut self.stress_text).desired_rows(3).desired_width(ui.available_width()));}
                        if ui.button("Preview stress test").clicked(){match stress(source,self.stress_mode,&self.stress_text){Ok(b)=>self.generated=vec![b],Err(e)=>self.error=e}}
                    },
                    3=>{
                        ui.label("Named checkpoints travel inside the document. Save the file to persist them. Restoring keeps a recovery checkpoint and supports Undo.");
                        ui.horizontal(|ui|{ui.add(egui::TextEdit::singleline(&mut self.checkpoint_name).hint_text("Checkpoint name"));if ui.button("Create checkpoint").clicked(){command=Some(Command::SavePoint(self.checkpoint_name.clone()));}});
                        for(i,point)in doc.checkpoints.iter().enumerate().rev(){if ui.selectable_label(self.history_index==Some(i),format!("{} · {}",point.name, age(point.created))).clicked(){self.history_index=Some(i);match Document::from_json(&point.snapshot){Ok(d)=>self.history_preview=Some(d),Err(e)=>{self.history_preview=None;self.error=e;}}}}
                        if let(Some(index),Some(saved))=(self.history_index,&self.history_preview){ui.horizontal_wrapped(|ui|{ui.vertical(|ui|{ui.label("Checkpoint");if let Some(board) = saved.pages.get(page).and_then(|p| p.artboards.get(board)) { preview(ui,board,240.0); } else { ui.label("This frame did not exist at the checkpoint."); }});ui.vertical(|ui|{ui.label("Current frame");preview(ui,source,240.0);});});let(old,new)=(saved.pages.iter().flat_map(|p|&p.artboards).count(),doc.pages.iter().flat_map(|p|&p.artboards).count());ui.label(format!("Frames: {old} at checkpoint, {new} now"));if ui.button("Restore this checkpoint").clicked(){command=Some(Command::Restore(index));close=true;}}
                    },
                    4=>{
                        ui.label("Map CSV/JSON columns to text or image layers. Image paths are resolved relative to the data file; originals stay unchanged.");
                        if ui.button("Import CSV or JSON…").clicked(){if let Some(path)=rfd::FileDialog::new().add_filter("Content data",&["csv","json"]).pick_file(){let result=(||{let file=std::fs::File::open(&path).map_err(|e|e.to_string())?;if file.metadata().map_err(|e|e.to_string())?.len()>4*1024*1024{return Err("Data file exceeds 4 MB".into());}let text=std::fs::read_to_string(&path).map_err(|e|e.to_string())?;parse_rows(text.trim_start_matches('\u{feff}'),path.extension().is_some_and(|e|e=="json"))})();match result{Ok(rows)=>{self.rows=rows;self.data_base=path.parent().unwrap_or(Path::new(".")).into();self.data_name=path.file_name().unwrap_or_default().to_string_lossy().into();self.mappings.clear();self.generated.clear();self.error.clear();},Err(e)=>self.error=e}}}
                        ui.label(format!("{} · {} rows",self.data_name,self.rows.len()));
                        let columns:Vec<_>=self.rows.iter().flat_map(|r|r.keys().cloned()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
                        for(i,l)in source.layers.iter().enumerate().filter(|(_,l)|matches!(l.kind,Kind::Text|Kind::Image)){let mut col=self.mappings.iter().find(|m|m.layer==i).map(|m|m.column.clone()).unwrap_or_default();ui.horizontal(|ui|{ui.add_sized([(ui.available_width()*0.4).min(230.0),22.0],egui::Label::new(l.label.as_deref().unwrap_or(&l.name)).truncate());egui::ComboBox::from_id_salt(("batch_column",i)).selected_text(if col.is_empty(){"Unmapped"}else{&col}).show_ui(ui,|ui|{ui.selectable_value(&mut col,String::new(),"Unmapped");for c in &columns{ui.selectable_value(&mut col,c.clone(),c);}});});self.mappings.retain(|m|m.layer!=i);if !col.is_empty(){self.mappings.push(Mapping{layer:i,column:col});}}
                        if ui.add_enabled(!self.rows.is_empty(),egui::Button::new("Generate batch preview")).clicked(){match batch(source,&self.rows,&self.mappings,&self.data_base){Ok(b)=>self.generated=b,Err(e)=>self.error=e}}
                    },
                    _=>{
                        ui.label("Create a local ZIP with the native document, PNG/SVG frames, embedded images, offline prototypes, checks and notes. Fonts are referenced and never redistributed.");
                        ui.add(egui::TextEdit::multiline(&mut self.notes).hint_text("Design notes for the recipient").desired_rows(6).desired_width(ui.available_width()));
                        if ui.button("Export delivery package…").clicked(){if let Some(path)=rfd::FileDialog::new().add_filter("Delivery ZIP",&["zip"]).set_file_name("SUIZA-Form-delivery.zip").save_file(){command=Some(Command::Deliver(path,self.notes.clone()));}}
                    }
                }
                if !self.generated.is_empty(){ui.separator();ui.label(format!("{} editable frames ready",self.generated.len()));
                    for chunk in self.generated.chunks(((ui.available_width() / 226.0).floor() as usize).max(1)){ui.horizontal_top(|ui|{for b in chunk{preview(ui,b,210.0);}});}
                    if ui.button("Add generated frames to document").clicked(){command=Some(Command::Insert(self.generated.clone()));self.generated.clear();close=true;}
                }
                if !self.error.is_empty(){ui.label(egui::RichText::new(&self.error).color(if ui.visuals().dark_mode{Color32::from_rgb(255,170,160)}else{Color32::from_rgb(150,30,20)}));}
            });
        });
        let settings_after = format!(
            "{:?}{:?}{:?}{}{:?}",
            self.targets, self.rules, self.stress_mode, self.stress_text, self.mappings
        );
        if settings_before != settings_after {
            self.generated.clear();
        }
        if close {
            self.open = false;
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Document {
        let mut text = Layer::new(Kind::Text, 20.0, 20.0);
        text.name = "Hello".into();
        text.label = Some("Headline".into());
        text.width = 180.0;
        text.height = 60.0;
        text.color = [200, 200, 200];
        text.typography.size = Some(16.0);
        Document {
            checkpoints: vec![],
            prototype: Default::default(),
            name: "Campaign".into(),
            pages: vec![crate::Page {
                name: "Page".into(),
                artboards: vec![Artboard {
                    prototype_id: "frame-a".into(),
                    name: "Source".into(),
                    frame: crate::FrameStyle {
                        width: 300.0,
                        height: 200.0,
                        ..Default::default()
                    },
                    layers: vec![text],
                }],
            }],
        }
    }
    #[test]
    fn adaptation_keeps_sources_and_proportions_and_applies_anchors() {
        let doc = fixture();
        let source = &doc.pages[0].artboards[0];
        let before = source.clone();
        let b = adapt(source, [600.0, 400.0], &BTreeMap::new()).unwrap();
        assert_eq!(b.layers[0].width, 360.0);
        assert_eq!(b.layers[0].typography.size, Some(32.0));
        assert!(source == &before);
        assert!(b.prototype_id.is_empty());
        let rules = BTreeMap::from([(0, Rule::Right)]);
        let b = adapt(source, [600.0, 400.0], &rules).unwrap();
        assert_eq!(b.layers[0].x, 320.0);
        assert_eq!(b.layers[0].width, 180.0);
        let rules = BTreeMap::from([(0, Rule::Stretch)]);
        let b = adapt(source, [500.0, 250.0], &rules).unwrap();
        assert_eq!(b.layers[0].width, 380.0);
        assert_eq!(b.layers[0].typography.size, Some(16.0));
        assert!(adapt(source, [f32::NAN, 200.0], &rules).is_err());
    }
    #[test]
    fn image_crops_preserve_source_pixels_and_target_aspect() {
        let mut doc = fixture();
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            60,
            20,
            image::Rgba([10, 20, 30, 255]),
        ));
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        let mut layer = Layer::new(Kind::Image, 0.0, 0.0);
        layer.image_data = Some(bytes.into_inner());
        doc.pages[0].artboards[0].layers = vec![layer];
        let source = &doc.pages[0].artboards[0];
        let before = source.clone();
        let b = adapt(source, [100.0, 200.0], &BTreeMap::from([(0, Rule::Cover)])).unwrap();
        let img = image::load_from_memory(b.layers[0].image_data.as_ref().unwrap()).unwrap();
        assert_eq!((img.width(), img.height()), (100, 200));
        assert!(source == &before);
    }
    #[test]
    fn checks_find_low_contrast_clipping_fonts_and_small_targets() {
        let mut doc = fixture();
        doc.pages[0].artboards[0].layers[0].name = "AnUnbreakableHeadlineThatCannotFit".into();
        doc.pages[0].artboards[0].layers[0].width = 10.0;
        doc.pages[0].artboards[0].layers[0].height = 10.0;
        doc.pages[0].artboards[0].layers[0].x = 295.0;
        doc.pages[0].artboards[0].layers[0].typography.family =
            Some("Definitely Missing SUIZA Font 1234".into());
        let found = checks(&doc);
        assert!(found.iter().any(|f| f.message.contains("text contrast")));
        assert!(found.iter().any(|f| f.message.contains("clipped")));
        assert!(found.iter().any(|f| f.message.contains("not installed")));
        assert!(found.iter().all(|f| f.layer == Some(0)));
        assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
    }
    #[test]
    fn stress_variations_are_editable_and_do_not_replace_source() {
        let doc = fixture();
        let source = &doc.pages[0].artboards[0];
        assert_eq!(
            stress(source, Stress::Empty, "").unwrap().layers[0].name,
            ""
        );
        assert_eq!(
            stress(source, Stress::Translation, "Bonjour")
                .unwrap()
                .layers[0]
                .name,
            "Bonjour"
        );
        assert!(stress(source, Stress::Long, "Long copy").unwrap().layers[0]
            .name
            .contains("Long copy"));
        assert_eq!(source.layers[0].name, "Hello");
    }
    #[test]
    fn csv_and_json_batches_map_content_and_reject_malformed_rows() {
        let rows = parse_rows(
            "headline,price\r\n\"Hello, world\",10\r\n\"Two\nlines\",20\r\n",
            false,
        )
        .unwrap();
        assert_eq!(rows[0]["headline"], "Hello, world");
        assert_eq!(rows[1]["headline"], "Two\nlines");
        let doc = fixture();
        let boards = batch(
            &doc.pages[0].artboards[0],
            &rows,
            &[Mapping {
                layer: 0,
                column: "headline".into(),
            }],
            Path::new("."),
        )
        .unwrap();
        assert_eq!(boards.len(), 2);
        assert_eq!(boards[0].layers[0].name, "Hello, world");
        assert_eq!(doc.pages[0].artboards[0].layers[0].name, "Hello");
        assert_eq!(boards[0].layers[0].label.as_deref(), Some("Headline"));
        assert!(parse_rows("a,a\n1,2", false).is_err());
        assert!(parse_rows("a,b\n1", false).is_err());
        assert!(parse_rows("a\n\"oops", false).is_err());
        assert!(parse_rows("[{\"a\":[]}]", true).is_err());
        assert_eq!(
            parse_rows("[{\"a\":42,\"b\":null}]", true).unwrap()[0]["a"],
            "42"
        );
        assert!(batch(
            &doc.pages[0].artboards[0],
            &rows,
            &[Mapping {
                layer: 0,
                column: "missing".into()
            }],
            Path::new(".")
        )
        .is_err());
    }
    #[test]
    fn checkpoints_survive_native_round_trip_and_restore_with_recovery() {
        let mut doc = fixture();
        checkpoint(&mut doc, "First").unwrap();
        doc.pages[0].artboards[0].layers[0].name = "Changed".into();
        checkpoint(&mut doc, "Second").unwrap();
        let bytes = crate::document_file::encode(&doc, Path::new("checkpoint.suiza-ui")).unwrap();
        let loaded = crate::document_file::decode(&bytes).unwrap();
        assert!(loaded == doc);
        let restored = restore(&loaded, 0).unwrap();
        assert_eq!(restored.pages[0].artboards[0].layers[0].name, "Hello");
        let recovered = restore(&restored, restored.checkpoints.len() - 1).unwrap();
        assert_eq!(recovered.pages[0].artboards[0].layers[0].name, "Changed");
        assert!(restored.checkpoints[0]
            .snapshot
            .contains("\"checkpoints\":[]"));
        assert!(checkpoint(&mut doc, "").is_err());
        assert!(restore(&doc, 999).is_err());
    }
    #[test]
    fn delivery_contains_editable_project_offline_prototype_and_exports() {
        let doc = fixture();
        let path = std::env::temp_dir().join(format!(
            "suiza-workbench-delivery-test-{}.zip",
            std::process::id()
        ));
        delivery(&doc, &path, "Notes for review").unwrap();
        let original_bytes = std::fs::read(&path).unwrap();
        let bytes = original_bytes.clone();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        for entry in [
            "project.suiza-ui",
            "design-notes.md",
            "design-checks.json",
            "frames/page-1-frame-1.svg",
            "frames/page-1-frame-1.png",
            "prototypes/flow-1.html",
            "manifest.json",
        ] {
            assert!(zip.by_name(entry).is_ok(), "missing {entry}");
        }
        let mut native = vec![];
        zip.by_name("project.suiza-ui")
            .unwrap()
            .read_to_end(&mut native)
            .unwrap();
        assert!(crate::document_file::decode(&native).unwrap() == doc);
        let mut html = String::new();
        zip.by_name("prototypes/flow-1.html")
            .unwrap()
            .read_to_string(&mut html)
            .unwrap();
        assert!(!html.contains("https://"));
        let mut oversized = doc.clone();
        oversized.pages[0].artboards[0].frame.width = 8192.0;
        oversized.pages[0].artboards[0].frame.height = 8192.0;
        assert!(delivery(&oversized, &path, "").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original_bytes);
        let _ = std::fs::remove_file(path);
    }
}
