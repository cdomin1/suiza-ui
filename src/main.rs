#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod plugins;

use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
mod assets;
mod document_file;
mod fonts;
mod prototype;
mod sketch_import;
mod text;
mod vector;
mod workbench;

const ACCENT: Color32 = Color32::from_rgb(45, 113, 225);

// Canvas presets are editable starting dimensions, not platform upload restrictions.
const CANVAS_PRESETS: [&[(&str, [f32; 2])]; 3] = [
    &[
        ("Phone · 390 × 844", [390.0, 844.0]),
        ("Phone compact · 375 × 812", [375.0, 812.0]),
        ("Phone large · 430 × 932", [430.0, 932.0]),
        ("Android · 360 × 800", [360.0, 800.0]),
        ("Tablet · 834 × 1194", [834.0, 1194.0]),
        ("Tablet landscape · 1194 × 834", [1194.0, 834.0]),
        ("Tablet large · 1024 × 1366", [1024.0, 1366.0]),
    ],
    &[
        ("Web HD · 1920 × 1080", [1920.0, 1080.0]),
        ("Web desktop · 1440 × 900", [1440.0, 900.0]),
        ("Web laptop · 1366 × 768", [1366.0, 768.0]),
        ("Web tablet · 768 × 1024", [768.0, 1024.0]),
        ("Web mobile · 390 × 844", [390.0, 844.0]),
        ("Presentation · 1280 × 720", [1280.0, 720.0]),
        ("4K · 3840 × 2160", [3840.0, 2160.0]),
    ],
    &[
        ("Square post · 1080 × 1080", [1080.0, 1080.0]),
        ("Portrait post · 1080 × 1350", [1080.0, 1350.0]),
        ("Tall post · 1080 × 1440", [1080.0, 1440.0]),
        ("Story / Reel · 1080 × 1920", [1080.0, 1920.0]),
        ("Landscape post · 1200 × 628", [1200.0, 628.0]),
        ("Pinterest Pin · 1000 × 1500", [1000.0, 1500.0]),
        ("Video thumbnail · 1280 × 720", [1280.0, 720.0]),
        ("Square ad · 1440 × 1440", [1440.0, 1440.0]),
        ("Portrait ad · 1440 × 1800", [1440.0, 1800.0]),
        ("Display rectangle · 300 × 250", [300.0, 250.0]),
        ("Large rectangle · 336 × 280", [336.0, 280.0]),
        ("Leaderboard · 728 × 90", [728.0, 90.0]),
        ("Large leaderboard · 970 × 90", [970.0, 90.0]),
        ("Half-page ad · 300 × 600", [300.0, 600.0]),
        ("Skyscraper · 160 × 600", [160.0, 600.0]),
        ("Mobile banner · 320 × 50", [320.0, 50.0]),
        ("Large mobile banner · 320 × 100", [320.0, 100.0]),
    ],
];

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Kind {
    Rectangle,
    Ellipse,
    Text,
    Image,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct ShadowStyle {
    enabled: bool,
    color: [u8; 3],
    opacity: f32,
    x: f32,
    y: f32,
    blur: f32,
    spread: f32,
}
impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            enabled: true,
            color: [0, 0, 0],
            opacity: 25.0,
            x: 0.0,
            y: 4.0,
            blur: 8.0,
            spread: 0.0,
        }
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
struct TypeStyle {
    size: Option<f32>,
    bold: bool,
    align: usize,
    tracking: f32,
    #[serde(default)]
    monospace: bool,
    #[serde(default)]
    family: Option<String>,
    line_height: Option<f32>,
    #[serde(default)]
    italic: bool,
    #[serde(default)]
    underline: bool,
    #[serde(default)]
    strike: bool,
    #[serde(default)]
    kerning: Option<bool>,
    #[serde(default)]
    ligatures: Option<bool>,
    #[serde(default)]
    baseline: f32,
    #[serde(default)]
    transformation: u8,
    #[serde(default)]
    list: u8,
    #[serde(default)]
    direction: u8,
    #[serde(default)]
    vertical: usize,
    #[serde(default)]
    small_caps: bool,
    #[serde(default)]
    tabular: bool,
    #[serde(default)]
    slashed_zero: bool,
    #[serde(default)]
    discretionary: bool,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct EffectStyle {
    #[serde(default)]
    zoom: bool,
    kind: String,
    amount: f32,
    enabled: bool,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Layer {
    #[serde(default)]
    prototype_id: String,
    name: String,
    #[serde(default)]
    label: Option<String>,
    kind: Kind,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [u8; 3],
    visible: bool,
    #[serde(default = "default_radius")]
    radius: f32,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    locked: bool,
    #[serde(default)]
    drop_shadow: Option<ShadowStyle>,
    #[serde(default)]
    inner_shadow: Option<ShadowStyle>,
    #[serde(default)]
    effects: Vec<EffectStyle>,
    #[serde(default)]
    typography: TypeStyle,
    #[serde(default)]
    mask: bool,
    #[serde(default)]
    image_data: Option<Vec<u8>>,
    #[serde(default)]
    corners: Option<[f32; 4]>,
    #[serde(default = "default_corner_style")]
    corner_style: String,
    #[serde(default)]
    path: Option<vector::Path>,
}

fn default_corner_style() -> String {
    "Rounded".into()
}

fn default_radius() -> f32 {
    8.0
}

impl Layer {
    fn new(kind: Kind, x: f32, y: f32) -> Self {
        Self {
            prototype_id: String::new(),
            name: match kind {
                Kind::Rectangle => "Rectangle",
                Kind::Ellipse => "Ellipse",
                Kind::Text => "Your next big idea",
                Kind::Image => "Image",
            }
            .into(),
            label: None,
            kind,
            x,
            y,
            width: 180.0,
            height: 80.0,
            color: [96, 83, 255],
            visible: true,
            radius: 8.0,
            group: None,
            locked: false,
            drop_shadow: None,
            inner_shadow: None,
            effects: vec![],
            typography: TypeStyle::default(),
            mask: false,
            image_data: None,
            corners: None,
            corner_style: default_corner_style(),
            path: None,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct FrameStyle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
    background: [u8; 3],
    clip: bool,
}
impl Default for FrameStyle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 640.0,
            height: 440.0,
            radius: 0.0,
            background: [255, 255, 255],
            clip: false,
        }
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Artboard {
    #[serde(default)]
    prototype_id: String,
    name: String,
    #[serde(default)]
    frame: FrameStyle,
    layers: Vec<Layer>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Page {
    name: String,
    artboards: Vec<Artboard>,
}
#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Document {
    #[serde(default)]
    checkpoints: Vec<workbench::Checkpoint>,
    #[serde(default)]
    prototype: prototype::Prototype,
    name: String,
    pages: Vec<Page>,
}
impl Document {
    fn from_json(text: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let doc: Self = if value.get("layers").is_some() {
            #[derive(Deserialize)]
            struct Legacy {
                layers: Vec<Layer>,
            }
            let old: Legacy = serde_json::from_value(value).map_err(|e| e.to_string())?;
            Self {
                prototype: prototype::Prototype::default(),
                checkpoints: vec![],
                name: "Imported document".into(),
                pages: vec![Page {
                    name: "Page 1".into(),
                    artboards: vec![Artboard {
                        prototype_id: String::new(),
                        frame: FrameStyle::default(),
                        name: "Desktop".into(),
                        layers: old.layers,
                    }],
                }],
            }
        } else {
            serde_json::from_value(value).map_err(|e| e.to_string())?
        };
        doc.validate()?;
        Ok(doc)
    }
    fn validate(&self) -> Result<(), String> {
        workbench::validate_history(&self.checkpoints)?;
        let doc = self;
        if doc.pages.iter().flat_map(|p| &p.artboards).any(|b| {
            let f = &b.frame;
            ![f.x, f.y, f.width, f.height, f.radius]
                .iter()
                .all(|v| v.is_finite())
                || f.width < 1.0
                || f.height < 1.0
                || f.width > 8192.0
                || f.height > 8192.0
                || f.radius < 0.0
                || f.radius > 200.0
        }) {
            return Err("Invalid frame geometry".into());
        }
        if doc.pages.is_empty() || doc.pages.iter().any(|p| p.artboards.is_empty()) {
            return Err("Documents must contain a page and artboard".into());
        }
        if !doc
            .pages
            .iter()
            .flat_map(|p| &p.artboards)
            .flat_map(|a| &a.layers)
            .all(|l| {
                [l.x, l.y, l.width, l.height, l.radius]
                    .iter()
                    .all(|v| v.is_finite())
                    && l.width > 0.0
                    && l.height > 0.0
                    && l.radius >= 0.0
                    && l.path.as_ref().is_none_or(vector::Path::valid)
                    && l.corners
                        .is_none_or(|c| c.iter().all(|r| r.is_finite() && *r >= 0.0 && *r <= 200.0))
                    && [
                        "Auto",
                        "Rounded",
                        "Smooth",
                        "Angled",
                        "Inside Square",
                        "Inside Arc",
                    ]
                    .contains(&l.corner_style.as_str())
                    && (l.kind != Kind::Image
                        || l.image_data.as_ref().is_some_and(|data| {
                            data.len() <= 32_000_000
                                && image::ImageReader::new(std::io::Cursor::new(data))
                                    .with_guessed_format()
                                    .ok()
                                    .and_then(|r| r.into_dimensions().ok())
                                    .is_some_and(|(w, h)| u64::from(w) * u64::from(h) <= 16_000_000)
                        }))
                    && l.typography
                        .size
                        .is_none_or(|v| v.is_finite() && v > 0.0 && v <= 240.0)
                    && l.typography
                        .line_height
                        .is_none_or(|v| v.is_finite() && v > 0.0 && v <= 400.0)
                    && l.typography.tracking.is_finite()
                    && l.typography.baseline.is_finite()
                    && l.typography.baseline.abs() <= 200.0
                    && l.typography.align <= 3
                    && l.typography.vertical <= 2
                    && l.typography.transformation <= 3
                    && l.typography.list <= 2
                    && l.typography.direction <= 2
                    && l.effects
                        .iter()
                        .all(|e| e.amount.is_finite() && (0.0..=100.0).contains(&e.amount))
                    && [&l.drop_shadow, &l.inner_shadow]
                        .into_iter()
                        .flatten()
                        .all(|s| {
                            [s.x, s.y, s.blur, s.spread, s.opacity]
                                .iter()
                                .all(|v| v.is_finite())
                                && s.x.abs() <= 100.0
                                && s.y.abs() <= 100.0
                                && (0.0..=64.0).contains(&s.blur)
                                && (0.0..=32.0).contains(&s.spread)
                                && (0.0..=100.0).contains(&s.opacity)
                        })
            })
        {
            return Err("Invalid layer geometry".into());
        }
        Ok(())
    }
}
impl Default for Document {
    fn default() -> Self {
        let mut layers = Vec::new();
        let mut add = |kind, name: &str, x, y, width, height, color, radius| {
            layers.push(Layer {
                prototype_id: String::new(),
                name: name.into(),
                label: None,
                kind,
                x,
                y,
                width,
                height,
                color,
                visible: true,
                radius,
                group: None,
                locked: false,
                drop_shadow: None,
                inner_shadow: None,
                effects: vec![],
                typography: TypeStyle::default(),
                mask: false,
                image_data: None,
                corners: None,
                corner_style: default_corner_style(),
                path: None,
            });
        };
        add(
            Kind::Rectangle,
            "Warm paper",
            0.0,
            0.0,
            640.0,
            440.0,
            [249, 248, 244],
            0.0,
        );
        add(
            Kind::Text,
            "atelier",
            32.0,
            24.0,
            130.0,
            32.0,
            [38, 45, 42],
            0.0,
        );
        add(
            Kind::Text,
            "Spaces     Objects     Our story",
            332.0,
            32.0,
            280.0,
            20.0,
            [100, 108, 100],
            0.0,
        );
        add(
            Kind::Rectangle,
            "Editorial panel",
            32.0,
            88.0,
            576.0,
            310.0,
            [233, 237, 225],
            16.0,
        );
        add(
            Kind::Text,
            "LESS, BUT BETTER",
            56.0,
            112.0,
            240.0,
            17.0,
            [99, 115, 92],
            0.0,
        );
        add(
            Kind::Text,
            "Room to",
            56.0,
            153.0,
            310.0,
            68.0,
            [42, 58, 45],
            0.0,
        );
        add(
            Kind::Text,
            "be yourself.",
            56.0,
            208.0,
            310.0,
            68.0,
            [42, 58, 45],
            0.0,
        );
        add(
            Kind::Text,
            "Thoughtful objects for everyday living.",
            56.0,
            282.0,
            304.0,
            20.0,
            [105, 119, 99],
            0.0,
        );
        add(
            Kind::Rectangle,
            "Explore button",
            56.0,
            329.0,
            156.0,
            42.0,
            [43, 62, 47],
            21.0,
        );
        add(
            Kind::Text,
            "Explore collection",
            73.0,
            341.0,
            135.0,
            18.0,
            [249, 248, 244],
            0.0,
        );
        add(
            Kind::Ellipse,
            "Sun disc",
            389.0,
            124.0,
            171.0,
            171.0,
            [214, 222, 199],
            0.0,
        );
        add(
            Kind::Rectangle,
            "Pedestal shadow",
            366.0,
            348.0,
            216.0,
            8.0,
            [205, 214, 190],
            4.0,
        );
        add(
            Kind::Rectangle,
            "Pedestal",
            382.0,
            304.0,
            184.0,
            45.0,
            [224, 228, 211],
            3.0,
        );
        add(
            Kind::Ellipse,
            "Vessel body",
            427.0,
            205.0,
            96.0,
            110.0,
            [172, 113, 82],
            0.0,
        );
        add(
            Kind::Rectangle,
            "Vessel neck",
            452.0,
            177.0,
            46.0,
            76.0,
            [172, 113, 82],
            12.0,
        );
        add(
            Kind::Ellipse,
            "Vessel lip",
            451.0,
            174.0,
            48.0,
            14.0,
            [192, 140, 105],
            0.0,
        );
        add(
            Kind::Ellipse,
            "Vessel opening",
            458.0,
            178.0,
            34.0,
            5.0,
            [103, 71, 55],
            0.0,
        );
        add(
            Kind::Text,
            "DESIGNED WITH INTENTION",
            32.0,
            414.0,
            280.0,
            14.0,
            [135, 140, 129],
            0.0,
        );
        add(
            Kind::Text,
            "01 / 03",
            553.0,
            411.0,
            60.0,
            18.0,
            [105, 119, 99],
            0.0,
        );
        for (index, layer) in layers.iter_mut().enumerate() {
            layer.group = match index {
                1 | 2 => Some("Navigation".into()),
                3..=16 => Some("Hero".into()),
                17 | 18 => Some("Footer".into()),
                _ => None,
            };
        }
        let mut alternate = layers.clone();
        alternate[3].color = [234, 228, 239];
        alternate[10].color = [218, 207, 225];
        alternate[5].name = "Made for".into();
        alternate[6].name = "slow living.".into();
        Self {
            prototype: prototype::Prototype::default(),
            checkpoints: vec![],
            name: "Atelier".into(),
            pages: vec![
                Page {
                    name: "Designs".into(),
                    artboards: vec![
                        Artboard {
                            prototype_id: String::new(),
                            frame: FrameStyle::default(),
                            name: "Landing page".into(),
                            layers,
                        },
                        Artboard {
                            prototype_id: String::new(),
                            frame: FrameStyle::default(),
                            name: "Alternate direction".into(),
                            layers: alternate,
                        },
                    ],
                },
                Page {
                    name: "Explorations".into(),
                    artboards: vec![Artboard {
                        prototype_id: String::new(),
                        frame: FrameStyle::default(),
                        name: "Blank canvas".into(),
                        layers: vec![],
                    }],
                },
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
enum ThemeMode {
    Light,
    HighContrastLight,
    Dark,
    #[default]
    System,
}
#[derive(Clone)]
struct DocumentTab {
    doc: Document,
    saved: Document,
    path: Option<std::path::PathBuf>,
    page: usize,
    board: usize,
    selected: Option<usize>,
    selected_group: Option<String>,
    selection: Vec<usize>,
    zoom: f32,
    pan: Vec2,
    history: Vec<Document>,
    redo: Vec<Document>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSession {
    tabs: Vec<SavedSessionTab>,
    active: usize,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSessionTab {
    doc: Document,
    saved: Document,
    path: Option<std::path::PathBuf>,
    page: usize,
    board: usize,
}
struct Studio {
    doc: Document,
    page: usize,
    artboard: usize,
    selected: Option<usize>,
    selected_group: Option<String>,
    layer_clipboard: Option<Layer>,
    clipboard_group: Vec<Layer>,
    board_clipboard: Option<Artboard>,
    prototype_ui: prototype::PrototypeUi,
    workbench_ui: workbench::Workbench,
    plugins_ui: plugins::Manager,
    font_chooser: bool,
    selection: Vec<usize>,
    group_drag: Vec<(usize, Layer)>,
    editing_text: Option<usize>,
    editing_path: Option<usize>,
    pen: bool,
    path_node: Option<usize>,
    path_drag: Option<usize>,
    tabs: Vec<DocumentTab>,
    active_tab: usize,
    closing_tab: Option<usize>,
    document_path: Option<std::path::PathBuf>,
    saved_document: Document,
    theme: ThemeMode,
    applied_dark: Option<bool>,
    tool: Option<Kind>,
    zoom: f32,
    status: String,
    history: Vec<Document>,
    redo: Vec<Document>,
    drag_origin: Option<Layer>,
    resize_corner: Option<usize>,
    marquee_start: Option<Pos2>,
    marquee_base: Vec<usize>,
    pan: Vec2,
    grid: bool,
    snap: bool,
    search: String,
    rulers: bool,
    canvas_bounds: Rect,
    help: bool,
    export_format: String,
    export_scale: u32,
    export_trim: bool,
    welcome: bool,
    recent: Vec<std::path::PathBuf>,
    welcome_search: String,
    assets_ui: assets::AssetsUi,
    swatches: Vec<[u8; 3]>,
    #[cfg(target_os = "macos")]
    native_menu: Option<NativeMenu>,
}
impl Default for Studio {
    fn default() -> Self {
        Self {
            doc: Document::default(),
            page: 0,
            artboard: 0,
            selected: Some(8),
            selected_group: None,
            layer_clipboard: None,
            clipboard_group: vec![],
            board_clipboard: None,
            prototype_ui: prototype::PrototypeUi::default(),
            workbench_ui: workbench::Workbench::default(),
            plugins_ui: plugins::Manager::default(),
            font_chooser: false,
            selection: vec![],
            group_drag: vec![],
            editing_text: None,
            editing_path: None,
            pen: false,
            path_node: None,
            path_drag: None,
            tabs: vec![],
            active_tab: 0,
            closing_tab: None,
            document_path: None,
            saved_document: Document::default(),
            theme: ThemeMode::System,
            applied_dark: None,
            tool: None,
            zoom: 1.0,
            status: "Ready to create".into(),
            history: vec![],
            redo: vec![],
            drag_origin: None,
            resize_corner: None,
            marquee_start: None,
            marquee_base: vec![],
            pan: Vec2::ZERO,
            grid: false,
            snap: true,
            search: String::new(),
            rulers: false,
            canvas_bounds: Rect::NOTHING,
            help: false,
            export_format: "PNG".into(),
            export_scale: 1,
            export_trim: false,
            welcome: false,
            recent: vec![],
            welcome_search: String::new(),
            assets_ui: Default::default(),
            swatches: vec![
                [96, 83, 255],
                [118, 202, 151],
                [255, 255, 255],
                [43, 62, 47],
                [179, 109, 74],
            ],
            #[cfg(target_os = "macos")]
            native_menu: None,
        }
    }
}
impl Studio {
    fn tab_snapshot(&self) -> DocumentTab {
        DocumentTab {
            doc: self.doc.clone(),
            saved: self.saved_document.clone(),
            path: self.document_path.clone(),
            page: self.page,
            board: self.artboard,
            selected: self.selected,
            selected_group: self.selected_group.clone(),
            selection: self.selection.clone(),
            zoom: self.zoom,
            pan: self.pan,
            history: self.history.clone(),
            redo: self.redo.clone(),
        }
    }
    fn ensure_tabs(&mut self) {
        if self.tabs.is_empty() {
            self.tabs.push(self.tab_snapshot());
        }
    }
    fn switch_tab(&mut self, index: usize) {
        self.ensure_tabs();
        if index >= self.tabs.len() || index == self.active_tab {
            return;
        }
        self.tabs[self.active_tab] = self.tab_snapshot();
        let tab = self.tabs[index].clone();
        self.active_tab = index;
        self.doc = tab.doc;
        self.saved_document = tab.saved;
        self.document_path = tab.path;
        self.page = tab.page;
        self.artboard = tab.board;
        self.selected = tab.selected;
        self.selected_group = tab.selected_group;
        self.selection = tab.selection;
        self.zoom = tab.zoom;
        self.pan = tab.pan;
        self.history = tab.history;
        self.redo = tab.redo;
        self.editing_text = None;
        self.editing_path = None;
        self.pen = false;
        self.path_node = None;
        self.drag_origin = None;
        self.group_drag.clear();
        self.marquee_start = None;
        self.marquee_base.clear();
    }
    fn open_in_tab(&mut self, doc: Document, path: Option<std::path::PathBuf>) {
        self.ensure_tabs();
        self.tabs[self.active_tab] = self.tab_snapshot();
        self.doc = doc;
        self.saved_document = self.doc.clone();
        self.document_path = path;
        self.page = 0;
        self.artboard = 0;
        self.reset_selection();
        self.history.clear();
        self.redo.clear();
        self.active_tab = self.tabs.len();
        self.tabs.push(self.tab_snapshot());
    }
    fn close_tab(&mut self, index: usize) {
        self.ensure_tabs();
        self.switch_tab(index);
        if self.tabs.len() == 1 {
            self.apply_action(Action::New, &egui::Context::default());
            self.tabs.remove(0);
            self.active_tab = 0;
        } else {
            let next = if index > 0 { index - 1 } else { 1 };
            self.switch_tab(next);
            self.tabs.remove(index);
            if self.active_tab > index {
                self.active_tab -= 1;
            }
        }
        self.closing_tab = None;
    }
    fn show_tabs(&mut self, ctx: &egui::Context) {
        self.ensure_tabs();
        self.tabs[self.active_tab] = self.tab_snapshot();
        let mut switch = None;
        let mut close = None;
        let mut new = false;
        egui::TopBottomPanel::top("document_tabs")
            .exact_height(34.0)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(8, 3)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SUIZA Canvas").strong().color(ACCENT));
                        ui.separator();
                        if ui.button("Workbench").clicked() {
                            self.workbench_ui.open = true;
                        }
                        if ui
                            .small_button("Home")
                            .on_hover_text("Recent documents and templates")
                            .clicked()
                        {
                            self.welcome = true;
                        }
                        for (i, tab) in self.tabs.iter().enumerate() {
                            egui::Frame::new()
                                .fill(if i == self.active_tab {
                                    ui.visuals().widgets.inactive.bg_fill
                                } else {
                                    Color32::TRANSPARENT
                                })
                                .corner_radius(5)
                                .inner_margin(egui::Margin::symmetric(7, 0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let title = format!(
                                            "{}{}",
                                            tab.doc.name,
                                            if tab.doc != tab.saved { " •" } else { "" }
                                        );
                                        if ui
                                            .add_sized(
                                                [120.0, 26.0],
                                                egui::Button::new(
                                                    egui::RichText::new(title).size(12.0),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                        {
                                            switch = Some(i);
                                        }
                                        if close_icon_button(ui, 24.0)
                                            .on_hover_text("Close document")
                                            .clicked()
                                        {
                                            close = Some(i);
                                        }
                                    });
                                });
                        }
                        new = ui
                            .add_sized([28.0, 26.0], egui::Button::new("+").frame(false))
                            .on_hover_text("New document")
                            .clicked();
                    });
                });
            });
        if let Some(i) = switch {
            self.switch_tab(i);
        }
        if let Some(i) = close {
            self.switch_tab(i);
            if self.doc != self.saved_document {
                self.closing_tab = Some(i);
            } else {
                self.close_tab(i);
            }
        }
        if new {
            self.apply_action(Action::New, ctx);
        }
        if let Some(i) = self.closing_tab {
            egui::Window::new("Unsaved changes")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .default_width(360.0)
                .frame(egui::Frame::window(&ctx.style()).inner_margin(24))
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(12.0, 12.0);
                    ui.label(egui::RichText::new("Unsaved changes").size(18.0).strong());
                    ui.label("Save this document before closing?");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([72.0, 32.0], egui::Button::new("Save"))
                            .clicked()
                        {
                            self.save();
                            if self.doc == self.saved_document {
                                self.close_tab(i);
                            }
                        }
                        if ui
                            .add_sized([136.0, 32.0], egui::Button::new("Discard changes"))
                            .clicked()
                        {
                            self.close_tab(i);
                        }
                        if ui
                            .add_sized([80.0, 32.0], egui::Button::new("Cancel"))
                            .clicked()
                        {
                            self.closing_tab = None;
                        }
                    });
                });
        }
    }
    fn apply_theme(&mut self, ctx: &egui::Context) {
        let dark = match self.theme {
            ThemeMode::Light | ThemeMode::HighContrastLight => false,
            ThemeMode::Dark => true,
            ThemeMode::System => ctx.system_theme() == Some(egui::Theme::Dark),
        };
        if self.applied_dark == Some(dark) {
            return;
        }
        let high_contrast = self.theme == ThemeMode::HighContrastLight;
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("high_contrast_ui"), high_contrast));
        let previous_style = (*ctx.style()).clone();
        ctx.set_theme(match self.theme {
            ThemeMode::System => egui::ThemePreference::System,
            ThemeMode::Light | ThemeMode::HighContrastLight => egui::ThemePreference::Light,
            ThemeMode::Dark => egui::ThemePreference::Dark,
        });
        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = if dark {
            Color32::from_rgb(39, 40, 43)
        } else {
            Color32::WHITE
        };
        visuals.override_text_color = Some(if dark {
            Color32::from_gray(226)
        } else {
            Color32::from_rgb(58, 59, 72)
        });
        visuals.widgets.inactive.bg_fill = if dark {
            Color32::from_rgb(60, 63, 70)
        } else {
            Color32::from_rgb(239, 239, 244)
        };
        visuals.widgets.inactive.weak_bg_fill = visuals.widgets.inactive.bg_fill;
        visuals.extreme_bg_color = if dark {
            Color32::from_rgb(47, 48, 55)
        } else {
            Color32::from_rgb(247, 247, 250)
        };
        visuals.widgets.noninteractive.bg_stroke =
            Stroke::new(1.0_f32, Color32::from_gray(if dark { 58 } else { 232 }));
        visuals.widgets.noninteractive.fg_stroke.color = if dark {
            Color32::from_gray(185)
        } else {
            Color32::from_gray(120)
        };
        visuals.widgets.inactive.fg_stroke.color = if dark {
            Color32::from_gray(225)
        } else {
            Color32::from_rgb(58, 59, 72)
        };
        visuals.widgets.hovered.fg_stroke.color = visuals.widgets.inactive.fg_stroke.color;
        visuals.widgets.active.fg_stroke.color = visuals.widgets.inactive.fg_stroke.color;
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(4);
        visuals.widgets.inactive.bg_stroke = if dark {
            Stroke::new(1.0_f32, Color32::from_rgb(135, 141, 153))
        } else {
            Stroke::new(1.0_f32, Color32::from_rgb(140, 147, 160))
        };
        visuals.widgets.hovered.bg_fill = if dark {
            Color32::from_rgb(75, 80, 90)
        } else {
            Color32::from_rgb(226, 233, 243)
        };
        visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, ACCENT);
        visuals.selection.bg_fill = if dark {
            Color32::from_rgb(35, 64, 104)
        } else {
            Color32::from_rgb(222, 236, 255)
        };
        visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
        if high_contrast {
            visuals.override_text_color = Some(Color32::BLACK);
            visuals.panel_fill = Color32::WHITE;
            visuals.extreme_bg_color = Color32::from_gray(245);
            visuals.window_fill = Color32::WHITE;
            visuals.window_stroke = Stroke::new(2.0_f32, Color32::BLACK);
            for widget in [
                &mut visuals.widgets.inactive,
                &mut visuals.widgets.hovered,
                &mut visuals.widgets.active,
                &mut visuals.widgets.open,
                &mut visuals.widgets.noninteractive,
            ] {
                widget.fg_stroke = Stroke::new(1.5_f32, Color32::BLACK);
                widget.bg_stroke = Stroke::new(1.5_f32, Color32::from_gray(65));
            }
            visuals.widgets.inactive.bg_fill = Color32::WHITE;
            visuals.widgets.inactive.weak_bg_fill = Color32::WHITE;
            visuals.selection.bg_fill = Color32::from_rgb(210, 231, 255);
            visuals.selection.stroke = Stroke::new(2.0_f32, Color32::from_rgb(0, 70, 160));
        }
        let mut style = previous_style;
        style.visuals = visuals;
        ctx.set_style(style);
        self.applied_dark = Some(dark);
    }
    fn remember_document(&mut self, path: std::path::PathBuf) {
        self.recent.retain(|p| p != &path);
        self.recent.insert(0, path);
        self.recent.truncate(12);
    }
    fn show_tool_rail(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("form_tool_rail")
            .exact_width(48.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.extreme_bg_color)
                    .inner_margin(8),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 12.0);
                ui.vertical_centered(|ui| {
                    for (tool, icon, tip) in [
                        (None, 0, "Select · V"),
                        (Some(Kind::Rectangle), 1, "Rectangle · R"),
                        (Some(Kind::Ellipse), 2, "Ellipse · O"),
                        (Some(Kind::Text), 3, "Text · T"),
                    ] {
                        if tool_button(
                            ui,
                            self.tool == tool && !self.pen && self.editing_path.is_none(),
                            icon,
                            tip,
                        )
                        .clicked()
                        {
                            self.apply_action(Action::Tool(tool), ctx);
                        }
                    }
                    if tool_button(
                        ui,
                        self.pen || self.editing_path.is_some(),
                        14,
                        "Bézier pen · P",
                    )
                    .clicked()
                    {
                        self.apply_action(Action::Pen, ctx);
                    }
                    ui.separator();
                    if tool_button(ui, false, 6, &format!("Undo · {} Z", shortcut_name())).clicked()
                    {
                        self.undo();
                    }
                    if tool_button(ui, self.grid, 7, "Toggle grid").clicked() {
                        self.grid = !self.grid;
                    }
                    if tool_button(ui, self.rulers, 8, "Toggle rulers").clicked() {
                        self.rulers = !self.rulers;
                    }
                });
            });
    }
    fn show_floating_tools(&mut self, ctx: &egui::Context) {
        egui::Area::new(egui::Id::new("canvas_zoom"))
            .order(egui::Order::Middle)
            .pivot(egui::Align2::RIGHT_TOP)
            .fixed_pos(self.canvas_bounds.right_top() + Vec2::new(-16.0, 12.0))
            .show(ctx, |ui| {
                island(ctx).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.toggle_value(&mut self.snap, "Snap");
                        ui.menu_button(format!("{:.0}%", self.zoom * 100.0), |ui| {
                            for z in [0.5, 0.75, 1.0, 1.5, 2.0] {
                                if ui.button(format!("{:.0}%", z * 100.0)).clicked() {
                                    self.zoom = z;
                                    ui.close_menu();
                                }
                            }
                            if ui.button("Fit artboard").clicked() {
                                self.apply_action(Action::Fit, ctx);
                                ui.close_menu();
                            }
                        });
                    });
                });
            });
    }
    fn show_welcome(&mut self, ctx: &egui::Context) {
        let screen = ctx.screen_rect();
        let width = (screen.width() - 64.0).clamp(540.0, 1080.0);
        let height = (screen.height() - 90.0).clamp(360.0, 700.0);
        let mut section = ctx
            .data(|d| d.get_temp::<usize>(egui::Id::new("welcome_section")))
            .unwrap_or(0);
        let mut custom = ctx
            .data(|d| d.get_temp::<[f32; 2]>(egui::Id::new("welcome_size")))
            .unwrap_or([600.0, 500.0]);
        let mut preset_choices = ctx
            .data(|d| d.get_temp::<[usize; 3]>(egui::Id::new("welcome_presets")))
            .unwrap_or([0; 3]);
        let mut chosen = None;
        let mut new_size = None;
        let mut template = false;
        let mut open_file = false;
        egui::Window::new("SUIZA Canvas Home")
            .title_bar(false).collapsible(false).resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(egui::Frame::window(&ctx.style()).fill(ctx.style().visuals.panel_fill).corner_radius(16).inner_margin(0))
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                egui::Frame::new().inner_margin(20).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SUIZA Canvas").size(20.0).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if close_icon_button(ui, 28.0).on_hover_text("Return to editor").clicked() { self.welcome = false; }
                        });
                    });
                });
                ui.horizontal_top(|ui| {
                    egui::Frame::new().fill(ui.visuals().extreme_bg_color).inner_margin(20).show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.set_width(130.0);
                        ui.set_min_height(height - 105.0);
                        ui.spacing_mut().item_spacing = Vec2::new(8.0, 12.0);
                        if ui.add_sized([130.0, 30.0], egui::Button::new(egui::RichText::new("New file").color(Color32::WHITE)).fill(ACCENT).stroke(Stroke::NONE)).clicked() { new_size = Some([640.0, 440.0]); }
                        if ui.add_sized([130.0, 30.0], egui::Button::new("Open…").frame(false)).clicked() { open_file = true; }
                        ui.add_space(14.0);
                        for (i, label) in ["Home", "Your files", "Templates"].iter().enumerate() {
                            if ui.add_sized([130.0, 30.0], egui::Button::new(egui::RichText::new(*label).color(if section == i { Color32::WHITE } else { ui.visuals().text_color() })).fill(if section == i { ACCENT } else { ui.visuals().widgets.inactive.bg_fill }).stroke(Stroke::NONE)).clicked() { section = i; }
                        }
                        ui.add_space(24.0);
                        ui.label(egui::RichText::new("LOCAL WORKSPACE").size(9.0).color(secondary_text_color(ui)));
                        ui.label(egui::RichText::new("Your files stay on your device.").size(11.0).color(secondary_text_color(ui)));
                        });
                    });
                    egui::Frame::new().inner_margin(24).show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        let content_width = width - 218.0;
                        ui.set_width(content_width);
                        ui.spacing_mut().item_spacing = Vec2::new(16.0, 16.0);
                        egui::ScrollArea::vertical().max_height(height - 115.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(match section { 1 => "Your files", 2 => "Templates", _ => "Give your ideas form." }).size(18.0).strong());
                            });
                            if section == 0 {
                                let card_width = (content_width - 48.0) / 4.0;
                                egui::Frame::new().fill(ui.visuals().extreme_bg_color).corner_radius(8).inner_margin(12).show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;
                                    ui.horizontal_top(|ui| {
                                        for (i, (label, size)) in [("Devices", CANVAS_PRESETS[0][preset_choices[0]].1), ("Web", CANVAS_PRESETS[1][preset_choices[1]].1), ("Social & ads", CANVAS_PRESETS[2][preset_choices[2]].1), ("Custom size", custom)].into_iter().enumerate() {
                                            ui.allocate_ui(Vec2::new(card_width, 140.0), |ui| {
                                                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                                    let (rect, response) = ui.allocate_exact_size(Vec2::new(card_width, 65.0), Sense::click());
                                                    let scale = (48.0 / size[0]).min(43.0 / size[1]);
                                                    let icon_size = Vec2::new(size[0] * scale, size[1] * scale);
                                                    ui.painter().rect_stroke(Rect::from_center_size(rect.center(), icon_size), 3.0, Stroke::new(1.2_f32, secondary_text_color(ui)), egui::StrokeKind::Inside);
                                                    if response.on_hover_text("Create a document at this size").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() { new_size = Some(size); }
                                                    if i < 3 {
                                                        egui::ComboBox::from_id_salt(("canvas_preset", i))
                                                            .wrap_mode(egui::TextWrapMode::Truncate)
                                                            .width(card_width - 8.0)
                                                            .selected_text(label)
                                                            .show_ui(ui, |ui| {
                                                                for (index, (name, _)) in CANVAS_PRESETS[i].iter().enumerate() {
                                                                    ui.selectable_value(&mut preset_choices[i], index, *name);
                                                                }
                                                            });
                                                        if ui.add(egui::Button::new(CANVAS_PRESETS[i][preset_choices[i]].0.split(" · ").next().unwrap_or(label)).frame(false)).on_hover_text("Create a document at this size").clicked() { new_size = Some(CANVAS_PRESETS[i][preset_choices[i]].1); }
                                                    } else if ui.add(egui::Button::new(egui::RichText::new(label).strong()).frame(false)).clicked() { new_size = Some(size); }
                                                    if i == 3 {
                                                        ui.horizontal(|ui| {
                                                            ui.add_sized([card_width / 2.0 - 10.0, 24.0], egui::DragValue::new(&mut custom[0]).range(1.0..=4000.0).prefix("W "));
                                                            ui.add_sized([card_width / 2.0 - 10.0, 24.0], egui::DragValue::new(&mut custom[1]).range(1.0..=4000.0).prefix("H "));
                                                        });
                                                    } else { ui.label(egui::RichText::new(format!("{} × {} px", CANVAS_PRESETS[i][preset_choices[i]].1[0], CANVAS_PRESETS[i][preset_choices[i]].1[1])).size(10.0).color(secondary_text_color(ui))); }
                                                });
                                            });
                                        }
                                    });
                                });
                            }
                            if section == 2 {
                                ui.label("Start with an editable design.");
                                let doc = Document::default();
                                if welcome_document_card(ui, &doc, "Atelier", "Editorial website", 180.0).clicked() { template = true; }
                            } else {
                                ui.horizontal(|ui| {
                                    section_heading(ui, "Recent");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        search_field(ui, "recent_search", &mut self.welcome_search, "Search recent files", 220.0);
                                    });
                                });
                                let files: Vec<_> = self.recent.iter().filter(|p| p.to_string_lossy().to_lowercase().contains(&self.welcome_search.to_lowercase())).cloned().collect();
                                if files.is_empty() { ui.label(egui::RichText::new("Opened and saved documents will appear here.").color(secondary_text_color(ui))); }
                                let columns = ((content_width + 16.0) / 160.0).floor().max(1.0) as usize;
                                let card_width = (content_width - (columns - 1) as f32 * 16.0) / columns as f32;
                                for row in files.chunks(columns) {
                                    ui.horizontal_top(|ui| {
                                        for path in row {
                                            let modified = std::fs::metadata(path).ok().and_then(|m| m.modified().ok());
                                            let id = egui::Id::new(("recent_preview", path, modified));
                                            let doc = ctx.data(|d| d.get_temp::<Option<std::sync::Arc<Document>>>(id)).unwrap_or_else(|| {
                                                let doc = document_file::load(path).ok().map(std::sync::Arc::new);
                                                ctx.data_mut(|d| d.insert_temp(id, doc.clone()));
                                                doc
                                            });
                                            let name = path.file_stem().unwrap_or_default().to_string_lossy();
                                            if let Some(doc) = doc {
                                                if welcome_document_card(ui, &doc, &doc.name, &name, card_width).on_hover_text(path.display().to_string()).clicked() { chosen = Some(path.clone()); }
                                            } else {
                                                ui.allocate_ui_with_layout(Vec2::new(card_width, 180.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                                                    if ui.add_sized([card_width, 120.0], egui::Button::new("Preview unavailable")).clicked() { chosen = Some(path.clone()); }
                                                    ui.add_sized([card_width, 24.0], egui::Label::new(name.to_string()).truncate());
                                                });
                                            }
                                        }
                                    });
                                }
                            }
                        });
                        });
                    });
                });
            });
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("welcome_section"), section);
            d.insert_temp(egui::Id::new("welcome_size"), custom);
            d.insert_temp(egui::Id::new("welcome_presets"), preset_choices);
        });
        if open_file {
            self.open();
        }
        if let Some(path) = chosen {
            self.open_path(path);
        }
        if template {
            self.open_in_tab(Document::default(), None);
            self.welcome = false;
        }
        if let Some(size) = new_size {
            self.apply_action(Action::New, ctx);
            self.doc.pages[0].artboards[0].frame.width = size[0];
            self.doc.pages[0].artboards[0].frame.height = size[1];
            self.saved_document = self.doc.clone();
            self.welcome = false;
        }
    }
    fn apply_workbench(&mut self, command: workbench::Command) {
        let before = self.doc.clone();
        let result = match command {
            workbench::Command::Insert(mut boards) => {
                let mut x = self.doc.pages[self.page]
                    .artboards
                    .iter()
                    .map(|b| b.frame.x + b.frame.width)
                    .fold(0.0_f32, f32::max)
                    + 80.0;
                for board in &mut boards {
                    board.frame.x = x;
                    board.frame.y = 0.0;
                    x += board.frame.width + 80.0;
                }
                self.doc.pages[self.page].artboards.extend(boards);
                self.artboard = self.doc.pages[self.page].artboards.len() - 1;
                self.selected = None;
                self.selection.clear();
                self.selected_group = None;
                self.editing_text = None;
                self.editing_path = None;
                self.pen = false;
                self.path_node = None;
                let frame = &self.doc.pages[self.page].artboards[self.artboard].frame;
                self.zoom = (self.canvas_bounds.width() / (frame.width + 100.0))
                    .min(self.canvas_bounds.height() / (frame.height + 100.0))
                    .clamp(0.1, 1.0);
                self.pan = -Vec2::new(frame.x, frame.y) * self.zoom;
                Ok("Generated frames added. Save to keep them.".to_string())
            }
            workbench::Command::Select(finding) => {
                if finding.page < self.doc.pages.len()
                    && finding.board < self.doc.pages[finding.page].artboards.len()
                {
                    self.page = finding.page;
                    self.artboard = finding.board;
                    self.selected = finding.layer;
                    self.selection = finding.layer.into_iter().collect();
                    self.selected_group = None;
                    let frame = &self.doc.pages[self.page].artboards[self.artboard].frame;
                    self.pan = -Vec2::new(frame.x, frame.y) * self.zoom;
                }
                Ok(finding.message)
            }
            workbench::Command::SavePoint(name) => workbench::checkpoint(&mut self.doc, &name)
                .map(|_| "Checkpoint created. Save to persist it.".to_string()),
            workbench::Command::Restore(index) => workbench::restore(&self.doc, index).map(|doc| {
                self.doc = doc;
                self.page = 0;
                self.artboard = 0;
                self.selected = None;
                self.selection.clear();
                self.selected_group = None;
                self.editing_text = None;
                self.editing_path = None;
                "Checkpoint restored; Undo returns to your previous document.".to_string()
            }),
            workbench::Command::Deliver(path, notes) => {
                workbench::delivery(&self.doc, &path, &notes)
                    .map(|_| format!("Delivery package exported to {}", path.display()))
            }
        };
        self.status = result.unwrap_or_else(|e| format!("Workbench: {e}"));
        if self.doc != before {
            self.workbench_ui.clear_history_selection();
            self.record_change(before);
        }
    }
    fn record_change(&mut self, before: Document) {
        self.redo.clear();
        self.history.push(before);
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }
    fn checkpoint(&mut self) {
        self.redo.clear();
        self.history.push(self.doc.clone());
        if self.history.len() > 100 {
            self.history.remove(0);
        }
    }
    fn undo(&mut self) {
        if let Some(doc) = self.history.pop() {
            self.redo.push(self.doc.clone());
            self.doc = doc;
            self.reset_selection();
        }
    }
    fn path_editor(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        response: &egui::Response,
        origin: Pos2,
        _board: Rect,
    ) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Enter)) {
            self.pen = false;
            self.editing_path = None;
            self.path_drag = None;
            return;
        }
        let before = self.doc.clone();
        let pointer = ctx.input(|i| i.pointer.interact_pos());
        if self.pen
            && self.editing_path.is_none()
            && (response.clicked() || response.drag_started())
            && pointer.is_some_and(|p| ui.clip_rect().contains(p))
        {
            let mut layer = Layer::new(Kind::Rectangle, 0.0, 0.0);
            layer.name = "Path".into();
            layer.radius = 0.0;
            layer.width = 640.0;
            layer.height = 440.0;
            layer.path = Some(vector::Path::default());
            let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
            layers.push(layer);
            self.selected = Some(layers.len() - 1);
            self.editing_path = self.selected;
        }
        let Some(index) = self.editing_path else {
            ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
            return;
        };
        let Some(layer) = self.doc.pages[self.page].artboards[self.artboard]
            .layers
            .get_mut(index)
        else {
            self.editing_path = None;
            return;
        };
        if layer.locked {
            self.editing_path = None;
            self.pen = false;
            return;
        }
        let Some(path) = layer.path.as_mut() else {
            self.editing_path = None;
            return;
        };
        let base = origin + Vec2::new(layer.x, layer.y) * self.zoom;
        let size = Vec2::new(layer.width, layer.height) * self.zoom;
        let to_screen = |p: [f32; 2]| base + Vec2::new(p[0] * size.x, p[1] * size.y);
        let to_local = |p: Pos2| [(p.x - base.x) / size.x, (p.y - base.y) / size.y];
        let mut handle_used = false;
        let p = ui.painter().clone();
        for i in 0..path.nodes.len() {
            let node = path.nodes[i].clone();
            let anchor = to_screen(node.point);
            let selected = self.path_node == Some(i);
            if selected {
                for (which, handle) in [(false, node.incoming), (true, node.outgoing)] {
                    let h = to_screen(handle);
                    if h.distance(anchor) < 2.0 {
                        continue;
                    }
                    p.line_segment([anchor, h], Stroke::new(1.0_f32, Color32::from_gray(150)));
                    p.rect_filled(
                        Rect::from_center_size(h, Vec2::splat(6.0)),
                        1.0,
                        Color32::WHITE,
                    );
                    p.rect_stroke(
                        Rect::from_center_size(h, Vec2::splat(6.0)),
                        1.0,
                        Stroke::new(1.0_f32, ACCENT),
                        egui::StrokeKind::Outside,
                    );
                    let r = ui.interact(
                        Rect::from_center_size(h, Vec2::splat(14.0)),
                        egui::Id::new(("bezier_handle", index, i, which)),
                        Sense::drag(),
                    );
                    if r.dragged() {
                        if let Some(pos) = pointer {
                            path.nodes[i].move_handle(
                                to_local(pos),
                                which,
                                ctx.input(|v| v.modifiers.alt),
                            );
                        }
                    }
                    handle_used |= r.hovered() || r.dragged();
                    r.on_hover_cursor(egui::CursorIcon::Crosshair);
                }
            }
            p.circle_filled(anchor, 4.0, if selected { ACCENT } else { Color32::WHITE });
            p.circle_stroke(anchor, 4.0, Stroke::new(1.2_f32, ACCENT));
            let r = ui.interact(
                Rect::from_center_size(anchor, Vec2::splat(14.0)),
                egui::Id::new(("bezier_anchor", index, i)),
                Sense::click_and_drag(),
            );
            if r.clicked() {
                self.path_node = Some(i);
                if self.pen && i == 0 && path.nodes.len() > 2 {
                    path.closed = true;
                    path.fill = true;
                    self.pen = false;
                    self.editing_path = None;
                    self.path_node = None;
                }
            }
            if r.dragged() && !self.pen {
                self.path_node = Some(i);
                if let Some(pos) = pointer {
                    path.nodes[i].move_anchor(to_local(pos));
                }
            }
            handle_used |= r.hovered() || r.dragged();
            r.on_hover_cursor(egui::CursorIcon::Crosshair);
        }
        if self.pen && !handle_used && (response.clicked() || response.drag_started()) {
            let pos = if response.drag_started() {
                ctx.input(|i| i.pointer.press_origin())
            } else {
                pointer
            };
            if let Some(pos) = pos.filter(|p| ui.clip_rect().contains(*p)) {
                let point = to_local(pos);
                path.nodes.push(vector::Node::at(point));
                self.path_node = Some(path.nodes.len() - 1);
                if response.drag_started() {
                    self.path_drag = self.path_node;
                    path.nodes.last_mut().unwrap().mode = vector::NodeMode::Symmetric;
                }
            }
        }
        if self.pen && response.dragged() {
            if let (Some(i), Some(pos)) = (self.path_drag, pointer) {
                path.nodes[i].move_handle(to_local(pos), true, false);
            }
        }
        if response.drag_stopped() {
            self.path_drag = None;
        }
        if !ctx.wants_keyboard_input()
            && ctx
                .input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
        {
            if let Some(i) = self.path_node.filter(|i| *i < path.nodes.len()) {
                path.nodes.remove(i);
                self.path_node = None;
            }
        }
        self.status="Path editing · drag anchors and handles · Alt for independent handles · Enter to finish".into();
        ctx.set_cursor_icon(egui::CursorIcon::Crosshair);
        if before != self.doc {
            vector::normalize_layer(
                &mut self.doc.pages[self.page].artboards[self.artboard].layers[index],
            );
            // One undo entry for a pen point or handle drag, not for every motion sample.
            let dragging = ctx.input(|i| i.pointer.primary_down());
            let id = egui::Id::new("path_undo_drag_active");
            let started = ctx.data(|d| d.get_temp::<bool>(id).unwrap_or(false));
            if !dragging || !started {
                self.record_change(before);
            }
            ctx.data_mut(|d| d.insert_temp(id, dragging));
        }
        if !ctx.input(|i| i.pointer.primary_down()) {
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("path_undo_drag_active"), false));
        }
    }

    fn select_group(&mut self, artboard: usize, group: String) {
        if artboard != self.artboard {
            self.pan = Vec2::ZERO;
        }
        self.artboard = artboard;
        self.selection = self.doc.pages[self.page].artboards[artboard]
            .layers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.group.as_ref() == Some(&group) && !l.locked)
            .map(|(i, _)| i)
            .collect();
        self.selected = self.selection.last().copied().or_else(|| {
            self.doc.pages[self.page].artboards[artboard]
                .layers
                .iter()
                .position(|l| l.group.as_ref() == Some(&group))
        });
        self.selected_group = Some(group);
        self.editing_text = None;
        self.editing_path = None;
    }
    fn reset_selection(&mut self) {
        self.marquee_start = None;
        self.marquee_base.clear();
        self.page = self.page.min(self.doc.pages.len() - 1);
        self.artboard = self
            .artboard
            .min(self.doc.pages[self.page].artboards.len() - 1);
        self.selected = None;
        self.selected_group = None;
        self.selection.clear();
        self.group_drag.clear();
        self.editing_text = None;
        self.editing_path = None;
        self.pen = false;
        self.path_node = None;
        self.drag_origin = None;
        self.resize_corner = None;
        self.pan = Vec2::ZERO;
    }
    fn session_json(&mut self) -> Result<String, String> {
        self.ensure_tabs();
        self.tabs[self.active_tab] = self.tab_snapshot();
        serde_json::to_string(&SavedSession {
            active: self.active_tab,
            tabs: self
                .tabs
                .iter()
                .map(|t| SavedSessionTab {
                    doc: t.doc.clone(),
                    saved: t.saved.clone(),
                    path: t.path.clone(),
                    page: t.page,
                    board: t.board,
                })
                .collect(),
        })
        .map_err(|e| e.to_string())
    }
    fn restore_session(&mut self, json: &str) -> Result<(), String> {
        let session: SavedSession = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if session.tabs.is_empty() {
            return Err("Empty saved session".into());
        }
        // Validate every document before touching the current session.
        for t in &session.tabs {
            Document::from_json(&serde_json::to_string(&t.doc).map_err(|e| e.to_string())?)?;
            Document::from_json(&serde_json::to_string(&t.saved).map_err(|e| e.to_string())?)?;
        }
        self.tabs.clear();
        for t in session.tabs {
            self.doc = t.doc;
            self.saved_document = t.saved;
            self.document_path = t.path;
            self.page = t.page;
            self.artboard = t.board;
            self.reset_selection();
            self.history.clear();
            self.redo.clear();
            self.tabs.push(self.tab_snapshot());
        }
        self.active_tab = self.tabs.len() - 1;
        let active = session.active.min(self.active_tab);
        self.switch_tab(active);
        self.welcome = false;
        self.status = "Previous session restored".into();
        Ok(())
    }
    fn save_to_path(&mut self, path: std::path::PathBuf) -> Result<(), String> {
        let bytes = document_file::encode(&self.doc, &path)?;
        // A failed write must not truncate an existing document or mark it saved.
        write_document_atomic(&path, &bytes)?;
        self.document_path = Some(path.clone());
        self.saved_document = self.doc.clone();
        self.remember_document(path);
        self.ensure_tabs();
        self.tabs[self.active_tab] = self.tab_snapshot();
        self.status = "Document saved".into();
        Ok(())
    }
    fn save(&mut self) {
        if let Some(path) = self.document_path.clone().or_else(|| {
            rfd::FileDialog::new()
                .add_filter("SUIZA Canvas document", &["suiza-ui"])
                .add_filter("Legacy JSON document", &["json"])
                .set_file_name(format!("{}.suiza-ui", self.doc.name))
                .save_file()
        }) {
            if let Err(e) = self.save_to_path(path) {
                self.status = format!("Save failed: {e}");
            }
        }
    }
    fn open_path(&mut self, path: std::path::PathBuf) {
        match document_file::load(&path) {
            Ok(doc) => {
                self.open_in_tab(doc, Some(path.clone()));
                self.remember_document(path);
                self.welcome = false;
                let missing = fonts::missing_families(&self.doc);
                self.status = if missing.is_empty() {
                    "Document opened".into()
                } else {
                    format!(
                        "Document opened · Missing fonts: {} (using fallback)",
                        missing.join(", ")
                    )
                };
            }
            Err(e) => self.status = format!("Open failed: {e}"),
        }
    }
    fn open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SUIZA Canvas document", &["suiza-ui"])
            .add_filter("Legacy JSON document", &["json"])
            .pick_file()
        {
            self.open_path(path);
        }
    }
}

impl Studio {
    fn show(&mut self, ctx: &egui::Context) {
        self.menus(ctx);
        self.show_tabs(ctx);
        if !ctx.wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(egui::Key::V) && !i.modifiers.command) {
                self.apply_action(Action::Tool(None), ctx);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::P) && !i.modifiers.command) {
                self.apply_action(Action::Pen, ctx);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::R) && !i.modifiers.command) {
                self.apply_action(Action::Tool(Some(Kind::Rectangle)), ctx);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::O) && !i.modifiers.command) {
                self.apply_action(Action::Tool(Some(Kind::Ellipse)), ctx);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::T) && !i.modifiers.command) {
                self.apply_action(Action::Tool(Some(Kind::Text)), ctx);
            }
            if self.editing_path.is_none()
                && ctx.input(|i| {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                })
            {
                self.apply_action(Action::Delete, ctx);
            }
        }
        egui::TopBottomPanel::bottom("status")
            .exact_height(24.0)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(egui::Margin::symmetric(12, 3)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&self.status)
                            .size(10.0)
                            .color(secondary_text_color(ui)),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Space to pan · Alt to bypass snapping")
                                .size(10.0)
                                .color(secondary_text_color(ui)),
                        );
                    });
                });
            });
        let frame_size = Vec2::new(
            self.doc.pages[self.page].artboards[self.artboard]
                .frame
                .width,
            self.doc.pages[self.page].artboards[self.artboard]
                .frame
                .height,
        );
        let mut navigate = None;
        let mut add_page = false;
        let mut add_artboard = false;
        let mut tree_selection = None;
        let mut tree_group_selection = None;
        let tree_before = self.doc.clone();
        self.show_tool_rail(ctx);
        egui::SidePanel::left("layers")
            .default_width(224.0)
            .width_range(200.0..=440.0)
            .resizable(true)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(10),
            )
            .show(ctx, |ui| {
                let pane_width = ui.available_width();
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.assets_ui.active, false, "Layers");
                    ui.selectable_value(&mut self.assets_ui.active, true, "Assets");
                });
                if self.assets_ui.active {
                    match self.assets_ui.show(ui, &self.doc) {
                        Ok(Some(component)) => {
                            self.checkpoint();
                            let board = &mut self.doc.pages[self.page].artboards[self.artboard];
                            let position = Vec2::new(
                                (board.frame.width - component.frame.width) / 2.0,
                                (board.frame.height - component.frame.height) / 2.0,
                            );
                            let (group, indices) =
                                assets::insert_layers(&component, board, position);
                            self.selected = indices.first().copied();
                            self.selection = indices;
                            self.selected_group = Some(group);
                            self.editing_text = None;
                            self.editing_path = None;
                            self.tool = None;
                            self.pen = false;
                            self.status = format!("Inserted {}", component.name);
                        }
                        Err(e) => self.status = format!("Library import failed: {e}"),
                        _ => {}
                    }
                    return;
                }
                ui.spacing_mut().item_spacing.y = 3.0;
                let search_height = 32.0;
                let tree_height = (ui.available_height() - search_height).max(80.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(pane_width, tree_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_max_width(pane_width);
                        ui.add_space(4.0);
                        let mut new_document = false;
                        ui.horizontal(|ui| {
                            section_heading(ui, "Workspace");
                            if ui
                                .small_button("Recents")
                                .on_hover_text("Recent documents and templates")
                                .clicked()
                            {
                                self.welcome = true;
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    new_document = small_icon(ui, 9, "New document").clicked();
                                },
                            );
                        });
                        if new_document {
                            self.apply_action(Action::New, ctx);
                        }
                        ui.add_space(5.0);
                        let (row, response) =
                            ui.allocate_exact_size(Vec2::new(pane_width, 30.0), Sense::click());
                        ui.painter().rect_filled(row, 8.0, ACCENT);
                        paint_icon(
                            ui.painter(),
                            row.left_center() + Vec2::new(16.0, 0.0),
                            13,
                            Color32::WHITE,
                            0.75,
                        );
                        let mut job = egui::text::LayoutJob::simple(
                            self.doc.name.clone(),
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                            pane_width - 40.0,
                        );
                        job.wrap = egui::text::TextWrapping::truncate_at_width(pane_width - 40.0);
                        let galley = ui.painter().layout_job(job);
                        ui.painter().galley(
                            row.left_center() + Vec2::new(31.0, -galley.size().y / 2.0),
                            galley,
                            Color32::WHITE,
                        );
                        response.on_hover_text(&self.doc.name);
                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(9.0);
                        ui.horizontal(|ui| {
                            section_heading(ui, "Pages");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if small_icon(ui, 9, "Add page").clicked() {
                                        add_page = true;
                                    }
                                },
                            );
                        });
                        for (pi, page) in self.doc.pages.iter().enumerate() {
                            if navigation_row(ui, &page.name, pi == self.page).clicked() {
                                navigate = Some((pi, 0));
                            }
                        }
                        ui.add_space(9.0);
                        ui.separator();
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Layers")
                                    .size(10.0)
                                    .color(secondary_text_color(ui)),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if small_icon(ui, 10, "Add artboard").clicked() {
                                        add_artboard = true;
                                    }
                                },
                            );
                        });
                        egui::ScrollArea::vertical()
                            .id_salt("layer_tree")
                            .max_height(ui.available_height().max(40.0))
                            .auto_shrink([true, false])
                            .show(ui, |ui| {
                                ui.set_max_width(pane_width);
                                for (ai, board) in
                                    self.doc.pages[self.page].artboards.iter_mut().enumerate()
                                {
                                    let header = tree_section(
                                        ui,
                                        egui::Id::new(("board", self.page, ai)),
                                        &board.name,
                                        10,
                                        ai == 0,
                                        ai == self.artboard && self.selected.is_none(),
                                        |ui| {
                                            ui.set_max_width((pane_width - 18.0).max(120.0));
                                            let groups: std::collections::BTreeSet<String> = board
                                                .layers
                                                .iter()
                                                .filter_map(|l| l.group.clone())
                                                .collect();
                                            for group in groups {
                                                let matching = board.layers.iter().any(|l| {
                                                    l.group.as_ref() == Some(&group)
                                                        && l.label
                                                            .as_deref()
                                                            .unwrap_or(&l.name)
                                                            .to_lowercase()
                                                            .contains(&self.search.to_lowercase())
                                                });
                                                if !matching {
                                                    continue;
                                                }
                                                let group_id =
                                                    egui::Id::new(("group", self.page, ai, &group));
                                                let group_header = tree_section(
                                                    ui,
                                                    egui::Id::new(("group", self.page, ai, &group)),
                                                    &group,
                                                    5,
                                                    true,
                                                    ai == self.artboard
                                                        && self.selected_group.as_ref()
                                                            == Some(&group),
                                                    |ui| {
                                                        ui.set_max_width(
                                                            (pane_width - 36.0).max(100.0),
                                                        );
                                                        for (li, layer) in board
                                                            .layers
                                                            .iter_mut()
                                                            .enumerate()
                                                            .rev()
                                                            .filter(|(_, l)| {
                                                                l.group.as_ref() == Some(&group)
                                                            })
                                                        {
                                                            if layer
                                                                .label
                                                                .as_deref()
                                                                .unwrap_or(&layer.name)
                                                                .to_lowercase()
                                                                .contains(
                                                                    &self.search.to_lowercase(),
                                                                )
                                                                && tree_row(
                                                                    ui,
                                                                    layer,
                                                                    ai == self.artboard
                                                                        && self
                                                                            .selected_group
                                                                            .is_none()
                                                                        && self.selected
                                                                            == Some(li),
                                                                    ui.available_width(),
                                                                    (ai, li),
                                                                )
                                                            {
                                                                tree_selection = Some((ai, li));
                                                            }
                                                        }
                                                    },
                                                );
                                                if group_header {
                                                    tree_group_selection =
                                                        Some((ai, group.clone()));
                                                }
                                                if let Some(name) = ui.ctx().data_mut(|d| {
                                                    d.remove_temp::<String>(
                                                        group_id.with("renamed"),
                                                    )
                                                }) {
                                                    if self.selected_group.as_ref() == Some(&group)
                                                    {
                                                        self.selected_group = Some(name.clone());
                                                    }
                                                    for layer in &mut board.layers {
                                                        if layer.group.as_ref() == Some(&group) {
                                                            layer.group = Some(name.clone());
                                                        }
                                                    }
                                                }
                                            }
                                            for (li, layer) in board
                                                .layers
                                                .iter_mut()
                                                .enumerate()
                                                .rev()
                                                .filter(|(_, l)| l.group.is_none())
                                            {
                                                if layer
                                                    .label
                                                    .as_deref()
                                                    .unwrap_or(&layer.name)
                                                    .to_lowercase()
                                                    .contains(&self.search.to_lowercase())
                                                    && tree_row(
                                                        ui,
                                                        layer,
                                                        ai == self.artboard
                                                            && self.selected == Some(li),
                                                        ui.available_width(),
                                                        (ai, li),
                                                    )
                                                {
                                                    tree_selection = Some((ai, li));
                                                }
                                            }
                                        },
                                    );
                                    if let Some(name) = ui.ctx().data_mut(|d| {
                                        d.remove_temp::<String>(
                                            egui::Id::new(("board", self.page, ai)).with("renamed"),
                                        )
                                    }) {
                                        board.name = name;
                                    }
                                    if header {
                                        navigate = Some((self.page, ai));
                                    }
                                }
                            });
                    },
                );
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    search_field(
                        ui,
                        "layer_search",
                        &mut self.search,
                        "Search layers…",
                        pane_width,
                    );
                });
            });
        // Frame headers accept drops even when their layer list is empty or collapsed.
        if ctx.input(|i| i.pointer.any_released()) {
            if let Some(source) =
                ctx.data(|d| d.get_temp::<(usize, usize)>(egui::Id::new("layer_reorder_source")))
            {
                for ai in 0..self.doc.pages[self.page].artboards.len() {
                    let row = ctx.data(|d| {
                        d.get_temp::<Rect>(egui::Id::new(("board", self.page, ai)).with("row_rect"))
                    });
                    if row.is_some_and(|r| ctx.pointer_hover_pos().is_some_and(|p| r.contains(p))) {
                        ctx.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new("layer_reorder_request"),
                                (source, (ai, usize::MAX)),
                            )
                        });
                        break;
                    }
                }
            }
        }
        if let Some(((source_board, source_layer), (target_board, target_layer))) =
            ctx.data_mut(|d| {
                d.remove_temp::<((usize, usize), (usize, usize))>(egui::Id::new(
                    "layer_reorder_request",
                ))
            })
        {
            if let Some(target) = reorder_layer(
                &mut self.doc.pages[self.page].artboards,
                (source_board, source_layer),
                (target_board, target_layer),
            ) {
                let old = &self.doc.pages[self.page].artboards[self.artboard].frame;
                let new = &self.doc.pages[self.page].artboards[target_board].frame;
                self.pan += Vec2::new(
                    (target_board as f32 - self.artboard as f32) * 720.0
                        + (new.width - old.width) / 2.0,
                    (new.height - old.height) / 2.0,
                ) * self.zoom;
                self.artboard = target_board;
                self.selected = Some(target);
                self.selection.clear();
                self.selected_group = None;
                self.editing_text = None;
                self.editing_path = None;
            }
        }

        if ctx.input(|i| i.pointer.any_released()) {
            ctx.data_mut(|d| {
                d.remove_temp::<(usize, usize)>(egui::Id::new("layer_reorder_source"))
            });
        }
        if tree_before != self.doc {
            self.record_change(tree_before);
        }
        if let Some((page, artboard)) = navigate {
            self.page = page;
            self.artboard = artboard;
            self.reset_selection();
        }
        if let Some((ai, group)) = tree_group_selection {
            self.select_group(ai, group);
            if ctx.input(|i| i.pointer.secondary_clicked()) {
                ctx.memory_mut(|m| m.open_popup(egui::Id::new("sidebar_layer_context")));
                let position = ctx
                    .input(|i| i.pointer.interact_pos())
                    .unwrap_or(Pos2::ZERO);
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("sidebar_context_position"), position)
                });
            }
        }
        if let Some((ai, li)) = tree_selection {
            self.selected_group = None;
            if ctx.input(|i| i.modifiers.shift) && ai == self.artboard {
                if let Some(i) = self.selected {
                    if !self.selection.contains(&i) {
                        self.selection.push(i);
                    }
                }
                if !self.selection.contains(&li) {
                    self.selection.push(li);
                }
            } else {
                self.selection.clear();
            }
            self.editing_text = None;
            if ai != self.artboard {
                self.pan = Vec2::ZERO;
            }
            self.artboard = ai;
            self.selected = Some(li);
            if ctx.input(|i| i.pointer.secondary_clicked()) {
                ctx.memory_mut(|m| m.open_popup(egui::Id::new("sidebar_layer_context")));
                let position = ctx
                    .input(|i| i.pointer.interact_pos())
                    .unwrap_or(Pos2::ZERO);
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("sidebar_context_position"), position)
                });
            }
        }
        if ctx.memory(|m| m.is_popup_open(egui::Id::new("sidebar_layer_context"))) {
            let position = ctx
                .data(|d| d.get_temp::<Pos2>(egui::Id::new("sidebar_context_position")))
                .unwrap_or(Pos2::ZERO);
            egui::Area::new(egui::Id::new("sidebar_layer_context_area"))
                .order(egui::Order::Foreground)
                .fixed_pos(position)
                .show(ctx, |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        self.layer_context_menu(ui, ctx);
                    });
                });
            if ctx.input(|i| i.pointer.primary_clicked() || i.key_pressed(egui::Key::Escape)) {
                ctx.memory_mut(|m| m.close_popup());
            }
        }
        if add_page {
            self.checkpoint();
            let number = self.doc.pages.len() + 1;
            self.doc.pages.push(Page {
                name: format!("Page {number}"),
                artboards: vec![Artboard {
                    prototype_id: String::new(),
                    frame: FrameStyle::default(),
                    name: "Artboard 1".into(),
                    layers: vec![],
                }],
            });
            self.page = self.doc.pages.len() - 1;
            self.artboard = 0;
            self.reset_selection();
        }
        if add_artboard {
            self.checkpoint();
            let boards = &mut self.doc.pages[self.page].artboards;
            let number = boards.len() + 1;
            boards.push(Artboard {
                prototype_id: String::new(),
                frame: FrameStyle::default(),
                name: format!("Artboard {number}"),
                layers: vec![],
            });
            self.artboard = boards.len() - 1;
            self.reset_selection();
        }
        egui::SidePanel::right("inspector")
            .exact_width(264.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .inner_margin(12),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let field_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                    if let Some(group) = self.selected_group.clone() {
                        let before = self.doc.clone();
                        section_heading(ui, "Group");
                        ui.label(&group);
                        let indices = self.affected_layers();
                        let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                        if let Some(bounds) = indices
                            .iter()
                            .map(|i| {
                                Rect::from_min_size(
                                    Pos2::new(layers[*i].x, layers[*i].y),
                                    Vec2::new(layers[*i].width, layers[*i].height),
                                )
                            })
                            .reduce(|a, b| a.union(b))
                        {
                            let mut x = bounds.left();
                            let mut y = bounds.top();
                            let mut w = bounds.width();
                            let mut h = bounds.height();
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                property(ui, "X", &mut x, false, field_width);
                                property(ui, "Y", &mut y, false, field_width);
                            });
                            section_heading(ui, "Layout");
                            ui.horizontal(|ui| {
                                property(ui, "W", &mut w, true, field_width);
                                property(ui, "H", &mut h, true, field_width);
                            });
                            for i in indices {
                                let l = &mut layers[i];
                                l.x = x + (l.x - bounds.left()) * w / bounds.width();
                                l.y = y + (l.y - bounds.top()) * h / bounds.height();
                                l.width *= w / bounds.width();
                                l.height *= h / bounds.height();
                            }
                        }
                        if before != self.doc {
                            self.record_change(before);
                        }
                        ui.add_space(10.0);
                        ui.label("Select a child in the layers pane to edit it independently.");
                        for (label, action) in [
                            ("Ungroup", Action::Ungroup),
                            ("Duplicate group", Action::Duplicate),
                            ("Delete group", Action::Delete),
                        ] {
                            if ui
                                .add_enabled(
                                    self.action_enabled(action, ctx),
                                    egui::Button::new(label)
                                        .min_size(Vec2::new(ui.available_width(), 28.0)),
                                )
                                .clicked()
                            {
                                self.apply_action(action, ctx);
                            }
                        }
                    } else if let Some(index) = self.selected {
                        let before = self.doc.clone();
                        let layer =
                            &mut self.doc.pages[self.page].artboards[self.artboard].layers[index];
                        section_heading(
                            ui,
                            match layer.kind {
                                Kind::Rectangle => "Rectangle",
                                Kind::Ellipse => "Ellipse",
                                Kind::Text => "Text",
                                Kind::Image => "Image",
                            },
                        );
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 1.0;
                            for (action, tip) in [
                                "Align left",
                                "Center horizontally",
                                "Align right",
                                "Align top",
                                "Center vertically",
                                "Align bottom",
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                if alignment_button(ui, action, tip).clicked() {
                                    match action {
                                        0 => layer.x = 0.0,
                                        1 => layer.x = (frame_size.x - layer.width) / 2.0,
                                        2 => layer.x = frame_size.x - layer.width,
                                        3 => layer.y = 0.0,
                                        4 => layer.y = (frame_size.y - layer.height) / 2.0,
                                        _ => layer.y = frame_size.y - layer.height,
                                    }
                                }
                            }
                        });
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            property(ui, "X", &mut layer.x, false, field_width);
                            property(ui, "Y", &mut layer.y, false, field_width);
                        });
                        ui.add_space(10.0);
                        section_heading(ui, "Layout");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            property(ui, "W", &mut layer.width, true, field_width);
                            property(ui, "H", &mut layer.height, true, field_width);
                        });
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Fixed size · Artboard coordinates")
                                .size(10.0)
                                .color(secondary_text_color(ui)),
                        );
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);
                        section_heading(ui, "Layer");
                        ui.checkbox(&mut layer.locked, "Locked")
                            .on_hover_text("Prevent canvas moves, resize, and deletion");
                        let group_names: std::collections::BTreeSet<_> = before.pages[self.page]
                            .artboards[self.artboard]
                            .layers
                            .iter()
                            .filter_map(|l| l.group.clone())
                            .collect();
                        egui::ComboBox::from_id_salt("group_membership")
                            .selected_text(layer.group.as_deref().unwrap_or("No group"))
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut layer.group, None, "No group");
                                for name in group_names {
                                    ui.selectable_value(&mut layer.group, Some(name.clone()), name);
                                }
                                if ui.button("New group").clicked() {
                                    layer.group = Some(next_group_name(
                                        &before.pages[self.page].artboards[self.artboard].layers,
                                    ));
                                    ui.close_menu();
                                }
                            });
                        ui.add_space(10.0);
                        section_heading(ui, "Content");
                        ui.add_space(8.0);
                        if layer.kind == Kind::Text {
                            ui.add(
                                egui::TextEdit::multiline(&mut layer.name)
                                    .desired_width(ui.available_width())
                                    .desired_rows(2),
                            );
                            ui.add_space(8.0);
                            section_heading(ui, "Type");
                            ui.add_space(6.0);
                            ui.scope(|ui| type_controls(ui, layer));
                        } else {
                            ui.add(
                                egui::TextEdit::singleline(&mut layer.name)
                                    .desired_width(ui.available_width()),
                            );
                        }
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);
                        section_heading(ui, "Style");
                        ui.add_space(8.0);
                        if layer.path.is_some() {
                            vector_inspector(
                                ui,
                                layer,
                                &mut self.editing_path,
                                index,
                                &mut self.path_node,
                            );
                        }
                        if layer.kind == Kind::Rectangle && layer.path.is_none() {
                            ui.horizontal(|ui| {
                                section_heading(ui, "Corners");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let mut individual = layer.corners.is_some();
                                        if ui
                                            .toggle_value(&mut individual, "▣")
                                            .on_hover_text("Individual corner radii")
                                            .changed()
                                        {
                                            layer.corners = if individual {
                                                Some([layer.radius; 4])
                                            } else {
                                                None
                                            };
                                        }
                                    },
                                );
                            });
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt("corner_style")
                                    .width(108.0)
                                    .selected_text(&layer.corner_style)
                                    .show_ui(ui, |ui| {
                                        for style in [
                                            "Auto",
                                            "Rounded",
                                            "Smooth",
                                            "Angled",
                                            "Inside Square",
                                            "Inside Arc",
                                        ] {
                                            ui.selectable_value(
                                                &mut layer.corner_style,
                                                style.into(),
                                                style,
                                            );
                                        }
                                    });
                                if layer.corners.is_none() {
                                    ui.add_sized(
                                        [88.0, 26.0],
                                        egui::DragValue::new(&mut layer.radius)
                                            .range(0.0..=200.0)
                                            .prefix("Radius "),
                                    );
                                }
                            });
                            if let Some(values) = &mut layer.corners {
                                for row in 0..2 {
                                    ui.horizontal(|ui| {
                                        for col in 0..2 {
                                            let i = row * 2 + col;
                                            ui.add_sized(
                                                [103.0, 26.0],
                                                egui::DragValue::new(&mut values[i])
                                                    .range(0.0..=200.0)
                                                    .prefix(["TL ", "TR ", "BL ", "BR "][i]),
                                            );
                                        }
                                    });
                                }
                            }
                            ui.add_space(8.0);
                        }
                        section_heading(ui, "Fills");
                        ui.add_space(8.0);
                        fill_control(ui, &mut layer.color, None);
                        ui.add_space(8.0);
                        ui.checkbox(&mut layer.visible, "Visible");
                        ui.add_space(8.0);
                        shadow_controls(ui, "Shadows", &mut layer.drop_shadow);
                        shadow_controls(ui, "Inner shadows", &mut layer.inner_shadow);
                        ui.horizontal(|ui| {
                            section_heading(ui, "Effects");
                            ui.add_space((ui.available_width() - 28.0).max(0.0));
                            ui.menu_button("+", |ui| {
                                for kind in [
                                    "Glass",
                                    "Fade",
                                    "Layer Blur",
                                    "Background Blur",
                                    "Motion & Zoom Blur",
                                ] {
                                    if ui
                                        .add_enabled(true, egui::Button::new(kind))
                                        .on_hover_text(
                                            if kind == "Glass" || kind == "Background Blur" {
                                                "Blur the artwork behind this object"
                                            } else {
                                                "Add effect"
                                            },
                                        )
                                        .clicked()
                                    {
                                        layer.effects.push(EffectStyle {
                                            zoom: false,
                                            kind: kind.into(),
                                            amount: if kind == "Fade" { 50.0 } else { 8.0 },
                                            enabled: true,
                                        });
                                        ui.close_menu();
                                    }
                                }
                            });
                        });
                        let mut remove = None;
                        for (i, effect) in layer.effects.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut effect.enabled, "");
                                ui.add_sized(
                                    [100.0, 24.0],
                                    egui::Label::new(&effect.kind).truncate(),
                                );
                                ui.add_sized(
                                    [42.0, 24.0],
                                    egui::DragValue::new(&mut effect.amount).range(0.0..=100.0),
                                );
                                if close_icon_button(ui, 24.0)
                                    .on_hover_text("Remove effect")
                                    .clicked()
                                {
                                    remove = Some(i);
                                }
                            });
                        }
                        for effect in layer
                            .effects
                            .iter_mut()
                            .filter(|e| e.kind == "Motion & Zoom Blur")
                        {
                            ui.checkbox(&mut effect.zoom, "Zoom blur (radial)");
                        }
                        if let Some(i) = remove {
                            layer.effects.remove(i);
                        }
                        if before != self.doc {
                            self.record_change(before);
                            if self.history.len() > 100 {
                                self.history.remove(0);
                            }
                        }
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);
                        section_heading(ui, "Layer actions");
                        ui.add_space(8.0);
                        for (label, action) in [
                            ("Duplicate", Action::Duplicate),
                            ("Bring to front", Action::Front),
                            ("Delete layer", Action::Delete),
                        ] {
                            if ui
                                .add_enabled(
                                    self.action_enabled(action, ctx),
                                    egui::Button::new(label)
                                        .min_size(Vec2::new(ui.available_width(), 28.0)),
                                )
                                .clicked()
                            {
                                self.apply_action(action, ctx);
                            }
                        }
                        if let Some(layer) = self
                            .selected
                            .and_then(|i| {
                                self.doc.pages[self.page].artboards[self.artboard]
                                    .layers
                                    .get(i)
                            })
                            .cloned()
                        {
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(8.0);
                            section_heading(ui, "Export");
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt("export_format")
                                    .width(90.0)
                                    .selected_text(&self.export_format)
                                    .show_ui(ui, |ui| {
                                        for format in ["PNG", "JPG", "TIFF", "WEBP", "PDF", "SVG"] {
                                            ui.selectable_value(
                                                &mut self.export_format,
                                                format.into(),
                                                format,
                                            );
                                        }
                                    });
                                egui::ComboBox::from_id_salt("export_scale")
                                    .width(86.0)
                                    .selected_text(format!("{}×", self.export_scale))
                                    .show_ui(ui, |ui| {
                                        for scale in [1, 2, 3, 4] {
                                            ui.selectable_value(
                                                &mut self.export_scale,
                                                scale,
                                                format!("{scale}×"),
                                            );
                                        }
                                    });
                            });
                            ui.checkbox(&mut self.export_trim, "Trim transparent pixels");
                            if ui
                                .add_sized(
                                    [ui.available_width(), 28.0],
                                    egui::Button::new("Export selected…"),
                                )
                                .clicked()
                            {
                                let ext = self.export_format.to_lowercase();
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter(&self.export_format, &[ext.as_str()])
                                    .set_file_name(format!("asset@{}x.{ext}", self.export_scale))
                                    .save_file()
                                {
                                    match export_layer(
                                        &layer,
                                        &path,
                                        &self.export_format,
                                        self.export_scale,
                                        self.export_trim,
                                        &self.doc.pages[self.page].artboards[self.artboard].layers
                                            [..self.selected.unwrap_or(0)],
                                    ) {
                                        Ok(()) => {
                                            self.status = format!("Exported {}", path.display())
                                        }
                                        Err(error) => {
                                            self.status = format!("Export failed: {error}")
                                        }
                                    }
                                }
                            }
                            ui.add_space(12.0);
                            export_preview(
                                ui,
                                &layer,
                                &self.doc.pages[self.page].artboards[self.artboard].layers
                                    [..self.selected.unwrap_or(0)],
                            );
                        }
                    } else {
                        let before = self.doc.clone();
                        section_heading(ui, "Frame");
                        ui.add_space(8.0);
                        let board = &mut self.doc.pages[self.page].artboards[self.artboard];
                        ui.add(
                            egui::TextEdit::singleline(&mut board.name)
                                .desired_width(ui.available_width()),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [field_width, 26.0],
                                egui::DragValue::new(&mut board.frame.x)
                                    .prefix("X ")
                                    .speed(1.0),
                            );
                            ui.add_sized(
                                [field_width, 26.0],
                                egui::DragValue::new(&mut board.frame.y)
                                    .prefix("Y ")
                                    .speed(1.0),
                            );
                        });
                        ui.add_space(16.0);
                        section_heading(ui, "Layout");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [field_width, 26.0],
                                egui::DragValue::new(&mut board.frame.width)
                                    .prefix("W ")
                                    .range(1.0..=8192.0)
                                    .speed(1.0),
                            );
                            ui.add_sized(
                                [field_width, 26.0],
                                egui::DragValue::new(&mut board.frame.height)
                                    .prefix("H ")
                                    .range(1.0..=8192.0)
                                    .speed(1.0),
                            );
                        });
                        ui.checkbox(&mut board.frame.clip, "Clip content");
                        ui.add_space(16.0);
                        section_heading(ui, "Corners");
                        ui.add_space(8.0);
                        ui.add_sized(
                            [ui.available_width(), 26.0],
                            egui::DragValue::new(&mut board.frame.radius)
                                .prefix("Rounded  ")
                                .range(0.0..=200.0),
                        );
                        ui.add_space(16.0);
                        section_heading(ui, "Background");
                        ui.add_space(8.0);
                        fill_control(ui, &mut board.frame.background, None);
                        if before != self.doc {
                            self.record_change(before);
                        }
                        ui.add_space(16.0);
                        section_heading(ui, "Export");
                        ui.add_space(8.0);
                        if ui
                            .add_sized(
                                [ui.available_width(), 28.0],
                                egui::Button::new("Export frame…"),
                            )
                            .clicked()
                        {
                            self.apply_action(Action::BoardExport, ctx);
                        }
                    }
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(if ctx.style().visuals.dark_mode {
                Color32::from_rgb(29, 30, 32)
            } else {
                Color32::from_rgb(239, 239, 239)
            }))
            .show(ctx, |ui| {
                self.canvas_bounds = ui.available_rect_before_wrap();
                let header_height = canvas_header_height(self.canvas_bounds.width());
                let header = Rect::from_min_size(
                    self.canvas_bounds.min,
                    Vec2::new(self.canvas_bounds.width(), header_height),
                );
                ui.painter()
                    .rect_filled(header, 0.0, ctx.style().visuals.panel_fill);
                ui.painter().line_segment(
                    [header.left_bottom(), header.right_bottom()],
                    Stroke::new(
                        1.0_f32,
                        ctx.style().visuals.widgets.noninteractive.bg_stroke.color,
                    ),
                );
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.allocate_space(Vec2::new(self.canvas_bounds.width(), header_height));
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                let area = response.rect;
                let space = ctx.input(|i| i.key_down(egui::Key::Space));
                if response.hovered() {
                    let scroll = ctx.input(|i| i.smooth_scroll_delta);
                    let mut zoom_delta = ctx.input(|i| i.zoom_delta());
                    if zoom_delta == 1.0 && ctx.input(|i| i.modifiers.ctrl || i.modifiers.command) {
                        zoom_delta = (scroll.y * 0.003).exp();
                    }
                    if zoom_delta != 1.0 {
                        let old = self.zoom;
                        self.zoom = (old * zoom_delta).clamp(0.25, 2.0);
                        let anchor = ctx
                            .pointer_hover_pos()
                            .unwrap_or(self.canvas_bounds.center())
                            - self.canvas_bounds.center();
                        self.pan = anchor - (anchor - self.pan) * (self.zoom / old);
                    }
                    if !ctx.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                        self.pan += scroll;
                    }
                }
                if space && response.dragged() {
                    self.pan += ctx.input(|i| i.pointer.delta());
                }
                if space {
                    ctx.set_cursor_icon(egui::CursorIcon::Grab);
                }
                let frame = self.doc.pages[self.page].artboards[self.artboard]
                    .frame
                    .clone();
                let origin = self.canvas_bounds.center()
                    + self.pan
                    + Vec2::new(frame.x, frame.y) * self.zoom
                    - Vec2::new(frame.width, frame.height) * self.zoom / 2.0;
                let mut picked_board = None;
                for (ai, board_data) in self.doc.pages[self.page].artboards.iter().enumerate() {
                    if ai == self.artboard {
                        continue;
                    }
                    let other_origin = origin
                        + Vec2::new(
                            (ai as f32 - self.artboard as f32) * 720.0 + board_data.frame.x
                                - frame.x,
                            board_data.frame.y - frame.y,
                        ) * self.zoom;
                    let other_rect = Rect::from_min_size(
                        other_origin,
                        Vec2::new(board_data.frame.width, board_data.frame.height) * self.zoom,
                    );
                    paint_artboard(&painter, other_origin, self.zoom, board_data);
                    if !space
                        && response.clicked()
                        && response
                            .interact_pointer_pos()
                            .is_some_and(|p| other_rect.contains(p))
                    {
                        picked_board = Some(ai);
                    }
                }
                if let Some(ai) = picked_board {
                    self.pan +=
                        Vec2::new((ai as f32 - self.artboard as f32) * 720.0 * self.zoom, 0.0);
                    self.artboard = ai;
                    self.selected = None;
                    return;
                }

                let board =
                    Rect::from_min_size(origin, Vec2::new(frame.width, frame.height) * self.zoom);
                for spread in (1..=8).rev() {
                    painter.rect_filled(
                        board.expand(spread as f32).translate(Vec2::new(0.0, 4.0)),
                        3.0,
                        Color32::from_black_alpha(2),
                    );
                }
                painter.rect_filled(
                    board,
                    (frame.radius * self.zoom).min(255.0) as u8,
                    Color32::from_rgb(
                        frame.background[0],
                        frame.background[1],
                        frame.background[2],
                    ),
                );
                painter.text(
                    origin - Vec2::new(0.0, 20.0),
                    egui::Align2::LEFT_CENTER,
                    &self.doc.pages[self.page].artboards[self.artboard].name,
                    egui::FontId::proportional(12.0),
                    Color32::GRAY,
                );
                let content_area = if self.rulers {
                    Rect::from_min_max(area.min + Vec2::splat(20.0), area.max)
                } else {
                    area
                };
                let clipped = painter.with_clip_rect(if frame.clip {
                    board.intersect(content_area)
                } else {
                    content_area
                });
                if self.rulers {
                    paint_rulers(&painter, area, origin, self.zoom);
                }
                let canvas_zoom = self.zoom;
                let rect_for = |l: &Layer| {
                    Rect::from_min_size(
                        origin + Vec2::new(l.x, l.y) * canvas_zoom,
                        Vec2::new(l.width, l.height) * canvas_zoom,
                    )
                };
                for (layer_index, layer) in self.doc.pages[self.page].artboards[self.artboard]
                    .layers
                    .iter()
                    .enumerate()
                {
                    if !layer.visible || layer.mask || self.editing_text == Some(layer_index) {
                        continue;
                    }
                    let rect = rect_for(layer);
                    if layer.path.is_some()
                        || layer.kind == Kind::Text
                        || layer.kind == Kind::Image
                        || layer.corners.is_some()
                        || layer.corner_style != "Rounded"
                        || layer.drop_shadow.is_some()
                        || layer.inner_shadow.is_some()
                        || !layer.effects.is_empty()
                        || active_mask(
                            layer,
                            &self.doc.pages[self.page].artboards[self.artboard].layers
                                [..layer_index],
                        )
                        .is_some()
                    {
                        paint_rendered_layer(
                            &clipped,
                            layer,
                            rect,
                            self.zoom,
                            &self.doc.pages[self.page].artboards[self.artboard].layers
                                [..layer_index],
                        );
                        continue;
                    }
                    let color = Color32::from_rgb(layer.color[0], layer.color[1], layer.color[2]);
                    match layer.kind {
                        Kind::Image => {}
                        Kind::Rectangle => {
                            clipped.rect_filled(
                                rect,
                                (layer.radius * self.zoom).min(255.0) as u8,
                                color,
                            );
                        }
                        Kind::Ellipse => {
                            clipped.add(egui::epaint::EllipseShape::filled(
                                rect.center(),
                                rect.size() / 2.0,
                                color,
                            ));
                        }
                        Kind::Text => {
                            clipped
                                .with_clip_rect(rect.intersect(board).intersect(area))
                                .text(
                                    rect.min,
                                    egui::Align2::LEFT_TOP,
                                    &layer.name,
                                    egui::FontId::new(
                                        (layer.height * 0.65).min(48.0) * self.zoom,
                                        fonts::editor_family(ctx, &layer.typography),
                                    ),
                                    color,
                                );
                        }
                    }
                }
                if self.grid {
                    for x in (0..=frame.width as usize).step_by(8) {
                        let x = origin.x + x as f32 * self.zoom;
                        clipped.line_segment(
                            [Pos2::new(x, board.top()), Pos2::new(x, board.bottom())],
                            Stroke::new(0.5_f32, Color32::from_black_alpha(14)),
                        );
                    }
                    for y in (0..=frame.height as usize).step_by(8) {
                        let y = origin.y + y as f32 * self.zoom;
                        clipped.line_segment(
                            [Pos2::new(board.left(), y), Pos2::new(board.right(), y)],
                            Stroke::new(0.5_f32, Color32::from_black_alpha(14)),
                        );
                    }
                }
                if self.selected.is_none() {
                    painter.rect_stroke(
                        board,
                        0.0,
                        Stroke::new(1.0_f32, ACCENT),
                        egui::StrokeKind::Outside,
                    );
                    for corner in corners(board) {
                        painter.rect_filled(
                            Rect::from_center_size(corner, Vec2::splat(5.0)),
                            0.0,
                            ACCENT,
                        );
                    }
                }
                for index in &self.selection {
                    if let Some(layer) = self.doc.pages[self.page].artboards[self.artboard]
                        .layers
                        .get(*index)
                    {
                        painter.rect_stroke(
                            rect_for(layer),
                            0.0,
                            Stroke::new(1.0_f32, ACCENT),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
                if self.pen || self.editing_path.is_some() {
                    self.path_editor(ui, ctx, &response, origin, board);
                    return;
                }
                let selected_rect = self
                    .affected_layers()
                    .into_iter()
                    .filter_map(|i| {
                        let l = &self.doc.pages[self.page].artboards[self.artboard].layers[i];
                        l.visible.then(|| rect_for(l))
                    })
                    .reduce(|a, b| a.union(b));
                let mut hovered_corner = None;
                if let Some(rect) = selected_rect {
                    painter.rect_stroke(
                        rect,
                        0.0,
                        Stroke::new(1.0_f32, ACCENT),
                        egui::StrokeKind::Outside,
                    );
                    for (corner, pos) in corners(rect).into_iter().enumerate() {
                        let handle = Rect::from_center_size(pos, Vec2::splat(7.0));
                        painter.rect_filled(handle, 1.0, Color32::WHITE);
                        painter.rect_stroke(
                            handle,
                            1.0,
                            Stroke::new(1.0_f32, ACCENT),
                            egui::StrokeKind::Outside,
                        );
                        if ctx
                            .pointer_hover_pos()
                            .is_some_and(|p| pos.distance(p) < 10.0)
                        {
                            hovered_corner = Some(corner);
                        }
                    }
                    let badge = Rect::from_center_size(
                        rect.center_bottom() + Vec2::new(0.0, 20.0),
                        Vec2::new(90.0, 22.0),
                    );
                    painter.rect_filled(badge, 5.0, ACCENT);
                    painter.text(
                        badge.center(),
                        egui::Align2::CENTER_CENTER,
                        format!(
                            "{:.0} × {:.0}",
                            rect.width() / self.zoom,
                            rect.height() / self.zoom
                        ),
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                }
                if let Some(corner) = hovered_corner {
                    ctx.set_cursor_icon(if corner == 0 || corner == 3 {
                        egui::CursorIcon::ResizeNwSe
                    } else {
                        egui::CursorIcon::ResizeNeSw
                    });
                }
                let pressed_corner = selected_rect.and_then(|rect| {
                    ctx.input(|i| i.pointer.press_origin())
                        .and_then(|pos| corners(rect).iter().position(|p| p.distance(pos) < 10.0))
                });
                if !space
                    && self.editing_text.is_none()
                    && (response.clicked() || response.drag_started())
                {
                    // Select the object under the press origin. The pointer may already
                    // have left a thin path by the time egui starts its drag gesture.
                    let press = if response.drag_started() {
                        ctx.input(|i| i.pointer.press_origin())
                    } else {
                        response.interact_pointer_pos()
                    };
                    if let Some(pos) = press {
                        let local = (pos - origin) / self.zoom;
                        if response.drag_started()
                            && self.tool.is_none()
                            && pressed_corner.is_some()
                        {
                            self.checkpoint();
                            self.group_drag = self
                                .affected_layers()
                                .into_iter()
                                .map(|i| {
                                    (
                                        i,
                                        self.doc.pages[self.page].artboards[self.artboard].layers
                                            [i]
                                            .clone(),
                                    )
                                })
                                .collect();
                            self.resize_corner = pressed_corner;
                            self.drag_origin = self.selected.map(|i| {
                                let mut original =
                                    self.doc.pages[self.page].artboards[self.artboard].layers[i]
                                        .clone();
                                if let Some(bounds) = selected_rect {
                                    original.x = (bounds.left() - origin.x) / self.zoom;
                                    original.y = (bounds.top() - origin.y) / self.zoom;
                                    original.width = bounds.width() / self.zoom;
                                    original.height = bounds.height() / self.zoom;
                                }
                                original
                            });
                        } else if board.contains(pos) && response.clicked() && self.tool.is_some() {
                            self.checkpoint();
                            self.doc.pages[self.page].artboards[self.artboard]
                                .layers
                                .push(Layer::new(self.tool.unwrap(), local.x, local.y));
                            self.selected = Some(
                                self.doc.pages[self.page].artboards[self.artboard]
                                    .layers
                                    .len()
                                    - 1,
                            );
                            self.tool = None;
                        } else if self.tool.is_none() {
                            let previous_selection = self.selected;
                            self.selected = self.doc.pages[self.page].artboards[self.artboard]
                                .layers
                                .iter()
                                .rposition(|l| {
                                    !l.locked
                                        && l.visible
                                        && Rect::from_min_size(
                                            Pos2::new(l.x, l.y),
                                            Vec2::new(l.width, l.height),
                                        )
                                        .contains(Pos2::new(local.x, local.y))
                                        && (l.path.is_none()
                                            || vector::hit(
                                                l,
                                                [local.x - l.x, local.y - l.y],
                                                4.0 / self.zoom,
                                            ))
                                });
                            if self.selected.is_none()
                                && response.drag_started()
                                && self.selected_group.is_some()
                                && selected_rect.is_some_and(|bounds| bounds.contains(pos))
                            {
                                self.selected = previous_selection;
                            }
                            let keep_group = response.drag_started()
                                && self.selected.is_some_and(|i| {
                                    self.selected_group.as_ref().is_some_and(|g| {
                                        self.doc.pages[self.page].artboards[self.artboard].layers[i]
                                            .group
                                            .as_ref()
                                            == Some(g)
                                    })
                                });
                            let keep_multi = response.drag_started()
                                && self.selected.is_some_and(|i| self.selection.contains(&i));
                            if !keep_group {
                                self.selected_group = None;
                            }
                            if ctx.input(|i| i.modifiers.shift) {
                                if let Some(i) = previous_selection {
                                    if !self.selection.contains(&i) {
                                        self.selection.push(i);
                                    }
                                }
                                if let Some(index) = self.selected {
                                    if !self.selection.contains(&index) {
                                        self.selection.push(index);
                                    }
                                }
                            } else if !keep_group && !keep_multi {
                                self.selection.clear();
                            }
                            if response.drag_started() {
                                if self.selected.is_none() {
                                    self.marquee_start = Some(pos);
                                    self.marquee_base = self.selection.clone();
                                    self.drag_origin = None;
                                    self.group_drag.clear();
                                } else {
                                    self.checkpoint();
                                }
                                self.group_drag = self
                                    .affected_layers()
                                    .into_iter()
                                    .map(|i| {
                                        (
                                            i,
                                            self.doc.pages[self.page].artboards[self.artboard]
                                                .layers[i]
                                                .clone(),
                                        )
                                    })
                                    .collect();
                                self.resize_corner = None;
                                self.drag_origin = self.selected.map(|i| {
                                    self.doc.pages[self.page].artboards[self.artboard].layers[i]
                                        .clone()
                                });
                            }
                        }
                    }
                }
                if response.dragged()
                    && !space
                    && self.editing_text.is_none()
                    && !ctx.data(|d| {
                        d.get_temp::<bool>(egui::Id::new("prototype_wire_dragging"))
                            .unwrap_or(false)
                    })
                {
                    if let (Some(_index), Some(original)) = (self.selected, &self.drag_origin) {
                        let delta = ctx.input(|i| {
                            i.pointer
                                .interact_pos()
                                .zip(i.pointer.press_origin())
                                .map(|(current, start)| current - start)
                                .unwrap_or(Vec2::ZERO)
                        }) / self.zoom;
                        let snap = self.snap && !ctx.input(|i| i.modifiers.alt);
                        let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                        if let Some(corner) = self.resize_corner {
                            let mut resized = original.clone();
                            resize_layer(&mut resized, original, corner, delta, snap);
                            let sx = resized.width / original.width;
                            let sy = resized.height / original.height;
                            for (i, start) in &self.group_drag {
                                let l = &mut layers[*i];
                                l.x = resized.x + (start.x - original.x) * sx;
                                l.y = resized.y + (start.y - original.y) * sy;
                                l.width = start.width * sx;
                                l.height = start.height * sy;
                            }
                        } else {
                            for (i, start) in &self.group_drag {
                                layers[*i].x = snap_value(start.x + delta.x, snap);
                                layers[*i].y = snap_value(start.y + delta.y, snap);
                            }
                        }
                        self.status = "Editing · hold Alt for precise movement".into();
                    }
                }
                if let Some(start) = self.marquee_start {
                    if let Some(end) = ctx.pointer_hover_pos() {
                        let bounds = Rect::from_two_pos(start, end);
                        painter.rect_filled(bounds, 0.0, ACCENT.gamma_multiply(0.12));
                        painter.rect_stroke(
                            bounds,
                            0.0,
                            Stroke::new(1.0_f32, ACCENT),
                            egui::StrokeKind::Inside,
                        );
                        self.selection = self.marquee_base.clone();
                        for (i, layer) in self.doc.pages[self.page].artboards[self.artboard]
                            .layers
                            .iter()
                            .enumerate()
                        {
                            if layer.visible
                                && !layer.locked
                                && bounds.intersects(rect_for(layer))
                                && !self.selection.contains(&i)
                            {
                                self.selection.push(i);
                            }
                        }
                        self.selected = self.selection.last().copied();
                        self.selected_group = None;
                        self.status = format!("{} layers selected", self.selection.len());
                    }
                    if response.drag_stopped() || !ctx.input(|i| i.pointer.primary_down()) {
                        self.marquee_start = None;
                        self.marquee_base.clear();
                    }
                }
                // Text layers remain movable objects until editing is entered.
                // The TextEdit widget below supplies the I-beam only while editing.
                if response.double_clicked() {
                    if self.selected.is_some_and(|i| {
                        self.doc.pages[self.page].artboards[self.artboard].layers[i]
                            .path
                            .is_some()
                    }) {
                        self.apply_action(Action::EditPath, ctx);
                    } else {
                        self.apply_action(Action::EditText, ctx);
                    }
                }
                if let Some(index) = self.editing_text {
                    if let Some(layer) = self.doc.pages[self.page].artboards[self.artboard]
                        .layers
                        .get_mut(index)
                    {
                        let rect = Rect::from_min_size(
                            origin + Vec2::new(layer.x, layer.y) * self.zoom,
                            Vec2::new(layer.width, layer.height) * self.zoom,
                        );
                        let family = fonts::editor_family(ctx, &layer.typography);
                        let font_id = egui::FontId::new(
                            layer
                                .typography
                                .size
                                .unwrap_or((layer.height * 0.65).min(48.0))
                                * self.zoom,
                            family,
                        );
                        let format = egui::TextFormat {
                            font_id: font_id.clone(),
                            color: Color32::from_rgb(
                                layer.color[0],
                                layer.color[1],
                                layer.color[2],
                            ),
                            extra_letter_spacing: layer.typography.tracking * self.zoom,
                            line_height: layer.typography.line_height.map(|v| v * self.zoom),
                            italics: fonts::shaping_data(&layer.typography).2,
                            ..Default::default()
                        };
                        let alignment = match layer.typography.align {
                            1 => egui::Align::Center,
                            2 => egui::Align::Max,
                            _ => egui::Align::Min,
                        };
                        let mut layouter = |ui: &egui::Ui, text: &str, wrap: f32| {
                            let mut job = egui::text::LayoutJob::default();
                            job.append(text, 0.0, format.clone());
                            job.wrap.max_width = wrap;
                            job.halign = alignment;
                            ui.fonts(|f| f.layout_job(job))
                        };
                        let id = egui::Id::new("canvas_text_editor");
                        let editor = ui.put(
                            rect,
                            egui::TextEdit::multiline(&mut layer.name)
                                .id(id)
                                .frame(false)
                                .margin(0.0)
                                .font(font_id)
                                .layouter(&mut layouter)
                                .text_color(Color32::from_rgb(
                                    layer.color[0],
                                    layer.color[1],
                                    layer.color[2],
                                ))
                                .desired_width(rect.width()),
                        );
                        if response.double_clicked() {
                            editor.request_focus();
                        }
                        if ctx.input(|i| i.key_pressed(egui::Key::Escape))
                            || (editor.lost_focus() && !response.double_clicked())
                        {
                            self.editing_text = None;
                        }
                    } else {
                        self.editing_text = None;
                    }
                }
                if response.secondary_clicked() {
                    self.selected_group = None;
                    self.selection.clear();
                    if let Some(pos) = response.interact_pointer_pos() {
                        let local = (pos - origin) / self.zoom;
                        self.selected = self.doc.pages[self.page].artboards[self.artboard]
                            .layers
                            .iter()
                            .rposition(|l| {
                                l.visible
                                    && Rect::from_min_size(
                                        Pos2::new(l.x, l.y),
                                        Vec2::new(l.width, l.height),
                                    )
                                    .contains(Pos2::new(local.x, local.y))
                                    && (l.path.is_none()
                                        || vector::hit(
                                            l,
                                            [local.x - l.x, local.y - l.y],
                                            4.0 / self.zoom,
                                        ))
                            });
                    }
                }
                response.context_menu(|ui| self.layer_context_menu(ui, ctx));
                if response.drag_stopped() {
                    self.group_drag.clear();
                    self.drag_origin = None;
                    self.resize_corner = None;
                }
            });
    }
}

impl eframe::App for Studio {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        #[cfg(target_os = "linux")]
        {
            raw.modifiers.command = raw.modifiers.mac_cmd;
            let mut clipboard = None;
            for event in &mut raw.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    modifiers.command = modifiers.mac_cmd;
                    if modifiers.command {
                        match key {
                            egui::Key::C => clipboard = Some(egui::Event::Copy),
                            egui::Key::X => clipboard = Some(egui::Event::Cut),
                            egui::Key::V if ctx.wants_keyboard_input() => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste)
                            }
                            _ => {}
                        }
                    }
                }
            }
            if let Some(event) = clipboard {
                if ctx.wants_keyboard_input() {
                    raw.events.push(event);
                }
            }
        }
        #[cfg(not(target_os = "linux"))]
        let _ = (ctx, raw);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_secs(5));
        let mut used: Vec<String> = self
            .doc
            .pages
            .iter()
            .flat_map(|p| &p.artboards)
            .flat_map(|b| &b.layers)
            .filter(|l| l.kind == Kind::Text)
            .map(|l| {
                l.typography.family.clone().unwrap_or_else(|| {
                    if l.typography.monospace {
                        "Hack".into()
                    } else {
                        "Ubuntu".into()
                    }
                })
            })
            .collect();
        used.sort();
        used.dedup();
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("document_fonts"), used));
        #[cfg(target_os = "macos")]
        {
            let mut actions = Vec::new();
            let mut recent_actions = Vec::new();
            if let Some(menu) = &mut self.native_menu {
                menu.sync_recent(&self.recent);
            }
            if let Some(menu) = &self.native_menu {
                while let Ok(id) = menu.events.try_recv() {
                    if let Some((_, action)) = menu.items.iter().find(|(item, _)| item.id() == &id)
                    {
                        actions.push(*action);
                    } else if let Some((_, path)) =
                        menu.recent_items.iter().find(|(item, _)| item.id() == &id)
                    {
                        recent_actions.push(path.clone());
                    }
                }
                for (item, action) in &menu.items {
                    item.set_enabled(self.action_enabled(*action, ctx));
                }
            }
            for action in actions {
                self.apply_action(action, ctx);
            }
            for path in recent_actions {
                if let Some(path) = path {
                    self.open_path(path);
                } else {
                    self.recent.clear();
                }
            }
        }
        for path in ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect::<Vec<_>>()
        }) {
            if path
                .extension()
                .is_some_and(|e| e == "suiza-ui" || e == "json")
            {
                self.open_path(path);
            } else {
                self.status = "Drop a .suiza-ui or JSON document to open it".into();
            }
        }
        self.apply_theme(ctx);
        self.show(ctx);
        if let Some(palette) =
            ctx.data(|d| d.get_temp::<Vec<[u8; 3]>>(egui::Id::new("suiza_swatches")))
        {
            self.swatches = palette;
        }
        if !self.workbench_ui.open && !self.plugins_ui.open && !self.welcome {
            self.show_floating_tools(ctx);
        }
        let before = self.doc.clone();
        if !self.workbench_ui.open && !self.plugins_ui.open && !self.welcome {
            if let Some(error) = self.prototype_ui.show(
                ctx,
                self.canvas_bounds,
                &mut self.doc,
                self.page,
                self.artboard,
                self.selected,
                self.pan,
                self.zoom,
            ) {
                self.status = error;
            }
        }
        if before != self.doc {
            self.record_change(before);
        }
        if let Some(command) = self
            .workbench_ui
            .show(ctx, &self.doc, self.page, self.artboard)
        {
            self.apply_workbench(command);
        }
        if let Some((plugin, command)) = self.plugins_ui.show(ctx) {
            let mut selection = self.selection.clone();
            if let Some(index) = self.selected {
                selection.push(index);
            }
            match self.plugins_ui.installed[plugin].run(
                command,
                &self.doc,
                self.page,
                self.artboard,
                &selection,
            ) {
                Ok(next) => {
                    if next != self.doc {
                        let before = std::mem::replace(&mut self.doc, next);
                        self.record_change(before);
                    }
                    self.status = "Plugin completed. Undo is available.".into();
                }
                Err(e) => self.status = format!("Plugin: {e}"),
            }
        }
        if self.welcome {
            self.show_welcome(ctx);
        }
    }
    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(json) = serde_json::to_string(&self.plugins_ui.installed) {
            storage.set_string("suiza_plugins_v1", json);
        }
        if let Ok(json) = self.session_json() {
            storage.set_string("suiza_session_v1", json);
        }
        if let Ok(json) = serde_json::to_string(&self.swatches) {
            storage.set_string("suiza_swatches", json);
        }
        if let Ok(json) = serde_json::to_string(&self.theme) {
            storage.set_string("suiza_theme", json);
        }
        if let Ok(json) = serde_json::to_string(&self.assets_ui.libraries) {
            storage.set_string("suiza_libraries_v1", json);
        }
        if let Ok(json) = serde_json::to_string(&self.recent) {
            storage.set_string("suiza_recent", json);
        }
    }
}

#[derive(Clone, Copy)]
enum Action {
    Plugins,
    Workbench,
    Prototype,
    SharePrototype,
    PreviewPrototype,
    New,
    Open,
    Save,
    SaveAs,
    Quit,
    BoardDuplicate,
    BoardCopy,
    BoardCut,
    BoardPaste,
    BoardDelete,
    BoardRename,
    RenameLayer,
    BoardExport,
    BoardLock,
    BoardHide,
    Undo,
    Redo,
    CloseTab,
    NextTab(i32),
    Duplicate,
    DuplicateBehind,
    CopyLayer,
    CutLayer,
    PasteLayer,
    Bold,
    Italic,
    Underline,
    Strike,
    Typeface,
    FontSize(f32),
    Kern(bool),
    Tracking(f32),
    Ligatures(bool),
    Baseline(i8),
    TextCase(u8),
    List(u8),
    Direction(u8),
    OpenType(u8),
    Vertical(usize),
    EditText,
    Pen,
    EditPath,
    ConvertPath,
    Boolean(u8),
    Compound,
    TextAlign(usize),
    Mask,
    ImportImage,
    ReplaceImage,
    OriginalSize,
    TrimImage,
    ExportSelected,
    Delete,
    AddPage,
    AddBoard,
    Tool(Option<Kind>),
    Lock,
    Visible,
    Group,
    Ungroup,
    Front,
    Back,
    Align(usize),
    ToggleGrid,
    ToggleRulers,
    Welcome,
    Help,
    Theme(ThemeMode),
    Zoom(f32),
    Fit,
    Page(i32),
    Board(i32),
}
impl Studio {
    fn action_enabled(&self, action: Action, ctx: &egui::Context) -> bool {
        let layer = self.selected.and_then(|i| {
            self.doc.pages[self.page].artboards[self.artboard]
                .layers
                .get(i)
        });
        let editable = layer.is_some_and(|l| !l.locked);
        match action {
            Action::EditPath => editable && layer.is_some_and(|l| l.path.is_some()),
            Action::ConvertPath => editable && layer.is_some_and(|l| l.kind != Kind::Image),
            Action::Boolean(_) | Action::Compound => self.selection.len() >= 2 && editable,
            Action::Undo => !self.history.is_empty() || ctx.wants_keyboard_input(),
            Action::Redo => !self.redo.is_empty() || ctx.wants_keyboard_input(),
            Action::CopyLayer => layer.is_some() || ctx.wants_keyboard_input(),
            Action::PasteLayer => self.layer_clipboard.is_some() || ctx.wants_keyboard_input(),
            Action::Lock | Action::Visible => layer.is_some(),
            Action::Bold
            | Action::Italic
            | Action::Underline
            | Action::Strike
            | Action::Typeface
            | Action::FontSize(_)
            | Action::Kern(_)
            | Action::Tracking(_)
            | Action::Ligatures(_)
            | Action::Baseline(_)
            | Action::TextCase(_)
            | Action::List(_)
            | Action::Direction(_)
            | Action::OpenType(_)
            | Action::Vertical(_)
            | Action::TextAlign(_)
            | Action::EditText => editable && layer.is_some_and(|l| l.kind == Kind::Text),
            Action::Mask => {
                editable && layer.is_some_and(|l| matches!(l.kind, Kind::Rectangle | Kind::Ellipse))
            }
            Action::ReplaceImage | Action::OriginalSize | Action::TrimImage => {
                editable && layer.is_some_and(|l| l.kind == Kind::Image)
            }
            Action::RenameLayer
            | Action::Delete
            | Action::Duplicate
            | Action::DuplicateBehind
            | Action::CutLayer
            | Action::Front
            | Action::Back
            | Action::Align(_)
            | Action::Group
            | Action::Ungroup
            | Action::ExportSelected => editable,
            _ => true,
        }
    }
    fn board_context_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.set_min_width(210.0);
        section_heading(ui, "Frame");
        let mut action = None;
        for (label, a) in [
            ("Rename…", Action::BoardRename),
            ("Cut", Action::BoardCut),
            ("Copy", Action::BoardCopy),
            ("Paste frame", Action::BoardPaste),
            ("Duplicate", Action::BoardDuplicate),
            ("Lock / unlock contents", Action::BoardLock),
            ("Show / hide contents", Action::BoardHide),
            ("Fit frame", Action::Fit),
            ("Export frame…", Action::BoardExport),
            ("Delete frame", Action::BoardDelete),
        ] {
            if ui
                .add_enabled(
                    !matches!(a, Action::BoardPaste) || self.board_clipboard.is_some(),
                    egui::Button::new(label),
                )
                .clicked()
            {
                action = Some(a);
            }
        }
        ui.separator();
        ui.menu_button("Export format", |ui| {
            for format in ["PNG", "JPG", "TIFF", "WEBP", "PDF", "SVG"] {
                ui.selectable_value(&mut self.export_format, format.into(), format);
            }
        });
        if let Some(a) = action {
            self.apply_action(a, ctx);
            ui.close_menu();
        }
    }
    fn layer_context_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.selected.is_none() {
            self.board_context_menu(ui, ctx);
            return;
        }

        ui.set_min_width(210.0);
        ui.spacing_mut().item_spacing.y = 4.0;
        let selected = self
            .selected
            .and_then(|i| {
                self.doc.pages[self.page].artboards[self.artboard]
                    .layers
                    .get(i)
            })
            .cloned();
        let editable = selected.as_ref().is_some_and(|l| !l.locked);
        let mut action = None;
        if selected.as_ref().is_some_and(|l| l.path.is_some())
            && ui.button("Edit vector path").clicked()
        {
            action = Some(Action::EditPath);
        }
        if editable && selected.as_ref().is_some_and(|l| l.kind != Kind::Image) {
            if ui.button("Convert to outlines").clicked() {
                action = Some(Action::ConvertPath);
            }
            ui.menu_button("Boolean operations", |ui| {
                for (i, label) in ["Union", "Subtract", "Intersect", "Exclude"]
                    .iter()
                    .enumerate()
                {
                    if ui.button(*label).clicked() {
                        action = Some(Action::Boolean(i as u8));
                    }
                }
                if ui.button("Make compound path").clicked() {
                    action = Some(Action::Compound);
                }
            });
        }
        ui.menu_button("Select", |ui| {
            for (i, l) in self.doc.pages[self.page].artboards[self.artboard]
                .layers
                .iter()
                .enumerate()
                .rev()
            {
                if ui
                    .selectable_label(self.selected == Some(i), &l.name)
                    .clicked()
                {
                    self.selected = Some(i);
                    ui.close_menu();
                }
            }
        });
        ui.separator();
        for (label, a, enabled) in [
            ("Cut", Action::CutLayer, editable),
            ("Copy layer", Action::CopyLayer, selected.is_some()),
            (
                "Paste layer",
                Action::PasteLayer,
                self.layer_clipboard.is_some(),
            ),
            ("Duplicate", Action::Duplicate, editable),
            ("Duplicate behind", Action::DuplicateBehind, editable),
        ] {
            if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                action = Some(a);
            }
        }
        ui.separator();
        ui.add_enabled_ui(editable, |ui| {
            ui.menu_button("Group", |ui| {
                if ui.button("Group selected objects").clicked() {
                    action = Some(Action::Group);
                }
                if ui.button("Ungroup").clicked() {
                    action = Some(Action::Ungroup);
                }
            });
            ui.menu_button("Move", |ui| {
                if ui.button("Bring to front").clicked() {
                    action = Some(Action::Front);
                }
                if ui.button("Send to back").clicked() {
                    action = Some(Action::Back);
                }
            });
            ui.menu_button("Align to artboard", |ui| {
                for (i, label) in [
                    "Left",
                    "Horizontal center",
                    "Right",
                    "Top",
                    "Vertical center",
                    "Bottom",
                ]
                .iter()
                .enumerate()
                {
                    if ui.button(*label).clicked() {
                        action = Some(Action::Align(i));
                    }
                }
            });
        });
        if let Some(l) = selected.as_ref() {
            ui.separator();
            if ui
                .add_enabled(editable, egui::Button::new("Rename…"))
                .clicked()
            {
                action = Some(Action::RenameLayer);
            }
            if ui
                .button(if l.locked { "Unlock" } else { "Lock" })
                .clicked()
            {
                action = Some(Action::Lock);
            }
            if ui.button(if l.visible { "Hide" } else { "Show" }).clicked() {
                action = Some(Action::Visible);
            }
            if l.kind == Kind::Text {
                ui.separator();
                section_heading(ui, "Text");
                ui.add_enabled_ui(editable, |ui| {
                    if ui.button("Edit text").clicked() {
                        action = Some(Action::EditText);
                    }
                    if ui.selectable_label(l.typography.bold, "Bold").clicked() {
                        action = Some(Action::Bold);
                    }
                    ui.menu_button("Text alignment", |ui| {
                        for (i, label) in ["Left", "Center", "Right"].iter().enumerate() {
                            if ui
                                .selectable_label(l.typography.align == i, *label)
                                .clicked()
                            {
                                action = Some(Action::TextAlign(i));
                            }
                        }
                    });
                });
            } else {
                if l.kind == Kind::Image {
                    ui.separator();
                    section_heading(ui, "Image");
                    for (label, a) in [
                        ("Replace image…", Action::ReplaceImage),
                        ("Trim transparency", Action::TrimImage),
                        ("Size to original size", Action::OriginalSize),
                    ] {
                        if ui.add_enabled(editable, egui::Button::new(label)).clicked() {
                            action = Some(a);
                        }
                    }
                }
                ui.separator();
                if ui
                    .add_enabled(
                        editable && l.kind != Kind::Image,
                        egui::Button::new(if l.mask { "Remove mask" } else { "Use as mask" }),
                    )
                    .clicked()
                {
                    action = Some(Action::Mask);
                }
            }
            ui.separator();
            if ui
                .add_enabled(editable, egui::Button::new("Delete"))
                .clicked()
            {
                action = Some(Action::Delete);
            }
        }
        if let Some(a) = action {
            self.apply_action(a, ctx);
            ui.close_menu();
        }
    }
    fn affected_layers(&self) -> Vec<usize> {
        let layers = &self.doc.pages[self.page].artboards[self.artboard].layers;
        let group = self.selected_group.as_ref();
        layers
            .iter()
            .enumerate()
            .filter(|(i, l)| {
                !l.locked
                    && (self.selected == Some(*i)
                        || self.selection.contains(i)
                        || group.is_some_and(|g| l.group.as_ref() == Some(g)))
            })
            .map(|(i, _)| i)
            .collect()
    }
    fn apply_action(&mut self, action: Action, ctx: &egui::Context) {
        if self.selected_group.is_some() && matches!(action, Action::Lock | Action::Visible) {
            self.checkpoint();
            let group = self.selected_group.clone();
            let members: Vec<_> = self.doc.pages[self.page].artboards[self.artboard]
                .layers
                .iter_mut()
                .filter(|l| l.group == group)
                .collect();
            let value = if matches!(action, Action::Lock) {
                !members.iter().all(|l| l.locked)
            } else {
                !members.iter().all(|l| l.visible)
            };
            for layer in members {
                if matches!(action, Action::Lock) {
                    layer.locked = value;
                } else {
                    layer.visible = value;
                }
            }
            return;
        }
        let frame_size = Vec2::new(
            self.doc.pages[self.page].artboards[self.artboard]
                .frame
                .width,
            self.doc.pages[self.page].artboards[self.artboard]
                .frame
                .height,
        );
        let affected = if self.selected_group.is_none()
            && self.selected.is_some_and(|i| {
                self.doc.pages[self.page].artboards[self.artboard]
                    .layers
                    .get(i)
                    .is_some_and(|l| l.locked)
            }) {
            vec![]
        } else {
            self.affected_layers()
        };
        if !ctx.wants_keyboard_input() && matches!(action, Action::CopyLayer | Action::CutLayer) {
            self.clipboard_group.clear();
            if affected.len() > 1 {
                self.clipboard_group = affected
                    .iter()
                    .map(|i| self.doc.pages[self.page].artboards[self.artboard].layers[*i].clone())
                    .collect();
                self.layer_clipboard = self.clipboard_group.first().cloned();
                if matches!(action, Action::CutLayer) {
                    self.apply_action(Action::Delete, ctx);
                }
                return;
            }
        }
        if !ctx.wants_keyboard_input()
            && matches!(action, Action::PasteLayer)
            && self.clipboard_group.len() > 1
        {
            self.checkpoint();
            let copies = self.clipboard_group.clone();
            self.selection.clear();
            let group = format!("Pasted group {}", self.history.len());
            let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
            for mut l in copies {
                l.prototype_id.clear();
                l.x += 16.0;
                l.y += 16.0;
                l.group = Some(group.clone());
                l.locked = false;
                self.selection.push(layers.len());
                layers.push(l);
            }
            self.selected = self.selection.last().copied();
            self.selected_group = Some(group);
            return;
        }
        if affected.len() > 1
            && matches!(
                action,
                Action::Delete
                    | Action::Duplicate
                    | Action::DuplicateBehind
                    | Action::Front
                    | Action::Back
                    | Action::Align(_)
            )
        {
            self.checkpoint();
            let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
            let mut members: Vec<_> = affected.iter().map(|i| layers[*i].clone()).collect();
            if let Action::Align(a) = action {
                let min_x = members.iter().map(|l| l.x).fold(f32::INFINITY, f32::min);
                let min_y = members.iter().map(|l| l.y).fold(f32::INFINITY, f32::min);
                let max_x = members
                    .iter()
                    .map(|l| l.x + l.width)
                    .fold(f32::NEG_INFINITY, f32::max);
                let max_y = members
                    .iter()
                    .map(|l| l.y + l.height)
                    .fold(f32::NEG_INFINITY, f32::max);
                let dx = match a {
                    0 => -min_x,
                    1 => (frame_size.x - (max_x - min_x)) / 2.0 - min_x,
                    2 => frame_size.x - max_x,
                    _ => 0.0,
                };
                let dy = match a {
                    3 => -min_y,
                    4 => (frame_size.y - (max_y - min_y)) / 2.0 - min_y,
                    5 => frame_size.y - max_y,
                    _ => 0.0,
                };
                for i in affected {
                    layers[i].x += dx;
                    layers[i].y += dy;
                }
                return;
            }
            let duplicate = matches!(action, Action::Duplicate | Action::DuplicateBehind);
            if !duplicate {
                for i in affected.iter().rev() {
                    layers.remove(*i);
                }
            }
            if matches!(action, Action::Delete) {
                self.selected = None;
                self.selection.clear();
                self.editing_text = None;
                self.editing_path = None;
                return;
            }
            if duplicate {
                for l in &mut members {
                    l.prototype_id.clear();
                    if matches!(action, Action::Duplicate) {
                        l.x += 20.0;
                        l.y += 20.0;
                    }
                    l.name.push_str(" copy");
                    if let Some(g) = &mut l.group {
                        *g = format!("{} copy {}", g, self.history.len());
                    }
                }
            }
            let start = if matches!(action, Action::Back) {
                0
            } else if matches!(action, Action::DuplicateBehind) {
                affected[0]
            } else {
                layers.len()
            };
            let count = members.len();
            layers.splice(start..start, members);
            self.selection = (start..start + count).collect();
            self.selected = Some(start + count - 1);
            self.selected_group = if duplicate {
                layers[start].group.clone()
            } else {
                self.selected_group.clone()
            };
            return;
        }
        match action {
            Action::SaveAs => {
                let previous = self.document_path.take();
                self.save();
                if self.document_path.is_none() {
                    self.document_path = previous;
                }
            }
            Action::BoardCopy | Action::BoardCut => {
                self.board_clipboard =
                    Some(self.doc.pages[self.page].artboards[self.artboard].clone());
                if matches!(action, Action::BoardCut) {
                    self.apply_action(Action::BoardDelete, ctx);
                }
            }
            Action::BoardDuplicate | Action::BoardPaste => {
                let copy = if matches!(action, Action::BoardPaste) {
                    self.board_clipboard.clone()
                } else {
                    Some(self.doc.pages[self.page].artboards[self.artboard].clone())
                };
                if let Some(mut copy) = copy {
                    self.checkpoint();
                    copy.prototype_id.clear();
                    for layer in &mut copy.layers {
                        layer.prototype_id.clear();
                    }
                    copy.name.push_str(" copy");
                    self.doc.pages[self.page]
                        .artboards
                        .insert(self.artboard + 1, copy);
                    self.artboard += 1;
                    self.selected = None;
                    self.selection.clear();
                }
            }
            Action::BoardDelete => {
                self.checkpoint();
                let boards = &mut self.doc.pages[self.page].artboards;
                boards.remove(self.artboard);
                if boards.is_empty() {
                    boards.push(Artboard {
                        prototype_id: String::new(),
                        frame: FrameStyle::default(),
                        name: "Artboard 1".into(),
                        layers: vec![],
                    });
                }
                self.reset_selection();
            }
            Action::RenameLayer => {
                if let Some(group) = &self.selected_group {
                    ctx.data_mut(|d| {
                        d.insert_temp(
                            egui::Id::new(("group", self.page, self.artboard, group))
                                .with("rename"),
                            group.clone(),
                        )
                    });
                } else if let Some(i) = self.selected {
                    let layer = &self.doc.pages[self.page].artboards[self.artboard].layers[i];
                    ctx.data_mut(|d| {
                        d.insert_temp(
                            egui::Id::new(("inline_layer", self.artboard, i)),
                            layer.label.clone().unwrap_or_else(|| layer.name.clone()),
                        )
                    });
                }
            }
            Action::BoardRename => {
                ctx.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(("board", self.page, self.artboard)).with("rename"),
                        self.doc.pages[self.page].artboards[self.artboard]
                            .name
                            .clone(),
                    )
                });
            }
            Action::Plugins => self.plugins_ui.open = true,
            Action::Workbench => self.workbench_ui.open = true,
            Action::Prototype => self.prototype_ui.prototype(),
            Action::SharePrototype => self.prototype_ui.share(),
            Action::PreviewPrototype => self.prototype_ui.preview(),
            Action::Typeface => {
                self.font_chooser = true;
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("focus_font_search"), true));
            }
            Action::BoardLock | Action::BoardHide => {
                self.checkpoint();
                let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                let locked = layers.iter().all(|l| l.locked);
                let hidden = layers.iter().all(|l| !l.visible);
                for l in layers {
                    if matches!(action, Action::BoardLock) {
                        l.locked = !locked;
                    } else {
                        l.visible = hidden;
                    }
                }
            }
            Action::BoardExport => {
                let ext = self.export_format.to_lowercase();
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(&self.export_format, &[ext.as_str()])
                    .set_file_name(format!("frame.{ext}"))
                    .save_file()
                {
                    self.status = match export_board(
                        &self.doc.pages[self.page].artboards[self.artboard],
                        &path,
                        &self.export_format,
                        self.export_scale,
                    ) {
                        Ok(()) => format!("Exported {}", path.display()),
                        Err(e) => format!("Export failed: {e}"),
                    };
                }
            }
            Action::ImportImage | Action::ReplaceImage => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "tiff", "webp"])
                    .pick_file()
                {
                    match load_image_asset(&path) {
                        Ok((data, w, h)) => {
                            self.checkpoint();
                            let mut image = Layer::new(Kind::Image, 32.0, 32.0);
                            image.name = path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into();
                            let factor = (480.0 / w as f32).min(320.0 / h as f32).min(1.0);
                            image.width = w as f32 * factor;
                            image.height = h as f32 * factor;
                            image.image_data = Some(data);
                            let layers =
                                &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                            if matches!(action, Action::ReplaceImage) {
                                if let Some(i) = self.selected.filter(|i| {
                                    !layers[*i].locked && layers[*i].kind == Kind::Image
                                }) {
                                    layers[i].image_data = image.image_data;
                                    layers[i].name = image.name;
                                }
                            } else {
                                layers.push(image);
                                self.selected = Some(layers.len() - 1);
                            }
                        }
                        Err(e) => self.status = format!("Image import failed: {e}"),
                    }
                }
            }
            Action::ExportSelected => {
                if let Some(i) = self.selected {
                    let ext = self.export_format.to_lowercase();
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter(&self.export_format, &[ext.as_str()])
                        .set_file_name(format!("asset.{ext}"))
                        .save_file()
                    {
                        let layers = &self.doc.pages[self.page].artboards[self.artboard].layers;
                        self.status = match export_layer(
                            &layers[i],
                            &path,
                            &self.export_format,
                            self.export_scale,
                            self.export_trim,
                            &layers[..i],
                        ) {
                            Ok(()) => format!("Exported {}", path.display()),
                            Err(e) => format!("Export failed: {e}"),
                        };
                    }
                }
            }
            Action::CopyLayer | Action::CutLayer => {
                if ctx.wants_keyboard_input() {
                    ctx.input_mut(|i| {
                        i.events.push(if matches!(action, Action::CutLayer) {
                            egui::Event::Cut
                        } else {
                            egui::Event::Copy
                        })
                    });
                    return;
                }
                if let Some(index) = self.selected {
                    let layer =
                        self.doc.pages[self.page].artboards[self.artboard].layers[index].clone();
                    if layer.kind == Kind::Text {
                        ctx.copy_text(layer.name.clone());
                    }
                    self.layer_clipboard = Some(layer);
                    if matches!(action, Action::CutLayer) {
                        self.apply_action(Action::Delete, ctx);
                    }
                }
            }
            Action::PasteLayer => {
                if ctx.wants_keyboard_input() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                    return;
                }
                if let Some(mut layer) = self.layer_clipboard.clone() {
                    self.checkpoint();
                    layer.prototype_id.clear();
                    layer.x += 16.0;
                    layer.y += 16.0;
                    layer.locked = false;
                    let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                    layers.push(layer);
                    self.selected = Some(layers.len() - 1);
                    self.selection.clear();
                }
            }
            Action::EditText => {
                if let Some(i) = self.selected.filter(|i| {
                    self.doc.pages[self.page].artboards[self.artboard].layers[*i].kind == Kind::Text
                        && !self.doc.pages[self.page].artboards[self.artboard].layers[*i].locked
                }) {
                    self.checkpoint();
                    self.editing_text = Some(i);
                }
            }
            Action::Theme(theme) => {
                self.theme = theme;
                self.applied_dark = None;
                ctx.request_repaint();
            }
            Action::ToggleGrid => self.grid = !self.grid,
            Action::ToggleRulers => self.rulers = !self.rulers,
            Action::Welcome => self.welcome = true,
            Action::Help => {
                self.help = true;
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("help_opened_frame"), ctx.cumulative_pass_nr())
                });
            }
            Action::Open => self.open(),
            Action::Save => self.save(),
            Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Action::CloseTab => {
                self.ensure_tabs();
                if self.doc != self.saved_document {
                    self.closing_tab = Some(self.active_tab);
                } else {
                    self.close_tab(self.active_tab);
                }
            }
            Action::NextTab(delta) => {
                self.ensure_tabs();
                let index =
                    (self.active_tab as i32 + delta).rem_euclid(self.tabs.len() as i32) as usize;
                self.switch_tab(index);
            }
            Action::Redo => {
                if ctx.wants_keyboard_input() {
                    let mut modifiers = egui::Modifiers::COMMAND;
                    modifiers.shift = true;
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::Key {
                            key: egui::Key::Z,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers,
                        })
                    });
                } else if let Some(doc) = self.redo.pop() {
                    self.history.push(self.doc.clone());
                    self.doc = doc;
                    self.reset_selection();
                }
            }
            Action::Undo => {
                if ctx.wants_keyboard_input() {
                    ctx.input_mut(|i| {
                        i.events.push(egui::Event::Key {
                            key: egui::Key::Z,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::COMMAND,
                        })
                    });
                } else {
                    self.undo();
                }
            }
            Action::ConvertPath => {
                if let Some(i) = self.selected.filter(|i| {
                    !self.doc.pages[self.page].artboards[self.artboard].layers[*i].locked
                }) {
                    let original =
                        self.doc.pages[self.page].artboards[self.artboard].layers[i].clone();
                    if let Some(path) = vector::from_layer(&original) {
                        self.checkpoint();
                        let layer =
                            &mut self.doc.pages[self.page].artboards[self.artboard].layers[i];
                        layer.path = Some(path);
                        layer.kind = Kind::Rectangle;
                        layer.name.push_str(" outlines");
                        layer.radius = 0.0;
                        layer.corners = None;
                        self.editing_path = Some(i);
                        self.editing_text = None;
                        self.pen = false;
                    }
                }
            }
            Action::Boolean(_) | Action::Compound => {
                let mut indices = self.selection.clone();
                if let Some(i) = self.selected {
                    indices.push(i);
                }
                indices.sort_unstable();
                indices.dedup();
                let layers = &self.doc.pages[self.page].artboards[self.artboard].layers;
                indices.retain(|i| {
                    *i < layers.len()
                        && !layers[*i].locked
                        && vector::from_layer(&layers[*i]).is_some()
                });
                if indices.len() >= 2 {
                    let originals: Vec<_> = indices.iter().map(|i| layers[*i].clone()).collect();
                    let operation = if let Action::Boolean(operation) = action {
                        operation
                    } else {
                        4
                    };
                    match vector::combine(&originals, operation) {
                        Ok(mut result) => {
                            self.checkpoint();
                            let layers =
                                &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                            let insertion = indices[0];
                            for i in indices.iter().rev() {
                                layers.remove(*i);
                            }
                            result.group = None;
                            layers.insert(insertion, result);
                            self.selected = Some(insertion);
                            self.selection.clear();
                            self.editing_path = None;
                            self.editing_text = None;
                        }
                        Err(error) => self.status = error,
                    }
                } else {
                    self.status = "Select at least two vector shapes with Shift".into();
                }
            }
            Action::Tool(tool) => {
                self.tool = tool;
                self.pen = false;
                self.editing_path = None;
            }
            Action::Pen => {
                self.pen = true;
                self.editing_path = None;
                self.tool = None;
                self.selected = None;
                self.selection.clear();
            }
            Action::EditPath => {
                if let Some(i) = self.selected.filter(|i| {
                    self.doc.pages[self.page].artboards[self.artboard].layers[*i]
                        .path
                        .is_some()
                }) {
                    self.editing_path = Some(i);
                    self.pen = false;
                    self.editing_text = None;
                }
            }
            Action::Zoom(zoom) => self.zoom = zoom,
            Action::Fit => {
                let frame = &self.doc.pages[self.page].artboards[self.artboard].frame;
                self.zoom = ((ctx.screen_rect().width() - 520.0) / frame.width)
                    .min((ctx.screen_rect().height() - 160.0) / frame.height)
                    .clamp(0.25, 2.0);
                self.pan = Vec2::ZERO;
            }
            Action::Page(delta) => {
                self.page =
                    (self.page as i32 + delta).clamp(0, self.doc.pages.len() as i32 - 1) as usize;
                self.artboard = 0;
                self.reset_selection();
            }
            Action::Board(delta) => {
                self.artboard = (self.artboard as i32 + delta)
                    .clamp(0, self.doc.pages[self.page].artboards.len() as i32 - 1)
                    as usize;
                self.reset_selection();
            }
            Action::New => {
                self.open_in_tab(
                    Document {
                        prototype: prototype::Prototype::default(),
                        checkpoints: vec![],
                        name: "Untitled".into(),
                        pages: vec![Page {
                            name: "Page 1".into(),
                            artboards: vec![Artboard {
                                prototype_id: String::new(),
                                frame: FrameStyle::default(),
                                name: "Artboard 1".into(),
                                layers: vec![],
                            }],
                        }],
                    },
                    None,
                );
            }
            Action::AddPage => {
                self.checkpoint();
                let n = self.doc.pages.len() + 1;
                self.doc.pages.push(Page {
                    name: format!("Page {n}"),
                    artboards: vec![Artboard {
                        prototype_id: String::new(),
                        frame: FrameStyle::default(),
                        name: "Artboard 1".into(),
                        layers: vec![],
                    }],
                });
                self.page = n - 1;
                self.artboard = 0;
                self.reset_selection();
            }
            Action::AddBoard => {
                self.checkpoint();
                let boards = &mut self.doc.pages[self.page].artboards;
                let n = boards.len() + 1;
                boards.push(Artboard {
                    prototype_id: String::new(),
                    frame: FrameStyle::default(),
                    name: format!("Artboard {n}"),
                    layers: vec![],
                });
                self.artboard = n - 1;
                self.reset_selection();
            }
            other => {
                let Some(index) = self.selected else { return };
                let locked =
                    self.doc.pages[self.page].artboards[self.artboard].layers[index].locked;
                if locked && !matches!(other, Action::Lock | Action::Visible) {
                    return;
                }
                self.checkpoint();
                let layers = &mut self.doc.pages[self.page].artboards[self.artboard].layers;
                match other {
                    Action::Duplicate => {
                        let mut copy = layers[index].clone();
                        copy.prototype_id.clear();
                        copy.name.push_str(" copy");
                        copy.x += 20.0;
                        copy.y += 20.0;
                        layers.push(copy);
                        self.selected = Some(layers.len() - 1);
                    }
                    Action::OriginalSize => {
                        if let Some(data) = &layers[index].image_data {
                            if let Ok(img) = image::load_from_memory(data) {
                                layers[index].width = img.width() as f32;
                                layers[index].height = img.height() as f32;
                            }
                        }
                    }
                    Action::TrimImage => {
                        let l = &mut layers[index];
                        if let Some(data) = &l.image_data {
                            if let Ok(img) = image::load_from_memory(data) {
                                let img = img.to_rgba8();
                                let mut min = (img.width(), img.height());
                                let mut max = (0, 0);
                                let mut found = false;
                                for (x, y, p) in img.enumerate_pixels().filter(|(_, _, p)| p[3] > 0)
                                {
                                    let _ = p;
                                    found = true;
                                    min.0 = min.0.min(x);
                                    min.1 = min.1.min(y);
                                    max.0 = max.0.max(x);
                                    max.1 = max.1.max(y);
                                }
                                if found {
                                    let width = max.0 - min.0 + 1;
                                    let height = max.1 - min.1 + 1;
                                    let mut out = std::io::Cursor::new(Vec::new());
                                    if image::DynamicImage::ImageRgba8(
                                        image::imageops::crop_imm(
                                            &img, min.0, min.1, width, height,
                                        )
                                        .to_image(),
                                    )
                                    .write_to(&mut out, image::ImageFormat::Png)
                                    .is_ok()
                                    {
                                        l.x += l.width * min.0 as f32 / img.width() as f32;
                                        l.y += l.height * min.1 as f32 / img.height() as f32;
                                        l.width *= width as f32 / img.width() as f32;
                                        l.height *= height as f32 / img.height() as f32;
                                        l.image_data = Some(out.into_inner());
                                    }
                                }
                            }
                        }
                    }
                    Action::DuplicateBehind => {
                        let mut copy = layers[index].clone();
                        copy.prototype_id.clear();
                        copy.name.push_str(" copy");
                        layers.insert(index, copy);
                        self.selected = Some(index);
                    }
                    Action::Bold => layers[index].typography.bold = !layers[index].typography.bold,
                    Action::Italic => {
                        layers[index].typography.italic = !layers[index].typography.italic
                    }
                    Action::Underline => {
                        layers[index].typography.underline = !layers[index].typography.underline
                    }
                    Action::Strike => {
                        layers[index].typography.strike = !layers[index].typography.strike
                    }
                    Action::FontSize(delta) => {
                        let l = &mut layers[index];
                        l.typography.size = Some(
                            (l.typography.size.unwrap_or((l.height * 0.65).min(48.0)) + delta)
                                .clamp(1.0, 240.0),
                        );
                    }
                    Action::Kern(on) => layers[index].typography.kerning = Some(on),
                    Action::Tracking(delta) => {
                        let t = &mut layers[index].typography;
                        t.tracking = (t.tracking + delta).clamp(-10.0, 40.0);
                    }
                    Action::Ligatures(on) => layers[index].typography.ligatures = Some(on),
                    Action::Baseline(mode) => {
                        let t = &mut layers[index].typography;
                        t.baseline = match mode {
                            0 => 0.0,
                            1 => t.baseline + 1.0,
                            2 => t.baseline - 1.0,
                            3 => 6.0,
                            _ => -6.0,
                        };
                    }
                    Action::TextCase(mode) => layers[index].typography.transformation = mode,
                    Action::List(mode) => layers[index].typography.list = mode,
                    Action::Direction(mode) => layers[index].typography.direction = mode,
                    Action::Vertical(mode) => layers[index].typography.vertical = mode,
                    Action::OpenType(feature) => {
                        let t = &mut layers[index].typography;
                        let value = match feature {
                            0 => &mut t.small_caps,
                            1 => &mut t.tabular,
                            2 => &mut t.slashed_zero,
                            _ => &mut t.discretionary,
                        };
                        *value = !*value;
                    }
                    Action::TextAlign(a) => layers[index].typography.align = a,
                    Action::Mask => {
                        if matches!(layers[index].kind, Kind::Rectangle | Kind::Ellipse) {
                            layers[index].mask = !layers[index].mask;
                            if layers[index].mask {
                                let target = self
                                    .selection
                                    .iter()
                                    .copied()
                                    .filter(|i| *i != index && *i < layers.len())
                                    .min()
                                    .or_else(|| index.checked_sub(1));
                                if let Some(target) = target {
                                    layers[index].group = layers[target].group.clone();
                                    if target < index {
                                        let mask = layers.remove(index);
                                        layers.insert(target, mask);
                                        self.selected = Some(target);
                                    }
                                }
                            }
                        }
                        self.selection.clear();
                    }
                    Action::Delete => {
                        layers.remove(index);
                        self.selected = None;
                        self.selection.clear();
                        self.editing_text = None;
                    }
                    Action::Lock => layers[index].locked = !layers[index].locked,
                    Action::Visible => layers[index].visible = !layers[index].visible,
                    Action::Group => {
                        let group = next_group_name(layers);
                        for i in self.selection.iter().copied().chain(std::iter::once(index)) {
                            if let Some(layer) = layers.get_mut(i).filter(|l| !l.locked) {
                                layer.group = Some(group.clone());
                            }
                        }
                        self.selected_group = Some(group);
                    }
                    Action::Ungroup => {
                        self.selected_group = None;
                        let group = layers[index].group.clone();
                        for layer in layers
                            .iter_mut()
                            .filter(|l| group.is_some() && l.group == group)
                        {
                            layer.group = None;
                        }
                    }
                    Action::Front => {
                        let layer = layers.remove(index);
                        layers.push(layer);
                        self.selected = Some(layers.len() - 1);
                    }
                    Action::Back => {
                        let layer = layers.remove(index);
                        layers.insert(0, layer);
                        self.selected = Some(0);
                    }
                    Action::Align(a) => {
                        let l = &mut layers[index];
                        match a {
                            0 => l.x = 0.0,
                            1 => l.x = (frame_size.x - l.width) / 2.0,
                            2 => l.x = frame_size.x - l.width,
                            3 => l.y = 0.0,
                            4 => l.y = (frame_size.y - l.height) / 2.0,
                            _ => l.y = frame_size.y - l.height,
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    fn menus(&mut self, ctx: &egui::Context) {
        let mut action = None;
        let mut recent_action = None;
        let mut clear_recent = false;
        if !cfg!(target_os = "macos") {
            egui::TopBottomPanel::top("menus")
                .exact_height(30.0)
                .show(ctx, |ui| {
                    egui::menu::bar(ui, |ui| {
                        for (name, commands) in menu_groups() {
                            ui.menu_button(name, |ui| {
                                for (i, (title, commands)) in
                                    menu_sections(name, commands).into_iter().enumerate()
                                {
                                    if i > 0 {
                                        ui.separator();
                                    }
                                    let mut draw = |ui: &mut egui::Ui| {
                                        for (label, a, key) in commands.iter().copied() {
                                            menu_item(
                                                ui,
                                                label,
                                                &key.unwrap_or("").replace("Cmd", shortcut_name()),
                                                self.action_enabled(a, ctx),
                                                a,
                                                &mut action,
                                            );
                                            if matches!(a, Action::Open) {
                                                ui.menu_button("Open Recent", |ui| {
                                                    if self.recent.is_empty() {
                                                        ui.add_enabled(
                                                            false,
                                                            egui::Button::new(
                                                                "No Recent Documents",
                                                            ),
                                                        );
                                                    }
                                                    for path in &self.recent {
                                                        if ui
                                                            .button(path.display().to_string())
                                                            .clicked()
                                                        {
                                                            recent_action = Some(path.clone());
                                                            ui.close_menu();
                                                        }
                                                    }
                                                    if !self.recent.is_empty() {
                                                        ui.separator();
                                                        if ui
                                                            .button("Clear Recent Documents")
                                                            .clicked()
                                                        {
                                                            clear_recent = true;
                                                            ui.close_menu();
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    };
                                    if let Some(title) = title {
                                        ui.menu_button(title, draw);
                                    } else {
                                        draw(ui);
                                    }
                                }
                            });
                        }
                    });
                });
        }
        if let Some(action) = action {
            self.apply_action(action, ctx);
        }
        if let Some(path) = recent_action {
            self.open_path(path);
        }
        if clear_recent {
            self.recent.clear();
        }
        for (_, commands) in menu_groups() {
            for (_, a, key) in commands {
                if let Some(shortcut) = key {
                    let mut parts = shortcut.split('+');
                    let _ = parts.next();
                    let rest: Vec<_> = parts.collect();
                    let shift = rest.contains(&"Shift");
                    let key = match *rest.last().unwrap_or(&"") {
                        "P" => egui::Key::P,
                        "Enter" => egui::Key::Enter,
                        "N" => egui::Key::N,
                        "O" => egui::Key::O,
                        "S" => egui::Key::S,
                        "Z" => egui::Key::Z,
                        "X" => egui::Key::X,
                        "C" => egui::Key::C,
                        "V" => egui::Key::V,
                        "D" => egui::Key::D,
                        "B" => egui::Key::B,
                        "G" => egui::Key::G,
                        "I" => egui::Key::I,
                        "E" => egui::Key::E,
                        "L" => egui::Key::L,
                        "H" => egui::Key::H,
                        "R" => egui::Key::R,
                        "T" => egui::Key::T,
                        "U" => egui::Key::U,
                        "Equal" => egui::Key::Equals,
                        "Minus" => egui::Key::Minus,
                        "W" => egui::Key::W,
                        "BracketLeft" => egui::Key::OpenBracket,
                        "BracketRight" => egui::Key::CloseBracket,
                        _ => continue,
                    };
                    if ctx.wants_keyboard_input()
                        && !matches!(
                            a,
                            Action::Save | Action::SaveAs | Action::Open | Action::New
                        )
                    {
                        continue;
                    }
                    let mut modifiers = egui::Modifiers::COMMAND;
                    modifiers.shift = shift;
                    if ctx.input_mut(|i| i.consume_key(modifiers, key))
                        && self.action_enabled(a, ctx)
                    {
                        self.apply_action(a, ctx);
                    }
                }
            }
        }
        if ctx
            .data_mut(|d| d.remove_temp::<bool>(egui::Id::new("open_font_browser")))
            .unwrap_or(false)
        {
            self.font_chooser = !self.font_chooser;
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("focus_font_search"), true));
        }
        if self.font_chooser {
            let anchor = ctx.data(|d| d.get_temp::<Rect>(egui::Id::new("font_picker_anchor")));
            if let Some(anchor) = anchor.filter(|_| {
                self.selected.is_some_and(|i| {
                    self.doc.pages[self.page].artboards[self.artboard]
                        .layers
                        .get(i)
                        .is_some_and(|l| l.kind == Kind::Text)
                })
            }) {
                let screen = ctx.screen_rect();
                let position = Pos2::new(
                    (anchor.left() - 308.0).max(screen.left() + 8.0),
                    anchor.top().clamp(
                        screen.top() + 8.0,
                        (screen.bottom() - 390.0).max(screen.top() + 8.0),
                    ),
                );
                let popup = egui::Area::new(egui::Id::new("font_dropdown_popup"))
                    .order(egui::Order::Foreground)
                    .fixed_pos(position)
                    .show(ctx, |ui| {
                        egui::Frame::popup(ui.style())
                            .inner_margin(10)
                            .corner_radius(10)
                            .show(ui, |ui| {
                                if let Some(index) = self.selected {
                                    let before = self.doc.clone();
                                    let t = &mut self.doc.pages[self.page].artboards[self.artboard]
                                        .layers[index]
                                        .typography;
                                    if fonts::font_browser(ui, &mut t.family, &mut t.monospace) {
                                        self.font_chooser = false;
                                    }
                                    if before != self.doc {
                                        self.record_change(before);
                                    }
                                }
                            });
                    });
                if dismiss_transient_menu(ctx, popup.response.rect, anchor) {
                    self.font_chooser = false;
                }
            } else {
                self.font_chooser = false;
            }
        }
        if self.help {
            let popup = egui::Window::new("Keyboard shortcuts")
                .frame(egui::Frame::window(&ctx.style()).inner_margin(20))
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    section_heading(ui, "Keyboard shortcuts");
                    ui.add_space(8.0);
                    for (key, label) in [
                        ("V", "Select"),
                        ("R / O / T", "Rectangle / Ellipse / Text"),
                        ("Space + drag", "Pan canvas"),
                        ("Alt + drag", "Bypass snapping"),
                        ("Cmd/Ctrl + scroll", "Zoom"),
                        ("Cmd/Ctrl S", "Save"),
                        ("Cmd/Ctrl O", "Open"),
                        ("Cmd/Ctrl N", "New document"),
                        ("Cmd/Ctrl D", "Duplicate"),
                        ("Cmd/Ctrl Z", "Undo"),
                        ("Delete", "Delete unlocked object"),
                    ] {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [150.0, 22.0],
                                egui::Label::new(
                                    egui::RichText::new(key.replace("Cmd/Ctrl", shortcut_name()))
                                        .monospace()
                                        .size(11.0),
                                ),
                            );
                            ui.label(label);
                        });
                    }
                });
            if let Some(popup) = popup {
                if ctx.data(|d| d.get_temp::<u64>(egui::Id::new("help_opened_frame")))
                    != Some(ctx.cumulative_pass_nr())
                    && dismiss_transient_menu(ctx, popup.response.rect, Rect::NOTHING)
                {
                    self.help = false;
                }
            }
        }
    }
}
fn menu_item(
    ui: &mut egui::Ui,
    label: &str,
    shortcut: &str,
    enabled: bool,
    action: Action,
    result: &mut Option<Action>,
) {
    if ui
        .add_enabled(
            enabled,
            egui::Button::new(label)
                .shortcut_text(shortcut)
                .min_size(Vec2::new(210.0, 24.0)),
        )
        .clicked()
    {
        *result = Some(action);
        ui.close_menu();
    }
}

fn inline_name_editor(ui: &mut egui::Ui, rect: Rect, id: egui::Id) -> Option<String> {
    let mut name = ui.ctx().data(|d| d.get_temp::<String>(id))?;
    let response = ui.put(
        rect,
        egui::TextEdit::singleline(&mut name)
            .id(id)
            .font(egui::FontId::proportional(11.0))
            .margin(egui::vec2(3.0, 2.0)),
    );
    if !ui
        .ctx()
        .data(|d| d.get_temp::<bool>(id.with("focused")).unwrap_or(false))
    {
        response.request_focus();
        ui.ctx()
            .data_mut(|d| d.insert_temp(id.with("focused"), true));
    }
    let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));
    let commit = response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter));
    if cancel || commit {
        ui.ctx().data_mut(|d| {
            d.remove_temp::<String>(id);
            d.remove_temp::<bool>(id.with("focused"));
        });
        if commit && !cancel && !name.trim().is_empty() {
            return Some(name.trim().to_owned());
        }
    } else {
        ui.ctx().data_mut(|d| d.insert_temp(id, name));
    }
    None
}

fn tree_section(
    ui: &mut egui::Ui,
    id: egui::Id,
    title: &str,
    icon: usize,
    open: bool,
    selected: bool,
    body: impl FnOnce(&mut egui::Ui),
) -> bool {
    let response =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, open)
            .show_header(ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(ui.available_width(), 23.0), Sense::click());
                if selected || response.hovered() {
                    ui.painter().rect_filled(
                        rect,
                        5.0,
                        if selected {
                            ACCENT
                        } else {
                            ui.visuals().widgets.hovered.bg_fill
                        },
                    );
                }
                paint_icon(
                    ui.painter(),
                    rect.left_center() + Vec2::new(16.0, 0.0),
                    icon,
                    if selected {
                        Color32::WHITE
                    } else {
                        ui.visuals().text_color()
                    },
                    0.65,
                );
                let text_rect = Rect::from_min_max(
                    rect.min + Vec2::new(30.0, 0.0),
                    rect.max - Vec2::new(8.0, 0.0),
                );
                ui.ctx()
                    .data_mut(|d| d.insert_temp(id.with("row_rect"), rect));
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::WidgetType::SelectableLabel,
                        true,
                        selected,
                        title,
                    )
                });
                let color = if selected {
                    Color32::WHITE
                } else {
                    ui.visuals().text_color()
                };
                if response.double_clicked() {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(id.with("rename"), title.to_owned()));
                }
                if let Some(name) = inline_name_editor(ui, text_rect, id.with("rename")) {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(id.with("renamed"), name));
                }
                let mut job = egui::text::LayoutJob::simple(
                    title.to_owned(),
                    egui::FontId::new(11.0, egui::FontFamily::Name("UI Bold".into())),
                    color,
                    text_rect.width(),
                );
                job.wrap = egui::text::TextWrapping::truncate_at_width(text_rect.width());
                let galley = ui.painter().layout_job(job);
                if !ui
                    .ctx()
                    .data(|d| d.get_temp::<String>(id.with("rename")).is_some())
                {
                    ui.painter().galley(
                        text_rect.left_center() - Vec2::new(0.0, galley.size().y / 2.0),
                        galley,
                        color,
                    );
                }
                response
            })
            .body(body);
    response.1.inner.clicked() || response.1.inner.secondary_clicked()
}

fn navigation_row(ui: &mut egui::Ui, title: &str, selected: bool) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 26.0), Sense::click());
    if selected || response.hovered() {
        ui.painter()
            .rect_filled(rect, 6.0, ui.visuals().widgets.inactive.bg_fill);
    }
    let mut job = egui::text::LayoutJob::simple(
        title.to_owned(),
        egui::FontId::proportional(11.0),
        ui.visuals().text_color(),
        188.0,
    );
    job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 16.0);
    let galley = ui.painter().layout_job(job);
    ui.painter().galley(
        rect.left_center() + Vec2::new(8.0, -galley.size().y / 2.0),
        galley,
        ui.visuals().text_color(),
    );
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::SelectableLabel, true, title));
    response
}
fn small_icon(ui: &mut egui::Ui, icon: usize, tip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::click());
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 6.0, ui.visuals().widgets.hovered.bg_fill);
    }
    paint_icon(
        ui.painter(),
        rect.center(),
        icon,
        ui.visuals().text_color(),
        0.8,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    response.on_hover_text(tip)
}
fn next_group_name(layers: &[Layer]) -> String {
    let mut n = 1;
    while layers
        .iter()
        .any(|l| l.group.as_deref() == Some(format!("Group {n}").as_str()))
    {
        n += 1;
    }
    format!("Group {n}")
}

/// Reparent without changing the object's position on the pasteboard.
fn reorder_layer(
    boards: &mut [Artboard],
    source: (usize, usize),
    target: (usize, usize),
) -> Option<usize> {
    let (from, index) = source;
    let (to, destination) = target;
    if from >= boards.len()
        || to >= boards.len()
        || index >= boards[from].layers.len()
        || boards[from].layers[index].locked
        || source == target
    {
        return None;
    }
    let group = boards[to]
        .layers
        .get(destination)
        .and_then(|l| l.group.clone());
    let offset = Vec2::new(
        (from as f32 - to as f32) * 720.0 + boards[from].frame.x - boards[to].frame.x,
        boards[from].frame.y - boards[to].frame.y,
    );
    let mut layer = boards[from].layers.remove(index);
    layer.x += offset.x;
    layer.y += offset.y;
    layer.group = group;
    let destination = if from == to && index < destination {
        destination.saturating_sub(1)
    } else {
        destination
    };
    let destination = destination.min(boards[to].layers.len());
    boards[to].layers.insert(destination, layer);
    Some(destination)
}

fn tree_row(
    ui: &mut egui::Ui,
    layer: &mut Layer,
    selected: bool,
    width: f32,
    location: (usize, usize),
) -> bool {
    let (row, response) = ui.allocate_exact_size(Vec2::new(width, 23.0), Sense::click_and_drag());
    let drag_id = egui::Id::new("layer_reorder_source");
    if response.drag_started() && !layer.locked {
        ui.ctx().data_mut(|d| d.insert_temp(drag_id, location));
    }
    if response.contains_pointer() && ui.input(|i| i.pointer.any_released()) {
        if let Some(source) = ui
            .ctx()
            .data_mut(|d| d.remove_temp::<(usize, usize)>(drag_id))
        {
            ui.ctx().data_mut(|d| {
                d.insert_temp(egui::Id::new("layer_reorder_request"), (source, location))
            });
        }
    }
    if response.contains_pointer()
        && ui
            .ctx()
            .data(|d| d.get_temp::<(usize, usize)>(drag_id))
            .is_some()
    {
        ui.painter().line_segment(
            [row.left_top(), row.right_top()],
            Stroke::new(2.0_f32, ACCENT),
        );
    }
    let color = if selected {
        Color32::WHITE
    } else if !layer.visible {
        Color32::from_gray(170)
    } else {
        ui.visuals().text_color()
    };
    if selected || response.hovered() {
        ui.painter().rect_filled(
            row,
            5.0,
            if selected {
                ACCENT
            } else {
                ui.visuals().widgets.hovered.bg_fill
            },
        );
    }
    paint_icon(
        ui.painter(),
        Pos2::new(row.left() + 16.0, row.center().y),
        match layer.kind {
            Kind::Rectangle => 1,
            Kind::Ellipse => 2,
            Kind::Text => 3,
            Kind::Image => 13,
        },
        color,
        0.65,
    );
    let rename_id = egui::Id::new(("inline_layer", location.0, location.1));
    if response.double_clicked() {
        ui.ctx().data_mut(|d| {
            d.insert_temp(
                rename_id,
                layer.label.clone().unwrap_or_else(|| layer.name.clone()),
            )
        });
    }
    if let Some(name) = inline_name_editor(
        ui,
        Rect::from_min_max(
            row.left_top() + Vec2::new(30.0, 0.0),
            row.right_bottom() - Vec2::new(40.0, 0.0),
        ),
        rename_id,
    ) {
        layer.label = Some(name);
    }
    if !ui.ctx().data(|d| d.get_temp::<String>(rename_id).is_some()) {
        let mut job = egui::text::LayoutJob::simple_singleline(
            layer.label.clone().unwrap_or_else(|| layer.name.clone()),
            egui::FontId::proportional(11.0),
            color,
        );
        job.wrap = egui::text::TextWrapping::truncate_at_width(width - 66.0);
        let galley = ui.painter().layout_job(job);
        ui.painter()
            .galley(Pos2::new(row.left() + 30.0, row.top() + 5.0), galley, color);
    }
    let lock_rect = Rect::from_center_size(
        Pos2::new(row.right() - 10.0, row.center().y),
        Vec2::splat(18.0),
    );
    let eye_rect = lock_rect.translate(Vec2::new(-19.0, 0.0));
    let lock = ui.interact(lock_rect, response.id.with("lock"), Sense::click());
    let eye = ui.interact(eye_rect, response.id.with("eye"), Sense::click());
    if layer.locked || response.hovered() || lock.hovered() {
        paint_icon(ui.painter(), lock_rect.center(), 11, color, 0.65);
    }
    if !layer.visible || response.hovered() || eye.hovered() {
        paint_icon(ui.painter(), eye_rect.center(), 12, color, 0.65);
    }
    if lock.clicked() {
        layer.locked = !layer.locked;
    }
    if eye.clicked() {
        layer.visible = !layer.visible;
    }
    lock.clone().on_hover_text(if layer.locked {
        "Unlock layer"
    } else {
        "Lock layer"
    });
    eye.clone().on_hover_text(if layer.visible {
        "Hide layer"
    } else {
        "Show layer"
    });
    response
        .clone()
        .on_hover_text("Double-click to rename · right-click for layer commands");
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            true,
            selected,
            layer.label.as_deref().unwrap_or(&layer.name),
        )
    });
    (response.clicked() || response.secondary_clicked()) && !lock.clicked() && !eye.clicked()
}
fn canvas_header_height(width: f32) -> f32 {
    if width < 400.0 {
        76.0
    } else {
        48.0
    }
}
fn island(ctx: &egui::Context) -> egui::Frame {
    egui::Frame::new()
        .fill(if ctx.style().visuals.dark_mode {
            Color32::from_rgb(48, 50, 57)
        } else {
            ctx.style().visuals.panel_fill
        })
        .corner_radius(4)
        .stroke(Stroke::new(
            1.0_f32,
            if ctx.style().visuals.dark_mode {
                Color32::from_rgb(86, 89, 99)
            } else {
                Color32::from_black_alpha(10)
            },
        ))
        .inner_margin(egui::Margin::symmetric(5, 2))
}
fn paint_icon(p: &egui::Painter, c: Pos2, icon: usize, color: Color32, scale: f32) {
    let pt = |x: f32, y: f32| c + Vec2::new(x, y) * scale;
    let st = Stroke::new(1.25_f32 * scale.max(0.85), color);
    let line = |points: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            points.iter().map(|&(x, y)| pt(x, y)).collect(),
            st,
        ));
    };
    let rect = |x: f32, y: f32, w: f32, h: f32, r: u8| {
        p.rect_stroke(
            Rect::from_center_size(pt(x, y), Vec2::new(w, h) * scale),
            r,
            st,
            egui::StrokeKind::Inside,
        );
    };
    match icon {
        0 => {
            line(&[
                (-6.0, -9.0),
                (-5.0, 8.0),
                (-1.0, 3.0),
                (3.0, 10.0),
                (6.0, 8.0),
                (2.0, 1.0),
                (8.0, 0.0),
                (-6.0, -9.0),
            ]);
        }
        1 => {
            rect(0.0, 0.0, 16.0, 14.0, 2);
        }
        2 => {
            p.circle_stroke(c, 8.0 * scale, st);
        }
        3 => {
            line(&[(-7.0, -7.0), (-7.0, -9.0), (7.0, -9.0), (7.0, -7.0)]);
            line(&[(0.0, -9.0), (0.0, 9.0)]);
            line(&[(-4.0, 9.0), (4.0, 9.0)]);
        }
        4 => {
            rect(0.0, 0.0, 17.0, 18.0, 2);
            rect(0.0, -5.0, 9.0, 6.0, 1);
            rect(0.0, 5.0, 10.0, 7.0, 1);
        }
        5 => {
            line(&[
                (-9.0, -5.0),
                (-9.0, -8.0),
                (-2.0, -8.0),
                (1.0, -4.0),
                (9.0, -4.0),
                (7.0, 8.0),
                (-9.0, 8.0),
                (-9.0, -5.0),
                (9.0, -5.0),
            ]);
        }
        6 => {
            line(&[(-3.0, -8.0), (-8.0, -3.0), (-3.0, 2.0)]);
            line(&[
                (-8.0, -3.0),
                (2.0, -3.0),
                (6.0, -1.0),
                (8.0, 3.0),
                (7.0, 7.0),
                (4.0, 9.0),
            ]);
        }
        7 => {
            rect(0.0, 0.0, 18.0, 18.0, 3);
            for v in [-3.0, 3.0] {
                line(&[(v, -8.0), (v, 8.0)]);
                line(&[(-8.0, v), (8.0, v)]);
            }
        }
        8 => {
            rect(0.0, 0.0, 20.0, 10.0, 2);
            for v in [-6.0, -2.0, 2.0, 6.0] {
                line(&[(v, -4.0), (v, if v == 2.0 { 2.0 } else { 0.0 })]);
            }
        }
        9 => {
            p.circle_stroke(c, 9.0 * scale, st);
            line(&[(-4.0, 0.0), (4.0, 0.0)]);
            line(&[(0.0, -4.0), (0.0, 4.0)]);
        }
        10 => {
            rect(-1.0, 1.0, 14.0, 14.0, 2);
            line(&[(4.0, -8.0), (4.0, 0.0)]);
            line(&[(0.0, -4.0), (8.0, -4.0)]);
        }
        11 => {
            rect(0.0, 3.0, 12.0, 10.0, 2);
            line(&[
                (-4.0, -2.0),
                (-4.0, -6.0),
                (-2.0, -9.0),
                (2.0, -9.0),
                (4.0, -6.0),
                (4.0, -2.0),
            ]);
            p.circle_filled(pt(0.0, 3.0), scale, color);
        }
        12 => {
            line(&[
                (-10.0, 0.0),
                (-5.0, -5.0),
                (0.0, -6.0),
                (5.0, -5.0),
                (10.0, 0.0),
                (5.0, 5.0),
                (0.0, 6.0),
                (-5.0, 5.0),
                (-10.0, 0.0),
            ]);
            p.circle_stroke(c, 2.5 * scale, st);
        }
        13 => {
            line(&[
                (-7.0, -9.0),
                (2.0, -9.0),
                (7.0, -4.0),
                (7.0, 8.0),
                (-7.0, 8.0),
                (-7.0, -9.0),
            ]);
            line(&[(2.0, -9.0), (2.0, -4.0), (7.0, -4.0)]);
        }
        14 => {
            line(&[
                (0.0, -10.0),
                (-7.0, 4.0),
                (-4.0, 9.0),
                (4.0, 9.0),
                (7.0, 4.0),
                (0.0, -10.0),
                (0.0, 2.0),
            ]);
            p.circle_stroke(pt(0.0, 3.0), 2.0 * scale, st);
            line(&[(-4.0, 9.0), (4.0, 9.0)]);
        }
        _ => {}
    }
}
fn paint_artboard(p: &egui::Painter, origin: Pos2, zoom: f32, board: &Artboard) {
    let bounds = Rect::from_min_size(
        origin,
        Vec2::new(board.frame.width, board.frame.height) * zoom,
    );
    p.rect_filled(
        bounds.translate(Vec2::new(0.0, 3.0)),
        0.0,
        Color32::from_black_alpha(12),
    );
    p.rect_filled(
        bounds,
        (board.frame.radius * zoom).min(255.0) as u8,
        Color32::from_rgb(
            board.frame.background[0],
            board.frame.background[1],
            board.frame.background[2],
        ),
    );
    p.text(
        origin - Vec2::new(0.0, 18.0),
        egui::Align2::LEFT_CENTER,
        &board.name,
        egui::FontId::proportional(11.0),
        Color32::GRAY,
    );
    let p = p.with_clip_rect(if board.frame.clip {
        bounds.intersect(p.clip_rect())
    } else {
        p.clip_rect()
    });
    for (layer_index, l) in board.layers.iter().enumerate() {
        if !l.visible || l.mask {
            continue;
        }
        let r = Rect::from_min_size(
            origin + Vec2::new(l.x, l.y) * zoom,
            Vec2::new(l.width, l.height) * zoom,
        );
        if l.path.is_some()
            || l.kind == Kind::Text
            || l.kind == Kind::Image
            || l.corners.is_some()
            || l.corner_style != "Rounded"
            || l.drop_shadow.is_some()
            || l.inner_shadow.is_some()
            || !l.effects.is_empty()
            || active_mask(l, &board.layers[..layer_index]).is_some()
        {
            paint_rendered_layer(&p, l, r, zoom, &board.layers[..layer_index]);
            continue;
        }
        let color = Color32::from_rgb(l.color[0], l.color[1], l.color[2]);
        match l.kind {
            Kind::Image => {}
            Kind::Rectangle => {
                p.rect_filled(r, (l.radius * zoom).min(255.0) as u8, color);
            }
            Kind::Ellipse => {
                p.add(egui::epaint::EllipseShape::filled(
                    r.center(),
                    r.size() / 2.0,
                    color,
                ));
            }
            Kind::Text => {
                p.with_clip_rect(r.intersect(p.clip_rect())).text(
                    r.min,
                    egui::Align2::LEFT_TOP,
                    &l.name,
                    egui::FontId::new(
                        (l.height * 0.65).min(48.0) * zoom,
                        fonts::editor_family(p.ctx(), &l.typography),
                    ),
                    color,
                );
            }
        }
    }
}

/// Shared search surface with visible boundaries and a drawn magnifier.
fn search_field(
    ui: &mut egui::Ui,
    salt: &str,
    value: &mut String,
    hint: &str,
    width: f32,
) -> egui::Response {
    let width = width.min(ui.available_width()).max(80.0);
    let id = ui.make_persistent_id(salt);
    let dark = ui.visuals().dark_mode;
    let focused = ui.memory(|m| m.has_focus(id));
    let border = if focused {
        ACCENT
    } else if dark {
        Color32::from_rgb(112, 117, 133)
    } else {
        Color32::from_rgb(174, 179, 195)
    };
    let high_contrast = ui.ctx().data(|d| {
        d.get_temp::<bool>(egui::Id::new("high_contrast_ui"))
            .unwrap_or(false)
    });
    let border = if high_contrast && !focused {
        Color32::BLACK
    } else {
        border
    };
    let foreground = if dark {
        Color32::from_gray(210)
    } else {
        secondary_text_color(ui)
    };
    egui::Frame::new()
        .fill(if dark {
            Color32::from_rgb(43, 46, 54)
        } else {
            Color32::WHITE
        })
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(8)
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width((width - 22.0).max(58.0));
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let (rect, icon) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::click());
                let center = rect.center() - Vec2::splat(1.5);
                ui.painter()
                    .circle_stroke(center, 4.5, Stroke::new(1.4_f32, foreground));
                ui.painter().line_segment(
                    [center + Vec2::splat(3.2), center + Vec2::splat(7.0)],
                    Stroke::new(1.4_f32, foreground),
                );
                let response = ui.add(
                    egui::TextEdit::singleline(value)
                        .id(id)
                        .frame(false)
                        .margin(0)
                        .hint_text(egui::RichText::new(hint).color(foreground))
                        .desired_width(ui.available_width()),
                );
                if icon.clicked() {
                    response.request_focus();
                }
                response
            })
            .inner
        })
        .inner
}

fn welcome_document_card(
    ui: &mut egui::Ui,
    doc: &Document,
    title: &str,
    subtitle: &str,
    width: f32,
) -> egui::Response {
    ui.allocate_ui_with_layout(
        Vec2::new(width, 180.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 8.0;
            let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 120.0), Sense::click());
            ui.painter()
                .rect_filled(rect, 8.0, ui.visuals().widgets.inactive.bg_fill);
            if let Some(board) = doc.pages.first().and_then(|p| p.artboards.first()) {
                let scale = ((width - 20.0) / board.frame.width).min(100.0 / board.frame.height);
                let origin =
                    rect.center() - Vec2::new(board.frame.width, board.frame.height) * scale / 2.0;
                paint_artboard(
                    &ui.painter()
                        .with_clip_rect(rect.shrink(8.0).intersect(Rect::from_min_size(
                            origin,
                            Vec2::new(board.frame.width, board.frame.height) * scale,
                        ))),
                    origin,
                    scale,
                    board,
                );
            }
            let title_response = ui.add_sized(
                [width, 24.0],
                egui::Button::new(egui::RichText::new(title).strong())
                    .frame(false)
                    .truncate(),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(subtitle)
                        .size(10.0)
                        .color(secondary_text_color(ui)),
                )
                .truncate(),
            );
            response.union(title_response)
        },
    )
    .inner
}

/// A consistent circular rollover target for every app-owned close icon.
fn close_icon_button(ui: &mut egui::Ui, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click());
    let visuals = ui.style().interact(&response);
    if response.hovered() || response.is_pointer_button_down_on() || response.has_focus() {
        ui.painter()
            .circle_filled(rect.center(), size * 0.5, visuals.bg_fill);
    }
    let half = 3.5;
    let center = rect.center();
    let stroke = egui::Stroke::new(1.4_f32, visuals.fg_stroke.color);
    ui.painter().line_segment(
        [
            center + egui::vec2(-half, -half),
            center + egui::vec2(half, half),
        ],
        stroke,
    );
    ui.painter().line_segment(
        [
            center + egui::vec2(-half, half),
            center + egui::vec2(half, -half),
        ],
        stroke,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Close"));
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn type_segments(
    ui: &mut egui::Ui,
    width: f32,
    labels: &[&str],
    tips: &[&str],
    selected: &mut usize,
) {
    egui::Frame::new()
        .fill(ui.visuals().widgets.inactive.bg_fill)
        .corner_radius(7)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(|ui| {
                for (i, label) in labels.iter().enumerate() {
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(width / labels.len() as f32, 28.0),
                        Sense::click(),
                    );
                    let active = *selected == i;
                    if active || response.hovered() {
                        ui.painter().rect_filled(
                            rect.shrink(1.0),
                            7.0,
                            if active {
                                ACCENT
                            } else {
                                ui.visuals().widgets.hovered.bg_fill
                            },
                        );
                    }
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        *label,
                        egui::FontId::proportional(13.0),
                        if active {
                            Color32::WHITE
                        } else {
                            ui.visuals().text_color()
                        },
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::WidgetType::SelectableLabel,
                            true,
                            active,
                            tips[i],
                        )
                    });
                    if response.on_hover_text(tips[i]).clicked() {
                        *selected = i;
                    }
                }
            });
        });
}

fn type_controls(ui: &mut egui::Ui, layer: &mut Layer) {
    fonts::family_picker(
        ui,
        &mut layer.typography.family,
        &mut layer.typography.monospace,
    );
    let width = ui.available_width();
    let gap = 8.0;
    ui.spacing_mut().item_spacing.y = 10.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        ui.spacing_mut().button_padding.y = 7.0;
        egui::ComboBox::from_id_salt("font_weight")
            .width(width * 0.65 - gap)
            .selected_text(match (layer.typography.bold, layer.typography.italic) {
                (true, true) => "Bold Italic",
                (true, false) => "Bold",
                (false, true) => "Italic",
                _ => "Regular",
            })
            .show_ui(ui, |ui| {
                for (bold, italic, label) in [
                    (false, false, "Regular"),
                    (true, false, "Bold"),
                    (false, true, "Italic"),
                    (true, true, "Bold Italic"),
                ] {
                    if ui
                        .selectable_label(
                            layer.typography.bold == bold && layer.typography.italic == italic,
                            label,
                        )
                        .clicked()
                    {
                        layer.typography.bold = bold;
                        layer.typography.italic = italic;
                    }
                }
            });
        let mut size = layer
            .typography
            .size
            .unwrap_or((layer.height * 0.65).min(48.0));
        if ui
            .add_sized(
                [width * 0.35, 28.0],
                egui::DragValue::new(&mut size)
                    .range(1.0..=240.0)
                    .prefix("aA ")
                    .max_decimals(1),
            )
            .changed()
        {
            layer.typography.size = Some(size);
        }
    });
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let field = (width - gap * 2.0) / 3.0;
        ui.add_sized(
            [field, 28.0],
            egui::DragValue::new(&mut layer.typography.tracking)
                .range(-10.0..=40.0)
                .speed(0.1)
                .prefix("↔ ")
                .max_decimals(1),
        )
        .on_hover_text("Letter spacing");
        let mut line = layer.typography.line_height.unwrap_or(
            layer
                .typography
                .size
                .unwrap_or((layer.height * 0.65).min(48.0))
                * 1.4,
        );
        if ui
            .add_sized(
                [field, 28.0],
                egui::DragValue::new(&mut line)
                    .range(1.0..=400.0)
                    .prefix("↕ ")
                    .max_decimals(1),
            )
            .on_hover_text("Line height")
            .changed()
        {
            layer.typography.line_height = Some(line);
        }
        ui.add_sized(
            [field, 28.0],
            egui::DragValue::new(&mut layer.typography.baseline)
                .range(-200.0..=200.0)
                .prefix("B ")
                .max_decimals(1),
        )
        .on_hover_text("Baseline offset");
    });
    egui::Frame::new()
        .fill(ui.visuals().widgets.inactive.bg_fill)
        .corner_radius(8)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.horizontal(|ui| {
                for i in 0..4 {
                    let response =
                        ui.add_sized([width / 4.0, 28.0], egui::Button::new("").frame(false));
                    let p = ui.painter();
                    if layer.typography.align == i {
                        p.rect_filled(response.rect.shrink(1.0), 7.0, ACCENT);
                    }
                    let c = response.rect.center();
                    for (row, len) in [18.0, 12.0, 16.0].into_iter().enumerate() {
                        let len = if i == 3 { 18.0 } else { len };
                        let left = match i {
                            0 => c.x - 9.0,
                            1 => c.x - len / 2.0,
                            _ => c.x + 9.0 - len,
                        };
                        let y = c.y - 5.0 + row as f32 * 5.0;
                        p.line_segment(
                            [Pos2::new(left, y), Pos2::new(left + len, y)],
                            Stroke::new(
                                1.2_f32,
                                if layer.typography.align == i {
                                    Color32::WHITE
                                } else {
                                    Color32::GRAY
                                },
                            ),
                        );
                    }
                    if response.clicked() {
                        layer.typography.align = i;
                    }
                    response.on_hover_text(["Align left", "Center", "Align right", "Justify"][i]);
                }
            });
        });
    type_segments(
        ui,
        width,
        &["↑", "↕", "↓"],
        &["Align top", "Center vertically", "Align bottom"],
        &mut layer.typography.vertical,
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        let mut transformation = layer.typography.transformation as usize;
        type_segments(
            ui,
            width * 0.65,
            &["Aa", "AA", "aa"],
            &["Original case", "Uppercase", "Lowercase"],
            &mut transformation,
        );
        layer.typography.transformation = transformation as u8;
        let toggle_width = (width * 0.35 - gap * 2.0) / 2.0;
        ui.add_sized(
            [toggle_width, 28.0],
            egui::Button::new(egui::RichText::new("Aa").underline().color(
                if layer.typography.underline {
                    Color32::WHITE
                } else {
                    ui.visuals().text_color()
                },
            ))
            .fill(if layer.typography.underline {
                ACCENT
            } else {
                ui.visuals().widgets.inactive.bg_fill
            })
            .stroke(Stroke::NONE),
        )
        .on_hover_text("Underline")
        .clicked()
        .then(|| layer.typography.underline = !layer.typography.underline);
        ui.add_sized(
            [toggle_width, 28.0],
            egui::Button::new(egui::RichText::new("Aa").strikethrough().color(
                if layer.typography.strike {
                    Color32::WHITE
                } else {
                    ui.visuals().text_color()
                },
            ))
            .fill(if layer.typography.strike {
                ACCENT
            } else {
                ui.visuals().widgets.inactive.bg_fill
            })
            .stroke(Stroke::NONE),
        )
        .on_hover_text("Strikethrough")
        .clicked()
        .then(|| layer.typography.strike = !layer.typography.strike);
    });
    ui.collapsing("Advanced typography", |ui| {
        let mut kerning = layer.typography.kerning.unwrap_or(true);
        if ui.checkbox(&mut kerning, "Font kerning").changed() {
            layer.typography.kerning = Some(kerning);
        }
        let mut ligatures = layer.typography.ligatures.unwrap_or(true);
        if ui.checkbox(&mut ligatures, "Standard ligatures").changed() {
            layer.typography.ligatures = Some(ligatures);
        }
        ui.checkbox(&mut layer.typography.small_caps, "Small caps");
        ui.checkbox(&mut layer.typography.tabular, "Tabular figures");
        ui.checkbox(&mut layer.typography.slashed_zero, "Slashed zero");
        ui.checkbox(
            &mut layer.typography.discretionary,
            "Discretionary ligatures",
        );
        ui.add(
            egui::DragValue::new(&mut layer.typography.baseline)
                .prefix("Baseline ")
                .range(-200.0..=200.0),
        );
        egui::ComboBox::from_id_salt("text_direction")
            .selected_text(
                ["Automatic", "Left to right", "Right to left"]
                    [layer.typography.direction.min(2) as usize],
            )
            .show_ui(ui, |ui| {
                for (i, label) in ["Automatic", "Left to right", "Right to left"]
                    .iter()
                    .enumerate()
                {
                    ui.selectable_value(&mut layer.typography.direction, i as u8, *label);
                }
            });
        egui::ComboBox::from_id_salt("text_list")
            .selected_text(
                ["No list", "Bulleted", "Numbered"][layer.typography.list.min(2) as usize],
            )
            .show_ui(ui, |ui| {
                for (i, label) in ["No list", "Bulleted", "Numbered"].iter().enumerate() {
                    ui.selectable_value(&mut layer.typography.list, i as u8, *label);
                }
            });
        ui.small("OpenType feature support depends on the font.");
    });
}
fn shadow_controls(ui: &mut egui::Ui, title: &str, value: &mut Option<ShadowStyle>) {
    ui.push_id(title, |ui| {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            section_heading(ui, title);
            ui.add_space((ui.available_width() - 24.0).max(0.0));
            if ui
                .small_button(if value.is_some() { "−" } else { "+" })
                .clicked()
            {
                *value = if value.is_some() {
                    None
                } else {
                    Some(ShadowStyle::default())
                };
            }
        });
        if let Some(shadow) = value {
            ui.checkbox(&mut shadow.enabled, "Enabled");
            fill_control(ui, &mut shadow.color, Some(&mut shadow.opacity));
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut shadow.x)
                        .range(-100.0..=100.0)
                        .prefix("X "),
                );
                ui.add(
                    egui::DragValue::new(&mut shadow.y)
                        .range(-100.0..=100.0)
                        .prefix("Y "),
                );
            });
            ui.horizontal(|ui| {
                ui.add(
                    egui::DragValue::new(&mut shadow.blur)
                        .range(0.0..=64.0)
                        .prefix("Blur "),
                );
                ui.add(
                    egui::DragValue::new(&mut shadow.spread)
                        .range(0.0..=32.0)
                        .prefix("Spread "),
                );
            });
        }
    });
}

fn effect_padding(layer: &Layer) -> f32 {
    let shadow = layer
        .drop_shadow
        .as_ref()
        .filter(|s| s.enabled)
        .map(|s| s.x.abs().max(s.y.abs()) + s.blur * 3.0 + s.spread)
        .unwrap_or(0.0);
    layer
        .effects
        .iter()
        .filter(|e| {
            e.enabled && e.kind != "Fade" && e.kind != "Glass" && e.kind != "Background Blur"
        })
        .map(|e| e.amount * 3.0)
        .fold(shadow, f32::max)
}

fn paint_rendered_layer(
    p: &egui::Painter,
    layer: &Layer,
    rect: Rect,
    zoom: f32,
    backdrop: &[Layer],
) {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_string(layer)
        .unwrap_or_default()
        .hash(&mut hash);
    let id = egui::Id::new(("rendered_layer", hash.finish()));
    let ctx = p.ctx();
    let existing = ctx.data_mut(|d| d.get_temp::<egui::TextureHandle>(id));
    let texture = existing.or_else(|| {
        let img = rasterize_layer(layer, 2, false, backdrop).ok()?;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [img.width() as usize, img.height() as usize],
            img.as_raw(),
        );
        let texture = ctx.load_texture("layer asset", image, egui::TextureOptions::LINEAR);
        ctx.data_mut(|d| d.insert_temp(id, texture.clone()));
        Some(texture)
    });
    if let Some(texture) = texture {
        p.image(
            texture.id(),
            rect.expand(effect_padding(layer) * zoom),
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

fn export_preview(ui: &mut egui::Ui, layer: &Layer, backdrop: &[Layer]) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    let p = &painter;
    for y in 0..10 {
        for x in 0..(rect.width() / 10.0).ceil() as usize {
            p.rect_filled(
                Rect::from_min_size(
                    rect.min + Vec2::new(x as f32 * 10.0, y as f32 * 10.0),
                    Vec2::splat(10.0),
                )
                .intersect(rect),
                0.0,
                Color32::from_gray(if (x + y) % 2 == 0 { 245 } else { 225 }),
            );
        }
    }
    if !layer.visible {
        return;
    }
    let scale = ((rect.width() - 20.0) / layer.width)
        .min(80.0 / layer.height)
        .min(1.0);
    let bounds =
        Rect::from_center_size(rect.center(), Vec2::new(layer.width, layer.height) * scale);
    if layer.kind == Kind::Text
        || layer.kind == Kind::Image
        || layer.corners.is_some()
        || layer.corner_style != "Rounded"
        || layer.drop_shadow.is_some()
        || layer.inner_shadow.is_some()
        || !layer.effects.is_empty()
        || active_mask(layer, backdrop).is_some()
    {
        paint_rendered_layer(p, layer, bounds, scale, backdrop);
        return;
    }
    let color = Color32::from_rgb(layer.color[0], layer.color[1], layer.color[2]);
    match layer.kind {
        Kind::Image => {}
        Kind::Rectangle => {
            p.rect_filled(bounds, layer.radius * scale, color);
        }
        Kind::Ellipse => {
            p.add(egui::Shape::ellipse_filled(
                bounds.center(),
                bounds.size() / 2.0,
                color,
            ));
        }
        Kind::Text => {
            p.text(
                bounds.min,
                egui::Align2::LEFT_TOP,
                &layer.name,
                egui::FontId::new(
                    (layer.height * 0.65).min(48.0) * scale,
                    fonts::editor_family(p.ctx(), &layer.typography),
                ),
                color,
            );
        }
    }
}

fn load_image_asset(path: &std::path::Path) -> Result<(Vec<u8>, u32, u32), String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    if data.len() > 32_000_000 {
        return Err("Image exceeds 32 MB".into());
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(&data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let (w, h) = reader.into_dimensions().map_err(|e| e.to_string())?;
    if u64::from(w) * u64::from(h) > 16_000_000 {
        return Err("Image exceeds 16 megapixels".into());
    }
    image::load_from_memory(&data).map_err(|e| e.to_string())?;
    Ok((data, w, h))
}
fn active_mask<'a>(layer: &Layer, backdrop: &'a [Layer]) -> Option<&'a Layer> {
    backdrop
        .iter()
        .rev()
        .find(|m| m.mask && m.visible && m.group == layer.group)
}
fn shape_contains(layer: &Layer, p: Pos2) -> bool {
    let x = p.x - layer.x;
    let y = p.y - layer.y;
    if x < 0.0 || y < 0.0 || x > layer.width || y > layer.height {
        return false;
    }
    if let Some(path) = &layer.path {
        return path.fill
            && vector::path_contains(path, Vec2::new(layer.width, layer.height), [x, y]);
    }
    if layer.kind == Kind::Ellipse {
        ((x - layer.width / 2.0) / (layer.width / 2.0)).powi(2)
            + ((y - layer.height / 2.0) / (layer.height / 2.0)).powi(2)
            <= 1.0
    } else {
        let i = usize::from(x > layer.width / 2.0) + 2 * usize::from(y > layer.height / 2.0);
        let r = layer
            .corners
            .map(|c| c[i])
            .unwrap_or(layer.radius)
            .min(layer.width / 2.0)
            .min(layer.height / 2.0);
        let u = if i % 2 == 0 { x } else { layer.width - x };
        let v = if i < 2 { y } else { layer.height - y };
        if r == 0.0 || u >= r || v >= r {
            return true;
        }
        match layer.corner_style.as_str() {
            "Smooth" => (r - u).powi(4) + (r - v).powi(4) <= r.powi(4),
            "Angled" => u + v >= r,
            "Inside Square" => false,
            "Inside Arc" => u * u + v * v >= r * r,
            _ => (r - u).powi(2) + (r - v).powi(2) <= r * r,
        }
    }
}
fn rasterize_layer(
    layer: &Layer,
    scale: u32,
    trim: bool,
    backdrop: &[Layer],
) -> Result<image::RgbaImage, String> {
    let w = (layer.width * scale as f32).ceil().max(1.0) as u32;
    let h = (layer.height * scale as f32).ceil().max(1.0) as u32;
    if u64::from(w) * u64::from(h) > 16_000_000 {
        return Err("Asset is too large; lower the export scale".into());
    }
    let pad = (effect_padding(layer) * scale as f32).ceil() as u64;
    if (u64::from(w) + 2 * pad) * (u64::from(h) + 2 * pad) > 16_000_000 {
        return Err("Asset including effects is too large".into());
    }
    let mut img = image::RgbaImage::new(w, h);
    if layer.visible {
        if let Some(path) = &layer.path {
            img = vector::render(path, layer.color, w, h, scale as f32)?;
        } else if layer.kind == Kind::Image {
            if let Some(data) = &layer.image_data {
                let decoded = image::load_from_memory(data)
                    .map_err(|e| e.to_string())?
                    .to_rgba8();
                img =
                    image::imageops::resize(&decoded, w, h, image::imageops::FilterType::Lanczos3);
            }
        } else if layer.kind == Kind::Text {
            img = text::render(layer, scale)?;
        } else {
            for y in 0..h {
                for x in 0..w {
                    let mut covered = 0;
                    for sy in 0..4 {
                        for sx in 0..4 {
                            let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                            let py = y as f32 + (sy as f32 + 0.5) / 4.0;
                            let inside = if layer.kind == Kind::Ellipse {
                                ((px - w as f32 / 2.0) / (w as f32 / 2.0)).powi(2)
                                    + ((py - h as f32 / 2.0) / (h as f32 / 2.0)).powi(2)
                                    <= 1.0
                            } else {
                                shape_contains(
                                    layer,
                                    Pos2::new(
                                        layer.x + px / scale as f32,
                                        layer.y + py / scale as f32,
                                    ),
                                )
                            };
                            if inside {
                                covered += 1;
                            }
                        }
                    }
                    img.put_pixel(
                        x,
                        y,
                        image::Rgba([
                            layer.color[0],
                            layer.color[1],
                            layer.color[2],
                            (covered * 255 / 16) as u8,
                        ]),
                    );
                }
            }
        }
    }
    if let Some(effect) = layer
        .effects
        .iter()
        .find(|e| e.enabled && (e.kind == "Glass" || e.kind == "Background Blur"))
    {
        let mut behind = image::RgbaImage::new(w, h);
        for previous in backdrop.iter().filter(|l| l.visible) {
            if let Ok(asset) = rasterize_layer(previous, scale, false, &[]) {
                let pad = effect_padding(previous);
                image::imageops::overlay(
                    &mut behind,
                    &asset,
                    ((previous.x - layer.x - pad) * scale as f32) as i64,
                    ((previous.y - layer.y - pad) * scale as f32) as i64,
                );
            }
        }
        let blurred = image::imageops::blur(&behind, effect.amount * scale as f32 / 2.0);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let background = blurred.get_pixel(x, y);
            let a = pixel[3] as f32 / 255.0;
            if effect.kind == "Glass" {
                for c in 0..3 {
                    pixel[c] = (background[c] as f32 * 0.8 + pixel[c] as f32 * 0.2) as u8;
                }
                pixel[3] = (a * (background[3] as f32 * 0.8 + 51.0)) as u8;
            } else if background[3] > 0 {
                for c in 0..3 {
                    pixel[c] = background[c];
                }
            }
        }
    }
    img = render_effects(layer, img, scale);
    if let Some(mask) = active_mask(layer, backdrop) {
        let pad = effect_padding(layer);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let mut coverage = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let point = Pos2::new(
                        layer.x - pad + (x as f32 + (sx as f32 + 0.5) / 4.0) / scale as f32,
                        layer.y - pad + (y as f32 + (sy as f32 + 0.5) / 4.0) / scale as f32,
                    );
                    if shape_contains(mask, point) {
                        coverage += 1;
                    }
                }
            }
            pixel[3] = (pixel[3] as u32 * coverage / 16) as u8;
        }
    }
    let (w, h) = img.dimensions();
    if trim {
        let mut min = (w, h);
        let mut max = (0, 0);
        let mut found = false;
        for (x, y, pixel) in img.enumerate_pixels() {
            if pixel[3] > 0 {
                found = true;
                min.0 = min.0.min(x);
                min.1 = min.1.min(y);
                max.0 = max.0.max(x);
                max.1 = max.1.max(y);
            }
        }
        if found {
            img =
                image::imageops::crop_imm(&img, min.0, min.1, max.0 - min.0 + 1, max.1 - min.1 + 1)
                    .to_image();
        }
    }
    Ok(img)
}

fn render_effects(layer: &Layer, source: image::RgbaImage, scale: u32) -> image::RgbaImage {
    let pad = (effect_padding(layer) * scale as f32).ceil() as u32;
    let (w, h) = (source.width() + pad * 2, source.height() + pad * 2);
    let mut content = image::RgbaImage::new(w, h);
    image::imageops::overlay(&mut content, &source, pad as i64, pad as i64);
    let mask = content.clone();
    if let Some(s) = layer.inner_shadow.as_ref().filter(|s| s.enabled) {
        let dx = (s.x * scale as f32) as i64;
        let dy = (s.y * scale as f32) as i64;
        let inverse = image::RgbaImage::from_fn(w, h, |x, y| {
            let shrink_x = (layer.width * scale as f32
                / (layer.width * scale as f32 - s.spread * scale as f32 * 2.0).max(1.0))
            .max(1.0);
            let shrink_y = (layer.height * scale as f32
                / (layer.height * scale as f32 - s.spread * scale as f32 * 2.0).max(1.0))
            .max(1.0);
            let sx = (w as f32 / 2.0 + (x as f32 - dx as f32 - w as f32 / 2.0) * shrink_x) as i64;
            let sy = (h as f32 / 2.0 + (y as f32 - dy as f32 - h as f32 / 2.0) * shrink_y) as i64;
            let a = if sx >= 0 && sy >= 0 && sx < w as i64 && sy < h as i64 {
                mask.get_pixel(sx as u32, sy as u32)[3]
            } else {
                0
            };
            image::Rgba([s.color[0], s.color[1], s.color[2], 255 - a])
        });
        let mut shadow = image::imageops::blur(&inverse, s.blur * scale as f32 / 2.0);
        for (x, y, p) in shadow.enumerate_pixels_mut() {
            p[3] = (p[3] as f32 * mask.get_pixel(x, y)[3] as f32 / 255.0 * s.opacity / 100.0) as u8;
        }
        image::imageops::overlay(&mut content, &shadow, 0, 0);
    }
    let mut out = image::RgbaImage::new(w, h);
    if let Some(s) = layer.drop_shadow.as_ref().filter(|s| s.enabled) {
        let spread = (s.spread * scale as f32).round() as i32;
        let shadow = image::RgbaImage::from_fn(w, h, |x, y| {
            let sx = x as i32 - (s.x * scale as f32) as i32;
            let sy = y as i32 - (s.y * scale as f32) as i32;
            let expanded = if spread > 0 {
                let cx = w as f32 / 2.0;
                let cy = h as f32 / 2.0;
                (
                    (cx + (sx as f32 - cx) * layer.width * scale as f32
                        / (layer.width * scale as f32 + spread as f32 * 2.0))
                        as i32,
                    (cy + (sy as f32 - cy) * layer.height * scale as f32
                        / (layer.height * scale as f32 + spread as f32 * 2.0))
                        as i32,
                )
            } else {
                (sx, sy)
            };
            let a = if expanded.0 >= 0
                && expanded.1 >= 0
                && expanded.0 < w as i32
                && expanded.1 < h as i32
            {
                mask.get_pixel(expanded.0 as u32, expanded.1 as u32)[3]
            } else {
                0
            };
            image::Rgba([
                s.color[0],
                s.color[1],
                s.color[2],
                (a as f32 * s.opacity / 100.0) as u8,
            ])
        });
        out = image::imageops::blur(&shadow, s.blur * scale as f32 / 2.0);
    }
    image::imageops::overlay(&mut out, &content, 0, 0);
    for effect in layer.effects.iter().filter(|e| e.enabled) {
        match effect.kind.as_str() {
            "Fade" => {
                for p in out.pixels_mut() {
                    p[3] = (p[3] as f32 * (1.0 - effect.amount / 100.0)) as u8;
                }
            }
            "Layer Blur" => out = image::imageops::blur(&out, effect.amount * scale as f32 / 2.0),
            "Motion & Zoom Blur" => {
                let source = out.clone();
                let distance = (effect.amount * scale as f32) as i32;
                for (x, y, p) in out.enumerate_pixels_mut() {
                    let mut sums = [0u32; 4];
                    let mut count = 0;
                    for step in -4..=4 {
                        let (sx, sy) = if effect.zoom {
                            let factor =
                                1.0 + step as f32 * distance as f32 / (w.max(h) as f32 * 4.0);
                            (
                                (w as f32 / 2.0 + (x as f32 - w as f32 / 2.0) * factor) as i32,
                                (h as f32 / 2.0 + (y as f32 - h as f32 / 2.0) * factor) as i32,
                            )
                        } else {
                            (x as i32 + step * distance / 4, y as i32)
                        };
                        if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                            let sample = source.get_pixel(sx as u32, sy as u32);
                            for i in 0..4 {
                                sums[i] += sample[i] as u32;
                            }
                            count += 1;
                        }
                    }
                    for i in 0..4 {
                        p[i] = (sums[i] / count) as u8;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn export_layer(
    layer: &Layer,
    path: &std::path::Path,
    format: &str,
    scale: u32,
    trim: bool,
    backdrop: &[Layer],
) -> Result<(), String> {
    if format == "SVG"
        && (layer.kind == Kind::Text
            || layer.drop_shadow.is_some()
            || layer.inner_shadow.is_some()
            || !layer.effects.is_empty()
            || layer.kind == Kind::Image
            || layer.corners.is_some()
            || layer.corner_style != "Rounded"
            || active_mask(layer, backdrop).is_some())
    {
        let img = rasterize_layer(layer, scale, trim, backdrop)?;
        let (w, h) = img.dimensions();
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        let encoded = encode_base64(png.get_ref());
        return std::fs::write(path,format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\"><image width=\"{w}\" height=\"{h}\" href=\"data:image/png;base64,{encoded}\"/></svg>")).map_err(|e| e.to_string());
    }
    if format == "SVG" {
        let color = format!(
            "#{:02X}{:02X}{:02X}",
            layer.color[0], layer.color[1], layer.color[2]
        );
        let body = if !layer.visible {
            String::new()
        } else if let Some(path) = &layer.path {
            path.svg(Vec2::new(layer.width, layer.height), &color)
        } else {
            match layer.kind {
                Kind::Image => String::new(),
                Kind::Rectangle => format!(
                    "<rect width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{color}\"/>",
                    layer.width, layer.height, layer.radius
                ),
                Kind::Ellipse => format!(
                    "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{color}\"/>",
                    layer.width / 2.0,
                    layer.height / 2.0,
                    layer.width / 2.0,
                    layer.height / 2.0
                ),
                Kind::Text => {
                    let text = layer
                        .name
                        .replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('>', "&gt;");
                    let size = (layer.height * 0.65).min(48.0);
                    format!("<text x=\"0\" y=\"{size}\" font-family=\"Ubuntu, sans-serif\" font-size=\"{size}\" font-weight=\"300\" fill=\"{color}\">{text}</text>")
                }
            }
        };
        return std::fs::write(path, format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">{body}</svg>", layer.width*scale as f32, layer.height*scale as f32, layer.width, layer.height)).map_err(|e| e.to_string());
    }
    let img = rasterize_layer(layer, scale, trim, backdrop)?;
    let output = if format == "JPG" || format == "PDF" {
        let mut rgb = image::RgbImage::new(img.width(), img.height());
        for (x, y, p) in img.enumerate_pixels() {
            let a = p[3] as u32;
            rgb.put_pixel(
                x,
                y,
                image::Rgb(std::array::from_fn(|i| {
                    ((p[i] as u32 * a + 255 * (255 - a)) / 255) as u8
                })),
            );
        }
        image::DynamicImage::ImageRgb8(rgb)
    } else {
        image::DynamicImage::ImageRgba8(img)
    };
    if format == "PDF" {
        let mut jpeg = std::io::Cursor::new(Vec::new());
        output
            .write_to(&mut jpeg, image::ImageFormat::Jpeg)
            .map_err(|e| e.to_string())?;
        return std::fs::write(
            path,
            image_pdf(jpeg.get_ref(), output.width(), output.height()),
        )
        .map_err(|e| e.to_string());
    }
    let kind = match format {
        "PNG" => image::ImageFormat::Png,
        "JPG" => image::ImageFormat::Jpeg,
        "TIFF" => image::ImageFormat::Tiff,
        "WEBP" => image::ImageFormat::WebP,
        _ => return Err("Unsupported export format".into()),
    };
    output
        .save_with_format(path, kind)
        .map_err(|e| e.to_string())
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as usize;
        let b = chunk.get(1).copied().unwrap_or(0) as usize;
        let c = chunk.get(2).copied().unwrap_or(0) as usize;
        out.push(TABLE[a >> 2] as char);
        out.push(TABLE[((a & 3) << 4) | (b >> 4)] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((b & 15) << 2) | (c >> 6)] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[c & 63] as char
        } else {
            '='
        });
    }
    out
}

fn image_pdf(jpeg: &[u8], width: u32, height: u32) -> Vec<u8> {
    let command = format!("q {width} 0 0 {height} 0 0 cm /Im0 Do Q");
    let mut objects = vec![b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(), b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(), format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Resources << /XObject << /Im0 4 0 R >> >> /Contents 5 0 R >>").into_bytes()];
    let mut stream = format!("<< /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n", jpeg.len()).into_bytes();
    stream.extend_from_slice(jpeg);
    stream.extend_from_slice(b"\nendstream");
    objects.push(stream);
    objects.push(
        format!(
            "<< /Length {} >>\nstream\n{command}\nendstream",
            command.len()
        )
        .into_bytes(),
    );
    let mut bytes = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0];
    for (i, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        bytes.extend_from_slice(object);
        bytes.extend_from_slice(b"\nendobj\n");
    }
    let xref = bytes.len();
    bytes.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n").as_bytes(),
    );
    bytes
}

fn write_document_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let name = path
        .file_name()
        .ok_or("Missing document filename")?
        .to_string_lossy();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let temporary = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), stamp));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        #[cfg(not(target_os = "windows"))]
        std::fs::rename(&temporary, path)?;
        #[cfg(target_os = "windows")]
        {
            let backup = parent.join(format!(".{name}.{stamp}.backup"));
            let existed = path.exists();
            if existed {
                std::fs::rename(path, &backup)?;
            }
            if let Err(error) = std::fs::rename(&temporary, path) {
                if existed {
                    let _ = std::fs::rename(&backup, path);
                }
                return Err(error);
            }
            if existed {
                let _ = std::fs::remove_file(backup);
            }
        }
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(|e| e.to_string())
}

fn inspector_popover_position(ctx: &egui::Context, width: f32, height: f32, top: f32) -> Pos2 {
    let screen = ctx.screen_rect();
    Pos2::new(
        (screen.right() - 264.0 - width).max(screen.left()),
        top.clamp(
            screen.top() + 8.0,
            (screen.bottom() - height - 8.0).max(screen.top() + 8.0),
        ),
    )
}

fn secondary_text_color(ui: &egui::Ui) -> Color32 {
    if ui.ctx().data(|d| {
        d.get_temp::<bool>(egui::Id::new("high_contrast_ui"))
            .unwrap_or(false)
    }) {
        Color32::from_gray(45)
    } else if ui.visuals().dark_mode {
        Color32::from_gray(185)
    } else {
        Color32::from_gray(90)
    }
}
fn swatch_outline(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_gray(215)
    } else {
        Color32::from_gray(60)
    }
}
fn fill_control(ui: &mut egui::Ui, color: &mut [u8; 3], opacity: Option<&mut f32>) {
    let id = ui.make_persistent_id("fill_picker");
    let mut open = ui
        .ctx()
        .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false));
    let response = ui
        .horizontal(|ui| {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
            let c = Color32::from_rgb(color[0], color[1], color[2]);
            ui.painter().circle_filled(rect.center(), 10.0, c);
            ui.painter().circle_stroke(
                rect.center(),
                10.0,
                Stroke::new(1.5_f32, swatch_outline(ui)),
            );
            if open {
                ui.painter()
                    .circle_stroke(rect.center(), 13.0, Stroke::new(1.5_f32, ACCENT));
            }
            response.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, "Edit fill color")
            });
            if response.clicked() {
                open = !open;
            }
            egui::Frame::new()
                .fill(ui.visuals().widgets.inactive.bg_fill)
                .corner_radius(7)
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "#  {:02X}{:02X}{:02X}",
                            color[0], color[1], color[2]
                        ))
                        .monospace()
                        .size(11.0),
                    );
                });
            if let Some(opacity) = opacity {
                ui.add_sized(
                    [56.0, 26.0],
                    egui::DragValue::new(opacity).range(0.0..=100.0).suffix("%"),
                );
            } else {
                ui.label(
                    egui::RichText::new("100%")
                        .size(11.0)
                        .color(secondary_text_color(ui)),
                );
            }
        })
        .response;
    let ctx = ui.ctx().clone();
    let mut hsva = egui::ecolor::Hsva::from(Color32::from_rgb(color[0], color[1], color[2]));
    if let Some((previous, saved)) =
        ctx.data_mut(|d| d.get_temp::<([u8; 3], egui::ecolor::Hsva)>(id.with("hsv")))
    {
        if previous == *color {
            hsva = saved;
        }
    }
    if open {
        let popup = egui::Window::new("Fill color")
            .id(id.with("window"))
            .title_bar(false)
            .movable(false)
            .collapsible(false)
            .resizable(false)
            .fixed_pos(inspector_popover_position(
                &ctx,
                262.0,
                460.0,
                response.rect.top(),
            ))
            .frame(
                egui::Frame::window(ui.style())
                    .fill(ctx.style().visuals.panel_fill)
                    .corner_radius(16)
                    .inner_margin(14),
            )
            .show(&ctx, |ui| {
                ui.set_width(232.0);
                section_heading(ui, "Custom color");
                ui.add_space(8.0);
                let (rect, response) =
                    ui.allocate_exact_size(Vec2::new(232.0, 158.0), Sense::click_and_drag());
                let mut mesh = egui::Mesh::default();
                for y in 0..=16 {
                    for x in 0..=16 {
                        let c = Color32::from(egui::ecolor::Hsva::new(
                            hsva.h,
                            x as f32 / 16.0,
                            1.0 - y as f32 / 16.0,
                            1.0,
                        ));
                        mesh.colored_vertex(
                            rect.min
                                + Vec2::new(
                                    x as f32 * rect.width() / 16.0,
                                    y as f32 * rect.height() / 16.0,
                                ),
                            c,
                        );
                    }
                }
                for y in 0..16 {
                    for x in 0..16 {
                        let i = y * 17 + x;
                        mesh.add_triangle(i, i + 1, i + 17);
                        mesh.add_triangle(i + 1, i + 18, i + 17);
                    }
                }
                ui.painter().add(egui::Shape::mesh(mesh));
                if response.dragged() || response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        hsva.s = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                        hsva.v = (1.0 - (pos.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
                    }
                }
                let marker =
                    rect.min + Vec2::new(hsva.s * rect.width(), (1.0 - hsva.v) * rect.height());
                ui.painter()
                    .circle_stroke(marker, 5.0, Stroke::new(2.0_f32, Color32::WHITE));
                ui.add_space(8.0);
                let (hue, response) =
                    ui.allocate_exact_size(Vec2::new(232.0, 16.0), Sense::click_and_drag());
                for x in 0..232 {
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            hue.min + Vec2::new(x as f32, 0.0),
                            Vec2::new(1.0, 16.0),
                        ),
                        0.0,
                        Color32::from(egui::ecolor::Hsva::new(x as f32 / 232.0, 1.0, 1.0, 1.0)),
                    );
                }
                if response.dragged() || response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        hsva.h = ((pos.x - hue.left()) / hue.width()).clamp(0.0, 1.0);
                    }
                }
                ui.painter().circle_stroke(
                    Pos2::new(hue.left() + hsva.h * hue.width(), hue.center().y),
                    7.0,
                    Stroke::new(2.0_f32, Color32::WHITE),
                );
                let rgb = Color32::from(hsva);
                *color = [rgb.r(), rgb.g(), rgb.b()];
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    for (label, channel) in ["R", "G", "B"].into_iter().zip(color.iter_mut()) {
                        egui::Frame::new()
                            .fill(ui.visuals().widgets.inactive.bg_fill)
                            .corner_radius(7)
                            .inner_margin(6)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(label).color(secondary_text_color(ui)),
                                    );
                                    ui.add_sized(
                                        [38.0, 20.0],
                                        egui::DragValue::new(channel).range(0..=255),
                                    );
                                });
                            });
                    }
                });
                let hex_id = id.with("hex");
                let current = format!("{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
                let mut hex =
                    ctx.data_mut(|d| d.get_temp::<String>(hex_id).unwrap_or(current.clone()));
                ui.horizontal(|ui| {
                    ui.label("#");
                    let edit = ui.add(
                        egui::TextEdit::singleline(&mut hex)
                            .background_color(ui.visuals().widgets.inactive.bg_fill)
                            .margin(egui::Margin::symmetric(8, 6))
                            .font(egui::TextStyle::Monospace)
                            .desired_width(200.0)
                            .char_limit(6),
                    );
                    if edit.changed() {
                        if let Some(parsed) = parse_hex_color(&hex) {
                            *color = parsed;
                        }
                    }
                    if !edit.has_focus() {
                        hex = format!("{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
                    }
                });
                ctx.data_mut(|d| d.insert_temp(hex_id, hex));
                ui.add_space(8.0);
                ui.separator();
                section_heading(ui, "Swatches");
                let palette_id = egui::Id::new("suiza_swatches");
                let mut palette = ctx.data_mut(|d| {
                    d.get_temp::<Vec<[u8; 3]>>(palette_id).unwrap_or(vec![
                        [96, 83, 255],
                        [118, 202, 151],
                        [255, 255, 255],
                        [43, 62, 47],
                        [179, 109, 74],
                    ])
                });
                ui.horizontal_wrapped(|ui| {
                    for saved in &palette {
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::splat(28.0), Sense::click());
                        ui.painter().circle_filled(
                            rect.center(),
                            10.0,
                            Color32::from_rgb(saved[0], saved[1], saved[2]),
                        );
                        ui.painter().circle_stroke(
                            rect.center(),
                            10.0,
                            Stroke::new(1.5_f32, swatch_outline(ui)),
                        );
                        if response.clicked() {
                            *color = *saved;
                        }
                        response.on_hover_text(format!(
                            "#{:02X}{:02X}{:02X}",
                            saved[0], saved[1], saved[2]
                        ));
                    }
                });
                if ui.button("Save current swatch").clicked() && !palette.contains(color) {
                    palette.push(*color);
                }
                ctx.data_mut(|d| d.insert_temp(palette_id, palette));
            });
        if let Some(popup) = popup {
            if dismiss_transient_menu(&ctx, popup.response.rect, response.rect) {
                open = false;
            }
        }
    }
    let rendered = Color32::from(hsva);
    if [rendered.r(), rendered.g(), rendered.b()] != *color {
        let new = egui::ecolor::Hsva::from(Color32::from_rgb(color[0], color[1], color[2]));
        hsva.s = new.s;
        hsva.v = new.v;
        if new.s > 0.0 {
            hsva.h = new.h;
        }
    }
    ctx.data_mut(|d| {
        d.insert_temp(id, open);
        d.insert_temp(id.with("hsv"), (*color, hsva));
    });
}

// Shared behavior for transient inspector and toolbar menus. Native egui
// dropdown/context menus already handle outside clicks and Escape themselves.
fn dismiss_transient_menu(ctx: &egui::Context, menu: Rect, anchor: Rect) -> bool {
    ctx.input(|i| i.key_pressed(egui::Key::Escape))
        || (!ctx.memory(|m| m.any_popup_open())
            && ctx.input(|i| i.pointer.any_click())
            && ctx
                .pointer_interact_pos()
                .is_some_and(|p| !menu.contains(p) && !anchor.contains(p)))
}
fn anchored_menu_position(ctx: &egui::Context, anchor: Rect, width: f32, height: f32) -> Pos2 {
    let screen = ctx.screen_rect();
    Pos2::new(
        anchor.left().clamp(
            screen.left() + 8.0,
            (screen.right() - width - 8.0).max(screen.left() + 8.0),
        ),
        (anchor.bottom() + 8.0).min((screen.bottom() - height - 8.0).max(screen.top() + 8.0)),
    )
}

fn parse_hex_color(text: &str) -> Option<[u8; 3]> {
    if text.len() != 6 || !text.is_ascii() {
        return None;
    }
    let value = u32::from_str_radix(text, 16).ok()?;
    Some([(value >> 16) as u8, (value >> 8) as u8, value as u8])
}

fn section_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .font(egui::FontId::new(
                11.0,
                egui::FontFamily::Name("UI Bold".into()),
            ))
            .color(ui.visuals().text_color()),
    );
}
fn property(ui: &mut egui::Ui, label: &str, value: &mut f32, positive: bool, width: f32) {
    egui::Frame::new()
        .fill(ui.visuals().widgets.inactive.bg_fill)
        .corner_radius(7)
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.set_width((width - 16.0).max(60.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.add_sized(
                    [12.0, 20.0],
                    egui::Label::new(egui::RichText::new(label).size(10.0).color(
                        Color32::from_gray(if ui.visuals().dark_mode { 190 } else { 100 }),
                    )),
                );
                let mut drag = egui::DragValue::new(value).speed(1.0);
                if positive {
                    drag = drag.range(1.0..=4000.0);
                }
                ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                // The surrounding property frame owns the field background.
                // Avoid a second, smaller rounded rectangle on hover or drag.
                ui.visuals_mut().widgets.hovered.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.active.bg_fill = Color32::TRANSPARENT;
                ui.visuals_mut().widgets.active.weak_bg_fill = Color32::TRANSPARENT;
                ui.add_sized([(width - 36.0).max(40.0), 20.0], drag);
            });
        });
}
fn alignment_button(ui: &mut egui::Ui, action: usize, tip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(32.0, 26.0), Sense::click());
    let c = rect.center();
    let p = ui.painter();
    if response.hovered() {
        p.rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
    }
    let stroke = Stroke::new(1.0_f32, Color32::from_gray(135));
    let offset = match action % 3 {
        0 => -6.0,
        1 => 0.0,
        _ => 6.0,
    };
    if action < 3 {
        p.line_segment(
            [c + Vec2::new(offset, -8.0), c + Vec2::new(offset, 8.0)],
            stroke,
        );
        p.rect_stroke(
            Rect::from_center_size(c, Vec2::new(9.0, 4.0)),
            1.0,
            stroke,
            egui::StrokeKind::Inside,
        );
    } else {
        p.line_segment(
            [c + Vec2::new(-8.0, offset), c + Vec2::new(8.0, offset)],
            stroke,
        );
        p.rect_stroke(
            Rect::from_center_size(c, Vec2::new(4.0, 9.0)),
            1.0,
            stroke,
            egui::StrokeKind::Inside,
        );
    }
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    response.on_hover_text(tip)
}

fn corners(rect: Rect) -> [Pos2; 4] {
    [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ]
}
fn snap_value(value: f32, snap: bool) -> f32 {
    if snap {
        (value / 8.0).round() * 8.0
    } else {
        value
    }
}
fn resize_layer(layer: &mut Layer, original: &Layer, corner: usize, delta: Vec2, snap: bool) {
    let (mut left, mut top, mut right, mut bottom) = (
        original.x,
        original.y,
        original.x + original.width,
        original.y + original.height,
    );
    if corner == 0 || corner == 2 {
        left = snap_value(left + delta.x, snap).min(right - 8.0);
    } else {
        right = snap_value(right + delta.x, snap).max(left + 8.0);
    }
    if corner < 2 {
        top = snap_value(top + delta.y, snap).min(bottom - 8.0);
    } else {
        bottom = snap_value(bottom + delta.y, snap).max(top + 8.0);
    }
    layer.x = left;
    layer.y = top;
    layer.width = right - left;
    layer.height = bottom - top;
}
fn tool_button(ui: &mut egui::Ui, active: bool, icon: usize, tip: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
    if active || response.hovered() {
        ui.painter().rect_filled(
            rect.shrink(1.0),
            4.0,
            if active {
                Color32::from_rgb(222, 236, 255)
            } else {
                ui.visuals().widgets.hovered.bg_fill
            },
        );
    }
    paint_icon(
        ui.painter(),
        rect.center(),
        icon,
        if active {
            ACCENT
        } else {
            ui.visuals().text_color()
        },
        0.85,
    );
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tip));
    response.on_hover_text(tip)
}
fn paint_rulers(painter: &egui::Painter, area: Rect, origin: Pos2, zoom: f32) {
    let fill = if painter.ctx().style().visuals.dark_mode {
        Color32::from_rgb(32, 34, 39)
    } else {
        Color32::from_rgb(244, 244, 247)
    };
    painter.rect_filled(
        Rect::from_min_size(area.min, Vec2::new(area.width(), 20.0)),
        0.0,
        fill,
    );
    painter.rect_filled(
        Rect::from_min_size(area.min, Vec2::new(20.0, area.height())),
        0.0,
        fill,
    );
    let stroke = Stroke::new(1.0_f32, Color32::from_gray(190));
    let step = if zoom < 0.6 { 100 } else { 50 };
    for value in (-2000..=3000).step_by(step) {
        let x = origin.x + value as f32 * zoom;
        if x > area.left() + 20.0 && x < area.right() {
            painter.line_segment(
                [
                    Pos2::new(x, area.top() + 15.0),
                    Pos2::new(x, area.top() + 20.0),
                ],
                stroke,
            );
            painter.text(
                Pos2::new(x + 3.0, area.top() + 8.0),
                egui::Align2::LEFT_CENTER,
                value.to_string(),
                egui::FontId::proportional(9.0),
                Color32::from_gray(if painter.ctx().style().visuals.dark_mode {
                    205
                } else {
                    100
                }),
            );
        }
        let y = origin.y + value as f32 * zoom;
        if y > area.top() + 20.0 && y < area.bottom() {
            painter.line_segment(
                [
                    Pos2::new(area.left() + 15.0, y),
                    Pos2::new(area.left() + 20.0, y),
                ],
                stroke,
            );
            painter.text(
                Pos2::new(area.left() + 2.0, y - 7.0),
                egui::Align2::LEFT_CENTER,
                value.to_string(),
                egui::FontId::proportional(8.0),
                Color32::from_gray(if painter.ctx().style().visuals.dark_mode {
                    205
                } else {
                    100
                }),
            );
        }
    }
}

fn shortcut_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "Command"
    } else if cfg!(target_os = "linux") {
        "Super"
    } else {
        "Ctrl"
    }
}
fn menu_groups() -> Vec<(
    &'static str,
    Vec<(&'static str, Action, Option<&'static str>)>,
)> {
    vec![
        (
            "File",
            vec![
                ("New document", Action::New, Some("Cmd+N")),
                ("Open…", Action::Open, Some("Cmd+O")),
                ("Save…", Action::Save, Some("Cmd+S")),
                ("Save as…", Action::SaveAs, Some("Cmd+Shift+S")),
                ("Close document", Action::CloseTab, Some("Cmd+W")),
                ("Document browser…", Action::Welcome, None),
                ("Import image…", Action::ImportImage, Some("Cmd+Shift+I")),
                (
                    "Export selected…",
                    Action::ExportSelected,
                    Some("Cmd+Shift+E"),
                ),
            ],
        ),
        (
            "Edit",
            vec![
                ("Creative Workbench…", Action::Workbench, None),
                ("Plugins…", Action::Plugins, None),
                ("Undo", Action::Undo, Some("Cmd+Z")),
                ("Redo", Action::Redo, Some("Cmd+Shift+Z")),
                ("Cut layer", Action::CutLayer, Some("Cmd+X")),
                ("Copy layer", Action::CopyLayer, Some("Cmd+C")),
                ("Paste layer", Action::PasteLayer, Some("Cmd+V")),
                (
                    "Duplicate behind",
                    Action::DuplicateBehind,
                    Some("Cmd+Shift+D"),
                ),
                ("Duplicate", Action::Duplicate, Some("Cmd+D")),
                ("Rename layer…", Action::RenameLayer, Some("Cmd+R")),
                ("Delete", Action::Delete, None),
            ],
        ),
        (
            "Insert",
            vec![
                ("Page", Action::AddPage, None),
                ("Artboard", Action::AddBoard, None),
                ("Rectangle", Action::Tool(Some(Kind::Rectangle)), None),
                ("Ellipse", Action::Tool(Some(Kind::Ellipse)), None),
                ("Text", Action::Tool(Some(Kind::Text)), None),
                ("Bézier pen", Action::Pen, None),
            ],
        ),
        (
            "Layer",
            vec![
                ("Lock / unlock", Action::Lock, Some("Cmd+Shift+L")),
                ("Show / hide", Action::Visible, Some("Cmd+Shift+H")),
                ("Edit vector path", Action::EditPath, None),
                (
                    "Convert to outlines",
                    Action::ConvertPath,
                    Some("Cmd+Shift+O"),
                ),
                ("Union", Action::Boolean(0), None),
                ("Subtract", Action::Boolean(1), None),
                ("Intersect", Action::Boolean(2), None),
                ("Exclude", Action::Boolean(3), None),
                ("Make compound path", Action::Compound, None),
                ("Group", Action::Group, Some("Cmd+G")),
                ("Ungroup", Action::Ungroup, Some("Cmd+Shift+G")),
                ("Use as mask / remove mask", Action::Mask, None),
                ("Replace image…", Action::ReplaceImage, None),
                ("Size to original size", Action::OriginalSize, None),
                ("Trim transparency", Action::TrimImage, None),
            ],
        ),
        (
            "Text",
            vec![
                ("Change typeface…", Action::Typeface, Some("Cmd+T")),
                ("Edit text", Action::EditText, None),
                ("Bold", Action::Bold, Some("Cmd+B")),
                ("Italic", Action::Italic, Some("Cmd+I")),
                ("Underline", Action::Underline, Some("Cmd+U")),
                ("Strikethrough", Action::Strike, None),
                ("Bigger", Action::FontSize(1.0), Some("Cmd+Shift+Equal")),
                ("Smaller", Action::FontSize(-1.0), Some("Cmd+Minus")),
                ("Use font kerning", Action::Kern(true), None),
                ("Disable kerning", Action::Kern(false), None),
                ("Tighten", Action::Tracking(-0.1), None),
                ("Loosen", Action::Tracking(0.1), None),
                ("Use standard ligatures", Action::Ligatures(true), None),
                ("Disable ligatures", Action::Ligatures(false), None),
                ("Default baseline", Action::Baseline(0), None),
                ("Raise baseline", Action::Baseline(1), None),
                ("Lower baseline", Action::Baseline(2), None),
                ("Superscript position", Action::Baseline(3), None),
                ("Subscript position", Action::Baseline(4), None),
                ("Original case", Action::TextCase(0), None),
                ("Uppercase", Action::TextCase(1), None),
                ("Lowercase", Action::TextCase(2), None),
                ("Title case", Action::TextCase(3), None),
                ("No list", Action::List(0), None),
                ("Bulleted list", Action::List(1), None),
                ("Numbered list", Action::List(2), None),
                ("Align left", Action::TextAlign(0), None),
                ("Center", Action::TextAlign(1), None),
                ("Align right", Action::TextAlign(2), None),
                ("Justify", Action::TextAlign(3), None),
                ("Align top", Action::Vertical(0), None),
                ("Center vertically", Action::Vertical(1), None),
                ("Align bottom", Action::Vertical(2), None),
                ("Small caps", Action::OpenType(0), None),
                ("Tabular figures", Action::OpenType(1), None),
                ("Slashed zero", Action::OpenType(2), None),
                ("Discretionary ligatures", Action::OpenType(3), None),
                ("Automatic direction", Action::Direction(0), None),
                ("Left to right", Action::Direction(1), None),
                ("Right to left", Action::Direction(2), None),
                (
                    "Convert to outlines",
                    Action::ConvertPath,
                    Some("Cmd+Shift+O"),
                ),
            ],
        ),
        (
            "Prototype",
            vec![
                ("Edit interactions", Action::Prototype, Some("Cmd+Shift+P")),
                ("Preview flow", Action::PreviewPrototype, Some("Cmd+Enter")),
                ("Share prototype…", Action::SharePrototype, None),
            ],
        ),
        (
            "Arrange",
            vec![
                ("Bring to front", Action::Front, None),
                ("Send to back", Action::Back, None),
                ("Align left", Action::Align(0), None),
                ("Center horizontally", Action::Align(1), None),
                ("Align right", Action::Align(2), None),
                ("Align top", Action::Align(3), None),
                ("Center vertically", Action::Align(4), None),
                ("Align bottom", Action::Align(5), None),
            ],
        ),
        (
            "View",
            vec![
                ("Fit artboard", Action::Fit, None),
                ("Actual size", Action::Zoom(1.0), None),
                ("Toggle grid", Action::ToggleGrid, None),
                ("Toggle rulers", Action::ToggleRulers, None),
                ("Appearance: Light", Action::Theme(ThemeMode::Light), None),
                (
                    "Appearance: High Contrast Light",
                    Action::Theme(ThemeMode::HighContrastLight),
                    None,
                ),
                ("Appearance: Dark", Action::Theme(ThemeMode::Dark), None),
                ("Appearance: System", Action::Theme(ThemeMode::System), None),
                (
                    "Previous document",
                    Action::NextTab(-1),
                    Some("Cmd+Shift+BracketLeft"),
                ),
                (
                    "Next document",
                    Action::NextTab(1),
                    Some("Cmd+Shift+BracketRight"),
                ),
                ("Previous page", Action::Page(-1), None),
                ("Next page", Action::Page(1), None),
                ("Previous artboard", Action::Board(-1), None),
                ("Next artboard", Action::Board(1), None),
            ],
        ),
        (
            "Help",
            vec![
                ("Keyboard shortcuts", Action::Help, None),
                ("Welcome to SUIZA Canvas", Action::Welcome, None),
            ],
        ),
    ]
}

#[cfg(target_os = "macos")]
struct NativeMenu {
    _menu: muda::Menu,
    events: std::sync::mpsc::Receiver<muda::MenuId>,
    items: Vec<(muda::MenuItem, Action)>,
    recent_menu: muda::Submenu,
    recent_items: Vec<(muda::MenuItem, Option<std::path::PathBuf>)>,
    recent_paths: Vec<std::path::PathBuf>,
}
#[cfg(target_os = "macos")]
impl NativeMenu {
    fn new(ctx: egui::Context) -> Result<Self, muda::Error> {
        use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
        let menu = Menu::new();
        let app = Submenu::new("SUIZA Canvas", true);
        app.append(&PredefinedMenuItem::about(Some("About SUIZA Canvas"), None))?;
        app.append(&PredefinedMenuItem::separator())?;
        app.append(&PredefinedMenuItem::hide(None))?;
        app.append(&PredefinedMenuItem::hide_others(None))?;
        app.append(&PredefinedMenuItem::separator())?;
        let quit = MenuItem::new("Quit SUIZA Canvas", true, "Cmd+Q".parse().ok());
        app.append(&quit)?;
        menu.append(&app)?;
        let groups = menu_groups();
        let recent_menu = Submenu::new("Open Recent", true);
        let mut items = vec![(quit, Action::Quit)];
        for (name, commands) in groups {
            let submenu = Submenu::new(name, true);
            for (i, (title, commands)) in menu_sections(name, commands).into_iter().enumerate() {
                if i > 0 {
                    submenu.append(&PredefinedMenuItem::separator())?;
                }
                if let Some(title) = title {
                    let child = Submenu::new(title, true);
                    for (label, action, shortcut) in commands {
                        let item =
                            MenuItem::new(label, true, shortcut.and_then(|s| s.parse().ok()));
                        child.append(&item)?;
                        items.push((item, action));
                    }
                    submenu.append(&child)?;
                } else {
                    for (label, action, shortcut) in commands {
                        let item =
                            MenuItem::new(label, true, shortcut.and_then(|s| s.parse().ok()));
                        submenu.append(&item)?;
                        if matches!(action, Action::Open) {
                            submenu.append(&recent_menu)?;
                        }
                        items.push((item, action));
                    }
                }
            }
            menu.append(&submenu)?;
        }
        menu.init_for_nsapp();
        let (sender, events) = std::sync::mpsc::channel();
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            let _ = sender.send(event.id);
            ctx.request_repaint();
        }));
        Ok(Self {
            _menu: menu,
            events,
            items,
            recent_menu,
            recent_items: Vec::new(),
            recent_paths: Vec::new(),
        })
    }
    fn sync_recent(&mut self, paths: &[std::path::PathBuf]) {
        if self.recent_paths == paths && !self.recent_items.is_empty() {
            return;
        }
        while self.recent_menu.remove_at(0).is_some() {}
        self.recent_items.clear();
        self.recent_paths = paths.to_vec();
        if paths.is_empty() {
            let item = muda::MenuItem::new("No Recent Documents", false, None);
            let _ = self.recent_menu.append(&item);
            self.recent_items.push((item, None));
        } else {
            for path in paths {
                let item = muda::MenuItem::new(path.display().to_string(), true, None);
                let _ = self.recent_menu.append(&item);
                self.recent_items.push((item, Some(path.clone())));
            }
            let _ = self
                .recent_menu
                .append(&muda::PredefinedMenuItem::separator());
            let item = muda::MenuItem::new("Clear Recent Documents", true, None);
            let _ = self.recent_menu.append(&item);
            self.recent_items.push((item, None));
        }
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Document defaults remain independent of the application UI font.
    fonts.families.insert(
        egui::FontFamily::Name("Document Regular".into()),
        fonts.families[&egui::FontFamily::Proportional].clone(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("Document Mono".into()),
        fonts.families[&egui::FontFamily::Monospace].clone(),
    );
    fonts.font_data.insert(
        "document-bold".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/UI-Bold.ttf")).into(),
    );
    fonts.families.insert(
        egui::FontFamily::Name("Document Bold".into()),
        vec!["document-bold".into()],
    );
    fonts.font_data.insert(
        "inter-regular".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "ui-bold".into(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Bold.ttf")).into(),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "inter-regular".into());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "inter-regular".into());
    let mut bold = vec!["ui-bold".into()];
    bold.extend(fonts.families[&egui::FontFamily::Name("Document Regular".into())].clone());
    fonts
        .families
        .insert(egui::FontFamily::Name("UI Bold".into()), bold);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("font_definitions"), fonts.clone()));
    ctx.set_fonts(fonts);
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Suiza",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("SUIZA Canvas")
                .with_icon(
                    eframe::icon_data::from_png_bytes(include_bytes!("../assets/suiza-icon.png"))
                        .expect("valid bundled SUIZA icon"),
                )
                .with_inner_size([1320.0, 860.0])
                .with_min_inner_size([960.0, 640.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);
            configure_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::light());
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.menu_corner_radius = egui::CornerRadius::same(12);
            style.visuals.window_corner_radius = egui::CornerRadius::same(12);
            style.visuals.popup_shadow = egui::epaint::Shadow {
                offset: [0, 5],
                blur: 22,
                spread: 1,
                color: Color32::from_black_alpha(35),
            };
            style.spacing.item_spacing = Vec2::new(6.0, 6.0);
            style.spacing.button_padding = Vec2::new(8.0, 5.0);
            style.spacing.interact_size.y = 24.0;
            style.visuals.panel_fill = Color32::WHITE;
            style.visuals.override_text_color = Some(Color32::from_rgb(58, 59, 72));
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(239, 239, 244);
            style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(239, 239, 244);
            style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(4);
            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(229, 225, 245);
            style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(229, 225, 245);
            style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(12.0));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(12.0));
            style.visuals.selection.bg_fill = Color32::from_rgb(222, 236, 255);
            style.visuals.widgets.noninteractive.bg_stroke =
                Stroke::new(1.0_f32, Color32::from_gray(232));
            style.visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
            cc.egui_ctx.set_style(style);
            let mut studio = Studio {
                welcome: true,
                ..Studio::default()
            };
            if let Some(storage) = cc.storage {
                if let Some(json) = storage.get_string("suiza_plugins_v1") {
                    if json.len() <= 8 * 1024 * 1024 {
                        if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(&json) {
                            studio.plugins_ui.installed = values
                                .into_iter()
                                .take(100)
                                .filter_map(|value| {
                                    serde_json::to_vec(&value)
                                        .ok()
                                        .and_then(|bytes| plugins::Plugin::parse(&bytes).ok())
                                })
                                .collect();
                        }
                    }
                }
                if let Some(json) = storage.get_string("suiza_swatches") {
                    studio.swatches = serde_json::from_str(&json).unwrap_or(studio.swatches);
                }
                if let Some(json) = storage.get_string("suiza_theme") {
                    studio.theme = serde_json::from_str(&json).unwrap_or_default();
                }
                if let Some(json) = storage.get_string("suiza_libraries_v1") {
                    if let Ok(libraries) = serde_json::from_str::<Vec<assets::Library>>(&json) {
                        studio.assets_ui.libraries = libraries
                            .into_iter()
                            .filter(|l| {
                                serde_json::to_string(&l.document)
                                    .ok()
                                    .is_some_and(|s| Document::from_json(&s).is_ok())
                            })
                            .collect();
                    }
                }
                if let Some(json) = storage.get_string("suiza_recent") {
                    studio.recent = serde_json::from_str(&json).unwrap_or_default();
                }
                if let Some(json) = storage.get_string("suiza_session_v1") {
                    if let Err(e) = studio.restore_session(&json) {
                        studio.status = format!("Session recovery failed: {e}");
                    }
                }
                studio.welcome = true;
            }
            for path in std::env::args_os().skip(1).map(std::path::PathBuf::from) {
                if path
                    .extension()
                    .is_some_and(|e| e == "suiza-ui" || e == "json")
                {
                    studio.open_path(path);
                }
            }
            cc.egui_ctx.data_mut(|d| {
                d.insert_temp(egui::Id::new("suiza_swatches"), studio.swatches.clone())
            });
            #[cfg(target_os = "macos")]
            {
                studio.native_menu = Some(NativeMenu::new(cc.egui_ctx.clone())?);
            }
            Ok(Box::new(studio))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_document_save_close_reopen_and_failed_open_preserve_work() {
        let path =
            std::env::temp_dir().join(format!("suiza-native-save-{}.suiza-ui", std::process::id()));
        let mut app = Studio::default();
        app.doc.pages[0].artboards[0].name = "Saved native frame".into();
        app.doc.pages[0].artboards[0].layers[1].label = Some("Saved native label".into());
        let expected = app.doc.clone();
        app.save_to_path(path.clone()).unwrap();
        assert!(app.saved_document == expected);
        app.close_tab(app.active_tab);
        app.open_path(path.clone());
        assert!(app.doc == expected);
        assert_eq!(app.document_path.as_ref(), Some(&path));
        assert_eq!(app.recent.first(), Some(&path));
        let old_path = app.document_path.clone();
        let damaged = path.with_extension("broken.suiza-ui");
        std::fs::write(&damaged, b"PKbroken").unwrap();
        app.open_path(damaged.clone());
        assert!(app.doc == expected);
        assert_eq!(app.document_path, old_path);
        assert!(app.status.starts_with("Open failed:"));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(damaged).unwrap();
    }
    #[test]
    fn save_command_reopens_exact_edits_and_failed_save_stays_dirty() {
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        let root = std::env::temp_dir().join(format!("suiza-save-qa-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("Edited.json");
        app.document_path = Some(path.clone());
        app.doc.name = "Persistence QA".into();
        app.doc.pages[0].artboards[0].layers[0].x = -73.0;
        app.doc.pages[0].artboards[0].layers[0].label = Some("Saved layer label".into());
        app.apply_action(Action::Save, &ctx);
        assert_eq!(app.status, "Document saved");
        let reopened = Document::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(reopened == app.doc);
        let mut restarted = Studio::default();
        restarted.open_path(path.clone());
        assert_eq!(restarted.recent.first(), Some(&path));
        assert!(restarted.doc == reopened);
        assert_eq!(restarted.doc.pages[0].artboards[0].layers[0].x, -73.0);
        restarted.doc.pages[0].artboards[0].layers[0].x = 41.0;
        restarted.apply_action(Action::Save, &ctx);
        assert!(
            Document::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap() == restarted.doc
        );
        let saved = restarted.saved_document.clone();
        restarted.doc.name = "Unsaved change".into();
        assert!(restarted
            .save_to_path(root.join("missing/failed.json"))
            .is_err());
        assert!(restarted.saved_document == saved);
        assert_eq!(restarted.document_path, Some(path.clone()));
        assert!(Document::from_json(&std::fs::read_to_string(path).unwrap()).unwrap() == saved);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn restart_restores_all_tabs_paths_and_unsaved_progress() {
        let mut app = Studio::default();
        app.doc.name = "First unsaved document".into();
        app.doc.pages[0].artboards[0].layers[0].x = 51.0;
        app.open_in_tab(
            Document::default(),
            Some(std::path::PathBuf::from("saved-document.json")),
        );
        app.doc.name = "Second document edits".into();
        let json = app.session_json().unwrap();
        let mut restarted = Studio::default();
        restarted.restore_session(&json).unwrap();
        assert_eq!(restarted.doc.name, "Second document edits");
        assert!(restarted.doc != restarted.saved_document);
        assert_eq!(
            restarted.document_path,
            Some(std::path::PathBuf::from("saved-document.json"))
        );
        restarted.switch_tab(0);
        assert_eq!(restarted.doc.name, "First unsaved document");
        assert_eq!(restarted.doc.pages[0].artboards[0].layers[0].x, 51.0);
        assert!(restarted.document_path.is_none());
        assert!(!restarted.welcome);
        assert!(restarted.restore_session("broken JSON").is_err());
        assert_eq!(restarted.doc.name, "First unsaved document");
    }
    #[test]
    fn hex_color_input_validates_without_panicking() {
        assert_eq!(parse_hex_color("535DFF"), Some([83, 93, 255]));
        assert_eq!(parse_hex_color("ffffff"), Some([255, 255, 255]));
        for invalid in ["", "123", "GGGGGG", "é12345", "1234567"] {
            assert_eq!(parse_hex_color(invalid), None);
        }
    }
    #[test]
    fn exports_are_valid_in_all_supported_formats() {
        let mut layer = Layer::new(Kind::Ellipse, 0.0, 0.0);
        layer.width = 24.0;
        layer.height = 16.0;
        let root = std::env::temp_dir().join(format!("suiza-export-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        for (format, kind) in [
            ("PNG", image::ImageFormat::Png),
            ("JPG", image::ImageFormat::Jpeg),
            ("TIFF", image::ImageFormat::Tiff),
            ("WEBP", image::ImageFormat::WebP),
        ] {
            let path = root.join(format!("asset.{format}"));
            export_layer(&layer, &path, format, 2, false, &[]).unwrap();
            let bytes = std::fs::read(path).unwrap();
            let decoded = image::load_from_memory_with_format(&bytes, kind).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (48, 32));
        }
        let png = rasterize_layer(&layer, 1, false, &[]).unwrap();
        assert_eq!(png.get_pixel(0, 0)[3], 0);
        assert_eq!(png.get_pixel(12, 8)[3], 255);
        let pdf = root.join("asset.pdf");
        export_layer(&layer, &pdf, "PDF", 1, false, &[]).unwrap();
        let bytes = std::fs::read(pdf).unwrap();
        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF\n"));
        let svg = root.join("asset.svg");
        export_layer(&layer, &svg, "SVG", 1, false, &[]).unwrap();
        assert!(std::fs::read_to_string(svg).unwrap().contains("<ellipse"));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn typography_and_effects_change_rendered_pixels_and_round_trip() {
        let mut layer = Layer::new(Kind::Text, 0.0, 0.0);
        layer.name = "SUIZA".into();
        layer.width = 160.0;
        layer.height = 50.0;
        let original = rasterize_layer(&layer, 1, false, &[]).unwrap();
        layer.typography.bold = true;
        layer.typography.size = Some(24.0);
        layer.typography.align = 2;
        assert_ne!(original, rasterize_layer(&layer, 1, false, &[]).unwrap());
        layer.drop_shadow = Some(ShadowStyle::default());
        layer.inner_shadow = Some(ShadowStyle::default());
        let shadowed = rasterize_layer(&layer, 1, false, &[]).unwrap();
        assert!(shadowed.width() > 160);
        layer.effects.push(EffectStyle {
            zoom: false,
            kind: "Fade".into(),
            amount: 100.0,
            enabled: true,
        });
        assert!(rasterize_layer(&layer, 1, false, &[])
            .unwrap()
            .pixels()
            .all(|p| p[3] == 0));
        let json = serde_json::to_string(&layer).unwrap();
        let restored: Layer = serde_json::from_str(&json).unwrap();
        assert!(restored == layer);
        assert_eq!(encode_base64(b"SUIZA"), "U1VJWkE=");
    }
    #[test]
    fn document_round_trip() {
        let doc = Document::default();
        let encoded = serde_json::to_string(&doc).unwrap();
        let decoded: Document = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.pages[0].artboards[0].layers.len(), 19);
        assert_eq!(decoded.pages[0].artboards[0].layers[1].name, "atelier");
    }
    #[test]
    fn resizing_preserves_opposite_corner_and_minimum_size() {
        let original = Layer::new(Kind::Rectangle, 40.0, 60.0);
        let mut layer = original.clone();
        resize_layer(&mut layer, &original, 0, Vec2::new(500.0, 500.0), false);
        assert_eq!(layer.width, 8.0);
        assert_eq!(layer.height, 8.0);
        assert_eq!(layer.x + layer.width, original.x + original.width);
        assert_eq!(layer.y + layer.height, original.y + original.height);
        resize_layer(&mut layer, &original, 3, Vec2::new(13.0, 17.0), true);
        assert_eq!(layer.x, original.x);
        assert_eq!(layer.y, original.y);
        assert_eq!((layer.x + layer.width) % 8.0, 0.0);
        assert_eq!((layer.y + layer.height) % 8.0, 0.0);
    }
    #[test]
    fn original_documents_receive_default_corner_radius() {
        let mut value = serde_json::to_value(Layer::new(Kind::Rectangle, 0.0, 0.0)).unwrap();
        value.as_object_mut().unwrap().remove("radius");
        let layer: Layer = serde_json::from_value(value).unwrap();
        assert_eq!(layer.radius, 8.0);
    }
    #[test]
    fn undo_restores_document() {
        let mut app = Studio::default();
        app.checkpoint();
        app.doc.pages[0].artboards[0].layers.clear();
        app.undo();
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), 19);
        assert!(app.selected.is_none());
    }
    #[test]
    fn legacy_documents_migrate_to_page_and_artboard() {
        let json = serde_json::json!({"layers":[Layer::new(Kind::Rectangle,0.0,0.0)]}).to_string();
        let doc = Document::from_json(&json).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.pages[0].artboards[0].layers.len(), 1);
        assert!(Document::from_json(r#"{"name":"Broken","pages":[]}"#).is_err());
    }
    #[test]
    fn undo_clamps_navigation_after_removing_a_page() {
        let mut app = Studio::default();
        app.checkpoint();
        app.doc.pages.push(Page {
            name: "New page".into(),
            artboards: vec![Artboard {
                prototype_id: String::new(),
                frame: FrameStyle::default(),
                name: "New board".into(),
                layers: vec![],
            }],
        });
        app.page = 2;
        app.undo();
        assert_eq!(app.page, 1);
        assert_eq!(app.artboard, 0);
        assert!(app.selected.is_none());
    }
    #[test]
    fn nested_tree_stays_inside_sidebar_at_supported_sizes() {
        for (width, height) in [(960.0, 640.0), (1320.0, 860.0), (1920.0, 1080.0)] {
            let ctx = egui::Context::default();
            configure_fonts(&ctx);
            let mut app = Studio::default();
            app.grid = true;
            app.rulers = true;
            app.doc.pages[0].artboards[0].layers[0].name =
                "A very long layer name that must never expand the sidebar".repeat(4);
            for _ in 0..3 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.show(ctx),
                );
            }
            assert!(
                (app.canvas_bounds.left() - 272.0).abs() < 2.0,
                "sidebar overflow: {:?}",
                app.canvas_bounds
            );
            assert!(
                (app.canvas_bounds.right() - (width - 264.0)).abs() < 2.0,
                "inspector overflow: {:?}",
                app.canvas_bounds
            );
        }
    }
    #[test]
    fn commands_respect_locks_and_pages_stay_independent() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut app = Studio::default();
        app.doc.pages[0].artboards[0].layers[app.selected.unwrap()].group = None;
        app.apply_action(Action::Lock, &ctx);
        let count = app.doc.pages[0].artboards[0].layers.len();
        app.apply_action(Action::Delete, &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), count);
        app.apply_action(Action::Lock, &ctx);
        app.apply_action(Action::Duplicate, &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), count + 1);
        app.apply_action(Action::AddPage, &ctx);
        assert_eq!(app.page, 2);
        assert!(app.selected.is_none());
        assert!(app.doc.pages[2].artboards[0].layers.is_empty());
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), count + 1);
        let json = serde_json::to_string(&app.doc).unwrap();
        let restored = Document::from_json(&json).unwrap();
        assert_eq!(restored.pages.len(), 3);
        assert_eq!(restored.pages[0].artboards.len(), 2);
    }
    #[test]
    fn tabs_keep_documents_history_and_view_independent() {
        let mut app = Studio::default();
        let ctx = egui::Context::default();
        app.doc.name = "First".into();
        app.zoom = 1.5;
        app.pan = Vec2::new(20.0, 30.0);
        app.checkpoint();
        app.apply_action(Action::New, &ctx);
        assert_eq!(app.tabs.len(), 2);
        app.doc.name = "Second".into();
        app.zoom = 0.5;
        app.switch_tab(0);
        assert_eq!(app.doc.name, "First");
        assert_eq!(app.zoom, 1.5);
        assert_eq!(app.pan, Vec2::new(20.0, 30.0));
        assert_eq!(app.history.len(), 1);
        app.switch_tab(1);
        assert_eq!(app.doc.name, "Second");
        assert_eq!(app.zoom, 0.5);
        assert!(app.history.is_empty());
        app.close_tab(1);
        assert_eq!(app.doc.name, "First");
        assert_eq!(app.tabs.len(), 1);
    }
    #[test]
    fn context_actions_copy_paste_group_and_mask_work() {
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        app.doc.pages[0].artboards[0].layers = vec![
            Layer::new(Kind::Rectangle, 0.0, 0.0),
            Layer::new(Kind::Ellipse, 10.0, 10.0),
        ];
        app.selected = Some(1);
        app.selection = vec![0, 1];
        app.apply_action(Action::Group, &ctx);
        assert_eq!(
            app.doc.pages[0].artboards[0].layers[0].group,
            app.doc.pages[0].artboards[0].layers[1].group
        );
        app.apply_action(Action::Ungroup, &ctx);
        assert!(app.doc.pages[0].artboards[0]
            .layers
            .iter()
            .all(|l| l.group.is_none()));
        app.selection.clear();
        app.apply_action(Action::CopyLayer, &ctx);
        app.apply_action(Action::PasteLayer, &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), 3);
        app.selected = Some(1);
        app.selection.clear();
        app.apply_action(Action::Mask, &ctx);
        let layers = &app.doc.pages[0].artboards[0].layers;
        assert!(layers[0].mask);
        let asset = rasterize_layer(&layers[1], 1, false, &layers[..1]).unwrap();
        assert_eq!(asset.get_pixel(0, 0)[3], 0);
        assert!(asset.get_pixel(90, 40)[3] > 0);
    }
    #[test]
    fn corner_styles_and_individual_values_change_geometry() {
        let mut layer = Layer::new(Kind::Rectangle, 0.0, 0.0);
        layer.width = 100.0;
        layer.height = 100.0;
        layer.radius = 20.0;
        assert!(!shape_contains(&layer, Pos2::new(1.0, 1.0)));
        layer.corners = Some([0.0, 20.0, 20.0, 20.0]);
        assert!(shape_contains(&layer, Pos2::new(1.0, 1.0)));
        layer.corners = None;
        layer.corner_style = "Inside Arc".into();
        assert!(!shape_contains(&layer, Pos2::new(10.0, 10.0)));
        layer.corner_style = "Angled".into();
        assert!(shape_contains(&layer, Pos2::new(10.0, 10.0)));
        let restored =
            Document::from_json(&serde_json::to_string(&Studio::default().doc).unwrap()).unwrap();
        assert_eq!(restored.pages.len(), 2);
    }
    #[test]
    fn embedded_images_trim_and_round_trip() {
        let source = image::RgbaImage::from_fn(10, 10, |x, y| {
            if (2..8).contains(&x) && (2..8).contains(&y) {
                image::Rgba([255, 20, 30, 255])
            } else {
                image::Rgba([0, 0, 0, 0])
            }
        });
        let mut encoded = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(source)
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let mut layer = Layer::new(Kind::Image, 0.0, 0.0);
        layer.width = 10.0;
        layer.height = 10.0;
        layer.image_data = Some(encoded.into_inner());
        assert_eq!(
            rasterize_layer(&layer, 1, true, &[]).unwrap().dimensions(),
            (6, 6)
        );
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        app.doc.pages[0].artboards[0].layers = vec![layer];
        app.selected = Some(0);
        app.apply_action(Action::TrimImage, &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers[0].width, 6.0);
        assert_eq!(app.doc.pages[0].artboards[0].layers[0].x, 2.0);
        Document::from_json(&serde_json::to_string(&app.doc).unwrap()).unwrap();
    }
}

fn vector_inspector(
    ui: &mut egui::Ui,
    layer: &mut Layer,
    editing: &mut Option<usize>,
    index: usize,
    selected: &mut Option<usize>,
) {
    section_heading(ui, "Vector path");
    let path = layer.path.as_mut().unwrap();
    ui.horizontal(|ui| {
        ui.checkbox(&mut path.closed, "Closed");
        ui.checkbox(&mut path.fill, "Fill");
    });
    ui.horizontal(|ui| {
        ui.checkbox(&mut path.stroke, "Stroke");
        ui.add_sized(
            [104.0, 26.0],
            egui::DragValue::new(&mut path.stroke_width)
                .range(0.0..=200.0)
                .prefix("Width "),
        );
    });
    let mut stroke_color = Color32::from_rgb(
        path.stroke_color[0],
        path.stroke_color[1],
        path.stroke_color[2],
    );
    ui.horizontal(|ui| {
        ui.label("Stroke color");
        if ui.color_edit_button_srgb(&mut path.stroke_color).changed() {
            stroke_color = Color32::from_rgb(
                path.stroke_color[0],
                path.stroke_color[1],
                path.stroke_color[2],
            );
        }
    });
    let _ = stroke_color;
    ui.horizontal(|ui| {
        if ui.button("Edit nodes").clicked() {
            *editing = Some(index);
        }
        if ui.button("Reverse").clicked() {
            path.reverse();
            *selected = None;
        }
    });
    ui.small("Click to add · drag to curve · click first point to close");
    if !path.contours.is_empty() {
        egui::ComboBox::from_id_salt("compound_contour")
            .width(208.0)
            .selected_text("Active contour")
            .show_ui(ui, |ui| {
                for i in 0..path.contours.len() {
                    if ui.button(format!("Edit contour {}", i + 2)).clicked() {
                        std::mem::swap(&mut path.nodes, &mut path.contours[i]);
                        *selected = None;
                        ui.close_menu();
                    }
                }
            });
    }
    if let Some(i) = selected.filter(|i| *i < path.nodes.len()) {
        let node = &mut path.nodes[i];
        egui::ComboBox::from_id_salt("node_mode")
            .width(208.0)
            .selected_text(format!("{:?}", node.mode))
            .show_ui(ui, |ui| {
                for (mode, label) in [
                    (vector::NodeMode::Corner, "Corner / independent"),
                    (vector::NodeMode::Smooth, "Smooth"),
                    (vector::NodeMode::Symmetric, "Symmetric"),
                ] {
                    if ui.selectable_label(node.mode == mode, label).clicked() {
                        node.set_mode(mode);
                    }
                }
            });
        let mut x = node.point[0] * layer.width;
        let mut y = node.point[1] * layer.height;
        let mut changed = false;
        ui.horizontal(|ui| {
            changed |= ui
                .add_sized(
                    [102.0, 26.0],
                    egui::DragValue::new(&mut x).prefix("X ").speed(0.1),
                )
                .changed();
            changed |= ui
                .add_sized(
                    [102.0, 26.0],
                    egui::DragValue::new(&mut y).prefix("Y ").speed(0.1),
                )
                .changed();
        });
        if changed {
            node.move_anchor([x / layer.width, y / layer.height]);
        }
        ui.collapsing("Control handles", |ui| {
            for outgoing in [false, true] {
                let handle = if outgoing {
                    node.outgoing
                } else {
                    node.incoming
                };
                let mut x = handle[0] * layer.width;
                let mut y = handle[1] * layer.height;
                let mut changed = false;
                ui.small(if outgoing { "Outgoing" } else { "Incoming" });
                ui.horizontal(|ui| {
                    changed |= ui
                        .add_sized(
                            [102.0, 26.0],
                            egui::DragValue::new(&mut x).prefix("X ").speed(0.1),
                        )
                        .changed();
                    changed |= ui
                        .add_sized(
                            [102.0, 26.0],
                            egui::DragValue::new(&mut y).prefix("Y ").speed(0.1),
                        )
                        .changed();
                });
                if changed {
                    node.move_handle([x / layer.width, y / layer.height], outgoing, false);
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Insert midpoint").clicked() {
                path.insert(i, 0.5);
            }
            if ui.button("Delete node").clicked() {
                path.nodes.remove(i);
                *selected = None;
            }
        });
    }
    ui.add_space(8.0);
}

fn export_board(
    board: &Artboard,
    path: &std::path::Path,
    format: &str,
    scale: u32,
) -> Result<(), String> {
    let w = board.frame.width;
    let h = board.frame.height;
    if scale == 0
        || (w * scale as f32).ceil() as u64 * (h * scale as f32).ceil() as u64 > 16_000_000
    {
        return Err("Frame is too large; lower export scale".into());
    }
    let [red, green, blue] = board.frame.background;
    let radius = board.frame.radius;
    if format == "SVG" {
        let temp =
            std::env::temp_dir().join(format!("suiza-frame-export-{}.svg", std::process::id()));
        let mut body = String::new();
        for (i, layer) in board
            .layers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.visible && !l.mask)
        {
            export_layer(layer, &temp, "SVG", 1, false, &board.layers[..i])?;
            let svg = std::fs::read_to_string(&temp).map_err(|e| e.to_string())?;
            let start = svg.find('>').ok_or("Invalid SVG")? + 1;
            let end = svg.rfind("</svg>").ok_or("Invalid SVG")?;
            let padding = effect_padding(layer);
            body.push_str(&format!(
                "<g transform=\"translate({} {})\">{}</g>",
                layer.x - padding,
                layer.y - padding,
                &svg[start..end]
            ));
        }
        let _ = std::fs::remove_file(temp);
        let background=format!("<rect width=\"{w}\" height=\"{h}\" rx=\"{radius}\" fill=\"#{red:02X}{green:02X}{blue:02X}\"/>");
        let content = if board.frame.clip {
            format!("<g clip-path=\"url(#frame)\">{body}</g>")
        } else {
            body
        };
        return std::fs::write(path,format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {w} {h}\"><defs><clipPath id=\"frame\"><rect width=\"{w}\" height=\"{h}\" rx=\"{radius}\"/></clipPath></defs>{background}{content}</svg>",w*scale as f32,h*scale as f32)).map_err(|e|e.to_string());
    }
    let mut img = image::RgbaImage::from_pixel(
        (w * scale as f32).ceil() as u32,
        (h * scale as f32).ceil() as u32,
        image::Rgba([red, green, blue, 255]),
    );
    for (i, l) in board
        .layers
        .iter()
        .enumerate()
        .filter(|(_, l)| l.visible && !l.mask)
    {
        let asset = rasterize_layer(l, scale, false, &board.layers[..i])?;
        let pad = effect_padding(l);
        image::imageops::overlay(
            &mut img,
            &asset,
            ((l.x - pad) * scale as f32).round() as i64,
            ((l.y - pad) * scale as f32).round() as i64,
        );
    }
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let mut layer = Layer::new(Kind::Image, 0.0, 0.0);
    layer.width = w;
    layer.height = h;
    layer.image_data = Some(png.into_inner());
    export_layer(&layer, path, format, scale, false, &[])
}

type MenuCommand = (&'static str, Action, Option<&'static str>);
fn menu_sections(
    name: &str,
    commands: Vec<MenuCommand>,
) -> Vec<(Option<&'static str>, Vec<MenuCommand>)> {
    let category = |a: Action| -> &'static str {
        match (name, a) {
            ("Text", Action::Typeface | Action::EditText) => "@face",
            ("Text", Action::Bold | Action::Italic | Action::Underline | Action::Strike) => {
                "@style"
            }
            ("Text", Action::FontSize(_)) => "@size",
            ("Text", Action::Kern(_) | Action::Tracking(_)) => "Kern",
            ("Text", Action::Ligatures(_)) => "Ligature",
            ("Text", Action::Baseline(_)) => "Baseline",
            ("Text", Action::TextCase(_)) => "Transformation",
            ("Text", Action::List(_)) => "List Type",
            ("Text", Action::TextAlign(_) | Action::Vertical(_)) => "Align",
            ("Text", Action::OpenType(_)) => "OpenType Features",
            ("Text", Action::Direction(_)) => "Writing Direction",
            ("Text", Action::ConvertPath) => "@outline",
            ("Layer", Action::Boolean(_) | Action::Compound) => "Boolean Operations",
            ("Layer", Action::ReplaceImage | Action::OriginalSize | Action::TrimImage) => "Image",
            ("Arrange", Action::Align(_)) => "Align to Frame",
            ("View", Action::Theme(_)) => "Appearance",
            ("View", Action::NextTab(_) | Action::Page(_) | Action::Board(_)) => "Navigation",
            ("File", Action::Welcome) => "@browser",
            ("File", Action::ImportImage | Action::ExportSelected) => "@assets",
            ("Edit", Action::Undo | Action::Redo) => "@history",
            _ => "@commands",
        }
    };
    let mut chunks: Vec<(&str, Vec<MenuCommand>)> = vec![];
    for command in commands {
        let group = category(command.1);
        if chunks.last().is_some_and(|(label, _)| *label == group) {
            chunks.last_mut().unwrap().1.push(command);
        } else {
            chunks.push((group, vec![command]));
        }
    }
    chunks
        .into_iter()
        .map(|(name, items)| {
            (
                if name.starts_with('@') {
                    None
                } else {
                    Some(name)
                },
                items,
            )
        })
        .collect()
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    #[test]
    fn all_object_types_can_be_dragged_onto_pasteboard_and_back() {
        for (kind, is_path) in [
            (Kind::Rectangle, true),
            (Kind::Rectangle, false),
            (Kind::Ellipse, false),
            (Kind::Text, false),
            (Kind::Image, false),
        ] {
            let ctx = egui::Context::default();
            configure_fonts(&ctx);
            let mut app = Studio::default();
            app.welcome = false;
            let mut layer = Layer::new(kind, 220.0, 180.0);
            layer.width = 100.0;
            layer.height = 80.0;
            layer.radius = 0.0;
            if is_path {
                layer.path = vector::from_layer(&layer);
            }
            app.doc.pages[0].artboards[0].layers = vec![layer];
            app.selected = Some(0);
            if is_path {
                app.editing_path = Some(0);
            }
            app.apply_action(Action::Tool(None), &ctx);
            assert!(app.editing_path.is_none());
            let mut time = 0.0;
            let mut frame = |app: &mut Studio, events: Vec<egui::Event>| {
                time += 0.016;
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(
                            Pos2::ZERO,
                            Vec2::new(1320.0, 860.0),
                        )),
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ctx| app.show(ctx),
                );
            };
            frame(&mut app, vec![]);
            let origin = app.canvas_bounds.center() + app.pan - Vec2::new(320.0, 220.0) * app.zoom;
            let press = origin + Vec2::new(270.0, 220.0) * app.zoom;
            let outside = press - Vec2::new(340.0, 0.0) * app.zoom;
            let event = |pos, pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Default::default(),
            };
            frame(
                &mut app,
                vec![egui::Event::PointerMoved(press), event(press, true)],
            );
            frame(
                &mut app,
                vec![egui::Event::PointerMoved(press - Vec2::new(40.0, 0.0))],
            );
            frame(
                &mut app,
                vec![egui::Event::PointerMoved(press - Vec2::new(160.0, 0.0))],
            );
            frame(&mut app, vec![egui::Event::PointerMoved(outside)]);
            frame(&mut app, vec![]);
            frame(&mut app, vec![event(outside, false)]);
            assert!(app.doc.pages[0].artboards[0].layers[0].x < 0.0);
            assert_eq!(app.selected, Some(0));
            frame(
                &mut app,
                vec![egui::Event::PointerMoved(outside), event(outside, true)],
            );
            frame(
                &mut app,
                vec![egui::Event::PointerMoved(outside + Vec2::new(100.0, 0.0))],
            );
            frame(&mut app, vec![egui::Event::PointerMoved(press)]);
            frame(&mut app, vec![]);
            frame(&mut app, vec![event(press, false)]);
            assert!(app.doc.pages[0].artboards[0].layers[0].x > 100.0);
        }
    }
    #[test]
    fn group_row_selection_moves_members_but_child_selection_moves_only_child() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut app = Studio::default();
        app.welcome = false;
        app.snap = false;
        app.reset_selection();
        let group = "Test group".to_string();
        app.doc.pages[0].artboards[0].layers = [220.0, 360.0]
            .into_iter()
            .map(|x| {
                let mut layer = Layer::new(Kind::Rectangle, x, 180.0);
                layer.width = 100.0;
                layer.height = 60.0;
                layer.group = Some(group.clone());
                layer
            })
            .collect();
        let mut time = 0.0;
        let mut frame = |app: &mut Studio, events| {
            time += 0.016;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1320.0, 860.0))),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| app.show(ctx),
            );
        };
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(&mut app, vec![]);
        frame(&mut app, vec![]);
        let row = ctx
            .data(|d| {
                d.get_temp::<Rect>(
                    egui::Id::new(("group", 0usize, 0usize, &group)).with("row_rect"),
                )
            })
            .unwrap();
        let point = row.center();
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(point), button(point, true)],
        );
        frame(&mut app, vec![button(point, false)]);
        assert_eq!(app.selected_group.as_deref(), Some("Test group"));
        assert_eq!(app.affected_layers(), vec![0, 1]);
        let origin = app.canvas_bounds.center() + app.pan - Vec2::new(320.0, 220.0) * app.zoom;
        let press = origin + Vec2::new(270.0, 210.0) * app.zoom;
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(press), button(press, true)],
        );
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(press + Vec2::new(20.0, 10.0))],
        );
        let end = press + Vec2::new(70.0, 40.0) * app.zoom;
        frame(&mut app, vec![egui::Event::PointerMoved(end)]);
        frame(&mut app, vec![button(end, false)]);
        assert_eq!(app.doc.pages[0].artboards[0].layers[0].x, 290.0);
        assert_eq!(app.doc.pages[0].artboards[0].layers[1].x, 430.0);
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(end), button(end, true)],
        );
        frame(&mut app, vec![button(end, false)]);
        assert!(app.selected_group.is_none());
        assert_eq!(app.affected_layers(), vec![0]);
        frame(&mut app, vec![button(end, true)]);
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(end + Vec2::new(20.0, 30.0))],
        );
        frame(&mut app, vec![button(end + Vec2::new(20.0, 30.0), false)]);
        assert_eq!(app.doc.pages[0].artboards[0].layers[0].x, 310.0);
        assert_eq!(app.doc.pages[0].artboards[0].layers[1].x, 430.0);
        app.select_group(0, group);
        frame(&mut app, vec![]);
        let press = origin + Vec2::new(530.0, 310.0) * app.zoom;
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(press), button(press, true)],
        );
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(press + Vec2::new(20.0, 20.0))],
        );
        let end = press + Vec2::new(220.0, 90.0) * app.zoom;
        frame(&mut app, vec![egui::Event::PointerMoved(end)]);
        frame(&mut app, vec![button(end, false)]);
        let layers = &app.doc.pages[0].artboards[0].layers;
        assert_eq!(layers[0].width, 200.0);
        assert_eq!(layers[1].width, 200.0);
        assert_eq!(layers[0].height, 120.0);
        assert_eq!(layers[1].x, 550.0);
        app.undo();
        assert_eq!(app.doc.pages[0].artboards[0].layers[1].width, 100.0);
    }
    #[test]
    fn marquee_selects_visible_unlocked_objects_without_editing_document() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut app = Studio::default();
        app.snap = false;
        app.reset_selection();
        app.doc.pages[0].artboards[0].layers = [150.0, 270.0, 350.0, 240.0]
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                let mut layer = Layer::new(Kind::Rectangle, x, 180.0);
                layer.width = 100.0;
                layer.height = 60.0;
                layer.locked = i == 2;
                layer.visible = i != 3;
                layer
            })
            .collect();
        let original = app.doc.clone();
        let mut time = 0.0;
        let mut frame = |app: &mut Studio, events| {
            time += 0.016;
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1320.0, 860.0))),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| app.show(ctx),
            );
        };
        let button = |pos, pressed| egui::Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: Default::default(),
        };
        frame(&mut app, vec![]);
        frame(&mut app, vec![]);
        let origin = app.canvas_bounds.center() + app.pan - Vec2::new(320.0, 220.0) * app.zoom;
        let start = origin + Vec2::new(140.0, 170.0) * app.zoom;
        let end = origin + Vec2::new(390.0, 260.0) * app.zoom;
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(start), button(start, true)],
        );
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(start + Vec2::new(20.0, 20.0))],
        );
        frame(&mut app, vec![egui::Event::PointerMoved(end)]);
        frame(&mut app, vec![button(end, false)]);
        assert_eq!(app.selection, vec![0, 1]);
        assert!(app.doc == original);
        assert!(app.history.is_empty());
    }
    #[test]
    fn reparenting_to_empty_frames_preserves_world_position_and_locks() {
        let mut doc = Document::default();
        let boards = &mut doc.pages[0].artboards;
        boards[1].layers.clear();
        boards[0].frame.x = 20.0;
        boards[1].frame.x = 80.0;
        let original = boards[0].layers[0].clone();
        let index = reorder_layer(boards, (0, 0), (1, usize::MAX)).unwrap();
        assert_eq!(index, 0);
        assert_eq!(boards[1].layers[0].x + 720.0 + 80.0, original.x + 20.0);
        assert!(boards[1].layers[0].group.is_none());
        boards[1].layers[0].locked = true;
        assert!(reorder_layer(boards, (1, 0), (0, 0)).is_none());
        assert!(Document::from_json(&serde_json::to_string(&doc).unwrap()).is_ok());
    }
    #[test]
    fn frame_geometry_persists_and_controls_export_dimensions() {
        let mut app = Studio::default();
        let frame = &mut app.doc.pages[0].artboards[0].frame;
        frame.width = 291.0;
        frame.height = 367.0;
        frame.x = -50.0;
        frame.y = 20.0;
        frame.clip = true;
        frame.background = [235, 94, 52];
        let restored = Document::from_json(&serde_json::to_string(&app.doc).unwrap()).unwrap();
        assert_eq!(restored.pages[0].artboards[0].frame.width, 291.0);
        let path = std::env::temp_dir().join(format!("suiza-frame-qa-{}.png", std::process::id()));
        export_board(&restored.pages[0].artboards[0], &path, "PNG", 1).unwrap();
        assert_eq!(image::open(&path).unwrap().width(), 291);
        assert_eq!(image::open(&path).unwrap().height(), 367);
        std::fs::remove_file(path).unwrap();
        app.doc.pages[0].artboards[0].frame.width = f32::INFINITY;
        assert!(Document::from_json(&serde_json::to_string(&app.doc).unwrap()).is_err());
    }
    #[test]
    fn text_layer_labels_survive_round_trip_without_changing_copy() {
        let mut doc = Document::default();
        let layer = doc.pages[0].artboards[0]
            .layers
            .iter_mut()
            .find(|l| l.kind == Kind::Text)
            .unwrap();
        let original = layer.name.clone();
        layer.label = Some("Brand wordmark".into());
        let restored = Document::from_json(&serde_json::to_string(&doc).unwrap()).unwrap();
        let layer = restored.pages[0].artboards[0]
            .layers
            .iter()
            .find(|l| l.kind == Kind::Text)
            .unwrap();
        assert_eq!(layer.name, original);
        assert_eq!(layer.label.as_deref(), Some("Brand wordmark"));
    }
    #[test]
    fn group_commands_keep_members_together_and_respect_locks() {
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        app.doc.pages[0].artboards[0].layers = vec![
            Layer::new(Kind::Rectangle, 10.0, 20.0),
            Layer::new(Kind::Ellipse, 40.0, 60.0),
        ];
        app.selected = Some(1);
        app.selection = vec![0, 1];
        app.apply_action(Action::Group, &ctx);
        app.selection.clear();
        app.apply_action(Action::Duplicate, &ctx);
        let layers = &app.doc.pages[0].artboards[0].layers;
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[2].group, layers[3].group);
        assert_ne!(layers[0].group, layers[2].group);
        app.apply_action(Action::Align(0), &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers[2].x, 0.0);
        assert_eq!(app.doc.pages[0].artboards[0].layers[3].x, 30.0);
        app.apply_action(Action::CopyLayer, &ctx);
        app.apply_action(Action::PasteLayer, &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), 6);
        app.apply_action(Action::Delete, &ctx);
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), 4);
        app.undo();
        assert_eq!(app.doc.pages[0].artboards[0].layers.len(), 6);
    }
    #[test]
    fn frame_clipboard_commands_and_undo_preserve_documents() {
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        let original = app.doc.clone();
        app.selected = None;
        app.apply_action(Action::BoardCopy, &ctx);
        app.apply_action(Action::BoardPaste, &ctx);
        assert_eq!(app.doc.pages[0].artboards.len(), 3);
        assert!(app.doc.pages[0].artboards[1].layers == original.pages[0].artboards[0].layers);
        app.apply_action(Action::BoardDelete, &ctx);
        assert_eq!(app.doc.pages[0].artboards.len(), 2);
        app.undo();
        assert_eq!(app.doc.pages[0].artboards.len(), 3);
        app.apply_action(Action::Redo, &ctx);
        assert_eq!(app.doc.pages[0].artboards.len(), 2);
        app.apply_action(Action::BoardDuplicate, &ctx);
        assert!(app.redo.is_empty());
    }
    #[test]
    fn selecting_type_controls_does_not_dirty_the_document() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut layer = Layer::new(Kind::Text, 0.0, 0.0);
        let before = layer.clone();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| type_controls(ui, &mut layer));
        });
        assert!(layer == before);
    }
    #[test]
    fn home_window_stays_within_screen_at_supported_sizes() {
        for size in [
            Vec2::new(740.0, 540.0),
            Vec2::new(1320.0, 860.0),
            Vec2::new(2056.0, 1289.0),
        ] {
            let ctx = egui::Context::default();
            configure_fonts(&ctx);
            let mut app = Studio::default();
            app.welcome = true;
            app.recent.push(std::path::PathBuf::from(
                "/missing/a-very-long-document-title-that-must-not-stretch-the-startup-window.json",
            ));
            let screen = Rect::from_min_size(Pos2::ZERO, size);
            for _ in 0..4 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(screen),
                        ..Default::default()
                    },
                    |ctx| app.show_welcome(ctx),
                );
            }
            let bounds = ctx
                .memory(|m| m.area_rect(egui::Id::new("SUIZA Canvas Home")))
                .unwrap();
            assert!(
                screen.contains_rect(bounds),
                "Home overflow: {bounds:?} in {screen:?}"
            );
        }
    }
    #[test]
    fn dark_theme_keeps_spacing_and_legible_foreground() {
        let ctx = egui::Context::default();
        let original = ctx.style().spacing.item_spacing;
        let mut app = Studio::default();
        app.theme = ThemeMode::Dark;
        app.apply_theme(&ctx);
        let visuals = ctx.style().visuals.clone();
        assert_eq!(ctx.style().spacing.item_spacing, original);
        let luminance = |color: Color32| {
            let f = |n: u8| {
                let n = n as f64 / 255.0;
                if n <= 0.04045 {
                    n / 12.92
                } else {
                    ((n + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * f(color.r()) + 0.7152 * f(color.g()) + 0.0722 * f(color.b())
        };
        let contrast =
            (luminance(visuals.text_color()) + 0.05) / (luminance(visuals.panel_fill) + 0.05);
        assert!(contrast >= 7.0);
        assert!(
            luminance(visuals.widgets.inactive.fg_stroke.color) > luminance(visuals.panel_fill)
        );
        app.theme = ThemeMode::Light;
        app.apply_theme(&ctx);
        assert!(!ctx.style().visuals.dark_mode);
    }
    #[test]
    fn high_contrast_appearance_is_persistent_and_keeps_artwork_unchanged() {
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        let before = app.doc.clone();
        app.apply_action(Action::Theme(ThemeMode::HighContrastLight), &ctx);
        app.apply_theme(&ctx);
        let visuals = ctx.style().visuals.clone();
        assert!(!visuals.dark_mode);
        assert_eq!(visuals.text_color(), Color32::BLACK);
        assert_eq!(visuals.panel_fill, Color32::WHITE);
        assert!(visuals.widgets.inactive.bg_stroke.width >= 1.5);
        assert!(app.doc == before);
        let saved = serde_json::to_string(&app.theme).unwrap();
        assert!(serde_json::from_str::<ThemeMode>(&saved).unwrap() == ThemeMode::HighContrastLight);
        app.apply_action(Action::Theme(ThemeMode::Light), &ctx);
        app.apply_theme(&ctx);
        assert!(!ctx.data(|d| d
            .get_temp::<bool>(egui::Id::new("high_contrast_ui"))
            .unwrap()));
    }
    #[test]
    fn system_appearance_tracks_os_and_menu_overrides() {
        let ctx = egui::Context::default();
        let mut app = Studio::default();
        assert!(app.theme == ThemeMode::System);
        let update = |app: &mut Studio, theme| {
            let _ = ctx.run(
                egui::RawInput {
                    system_theme: Some(theme),
                    ..Default::default()
                },
                |ctx| app.apply_theme(ctx),
            );
        };
        update(&mut app, egui::Theme::Dark);
        assert!(ctx.style().visuals.dark_mode);
        update(&mut app, egui::Theme::Light);
        assert!(!ctx.style().visuals.dark_mode);
        app.apply_action(Action::Theme(ThemeMode::Dark), &ctx);
        update(&mut app, egui::Theme::Light);
        assert!(ctx.style().visuals.dark_mode);
        app.apply_action(Action::Theme(ThemeMode::System), &ctx);
        update(&mut app, egui::Theme::Light);
        assert!(!ctx.style().visuals.dark_mode);
    }
    #[test]
    fn pinch_zoom_keeps_pointer_anchor_and_two_finger_scroll_pans() {
        let ctx = egui::Context::default();
        configure_fonts(&ctx);
        let mut app = Studio::default();
        app.selected = None;
        let raw = || egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1320.0, 860.0))),
            ..Default::default()
        };
        let _ = ctx.run(raw(), |ctx| app.show(ctx));
        let pointer = app.canvas_bounds.center() + Vec2::new(100.0, 50.0);
        let before = (pointer - app.canvas_bounds.center() - app.pan) / app.zoom;
        let mut input = raw();
        input.events = vec![egui::Event::PointerMoved(pointer), egui::Event::Zoom(1.25)];
        let _ = ctx.run(input, |ctx| app.show(ctx));
        let after = (pointer - app.canvas_bounds.center() - app.pan) / app.zoom;
        assert!((before - after).length() < 0.01);
        assert!(app.zoom > 1.0);
        let previous = app.pan;
        let mut input = raw();
        input.events = vec![
            egui::Event::PointerMoved(pointer),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: Vec2::new(40.0, 25.0),
                modifiers: Default::default(),
            },
        ];
        let _ = ctx.run(input, |ctx| app.show(ctx));
        assert!(app.pan.x > previous.x && app.pan.y > previous.y);
    }
}
