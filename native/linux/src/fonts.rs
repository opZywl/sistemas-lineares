use std::sync::{Arc, OnceLock};

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId};
use sistemas_lineares_ui::text::{self, TextMetrics};
use sistemas_lineares_ui::theme::Font;
use sistemas_lineares_ui::thinking::Measure;

pub const REGULAR: &str = "SpaceGrotesk-Regular";
pub const BOLD: &str = "SpaceGrotesk-Bold";
pub const DPI: u32 = 96;

const EXTRA_FALLBACKS: [&str; 2] = ["Ubuntu-Light", "Hack"];

pub fn definitions() -> FontDefinitions {
    let mut definitions = FontDefinitions::default();
    definitions.font_data.insert(
        REGULAR.to_owned(),
        Arc::new(FontData::from_static(text::REGULAR)),
    );
    definitions
        .font_data
        .insert(BOLD.to_owned(), Arc::new(FontData::from_static(text::BOLD)));
    let defaults = definitions
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let chain = |primary: &str| -> Vec<String> {
        let mut names = vec![primary.to_owned()];
        for name in EXTRA_FALLBACKS {
            if definitions.font_data.contains_key(name) {
                names.push(name.to_owned());
            }
        }
        for name in &defaults {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    };
    let regular = chain(REGULAR);
    let bold = chain(BOLD);
    definitions
        .families
        .insert(FontFamily::Proportional, regular);
    definitions
        .families
        .insert(FontFamily::Name(BOLD.into()), bold);
    definitions
}

pub fn family(font: Font) -> FontFamily {
    if font.bold() {
        FontFamily::Name(BOLD.into())
    } else {
        FontFamily::Proportional
    }
}

pub fn pixels(font: Font) -> f32 {
    font.pixels(DPI) as f32
}

pub fn id(font: Font) -> FontId {
    FontId::new(pixels(font), family(font))
}

struct Vertical {
    units: f32,
    ascent: f32,
    descent: f32,
}

fn vertical(data: &[u8]) -> Option<Vertical> {
    let read16 = |offset: usize| {
        data.get(offset..offset + 2)
            .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
    };
    let read32 = |offset: usize| {
        data.get(offset..offset + 4)
            .map(|bytes| u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };
    let tables = usize::from(read16(4)?);
    let mut head = None;
    let mut os2 = None;
    for index in 0..tables {
        let record = 12 + 16 * index;
        let tag = data.get(record..record + 4)?;
        let offset = read32(record + 8)? as usize;
        match tag {
            b"head" => head = Some(offset),
            b"OS/2" => os2 = Some(offset),
            _ => {}
        }
    }
    let units = f32::from(read16(head? + 18)?);
    let os2 = os2?;
    let ascent = f32::from(read16(os2 + 74)?);
    let descent = f32::from(read16(os2 + 76)?);
    (units > 0.0).then_some(Vertical {
        units,
        ascent,
        descent,
    })
}

fn compute(font: Font) -> TextMetrics {
    let pixels = font.pixels(DPI);
    let data = if font.bold() {
        text::BOLD
    } else {
        text::REGULAR
    };
    match vertical(data) {
        Some(vertical) => {
            let scale = pixels as f32 / vertical.units;
            let ascent = (vertical.ascent * scale).round() as i32;
            let descent = (vertical.descent * scale).round() as i32;
            TextMetrics {
                height: ascent + descent,
                ascent,
                descent,
                external_leading: 0,
            }
        }
        None => TextMetrics {
            height: pixels * 3 / 2,
            ascent: pixels * 6 / 5,
            descent: pixels * 3 / 10,
            external_leading: 0,
        },
    }
}

pub fn metrics(font: Font) -> TextMetrics {
    static TABLE: OnceLock<[TextMetrics; Font::COUNT]> = OnceLock::new();
    TABLE.get_or_init(|| Font::ALL.map(compute))[font.index()]
}

pub fn galley(ctx: &egui::Context, text: &str, font: Font, color: Color32) -> Arc<egui::Galley> {
    ctx.fonts_mut(|fonts| fonts.layout_no_wrap(text.to_owned(), id(font), color))
}

pub fn width(ctx: &egui::Context, text: &str, font: Font) -> i32 {
    if text.is_empty() {
        return 0;
    }
    galley(ctx, text, font, Color32::WHITE).size().x.ceil() as i32
}

pub fn baseline(ctx: &egui::Context, font: Font) -> f32 {
    let probe = galley(ctx, "Hg", font, Color32::WHITE);
    probe
        .rows
        .first()
        .and_then(|row| row.glyphs.first())
        .map_or(pixels(font) * 0.8, |glyph| glyph.pos.y)
}

pub struct Measurer<'a>(pub &'a egui::Context);

impl Measure for Measurer<'_> {
    fn width(&self, text: &str, font: Font) -> i32 {
        width(self.0, text, font)
    }

    fn metrics(&self, font: Font) -> TextMetrics {
        metrics(font)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metricas_iguais_as_do_gdi() {
        let esperado = [
            (Font::Small, 18, 14),
            (Font::Body, 20, 16),
            (Font::Medium, 20, 16),
            (Font::Heading, 31, 25),
            (Font::Large, 60, 48),
            (Font::Value, 46, 37),
            (Font::Input, 23, 18),
            (Font::Title, 23, 18),
        ];
        for (font, altura, ascendente) in esperado {
            let metrica = metrics(font);
            assert_eq!(metrica.height, altura, "{font:?}");
            assert_eq!(metrica.ascent, ascendente, "{font:?}");
            assert_eq!(metrica.descent, altura - ascendente, "{font:?}");
        }
    }

    #[test]
    fn familias_com_space_grotesk_e_reserva() {
        let definicoes = definitions();
        let regular = &definicoes.families[&FontFamily::Proportional];
        let negrito = &definicoes.families[&FontFamily::Name(BOLD.into())];
        assert_eq!(regular[0], REGULAR);
        assert_eq!(negrito[0], BOLD);
        for familia in [regular, negrito] {
            assert!(familia.iter().any(|nome| nome == "Hack"));
            assert!(familia.len() > 2);
        }
        assert_eq!(id(Font::Value).size, 32.0);
        assert_eq!(id(Font::Small).size, 12.0);
        assert_eq!(family(Font::Title), FontFamily::Name(BOLD.into()));
        assert_eq!(family(Font::Input), FontFamily::Proportional);
    }
}
