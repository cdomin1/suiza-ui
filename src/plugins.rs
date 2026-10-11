//! Versioned, data-only third-party extensions. No executable code or ambient I/O.
use crate::Document;
use eframe::egui;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    pub api_version: u32,
    pub id: String,
    pub name: String,
    pub author: String,
    pub permissions: Vec<String>,
    pub commands: Vec<Command>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub name: String,
    pub operations: Vec<Operation>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Rename { prefix: String, suffix: String },
    Fill { color: [u8; 3] },
    Move { x: f32, y: f32 },
    TextSize { size: f32 },
}
impl Plugin {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 1024 * 1024 {
            return Err("Plugin exceeds 1 MB".into());
        }
        let p: Self = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if p.api_version != 1 {
            return Err("Unsupported plugin API version".into());
        }
        if p.id.is_empty()
            || p.id.len() > 120
            || !p
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
        {
            return Err("Invalid plugin identifier".into());
        }
        if p.name.trim().is_empty() || p.name.len() > 120 || p.author.len() > 120 {
            return Err("Invalid plugin metadata".into());
        }
        if p.permissions != ["selection.write"] {
            return Err("API v1 requires only selection.write permission".into());
        }
        if p.commands.is_empty() || p.commands.len() > 50 {
            return Err("Plugins need 1–50 commands".into());
        }
        for c in &p.commands {
            if c.name.trim().is_empty()
                || c.name.len() > 120
                || c.operations.is_empty()
                || c.operations.len() > 100
            {
                return Err("Invalid plugin command".into());
            }
            for op in &c.operations {
                match op {
                    Operation::Rename { prefix, suffix } if prefix.len() + suffix.len() > 1024 => {
                        return Err("Rename text too long".into())
                    }
                    Operation::Move { x, y }
                        if !x.is_finite()
                            || !y.is_finite()
                            || x.abs() > 8192.0
                            || y.abs() > 8192.0 =>
                    {
                        return Err("Invalid movement".into())
                    }
                    Operation::TextSize { size }
                        if !size.is_finite() || !(1.0..=512.0).contains(size) =>
                    {
                        return Err("Invalid text size".into())
                    }
                    _ => {}
                }
            }
        }
        Ok(p)
    }
    pub fn run(
        &self,
        command: usize,
        doc: &Document,
        page: usize,
        board: usize,
        selection: &[usize],
    ) -> Result<Document, String> {
        let command = self.commands.get(command).ok_or("Unknown plugin command")?;
        let mut next = doc.clone();
        let b = next
            .pages
            .get_mut(page)
            .and_then(|p| p.artboards.get_mut(board))
            .ok_or("Invalid frame")?;
        if selection.is_empty() {
            return Err("Select layers before running a plugin".into());
        }
        let mut indices = selection.to_vec();
        indices.sort_unstable();
        indices.dedup();
        for index in indices {
            let layer = b.layers.get_mut(index).ok_or("Invalid selection")?;
            if layer.locked {
                continue;
            }
            for op in &command.operations {
                match op {
                    Operation::Rename { prefix, suffix } => {
                        let name = layer.label.as_deref().unwrap_or(&layer.name);
                        layer.label = Some(format!("{prefix}{name}{suffix}"));
                    }
                    Operation::Fill { color } => layer.color = *color,
                    Operation::Move { x, y } => {
                        layer.x += x;
                        layer.y += y;
                    }
                    Operation::TextSize { size } => {
                        if layer.kind == crate::Kind::Text {
                            layer.typography.size = Some(*size);
                        }
                    }
                }
            }
        }
        next.validate()?;
        Ok(next)
    }
}
#[derive(Default)]
pub struct Manager {
    pub open: bool,
    pub installed: Vec<Plugin>,
    pending: Option<Plugin>,
    error: String,
}
impl Manager {
    pub fn show(&mut self, ctx: &egui::Context) -> Option<(usize, usize)> {
        if !self.open {
            return None;
        }
        let mut run = None;
        let mut close = false;
        egui::Window::new("Plugins").title_bar(false).frame(egui::Frame::window(&ctx.style()).inner_margin(16)).default_width(440.0).show(ctx,|ui|{
            ui.horizontal(|ui| { ui.heading("Plugins"); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { close = crate::close_icon_button(ui, 28.0).clicked(); }); });
            ui.label("Third-party extensions · API v1");
            if ui.button("Install plugin…").clicked() {
                if let Some(path)=rfd::FileDialog::new().add_filter("SUIZA plugin",&["json"]).pick_file() {
                    let result=std::fs::metadata(&path).map_err(|e|e.to_string()).and_then(|m|if m.len()>1024*1024{Err("Plugin exceeds 1 MB".into())}else{std::fs::read(&path).map_err(|e|e.to_string())}).and_then(|b|Plugin::parse(&b));
                    match result {Ok(p)=>{self.pending=Some(p);self.error.clear();},Err(e)=>self.error=e}
                }
            }
            if let Some(p)=self.pending.clone() {
                ui.separator(); ui.label(format!("{} by {}",p.name,p.author));
                ui.label("Permission: modify selected unlocked layers. Installation does not run commands.");
                ui.horizontal(|ui|{if ui.button("Allow and install").clicked(){self.installed.retain(|x|x.id!=p.id);self.installed.push(p);self.pending=None;}if ui.button("Cancel").clicked(){self.pending=None;}});
            }
            let mut remove=None;
            egui::ScrollArea::vertical().max_height(420.0).show(ui,|ui|{
                for(i,p)in self.installed.iter().enumerate(){ui.separator();ui.horizontal(|ui|{ui.strong(&p.name);if ui.small_button("Uninstall").clicked(){remove=Some(i);}});ui.label(&p.author);for(j,c)in p.commands.iter().enumerate(){if ui.button(&c.name).clicked(){run=Some((i,j));}}}
            });
            if let Some(i)=remove {self.installed.remove(i);run=None;}
            if !self.error.is_empty(){ui.label(&self.error);}
        });
        if close {
            self.open = false;
        }
        run
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = include_str!("../examples/plugins/blue-labels.json");
    #[test]
    fn plugin_is_atomic_and_selection_scoped() {
        let p = Plugin::parse(SAMPLE.as_bytes()).unwrap();
        let doc = Document::default();
        let next = p.run(0, &doc, 0, 0, &[0, 0]).unwrap();
        assert_eq!(next.pages[0].artboards[0].layers[0].color, [45, 113, 225]);
        assert!(next.pages[0].artboards[0].layers[1] == doc.pages[0].artboards[0].layers[1]);
        assert!(p.run(0, &doc, 0, 0, &[usize::MAX]).is_err());
        assert!(p.run(0, &doc, 0, 0, &[]).is_err());
    }
    #[test]
    fn unsupported_capabilities_rejected() {
        assert!(Plugin::parse(SAMPLE.replace("selection.write", "network").as_bytes()).is_err());
        assert!(Plugin::parse(
            SAMPLE
                .replace("\"api_version\": 1", "\"api_version\": 2")
                .as_bytes()
        )
        .is_err());
        assert!(Plugin::parse(SAMPLE.replace("\"fill\"", "\"execute\"").as_bytes()).is_err());
    }
}
