use std::sync::OnceLock;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::{
    AddFontMemResourceEx, CLIP_DEFAULT_PRECIS, CreateCompatibleDC, CreateFontIndirectW,
    DEFAULT_CHARSET, DEFAULT_QUALITY, DeleteDC, DeleteObject, FF_DONTCARE, FW_BOLD, FW_NORMAL,
    GetTextFaceW, GetTextMetricsW, HDC, HFONT, HGDIOBJ, LOGFONTW, OUT_DEFAULT_PRECIS, SelectObject,
    TEXTMETRICW,
};

use crate::theme::Font;

pub use sistemas_lineares_ui::text::{
    BOLD, FAMILY, REGULAR, TextMetrics, line_spacing_ratio, preferred_height,
};

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

pub fn role_metrics(dpi: u32) -> [TextMetrics; Font::COUNT] {
    let mut result = [TextMetrics::default(); Font::COUNT];
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
    fn fonte_gdi_em_memoria() {
        assert!(register_gdi());
        let font = GdiFont::create(FAMILY, 16, false).unwrap();
        assert_eq!(font.face_name(), FAMILY);
        assert!(font.metrics().height >= 20);
    }
}
