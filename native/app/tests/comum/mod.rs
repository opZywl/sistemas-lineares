#![allow(dead_code)]

use std::mem::ManuallyDrop;

use sistemas_lineares::layout::Rect;
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
use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
use windows::Win32::UI::Accessibility::{AccessibleObjectFromWindow, IAccessible};
use windows::Win32::UI::Controls::{
    EM_GETSEL, TOOLTIP_FLAGS, TTF_IDISHWND, TTM_GETTEXTW, TTTOOLINFOW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetFocus, GetKeyboardState, IsWindowEnabled, SetKeyboardState, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    BN_CLICKED, DestroyWindow, GetClientRect, IsWindow, IsWindowVisible, MSG, PRF_CHILDREN,
    PRF_CLIENT, PRF_ERASEBKGND, PW_RENDERFULLCONTENT, SendMessageW, WM_CHAR, WM_COMMAND,
    WM_KEYDOWN, WM_KEYUP, WM_PRINT, WM_SETTEXT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, PRF_NONCLIENT};
use windows::Win32::UI::WindowsAndMessaging::{OBJID_CLIENT, WM_TIMER};
use windows::core::{HSTRING, Interface, PWSTR};

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

pub fn janela_animada(animar: bool) -> Handles {
    janela_com(Launch {
        animations: Some(animar),
        ..Launch::background(FORA_DA_TELA, CLIENTE)
    })
}

pub fn esperar_grafico(handles: &Handles) -> bool {
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(8);
    while std::time::Instant::now() < limite {
        pump(handles.main);
        let animando =
            sistemas_lineares::diagnostics(handles.main).is_some_and(|estado| estado.graph_timer);
        if !animando {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    false
}

pub fn posicao_no_canvas(handles: &Handles, hwnd: HWND) -> Rect {
    let mut canvas = RECT::default();
    let mut alvo = RECT::default();
    unsafe {
        let _ = GetWindowRect(handles.canvas, &mut canvas);
        let _ = GetWindowRect(hwnd, &mut alvo);
    }
    Rect::new(
        alvo.left - canvas.left,
        alvo.top - canvas.top,
        alvo.right - alvo.left,
        alvo.bottom - alvo.top,
    )
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

pub fn ocioso(handles: &Handles) {
    unsafe {
        SendMessageW(handles.main, WM_TIMER, Some(WPARAM(1)), Some(LPARAM(0)));
    }
    pump(handles.main);
}

pub fn com_teclas<T>(teclas: &[VIRTUAL_KEY], acao: impl FnOnce() -> T) -> T {
    let mut estado = [0u8; 256];
    unsafe {
        let _ = GetKeyboardState(&mut estado);
        let mut com = estado;
        for modificadora in [0x10usize, 0x11, 0x12, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5] {
            com[modificadora] = 0;
        }
        for tecla in teclas {
            com[usize::from(tecla.0)] = 0x80;
        }
        let _ = SetKeyboardState(&com);
    }
    let resultado = acao();
    unsafe {
        let _ = SetKeyboardState(&estado);
    }
    resultado
}

pub fn selecao(hwnd: HWND) -> (u32, u32) {
    let mut inicio = 0u32;
    let mut fim = 0u32;
    unsafe {
        SendMessageW(
            hwnd,
            EM_GETSEL,
            Some(WPARAM((&mut inicio as *mut u32) as usize)),
            Some(LPARAM((&mut fim as *mut u32) as isize)),
        );
    }
    (inicio, fim)
}

fn variante(valor: i32) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_I4,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { lVal: valor },
            }),
        },
    }
}

fn acessivel(hwnd: HWND) -> Option<IAccessible> {
    let mut objeto = std::ptr::null_mut();
    unsafe {
        AccessibleObjectFromWindow(hwnd, OBJID_CLIENT.0 as u32, &IAccessible::IID, &mut objeto)
    }
    .ok()?;
    (!objeto.is_null()).then(|| unsafe { IAccessible::from_raw(objeto) })
}

pub fn nome_acessivel(hwnd: HWND) -> String {
    acessivel(hwnd)
        .and_then(|objeto| unsafe { objeto.get_accName(&variante(0)) }.ok())
        .map(|nome| nome.to_string())
        .unwrap_or_default()
}

pub fn descricao_acessivel(hwnd: HWND) -> String {
    acessivel(hwnd)
        .and_then(|objeto| unsafe { objeto.get_accDescription(&variante(0)) }.ok())
        .map(|texto| texto.to_string())
        .unwrap_or_default()
}

pub fn papel_acessivel(hwnd: HWND) -> Option<i32> {
    let papel =
        acessivel(hwnd).and_then(|objeto| unsafe { objeto.get_accRole(&variante(0)) }.ok())?;
    unsafe {
        (papel.Anonymous.Anonymous.vt == VT_I4).then_some(papel.Anonymous.Anonymous.Anonymous.lVal)
    }
}

pub fn dica(handles: &Handles, ferramenta: usize, por_janela: bool) -> String {
    let mut buffer = [0u16; 256];
    let mut info = TTTOOLINFOW {
        cbSize: std::mem::size_of::<TTTOOLINFOW>() as u32,
        uFlags: if por_janela {
            TTF_IDISHWND
        } else {
            TOOLTIP_FLAGS(0)
        },
        hwnd: handles.canvas,
        uId: ferramenta,
        lpszText: PWSTR(buffer.as_mut_ptr()),
        ..Default::default()
    };
    unsafe {
        SendMessageW(
            handles.tooltip,
            TTM_GETTEXTW,
            Some(WPARAM(buffer.len())),
            Some(LPARAM((&mut info as *mut TTTOOLINFOW) as isize)),
        );
    }
    let fim = buffer.iter().position(|unit| *unit == 0).unwrap_or(0);
    String::from_utf16_lossy(&buffer[..fim])
}

pub fn area(rect: Rect) -> (i32, i32, i32, i32) {
    (rect.x, rect.y, rect.right(), rect.bottom())
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
