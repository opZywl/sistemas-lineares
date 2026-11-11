#![allow(dead_code)]

use sistemas_lineares::theme::Color;
use sistemas_lineares::{Button, Handles, Launch, Renderer, create, dispatch, pump, window_text};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, HGDIOBJ, RDW_ALLCHILDREN, RDW_ERASE, RDW_INVALIDATE, RDW_UPDATENOW,
    RedrawWindow, SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PW_CLIENTONLY, PrintWindow};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled, VIRTUAL_KEY};
use windows::Win32::UI::WindowsAndMessaging::{
    BN_CLICKED, DestroyWindow, GetClientRect, IsWindow, IsWindowVisible, MSG, PRF_CHILDREN,
    PRF_CLIENT, PRF_ERASEBKGND, PW_RENDERFULLCONTENT, SendMessageW, WM_CHAR, WM_COMMAND,
    WM_KEYDOWN, WM_KEYUP, WM_PRINT, WM_SETTEXT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, PRF_NONCLIENT};
use windows::core::HSTRING;

pub const FORA_DA_TELA: (i32, i32) = (-4000, 0);
pub const CLIENTE: (i32, i32) = (1160, 810);

pub fn janela() -> Handles {
    janela_com(Launch::background(FORA_DA_TELA, CLIENTE))
}

pub fn janela_gdi() -> Handles {
    janela_com(Launch {
        renderer: Renderer::Gdi,
        ..Launch::background(FORA_DA_TELA, CLIENTE)
    })
}

pub fn janela_com(launch: Launch) -> Handles {
    let handles = match create(&launch) {
        Ok(handles) => handles,
        Err(error) => panic!("não foi possível criar a janela: {error}"),
    };
    pump(handles.main);
    handles
}

pub fn fechar(handles: &Handles) {
    unsafe {
        if IsWindow(Some(handles.main)).as_bool() {
            let _ = DestroyWindow(handles.main);
        }
    }
    pump(handles.main);
}

pub fn clicar(handles: &Handles, button: Button) {
    unsafe {
        SendMessageW(
            handles.canvas,
            WM_COMMAND,
            Some(WPARAM(
                usize::from(button.id()) | ((BN_CLICKED as usize) << 16),
            )),
            Some(LPARAM(handles.button(button).0 as isize)),
        );
    }
    pump(handles.main);
}

pub fn escrever(handles: &Handles, row: usize, column: usize, text: &str) {
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

pub fn tecla(handles: &Handles, target: HWND, key: VIRTUAL_KEY) {
    let down = MSG {
        hwnd: target,
        message: WM_KEYDOWN,
        wParam: WPARAM(usize::from(key.0)),
        lParam: LPARAM(1),
        ..Default::default()
    };
    dispatch(handles.main, &down);
    pump(handles.main);
    let up = MSG {
        hwnd: target,
        message: WM_KEYUP,
        wParam: WPARAM(usize::from(key.0)),
        lParam: LPARAM(0xC000_0001u32 as i32 as isize),
        ..Default::default()
    };
    dispatch(handles.main, &up);
    pump(handles.main);
}

pub fn digitar(handles: &Handles, target: HWND, text: &str) {
    for unit in text.encode_utf16() {
        let message = MSG {
            hwnd: target,
            message: WM_CHAR,
            wParam: WPARAM(usize::from(unit)),
            lParam: LPARAM(1),
            ..Default::default()
        };
        dispatch(handles.main, &message);
    }
    pump(handles.main);
}

pub fn texto(hwnd: HWND) -> String {
    window_text(hwnd)
}

pub fn visivel(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

pub fn habilitado(hwnd: HWND) -> bool {
    unsafe { IsWindowEnabled(hwnd).as_bool() }
}

pub fn foco() -> HWND {
    unsafe { GetFocus() }
}

pub struct Imagem {
    pub largura: i32,
    pub altura: i32,
    pub bgra: Vec<u8>,
    pub metodo: &'static str,
}

impl Imagem {
    pub fn cor(&self, x: i32, y: i32) -> Color {
        let index = ((y * self.largura + x) * 4) as usize;
        Color::rgb(self.bgra[index + 2], self.bgra[index + 1], self.bgra[index])
    }

    pub fn conta(&self, area: (i32, i32, i32, i32), condicao: impl Fn(Color) -> bool) -> usize {
        let mut total = 0;
        for y in area.1.max(0)..area.3.min(self.altura) {
            for x in area.0.max(0)..area.2.min(self.largura) {
                if condicao(self.cor(x, y)) {
                    total += 1;
                }
            }
        }
        total
    }
}

pub fn distancia(a: Color, b: Color) -> i32 {
    (i32::from(a.r) - i32::from(b.r)).abs()
        + (i32::from(a.g) - i32::from(b.g)).abs()
        + (i32::from(a.b) - i32::from(b.b)).abs()
}

fn render(
    hwnd: HWND,
    largura: i32,
    altura: i32,
    desenhar: impl Fn(windows::Win32::Graphics::Gdi::HDC) -> bool,
) -> Option<Vec<u8>> {
    let mut bits = std::ptr::null_mut();
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: largura,
            biHeight: -altura,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    unsafe {
        let dc = CreateCompatibleDC(None);
        let bitmap = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;
        let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
        let feito = desenhar(dc);
        let pixels =
            std::slice::from_raw_parts(bits.cast::<u8>(), (largura * altura * 4) as usize).to_vec();
        SelectObject(dc, previous);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
        let _ = hwnd;
        feito.then_some(pixels)
    }
}

pub fn capturar(hwnd: HWND) -> Imagem {
    let mut client = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut client);
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_ERASE,
        );
    }
    let (largura, altura) = (client.right, client.bottom);
    let cheia = render(hwnd, largura, altura, |dc| unsafe {
        PrintWindow(
            hwnd,
            dc,
            PRINT_WINDOW_FLAGS(PW_CLIENTONLY.0 | PW_RENDERFULLCONTENT),
        )
        .as_bool()
    });
    let imagem = cheia
        .filter(|pixels| {
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0)
        })
        .map(|bgra| Imagem {
            largura,
            altura,
            bgra,
            metodo: "PrintWindow",
        });
    imagem.unwrap_or_else(|| imprimir(hwnd))
}

pub fn imprimir(hwnd: HWND) -> Imagem {
    let mut janela = RECT::default();
    let mut cliente = RECT::default();
    let mut origem = POINT::default();
    unsafe {
        let _ = GetWindowRect(hwnd, &mut janela);
        let _ = GetClientRect(hwnd, &mut cliente);
        let _ = ClientToScreen(hwnd, &mut origem);
    }
    let (largura_total, altura_total) = (janela.right - janela.left, janela.bottom - janela.top);
    let (dx, dy) = (origem.x - janela.left, origem.y - janela.top);
    let total = render(hwnd, largura_total, altura_total, |dc| unsafe {
        SendMessageW(
            hwnd,
            WM_PRINT,
            Some(WPARAM(dc.0 as usize)),
            Some(LPARAM(
                (PRF_NONCLIENT | PRF_CLIENT | PRF_CHILDREN | PRF_ERASEBKGND) as isize,
            )),
        );
        true
    })
    .unwrap_or_default();
    let (largura, altura) = (cliente.right, cliente.bottom);
    let mut bgra = vec![0u8; (largura.max(0) * altura.max(0) * 4) as usize];
    for y in 0..altura {
        let origem_linha = (((y + dy) * largura_total + dx) * 4) as usize;
        let destino = (y * largura * 4) as usize;
        let tamanho = (largura * 4) as usize;
        if let (Some(fonte), Some(alvo)) = (
            total.get(origem_linha..origem_linha + tamanho),
            bgra.get_mut(destino..destino + tamanho),
        ) {
            alvo.copy_from_slice(fonte);
        }
    }
    Imagem {
        largura,
        altura,
        bgra,
        metodo: "WM_PRINT",
    }
}
