#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Polyline(Vec<(f32, f32)>),
    Polygon(Vec<(f32, f32)>),
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
    },
    Circle {
        cx: f32,
        cy: f32,
        r: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Matrix,
    Grid,
    ArrowRight,
    External,
    Copy,
    Reset,
    Sparkles,
}

pub const VIEWBOX: f32 = 24.0;

impl Icon {
    pub const ALL: [Icon; 7] = [
        Icon::Matrix,
        Icon::Grid,
        Icon::ArrowRight,
        Icon::External,
        Icon::Copy,
        Icon::Reset,
        Icon::Sparkles,
    ];

    pub const fn index(self) -> usize {
        match self {
            Icon::Matrix => 0,
            Icon::Grid => 1,
            Icon::ArrowRight => 2,
            Icon::External => 3,
            Icon::Copy => 4,
            Icon::Reset => 5,
            Icon::Sparkles => 6,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Icon::Matrix => "matrix",
            Icon::Grid => "grid",
            Icon::ArrowRight => "arrow-right",
            Icon::External => "external",
            Icon::Copy => "copy",
            Icon::Reset => "reset",
            Icon::Sparkles => "sparkles",
        }
    }

    pub const fn source(self) -> &'static str {
        match self {
            Icon::Matrix => include_str!("../../../assets/matrix.svg"),
            Icon::Grid => include_str!("../../../assets/grid.svg"),
            Icon::ArrowRight => include_str!("../../../assets/arrow-right.svg"),
            Icon::External => include_str!("../../../assets/external.svg"),
            Icon::Copy => include_str!("../../../assets/copy.svg"),
            Icon::Reset => include_str!("../../../assets/reset.svg"),
            Icon::Sparkles => include_str!("../../../assets/sparkles.svg"),
        }
    }

    pub fn shapes(self) -> Vec<Shape> {
        parse(self.source())
    }
}

pub fn parse(svg: &str) -> Vec<Shape> {
    let mut shapes = Vec::new();
    let mut rest = svg;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        let end = tag_end(rest);
        let tag = &rest[..end];
        rest = &rest[end..];
        let name_end = tag
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .unwrap_or(tag.len());
        let name = &tag[..name_end];
        let attributes = attributes(&tag[name_end..]);
        let number = |key: &str, fallback: f32| {
            attributes
                .iter()
                .find(|(name, _)| *name == key)
                .and_then(|(_, value)| value.trim().parse::<f32>().ok())
                .unwrap_or(fallback)
        };
        let text = |key: &str| {
            attributes
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| *value)
        };
        match name {
            "line" => shapes.push(Shape::Line {
                x1: number("x1", 0.0),
                y1: number("y1", 0.0),
                x2: number("x2", 0.0),
                y2: number("y2", 0.0),
            }),
            "circle" => shapes.push(Shape::Circle {
                cx: number("cx", 0.0),
                cy: number("cy", 0.0),
                r: number("r", 1.0),
            }),
            "rect" => shapes.push(Shape::Rect {
                x: number("x", 0.0),
                y: number("y", 0.0),
                width: number("width", 1.0),
                height: number("height", 1.0),
                radius: number("rx", 0.0),
            }),
            "polyline" | "polygon" => {
                let points = points(text("points").unwrap_or(""));
                if points.len() >= 2 {
                    shapes.push(if name == "polygon" {
                        Shape::Polygon(points)
                    } else {
                        Shape::Polyline(points)
                    });
                }
            }
            _ => {}
        }
    }
    shapes
}

fn tag_end(text: &str) -> usize {
    let mut quote = None;
    for (index, character) in text.char_indices() {
        match (quote, character) {
            (None, '"' | '\'') => quote = Some(character),
            (Some(open), _) if open == character => quote = None,
            (None, '>') => return index,
            _ => {}
        }
    }
    text.len()
}

fn attributes(text: &str) -> Vec<(&str, &str)> {
    let mut result = Vec::new();
    let mut rest = text;
    while let Some(equals) = rest.find('=') {
        let name = rest[..equals].trim().trim_start_matches('/').trim();
        let after = rest[equals + 1..].trim_start();
        let Some(quote) = after.chars().next().filter(|c| *c == '"' || *c == '\'') else {
            break;
        };
        let body = &after[1..];
        let Some(close) = body.find(quote) else {
            break;
        };
        result.push((
            name.rsplit(char::is_whitespace).next().unwrap_or(name),
            &body[..close],
        ));
        rest = &body[close + 1..];
    }
    result
}

fn points(text: &str) -> Vec<(f32, f32)> {
    let numbers: Vec<f32> = text
        .split([' ', ',', '\t', '\n', '\r'])
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect();
    numbers
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todos_os_icones_tem_primitivas() {
        for icon in Icon::ALL {
            assert!(!icon.shapes().is_empty(), "{}", icon.name());
        }
    }

    #[test]
    fn interpreta_subconjunto_svg() {
        let shapes = parse(
            r#"<svg viewBox="0 0 24 24"><line x1="4" y1="12" x2="20" y2="12"/><polyline points="14,6 20,12 14,18"/><rect x="8" y="8" width="13" height="13" rx="2"/><circle cx="12" cy="12" r="3"/><polygon points="1 2 3 4 5 6"/></svg>"#,
        );
        assert_eq!(shapes.len(), 5);
        assert_eq!(
            shapes[0],
            Shape::Line {
                x1: 4.0,
                y1: 12.0,
                x2: 20.0,
                y2: 12.0
            }
        );
        assert_eq!(
            shapes[1],
            Shape::Polyline(vec![(14.0, 6.0), (20.0, 12.0), (14.0, 18.0)])
        );
        assert_eq!(
            shapes[2],
            Shape::Rect {
                x: 8.0,
                y: 8.0,
                width: 13.0,
                height: 13.0,
                radius: 2.0
            }
        );
        assert_eq!(
            shapes[3],
            Shape::Circle {
                cx: 12.0,
                cy: 12.0,
                r: 3.0
            }
        );
        assert_eq!(
            shapes[4],
            Shape::Polygon(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)])
        );
        assert_eq!(Icon::Matrix.shapes().len(), 8);
        assert_eq!(Icon::Grid.shapes().len(), 4);
    }
}
