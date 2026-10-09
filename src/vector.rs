//! Cubic Bézier geometry. Curves are stored exactly; flattening is only for raster coverage.
use super::{Kind, Layer, Vec2};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default, Debug)]
pub enum NodeMode {
    #[default]
    Corner,
    Smooth,
    Symmetric,
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Node {
    pub point: [f32; 2],
    pub incoming: [f32; 2],
    pub outgoing: [f32; 2],
    pub mode: NodeMode,
}
impl Node {
    pub fn at(p: [f32; 2]) -> Self {
        Self {
            point: p,
            incoming: p,
            outgoing: p,
            mode: NodeMode::Corner,
        }
    }
    pub fn set_mode(&mut self, mode: NodeMode) {
        self.mode = mode;
        if mode != NodeMode::Corner {
            let handle = if self.outgoing == self.point && self.incoming != self.point {
                [
                    2.0 * self.point[0] - self.incoming[0],
                    2.0 * self.point[1] - self.incoming[1],
                ]
            } else {
                self.outgoing
            };
            self.move_handle(handle, true, false);
        }
    }
    pub fn move_anchor(&mut self, p: [f32; 2]) {
        for i in 0..2 {
            let delta = p[i] - self.point[i];
            self.incoming[i] += delta;
            self.outgoing[i] += delta;
        }
        self.point = p;
    }
    pub fn move_handle(&mut self, p: [f32; 2], outgoing: bool, independent: bool) {
        let (handle, other) = if outgoing {
            (&mut self.outgoing, &mut self.incoming)
        } else {
            (&mut self.incoming, &mut self.outgoing)
        };
        *handle = p;
        if independent || self.mode == NodeMode::Corner {
            return;
        }
        let dx = p[0] - self.point[0];
        let dy = p[1] - self.point[1];
        let distance = dx.hypot(dy);
        let length = if self.mode == NodeMode::Symmetric {
            distance
        } else {
            (other[0] - self.point[0]).hypot(other[1] - self.point[1])
        };
        if distance > 1e-8 {
            *other = [
                self.point[0] - dx * length / distance,
                self.point[1] - dy * length / distance,
            ];
        }
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Path {
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub contours: Vec<Vec<Node>>,
    pub closed: bool,
    pub fill: bool,
    pub stroke: bool,
    pub stroke_color: [u8; 3],
    pub stroke_width: f32,
}
impl Default for Path {
    fn default() -> Self {
        Self {
            nodes: vec![],
            contours: vec![],
            closed: false,
            fill: false,
            stroke: true,
            stroke_color: [96, 83, 255],
            stroke_width: 2.0,
        }
    }
}
fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}
#[cfg(test)]
pub fn cubic(points: [[f32; 2]; 4], t: f32) -> [f32; 2] {
    let a = lerp(points[0], points[1], t);
    let b = lerp(points[1], points[2], t);
    let c = lerp(points[2], points[3], t);
    lerp(lerp(a, b, t), lerp(b, c, t), t)
}
fn split(c: [[f32; 2]; 4], t: f32) -> ([[f32; 2]; 4], [[f32; 2]; 4]) {
    let a = lerp(c[0], c[1], t);
    let b = lerp(c[1], c[2], t);
    let d = lerp(c[2], c[3], t);
    let e = lerp(a, b, t);
    let f = lerp(b, d, t);
    let p = lerp(e, f, t);
    ([c[0], a, e, p], [p, f, d, c[3]])
}
fn distance_to_line(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let v = [b[0] - a[0], b[1] - a[1]];
    let denom = v[0] * v[0] + v[1] * v[1];
    let t = if denom > 0.0 {
        ((p[0] - a[0]) * v[0] + (p[1] - a[1]) * v[1]) / denom
    } else {
        0.0
    }
    .clamp(0.0, 1.0);
    (p[0] - a[0] - t * v[0]).hypot(p[1] - a[1] - t * v[1])
}
fn flatten(c: [[f32; 2]; 4], tolerance: f32, depth: u8, out: &mut Vec<[f32; 2]>) {
    if depth >= 18
        || distance_to_line(c[1], c[0], c[3]).max(distance_to_line(c[2], c[0], c[3])) <= tolerance
    {
        out.push(c[3]);
    } else {
        let (a, b) = split(c, 0.5);
        flatten(a, tolerance, depth + 1, out);
        flatten(b, tolerance, depth + 1, out);
    }
}
impl Path {
    pub fn valid(&self) -> bool {
        self.nodes.len() + self.contours.iter().map(Vec::len).sum::<usize>() <= 4096
            && self.stroke_width.is_finite()
            && (0.0..=200.0).contains(&self.stroke_width)
            && self
                .nodes
                .iter()
                .chain(self.contours.iter().flatten())
                .all(|n| {
                    [n.point, n.incoming, n.outgoing]
                        .into_iter()
                        .flatten()
                        .all(|v| v.is_finite() && v.abs() <= 100.0)
                })
    }
    pub fn segments(&self) -> usize {
        self.nodes.len().saturating_sub(usize::from(!self.closed))
    }
    pub fn segment(&self, i: usize) -> [[f32; 2]; 4] {
        let a = &self.nodes[i];
        let b = &self.nodes[(i + 1) % self.nodes.len()];
        [a.point, a.outgoing, b.incoming, b.point]
    }
    pub fn points(&self, size: Vec2, tolerance: f32) -> Vec<[f32; 2]> {
        if self.nodes.is_empty() {
            return vec![];
        }
        let convert = |p: [f32; 2]| [p[0] * size.x, p[1] * size.y];
        let mut out = vec![convert(self.nodes[0].point)];
        for i in 0..self.segments() {
            flatten(
                self.segment(i).map(convert),
                tolerance.max(0.001),
                0,
                &mut out,
            );
        }
        out
    }
    pub fn insert(&mut self, i: usize, t: f32) {
        if i >= self.segments() {
            return;
        }
        let next = (i + 1) % self.nodes.len();
        let (a, b) = split(self.segment(i), t.clamp(0.0, 1.0));
        self.nodes[i].outgoing = a[1];
        self.nodes[next].incoming = b[2];
        self.nodes.insert(
            i + 1,
            Node {
                point: a[3],
                incoming: a[2],
                outgoing: b[1],
                mode: NodeMode::Smooth,
            },
        );
    }
    pub fn reverse(&mut self) {
        for nodes in std::iter::once(&mut self.nodes).chain(self.contours.iter_mut()) {
            nodes.reverse();
            for n in nodes {
                std::mem::swap(&mut n.incoming, &mut n.outgoing);
            }
        }
    }
    pub fn svg(&self, size: Vec2, fill: &str) -> String {
        if self.nodes.is_empty() {
            return String::new();
        }
        let mut d = String::new();
        for nodes in std::iter::once(&self.nodes).chain(&self.contours) {
            if nodes.is_empty() {
                continue;
            }
            let p = nodes[0].point;
            d.push_str(&format!("M {} {}", p[0] * size.x, p[1] * size.y));
            for i in 0..nodes.len().saturating_sub(usize::from(!self.closed)) {
                let a = &nodes[i];
                let b = &nodes[(i + 1) % nodes.len()];
                d.push_str(&format!(
                    " C {} {} {} {} {} {}",
                    a.outgoing[0] * size.x,
                    a.outgoing[1] * size.y,
                    b.incoming[0] * size.x,
                    b.incoming[1] * size.y,
                    b.point[0] * size.x,
                    b.point[1] * size.y
                ));
            }
            if self.closed {
                d.push_str(" Z");
            }
        }
        let stroke = if self.stroke {
            format!(
                "#{:02X}{:02X}{:02X}",
                self.stroke_color[0], self.stroke_color[1], self.stroke_color[2]
            )
        } else {
            "none".into()
        };
        format!("<path d=\"{d}\" fill=\"{}\" fill-rule=\"evenodd\" stroke=\"{stroke}\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",if self.fill {fill} else {"none"},self.stroke_width)
    }
}
pub fn contains(points: &[[f32; 2]], p: [f32; 2]) -> bool {
    let mut inside = false;
    if points.len() < 3 {
        return false;
    }
    let mut a = points[points.len() - 1];
    for &b in points {
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        a = b;
    }
    inside
}
pub fn stroke_contains(points: &[[f32; 2]], p: [f32; 2], width: f32) -> bool {
    points
        .windows(2)
        .any(|s| distance_to_line(p, s[0], s[1]) <= width / 2.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn splitting_preserves_exact_curve() {
        let mut p = Path::default();
        p.nodes = vec![Node::at([0., 0.]), Node::at([1., 1.])];
        p.nodes[0].outgoing = [1., 0.];
        p.nodes[1].incoming = [0., 1.];
        let before = p.segment(0);
        p.insert(0, 0.37);
        for i in 0..101 {
            let t = i as f32 / 100.;
            let a = cubic(before, t);
            let b = if t <= 0.37 {
                cubic(p.segment(0), t / 0.37)
            } else {
                cubic(p.segment(1), (t - 0.37) / 0.63)
            };
            assert!((a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6);
        }
    }
    #[test]
    fn moving_anchor_and_symmetric_handles_preserves_tangency() {
        let mut n = Node::at([0.5, 0.5]);
        n.mode = NodeMode::Symmetric;
        n.move_handle([0.8, 0.7], true, false);
        assert_eq!(n.incoming, [0.19999999, 0.3]);
        let before = n.incoming;
        n.move_anchor([0.6, 0.6]);
        assert!((n.incoming[0] - before[0] - 0.1).abs() < 1e-6);
    }
}

pub fn render(
    path: &Path,
    fill: [u8; 3],
    w: u32,
    h: u32,
    scale: f32,
) -> Result<image::RgbaImage, String> {
    let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or("Invalid path dimensions")?;
    if path.nodes.is_empty() {
        return Ok(image::RgbaImage::new(w, h));
    }
    let mut builder = tiny_skia::PathBuilder::new();
    for nodes in std::iter::once(&path.nodes).chain(&path.contours) {
        if nodes.is_empty() {
            continue;
        }
        let first = nodes[0].point;
        builder.move_to(first[0] * w as f32, first[1] * h as f32);
        for i in 0..nodes.len().saturating_sub(usize::from(!path.closed)) {
            let a = &nodes[i];
            let b = &nodes[(i + 1) % nodes.len()];
            builder.cubic_to(
                a.outgoing[0] * w as f32,
                a.outgoing[1] * h as f32,
                b.incoming[0] * w as f32,
                b.incoming[1] * h as f32,
                b.point[0] * w as f32,
                b.point[1] * h as f32,
            );
        }
        if path.closed {
            builder.close();
        }
    }
    if let Some(curve) = builder.finish() {
        let mut paint = tiny_skia::Paint::default();
        paint.anti_alias = true;
        if path.fill {
            paint.set_color_rgba8(fill[0], fill[1], fill[2], 255);
            pixmap.fill_path(
                &curve,
                &paint,
                tiny_skia::FillRule::EvenOdd,
                tiny_skia::Transform::identity(),
                None,
            );
        }
        if path.stroke {
            paint.set_color_rgba8(
                path.stroke_color[0],
                path.stroke_color[1],
                path.stroke_color[2],
                255,
            );
            let stroke = tiny_skia::Stroke {
                width: path.stroke_width * scale,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };
            pixmap.stroke_path(
                &curve,
                &paint,
                &stroke,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }
    let mut data = pixmap.take();
    for p in data.chunks_exact_mut(4) {
        if p[3] > 0 {
            for i in 0..3 {
                p[i] = (u32::from(p[i]) * 255 / u32::from(p[3])).min(255) as u8;
            }
        }
    }
    image::RgbaImage::from_raw(w, h, data).ok_or("Invalid rendered path".into())
}

pub fn path_contains(path: &Path, size: Vec2, p: [f32; 2]) -> bool {
    let mut hit = contains(&path.points(size, 0.02), p);
    for nodes in &path.contours {
        let contour = Path {
            nodes: nodes.clone(),
            contours: vec![],
            ..path.clone()
        };
        hit ^= contains(&contour.points(size, 0.02), p);
    }
    hit
}
fn from_segments(segments: Vec<[[f32; 2]; 4]>) -> Vec<Node> {
    if segments.is_empty() {
        return vec![];
    }
    let mut nodes = vec![Node::at(segments[0][0])];
    for c in segments {
        nodes.last_mut().unwrap().outgoing = c[1];
        let mut n = Node::at(c[3]);
        n.incoming = c[2];
        nodes.push(n);
    }
    if nodes.len() > 1
        && (nodes[0].point[0] - nodes.last().unwrap().point[0])
            .hypot(nodes[0].point[1] - nodes.last().unwrap().point[1])
            < 1e-5
    {
        nodes[0].incoming = nodes.pop().unwrap().incoming;
    }
    nodes
}
pub fn from_layer(layer: &Layer) -> Option<Path> {
    if let Some(p) = &layer.path {
        return Some(p.clone());
    }
    if layer.kind == Kind::Image {
        return None;
    }
    let mut path = Path {
        closed: true,
        fill: true,
        stroke: false,
        ..Default::default()
    };
    if layer.kind == Kind::Text {
        use ab_glyph::{Font, OutlineCurve, ScaleFont};
        let layout = super::text::layout(layer).ok()?;
        let font = super::fonts::render_font(&layer.typography);
        let factor = font.as_scaled(layout.size).scale_factor();
        let mut contours = vec![];
        for glyph in &layout.glyphs {
            if let Some(outline) = font.outline(glyph.id) {
                let point = |p: ab_glyph::Point| {
                    let y = glyph.y - p.y * factor.vertical;
                    let shear = if layout.shear {
                        (glyph.y - y) * 0.2
                    } else {
                        0.0
                    };
                    [
                        (glyph.x + p.x * factor.horizontal + shear) / layer.width,
                        y / layer.height,
                    ]
                };
                let mut segments = vec![];
                let mut last = None;
                for curve in outline.curves {
                    let c = match curve {
                        OutlineCurve::Line(a, b) => {
                            let a = point(a);
                            let b = point(b);
                            [a, a, b, b]
                        }
                        OutlineCurve::Quad(a, b, c) => {
                            let a = point(a);
                            let b = point(b);
                            let c = point(c);
                            [a, lerp(a, b, 2.0 / 3.0), lerp(c, b, 2.0 / 3.0), c]
                        }
                        OutlineCurve::Cubic(a, b, c, d) => [point(a), point(b), point(c), point(d)],
                    };
                    if last.is_some_and(|p: [f32; 2]| (p[0] - c[0][0]).hypot(p[1] - c[0][1]) > 1e-5)
                    {
                        contours.push(from_segments(std::mem::take(&mut segments)));
                    }
                    last = Some(c[3]);
                    segments.push(c);
                }
                if !segments.is_empty() {
                    contours.push(from_segments(segments));
                }
            }
        }
        for [x, y, end, thickness] in layout.rules {
            contours.push(
                [
                    [x, y - thickness / 2.0],
                    [end, y - thickness / 2.0],
                    [end, y + thickness / 2.0],
                    [x, y + thickness / 2.0],
                ]
                .map(|p| Node::at([p[0] / layer.width, p[1] / layer.height]))
                .to_vec(),
            );
        }
        if !contours.is_empty() {
            path.nodes = contours.remove(0);
            path.contours = contours;
        }
    } else if layer.kind == Kind::Ellipse {
        let k = 0.552_284_8;
        path.nodes = vec![
            Node::at([0.5, 0.0]),
            Node::at([1.0, 0.5]),
            Node::at([0.5, 1.0]),
            Node::at([0.0, 0.5]),
        ];
        for i in 0..4 {
            let (incoming, outgoing) = match i {
                0 => ([0.5 - k / 2.0, 0.0], [0.5 + k / 2.0, 0.0]),
                1 => ([1.0, 0.5 - k / 2.0], [1.0, 0.5 + k / 2.0]),
                2 => ([0.5 + k / 2.0, 1.0], [0.5 - k / 2.0, 1.0]),
                _ => ([0.0, 0.5 + k / 2.0], [0.0, 0.5 - k / 2.0]),
            };
            path.nodes[i].incoming = incoming;
            path.nodes[i].outgoing = outgoing;
            path.nodes[i].mode = NodeMode::Smooth;
        }
    } else {
        let radii = layer.corners.unwrap_or([layer.radius; 4]);
        let w = layer.width;
        let h = layer.height;
        let k = 0.552_284_8;
        let r = radii.map(|r| r.min(w / 2.0).min(h / 2.0));
        // Clockwise tangent endpoints, one cubic arc at each corner.
        let endpoints = [
            [r[0], 0.0],
            [w - r[1], 0.0],
            [w, r[1]],
            [w, h - r[3]],
            [w - r[3], h],
            [r[2], h],
            [0.0, h - r[2]],
            [0.0, r[0]],
        ];
        path.nodes = endpoints.map(|p| Node::at([p[0] / w, p[1] / h])).to_vec();
        for (a, b, corner, cp1, cp2) in [
            (1, 2, 1, [w - r[1] + k * r[1], 0.0], [w, r[1] - k * r[1]]),
            (3, 4, 3, [w, h - r[3] + k * r[3]], [w - r[3] + k * r[3], h]),
            (5, 6, 2, [r[2] - k * r[2], h], [0.0, h - r[2] + k * r[2]]),
            (7, 0, 0, [0.0, r[0] - k * r[0]], [r[0] - k * r[0], 0.0]),
        ] {
            let _ = corner;
            if layer.corner_style != "Angled" && layer.corner_style != "Inside Square" {
                path.nodes[a].outgoing = [cp1[0] / w, cp1[1] / h];
                path.nodes[b].incoming = [cp2[0] / w, cp2[1] / h];
            }
        }
        if matches!(
            layer.corner_style.as_str(),
            "Smooth" | "Inside Arc" | "Inside Square"
        ) {
            path.nodes.clear();
            for (corner, radius) in [(1, r[1]), (3, r[3]), (2, r[2]), (0, r[0])] {
                let points: Vec<[f32; 2]> = if layer.corner_style == "Inside Square" {
                    vec![[radius, 0.0], [radius, radius], [0.0, radius]]
                } else {
                    (0..=64)
                        .map(|step| {
                            let t = step as f32 / 64.0 * std::f32::consts::FRAC_PI_2;
                            if layer.corner_style == "Inside Arc" {
                                [radius * t.cos(), radius * t.sin()]
                            } else {
                                [
                                    radius * (1.0 - t.sin().sqrt()),
                                    radius * (1.0 - t.cos().max(0.0).sqrt()),
                                ]
                            }
                        })
                        .collect()
                };
                for [u, v] in points {
                    let [x, y] = match corner {
                        1 => [w - u, v],
                        3 => [w - v, h - u],
                        2 => [u, h - v],
                        _ => [v, u],
                    };
                    path.nodes.push(Node::at([x / w, y / h]));
                }
            }
        }
    }
    Some(path)
}
fn world_paths(layer: &Layer) -> Option<Vec<flo_curves::bezier::path::SimpleBezierPath>> {
    use flo_curves::Coord2;
    let p = from_layer(layer)?;
    if !p.closed {
        return None;
    }
    let convert = |p: [f32; 2]| {
        Coord2(
            (layer.x + p[0] * layer.width) as f64,
            (layer.y + p[1] * layer.height) as f64,
        )
    };
    Some(
        std::iter::once(p.nodes)
            .chain(p.contours)
            .filter(|n| n.len() >= 2)
            .map(|nodes| {
                let start = convert(nodes[0].point);
                let curves = (0..nodes.len())
                    .map(|i| {
                        let a = &nodes[i];
                        let b = &nodes[(i + 1) % nodes.len()];
                        (convert(a.outgoing), convert(b.incoming), convert(b.point))
                    })
                    .collect();
                (start, curves)
            })
            .collect(),
    )
}
pub fn combine(layers: &[Layer], operation: u8) -> Result<Layer, String> {
    use flo_curves::bezier::path::{path_add, path_intersect, path_sub, SimpleBezierPath};
    let mut paths = world_paths(&layers[0]).ok_or("Boolean operations need closed paths")?;
    for layer in &layers[1..] {
        let next = world_paths(layer).ok_or("Boolean operations need closed paths")?;
        paths = match operation {
            0 => path_add::<SimpleBezierPath>(&paths, &next, 0.001),
            1 => path_sub::<SimpleBezierPath>(&paths, &next, 0.001),
            2 => path_intersect::<SimpleBezierPath>(&paths, &next, 0.001),
            3 => {
                let mut a = path_sub::<SimpleBezierPath>(&paths, &next, 0.001);
                a.extend(path_sub::<SimpleBezierPath>(&next, &paths, 0.001));
                a
            }
            _ => {
                paths.extend(next);
                paths
            }
        };
    }
    if paths.is_empty() {
        return Err("The operation produced an empty path".into());
    }
    let mut min = [f64::INFINITY; 2];
    let mut max = [f64::NEG_INFINITY; 2];
    for (start, curves) in &paths {
        for p in std::iter::once(*start).chain(curves.iter().flat_map(|(a, b, c)| [*a, *b, *c])) {
            min[0] = min[0].min(p.0);
            min[1] = min[1].min(p.1);
            max[0] = max[0].max(p.0);
            max[1] = max[1].max(p.1);
        }
    }
    let mut result = layers[0].clone();
    result.x = min[0] as f32;
    result.y = min[1] as f32;
    result.width = (max[0] - min[0]).max(0.001) as f32;
    result.height = (max[1] - min[1]).max(0.001) as f32;
    result.kind = Kind::Rectangle;
    result.radius = 0.0;
    result.corners = None;
    result.corner_style = "Rounded".into();
    result.mask = false;
    result.name = ["Union", "Subtract", "Intersect", "Exclude", "Compound path"]
        [operation.min(4) as usize]
        .into();
    let point = |p: flo_curves::Coord2| {
        [
            ((p.0 - min[0]) / result.width as f64) as f32,
            ((p.1 - min[1]) / result.height as f64) as f32,
        ]
    };
    let mut contours: Vec<_> = paths
        .into_iter()
        .map(|(start, curves)| {
            let mut previous = start;
            from_segments(
                curves
                    .into_iter()
                    .map(|(a, b, end)| {
                        let c = [point(previous), point(a), point(b), point(end)];
                        previous = end;
                        c
                    })
                    .collect(),
            )
        })
        .collect();
    result.path = Some(Path {
        nodes: contours.remove(0),
        contours,
        closed: true,
        fill: true,
        stroke: false,
        ..Default::default()
    });
    Ok(result)
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    #[test]
    fn boolean_operations_preserve_coverage_and_compound_holes() {
        let mut a = Layer::new(Kind::Rectangle, 0.0, 0.0);
        a.width = 100.0;
        a.height = 100.0;
        a.radius = 0.0;
        let mut b = a.clone();
        b.x = 50.0;
        let hit = |l: &Layer, p: [f32; 2]| {
            path_contains(
                l.path.as_ref().unwrap(),
                Vec2::new(l.width, l.height),
                [p[0] - l.x, p[1] - l.y],
            )
        };
        for (operation, left, middle, right) in [
            (0, true, true, true),
            (1, true, false, false),
            (2, false, true, false),
            (3, true, false, true),
            (4, true, false, true),
        ] {
            let result = combine(&[a.clone(), b.clone()], operation).unwrap();
            assert_eq!(hit(&result, [25.0, 50.0]), left);
            assert_eq!(hit(&result, [75.0, 50.0]), middle);
            assert_eq!(hit(&result, [125.0, 50.0]), right);
            assert!(result
                .path
                .as_ref()
                .unwrap()
                .svg(Vec2::new(result.width, result.height), "#000000")
                .contains(" C "));
        }
    }
    #[test]
    fn text_outlines_match_font_coverage_and_retain_glyph_holes() {
        let mut l = Layer::new(Kind::Text, 0.0, 0.0);
        l.name = "Bézier O".into();
        l.width = 180.0;
        l.height = 50.0;
        l.typography.size = Some(36.0);
        let original = super::super::rasterize_layer(&l, 2, false, &[]).unwrap();
        let path = from_layer(&l).unwrap();
        assert!(!path.contours.is_empty());
        l.path = Some(path);
        l.kind = Kind::Rectangle;
        let outline = super::super::rasterize_layer(&l, 2, false, &[]).unwrap();
        let sum = |img: &image::RgbaImage| img.pixels().map(|p| p[3] as f64).sum::<f64>();
        let ratio = sum(&outline) / sum(&original);
        assert!((0.95..1.05).contains(&ratio), "outline area ratio {ratio}");
    }
}

pub fn normalize_layer(layer: &mut Layer) {
    let Some(path) = &mut layer.path else {
        return;
    };
    let mut min = [f32::INFINITY; 2];
    let mut max = [f32::NEG_INFINITY; 2];
    for n in path.nodes.iter().chain(path.contours.iter().flatten()) {
        for p in [n.point, n.incoming, n.outgoing] {
            for i in 0..2 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
    }
    if !min[0].is_finite() {
        return;
    }
    let margin = if path.stroke {
        path.stroke_width / 2.0 + 1.0
    } else {
        0.5
    };
    let old = Vec2::new(layer.width, layer.height);
    let offset = Vec2::new(min[0] * old.x - margin, min[1] * old.y - margin);
    let size = Vec2::new(
        ((max[0] - min[0]) * old.x + 2.0 * margin).max(1.0),
        ((max[1] - min[1]) * old.y + 2.0 * margin).max(1.0),
    );
    for n in path
        .nodes
        .iter_mut()
        .chain(path.contours.iter_mut().flatten())
    {
        for p in [&mut n.point, &mut n.incoming, &mut n.outgoing] {
            p[0] = (p[0] * old.x - offset.x) / size.x;
            p[1] = (p[1] * old.y - offset.y) / size.y;
        }
    }
    layer.x += offset.x;
    layer.y += offset.y;
    layer.width = size.x;
    layer.height = size.y;
}
pub fn hit(layer: &Layer, p: [f32; 2], tolerance: f32) -> bool {
    let Some(path) = &layer.path else {
        return false;
    };
    let size = Vec2::new(layer.width, layer.height);
    (path.fill && path_contains(path, size, p))
        || (path.stroke
            && std::iter::once(&path.nodes)
                .chain(&path.contours)
                .any(|nodes| {
                    let contour = Path {
                        nodes: nodes.clone(),
                        contours: vec![],
                        ..path.clone()
                    };
                    stroke_contains(
                        &contour.points(size, 0.1),
                        p,
                        path.stroke_width + 2.0 * tolerance,
                    )
                }))
}
