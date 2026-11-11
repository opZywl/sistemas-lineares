#[path = "comum/mod.rs"]
mod comum;

use std::path::PathBuf;

use comum::{click, snap, write};
use sistemas_lineares::{Button, Launch, Renderer, create, pump};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos,
};

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
    click(&handles, Button::Three);
    click(&handles, Button::Example);
    click(&handles, Button::Solve);
    snap(main, &directory, "04-exemplo-3x3-resolvido")?;
    for row in 0..3 {
        for column in 0..4 {
            write(&handles, row, column, "0");
        }
    }
    click(&handles, Button::Solve);
    snap(main, &directory, "05-infinitas-3x3")?;
    write(&handles, 0, 3, "1");
    click(&handles, Button::Solve);
    snap(main, &directory, "06-sem-solucao-3x3")?;
    click(&handles, Button::Clear);
    click(&handles, Button::Two);
    click(&handles, Button::Solve);
    snap(main, &directory, "07-erro-campo-vazio-2x2")?;
    for (row, values) in [["3", "0", "1"], ["0", "7", "1e-30"]].iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            write(&handles, row, if column == 2 { 3 } else { column }, value);
        }
    }
    click(&handles, Button::Solve);
    snap(main, &directory, "08-valores-longos-2x2")?;
    write(&handles, 0, 0, "1/0");
    click(&handles, Button::Solve);
    snap(main, &directory, "09-erro-denominador-zero")?;
    for (row, values) in [["1", "1", "2"], ["2", "2", "4"]].iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            write(&handles, row, if column == 2 { 3 } else { column }, value);
        }
    }
    click(&handles, Button::Solve);
    snap(main, &directory, "10-infinitas-2x2")?;
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
    snap(main, &directory, "11-janela-minima-3x3")?;
    unsafe { DestroyWindow(main)? };
    pump(main);
    Ok(())
}
