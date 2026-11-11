use std::sync::OnceLock;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, CLIP_DEFAULT_PRECIS, CreateCompatibleDC, CreateFontIndirectW,
    DEFAULT_CHARSET, DEFAULT_QUALITY, DeleteDC, DeleteObject, FF_DONTCARE, FW_BOLD, FW_NORMAL,
    GetTextFaceW, GetTextMetricsW, HDC, HFONT, HGDIOBJ, LOGFONTW, OUT_DEFAULT_PRECIS, SelectObject,
    TEXTMETRICW,
};

use crate::theme::Font;

pub static REGULAR: &[u8] = include_bytes!("../../../assets/SpaceGrotesk-Regular.ttf");
pub static BOLD: &[u8] = include_bytes!("../../../assets/SpaceGrotesk-Bold.ttf");
pub const FAMILY: &str = "Space Grotesk";
pub const FALLBACK: &str = "Segoe UI";

struct Registration(Vec<isize>);

static GDI_FONTS: OnceLock<Registration> = OnceLock::new();

pub fn register_gdi() -> bool {
    let registration = GDI_FONTS.get_or_init(|| {
        let mut handles = Vec::new();
        for data in [REGULAR, BOLD] {
            let mut count = 0u32;
            let handle: HANDLE = unsafe {
                AddFontMemResourceEx(
                    data.as_ptr().cast(),
                    data.len() as u32,
                    None,
                    (&raw mut count).cast_const(),
                )
            };
            if !handle.is_invalid() && count > 0 {
                handles.push(handle.0 as isize);
            }
        }
        Registration(handles)
    });
    registration.0.len() == 2
}

pub fn gdi_face() -> &'static str {
    if register_gdi() { FAMILY } else { FALLBACK }
}

pub fn line_spacing_ratio(data: &[u8]) -> Option<f32> {
    let read16 = |offset: usize| {
        data.get(offset..offset + 2)
            .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
    };
    let read32 = |offset: usize| {
        data.get(offset..offset + 4)
            .map(|bytes| u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };
    let tables = read16(4)? as usize;
    let mut head = None;
    let mut hhea = None;
    for index in 0..tables {
        let record = 12 + 16 * index;
        let tag = data.get(record..record + 4)?;
        let offset = read32(record + 8)? as usize;
        match tag {
            b"head" => head = Some(offset),
            b"hhea" => hhea = Some(offset),
            _ => {}
        }
    }
    let units = read16(head? + 18)? as f32;
    let hhea = hhea?;
    let ascender = read16(hhea + 4)? as i16 as f32;
    let descender = read16(hhea + 6)? as i16 as f32;
    let gap = read16(hhea + 8)? as i16 as f32;
    (units > 0.0).then(|| (ascender - descender + gap) / units)
}

pub fn preferred_height(pixels: i32) -> Option<i32> {
    line_spacing_ratio(REGULAR).map(|ratio| (pixels as f32 * ratio).ceil() as i32)
}

pub struct GdiFont(HFONT);

impl GdiFont {
    pub fn create(face: &str, pixels: i32, bold: bool) -> Option<GdiFont> {
        let mut logfont = LOGFONTW {
            lfHeight: -pixels.max(1),
            lfWeight: if bold {
                FW_BOLD.0 as i32
            } else {
                FW_NORMAL.0 as i32
            },
            lfCharSet: DEFAULT_CHARSET,
            lfOutPrecision: OUT_DEFAULT_PRECIS,
            lfClipPrecision: CLIP_DEFAULT_PRECIS,
            lfQuality: DEFAULT_QUALITY,
            lfPitchAndFamily: FF_DONTCARE.0,
            ..Default::default()
        };
        for (slot, unit) in logfont
            .lfFaceName
            .iter_mut()
            .zip(face.encode_utf16().take(31))
        {
            *slot = unit;
        }
        let font = unsafe { CreateFontIndirectW(&logfont) };
        (!font.is_invalid()).then_some(GdiFont(font))
    }

    pub fn for_role(role: Font, dpi: u32) -> Option<GdiFont> {
        GdiFont::create(gdi_face(), role.pixels(dpi), role.bold())
            .or_else(|| GdiFont::create(FALLBACK, role.pixels(dpi), role.bold()))
    }

    pub fn handle(&self) -> HFONT {
        self.0
    }

    pub fn metrics(&self) -> TextMetrics {
        let mut result = TextMetrics::default();
        unsafe {
            let dc: HDC = CreateCompatibleDC(None);
            if dc.is_invalid() {
                return result;
            }
            let previous = SelectObject(dc, HGDIOBJ(self.0.0));
            let mut metric = TEXTMETRICW::default();
            if GetTextMetricsW(dc, &mut metric).as_bool() {
                result = TextMetrics {
                    height: metric.tmHeight,
                    ascent: metric.tmAscent,
                    descent: metric.tmDescent,
                    external_leading: metric.tmExternalLeading,
                };
            }
            SelectObject(dc, previous);
            let _ = DeleteDC(dc);
        }
        result
    }

    pub fn face_name(&self) -> String {
        let mut buffer = [0u16; 64];
        unsafe {
            let dc: HDC = CreateCompatibleDC(None);
            if dc.is_invalid() {
                return String::new();
            }
            let previous = SelectObject(dc, HGDIOBJ(self.0.0));
            let length = GetTextFaceW(dc, Some(&mut buffer));
            SelectObject(dc, previous);
            let _ = DeleteDC(dc);
            String::from_utf16_lossy(&buffer[..(length.max(1) as usize - 1).min(buffer.len())])
        }
    }
}

impl Drop for GdiFont {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ(self.0.0));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextMetrics {
    pub height: i32,
    pub ascent: i32,
    pub descent: i32,
    pub external_leading: i32,
}

pub fn role_metrics(dpi: u32) -> [TextMetrics; 7] {
    let mut result = [TextMetrics::default(); 7];
    for role in Font::ALL {
        let pixels = role.pixels(dpi);
        let metrics = GdiFont::for_role(role, dpi)
            .map(|font| font.metrics())
            .filter(|metrics| metrics.height > 0);
        result[role.index()] = metrics.unwrap_or(TextMetrics {
            height: pixels * 3 / 2,
            ascent: pixels * 6 / 5,
            descent: pixels * 3 / 10,
            external_leading: 0,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn espacamento_da_space_grotesk() {
        let ratio = line_spacing_ratio(REGULAR).unwrap();
        assert!((ratio - 1.276).abs() < 1e-4);
        assert_eq!(preferred_height(16), Some(21));
        assert_eq!(preferred_height(32), Some(41));
    }

    #[test]
    fn fonte_gdi_em_memoria() {
        assert!(register_gdi());
        let font = GdiFont::create(FAMILY, 16, false).unwrap();
        assert_eq!(font.face_name(), FAMILY);
        assert!(font.metrics().height >= 20);
    }
}
