#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 255 }
    }

    pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a }
    }

    pub const fn colorref(self) -> u32 {
        self.r as u32 | (self.g as u32) << 8 | (self.b as u32) << 16
    }

    pub const fn mix(self, other: Color, percent: u8) -> Color {
        let percent = if percent > 100 { 100 } else { percent } as u32;
        Color {
            r: blend(self.r, other.r, percent),
            g: blend(self.g, other.g, percent),
            b: blend(self.b, other.b, percent),
            a: blend(self.a, other.a, percent),
        }
    }

    pub const fn from_colorref(value: u32) -> Color {
        Color::rgb(
            (value & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
        )
    }
}

const fn blend(first: u8, second: u8, percent: u32) -> u8 {
    ((first as u32 * percent + second as u32 * (100 - percent) + 50) / 100) as u8
}

pub const CANVAS: Color = Color::rgb(17, 16, 16);
pub const SURFACE: Color = Color::rgb(22, 21, 21);
pub const FIELD: Color = Color::rgb(29, 28, 28);
pub const INK: Color = Color::rgb(243, 240, 247);
pub const MUTED: Color = Color::rgb(170, 165, 177);
pub const BORDER: Color = Color::rgb(53, 49, 55);
pub const ACCENT: Color = Color::rgb(206, 169, 243);
pub const SOFT: Color = Color::rgb(48, 39, 57);
pub const ERROR: Color = Color::rgb(255, 153, 145);
pub const ACCENT_HOVER: Color = Color::rgb(222, 193, 249);
pub const SOLUTION_STROKE: Color = Color::rgb(222, 192, 249);
pub const DEEP_TEXT: Color = Color::rgb(58, 43, 70);
pub const TILE_FILL: Color = Color::rgb(224, 201, 245);
pub const TILE_STROKE: Color = Color::rgb(186, 148, 219);
pub const DISABLED_FILL: Color = Color::rgb(42, 37, 47);
pub const DISABLED_TEXT: Color = Color::rgb(135, 130, 143);
pub const CARD_HIGHLIGHT: Color = Color::argb(18, 255, 255, 255);
pub const FIELD_HOVER: Color = Color::rgb(84, 78, 90);
pub const FOCUS_GLOW: Color = Color::argb(70, 206, 169, 243);
pub const MINT: Color = Color::rgb(125, 211, 192);
pub const PEACH: Color = Color::rgb(247, 178, 140);
pub const GRAPH_TEXTURE: Color = Color::rgb(50, 46, 55);
pub const GRAPH_GRID: Color = Color::rgb(36, 33, 39);
pub const GRAPH_AXIS: Color = Color::rgb(99, 92, 107);
pub const GRAPH_PANE: Color = Color::rgb(26, 24, 29);
pub const GRAPH_EDGE: Color = Color::rgb(64, 59, 69);

pub const STALE_PERCENT: u8 = 70;

pub const CARD_RADIUS: f32 = 16.0;
pub const CARD_HIGHLIGHT_RADIUS: f32 = 13.0;
pub const BUTTON_RADIUS: f32 = 10.0;
pub const FOCUS_RADIUS: f32 = 7.0;
pub const FIELD_RADIUS: f32 = 9.0;
pub const GRAPH_RADIUS: f32 = 12.0;
pub const ICON_STROKE: f32 = 1.65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemColors {
    pub window: Color,
    pub window_text: Color,
    pub button_face: Color,
    pub button_text: Color,
    pub highlight: Color,
    pub highlight_text: Color,
    pub gray_text: Color,
    pub hot_light: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub high_contrast: bool,
    pub canvas: Color,
    pub surface: Color,
    pub field: Color,
    pub ink: Color,
    pub muted: Color,
    pub border: Color,
    pub accent: Color,
    pub icon: Color,
    pub error: Color,
    pub error_text: Color,
    pub solution_fill: Color,
    pub solution_stroke: Color,
    pub solution_text: Color,
    pub solution_muted: Color,
    pub tile_fill: Color,
    pub tile_stroke: Color,
    pub card_highlight: Option<Color>,
    pub button_face: Color,
    pub button_text: Color,
    pub button_stroke: Color,
    pub quiet_stroke: Option<Color>,
    pub hover_fill: Color,
    pub hover_text: Color,
    pub accent_fill: Color,
    pub accent_hover: Color,
    pub accent_text: Color,
    pub disabled_fill: Color,
    pub disabled_text: Color,
    pub disabled_stroke: Option<Color>,
    pub focus: Color,
    pub primary_focus: Color,
    pub link_face: Color,
    pub link: Color,
    pub link_muted: Color,
    pub link_focus: Color,
    pub operation: Color,
    pub badge_fill: Color,
    pub badge_text: Color,
    pub pivot: Color,
    pub marked_fill: Option<Color>,
    pub marked_stroke: Option<Color>,
    pub field_hover: Color,
    pub focus_glow: Option<Color>,
    pub track_fill: Color,
    pub track_stroke: Color,
    pub keycap_fill: Color,
    pub keycap_stroke: Color,
    pub keycap_text: Color,
    pub footer_link: Color,
    pub footer_link_hover: Color,
    pub cta_fill: Color,
    pub cta_icon: Color,
    pub ghost_stroke: Color,
    pub ghost_text: Color,
    pub ghost_hover: Color,
    pub ghost_hover_text: Color,
    pub ghost_disabled: Color,
    pub graph_fill: Color,
    pub graph_stroke: Color,
    pub graph_texture: Color,
    pub graph_grid: Color,
    pub graph_axis: Color,
    pub graph_label: Color,
    pub graph_pane: Color,
    pub graph_edge: Color,
    pub graph_rows: [Color; 3],
    pub graph_meet: Color,
    pub graph_guide: Color,
    pub graph_set: Color,
    pub graph_halo: Color,
    pub graph_point: Color,
}

impl Palette {
    pub const fn dark() -> Palette {
        Palette {
            high_contrast: false,
            canvas: CANVAS,
            surface: SURFACE,
            field: FIELD,
            ink: INK,
            muted: MUTED,
            border: BORDER,
            accent: ACCENT,
            icon: ACCENT,
            error: ERROR,
            error_text: ERROR,
            solution_fill: ACCENT,
            solution_stroke: SOLUTION_STROKE,
            solution_text: CANVAS,
            solution_muted: DEEP_TEXT,
            tile_fill: TILE_FILL,
            tile_stroke: TILE_STROKE,
            card_highlight: Some(CARD_HIGHLIGHT),
            button_face: SURFACE,
            button_text: INK,
            button_stroke: BORDER,
            quiet_stroke: None,
            hover_fill: SOFT,
            hover_text: INK,
            accent_fill: ACCENT,
            accent_hover: ACCENT_HOVER,
            accent_text: CANVAS,
            disabled_fill: DISABLED_FILL,
            disabled_text: DISABLED_TEXT,
            disabled_stroke: None,
            focus: ACCENT,
            primary_focus: CANVAS,
            link_face: CANVAS,
            link: INK,
            link_muted: MUTED,
            link_focus: ACCENT,
            operation: ACCENT,
            badge_fill: ACCENT,
            badge_text: CANVAS,
            pivot: ACCENT,
            marked_fill: Some(SOFT),
            marked_stroke: None,
            field_hover: FIELD_HOVER,
            focus_glow: Some(FOCUS_GLOW),
            track_fill: FIELD,
            track_stroke: BORDER,
            keycap_fill: FIELD,
            keycap_stroke: BORDER,
            keycap_text: INK,
            footer_link: INK,
            footer_link_hover: ACCENT,
            cta_fill: CANVAS,
            cta_icon: ACCENT,
            ghost_stroke: CANVAS,
            ghost_text: CANVAS,
            ghost_hover: TILE_FILL,
            ghost_hover_text: CANVAS,
            ghost_disabled: CANVAS.mix(ACCENT, 40),
            graph_fill: CANVAS,
            graph_stroke: BORDER,
            graph_texture: GRAPH_TEXTURE,
            graph_grid: GRAPH_GRID,
            graph_axis: GRAPH_AXIS,
            graph_label: MUTED,
            graph_pane: GRAPH_PANE,
            graph_edge: GRAPH_EDGE,
            graph_rows: [ACCENT, MINT, PEACH],
            graph_meet: INK,
            graph_guide: MUTED,
            graph_set: INK,
            graph_halo: ACCENT,
            graph_point: INK,
        }
    }

    pub const fn high_contrast(system: SystemColors) -> Palette {
        Palette {
            high_contrast: true,
            canvas: system.window,
            surface: system.window,
            field: system.window,
            ink: system.window_text,
            muted: system.window_text,
            border: system.window_text,
            accent: system.highlight,
            icon: system.window_text,
            error: system.highlight,
            error_text: system.window_text,
            solution_fill: system.window,
            solution_stroke: system.window_text,
            solution_text: system.window_text,
            solution_muted: system.window_text,
            tile_fill: system.window,
            tile_stroke: system.window_text,
            card_highlight: None,
            button_face: system.button_face,
            button_text: system.button_text,
            button_stroke: system.button_text,
            quiet_stroke: Some(system.button_text),
            hover_fill: system.highlight,
            hover_text: system.highlight_text,
            accent_fill: system.highlight,
            accent_hover: system.highlight,
            accent_text: system.highlight_text,
            disabled_fill: system.button_face,
            disabled_text: system.gray_text,
            disabled_stroke: Some(system.gray_text),
            focus: system.button_text,
            primary_focus: system.highlight_text,
            link_face: system.window,
            link: system.hot_light,
            link_muted: system.hot_light,
            link_focus: system.window_text,
            operation: system.window_text,
            badge_fill: system.highlight,
            badge_text: system.highlight_text,
            pivot: system.highlight,
            marked_fill: None,
            marked_stroke: Some(system.window_text),
            field_hover: system.highlight,
            focus_glow: None,
            track_fill: system.window,
            track_stroke: system.window_text,
            keycap_fill: system.window,
            keycap_stroke: system.window_text,
            keycap_text: system.window_text,
            footer_link: system.hot_light,
            footer_link_hover: system.hot_light,
            cta_fill: system.button_face,
            cta_icon: system.button_text,
            ghost_stroke: system.window_text,
            ghost_text: system.window_text,
            ghost_hover: system.highlight,
            ghost_hover_text: system.highlight_text,
            ghost_disabled: system.gray_text,
            graph_fill: system.window,
            graph_stroke: system.window_text,
            graph_texture: system.gray_text,
            graph_grid: system.gray_text,
            graph_axis: system.window_text,
            graph_label: system.window_text,
            graph_pane: system.window,
            graph_edge: system.window_text,
            graph_rows: [system.highlight, system.hot_light, system.window_text],
            graph_meet: system.window_text,
            graph_guide: system.window_text,
            graph_set: system.window_text,
            graph_halo: system.highlight,
            graph_point: system.window_text,
        }
    }
}

impl Palette {
    pub const fn stale(&self, value: Color, back: Color) -> Color {
        if self.high_contrast {
            self.disabled_text
        } else {
            value.mix(back, STALE_PERCENT)
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::dark()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Font {
    Small,
    Body,
    Medium,
    Heading,
    Large,
    Value,
    Input,
    Title,
}

impl Font {
    pub const COUNT: usize = 8;

    pub const ALL: [Font; Font::COUNT] = [
        Font::Small,
        Font::Body,
        Font::Medium,
        Font::Heading,
        Font::Large,
        Font::Value,
        Font::Input,
        Font::Title,
    ];

    pub const fn points(self) -> u32 {
        match self {
            Font::Small => 9,
            Font::Body | Font::Medium => 10,
            Font::Heading => 16,
            Font::Large => 31,
            Font::Value => 24,
            Font::Input | Font::Title => 12,
        }
    }

    pub const fn bold(self) -> bool {
        matches!(
            self,
            Font::Medium | Font::Heading | Font::Large | Font::Value | Font::Title
        )
    }

    pub const fn index(self) -> usize {
        match self {
            Font::Small => 0,
            Font::Body => 1,
            Font::Medium => 2,
            Font::Heading => 3,
            Font::Large => 4,
            Font::Value => 5,
            Font::Input => 6,
            Font::Title => 7,
        }
    }

    pub const fn pixels(self, dpi: u32) -> i32 {
        (self.points() * dpi).div_ceil(72) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mistura_e_esmaecimento() {
        let branco = Color::rgb(255, 255, 255);
        let preto = Color::rgb(0, 0, 0);
        assert_eq!(branco.mix(preto, 100), branco);
        assert_eq!(branco.mix(preto, 0), preto);
        assert_eq!(branco.mix(preto, 70), Color::rgb(179, 179, 179));
        assert_eq!(
            Color::rgb(10, 20, 30).mix(Color::rgb(30, 40, 50), 50),
            Color::rgb(20, 30, 40)
        );
        let escuro = Palette::dark();
        assert_eq!(escuro.stale(INK, CANVAS), INK.mix(CANVAS, STALE_PERCENT));
        let sistema = SystemColors {
            window: Color::rgb(0, 0, 0),
            window_text: Color::rgb(255, 255, 255),
            button_face: Color::rgb(0, 0, 0),
            button_text: Color::rgb(255, 255, 255),
            highlight: Color::rgb(26, 235, 255),
            highlight_text: Color::rgb(0, 0, 0),
            gray_text: Color::rgb(63, 242, 63),
            hot_light: Color::rgb(255, 255, 0),
        };
        let contraste = Palette::high_contrast(sistema);
        assert_eq!(
            contraste.stale(sistema.window_text, sistema.window),
            sistema.gray_text
        );
        assert_eq!(escuro.primary_focus, CANVAS);
    }

    #[test]
    fn tamanhos_como_textrenderer() {
        assert_eq!(Font::Small.pixels(96), 12);
        assert_eq!(Font::Body.pixels(96), 14);
        assert_eq!(Font::Heading.pixels(96), 22);
        assert_eq!(Font::Large.pixels(96), 42);
        assert_eq!(Font::Value.pixels(96), 32);
        assert_eq!(Font::Input.pixels(96), 16);
        assert_eq!(Font::Input.pixels(144), 24);
        assert_eq!(CANVAS.colorref(), 0x00101011);
        assert_eq!(Color::from_colorref(ACCENT.colorref()), ACCENT);
    }

    #[test]
    fn alto_contraste_usa_somente_cores_do_sistema() {
        let system = SystemColors {
            window: Color::rgb(0, 0, 0),
            window_text: Color::rgb(255, 255, 255),
            button_face: Color::rgb(1, 1, 1),
            button_text: Color::rgb(254, 254, 254),
            highlight: Color::rgb(26, 235, 255),
            highlight_text: Color::rgb(2, 2, 2),
            gray_text: Color::rgb(63, 242, 63),
            hot_light: Color::rgb(255, 255, 0),
        };
        let palette = Palette::high_contrast(system);
        let allowed = [
            system.window,
            system.window_text,
            system.button_face,
            system.button_text,
            system.highlight,
            system.highlight_text,
            system.gray_text,
            system.hot_light,
        ];
        let used = [
            palette.canvas,
            palette.surface,
            palette.field,
            palette.ink,
            palette.muted,
            palette.border,
            palette.accent,
            palette.icon,
            palette.error,
            palette.error_text,
            palette.solution_fill,
            palette.solution_stroke,
            palette.solution_text,
            palette.solution_muted,
            palette.tile_fill,
            palette.tile_stroke,
            palette.button_face,
            palette.button_text,
            palette.button_stroke,
            palette.hover_fill,
            palette.hover_text,
            palette.accent_fill,
            palette.accent_hover,
            palette.accent_text,
            palette.disabled_fill,
            palette.disabled_text,
            palette.focus,
            palette.primary_focus,
            palette.link_face,
            palette.link,
            palette.link_muted,
            palette.link_focus,
            palette.operation,
            palette.badge_fill,
            palette.badge_text,
            palette.pivot,
            palette.field_hover,
            palette.track_fill,
            palette.track_stroke,
            palette.keycap_fill,
            palette.keycap_stroke,
            palette.keycap_text,
            palette.footer_link,
            palette.footer_link_hover,
            palette.cta_fill,
            palette.cta_icon,
            palette.ghost_stroke,
            palette.ghost_text,
            palette.ghost_hover,
            palette.ghost_hover_text,
            palette.ghost_disabled,
            palette.graph_fill,
            palette.graph_stroke,
            palette.graph_texture,
            palette.graph_grid,
            palette.graph_axis,
            palette.graph_label,
            palette.graph_pane,
            palette.graph_edge,
            palette.graph_rows[0],
            palette.graph_rows[1],
            palette.graph_rows[2],
            palette.graph_meet,
            palette.graph_guide,
            palette.graph_set,
            palette.graph_halo,
            palette.graph_point,
        ];
        for color in used {
            assert!(allowed.contains(&color), "{color:?}");
        }
        assert!(palette.card_highlight.is_none());
        assert!(palette.marked_fill.is_none());
        assert_eq!(palette.marked_stroke, Some(system.window_text));
        assert_eq!(palette.canvas, system.window);
        assert_eq!(palette.ink, system.window_text);
        assert_eq!(palette.accent_fill, system.highlight);
        assert_eq!(palette.accent_text, system.highlight_text);
        assert_eq!(palette.disabled_text, system.gray_text);
        assert_eq!(palette.link, system.hot_light);
        assert_eq!(palette.border, system.window_text);
        assert!(!Palette::dark().high_contrast);
        assert_eq!(Palette::dark().solution_fill, ACCENT);
    }
}
