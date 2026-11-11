#[path = "comum/mod.rs"]
mod comum;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use comum::{capture, click, settle};
use sistemas_lineares::gfx::save_png;
use sistemas_lineares::{Button, Handles, Launch, create, dispatch, pump};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, MSG, SendMessageW, WM_CHAR, WM_MOUSEMOVE,
};

struct Filme {
    pasta: PathBuf,
    lista: String,
    quadros: usize,
}

impl Filme {
    fn quadro(&mut self, main: HWND, segundos: f32, pintar: bool) -> windows::core::Result<()> {
        if pintar {
            settle(main);
        } else {
            unsafe {
                let _ = RedrawWindow(
                    Some(main),
                    None,
                    None,
                    RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
                );
            }
        }
        let (width, height, pixels) = capture(main)?;
        let nome = format!("{:03}.png", self.quadros);
        save_png(&self.pasta.join(&nome), width, height, &pixels)?;
        let _ = writeln!(self.lista, "file '{nome}'\nduration {segundos:.2}");
        self.quadros += 1;
        Ok(())
    }

    fn fechar(&mut self) -> std::io::Result<()> {
        if let Some(ultimo) = self.quadros.checked_sub(1) {
            let _ = writeln!(self.lista, "file '{ultimo:03}.png'");
        }
        std::fs::write(
            self.pasta.join("quadros.txt"),
            format!("ffconcat version 1.0\n{}", self.lista),
        )
    }
}

fn digitar(handles: &Handles, row: usize, column: usize, texto: &str) {
    let alvo = handles.inputs[row][column];
    unsafe {
        let _ = SetFocus(Some(alvo));
    }
    for unidade in texto.encode_utf16() {
        let mensagem = MSG {
            hwnd: alvo,
            message: WM_CHAR,
            wParam: WPARAM(usize::from(unidade)),
            lParam: LPARAM(1),
            ..Default::default()
        };
        dispatch(handles.main, &mensagem);
    }
    pump(handles.main);
}

fn preencher(
    filme: &mut Filme,
    handles: &Handles,
    linhas: &[&[&str]],
) -> windows::core::Result<()> {
    let dimensao = linhas.len();
    for (row, valores) in linhas.iter().enumerate() {
        for (coluna, valor) in valores.iter().enumerate() {
            let column = if coluna == dimensao { 3 } else { coluna };
            digitar(handles, row, column, valor);
            filme.quadro(handles.main, 0.32, true)?;
        }
    }
    Ok(())
}

fn passar_o_mouse(handles: &Handles, button: Button) {
    unsafe {
        SendMessageW(
            handles.button(button),
            WM_MOUSEMOVE,
            Some(WPARAM(0)),
            Some(LPARAM(8 | (8 << 16))),
        );
    }
}

fn main() -> windows::core::Result<()> {
    let pasta = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("demo-frames"));
    std::fs::create_dir_all(&pasta).map_err(|error| {
        windows::core::Error::new(windows::core::HRESULT(-1), error.to_string())
    })?;
    for entrada in std::fs::read_dir(&pasta)
        .map_err(|error| windows::core::Error::new(windows::core::HRESULT(-1), error.to_string()))?
        .flatten()
    {
        let caminho = entrada.path();
        if caminho
            .extension()
            .is_some_and(|extensao| extensao == "png" || extensao == "txt")
        {
            let _ = std::fs::remove_file(&caminho);
        }
    }
    let handles = create(&Launch::background((-4000, 0), (1160, 810)))?;
    let main = handles.main;
    let mut filme = Filme {
        pasta: pasta.clone(),
        lista: String::new(),
        quadros: 0,
    };
    filme.quadro(main, 1.6, true)?;
    passar_o_mouse(&handles, Button::Three);
    filme.quadro(main, 0.45, false)?;
    pump(main);
    click(&handles, Button::Three);
    filme.quadro(main, 0.9, true)?;
    preencher(
        &mut filme,
        &handles,
        &[
            &["1", "1", "1", "6"],
            &["2", "-1", "1", "3"],
            &["1", "2", "-1", "2"],
        ],
    )?;
    passar_o_mouse(&handles, Button::Solve);
    filme.quadro(main, 0.6, false)?;
    pump(main);
    click(&handles, Button::Solve);
    filme.quadro(main, 3.2, true)?;
    passar_o_mouse(&handles, Button::Clear);
    filme.quadro(main, 0.45, false)?;
    pump(main);
    click(&handles, Button::Clear);
    filme.quadro(main, 0.6, true)?;
    click(&handles, Button::Two);
    filme.quadro(main, 0.7, true)?;
    preencher(&mut filme, &handles, &[&["1", "1", "2"], &["2", "2", "4"]])?;
    passar_o_mouse(&handles, Button::Solve);
    filme.quadro(main, 0.6, false)?;
    pump(main);
    click(&handles, Button::Solve);
    filme.quadro(main, 3.6, true)?;
    filme.fechar().map_err(|error| {
        windows::core::Error::new(windows::core::HRESULT(-1), error.to_string())
    })?;
    unsafe { DestroyWindow(main)? };
    pump(main);
    println!(
        "{} quadros em {}",
        filme.quadros,
        Path::new(&pasta).display()
    );
    Ok(())
}
