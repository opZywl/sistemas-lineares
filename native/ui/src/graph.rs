use crate::theme::{Color, Palette};

pub const LIMIT: f64 = 1e100;
pub const MARGIN: f64 = 1.3;
pub const MIN_HALF: f64 = 2.5;
pub const BOX_HEIGHT: f64 = 0.6;
pub const APPEAR: f64 = 0.6;
pub const MORPH: f64 = 0.35;
pub const VANISH: f64 = 0.25;
pub const VIEW: f64 = 0.45;
pub const POP: f64 = 0.45;
pub const PULSE: f64 = 1.2;
pub const PULSE_DELAY: f64 = 0.15;
pub const PULSE_GAP: f64 = 0.4;
pub const PULSES: usize = 3;
pub const MARCH: f64 = 2.4;
pub const MARCH_SPEED: f64 = 18.0;
pub const LIGHT_UP: f64 = 0.5;
pub const LABEL_LIMIT: usize = 7;
pub const TEXT_HEIGHT: f64 = 14.0;
pub const CHAR_WIDTH: f64 = 7.0;

const SX_X: f64 = std::f64::consts::FRAC_1_SQRT_2;
const SX_Z: f64 = -std::f64::consts::FRAC_1_SQRT_2;
const SY_X: f64 = 0.29883623873011983;
const SY_Y: f64 = -0.9063077870366499;
const SY_Z: f64 = 0.29883623873011983;
const DEPTH_X: f64 = 0.6408563820557885;
const DEPTH_Y: f64 = 0.42261826174069944;
const DEPTH_Z: f64 = 0.6408563820557885;

pub const INTRO_TWO: &str = "Cada equação é uma reta no plano. Preencha uma equação e ela aparece aqui, na cor do seu número.";
pub const INTRO_THREE: &str = "Cada equação é um plano no espaço. Preencha uma equação e ele aparece aqui, na cor do seu número.";
pub const WAITING: &str =
    "Pressione Enter em um dos campos para resolver e marcar a solução no gráfico.";
pub const HUGE: &str = "Os valores são grandes demais para desenhar o gráfico; a resposta exata continua no resultado.";
pub const COINCIDENT_LINES: &str = "As retas coincidem: cada ponto delas é uma solução.";
pub const SHARED_LINE: &str = "Os planos se cruzam numa mesma reta: cada ponto dela é uma solução.";
pub const COINCIDENT_PLANES: &str = "Os planos coincidem: cada ponto deles é uma solução.";
pub const EVERYTHING_TWO: &str =
    "Todas as equações valem para qualquer ponto (0 = 0): o plano inteiro é solução.";
pub const EVERYTHING_THREE: &str =
    "Todas as equações valem para qualquer ponto (0 = 0): o espaço inteiro é solução.";
pub const PARALLEL_LINES: &str =
    "As retas são paralelas e nunca se cruzam: o sistema não tem solução.";
pub const PARALLEL_PLANES: &str =
    "Os planos são paralelos e nunca se encontram: o sistema não tem solução.";
pub const NO_COMMON_POINT: &str =
    "Não existe um ponto comum aos três planos: o sistema não tem solução.";
pub const GRAPH_NAME: &str = "Gráfico do sistema";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub normal: [f64; 3],
    pub offset: f64,
}

impl Plane {
    pub fn new(coefficients: [f64; 3], constant: f64) -> Option<Plane> {
        let [a, b, c] = coefficients;
        let length = (a * a + b * b + c * c).sqrt();
        let drawable = length > 0.0 && length.is_finite() && constant.is_finite();
        if !drawable {
            return None;
        }
        let offset = constant / length;
        if offset.abs() > LIMIT {
            return None;
        }
        Some(Plane {
            normal: [a / length, b / length, c / length],
            offset,
        })
    }

    pub fn foot(&self) -> [f64; 3] {
        scale(self.normal, self.offset)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Row {
    Blank,
    Partial,
    Drawn(Plane),
    Trivial,
    Impossible(String),
    Huge,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Pending,
    Waiting,
    Point {
        at: Option<[f64; 3]>,
        coordinates: Vec<String>,
    },
    Line,
    Plane,
    Everything,
    Nothing {
        rank: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sketch {
    pub dimension: usize,
    pub rows: Vec<Row>,
    pub verdict: Verdict,
}

impl Sketch {
    pub fn blank(dimension: usize) -> Sketch {
        let dimension = dimension.clamp(2, 3);
        Sketch {
            dimension,
            rows: vec![Row::Blank; dimension],
            verdict: Verdict::Pending,
        }
    }

    pub fn plane(&self, row: usize) -> Option<Plane> {
        match self.rows.get(row) {
            Some(Row::Drawn(plane)) => Some(*plane),
            _ => None,
        }
    }

    pub fn huge(&self) -> bool {
        self.rows.contains(&Row::Huge)
    }

    pub fn point(&self) -> Option<[f64; 3]> {
        match &self.verdict {
            Verdict::Point { at, .. } => *at,
            _ => None,
        }
    }

    pub fn coordinates(&self) -> &[String] {
        match &self.verdict {
            Verdict::Point { coordinates, .. } => coordinates,
            _ => &[],
        }
    }
}

fn numbered(rows: &[usize]) -> String {
    match rows {
        [] => String::new(),
        [only] => format!("a equação {only}"),
        [first @ .., last] => {
            let first: Vec<String> = first.iter().map(ToString::to_string).collect();
            format!("as equações {} e {last}", first.join(", "))
        }
    }
}

pub fn caption(sketch: &Sketch) -> String {
    let three = sketch.dimension == 3;
    let rows = &sketch.rows;
    let picked = |wanted: fn(&Row) -> bool| -> Vec<usize> {
        rows.iter()
            .enumerate()
            .filter(|(_, row)| wanted(row))
            .map(|(index, _)| index + 1)
            .collect()
    };
    if sketch.huge() {
        return HUGE.to_string();
    }
    match &sketch.verdict {
        Verdict::Pending => {
            let missing = picked(|row| matches!(row, Row::Blank | Row::Partial));
            if missing.is_empty() || missing.len() == rows.len() {
                return if three { INTRO_THREE } else { INTRO_TWO }.to_string();
            }
            format!(
                "Preencha {} para ver onde {} se cruzam.",
                numbered(&missing),
                if three { "os planos" } else { "as retas" }
            )
        }
        Verdict::Waiting => WAITING.to_string(),
        Verdict::Point { at, coordinates } => {
            let label = format!("({})", coordinates.join(", "));
            match at {
                None => format!("A solução é {label}, longe demais para caber no gráfico."),
                Some(_) if three => format!(
                    "Os três planos se cruzam em {label}: esse ponto é a solução do sistema."
                ),
                Some(_) => {
                    format!("As retas se cruzam em {label}: esse ponto é a solução do sistema.")
                }
            }
        }
        Verdict::Line if three => SHARED_LINE.to_string(),
        Verdict::Line => {
            let trivial = picked(|row| *row == Row::Trivial);
            let drawn = picked(|row| matches!(row, Row::Drawn(_)));
            match (trivial.as_slice(), drawn.as_slice()) {
                ([zero], [line]) => format!(
                    "A equação {zero} vale para qualquer ponto (0 = 0), então toda a reta da equação {line} é solução."
                ),
                _ => COINCIDENT_LINES.to_string(),
            }
        }
        Verdict::Plane => {
            let drawn = picked(|row| matches!(row, Row::Drawn(_)));
            match drawn.as_slice() {
                [plane] => format!(
                    "As outras equações valem para qualquer ponto (0 = 0), então todo o plano da equação {plane} é solução."
                ),
                _ => COINCIDENT_PLANES.to_string(),
            }
        }
        Verdict::Everything => if three {
            EVERYTHING_THREE
        } else {
            EVERYTHING_TWO
        }
        .to_string(),
        Verdict::Nothing { rank } => {
            let impossible = rows.iter().enumerate().find_map(|(index, row)| match row {
                Row::Impossible(constant) => Some((index + 1, constant)),
                _ => None,
            });
            match impossible {
                Some((index, constant)) => format!(
                    "A equação {index} diz que 0 = {constant}, o que nunca acontece: o sistema não tem solução."
                ),
                None if !three => PARALLEL_LINES.to_string(),
                None if *rank <= 1 => PARALLEL_PLANES.to_string(),
                None => NO_COMMON_POINT.to_string(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ink {
    Texture,
    Grid,
    Axis,
    Label,
    Pane,
    Edge,
    Row(usize),
    Meet,
    Guide,
    Set,
    Halo,
    Point,
    Backdrop,
}

impl Ink {
    pub const fn code(self) -> &'static str {
        match self {
            Ink::Texture => "t",
            Ink::Grid => "g",
            Ink::Axis => "a",
            Ink::Label => "l",
            Ink::Pane => "p",
            Ink::Edge => "e",
            Ink::Row(0) => "r0",
            Ink::Row(1) => "r1",
            Ink::Row(_) => "r2",
            Ink::Meet => "m",
            Ink::Guide => "u",
            Ink::Set => "s",
            Ink::Halo => "h",
            Ink::Point => "o",
            Ink::Backdrop => "b",
        }
    }

    pub const fn color(self, palette: &Palette) -> Color {
        match self {
            Ink::Texture => palette.graph_texture,
            Ink::Grid => palette.graph_grid,
            Ink::Axis => palette.graph_axis,
            Ink::Label => palette.graph_label,
            Ink::Pane => palette.graph_pane,
            Ink::Edge => palette.graph_edge,
            Ink::Row(0) => palette.graph_rows[0],
            Ink::Row(1) => palette.graph_rows[1],
            Ink::Row(_) => palette.graph_rows[2],
            Ink::Meet => palette.graph_meet,
            Ink::Guide => palette.graph_guide,
            Ink::Set => palette.graph_set,
            Ink::Halo => palette.graph_halo,
            Ink::Point => palette.graph_point,
            Ink::Backdrop => palette.graph_fill,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Pt {
    pub const fn new(x: f64, y: f64) -> Pt {
        Pt { x, y }
    }

    fn lerp(self, other: Pt, amount: f64) -> Pt {
        Pt::new(
            self.x + (other.x - self.x) * amount,
            self.y + (other.y - self.y) * amount,
        )
    }

    fn distance(self, other: Pt) -> f64 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

impl Anchor {
    const fn code(self) -> &'static str {
        match self {
            Anchor::Start => "s",
            Anchor::Middle => "m",
            Anchor::End => "e",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Dots {
        spacing: f64,
        radius: f64,
    },
    Segment {
        from: Pt,
        to: Pt,
        width: f64,
    },
    Polygon {
        points: Vec<Pt>,
    },
    Outline {
        points: Vec<Pt>,
        width: f64,
    },
    Disc {
        center: Pt,
        radius: f64,
    },
    Ring {
        center: Pt,
        radius: f64,
        width: f64,
    },
    Text {
        at: Pt,
        text: String,
        horizontal: Anchor,
        vertical: Anchor,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub shape: Shape,
    pub ink: Ink,
    pub alpha: f64,
}

fn quantize(value: f64) -> i64 {
    (value * 100.0 + 0.5).floor() as i64
}

fn describe_points(points: &[Pt]) -> String {
    points
        .iter()
        .map(|point| format!("{},{}", quantize(point.x), quantize(point.y)))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn describe(items: &[Item]) -> String {
    items
        .iter()
        .map(|item| {
            let shape = match &item.shape {
                Shape::Dots { spacing, radius } => {
                    format!("D{},{}", quantize(*spacing), quantize(*radius))
                }
                Shape::Segment { from, to, width } => format!(
                    "S{},{},{},{},{}",
                    quantize(from.x),
                    quantize(from.y),
                    quantize(to.x),
                    quantize(to.y),
                    quantize(*width)
                ),
                Shape::Polygon { points } => format!("P{}", describe_points(points)),
                Shape::Outline { points, width } => {
                    format!("O{}/{}", describe_points(points), quantize(*width))
                }
                Shape::Disc { center, radius } => format!(
                    "C{},{},{}",
                    quantize(center.x),
                    quantize(center.y),
                    quantize(*radius)
                ),
                Shape::Ring {
                    center,
                    radius,
                    width,
                } => format!(
                    "R{},{},{},{}",
                    quantize(center.x),
                    quantize(center.y),
                    quantize(*radius),
                    quantize(*width)
                ),
                Shape::Text {
                    at,
                    text,
                    horizontal,
                    vertical,
                } => format!(
                    "T{},{},{}{},{}",
                    quantize(at.x),
                    quantize(at.y),
                    horizontal.code(),
                    vertical.code(),
                    text
                ),
            };
            format!(
                "{}{}:{}",
                item.ink.code(),
                (item.alpha * 1000.0 + 0.5).floor() as i64,
                shape
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn clamp01(value: f64) -> f64 {
    if value.is_nan() {
        1.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

pub fn ease_out(value: f64) -> f64 {
    let rest = 1.0 - clamp01(value);
    1.0 - rest * rest * rest
}

pub fn ease_in_out(value: f64) -> f64 {
    let value = clamp01(value);
    if value < 0.5 {
        4.0 * value * value * value
    } else {
        let rest = -2.0 * value + 2.0;
        1.0 - rest * rest * rest / 2.0
    }
}

pub fn ease_out_back(value: f64) -> f64 {
    let value = clamp01(value);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    let rest = value - 1.0;
    1.0 + c3 * rest * rest * rest + c1 * rest * rest
}

fn add(first: [f64; 3], second: [f64; 3]) -> [f64; 3] {
    [
        first[0] + second[0],
        first[1] + second[1],
        first[2] + second[2],
    ]
}

fn sub(first: [f64; 3], second: [f64; 3]) -> [f64; 3] {
    [
        first[0] - second[0],
        first[1] - second[1],
        first[2] - second[2],
    ]
}

fn scale(vector: [f64; 3], amount: f64) -> [f64; 3] {
    [vector[0] * amount, vector[1] * amount, vector[2] * amount]
}

fn dot(first: [f64; 3], second: [f64; 3]) -> f64 {
    first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}

fn cross(first: [f64; 3], second: [f64; 3]) -> [f64; 3] {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}

fn lerp3(first: [f64; 3], second: [f64; 3], amount: f64) -> [f64; 3] {
    [
        first[0] + (second[0] - first[0]) * amount,
        first[1] + (second[1] - first[1]) * amount,
        first[2] + (second[2] - first[2]) * amount,
    ]
}

fn lerp_plane(from: Plane, to: Plane, amount: f64) -> Plane {
    let (normal, offset) = if dot(from.normal, to.normal) < 0.0 {
        (scale(to.normal, -1.0), -to.offset)
    } else {
        (to.normal, to.offset)
    };
    let blended = lerp3(from.normal, normal, amount);
    let length = dot(blended, blended).sqrt();
    if length < 1e-9 {
        return to;
    }
    let offset = from.offset + (offset - from.offset) * amount;
    Plane {
        normal: scale(blended, 1.0 / length),
        offset: offset / length,
    }
}

pub fn nice(raw: f64) -> f64 {
    if raw.is_nan() || raw <= 0.0 || raw.is_infinite() {
        return 1.0;
    }
    let mut power = 1.0;
    while power > raw {
        power /= 10.0;
    }
    while power * 10.0 <= raw {
        power *= 10.0;
    }
    for multiple in [1.0, 2.0, 5.0] {
        if power * multiple >= raw {
            return power * multiple;
        }
    }
    power * 10.0
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub center: [f64; 3],
    pub half: [f64; 3],
}

impl Bounds {
    fn lerp(self, other: Bounds, amount: f64) -> Bounds {
        Bounds {
            center: lerp3(self.center, other.center, amount),
            half: lerp3(self.half, other.half, amount),
        }
    }

    fn low(self) -> [f64; 3] {
        sub(self.center, self.half)
    }

    fn high(self) -> [f64; 3] {
        add(self.center, self.half)
    }
}

fn line_of(first: Plane, second: Plane) -> Option<([f64; 3], [f64; 3])> {
    let direction = cross(first.normal, second.normal);
    let length = dot(direction, direction);
    if length < 1e-10 {
        return None;
    }
    let point = scale(
        add(
            scale(cross(second.normal, direction), first.offset),
            scale(cross(direction, first.normal), second.offset),
        ),
        1.0 / length,
    );
    Some((point, direction))
}

fn solution_line(dimension: usize, planes: &[Option<Plane>]) -> Option<([f64; 3], [f64; 3])> {
    if dimension == 2 {
        let plane = planes.iter().flatten().next()?;
        return Some((plane.foot(), [-plane.normal[1], plane.normal[0], 0.0]));
    }
    for first in 0..planes.len() {
        for second in first + 1..planes.len() {
            if let (Some(one), Some(two)) = (planes[first], planes[second]) {
                if let Some(line) = line_of(one, two) {
                    return Some(line);
                }
            }
        }
    }
    None
}

pub fn bounds(sketch: &Sketch) -> Bounds {
    let three = sketch.dimension == 3;
    let mut anchors = vec![[0.0, 0.0, 0.0]];
    let planes: Vec<Option<Plane>> = (0..sketch.rows.len())
        .map(|row| sketch.plane(row))
        .collect();
    anchors.extend(planes.iter().flatten().map(Plane::foot));
    if let Some(point) = sketch.point() {
        anchors.push(point);
    }
    if three && sketch.verdict == Verdict::Line {
        if let Some((point, _)) = solution_line(3, &planes) {
            anchors.push(point);
        }
    }
    let mut low = anchors[0];
    let mut high = anchors[0];
    for anchor in &anchors {
        for axis in 0..3 {
            low[axis] = low[axis].min(anchor[axis]);
            high[axis] = high[axis].max(anchor[axis]);
        }
    }
    let center = scale(add(low, high), 0.5);
    let extent = scale(sub(high, low), 0.5);
    let half = if three {
        let size = MIN_HALF
            .max(extent[0] * MARGIN)
            .max(extent[2] * MARGIN)
            .max(extent[1] * MARGIN / BOX_HEIGHT);
        [size, size * BOX_HEIGHT, size]
    } else {
        [
            MIN_HALF.max(extent[0] * MARGIN),
            MIN_HALF.max(extent[1] * MARGIN),
            0.0,
        ]
    };
    let center = if three {
        center
    } else {
        [center[0], center[1], 0.0]
    };
    Bounds { center, half }
}

fn clip(
    point: [f64; 3],
    direction: [f64; 3],
    low: [f64; 3],
    high: [f64; 3],
    axes: usize,
) -> Option<([f64; 3], [f64; 3])> {
    let mut start = f64::NEG_INFINITY;
    let mut end = f64::INFINITY;
    for axis in 0..axes {
        if direction[axis].abs() < 1e-12 {
            if point[axis] < low[axis] || point[axis] > high[axis] {
                return None;
            }
            continue;
        }
        let first = (low[axis] - point[axis]) / direction[axis];
        let second = (high[axis] - point[axis]) / direction[axis];
        start = start.max(first.min(second));
        end = end.min(first.max(second));
    }
    if start.is_nan() || end.is_nan() || start >= end || start.is_infinite() || end.is_infinite() {
        return None;
    }
    Some((
        add(point, scale(direction, start)),
        add(point, scale(direction, end)),
    ))
}

fn pseudo_angle(x: f64, y: f64) -> f64 {
    if y >= 0.0 {
        if x >= 0.0 {
            y / (x + y)
        } else {
            1.0 - x / (y - x)
        }
    } else if x < 0.0 {
        2.0 - y / (-x - y)
    } else {
        3.0 + x / (x - y)
    }
}

fn section(plane: Plane, low: [f64; 3], high: [f64; 3]) -> Vec<[f64; 3]> {
    let corner = |index: usize| {
        [
            if index & 1 != 0 { high[0] } else { low[0] },
            if index & 2 != 0 { high[1] } else { low[1] },
            if index & 4 != 0 { high[2] } else { low[2] },
        ]
    };
    let size = sub(high, low);
    let tolerance = 1e-9 * dot(size, size);
    let mut points: Vec<[f64; 3]> = Vec::new();
    let mut push = |candidate: [f64; 3]| {
        let fresh = points.iter().all(|existing| {
            let gap = sub(*existing, candidate);
            dot(gap, gap) > tolerance
        });
        if fresh {
            points.push(candidate);
        }
    };
    for bit in [1usize, 2, 4] {
        for index in 0..8 {
            if index & bit != 0 {
                continue;
            }
            let start = corner(index);
            let end = corner(index | bit);
            let first = dot(plane.normal, start) - plane.offset;
            let second = dot(plane.normal, end) - plane.offset;
            if first == 0.0 {
                push(start);
            }
            if second == 0.0 {
                push(end);
            }
            if (first < 0.0 && second > 0.0) || (first > 0.0 && second < 0.0) {
                push(add(start, scale(sub(end, start), first / (first - second))));
            }
        }
    }
    if points.len() < 3 {
        return Vec::new();
    }
    let count = points.len() as f64;
    let center = scale(
        points.iter().fold([0.0; 3], |sum, point| add(sum, *point)),
        1.0 / count,
    );
    let normal = plane.normal;
    let axis = if normal[0].abs() <= normal[1].abs() && normal[0].abs() <= normal[2].abs() {
        [1.0, 0.0, 0.0]
    } else if normal[1].abs() <= normal[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let along = sub(axis, scale(normal, dot(axis, normal)));
    let along = scale(along, 1.0 / dot(along, along).sqrt());
    let across = cross(normal, along);
    let mut keyed: Vec<(f64, [f64; 3])> = points
        .into_iter()
        .map(|point| {
            let offset = sub(point, center);
            (pseudo_angle(dot(offset, along), dot(offset, across)), point)
        })
        .collect();
    keyed.sort_by(|first, second| first.0.total_cmp(&second.0));
    keyed.into_iter().map(|(_, point)| point).collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Shown {
    plane: Plane,
    progress: f64,
    alpha: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Track {
    target: Option<Plane>,
    appear: f64,
    from: Option<Plane>,
    morph: f64,
    ghost: Option<(Plane, f64)>,
    vanish: f64,
}

impl Default for Track {
    fn default() -> Self {
        Track {
            target: None,
            appear: f64::NEG_INFINITY,
            from: None,
            morph: f64::NEG_INFINITY,
            ghost: None,
            vanish: f64::NEG_INFINITY,
        }
    }
}

impl Track {
    fn shown(&self, now: f64) -> Option<Shown> {
        if let Some(target) = self.target {
            let plane = match self.from {
                Some(from) if now < self.morph + MORPH => {
                    lerp_plane(from, target, ease_in_out((now - self.morph) / MORPH))
                }
                _ => target,
            };
            return Some(Shown {
                plane,
                progress: ease_out((now - self.appear) / APPEAR),
                alpha: 1.0,
            });
        }
        let (plane, progress) = self.ghost?;
        let fade = (now - self.vanish) / VANISH;
        (fade < 1.0).then(|| Shown {
            plane,
            progress,
            alpha: 1.0 - clamp01(fade),
        })
    }

    fn busy(&self, now: f64) -> bool {
        match self.target {
            Some(_) => {
                now < self.appear + APPEAR || (self.from.is_some() && now < self.morph + MORPH)
            }
            None => self.ghost.is_some() && now < self.vanish + VANISH,
        }
    }

    fn retarget(&mut self, target: Option<Plane>, now: f64, animate: bool) {
        if self.target == target {
            return;
        }
        let shown = self.shown(now);
        if !animate {
            *self = Track {
                target,
                ..Track::default()
            };
            return;
        }
        match target {
            Some(plane) => {
                match shown {
                    Some(current) => {
                        if self.target.is_none() {
                            self.appear = f64::NEG_INFINITY;
                        }
                        self.from = Some(current.plane);
                        self.morph = now;
                    }
                    None => {
                        self.appear = now;
                        self.from = None;
                    }
                }
                self.ghost = None;
                self.target = Some(plane);
            }
            None => {
                self.ghost = shown.map(|current| (current.plane, current.progress));
                self.vanish = now;
                self.target = None;
                self.from = None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Marker {
    target: Option<[f64; 3]>,
    from: Option<[f64; 3]>,
    moved: f64,
    popped: f64,
    pulsed: f64,
    ghost: Option<[f64; 3]>,
    vanish: f64,
}

impl Default for Marker {
    fn default() -> Self {
        Marker {
            target: None,
            from: None,
            moved: f64::NEG_INFINITY,
            popped: f64::NEG_INFINITY,
            pulsed: f64::NEG_INFINITY,
            ghost: None,
            vanish: f64::NEG_INFINITY,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Spot {
    at: [f64; 3],
    size: f64,
    alpha: f64,
    reveal: f64,
}

impl Marker {
    fn shown(&self, now: f64) -> Option<Spot> {
        if let Some(target) = self.target {
            let at = match self.from {
                Some(from) if now < self.moved + MORPH => {
                    lerp3(from, target, ease_in_out((now - self.moved) / MORPH))
                }
                _ => target,
            };
            let grow = (now - self.popped) / POP;
            return Some(Spot {
                at,
                size: ease_out_back(grow),
                alpha: 1.0,
                reveal: clamp01(grow),
            });
        }
        let ghost = self.ghost?;
        let fade = (now - self.vanish) / VANISH;
        (fade < 1.0).then(|| Spot {
            at: ghost,
            size: 1.0,
            alpha: 1.0 - clamp01(fade),
            reveal: 1.0 - clamp01(fade),
        })
    }

    fn rings(&self, now: f64) -> Vec<(f64, f64)> {
        if self.target.is_none() {
            return Vec::new();
        }
        (0..PULSES)
            .filter_map(|index| {
                let start = self.pulsed + PULSE_DELAY + index as f64 * PULSE_GAP;
                let progress = (now - start) / PULSE;
                (0.0..1.0).contains(&progress).then(|| {
                    let rest = 1.0 - progress;
                    (ease_out(progress), 0.6 * rest * rest)
                })
            })
            .collect()
    }

    fn busy(&self, now: f64, contrast: bool) -> bool {
        match self.target {
            Some(_) => {
                now < self.popped + POP
                    || (self.from.is_some() && now < self.moved + MORPH)
                    || (!contrast
                        && now
                            < self.pulsed + PULSE_DELAY + (PULSES - 1) as f64 * PULSE_GAP + PULSE)
            }
            None => self.ghost.is_some() && now < self.vanish + VANISH,
        }
    }

    fn retarget(&mut self, target: Option<[f64; 3]>, now: f64, animate: bool) {
        if self.target == target {
            return;
        }
        let shown = self.shown(now);
        if !animate {
            *self = Marker {
                target,
                ..Marker::default()
            };
            return;
        }
        match target {
            Some(point) => {
                match shown {
                    Some(current) => {
                        if self.target.is_none() {
                            self.popped = f64::NEG_INFINITY;
                        }
                        self.from = Some(current.at);
                        self.moved = now;
                    }
                    None => {
                        self.from = None;
                        self.popped = now;
                    }
                }
                self.pulsed = now;
                self.ghost = None;
                self.target = Some(point);
            }
            None => {
                self.ghost = shown.map(|current| current.at);
                self.vanish = now;
                self.target = None;
                self.from = None;
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Motion {
    sketch: Sketch,
    tracks: [Track; 3],
    marker: Marker,
    view: Option<Bounds>,
    view_from: Option<Bounds>,
    view_start: f64,
    set_start: f64,
}

impl Default for Motion {
    fn default() -> Self {
        Motion::new()
    }
}

#[derive(Clone, Copy, Debug)]
struct Stage {
    width: f64,
    height: f64,
    unit: f64,
    contrast: bool,
}

impl Motion {
    pub fn new() -> Motion {
        Motion {
            sketch: Sketch::blank(2),
            tracks: [Track::default(); 3],
            marker: Marker::default(),
            view: None,
            view_from: None,
            view_start: f64::NEG_INFINITY,
            set_start: f64::NEG_INFINITY,
        }
    }

    pub fn sketch(&self) -> &Sketch {
        &self.sketch
    }

    pub fn update(&mut self, sketch: &Sketch, now: f64, animate: bool) {
        if *sketch == self.sketch && self.view.is_some() {
            return;
        }
        let animate = animate && self.view.is_some() && sketch.dimension == self.sketch.dimension;
        if !animate && sketch.dimension != self.sketch.dimension {
            self.tracks = [Track::default(); 3];
            self.marker = Marker::default();
        }
        let huge = sketch.huge();
        for (row, track) in self.tracks.iter_mut().enumerate() {
            let target = if huge { None } else { sketch.plane(row) };
            track.retarget(target, now, animate);
        }
        let point = if huge { None } else { sketch.point() };
        self.marker.retarget(point, now, animate);
        let target = if huge {
            bounds(&Sketch::blank(sketch.dimension))
        } else {
            bounds(sketch)
        };
        if self.view != Some(target) {
            match (animate, self.view) {
                (true, Some(_)) => {
                    self.view_from = Some(self.shown_view(now));
                    self.view_start = now;
                }
                _ => {
                    self.view_from = None;
                    self.view_start = f64::NEG_INFINITY;
                }
            }
            self.view = Some(target);
        }
        let set_changed = std::mem::discriminant(&sketch.verdict)
            != std::mem::discriminant(&self.sketch.verdict)
            || sketch.rows != self.sketch.rows;
        if set_changed {
            self.set_start = if animate { now } else { f64::NEG_INFINITY };
        }
        self.sketch = sketch.clone();
    }

    pub fn settle(&mut self) {
        for track in &mut self.tracks {
            *track = Track {
                target: track.target,
                ..Track::default()
            };
        }
        self.marker = Marker {
            target: self.marker.target,
            ..Marker::default()
        };
        self.view_from = None;
        self.view_start = f64::NEG_INFINITY;
        self.set_start = f64::NEG_INFINITY;
    }

    fn shown_view(&self, now: f64) -> Bounds {
        let target = self.view.unwrap_or_else(|| bounds(&self.sketch));
        match self.view_from {
            Some(from) if now < self.view_start + VIEW => {
                from.lerp(target, ease_in_out((now - self.view_start) / VIEW))
            }
            _ => target,
        }
    }

    fn settling(&self, contrast: bool) -> f64 {
        match self.sketch.verdict {
            Verdict::Line => MARCH,
            Verdict::Everything if !contrast => LIGHT_UP,
            _ => 0.0,
        }
    }

    pub fn busy(&self, now: f64, contrast: bool) -> bool {
        (self.view_from.is_some() && now < self.view_start + VIEW)
            || self.tracks.iter().any(|track| track.busy(now))
            || self.marker.busy(now, contrast)
            || now < self.set_start + self.settling(contrast)
    }

    pub fn scene(&self, now: f64, width: f64, height: f64, unit: f64, contrast: bool) -> Vec<Item> {
        if width.is_nan() || height.is_nan() || width <= 0.0 || height <= 0.0 {
            return Vec::new();
        }
        let stage = Stage {
            width,
            height,
            unit,
            contrast,
        };
        let mut items = Vec::new();
        let everything = self.sketch.verdict == Verdict::Everything && !self.sketch.huge();
        let fade = if everything {
            clamp01((now - self.set_start) / LIGHT_UP)
        } else {
            0.0
        };
        if !contrast {
            items.push(Item {
                shape: Shape::Dots {
                    spacing: 8.0 * unit,
                    radius: 0.9 * unit,
                },
                ink: Ink::Texture,
                alpha: 1.0,
            });
        }
        if everything {
            items.push(Item {
                shape: Shape::Dots {
                    spacing: 8.0 * unit,
                    radius: 0.9 * unit,
                },
                ink: Ink::Set,
                alpha: if contrast { 1.0 } else { 0.55 * fade },
            });
        }
        let view = self.shown_view(now);
        let shown: Vec<Option<Shown>> = (0..self.sketch.dimension)
            .map(|row| self.tracks[row].shown(now))
            .collect();
        let phase = MARCH_SPEED * unit * (now - self.set_start).min(MARCH);
        if self.sketch.dimension == 3 {
            self.scene_three(now, &stage, view, &shown, phase, &mut items);
        } else {
            self.scene_two(now, &stage, view, &shown, phase, &mut items);
        }
        items
    }

    fn spot_items(
        &self,
        now: f64,
        stage: &Stage,
        center: Pt,
        alpha: f64,
        size: f64,
        items: &mut Vec<Item>,
    ) {
        let unit = stage.unit;
        if !stage.contrast {
            items.push(Item {
                shape: Shape::Disc {
                    center,
                    radius: 11.0 * unit * size.max(0.0),
                },
                ink: Ink::Halo,
                alpha: 0.2 * alpha,
            });
            for (growth, fade) in self.marker.rings(now) {
                items.push(Item {
                    shape: Shape::Ring {
                        center,
                        radius: (4.0 + 13.0 * growth) * unit,
                        width: 1.5 * unit,
                    },
                    ink: Ink::Halo,
                    alpha: fade * alpha,
                });
            }
        }
        let radius = 4.25 * unit * size.max(0.0);
        if radius > 0.0 {
            items.push(Item {
                shape: Shape::Disc {
                    center,
                    radius: radius + 1.75 * unit,
                },
                ink: Ink::Backdrop,
                alpha: if stage.contrast { 1.0 } else { alpha },
            });
            items.push(Item {
                shape: Shape::Disc { center, radius },
                ink: Ink::Point,
                alpha: if stage.contrast { 1.0 } else { alpha },
            });
        }
    }

    fn scene_two(
        &self,
        now: f64,
        stage: &Stage,
        view: Bounds,
        shown: &[Option<Shown>],
        phase: f64,
        items: &mut Vec<Item>,
    ) {
        let Stage {
            width,
            height,
            unit,
            contrast,
        } = *stage;
        let aspect = width / height;
        let half_height = view.half[1].max(view.half[0] / aspect);
        let half_width = half_height * aspect;
        let left = view.center[0] - half_width;
        let right = view.center[0] + half_width;
        let bottom = view.center[1] - half_height;
        let top = view.center[1] + half_height;
        let screen_x = |x: f64| (x - left) / (right - left) * width;
        let screen_y = |y: f64| (top - y) / (top - bottom) * height;
        let screen = |point: [f64; 3]| Pt::new(screen_x(point[0]), screen_y(point[1]));
        let opaque = |alpha: f64| if contrast { 1.0 } else { alpha };
        let step = nice(2.0 * half_height / 5.0);
        let even = unit.floor() % 2.0 == 0.0;
        let snap = |value: f64, limit: f64| {
            let snapped = if even {
                (value + 0.5).floor()
            } else {
                value.floor() + 0.5
            };
            (snapped >= 0.0 && snapped <= limit).then_some(snapped)
        };
        let mut index = (left / step).ceil() as i64;
        while (index as f64) * step <= right {
            if let (true, Some(x)) = (index != 0, snap(screen_x(index as f64 * step), width)) {
                items.push(Item {
                    shape: Shape::Segment {
                        from: Pt::new(x, 0.0),
                        to: Pt::new(x, height),
                        width: unit,
                    },
                    ink: Ink::Grid,
                    alpha: 1.0,
                });
            }
            index += 1;
        }
        let mut index = (bottom / step).ceil() as i64;
        while (index as f64) * step <= top {
            if let (true, Some(y)) = (index != 0, snap(screen_y(index as f64 * step), height)) {
                items.push(Item {
                    shape: Shape::Segment {
                        from: Pt::new(0.0, y),
                        to: Pt::new(width, y),
                        width: unit,
                    },
                    ink: Ink::Grid,
                    alpha: 1.0,
                });
            }
            index += 1;
        }
        let axis_y = if bottom < 0.0 && top > 0.0 {
            snap(screen_y(0.0), height)
        } else {
            None
        };
        let axis_x = if left < 0.0 && right > 0.0 {
            snap(screen_x(0.0), width)
        } else {
            None
        };
        let tall = TEXT_HEIGHT * unit;
        let letter = CHAR_WIDTH * unit;
        let lines: Vec<[f64; 3]> = self.tracks[..self.sketch.dimension]
            .iter()
            .filter_map(|track| track.target)
            .map(|plane| {
                [
                    plane.normal[0] * (right - left) / width,
                    -plane.normal[1] * (top - bottom) / height,
                    plane.offset - plane.normal[0] * left - plane.normal[1] * top,
                ]
            })
            .collect();
        let clear = |area: Area| !lines.iter().any(|line| area.grow(unit).crossed(*line));
        let mut areas = Vec::new();
        if let Some(y) = axis_y {
            items.push(Item {
                shape: Shape::Segment {
                    from: Pt::new(0.0, y),
                    to: Pt::new(width, y),
                    width: unit,
                },
                ink: Ink::Axis,
                alpha: 1.0,
            });
            items.push(Item {
                shape: Shape::Polygon {
                    points: vec![
                        Pt::new(width - unit, y),
                        Pt::new(width - 7.0 * unit, y - 3.5 * unit),
                        Pt::new(width - 7.0 * unit, y + 3.5 * unit),
                    ],
                },
                ink: Ink::Axis,
                alpha: 1.0,
            });
            let over = Pt::new(width - 3.0 * unit, y - 5.0 * unit);
            let under = Pt::new(width - 3.0 * unit, y + 5.0 * unit);
            let (at, vertical) = if y + 5.0 * unit + tall <= height
                && !clear(Area::text(over, Anchor::End, Anchor::End, letter, tall))
                && clear(Area::text(under, Anchor::End, Anchor::Start, letter, tall))
            {
                (under, Anchor::Start)
            } else {
                (over, Anchor::End)
            };
            areas.push(Area::text(at, Anchor::End, vertical, letter, tall));
            areas.push(Area::span(
                Pt::new(width - 7.0 * unit, y - 3.5 * unit),
                Pt::new(width - unit, y + 3.5 * unit),
                0.0,
            ));
            areas.push(Area::span(Pt::new(0.0, y), Pt::new(width, y), 0.5 * unit));
            items.push(Item {
                shape: Shape::Text {
                    at,
                    text: "x".to_string(),
                    horizontal: Anchor::End,
                    vertical,
                },
                ink: Ink::Label,
                alpha: 1.0,
            });
        }
        if let Some(x) = axis_x {
            items.push(Item {
                shape: Shape::Segment {
                    from: Pt::new(x, 0.0),
                    to: Pt::new(x, height),
                    width: unit,
                },
                ink: Ink::Axis,
                alpha: 1.0,
            });
            items.push(Item {
                shape: Shape::Polygon {
                    points: vec![
                        Pt::new(x, unit),
                        Pt::new(x - 3.5 * unit, 7.0 * unit),
                        Pt::new(x + 3.5 * unit, 7.0 * unit),
                    ],
                },
                ink: Ink::Axis,
                alpha: 1.0,
            });
            let east = Pt::new(x + 6.0 * unit, 2.0 * unit);
            let west = Pt::new(x - 6.0 * unit, 2.0 * unit);
            let (at, horizontal) = if x - 6.0 * unit - letter >= 0.0
                && !clear(Area::text(east, Anchor::Start, Anchor::Start, letter, tall))
                && clear(Area::text(west, Anchor::End, Anchor::Start, letter, tall))
            {
                (west, Anchor::End)
            } else {
                (east, Anchor::Start)
            };
            areas.push(Area::text(at, horizontal, Anchor::Start, letter, tall));
            areas.push(Area::span(
                Pt::new(x - 3.5 * unit, unit),
                Pt::new(x + 3.5 * unit, 7.0 * unit),
                0.0,
            ));
            areas.push(Area::span(Pt::new(x, 0.0), Pt::new(x, height), 0.5 * unit));
            items.push(Item {
                shape: Shape::Text {
                    at,
                    text: "y".to_string(),
                    horizontal,
                    vertical: Anchor::Start,
                },
                ink: Ink::Label,
                alpha: 1.0,
            });
        }
        let low = [left, bottom, 0.0];
        let high = [right, top, 0.0];
        let segments: Vec<Option<(Pt, Pt, f64)>> = shown
            .iter()
            .map(|entry| {
                let entry = entry.as_ref()?;
                let plane = entry.plane;
                let direction = [-plane.normal[1], plane.normal[0], 0.0];
                let (start, end) = clip(plane.foot(), direction, low, high, 2)?;
                let (start, end) = (screen(start), screen(end));
                let middle = start.lerp(end, 0.5);
                Some((
                    middle.lerp(start, entry.progress),
                    middle.lerp(end, entry.progress),
                    entry.alpha,
                ))
            })
            .collect();
        if !contrast {
            for (row, segment) in segments.iter().enumerate() {
                if let Some((from, to, alpha)) = segment {
                    items.push(Item {
                        shape: Shape::Segment {
                            from: *from,
                            to: *to,
                            width: 7.0 * unit,
                        },
                        ink: Ink::Row(row),
                        alpha: 0.14 * alpha,
                    });
                }
            }
        }
        for (row, segment) in segments.iter().enumerate() {
            if let Some((from, to, alpha)) = segment {
                items.push(Item {
                    shape: Shape::Segment {
                        from: *from,
                        to: *to,
                        width: 2.0 * unit,
                    },
                    ink: Ink::Row(row),
                    alpha: opaque(*alpha),
                });
            }
        }
        if self.sketch.verdict == Verdict::Line {
            let planes: Vec<Option<Plane>> =
                shown.iter().map(|entry| entry.map(|e| e.plane)).collect();
            if let Some((point, direction)) = solution_line(2, &planes) {
                if let Some((start, end)) = clip(point, direction, low, high, 2) {
                    march(screen(start), screen(end), phase, unit, contrast, items);
                }
            }
        }
        if let Some(spot) = self.marker.shown(now) {
            let center = screen(spot.at);
            let coordinates = self.sketch.coordinates();
            let guide = spot.reveal * spot.alpha;
            if guide > 0.0 {
                let label = |index: usize| {
                    coordinates
                        .get(index)
                        .filter(|text| text.as_str() != "0")
                        .cloned()
                };
                let x_text = axis_y.and(label(0));
                let y_text = axis_x.and(label(1));
                let mut obstacles = Obstacles { lines, areas };
                obstacles.areas.push(Area::span(center, center, 6.0 * unit));
                if let (Some(axis), Some(_)) = (axis_y, &x_text) {
                    obstacles
                        .areas
                        .push(Area::span(center, Pt::new(center.x, axis), 0.5 * unit));
                }
                if let (Some(axis), Some(_)) = (axis_x, &y_text) {
                    obstacles
                        .areas
                        .push(Area::span(center, Pt::new(axis, center.y), 0.5 * unit));
                }
                if let (Some(axis), Some(text)) = (axis_y, x_text) {
                    dashes(
                        center,
                        Pt::new(center.x, axis),
                        unit,
                        opaque(0.85 * guide),
                        items,
                    );
                    if let Some((at, horizontal, vertical)) =
                        x_label(center, axis, &text, stage, &obstacles)
                    {
                        let wide = text.chars().count() as f64 * CHAR_WIDTH * unit;
                        obstacles
                            .areas
                            .push(Area::text(at, horizontal, vertical, wide, tall));
                        items.push(Item {
                            shape: Shape::Text {
                                at,
                                text,
                                horizontal,
                                vertical,
                            },
                            ink: Ink::Label,
                            alpha: opaque(guide),
                        });
                    }
                }
                if let (Some(axis), Some(text)) = (axis_x, y_text) {
                    dashes(
                        center,
                        Pt::new(axis, center.y),
                        unit,
                        opaque(0.85 * guide),
                        items,
                    );
                    if let Some((at, horizontal, vertical)) =
                        y_label(center, axis, &text, stage, &obstacles)
                    {
                        items.push(Item {
                            shape: Shape::Text {
                                at,
                                text,
                                horizontal,
                                vertical,
                            },
                            ink: Ink::Label,
                            alpha: opaque(guide),
                        });
                    }
                }
            }
            self.spot_items(now, stage, center, spot.alpha, spot.size, items);
        }
    }

    fn scene_three(
        &self,
        now: f64,
        stage: &Stage,
        view: Bounds,
        shown: &[Option<Shown>],
        phase: f64,
        items: &mut Vec<Item>,
    ) {
        let Stage {
            width,
            height,
            unit,
            contrast,
        } = *stage;
        let opaque = |alpha: f64| if contrast { 1.0 } else { alpha };
        let low = view.low();
        let high = view.high();
        let project = |point: [f64; 3]| {
            (
                SX_X * point[0] + SX_Z * point[2],
                SY_X * point[0] + SY_Y * point[1] + SY_Z * point[2],
            )
        };
        let corner = |index: usize| {
            [
                if index & 1 != 0 { high[0] } else { low[0] },
                if index & 2 != 0 { high[1] } else { low[1] },
                if index & 4 != 0 { high[2] } else { low[2] },
            ]
        };
        let mut min_u = f64::INFINITY;
        let mut max_u = f64::NEG_INFINITY;
        let mut min_v = f64::INFINITY;
        let mut max_v = f64::NEG_INFINITY;
        for index in 0..8 {
            let (u, v) = project(corner(index));
            min_u = min_u.min(u);
            max_u = max_u.max(u);
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
        let pad_side = 12.0 * unit;
        let pad_top = 15.0 * unit;
        let pad_bottom = 4.0 * unit;
        let room_w = (width - 2.0 * pad_side).max(1.0);
        let room_h = (height - pad_top - pad_bottom).max(1.0);
        let zoom = (room_w / (max_u - min_u)).min(room_h / (max_v - min_v));
        let origin_x = pad_side + (room_w - zoom * (max_u - min_u)) / 2.0 - zoom * min_u;
        let origin_y = pad_top + (room_h - zoom * (max_v - min_v)) / 2.0 - zoom * min_v;
        let screen = |point: [f64; 3]| {
            let (u, v) = project(point);
            Pt::new(origin_x + zoom * u, origin_y + zoom * v)
        };
        let depth = |point: [f64; 3]| DEPTH_X * point[0] + DEPTH_Y * point[1] + DEPTH_Z * point[2];
        let panes = [
            [corner(0), corner(1), corner(5), corner(4)],
            [corner(0), corner(2), corner(6), corner(4)],
            [corner(0), corner(1), corner(3), corner(2)],
        ];
        if !contrast {
            for pane in &panes {
                items.push(Item {
                    shape: Shape::Polygon {
                        points: pane.iter().map(|point| screen(*point)).collect(),
                    },
                    ink: Ink::Pane,
                    alpha: 1.0,
                });
            }
        }
        let step = nice(view.half[0] / 2.0);
        let grid = |axis: usize, span: usize, items: &mut Vec<Item>| {
            let mut index = (low[axis] / step).ceil() as i64;
            while (index as f64) * step <= high[axis] {
                let mut start = low;
                start[axis] = index as f64 * step;
                let mut end = start;
                end[span] = high[span];
                items.push(Item {
                    shape: Shape::Segment {
                        from: screen(start),
                        to: screen(end),
                        width: unit,
                    },
                    ink: if index == 0 { Ink::Axis } else { Ink::Grid },
                    alpha: 1.0,
                });
                index += 1;
            }
        };
        for (axis, span) in [(0, 2), (2, 0), (1, 2), (2, 1), (1, 0), (0, 1)] {
            grid(axis, span, items);
        }
        for pane in &panes {
            items.push(Item {
                shape: Shape::Outline {
                    points: pane.iter().map(|point| screen(*point)).collect(),
                    width: unit,
                },
                ink: Ink::Edge,
                alpha: 1.0,
            });
        }
        let x_end = screen(corner(1));
        let z_end = screen(corner(4));
        let y_end = screen(corner(2));
        for (at, text, horizontal, vertical) in [
            (
                Pt::new(x_end.x + 4.0 * unit, x_end.y),
                "x",
                Anchor::Start,
                Anchor::Middle,
            ),
            (
                Pt::new(z_end.x - 4.0 * unit, z_end.y),
                "z",
                Anchor::End,
                Anchor::Middle,
            ),
            (
                Pt::new(y_end.x, y_end.y - 2.0 * unit),
                "y",
                Anchor::Middle,
                Anchor::End,
            ),
        ] {
            items.push(Item {
                shape: Shape::Text {
                    at,
                    text: text.to_string(),
                    horizontal,
                    vertical,
                },
                ink: Ink::Label,
                alpha: 1.0,
            });
        }
        let mut sections: Vec<(f64, usize, Vec<Pt>, f64)> = Vec::new();
        for (row, entry) in shown.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            let polygon = section(entry.plane, low, high);
            if polygon.is_empty() {
                continue;
            }
            let count = polygon.len() as f64;
            let middle = scale(
                polygon.iter().fold([0.0; 3], |sum, point| add(sum, *point)),
                1.0 / count,
            );
            let flat: Vec<Pt> = polygon.iter().map(|point| screen(*point)).collect();
            let center = flat.iter().fold(Pt::default(), |sum, point| {
                Pt::new(sum.x + point.x, sum.y + point.y)
            });
            let center = Pt::new(center.x / count, center.y / count);
            let grown = flat
                .iter()
                .map(|point| center.lerp(*point, entry.progress))
                .collect();
            sections.push((depth(middle), row, grown, entry.alpha));
        }
        sections.sort_by(|first, second| first.0.total_cmp(&second.0).then(first.1.cmp(&second.1)));
        for (_, row, points, alpha) in &sections {
            if !contrast {
                items.push(Item {
                    shape: Shape::Polygon {
                        points: points.clone(),
                    },
                    ink: Ink::Row(*row),
                    alpha: 0.26 * alpha,
                });
            }
            items.push(Item {
                shape: Shape::Outline {
                    points: points.clone(),
                    width: 1.25 * unit,
                },
                ink: Ink::Row(*row),
                alpha: opaque(0.95 * alpha),
            });
        }
        if self.sketch.verdict == Verdict::Plane {
            if let Some((_, _, points, alpha)) = sections
                .iter()
                .filter(|section| self.sketch.plane(section.1).is_some())
                .min_by_key(|section| section.1)
            {
                items.push(Item {
                    shape: Shape::Outline {
                        points: points.clone(),
                        width: 2.0 * unit,
                    },
                    ink: Ink::Set,
                    alpha: opaque(*alpha),
                });
            }
        }
        for first in 0..shown.len() {
            for second in first + 1..shown.len() {
                let (Some(one), Some(two)) = (shown[first], shown[second]) else {
                    continue;
                };
                let Some((point, direction)) = line_of(one.plane, two.plane) else {
                    continue;
                };
                let Some((start, end)) = clip(point, direction, low, high, 3) else {
                    continue;
                };
                let strength = one.alpha.min(two.alpha) * one.progress.min(two.progress);
                items.push(Item {
                    shape: Shape::Segment {
                        from: screen(start),
                        to: screen(end),
                        width: unit,
                    },
                    ink: Ink::Meet,
                    alpha: opaque(0.5 * strength),
                });
            }
        }
        if self.sketch.verdict == Verdict::Line {
            let planes: Vec<Option<Plane>> =
                shown.iter().map(|entry| entry.map(|e| e.plane)).collect();
            if let Some((point, direction)) = solution_line(3, &planes) {
                if let Some((start, end)) = clip(point, direction, low, high, 3) {
                    march(screen(start), screen(end), phase, unit, contrast, items);
                }
            }
        }
        if let Some(spot) = self.marker.shown(now) {
            let center = screen(spot.at);
            let guide = spot.reveal * spot.alpha;
            if guide > 0.0 {
                let floor = screen([spot.at[0], low[1], spot.at[2]]);
                dashes(center, floor, unit, opaque(0.85 * guide), items);
                items.push(Item {
                    shape: Shape::Ring {
                        center: floor,
                        radius: 2.5 * unit,
                        width: unit,
                    },
                    ink: Ink::Guide,
                    alpha: opaque(0.85 * guide),
                });
            }
            self.spot_items(now, stage, center, spot.alpha, spot.size, items);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Area {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Area {
    fn text(at: Pt, horizontal: Anchor, vertical: Anchor, wide: f64, tall: f64) -> Area {
        let left = match horizontal {
            Anchor::Start => at.x,
            Anchor::Middle => at.x - wide / 2.0,
            Anchor::End => at.x - wide,
        };
        let top = match vertical {
            Anchor::Start => at.y,
            Anchor::Middle => at.y - tall / 2.0,
            Anchor::End => at.y - tall,
        };
        Area {
            left,
            top,
            right: left + wide,
            bottom: top + tall,
        }
    }

    fn span(from: Pt, to: Pt, pad: f64) -> Area {
        Area {
            left: from.x.min(to.x) - pad,
            top: from.y.min(to.y) - pad,
            right: from.x.max(to.x) + pad,
            bottom: from.y.max(to.y) + pad,
        }
    }

    fn grow(self, pad: f64) -> Area {
        Area {
            left: self.left - pad,
            top: self.top - pad,
            right: self.right + pad,
            bottom: self.bottom + pad,
        }
    }

    fn meets(self, other: Area) -> bool {
        self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom
    }

    fn crossed(self, line: [f64; 3]) -> bool {
        let side = |x: f64, y: f64| line[0] * x + line[1] * y - line[2];
        let corners = [
            side(self.left, self.top),
            side(self.right, self.top),
            side(self.left, self.bottom),
            side(self.right, self.bottom),
        ];
        let above = corners.iter().all(|value| *value > 0.0);
        let below = corners.iter().all(|value| *value < 0.0);
        !above && !below
    }
}

struct Obstacles {
    lines: Vec<[f64; 3]>,
    areas: Vec<Area>,
}

impl Obstacles {
    fn blocks(&self, area: Area) -> bool {
        self.lines.iter().any(|line| area.crossed(*line))
            || self.areas.iter().any(|other| area.meets(*other))
    }
}

type Placement = (Pt, Anchor, Anchor);

fn pick(
    candidates: &[Placement],
    wide: f64,
    tall: f64,
    unit: f64,
    obstacles: &Obstacles,
) -> Option<Placement> {
    candidates
        .iter()
        .find(|(at, horizontal, vertical)| {
            !obstacles.blocks(Area::text(*at, *horizontal, *vertical, wide, tall).grow(unit))
        })
        .or(candidates.first())
        .copied()
}

fn x_label(
    center: Pt,
    axis: f64,
    text: &str,
    stage: &Stage,
    obstacles: &Obstacles,
) -> Option<Placement> {
    let count = text.chars().count();
    if count > LABEL_LIMIT {
        return None;
    }
    let unit = stage.unit;
    let tall = TEXT_HEIGHT * unit;
    let wide = count as f64 * CHAR_WIDTH * unit;
    let gap = 4.0 * unit;
    let below = (axis + gap + tall <= stage.height, axis + gap, Anchor::Start);
    let above = (axis - gap - tall >= 0.0, axis - gap, Anchor::End);
    let (far, near) = if center.y < axis {
        (below, above)
    } else {
        (above, below)
    };
    let middle = if stage.width > wide + 4.0 * unit {
        center.x.clamp(
            wide / 2.0 + 2.0 * unit,
            stage.width - wide / 2.0 - 2.0 * unit,
        )
    } else {
        stage.width / 2.0
    };
    let east = center.x + 5.0 * unit + wide <= stage.width - 2.0 * unit;
    let west = center.x - 5.0 * unit - wide >= 2.0 * unit;
    let mut candidates = Vec::new();
    if far.0 {
        candidates.push((Pt::new(middle, far.1), Anchor::Middle, far.2));
    }
    for (fits, y, vertical) in [far, near] {
        if !fits {
            continue;
        }
        if east {
            candidates.push((Pt::new(center.x + 5.0 * unit, y), Anchor::Start, vertical));
        }
        if west || !east {
            candidates.push((Pt::new(center.x - 5.0 * unit, y), Anchor::End, vertical));
        }
    }
    pick(&candidates, wide, tall, unit, obstacles)
}

fn y_label(
    center: Pt,
    axis: f64,
    text: &str,
    stage: &Stage,
    obstacles: &Obstacles,
) -> Option<Placement> {
    let count = text.chars().count();
    if count > LABEL_LIMIT {
        return None;
    }
    let unit = stage.unit;
    let tall = TEXT_HEIGHT * unit;
    let wide = count as f64 * CHAR_WIDTH * unit;
    let gap = 5.0 * unit;
    let west = (axis - gap - wide >= 0.0, axis - gap, Anchor::End);
    let east = (axis + gap + wide <= stage.width, axis + gap, Anchor::Start);
    let (far, near) = if center.x > axis {
        (west, east)
    } else {
        (east, west)
    };
    let middle = if stage.height > tall {
        center.y.clamp(tall / 2.0, stage.height - tall / 2.0)
    } else {
        stage.height / 2.0
    };
    let up = center.y - 3.0 * unit - tall >= 0.0;
    let down = center.y + 3.0 * unit + tall <= stage.height;
    let mut candidates = Vec::new();
    if far.0 {
        candidates.push((Pt::new(far.1, middle), far.2, Anchor::Middle));
    }
    for (fits, x, horizontal) in [far, near] {
        if !fits {
            continue;
        }
        if up {
            candidates.push((Pt::new(x, center.y - 3.0 * unit), horizontal, Anchor::End));
        }
        if down || !up {
            candidates.push((Pt::new(x, center.y + 3.0 * unit), horizontal, Anchor::Start));
        }
    }
    pick(&candidates, wide, tall, unit, obstacles)
}

fn dashes(from: Pt, to: Pt, unit: f64, alpha: f64, items: &mut Vec<Item>) {
    let length = from.distance(to);
    if length.is_nan() || length <= 0.5 {
        return;
    }
    let on = 3.0 * unit;
    let period = 6.0 * unit;
    let mut start = 0.0;
    while start < length {
        let end = (start + on).min(length);
        items.push(Item {
            shape: Shape::Segment {
                from: from.lerp(to, start / length),
                to: from.lerp(to, end / length),
                width: unit,
            },
            ink: Ink::Guide,
            alpha,
        });
        start += period;
    }
}

fn march(from: Pt, to: Pt, phase: f64, unit: f64, contrast: bool, items: &mut Vec<Item>) {
    let length = from.distance(to);
    if length.is_nan() || length <= 0.5 {
        return;
    }
    let spacing = 10.0 * unit;
    let mut offset = phase % spacing;
    if offset < 0.0 {
        offset += spacing;
    }
    while offset <= length {
        let center = from.lerp(to, offset / length);
        if contrast {
            items.push(Item {
                shape: Shape::Disc {
                    center,
                    radius: 3.2 * unit,
                },
                ink: Ink::Backdrop,
                alpha: 1.0,
            });
        }
        items.push(Item {
            shape: Shape::Disc {
                center,
                radius: 1.7 * unit,
            },
            ink: Ink::Set,
            alpha: if contrast { 1.0 } else { 0.95 },
        });
        offset += spacing;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane2(a: f64, b: f64, d: f64) -> Row {
        Row::Drawn(Plane::new([a, b, 0.0], d).unwrap())
    }

    fn plane3(a: f64, b: f64, c: f64, d: f64) -> Row {
        Row::Drawn(Plane::new([a, b, c], d).unwrap())
    }

    fn exemplo2() -> Sketch {
        Sketch {
            dimension: 2,
            rows: vec![plane2(2.0, 1.0, 5.0), plane2(1.0, -1.0, 1.0)],
            verdict: Verdict::Point {
                at: Some([2.0, 1.0, 0.0]),
                coordinates: vec!["2".to_string(), "1".to_string()],
            },
        }
    }

    fn exemplo3() -> Sketch {
        Sketch {
            dimension: 3,
            rows: vec![
                plane3(1.0, 1.0, 1.0, 6.0),
                plane3(2.0, -1.0, 1.0, 3.0),
                plane3(1.0, 2.0, -1.0, 2.0),
            ],
            verdict: Verdict::Point {
                at: Some([1.0, 2.0, 3.0]),
                coordinates: vec!["1".to_string(), "2".to_string(), "3".to_string()],
            },
        }
    }

    fn rest(sketch: &Sketch, width: f64, height: f64) -> Vec<Item> {
        let mut motion = Motion::new();
        motion.update(sketch, 0.0, false);
        motion.scene(1e9, width, height, 1.0, false)
    }

    fn inside(items: &[Item], width: f64, height: f64, slack: f64) {
        let ok = |point: &Pt| {
            point.x >= -slack
                && point.y >= -slack
                && point.x <= width + slack
                && point.y <= height + slack
        };
        for item in items {
            match &item.shape {
                Shape::Segment { from, to, .. } => {
                    assert!(ok(from) && ok(to), "{item:?}");
                }
                Shape::Polygon { points } | Shape::Outline { points, .. } => {
                    assert!(points.iter().all(ok), "{item:?}");
                }
                Shape::Disc { center, .. } | Shape::Ring { center, .. } => {
                    assert!(ok(center), "{item:?}");
                }
                _ => {}
            }
        }
    }

    #[test]
    fn passo_da_grade_e_redondo() {
        assert_eq!(nice(1.0), 1.0);
        assert_eq!(nice(1.25), 2.0);
        assert_eq!(nice(3.0), 5.0);
        assert_eq!(nice(7.0), 10.0);
        assert_eq!(nice(0.03), 0.05);
        assert_eq!(nice(1500.0), 2000.0);
    }

    #[test]
    fn plano_normaliza_e_rejeita_o_que_nao_desenha() {
        let plane = Plane::new([3.0, 4.0, 0.0], 10.0).unwrap();
        assert_eq!(plane.normal, [0.6, 0.8, 0.0]);
        assert_eq!(plane.offset, 2.0);
        assert_eq!(plane.foot(), [1.2, 1.6, 0.0]);
        assert!(Plane::new([0.0, 0.0, 0.0], 1.0).is_none());
        assert!(Plane::new([1.0, 0.0, 0.0], f64::INFINITY).is_none());
        assert!(Plane::new([1.0, 0.0, 0.0], 1e101).is_none());
    }

    #[test]
    fn legendas_de_cada_caso() {
        assert_eq!(caption(&Sketch::blank(2)), INTRO_TWO);
        assert_eq!(caption(&Sketch::blank(3)), INTRO_THREE);
        let mut sketch = Sketch::blank(3);
        sketch.rows[0] = plane3(1.0, 0.0, 0.0, 1.0);
        assert_eq!(
            caption(&sketch),
            "Preencha as equações 2 e 3 para ver onde os planos se cruzam."
        );
        sketch.rows[1] = Row::Trivial;
        sketch.rows[2] = Row::Partial;
        assert_eq!(
            caption(&sketch),
            "Preencha a equação 3 para ver onde os planos se cruzam."
        );
        let mut duas = Sketch::blank(2);
        duas.rows[1] = plane2(1.0, 1.0, 1.0);
        assert_eq!(
            caption(&duas),
            "Preencha a equação 1 para ver onde as retas se cruzam."
        );
        assert_eq!(
            caption(&exemplo2()),
            "As retas se cruzam em (2, 1): esse ponto é a solução do sistema."
        );
        assert_eq!(
            caption(&exemplo3()),
            "Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema."
        );
        let longe = Sketch {
            verdict: Verdict::Point {
                at: None,
                coordinates: vec!["1e200".to_string(), "2".to_string()],
            },
            ..exemplo2()
        };
        assert_eq!(
            caption(&longe),
            "A solução é (1e200, 2), longe demais para caber no gráfico."
        );
        let esperando = Sketch {
            verdict: Verdict::Waiting,
            ..exemplo2()
        };
        assert_eq!(caption(&esperando), WAITING);
        let coincidentes = Sketch {
            dimension: 2,
            rows: vec![plane2(1.0, 1.0, 2.0), plane2(2.0, 2.0, 4.0)],
            verdict: Verdict::Line,
        };
        assert_eq!(caption(&coincidentes), COINCIDENT_LINES);
        let trivial = Sketch {
            dimension: 2,
            rows: vec![Row::Trivial, plane2(1.0, 1.0, 2.0)],
            verdict: Verdict::Line,
        };
        assert_eq!(
            caption(&trivial),
            "A equação 1 vale para qualquer ponto (0 = 0), então toda a reta da equação 2 é solução."
        );
        let paralelas = Sketch {
            dimension: 2,
            rows: vec![plane2(1.0, 1.0, 2.0), plane2(1.0, 1.0, 3.0)],
            verdict: Verdict::Nothing { rank: 1 },
        };
        assert_eq!(caption(&paralelas), PARALLEL_LINES);
        let impossivel = Sketch {
            dimension: 2,
            rows: vec![plane2(1.0, 1.0, 2.0), Row::Impossible("5".to_string())],
            verdict: Verdict::Nothing { rank: 1 },
        };
        assert_eq!(
            caption(&impossivel),
            "A equação 2 diz que 0 = 5, o que nunca acontece: o sistema não tem solução."
        );
        let tudo = Sketch {
            dimension: 3,
            rows: vec![Row::Trivial; 3],
            verdict: Verdict::Everything,
        };
        assert_eq!(caption(&tudo), EVERYTHING_THREE);
        let plano = Sketch {
            dimension: 3,
            rows: vec![Row::Trivial, plane3(1.0, 1.0, 1.0, 1.0), Row::Trivial],
            verdict: Verdict::Plane,
        };
        assert_eq!(
            caption(&plano),
            "As outras equações valem para qualquer ponto (0 = 0), então todo o plano da equação 2 é solução."
        );
        let enorme = Sketch {
            rows: vec![Row::Huge, plane2(1.0, 1.0, 1.0)],
            ..exemplo2()
        };
        assert_eq!(caption(&enorme), HUGE);
        let prisma = Sketch {
            dimension: 3,
            rows: vec![
                plane3(1.0, 0.0, 0.0, 0.0),
                plane3(0.0, 1.0, 0.0, 0.0),
                plane3(1.0, 1.0, 0.0, 1.0),
            ],
            verdict: Verdict::Nothing { rank: 2 },
        };
        assert_eq!(caption(&prisma), NO_COMMON_POINT);
    }

    #[test]
    fn janela_contem_origem_ponto_e_pes() {
        let view = bounds(&exemplo2());
        assert!((view.center[0] - 1.0).abs() < 1e-12);
        assert!((view.center[1] - 0.25).abs() < 1e-12);
        assert_eq!(view.center[2], 0.0);
        assert_eq!(view.half, [MIN_HALF, MIN_HALF, 0.0]);
        let view = bounds(&exemplo3());
        let low = view.low();
        let high = view.high();
        for point in [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0], [2.0, 2.0, 2.0]] {
            for axis in 0..3 {
                assert!(low[axis] < point[axis] && point[axis] < high[axis]);
            }
        }
        assert_eq!(view.half[1], view.half[0] * BOX_HEIGHT);
        let longe = Sketch {
            verdict: Verdict::Point {
                at: Some([1000.0, -400.0, 0.0]),
                coordinates: vec!["1000".to_string(), "-400".to_string()],
            },
            ..exemplo2()
        };
        let view = bounds(&longe);
        assert!(view.low()[0] < 0.0 && view.high()[0] > 1000.0);
        assert!(view.low()[1] < -400.0 && view.high()[1] > 0.0);
    }

    #[test]
    fn cena_2d_em_repouso_fica_dentro_da_area() {
        let items = rest(&exemplo2(), 252.0, 120.0);
        inside(&items, 252.0, 120.0, 0.01);
        let lines: Vec<&Item> = items
            .iter()
            .filter(|item| matches!(item.ink, Ink::Row(_)) && item.alpha == 1.0)
            .collect();
        assert_eq!(lines.len(), 2);
        let points: Vec<&Item> = items.iter().filter(|item| item.ink == Ink::Point).collect();
        assert_eq!(points.len(), 1);
        let Shape::Disc { center, .. } = points[0].shape else {
            panic!();
        };
        let aspect = 252.0 / 120.0;
        let half_w = MIN_HALF * aspect;
        let expected_x = (2.0 - (1.0 - half_w)) / (2.0 * half_w) * 252.0;
        let expected_y = (0.25 + MIN_HALF - 1.0) / (2.0 * MIN_HALF) * 120.0;
        assert!((center.x - expected_x).abs() < 1e-9);
        assert!((center.y - expected_y).abs() < 1e-9);
        let texts: Vec<String> = items
            .iter()
            .filter_map(|item| match &item.shape {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, ["x", "y", "2", "1"]);
        for (row, segment) in lines.iter().enumerate() {
            let Shape::Segment { from, to, .. } = segment.shape else {
                panic!();
            };
            let plane = exemplo2().plane(row).unwrap();
            for point in [from, to] {
                let world_x = (1.0 - half_w) + point.x / 252.0 * 2.0 * half_w;
                let world_y = (0.25 + MIN_HALF) - point.y / 120.0 * 2.0 * MIN_HALF;
                let residue = plane.normal[0] * world_x + plane.normal[1] * world_y - plane.offset;
                assert!(residue.abs() < 1e-9, "{residue}");
            }
        }
    }

    #[test]
    fn cena_3d_em_repouso_tem_planos_ordenados_e_ponto() {
        let items = rest(&exemplo3(), 147.0, 70.0);
        inside(&items, 147.0, 70.0, 0.01);
        let fills = items
            .iter()
            .filter(|item| {
                matches!(item.ink, Ink::Row(_)) && matches!(item.shape, Shape::Polygon { .. })
            })
            .count();
        assert_eq!(fills, 3);
        let meets = items.iter().filter(|item| item.ink == Ink::Meet).count();
        assert_eq!(meets, 3);
        assert_eq!(
            items.iter().filter(|item| item.ink == Ink::Point).count(),
            1
        );
        let texts: Vec<String> = items
            .iter()
            .filter_map(|item| match &item.shape {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, ["x", "z", "y"]);
    }

    #[test]
    fn secao_do_plano_no_cubo_e_convexa() {
        let low = [-1.0, -1.0, -1.0];
        let high = [1.0, 1.0, 1.0];
        let hexagono = section(Plane::new([1.0, 1.0, 1.0], 0.0).unwrap(), low, high);
        assert_eq!(hexagono.len(), 6);
        let quadrado = section(Plane::new([0.0, 0.0, 1.0], 0.0).unwrap(), low, high);
        assert_eq!(quadrado.len(), 4);
        let triangulo = section(Plane::new([1.0, 1.0, 1.0], 2.0).unwrap(), low, high);
        assert_eq!(triangulo.len(), 3);
        assert!(section(Plane::new([1.0, 0.0, 0.0], 5.0).unwrap(), low, high).is_empty());
        let face = section(Plane::new([1.0, 0.0, 0.0], 1.0).unwrap(), low, high);
        assert_eq!(face.len(), 4);
        for polygon in [hexagono, quadrado, triangulo, face] {
            let normal = cross(sub(polygon[1], polygon[0]), sub(polygon[2], polygon[1]));
            for index in 0..polygon.len() {
                let a = polygon[index];
                let b = polygon[(index + 1) % polygon.len()];
                let c = polygon[(index + 2) % polygon.len()];
                let turn = cross(sub(b, a), sub(c, b));
                assert!(dot(turn, normal) > -1e-9);
            }
        }
    }

    #[test]
    fn animacao_comeca_vazia_e_termina_em_repouso() {
        let mut motion = Motion::new();
        motion.update(&Sketch::blank(2), 0.0, true);
        assert!(!motion.busy(0.0, false));
        motion.update(&exemplo2(), 1.0, true);
        assert!(motion.busy(1.0, false));
        let early = motion.scene(1.05, 252.0, 120.0, 1.0, false);
        let lines = |items: &[Item]| -> Vec<f64> {
            items
                .iter()
                .filter(|item| matches!(item.ink, Ink::Row(_)) && item.alpha > 0.5)
                .map(|item| match item.shape {
                    Shape::Segment { from, to, .. } => from.distance(to),
                    _ => 0.0,
                })
                .collect()
        };
        let partial = lines(&early);
        let end = 1.0 + APPEAR.max(POP).max(PULSE_DELAY + 2.0 * PULSE_GAP + PULSE) + 0.01;
        assert!(!motion.busy(end, false));
        let done = motion.scene(end, 252.0, 120.0, 1.0, false);
        let full = lines(&done);
        assert_eq!(partial.len(), 2);
        for (start, finish) in partial.iter().zip(&full) {
            assert!(start < finish);
        }
        assert_eq!(done, rest(&exemplo2(), 252.0, 120.0));
        let rings = early
            .iter()
            .filter(|item| matches!(item.shape, Shape::Ring { .. }))
            .count();
        assert_eq!(rings, 0);
        let pulsing = motion.scene(1.0 + PULSE_DELAY + 0.3, 252.0, 120.0, 1.0, false);
        assert!(
            pulsing
                .iter()
                .any(|item| matches!(item.shape, Shape::Ring { .. }))
        );
    }

    #[test]
    fn linha_que_some_desbota_e_volta_sem_redesenhar() {
        let mut motion = Motion::new();
        motion.update(&exemplo2(), 0.0, false);
        let mut editando = exemplo2();
        editando.rows[1] = Row::Partial;
        editando.verdict = Verdict::Pending;
        motion.update(&editando, 10.0, true);
        let fading = motion.scene(10.1, 252.0, 120.0, 1.0, false);
        let ghost = fading
            .iter()
            .filter(|item| {
                item.ink == Ink::Row(1)
                    && matches!(item.shape, Shape::Segment { width, .. } if width == 2.0)
            })
            .map(|item| item.alpha)
            .next()
            .unwrap();
        assert!(ghost > 0.0 && ghost < 1.0);
        assert!(motion.busy(10.1, false));
        assert!(!motion.busy(10.0 + VANISH + PULSE * 4.0, false));
        let gone = motion.scene(11.0, 252.0, 120.0, 1.0, false);
        assert!(!gone.iter().any(|item| item.ink == Ink::Row(1)));
        let mut movido = exemplo2();
        movido.rows[0] = plane2(2.0, 1.0, 6.0);
        movido.verdict = Verdict::Point {
            at: Some([7.0 / 3.0, 4.0 / 3.0, 0.0]),
            coordinates: vec!["7/3".to_string(), "4/3".to_string()],
        };
        let mut motion = Motion::new();
        motion.update(&exemplo2(), 0.0, false);
        motion.update(&movido, 5.0, true);
        let morphing = motion.scene(5.1, 252.0, 120.0, 1.0, false);
        assert!(
            morphing
                .iter()
                .any(|item| item.ink == Ink::Row(0) && item.alpha == 1.0)
        );
        assert!(motion.busy(5.1, false));
    }

    #[test]
    fn sem_animacao_e_alto_contraste() {
        let mut motion = Motion::new();
        motion.update(&exemplo3(), 0.0, true);
        assert!(!motion.busy(0.0, false));
        let items = motion.scene(0.0, 147.0, 70.0, 1.0, true);
        assert!(items.iter().all(|item| item.alpha == 1.0));
        assert!(
            !items
                .iter()
                .any(|item| matches!(item.shape, Shape::Dots { .. }))
        );
        assert!(!items.iter().any(|item| item.ink == Ink::Halo));
        assert!(!items.iter().any(|item| item.ink == Ink::Pane));
        let fills = items
            .iter()
            .filter(|item| {
                matches!(item.ink, Ink::Row(_)) && matches!(item.shape, Shape::Polygon { .. })
            })
            .count();
        assert_eq!(fills, 0);
    }

    #[test]
    fn conjunto_solucao_marcha_e_para() {
        let sketch = Sketch {
            dimension: 2,
            rows: vec![plane2(1.0, 1.0, 2.0), plane2(2.0, 2.0, 4.0)],
            verdict: Verdict::Line,
        };
        let mut motion = Motion::new();
        motion.update(&Sketch::blank(2), 0.0, false);
        motion.update(&sketch, 1.0, true);
        let dots = |items: &[Item]| -> Vec<Pt> {
            items
                .iter()
                .filter(|item| item.ink == Ink::Set)
                .filter_map(|item| match item.shape {
                    Shape::Disc { center, .. } => Some(center),
                    _ => None,
                })
                .collect()
        };
        let first = dots(&motion.scene(1.2, 252.0, 120.0, 1.0, false));
        let second = dots(&motion.scene(1.4, 252.0, 120.0, 1.0, false));
        assert!(!first.is_empty());
        assert_ne!(first, second);
        assert!(motion.busy(1.0 + MARCH - 0.01, false));
        assert!(!motion.busy(1.0 + MARCH + 1.0, false));
        assert_eq!(
            motion.scene(1.0 + MARCH + 1.0, 252.0, 120.0, 1.0, false),
            rest(&sketch, 252.0, 120.0)
        );
        let tudo = Sketch {
            dimension: 2,
            rows: vec![Row::Trivial, Row::Trivial],
            verdict: Verdict::Everything,
        };
        let items = rest(&tudo, 252.0, 120.0);
        assert!(
            items
                .iter()
                .any(|item| item.ink == Ink::Set && matches!(item.shape, Shape::Dots { .. }))
        );
    }

    #[test]
    fn valores_enormes_nao_desenham() {
        let sketch = Sketch {
            dimension: 2,
            rows: vec![Row::Huge, plane2(1.0, 1.0, 1.0)],
            verdict: Verdict::Nothing { rank: 1 },
        };
        let items = rest(&sketch, 252.0, 120.0);
        assert!(
            !items
                .iter()
                .any(|item| matches!(item.ink, Ink::Row(_) | Ink::Point))
        );
    }

    #[test]
    fn ponto_desliza_do_fantasma_quando_a_equacao_volta() {
        let mut marker = Marker::default();
        marker.retarget(Some([2.0, 1.0, 0.0]), 0.0, false);
        marker.retarget(None, 10.0, true);
        marker.retarget(Some([1.0, 3.0, 0.0]), 10.1, true);
        let start = marker.shown(10.1).unwrap();
        assert_eq!(start.at, [2.0, 1.0, 0.0]);
        assert_eq!(start.size, 1.0);
        let middle = marker.shown(10.1 + MORPH / 2.0).unwrap();
        assert!(middle.at[0] > 1.0 && middle.at[0] < 2.0);
        assert!(middle.at[1] > 1.0 && middle.at[1] < 3.0);
        assert_eq!(marker.shown(10.1 + MORPH).unwrap().at, [1.0, 3.0, 0.0]);
        let mut sumido = Marker::default();
        sumido.retarget(Some([2.0, 1.0, 0.0]), 0.0, false);
        sumido.retarget(None, 10.0, true);
        sumido.retarget(Some([1.0, 3.0, 0.0]), 11.0, true);
        let novo = sumido.shown(11.0).unwrap();
        assert_eq!(novo.at, [1.0, 3.0, 0.0]);
        assert!(novo.size.abs() < 1e-9);
    }

    #[test]
    fn alto_contraste_nao_espera_o_que_nao_aparece() {
        let mut motion = Motion::new();
        motion.update(&Sketch::blank(2), 0.0, false);
        motion.update(&exemplo2(), 1.0, true);
        let calmo = 1.0 + APPEAR.max(POP).max(VIEW) + 0.01;
        assert!(motion.busy(calmo, false));
        assert!(!motion.busy(calmo, true));
        let tudo = Sketch {
            dimension: 2,
            rows: vec![Row::Trivial, Row::Trivial],
            verdict: Verdict::Everything,
        };
        let mut motion = Motion::new();
        motion.update(&Sketch::blank(2), 0.0, false);
        motion.update(&tudo, 1.0, true);
        assert!(motion.busy(1.0 + LIGHT_UP - 0.01, false));
        assert!(!motion.busy(1.0 + LIGHT_UP + 0.01, false));
        assert!(!motion.busy(1.0, true));
        let plano = Sketch {
            dimension: 3,
            rows: vec![Row::Trivial, plane3(1.0, 1.0, 1.0, 1.0), Row::Trivial],
            verdict: Verdict::Plane,
        };
        let mut motion = Motion::new();
        motion.update(&plano, 0.0, false);
        let mut outro = plano.clone();
        outro.rows[0] = Row::Partial;
        motion.update(&outro, 1.0, true);
        assert!(!motion.busy(1.0, false));
    }

    #[test]
    fn rotulos_fogem_das_retas_e_das_letras() {
        let sketch = Sketch {
            dimension: 2,
            rows: vec![plane2(3.0, 0.0, 1.0), plane2(0.0, 1.0, 2.0)],
            verdict: Verdict::Point {
                at: Some([1.0 / 3.0, 2.0, 0.0]),
                coordinates: vec!["1/3".to_string(), "2".to_string()],
            },
        };
        for (width, height) in [(252.0, 120.0), (267.0, 128.0), (233.0, 111.0)] {
            let items = rest(&sketch, width, height);
            let lines: Vec<[f64; 3]> = items
                .iter()
                .filter(|item| matches!(item.ink, Ink::Row(_)) && item.alpha == 1.0)
                .filter_map(|item| match item.shape {
                    Shape::Segment { from, to, .. } => {
                        let a = to.y - from.y;
                        let b = from.x - to.x;
                        Some([a, b, a * from.x + b * from.y])
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(lines.len(), 2);
            let labels: Vec<Area> = items
                .iter()
                .filter_map(|item| match &item.shape {
                    Shape::Text {
                        at,
                        text,
                        horizontal,
                        vertical,
                    } => Some(Area::text(
                        *at,
                        *horizontal,
                        *vertical,
                        text.chars().count() as f64 * CHAR_WIDTH,
                        TEXT_HEIGHT,
                    )),
                    _ => None,
                })
                .collect();
            assert_eq!(labels.len(), 4);
            for area in &labels {
                for line in &lines {
                    assert!(!area.crossed(*line), "{width}x{height} {area:?}");
                }
            }
            for (index, first) in labels.iter().enumerate() {
                for second in &labels[index + 1..] {
                    assert!(!first.meets(*second), "{first:?} {second:?}");
                }
            }
        }
        let padrao = describe(&rest(&exemplo2(), 252.0, 120.0));
        let mut motion = Motion::new();
        motion.update(&exemplo2(), 0.0, false);
        assert_eq!(
            describe(&motion.scene(1e9, 252.0, 120.0, 1.0, false)),
            padrao
        );
    }

    #[test]
    fn grade_nitida_em_qualquer_escala() {
        for (unit, fraction) in [(1.0, 0.5), (1.5, 0.5), (2.0, 0.0), (3.0, 0.5)] {
            let mut motion = Motion::new();
            motion.update(&exemplo2(), 0.0, false);
            let items = motion.scene(1e9, 252.0 * unit, 120.0 * unit, unit, false);
            let positions: Vec<f64> = items
                .iter()
                .filter(|item| matches!(item.ink, Ink::Grid | Ink::Axis))
                .filter_map(|item| match item.shape {
                    Shape::Segment { from, to, .. } if from.x == to.x => Some(from.x),
                    Shape::Segment { from, to, .. } if from.y == to.y => Some(from.y),
                    _ => None,
                })
                .collect();
            assert!(positions.len() > 4);
            for value in positions {
                assert_eq!(value - value.floor(), fraction, "{unit} {value}");
            }
        }
    }

    #[test]
    fn pontinhos_do_conjunto_tem_fundo_no_alto_contraste() {
        let sketch = Sketch {
            dimension: 2,
            rows: vec![plane2(1.0, 1.0, 2.0), plane2(2.0, 2.0, 4.0)],
            verdict: Verdict::Line,
        };
        let mut motion = Motion::new();
        motion.update(&sketch, 0.0, false);
        let items = motion.scene(1e9, 252.0, 120.0, 1.0, true);
        let dots: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.ink == Ink::Set)
            .map(|(index, _)| index)
            .collect();
        assert!(dots.len() > 3);
        for index in dots {
            assert_eq!(items[index - 1].ink, Ink::Backdrop);
            let (
                Shape::Disc { center, radius },
                Shape::Disc {
                    center: under,
                    radius: wider,
                },
            ) = (&items[index].shape, &items[index - 1].shape)
            else {
                panic!();
            };
            assert_eq!(center, under);
            assert!(wider > radius);
        }
        let normal = motion.scene(1e9, 252.0, 120.0, 1.0, false);
        assert!(!normal.iter().any(|item| item.ink == Ink::Backdrop));
    }

    #[test]
    fn descricao_textual_e_estavel() {
        let text = describe(&rest(&exemplo2(), 252.0, 120.0));
        assert!(text.starts_with("t1000:D800,90;g1000:S"));
        assert!(text.contains("o1000:C"));
        assert_eq!(text, describe(&rest(&exemplo2(), 252.0, 120.0)));
    }
}
