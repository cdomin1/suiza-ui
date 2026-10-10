//! Versioned, portable documents. Archive entries are read in memory, never extracted.
use crate::Document;
use std::{
    io::{Cursor, Read, Write},
    path::Path,
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};
const MAX_BYTES: u64 = 256 * 1024 * 1024;

pub fn load(path: &Path) -> Result<Document, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > MAX_BYTES {
        return Err("Document exceeds the 256 MB limit".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decode(&bytes)
}
fn entry(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
    budget: &mut u64,
) -> Result<Vec<u8>, String> {
    let file = archive
        .by_name(name)
        .map_err(|e| format!("Missing or invalid {name}: {e}"))?;
    if file.size() > *budget {
        return Err("Document contents exceed the size limit".into());
    }
    let mut bytes = Vec::new();
    file.take(*budget + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > *budget {
        return Err("Document contents exceed the size limit".into());
    }
    *budget -= bytes.len() as u64;
    Ok(bytes)
}
pub fn decode(bytes: &[u8]) -> Result<Document, String> {
    if !bytes.starts_with(b"PK") {
        return Document::from_json(std::str::from_utf8(bytes).map_err(|e| e.to_string())?);
    }
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let mut budget = MAX_BYTES;
    let manifest: serde_json::Value =
        serde_json::from_slice(&entry(&mut archive, "manifest.json", &mut budget)?)
            .map_err(|e| e.to_string())?;
    if manifest["format"] != "suiza-ui" || manifest["version"] != 1 {
        return Err("Unsupported Suiza-UI document version".into());
    }
    let value: serde_json::Value =
        serde_json::from_slice(&entry(&mut archive, "document.json", &mut budget)?)
            .map_err(|e| e.to_string())?;
    let mut doc: Document = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    for (pi, page) in doc.pages.iter_mut().enumerate() {
        for (bi, board) in page.artboards.iter_mut().enumerate() {
            for (li, layer) in board.layers.iter_mut().enumerate() {
                if let Some(name) =
                    value["pages"][pi]["artboards"][bi]["layers"][li]["image_asset"].as_str()
                {
                    if !name.starts_with("assets/") || name.contains("..") || name.contains('\\') {
                        return Err("Invalid image asset reference".into());
                    }
                    layer.image_data = Some(entry(&mut archive, name, &mut budget)?);
                }
            }
        }
    }
    doc.validate()?;
    Ok(doc)
}

pub fn encode(doc: &Document, path: &Path) -> Result<Vec<u8>, String> {
    doc.validate()?;
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    {
        return serde_json::to_vec_pretty(doc).map_err(|e| e.to_string());
    }
    // Move image buffers out before making JSON values: numeric byte arrays
    // otherwise inflate memory usage dramatically for image-heavy documents.
    let mut metadata = doc.clone();
    let mut assets = Vec::new();
    let mut references = Vec::new();
    for (pi, page) in metadata.pages.iter_mut().enumerate() {
        for (bi, board) in page.artboards.iter_mut().enumerate() {
            for (li, layer) in board.layers.iter_mut().enumerate() {
                if let Some(bytes) = layer.image_data.take() {
                    let name = format!("assets/image-{}.bin", assets.len());
                    references.push((pi, bi, li, name.clone()));
                    assets.push((name, bytes));
                }
            }
        }
    }
    let mut value = serde_json::to_value(metadata).map_err(|e| e.to_string())?;
    for (pi, bi, li, name) in references {
        value["pages"][pi]["artboards"][bi]["layers"][li]["image_asset"] = name.into();
    }
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let mut entries = vec![
        (
            "manifest.json".to_owned(),
            br#"{"format":"suiza-ui","version":1,"document":"document.json"}"#.to_vec(),
        ),
        (
            "document.json".to_owned(),
            serde_json::to_vec(&value).map_err(|e| e.to_string())?,
        ),
    ];
    entries.extend(assets);
    if let Ok(preview) = preview(doc) {
        entries.push(("preview.png".into(), preview));
    }
    if entries.iter().map(|(_, b)| b.len() as u64).sum::<u64>() > MAX_BYTES {
        return Err("Document exceeds the 256 MB limit".into());
    }
    for (name, data) in entries {
        archive
            .start_file(name, options)
            .map_err(|e| e.to_string())?;
        archive.write_all(&data).map_err(|e| e.to_string())?;
    }
    Ok(archive.finish().map_err(|e| e.to_string())?.into_inner())
}
fn preview(doc: &Document) -> Result<Vec<u8>, String> {
    let board = &doc.pages[0].artboards[0];
    let ratio = (512.0 / board.frame.width.max(board.frame.height)).min(1.0);
    let mut layers = board.layers.clone();
    for layer in &mut layers {
        layer.x *= ratio;
        layer.y *= ratio;
        layer.width *= ratio;
        layer.height *= ratio;
        layer.radius *= ratio;
        if let Some(corners) = &mut layer.corners {
            for r in corners {
                *r *= ratio;
            }
        }
        if let Some(size) = &mut layer.typography.size {
            *size *= ratio;
        }
        if let Some(leading) = &mut layer.typography.line_height {
            *leading *= ratio;
        }
        layer.typography.tracking *= ratio;
        layer.typography.baseline *= ratio;
        if let Some(path) = &mut layer.path {
            path.stroke_width *= ratio;
        }
        for shadow in [&mut layer.drop_shadow, &mut layer.inner_shadow]
            .into_iter()
            .flatten()
        {
            shadow.x *= ratio;
            shadow.y *= ratio;
            shadow.blur *= ratio;
            shadow.spread *= ratio;
        }
    }
    let [r, g, b] = board.frame.background;
    let mut img = image::RgbaImage::from_pixel(
        (board.frame.width * ratio).ceil().max(1.0) as u32,
        (board.frame.height * ratio).ceil().max(1.0) as u32,
        image::Rgba([r, g, b, 255]),
    );
    for (i, layer) in layers
        .iter()
        .enumerate()
        .filter(|(_, l)| l.visible && !l.mask)
    {
        if layer.x > img.width() as f32
            || layer.y > img.height() as f32
            || layer.x + layer.width < 0.0
            || layer.y + layer.height < 0.0
        {
            continue;
        }
        let asset = crate::rasterize_layer(layer, 1, false, &layers[..i])?;
        let pad = crate::effect_padding(layer);
        image::imageops::overlay(
            &mut img,
            &asset,
            (layer.x - pad).round() as i64,
            (layer.y - pad).round() as i64,
        );
    }
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_package_round_trip_preserves_images_and_rejects_damage() {
        let mut doc = Document::default();
        let mut layer = crate::Layer::new(crate::Kind::Image, 12.0, 34.0);
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([12, 34, 56, 255]),
        ))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
        layer.image_data = Some(png.into_inner());
        doc.pages[0].artboards[0].layers.push(layer);
        let bytes = encode(&doc, Path::new("test.suiza-ui")).unwrap();
        assert!(bytes.starts_with(b"PK"));
        assert!(decode(&bytes).unwrap() == doc);
        assert!(decode(&bytes[..bytes.len() / 2]).is_err());
        assert!(decode(&encode(&doc, Path::new("test.json")).unwrap()).unwrap() == doc);
    }
}
