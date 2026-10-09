//! One text layout shared by canvas rendering, exports, and outline conversion.
use super::{fonts, Layer};
use ab_glyph::{Font, ScaleFont};
#[derive(Clone, Copy)]
pub struct Glyph {
    pub id: ab_glyph::GlyphId,
    pub x: f32,
    pub y: f32,
}
pub struct Layout {
    pub glyphs: Vec<Glyph>,
    pub rules: Vec<[f32; 4]>,
    pub size: f32,
    pub shear: bool,
}
pub fn displayed_text(layer: &Layer) -> String {
    let t = &layer.typography;
    let text = match t.transformation {
        1 => layer.name.to_uppercase(),
        2 => layer.name.to_lowercase(),
        3 => {
            let mut start = true;
            layer
                .name
                .chars()
                .flat_map(|ch| {
                    let out = if start {
                        ch.to_uppercase().collect::<String>()
                    } else {
                        ch.to_lowercase().collect()
                    };
                    start = ch.is_whitespace();
                    out.chars().collect::<Vec<_>>()
                })
                .collect()
        }
        _ => layer.name.clone(),
    };
    text.lines()
        .enumerate()
        .map(|(i, line)| match t.list {
            1 => format!("• {line}"),
            2 => format!("{}. {line}", i + 1),
            _ => line.into(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
pub fn layout(layer: &Layer) -> Result<Layout, String> {
    let t = &layer.typography;
    let size = t.size.unwrap_or((layer.height * 0.65).min(48.0));
    let font = fonts::render_font(t);
    let scaled = font.as_scaled(size);
    let factor = scaled.scale_factor();
    let (data, index, shear) = fonts::shaping_data(t);
    let face = rustybuzz::Face::from_slice(&data, index).ok_or("Unable to shape selected font")?;
    let features: Vec<rustybuzz::Feature> = [
        ("kern", t.kerning.unwrap_or(true)),
        ("liga", t.ligatures.unwrap_or(true)),
        ("clig", t.ligatures.unwrap_or(true)),
        ("smcp", t.small_caps),
        ("tnum", t.tabular),
        ("zero", t.slashed_zero),
        ("dlig", t.discretionary),
    ]
    .iter()
    .filter_map(|(tag, on)| format!("{tag}={}", u8::from(*on)).parse().ok())
    .collect();
    let shape = |line: &str| {
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(line);
        buffer.guess_segment_properties();
        if t.direction != 0 {
            buffer.set_direction(if t.direction == 2 {
                rustybuzz::Direction::RightToLeft
            } else {
                rustybuzz::Direction::LeftToRight
            });
        }
        rustybuzz::shape(&face, &features, buffer)
    };
    let width = |line: &str| {
        let shaped = shape(line);
        shaped
            .glyph_positions()
            .iter()
            .map(|p| p.x_advance as f32 * factor.horizontal)
            .sum::<f32>()
            + t.tracking * (shaped.len().saturating_sub(1) as f32)
    };
    let mut lines = Vec::new();
    for paragraph in displayed_text(layer).split('\n') {
        let mut line = String::new();
        for word in paragraph.split_inclusive(' ') {
            let candidate = format!("{line}{word}");
            if !line.is_empty() && width(candidate.trim_end()) > layer.width {
                lines.push(line.trim_end().to_owned());
                line = word.into();
            } else {
                line = candidate;
            }
        }
        lines.push(line.trim_end().to_owned());
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    let leading = t.line_height.unwrap_or(size * 1.4);
    let height = (lines.len().saturating_sub(1) as f32) * leading + size;
    let vertical = match t.vertical {
        1 => (layer.height - height) / 2.0,
        2 => layer.height - height,
        _ => 0.0,
    };
    let mut result = Layout {
        glyphs: vec![],
        rules: vec![],
        size,
        shear,
    };
    for (line_index, line) in lines.iter().enumerate() {
        let shaped = shape(line);
        let line_width = width(line);
        let mut x = match t.align {
            1 => (layer.width - line_width) / 2.0,
            2 => layer.width - line_width,
            _ => 0.0,
        };
        let start = x;
        let baseline = vertical + scaled.ascent() + line_index as f32 * leading - t.baseline;
        let spaces = line.chars().filter(|ch| *ch == ' ').count();
        let justify = if t.align == 3 && line_index + 1 < lines.len() && spaces > 0 {
            (layer.width - line_width).max(0.0) / spaces as f32
        } else {
            0.0
        };
        for (info, position) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
            result.glyphs.push(Glyph {
                id: ab_glyph::GlyphId(info.glyph_id as u16),
                x: x + position.x_offset as f32 * factor.horizontal,
                y: baseline - position.y_offset as f32 * factor.vertical,
            });
            x += position.x_advance as f32 * factor.horizontal + t.tracking;
            if line.as_bytes().get(info.cluster as usize) == Some(&b' ') {
                x += justify;
            }
        }
        let end = (x - t.tracking).max(start);
        let thickness = (size / 16.0).max(1.0);
        if t.underline {
            result
                .rules
                .push([start, baseline + size * 0.08, end, thickness]);
        }
        if t.strike {
            result
                .rules
                .push([start, baseline - size * 0.25, end, thickness]);
        }
    }
    Ok(result)
}
pub fn render(layer: &Layer, scale: u32) -> Result<image::RgbaImage, String> {
    let mut img = image::RgbaImage::new(
        (layer.width * scale as f32).ceil().max(1.0) as u32,
        (layer.height * scale as f32).ceil().max(1.0) as u32,
    );
    let layout = layout(layer)?;
    let font = fonts::render_font(&layer.typography);
    let scaled = font.as_scaled(layout.size * scale as f32);
    for glyph in layout.glyphs {
        let glyph = glyph.id.with_scale_and_position(
            scaled.scale(),
            ab_glyph::point(glyph.x * scale as f32, glyph.y * scale as f32),
        );
        if let Some(outline) = font.outline_glyph(glyph.clone()) {
            let bounds = outline.px_bounds();
            outline.draw(|x, y, coverage| {
                let py = y as i32 + bounds.min.y as i32;
                let shear = if layout.shear {
                    (glyph.position.y - py as f32) * 0.2
                } else {
                    0.0
                };
                let px = x as i32 + bounds.min.x as i32 + shear.round() as i32;
                if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                    let current = img.get_pixel(px as u32, py as u32)[3];
                    let alpha = (coverage * 255.0) as u8;
                    img.put_pixel(
                        px as u32,
                        py as u32,
                        image::Rgba([
                            layer.color[0],
                            layer.color[1],
                            layer.color[2],
                            current.saturating_add(
                                ((255 - current) as u16 * alpha as u16 / 255) as u8,
                            ),
                        ]),
                    );
                }
            });
        }
    }
    for [start, y, end, thickness] in layout.rules {
        let y0 = (y - thickness / 2.0) * scale as f32;
        let y1 = (y + thickness / 2.0) * scale as f32;
        for py in y0.floor().max(0.0) as u32..(y1.ceil().max(0.0) as u32).min(img.height()) {
            let alpha =
                ((y1.min(py as f32 + 1.0) - y0.max(py as f32)).clamp(0.0, 1.0) * 255.0) as u8;
            for px in ((start * scale as f32).max(0.0) as u32)
                ..((end * scale as f32).max(0.0) as u32).min(img.width())
            {
                img.put_pixel(
                    px,
                    py,
                    image::Rgba([layer.color[0], layer.color[1], layer.color[2], alpha]),
                );
            }
        }
    }
    Ok(img)
}
#[cfg(test)]
mod tests {
    use super::super::Kind;
    use super::*;
    #[test]
    fn ligatures_kerning_case_lists_and_direction_use_real_shaping() {
        let mut l = Layer::new(Kind::Text, 0.0, 0.0);
        l.width = 400.0;
        l.height = 100.0;
        l.name = "office AV".into();
        l.typography.size = Some(28.0);
        let default = layout(&l).unwrap();
        l.typography.ligatures = Some(false);
        let plain = layout(&l).unwrap();
        assert!(
            default.glyphs.len() < plain.glyphs.len(),
            "ligatures must change glyph sequence"
        );
        l.typography.kerning = Some(false);
        let no_kern = layout(&l).unwrap();
        assert_ne!(
            plain.glyphs.last().unwrap().x,
            no_kern.glyphs.last().unwrap().x
        );
        l.typography.transformation = 1;
        assert_eq!(displayed_text(&l), "OFFICE AV");
        l.typography.list = 2;
        assert_eq!(displayed_text(&l), "1. OFFICE AV");
        l.typography.transformation = 0;
        l.typography.list = 0;
        l.name = "ABC".into();
        l.typography.direction = 2;
        let rtl = layout(&l).unwrap();
        let font = fonts::render_font(&l.typography);
        assert_eq!(rtl.glyphs[0].id, font.glyph_id('C'));
    }
    #[test]
    fn typography_menu_controls_change_canvas_and_exports() {
        let mut l = Layer::new(Kind::Text, 0.0, 0.0);
        l.name = "Text".into();
        l.typography.size = Some(24.0);
        let regular = render(&l, 1).unwrap();
        for mode in 0..4 {
            let mut m = l.clone();
            match mode {
                0 => m.typography.italic = true,
                1 => m.typography.underline = true,
                2 => m.typography.strike = true,
                _ => m.typography.baseline = 4.0,
            };
            assert_ne!(render(&m, 1).unwrap(), regular);
        }
        l.typography.vertical = 2;
        let bottom = layout(&l).unwrap();
        assert!(
            bottom.glyphs[0].y
                > layout(&Layer {
                    typography: super::super::TypeStyle {
                        size: Some(24.0),
                        ..Default::default()
                    },
                    ..l.clone()
                })
                .unwrap()
                .glyphs[0]
                    .y
        );
    }
}
