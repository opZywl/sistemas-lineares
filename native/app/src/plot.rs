use sistemas_lineares_ui::graph::{Anchor, Ink, Item, Pt, Shape, TEXT_HEIGHT};

use crate::gfx::Painter;
use crate::layout::Rect;
use crate::theme::{Color, Font, Palette};

pub const LABEL_FONT: Font = Font::Small;
pub const GDI_MIN_ALPHA: f64 = 0.1;

pub fn tint(ink: Ink, alpha: f64, palette: &Palette, fallback: bool) -> Option<Color> {
    let base = ink.color(palette);
    let alpha = if alpha.is_nan() {
        0.0
    } else {
        alpha.clamp(0.0, 1.0)
    };
    if fallback {
        return (alpha >= GDI_MIN_ALPHA)
            .then(|| base.mix(palette.graph_fill, (alpha * 100.0).round() as u8));
    }
    let opacity = (f64::from(base.a) * alpha).round() as u8;
    (opacity > 0).then_some(Color { a: opacity, ..base })
}

pub const fn rounded(ink: Ink) -> bool {
    matches!(ink, Ink::Row(_) | Ink::Guide | Ink::Set | Ink::Meet)
}

pub const fn translucent_fill(item: &Item) -> bool {
    matches!(
        (item.ink, &item.shape),
        (Ink::Halo, Shape::Disc { .. }) | (Ink::Row(_), Shape::Polygon { .. })
    )
}

pub fn label_band(unit: f64) -> i32 {
    ((TEXT_HEIGHT * unit).round() as i32).max(1)
}

pub const fn label_offset(band: i32, cap: i32, ascent: i32) -> i32 {
    (band + cap) / 2 - ascent
}

pub fn anchored(
    at: Pt,
    horizontal: Anchor,
    vertical: Anchor,
    size: (i32, i32),
    area: Rect,
) -> (i32, i32) {
    let (width, height) = (f64::from(size.0), f64::from(size.1));
    let left = match horizontal {
        Anchor::Start => at.x,
        Anchor::Middle => at.x - width / 2.0,
        Anchor::End => at.x - width,
    };
    let top = match vertical {
        Anchor::Start => at.y,
        Anchor::Middle => at.y - height / 2.0,
        Anchor::End => at.y - height,
    };
    let x = (left.round() as i32).clamp(0, (area.w - size.0).max(0));
    let y = (top.round() as i32).clamp(0, (area.h - size.1).max(0));
    (area.x + x, area.y + y)
}

pub fn draw(painter: &Painter, items: &[Item], area: Rect, unit: f64, palette: &Palette) {
    if area.is_empty() {
        return;
    }
    let fallback = painter.is_fallback();
    let (left, top) = (f64::from(area.x), f64::from(area.y));
    let place = |point: Pt| ((left + point.x) as f32, (top + point.y) as f32);
    let band = label_band(unit);
    let offset = label_offset(
        band,
        painter.cap_height(LABEL_FONT),
        painter.text_metrics(LABEL_FONT).ascent,
    );
    painter.clipped(area, |painter| {
        for item in items {
            if fallback && translucent_fill(item) {
                continue;
            }
            let Some(color) = tint(item.ink, item.alpha, palette, fallback) else {
                continue;
            };
            match &item.shape {
                Shape::Dots { spacing, radius } => {
                    painter.dots(area, *spacing as f32, *radius as f32, color)
                }
                Shape::Segment { from, to, width } => painter.segment(
                    place(*from),
                    place(*to),
                    *width as f32,
                    color,
                    rounded(item.ink),
                ),
                Shape::Polygon { points } => {
                    let points: Vec<(f32, f32)> =
                        points.iter().map(|point| place(*point)).collect();
                    painter.polygon(&points, color);
                }
                Shape::Outline { points, width } => {
                    let points: Vec<(f32, f32)> =
                        points.iter().map(|point| place(*point)).collect();
                    painter.outline(&points, *width as f32, color);
                }
                Shape::Disc { center, radius } => {
                    painter.disc(place(*center), *radius as f32, color)
                }
                Shape::Ring {
                    center,
                    radius,
                    width,
                } => painter.circle(place(*center), *radius as f32, *width as f32, color),
                Shape::Text {
                    at,
                    text,
                    horizontal,
                    vertical,
                } => {
                    let size = (painter.measure(text, LABEL_FONT), band);
                    let (x, y) = anchored(*at, *horizontal, *vertical, size, area);
                    painter.line(text, LABEL_FONT, color, x, y + offset, area);
                }
            }
        }
    });
}
