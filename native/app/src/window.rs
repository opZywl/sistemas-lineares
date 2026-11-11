use std::cell::{Cell, RefCell};
use std::mem::ManuallyDrop;
use std::rc::Rc;

use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAK, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWINDOWATTRIBUTE, DwmSetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, COLOR_BTNFACE, COLOR_BTNTEXT, COLOR_GRAYTEXT, COLOR_HIGHLIGHT, COLOR_HIGHLIGHTTEXT,
    COLOR_HOTLIGHT, COLOR_WINDOW, COLOR_WINDOWTEXT, CreateSolidBrush, DeleteObject, EndPaint,
    FillRect, GetDC, GetMonitorInfoW, GetSysColor, GetTextMetricsW, HBRUSH, HDC, HGDIOBJ, HMONITOR,
    InvalidateRect, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, MONITORINFO, MonitorFromPoint,
    MonitorFromRect, OPAQUE, PAINTSTRUCT, RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE,
    RDW_UPDATENOW, RedrawWindow, ReleaseDC, SelectObject, SetBkColor, SetBkMode, SetTextColor,
    TEXTMETRICW, UpdateWindow,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
    CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemServices::{MK_SHIFT, SS_NOPREFIX, SS_OWNERDRAW};
use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
use windows::Win32::UI::Accessibility::{
    CAccPropServices, HCF_HIGHCONTRASTON, HIGHCONTRASTW, IAccPropServices,
    LiveSetting_Property_GUID, NotifyWinEvent, PROPID_ACC_DESCRIPTION, PROPID_ACC_NAME, Polite,
};
use windows::Win32::UI::Controls::{
    DRAWITEMSTRUCT, EM_GETLINECOUNT, EM_GETSEL, EM_REPLACESEL, EM_SETLIMITTEXT, EM_SETMARGINS,
    EM_SETSEL, ICC_STANDARD_CLASSES, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX, InitCommonControlsEx,
    LoadIconWithScaleDown, ODS_DISABLED, ODS_FOCUS, ODS_NOFOCUSRECT, ODT_BUTTON, ODT_STATIC,
    SetScrollInfo, SetWindowTheme, ShowScrollBar, TOOLTIPS_CLASSW, TTF_IDISHWND, TTF_SUBCLASS,
    TTM_ADDTOOLW, TTM_UPDATETIPTEXTW, TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW, WM_MOUSELEAVE,
};
use windows::Win32::UI::HiDpi::{
    AdjustWindowRectExForDpi, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForMonitor,
    GetDpiForWindow, GetSystemMetricsForDpi, MDT_EFFECTIVE_DPI, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    EnableWindow, GetFocus, GetKeyState, IsWindowEnabled, SetFocus, TME_LEAVE, TRACKMOUSEEVENT,
    TrackMouseEvent, VK_BACK, VK_CONTROL, VK_ESCAPE, VK_RETURN,
};
use windows::Win32::UI::Shell::{
    DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass, ShellExecuteW,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{BOOL, HSTRING, PCWSTR, PWSTR, Result, w};

use crate::clipboard;
use crate::fonts::{self, GdiFont, TextMetrics};
use crate::gfx::{Align, Engine, Frame, GdiResources, PaintStatus, Painter, Run, VAlign};
use crate::icons::Icon;
use crate::install;
use crate::layout::{self, Layout, Metrics, Rect, Viewport, d};
use crate::model::{self, Focus, Model, Outcome};
use crate::settings::{self, Bounds, Placement, Saved};
use crate::theme::{Color, Font, Palette, SystemColors};

pub const ID_TWO: u16 = 100;
pub const ID_THREE: u16 = 101;
pub const ID_INPUT: u16 = 110;
pub const ID_SOLVE: u16 = 130;
pub const ID_EXAMPLE: u16 = 131;
pub const ID_CLEAR: u16 = 132;
pub const ID_COPY: u16 = 133;
pub const ID_AUTHOR: u16 = 134;
pub const ID_VALUE: u16 = 140;
pub const ID_GENERAL: u16 = 143;
pub const ID_DIMENSION: u16 = 150;
pub const ID_HELPER: u16 = 151;
pub const ID_TITLE: u16 = 152;
pub const ID_DESCRIPTION: u16 = 153;
pub const ID_EMPTY_TITLE: u16 = 154;
pub const ID_EMPTY_DESCRIPTION: u16 = 155;
pub const ID_NOTE: u16 = 156;

pub const MAIN_CLASS_NAME: &str = "SistemasLineares.Janela";
const MAIN_CLASS: PCWSTR = w!("SistemasLineares.Janela");
const CANVAS_CLASS: PCWSTR = w!("SistemasLineares.Tela");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Two,
    Three,
    Solve,
    Example,
    Clear,
    Copy,
    Author,
}

impl Button {
    pub const ALL: [Button; 7] = [
        Button::Two,
        Button::Three,
        Button::Solve,
        Button::Example,
        Button::Clear,
        Button::Copy,
        Button::Author,
    ];

    pub const fn id(self) -> u16 {
        match self {
            Button::Two => ID_TWO,
            Button::Three => ID_THREE,
            Button::Solve => ID_SOLVE,
            Button::Example => ID_EXAMPLE,
            Button::Clear => ID_CLEAR,
            Button::Copy => ID_COPY,
            Button::Author => ID_AUTHOR,
        }
    }

    pub fn from_id(id: u16) -> Option<Button> {
        Button::ALL.into_iter().find(|button| button.id() == id)
    }

    const fn index(self) -> usize {
        match self {
            Button::Two => 0,
            Button::Three => 1,
            Button::Solve => 2,
            Button::Example => 3,
            Button::Clear => 4,
            Button::Copy => 5,
            Button::Author => 6,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Button::Two => model::TWO_LABEL,
            Button::Three => model::THREE_LABEL,
            Button::Solve => model::SOLVE_LABEL,
            Button::Example => model::EXAMPLE_LABEL,
            Button::Clear => model::CLEAR_LABEL,
            Button::Copy => model::COPY_LABEL,
            Button::Author => model::AUTHOR_LABEL,
        }
    }

    const fn icon(self) -> Option<Icon> {
        match self {
            Button::Two | Button::Three => None,
            Button::Solve => Some(Icon::ArrowRight),
            Button::Example => Some(Icon::Sparkles),
            Button::Clear => Some(Icon::Reset),
            Button::Copy => Some(Icon::Copy),
            Button::Author => Some(Icon::External),
        }
    }

    const fn primary(self) -> bool {
        matches!(self, Button::Solve)
    }

    const fn quiet(self) -> bool {
        matches!(self, Button::Two | Button::Three | Button::Author)
    }

    fn rect(self, layout: &Layout) -> Rect {
        match self {
            Button::Two => layout.two,
            Button::Three => layout.three,
            Button::Solve => layout.solve,
            Button::Example => layout.example,
            Button::Clear => layout.clear,
            Button::Copy => layout.copy,
            Button::Author => layout.author,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Label {
    Dimension,
    Helper,
    Title,
    Description,
    EmptyTitle,
    EmptyDescription,
    Note,
}

impl Label {
    const ALL: [Label; 7] = [
        Label::Dimension,
        Label::Helper,
        Label::Title,
        Label::Description,
        Label::EmptyTitle,
        Label::EmptyDescription,
        Label::Note,
    ];

    const fn id(self) -> u16 {
        match self {
            Label::Dimension => ID_DIMENSION,
            Label::Helper => ID_HELPER,
            Label::Title => ID_TITLE,
            Label::Description => ID_DESCRIPTION,
            Label::EmptyTitle => ID_EMPTY_TITLE,
            Label::EmptyDescription => ID_EMPTY_DESCRIPTION,
            Label::Note => ID_NOTE,
        }
    }

    fn from_id(id: u16) -> Option<Label> {
        Label::ALL.into_iter().find(|label| label.id() == id)
    }

    const fn font(self) -> Font {
        match self {
            Label::Dimension | Label::Helper | Label::Note => Font::Small,
            Label::Title | Label::EmptyTitle => Font::Heading,
            Label::Description | Label::EmptyDescription => Font::Body,
        }
    }

    const fn color(self, palette: &Palette, helper_error: bool) -> Color {
        match self {
            Label::Dimension => palette.muted,
            Label::Helper if helper_error => palette.error_text,
            Label::Helper => palette.muted,
            Label::Title | Label::EmptyTitle => palette.solution_text,
            Label::Description | Label::EmptyDescription | Label::Note => palette.solution_muted,
        }
    }

    const fn background(self, palette: &Palette) -> Color {
        match self {
            Label::Dimension | Label::Helper => palette.surface,
            _ => palette.solution_fill,
        }
    }

    const fn align(self) -> (Align, VAlign) {
        match self {
            Label::EmptyTitle => (Align::Center, VAlign::Center),
            Label::EmptyDescription => (Align::Center, VAlign::Top),
            _ => (Align::Left, VAlign::Top),
        }
    }

    const fn live(self) -> bool {
        matches!(
            self,
            Label::Dimension | Label::Helper | Label::Title | Label::Note
        )
    }

    fn rect(self, layout: &Layout) -> Rect {
        match self {
            Label::Dimension => layout.dimension_text,
            Label::Helper => layout.helper,
            Label::Title => layout.solution_title,
            Label::Description => layout.solution_description,
            Label::EmptyTitle => layout.empty_title,
            Label::EmptyDescription => layout.empty_description,
            Label::Note => layout.note,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Control {
    Button(Button),
    Input(usize, usize),
    Value(usize),
    General,
    Label(Label),
}

fn classify(id: u16) -> Option<Control> {
    if let Some(button) = Button::from_id(id) {
        return Some(Control::Button(button));
    }
    if (ID_INPUT..ID_INPUT + 12).contains(&id) {
        let index = usize::from(id - ID_INPUT);
        return Some(Control::Input(index / 4, index % 4));
    }
    if (ID_VALUE..ID_VALUE + 3).contains(&id) {
        return Some(Control::Value(usize::from(id - ID_VALUE)));
    }
    if id == ID_GENERAL {
        return Some(Control::General);
    }
    Label::from_id(id).map(Control::Label)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Handles {
    pub main: HWND,
    pub canvas: HWND,
    pub tooltip: HWND,
    pub inputs: [[HWND; 4]; 3],
    pub two: HWND,
    pub three: HWND,
    pub solve: HWND,
    pub example: HWND,
    pub clear: HWND,
    pub copy: HWND,
    pub author: HWND,
    pub values: [HWND; 3],
    pub general: HWND,
    pub dimension: HWND,
    pub helper: HWND,
    pub title: HWND,
    pub description: HWND,
    pub empty_title: HWND,
    pub empty_description: HWND,
    pub note: HWND,
}

impl Handles {
    pub fn button(&self, button: Button) -> HWND {
        match button {
            Button::Two => self.two,
            Button::Three => self.three,
            Button::Solve => self.solve,
            Button::Example => self.example,
            Button::Clear => self.clear,
            Button::Copy => self.copy,
            Button::Author => self.author,
        }
    }

    fn label(&self, label: Label) -> HWND {
        match label {
            Label::Dimension => self.dimension,
            Label::Helper => self.helper,
            Label::Title => self.title,
            Label::Description => self.description,
            Label::EmptyTitle => self.empty_title,
            Label::EmptyDescription => self.empty_description,
            Label::Note => self.note,
        }
    }

    fn control(&self, hwnd: HWND) -> Option<Control> {
        if hwnd.is_invalid() {
            return None;
        }
        if let Some(button) = Button::ALL
            .into_iter()
            .find(|button| self.button(*button) == hwnd)
        {
            return Some(Control::Button(button));
        }
        for row in 0..3 {
            for column in 0..4 {
                if self.inputs[row][column] == hwnd {
                    return Some(Control::Input(row, column));
                }
            }
        }
        if let Some(index) = self.values.iter().position(|value| *value == hwnd) {
            return Some(Control::Value(index));
        }
        if self.general == hwnd {
            return Some(Control::General);
        }
        Label::ALL
            .into_iter()
            .find(|label| self.label(*label) == hwnd)
            .map(Control::Label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    Direct2D,
    Gdi,
}

#[derive(Clone, Debug)]
pub struct Launch {
    pub position: Option<(i32, i32)>,
    pub client_size: Option<(i32, i32)>,
    pub show: SHOW_WINDOW_CMD,
    pub quit_on_close: bool,
    pub settings_key: Option<String>,
    pub activate: bool,
    pub renderer: Renderer,
    pub high_contrast: Option<bool>,
}

impl Default for Launch {
    fn default() -> Self {
        Launch {
            position: None,
            client_size: None,
            show: SW_SHOWNORMAL,
            quit_on_close: true,
            settings_key: Some(settings::KEY.to_string()),
            activate: true,
            renderer: Renderer::Direct2D,
            high_contrast: None,
        }
    }
}

impl Launch {
    pub fn background(position: (i32, i32), client_size: (i32, i32)) -> Launch {
        Launch {
            position: Some(position),
            client_size: Some(client_size),
            show: SW_SHOWNOACTIVATE,
            quit_on_close: false,
            settings_key: None,
            activate: false,
            renderer: Renderer::Direct2D,
            high_contrast: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Diagnostics {
    pub dpi: u32,
    pub palette: Palette,
    pub direct2d: bool,
    pub dimension: usize,
    pub scroll: (i32, i32),
    pub layout: Layout,
    pub live_regions: usize,
    pub annotated: usize,
}

struct Brushes(Vec<(Color, HBRUSH)>);

impl Brushes {
    fn get(&mut self, value: Color) -> HBRUSH {
        if let Some((_, brush)) = self.0.iter().find(|(color, _)| *color == value) {
            return *brush;
        }
        let brush = unsafe { CreateSolidBrush(COLORREF(value.colorref())) };
        if !brush.is_invalid() {
            self.0.push((value, brush));
        }
        brush
    }

    fn clear(&mut self) {
        for (_, brush) in self.0.drain(..) {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(brush.0));
            }
        }
    }
}

impl Drop for Brushes {
    fn drop(&mut self) {
        self.clear();
    }
}

struct Icons {
    big: HICON,
    small: HICON,
}

impl Icons {
    fn load(instance: HINSTANCE, dpi: u32) -> Icons {
        let load = |metric_x: SYSTEM_METRICS_INDEX, metric_y: SYSTEM_METRICS_INDEX| unsafe {
            LoadIconWithScaleDown(
                Some(instance),
                PCWSTR(1 as _),
                GetSystemMetricsForDpi(metric_x, dpi),
                GetSystemMetricsForDpi(metric_y, dpi),
            )
            .unwrap_or_default()
        };
        Icons {
            big: load(SM_CXICON, SM_CYICON),
            small: load(SM_CXSMICON, SM_CYSMICON),
        }
    }
}

impl Drop for Icons {
    fn drop(&mut self) {
        for icon in [self.big, self.small] {
            if !icon.is_invalid() {
                unsafe {
                    let _ = DestroyIcon(icon);
                }
            }
        }
    }
}

struct View {
    dpi: u32,
    scale: f32,
    layout: Layout,
    viewport: Viewport,
    scroll: (i32, i32),
    metrics: Metrics,
    input_metrics: TextMetrics,
    input_font: Option<GdiFont>,
    value_font: Option<GdiFont>,
    icons: Option<Icons>,
    wheel: (i32, i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Announce {
    Nothing,
    Label(Label),
    Outcome,
}

struct App {
    instance: HINSTANCE,
    handles: Cell<Handles>,
    model: RefCell<Model>,
    view: RefCell<View>,
    engine: RefCell<Option<Engine>>,
    fallback: RefCell<Option<GdiResources>>,
    renderer: Renderer,
    palette: Cell<Palette>,
    contrast: Cell<Option<bool>>,
    theming: Cell<bool>,
    brushes: RefCell<Brushes>,
    syncing: Cell<bool>,
    in_layout: Cell<bool>,
    hover: [Cell<bool>; 7],
    last_focus: Cell<HWND>,
    quit_on_destroy: bool,
    settings_key: Option<String>,
    normal: Cell<Option<(Bounds, u32)>>,
    accessibility: RefCell<Option<IAccPropServices>>,
    annotated: RefCell<Vec<HWND>>,
    live: Cell<usize>,
}

fn loword(value: usize) -> u16 {
    (value & 0xFFFF) as u16
}

fn hiword(value: usize) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}

fn makelong(low: u16, high: u16) -> u32 {
    u32::from(low) | (u32::from(high) << 16)
}

fn point_from(lparam: LPARAM) -> (i32, i32) {
    (
        i32::from((lparam.0 & 0xFFFF) as u16 as i16),
        i32::from(((lparam.0 >> 16) & 0xFFFF) as u16 as i16),
    )
}

pub fn window_text(hwnd: HWND) -> String {
    unsafe {
        let length = GetWindowTextLengthW(hwnd);
        if length <= 0 {
            return String::new();
        }
        let mut buffer = vec![0u16; length as usize + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        String::from_utf16_lossy(&buffer[..copied.max(0) as usize])
    }
}

fn set_text(hwnd: HWND, text: &str) -> bool {
    if hwnd.is_invalid() || window_text(hwnd) == text {
        return false;
    }
    unsafe { SetWindowTextW(hwnd, &HSTRING::from(text)).is_ok() }
}

fn send(hwnd: HWND, message: u32, wparam: usize, lparam: isize) -> LRESULT {
    unsafe { SendMessageW(hwnd, message, Some(WPARAM(wparam)), Some(LPARAM(lparam))) }
}

fn invalidate(hwnd: HWND) {
    if !hwnd.is_invalid() {
        unsafe {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
}

fn win_rect(rect: Rect) -> RECT {
    RECT {
        left: rect.x,
        top: rect.y,
        right: rect.right(),
        bottom: rect.bottom(),
    }
}

fn bounds_of(rect: RECT) -> Bounds {
    Bounds {
        left: rect.left,
        top: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    }
}

fn rect_of(bounds: Bounds) -> RECT {
    RECT {
        left: bounds.left,
        top: bounds.top,
        right: bounds.right(),
        bottom: bounds.bottom(),
    }
}

unsafe fn app_from(hwnd: HWND) -> Option<Rc<App>> {
    unsafe {
        let raw = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const App;
        if raw.is_null() {
            return None;
        }
        Rc::increment_strong_count(raw);
        Some(Rc::from_raw(raw))
    }
}

pub fn system_high_contrast() -> bool {
    let mut contrast = HIGHCONTRASTW {
        cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
        ..Default::default()
    };
    let queried = unsafe {
        SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            contrast.cbSize,
            Some((&mut contrast as *mut HIGHCONTRASTW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .is_ok();
    queried && (contrast.dwFlags.0 & HCF_HIGHCONTRASTON.0) != 0
}

pub fn system_colors() -> SystemColors {
    let read = |index| Color::from_colorref(unsafe { GetSysColor(index) });
    SystemColors {
        window: read(COLOR_WINDOW),
        window_text: read(COLOR_WINDOWTEXT),
        button_face: read(COLOR_BTNFACE),
        button_text: read(COLOR_BTNTEXT),
        highlight: read(COLOR_HIGHLIGHT),
        highlight_text: read(COLOR_HIGHLIGHTTEXT),
        gray_text: read(COLOR_GRAYTEXT),
        hot_light: read(COLOR_HOTLIGHT),
    }
}

fn palette_for(high_contrast: bool) -> Palette {
    if high_contrast {
        Palette::high_contrast(system_colors())
    } else {
        Palette::dark()
    }
}

thread_local! {
    static THREAD_READY: Cell<bool> = const { Cell::new(false) };
}

fn init_thread() {
    if THREAD_READY.with(|ready| ready.replace(true)) {
        return;
    }
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE);
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let controls = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES | ICC_WIN95_CLASSES,
        };
        let _ = InitCommonControlsEx(&controls);
    }
    fonts::register_gdi();
}

fn register_classes(instance: HINSTANCE) {
    unsafe {
        let cursor = LoadCursorW(None, IDC_ARROW).unwrap_or_default();
        let icon = LoadImageW(
            Some(instance),
            PCWSTR(1 as _),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        )
        .map(|handle| HICON(handle.0))
        .unwrap_or_default();
        let small = LoadImageW(
            Some(instance),
            PCWSTR(1 as _),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            LR_SHARED,
        )
        .map(|handle| HICON(handle.0))
        .unwrap_or_default();
        let background = CreateSolidBrush(COLORREF(crate::theme::CANVAS.colorref()));
        let main = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_DBLCLKS,
            lpfnWndProc: Some(main_proc),
            hInstance: instance,
            hIcon: icon,
            hIconSm: small,
            hCursor: cursor,
            hbrBackground: background,
            lpszClassName: MAIN_CLASS,
            ..Default::default()
        };
        if RegisterClassExW(&main) == 0 && !background.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(background.0));
        }
        let canvas = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(canvas_proc),
            hInstance: instance,
            hCursor: cursor,
            lpszClassName: CANVAS_CLASS,
            ..Default::default()
        };
        RegisterClassExW(&canvas);
    }
}

fn monitor_details(monitor: HMONITOR) -> (u32, Bounds) {
    unsafe {
        let mut x = 96;
        let mut y = 96;
        if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y).is_err() {
            x = 96;
        }
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let work = if GetMonitorInfoW(monitor, &mut info).as_bool() {
            bounds_of(info.rcWork)
        } else {
            Bounds {
                left: 0,
                top: 0,
                width: 1920,
                height: 1040,
            }
        };
        (x.max(48), work)
    }
}

fn frame_for(client: (i32, i32), dpi: u32) -> (i32, i32) {
    let mut frame = RECT {
        left: 0,
        top: 0,
        right: client.0,
        bottom: client.1,
    };
    unsafe {
        let _ = AdjustWindowRectExForDpi(
            &mut frame,
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            false,
            WS_EX_CONTROLPARENT,
            dpi,
        );
    }
    (frame.right - frame.left, frame.bottom - frame.top)
}

fn minimum_window(dpi: u32) -> (i32, i32) {
    let scale = layout::dpi_scale(dpi);
    (
        d(scale, layout::MIN_WINDOW_WIDTH),
        d(scale, layout::MIN_WINDOW_HEIGHT),
    )
}

pub fn restored_bounds(placement: Placement) -> Option<(Bounds, u32)> {
    let rect = rect_of(placement.bounds);
    let monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) };
    if monitor.is_invalid() {
        return None;
    }
    let (dpi, work) = monitor_details(monitor);
    Some((
        settings::fit(placement, work, dpi, minimum_window(dpi)),
        dpi,
    ))
}

fn default_bounds(anchor: POINT, client_size: Option<(i32, i32)>) -> (Bounds, u32) {
    let monitor = unsafe { MonitorFromPoint(anchor, MONITOR_DEFAULTTONEAREST) };
    let (dpi, work) = monitor_details(monitor);
    let scale = layout::dpi_scale(dpi);
    let client = client_size.unwrap_or_else(|| {
        (
            d(scale, layout::BASE_CLIENT_WIDTH).min(work.width - 40),
            d(scale, layout::BASE_CLIENT_HEIGHT).min(work.height - 60),
        )
    });
    let (width, height) = frame_for(client, dpi);
    (
        Bounds {
            left: work.left + (work.width - width) / 2,
            top: work.top + (work.height - height) / 2,
            width,
            height,
        },
        dpi,
    )
}

pub fn startup_show(maximized: bool, requested: SHOW_WINDOW_CMD) -> SHOW_WINDOW_CMD {
    match requested {
        SW_HIDE => SW_HIDE,
        SW_MINIMIZE | SW_SHOWMINIMIZED | SW_SHOWMINNOACTIVE | SW_FORCEMINIMIZE => {
            if maximized {
                SW_SHOWMAXIMIZED
            } else {
                SW_SHOWNORMAL
            }
        }
        _ if maximized => SW_SHOWMAXIMIZED,
        other => other,
    }
}

fn cloak(hwnd: HWND, cloaked: bool) -> bool {
    let value = BOOL::from(cloaked);
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CLOAK,
            (&value as *const BOOL).cast(),
            std::mem::size_of::<BOOL>() as u32,
        )
    }
    .is_ok()
}

pub fn create(launch: &Launch) -> Result<Handles> {
    init_thread();
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        register_classes(instance);
        let saved = launch
            .settings_key
            .as_deref()
            .map(settings::load)
            .unwrap_or_default();
        let mut maximized = false;
        let (bounds, dpi) = match (
            launch.position,
            saved
                .placement
                .filter(|_| launch.client_size.is_none())
                .and_then(|placement| {
                    restored_bounds(placement).map(|restored| (restored, placement.maximized))
                }),
        ) {
            (Some((x, y)), _) => {
                let (default, dpi) = default_bounds(POINT { x, y }, launch.client_size);
                (
                    Bounds {
                        left: x,
                        top: y,
                        ..default
                    },
                    dpi,
                )
            }
            (None, Some((restored, was_maximized))) => {
                maximized = was_maximized;
                restored
            }
            (None, None) => {
                let mut cursor = POINT::default();
                let _ = GetCursorPos(&mut cursor);
                default_bounds(cursor, launch.client_size)
            }
        };
        let style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN;
        let ex_style = if launch.activate {
            WS_EX_CONTROLPARENT
        } else {
            WS_EX_CONTROLPARENT | WS_EX_NOACTIVATE
        };
        let high_contrast = launch.high_contrast.unwrap_or_else(system_high_contrast);
        let app = Rc::new(App::new(instance, dpi, launch, palette_for(high_contrast)));
        if let Some(dimension) = saved.dimension {
            if let Ok(mut model) = app.model.try_borrow_mut() {
                model.change_dimension(dimension);
            }
        }
        let main = CreateWindowExW(
            ex_style,
            MAIN_CLASS,
            &HSTRING::from(model::WINDOW_TITLE),
            style,
            bounds.left,
            bounds.top,
            bounds.width,
            bounds.height,
            None,
            None,
            Some(instance),
            Some(Rc::as_ptr(&app).cast()),
        )?;
        app.adopt_dpi(GetDpiForWindow(main));
        if let Err(error) = app.build(main) {
            let _ = DestroyWindow(main);
            return Err(error);
        }
        app.track_normal(main);
        let handles = app.handles.get();
        let show = startup_show(maximized, launch.show);
        let cloaked = show != SW_HIDE && cloak(main, true);
        let _ = ShowWindow(main, show);
        if IsWindowVisible(main).as_bool() {
            let _ = SetFocus(Some(handles.inputs[0][0]));
        }
        app.last_focus.set(handles.inputs[0][0]);
        let _ = RedrawWindow(
            Some(main),
            None,
            None,
            RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN | RDW_UPDATENOW,
        );
        if cloaked {
            cloak(main, false);
        }
        let _ = UpdateWindow(main);
        Ok(handles)
    }
}

pub fn run() -> i32 {
    let arguments = install::parse_arguments(
        std::env::args_os()
            .skip(1)
            .map(|argument| argument.to_string_lossy().into_owned()),
    );
    let com = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
    let code = match arguments.command {
        Some(install::Command::Install) => install::run_install(arguments.silent),
        Some(install::Command::Uninstall) => install::run_uninstall(arguments.silent),
        None => match create(&Launch::default()) {
            Ok(handles) => message_loop(handles.main),
            Err(error) => {
                let text = HSTRING::from(format!(
                    "Não foi possível iniciar o aplicativo.\n\n{}",
                    error.message()
                ));
                unsafe {
                    MessageBoxW(
                        None,
                        &text,
                        &HSTRING::from(model::BRAND),
                        MB_ICONERROR | MB_OK,
                    )
                };
                1
            }
        },
    };
    if com.is_ok() {
        unsafe { CoUninitialize() };
    }
    code
}

pub fn message_loop(main: HWND) -> i32 {
    let mut message = MSG::default();
    loop {
        let result = unsafe { GetMessageW(&mut message, None, 0, 0) };
        match result.0 {
            0 => return message.wParam.0 as i32,
            -1 => return 1,
            _ => dispatch(main, &message),
        }
    }
}

pub fn pump(main: HWND) -> bool {
    let mut message = MSG::default();
    unsafe {
        while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
            if message.message == WM_QUIT {
                return false;
            }
            dispatch(main, &message);
        }
    }
    true
}

pub fn dispatch(main: HWND, message: &MSG) {
    unsafe {
        if IsWindow(Some(main)).as_bool() {
            if handle_return(main, message) {
                return;
            }
            if IsDialogMessageW(main, message).as_bool() {
                return;
            }
        }
        let _ = TranslateMessage(message);
        DispatchMessageW(message);
    }
}

fn handle_return(main: HWND, message: &MSG) -> bool {
    if message.message != WM_KEYDOWN || message.wParam.0 != usize::from(VK_RETURN.0) {
        return false;
    }
    let Some(app) = (unsafe { app_from(main) }) else {
        return false;
    };
    let handles = app.handles.get();
    if message.hwnd != handles.main && unsafe { !IsChild(handles.main, message.hwnd).as_bool() } {
        return false;
    }
    let focus = unsafe { GetFocus() };
    match handles.control(focus) {
        Some(Control::Button(button)) => {
            if unsafe { IsWindowEnabled(focus).as_bool() } {
                send(
                    handles.canvas,
                    WM_COMMAND,
                    makelong(button.id(), BN_CLICKED as u16) as usize,
                    focus.0 as isize,
                );
            }
            true
        }
        Some(Control::Input(..)) | Some(Control::Value(_)) => {
            send(
                handles.canvas,
                WM_COMMAND,
                makelong(ID_SOLVE, BN_CLICKED as u16) as usize,
                handles.solve.0 as isize,
            );
            true
        }
        Some(Control::General) => true,
        _ => false,
    }
}

pub fn diagnostics(main: HWND) -> Option<Diagnostics> {
    let app = unsafe { app_from(main) }?;
    let view = app.view.try_borrow().ok()?;
    let dimension = app.model.try_borrow().map_or(2, |model| model.dimension());
    let direct2d = app.renderer == Renderer::Direct2D
        && app
            .engine
            .try_borrow()
            .map(|engine| engine.is_some())
            .unwrap_or(false);
    Some(Diagnostics {
        dpi: view.dpi,
        palette: app.palette.get(),
        direct2d,
        dimension,
        scroll: view.scroll,
        layout: view.layout,
        live_regions: app.live.get(),
        annotated: app.annotated.try_borrow().map_or(0, |list| list.len()),
    })
}

pub fn force_high_contrast(main: HWND, high_contrast: Option<bool>) {
    if let Some(app) = unsafe { app_from(main) } {
        app.contrast.set(high_contrast);
        app.refresh_theme(main);
    }
}

unsafe fn adopt_create_params(hwnd: HWND, lparam: LPARAM) {
    unsafe {
        if let Some(create) = (lparam.0 as *const CREATESTRUCTW).as_ref() {
            let raw = create.lpCreateParams as *const App;
            if !raw.is_null() {
                Rc::increment_strong_count(raw);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, raw as isize);
            }
        }
    }
}

unsafe extern "system" fn main_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            adopt_create_params(hwnd, lparam);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        if message == WM_NCDESTROY {
            let raw = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *const App;
            if !raw.is_null() {
                drop(Rc::from_raw(raw));
            }
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        match app_from(hwnd) {
            Some(app) => app.main_message(hwnd, message, wparam, lparam),
            None => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

unsafe extern "system" fn canvas_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            adopt_create_params(hwnd, lparam);
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        if message == WM_NCDESTROY {
            let raw = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0) as *const App;
            if !raw.is_null() {
                drop(Rc::from_raw(raw));
            }
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        match app_from(hwnd) {
            Some(app) => app.canvas_message(hwnd, message, wparam, lparam),
            None => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

unsafe extern "system" fn control_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    unsafe {
        if message == WM_NCDESTROY {
            let _ = RemoveWindowSubclass(hwnd, Some(control_proc), id);
            let raw = data as *const App;
            if !raw.is_null() {
                drop(Rc::from_raw(raw));
            }
            return DefSubclassProc(hwnd, message, wparam, lparam);
        }
        let raw = data as *const App;
        if raw.is_null() {
            return DefSubclassProc(hwnd, message, wparam, lparam);
        }
        Rc::increment_strong_count(raw);
        let app = Rc::from_raw(raw);
        app.control_message(hwnd, id as u16, message, wparam, lparam)
    }
}

fn live_variant() -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_I4,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { lVal: Polite.0 },
            }),
        },
    }
}

impl App {
    fn new(instance: HINSTANCE, dpi: u32, launch: &Launch, palette: Palette) -> App {
        App {
            instance,
            handles: Cell::new(Handles::default()),
            model: RefCell::new(Model::new()),
            view: RefCell::new(View {
                dpi,
                scale: layout::dpi_scale(dpi),
                layout: Layout::default(),
                viewport: Viewport::default(),
                scroll: (0, 0),
                metrics: Metrics::default(),
                input_metrics: TextMetrics::default(),
                input_font: None,
                value_font: None,
                icons: None,
                wheel: (0, 0),
            }),
            engine: RefCell::new(None),
            fallback: RefCell::new(None),
            renderer: launch.renderer,
            palette: Cell::new(palette),
            contrast: Cell::new(launch.high_contrast),
            theming: Cell::new(false),
            brushes: RefCell::new(Brushes(Vec::new())),
            syncing: Cell::new(false),
            in_layout: Cell::new(false),
            hover: Default::default(),
            last_focus: Cell::new(HWND::default()),
            quit_on_destroy: launch.quit_on_close,
            settings_key: launch.settings_key.clone(),
            normal: Cell::new(None),
            accessibility: RefCell::new(None),
            annotated: RefCell::new(Vec::new()),
            live: Cell::new(0),
        }
    }

    fn adopt_dpi(&self, dpi: u32) {
        if dpi < 48 {
            return;
        }
        if let Ok(mut view) = self.view.try_borrow_mut() {
            view.dpi = dpi;
            view.scale = layout::dpi_scale(dpi);
        }
    }

    fn dpi(&self) -> u32 {
        self.view.try_borrow().map_or(96, |view| view.dpi)
    }

    fn scale(&self) -> f32 {
        self.view.try_borrow().map_or(1.0, |view| view.scale)
    }

    fn layout(&self) -> Layout {
        self.view
            .try_borrow()
            .map(|view| view.layout)
            .unwrap_or_default()
    }

    fn brush(&self, value: Color) -> HBRUSH {
        self.brushes
            .try_borrow_mut()
            .map(|mut brushes| brushes.get(value))
            .unwrap_or_default()
    }

    fn apply_frame_theme(&self, main: HWND) {
        if self.theming.replace(true) {
            return;
        }
        let palette = self.palette.get();
        unsafe {
            let dark = BOOL::from(!palette.high_contrast);
            if DwmSetWindowAttribute(
                main,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                (&dark as *const BOOL).cast(),
                4,
            )
            .is_err()
            {
                let _ = DwmSetWindowAttribute(
                    main,
                    DWMWINDOWATTRIBUTE(19),
                    (&dark as *const BOOL).cast(),
                    4,
                );
            }
            if palette.high_contrast {
                let _ = SetWindowTheme(main, PCWSTR::null(), PCWSTR::null());
            } else {
                let _ = SetWindowTheme(main, w!("DarkMode_Explorer"), PCWSTR::null());
            }
        }
        self.theming.set(false);
    }

    fn build(self: &Rc<Self>, main: HWND) -> Result<()> {
        self.apply_frame_theme(main);
        let dpi = self.dpi();
        if self.renderer == Renderer::Direct2D {
            if let Ok(mut engine) = self.engine.try_borrow_mut() {
                *engine = Engine::new(dpi).ok();
            }
        }
        self.apply_fonts(dpi);
        let canvas = unsafe {
            CreateWindowExW(
                WS_EX_CONTROLPARENT,
                CANVAS_CLASS,
                PCWSTR::null(),
                WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                0,
                0,
                0,
                0,
                Some(main),
                None,
                Some(self.instance),
                Some(Rc::as_ptr(self).cast()),
            )
        }?;
        let mut handles = Handles {
            main,
            canvas,
            ..Handles::default()
        };
        let button_style = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32);
        handles.two = self.child(
            canvas,
            w!("BUTTON"),
            Button::Two.label(),
            button_style,
            ID_TWO,
        )?;
        handles.three = self.child(
            canvas,
            w!("BUTTON"),
            Button::Three.label(),
            button_style,
            ID_THREE,
        )?;
        for row in 0..3 {
            for column in 0..4 {
                let id = ID_INPUT + (row * 4 + column) as u16;
                let edit = self.child(
                    canvas,
                    w!("EDIT"),
                    "",
                    WS_CHILD | WS_TABSTOP | WINDOW_STYLE((ES_CENTER | ES_AUTOHSCROLL) as u32),
                    id,
                )?;
                send(edit, EM_SETLIMITTEXT, 32767, 0);
                handles.inputs[row][column] = edit;
            }
        }
        handles.solve = self.child(
            canvas,
            w!("BUTTON"),
            Button::Solve.label(),
            button_style,
            ID_SOLVE,
        )?;
        handles.example = self.child(
            canvas,
            w!("BUTTON"),
            Button::Example.label(),
            button_style,
            ID_EXAMPLE,
        )?;
        handles.clear = self.child(
            canvas,
            w!("BUTTON"),
            Button::Clear.label(),
            button_style,
            ID_CLEAR,
        )?;
        let label_style = WS_CHILD | WINDOW_STYLE(SS_OWNERDRAW.0 | SS_NOPREFIX.0);
        handles.dimension = self.child(
            canvas,
            w!("STATIC"),
            "",
            label_style | WS_VISIBLE,
            ID_DIMENSION,
        )?;
        handles.helper = self.child(
            canvas,
            w!("STATIC"),
            "",
            label_style | WS_VISIBLE,
            ID_HELPER,
        )?;
        handles.title = self.child(canvas, w!("STATIC"), "", label_style, ID_TITLE)?;
        handles.description = self.child(canvas, w!("STATIC"), "", label_style, ID_DESCRIPTION)?;
        handles.empty_title = self.child(
            canvas,
            w!("STATIC"),
            model::EMPTY_TITLE,
            label_style | WS_VISIBLE,
            ID_EMPTY_TITLE,
        )?;
        handles.empty_description = self.child(
            canvas,
            w!("STATIC"),
            model::EMPTY_DESCRIPTION,
            label_style | WS_VISIBLE,
            ID_EMPTY_DESCRIPTION,
        )?;
        handles.note = self.child(canvas, w!("STATIC"), "", label_style | WS_VISIBLE, ID_NOTE)?;
        for index in 0..3 {
            handles.values[index] = self.child(
                canvas,
                w!("EDIT"),
                "",
                WS_CHILD | WINDOW_STYLE((ES_LEFT | ES_AUTOHSCROLL | ES_READONLY) as u32),
                ID_VALUE + index as u16,
            )?;
        }
        handles.general = self.child(
            canvas,
            w!("EDIT"),
            "",
            WS_CHILD
                | WS_VSCROLL
                | WINDOW_STYLE((ES_LEFT | ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL) as u32),
            ID_GENERAL,
        )?;
        handles.copy = self.child(
            canvas,
            w!("BUTTON"),
            Button::Copy.label(),
            button_style,
            ID_COPY,
        )?;
        handles.author = self.child(
            canvas,
            w!("BUTTON"),
            Button::Author.label(),
            button_style,
            ID_AUTHOR,
        )?;
        unsafe {
            let _ = EnableWindow(handles.copy, false);
        }
        handles.tooltip = self.create_tooltip(main, canvas, &handles);
        self.handles.set(handles);
        self.last_focus.set(handles.inputs[0][0]);
        self.send_fonts();
        self.annotate_all(&handles);
        self.update_icons();
        self.sync(None, true, Announce::Nothing);
        self.relayout(false);
        unsafe {
            let mut cues = BOOL(0);
            let _ = SystemParametersInfoW(
                SPI_GETKEYBOARDCUES,
                0,
                Some((&mut cues as *mut BOOL).cast()),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
            if !cues.as_bool() {
                send(
                    main,
                    WM_CHANGEUISTATE,
                    makelong(UIS_SET as u16, (UISF_HIDEFOCUS | UISF_HIDEACCEL) as u16) as usize,
                    0,
                );
            }
        }
        Ok(())
    }

    fn child(
        self: &Rc<Self>,
        parent: HWND,
        class: PCWSTR,
        text: &str,
        style: WINDOW_STYLE,
        id: u16,
    ) -> Result<HWND> {
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                &HSTRING::from(text),
                style,
                0,
                0,
                0,
                0,
                Some(parent),
                Some(HMENU(usize::from(id) as *mut _)),
                Some(self.instance),
                None,
            )?;
            let raw = Rc::into_raw(self.clone());
            if !SetWindowSubclass(hwnd, Some(control_proc), usize::from(id), raw as usize).as_bool()
            {
                drop(Rc::from_raw(raw));
            }
            Ok(hwnd)
        }
    }

    fn create_tooltip(&self, main: HWND, canvas: HWND, handles: &Handles) -> HWND {
        unsafe {
            let Ok(tooltip) = CreateWindowExW(
                WS_EX_TOPMOST,
                TOOLTIPS_CLASSW,
                PCWSTR::null(),
                WS_POPUP | WINDOW_STYLE(TTS_ALWAYSTIP | TTS_NOPREFIX),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                Some(main),
                None,
                Some(self.instance),
                None,
            ) else {
                return HWND::default();
            };
            let _ = SetWindowPos(
                tooltip,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            add_tool(tooltip, canvas, handles.author, model::AUTHOR_TOOLTIP);
            for value in handles.values {
                add_tool(tooltip, canvas, value, "");
            }
            tooltip
        }
    }

    fn apply_fonts(&self, dpi: u32) {
        let input = GdiFont::for_role(Font::Input, dpi);
        let value = GdiFont::for_role(Font::Value, dpi);
        let input_metrics = input.as_ref().map(GdiFont::metrics).unwrap_or_default();
        let preferred = |role: Font, metrics: Option<TextMetrics>| {
            fonts::preferred_height(role.pixels(dpi))
                .filter(|_| fonts::register_gdi())
                .unwrap_or_else(|| metrics.map_or(role.pixels(dpi) * 3 / 2, |metric| metric.height))
        };
        let metrics = Metrics {
            input_height: preferred(Font::Input, Some(input_metrics)),
            value_height: preferred(Font::Value, value.as_ref().map(GdiFont::metrics)),
        };
        if let Ok(mut view) = self.view.try_borrow_mut() {
            view.input_font = input;
            view.value_font = value;
            view.input_metrics = input_metrics;
            view.metrics = metrics;
        }
    }

    fn send_fonts(&self) {
        let handles = self.handles.get();
        let (input, value) = match self.view.try_borrow() {
            Ok(view) => (
                view.input_font
                    .as_ref()
                    .map_or(0, |font| font.handle().0 as usize),
                view.value_font
                    .as_ref()
                    .map_or(0, |font| font.handle().0 as usize),
            ),
            Err(_) => return,
        };
        for row in handles.inputs {
            for edit in row {
                set_edit_font(edit, input);
            }
        }
        for edit in handles.values {
            set_edit_font(edit, value);
        }
        set_edit_font(handles.general, input);
    }

    fn update_icons(&self) {
        let handles = self.handles.get();
        let icons = Icons::load(self.instance, self.dpi());
        send(
            handles.main,
            WM_SETICON,
            ICON_BIG as usize,
            icons.big.0 as isize,
        );
        send(
            handles.main,
            WM_SETICON,
            ICON_SMALL as usize,
            icons.small.0 as isize,
        );
        if let Ok(mut view) = self.view.try_borrow_mut() {
            view.icons = Some(icons);
        }
    }

    fn annotate_all(&self, handles: &Handles) {
        let service: Option<IAccPropServices> =
            unsafe { CoCreateInstance(&CAccPropServices, None, CLSCTX_INPROC_SERVER).ok() };
        let Some(service) = service else {
            return;
        };
        let mut annotated = Vec::new();
        let mut live = 0;
        unsafe {
            let mut name = |hwnd: HWND, text: &str| {
                if service
                    .SetHwndPropStr(
                        hwnd,
                        OBJID_CLIENT.0 as u32,
                        CHILDID_SELF,
                        PROPID_ACC_NAME,
                        &HSTRING::from(text),
                    )
                    .is_ok()
                    && !annotated.contains(&hwnd)
                {
                    annotated.push(hwnd);
                }
            };
            for row in 0..3 {
                for column in 0..4 {
                    name(
                        handles.inputs[row][column],
                        &model::accessible_input_name(row, column),
                    );
                }
            }
            for index in 0..3 {
                name(handles.values[index], &model::accessible_value_name(index));
            }
            if service
                .SetHwndPropStr(
                    handles.author,
                    OBJID_CLIENT.0 as u32,
                    CHILDID_SELF,
                    PROPID_ACC_DESCRIPTION,
                    &HSTRING::from(model::AUTHOR_DESCRIPTION),
                )
                .is_ok()
                && !annotated.contains(&handles.author)
            {
                annotated.push(handles.author);
            }
            let variant = live_variant();
            for label in Label::ALL.into_iter().filter(|label| label.live()) {
                let hwnd = handles.label(label);
                if service
                    .SetHwndProp(
                        hwnd,
                        OBJID_CLIENT.0 as u32,
                        CHILDID_SELF,
                        LiveSetting_Property_GUID,
                        &variant,
                    )
                    .is_ok()
                {
                    live += 1;
                    if !annotated.contains(&hwnd) {
                        annotated.push(hwnd);
                    }
                }
            }
        }
        if let Ok(mut list) = self.annotated.try_borrow_mut() {
            *list = annotated;
        }
        self.live.set(live);
        if let Ok(mut slot) = self.accessibility.try_borrow_mut() {
            *slot = Some(service);
        }
    }

    fn set_accessible_name(&self, hwnd: HWND, text: &str) {
        let Ok(service) = self.accessibility.try_borrow() else {
            return;
        };
        if let Some(service) = service.as_ref() {
            unsafe {
                let _ = service.SetHwndPropStr(
                    hwnd,
                    OBJID_CLIENT.0 as u32,
                    CHILDID_SELF,
                    PROPID_ACC_NAME,
                    &HSTRING::from(text),
                );
            }
            if let Ok(mut list) = self.annotated.try_borrow_mut() {
                if !list.contains(&hwnd) {
                    list.push(hwnd);
                }
            }
        }
    }

    fn clear_annotations(&self) {
        let service = self
            .accessibility
            .try_borrow_mut()
            .ok()
            .and_then(|mut slot| slot.take());
        let annotated = self
            .annotated
            .try_borrow_mut()
            .map(|mut list| std::mem::take(&mut *list))
            .unwrap_or_default();
        if let Some(service) = service {
            for hwnd in annotated {
                unsafe {
                    let _ = service.ClearHwndProps(
                        hwnd,
                        OBJID_CLIENT.0 as u32,
                        CHILDID_SELF,
                        &[
                            PROPID_ACC_NAME,
                            PROPID_ACC_DESCRIPTION,
                            LiveSetting_Property_GUID,
                        ],
                    );
                }
            }
        }
    }

    fn announce(&self, hwnd: HWND) {
        unsafe {
            if !hwnd.is_invalid()
                && IsWindowVisible(hwnd).as_bool()
                && GetWindowTextLengthW(hwnd) > 0
            {
                NotifyWinEvent(
                    EVENT_OBJECT_LIVEREGIONCHANGED,
                    hwnd,
                    OBJID_CLIENT.0,
                    CHILDID_SELF as i32,
                );
            }
        }
    }

    fn track_normal(&self, hwnd: HWND) {
        unsafe {
            if IsIconic(hwnd).as_bool() || IsZoomed(hwnd).as_bool() {
                return;
            }
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok()
                && rect.right > rect.left
                && rect.bottom > rect.top
            {
                self.normal
                    .set(Some((bounds_of(rect), GetDpiForWindow(hwnd).max(48))));
            }
        }
    }

    fn save_settings(&self, hwnd: HWND) {
        let Some(key) = self.settings_key.as_deref() else {
            return;
        };
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        let have_placement = unsafe { GetWindowPlacement(hwnd, &mut placement) }.is_ok();
        let maximized = have_placement
            && (placement.showCmd == SW_SHOWMAXIMIZED.0 as u32
                || (placement.showCmd == SW_SHOWMINIMIZED.0 as u32
                    && (placement.flags.0 & WPF_RESTORETOMAXIMIZED.0) != 0));
        let normal = self.normal.get().or_else(|| {
            if !have_placement {
                return None;
            }
            let mut rect = placement.rcNormalPosition;
            let monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONEAREST) };
            let mut info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
                let dx = info.rcWork.left - info.rcMonitor.left;
                let dy = info.rcWork.top - info.rcMonitor.top;
                rect = RECT {
                    left: rect.left + dx,
                    top: rect.top + dy,
                    right: rect.right + dx,
                    bottom: rect.bottom + dy,
                };
            }
            Some((bounds_of(rect), unsafe { GetDpiForWindow(hwnd) }.max(48)))
        });
        let dimension = self.model.try_borrow().map(|model| model.dimension()).ok();
        let saved = Saved {
            placement: normal.map(|(bounds, dpi)| Placement {
                bounds,
                maximized,
                dpi,
            }),
            dimension,
        };
        settings::save(key, &saved);
    }

    fn refresh_theme(&self, main: HWND) {
        if self.theming.get() {
            return;
        }
        let high_contrast = self.contrast.get().unwrap_or_else(system_high_contrast);
        let palette = palette_for(high_contrast);
        if palette == self.palette.get() {
            return;
        }
        self.palette.set(palette);
        self.apply_frame_theme(main);
        if let Ok(mut brushes) = self.brushes.try_borrow_mut() {
            brushes.clear();
        }
        unsafe {
            let _ = RedrawWindow(
                Some(main),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN,
            );
        }
    }

    fn main_message(&self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_SIZE => {
                    if wparam.0 as u32 != SIZE_MINIMIZED {
                        self.relayout(true);
                    }
                    LRESULT(0)
                }
                WM_WINDOWPOSCHANGED => {
                    let result = DefWindowProcW(hwnd, message, wparam, lparam);
                    self.track_normal(hwnd);
                    result
                }
                WM_GETMINMAXINFO => {
                    if let Some(info) = (lparam.0 as *mut MINMAXINFO).as_mut() {
                        let (width, height) = minimum_window(GetDpiForWindow(hwnd).max(48));
                        info.ptMinTrackSize = POINT {
                            x: width,
                            y: height,
                        };
                    }
                    LRESULT(0)
                }
                WM_DPICHANGED => {
                    let suggested = (lparam.0 as *const RECT).as_ref().copied();
                    self.dpi_changed(hwnd, u32::from(loword(wparam.0)), suggested);
                    LRESULT(0)
                }
                WM_VSCROLL | WM_HSCROLL => {
                    self.scroll_bar(message == WM_VSCROLL, i32::from(loword(wparam.0)));
                    LRESULT(0)
                }
                WM_MOUSEWHEEL => {
                    let delta = i32::from(hiword(wparam.0) as i16);
                    let shift = (u32::from(loword(wparam.0)) & MK_SHIFT.0) != 0;
                    self.wheel(delta, shift);
                    LRESULT(0)
                }
                WM_MOUSEHWHEEL => {
                    let delta = i32::from(hiword(wparam.0) as i16);
                    self.wheel(-delta, true);
                    LRESULT(0)
                }
                WM_COMMAND => {
                    self.command(loword(wparam.0), hiword(wparam.0));
                    LRESULT(0)
                }
                DM_GETDEFID => LRESULT(makelong(ID_SOLVE, DC_HASDEFID as u16) as isize),
                WM_ACTIVATE => {
                    if u32::from(loword(wparam.0)) == WA_INACTIVE {
                        self.remember_focus();
                        DefWindowProcW(hwnd, message, wparam, lparam)
                    } else if hiword(wparam.0) == 0 {
                        self.restore_focus();
                        LRESULT(0)
                    } else {
                        DefWindowProcW(hwnd, message, wparam, lparam)
                    }
                }
                WM_SETFOCUS => {
                    self.restore_focus();
                    LRESULT(0)
                }
                WM_ERASEBKGND => {
                    let mut client = RECT::default();
                    let _ = GetClientRect(hwnd, &mut client);
                    FillRect(
                        HDC(wparam.0 as *mut _),
                        &client,
                        self.brush(self.palette.get().canvas),
                    );
                    LRESULT(1)
                }
                WM_PAINT => {
                    let mut paint = PAINTSTRUCT::default();
                    let hdc = BeginPaint(hwnd, &mut paint);
                    FillRect(hdc, &paint.rcPaint, self.brush(self.palette.get().canvas));
                    let _ = EndPaint(hwnd, &paint);
                    LRESULT(0)
                }
                WM_PRINTCLIENT => {
                    let mut client = RECT::default();
                    let _ = GetClientRect(hwnd, &mut client);
                    FillRect(
                        HDC(wparam.0 as *mut _),
                        &client,
                        self.brush(self.palette.get().canvas),
                    );
                    LRESULT(0)
                }
                WM_THEMECHANGED | WM_SETTINGCHANGE | WM_SYSCOLORCHANGE => {
                    self.refresh_theme(hwnd);
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
                WM_ENDSESSION => {
                    if wparam.0 != 0 {
                        self.save_settings(hwnd);
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    self.save_settings(hwnd);
                    self.clear_annotations();
                    if self.quit_on_destroy {
                        PostQuitMessage(0);
                    }
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, message, wparam, lparam),
            }
        }
    }

    fn canvas_message(&self, hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    let mut paint = PAINTSTRUCT::default();
                    let hdc = BeginPaint(hwnd, &mut paint);
                    self.render_canvas(hdc, paint.rcPaint);
                    let _ = EndPaint(hwnd, &paint);
                    LRESULT(0)
                }
                WM_PRINTCLIENT => {
                    let mut client = RECT::default();
                    let _ = GetClientRect(hwnd, &mut client);
                    self.render_canvas(HDC(wparam.0 as *mut _), client);
                    LRESULT(0)
                }
                WM_ERASEBKGND => LRESULT(1),
                WM_COMMAND => {
                    self.command(loword(wparam.0), hiword(wparam.0));
                    LRESULT(0)
                }
                WM_DRAWITEM => {
                    if let Some(item) = (lparam.0 as *const DRAWITEMSTRUCT).as_ref() {
                        self.draw_item(item);
                    }
                    LRESULT(1)
                }
                WM_CTLCOLOREDIT | WM_CTLCOLORSTATIC => {
                    self.control_color(HDC(wparam.0 as *mut _), HWND(lparam.0 as *mut _))
                }
                WM_LBUTTONDOWN => {
                    let (x, y) = point_from(lparam);
                    self.canvas_click(x, y);
                    LRESULT(0)
                }
                WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                    SendMessageW(self.handles.get().main, message, Some(wparam), Some(lparam))
                }
                WM_CLOSE => LRESULT(0),
                _ => DefWindowProcW(hwnd, message, wparam, lparam),
            }
        }
    }

    fn control_message(
        &self,
        hwnd: HWND,
        id: u16,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let control = classify(id);
        unsafe {
            match (control, message) {
                (Some(Control::Button(button)), WM_MOUSEMOVE) => {
                    let slot = &self.hover[button.index()];
                    if !slot.get() {
                        slot.set(true);
                        let mut track = TRACKMOUSEEVENT {
                            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            dwHoverTime: 0,
                        };
                        let _ = TrackMouseEvent(&mut track);
                        invalidate(hwnd);
                    }
                }
                (Some(Control::Button(button)), WM_MOUSELEAVE) => {
                    self.hover[button.index()].set(false);
                    invalidate(hwnd);
                }
                (Some(Control::Button(button)), WM_ENABLE) => {
                    if wparam.0 == 0 {
                        self.hover[button.index()].set(false);
                    }
                }
                (Some(Control::Button(_)), WM_SETCURSOR) => {
                    if u32::from(loword(lparam.0 as usize)) == HTCLIENT
                        && IsWindowEnabled(hwnd).as_bool()
                    {
                        if let Ok(cursor) = LoadCursorW(None, IDC_HAND) {
                            SetCursor(Some(cursor));
                            return LRESULT(1);
                        }
                    }
                }
                (Some(Control::Button(_)) | Some(Control::Label(_)), WM_ERASEBKGND) => {
                    return LRESULT(1);
                }
                (Some(Control::Button(_)), BM_SETSTYLE) => {
                    let style = (wparam.0 & !(BS_TYPEMASK as usize)) | BS_OWNERDRAW as usize;
                    return DefSubclassProc(hwnd, message, WPARAM(style), lparam);
                }
                (Some(Control::Button(_)), WM_UPDATEUISTATE) => {
                    let result = DefSubclassProc(hwnd, message, wparam, lparam);
                    invalidate(hwnd);
                    return result;
                }
                (
                    Some(
                        Control::Button(_)
                        | Control::Input(..)
                        | Control::Value(_)
                        | Control::General,
                    ),
                    WM_SETFOCUS | WM_KILLFOCUS,
                ) => {
                    let result = DefSubclassProc(hwnd, message, wparam, lparam);
                    if message == WM_SETFOCUS && !matches!(control, Some(Control::Button(_))) {
                        fit_caret(hwnd);
                    }
                    self.focus_changed(hwnd, message == WM_SETFOCUS);
                    return result;
                }
                (Some(Control::Input(..) | Control::Value(_) | Control::General), WM_SIZE) => {
                    let result = DefSubclassProc(hwnd, message, wparam, lparam);
                    fit_caret(hwnd);
                    return result;
                }
                (Some(Control::Input(..) | Control::Value(_) | Control::General), WM_KEYDOWN) => {
                    let control_down = GetKeyState(i32::from(VK_CONTROL.0)) < 0;
                    if wparam.0 == usize::from(b'A') && control_down {
                        send(hwnd, EM_SETSEL, 0, -1);
                        return LRESULT(0);
                    }
                    if wparam.0 == usize::from(VK_BACK.0)
                        && control_down
                        && matches!(control, Some(Control::Input(..)))
                    {
                        delete_previous_word(hwnd);
                        return LRESULT(0);
                    }
                    if matches!(control, Some(Control::General))
                        && (wparam.0 == usize::from(VK_ESCAPE.0)
                            || wparam.0 == usize::from(VK_RETURN.0))
                    {
                        return LRESULT(0);
                    }
                }
                (Some(Control::Input(..) | Control::Value(_) | Control::General), WM_CHAR) => {
                    if wparam.0 == 1 || wparam.0 == 0x7F {
                        return LRESULT(0);
                    }
                    if matches!(control, Some(Control::General))
                        && (wparam.0 == 0x1B || wparam.0 == 0x0D)
                    {
                        return LRESULT(0);
                    }
                }
                (Some(Control::General), WM_GETDLGCODE) => {
                    let code = DefSubclassProc(hwnd, message, wparam, lparam).0 as u32;
                    return LRESULT((code & !(DLGC_WANTALLKEYS | DLGC_WANTTAB)) as isize);
                }
                _ => {}
            }
            DefSubclassProc(hwnd, message, wparam, lparam)
        }
    }

    fn command(&self, id: u16, code: u16) {
        match classify(id) {
            Some(Control::Button(button)) if u32::from(code) == BN_CLICKED => match button {
                Button::Two => self.change_dimension(2),
                Button::Three => self.change_dimension(3),
                Button::Solve => self.solve(),
                Button::Example => self.fill_example(),
                Button::Clear => self.clear(),
                Button::Copy => self.copy(),
                Button::Author => self.open_author(),
            },
            Some(Control::Input(row, column)) if u32::from(code) == EN_CHANGE => {
                self.input_changed(row, column)
            }
            _ => {}
        }
    }

    fn change_dimension(&self, dimension: usize) {
        match self.model.try_borrow_mut() {
            Ok(mut model) => model.change_dimension(dimension),
            Err(_) => return,
        }
        self.sync(None, false, Announce::Label(Label::Dimension));
        self.relayout(true);
        let handles = self.handles.get();
        invalidate(handles.two);
        invalidate(handles.three);
    }

    fn read_inputs(&self) {
        let handles = self.handles.get();
        let texts: Vec<(usize, usize, String)> = (0..3)
            .flat_map(|row| (0..4).map(move |column| (row, column)))
            .map(|(row, column)| (row, column, window_text(handles.inputs[row][column])))
            .collect();
        if let Ok(mut model) = self.model.try_borrow_mut() {
            for (row, column, text) in texts {
                model.store_input(row, column, &text);
            }
        }
    }

    fn solve(&self) {
        self.read_inputs();
        let focus = match self.model.try_borrow_mut() {
            Ok(mut model) => model.solve(),
            Err(_) => return,
        };
        let announce = if focus.is_some() {
            Announce::Label(Label::Helper)
        } else {
            Announce::Outcome
        };
        self.sync(focus, false, announce);
    }

    fn fill_example(&self) {
        let focus = match self.model.try_borrow_mut() {
            Ok(mut model) => model.fill_example(),
            Err(_) => return,
        };
        self.sync(Some(focus), true, Announce::Label(Label::Helper));
    }

    fn clear(&self) {
        let focus = match self.model.try_borrow_mut() {
            Ok(mut model) => model.clear(),
            Err(_) => return,
        };
        self.sync(Some(focus), true, Announce::Nothing);
    }

    fn copy(&self) {
        let text = self
            .model
            .try_borrow()
            .ok()
            .and_then(|model| model.last_result().map(str::to_owned));
        let Some(text) = text else {
            return;
        };
        let copied = clipboard::set_text(self.handles.get().main, &text);
        if let Ok(mut model) = self.model.try_borrow_mut() {
            model.copy(|_| copied);
        }
        self.sync(
            None,
            false,
            if copied {
                Announce::Nothing
            } else {
                Announce::Label(Label::Note)
            },
        );
    }

    fn open_author(&self) {
        let main = self.handles.get().main;
        let result = unsafe {
            ShellExecuteW(
                Some(main),
                w!("open"),
                &HSTRING::from(model::AUTHOR_URL),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        if result.0 as isize <= 32 {
            if let Ok(mut model) = self.model.try_borrow_mut() {
                model.author_unavailable();
            }
            self.sync(None, false, Announce::Label(Label::Helper));
        }
    }

    fn input_changed(&self, row: usize, column: usize) {
        if self.syncing.get() {
            return;
        }
        let text = window_text(self.handles.get().inputs[row][column]);
        match self.model.try_borrow_mut() {
            Ok(mut model) => model.set_input(row, column, &text),
            Err(_) => return,
        }
        self.sync(None, false, Announce::Nothing);
    }

    fn sync(&self, focus: Option<Focus>, push_inputs: bool, announce: Announce) {
        let handles = self.handles.get();
        if handles.main.is_invalid() {
            return;
        }
        let Ok(model) = self.model.try_borrow().map(|model| model.clone()) else {
            return;
        };
        if push_inputs {
            self.syncing.set(true);
            for row in 0..3 {
                for column in 0..4 {
                    set_text(handles.inputs[row][column], model.input(row, column));
                }
            }
            self.syncing.set(false);
        }
        set_text(handles.dimension, &model.dimension_text());
        set_text(handles.helper, model.helper());
        invalidate(handles.helper);
        set_text(handles.title, model.title().unwrap_or(""));
        if let Some(text) = model.announcement() {
            self.set_accessible_name(handles.title, &text);
        }
        set_text(handles.description, model.description().unwrap_or(""));
        set_text(handles.note, model.note());
        set_text(handles.copy, model.copy_label());
        unsafe {
            if IsWindowEnabled(handles.copy).as_bool() != model.copy_enabled() {
                let _ = EnableWindow(handles.copy, model.copy_enabled());
            }
        }
        invalidate(handles.copy);
        for index in 0..3 {
            let value = model.values().get(index).map_or("", String::as_str);
            if set_text(handles.values[index], value) {
                send(handles.values[index], EM_SETSEL, 0, 0);
                update_tool(
                    handles.tooltip,
                    handles.canvas,
                    handles.values[index],
                    value,
                );
            }
        }
        set_text(handles.general, model.general().unwrap_or(""));
        self.place_controls();
        self.update_general_scrollbar();
        for label in Label::ALL {
            invalidate(handles.label(label));
        }
        invalidate(handles.canvas);
        invalidate(handles.two);
        invalidate(handles.three);
        match focus {
            Some(Focus::Field {
                row,
                column,
                select_all,
            }) => {
                let edit = handles.inputs[row][column];
                unsafe {
                    let _ = SetFocus(Some(edit));
                }
                if select_all {
                    send(edit, EM_SETSEL, 0, -1);
                }
            }
            Some(Focus::Solve) => unsafe {
                let _ = SetFocus(Some(handles.solve));
            },
            None => {}
        }
        match announce {
            Announce::Nothing => {}
            Announce::Label(label) => self.announce(handles.label(label)),
            Announce::Outcome => self.announce(handles.title),
        }
    }

    fn update_general_scrollbar(&self) {
        let handles = self.handles.get();
        let line = self
            .view
            .try_borrow()
            .map_or(1, |view| view.input_metrics.height.max(1));
        let lines = send(handles.general, EM_GETLINECOUNT, 0, 0).0 as i32;
        unsafe {
            let mut window = RECT::default();
            let _ = GetWindowRect(handles.general, &mut window);
            let height = window.bottom - window.top;
            let needed = lines * line > height && GetWindowTextLengthW(handles.general) > 0;
            let _ = ShowScrollBar(handles.general, SB_VERT, needed);
        }
    }

    fn place_controls(&self) {
        let handles = self.handles.get();
        if handles.canvas.is_invalid() {
            return;
        }
        let layout = self.layout();
        let Ok((dimension, outcome)) = self
            .model
            .try_borrow()
            .map(|model| (model.dimension(), model.outcome().clone()))
        else {
            return;
        };
        let empty = outcome == Outcome::Empty;
        let values = match &outcome {
            Outcome::Unique(values) => values.len(),
            _ => 0,
        };
        let general = matches!(outcome, Outcome::Infinite(_) | Outcome::NoSolution(_));
        let mut entries: Vec<(HWND, Rect, bool)> = Vec::with_capacity(40);
        for button in Button::ALL {
            entries.push((handles.button(button), button.rect(&layout), true));
        }
        for row in 0..3 {
            for column in 0..4 {
                let visible = layout::field_visible(dimension, row, column);
                let rect = if visible {
                    layout.editors[row][column]
                } else {
                    Rect::default()
                };
                entries.push((handles.inputs[row][column], rect, visible));
            }
        }
        for label in Label::ALL {
            let visible = match label {
                Label::Title | Label::Description => !empty,
                Label::EmptyTitle | Label::EmptyDescription => empty,
                _ => true,
            };
            entries.push((handles.label(label), label.rect(&layout), visible));
        }
        for index in 0..3 {
            entries.push((
                handles.values[index],
                layout.tile_values[index],
                index < values,
            ));
        }
        entries.push((handles.general, layout.general, general));
        let focus = unsafe { GetFocus() };
        let focus_hidden = entries
            .iter()
            .any(|(hwnd, _, visible)| !*visible && *hwnd == focus);
        unsafe {
            let mut batch = BeginDeferWindowPos(entries.len() as i32).ok();
            for (hwnd, rect, visible) in &entries {
                let flags = SWP_NOZORDER
                    | SWP_NOACTIVATE
                    | SWP_NOOWNERZORDER
                    | if *visible {
                        SWP_SHOWWINDOW
                    } else {
                        SWP_HIDEWINDOW
                    };
                let (width, height) = (rect.w.max(0), rect.h.max(0));
                batch = match batch {
                    Some(current) => match DeferWindowPos(
                        current, *hwnd, None, rect.x, rect.y, width, height, flags,
                    ) {
                        Ok(next) => Some(next),
                        Err(_) => {
                            let _ = SetWindowPos(*hwnd, None, rect.x, rect.y, width, height, flags);
                            None
                        }
                    },
                    None => {
                        let _ = SetWindowPos(*hwnd, None, rect.x, rect.y, width, height, flags);
                        None
                    }
                };
            }
            if let Some(batch) = batch {
                let _ = EndDeferWindowPos(batch);
            }
        }
        if focus_hidden {
            unsafe {
                let _ = SetFocus(Some(handles.inputs[0][0]));
            }
        }
    }

    fn relayout(&self, ensure_focus: bool) {
        let handles = self.handles.get();
        if handles.canvas.is_invalid() || self.in_layout.replace(true) {
            return;
        }
        let (dpi, scale) = (self.dpi(), self.scale());
        let bar_w = unsafe { GetSystemMetricsForDpi(SM_CXVSCROLL, dpi) };
        let bar_h = unsafe { GetSystemMetricsForDpi(SM_CYHSCROLL, dpi) };
        let dimension = self.model.try_borrow().map_or(2, |model| model.dimension());
        for _ in 0..3 {
            let mut client = RECT::default();
            let style = unsafe {
                let _ = GetClientRect(handles.main, &mut client);
                GetWindowLongW(handles.main, GWL_STYLE) as u32
            };
            let full_w = client.right + if style & WS_VSCROLL.0 != 0 { bar_w } else { 0 };
            let full_h = client.bottom + if style & WS_HSCROLL.0 != 0 { bar_h } else { 0 };
            let viewport = layout::viewport(scale, full_w, full_h, bar_w, bar_h);
            let scroll = match self.view.try_borrow_mut() {
                Ok(mut view) => {
                    let (max_x, max_y) = viewport.max_scroll();
                    view.scroll = (view.scroll.0.clamp(0, max_x), view.scroll.1.clamp(0, max_y));
                    view.layout = layout::compute(
                        scale,
                        viewport.width,
                        viewport.height,
                        dimension,
                        view.metrics,
                    );
                    view.viewport = viewport;
                    view.scroll
                }
                Err(_) => break,
            };
            set_scroll(
                handles.main,
                SB_HORZ,
                viewport.content_w,
                viewport.client_w,
                scroll.0,
            );
            set_scroll(
                handles.main,
                SB_VERT,
                viewport.content_h,
                viewport.client_h,
                scroll.1,
            );
            let mut after = RECT::default();
            unsafe {
                let _ = GetClientRect(handles.main, &mut after);
            }
            if after.right == viewport.client_w && after.bottom == viewport.client_h {
                break;
            }
        }
        let (scroll, canvas_size) = self
            .view
            .try_borrow()
            .map(|view| (view.scroll, view.viewport.canvas_size()))
            .unwrap_or_default();
        unsafe {
            let _ = SetWindowPos(
                handles.canvas,
                None,
                -scroll.0,
                -scroll.1,
                canvas_size.0,
                canvas_size.1,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        self.place_controls();
        self.update_general_scrollbar();
        invalidate(handles.canvas);
        self.in_layout.set(false);
        if ensure_focus {
            self.ensure_focus_visible();
        }
    }

    fn focus_rect(&self, hwnd: HWND) -> Option<Rect> {
        let handles = self.handles.get();
        let layout = self.layout();
        match handles.control(hwnd)? {
            Control::Button(button) => Some(button.rect(&layout)),
            Control::Input(row, column) => Some(layout.fields[row][column]),
            Control::Value(index) => Some(layout.tiles[index]),
            Control::General => Some(layout.general),
            Control::Label(label) => Some(label.rect(&layout)),
        }
    }

    fn ensure_focus_visible(&self) {
        let focus = unsafe { GetFocus() };
        if let Some(rect) = self.focus_rect(focus) {
            self.scroll_into_view(rect);
        }
    }

    fn scroll_into_view(&self, rect: Rect) {
        let Ok((scroll, client)) = self.view.try_borrow().map(|view| {
            (
                view.scroll,
                (view.viewport.client_w, view.viewport.client_h),
            )
        }) else {
            return;
        };
        let axis = |position: i32, size: i32, current: i32, visible: i32| {
            let relative = position - current;
            if relative < 0 {
                position
            } else if relative + size > visible {
                let target = position + size - visible;
                if position - target < 0 {
                    position
                } else {
                    target
                }
            } else {
                current
            }
        };
        let x = axis(rect.x, rect.w, scroll.0, client.0);
        let y = axis(rect.y, rect.h, scroll.1, client.1);
        self.scroll_to(x, y);
    }

    fn scroll_to(&self, x: i32, y: i32) {
        let handles = self.handles.get();
        let target = {
            let Ok(mut view) = self.view.try_borrow_mut() else {
                return;
            };
            let (max_x, max_y) = view.viewport.max_scroll();
            let target = (x.clamp(0, max_x), y.clamp(0, max_y));
            if target == view.scroll {
                return;
            }
            view.scroll = target;
            target
        };
        unsafe {
            let info = SCROLLINFO {
                cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                fMask: SIF_POS,
                nPos: target.0,
                ..Default::default()
            };
            SetScrollInfo(handles.main, SB_HORZ, &info, true);
            let info = SCROLLINFO {
                cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                fMask: SIF_POS,
                nPos: target.1,
                ..Default::default()
            };
            SetScrollInfo(handles.main, SB_VERT, &info, true);
            let _ = SetWindowPos(
                handles.canvas,
                None,
                -target.0,
                -target.1,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    fn scroll_bar(&self, vertical: bool, code: i32) {
        let handles = self.handles.get();
        let Ok((scroll, viewport, scale)) = self
            .view
            .try_borrow()
            .map(|view| (view.scroll, view.viewport, view.scale))
        else {
            return;
        };
        let (position, page, max) = if vertical {
            (scroll.1, viewport.client_h, viewport.max_scroll().1)
        } else {
            (scroll.0, viewport.client_w, viewport.max_scroll().0)
        };
        let line = d(scale, 20).max(1);
        let target = match SCROLLBAR_COMMAND(code) {
            SB_LINEUP => position - line,
            SB_LINEDOWN => position + line,
            SB_PAGEUP => position - page,
            SB_PAGEDOWN => position + page,
            SB_TOP => 0,
            SB_BOTTOM => max,
            SB_THUMBTRACK | SB_THUMBPOSITION => {
                let mut info = SCROLLINFO {
                    cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_TRACKPOS,
                    ..Default::default()
                };
                unsafe {
                    if GetScrollInfo(
                        handles.main,
                        if vertical { SB_VERT } else { SB_HORZ },
                        &mut info,
                    )
                    .is_ok()
                    {
                        info.nTrackPos
                    } else {
                        position
                    }
                }
            }
            _ => position,
        };
        if vertical {
            self.scroll_to(scroll.0, target)
        } else {
            self.scroll_to(target, scroll.1)
        }
    }

    fn wheel(&self, delta: i32, horizontal_request: bool) {
        let Ok((scroll, viewport, scale, remainder)) = self
            .view
            .try_borrow()
            .map(|view| (view.scroll, view.viewport, view.scale, view.wheel))
        else {
            return;
        };
        let horizontal = horizontal_request || (!viewport.vertical && viewport.horizontal);
        let mut lines = 3u32;
        unsafe {
            let _ = SystemParametersInfoW(
                if horizontal {
                    SPI_GETWHEELSCROLLCHARS
                } else {
                    SPI_GETWHEELSCROLLLINES
                },
                0,
                Some((&mut lines as *mut u32).cast()),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
        }
        let step = if lines == u32::MAX {
            if horizontal {
                viewport.client_w
            } else {
                viewport.client_h
            }
        } else {
            d(scale, 120) * lines.min(100) as i32 / 3
        };
        let accumulated = if horizontal { remainder.0 } else { remainder.1 } + delta * step;
        let pixels = accumulated / 120;
        let left = accumulated % 120;
        if let Ok(mut view) = self.view.try_borrow_mut() {
            view.wheel = if horizontal { (left, 0) } else { (0, left) };
        }
        if pixels == 0 {
            return;
        }
        if horizontal {
            self.scroll_to(scroll.0 - pixels, scroll.1)
        } else {
            self.scroll_to(scroll.0, scroll.1 - pixels)
        }
    }

    fn dpi_changed(&self, hwnd: HWND, dpi: u32, suggested: Option<RECT>) {
        let dpi = dpi.max(48);
        self.adopt_dpi(dpi);
        if let Ok(mut engine) = self.engine.try_borrow_mut() {
            if let Some(engine) = engine.as_mut() {
                let _ = engine.set_dpi(dpi);
            }
        }
        if let Ok(mut fallback) = self.fallback.try_borrow_mut() {
            *fallback = None;
        }
        let previous = match self.view.try_borrow_mut() {
            Ok(mut view) => (view.input_font.take(), view.value_font.take()),
            Err(_) => (None, None),
        };
        self.apply_fonts(dpi);
        self.send_fonts();
        drop(previous);
        self.update_icons();
        if let Some(rect) = suggested {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        self.relayout(true);
        unsafe {
            let _ = RedrawWindow(
                Some(hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN,
            );
        }
    }

    fn remember_focus(&self) {
        let focus = unsafe { GetFocus() };
        if self.handles.get().control(focus).is_some() {
            self.last_focus.set(focus);
        }
    }

    fn restore_focus(&self) {
        let handles = self.handles.get();
        let saved = self.last_focus.get();
        let usable = |hwnd: HWND| unsafe {
            !hwnd.is_invalid()
                && IsWindow(Some(hwnd)).as_bool()
                && IsWindowVisible(hwnd).as_bool()
                && IsWindowEnabled(hwnd).as_bool()
        };
        let target = if usable(saved) {
            saved
        } else {
            handles.inputs[0][0]
        };
        if !target.is_invalid() {
            unsafe {
                let _ = SetFocus(Some(target));
            }
        }
    }

    fn focus_changed(&self, hwnd: HWND, gained: bool) {
        let handles = self.handles.get();
        let control = handles.control(hwnd);
        if gained && control.is_some() {
            self.last_focus.set(hwnd);
            if !self.in_layout.get() {
                if let Some(rect) = self.focus_rect(hwnd) {
                    self.scroll_into_view(rect);
                }
            }
        }
        match control {
            Some(Control::Input(row, column)) => {
                let rect = self.layout().fields[row][column];
                unsafe {
                    let _ = InvalidateRect(Some(handles.canvas), Some(&win_rect(rect)), false);
                }
            }
            Some(Control::Button(_)) => invalidate(hwnd),
            _ => {}
        }
    }

    fn canvas_click(&self, x: i32, y: i32) {
        let handles = self.handles.get();
        let layout = self.layout();
        let dimension = self.model.try_borrow().map_or(2, |model| model.dimension());
        for row in 0..3 {
            for column in 0..4 {
                if layout::field_visible(dimension, row, column)
                    && layout.fields[row][column].contains_point(x, y)
                {
                    unsafe {
                        let _ = SetFocus(Some(handles.inputs[row][column]));
                    }
                    return;
                }
            }
        }
    }

    fn control_color(&self, hdc: HDC, control: HWND) -> LRESULT {
        let handles = self.handles.get();
        let palette = self.palette.get();
        let (text, back) = match handles.control(control) {
            Some(Control::Input(..)) => (palette.ink, palette.field),
            Some(Control::Value(_)) => (palette.solution_text, palette.tile_fill),
            Some(Control::General) => (palette.solution_text, palette.solution_fill),
            Some(Control::Label(label)) => {
                (label.color(&palette, false), label.background(&palette))
            }
            _ => (palette.ink, palette.canvas),
        };
        unsafe {
            SetTextColor(hdc, COLORREF(text.colorref()));
            SetBkColor(hdc, COLORREF(back.colorref()));
            SetBkMode(hdc, OPAQUE);
        }
        LRESULT(self.brush(back).0 as isize)
    }

    fn paint_with(&self, hdc: HDC, area: RECT, draw: &dyn Fn(&Painter)) {
        if self.renderer == Renderer::Direct2D {
            for _ in 0..2 {
                let status = match self.engine.try_borrow_mut() {
                    Ok(mut engine) => match engine.as_mut() {
                        Some(engine) => engine.paint(hdc, area, draw),
                        None => PaintStatus::Failed,
                    },
                    Err(_) => PaintStatus::Failed,
                };
                match status {
                    PaintStatus::Done => return,
                    PaintStatus::Recreate => continue,
                    PaintStatus::Failed => break,
                }
            }
        }
        let dpi = self.dpi();
        let Ok(mut fallback) = self.fallback.try_borrow_mut() else {
            return;
        };
        if fallback
            .as_ref()
            .is_none_or(|resources| resources.dpi() != dpi)
        {
            *fallback = Some(GdiResources::new(dpi));
        }
        if let Some(resources) = fallback.as_ref() {
            resources.paint(hdc, area, draw);
        }
    }

    fn render_canvas(&self, hdc: HDC, area: RECT) {
        let handles = self.handles.get();
        let focus = unsafe { GetFocus() };
        let layout = self.layout();
        let scale = self.scale();
        let palette = self.palette.get();
        let Ok(state) = self.model.try_borrow().map(|model| CanvasState {
            layout,
            scale,
            palette,
            dimension: model.dimension(),
            error: model.error_field(),
            focused: (0..3)
                .flat_map(|row| (0..4).map(move |column| (row, column)))
                .find(|(row, column)| handles.inputs[*row][*column] == focus),
            empty: model.is_empty(),
            values: model.values().len(),
        }) else {
            unsafe { FillRect(hdc, &area, self.brush(palette.canvas)) };
            return;
        };
        self.paint_with(hdc, area, &|painter| draw_canvas(painter, &state));
    }

    fn draw_item(&self, item: &DRAWITEMSTRUCT) {
        let Ok(id) = u16::try_from(item.CtlID) else {
            return;
        };
        let text = window_text(item.hwndItem);
        let scale = self.scale();
        let palette = self.palette.get();
        let (dimension, helper_error) = self.model.try_borrow().map_or((2, false), |model| {
            (model.dimension(), model.helper_is_error())
        });
        let width = item.rcItem.right - item.rcItem.left;
        let height = item.rcItem.bottom - item.rcItem.top;
        if item.CtlType == ODT_BUTTON {
            let Some(button) = Button::from_id(id) else {
                return;
            };
            let cues_hidden = (item.itemState.0 & ODS_NOFOCUSRECT.0) != 0
                || (send(item.hwndItem, WM_QUERYUISTATE, 0, 0).0 as u32 & UISF_HIDEFOCUS) != 0;
            let look = ButtonLook {
                button,
                text: &text,
                width,
                height,
                scale,
                palette,
                hover: self.hover[button.index()].get(),
                disabled: (item.itemState.0 & ODS_DISABLED.0) != 0,
                focused: (item.itemState.0 & ODS_FOCUS.0) != 0 && !cues_hidden,
                selected: (button == Button::Two && dimension == 2)
                    || (button == Button::Three && dimension == 3),
            };
            self.paint_with(item.hDC, item.rcItem, &|painter| {
                draw_button(painter, &look)
            });
        } else if item.CtlType == ODT_STATIC {
            let Some(label) = Label::from_id(id) else {
                return;
            };
            let (align, valign) = label.align();
            self.paint_with(item.hDC, item.rcItem, &|painter| {
                painter.clear(label.background(&palette));
                painter.text(
                    &text,
                    label.font(),
                    label.color(&palette, helper_error),
                    Rect::new(0, 0, width, height),
                    align,
                    valign,
                );
            });
        }
    }
}

fn delete_previous_word(hwnd: HWND) {
    let mut start = 0u32;
    let mut end = 0u32;
    send(
        hwnd,
        EM_GETSEL,
        (&mut start as *mut u32) as usize,
        (&mut end as *mut u32) as isize,
    );
    let empty: [u16; 1] = [0];
    if start != end {
        send(hwnd, EM_REPLACESEL, 1, empty.as_ptr() as isize);
        return;
    }
    let units: Vec<u16> = window_text(hwnd).encode_utf16().collect();
    let caret = (start as usize).min(units.len());
    let blank = |unit: u16| unit == 0x20 || unit == 0x09;
    let mut boundary = caret;
    while boundary > 0 && blank(units[boundary - 1]) {
        boundary -= 1;
    }
    while boundary > 0 && !blank(units[boundary - 1]) {
        boundary -= 1;
    }
    if boundary == caret {
        return;
    }
    send(hwnd, EM_SETSEL, boundary, caret as isize);
    send(hwnd, EM_REPLACESEL, 1, empty.as_ptr() as isize);
}

fn set_edit_font(edit: HWND, font: usize) {
    send(edit, WM_SETFONT, font, 1);
    send(
        edit,
        EM_SETMARGINS,
        (EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize,
        0,
    );
    fit_caret(edit);
}

fn fit_caret(edit: HWND) {
    unsafe {
        if GetFocus() != edit {
            return;
        }
        let mut client = RECT::default();
        if GetClientRect(edit, &mut client).is_err() || client.bottom <= 0 {
            return;
        }
        let font = HGDIOBJ(send(edit, WM_GETFONT, 0, 0).0 as *mut _);
        let dc = GetDC(Some(edit));
        if dc.is_invalid() {
            return;
        }
        let previous = SelectObject(dc, font);
        let mut metric = TEXTMETRICW::default();
        let measured = GetTextMetricsW(dc, &mut metric).as_bool();
        SelectObject(dc, previous);
        ReleaseDC(Some(edit), dc);
        if !measured || metric.tmHeight <= client.bottom {
            return;
        }
        let mut position = POINT::default();
        let _ = GetCaretPos(&mut position);
        let mut width = 1u32;
        let _ = SystemParametersInfoW(
            SPI_GETCARETWIDTH,
            0,
            Some((&mut width as *mut u32).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        let _ = DestroyCaret();
        if CreateCaret(edit, None, width.clamp(1, 20) as i32, client.bottom).is_ok() {
            let _ = SetCaretPos(position.x, position.y.max(0));
            let _ = ShowCaret(Some(edit));
        }
    }
}

fn set_scroll(hwnd: HWND, bar: SCROLLBAR_CONSTANTS, content: i32, page: i32, position: i32) {
    let info = SCROLLINFO {
        cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
        fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
        nMin: 0,
        nMax: (content - 1).max(0),
        nPage: page.max(0) as u32,
        nPos: position,
        nTrackPos: 0,
    };
    unsafe {
        SetScrollInfo(hwnd, bar, &info, true);
    }
}

fn tool_info(canvas: HWND, control: HWND, text: &mut [u16]) -> TTTOOLINFOW {
    TTTOOLINFOW {
        cbSize: std::mem::size_of::<TTTOOLINFOW>() as u32,
        uFlags: TTF_IDISHWND | TTF_SUBCLASS,
        hwnd: canvas,
        uId: control.0 as usize,
        lpszText: PWSTR(text.as_mut_ptr()),
        ..Default::default()
    }
}

fn add_tool(tooltip: HWND, canvas: HWND, control: HWND, text: &str) {
    if tooltip.is_invalid() {
        return;
    }
    let mut buffer: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let info = tool_info(canvas, control, &mut buffer);
    send(
        tooltip,
        TTM_ADDTOOLW,
        0,
        (&info as *const TTTOOLINFOW) as isize,
    );
}

fn update_tool(tooltip: HWND, canvas: HWND, control: HWND, text: &str) {
    if tooltip.is_invalid() {
        return;
    }
    let mut buffer: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let info = tool_info(canvas, control, &mut buffer);
    send(
        tooltip,
        TTM_UPDATETIPTEXTW,
        0,
        (&info as *const TTTOOLINFOW) as isize,
    );
}

struct CanvasState {
    layout: Layout,
    scale: f32,
    palette: Palette,
    dimension: usize,
    error: Option<(usize, usize)>,
    focused: Option<(usize, usize)>,
    empty: bool,
    values: usize,
}

fn draw_canvas(painter: &Painter, state: &CanvasState) {
    let layout = &state.layout;
    let scale = state.scale;
    let palette = &state.palette;
    painter.clear(palette.canvas);
    painter.fill_rect(layout.separator, palette.border);
    painter.icon_view(Icon::Matrix, layout.logo, palette.icon);
    painter.text(
        model::BRAND,
        Font::Medium,
        palette.ink,
        layout.brand,
        Align::Left,
        VAlign::Top,
    );
    painter.text(
        model::INTRO,
        Font::Large,
        palette.ink,
        layout.intro,
        Align::Left,
        VAlign::Top,
    );
    painter.text(
        model::SUBTITLE,
        Font::Body,
        palette.muted,
        layout.subtitle,
        Align::Left,
        VAlign::Top,
    );
    painter.card(
        layout.entry,
        palette.surface,
        palette.border,
        palette.card_highlight,
        scale,
    );
    painter.icon_view(Icon::Grid, layout.entry_icon, palette.icon);
    painter.text(
        model::ENTRY_TITLE,
        Font::Heading,
        palette.ink,
        layout.entry_title,
        Align::Left,
        VAlign::Top,
    );
    painter.text(
        model::ENTRY_DESCRIPTION,
        Font::Body,
        palette.muted,
        layout.entry_description,
        Align::Left,
        VAlign::Top,
    );
    painter.text(
        model::GRID_LABEL,
        Font::Small,
        palette.muted,
        layout.grid_label,
        Align::Left,
        VAlign::Top,
    );
    for row in 0..state.dimension.min(3) {
        painter.text(
            &model::row_number(row),
            Font::Small,
            palette.muted,
            layout.row_numbers[row],
            Align::Left,
            VAlign::Top,
        );
        for column in 0..4 {
            if !layout::field_visible(state.dimension, row, column) {
                continue;
            }
            let field = layout.fields[row][column];
            let stroke = if state.error == Some((row, column)) {
                palette.error
            } else if state.focused == Some((row, column)) {
                palette.accent
            } else {
                palette.border
            };
            let (x, y, w, h) = layout::field_box(field);
            painter.rounded_box(
                Frame::new(x, y, w, h, crate::theme::FIELD_RADIUS * scale),
                palette.field,
                Some(stroke),
            );
            if let Some(suffix) = layout::suffix(column) {
                painter.text(
                    suffix,
                    Font::Medium,
                    palette.muted,
                    layout::suffix_rect(field, scale),
                    Align::Center,
                    VAlign::Center,
                );
            }
        }
        for column in 0..state.dimension {
            painter.text(
                layout::operator_text(state.dimension, column),
                Font::Medium,
                palette.muted,
                layout.operators[row][column],
                Align::Left,
                VAlign::Top,
            );
        }
    }
    painter.card(
        layout.solution,
        palette.solution_fill,
        palette.solution_stroke,
        palette.card_highlight,
        scale,
    );
    painter.text(
        model::RESULT_LABEL,
        Font::Small,
        palette.solution_text,
        layout.solution_label,
        Align::Left,
        VAlign::Top,
    );
    if state.empty {
        painter.icon_view(Icon::Matrix, layout.empty_icon, palette.solution_text);
    }
    for index in 0..state.values.min(3) {
        painter.card(
            layout.tiles[index],
            palette.tile_fill,
            palette.tile_stroke,
            palette.card_highlight,
            scale,
        );
        painter.text(
            sistemas_lineares_core::VARIABLE_NAMES[index],
            Font::Heading,
            palette.solution_text,
            layout.tile_names[index],
            Align::Left,
            VAlign::Top,
        );
    }
    painter.text(
        model::FOOTER,
        Font::Small,
        palette.muted,
        layout.footer,
        Align::Left,
        VAlign::Top,
    );
    painter.text(
        model::SHORTCUTS,
        Font::Small,
        palette.muted,
        layout.shortcut,
        Align::Right,
        VAlign::Top,
    );
}

struct ButtonLook<'a> {
    button: Button,
    text: &'a str,
    width: i32,
    height: i32,
    scale: f32,
    palette: Palette,
    hover: bool,
    disabled: bool,
    focused: bool,
    selected: bool,
}

fn draw_button(painter: &Painter, look: &ButtonLook) {
    let button = look.button;
    let palette = &look.palette;
    let author = button == Button::Author;
    let parent = match button {
        Button::Author => palette.canvas,
        Button::Copy => palette.solution_fill,
        _ => palette.surface,
    };
    painter.clear(parent);
    let accent = button.primary() || look.selected;
    let face = if author {
        palette.link_face
    } else {
        palette.button_face
    };
    let fill = if look.disabled {
        palette.disabled_fill
    } else if accent {
        if look.hover {
            palette.accent_hover
        } else {
            palette.accent_fill
        }
    } else if look.hover {
        palette.hover_fill
    } else {
        face
    };
    let foreground = if look.disabled {
        palette.disabled_text
    } else if accent {
        palette.accent_text
    } else if look.hover {
        palette.hover_text
    } else if author {
        palette.link
    } else {
        palette.button_text
    };
    let secondary = if look.disabled || accent || (look.hover && palette.high_contrast) {
        foreground
    } else if author {
        palette.link_muted
    } else {
        foreground
    };
    let stroke = if look.disabled && palette.disabled_stroke.is_some() {
        palette.disabled_stroke
    } else if accent || author {
        None
    } else if button.quiet() {
        palette.quiet_stroke
    } else {
        Some(palette.button_stroke)
    };
    painter.rounded_box(
        Frame::new(
            1.0,
            1.0,
            (look.width - 3) as f32,
            (look.height - 3) as f32,
            crate::theme::BUTTON_RADIUS * look.scale,
        ),
        fill,
        stroke,
    );
    let icon = button.icon();
    let icon_size = if icon.is_some() { d(look.scale, 22) } else { 0 };
    let gap = if icon.is_some() { d(look.scale, 8) } else { 0 };
    let full = Rect::new(0, 0, look.width, look.height);
    if author {
        let run_gap = d(look.scale, 8);
        let (left_margin, right_margin) = painter.margins(Font::Medium);
        let runs = [
            Run {
                text: model::AUTHOR_NAME,
                color: foreground,
            },
            Run {
                text: model::AUTHOR_SEPARATOR,
                color: secondary,
            },
            Run {
                text: model::AUTHOR_RA,
                color: secondary,
            },
        ];
        let text_width: i32 = runs
            .iter()
            .map(|run| painter.measure(run.text, Font::Medium))
            .sum::<i32>()
            + run_gap * (runs.len() as i32 - 1)
            + left_margin
            + right_margin;
        let x = (look.width - (text_width + icon_size + gap)) / 2;
        painter.icon(
            Icon::External,
            x as f32,
            ((look.height - icon_size) / 2) as f32,
            icon_size as f32,
            icon_size as f32,
            foreground,
        );
        painter.runs(
            &runs,
            run_gap,
            Font::Medium,
            x + icon_size + gap + left_margin,
            full,
        );
    } else {
        let text_width = painter.measure_padded(look.text, Font::Medium);
        let x = (look.width - (text_width + icon_size + gap)) / 2;
        if let Some(icon) = icon {
            painter.icon(
                icon,
                x as f32,
                ((look.height - icon_size) / 2) as f32,
                icon_size as f32,
                icon_size as f32,
                foreground,
            );
        }
        painter.text(
            look.text,
            Font::Medium,
            foreground,
            Rect::new(x + icon_size + gap, 0, text_width, look.height),
            Align::Center,
            VAlign::Center,
        );
    }
    if look.focused {
        let inset = d(look.scale, 5);
        let ring = if button.primary() || (accent && palette.high_contrast) {
            palette.primary_focus
        } else if author {
            palette.link_focus
        } else {
            palette.focus
        };
        painter.rounded_outline(
            Frame::new(
                inset as f32,
                inset as f32,
                (look.width - 2 * inset - 1) as f32,
                (look.height - 2 * inset - 1) as f32,
                crate::theme::FOCUS_RADIUS * look.scale,
            ),
            ring,
            true,
        );
    }
}
