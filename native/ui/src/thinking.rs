use sistemas_lineares_core::{LineKind, MatrixView, Walkthrough};

use crate::layout::{Rect, d};
use crate::model;
use crate::text::{TextMetrics, margins_for, wrap_lines};
use crate::theme::Font;

pub const STEP_POOL: usize = 24;

pub trait Measure {
    fn width(&self, text: &str, font: Font) -> i32;
    fn metrics(&self, font: Font) -> TextMetrics;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ink {
    Ink,
    Muted,
    Operation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    Text {
        text: String,
        font: Font,
        ink: Ink,
        rect: Rect,
        centered: bool,
    },
    Badge {
        rect: Rect,
        text: String,
    },
    Marked {
        rect: Rect,
    },
    Pivot {
        rect: Rect,
    },
    Stroke {
        rect: Rect,
        muted: bool,
    },
    Rule {
        rect: Rect,
    },
}

impl Item {
    pub fn rect(&self) -> Rect {
        match self {
            Item::Text { rect, .. }
            | Item::Badge { rect, .. }
            | Item::Marked { rect }
            | Item::Pivot { rect }
            | Item::Stroke { rect, .. }
            | Item::Rule { rect } => *rect,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StepBox {
    pub rect: Rect,
    pub items: Vec<Item>,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Page {
    pub summary: Vec<Item>,
    pub steps: Vec<StepBox>,
    pub bottom: i32,
}

struct Writer<'a> {
    measure: &'a dyn Measure,
    items: Vec<Item>,
}

impl Writer<'_> {
    fn margins(&self, font: Font) -> (i32, i32) {
        margins_for(self.measure.metrics(font))
    }

    fn lines(&self, text: &str, font: Font, width: i32) -> Vec<String> {
        let (left, right) = self.margins(font);
        wrap_lines(text, width - left - right, &|line: &str| {
            self.measure.width(line, font)
        })
    }

    fn paragraph(
        &mut self,
        text: &str,
        (font, ink, spacing): (Font, Ink, i32),
        x: i32,
        y: i32,
        width: i32,
    ) -> i32 {
        let height = self.measure.metrics(font).height.max(1);
        let mut cursor = y;
        for line in self.lines(text, font, width) {
            if !line.is_empty() {
                self.items.push(Item::Text {
                    text: line,
                    font,
                    ink,
                    rect: Rect::new(x, cursor, width, height),
                    centered: false,
                });
            }
            cursor += height + spacing;
        }
        cursor - spacing
    }
}

fn summary(
    writer: &mut Writer,
    walkthrough: &Walkthrough,
    scale: f32,
    left: i32,
    top: i32,
    width: i32,
) -> i32 {
    let gap = d(scale, 32);
    let column = ((width - gap) / 2).max(1);
    let right = left + column + gap;
    let label_height = writer.measure.metrics(Font::Small).height;
    let mut system_y = top;
    writer.items.push(Item::Text {
        text: model::SYSTEM_LABEL.to_string(),
        font: Font::Small,
        ink: Ink::Muted,
        rect: Rect::new(left, system_y, column, label_height),
        centered: false,
    });
    system_y += label_height + d(scale, 8);
    for equation in &walkthrough.system {
        system_y = writer.paragraph(
            equation,
            (Font::Input, Ink::Ink, d(scale, 4)),
            left,
            system_y,
            column,
        ) + d(scale, 4);
    }
    let mut answer_y = top;
    writer.items.push(Item::Text {
        text: model::ANSWER_LABEL.to_string(),
        font: Font::Small,
        ink: Ink::Muted,
        rect: Rect::new(right, answer_y, column, label_height),
        centered: false,
    });
    answer_y += label_height + d(scale, 8);
    for answer in &walkthrough.answer {
        answer_y = writer.paragraph(
            answer,
            (Font::Title, Ink::Ink, d(scale, 4)),
            right,
            answer_y,
            column,
        ) + d(scale, 4);
    }
    answer_y += d(scale, 4);
    answer_y = writer.paragraph(
        walkthrough.classification,
        (Font::Body, Ink::Muted, d(scale, 2)),
        right,
        answer_y,
        column,
    );
    system_y.max(answer_y)
}

struct MatrixBlock {
    items: Vec<Item>,
    width: i32,
    height: i32,
}

fn matrix_block(
    measure: &dyn Measure,
    view: &MatrixView,
    scale: f32,
    x: i32,
    y: i32,
) -> MatrixBlock {
    let cells = view.cells();
    let rows = cells.len();
    let columns = cells.first().map_or(0, Vec::len);
    let metric = measure.metrics(Font::Input);
    let (margin_left, margin_right) = margins_for(metric);
    let padding = d(scale, 11);
    let widths: Vec<i32> = (0..columns)
        .map(|column| {
            let widest = cells
                .iter()
                .map(|row| measure.width(&row[column], Font::Input))
                .max()
                .unwrap_or(0);
            (widest + margin_left + margin_right + 2 * padding).max(d(scale, 40))
        })
        .collect();
    let row_height = metric.height + d(scale, 10);
    let thickness = d(scale, 2).max(1);
    let tick = d(scale, 7);
    let inner = d(scale, 4);
    let vertical = d(scale, 6);
    let divider = if view.augmented { d(scale, 14) } else { 0 };
    let mut items = Vec::new();
    let mut cursor = x;
    let label_text = view.label.as_ref().map(|label| format!("{label} ="));
    let body_height = rows as i32 * row_height + 2 * vertical;
    if let Some(text) = &label_text {
        let (left, right) = margins_for(measure.metrics(Font::Title));
        let width = measure.width(text, Font::Title) + left + right;
        let height = measure.metrics(Font::Title).height;
        items.push(Item::Text {
            text: text.clone(),
            font: Font::Title,
            ink: Ink::Ink,
            rect: Rect::new(cursor, y + (body_height - height) / 2, width, height),
            centered: false,
        });
        cursor += width + d(scale, 10);
    }
    let left_bracket = cursor;
    let cells_left = left_bracket + thickness + inner;
    let mut column_x = Vec::with_capacity(columns);
    let mut position = cells_left;
    for (column, width) in widths.iter().enumerate() {
        if view.augmented && column + 1 == columns {
            position += divider;
        }
        column_x.push(position);
        position += width;
    }
    let cells_right = position;
    let right_bracket = cells_right + inner;
    let top = y;
    let first_row = top + vertical;
    for &row in &view.marked_rows {
        if row < rows {
            items.push(Item::Marked {
                rect: Rect::new(
                    cells_left,
                    first_row + row as i32 * row_height + d(scale, 1),
                    cells_right - cells_left,
                    row_height - d(scale, 2),
                ),
            });
        }
    }
    if let Some(column) = view.marked_column.filter(|column| *column < columns) {
        items.push(Item::Marked {
            rect: Rect::new(
                column_x[column] + d(scale, 2),
                first_row,
                widths[column] - d(scale, 4),
                rows as i32 * row_height,
            ),
        });
    }
    if let Some((row, column)) = view
        .pivot
        .filter(|(row, column)| *row < rows && *column < columns)
    {
        items.push(Item::Pivot {
            rect: Rect::new(
                column_x[column] + d(scale, 4),
                first_row + row as i32 * row_height + d(scale, 3),
                widths[column] - d(scale, 8),
                row_height - d(scale, 6),
            ),
        });
    }
    for (row, values) in cells.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            items.push(Item::Text {
                text: value.clone(),
                font: Font::Input,
                ink: Ink::Ink,
                rect: Rect::new(
                    column_x[column],
                    first_row + row as i32 * row_height,
                    widths[column],
                    row_height,
                ),
                centered: true,
            });
        }
    }
    for (bracket_x, tick_x) in [
        (left_bracket, left_bracket),
        (right_bracket, right_bracket + thickness - tick),
    ] {
        items.push(Item::Stroke {
            rect: Rect::new(bracket_x, top, thickness, body_height),
            muted: false,
        });
        items.push(Item::Stroke {
            rect: Rect::new(tick_x, top, tick, thickness),
            muted: false,
        });
        items.push(Item::Stroke {
            rect: Rect::new(tick_x, top + body_height - thickness, tick, thickness),
            muted: false,
        });
    }
    if view.augmented && columns > 1 {
        let line_x = column_x[columns - 1] - divider / 2;
        items.push(Item::Stroke {
            rect: Rect::new(
                line_x,
                first_row + d(scale, 2),
                d(scale, 1).max(1),
                rows as i32 * row_height - d(scale, 4),
            ),
            muted: true,
        });
    }
    MatrixBlock {
        items,
        width: right_bracket + thickness - x,
        height: body_height,
    }
}

fn matrix_rows_text(view: &MatrixView) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(label) = &view.label {
        lines.push(format!("{label} ="));
    }
    let columns = view.rows.first().map_or(0, Vec::len);
    for row in view.cells() {
        if view.augmented && columns > 1 {
            lines.push(format!(
                "({} | {})",
                row[..columns - 1].join(", "),
                row[columns - 1]
            ));
        } else {
            lines.push(format!("({})", row.join(", ")));
        }
    }
    lines
}

fn place_matrix(
    writer: &mut Writer,
    view: &MatrixView,
    scale: f32,
    x: i32,
    y: i32,
    width: i32,
) -> i32 {
    let block = matrix_block(writer.measure, view, scale, x, y);
    if block.width <= width {
        writer.items.extend(block.items);
        return y + block.height;
    }
    let mut cursor = y;
    for line in matrix_rows_text(view) {
        cursor = writer.paragraph(
            &line,
            (Font::Input, Ink::Ink, d(scale, 4)),
            x,
            cursor,
            width,
        ) + d(scale, 4);
    }
    cursor - d(scale, 4)
}

fn line_style(kind: LineKind) -> (Font, Ink) {
    match kind {
        LineKind::Operation => (Font::Input, Ink::Operation),
        LineKind::Calculation => (Font::Input, Ink::Ink),
        LineKind::Result => (Font::Title, Ink::Ink),
    }
}

pub fn step_text(index: usize, step: &sistemas_lineares_core::Step) -> String {
    let mut text = format!("Passo {} · {}\n{}", index + 1, step.title, step.reasoning);
    let matrix: Vec<String> = step
        .matrix
        .as_ref()
        .map(MatrixView::text_lines)
        .unwrap_or_default();
    if step.matrix_first {
        for line in &matrix {
            text.push('\n');
            text.push_str(line);
        }
    }
    for line in &step.lines {
        text.push('\n');
        text.push_str(&line.text);
    }
    if !step.matrix_first {
        for line in &matrix {
            text.push('\n');
            text.push_str(line);
        }
    }
    text
}

fn step_box(
    measure: &dyn Measure,
    index: usize,
    step: &sistemas_lineares_core::Step,
    scale: f32,
    width: i32,
) -> (Vec<Item>, i32) {
    let mut writer = Writer {
        measure,
        items: Vec::new(),
    };
    let padding = d(scale, 22);
    if index > 0 {
        writer.items.push(Item::Rule {
            rect: Rect::new(0, 0, width, 1),
        });
    }
    let badge = d(scale, 30);
    writer.items.push(Item::Badge {
        rect: Rect::new(0, padding, badge, badge),
        text: (index + 1).to_string(),
    });
    let content_x = d(scale, 48);
    let content_w = (width - content_x).max(1);
    let title_height = measure.metrics(Font::Title).height;
    let title_y = padding + (badge - title_height) / 2;
    let mut cursor = writer.paragraph(
        &step.title,
        (Font::Title, Ink::Ink, d(scale, 2)),
        content_x,
        title_y,
        content_w,
    );
    cursor = cursor.max(padding + badge - d(scale, 8));
    cursor = writer.paragraph(
        &step.reasoning,
        (Font::Body, Ink::Muted, d(scale, 3)),
        content_x,
        cursor + d(scale, 6),
        content_w,
    );
    let gap = d(scale, 14);
    if step.matrix_first {
        if let Some(view) = &step.matrix {
            cursor = place_matrix(&mut writer, view, scale, content_x, cursor + gap, content_w);
        }
    }
    if !step.lines.is_empty() {
        cursor += gap - d(scale, 4);
        for line in &step.lines {
            let (font, ink) = line_style(line.kind);
            cursor = writer.paragraph(
                &line.text,
                (font, ink, d(scale, 2)),
                content_x,
                cursor + d(scale, 4),
                content_w,
            );
        }
    }
    if !step.matrix_first {
        if let Some(view) = &step.matrix {
            cursor = place_matrix(&mut writer, view, scale, content_x, cursor + gap, content_w);
        }
    }
    (writer.items, cursor + padding)
}

pub fn page(walkthrough: &Walkthrough, measure: &dyn Measure, scale: f32, content: Rect) -> Page {
    let mut writer = Writer {
        measure,
        items: Vec::new(),
    };
    let summary_bottom = summary(
        &mut writer,
        walkthrough,
        scale,
        content.x,
        content.y,
        content.w,
    );
    let rule_y = summary_bottom + d(scale, 24);
    writer.items.push(Item::Rule {
        rect: Rect::new(content.x, rule_y, content.w, 1),
    });
    let mut cursor = rule_y + 1;
    let mut steps = Vec::with_capacity(walkthrough.steps.len());
    for (index, step) in walkthrough.steps.iter().enumerate().take(STEP_POOL) {
        let (items, height) = step_box(measure, index, step, scale, content.w);
        steps.push(StepBox {
            rect: Rect::new(content.x, cursor, content.w, height),
            items,
            text: step_text(index, step),
        });
        cursor += height;
    }
    Page {
        summary: writer.items,
        steps,
        bottom: cursor,
    }
}
