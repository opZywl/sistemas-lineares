use std::sync::{Arc, OnceLock};

use eframe::egui::{self, Color32, CornerRadius, Painter, Pos2, Stroke, StrokeKind, pos2, vec2};
use sistemas_lineares_ui::icons::{Icon, Shape, VIEWBOX};
use sistemas_lineares_ui::layout::Rect;
use sistemas_lineares_ui::text::{TextMetrics, margins_for, wrap};
use sistemas_lineares_ui::theme::{self, Color, Font};

use crate::fonts;

pub fn color(value: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(value.r, value.g, value.b, value.a)
}

pub fn with_alpha(value: Color, alpha: u8) -> Color {
    Color { a: alpha, ..value }
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
