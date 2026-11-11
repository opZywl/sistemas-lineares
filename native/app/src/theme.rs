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

    pub const fn from_colorref(value: u32) -> Color {
        Color::rgb(
            (value & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            ((value >> 16) & 0xFF) as u8,
        )
    }
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
pub const PRIMARY_FOCUS: Color = Color::rgb(163, 216, 191);
pub const CARD_HIGHLIGHT: Color = Color::argb(18, 255, 255, 255);

pub const CARD_RADIUS: f32 = 16.0;
pub const CARD_HIGHLIGHT_RADIUS: f32 = 13.0;
pub const BUTTON_RADIUS: f32 = 10.0;
pub const FOCUS_RADIUS: f32 = 7.0;
pub const FIELD_RADIUS: f32 = 9.0;
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
            primary_focus: PRIMARY_FOCUS,
            link_face: CANVAS,
            link: INK,
            link_muted: MUTED,
            link_focus: ACCENT,
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
}

impl Font {
    pub const ALL: [Font; 7] = [
        Font::Small,
        Font::Body,
        Font::Medium,
        Font::Heading,
        Font::Large,
        Font::Value,
        Font::Input,
    ];

    pub const fn points(self) -> u32 {
        match self {
            Font::Small => 9,
            Font::Body | Font::Medium => 10,
            Font::Heading => 16,
            Font::Large => 31,
            Font::Value => 24,
            Font::Input => 12,
        }
    }

    pub const fn bold(self) -> bool {
        matches!(
            self,
            Font::Medium | Font::Heading | Font::Large | Font::Value
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
        ];
        for color in used {
            assert!(allowed.contains(&color), "{color:?}");
        }
        assert!(palette.card_highlight.is_none());
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
