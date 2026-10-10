//! Local asset libraries and reusable, editable groups. No hosted service required.
use crate::{Artboard, Document, FrameStyle, Kind, Layer};
use eframe::egui::{self, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub document: Document,
    #[serde(default)]
    pub warnings: Vec<String>,
}
pub struct AssetsUi {
    pub active: bool,
    search: String,
    pub libraries: Vec<Library>,
    selected: usize,
    catalog: bool,
    cached_components: Option<(usize, Vec<Artboard>)>,
    importing: Option<std::sync::mpsc::Receiver<Result<Library, String>>>,
}
impl Default for AssetsUi {
    fn default() -> Self {
        Self {
            active: false,
            search: String::new(),
            libraries: Vec::new(),
            selected: 1,
            catalog: false,
            importing: None,
            cached_components: None,
        }
    }
}
impl AssetsUi {
    pub fn show(&mut self, ui: &mut egui::Ui, doc: &Document) -> Result<Option<Artboard>, String> {
        if let Some(receiver) = &self.importing {
            match receiver.try_recv() {
                Ok(result) => {
                    self.importing = None;
                    self.cached_components = None;
                    let library = result?;
                    self.libraries.push(library);
                    self.selected = self.libraries.len() + 1;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.importing = None;
                    return Err("Library import interrupted".into());
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            crate::section_heading(ui, "Assets");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(
                        self.importing.is_none(),
                        egui::Button::new("Import…").small(),
                    )
                    .on_hover_text("Import Sketch, Suiza-UI or JSON libraries")
                    .clicked()
                {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Asset libraries", &["sketch", "suiza-ui", "json"])
                        .pick_file()
                    {
                        let (sender, receiver) = std::sync::mpsc::channel();
                        self.importing = Some(receiver);
                        let ctx = ui.ctx().clone();
                        std::thread::spawn(move || {
                            let _ = sender.send(load_library(&path));
                            ctx.request_repaint();
                        });
                    }
                }
            });
        });
        if self.importing.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Importing library…");
            });
        }
        ui.add_space(8.0);
        crate::search_field(
            ui,
            "asset_search",
            &mut self.search,
            "Search assets…",
            ui.available_width(),
        );
        ui.add_space(8.0);
        self.selected = self.selected.min(self.libraries.len() + 1);
        let label = match self.selected {
            0 => "This document",
            1 => "Suiza essentials",
            n => &self.libraries[n - 2].name,
        };
        egui::ComboBox::from_id_salt("asset_library")
            .width(ui.available_width())
            .selected_text(label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected, 0, "This document");
                ui.selectable_value(&mut self.selected, 1, "Suiza essentials");
                for (i, library) in self.libraries.iter().enumerate() {
                    ui.selectable_value(&mut self.selected, i + 2, &library.name);
                }
            });
        if self.selected >= 2 && ui.small_button("Remove library").clicked() {
            self.libraries.remove(self.selected - 2);
            self.cached_components = None;
            self.selected = 0;
        }
        if self.selected >= 2 && !self.libraries[self.selected - 2].warnings.is_empty() {
            ui.collapsing("Import notes", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .id_salt("import_notes")
                    .show(ui, |ui| {
                        for note in &self.libraries[self.selected - 2].warnings {
                            ui.small(note);
                        }
                    });
            });
        }
        ui.add_space(8.0);
        if self.catalog {
            if ui.small_button("← Back to assets").clicked() {
                self.catalog = false;
            }
            ui.add_space(8.0);
            crate::section_heading(ui, "Platform UI kits");
            egui::ScrollArea::vertical()
                .id_salt("kit_catalog")
                .show(ui, |ui| {
                    for (index, title, publisher, url) in [
                        (
                            0,
                            "iOS & iPadOS 27",
                            "Apple Design Resources",
                            "https://developer.apple.com/design/resources/",
                        ),
                        (
                            1,
                            "macOS 27",
                            "Apple Design Resources",
                            "https://developer.apple.com/design/resources/",
                        ),
                        (
                            2,
                            "Material 3",
                            "Google Material Design",
                            "https://m3.material.io/get-started",
                        ),
                    ] {
                        kit_card(ui, index, title, publisher, url);
                        ui.add_space(20.0);
                    }
                });
            return Ok(None);
        }
        let local_components;
        let components = if self.selected == 0 {
            local_components = components(doc);
            &local_components
        } else {
            if self
                .cached_components
                .as_ref()
                .is_none_or(|(selected, _)| *selected != self.selected)
            {
                let items = if self.selected == 1 {
                    components(&essentials())
                } else {
                    components(&self.libraries[self.selected - 2].document)
                };
                self.cached_components = Some((self.selected, items));
            }
            &self.cached_components.as_ref().unwrap().1
        };
        let search = self.search.to_lowercase();
        let filtered: Vec<_> = components
            .iter()
            .filter(|b| b.name.to_lowercase().contains(&search))
            .collect();
        ui.label(
            egui::RichText::new(format!("{} assets", filtered.len()))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        let mut insert = None;
        egui::ScrollArea::vertical()
            .id_salt("assets_browser")
            .max_height((ui.available_height() - 150.0).max(80.0))
            .show_rows(ui, 190.0, filtered.len(), |ui, rows| {
                ui.set_max_width(ui.available_width());
                for component in &filtered[rows] {
                    let preview = Document {
                        name: component.name.clone(),
                        prototype: Default::default(),
                        pages: vec![crate::Page {
                            name: "Assets".into(),
                            artboards: vec![(*component).clone()],
                        }],
                    };
                    if crate::welcome_document_card(
                        ui,
                        &preview,
                        &component.name,
                        "Insert editable group",
                        ui.available_width(),
                    )
                    .clicked()
                    {
                        insert = Some((*component).clone());
                    }
                    ui.add_space(10.0);
                }
            });
        ui.add_space(8.0);
        ui.separator();
        if ui
            .add_sized(
                [ui.available_width(), 32.0],
                egui::Button::new("Browse platform UI kits →"),
            )
            .clicked()
        {
            self.catalog = true;
        }
        Ok(insert)
    }
}
fn components(doc: &Document) -> Vec<Artboard> {
    let mut out = Vec::new();
    for page in &doc.pages {
        for board in &page.artboards {
            if !board.layers.is_empty() {
                out.push(board.clone());
            }
            let groups: std::collections::BTreeSet<_> = board
                .layers
                .iter()
                .filter_map(|l| l.group.clone())
                .collect();
            for name in groups {
                let mut layers: Vec<_> = board
                    .layers
                    .iter()
                    .filter(|l| l.group.as_ref() == Some(&name))
                    .cloned()
                    .collect();
                let x = layers.iter().map(|l| l.x).fold(f32::INFINITY, f32::min);
                let y = layers.iter().map(|l| l.y).fold(f32::INFINITY, f32::min);
                let right = layers
                    .iter()
                    .map(|l| l.x + l.width)
                    .fold(f32::NEG_INFINITY, f32::max);
                let bottom = layers
                    .iter()
                    .map(|l| l.y + l.height)
                    .fold(f32::NEG_INFINITY, f32::max);
                for layer in &mut layers {
                    layer.x -= x;
                    layer.y -= y;
                }
                out.push(Artboard {
                    prototype_id: String::new(),
                    name,
                    frame: FrameStyle {
                        width: (right - x).max(1.0),
                        height: (bottom - y).max(1.0),
                        ..Default::default()
                    },
                    layers,
                });
            }
        }
    }
    out
}
fn essentials() -> Document {
    let mut boards = Vec::new();
    for (name, w, h, radius, color) in [
        ("Primary button", 160.0, 44.0, 12.0, [96, 83, 255]),
        ("Text field", 240.0, 44.0, 8.0, [238, 238, 244]),
        ("Card", 280.0, 160.0, 16.0, [238, 238, 244]),
        ("Navigation bar", 390.0, 56.0, 0.0, [238, 238, 244]),
        ("Dialog", 320.0, 200.0, 16.0, [238, 238, 244]),
        ("Chip", 90.0, 32.0, 16.0, [235, 232, 255]),
    ] {
        let mut base = Layer::new(Kind::Rectangle, 0.0, 0.0);
        base.width = w;
        base.height = h;
        base.radius = radius;
        base.color = color;
        base.label = Some(name.into());
        let mut label = Layer::new(
            Kind::Text,
            16.0,
            if h <= 56.0 { (h - 20.0) / 2.0 } else { 20.0 },
        );
        label.name = if name == "Primary button" {
            "Continue"
        } else {
            name
        }
        .into();
        label.width = w - 32.0;
        label.height = 24.0;
        label.color = if name == "Primary button" {
            [255, 255, 255]
        } else {
            [40, 40, 48]
        };
        label.typography.size = Some(14.0);
        label.typography.family = Some("Inter".into());
        boards.push(Artboard {
            prototype_id: String::new(),
            name: name.into(),
            frame: FrameStyle {
                width: w,
                height: h,
                ..Default::default()
            },
            layers: vec![base, label],
        });
    }
    Document {
        name: "Suiza essentials".into(),
        prototype: Default::default(),
        pages: vec![crate::Page {
            name: "Components".into(),
            artboards: boards,
        }],
    }
}
pub fn insert_layers(
    component: &Artboard,
    target: &mut Artboard,
    position: Vec2,
) -> (String, Vec<usize>) {
    let mut suffix = 1;
    let mut group = component.name.clone();
    while target
        .layers
        .iter()
        .any(|l| l.group.as_ref() == Some(&group))
    {
        suffix += 1;
        group = format!("{} {}", component.name, suffix);
    }
    let start = target.layers.len();
    for mut layer in component.layers.clone() {
        layer.prototype_id.clear();
        layer.group = Some(group.clone());
        layer.locked = false;
        layer.x += position.x;
        layer.y += position.y;
        target.layers.push(layer);
    }
    (group, (start..target.layers.len()).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn insertions_are_independent_editable_groups_and_survive_save() {
        let library = essentials();
        let component = &library.pages[0].artboards[0];
        let mut doc = Document::default();
        let target = &mut doc.pages[0].artboards[0];
        let (first, indices) = insert_layers(component, target, Vec2::new(20.0, 30.0));
        let (second, _) = insert_layers(component, target, Vec2::new(40.0, 50.0));
        assert_ne!(first, second);
        assert_eq!(indices.len(), 2);
        assert_eq!(target.layers[indices[0]].x, 20.0);
        assert!(Document::from_json(&serde_json::to_string(&doc).unwrap()).is_ok());
    }
}

fn load_library(path: &std::path::Path) -> Result<Library, String> {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("sketch"))
    {
        let imported = crate::sketch_import::load(path)?;
        Ok(Library {
            name: imported.document.name.clone(),
            document: imported.document,
            warnings: imported.warnings,
        })
    } else {
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            if file.metadata().map_err(|e| e.to_string())?.len() > 256 * 1024 * 1024 {
                return Err("Library exceeds 256 MB".into());
            }
            let value: serde_json::Value =
                serde_json::from_reader(file).map_err(|e| e.to_string())?;
            if value["format"] == "suiza-library" {
                if value["version"] != 1 {
                    return Err("Unsupported library version".into());
                }
                let document: Document =
                    serde_json::from_value(value["document"].clone()).map_err(|e| e.to_string())?;
                document.validate()?;
                let warnings = value["warnings"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default();
                return Ok(Library {
                    name: document.name.clone(),
                    document,
                    warnings,
                });
            }
        }
        let document = crate::document_file::load(path)?;
        Ok(Library {
            name: document.name.clone(),
            document,
            warnings: Vec::new(),
        })
    }
}

#[cfg(test)]
mod import_tests {
    use super::*;
    #[test]
    fn bridge_envelope_imports_and_preserves_notes_through_preferences() {
        let path = std::env::temp_dir().join(format!("suiza-bridge-{}.json", std::process::id()));
        let doc = essentials();
        std::fs::write(&path,serde_json::to_vec(&serde_json::json!({"format":"suiza-library","version":1,"document":doc,"warnings":["Effect rendered as an image"]})).unwrap()).unwrap();
        let imported = load_library(&path).unwrap();
        assert_eq!(imported.warnings, vec!["Effect rendered as an image"]);
        let stored = serde_json::to_string(&imported).unwrap();
        let restored: Library = serde_json::from_str(&stored).unwrap();
        assert!(restored.document == doc);
        assert_eq!(restored.warnings, imported.warnings);
        std::fs::write(&path, br#"{"format":"suiza-library","version":99}"#).unwrap();
        assert!(load_library(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    #[ignore = "Requires generated independent Figma bridge fixture"]
    fn actual_bridge_output_imports() {
        let path = std::env::var("SUIZA_BRIDGE_FIXTURE").unwrap();
        let imported = load_library(std::path::Path::new(&path)).unwrap();
        let board = &imported.document.pages[0].artboards[0];
        assert_eq!(board.layers[1].name, "Continue");
        assert_eq!(board.layers[1].x, 16.0);
        assert_eq!(board.layers[1].typography.family.as_deref(), Some("Inter"));
        let native =
            crate::document_file::encode(&imported.document, std::path::Path::new("kit.suiza-ui"))
                .unwrap();
        assert!(crate::document_file::decode(&native).unwrap() == imported.document);
    }
}

/// Original code-drawn covers; publisher assets are not bundled.
fn kit_card(ui: &mut egui::Ui, index: usize, title: &str, publisher: &str, url: &str) {
    use egui::{Align2, Color32, FontId, Pos2, Rect, Sense, Stroke};
    let width = ui.available_width();
    let height = width * 0.56;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let painter = ui.painter().with_clip_rect(rect);
    let palettes = [
        [
            Color32::from_rgb(65, 49, 63),
            Color32::from_rgb(170, 132, 91),
        ],
        [
            Color32::from_rgb(28, 43, 73),
            Color32::from_rgb(104, 125, 173),
        ],
        [
            Color32::from_rgb(77, 30, 116),
            Color32::from_rgb(184, 111, 241),
        ],
    ];
    painter.rect_filled(rect, 10.0, palettes[index][0]);
    if index < 2 {
        for n in 0..5 {
            let center = rect.right_bottom() + Vec2::new(20.0 + n as f32 * 16.0, 30.0);
            painter.circle_stroke(
                center,
                width * (0.36 + n as f32 * 0.12),
                Stroke::new(
                    width * 0.10,
                    palettes[index][1].gamma_multiply(0.25 + n as f32 * 0.12),
                ),
            );
        }
    } else {
        painter.circle_filled(
            rect.right_top() + Vec2::new(-width * 0.18, height * 0.24),
            width * 0.28,
            palettes[index][1],
        );
        painter.rect_filled(
            Rect::from_min_size(
                rect.right_bottom() - Vec2::new(width * 0.5, height * 0.38),
                Vec2::new(width * 0.42, height * 0.3),
            ),
            18.0,
            Color32::from_rgb(211, 163, 249),
        );
    }
    painter.text(
        rect.left_top() + Vec2::new(16.0, 18.0),
        Align2::LEFT_TOP,
        format!("{title}\nUI Kit"),
        FontId::proportional((width * 0.075).clamp(16.0, 24.0)),
        Color32::WHITE,
    );
    painter.text(
        Pos2::new(rect.left() + 16.0, rect.bottom() - 14.0),
        Align2::LEFT_BOTTOM,
        "DESIGN RESOURCES",
        FontId::proportional(9.0),
        Color32::WHITE,
    );
    if response.hovered() {
        painter.rect_stroke(
            rect.shrink(0.5),
            10.0,
            Stroke::new(1.5_f32, crate::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title));
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Open the publisher’s official kit page")
        .clicked()
    {
        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
    }
    ui.add_space(8.0);
    ui.label(egui::RichText::new(title).strong());
    ui.label(
        egui::RichText::new(publisher)
            .small()
            .color(ui.visuals().weak_text_color()),
    );
    ui.hyperlink_to("View official kit ↗", url);
}
