use std::sync::{Arc, OnceLock};

use eframe::egui::{
    self, Color32, CornerRadius, Painter, Pos2, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use sistemas_lineares_ui::graph::{Anchor, Ink, Item, Pt, Shape as Figure};
use sistemas_lineares_ui::icons::{Icon, Shape, VIEWBOX};
use sistemas_lineares_ui::layout::Rect;
use sistemas_lineares_ui::text::{TextMetrics, margins_for, wrap};
use sistemas_lineares_ui::theme::{self, Color, Font, Palette};

use crate::fonts;

pub fn color(value: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(value.r, value.g, value.b, value.a)
}

pub fn with_alpha(value: Color, alpha: u8) -> Color {
    Color { a: alpha, ..value }
}

pub fn tint(value: Color, alpha: f64) -> Color32 {
    let alpha = if alpha.is_nan() {
        0.0
    } else {
        alpha.clamp(0.0, 1.0)
    };
    let opacity = (f64::from(value.a) * alpha).round() as u8;
    Color32::from_rgba_unmultiplied(value.r, value.g, value.b, opacity)
}

pub const fn round_caps(ink: Ink) -> bool {
    matches!(ink, Ink::Row(_) | Ink::Guide | Ink::Set | Ink::Meet)
}

pub fn capsule(from: Pos2, to: Pos2, width: f32) -> Vec<Pos2> {
    let radius = (width / 2.0).max(0.0);
    let delta = to - from;
    let length = delta.length();
    let along = if length > f32::EPSILON {
        delta / length
    } else {
        vec2(1.0, 0.0)
    };
    let across = vec2(-along.y, along.x);
    let steps = ((radius * 4.0).ceil() as usize).clamp(4, 16);
    let mut points = Vec::with_capacity(2 * steps + 2);
    for (center, side, forward) in [(to, across, along), (from, -across, -along)] {
        for step in 0..=steps {
            let angle = std::f32::consts::PI * step as f32 / steps as f32;
            let (sin, cos) = angle.sin_cos();
            points.push(center + (side * cos + forward * sin) * radius);
        }
    }
    points
}

pub fn dot_count(size: f64, spacing: f64, radius: f64) -> usize {
    if spacing.is_nan() || spacing <= 0.0 {
        return 0;
    }
    let room = size - radius - spacing / 2.0;
    if room.is_nan() || room < 0.0 {
        0
    } else {
        (room / spacing).floor() as usize + 1
    }
}

pub fn anchored(at: Pt, size: Vec2, horizontal: Anchor, vertical: Anchor, bounds: Vec2) -> Pos2 {
    let shift = |anchor: Anchor, extent: f32| match anchor {
        Anchor::Start => 0.0,
        Anchor::Middle => extent / 2.0,
        Anchor::End => extent,
    };
    let x = at.x as f32 - shift(horizontal, size.x);
    let y = at.y as f32 - shift(vertical, size.y);
    pos2(
        x.min(bounds.x - size.x).max(0.0),
        y.min(bounds.y - size.y).max(0.0),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

impl Frame {
    pub const fn new(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Frame {
        Frame {
            x,
            y,
            width,
            height,
            radius,
        }
    }

    pub fn inside(rect: Rect, radius: f32) -> Frame {
        Frame::new(
            rect.x as f32,
            rect.y as f32,
            (rect.w - 1) as f32,
            (rect.h - 1) as f32,
            radius,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlacedLine {
    pub text: String,
    pub x: i32,
    pub y: i32,
}

pub fn plan_text(
    text: &str,
    rect: Rect,
    align: Align,
    valign: VAlign,
    metric: TextMetrics,
    measure: &dyn Fn(&str) -> i32,
) -> Vec<PlacedLine> {
    let line_height = metric.height.max(1);
    let (left_margin, right_margin) = margins_for(metric);
    let left = rect.x + left_margin;
    let right = rect.right() - right_margin;
    let available = (right - left).max(1);
    let paragraphs: Vec<&str> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let wrapping = paragraphs.iter().any(|line| measure(line) > available);
    let lines: Vec<String> = if wrapping {
        paragraphs
            .iter()
            .flat_map(|line| wrap(line, available, measure))
            .collect()
    } else {
        paragraphs.iter().map(|line| (*line).to_string()).collect()
    };
    let total = lines.len() as i32 * line_height;
    let top = match valign {
        VAlign::Top => rect.y,
        VAlign::Center if total > rect.h => rect.y,
        VAlign::Center => rect.y + rect.h / 2 - total / 2,
    };
    let mut placed = Vec::with_capacity(lines.len());
    for (index, line) in lines.into_iter().enumerate() {
        let y = top + index as i32 * line_height;
        if wrapping && index > 0 && y + line_height > rect.bottom() {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let width = measure(&line);
        let x = match align {
            Align::Left => left,
            Align::Center => (left + right - width) / 2,
            Align::Right => right - width,
        };
        placed.push(PlacedLine { text: line, x, y });
    }
    placed
}

pub fn padded(rect: Rect, margins: (i32, i32)) -> Rect {
    Rect::new(
        rect.x - margins.0,
        rect.y,
        rect.w + margins.0 + margins.1,
        rect.h,
    )
}

pub fn ring_width(scale: f32) -> f32 {
    (2.0 * scale).round().max(2.0)
}

fn icon_shapes(icon: Icon) -> &'static [Shape] {
    static SHAPES: OnceLock<Vec<Vec<Shape>>> = OnceLock::new();
    let all = SHAPES.get_or_init(|| Icon::ALL.iter().map(|icon| icon.shapes()).collect());
    all.get(icon.index()).map_or(&[], Vec::as_slice)
}

#[derive(Clone)]
pub struct Canvas {
    pub painter: Painter,
    pub origin: Pos2,
}

impl Canvas {
    pub fn new(painter: Painter, origin: Pos2) -> Canvas {
        Canvas { painter, origin }
    }

    pub fn ctx(&self) -> &egui::Context {
        self.painter.ctx()
    }

    pub fn offset(&self, dx: i32, dy: i32) -> Canvas {
        Canvas {
            painter: self.painter.clone(),
            origin: self.origin + vec2(dx as f32, dy as f32),
        }
    }

    pub fn faded(&self, opacity: f32) -> Canvas {
        let mut painter = self.painter.clone();
        painter.multiply_opacity(opacity);
        Canvas {
            painter,
            origin: self.origin,
        }
    }

    pub fn point(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.origin.x + x, self.origin.y + y)
    }

    pub fn at(&self, rect: Rect) -> egui::Rect {
        egui::Rect::from_min_size(
            self.point(rect.x as f32, rect.y as f32),
            vec2(rect.w.max(0) as f32, rect.h.max(0) as f32),
        )
    }

    pub fn measure(&self, text: &str, font: Font) -> i32 {
        fonts::width(self.ctx(), text, font)
    }

    pub fn measure_padded(&self, text: &str, font: Font) -> i32 {
        let (left, right) = margins_for(fonts::metrics(font));
        self.measure(text, font) + left + right
    }

    pub fn fill_rect(&self, rect: Rect, value: Color) {
        if rect.is_empty() {
            return;
        }
        self.painter
            .rect_filled(self.at(rect), CornerRadius::ZERO, color(value));
    }

    fn shape(&self, frame: Frame) -> (egui::Rect, CornerRadius) {
        let diameter = (frame.radius * 2.0)
            .min(frame.width)
            .min(frame.height)
            .max(0.0);
        let rect = egui::Rect::from_min_max(
            self.point(frame.x + 0.5, frame.y + 0.5),
            self.point(frame.x + frame.width + 0.5, frame.y + frame.height + 0.5),
        );
        (
            rect,
            CornerRadius::same((diameter / 2.0).round().clamp(0.0, 255.0) as u8),
        )
    }

    pub fn rounded_box(&self, frame: Frame, fill: Color, stroke: Option<Color>) {
        if frame.width <= 0.0 || frame.height <= 0.0 {
            return;
        }
        let (rect, radius) = self.shape(frame);
        match stroke {
            Some(stroke) => {
                self.painter.rect(
                    rect,
                    radius,
                    color(fill),
                    Stroke::new(1.0, color(stroke)),
                    StrokeKind::Middle,
                );
            }
            None => {
                self.painter.rect_filled(rect, radius, color(fill));
            }
        }
    }

    pub fn rounded_outline(&self, frame: Frame, value: Color) {
        if frame.width <= 0.0 || frame.height <= 0.0 || value.a == 0 {
            return;
        }
        let (rect, radius) = self.shape(frame);
        self.painter.rect_stroke(
            rect,
            radius,
            Stroke::new(1.0, color(value)),
            StrokeKind::Middle,
        );
    }

    pub fn ring(&self, frame: Frame, value: Color, width: f32) {
        if frame.width <= 0.0 || frame.height <= 0.0 || value.a == 0 || width <= 0.0 {
            return;
        }
        let rect = egui::Rect::from_min_size(
            self.point(frame.x, frame.y),
            vec2(frame.width, frame.height),
        );
        let radius = frame
            .radius
            .min(frame.width / 2.0)
            .min(frame.height / 2.0)
            .max(0.0);
        self.painter.rect_stroke(
            rect,
            CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8),
            Stroke::new(width, color(value)),
            StrokeKind::Inside,
        );
    }

    pub fn filled_round(&self, rect: Rect, radius: f32, fill: Color, stroke: Option<Color>) {
        if rect.is_empty() {
            return;
        }
        let radius = radius.min(rect.w as f32 / 2.0).min(rect.h as f32 / 2.0);
        let corner = CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8);
        match stroke {
            Some(stroke) => {
                self.painter.rect(
                    self.at(rect),
                    corner,
                    color(fill),
                    Stroke::new(1.0, color(stroke)),
                    StrokeKind::Inside,
                );
            }
            None => {
                self.painter.rect_filled(self.at(rect), corner, color(fill));
            }
        }
    }

    pub fn card(&self, rect: Rect, fill: Color, stroke: Color, highlight: Option<Color>) {
        self.rounded_box(
            Frame::new(
                (rect.x + 1) as f32,
                (rect.y + 1) as f32,
                (rect.w - 3) as f32,
                (rect.h - 3) as f32,
                theme::CARD_RADIUS,
            ),
            fill,
            Some(stroke),
        );
        if let Some(highlight) = highlight {
            self.rounded_outline(
                Frame::new(
                    (rect.x + 4) as f32,
                    (rect.y + 4) as f32,
                    (rect.w - 9) as f32,
                    (rect.h - 9) as f32,
                    theme::CARD_HIGHLIGHT_RADIUS,
                ),
                highlight,
            );
        }
    }

    pub fn icon(&self, icon: Icon, x: f32, y: f32, width: f32, height: f32, value: Color) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let scale_x = width / VIEWBOX;
        let scale_y = height / VIEWBOX;
        let stroke_width = theme::ICON_STROKE * (scale_x + scale_y) / 2.0;
        let paint = color(value);
        let stroke = Stroke::new(stroke_width, paint);
        let map = |(px, py): (f32, f32)| self.point(x + 0.5 + px * scale_x, y + 0.5 + py * scale_y);
        let joint = |point: Pos2| {
            self.painter.circle_filled(point, stroke_width / 2.0, paint);
        };
        let polyline = |points: &[(f32, f32)], closed: bool| {
            let mapped: Vec<Pos2> = points.iter().copied().map(map).collect();
            for pair in mapped.windows(2) {
                self.painter.line_segment([pair[0], pair[1]], stroke);
            }
            if closed && mapped.len() > 2 {
                self.painter
                    .line_segment([mapped[mapped.len() - 1], mapped[0]], stroke);
            }
            for point in mapped {
                joint(point);
            }
        };
        for shape in icon_shapes(icon) {
            match shape {
                Shape::Line { x1, y1, x2, y2 } => polyline(&[(*x1, *y1), (*x2, *y2)], false),
                Shape::Polyline(points) => polyline(points, false),
                Shape::Polygon(points) => polyline(points, true),
                Shape::Rect {
                    x: rx,
                    y: ry,
                    width: rw,
                    height: rh,
                    radius,
                } => {
                    let rect = egui::Rect::from_min_max(map((*rx, *ry)), map((rx + rw, ry + rh)));
                    let radius = (radius * (scale_x + scale_y) / 2.0)
                        .min(rect.width() / 2.0)
                        .min(rect.height() / 2.0);
                    self.painter.rect_stroke(
                        rect,
                        CornerRadius::same(radius.round().clamp(0.0, 255.0) as u8),
                        stroke,
                        StrokeKind::Middle,
                    );
                }
                Shape::Circle { cx, cy, r } => {
                    self.painter.circle_stroke(
                        map((*cx, *cy)),
                        r * (scale_x + scale_y) / 2.0,
                        stroke,
                    );
                }
            }
        }
    }

    pub fn icon_view(&self, icon: Icon, rect: Rect, value: Color) {
        self.icon(
            icon,
            (rect.x + 1) as f32,
            (rect.y + 1) as f32,
            (rect.w - 2) as f32,
            (rect.h - 2) as f32,
            value,
        );
    }

    pub fn galley(&self, text: &str, font: Font, value: Color) -> Arc<egui::Galley> {
        fonts::galley(self.ctx(), text, font, color(value))
    }

    pub fn line(&self, text: &str, font: Font, value: Color, x: i32, y: i32, clip: Rect) {
        if text.is_empty() || clip.is_empty() {
            return;
        }
        let metric = fonts::metrics(font);
        let shift = metric.ascent as f32 - fonts::baseline(self.ctx(), font);
        let galley = self.galley(text, font, value);
        let position = self.point(x as f32, y as f32 + shift).round();
        self.painter
            .with_clip_rect(self.at(clip).intersect(self.painter.clip_rect()))
            .galley(position, galley, color(value));
    }

    pub fn text(
        &self,
        text: &str,
        font: Font,
        value: Color,
        rect: Rect,
        align: Align,
        valign: VAlign,
    ) {
        if text.is_empty() || rect.is_empty() {
            return;
        }
        let metric = fonts::metrics(font);
        let measure = |line: &str| self.measure(line, font);
        for line in plan_text(text, rect, align, valign, metric, &measure) {
            self.line(&line.text, font, value, line.x, line.y, rect);
        }
    }

    pub fn runs(&self, runs: &[(&str, Color)], gap: i32, font: Font, x: i32, rect: Rect) {
        let metric = fonts::metrics(font);
        let y = rect.y + rect.h / 2 - metric.height.max(1) / 2;
        let mut cursor = x;
        for (text, value) in runs {
            if text.is_empty() {
                continue;
            }
            self.line(text, font, *value, cursor, y, rect);
            cursor += self.measure(text, font) + gap;
        }
    }

    pub fn dotted_underline(&self, x: i32, y: i32, width: i32, value: Color) {
        let mut cursor = x;
        while cursor < x + width {
            self.fill_rect(Rect::new(cursor, y, 1, 1), value);
            cursor += 2;
        }
    }

    pub fn scene(&self, plot: Rect, items: &[Item], palette: &Palette) {
        if plot.is_empty() || items.is_empty() {
            return;
        }
        let area = self.at(plot);
        let origin = area.min;
        let place = |point: Pt| pos2(origin.x + point.x as f32, origin.y + point.y as f32);
        let mut shapes = Vec::with_capacity(items.len());
        for item in items {
            let paint = tint(item.ink.color(palette), item.alpha);
            if paint.a() == 0 {
                continue;
            }
            match &item.shape {
                Figure::Dots { spacing, radius } => {
                    let columns = dot_count(f64::from(plot.w), *spacing, *radius);
                    let rows = dot_count(f64::from(plot.h), *spacing, *radius);
                    shapes.reserve(columns * rows);
                    for row in 0..rows {
                        for column in 0..columns {
                            let center = Pt::new(
                                spacing / 2.0 + column as f64 * spacing,
                                spacing / 2.0 + row as f64 * spacing,
                            );
                            shapes.push(egui::Shape::circle_filled(
                                place(center),
                                *radius as f32,
                                paint,
                            ));
                        }
                    }
                }
                Figure::Segment { from, to, width } => {
                    let (from, to) = (place(*from), place(*to));
                    if round_caps(item.ink) {
                        shapes.push(egui::Shape::convex_polygon(
                            capsule(from, to, *width as f32),
                            paint,
                            Stroke::NONE,
                        ));
                    } else {
                        shapes.push(egui::Shape::line_segment(
                            [from, to],
                            Stroke::new(*width as f32, paint),
                        ));
                    }
                }
                Figure::Polygon { points } => {
                    shapes.push(egui::Shape::convex_polygon(
                        points.iter().map(|point| place(*point)).collect(),
                        paint,
                        Stroke::NONE,
                    ));
                }
                Figure::Outline { points, width } => {
                    shapes.push(egui::Shape::closed_line(
                        points.iter().map(|point| place(*point)).collect(),
                        Stroke::new(*width as f32, paint),
                    ));
                }
                Figure::Disc { center, radius } => {
                    shapes.push(egui::Shape::circle_filled(
                        place(*center),
                        *radius as f32,
                        paint,
                    ));
                }
                Figure::Ring {
                    center,
                    radius,
                    width,
                } => {
                    shapes.push(egui::Shape::circle_stroke(
                        place(*center),
                        *radius as f32,
                        Stroke::new(*width as f32, paint),
                    ));
                }
                Figure::Text {
                    at,
                    text,
                    horizontal,
                    vertical,
                } => {
                    let galley = fonts::galley(self.ctx(), text, Font::Small, Color32::PLACEHOLDER);
                    let corner = anchored(*at, galley.size(), *horizontal, *vertical, area.size());
                    shapes.push(egui::Shape::galley(
                        (origin + corner.to_vec2()).round(),
                        galley,
                        paint,
                    ));
                }
            }
        }
        self.painter.with_clip_rect(area).extend(shapes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sistemas_lineares_ui::text::keep_groups;

    fn medir(texto: &str) -> i32 {
        texto.chars().count() as i32 * 10
    }

    #[test]
    fn planeja_texto_como_no_windows() {
        let metrica = TextMetrics {
            height: 18,
            ascent: 14,
            descent: 4,
            external_leading: 0,
        };
        let linhas = plan_text(
            "um\ndois",
            Rect::new(0, 0, 100, 60),
            Align::Left,
            VAlign::Top,
            metrica,
            &medir,
        );
        assert_eq!(
            linhas,
            vec![
                PlacedLine {
                    text: "um".to_string(),
                    x: 3,
                    y: 0
                },
                PlacedLine {
                    text: "dois".to_string(),
                    x: 3,
                    y: 18
                }
            ]
        );
        let centro = plan_text(
            "ab",
            Rect::new(0, 0, 100, 40),
            Align::Center,
            VAlign::Center,
            metrica,
            &medir,
        );
        assert_eq!(centro[0].x, (3 + 95 - 20) / 2);
        assert_eq!(centro[0].y, 20 - 9);
        let direita = plan_text(
            "abc",
            Rect::new(10, 0, 100, 20),
            Align::Right,
            VAlign::Top,
            metrica,
            &medir,
        );
        assert_eq!(direita[0].x, 110 - 5 - 30);
    }

    #[test]
    fn quebra_de_linha_respeita_os_grupos() {
        let metrica = TextMetrics {
            height: 18,
            ascent: 14,
            descent: 4,
            external_leading: 0,
        };
        let linhas = |texto: &str| -> Vec<String> {
            plan_text(
                texto,
                Rect::new(0, 0, 88, 200),
                Align::Left,
                VAlign::Top,
                metrica,
                &medir,
            )
            .into_iter()
            .map(|linha| linha.text)
            .collect()
        };
        let texto = "em (2, 1): esse ponto";
        assert_eq!(linhas(texto), ["em (2,", "1): esse", "ponto"]);
        assert_eq!(
            linhas(&keep_groups(texto)),
            ["em", "(2,\u{a0}1):", "esse", "ponto"]
        );
    }

    #[test]
    fn cores_margens_e_anel() {
        assert_eq!(color(theme::ACCENT), Color32::from_rgb(206, 169, 243));
        assert_eq!(with_alpha(theme::ERROR, 70).a, 70);
        assert_eq!(Frame::inside(Rect::new(2, 3, 10, 20), 4.0).width, 9.0);
        assert_eq!(
            padded(Rect::new(10, 5, 20, 8), (3, 5)),
            Rect::new(7, 5, 28, 8)
        );
        assert_eq!(ring_width(1.0), 2.0);
        assert_eq!(ring_width(2.0), 4.0);
        assert_eq!(ring_width(0.5), 2.0);
    }

    #[test]
    fn tinta_com_opacidade_da_cena() {
        assert_eq!(tint(theme::MINT, 1.0), color(theme::MINT));
        assert_eq!(tint(theme::MINT, 0.0), Color32::TRANSPARENT);
        assert_eq!(tint(theme::MINT, f64::NAN), Color32::TRANSPARENT);
        assert_eq!(tint(theme::MINT, 7.0), color(theme::MINT));
        assert_eq!(
            tint(theme::ACCENT, 0.26),
            Color32::from_rgba_unmultiplied(206, 169, 243, 66)
        );
        assert_eq!(
            tint(with_alpha(theme::ACCENT, 128), 0.5),
            Color32::from_rgba_unmultiplied(206, 169, 243, 64)
        );
    }

    #[test]
    fn pontas_redondas_so_nas_retas_e_tracos() {
        for ink in [Ink::Row(0), Ink::Row(2), Ink::Guide, Ink::Set, Ink::Meet] {
            assert!(round_caps(ink), "{ink:?}");
        }
        for ink in [Ink::Grid, Ink::Axis, Ink::Edge, Ink::Texture] {
            assert!(!round_caps(ink), "{ink:?}");
        }
    }

    #[test]
    fn capsula_envolve_o_segmento_com_meia_largura() {
        let from = pos2(10.0, 20.0);
        let to = pos2(40.0, 60.0);
        let points = capsule(from, to, 4.0);
        assert!(points.len() >= 10);
        let direction = (to - from).normalized();
        let distance = |point: Pos2| {
            let offset = point - from;
            let along = offset.dot(direction).clamp(0.0, (to - from).length());
            (offset - direction * along).length()
        };
        for point in &points {
            assert!((distance(*point) - 2.0).abs() < 1e-3, "{point:?}");
        }
        let farthest = points
            .iter()
            .map(|point| (*point - from).dot(direction))
            .fold(f32::MIN, f32::max);
        assert!((farthest - (50.0 + 2.0)).abs() < 1e-3, "{farthest}");
        let nearest = points
            .iter()
            .map(|point| (*point - from).dot(direction))
            .fold(f32::MAX, f32::min);
        assert!((nearest + 2.0).abs() < 1e-3, "{nearest}");
        let turns: Vec<f32> = (0..points.len())
            .map(|index| {
                let a = points[index];
                let b = points[(index + 1) % points.len()];
                let c = points[(index + 2) % points.len()];
                (b - a).x * (c - b).y - (b - a).y * (c - b).x
            })
            .collect();
        assert!(
            turns.iter().all(|turn| *turn <= 1e-3) || turns.iter().all(|turn| *turn >= -1e-3),
            "a cápsula é convexa: {turns:?}"
        );
        let ponto = capsule(from, from, 2.0);
        for point in &ponto {
            assert!(((*point - from).length() - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn textura_cabe_inteira_na_area() {
        assert_eq!(dot_count(267.0, 8.0, 0.9), 33);
        assert_eq!(dot_count(128.0, 8.0, 0.9), 16);
        assert_eq!(dot_count(163.0, 8.0, 0.9), 20);
        assert_eq!(dot_count(78.0, 8.0, 0.9), 10);
        assert_eq!(dot_count(4.9, 8.0, 0.9), 1);
        assert_eq!(dot_count(4.8, 8.0, 0.9), 0);
        assert_eq!(dot_count(100.0, 0.0, 0.9), 0);
        assert_eq!(dot_count(100.0, f64::NAN, 0.9), 0);
    }

    #[test]
    fn texto_ancorado_e_trazido_para_dentro() {
        let tamanho = vec2(20.0, 14.0);
        let area = vec2(200.0, 100.0);
        assert_eq!(
            anchored(
                Pt::new(50.0, 40.0),
                tamanho,
                Anchor::Start,
                Anchor::Start,
                area
            ),
            pos2(50.0, 40.0)
        );
        assert_eq!(
            anchored(
                Pt::new(50.0, 40.0),
                tamanho,
                Anchor::Middle,
                Anchor::Middle,
                area
            ),
            pos2(40.0, 33.0)
        );
        assert_eq!(
            anchored(Pt::new(50.0, 40.0), tamanho, Anchor::End, Anchor::End, area),
            pos2(30.0, 26.0)
        );
        assert_eq!(
            anchored(
                Pt::new(198.0, 99.0),
                tamanho,
                Anchor::Start,
                Anchor::Start,
                area
            ),
            pos2(180.0, 86.0)
        );
        assert_eq!(
            anchored(Pt::new(3.0, 2.0), tamanho, Anchor::End, Anchor::End, area),
            pos2(0.0, 0.0)
        );
        assert_eq!(
            anchored(
                Pt::new(5.0, 5.0),
                vec2(300.0, 14.0),
                Anchor::Middle,
                Anchor::Start,
                area
            ),
            pos2(0.0, 5.0),
            "texto maior que a área começa na borda esquerda"
        );
    }
}
