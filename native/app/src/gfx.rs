use std::cell::RefCell;
use std::collections::HashMap;

use windows::Win32::Foundation::{
    COLORREF, D2DERR_RECREATE_TARGET, E_FAIL, GENERIC_WRITE, POINT, RECT,
};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D_SIZE_F, D2D1_ALPHA_MODE, D2D1_ALPHA_MODE_IGNORE, D2D1_ALPHA_MODE_PREMULTIPLIED,
    D2D1_COLOR_F, D2D1_FIGURE_BEGIN_HOLLOW, D2D1_FIGURE_END_CLOSED, D2D1_FIGURE_END_OPEN,
    D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_ANTIALIAS_MODE_ALIASED, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE, D2D1_ARC_SEGMENT,
    D2D1_ARC_SIZE_SMALL, D2D1_CAP_STYLE_FLAT, D2D1_CAP_STYLE_ROUND, D2D1_DASH_STYLE_CUSTOM,
    D2D1_DASH_STYLE_SOLID, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED,
    D2D1_FEATURE_LEVEL_DEFAULT, D2D1_LINE_JOIN_ROUND, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_SOFTWARE, D2D1_RENDER_TARGET_USAGE_NONE, D2D1_ROUNDED_RECT,
    D2D1_STROKE_STYLE_PROPERTIES, D2D1_SWEEP_DIRECTION_CLOCKWISE, D2D1CreateFactory,
    ID2D1DCRenderTarget, ID2D1Factory, ID2D1GeometrySink, ID2D1PathGeometry, ID2D1RenderTarget,
    ID2D1SolidColorBrush, ID2D1StrokeStyle,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_BOLD, DWRITE_FONT_WEIGHT_NORMAL, DWRITE_LINE_SPACING_METHOD_UNIFORM,
    DWRITE_TEXT_METRICS, DWRITE_WORD_WRAPPING_NO_WRAP, DWriteCreateFactory, IDWriteFactory,
    IDWriteFactory5, IDWriteFontCollection, IDWriteInMemoryFontFileLoader, IDWriteTextFormat,
    IDWriteTextLayout,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::{
    BS_SOLID, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreatePen, CreateSolidBrush,
    DT_CALCRECT, DT_NOCLIP, DT_NOPREFIX, DT_SINGLELINE, DeleteDC, DeleteObject, DrawTextW,
    ETO_CLIPPED, Ellipse, ExtCreatePen, ExtTextOutW, FillRect, GetStockObject, HDC, HGDIOBJ,
    LOGBRUSH, LineTo, MoveToEx, NULL_BRUSH, PEN_STYLE, PS_DOT, PS_ENDCAP_ROUND, PS_GEOMETRIC,
    PS_JOIN_ROUND, PS_SOLID, Polyline, RoundRect, SRCCOPY, SelectObject, SetBkMode, SetTextColor,
    SetViewportOrgEx, TRANSPARENT,
};
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_ContainerFormatPng, GUID_WICPixelFormat32bppBGRA,
    GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory, WICBitmapCacheOnLoad,
    WICBitmapEncoderNoCache,
};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::core::{BOOL, HSTRING, IUnknown, Interface, Result};
use windows_numerics::{Matrix3x2, Vector2};

use crate::fonts::{self, GdiFont, TextMetrics};
use crate::icons::{Icon, Shape, VIEWBOX};
use crate::layout::Rect;
use crate::theme::{self, Color, Font};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintStatus {
    Done,
    Recreate,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
}

impl Frame {
    pub const fn new(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Frame {
        Frame {
            x,
            y,
            width,
            height,
            radius,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run<'a> {
    pub text: &'a str,
    pub color: Color,
}

pub fn color(value: Color) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: f32::from(value.r) / 255.0,
        g: f32::from(value.g) / 255.0,
        b: f32::from(value.b) / 255.0,
        a: f32::from(value.a) / 255.0,
    }
}

fn point(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}

pub fn margins_for(metric: TextMetrics) -> (i32, i32) {
    let overhang = metric.height.max(0) as f32 / 6.0;
    (overhang.ceil() as i32, (overhang * 1.5).ceil() as i32)
}

fn wrap(paragraph: &str, available: i32, measure: &dyn Fn(&str) -> i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in paragraph.split(' ') {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if measure(&candidate) <= available {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        let mut piece = String::new();
        for character in word.chars() {
            let mut next = piece.clone();
            next.push(character);
            if !piece.is_empty() && measure(&next) > available {
                lines.push(std::mem::take(&mut piece));
                piece.push(character);
            } else {
                piece = next;
            }
        }
        current = piece;
    }
    lines.push(current);
    lines
}

struct PlacedLine {
    text: String,
    x: i32,
    y: i32,
}

fn plan_text(
    text: &str,
    rect: Rect,
    align: Align,
    valign: VAlign,
    metric: TextMetrics,
    measure: &dyn Fn(&str) -> i32,
) -> Vec<PlacedLine> {
    let line_height = metric.height.max(1);
    let (left_margin, right_margin) = margins_for(metric);
    let left = rect.x + left_margin;
    let right = rect.right() - right_margin;
    let available = (right - left).max(1);
    let paragraphs: Vec<&str> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let wrapping = paragraphs.iter().any(|line| measure(line) > available);
    let lines: Vec<String> = if wrapping {
        paragraphs
            .iter()
            .flat_map(|line| wrap(line, available, measure))
            .collect()
    } else {
        paragraphs.iter().map(|line| line.to_string()).collect()
    };
    let total = lines.len() as i32 * line_height;
    let top = match valign {
        VAlign::Top => rect.y,
        VAlign::Center if total > rect.h => rect.y,
        VAlign::Center => rect.y + rect.h / 2 - total / 2,
    };
    let mut placed = Vec::with_capacity(lines.len());
    for (index, line) in lines.into_iter().enumerate() {
        let y = top + index as i32 * line_height;
        if wrapping && index > 0 && y + line_height > rect.bottom() {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let width = measure(&line);
        let x = match align {
            Align::Left => left,
            Align::Center => (left + right - width) / 2,
            Align::Right => right - width,
        };
        placed.push(PlacedLine { text: line, x, y });
    }
    placed
}

pub fn gdi_extent(font: &GdiFont, text: &str) -> Option<i32> {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    if wide.is_empty() {
        return Some(0);
    }
    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return None;
        }
        let previous = SelectObject(dc, HGDIOBJ(font.handle().0));
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 1_000_000,
            bottom: 10_000,
        };
        let height = DrawTextW(
            dc,
            &mut wide,
            &mut rect,
            DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX | DT_NOCLIP,
        );
        SelectObject(dc, previous);
        let _ = DeleteDC(dc);
        (height != 0).then_some(rect.right - rect.left)
    }
}

struct CachedText {
    layout: IDWriteTextLayout,
    width: i32,
}

pub struct Engine {
    factory: ID2D1Factory,
    dwrite: IDWriteFactory,
    loader: Option<(IDWriteFactory5, IDWriteInMemoryFontFileLoader)>,
    collection: Option<IDWriteFontCollection>,
    family: HSTRING,
    gdi_fonts: Vec<Option<GdiFont>>,
    round: ID2D1StrokeStyle,
    dotted: ID2D1StrokeStyle,
    icons: Vec<ID2D1PathGeometry>,
    dpi: u32,
    formats: Vec<IDWriteTextFormat>,
    metrics: [TextMetrics; 7],
    target: Option<ID2D1DCRenderTarget>,
    brush: Option<ID2D1SolidColorBrush>,
    cache: RefCell<HashMap<(Font, String), CachedText>>,
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.cache.get_mut().clear();
        self.formats.clear();
        self.collection = None;
        if let Some((factory, loader)) = self.loader.take() {
            unsafe {
                let _ = factory.UnregisterFontFileLoader(&loader);
            }
        }
    }
}

fn load_collection(
    dwrite: &IDWriteFactory,
) -> Result<(
    IDWriteFontCollection,
    IDWriteFactory5,
    IDWriteInMemoryFontFileLoader,
)> {
    let factory: IDWriteFactory5 = dwrite.cast()?;
    unsafe {
        let loader = factory.CreateInMemoryFontFileLoader()?;
        factory.RegisterFontFileLoader(&loader)?;
        let built = (|| -> Result<IDWriteFontCollection> {
            let builder = factory.CreateFontSetBuilder()?;
            for data in [fonts::REGULAR, fonts::BOLD] {
                let file = loader.CreateInMemoryFontFileReference(
                    &factory,
                    data.as_ptr().cast(),
                    data.len() as u32,
                    None::<&IUnknown>,
                )?;
                builder.AddFontFile(&file)?;
            }
            let set = builder.CreateFontSet()?;
            let collection = factory.CreateFontCollectionFromFontSet(&set)?;
            let mut index = 0u32;
            let mut exists = BOOL(0);
            collection.FindFamilyName(&HSTRING::from(fonts::FAMILY), &mut index, &mut exists)?;
            if !exists.as_bool() {
                return Err(E_FAIL.into());
            }
            collection.cast()
        })();
        match built {
            Ok(collection) => Ok((collection, factory, loader)),
            Err(error) => {
                let _ = factory.UnregisterFontFileLoader(&loader);
                Err(error)
            }
        }
    }
}

fn icon_geometry(factory: &ID2D1Factory, icon: Icon) -> Result<ID2D1PathGeometry> {
    unsafe {
        let geometry = factory.CreatePathGeometry()?;
        let sink: ID2D1GeometrySink = geometry.Open()?;
        for shape in icon.shapes() {
            add_shape(&sink, &shape);
        }
        sink.Close()?;
        Ok(geometry)
    }
}

unsafe fn add_rounded(
    sink: &ID2D1GeometrySink,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radius: f32,
) {
    let diameter = (radius * 2.0).min(width).min(height);
    let r = diameter / 2.0;
    let arc = |px: f32, py: f32| D2D1_ARC_SEGMENT {
        point: point(px, py),
        size: D2D_SIZE_F {
            width: r,
            height: r,
        },
        rotationAngle: 0.0,
        sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
        arcSize: D2D1_ARC_SIZE_SMALL,
    };
    unsafe {
        sink.BeginFigure(point(x, y + r), D2D1_FIGURE_BEGIN_HOLLOW);
        if r > 0.0 {
            sink.AddArc(&arc(x + r, y));
        }
        sink.AddLine(point(x + width - r, y));
        if r > 0.0 {
            sink.AddArc(&arc(x + width, y + r));
        }
        sink.AddLine(point(x + width, y + height - r));
        if r > 0.0 {
            sink.AddArc(&arc(x + width - r, y + height));
        }
        sink.AddLine(point(x + r, y + height));
        if r > 0.0 {
            sink.AddArc(&arc(x, y + height - r));
        }
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
    }
}

fn add_shape(sink: &ID2D1GeometrySink, shape: &Shape) {
    unsafe {
        match shape {
            Shape::Line { x1, y1, x2, y2 } => {
                sink.BeginFigure(point(*x1, *y1), D2D1_FIGURE_BEGIN_HOLLOW);
                sink.AddLine(point(*x2, *y2));
                sink.EndFigure(D2D1_FIGURE_END_OPEN);
            }
            Shape::Polyline(points) | Shape::Polygon(points) => {
                let Some(first) = points.first() else {
                    return;
                };
                sink.BeginFigure(point(first.0, first.1), D2D1_FIGURE_BEGIN_HOLLOW);
                for next in &points[1..] {
                    sink.AddLine(point(next.0, next.1));
                }
                sink.EndFigure(if matches!(shape, Shape::Polygon(_)) {
                    D2D1_FIGURE_END_CLOSED
                } else {
                    D2D1_FIGURE_END_OPEN
                });
            }
            Shape::Rect {
                x,
                y,
                width,
                height,
                radius,
            } => add_rounded(sink, *x, *y, *width, *height, radius.max(0.0)),
            Shape::Circle { cx, cy, r } => {
                let arc = |px: f32, py: f32| D2D1_ARC_SEGMENT {
                    point: point(px, py),
                    size: D2D_SIZE_F {
                        width: *r,
                        height: *r,
                    },
                    rotationAngle: 0.0,
                    sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                    arcSize: D2D1_ARC_SIZE_SMALL,
                };
                sink.BeginFigure(point(cx - r, *cy), D2D1_FIGURE_BEGIN_HOLLOW);
                sink.AddArc(&arc(*cx, cy - r));
                sink.AddArc(&arc(cx + r, *cy));
                sink.AddArc(&arc(*cx, cy + r));
                sink.AddArc(&arc(cx - r, *cy));
                sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            }
        }
    }
}

fn target_properties(alpha: D2D1_ALPHA_MODE) -> D2D1_RENDER_TARGET_PROPERTIES {
    D2D1_RENDER_TARGET_PROPERTIES {
        r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            alphaMode: alpha,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        usage: D2D1_RENDER_TARGET_USAGE_NONE,
        minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
    }
}

impl Engine {
    pub fn new(dpi: u32) -> Result<Engine> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let dwrite: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let (collection, loader, family) = match load_collection(&dwrite) {
                Ok((collection, factory5, loader)) => {
                    (Some(collection), Some((factory5, loader)), fonts::FAMILY)
                }
                Err(_) => (None, None, fonts::FALLBACK),
            };
            let round = factory.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES {
                    startCap: D2D1_CAP_STYLE_ROUND,
                    endCap: D2D1_CAP_STYLE_ROUND,
                    dashCap: D2D1_CAP_STYLE_ROUND,
                    lineJoin: D2D1_LINE_JOIN_ROUND,
                    miterLimit: 10.0,
                    dashStyle: D2D1_DASH_STYLE_SOLID,
                    dashOffset: 0.0,
                },
                None,
            )?;
            let dotted = factory.CreateStrokeStyle(
                &D2D1_STROKE_STYLE_PROPERTIES {
                    startCap: D2D1_CAP_STYLE_FLAT,
                    endCap: D2D1_CAP_STYLE_FLAT,
                    dashCap: D2D1_CAP_STYLE_FLAT,
                    lineJoin: D2D1_LINE_JOIN_ROUND,
                    miterLimit: 10.0,
                    dashStyle: D2D1_DASH_STYLE_CUSTOM,
                    dashOffset: 0.0,
                },
                Some(&[1.0, 1.0]),
            )?;
            let mut icons = Vec::with_capacity(Icon::ALL.len());
            for icon in Icon::ALL {
                icons.push(icon_geometry(&factory, icon)?);
            }
            let mut engine = Engine {
                factory,
                dwrite,
                loader,
                collection,
                family: HSTRING::from(family),
                gdi_fonts: Vec::new(),
                round,
                dotted,
                icons,
                dpi: 0,
                formats: Vec::new(),
                metrics: [TextMetrics::default(); 7],
                target: None,
                brush: None,
                cache: RefCell::new(HashMap::new()),
            };
            engine.set_dpi(dpi)?;
            Ok(engine)
        }
    }

    pub fn family(&self) -> String {
        self.family.to_string()
    }

    pub fn uses_embedded_font(&self) -> bool {
        self.collection.is_some()
    }

    pub fn dpi(&self) -> u32 {
        self.dpi
    }

    pub fn set_dpi(&mut self, dpi: u32) -> Result<()> {
        let dpi = dpi.max(48);
        if dpi == self.dpi && !self.formats.is_empty() {
            return Ok(());
        }
        let metrics = fonts::role_metrics(dpi);
        let mut formats = Vec::with_capacity(Font::ALL.len());
        unsafe {
            for role in Font::ALL {
                let format = self.dwrite.CreateTextFormat(
                    &self.family,
                    self.collection.as_ref(),
                    if role.bold() {
                        DWRITE_FONT_WEIGHT_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    role.pixels(dpi) as f32,
                    &HSTRING::from("pt-BR"),
                )?;
                let metric = metrics[role.index()];
                format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
                format.SetLineSpacing(
                    DWRITE_LINE_SPACING_METHOD_UNIFORM,
                    metric.height as f32,
                    metric.ascent as f32,
                )?;
                formats.push(format);
            }
        }
        if let Ok(mut cache) = self.cache.try_borrow_mut() {
            cache.clear();
        }
        self.formats = formats;
        self.metrics = metrics;
        self.gdi_fonts = Font::ALL
            .iter()
            .map(|role| GdiFont::for_role(*role, dpi))
            .collect();
        self.dpi = dpi;
        Ok(())
    }

    pub fn text_metrics(&self, font: Font) -> TextMetrics {
        self.metrics[font.index()]
    }

    fn layout(&self, text: &str, font: Font) -> Option<(IDWriteTextLayout, i32)> {
        let key = (font, text.to_string());
        if let Ok(cache) = self.cache.try_borrow() {
            if let Some(cached) = cache.get(&key) {
                return Some((cached.layout.clone(), cached.width));
            }
        }
        let format = self.formats.get(font.index())?;
        let wide: Vec<u16> = text.encode_utf16().collect();
        let layout = unsafe {
            self.dwrite.CreateGdiCompatibleTextLayout(
                &wide, format, 100_000.0, 100_000.0, 1.0, None, false,
            )
        }
        .ok()?;
        let width = match self
            .gdi_fonts
            .get(font.index())
            .and_then(Option::as_ref)
            .and_then(|gdi_font| gdi_extent(gdi_font, text))
        {
            Some(width) => width,
            None => {
                let mut metrics = DWRITE_TEXT_METRICS::default();
                unsafe { layout.GetMetrics(&mut metrics) }.ok()?;
                metrics.widthIncludingTrailingWhitespace.round() as i32
            }
        };
        if let Ok(mut cache) = self.cache.try_borrow_mut() {
            if cache.len() > 512 {
                cache.clear();
            }
            cache.insert(
                key,
                CachedText {
                    layout: layout.clone(),
                    width,
                },
            );
        }
        Some((layout, width))
    }

    pub fn measure(&self, text: &str, font: Font) -> i32 {
        self.layout(text, font).map_or(0, |(_, width)| width)
    }

    pub fn margins(&self, font: Font) -> (i32, i32) {
        margins_for(self.metrics[font.index()])
    }

    pub fn measure_padded(&self, text: &str, font: Font) -> i32 {
        let (left, right) = self.margins(font);
        self.measure(text, font) + left + right
    }

    fn ensure_target(&mut self) -> Option<(ID2D1DCRenderTarget, ID2D1SolidColorBrush)> {
        if self.target.is_none() || self.brush.is_none() {
            unsafe {
                let target = self
                    .factory
                    .CreateDCRenderTarget(&target_properties(D2D1_ALPHA_MODE_IGNORE))
                    .ok()?;
                let brush = target
                    .CreateSolidColorBrush(&color(theme::INK), None)
                    .ok()?;
                self.target = Some(target);
                self.brush = Some(brush);
            }
        }
        Some((self.target.clone()?, self.brush.clone()?))
    }

    pub fn paint(&mut self, hdc: HDC, area: RECT, draw: impl FnOnce(&Painter)) -> PaintStatus {
        if area.right <= area.left || area.bottom <= area.top {
            return PaintStatus::Done;
        }
        let Some((target, brush)) = self.ensure_target() else {
            return PaintStatus::Failed;
        };
        let result = unsafe {
            if target.BindDC(hdc, &area).is_err() {
                return PaintStatus::Failed;
            }
            target.BeginDraw();
            let origin = (-area.left as f32, -area.top as f32);
            target.SetTransform(&Matrix3x2::translation(origin.0, origin.1));
            let render: &ID2D1RenderTarget = &target;
            let painter = Painter {
                backend: Backend::Direct {
                    engine: self,
                    target: render,
                    brush: &brush,
                    origin,
                },
            };
            draw(&painter);
            target.EndDraw(None, None)
        };
        match result {
            Ok(()) => PaintStatus::Done,
            Err(error) if error.code() == D2DERR_RECREATE_TARGET => {
                self.target = None;
                self.brush = None;
                PaintStatus::Recreate
            }
            Err(_) => {
                self.target = None;
                self.brush = None;
                PaintStatus::Failed
            }
        }
    }

    pub fn rasterize_icon(
        &self,
        icon: Icon,
        size: u32,
        background: Color,
        foreground: Color,
    ) -> Result<Vec<u8>> {
        unsafe {
            let wic: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
            let bitmap = wic.CreateBitmap(
                size,
                size,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapCacheOnLoad,
            )?;
            let target = self.factory.CreateWicBitmapRenderTarget(
                &bitmap,
                &target_properties(D2D1_ALPHA_MODE_PREMULTIPLIED),
            )?;
            let brush = target.CreateSolidColorBrush(&color(foreground), None)?;
            target.BeginDraw();
            target.Clear(Some(&color(background)));
            let painter = Painter {
                backend: Backend::Direct {
                    engine: self,
                    target: &target,
                    brush: &brush,
                    origin: (0.0, 0.0),
                },
            };
            painter.icon(icon, 0.0, 0.0, size as f32, size as f32, foreground);
            target.EndDraw(None, None)?;
            let stride = size * 4;
            let mut pixels = vec![0u8; (stride * size) as usize];
            bitmap.CopyPixels(std::ptr::null(), stride, &mut pixels)?;
            Ok(pixels)
        }
    }
}

pub struct GdiResources {
    dpi: u32,
    fonts: Vec<Option<GdiFont>>,
    metrics: [TextMetrics; 7],
}

impl GdiResources {
    pub fn new(dpi: u32) -> GdiResources {
        let dpi = dpi.max(48);
        let fonts: Vec<Option<GdiFont>> = Font::ALL
            .iter()
            .map(|role| GdiFont::for_role(*role, dpi))
            .collect();
        let mut metrics = [TextMetrics::default(); 7];
        for role in Font::ALL {
            let pixels = role.pixels(dpi);
            metrics[role.index()] = fonts[role.index()]
                .as_ref()
                .map(GdiFont::metrics)
                .filter(|metric| metric.height > 0)
                .unwrap_or(TextMetrics {
                    height: pixels * 3 / 2,
                    ascent: pixels * 6 / 5,
                    descent: pixels * 3 / 10,
                    external_leading: 0,
                });
        }
        GdiResources {
            dpi,
            fonts,
            metrics,
        }
    }

    pub fn dpi(&self) -> u32 {
        self.dpi
    }

    fn font(&self, font: Font) -> Option<&GdiFont> {
        self.fonts.get(font.index()).and_then(Option::as_ref)
    }

    fn measure(&self, text: &str, font: Font) -> i32 {
        self.font(font)
            .and_then(|gdi_font| gdi_extent(gdi_font, text))
            .unwrap_or(0)
    }

    pub fn paint(&self, hdc: HDC, area: RECT, draw: impl FnOnce(&Painter)) {
        let width = area.right - area.left;
        let height = area.bottom - area.top;
        if width <= 0 || height <= 0 {
            return;
        }
        unsafe {
            let memory = CreateCompatibleDC(Some(hdc));
            let bitmap = if memory.is_invalid() {
                Default::default()
            } else {
                CreateCompatibleBitmap(hdc, width, height)
            };
            if memory.is_invalid() || bitmap.is_invalid() {
                if !memory.is_invalid() {
                    let _ = DeleteDC(memory);
                }
                let painter = Painter {
                    backend: Backend::Gdi {
                        resources: self,
                        hdc,
                        area,
                    },
                };
                draw(&painter);
                return;
            }
            let previous = SelectObject(memory, HGDIOBJ(bitmap.0));
            let _ = SetViewportOrgEx(memory, -area.left, -area.top, None);
            SetBkMode(memory, TRANSPARENT);
            let painter = Painter {
                backend: Backend::Gdi {
                    resources: self,
                    hdc: memory,
                    area,
                },
            };
            draw(&painter);
            let _ = SetViewportOrgEx(memory, 0, 0, None);
            let _ = BitBlt(
                hdc,
                area.left,
                area.top,
                width,
                height,
                Some(memory),
                0,
                0,
                SRCCOPY,
            );
            SelectObject(memory, previous);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory);
        }
    }
}

enum Backend<'a> {
    Direct {
        engine: &'a Engine,
        target: &'a ID2D1RenderTarget,
        brush: &'a ID2D1SolidColorBrush,
        origin: (f32, f32),
    },
    Gdi {
        resources: &'a GdiResources,
        hdc: HDC,
        area: RECT,
    },
}

pub struct Painter<'a> {
    backend: Backend<'a>,
}

struct GdiObject(HGDIOBJ);

impl Drop for GdiObject {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

fn solid_brush(value: Color) -> GdiObject {
    GdiObject(HGDIOBJ(
        unsafe { CreateSolidBrush(COLORREF(value.colorref())) }.0,
    ))
}

fn solid_pen(value: Color, style: PEN_STYLE) -> GdiObject {
    GdiObject(HGDIOBJ(
        unsafe { CreatePen(style, 1, COLORREF(value.colorref())) }.0,
    ))
}

fn rounded_shape(frame: Frame) -> D2D1_ROUNDED_RECT {
    let diameter = (frame.radius * 2.0)
        .min(frame.width)
        .min(frame.height)
        .max(0.0);
    D2D1_ROUNDED_RECT {
        rect: D2D_RECT_F {
            left: frame.x + 0.5,
            top: frame.y + 0.5,
            right: frame.x + frame.width + 0.5,
            bottom: frame.y + frame.height + 0.5,
        },
        radiusX: diameter / 2.0,
        radiusY: diameter / 2.0,
    }
}

impl Painter<'_> {
    pub fn is_fallback(&self) -> bool {
        matches!(self.backend, Backend::Gdi { .. })
    }

    pub fn text_metrics(&self, font: Font) -> TextMetrics {
        match &self.backend {
            Backend::Direct { engine, .. } => engine.text_metrics(font),
            Backend::Gdi { resources, .. } => resources.metrics[font.index()],
        }
    }

    pub fn measure(&self, text: &str, font: Font) -> i32 {
        match &self.backend {
            Backend::Direct { engine, .. } => engine.measure(text, font),
            Backend::Gdi { resources, .. } => resources.measure(text, font),
        }
    }

    pub fn margins(&self, font: Font) -> (i32, i32) {
        margins_for(self.text_metrics(font))
    }

    pub fn measure_padded(&self, text: &str, font: Font) -> i32 {
        let (left, right) = self.margins(font);
        self.measure(text, font) + left + right
    }

    fn set_color(brush: &ID2D1SolidColorBrush, value: Color) {
        unsafe { brush.SetColor(&color(value)) };
    }

    pub fn clear(&self, value: Color) {
        match &self.backend {
            Backend::Direct { target, .. } => unsafe { target.Clear(Some(&color(value))) },
            Backend::Gdi { hdc, area, .. } => {
                let brush = solid_brush(value);
                unsafe { FillRect(*hdc, area, windows::Win32::Graphics::Gdi::HBRUSH(brush.0.0)) };
            }
        }
    }

    pub fn fill_rect(&self, rect: Rect, value: Color) {
        if rect.is_empty() {
            return;
        }
        match &self.backend {
            Backend::Direct { target, brush, .. } => {
                Painter::set_color(brush, value);
                unsafe {
                    target.FillRectangle(
                        &D2D_RECT_F {
                            left: rect.x as f32,
                            top: rect.y as f32,
                            right: rect.right() as f32,
                            bottom: rect.bottom() as f32,
                        },
                        *brush,
                    )
                };
            }
            Backend::Gdi { hdc, .. } => {
                let brush = solid_brush(value);
                let area = RECT {
                    left: rect.x,
                    top: rect.y,
                    right: rect.right(),
                    bottom: rect.bottom(),
                };
                unsafe {
                    FillRect(
                        *hdc,
                        &area,
                        windows::Win32::Graphics::Gdi::HBRUSH(brush.0.0),
                    )
                };
            }
        }
    }

    fn gdi_round_rect(
        hdc: HDC,
        frame: Frame,
        fill: Option<Color>,
        stroke: Option<(Color, PEN_STYLE)>,
    ) {
        let diameter = (frame.radius * 2.0)
            .min(frame.width)
            .min(frame.height)
            .max(0.0)
            .round() as i32;
        let left = frame.x.round() as i32;
        let top = frame.y.round() as i32;
        let right = (frame.x + frame.width).round() as i32 + 1;
        let bottom = (frame.y + frame.height).round() as i32 + 1;
        unsafe {
            let brush = fill.map(solid_brush);
            let outline = stroke
                .map(|(value, style)| solid_pen(value, style))
                .or_else(|| fill.map(|value| solid_pen(value, PS_SOLID)));
            let brush_handle = brush
                .as_ref()
                .map_or(GetStockObject(NULL_BRUSH), |object| object.0);
            let Some(pen_handle) = outline.as_ref().map(|pen| pen.0) else {
                return;
            };
            let previous_brush = SelectObject(hdc, brush_handle);
            let previous_pen = SelectObject(hdc, pen_handle);
            let _ = RoundRect(hdc, left, top, right, bottom, diameter, diameter);
            SelectObject(hdc, previous_pen);
            SelectObject(hdc, previous_brush);
        }
    }

    pub fn rounded_box(&self, frame: Frame, fill: Color, stroke: Option<Color>) {
        if frame.width <= 0.0 || frame.height <= 0.0 {
            return;
        }
        match &self.backend {
            Backend::Direct { target, brush, .. } => {
                let shape = rounded_shape(frame);
                Painter::set_color(brush, fill);
                unsafe { target.FillRoundedRectangle(&shape, *brush) };
                if let Some(stroke) = stroke {
                    Painter::set_color(brush, stroke);
                    unsafe {
                        target.DrawRoundedRectangle(&shape, *brush, 1.0, None::<&ID2D1StrokeStyle>)
                    };
                }
            }
            Backend::Gdi { hdc, .. } => Painter::gdi_round_rect(
                *hdc,
                frame,
                Some(fill),
                stroke.map(|value| (value, PS_SOLID)),
            ),
        }
    }

    pub fn rounded_outline(&self, frame: Frame, value: Color, dotted: bool) {
        if frame.width <= 0.0 || frame.height <= 0.0 || value.a == 0 {
            return;
        }
        match &self.backend {
            Backend::Direct {
                engine,
                target,
                brush,
                ..
            } => {
                let shape = rounded_shape(frame);
                Painter::set_color(brush, value);
                unsafe {
                    if dotted {
                        target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_ALIASED);
                        target.DrawRoundedRectangle(&shape, *brush, 1.0, &engine.dotted);
                        target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
                    } else {
                        target.DrawRoundedRectangle(&shape, *brush, 1.0, None::<&ID2D1StrokeStyle>);
                    }
                }
            }
            Backend::Gdi { hdc, .. } => {
                if value.a < 255 {
                    return;
                }
                Painter::gdi_round_rect(
                    *hdc,
                    frame,
                    None,
                    Some((value, if dotted { PS_DOT } else { PS_SOLID })),
                );
            }
        }
    }

    pub fn card(
        &self,
        rect: Rect,
        fill: Color,
        stroke: Color,
        highlight: Option<Color>,
        scale: f32,
    ) {
        self.rounded_box(
            Frame::new(
                (rect.x + 1) as f32,
                (rect.y + 1) as f32,
                (rect.w - 3) as f32,
                (rect.h - 3) as f32,
                theme::CARD_RADIUS * scale,
            ),
            fill,
            Some(stroke),
        );
        if let Some(highlight) = highlight {
            self.rounded_outline(
                Frame::new(
                    (rect.x + 4) as f32,
                    (rect.y + 4) as f32,
                    (rect.w - 9) as f32,
                    (rect.h - 9) as f32,
                    theme::CARD_HIGHLIGHT_RADIUS * scale,
                ),
                highlight,
                false,
            );
        }
    }

    pub fn icon(&self, icon: Icon, x: f32, y: f32, width: f32, height: f32, value: Color) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        match &self.backend {
            Backend::Direct {
                engine,
                target,
                brush,
                origin,
            } => {
                let Some(geometry) = engine.icons.get(icon.index()) else {
                    return;
                };
                Painter::set_color(brush, value);
                unsafe {
                    target.SetTransform(
                        &(Matrix3x2::scale(width / VIEWBOX, height / VIEWBOX)
                            * Matrix3x2::translation(x + 0.5 + origin.0, y + 0.5 + origin.1)),
                    );
                    target.DrawGeometry(geometry, *brush, theme::ICON_STROKE, &engine.round);
                    target.SetTransform(&Matrix3x2::translation(origin.0, origin.1));
                }
            }
            Backend::Gdi { hdc, .. } => gdi_icon(*hdc, icon, x, y, width, height, value),
        }
    }

    pub fn icon_view(&self, icon: Icon, rect: Rect, value: Color) {
        self.icon(
            icon,
            (rect.x + 1) as f32,
            (rect.y + 1) as f32,
            (rect.w - 2) as f32,
            (rect.h - 2) as f32,
            value,
        );
    }

    pub fn text(
        &self,
        text: &str,
        font: Font,
        value: Color,
        rect: Rect,
        align: Align,
        valign: VAlign,
    ) {
        if text.is_empty() || rect.is_empty() {
            return;
        }
        let metric = self.text_metrics(font);
        let measure = |line: &str| self.measure(line, font);
        let lines = plan_text(text, rect, align, valign, metric, &measure);
        self.draw_lines(&lines, font, value, rect);
    }

    pub fn runs(&self, runs: &[Run], gap: i32, font: Font, x: i32, rect: Rect) {
        let metric = self.text_metrics(font);
        let y = rect.y + rect.h / 2 - metric.height.max(1) / 2;
        let mut cursor = x;
        for run in runs {
            if run.text.is_empty() {
                continue;
            }
            let line = PlacedLine {
                text: run.text.to_string(),
                x: cursor,
                y,
            };
            self.draw_lines(std::slice::from_ref(&line), font, run.color, rect);
            cursor += self.measure(run.text, font) + gap;
        }
    }

    fn draw_lines(&self, lines: &[PlacedLine], font: Font, value: Color, rect: Rect) {
        match &self.backend {
            Backend::Direct {
                engine,
                target,
                brush,
                ..
            } => {
                Painter::set_color(brush, value);
                unsafe {
                    target.PushAxisAlignedClip(
                        &D2D_RECT_F {
                            left: rect.x as f32,
                            top: rect.y as f32,
                            right: rect.right() as f32,
                            bottom: rect.bottom() as f32,
                        },
                        D2D1_ANTIALIAS_MODE_ALIASED,
                    );
                }
                for line in lines {
                    if let Some((layout, _)) = engine.layout(&line.text, font) {
                        unsafe {
                            target.DrawTextLayout(
                                point(line.x as f32, line.y as f32),
                                &layout,
                                *brush,
                                D2D1_DRAW_TEXT_OPTIONS_NONE,
                            )
                        };
                    }
                }
                unsafe { target.PopAxisAlignedClip() };
            }
            Backend::Gdi { resources, hdc, .. } => {
                let Some(gdi_font) = resources.font(font) else {
                    return;
                };
                let clip = RECT {
                    left: rect.x,
                    top: rect.y,
                    right: rect.right(),
                    bottom: rect.bottom(),
                };
                unsafe {
                    let previous = SelectObject(*hdc, HGDIOBJ(gdi_font.handle().0));
                    SetTextColor(*hdc, COLORREF(value.colorref()));
                    SetBkMode(*hdc, TRANSPARENT);
                    for line in lines {
                        let wide: Vec<u16> = line.text.encode_utf16().collect();
                        let _ = ExtTextOutW(
                            *hdc,
                            line.x,
                            line.y,
                            ETO_CLIPPED,
                            Some(&clip),
                            windows::core::PCWSTR(wide.as_ptr()),
                            wide.len() as u32,
                            None,
                        );
                    }
                    SelectObject(*hdc, previous);
                }
            }
        }
    }
}

fn gdi_icon(hdc: HDC, icon: Icon, x: f32, y: f32, width: f32, height: f32, value: Color) {
    let sx = width / VIEWBOX;
    let sy = height / VIEWBOX;
    let map = |px: f32, py: f32| POINT {
        x: (x + px * sx).round() as i32,
        y: (y + py * sy).round() as i32,
    };
    let thickness = (theme::ICON_STROKE * sx).round().max(1.0) as u32;
    let brush = LOGBRUSH {
        lbStyle: BS_SOLID,
        lbColor: COLORREF(value.colorref()),
        lbHatch: 0,
    };
    unsafe {
        let pen = GdiObject(HGDIOBJ(
            ExtCreatePen(
                PS_GEOMETRIC | PS_SOLID | PS_ENDCAP_ROUND | PS_JOIN_ROUND,
                thickness,
                &brush,
                None,
            )
            .0,
        ));
        if pen.0.is_invalid() {
            return;
        }
        let previous_pen = SelectObject(hdc, pen.0);
        let previous_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
        for shape in icon.shapes() {
            match shape {
                Shape::Line { x1, y1, x2, y2 } => {
                    let start = map(x1, y1);
                    let end = map(x2, y2);
                    let _ = MoveToEx(hdc, start.x, start.y, None);
                    let _ = LineTo(hdc, end.x, end.y);
                }
                Shape::Polyline(points) => {
                    let mapped: Vec<POINT> = points.iter().map(|(px, py)| map(*px, *py)).collect();
                    let _ = Polyline(hdc, &mapped);
                }
                Shape::Polygon(points) => {
                    let mut mapped: Vec<POINT> =
                        points.iter().map(|(px, py)| map(*px, *py)).collect();
                    if let Some(first) = mapped.first().copied() {
                        mapped.push(first);
                    }
                    let _ = Polyline(hdc, &mapped);
                }
                Shape::Rect {
                    x: rx,
                    y: ry,
                    width: rw,
                    height: rh,
                    radius,
                } => {
                    let top_left = map(rx, ry);
                    let bottom_right = map(rx + rw, ry + rh);
                    let diameter = (radius * 2.0 * sx).round() as i32;
                    let _ = RoundRect(
                        hdc,
                        top_left.x,
                        top_left.y,
                        bottom_right.x + 1,
                        bottom_right.y + 1,
                        diameter,
                        diameter,
                    );
                }
                Shape::Circle { cx, cy, r } => {
                    let top_left = map(cx - r, cy - r);
                    let bottom_right = map(cx + r, cy + r);
                    let _ = Ellipse(
                        hdc,
                        top_left.x,
                        top_left.y,
                        bottom_right.x + 1,
                        bottom_right.y + 1,
                    );
                }
            }
        }
        SelectObject(hdc, previous_brush);
        SelectObject(hdc, previous_pen);
    }
}

pub fn save_png(path: &std::path::Path, width: u32, height: u32, bgra: &[u8]) -> Result<()> {
    unsafe {
        let wic: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        let stream = wic.CreateStream()?;
        stream.InitializeFromFilename(&HSTRING::from(path.as_os_str()), GENERIC_WRITE.0)?;
        let encoder = wic.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
        encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
        let mut frame = None;
        encoder.CreateNewFrame(&mut frame, std::ptr::null_mut())?;
        let frame = frame.ok_or_else(|| windows::core::Error::from(E_FAIL))?;
        frame.Initialize(None)?;
        frame.SetSize(width, height)?;
        let mut format = GUID_WICPixelFormat32bppBGRA;
        frame.SetPixelFormat(&mut format)?;
        frame.WritePixels(height, width * 4, bgra)?;
        frame.Commit()?;
        encoder.Commit()?;
        Ok(())
    }
}
