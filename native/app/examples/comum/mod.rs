#![allow(dead_code)]

use std::path::Path;

use sistemas_lineares::gfx::save_png;
use sistemas_lineares::{Button, Handles, pump};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, HGDIOBJ, RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE,
    RDW_UPDATENOW, RedrawWindow, SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::System::Threading::Sleep;
use windows::Win32::UI::Input::KeyboardAndMouse::GetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    BN_CLICKED, GetWindowRect, HideCaret, PW_RENDERFULLCONTENT, SendMessageW, ShowCaret,
    WM_COMMAND, WM_SETTEXT,
};
use windows::core::HSTRING;

pub fn settle(main: HWND) {
    for _ in 0..4 {
        pump(main);
        unsafe { Sleep(40) };
    }
    unsafe {
        let _ = RedrawWindow(
            Some(main),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_ERASE | RDW_FRAME,
        );
        let focus = GetFocus();
        if !focus.is_invalid() && HideCaret(Some(focus)).is_ok() {
            let _ = ShowCaret(Some(focus));
        }
    }
    pump(main);
}

pub fn click(handles: &Handles, button: Button) {
    let hwnd = handles.button(button);
    unsafe {
        SendMessageW(
            handles.canvas,
            WM_COMMAND,
            Some(WPARAM(
                usize::from(button.id()) | ((BN_CLICKED as usize) << 16),
            )),
            Some(LPARAM(hwnd.0 as isize)),
        );
    }
    pump(handles.main);
}

pub fn write(handles: &Handles, row: usize, column: usize, text: &str) {
    let wide = HSTRING::from(text);
    unsafe {
        SendMessageW(
            handles.inputs[row][column],
            WM_SETTEXT,
            Some(WPARAM(0)),
            Some(LPARAM(wide.as_ptr() as isize)),
        );
    }
    pump(handles.main);
}

pub fn capture(main: HWND) -> windows::core::Result<(u32, u32, Vec<u8>)> {
    let mut frame = RECT::default();
    unsafe { GetWindowRect(main, &mut frame)? };
    let width = frame.right - frame.left;
    let height = frame.bottom - frame.top;
    let mut bits = std::ptr::null_mut();
    let header = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };
    let info = BITMAPINFO {
        bmiHeader: header,
        ..Default::default()
    };
    unsafe {
        let dc = CreateCompatibleDC(None);
        let bitmap = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)?;
        let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
        let printed = PrintWindow(main, dc, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool();
        let length = (width * height * 4) as usize;
        let mut pixels = std::slice::from_raw_parts(bits.cast::<u8>(), length).to_vec();
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
        if !printed {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(-1),
                "PrintWindow falhou",
            ));
        }
        Ok((width as u32, height as u32, pixels))
    }
}

pub fn snap(main: HWND, directory: &Path, name: &str) -> windows::core::Result<()> {
    settle(main);
    let (width, height, pixels) = capture(main)?;
    save_png(
        &directory.join(format!("{name}.png")),
        width,
        height,
        &pixels,
    )?;
    println!("{name}  {width}x{height}");
    Ok(())
}
