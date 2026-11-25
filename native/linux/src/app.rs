use std::sync::Arc;

use eframe::egui::text::{CCursor, CCursorRange};
use eframe::egui::text_edit::TextEditState;
use eframe::egui::{
    self, CursorIcon, Event, EventFilter, Id, Key, Margin, Modifiers, Response, Role, ScrollArea,
    Sense, TextEdit, TextStyle, Ui, UiBuilder, Vec2, WidgetInfo, accesskit, vec2,
};
use sistemas_lineares_core::{Method, Rational, VARIABLE_NAMES, Walkthrough};
use sistemas_lineares_ui::graph::{self, Motion, Sketch};
use sistemas_lineares_ui::icons::Icon;
use sistemas_lineares_ui::layout::{self, Layout, Metrics, Rect};
use sistemas_lineares_ui::model::{self, Card, Focus, MethodRequest, Model, Notice, Outcome};
use sistemas_lineares_ui::text::{ellipsize, keep_groups, margins_for, preferred_height};
use sistemas_lineares_ui::theme::{self, Color, Font, Palette};
use sistemas_lineares_ui::thinking::{self, Ink, Item, Page};

use crate::paint::{Align, Canvas, Frame, VAlign, color, padded, ring_width, with_alpha};
use crate::{contrast, fonts, install};

pub const IDLE_DELAY: f64 = 0.7;
pub const KEY_METHOD: &str = "metodo";
pub const KEY_DIMENSION: &str = "dimensao";
pub const INPUT_LIMIT: usize = 32_767;

const ICON_SIZE: i32 = 22;
const ICON_GAP: i32 = 8;
const KEYCAP_RADIUS: f32 = 5.0;

struct TileColors {
    fill: Color,
    stroke: Color,
    text: Color,
}

fn tile_colors(palette: &Palette, stale: bool) -> TileColors {
    if !stale {
        return TileColors {
            fill: palette.tile_fill,
            stroke: palette.tile_stroke,
            text: palette.solution_text,
        };
    }
    if palette.high_contrast {
        return TileColors {
            fill: palette.tile_fill,
            stroke: palette.disabled_text,
            text: palette.disabled_text,
        };
    }
    let fill = palette
        .tile_fill
        .mix(palette.solution_fill, theme::STALE_PERCENT);
    TileColors {
        fill,
        stroke: palette
            .tile_stroke
            .mix(palette.solution_fill, theme::STALE_PERCENT),
        text: palette.solution_text.mix(fill, theme::STALE_PERCENT),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Two,
    Three,
    Field(usize, usize),
    Example,
    Clear,
    Think,
    Copy,
    Link(Method),
    Back,
    Tab(Method),
    CopySteps,
    Author,
}

impl Target {
    pub fn id(self) -> Id {
        Id::unique(("sistemas-lineares", self))
    }

    pub fn all() -> Vec<Target> {
        let mut all = vec![Target::Two, Target::Three];
        for row in 0..3 {
            for column in 0..4 {
                all.push(Target::Field(row, column));
            }
        }
        all.extend([Target::Example, Target::Clear, Target::Think, Target::Copy]);
        all.extend(Method::ALL.map(Target::Link));
        all.push(Target::Back);
        all.extend(Method::ALL.map(Target::Tab));
        all.extend([Target::CopySteps, Target::Author]);
        all
    }

    pub fn from_id(id: Id) -> Option<Target> {
        Target::all().into_iter().find(|target| target.id() == id)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Launch {
    pub high_contrast: bool,
    pub animations: bool,
    pub open_url: fn(&str) -> bool,
}

impl Default for Launch {
    fn default() -> Self {
        Launch {
            high_contrast: false,
            animations: true,
            open_url: open_in_browser,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Seen {
    dimension: usize,
    inputs: [[String; 4]; 3],
    outcome: Outcome,
    stale: bool,
    missing: usize,
    invalid: usize,
}

impl Seen {
    fn of(model: &Model) -> Seen {
        Seen {
            dimension: model.dimension(),
            inputs: std::array::from_fn(|row| {
                std::array::from_fn(|column| model.input(row, column).to_string())
            }),
            outcome: model.outcome().clone(),
            stale: model.is_stale(),
            missing: model.missing(),
            invalid: model.invalid_count(),
        }
    }

    fn matches(&self, model: &Model) -> bool {
        self.dimension == model.dimension()
            && self.stale == model.is_stale()
            && self.missing == model.missing()
            && self.invalid == model.invalid_count()
            && self.outcome == *model.outcome()
            && self.inputs.iter().enumerate().all(|(row, values)| {
                values
                    .iter()
                    .enumerate()
                    .all(|(column, value)| value == model.input(row, column))
            })
    }
}

pub fn open_in_browser(url: &str) -> bool {
    webbrowser::open(url).is_ok()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Press(Target),
    Edit(usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Navigation {
    Enter,
    Up,
    Down,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    Plain,
    Segment,
    Author,
}

struct Look<'a> {
    target: Target,
    rect: Rect,
    text: &'a str,
    name: String,
    icon: Option<Icon>,
    style: Style,
    selected: bool,
    enabled: bool,
    tip: Option<&'static str>,
    description: Option<&'static str>,
    role: Role,
}

struct PageCache {
    walkthrough: Walkthrough,
    content: Rect,
    page: Page,
}

pub struct App {
    model: Model,
    palette: Palette,
    buffers: [[String; 4]; 3],
    focused: Option<Target>,
    opener: Target,
    keyboard: bool,
    deferred: Vec<Event>,
    raw_pending: Vec<Event>,
    idle_due: Option<f64>,
    spoken: Option<String>,
    announcement: Option<(u64, String)>,
    announcements: Vec<String>,
    helper: (u64, String, bool),
    helper_spoken: Vec<String>,
    status: (u64, String),
    page: Option<PageCache>,
    link_hover: Option<Method>,
    link_focus: Option<Method>,
    scroll_reset: bool,
    scroll_to: Option<Target>,
    started: bool,
    content: (i32, i32),
    open_url: fn(&str) -> bool,
    motion: Motion,
    sketch: Sketch,
    caption: String,
    seen: Option<Seen>,
    animate: bool,
}

pub fn metrics() -> Metrics {
    Metrics {
        input_height: preferred_height(Font::Input.pixels(fonts::DPI))
            .unwrap_or(fonts::metrics(Font::Input).height),
        value_height: preferred_height(Font::Value.pixels(fonts::DPI))
            .unwrap_or(fonts::metrics(Font::Value).height),
    }
}

pub fn native_options() -> eframe::NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(model::WINDOW_TITLE)
        .with_app_id(install::APP_ID)
        .with_inner_size([
            layout::BASE_CLIENT_WIDTH as f32,
            layout::BASE_CLIENT_HEIGHT as f32,
        ])
        .with_min_inner_size([
            layout::MIN_WINDOW_WIDTH as f32,
            layout::MIN_WINDOW_HEIGHT as f32,
        ]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(install::WINDOW_ICON) {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    eframe::NativeOptions {
        viewport,
        persist_window: true,
        persistence_path: install::preferences_path(),
        centered: true,
        ..Default::default()
    }
}

pub fn run(launch: Launch) -> Result<(), String> {
    eframe::run_native(
        install::APP_ID,
        native_options(),
        Box::new(move |context| Ok(Box::new(App::new(context, launch)))),
    )
    .map_err(|error| error.to_string())
}

pub fn configure(ctx: &egui::Context, palette: &Palette) {
    ctx.set_fonts(fonts::definitions());
    ctx.tessellation_options_mut(|options| options.round_line_segments_to_pixels = false);
    let palette = *palette;
    ctx.all_styles_mut(move |style| {
        style
            .text_styles
            .insert(TextStyle::Small, fonts::id(Font::Small));
        style
            .text_styles
            .insert(TextStyle::Body, fonts::id(Font::Body));
        style
            .text_styles
            .insert(TextStyle::Button, fonts::id(Font::Medium));
        style
            .text_styles
            .insert(TextStyle::Heading, fonts::id(Font::Heading));
        style.spacing.tooltip_width = 360.0;
        let visuals = &mut style.visuals;
        visuals.dark_mode = true;
        visuals.panel_fill = color(palette.canvas);
        visuals.window_fill = color(palette.surface);
        visuals.window_stroke = egui::Stroke::new(1.0, color(palette.border));
        visuals.extreme_bg_color = color(palette.field);
        visuals.faint_bg_color = color(palette.surface);
        visuals.hyperlink_color = color(palette.link);
        visuals.override_text_color = None;
        visuals.selection.bg_fill = color(with_alpha(palette.accent, 110));
        visuals.selection.stroke = egui::Stroke::new(1.0, color(palette.ink));
        visuals.text_cursor.stroke = egui::Stroke::new(1.5, color(palette.ink));
        visuals.window_corner_radius = egui::CornerRadius::same(10);
        visuals.menu_corner_radius = egui::CornerRadius::same(8);
        visuals.popup_shadow = egui::epaint::Shadow::NONE;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, color(palette.ink));
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, color(palette.border));
        visuals.widgets.inactive.bg_fill = color(palette.border);
        visuals.widgets.inactive.weak_bg_fill = color(palette.border);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, color(palette.muted));
        visuals.widgets.hovered.bg_fill = color(palette.field_hover);
        visuals.widgets.hovered.weak_bg_fill = color(palette.field_hover);
        visuals.widgets.active.bg_fill = color(palette.muted);
        visuals.widgets.active.weak_bg_fill = color(palette.muted);
    });
}

fn take_navigation(
    events: &mut Vec<Event>,
    field: bool,
    thinking: bool,
) -> (Option<Navigation>, Vec<Event>) {
    let found = events.iter().position(|event| match event {
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } => match key {
            Key::Enter => field && !thinking && !modifiers.alt,
            Key::ArrowUp | Key::ArrowDown => field && !thinking && modifiers.is_none(),
            Key::Escape => thinking && modifiers.is_none(),
            _ => false,
        },
        _ => false,
    });
    let Some(index) = found else {
        return (None, Vec::new());
    };
    let rest = events.split_off(index + 1);
    let navigation = match events.pop() {
        Some(Event::Key {
            key: Key::Enter, ..
        }) => Some(Navigation::Enter),
        Some(Event::Key {
            key: Key::ArrowUp, ..
        }) => Some(Navigation::Up),
        Some(Event::Key {
            key: Key::ArrowDown,
            ..
        }) => Some(Navigation::Down),
        Some(Event::Key {
            key: Key::Escape, ..
        }) => Some(Navigation::Escape),
        _ => None,
    };
    (navigation, rest)
}

fn starts_frame(event: &Event) -> bool {
    matches!(
        event,
        Event::Key {
            key: Key::Tab,
            pressed: true,
            ..
        } | Event::PointerButton { pressed: true, .. }
    )
}

fn ends_frame(event: &Event) -> bool {
    matches!(
        event,
        Event::Key {
            key: Key::Enter | Key::ArrowUp | Key::ArrowDown | Key::Escape,
            pressed: true,
            ..
        }
    )
}

pub fn split_batch(events: &mut Vec<Event>) -> Vec<Event> {
    let split = events.iter().enumerate().find_map(|(index, event)| {
        if starts_frame(event) {
            Some(if index == 0 { 1 } else { index })
        } else if ends_frame(event) {
            Some(index + 1)
        } else {
            None
        }
    });
    match split {
        Some(at) if at < events.len() => events.split_off(at),
        _ => Vec::new(),
    }
}

fn filter_altgr(events: &mut Vec<Event>) {
    for event in events.iter_mut() {
        if let Event::Key {
            key: Key::Backspace,
            modifiers,
            ..
        } = event
        {
            if modifiers.alt {
                *modifiers = Modifiers::NONE;
            }
        }
    }
    events.retain(|event| {
        !matches!(
            event,
            Event::Key {
                key: Key::A,
                modifiers,
                ..
            } if modifiers.alt && (modifiers.ctrl || modifiers.command)
        )
    });
}

fn ink_color(ink: Ink, palette: &Palette) -> Color {
    match ink {
        Ink::Ink => palette.ink,
        Ink::Muted => palette.muted,
        Ink::Operation => palette.operation,
    }
}

fn draw_items(canvas: &Canvas, items: &[Item], palette: &Palette) {
    for item in items {
        match item {
            Item::Rule { rect } => canvas.fill_rect(*rect, palette.border),
            Item::Marked { rect } => match (palette.marked_fill, palette.marked_stroke) {
                (Some(fill), _) => canvas.rounded_box(Frame::inside(*rect, 7.0), fill, None),
                (None, Some(stroke)) => canvas.rounded_outline(Frame::inside(*rect, 7.0), stroke),
                (None, None) => {}
            },
            Item::Pivot { rect } => {
                canvas.rounded_outline(Frame::inside(*rect, 6.0), palette.pivot)
            }
            Item::Stroke { rect, muted } => {
                canvas.fill_rect(*rect, if *muted { palette.muted } else { palette.ink })
            }
            Item::Badge { rect, text } => {
                canvas.rounded_box(
                    Frame::inside(*rect, rect.w as f32 / 2.0),
                    palette.badge_fill,
                    None,
                );
                canvas.text(
                    text,
                    Font::Medium,
                    palette.badge_text,
                    *rect,
                    Align::Center,
                    VAlign::Center,
                );
            }
            Item::Text {
                text,
                font,
                ink,
                rect,
                centered,
            } => canvas.text(
                text,
                *font,
                ink_color(*ink, palette),
                *rect,
                if *centered {
                    Align::Center
                } else {
                    Align::Left
                },
                if *centered {
                    VAlign::Center
                } else {
                    VAlign::Top
                },
            ),
        }
    }
}

fn paint_button(canvas: &Canvas, palette: &Palette, look: &Look, hover: bool, ring: bool) {
    let rect = look.rect;
    let author = look.style == Style::Author;
    let segment = look.style == Style::Segment;
    let on_card = matches!(look.target, Target::Think | Target::Copy);
    let cta = look.target == Target::Think;
    let ghost = look.target == Target::Copy;
    let accent = look.selected;
    let face = if author {
        palette.link_face
    } else if segment {
        palette.track_fill
    } else {
        palette.button_face
    };
    let fill = if !look.enabled {
        if ghost {
            palette.solution_fill
        } else {
            palette.disabled_fill
        }
    } else if accent {
        if hover {
            palette.accent_hover
        } else {
            palette.accent_fill
        }
    } else if hover {
        if ghost {
            palette.ghost_hover
        } else {
            palette.hover_fill
        }
    } else if cta {
        palette.cta_fill
    } else if ghost {
        palette.solution_fill
    } else {
        face
    };
    let foreground = if !look.enabled {
        if ghost {
            palette.ghost_disabled
        } else {
            palette.disabled_text
        }
    } else if accent {
        palette.accent_text
    } else if hover {
        if ghost {
            palette.ghost_hover_text
        } else {
            palette.hover_text
        }
    } else if author {
        palette.link
    } else if segment {
        palette.ink
    } else if ghost {
        palette.ghost_text
    } else {
        palette.button_text
    };
    let icon_color = if cta && look.enabled && !(hover && palette.high_contrast) {
        palette.cta_icon
    } else {
        foreground
    };
    let secondary = if !look.enabled || accent || (hover && palette.high_contrast) || !author {
        foreground
    } else {
        palette.link_muted
    };
    let stroke = if ghost {
        Some(if look.enabled {
            palette.ghost_stroke
        } else {
            palette.ghost_disabled
        })
    } else if !look.enabled && palette.disabled_stroke.is_some() {
        palette.disabled_stroke
    } else if cta && look.enabled {
        palette.high_contrast.then_some(palette.button_stroke)
    } else if accent || author || segment {
        None
    } else {
        Some(palette.button_stroke)
    };
    let radius = if segment {
        layout::TRACK_RADIUS - layout::TRACK_INSET as f32 - 1.0
    } else {
        theme::BUTTON_RADIUS
    };
    canvas.rounded_box(
        Frame::new(
            (rect.x + 1) as f32,
            (rect.y + 1) as f32,
            (rect.w - 3) as f32,
            (rect.h - 3) as f32,
            radius,
        ),
        fill,
        stroke,
    );
    let icon_size = if look.icon.is_some() { ICON_SIZE } else { 0 };
    let gap = if look.icon.is_some() { ICON_GAP } else { 0 };
    if author {
        let run_gap = 8;
        let (left_margin, right_margin) = margins_for(fonts::metrics(Font::Medium));
        let runs = [
            (model::AUTHOR_NAME, foreground),
            (model::AUTHOR_SEPARATOR, secondary),
            (model::AUTHOR_RA, secondary),
        ];
        let text_width: i32 = runs
            .iter()
            .map(|(text, _)| canvas.measure(text, Font::Medium))
            .sum::<i32>()
            + run_gap * (runs.len() as i32 - 1)
            + left_margin
            + right_margin;
        let x = rect.x + (rect.w - (text_width + icon_size + gap)) / 2;
        canvas.icon(
            Icon::External,
            x as f32,
            (rect.y + (rect.h - icon_size) / 2) as f32,
            icon_size as f32,
            icon_size as f32,
            foreground,
        );
        canvas.runs(
            &runs,
            run_gap,
            Font::Medium,
            x + icon_size + gap + left_margin,
            rect,
        );
    } else {
        let text_width = canvas.measure_padded(look.text, Font::Medium);
        let x = rect.x + (rect.w - (text_width + icon_size + gap)) / 2;
        if let Some(icon) = look.icon {
            canvas.icon(
                icon,
                x as f32,
                (rect.y + (rect.h - icon_size) / 2) as f32,
                icon_size as f32,
                icon_size as f32,
                icon_color,
            );
        }
        canvas.text(
            look.text,
            Font::Medium,
            foreground,
            Rect::new(x + icon_size + gap, rect.y, text_width, rect.h),
            Align::Center,
            VAlign::Center,
        );
    }
    if ring {
        let filled = look.enabled && (accent || (hover && palette.high_contrast));
        let inset = if filled || on_card { 3.0 } else { 1.0 };
        let value = if filled {
            palette.primary_focus
        } else if ghost {
            palette.ghost_stroke
        } else if author || segment {
            palette.link_focus
        } else {
            palette.focus
        };
        canvas.ring(
            Frame::new(
                rect.x as f32 + inset,
                rect.y as f32 + inset,
                rect.w as f32 - 2.0 * inset,
                rect.h as f32 - 2.0 * inset,
                (radius - (inset - 1.0)).max(0.0),
            ),
            value,
            ring_width(1.0),
        );
    }
}

fn label_node(
    ui: &Ui,
    area: egui::Rect,
    id: Id,
    role: Role,
    text: &str,
    description: Option<&str>,
    live: bool,
) {
    let response = ui.interact(area, id, Sense::hover());
    let text = text.replace("\r\n", "\n");
    response.widget_info(|| WidgetInfo::labeled(role, true, &text));
    ui.ctx().accesskit_node_builder(id, |node| {
        if let Some(description) = description.filter(|value| !value.is_empty()) {
            node.set_description(description.to_string());
        }
        if live {
            node.set_live(accesskit::Live::Polite);
        }
        if role == Role::Heading {
            node.set_level(2);
        }
    });
}

impl App {
    pub fn new(context: &eframe::CreationContext<'_>, launch: Launch) -> App {
        let mut app = App::with_launch(launch);
        configure(&context.egui_ctx, &app.palette);
        if let Some(storage) = context.storage {
            app.restore(storage);
        }
        app
    }

    pub fn with_launch(launch: Launch) -> App {
        App {
            model: Model::new(),
            palette: contrast::palette(launch.high_contrast),
            buffers: Default::default(),
            focused: None,
            opener: Target::Think,
            keyboard: false,
            deferred: Vec::new(),
            raw_pending: Vec::new(),
            idle_due: None,
            spoken: None,
            announcement: None,
            announcements: Vec::new(),
            helper: (0, String::new(), false),
            helper_spoken: Vec::new(),
            status: (0, String::new()),
            page: None,
            link_hover: None,
            link_focus: None,
            scroll_reset: false,
            scroll_to: None,
            started: false,
            content: (0, 0),
            open_url: launch.open_url,
            motion: Motion::new(),
            sketch: Sketch::blank(2),
            caption: String::new(),
            seen: None,
            animate: launch.animations,
        }
    }

    pub fn restore(&mut self, storage: &dyn eframe::Storage) {
        if let Some(dimension) = storage
            .get_string(KEY_DIMENSION)
            .and_then(|text| text.trim().parse::<usize>().ok())
        {
            self.model.change_dimension(dimension);
            self.pull_buffers();
        }
        if let Some(method) = storage
            .get_string(KEY_METHOD)
            .and_then(|text| Method::from_key(text.trim()))
        {
            self.model.set_method(method);
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn focused(&self) -> Option<Target> {
        self.focused
    }

    pub fn content_size(&self) -> (i32, i32) {
        self.content
    }

    pub fn announcements(&self) -> &[String] {
        &self.announcements
    }

    pub fn idle_pending(&self) -> bool {
        self.idle_due.is_some()
    }

    pub fn input_pending(&self) -> bool {
        !self.raw_pending.is_empty() || !self.deferred.is_empty()
    }

    pub fn focus_visible(&self) -> bool {
        self.keyboard
    }

    pub fn animates(&self) -> bool {
        self.animate
    }

    pub fn graph_busy(&self, now: f64) -> bool {
        self.motion.busy(now, self.palette.high_contrast)
    }

    pub fn graph_sketch(&self) -> &Sketch {
        &self.sketch
    }

    pub fn input_text(&self, row: usize, column: usize) -> &str {
        self.buffers
            .get(row)
            .and_then(|values| values.get(column))
            .map_or("", String::as_str)
    }

    pub fn write(&mut self, ctx: &egui::Context, row: usize, column: usize, text: &str) {
        if row >= 3 || column >= 4 {
            return;
        }
        self.buffers[row][column] = text.to_string();
        self.edited(ctx, row, column);
        ctx.request_repaint();
    }

    pub fn press(&mut self, ctx: &egui::Context, target: Target) {
        match target {
            Target::Two => self.change_dimension(2),
            Target::Three => self.change_dimension(3),
            Target::Field(row, column) => self.focus_field(ctx, row, column, true),
            Target::Example => self.fill_example(),
            Target::Clear => self.clear(ctx),
            Target::Think => self.open_thinking(ctx, Target::Think),
            Target::Copy => self.copy(ctx),
            Target::Link(method) => self.request_method(ctx, method),
            Target::Back => self.close_thinking(ctx),
            Target::Tab(method) => self.model.set_method(method),
            Target::CopySteps => self.copy_steps(ctx),
            Target::Author => self.open_author(),
        }
        ctx.request_repaint();
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if !self.deferred.is_empty() {
            let deferred = std::mem::take(&mut self.deferred);
            ctx.input_mut(|input| {
                input.events.splice(0..0, deferred);
            });
        }
        if !self.started {
            self.started = true;
            self.focus_field(&ctx, 0, 0, false);
        }
        self.sync_focus(&ctx);
        let navigation = self.observe_input(&ctx);
        self.check_idle(&ctx);
        let mut actions = Vec::new();
        self.draw(ui, &mut actions);
        self.sync_focus(&ctx);
        let busy = !actions.is_empty() || navigation.is_some() || !self.deferred.is_empty();
        for action in actions {
            match action {
                Action::Press(target) => self.press(&ctx, target),
                Action::Edit(row, column) => self.edited(&ctx, row, column),
            }
        }
        if let Some(navigation) = navigation {
            self.navigate(&ctx, navigation);
        }
        if !matches!(self.model.card(), Card::Result { .. }) {
            self.spoken = None;
        }
        if busy || (!self.model.is_thinking() && self.graph_outdated()) {
            ctx.request_repaint();
        }
        if let Some(due) = self.idle_due {
            let now = ctx.input(|input| input.time);
            ctx.request_repaint_after_secs((due - now).max(0.0) as f32);
        }
    }

    fn graph_outdated(&self) -> bool {
        !self
            .seen
            .as_ref()
            .is_some_and(|seen| seen.matches(&self.model))
    }

    fn sync_graph(&mut self, now: f64) {
        if !self.graph_outdated() {
            return;
        }
        self.seen = Some(Seen::of(&self.model));
        self.sketch = self.model.sketch();
        self.caption = graph::caption(&self.sketch);
        self.motion.update(&self.sketch, now, self.animate);
        if !self.animate {
            self.motion.settle();
        }
    }

    fn helper_text(&self) -> String {
        if self.model.helper_is_caption() {
            self.caption.clone()
        } else {
            self.model.helper()
        }
    }

    fn observe_input(&mut self, ctx: &egui::Context) -> Option<Navigation> {
        let field = matches!(self.focused, Some(Target::Field(..)));
        let thinking = self.model.is_thinking();
        let (keyboard, navigation, rest) = ctx.input_mut(|input| {
            let mut keyboard = None;
            for event in &input.events {
                match event {
                    Event::Key { pressed: true, .. } => keyboard = Some(true),
                    Event::PointerButton { pressed: true, .. } => keyboard = Some(false),
                    _ => {}
                }
            }
            filter_altgr(&mut input.events);
            let (navigation, rest) = take_navigation(&mut input.events, field, thinking);
            (keyboard, navigation, rest)
        });
        if let Some(keyboard) = keyboard {
            self.keyboard = keyboard;
        }
        self.deferred = rest;
        navigation
    }

    fn navigate(&mut self, ctx: &egui::Context, navigation: Navigation) {
        if navigation == Navigation::Escape {
            if self.model.is_thinking() {
                self.close_thinking(ctx);
            }
            return;
        }
        let Some(Target::Field(row, column)) = self.focused else {
            return;
        };
        match navigation {
            Navigation::Enter => self.enter(ctx, row, column),
            Navigation::Up | Navigation::Down => {
                if let Some(Focus::Field { row, column, .. }) =
                    self.model
                        .neighbor(row, column, navigation == Navigation::Down)
                {
                    self.focus_field(ctx, row, column, true);
                }
            }
            Navigation::Escape => {}
        }
    }

    fn sync_focus(&mut self, ctx: &egui::Context) {
        let current = ctx
            .memory(|memory| memory.focused())
            .and_then(Target::from_id);
        if current == self.focused {
            return;
        }
        let previous = std::mem::replace(&mut self.focused, current);
        if let Some(Target::Field(row, column)) = previous {
            self.model.leave(row, column);
        }
        match current {
            Some(Target::Field(row, column)) => {
                self.model.set_focus(Some((row, column)));
            }
            _ => {
                self.model.set_focus(None);
            }
        }
    }

    fn check_idle(&mut self, ctx: &egui::Context) {
        let Some(due) = self.idle_due else {
            return;
        };
        let now = ctx.input(|input| input.time);
        if now + 1e-6 < due {
            return;
        }
        self.idle_due = None;
        self.model.idle();
        if let Some(text) = self.model.announcement() {
            if self.spoken.as_deref() != Some(text.as_str()) {
                self.spoken = Some(text.clone());
                self.announce(text);
            }
        }
    }

    fn announce(&mut self, text: String) {
        if text.trim().is_empty() {
            return;
        }
        let serial = self
            .announcement
            .as_ref()
            .map_or(1, |(serial, _)| serial + 1);
        self.announcements.push(text.clone());
        self.announcement = Some((serial, text));
    }

    fn announce_result(&mut self, prefix: Option<&str>) {
        let result = self.model.announcement();
        if let Some(text) = &result {
            self.spoken = Some(text.clone());
        }
        let text = match (prefix, result) {
            (Some(prefix), Some(result)) => format!("{prefix} {result}"),
            (Some(prefix), None) => prefix.to_string(),
            (None, Some(result)) => result,
            (None, None) => return,
        };
        self.announce(text);
    }

    fn refresh_helper_voice(&mut self) {
        let helper = self.model.helper();
        self.helper_spoken.retain(|text| *text != helper);
    }

    fn edited(&mut self, ctx: &egui::Context, row: usize, column: usize) {
        let text = self.buffers[row][column].clone();
        if self.model.edit(row, column, &text) {
            let now = ctx.input(|input| input.time);
            self.idle_due = Some(now + IDLE_DELAY);
        }
    }

    fn pull_buffers(&mut self) {
        for (row, values) in self.buffers.iter_mut().enumerate() {
            for (column, value) in values.iter_mut().enumerate() {
                let current = self.model.input(row, column);
                if value != current {
                    *value = current.to_string();
                }
            }
        }
    }

    fn focus(&mut self, ctx: &egui::Context, target: Target) {
        ctx.memory_mut(|memory| memory.request_focus(target.id()));
        self.scroll_to = Some(target);
        self.sync_focus(ctx);
    }

    fn focus_field(&mut self, ctx: &egui::Context, row: usize, column: usize, select_all: bool) {
        let target = Target::Field(row, column);
        let id = target.id();
        let text = self.buffers[row][column].as_str();
        let range = if select_all {
            CCursorRange::select_all_str(text)
        } else {
            CCursorRange::one(CCursor::new(text.chars().count()))
        };
        let mut state = TextEditState::load(ctx, id).unwrap_or_default();
        state.cursor.set_char_range(Some(range));
        state.store(ctx, id);
        self.focus(ctx, target);
    }

    fn apply_focus(&mut self, ctx: &egui::Context, focus: Option<Focus>) {
        match focus {
            Some(Focus::Field {
                row,
                column,
                select_all,
            }) => self.focus_field(ctx, row, column, select_all),
            None => {}
        }
    }

    fn change_dimension(&mut self, dimension: usize) {
        self.model.change_dimension(dimension);
        self.pull_buffers();
        self.idle_due = None;
        let summary = match self.model.announcement() {
            Some(text) => {
                self.spoken = Some(text.clone());
                text
            }
            None => format!("{}.", self.model.waiting_title().unwrap_or_default()),
        };
        let text = format!("{}. {summary}", self.model.dimension_text());
        self.announce(text);
    }

    fn enter(&mut self, ctx: &egui::Context, row: usize, column: usize) {
        let focus = self.model.enter(row, column);
        match focus {
            None => {
                self.idle_due = None;
                self.announce_result(None);
            }
            Some(_) => {
                if self.model.notice() == Some(Notice::OnlyThisField)
                    || self.model.field_has_error(row, column)
                {
                    self.refresh_helper_voice();
                }
                self.apply_focus(ctx, focus);
            }
        }
    }

    fn fill_example(&mut self) {
        let focus = self.model.fill_example();
        debug_assert!(focus.is_none());
        self.pull_buffers();
        self.idle_due = None;
        self.announce_result(Some(model::EXAMPLE_HELPER));
    }

    fn clear(&mut self, ctx: &egui::Context) {
        let focus = self.model.clear();
        self.pull_buffers();
        self.idle_due = None;
        self.spoken = None;
        self.apply_focus(ctx, Some(focus));
    }

    fn copy(&mut self, ctx: &egui::Context) {
        self.model.copy(|text| {
            ctx.copy_text(text.replace("\r\n", "\n"));
            true
        });
    }

    fn copy_steps(&mut self, ctx: &egui::Context) {
        self.model.copy_steps(|text| {
            ctx.copy_text(text.replace("\r\n", "\n"));
            true
        });
    }

    fn open_author(&mut self) {
        if !(self.open_url)(model::AUTHOR_URL) {
            self.model.author_unavailable();
            self.refresh_helper_voice();
        }
    }

    fn open_thinking(&mut self, ctx: &egui::Context, opener: Target) {
        if self.model.open_thinking() {
            self.opener = opener;
            self.enter_thinking(ctx);
        }
    }

    fn enter_thinking(&mut self, ctx: &egui::Context) {
        self.idle_due = None;
        self.scroll_reset = true;
        self.focus(ctx, Target::Tab(self.model.method()));
    }

    fn close_thinking(&mut self, ctx: &egui::Context) {
        if self.model.close_thinking() {
            self.scroll_reset = true;
            let opener = self.opener;
            self.focus(ctx, opener);
        }
    }

    fn request_method(&mut self, ctx: &egui::Context, method: Method) {
        let had_result = self.model.think_enabled();
        match self.model.request_method(method) {
            MethodRequest::Opened => {
                if !had_result {
                    self.idle_due = None;
                    self.announce_result(None);
                }
                self.opener = Target::Link(method);
                self.enter_thinking(ctx);
            }
            MethodRequest::NeedsSystem(focus) => {
                self.refresh_helper_voice();
                self.apply_focus(ctx, focus);
            }
        }
    }

    fn page_for(&mut self, ctx: &egui::Context, content: Rect) {
        let Some(walkthrough) = self.model.walkthrough() else {
            self.page = None;
            return;
        };
        if self
            .page
            .as_ref()
            .is_some_and(|cache| cache.content == content && cache.walkthrough == *walkthrough)
        {
            return;
        }
        let page = thinking::page(walkthrough, &fonts::Measurer(ctx), 1.0, content);
        self.page = Some(PageCache {
            walkthrough: walkthrough.clone(),
            content,
            page,
        });
    }

    fn draw(&mut self, ui: &mut Ui, actions: &mut Vec<Action>) {
        let ctx = ui.ctx().clone();
        let full = ui.max_rect();
        ui.painter()
            .rect_filled(full, egui::CornerRadius::ZERO, color(self.palette.canvas));
        let client_w = full.width().floor() as i32;
        let client_h = full.height().floor() as i32;
        let (width, height) = layout::content_size(1.0, client_w, client_h, 0, 0);
        let mut arranged = layout::compute(1.0, width, height, self.model.dimension(), metrics());
        let cache = if self.model.is_thinking() {
            self.page_for(&ctx, arranged.sheet_content);
            self.page.take()
        } else {
            None
        };
        if let Some(cache) = &cache {
            arranged = arranged.with_sheet_bottom(cache.page.bottom);
        }
        self.content = (arranged.content_w, arranged.content_h);
        let mut area = ScrollArea::both()
            .id_salt("pagina")
            .auto_shrink([false, false]);
        if std::mem::take(&mut self.scroll_reset) {
            area = area.scroll_offset(Vec2::ZERO);
        }
        area.show_viewport(ui, |ui, viewport| {
            let (_, rect) =
                ui.allocate_space(vec2(arranged.content_w as f32, arranged.content_h as f32));
            let canvas = Canvas::new(ui.painter().clone(), rect.min);
            let visible = Rect::new(
                viewport.min.x.floor() as i32,
                viewport.min.y.floor() as i32,
                viewport.width().ceil() as i32 + 1,
                viewport.height().ceil() as i32 + 1,
            );
            self.draw_header(&canvas, &arranged);
            match &cache {
                Some(cache) => {
                    self.draw_thinking(ui, &canvas, &arranged, &cache.page, visible, actions);
                }
                None => self.draw_main(ui, &canvas, &arranged, actions),
            }
            self.draw_author(ui, &canvas, &arranged, actions);
        });
        if cache.is_some() {
            self.page = cache;
        }
    }

    fn draw_header(&self, canvas: &Canvas, layout: &Layout) {
        let palette = &self.palette;
        canvas.fill_rect(layout.separator, palette.border);
        canvas.icon_view(Icon::Matrix, layout.logo, palette.icon);
        canvas.text(
            model::BRAND,
            Font::Medium,
            palette.ink,
            layout.brand,
            Align::Left,
            VAlign::Top,
        );
    }

    fn button(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        look: Look,
        actions: &mut Vec<Action>,
    ) -> Response {
        let area = canvas.at(look.rect);
        let id = look.target.id();
        let sense = if look.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let mut response = ui.interact(area, id, sense);
        let hover = look.enabled && response.hovered();
        let ring = response.has_focus() && self.keyboard;
        paint_button(canvas, &self.palette, &look, hover, ring);
        let role = look.role;
        let enabled = look.enabled;
        let name = look.name.clone();
        response.widget_info(|| WidgetInfo::labeled(role, enabled, &name));
        let selected = look.selected;
        let description = look.description;
        ui.ctx().accesskit_node_builder(id, |node| {
            if role == Role::Tab {
                node.set_selected(selected);
            }
            if let Some(description) = description {
                node.set_description(description.to_string());
            }
        });
        if let Some(tip) = look.tip {
            response = response.on_hover_text(tip);
        }
        if look.enabled {
            response = response.on_hover_cursor(CursorIcon::PointingHand);
        }
        if response.clicked() {
            response.request_focus();
            actions.push(Action::Press(look.target));
        }
        self.reveal(&response, look.target);
        response
    }

    fn reveal(&mut self, response: &Response, target: Target) {
        if self.scroll_to == Some(target) {
            self.scroll_to = None;
            response.scroll_to_me(None);
        } else if response.gained_focus() {
            response.scroll_to_me(None);
        }
    }

    fn field(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        row: usize,
        column: usize,
        actions: &mut Vec<Action>,
    ) {
        let rect = layout.fields[row][column];
        let target = Target::Field(row, column);
        let id = target.id();
        let area = canvas.at(rect);
        let palette = self.palette;
        let focused = ui.memory(|memory| memory.has_focus(id));
        let hovered = ui.rect_contains_pointer(area);
        let error = self.model.field_has_error(row, column);
        let border = if error {
            palette.error
        } else if focused {
            palette.accent
        } else if hovered {
            palette.field_hover
        } else {
            palette.border
        };
        let (x, y, w, h) = layout::field_box(rect);
        canvas.rounded_box(
            Frame::new(x, y, w, h, theme::FIELD_RADIUS),
            palette.field,
            Some(border),
        );
        if focused && !error {
            if let Some(glow) = palette.focus_glow {
                canvas.ring(
                    Frame::new(
                        x + 1.0,
                        y + 1.0,
                        w - 1.0,
                        h - 1.0,
                        theme::FIELD_RADIUS - 1.0,
                    ),
                    glow,
                    ring_width(1.0),
                );
            }
        }
        if let Some(suffix) = layout::suffix(column) {
            canvas.text(
                suffix,
                Font::Medium,
                palette.muted,
                layout::suffix_rect(rect, 1.0),
                Align::Center,
                VAlign::Center,
            );
        }
        let editor = layout.editors[row][column];
        let margin = Margin {
            left: (editor.x - rect.x).clamp(0, 127) as i8,
            right: (rect.right() - editor.right()).clamp(0, 127) as i8,
            top: (editor.y - rect.y).clamp(0, 127) as i8,
            bottom: (rect.bottom() - editor.bottom()).clamp(0, 127) as i8,
        };
        let edit = TextEdit::singleline(&mut self.buffers[row][column])
            .id(id)
            .font(fonts::id(Font::Input))
            .text_color(color(palette.ink))
            .frame(egui::Frame::NONE.inner_margin(margin))
            .align(egui::Align2::CENTER_CENTER)
            .desired_width(area.width())
            .min_size(area.size())
            .char_limit(INPUT_LIMIT)
            .return_key(None)
            .event_filter(EventFilter {
                tab: false,
                horizontal_arrows: true,
                vertical_arrows: true,
                escape: true,
            });
        let response = ui
            .put(area, edit)
            .accessible_name(model::accessible_input_name(row, column));
        let message = match Rational::parse(&self.buffers[row][column]) {
            Err(problem) if error => problem.message(),
            _ => model::FIELD_DESCRIPTION,
        };
        ui.ctx().accesskit_node_builder(id, |node| {
            if error {
                node.set_invalid(accesskit::Invalid::True);
            }
            node.set_description(message.to_string());
        });
        if response.changed() {
            actions.push(Action::Edit(row, column));
        }
        self.reveal(&response, target);
    }

    fn draw_main(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        actions: &mut Vec<Action>,
    ) {
        let palette = self.palette;
        let now = ui.input(|input| input.time);
        self.sync_graph(now);
        canvas.text(
            model::INTRO,
            Font::Large,
            palette.ink,
            layout.intro,
            Align::Left,
            VAlign::Top,
        );
        canvas.text(
            model::SUBTITLE,
            Font::Body,
            palette.muted,
            layout.subtitle,
            Align::Left,
            VAlign::Top,
        );
        canvas.card(
            layout.entry,
            palette.surface,
            palette.border,
            palette.card_highlight,
        );
        canvas.icon_view(Icon::Grid, layout.entry_icon, palette.icon);
        canvas.text(
            model::ENTRY_TITLE,
            Font::Heading,
            palette.ink,
            layout.entry_title,
            Align::Left,
            VAlign::Top,
        );
        label_node(
            ui,
            canvas.at(layout.entry_title),
            Id::unique("titulo-entrada"),
            Role::Heading,
            model::ENTRY_TITLE,
            Some(model::ENTRY_DESCRIPTION),
            false,
        );
        canvas.text(
            model::ENTRY_DESCRIPTION,
            Font::Body,
            palette.muted,
            layout.entry_description,
            Align::Left,
            VAlign::Top,
        );
        canvas.text(
            model::GRID_LABEL,
            Font::Small,
            palette.muted,
            layout.grid_label,
            Align::Left,
            VAlign::Center,
        );
        canvas.rounded_box(
            Frame::inside(layout.size_track, layout::TRACK_RADIUS),
            palette.track_fill,
            Some(palette.track_stroke),
        );
        let dimension = self.model.dimension();
        let mut sizes = ui.new_child(
            UiBuilder::new()
                .max_rect(canvas.at(layout.size_track))
                .accessibility_role(Role::Group)
                .accessibility_label(model::SIZE_LABEL),
        );
        for (target, rect, value, text) in [
            (Target::Two, layout.two, 2, model::TWO_LABEL),
            (Target::Three, layout.three, 3, model::THREE_LABEL),
        ] {
            self.button(
                &mut sizes,
                canvas,
                Look {
                    target,
                    rect,
                    text,
                    name: model::size_name(value, dimension == value),
                    icon: None,
                    style: Style::Segment,
                    selected: dimension == value,
                    enabled: true,
                    tip: None,
                    description: None,
                    role: Role::Button,
                },
                actions,
            );
        }
        self.draw_actions(ui, canvas, layout, actions);
        let dimension_text = self.model.dimension_text();
        canvas.text(
            &dimension_text,
            Font::Small,
            palette.muted,
            layout.dimension_text,
            Align::Right,
            VAlign::Center,
        );
        label_node(
            ui,
            canvas.at(layout.dimension_text),
            Id::unique("dimensao"),
            Role::Label,
            &dimension_text,
            None,
            false,
        );
        for row in 0..dimension {
            let number = if self.model.row_complete(row) {
                palette.graph_rows[row]
            } else {
                palette.muted
            };
            canvas.text(
                &model::row_number(row),
                Font::Small,
                number,
                layout.row_numbers[row],
                Align::Left,
                VAlign::Top,
            );
            for column in 0..dimension {
                canvas.text(
                    layout::operator_text(dimension, column),
                    Font::Medium,
                    palette.muted,
                    layout.operators[row][column],
                    Align::Left,
                    VAlign::Top,
                );
            }
            for column in 0..4 {
                if layout::field_visible(dimension, row, column) {
                    self.field(ui, canvas, layout, row, column, actions);
                }
            }
        }
        self.draw_graph(ui, canvas, layout, now);
        let helper = self.helper_text();
        self.draw_helper(ui, canvas, layout, helper);
        self.draw_solution(ui, canvas, layout, actions);
        self.draw_footer(ui, canvas, layout, actions);
    }

    fn draw_actions(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        actions: &mut Vec<Action>,
    ) {
        for (target, rect, text, icon, tip) in [
            (
                Target::Example,
                layout.example,
                model::EXAMPLE_LABEL,
                Icon::Sparkles,
                model::TIP_EXAMPLE,
            ),
            (
                Target::Clear,
                layout.clear,
                model::CLEAR_LABEL,
                Icon::Reset,
                model::TIP_CLEAR,
            ),
        ] {
            self.button(
                ui,
                canvas,
                Look {
                    target,
                    rect,
                    text,
                    name: text.to_string(),
                    icon: Some(icon),
                    style: Style::Plain,
                    selected: false,
                    enabled: true,
                    tip: Some(tip),
                    description: None,
                    role: Role::Button,
                },
                actions,
            );
        }
    }

    fn draw_graph(&self, ui: &mut Ui, canvas: &Canvas, layout: &Layout, now: f64) {
        if layout.graph.is_empty() || layout.plot.is_empty() {
            return;
        }
        let palette = self.palette;
        canvas.rounded_box(
            Frame::inside(layout.graph, theme::GRAPH_RADIUS),
            palette.graph_fill,
            Some(palette.graph_stroke),
        );
        let plot = layout.plot;
        let items = self.motion.scene(
            now,
            f64::from(plot.w),
            f64::from(plot.h),
            1.0,
            palette.high_contrast,
        );
        canvas.scene(plot, &items, &palette);
        if self.motion.busy(now, palette.high_contrast) {
            ui.ctx().request_repaint();
        }
        let id = Id::unique("grafico");
        let response = ui.interact(canvas.at(plot), id, Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(Role::Image, true, graph::GRAPH_NAME));
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_description(self.caption.clone());
        });
    }

    fn draw_helper(&mut self, ui: &mut Ui, canvas: &Canvas, layout: &Layout, helper: String) {
        let palette = self.palette;
        let error = self.model.helper_is_error();
        canvas.text(
            &keep_groups(&helper),
            Font::Small,
            if error {
                palette.error_text
            } else if self.model.helper_is_reading() {
                palette.ink
            } else {
                palette.muted
            },
            layout.helper,
            Align::Left,
            VAlign::Center,
        );
        let important = error
            || matches!(
                self.model.notice(),
                Some(Notice::OnlyThisField | Notice::MethodNeedsSystem(_) | Notice::Author)
            );
        if !important {
            self.helper_spoken.clear();
        }
        if self.helper.1 != helper || (important && !self.helper_spoken.contains(&helper)) {
            let live = important && !self.helper_spoken.contains(&helper);
            if live {
                self.helper_spoken.push(helper.clone());
                self.announcements.push(helper.clone());
            }
            self.helper = (self.helper.0 + 1, helper.clone(), live);
        }
        label_node(
            ui,
            canvas.at(layout.helper),
            Id::unique(("dica", self.helper.0)),
            Role::Label,
            &helper,
            None,
            self.helper.2,
        );
    }

    fn draw_solution(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        actions: &mut Vec<Action>,
    ) {
        let palette = self.palette;
        canvas.card(
            layout.solution,
            palette.solution_fill,
            palette.solution_stroke,
            palette.card_highlight,
        );
        canvas.text(
            model::RESULT_LABEL,
            Font::Small,
            palette.solution_text,
            layout.solution_label,
            Align::Left,
            VAlign::Top,
        );
        label_node(
            ui,
            canvas.at(layout.solution_label),
            Id::unique("titulo-resultado"),
            Role::Heading,
            model::RESULT_LABEL,
            None,
            false,
        );
        match self.model.card() {
            Card::Waiting(_) => self.draw_waiting(ui, canvas, layout),
            Card::Result { stale } => self.draw_result(ui, canvas, layout, stale),
        }
        if let Some((serial, text)) = &self.announcement {
            label_node(
                ui,
                canvas.at(layout.solution_title),
                Id::unique(("anuncio", *serial)),
                Role::Label,
                text,
                None,
                true,
            );
        }
        let note = self.model.note().to_string();
        canvas.text(
            &note,
            Font::Small,
            palette.solution_muted,
            layout.note,
            Align::Left,
            VAlign::Top,
        );
        label_node(
            ui,
            canvas.at(layout.note),
            Id::unique("nota"),
            Role::Label,
            &note,
            None,
            false,
        );
        let think = self.model.think_enabled();
        self.button(
            ui,
            canvas,
            Look {
                target: Target::Think,
                rect: layout.think,
                text: model::THINK_LABEL,
                name: model::THINK_LABEL.to_string(),
                icon: Some(Icon::Bulb),
                style: Style::Plain,
                selected: false,
                enabled: think,
                tip: Some(if think {
                    model::TIP_THINK
                } else {
                    model::TIP_DISABLED
                }),
                description: None,
                role: Role::Button,
            },
            actions,
        );
        let copy = self.model.copy_enabled();
        let label = self.model.copy_label();
        self.button(
            ui,
            canvas,
            Look {
                target: Target::Copy,
                rect: layout.copy,
                text: label,
                name: label.to_string(),
                icon: Some(Icon::Copy),
                style: Style::Plain,
                selected: false,
                enabled: copy,
                tip: Some(if copy {
                    model::TIP_COPY
                } else {
                    model::TIP_DISABLED
                }),
                description: None,
                role: Role::Button,
            },
            actions,
        );
    }

    fn draw_waiting(&mut self, ui: &mut Ui, canvas: &Canvas, layout: &Layout) {
        let palette = self.palette;
        if self.model.shows_preview() {
            let font = Font::Title;
            let metric = fonts::metrics(font);
            let margins = margins_for(metric);
            let lines = self.model.preview();
            let plan = layout::preview_plan(layout.preview, 1.0, lines.len(), metric.height.max(1));
            canvas.rounded_box(
                Frame::inside(plan.frame, layout::PREVIEW_RADIUS),
                palette.tile_fill,
                Some(palette.tile_stroke),
            );
            let measure = |text: &str| canvas.measure(text, font);
            for (line, rect) in lines.iter().zip(&plan.lines) {
                let available = rect.w - margins.0 - margins.1;
                let text = ellipsize(line, available, &measure);
                canvas.text(
                    &text,
                    font,
                    palette.solution_text,
                    *rect,
                    Align::Center,
                    VAlign::Top,
                );
            }
        } else {
            canvas.icon_view(Icon::Matrix, layout.empty_icon, palette.solution_text);
        }
        let title = self.model.waiting_title().unwrap_or_default();
        let description = self.model.waiting_description().unwrap_or_default();
        canvas.text(
            &title,
            Font::Heading,
            palette.solution_text,
            layout.empty_title,
            Align::Center,
            VAlign::Center,
        );
        canvas.text(
            description,
            Font::Body,
            palette.solution_muted,
            layout.empty_description,
            Align::Center,
            VAlign::Top,
        );
        let spoken = if self.model.shows_preview() {
            self.model.preview_spoken()
        } else {
            String::new()
        };
        label_node(
            ui,
            canvas.at(layout.empty_title),
            Id::unique("titulo"),
            Role::Label,
            &title,
            Some(&spoken),
            false,
        );
        label_node(
            ui,
            canvas.at(layout.empty_description),
            Id::unique("descricao"),
            Role::Label,
            &description.replace('\n', " "),
            None,
            false,
        );
    }

    fn draw_result(&mut self, ui: &mut Ui, canvas: &Canvas, layout: &Layout, stale: bool) {
        let palette = self.palette;
        let title = self.model.title().unwrap_or_default();
        let description = self.model.description().unwrap_or_default();
        let title_color = if stale {
            palette.stale(palette.solution_text, palette.solution_fill)
        } else {
            palette.solution_text
        };
        canvas.text(
            title,
            Font::Heading,
            title_color,
            layout.solution_title,
            Align::Left,
            VAlign::Top,
        );
        canvas.text(
            description,
            Font::Body,
            palette.solution_muted,
            layout.solution_description,
            Align::Left,
            VAlign::Top,
        );
        let spoken = self
            .model
            .announcement()
            .unwrap_or_else(|| title.to_string());
        label_node(
            ui,
            canvas.at(layout.solution_title),
            Id::unique("titulo"),
            Role::Label,
            &spoken,
            None,
            false,
        );
        label_node(
            ui,
            canvas.at(layout.solution_description),
            Id::unique("descricao"),
            Role::Label,
            &description.replace('\n', " "),
            None,
            false,
        );
        let values: Vec<String> = self.model.values().to_vec();
        let colors = tile_colors(&palette, stale);
        for (index, value) in values.iter().enumerate().take(3) {
            let tile = layout.tiles[index];
            canvas.card(tile, colors.fill, colors.stroke, palette.card_highlight);
            canvas.text(
                VARIABLE_NAMES[index],
                Font::Heading,
                colors.text,
                layout.tile_names[index],
                Align::Left,
                VAlign::Top,
            );
            let field = layout.tile_values[index];
            let metric = fonts::metrics(Font::Value);
            let clip = Rect::new(
                field.x,
                field.y,
                field.w,
                metric.height.max(field.h).min(tile.bottom() - field.y - 1),
            );
            canvas.line(value, Font::Value, colors.text, field.x, field.y, clip);
            let response = ui.interact(
                canvas.at(tile),
                Id::unique(("valor", index)),
                Sense::hover(),
            );
            let name = format!("{}: {}", model::accessible_value_name(index), value);
            response.widget_info(|| WidgetInfo::labeled(Role::Label, true, &name));
            if canvas.measure(value, Font::Value) > field.w {
                let _ = response.on_hover_text(value.as_str());
            }
        }
        if let Some(general) = self.model.general() {
            let general = general.to_string();
            let ink = if stale {
                palette.stale(palette.solution_text, palette.solution_fill)
            } else {
                palette.solution_text
            };
            self.draw_general(ui, canvas, layout, &general, title, ink);
        }
    }

    fn draw_general(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        general: &str,
        title: &str,
        ink: Color,
    ) {
        let area = layout.general;
        let metric = fonts::metrics(Font::Input);
        let measure = |text: &str| canvas.measure(text, Font::Input);
        let tall = Rect::new(0, 0, area.w, i32::MAX / 4);
        let lines = crate::paint::plan_text(
            general,
            tall,
            Align::Left,
            VAlign::Top,
            metric,
            metric.height,
            &measure,
        );
        let needed = lines
            .last()
            .map_or(0, |line| line.y + metric.height)
            .max(area.h);
        label_node(
            ui,
            canvas.at(area),
            Id::unique("geral"),
            Role::Label,
            general,
            Some(title),
            false,
        );
        if needed <= area.h {
            canvas.text(general, Font::Input, ink, area, Align::Left, VAlign::Top);
            return;
        }
        let mut child = ui.new_child(UiBuilder::new().max_rect(canvas.at(area)));
        ScrollArea::vertical()
            .id_salt("geral-rolagem")
            .max_height(area.h as f32)
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                let (_, rect) = ui.allocate_space(vec2(area.w as f32 - 12.0, needed as f32));
                let inner = Canvas::new(ui.painter().clone(), rect.min);
                for line in &lines {
                    inner.line(
                        &line.text,
                        Font::Input,
                        ink,
                        line.x,
                        line.y,
                        Rect::new(0, 0, area.w, needed),
                    );
                }
            });
    }

    fn keycaps(&self, canvas: &Canvas, container: Rect, keys: &[(&str, &str)]) {
        let palette = &self.palette;
        let font = Font::Small;
        let margins = margins_for(fonts::metrics(font));
        let measure = |text: &str| canvas.measure(text, font);
        let caps = layout::keycaps(container, 1.0, keys, &measure);
        for (cap, (key, action)) in caps.iter().zip(keys) {
            canvas.rounded_box(
                Frame::inside(cap.key, KEYCAP_RADIUS),
                palette.keycap_fill,
                Some(palette.keycap_stroke),
            );
            canvas.text(
                key,
                font,
                palette.keycap_text,
                padded(cap.key, margins),
                Align::Center,
                VAlign::Center,
            );
            canvas.text(
                action,
                font,
                palette.muted,
                padded(cap.action, margins),
                Align::Left,
                VAlign::Center,
            );
        }
    }

    fn link(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        method: Method,
        rect: Rect,
        actions: &mut Vec<Action>,
    ) {
        let palette = self.palette;
        let target = Target::Link(method);
        let id = target.id();
        let mut response = ui.interact(canvas.at(rect), id, Sense::click());
        let focused = response.has_focus();
        let hovered = response.hovered();
        if hovered {
            self.link_hover = Some(method);
        } else if focused && self.link_focus.is_none() {
            self.link_focus = Some(method);
        }
        let active = hovered || focused;
        let value = if active {
            palette.footer_link_hover
        } else {
            palette.footer_link
        };
        let font = Font::Small;
        let metric = fonts::metrics(font);
        let (left_margin, right_margin) = margins_for(metric);
        let label = method.label();
        canvas.text(label, font, value, rect, Align::Center, VAlign::Center);
        let width = canvas.measure(label, font);
        let x = rect.x + (left_margin + rect.w - right_margin - width) / 2;
        let top = rect.h / 2 - metric.height.max(1) / 2;
        let underline = rect.y + (top + metric.ascent + 2).min(rect.h - 3);
        if active {
            canvas.fill_rect(Rect::new(x, underline, width, 1), value);
        } else {
            canvas.dotted_underline(x, underline, width, value);
        }
        if focused && self.keyboard {
            canvas.ring(
                Frame::new(
                    rect.x as f32,
                    rect.y as f32,
                    rect.w as f32,
                    rect.h as f32,
                    theme::FOCUS_RADIUS,
                ),
                palette.link_focus,
                ring_width(1.0),
            );
        }
        let name = model::method_link_name(method);
        response.widget_info(|| WidgetInfo::labeled(Role::Link, true, name));
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_description(model::method_tip(method).to_string());
        });
        response = response
            .on_hover_text(model::method_tip(method))
            .on_hover_cursor(CursorIcon::PointingHand);
        if response.clicked() {
            response.request_focus();
            actions.push(Action::Press(target));
        }
        self.reveal(&response, target);
    }

    fn draw_footer(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        actions: &mut Vec<Action>,
    ) {
        let palette = self.palette;
        canvas.text(
            model::FOOTER_METHODS_LABEL,
            Font::Small,
            palette.muted,
            layout.footer_label,
            Align::Left,
            VAlign::Center,
        );
        self.link_hover = None;
        self.link_focus = None;
        let group = layout::union(layout.footer_label, layout.method_links[2]);
        let mut links = ui.new_child(
            UiBuilder::new()
                .max_rect(canvas.at(group))
                .accessibility_role(Role::Group)
                .accessibility_label(model::FOOTER_METHODS_LABEL),
        );
        for (index, method) in Method::ALL.into_iter().enumerate() {
            self.link(
                &mut links,
                canvas,
                method,
                layout.method_links[index],
                actions,
            );
        }
        match self.footer_tip() {
            Some(method) => {
                let font = Font::Small;
                let (left, right) = margins_for(fonts::metrics(font));
                let available = layout.shortcut.w - left - right;
                let tip = ellipsize(model::method_tip(method), available, &|text| {
                    canvas.measure(text, font)
                });
                canvas.text(
                    &tip,
                    font,
                    palette.ink,
                    layout.shortcut,
                    Align::Right,
                    VAlign::Center,
                );
            }
            None => self.keycaps(canvas, layout.shortcut, &model::KEYS),
        }
    }

    pub fn footer_tip(&self) -> Option<Method> {
        if self.model.is_thinking() {
            return None;
        }
        self.link_hover.or(self.link_focus)
    }

    fn draw_author(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        actions: &mut Vec<Action>,
    ) {
        self.button(
            ui,
            canvas,
            Look {
                target: Target::Author,
                rect: layout.author,
                text: model::AUTHOR_LABEL,
                name: model::AUTHOR_LABEL.to_string(),
                icon: Some(Icon::External),
                style: Style::Author,
                selected: false,
                enabled: true,
                tip: Some(model::AUTHOR_TOOLTIP),
                description: Some(model::AUTHOR_DESCRIPTION),
                role: Role::Link,
            },
            actions,
        );
    }

    fn draw_thinking(
        &mut self,
        ui: &mut Ui,
        canvas: &Canvas,
        layout: &Layout,
        page: &Page,
        visible: Rect,
        actions: &mut Vec<Action>,
    ) {
        let palette = self.palette;
        canvas.text(
            model::THINK_INTRO,
            Font::Large,
            palette.ink,
            layout.intro,
            Align::Left,
            VAlign::Top,
        );
        canvas.text(
            model::THINK_SUBTITLE,
            Font::Body,
            palette.muted,
            layout.subtitle,
            Align::Left,
            VAlign::Top,
        );
        canvas.card(
            layout.sheet,
            palette.surface,
            palette.border,
            palette.card_highlight,
        );
        self.button(
            ui,
            canvas,
            Look {
                target: Target::Back,
                rect: layout.back,
                text: model::BACK_LABEL,
                name: model::BACK_LABEL.to_string(),
                icon: Some(Icon::ArrowLeft),
                style: Style::Plain,
                selected: false,
                enabled: true,
                tip: None,
                description: None,
                role: Role::Button,
            },
            actions,
        );
        canvas.text(
            model::METHOD_LABEL,
            Font::Small,
            palette.muted,
            layout.method_label,
            Align::Left,
            VAlign::Center,
        );
        canvas.rounded_box(
            Frame::inside(layout.method_track, layout::TRACK_RADIUS),
            palette.track_fill,
            Some(palette.track_stroke),
        );
        let current = self.model.method();
        let mut tabs = ui.new_child(
            UiBuilder::new()
                .max_rect(canvas.at(layout.method_track))
                .accessibility_role(Role::TabList)
                .accessibility_label(model::METHOD_LABEL),
        );
        for (index, method) in Method::ALL.into_iter().enumerate() {
            self.button(
                &mut tabs,
                canvas,
                Look {
                    target: Target::Tab(method),
                    rect: layout.methods[index],
                    text: method.label(),
                    name: method.label().to_string(),
                    icon: None,
                    style: Style::Segment,
                    selected: current == method,
                    enabled: true,
                    tip: None,
                    description: Some(model::method_tip(method)),
                    role: Role::Tab,
                },
                actions,
            );
        }
        let steps_label = self.model.steps_copy_label();
        self.button(
            ui,
            canvas,
            Look {
                target: Target::CopySteps,
                rect: layout.copy_steps,
                text: steps_label,
                name: steps_label.to_string(),
                icon: Some(Icon::Copy),
                style: Style::Plain,
                selected: false,
                enabled: true,
                tip: None,
                description: None,
                role: Role::Button,
            },
            actions,
        );
        canvas.icon_view(Icon::Bulb, layout.status_icon, palette.icon);
        let status = self.model.thinking_status();
        canvas.text(
            &status,
            Font::Heading,
            palette.ink,
            layout.status,
            Align::Left,
            VAlign::Top,
        );
        if self.status.1 != status {
            self.announcements.push(status.clone());
            self.status = (self.status.0 + 1, status.clone());
        }
        label_node(
            ui,
            canvas.at(layout.status),
            Id::unique(("status", self.status.0)),
            Role::Label,
            &status,
            None,
            true,
        );
        draw_items(canvas, &page.summary, &palette);
        if let Some(walkthrough) = self.model.walkthrough() {
            let content = layout.sheet_content;
            let column = (content.w - 32) / 2;
            let top = content.y;
            let bottom = page
                .summary
                .iter()
                .filter(|item| !matches!(item, Item::Rule { .. }))
                .map(|item| item.rect().bottom())
                .max()
                .unwrap_or(top + 40);
            let mut system = vec![model::SYSTEM_LABEL.to_string()];
            system.extend(walkthrough.system.iter().cloned());
            let mut answer = vec![model::ANSWER_LABEL.to_string()];
            answer.extend(walkthrough.answer.iter().cloned());
            answer.push(walkthrough.classification.to_string());
            label_node(
                ui,
                canvas.at(Rect::new(content.x, top, column, bottom - top)),
                Id::unique("resumo-sistema"),
                Role::Label,
                &system.join("\n"),
                None,
                false,
            );
            label_node(
                ui,
                canvas.at(Rect::new(
                    content.x + column + 32,
                    top,
                    column,
                    bottom - top,
                )),
                Id::unique("resumo-resposta"),
                Role::Label,
                &answer.join("\n"),
                None,
                false,
            );
        }
        for (index, step) in page.steps.iter().enumerate() {
            if step.rect.intersects(visible) {
                draw_items(
                    &canvas.offset(step.rect.x, step.rect.y),
                    &step.items,
                    &palette,
                );
            }
            label_node(
                ui,
                canvas.at(step.rect),
                Id::unique(("passo", index)),
                Role::Label,
                &step.text,
                None,
                false,
            );
        }
        canvas.text(
            model::THINK_FOOTER,
            Font::Small,
            palette.muted,
            layout.footer,
            Align::Left,
            VAlign::Center,
        );
        self.keycaps(canvas, layout.shortcut, &model::THINK_KEYS);
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        let mut events = std::mem::take(&mut self.raw_pending);
        events.append(&mut raw_input.events);
        self.raw_pending = split_batch(&mut events);
        raw_input.events = events;
        if !self.raw_pending.is_empty() {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(KEY_METHOD, self.model.method().key().to_string());
        storage.set_string(KEY_DIMENSION, self.model.dimension().to_string());
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        color(self.palette.canvas).to_normalized_gamma_f32()
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tecla(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    #[test]
    fn alvos_tem_ids_unicos() {
        let alvos = Target::all();
        assert_eq!(alvos.len(), 2 + 12 + 4 + 3 + 1 + 3 + 2);
        for alvo in &alvos {
            assert_eq!(Target::from_id(alvo.id()), Some(*alvo));
        }
        let mut ids: Vec<Id> = alvos.iter().map(|alvo| alvo.id()).collect();
        ids.sort_by_key(|id| id.value());
        ids.dedup();
        assert_eq!(ids.len(), alvos.len());
    }

    #[test]
    fn altgr_nao_seleciona_tudo_nem_apaga_palavra() {
        let altgr = Modifiers {
            alt: true,
            ctrl: true,
            command: true,
            ..Modifiers::NONE
        };
        let mut eventos = vec![
            tecla(Key::A, altgr),
            Event::Text("á".to_string()),
            tecla(Key::Backspace, altgr),
            tecla(Key::A, Modifiers::COMMAND),
        ];
        filter_altgr(&mut eventos);
        assert_eq!(eventos.len(), 3);
        assert_eq!(eventos[0], Event::Text("á".to_string()));
        assert_eq!(eventos[1], tecla(Key::Backspace, Modifiers::NONE));
        assert_eq!(eventos[2], tecla(Key::A, Modifiers::COMMAND));
    }

    #[test]
    fn navegacao_separa_eventos_seguintes() {
        let mut eventos = vec![
            Event::Text("2".to_string()),
            tecla(Key::Enter, Modifiers::NONE),
            Event::Text("3".to_string()),
        ];
        let (navegacao, resto) = take_navigation(&mut eventos, true, false);
        assert_eq!(navegacao, Some(Navigation::Enter));
        assert_eq!(eventos, vec![Event::Text("2".to_string())]);
        assert_eq!(resto, vec![Event::Text("3".to_string())]);
        let mut setas = vec![tecla(Key::ArrowDown, Modifiers::SHIFT)];
        assert_eq!(
            take_navigation(&mut setas, true, false).0,
            None,
            "Shift+↓ segue o normal"
        );
        let mut fora = vec![tecla(Key::Enter, Modifiers::NONE)];
        assert_eq!(
            take_navigation(&mut fora, false, false).0,
            None,
            "Enter fora de campo"
        );
        let mut esc = vec![tecla(Key::Escape, Modifiers::NONE)];
        assert_eq!(
            take_navigation(&mut esc, false, true).0,
            Some(Navigation::Escape)
        );
    }

    #[test]
    fn lote_de_eventos_respeita_tab_e_enter() {
        let texto = |valor: &str| Event::Text(valor.to_string());
        let mut lote = vec![
            texto("2"),
            tecla(Key::Tab, Modifiers::NONE),
            texto("1"),
            tecla(Key::Enter, Modifiers::NONE),
            texto("5"),
        ];
        let resto = split_batch(&mut lote);
        assert_eq!(lote, vec![texto("2")], "o Tab começa um quadro novo");
        let mut lote = resto;
        let resto = split_batch(&mut lote);
        assert_eq!(lote, vec![tecla(Key::Tab, Modifiers::NONE)]);
        let mut lote = resto;
        let resto = split_batch(&mut lote);
        assert_eq!(
            lote,
            vec![texto("1"), tecla(Key::Enter, Modifiers::NONE)],
            "o Enter termina o quadro"
        );
        let mut lote = resto;
        assert!(split_batch(&mut lote).is_empty());
        assert_eq!(lote, vec![texto("5")]);
    }

    #[test]
    fn metricas_do_layout() {
        assert_eq!(
            metrics(),
            Metrics {
                input_height: 21,
                value_height: 41
            }
        );
    }
}
