#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    pub const fn right(self) -> i32 {
        self.x + self.w
    }

    pub const fn bottom(self) -> i32 {
        self.y + self.h
    }

    pub const fn offset(self, dx: i32, dy: i32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    pub const fn is_empty(self) -> bool {
        self.w <= 0 || self.h <= 0
    }

    pub const fn intersects(self, other: Rect) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub const fn contains(self, other: Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }

    pub const fn contains_point(self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }
}

pub const MIN_WIDTH: i32 = 1040;
pub const MIN_HEIGHT: i32 = 760;
pub const BASE_CLIENT_WIDTH: i32 = 1160;
pub const BASE_CLIENT_HEIGHT: i32 = 810;
pub const MIN_WINDOW_WIDTH: i32 = 640;
pub const MIN_WINDOW_HEIGHT: i32 = 480;

pub fn dpi_scale(dpi: u32) -> f32 {
    dpi as f32 / 96.0
}

pub fn d(scale: f32, value: i32) -> i32 {
    f64::from(value as f32 * scale).round_ties_even() as i32
}

pub fn content_size(
    scale: f32,
    client_w: i32,
    client_h: i32,
    bar_w: i32,
    bar_h: i32,
) -> (i32, i32) {
    let width = ((client_w - bar_w) as f32 / scale) as i32;
    let height = ((client_h - bar_h) as f32 / scale) as i32;
    (width.max(MIN_WIDTH), height.max(MIN_HEIGHT))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Viewport {
    pub width: i32,
    pub height: i32,
    pub content_w: i32,
    pub content_h: i32,
    pub client_w: i32,
    pub client_h: i32,
    pub horizontal: bool,
    pub vertical: bool,
}

impl Viewport {
    pub fn max_scroll(self) -> (i32, i32) {
        (
            (self.content_w - self.client_w).max(0),
            (self.content_h - self.client_h).max(0),
        )
    }

    pub fn canvas_size(self) -> (i32, i32) {
        (
            self.content_w.max(self.client_w),
            self.content_h.max(self.client_h),
        )
    }
}

pub fn viewport(scale: f32, full_w: i32, full_h: i32, bar_w: i32, bar_h: i32) -> Viewport {
    let mut horizontal = false;
    let mut vertical = false;
    let mut result = Viewport::default();
    for _ in 0..4 {
        let client_w = full_w - if vertical { bar_w } else { 0 };
        let client_h = full_h - if horizontal { bar_h } else { 0 };
        let (width, height) = content_size(scale, client_w, client_h, bar_w, bar_h);
        let content_w = d(scale, width);
        let content_h = d(scale, height);
        let need_horizontal = content_w > client_w;
        let need_vertical = content_h > client_h;
        result = Viewport {
            width,
            height,
            content_w,
            content_h,
            client_w: client_w.max(0),
            client_h: client_h.max(0),
            horizontal,
            vertical,
        };
        if need_horizontal == horizontal && need_vertical == vertical {
            break;
        }
        horizontal = need_horizontal;
        vertical = need_vertical;
    }
    result
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Metrics {
    pub input_height: i32,
    pub value_height: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    pub scale: f32,
    pub dimension: usize,
    pub width: i32,
    pub height: i32,
    pub content_w: i32,
    pub content_h: i32,
    pub separator: Rect,
    pub logo: Rect,
    pub brand: Rect,
    pub author: Rect,
    pub intro: Rect,
    pub subtitle: Rect,
    pub entry: Rect,
    pub solution: Rect,
    pub entry_icon: Rect,
    pub entry_title: Rect,
    pub entry_description: Rect,
    pub two: Rect,
    pub three: Rect,
    pub dimension_text: Rect,
    pub grid_label: Rect,
    pub row_numbers: [Rect; 3],
    pub fields: [[Rect; 4]; 3],
    pub editors: [[Rect; 4]; 3],
    pub operators: [[Rect; 3]; 3],
    pub helper: Rect,
    pub solve: Rect,
    pub example: Rect,
    pub clear: Rect,
    pub solution_label: Rect,
    pub solution_title: Rect,
    pub solution_description: Rect,
    pub empty_icon: Rect,
    pub empty_title: Rect,
    pub empty_description: Rect,
    pub tiles: [Rect; 3],
    pub tile_names: [Rect; 3],
    pub tile_values: [Rect; 3],
    pub general: Rect,
    pub note: Rect,
    pub copy: Rect,
    pub footer: Rect,
    pub shortcut: Rect,
}

pub const fn field_visible(dimension: usize, row: usize, column: usize) -> bool {
    row < dimension && (column < dimension || column == 3)
}

pub const fn operator_visible(dimension: usize, row: usize, column: usize) -> bool {
    row < dimension && column < dimension
}

pub const fn operator_text(dimension: usize, column: usize) -> &'static str {
    if column + 1 == dimension { "=" } else { "+" }
}

pub const fn suffix(column: usize) -> Option<&'static str> {
    match column {
        0 => Some("x"),
        1 => Some("y"),
        2 => Some("z"),
        _ => None,
    }
}

pub fn field_box(field: Rect) -> (f32, f32, f32, f32) {
    (
        (field.x + 1) as f32,
        (field.y + 1) as f32,
        (field.w - 3) as f32,
        (field.h - 3) as f32,
    )
}

pub fn suffix_rect(field: Rect, scale: f32) -> Rect {
    Rect::new(field.right() - d(scale, 28), field.y, d(scale, 22), field.h)
}

pub fn field_editor(field: Rect, has_suffix: bool, scale: f32, line_height: i32) -> Rect {
    let x = field.x + d(scale, 8);
    let y = field.y + d(scale, 5).max((field.h - line_height) / 2);
    let w = d(scale, 15).max(field.w - d(scale, if has_suffix { 36 } else { 16 }));
    Rect::new(x, y, w, line_height)
}

pub fn compute(scale: f32, width: i32, height: i32, dimension: usize, metrics: Metrics) -> Layout {
    let dimension = dimension.clamp(2, 3);
    let place = |origin: Rect, x: i32, y: i32, w: i32, h: i32| {
        Rect::new(
            origin.x + d(scale, x),
            origin.y + d(scale, y),
            d(scale, w),
            d(scale, h),
        )
    };
    let root = Rect::default();
    let content_w = d(scale, width);
    let content_h = d(scale, height);
    let left = ((width - 84) as f32 * 0.59) as i32;
    let right = width - 84 - left;
    let card_height = height - 300;
    let entry = place(root, 32, 238, left, card_height);
    let solution = place(root, 52 + left, 238, right, card_height);
    let separator_left = d(scale, 32);
    let mut layout = Layout {
        scale,
        dimension,
        width,
        height,
        content_w,
        content_h,
        separator: Rect::new(
            separator_left,
            d(scale, 86),
            content_w - d(scale, 32) - separator_left + 1,
            1,
        ),
        logo: place(root, 32, 29, 35, 35),
        brand: place(root, 79, 36, 250, 28),
        author: place(root, width - 344, 28, 312, 42),
        intro: place(root, 30, 104, 370, 115),
        subtitle: place(root, 418, 156, width - 450, 62),
        entry,
        solution,
        entry_icon: place(entry, 26, 26, 26, 26),
        entry_title: place(entry, 65, 20, left - 90, 33),
        entry_description: place(entry, 26, 59, left - 52, 25),
        two: place(entry, 26, 103, 88, 40),
        three: place(entry, 120, 103, 88, 40),
        dimension_text: place(entry, 228, 113, left - 250, 24),
        grid_label: place(entry, 26, 168, left - 52, 22),
        helper: place(entry, 26, card_height - 113, left - 52, 42),
        solve: place(entry, 26, card_height - 64, 192, 42),
        example: place(entry, 228, card_height - 64, 155, 42),
        clear: place(entry, left - 115, card_height - 64, 89, 42),
        solution_label: place(solution, 26, 26, right - 52, 24),
        solution_title: place(solution, 26, 65, right - 52, 35),
        solution_description: place(solution, 26, 106, right - 52, 52),
        empty_icon: place(solution, (right - 76) / 2, 125, 76, 76),
        empty_title: place(solution, 20, 227, right - 40, 40),
        empty_description: place(solution, 25, 277, right - 50, 64),
        general: place(solution, 26, 170, right - 52, card_height - 286),
        note: place(solution, 26, card_height - 110, right - 52, 38),
        copy: place(solution, 26, card_height - 64, right - 52, 42),
        footer: place(root, 33, height - 43, 470, 25),
        shortcut: place(root, width - 485, height - 43, 450, 25),
        ..Layout::default()
    };
    let columns = dimension as i32;
    let field_width = (left - 26 - 26 - 29 - columns * 26) / (columns + 1);
    for row in 0..3 {
        let y = 194 + row as i32 * 50;
        layout.row_numbers[row] = place(entry, 26, y + 14, 25, 25);
        let mut x = 55;
        for column in 0..dimension {
            layout.fields[row][column] = place(entry, x, y, field_width, 46);
            x += field_width;
            layout.operators[row][column] = place(entry, x + 2, y + 9, 22, 28);
            x += 26;
        }
        layout.fields[row][3] = place(entry, x, y, field_width, 46);
        for column in 0..4 {
            let field = layout.fields[row][column];
            if !field.is_empty() {
                layout.editors[row][column] =
                    field_editor(field, suffix(column).is_some(), scale, metrics.input_height);
            }
        }
    }
    for index in 0..3 {
        let tile = place(solution, 26, 168 + index as i32 * 62, right - 52, 58);
        layout.tiles[index] = tile;
        layout.tile_names[index] = place(tile, 18, 9, 35, 40);
        layout.tile_values[index] = Rect::new(
            tile.x + d(scale, 68),
            tile.y + d(scale, 7),
            d(scale, right - 140),
            metrics.value_height,
        );
    }
    layout.note.y = layout.note.y.max(layout.tiles[2].bottom());
    layout
}
