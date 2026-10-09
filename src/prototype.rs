use crate::{egui, Artboard, Document, Rect, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Prototype {
    pub flows: Vec<Flow>,
    pub interactions: Vec<Interaction>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    pub name: String,
    pub home: String,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Trigger {
    Tap,
    Hover,
    Drag,
    Key,
    Time,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Action {
    Transition,
    Overlay,
    Previous,
    CloseOverlay,
    Url,
    ScrollTo,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Animation {
    None,
    Dissolve,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Interaction {
    pub source: String,
    pub trigger: Trigger,
    pub action: Action,
    pub target: String,
    pub animation: Animation,
    pub duration: f32,
    pub delay: f32,
    pub key: String,
    pub url: String,
    pub scroll_y: f32,
    pub overlay_x: f32,
    pub overlay_y: f32,
}
impl Default for Interaction {
    fn default() -> Self {
        Self {
            source: String::new(),
            trigger: Trigger::Tap,
            action: Action::Transition,
            target: String::new(),
            animation: Animation::Dissolve,
            duration: 0.3,
            delay: 1.0,
            key: "Enter".into(),
            url: "https://".into(),
            scroll_y: 0.0,
            overlay_x: 0.0,
            overlay_y: 0.0,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
pub enum Preset {
    DesignReview,
    Development,
    Presentation,
    UserTesting,
    Custom,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ShareSettings {
    pub preset: Preset,
    pub navigation: bool,
    pub hints: bool,
    pub fullscreen: bool,
    pub comments: bool,
    pub specs: bool,
}
impl Default for ShareSettings {
    fn default() -> Self {
        Self::preset(Preset::DesignReview)
    }
}
impl ShareSettings {
    fn preset(preset: Preset) -> Self {
        Self {
            preset,
            navigation: preset != Preset::UserTesting,
            hints: preset != Preset::UserTesting,
            fullscreen: matches!(preset, Preset::Presentation | Preset::UserTesting),
            comments: matches!(
                preset,
                Preset::DesignReview | Preset::Development | Preset::Custom
            ),
            specs: preset == Preset::Development,
        }
    }
}
#[derive(Default)]
pub struct PrototypeUi {
    editing: bool,
    prototype_menu: bool,
    sharing: bool,
    flow: usize,
    settings: ShareSettings,
    wire_source: Option<String>,
    preview_requested: bool,
}
fn uid() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    format!(
        "p-{:x}-{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        COUNT.fetch_add(1, Ordering::Relaxed)
    )
}
/// IDs are stable across rename/reorder and assigned lazily to older documents.
pub fn ensure_ids(doc: &mut Document) {
    let mut used = HashSet::new();
    for board in doc.pages.iter_mut().flat_map(|p| &mut p.artboards) {
        if board.prototype_id.is_empty() || !used.insert(board.prototype_id.clone()) {
            board.prototype_id = uid();
            used.insert(board.prototype_id.clone());
        }
        for layer in &mut board.layers {
            if layer.prototype_id.is_empty() || !used.insert(layer.prototype_id.clone()) {
                layer.prototype_id = uid();
                used.insert(layer.prototype_id.clone());
            }
        }
    }
}
impl PrototypeUi {
    pub fn prototype(&mut self) {
        self.editing = !self.editing;
        self.prototype_menu = self.editing;
        self.sharing = false;
    }
    pub fn share(&mut self) {
        self.sharing = !self.sharing;
        self.editing = false;
        self.prototype_menu = false;
    }
    pub fn preview(&mut self) {
        self.preview_requested = true;
    }

    fn show_wires(
        &mut self,
        ctx: &egui::Context,
        canvas: Rect,
        doc: &mut Document,
        page: usize,
        active: usize,
        selected: Option<usize>,
        pan: Vec2,
        zoom: f32,
    ) {
        let boards = &doc.pages[page].artboards;
        let frame = &boards[active].frame;
        let origin = canvas.center() + pan + Vec2::new(frame.x, frame.y) * zoom
            - Vec2::new(frame.width, frame.height) * zoom / 2.0;
        let board_rects: Vec<_> = boards
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let pos = origin
                    + Vec2::new(
                        (i as f32 - active as f32) * 720.0 + b.frame.x - frame.x,
                        b.frame.y - frame.y,
                    ) * zoom;
                (
                    b.prototype_id.clone(),
                    Rect::from_min_size(pos, Vec2::new(b.frame.width, b.frame.height) * zoom),
                )
            })
            .collect();
        let painter = ctx
            .layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("prototype_wires"),
            ))
            .with_clip_rect(canvas);
        for link in &doc.prototype.interactions {
            let source = boards.iter().enumerate().find_map(|(i, b)| {
                if b.prototype_id == link.source {
                    Some(board_rects[i].1.right_center())
                } else {
                    b.layers
                        .iter()
                        .find(|l| l.prototype_id == link.source)
                        .map(|l| {
                            board_rects[i].1.min
                                + Vec2::new(l.x + l.width, l.y + l.height / 2.0) * zoom
                        })
                }
            });
            let target = board_rects.iter().find(|(id, _)| *id == link.target);
            if matches!(link.action, Action::Transition | Action::Overlay) {
                if let (Some(from), Some((_, to))) = (source, target) {
                    painter.arrow(
                        from,
                        to.left_center() - from,
                        egui::Stroke::new(1.5_f32, crate::ACCENT),
                    );
                }
            }
        }
        let board = &boards[active];
        let (source, start) = selected
            .and_then(|i| board.layers.get(i))
            .map(|l| {
                (
                    l.prototype_id.clone(),
                    origin + Vec2::new(l.x + l.width, l.y + l.height / 2.0) * zoom,
                )
            })
            .unwrap_or_else(|| {
                (
                    board.prototype_id.clone(),
                    board_rects[active].1.right_center(),
                )
            });
        if canvas.contains(start) {
            egui::Area::new(egui::Id::new("prototype_connection_handle"))
                .order(egui::Order::Foreground)
                .fixed_pos(start - Vec2::splat(9.0))
                .show(ctx, |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::drag());
                    ui.painter()
                        .circle_filled(rect.center(), 7.0, crate::ACCENT);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "+",
                        egui::FontId::proportional(13.0),
                        egui::Color32::WHITE,
                    );
                    if response.drag_started() {
                        self.wire_source = Some(source.clone());
                    }
                    response.on_hover_text("Drag to a destination frame to create a connection");
                });
        }
        if self.wire_source.is_some() {
            if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
                painter.arrow(
                    start,
                    pos - start,
                    egui::Stroke::new(2.0_f32, crate::ACCENT),
                );
                let target = board_rects.iter().rev().find(|(_, r)| r.contains(pos));
                if let Some((_, r)) = target {
                    painter.rect_stroke(
                        *r,
                        0.0,
                        egui::Stroke::new(2.0_f32, crate::ACCENT),
                        egui::StrokeKind::Outside,
                    );
                }
                if ctx.input(|i| i.pointer.any_released()) {
                    if let (Some(source), Some((target, _))) = (self.wire_source.take(), target) {
                        doc.prototype.interactions.push(Interaction {
                            source,
                            target: target.clone(),
                            ..Interaction::default()
                        });
                    }
                }
            }
            if ctx.input(|i| {
                i.key_pressed(egui::Key::Escape)
                    || (!i.pointer.primary_down() && !i.pointer.any_released())
            }) {
                self.wire_source = None;
            }
        }
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("prototype_wire_dragging"),
                self.wire_source.is_some(),
            )
        });
    }
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        canvas: Rect,
        doc: &mut Document,
        page: usize,
        board_index: usize,
        selected: Option<usize>,
        pan: Vec2,
        zoom: f32,
    ) -> Option<String> {
        let mut preview = std::mem::take(&mut self.preview_requested);
        let mut export = false;
        let mut prototype_anchor = Rect::NOTHING;
        let mut share_anchor = Rect::NOTHING;
        egui::Area::new(egui::Id::new("prototype_share_pill"))
            .order(egui::Order::Foreground)
            .pivot(egui::Align2::CENTER_TOP)
            .fixed_pos(canvas.center_top() + Vec2::new(0.0, 12.0))
            .show(ctx, |ui| {
                crate::island(ctx).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let prototype_button = ui
                            .selectable_label(self.editing, "Prototype")
                            .on_hover_text("Connect screens and define interactions");
                        prototype_anchor = prototype_button.rect;
                        if prototype_button.clicked() {
                            self.prototype_menu = !self.prototype_menu;
                            self.editing = self.prototype_menu;
                            self.sharing = false;
                        }
                        ui.separator();
                        preview = ui.button("▶ Preview").clicked();
                        ui.separator();
                        let share_button = ui.selectable_label(self.sharing, "Share");
                        share_anchor = share_button.rect;
                        if share_button.clicked() {
                            self.sharing = !self.sharing;
                            self.editing = false;
                            self.prototype_menu = false;
                        }
                    });
                });
            });
        if self.editing || self.sharing || preview {
            ensure_ids(doc);
            if doc.prototype.flows.is_empty() {
                doc.prototype.flows.push(Flow {
                    name: "Flow 1".into(),
                    home: doc.pages[page].artboards[board_index].prototype_id.clone(),
                });
            }
            self.flow = self.flow.min(doc.prototype.flows.len() - 1);
        }
        if !self.editing {
            self.wire_source = None;
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("prototype_wire_dragging"), false));
        }
        if self.editing {
            self.show_wires(ctx, canvas, doc, page, board_index, selected, pan, zoom);
        }
        let boards: Vec<(String, String)> = doc
            .pages
            .iter()
            .flat_map(|p| {
                p.artboards
                    .iter()
                    .map(move |b| (b.prototype_id.clone(), format!("{} / {}", p.name, b.name)))
            })
            .collect();
        if self.prototype_menu {
            let popup = egui::Window::new("Prototype")
                .title_bar(false)
                .movable(false)
                .resizable(false)
                .fixed_pos(crate::anchored_menu_position(
                    ctx,
                    prototype_anchor,
                    292.0,
                    520.0,
                ))
                .default_width(266.0)
                .show(ctx, |ui| {
                    ui.set_width(266.0);
                    crate::section_heading(ui, "Prototype");
                    ui.add_space(8.0);
                    flow_picker(ui, doc, &mut self.flow, &boards);
                    ui.horizontal(|ui| {
                        if ui.button("New flow").clicked() {
                            doc.prototype.flows.push(Flow {
                                name: format!("Flow {}", doc.prototype.flows.len() + 1),
                                home: doc.pages[page].artboards[board_index].prototype_id.clone(),
                            });
                            self.flow = doc.prototype.flows.len() - 1;
                        }
                        if ui.button("Set home here").clicked() {
                            doc.prototype.flows[self.flow].home =
                                doc.pages[page].artboards[board_index].prototype_id.clone();
                        }
                    });
                    ui.separator();
                    let board = &doc.pages[page].artboards[board_index];
                    let (source, label) = selected
                        .and_then(|i| board.layers.get(i))
                        .map(|l| {
                            (
                                l.prototype_id.clone(),
                                l.label.as_deref().unwrap_or(&l.name).to_owned(),
                            )
                        })
                        .unwrap_or_else(|| (board.prototype_id.clone(), board.name.clone()));
                    ui.label(egui::RichText::new(label).strong());
                    ui.label("Interactions");
                    let mut remove = None;
                    egui::ScrollArea::vertical()
                        .max_height(340.0)
                        .show(ui, |ui| {
                            for (index, interaction) in doc
                                .prototype
                                .interactions
                                .iter_mut()
                                .enumerate()
                                .filter(|(_, i)| i.source == source)
                            {
                                ui.push_id(index, |ui| {
                                    ui.horizontal(|ui| {
                                        egui::ComboBox::from_id_salt("trigger")
                                            .selected_text(format!("{:?}", interaction.trigger))
                                            .width(170.0)
                                            .show_ui(ui, |ui| {
                                                for t in [
                                                    Trigger::Tap,
                                                    Trigger::Hover,
                                                    Trigger::Drag,
                                                    Trigger::Key,
                                                    Trigger::Time,
                                                ] {
                                                    ui.selectable_value(
                                                        &mut interaction.trigger,
                                                        t,
                                                        format!("{t:?}"),
                                                    );
                                                }
                                            });
                                        if ui
                                            .small_button("−")
                                            .on_hover_text("Remove interaction")
                                            .clicked()
                                        {
                                            remove = Some(index);
                                        }
                                    });
                                    if interaction.trigger == Trigger::Key {
                                        ui.horizontal(|ui| {
                                            ui.label("Key");
                                            ui.text_edit_singleline(&mut interaction.key);
                                        });
                                    }
                                    if interaction.trigger == Trigger::Time {
                                        ui.horizontal(|ui| {
                                            ui.label("Delay");
                                            ui.add(
                                                egui::DragValue::new(&mut interaction.delay)
                                                    .range(0.1..=60.0)
                                                    .suffix(" s"),
                                            );
                                        });
                                    }
                                    egui::ComboBox::from_id_salt("action")
                                        .selected_text(action_name(interaction.action))
                                        .width(230.0)
                                        .show_ui(ui, |ui| {
                                            for a in [
                                                Action::Transition,
                                                Action::Overlay,
                                                Action::Previous,
                                                Action::CloseOverlay,
                                                Action::Url,
                                                Action::ScrollTo,
                                            ] {
                                                ui.selectable_value(
                                                    &mut interaction.action,
                                                    a,
                                                    action_name(a),
                                                );
                                            }
                                        });
                                    if matches!(
                                        interaction.action,
                                        Action::Transition | Action::Overlay
                                    ) {
                                        target_picker(
                                            ui,
                                            "target",
                                            &mut interaction.target,
                                            &boards,
                                        );
                                        egui::ComboBox::from_id_salt("animation")
                                            .selected_text(format!("{:?}", interaction.animation))
                                            .width(230.0)
                                            .show_ui(ui, |ui| {
                                                for a in [
                                                    Animation::None,
                                                    Animation::Dissolve,
                                                    Animation::SlideLeft,
                                                    Animation::SlideRight,
                                                    Animation::SlideUp,
                                                    Animation::SlideDown,
                                                ] {
                                                    ui.selectable_value(
                                                        &mut interaction.animation,
                                                        a,
                                                        format!("{a:?}"),
                                                    );
                                                }
                                            });
                                        ui.horizontal(|ui| {
                                            ui.label("Duration");
                                            ui.add(
                                                egui::DragValue::new(&mut interaction.duration)
                                                    .range(0.0..=5.0)
                                                    .suffix(" s"),
                                            );
                                        });
                                        if interaction.action == Action::Overlay {
                                            ui.horizontal(|ui| {
                                                ui.label("Overlay X/Y");
                                                ui.add(egui::DragValue::new(
                                                    &mut interaction.overlay_x,
                                                ));
                                                ui.add(egui::DragValue::new(
                                                    &mut interaction.overlay_y,
                                                ));
                                            });
                                        }
                                    }
                                    if interaction.action == Action::Url {
                                        ui.text_edit_singleline(&mut interaction.url);
                                        ui.label("HTTP(S) links only");
                                    }
                                    if interaction.action == Action::ScrollTo {
                                        ui.horizontal(|ui| {
                                            ui.label("Scroll Y");
                                            ui.add(
                                                egui::DragValue::new(&mut interaction.scroll_y)
                                                    .range(0.0..=8192.0),
                                            );
                                        });
                                    }
                                    ui.separator();
                                });
                            }
                        });
                    if let Some(index) = remove {
                        doc.prototype.interactions.remove(index);
                    }
                    if ui.button("+ Add interaction").clicked() {
                        let target = boards
                            .iter()
                            .find(|(id, _)| *id != board.prototype_id)
                            .or_else(|| boards.first())
                            .map(|(id, _)| id.clone())
                            .unwrap_or_default();
                        doc.prototype.interactions.push(Interaction {
                            source,
                            target,
                            ..Interaction::default()
                        });
                    }
                    ui.label(
                        egui::RichText::new("Select a layer or frame to edit its interactions.")
                            .small()
                            .weak(),
                    );
                    preview |= ui.button("▶ Preview flow").clicked();
                });
            if let Some(popup) = popup {
                if crate::dismiss_transient_menu(ctx, popup.response.rect, prototype_anchor) {
                    self.prototype_menu = false;
                }
            }
        }
        if self.sharing {
            let popup = egui::Window::new("Share prototype").title_bar(false).movable(false).resizable(false)
                .fixed_pos(crate::anchored_menu_position(ctx, share_anchor, 292.0, 440.0))
                .default_width(266.0).show(ctx, |ui| {
                    ui.set_width(266.0);
                    crate::section_heading(ui, "Share prototype");
                    ui.add_space(8.0);
                    flow_picker(ui, doc, &mut self.flow, &boards);
                    let previous = self.settings.preset;
                    egui::ComboBox::from_id_salt("share_preset").selected_text(preset_name(previous)).width(246.0).show_ui(ui, |ui| {
                        for p in [Preset::DesignReview, Preset::Development, Preset::Presentation, Preset::UserTesting, Preset::Custom] { ui.selectable_value(&mut self.settings.preset, p, preset_name(p)); }
                    });
                    if previous != self.settings.preset { self.settings = ShareSettings::preset(self.settings.preset); }
                    ui.separator();
                    if self.settings.preset == Preset::Custom {
                        ui.checkbox(&mut self.settings.navigation, "Navigation controls");
                        ui.checkbox(&mut self.settings.hints, "Hotspot hints");
                        ui.checkbox(&mut self.settings.fullscreen, "Full-screen option");
                        ui.checkbox(&mut self.settings.comments, "Local review notes");
                        ui.checkbox(&mut self.settings.specs, "Design specifications");
                    }
                    ui.label("Export a standalone HTML file. All screens and interactions are included; no internet connection is required.");
                    if self.settings.comments { ui.label(egui::RichText::new("Review notes stay in the viewer’s browser and can be downloaded. They are not synced between people.").small().weak()); }
                    export = ui.button("Export interactive HTML…").clicked();
                    preview |= ui.button("Preview in browser").clicked();
                });
            if let Some(popup) = popup {
                if crate::dismiss_transient_menu(ctx, popup.response.rect, share_anchor) {
                    self.sharing = false;
                }
            }
        }
        if preview || export {
            let path = if export {
                rfd::FileDialog::new()
                    .add_filter("Interactive prototype", &["html"])
                    .set_file_name("suiza-prototype.html")
                    .save_file()
            } else {
                Some(std::env::temp_dir().join(format!("suiza-preview-{}.html", uid())))
            };
            if let Some(path) = path {
                match export_html(doc, self.flow, &self.settings, &path) {
                    Ok(()) => {
                        if preview {
                            if let Err(e) = open_browser(&path) {
                                return Some(format!(
                                    "Preview saved, but browser could not open: {e}"
                                ));
                            }
                        }
                        return Some(format!(
                            "{} {}",
                            if preview {
                                "Opened preview:"
                            } else {
                                "Exported prototype:"
                            },
                            path.display()
                        ));
                    }
                    Err(error) => return Some(format!("Prototype export failed: {error}")),
                }
            }
        }
        None
    }
}
fn action_name(action: Action) -> &'static str {
    match action {
        Action::Transition => "Transition",
        Action::Overlay => "Overlay",
        Action::Previous => "Previous artboard",
        Action::CloseOverlay => "Close overlay",
        Action::Url => "Open URL",
        Action::ScrollTo => "Scroll to",
    }
}
fn preset_name(p: Preset) -> &'static str {
    match p {
        Preset::DesignReview => "Design Review",
        Preset::Development => "Development",
        Preset::Presentation => "Presentation",
        Preset::UserTesting => "User Testing",
        Preset::Custom => "Custom",
    }
}
fn target_picker(ui: &mut egui::Ui, id: &str, target: &mut String, boards: &[(String, String)]) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(
            boards
                .iter()
                .find(|(i, _)| i == target)
                .map(|(_, n)| n.as_str())
                .unwrap_or("Choose destination…"),
        )
        .width(246.0)
        .show_ui(ui, |ui| {
            for (id, name) in boards {
                ui.selectable_value(target, id.clone(), name);
            }
        });
}
fn flow_picker(
    ui: &mut egui::Ui,
    doc: &mut Document,
    flow: &mut usize,
    boards: &[(String, String)],
) {
    egui::ComboBox::from_id_salt("flow")
        .selected_text(&doc.prototype.flows[*flow].name)
        .width(246.0)
        .show_ui(ui, |ui| {
            for (index, f) in doc.prototype.flows.iter().enumerate() {
                ui.selectable_value(flow, index, &f.name);
            }
        });
    ui.text_edit_singleline(&mut doc.prototype.flows[*flow].name);
    ui.label("Home screen");
    target_picker(ui, "home", &mut doc.prototype.flows[*flow].home, boards);
}
fn open_browser(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let command = "open";
    #[cfg(target_os = "linux")]
    let command = "xdg-open";
    #[cfg(target_os = "windows")]
    let command = "explorer";
    std::process::Command::new(command)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
#[derive(Serialize)]
struct Screen<'a> {
    id: &'a str,
    name: &'a str,
    width: f32,
    height: f32,
    image: String,
    layers: Vec<Spec<'a>>,
}
#[derive(Serialize)]
struct Spec<'a> {
    id: &'a str,
    name: &'a str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: String,
    font: Option<&'a str>,
    font_size: Option<f32>,
}
#[derive(Serialize)]
struct Payload<'a> {
    title: &'a str,
    flow: &'a Flow,
    screens: Vec<Screen<'a>>,
    interactions: Vec<&'a Interaction>,
    settings: &'a ShareSettings,
}
/// Only screens reachable from the chosen home are included in an exported flow.
fn reachable(doc: &Document, home: &str) -> HashSet<String> {
    let boards: Vec<_> = doc.pages.iter().flat_map(|p| &p.artboards).collect();
    let mut set = HashSet::from([home.to_owned()]);
    loop {
        let old = set.len();
        for b in &boards {
            if !set.contains(&b.prototype_id) {
                continue;
            }
            for link in &doc.prototype.interactions {
                if (link.source == b.prototype_id
                    || b.layers
                        .iter()
                        .any(|l| l.visible && l.prototype_id == link.source))
                    && matches!(link.action, Action::Transition | Action::Overlay)
                    && boards.iter().any(|b| b.prototype_id == link.target)
                {
                    set.insert(link.target.clone());
                }
            }
        }
        if set.len() == old {
            return set;
        }
    }
}
pub fn export_html(
    doc: &Document,
    flow_index: usize,
    settings: &ShareSettings,
    path: &std::path::Path,
) -> Result<(), String> {
    let flow = doc
        .prototype
        .flows
        .get(flow_index)
        .ok_or("Create a prototype flow first")?;
    if !doc
        .pages
        .iter()
        .flat_map(|p| &p.artboards)
        .any(|b| b.prototype_id == flow.home)
    {
        return Err("The flow’s home screen was deleted. Choose another home screen.".into());
    }
    let ids = reachable(doc, &flow.home);
    let mut screens = Vec::new();
    for b in doc
        .pages
        .iter()
        .flat_map(|p| &p.artboards)
        .filter(|b| ids.contains(&b.prototype_id))
    {
        let image = screen_image(b)?;
        let layers = b
            .layers
            .iter()
            .filter(|l| l.visible && !l.mask)
            .map(|l| Spec {
                id: &l.prototype_id,
                name: l.label.as_deref().unwrap_or(&l.name),
                x: l.x,
                y: l.y,
                width: l.width,
                height: l.height,
                color: format!("#{:02X}{:02X}{:02X}", l.color[0], l.color[1], l.color[2]),
                font: l.typography.family.as_deref(),
                font_size: l.typography.size,
            })
            .collect();
        screens.push(Screen {
            id: &b.prototype_id,
            name: &b.name,
            width: b.frame.width,
            height: b.frame.height,
            image,
            layers,
        });
    }
    let sources: HashSet<_> = screens
        .iter()
        .flat_map(|s| std::iter::once(s.id).chain(s.layers.iter().map(|l| l.id)))
        .collect();
    let interactions = doc
        .prototype
        .interactions
        .iter()
        .filter(|i| sources.contains(i.source.as_str()))
        .collect();
    let payload = Payload {
        title: &doc.name,
        flow,
        screens,
        interactions,
        settings,
    };
    // Escape script-closing text and JS line separators, including user-authored names/URLs.
    let json = serde_json::to_string(&payload)
        .map_err(|e| e.to_string())?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    let html = include_str!("prototype_player.html").replace("/*SUIZA_DATA*/", &json);
    std::fs::write(path, html).map_err(|e| e.to_string())
}
fn screen_image(board: &Artboard) -> Result<String, String> {
    let temp = std::env::temp_dir().join(format!("suiza-prototype-{}.png", uid()));
    let result = crate::export_board(board, &temp, "PNG", 1)
        .and_then(|_| std::fs::read(&temp).map_err(|e| e.to_string()))
        .map(|png| format!("data:image/png;base64,{}", crate::encode_base64(&png)));
    let _ = std::fs::remove_file(temp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Document {
        let mut doc = Document::default();
        for board in doc.pages.iter_mut().flat_map(|p| &mut p.artboards) {
            board.layers.truncate(1);
        }
        ensure_ids(&mut doc);
        let first = &doc.pages[0].artboards[0];
        let home = first.prototype_id.clone();
        let source = first.layers[0].prototype_id.clone();
        let destination = doc.pages[0].artboards[1].prototype_id.clone();
        doc.prototype.flows.push(Flow {
            name: "Checkout".into(),
            home,
        });
        doc.prototype.interactions.push(Interaction {
            source,
            target: destination,
            ..Interaction::default()
        });
        doc
    }
    #[test]
    fn stable_connections_survive_rename_reorder_and_json() {
        let mut doc = fixture();
        let original = doc.prototype.clone();
        doc.pages[0].artboards[0].name = "Renamed screen".into();
        doc.pages[0].artboards[0].layers[0].label = Some("Renamed button".into());
        doc.pages[0].artboards.swap(0, 1);
        ensure_ids(&mut doc);
        assert!(doc.prototype == original);
        let imported = Document::from_json(&serde_json::to_string(&doc).unwrap()).unwrap();
        assert!(imported == doc);
        assert_eq!(reachable(&doc, &doc.prototype.flows[0].home).len(), 2);
        let copy = doc.pages[0].artboards[0].clone();
        doc.pages[0].artboards.push(copy);
        ensure_ids(&mut doc);
        let ids: Vec<_> = doc
            .pages
            .iter()
            .flat_map(|p| &p.artboards)
            .map(|b| &b.prototype_id)
            .collect();
        assert_eq!(ids.len(), ids.iter().collect::<HashSet<_>>().len());
    }
    #[test]
    fn html_export_is_offline_safe_and_contains_only_reachable_screens() {
        let mut doc = fixture();
        doc.name = "</script><script>alert('bad')</script>".into();
        let path = std::env::temp_dir().join("suiza-prototype-test.html");
        export_html(&doc, 0, &ShareSettings::default(), &path).unwrap();
        let html = std::fs::read_to_string(&path).unwrap();
        let json = html
            .split("const data=")
            .nth(1)
            .unwrap()
            .split(";\nconst $=")
            .next()
            .unwrap();
        let data: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(data["screens"].as_array().unwrap().len(), 2);
        assert_eq!(data["interactions"].as_array().unwrap().len(), 1);
        assert!(data["screens"][0]["image"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert!(!json.contains("</script>"));
        assert!(!html.contains("<script src="));
        doc.pages[0]
            .artboards
            .retain(|b| b.prototype_id != doc.prototype.flows[0].home);
        assert!(export_html(&doc, 0, &ShareSettings::default(), &path).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn share_presets_have_meaningful_viewer_controls() {
        let testing = ShareSettings::preset(Preset::UserTesting);
        assert!(
            !testing.navigation
                && !testing.hints
                && !testing.comments
                && !testing.specs
                && testing.fullscreen
        );
        let development = ShareSettings::preset(Preset::Development);
        assert!(development.specs && development.comments && development.navigation);
        let doc = fixture();
        export_html(
            &doc,
            0,
            &development,
            &std::env::temp_dir().join("suiza-prototype-qa.html"),
        )
        .unwrap();
    }
}
