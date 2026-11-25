#[path = "comum/mod.rs"]
mod comum;

use std::path::{Path, PathBuf};

use comum::{click, snap, write};
use sistemas_lineares::{Button, Handles, Launch, Renderer, create, diagnostics, pump};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, GetClientRect, GetWindowRect, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
    SendMessageW, SetWindowPos, UIS_CLEAR, UISF_HIDEFOCUS, WM_CHANGEUISTATE, WM_TIMER,
};

fn idle(main: HWND) {
    unsafe {
        SendMessageW(main, WM_TIMER, Some(WPARAM(1)), Some(LPARAM(0)));
    }
    pump(main);
}

fn focus(main: HWND, target: HWND) {
    unsafe {
        let _ = SetFocus(Some(target));
    }
    pump(main);
}

fn keyboard_cues(main: HWND, visible: bool) {
    let action = if visible {
        UIS_CLEAR
    } else {
        windows::Win32::UI::WindowsAndMessaging::UIS_SET
    };
    unsafe {
        SendMessageW(
            main,
            WM_CHANGEUISTATE,
            Some(WPARAM(
                (action as usize) | ((UISF_HIDEFOCUS as usize) << 16),
            )),
            None,
        );
    }
    pump(main);
}

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
        idle_delay_ms: 60_000,
        ..Launch::background((-4000, 0), (1160, 810))
    })?;
    let dpi = unsafe { GetDpiForWindow(handles.main) };
    if dpi != 96 {
        eprintln!("aviso: DPI da janela de captura = {dpi} (a referência usa 96)");
    }
    let main = handles.main;
    snap(main, &directory, "01-inicial-2x2")?;
    write(&handles, 0, 0, "2");
    write(&handles, 0, 1, "1");
    write(&handles, 0, 3, "5");
    snap(main, &directory, "01a-digitando-faltam-3")?;
    write(&handles, 1, 0, "1");
    write(&handles, 1, 1, "-1");
    write(&handles, 1, 3, "1");
    snap(main, &directory, "01b-resposta-ao-vivo")?;
    write(&handles, 1, 3, "");
    snap(main, &directory, "01c-atualizando-esmaecido")?;
    idle(main);
    snap(main, &directory, "01d-assentado-falta-1")?;
    write(&handles, 1, 3, "1");
    focus(main, handles.inputs[0][0]);
    write(&handles, 0, 0, "2^1");
    snap(main, &directory, "01e-valor-lido-da-potencia")?;
    focus(main, handles.inputs[1][1]);
    write(&handles, 0, 1, "1/0");
    idle(main);
    snap(main, &directory, "01f-erro-revelado")?;
    write(&handles, 0, 1, "1e999");
    write(&handles, 0, 3, "1e998");
    write(&handles, 1, 3, "1e997");
    idle(main);
    snap(main, &directory, "01g-numeros-grandes")?;
    click(&handles, Button::Clear);
    keyboard_cues(main, true);
    focus(main, handles.links[1]);
    snap(main, &directory, "01h-foco-no-link-com-dica")?;
    focus(main, handles.two);
    snap(main, &directory, "01i-foco-no-segmento")?;
    focus(main, handles.example);
    snap(main, &directory, "01j-foco-em-usar-exemplo")?;
    focus(main, handles.inputs[0][0]);
    keyboard_cues(main, false);
    click(&handles, Button::Example);
    snap(main, &directory, "02-exemplo-2x2-resolvido")?;
    keyboard_cues(main, true);
    focus(main, handles.think);
    snap(main, &directory, "03-foco-no-modo-pensar")?;
    focus(main, handles.inputs[0][0]);
    keyboard_cues(main, false);
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
    snap(main, &directory, "12-infinitas-3x3")?;
    click(&handles, Button::Think);
    click(&handles, Button::MethodCramer);
    full_page(&handles, &directory, "13-modo-pensar-cramer-singular")?;
    click(&handles, Button::Back);
    write(&handles, 2, 3, "13");
    snap(main, &directory, "14-sem-solucao-3x3")?;
    click(&handles, Button::Think);
    click(&handles, Button::MethodGauss);
    full_page(&handles, &directory, "15-modo-pensar-sem-solucao")?;
    click(&handles, Button::Back);
    fill(
        &handles,
        &[
            &["1234567", "2", "3", "7654321"],
            &["4", "2345678", "6", "8765432"],
            &["7", "8", "3456789", "9876543"],
        ],
    );
    snap(main, &directory, "15a-legenda-longa-3x3")?;
    resize_client(main, 1057, 800)?;
    snap(main, &directory, "15b-legenda-longa-com-reticencias")?;
    resize_client(main, 1160, 810)?;
    click(&handles, Button::Clear);
    click(&handles, Button::Two);
    snap(main, &directory, "16-aguardando-2x2")?;
    fill(&handles, &[&["3", "0", "1"], &["0", "7", "1e-30"]]);
    snap(main, &directory, "17-valores-longos-2x2")?;
    click(&handles, Button::Think);
    full_page(&handles, &directory, "18-modo-pensar-valores-longos")?;
    click(&handles, Button::Back);
    write(&handles, 0, 0, "1/0");
    idle(main);
    snap(main, &directory, "19-erro-denominador-zero")?;
    fill(&handles, &[&["1", "1", "2"], &["2", "2", "4"]]);
    snap(main, &directory, "20-infinitas-2x2")?;
    write(&handles, 1, 3, "5");
    snap(main, &directory, "20a-sem-solucao-2x2")?;
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
    focus(main, handles.example);
    snap(main, &directory, "21a-janela-minima-legenda-sem-painel")?;
    click(&handles, Button::Two);
    click(&handles, Button::Example);
    focus(main, handles.example);
    snap(main, &directory, "21b-janela-minima-2x2-com-painel")?;
    click(&handles, Button::Three);
    focus(main, handles.inputs[0][0]);
    click(&handles, Button::Example);
    click(&handles, Button::Think);
    snap(main, &directory, "22-janela-minima-modo-pensar")?;
    unsafe { DestroyWindow(main)? };
    pump(main);
    Ok(())
}
