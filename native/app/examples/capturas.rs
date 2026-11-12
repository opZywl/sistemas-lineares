#[path = "comum/mod.rs"]
mod comum;

use std::path::{Path, PathBuf};

use comum::{click, snap, write};
use sistemas_lineares::{Button, Handles, Launch, Renderer, create, diagnostics, pump};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, GetClientRect, GetWindowRect, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
    SetWindowPos,
};

fn resize_client(main: HWND, width: i32, height: i32) -> windows::core::Result<()> {
    let mut window = RECT::default();
    let mut client = RECT::default();
    unsafe {
        GetWindowRect(main, &mut window)?;
        GetClientRect(main, &mut client)?;
        let extra_w = (window.right - window.left) - client.right;
        let extra_h = (window.bottom - window.top) - client.bottom;
        SetWindowPos(
            main,
            None,
            0,
            0,
            width + extra_w,
            height + extra_h,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
    }
    pump(main);
    Ok(())
}

fn full_page(handles: &Handles, directory: &Path, name: &str) -> windows::core::Result<()> {
    let main = handles.main;
    for _ in 0..3 {
        let height = diagnostics(main).map_or(810, |state| state.layout.content_h);
        resize_client(main, 1160, height)?;
    }
    snap(main, directory, name)?;
    resize_client(main, 1160, 810)
}

fn fill(handles: &Handles, rows: &[&[&str]]) {
    let dimension = rows.len();
    for (row, values) in rows.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            write(
                handles,
                row,
                if column == dimension { 3 } else { column },
                value,
            );
        }
    }
}

fn main() -> windows::core::Result<()> {
    let directory = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("capturas"));
    std::fs::create_dir_all(&directory).map_err(|error| {
        windows::core::Error::new(windows::core::HRESULT(-1), error.to_string())
    })?;
    let contraste = std::env::args().any(|argumento| argumento == "--alto-contraste");
    let renderer = if std::env::args().any(|argumento| argumento == "--gdi") {
        Renderer::Gdi
    } else {
        Renderer::Direct2D
    };
    let handles = create(&Launch {
        high_contrast: contraste.then_some(true),
        renderer,
        ..Launch::background((-4000, 0), (1160, 810))
    })?;
    let dpi = unsafe { GetDpiForWindow(handles.main) };
    if dpi != 96 {
        eprintln!("aviso: DPI da janela de captura = {dpi} (a referência usa 96)");
    }
    let main = handles.main;
    snap(main, &directory, "01-inicial-2x2")?;
    click(&handles, Button::Example);
    snap(main, &directory, "02-exemplo-2x2-pronto")?;
    click(&handles, Button::Solve);
    snap(main, &directory, "03-exemplo-2x2-resolvido")?;
    click(&handles, Button::Think);
    snap(main, &directory, "04-modo-pensar-2x2")?;
    full_page(&handles, &directory, "05-modo-pensar-gauss-jordan-2x2")?;
    click(&handles, Button::MethodCramer);
    full_page(&handles, &directory, "06-modo-pensar-cramer-2x2")?;
    click(&handles, Button::MethodGauss);
    full_page(&handles, &directory, "07-modo-pensar-escalonamento-2x2")?;
    click(&handles, Button::Back);
    click(&handles, Button::Three);
    click(&handles, Button::Example);
    click(&handles, Button::Solve);
    snap(main, &directory, "08-exemplo-3x3-resolvido")?;
    click(&handles, Button::Think);
    click(&handles, Button::MethodCramer);
    full_page(&handles, &directory, "09-modo-pensar-cramer-3x3")?;
    click(&handles, Button::MethodGaussJordan);
    full_page(&handles, &directory, "10-modo-pensar-gauss-jordan-3x3")?;
    click(&handles, Button::MethodGauss);
    full_page(&handles, &directory, "11-modo-pensar-escalonamento-3x3")?;
    click(&handles, Button::Back);
    fill(
        &handles,
        &[
            &["1", "2", "3", "4"],
            &["2", "4", "6", "8"],
            &["3", "6", "9", "12"],
        ],
    );
    click(&handles, Button::Solve);
    snap(main, &directory, "12-infinitas-3x3")?;
    click(&handles, Button::Think);
    click(&handles, Button::MethodCramer);
    full_page(&handles, &directory, "13-modo-pensar-cramer-singular")?;
    click(&handles, Button::Back);
    write(&handles, 2, 3, "13");
    click(&handles, Button::Solve);
    snap(main, &directory, "14-sem-solucao-3x3")?;
    click(&handles, Button::Think);
    click(&handles, Button::MethodGauss);
    full_page(&handles, &directory, "15-modo-pensar-sem-solucao")?;
    click(&handles, Button::Back);
    click(&handles, Button::Clear);
    click(&handles, Button::Two);
    click(&handles, Button::Solve);
    snap(main, &directory, "16-erro-campo-vazio-2x2")?;
    fill(&handles, &[&["3", "0", "1"], &["0", "7", "1e-30"]]);
    click(&handles, Button::Solve);
    snap(main, &directory, "17-valores-longos-2x2")?;
    click(&handles, Button::Think);
    full_page(&handles, &directory, "18-modo-pensar-valores-longos")?;
    click(&handles, Button::Back);
    write(&handles, 0, 0, "1/0");
    click(&handles, Button::Solve);
    snap(main, &directory, "19-erro-denominador-zero")?;
    fill(&handles, &[&["1", "1", "2"], &["2", "2", "4"]]);
    click(&handles, Button::Solve);
    snap(main, &directory, "20-infinitas-2x2")?;
    unsafe {
        SetWindowPos(
            main,
            None,
            0,
            0,
            640,
            480,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )?
    };
    pump(main);
    click(&handles, Button::Three);
    snap(main, &directory, "21-janela-minima-3x3")?;
    click(&handles, Button::Example);
    click(&handles, Button::Solve);
    click(&handles, Button::Think);
    snap(main, &directory, "22-janela-minima-modo-pensar")?;
    unsafe { DestroyWindow(main)? };
    pump(main);
    Ok(())
}
