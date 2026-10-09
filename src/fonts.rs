//! Installed fonts are discovered locally and never embedded in saved documents.
use super::{egui, TypeStyle};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};

fn database() -> &'static fontdb::Database {
    static DB: OnceLock<fontdb::Database> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    })
}
#[derive(Clone)]
struct LoadedFont {
    data: Arc<Vec<u8>>,
    index: u32,
    font: ab_glyph::FontArc,
    italic: bool,
}
fn installed(family: &str, bold: bool, italic: bool) -> Option<LoadedFont> {
    static CACHE: OnceLock<Mutex<BTreeMap<(String, bool, bool), Option<LoadedFont>>>> =
        OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().ok()?;
    cache
        .entry((family.to_owned(), bold, italic))
        .or_insert_with(|| {
            let id = database().query(&fontdb::Query {
                families: &[fontdb::Family::Name(family)],
                weight: if bold {
                    fontdb::Weight::BOLD
                } else {
                    fontdb::Weight::NORMAL
                },
                style: if italic {
                    fontdb::Style::Italic
                } else {
                    fontdb::Style::Normal
                },
                ..Default::default()
            })?;
            database().with_face_data(id, |data, index| {
                let font = ab_glyph::FontVec::try_from_vec_and_index(data.to_vec(), index).ok()?;
                Some(LoadedFont {
                    data: Arc::new(data.to_vec()),
                    index,
                    font: font.into(),
                    italic: database()
                        .face(id)
                        .is_some_and(|f| f.style != fontdb::Style::Normal),
                })
            })?
        })
        .clone()
}
pub(super) fn render_font(style: &TypeStyle) -> ab_glyph::FontArc {
    if let Some(font) = style
        .family
        .as_deref()
        .and_then(|name| installed(name, style.bold, style.italic))
    {
        return font.font;
    }
    ab_glyph::FontArc::try_from_slice(if style.monospace {
        epaint_default_fonts::HACK_REGULAR
    } else if style.bold {
        include_bytes!("../assets/fonts/UI-Bold.ttf")
    } else {
        epaint_default_fonts::UBUNTU_LIGHT
    })
    .expect("bundled font")
}
pub(super) fn editor_family(ctx: &egui::Context, style: &TypeStyle) -> egui::FontFamily {
    if let Some(name) = &style.family {
        if let Some(font) = installed(name, style.bold, style.italic) {
            let key = format!("installed:{name}:{}:{}", style.bold, style.italic);
            let mut definitions = ctx
                .data(|d| d.get_temp::<egui::FontDefinitions>(egui::Id::new("font_definitions")))
                .unwrap_or_default();
            let family = egui::FontFamily::Name(key.clone().into());
            if definitions.families.contains_key(&family) {
                return family;
            }
            let mut data = egui::FontData::from_owned((*font.data).clone());
            data.index = font.index;
            definitions.font_data.insert(key.clone(), data.into());
            definitions.families.insert(family, vec![key]);
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("font_definitions"), definitions.clone()));
            ctx.set_fonts(definitions);
            ctx.request_repaint();
        }
    }
    if style.monospace {
        egui::FontFamily::Name("Document Mono".into())
    } else if style.bold {
        egui::FontFamily::Name("Document Bold".into())
    } else {
        egui::FontFamily::Name("Document Regular".into())
    }
}
pub(super) fn family_picker(ui: &mut egui::Ui, family: &mut Option<String>, mono: &mut bool) {
    let name = family
        .as_deref()
        .unwrap_or(if *mono { "Hack" } else { "Ubuntu" });
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(rect, 6.0, visuals.bg_fill);
    ui.painter().text(
        rect.left_center() + egui::vec2(10.0, 0.0),
        egui::Align2::LEFT_CENTER,
        name,
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );
    let center = rect.right_center() - egui::vec2(12.0, 0.0);
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            center + egui::vec2(-3.0, -2.0),
            center + egui::vec2(3.0, -2.0),
            center + egui::vec2(0.0, 2.0),
        ],
        ui.visuals().text_color(),
        egui::Stroke::NONE,
    ));
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name));
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new("font_picker_anchor"), response.rect));
    if response
        .on_hover_text("Browse fonts · In Use / All Fonts")
        .clicked()
    {
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new("open_font_browser"), true));
    }
}

#[derive(Default)]
struct BrowserFonts {
    names: Option<Vec<String>>,
    previews: BTreeMap<String, Option<egui::ColorImage>>,
}
struct PreviewWorker {
    state: Arc<Mutex<BrowserFonts>>,
    sender: std::sync::mpsc::Sender<(String, egui::Context)>,
}
fn preview_worker(ctx: &egui::Context) -> &'static PreviewWorker {
    static WORKER: OnceLock<PreviewWorker> = OnceLock::new();
    WORKER.get_or_init(|| {
        let state = Arc::new(Mutex::new(BrowserFonts::default()));
        let (sender, receiver) = std::sync::mpsc::channel::<(String, egui::Context)>();
        let shared = state.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let mut names = vec!["Ubuntu".to_owned(), "Hack".to_owned()];
            names.extend(
                database()
                    .faces()
                    .flat_map(|f| f.families.iter().map(|(n, _)| n.clone()))
                    .filter(|n| !n.starts_with('.')),
            );
            names.sort_by_key(|s| s.to_lowercase());
            names.dedup();
            shared.lock().unwrap().names = Some(names);
            ctx.request_repaint();
            while let Ok((name, ctx)) = receiver.recv() {
                let image = preview_image(&name);
                let mut cache = shared.lock().unwrap();
                if cache.previews.len() > 128 {
                    if let Some(key) = cache.previews.keys().next().cloned() {
                        cache.previews.remove(&key);
                    }
                }
                cache.previews.insert(name, image);
                drop(cache);
                ctx.request_repaint();
            }
        });
        PreviewWorker { state, sender }
    })
}
fn preview_image(name: &str) -> Option<egui::ColorImage> {
    use ab_glyph::{Font, ScaleFont};
    let (data, index) = if name == "Ubuntu" {
        (epaint_default_fonts::UBUNTU_LIGHT.to_vec(), 0)
    } else if name == "Hack" {
        (epaint_default_fonts::HACK_REGULAR.to_vec(), 0)
    } else {
        let id = database().query(&fontdb::Query {
            families: &[fontdb::Family::Name(name)],
            ..Default::default()
        })?;
        database().with_face_data(id, |data, index| (data.to_vec(), index))?
    };
    let font = ab_glyph::FontVec::try_from_vec_and_index(data.clone(), index).ok()?;
    // Some symbol/non-Latin fonts cannot render their Latin family name. Keep
    // their name readable in the UI instead of showing missing-glyph boxes.
    if name
        .chars()
        .any(|c| !c.is_whitespace() && font.glyph_id(c).0 == 0)
    {
        return None;
    }
    let scaled = font.as_scaled(36.0);
    let face = rustybuzz::Face::from_slice(&data, index)?;
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(name);
    buffer.guess_segment_properties();
    let shape = rustybuzz::shape(&face, &[], buffer);
    let factor = scaled.scale_factor();
    let mut image = image::RgbaImage::new(512, 52);
    let mut x = 0.0;
    let baseline = 26.0 + (scaled.ascent() + scaled.descent()) / 2.0;
    for (info, pos) in shape.glyph_infos().iter().zip(shape.glyph_positions()) {
        let glyph = ab_glyph::GlyphId(info.glyph_id as u16).with_scale_and_position(
            scaled.scale(),
            ab_glyph::point(
                x + pos.x_offset as f32 * factor.horizontal,
                baseline - pos.y_offset as f32 * factor.vertical,
            ),
        );
        if let Some(outline) = font.outline_glyph(glyph) {
            let bounds = outline.px_bounds();
            outline.draw(|px, py, a| {
                let px = px as i32 + bounds.min.x as i32;
                let py = py as i32 + bounds.min.y as i32;
                if px >= 0 && py >= 0 && px < 512 && py < 52 {
                    image.put_pixel(
                        px as u32,
                        py as u32,
                        image::Rgba([255, 255, 255, (a * 255.0) as u8]),
                    );
                }
            });
        }
        x += pos.x_advance as f32 * factor.horizontal;
    }
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [512, 52],
        image.as_raw(),
    ))
}

pub(super) fn font_browser(
    ui: &mut egui::Ui,
    family: &mut Option<String>,
    mono: &mut bool,
) -> bool {
    ui.set_width(280.0);
    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
    let worker = preview_worker(ui.ctx());
    let names = worker.state.lock().ok().and_then(|s| s.names.clone());
    let id = egui::Id::new("font_browser");
    let mut in_use = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(id.with("in_use")))
        .unwrap_or(false);
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 6.0) / 2.0;
        if ui
            .add_sized([width, 28.0], egui::Button::new("In Use").selected(in_use))
            .clicked()
        {
            in_use = true;
        }
        if ui
            .add_sized(
                [width, 28.0],
                egui::Button::new("All Fonts").selected(!in_use),
            )
            .clicked()
        {
            in_use = false;
        }
    });
    ui.ctx()
        .data_mut(|d| d.insert_temp(id.with("in_use"), in_use));
    let mut search = ui
        .ctx()
        .data(|d| d.get_temp::<String>(id.with("search")))
        .unwrap_or_default();
    let input = ui.add_sized(
        [280.0, 28.0],
        egui::TextEdit::singleline(&mut search).hint_text("Search fonts"),
    );
    if ui
        .ctx()
        .data_mut(|d| d.remove_temp::<bool>(egui::Id::new("focus_font_search")))
        .unwrap_or(false)
    {
        input.request_focus();
    }
    ui.ctx()
        .data_mut(|d| d.insert_temp(id.with("search"), search.clone()));
    let used = ui
        .ctx()
        .data(|d| d.get_temp::<Vec<String>>(egui::Id::new("document_fonts")))
        .unwrap_or_default();
    let Some(names) = names else {
        ui.weak("Loading installed fonts…");
        return false;
    };
    let query = search.to_lowercase();
    let names: Vec<_> = names
        .iter()
        .filter(|n| (!in_use || used.contains(n)) && n.to_lowercase().contains(&query))
        .collect();
    let mut chosen = false;
    egui::ScrollArea::vertical()
        .id_salt("font_rows")
        .max_height(300.0)
        .min_scrolled_height(300.0)
        .auto_shrink([false, false])
        .show_rows(ui, 30.0, names.len(), |ui, range| {
            for row in range {
                let name = names[row];
                let bundled = name == "Ubuntu" || name == "Hack";
                let selected = family.as_ref() == Some(name)
                    || (family.is_none() && *mono == (name == "Hack") && bundled);
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(280.0, 30.0), egui::Sense::click());
                if selected || response.hovered() {
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        if selected {
                            crate::ACCENT
                        } else {
                            ui.visuals().widgets.hovered.bg_fill
                        },
                    );
                }
                let color = if selected {
                    egui::Color32::WHITE
                } else {
                    ui.visuals().text_color()
                };
                let texture_id = id.with(("preview", name));
                let mut texture = ui
                    .ctx()
                    .data(|d| d.get_temp::<egui::TextureHandle>(texture_id));
                if texture.is_none() {
                    let mut cache = worker.state.lock().unwrap();
                    if let Some(Some(image)) = cache.previews.get(name) {
                        let loaded = ui.ctx().load_texture(
                            format!("font-preview:{name}"),
                            image.clone(),
                            egui::TextureOptions::LINEAR,
                        );
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(texture_id, loaded.clone()));
                        texture = Some(loaded);
                    } else if !cache.previews.contains_key(name) {
                        cache.previews.insert(name.clone(), None);
                        let _ = worker.sender.send((name.clone(), ui.ctx().clone()));
                    }
                }
                if let Some(texture) = texture {
                    ui.painter().image(
                        texture.id(),
                        egui::Rect::from_min_size(
                            rect.left_top() + egui::vec2(10.0, 2.0),
                            egui::vec2(256.0, 26.0),
                        ),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        color,
                    );
                } else {
                    ui.painter().text(
                        rect.left_center() + egui::vec2(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        name,
                        egui::FontId::proportional(13.0),
                        color,
                    );
                }
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::SelectableLabel,
                        true,
                        selected,
                        name,
                    )
                });
                if response.clicked() {
                    *family = if bundled { None } else { Some(name.clone()) };
                    *mono = name == "Hack";
                    chosen = true;
                }
            }
        });
    if names.is_empty() {
        ui.weak("No matching fonts");
    }
    chosen
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browsing_previews_does_not_register_fonts_in_the_editor_atlas() {
        let ctx = egui::Context::default();
        crate::configure_fonts(&ctx);
        let before = ctx
            .data(|d| d.get_temp::<egui::FontDefinitions>(egui::Id::new("font_definitions")))
            .unwrap()
            .font_data
            .len();
        let mut family = None;
        let mut mono = false;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                font_browser(ui, &mut family, &mut mono);
            });
        });
        let after = ctx
            .data(|d| d.get_temp::<egui::FontDefinitions>(egui::Id::new("font_definitions")))
            .unwrap()
            .font_data
            .len();
        assert_eq!(before, after);
        let image = preview_image("Ubuntu").unwrap();
        assert_eq!(image.size, [512, 52]);
        assert!(image.pixels.iter().any(|p| p.a() > 0));
    }
    #[test]
    fn system_font_uses_its_actual_face_and_preserves_family() {
        let candidate = database()
            .faces()
            .flat_map(|f| f.families.iter())
            .map(|(n, _)| n)
            .find(|name| installed(name, false, false).is_some())
            .expect("test host has system fonts");
        let style = TypeStyle {
            family: Some(candidate.clone()),
            ..Default::default()
        };
        let font = render_font(&style);
        use ab_glyph::Font;
        assert_ne!(font.glyph_id('A').0, 0);
        let restored: TypeStyle =
            serde_json::from_str(&serde_json::to_string(&style).unwrap()).unwrap();
        assert_eq!(restored.family, style.family);
    }
}

pub(super) fn shaping_data(style: &TypeStyle) -> (Arc<Vec<u8>>, u32, bool) {
    if let Some(font) = style
        .family
        .as_deref()
        .and_then(|n| installed(n, style.bold, style.italic))
    {
        return (font.data, font.index, style.italic && !font.italic);
    }
    let bytes: &[u8] = if style.monospace {
        epaint_default_fonts::HACK_REGULAR
    } else if style.bold {
        include_bytes!("../assets/fonts/UI-Bold.ttf")
    } else {
        epaint_default_fonts::UBUNTU_LIGHT
    };
    (Arc::new(bytes.to_vec()), 0, style.italic)
}
