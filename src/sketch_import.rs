//! Bounded read-only Sketch library conversion. Source files are never modified.
use crate::{vector, Artboard, Document, FrameStyle, Kind, Layer, Page};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
    path::Path,
};
use zip::ZipArchive;
const LIMIT: u64 = 256 * 1024 * 1024;
pub struct Import {
    pub document: Document,
    pub warnings: Vec<String>,
}
struct Reader<'a> {
    zip: ZipArchive<Cursor<&'a [u8]>>,
    budget: u64,
    symbols: BTreeMap<String, Value>,
    warnings: BTreeMap<String, usize>,
    count: usize,
}
fn num(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().map(|n| n as f32).unwrap_or(default)
}
fn name(v: &Value) -> &str {
    v["name"].as_str().unwrap_or("Untitled asset")
}
fn color(v: &Value) -> [u8; 3] {
    ["red", "green", "blue"].map(|k| (num(v, k, 0.0).clamp(0.0, 1.0) * 255.0).round() as u8)
}
fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
fn point(v: &Value) -> Result<[f32; 2], String> {
    let s = v.as_str().ok_or("Missing Sketch curve point")?;
    let values: Vec<_> = s
        .trim_matches(|c| c == '{' || c == '}' || c == ' ')
        .split(',')
        .map(|s| s.trim().parse::<f32>())
        .collect();
    if values.len() != 2 {
        return Err("Invalid Sketch curve point".into());
    }
    Ok([
        values[0].as_ref().map_err(|_| "Invalid point")?.to_owned(),
        values[1].as_ref().map_err(|_| "Invalid point")?.to_owned(),
    ])
}
impl Reader<'_> {
    fn warn(&mut self, reason: &str) {
        *self.warnings.entry(reason.into()).or_default() += 1;
    }
    fn bytes(&mut self, name: &str) -> Result<Vec<u8>, String> {
        if name.starts_with('/') || name.split('/').any(|s| s == "..") || name.contains('\\') {
            return Err("Unsafe Sketch entry reference".into());
        }
        let file = self
            .zip
            .by_name(name)
            .map_err(|e| format!("Cannot read {name}: {e}"))?;
        if file.size() > self.budget {
            return Err("Sketch contents exceed 256 MB".into());
        }
        let mut bytes = Vec::new();
        file.take(self.budget + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() as u64 > self.budget {
            return Err("Sketch contents exceed 256 MB".into());
        }
        self.budget -= bytes.len() as u64;
        Ok(bytes)
    }
    fn collect_symbols(&mut self, v: &Value, depth: usize) -> Result<(), String> {
        if depth > 64 {
            return Err("Sketch nesting exceeds 64 levels".into());
        }
        if v["_class"] == "symbolMaster" {
            if let Some(id) = v["symbolID"].as_str() {
                self.symbols.insert(id.into(), v.clone());
            }
        }
        for child in list(v, "layers") {
            self.collect_symbols(child, depth + 1)?
        }
        Ok(())
    }
    fn layers(
        &mut self,
        v: &Value,
        offset: [f32; 2],
        scale: [f32; 2],
        group: Option<String>,
        visible: bool,
        locked: bool,
        out: &mut Vec<Layer>,
        depth: usize,
    ) -> Result<(), String> {
        self.count += 1;
        if depth > 64 || self.count > 100_000 {
            return Err("Sketch library exceeds layer/nesting limits".into());
        }
        let class = v["_class"].as_str().unwrap_or("");
        let f = &v["frame"];
        let xy = [
            offset[0] + num(f, "x", 0.0) * scale[0],
            offset[1] + num(f, "y", 0.0) * scale[1],
        ];
        let wh = [
            num(f, "width", 0.0) * scale[0],
            num(f, "height", 0.0) * scale[1],
        ];
        let visible = visible && v["isVisible"].as_bool().unwrap_or(true);
        let locked = locked || v["isLocked"].as_bool().unwrap_or(false);
        if num(v, "rotation", 0.0).abs() > 0.001
            || v["isFlippedHorizontal"] == true
            || v["isFlippedVertical"] == true
        {
            self.warn("Rotated or flipped layers use untransformed geometry")
        }
        if v["hasClippingMask"] == true {
            self.warn("Clipping mask relationships are not imported")
        }
        if num(&v["style"]["contextSettings"], "opacity", 1.0) < 0.999 {
            self.warn("Layer opacity is not supported")
        }
        if class == "symbolInstance" {
            let id = v["symbolID"].as_str().unwrap_or("");
            if let Some(master) = self.symbols.get(id).cloned() {
                let mf = &master["frame"];
                let next = [
                    wh[0] / num(mf, "width", 1.0).max(0.001),
                    wh[1] / num(mf, "height", 1.0).max(0.001),
                ];
                let mut master = master;
                for ov in list(v, "overrideValues") {
                    if let (Some(key), Some(value)) = (ov["overrideName"].as_str(), ov.get("value"))
                    {
                        if let Some(id) = key.strip_suffix("_stringValue") {
                            apply_text_override(&mut master, id, value);
                        } else {
                            self.warn("Non-text symbol overrides are not imported")
                        }
                    }
                }
                for child in list(&master, "layers") {
                    self.layers(
                        child,
                        xy,
                        next,
                        group.clone().or_else(|| Some(name(v).into())),
                        visible,
                        locked,
                        out,
                        depth + 1,
                    )?
                }
            } else {
                self.warn("Unresolved external symbol skipped")
            };
            return Ok(());
        }
        if ["page", "group", "artboard", "symbolMaster", "shapeGroup"].contains(&class) {
            if class == "shapeGroup" {
                self.warn("Compound Boolean shapes imported as separate paths")
            }
            if group.is_some() && class == "group" {
                self.warn("Nested group hierarchy flattened")
            }
            let next_group = group.or_else(|| {
                if class == "group" {
                    Some(name(v).into())
                } else {
                    None
                }
            });
            for child in list(v, "layers") {
                self.layers(
                    child,
                    xy,
                    scale,
                    next_group.clone(),
                    visible,
                    locked,
                    out,
                    depth + 1,
                )?
            }
            return Ok(());
        }
        if wh.iter().any(|n| !n.is_finite() || *n <= 0.0) {
            self.warn("Zero-sized layers skipped");
            return Ok(());
        }
        let kind = match class {
            "text" => Kind::Text,
            "bitmap" => Kind::Image,
            "oval" => Kind::Ellipse,
            "rectangle" | "shapePath" => Kind::Rectangle,
            "slice" => return Ok(()),
            _ => {
                self.warn(&format!("Unsupported {class} layers skipped"));
                return Ok(());
            }
        };
        let mut layer = Layer::new(kind, xy[0], xy[1]);
        layer.width = wh[0];
        layer.height = wh[1];
        layer.name = name(v).into();
        layer.radius = 0.0;
        layer.group = group;
        layer.visible = visible;
        layer.locked = locked;
        let style = &v["style"];
        let fills: Vec<_> = list(style, "fills")
            .iter()
            .filter(|s| s["isEnabled"].as_bool().unwrap_or(true))
            .collect();
        if fills.len() > 1 {
            self.warn("Multiple fills reduced to the first fill")
        }
        let fill = fills.first().copied();
        if let Some(fill) = fill {
            layer.color = color(&fill["color"]);
            if fill["fillType"].as_i64().unwrap_or(0) != 0 {
                self.warn("Gradient/pattern fills reduced to a solid color")
            }
            if num(&fill["color"], "alpha", 1.0) < 0.999 {
                self.warn("Fill transparency is not supported")
            }
        }
        if kind == Kind::Text {
            layer.name = v["attributedString"]["string"]
                .as_str()
                .unwrap_or("")
                .into();
            layer.label = Some(name(v).into());
            let runs = list(&v["attributedString"], "attributes");
            if runs.len() > 1 {
                self.warn("Mixed text styles reduced to the first run")
            }
            let attrs = runs
                .first()
                .map(|r| &r["attributes"])
                .unwrap_or(&style["textStyle"]["encodedAttributes"]);
            let font = &attrs["MSAttributedStringFontAttribute"]["attributes"];
            let post = font["name"].as_str().unwrap_or("Inter");
            layer.typography.family = Some(crate::fonts::family_from_postscript(post));
            layer.typography.bold = post.to_lowercase().contains("bold");
            layer.typography.italic = post.to_lowercase().contains("italic");
            layer.typography.size = Some(num(font, "size", 14.0) * scale[1]);
            layer.typography.tracking = num(attrs, "kerning", 0.0) * scale[0];
            if !attrs["MSAttributedStringColorAttribute"].is_null() {
                layer.color = color(&attrs["MSAttributedStringColorAttribute"])
            }
            let para = &attrs["paragraphStyle"];
            layer.typography.align = match para["alignment"].as_u64().unwrap_or(0) {
                1 => 2,
                2 => 1,
                3 => 3,
                _ => 0,
            };
            let leading = num(para, "maximumLineHeight", 0.0);
            if leading > 0.0 {
                layer.typography.line_height = Some(leading * scale[1])
            }
            layer.typography.underline = num(attrs, "underlineStyle", 0.0) > 0.0;
            layer.typography.strike = num(attrs, "strikethroughStyle", 0.0) > 0.0;
        } else if kind == Kind::Image {
            let reference = v["image"]["_ref"]
                .as_str()
                .ok_or("Bitmap has no image reference")?;
            let entry = if self.zip.index_for_name(reference).is_some() {
                reference.to_owned()
            } else {
                ["png", "jpg", "jpeg", "webp"]
                    .iter()
                    .map(|e| format!("{reference}.{e}"))
                    .find(|n| self.zip.index_for_name(n).is_some())
                    .ok_or("Missing Sketch bitmap")?
            };
            layer.image_data = Some(self.bytes(&entry)?);
        } else {
            if class == "rectangle" {
                layer.radius = num(
                    v,
                    "fixedRadius",
                    num(
                        &list(v, "points").first().cloned().unwrap_or(Value::Null),
                        "cornerRadius",
                        0.0,
                    ),
                ) * scale[0].min(scale[1]);
            }
            let borders: Vec<_> = list(style, "borders")
                .iter()
                .filter(|s| s["isEnabled"].as_bool().unwrap_or(true))
                .collect();
            if borders.len() > 1 {
                self.warn("Multiple borders reduced to the first border")
            }
            if class == "shapePath" || !borders.is_empty() || fill.is_none() {
                let mut path = vector::Path {
                    closed: v["isClosed"].as_bool().unwrap_or(true),
                    fill: fill.is_some(),
                    stroke: !borders.is_empty(),
                    stroke_width: borders
                        .first()
                        .map(|b| num(b, "thickness", 1.0) * scale[0])
                        .unwrap_or(1.0),
                    stroke_color: borders
                        .first()
                        .map(|b| color(&b["color"]))
                        .unwrap_or(layer.color),
                    ..Default::default()
                };
                for p in list(v, "points") {
                    let pt = point(&p["point"])?;
                    path.nodes.push(vector::Node {
                        point: pt,
                        incoming: if p["hasCurveTo"] == true {
                            point(&p["curveTo"])?
                        } else {
                            pt
                        },
                        outgoing: if p["hasCurveFrom"] == true {
                            point(&p["curveFrom"])?
                        } else {
                            pt
                        },
                        mode: match p["curveMode"].as_u64().unwrap_or(1) {
                            2 => vector::NodeMode::Symmetric,
                            3 => vector::NodeMode::Smooth,
                            _ => vector::NodeMode::Corner,
                        },
                    })
                }
                if path.nodes.is_empty() {
                    path.nodes = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]]
                        .into_iter()
                        .map(vector::Node::at)
                        .collect();
                    if class == "oval" {
                        self.warn(
                            "Border-only oval without curve points approximated by a rectangle",
                        )
                    }
                }
                layer.path = Some(path);
            }
        }
        if !list(style, "shadows").is_empty() || !list(style, "innerShadows").is_empty() {
            self.warn("Shadows are not imported")
        }
        if style["blur"]["isEnabled"] == true {
            self.warn("Blur effects are not imported")
        }
        out.push(layer);
        Ok(())
    }
}
fn apply_text_override(v: &mut Value, id: &str, value: &Value) {
    if v["do_objectID"] == id && v["_class"] == "text" {
        v["attributedString"]["string"] = value.clone();
    }
    if let Some(children) = v["layers"].as_array_mut() {
        for c in children {
            apply_text_override(c, id, value)
        }
    }
}
pub fn load(path: &Path) -> Result<Import, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > LIMIT {
        return Err("Sketch file exceeds 256 MB".into());
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    decode(
        &bytes,
        path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Sketch library"),
    )
}
pub fn decode(bytes: &[u8], title: &str) -> Result<Import, String> {
    let mut r = Reader {
        zip: ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?,
        budget: LIMIT,
        symbols: BTreeMap::new(),
        warnings: BTreeMap::new(),
        count: 0,
    };
    if r.zip.len() > 100_000 {
        return Err("Too many Sketch archive entries".into());
    }
    let document: Value =
        serde_json::from_slice(&r.bytes("document.json")?).map_err(|e| e.to_string())?;
    let mut pages = Vec::new();
    for reference in list(&document, "pages") {
        let n = reference["_ref"]
            .as_str()
            .ok_or("Missing Sketch page reference")?;
        let n = if n.ends_with(".json") {
            n.to_owned()
        } else {
            format!("{n}.json")
        };
        let page: Value = serde_json::from_slice(&r.bytes(&n)?).map_err(|e| e.to_string())?;
        r.collect_symbols(&page, 0)?;
        pages.push(page)
    }
    for foreign in list(&document, "foreignSymbols") {
        if let Some(master) = foreign.get("symbolMaster") {
            r.collect_symbols(master, 0)?
        }
    }
    let mut result = Document {
        name: title.into(),
        prototype: Default::default(),
        pages: vec![],
    };
    for page in pages {
        let mut boards = Vec::new();
        for node in list(&page, "layers") {
            let f = &node["frame"];
            let mut layers = Vec::new();
            if node["_class"] == "artboard" || node["_class"] == "symbolMaster" {
                for child in list(node, "layers") {
                    r.layers(child, [0., 0.], [1., 1.], None, true, false, &mut layers, 0)?
                }
            } else {
                let mut root = node.clone();
                root["frame"]["x"] = 0.into();
                root["frame"]["y"] = 0.into();
                r.layers(&root, [0., 0.], [1., 1.], None, true, false, &mut layers, 0)?
            }
            if layers.is_empty() {
                continue;
            }
            if num(f, "width", 640.0) > 8192.0 || num(f, "height", 440.0) > 8192.0 {
                r.warn("Oversized frames skipped; import smaller component frames");
                continue;
            }
            boards.push(Artboard {
                prototype_id: String::new(),
                name: name(node).into(),
                frame: FrameStyle {
                    width: num(f, "width", 640.0).max(1.),
                    height: num(f, "height", 440.0).max(1.),
                    background: if node["hasBackgroundColor"] == true {
                        color(&node["backgroundColor"])
                    } else {
                        [255; 3]
                    },
                    ..Default::default()
                },
                layers,
            });
        }
        if !boards.is_empty() {
            result.pages.push(Page {
                name: name(&page).into(),
                artboards: boards,
            })
        }
    }
    if result.pages.is_empty() {
        return Err("No supported editable assets found in Sketch file".into());
    }
    result.validate()?;
    Ok(Import {
        document: result,
        warnings: r
            .warnings
            .into_iter()
            .map(|(s, n)| format!("{s} ({n})"))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    fn archive(page: Value, extra: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
        let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, data) in [
            (
                "document.json",
                serde_json::to_vec(&json!({"pages":[{"_ref":"pages/p"}]})).unwrap(),
            ),
            ("pages/p.json", serde_json::to_vec(&page).unwrap()),
        ]
        .into_iter()
        .chain(extra)
        {
            z.start_file(name, opts).unwrap();
            z.write_all(&data).unwrap();
        }
        z.finish().unwrap().into_inner()
    }
    fn rect(id: &str) -> Value {
        json!({"_class":"rectangle","do_objectID":id,"name":"Background","frame":{"x":4,"y":6,"width":100,"height":40},"fixedRadius":8,"style":{"fills":[{"isEnabled":true,"fillType":0,"color":{"red":1,"green":0.5,"blue":0,"alpha":1}}]}})
    }
    #[test]
    fn compressed_symbols_text_overrides_and_handles_stay_editable() {
        let text = json!({"_class":"text","do_objectID":"label","name":"Label","frame":{"x":10,"y":12,"width":80,"height":20},"attributedString":{"string":"Original","attributes":[{"attributes":{"MSAttributedStringFontAttribute":{"attributes":{"name":"Inter","size":14}},"MSAttributedStringColorAttribute":{"red":0,"green":0,"blue":0},"paragraphStyle":{"alignment":2}}}]}});
        let path = json!({"_class":"shapePath","name":"Curve","frame":{"x":0,"y":0,"width":100,"height":100},"isClosed":false,"points":[{"point":"{0, 0}","curveFrom":"{0.25, 0.5}","hasCurveFrom":true},{"point":"{1, 1}","curveTo":"{0.75, 0.5}","hasCurveTo":true}],"style":{"borders":[{"isEnabled":true,"thickness":2,"color":{"red":0,"green":0,"blue":1}}]}});
        let page = json!({"_class":"page","name":"Components","layers":[{"_class":"symbolMaster","symbolID":"button","name":"Button","frame":{"width":120,"height":50},"layers":[rect("bg"),text]},{"_class":"artboard","name":"Examples","frame":{"width":400,"height":300},"layers":[{"_class":"symbolInstance","name":"Double size","symbolID":"button","frame":{"x":20,"y":30,"width":240,"height":100},"overrideValues":[{"overrideName":"label_stringValue","value":"Changed"}]},path]}]});
        let imported = decode(&archive(page, vec![]), "Test kit").unwrap();
        let boards = &imported.document.pages[0].artboards;
        assert_eq!(boards[0].layers[1].name, "Original");
        assert_eq!(boards[1].layers[1].name, "Changed");
        assert_eq!(boards[1].layers[0].x, 28.0);
        assert_eq!(boards[1].layers[0].width, 200.0);
        assert_eq!(
            boards[1].layers[2].path.as_ref().unwrap().nodes[0].outgoing,
            [0.25, 0.5]
        );
        assert_eq!(
            boards[1].layers[2].path.as_ref().unwrap().nodes[1].incoming,
            [0.75, 0.5]
        );
        assert!(imported.warnings.is_empty());
        let bytes =
            crate::document_file::encode(&imported.document, Path::new("kit.suiza-ui")).unwrap();
        assert!(crate::document_file::decode(&bytes).unwrap() == imported.document);
    }
    #[test]
    fn unsupported_features_are_reported_and_unsafe_archives_rejected() {
        let mut layer = rect("bg");
        layer["rotation"] = 45.into();
        layer["style"]["fills"][0]["fillType"] = 1.into();
        let p = json!({"_class":"page","layers":[{"_class":"artboard","frame":{"width":200,"height":100},"layers":[layer,{"_class":"symbolInstance","symbolID":"missing","frame":{"width":10,"height":10}}]}]});
        let bytes = archive(p, vec![]);
        let imported = decode(&bytes, "Warnings").unwrap();
        assert_eq!(imported.warnings.len(), 3);
        assert!(decode(&bytes[..bytes.len() / 2], "Broken").is_err());
        let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
        z.start_file("document.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(br#"{"pages":[{"_ref":"../secret"}]}"#).unwrap();
        assert!(decode(&z.finish().unwrap().into_inner(), "Unsafe").is_err());
    }
    #[test]
    fn cyclic_symbols_stop_without_unbounded_recursion() {
        let p = json!({"_class":"page","layers":[{"_class":"symbolMaster","symbolID":"cycle","frame":{"width":100,"height":100},"layers":[{"_class":"symbolInstance","symbolID":"cycle","frame":{"width":100,"height":100}}]}]});
        assert!(decode(&archive(p, vec![]), "Cycle")
            .unwrap_err_string()
            .contains("limits"));
    }
    trait ErrorString {
        fn unwrap_err_string(self) -> String;
    }
    impl ErrorString for Result<Import, String> {
        fn unwrap_err_string(self) -> String {
            match self {
                Err(e) => e,
                Ok(_) => panic!("Expected error"),
            }
        }
    }
    #[test]
    #[ignore = "Optional local fixture; source file is never copied into repository"]
    fn local_sketch_fixture_imports() {
        let p = std::env::var("SUIZA_SKETCH_FIXTURE").expect("Set SUIZA_SKETCH_FIXTURE");
        let imported = load(Path::new(&p)).unwrap();
        assert!(!imported.document.pages[0].artboards[0].layers.is_empty());
        imported.document.validate().unwrap();
    }
}
