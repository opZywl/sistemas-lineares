mod comum;

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use comum::*;
use sistemas_lineares::gfx::{Engine, GdiResources};
use sistemas_lineares::icons::Icon;
use sistemas_lineares::layout::d;
use sistemas_lineares::model;
use sistemas_lineares::theme::{self, Color};
use sistemas_lineares::window::{system_colors, system_high_contrast};
use sistemas_lineares::{Button, Launch, diagnostics, force_high_contrast, pump};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, GetObjectW, GetTextFaceW, GetTextMetricsW, HBRUSH, HGDIOBJ,
    LOGBRUSH, ReleaseDC, SelectObject, TEXTMETRICW,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::System::Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4};
use windows::Win32::UI::Accessibility::{
    AccessibleObjectFromWindow, CUIAutomation, HWINEVENTHOOK, IAccessible, IUIAutomation,
    SetWinEventHook, UIA_LiveSettingPropertyId, UnhookWinEvent,
};
use windows::Win32::UI::Controls::{
    EM_GETLIMITTEXT, EM_GETMARGINS, EM_GETSEL, EM_SETSEL, TTF_IDISHWND, TTM_GETTEXTW, TTTOOLINFOW,
    WM_MOUSELEAVE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyboardState, SetFocus, SetKeyboardState, VK_BACK, VK_CONTROL, VK_ESCAPE, VK_RETURN,
    VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{Interface, PWSTR};

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

fn nome_acessivel(hwnd: HWND) -> String {
    acessivel(hwnd)
        .and_then(|objeto| unsafe { objeto.get_accName(&variante(0)) }.ok())
        .map(|nome| nome.to_string())
        .unwrap_or_default()
}

fn descricao_acessivel(hwnd: HWND) -> String {
    acessivel(hwnd)
        .and_then(|objeto| unsafe { objeto.get_accDescription(&variante(0)) }.ok())
        .map(|texto| texto.to_string())
        .unwrap_or_default()
}

fn face_e_altura(hwnd: HWND) -> (String, i32) {
    unsafe {
        let fonte = HGDIOBJ(SendMessageW(hwnd, WM_GETFONT, None, None).0 as *mut _);
        let dc = GetDC(Some(hwnd));
        let anterior = SelectObject(dc, fonte);
        let mut buffer = [0u16; 64];
        let tamanho = GetTextFaceW(dc, Some(&mut buffer));
        let mut metricas = TEXTMETRICW::default();
        let _ = GetTextMetricsW(dc, &mut metricas);
        SelectObject(dc, anterior);
        ReleaseDC(Some(hwnd), dc);
        (
            String::from_utf16_lossy(&buffer[..(tamanho.max(1) as usize - 1)]),
            metricas.tmHeight,
        )
    }
}

fn selecao(hwnd: HWND) -> (u32, u32) {
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

fn com_ctrl<T>(acao: impl FnOnce() -> T) -> T {
    let mut estado = [0u8; 256];
    unsafe {
        let _ = GetKeyboardState(&mut estado);
        let mut com = estado;
        com[usize::from(VK_CONTROL.0)] = 0x80;
        let _ = SetKeyboardState(&com);
    }
    let resultado = acao();
    unsafe {
        let _ = SetKeyboardState(&estado);
    }
    resultado
}

fn nao_e_primeiro_plano(handles: &sistemas_lineares::Handles) {
    let primeiro = unsafe { GetForegroundWindow() };
    assert_ne!(
        primeiro, handles.main,
        "a janela de teste roubou o foco do usuário"
    );
    let estilo = unsafe { GetWindowLongW(handles.main, GWL_EXSTYLE) } as u32;
    assert!(
        estilo & WS_EX_NOACTIVATE.0 != 0,
        "janela de teste sem WS_EX_NOACTIVATE"
    );
}

#[test]
fn fluxo_completo_por_mensagens() {
    let h = janela();
    nao_e_primeiro_plano(&h);
    assert_eq!(texto(h.main), "Sistemas Lineares · Lucas Lima");
    assert_eq!(texto(h.author), "Feito por Lucas Lima · RA 185964");
    assert_eq!(foco(), h.inputs[0][0], "foco inicial no campo (0,0)");
    assert!(!habilitado(h.copy), "copiar desabilitado no início");
    assert_eq!(texto(h.copy), "Copiar resultado");
    assert_eq!(texto(h.helper), model::DEFAULT_HELPER);
    assert_eq!(texto(h.dimension), "2 equações · 2 incógnitas");
    assert!(
        visivel(h.empty_title)
            && visivel(h.empty_description)
            && !visivel(h.title)
            && !visivel(h.general)
    );
    assert!(!visivel(h.inputs[0][2]) && !visivel(h.inputs[2][0]) && visivel(h.inputs[1][3]));
    assert_eq!(
        unsafe { SendMessageW(h.inputs[0][0], EM_GETLIMITTEXT, None, None) }.0,
        32767
    );

    clicar(&h, Button::Solve);
    assert!(
        texto(h.helper).contains("Equação 1"),
        "validação inline: {}",
        texto(h.helper)
    );
    assert_eq!(
        texto(h.helper),
        "Equação 1, coeficiente de x: Preencha todos os campos."
    );
    assert_eq!(foco(), h.inputs[0][0]);

    clicar(&h, Button::Example);
    let esperado = [
        (0, 0, "2"),
        (0, 1, "1"),
        (0, 3, "5"),
        (1, 0, "1"),
        (1, 1, "-1"),
        (1, 3, "1"),
    ];
    for (row, column, valor) in esperado {
        assert_eq!(
            texto(h.inputs[row][column]),
            valor,
            "exemplo ({row},{column})"
        );
    }
    assert_eq!(foco(), h.solve, "exemplo foca Resolver");
    assert_eq!(texto(h.helper), model::EXAMPLE_HELPER);

    clicar(&h, Button::Solve);
    assert_eq!(texto(h.values[0]), "2");
    assert_eq!(texto(h.values[1]), "1");
    assert!(visivel(h.values[0]) && visivel(h.values[1]) && !visivel(h.values[2]));
    assert!(visivel(h.title) && !visivel(h.empty_title) && !visivel(h.general));
    assert_eq!(texto(h.title), "Solução encontrada");
    assert_eq!(texto(h.note), "Solução única · valores exatos");
    assert!(habilitado(h.copy), "copiar habilitado após resolver");
    assert_eq!(selecao(h.values[0]), (0, 0));

    escrever(&h, 0, 0, "4");
    assert!(!habilitado(h.copy), "editar invalida o resultado");
    assert!(!visivel(h.title) && visivel(h.empty_title));
    assert_eq!(texto(h.note), model::NOTE_DEFAULT);

    clicar(&h, Button::Three);
    assert_eq!(texto(h.dimension), "3 equações · 3 incógnitas");
    assert!(visivel(h.inputs[2][2]) && visivel(h.inputs[0][2]));
    clicar(&h, Button::Example);
    clicar(&h, Button::Solve);
    assert_eq!(
        [texto(h.values[0]), texto(h.values[1]), texto(h.values[2])],
        ["1", "2", "3"]
    );

    for row in 0..3 {
        for column in 0..4 {
            escrever(&h, row, column, "0");
        }
    }
    clicar(&h, Button::Solve);
    assert_eq!(texto(h.title), "Infinitas soluções");
    assert!(visivel(h.general) && !visivel(h.values[0]));
    assert_eq!(
        texto(h.general),
        "x = t1\r\ny = t2\r\nz = t3\r\n\r\nt1, t2, ... podem ser quaisquer números reais."
    );

    escrever(&h, 0, 3, "1");
    clicar(&h, Button::Solve);
    assert_eq!(texto(h.title), "Sem solução");
    assert_eq!(texto(h.general), model::GENERAL_NONE);
    assert_eq!(texto(h.note), model::NOTE_NONE);

    clicar(&h, Button::Clear);
    for row in 0..3 {
        for column in 0..4 {
            assert_eq!(texto(h.inputs[row][column]), "", "limpar ({row},{column})");
        }
    }
    assert!(!habilitado(h.copy));
    assert_eq!(foco(), h.inputs[0][0]);

    escrever(&h, 0, 0, "1/0");
    clicar(&h, Button::Solve);
    assert_eq!(foco(), h.inputs[0][0]);
    assert_eq!(
        selecao(h.inputs[0][0]),
        (0, 3),
        "seleciona tudo no campo com erro"
    );
    nao_e_primeiro_plano(&h);
    fechar(&h);
}

#[test]
fn ordem_do_tab_e_circular() {
    let h = janela();
    let sequencia = |h: &sistemas_lineares::Handles, voltas: usize, anterior: bool| {
        let mut atual = h.two;
        let mut vistos = vec![atual];
        for _ in 0..voltas {
            atual = unsafe { GetNextDlgTabItem(h.main, Some(atual), anterior) }.unwrap_or_default();
            vistos.push(atual);
        }
        vistos
    };
    let esperado = vec![
        h.two,
        h.three,
        h.inputs[0][0],
        h.inputs[0][1],
        h.inputs[0][3],
        h.inputs[1][0],
        h.inputs[1][1],
        h.inputs[1][3],
        h.solve,
        h.example,
        h.clear,
        h.author,
        h.two,
    ];
    assert_eq!(
        sequencia(&h, 12, false),
        esperado,
        "Tab no 2x2 com Copiar desabilitado"
    );
    let mut reverso = esperado.clone();
    reverso.reverse();
    assert_eq!(
        sequencia(&h, 12, true),
        reverso,
        "Shift+Tab inverte a ordem"
    );
    clicar(&h, Button::Three);
    clicar(&h, Button::Example);
    clicar(&h, Button::Solve);
    let tres = sequencia(&h, 20, false);
    let esperado3 = vec![
        h.two,
        h.three,
        h.inputs[0][0],
        h.inputs[0][1],
        h.inputs[0][2],
        h.inputs[0][3],
        h.inputs[1][0],
        h.inputs[1][1],
        h.inputs[1][2],
        h.inputs[1][3],
        h.inputs[2][0],
        h.inputs[2][1],
        h.inputs[2][2],
        h.inputs[2][3],
        h.solve,
        h.example,
        h.clear,
        h.copy,
        h.author,
        h.two,
        h.three,
    ];
    assert_eq!(tres, esperado3, "Tab no 3x3 com Copiar habilitado");
    fechar(&h);
}

#[test]
fn teclado_enter_espaco_esc_e_atalhos() {
    let h = janela();
    tecla(&h, h.inputs[0][0], VK_RETURN);
    assert!(
        texto(h.helper).starts_with("Equação 1"),
        "Enter no campo resolve"
    );
    unsafe {
        let _ = SetFocus(Some(h.example));
    }
    tecla(&h, h.example, VK_SPACE);
    assert_eq!(texto(h.inputs[0][0]), "2", "Espaço clica o botão focado");
    unsafe {
        let _ = SetFocus(Some(h.three));
    }
    tecla(&h, h.three, VK_RETURN);
    assert_eq!(
        texto(h.dimension),
        "3 equações · 3 incógnitas",
        "Enter clica o botão focado"
    );
    unsafe {
        let _ = SetFocus(Some(h.inputs[1][1]));
    }
    tecla(&h, h.inputs[1][1], VK_ESCAPE);
    unsafe {
        SendMessageW(
            h.main,
            WM_COMMAND,
            Some(WPARAM(IDCANCEL.0 as usize)),
            Some(LPARAM(0)),
        );
        SendMessageW(h.canvas, WM_CLOSE, None, None);
    }
    pump(h.main);
    assert!(
        unsafe { IsWindow(Some(h.main)).as_bool() } && visivel(h.main),
        "Esc/IDCANCEL nunca fecham a janela"
    );
    for row in 0..3 {
        for column in 0..4 {
            escrever(&h, row, column, "0");
        }
    }
    clicar(&h, Button::Solve);
    tecla(&h, h.general, VK_ESCAPE);
    tecla(&h, h.general, VK_RETURN);
    assert!(
        unsafe { IsWindow(Some(h.main)).as_bool() },
        "Esc no resultado geral não fecha"
    );
    unsafe {
        let _ = SetFocus(Some(h.inputs[0][0]));
    }
    escrever(&h, 0, 0, "12 34");
    unsafe {
        SendMessageW(h.inputs[0][0], EM_SETSEL, Some(WPARAM(5)), Some(LPARAM(5)));
    }
    com_ctrl(|| tecla(&h, h.inputs[0][0], VK_BACK));
    assert_eq!(
        texto(h.inputs[0][0]),
        "12 ",
        "Ctrl+Backspace apaga a palavra anterior"
    );
    digitar(&h, h.inputs[0][0], "\u{7f}");
    assert_eq!(
        texto(h.inputs[0][0]),
        "12 ",
        "caractere de controle não é inserido"
    );
    com_ctrl(|| {
        tecla(
            &h,
            h.inputs[0][0],
            windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(u16::from(b'A')),
        )
    });
    assert_eq!(selecao(h.inputs[0][0]), (0, 3), "Ctrl+A seleciona tudo");
    digitar(&h, h.inputs[0][0], "7");
    assert_eq!(texto(h.inputs[0][0]), "7");
    fechar(&h);
}

#[test]
fn fonte_space_grotesk_e_margens_dos_edits() {
    let h = janela();
    for hwnd in [h.inputs[0][0], h.inputs[2][3], h.values[0], h.general] {
        let (face, altura) = face_e_altura(hwnd);
        assert_eq!(face, "Space Grotesk", "fonte do EDIT");
        assert!(altura > 0);
        assert_eq!(
            unsafe { SendMessageW(hwnd, EM_GETMARGINS, None, None) }.0,
            0,
            "margens zeradas como no WinForms"
        );
    }
    let (_, entrada) = face_e_altura(h.inputs[0][0]);
    let (_, valor) = face_e_altura(h.values[0]);
    assert!(valor > entrada, "fonte Value maior que Input");
    let mut tooltip = [0u16; 128];
    let mut info = TTTOOLINFOW {
        cbSize: std::mem::size_of::<TTTOOLINFOW>() as u32,
        uFlags: TTF_IDISHWND,
        hwnd: h.canvas,
        uId: h.author.0 as usize,
        lpszText: PWSTR(tooltip.as_mut_ptr()),
        ..Default::default()
    };
    unsafe {
        SendMessageW(
            h.tooltip,
            TTM_GETTEXTW,
            Some(WPARAM(tooltip.len())),
            Some(LPARAM((&mut info as *mut TTTOOLINFOW) as isize)),
        );
    }
    let fim = tooltip.iter().position(|unit| *unit == 0).unwrap_or(0);
    assert_eq!(
        String::from_utf16_lossy(&tooltip[..fim]),
        "Visitar lucas-lima.xyz"
    );
    let mut info = MINMAXINFO::default();
    unsafe {
        SendMessageW(
            h.main,
            WM_GETMINMAXINFO,
            None,
            Some(LPARAM((&mut info as *mut MINMAXINFO) as isize)),
        );
    }
    let dpi = diagnostics(h.main).map_or(96, |diag| diag.dpi);
    let escala = dpi as f32 / 96.0;
    assert_eq!(
        info.ptMinTrackSize,
        POINT {
            x: d(escala, 640),
            y: d(escala, 480)
        }
    );
    fechar(&h);
}

fn pixels_de_icone_gdi(icon: Icon) -> usize {
    let recursos = GdiResources::new(96);
    let mut bits = std::ptr::null_mut();
    let info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: 48,
            biHeight: -48,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    unsafe {
        let dc = CreateCompatibleDC(None);
        let Ok(bitmap) = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0)
        else {
            let _ = DeleteDC(dc);
            return 0;
        };
        let anterior = SelectObject(dc, HGDIOBJ(bitmap.0));
        recursos.paint(
            dc,
            RECT {
                left: 0,
                top: 0,
                right: 48,
                bottom: 48,
            },
            |painter| {
                painter.clear(Color::rgb(255, 255, 255));
                painter.icon(icon, 0.0, 0.0, 48.0, 48.0, Color::rgb(0, 0, 0));
            },
        );
        let pixels = std::slice::from_raw_parts(bits.cast::<u8>(), 48 * 48 * 4);
        let total = pixels
            .chunks_exact(4)
            .filter(|pixel| pixel[0] != 255 || pixel[1] != 255 || pixel[2] != 255)
            .count();
        SelectObject(dc, anterior);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
        total
    }
}

#[test]
fn icones_desenham_mais_de_20_pixels() {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        );
    }
    match Engine::new(96) {
        Ok(engine) => {
            assert!(
                engine.uses_embedded_font(),
                "DirectWrite usa a Space Grotesk embutida"
            );
            assert_eq!(engine.family(), "Space Grotesk");
            for icon in Icon::ALL {
                let pixels = engine
                    .rasterize_icon(icon, 48, Color::rgb(255, 255, 255), Color::rgb(0, 0, 0))
                    .unwrap_or_default();
                let total = pixels
                    .chunks_exact(4)
                    .filter(|pixel| pixel[0] != 255 || pixel[1] != 255 || pixel[2] != 255)
                    .count();
                assert!(
                    total > 20,
                    "ícone {} (Direct2D) desenhou {total} pixels",
                    icon.name()
                );
            }
        }
        Err(error) => eprintln!(
            "aviso: Direct2D indisponível neste ambiente ({error}); só o desenho GDI foi verificado"
        ),
    }
    for icon in Icon::ALL {
        let total = pixels_de_icone_gdi(icon);
        assert!(
            total > 20,
            "ícone {} (GDI) desenhou {total} pixels",
            icon.name()
        );
    }
}

#[test]
fn primeira_pintura_escura_sem_branco() {
    let h = janela();
    let classe = unsafe { GetClassLongPtrW(h.main, GCLP_HBRBACKGROUND) };
    let mut pincel = LOGBRUSH::default();
    let lido = unsafe {
        GetObjectW(
            HGDIOBJ(classe as *mut _),
            std::mem::size_of::<LOGBRUSH>() as i32,
            Some((&mut pincel as *mut LOGBRUSH).cast()),
        )
    };
    assert!(lido > 0, "a classe da janela tem pincel de fundo");
    assert_eq!(
        pincel.lbColor,
        COLORREF(theme::CANVAS.colorref()),
        "fundo da classe é o Canvas escuro, nunca branco"
    );
    let imagem = capturar(h.main);
    assert!(
        imagem.largura >= 1160 && imagem.altura >= 810,
        "{}x{}",
        imagem.largura,
        imagem.altura
    );
    let brancos = imagem.conta((0, 0, imagem.largura, imagem.altura), |cor| {
        cor.r == 255 && cor.g == 255 && cor.b == 255
    });
    assert_eq!(
        brancos, 0,
        "pixels brancos na área cliente ({})",
        imagem.metodo
    );
    for (x, y) in [(4, 4), (600, 140), (1150, 800), (20, 400)] {
        assert_eq!(
            imagem.cor(x, y),
            theme::CANVAS,
            "fundo em ({x},{y}) via {}",
            imagem.metodo
        );
    }
    assert_eq!(
        imagem.cor(900, 320),
        theme::ACCENT,
        "card de resultado lilás"
    );
    let tinta = imagem.conta((79, 36, 329, 64), |cor| distancia(cor, theme::INK) < 60);
    assert!(tinta > 50, "texto da marca desenhado ({tinta} pixels)");
    fechar(&h);
}

#[test]
fn fallback_gdi_continua_visivel() {
    let h = janela_gdi();
    let diag = diagnostics(h.main).expect("diagnóstico");
    assert!(!diag.direct2d, "janela forçada a usar GDI");
    let imagem = capturar(h.main);
    assert_eq!(
        imagem.cor(4, 4),
        theme::CANVAS,
        "fundo via {}",
        imagem.metodo
    );
    assert_eq!(imagem.cor(900, 320), theme::ACCENT, "card de resultado");
    let marca = imagem.conta((79, 36, 329, 64), |cor| distancia(cor, theme::INK) < 120);
    assert!(marca > 40, "texto da marca no fallback ({marca})");
    let titulo = imagem.conta((696, 465, 1091, 505), |cor| {
        distancia(cor, theme::CANVAS) < 120
    });
    assert!(titulo > 100, "título vazio no fallback ({titulo})");
    let botao = imagem.conta((58, 667, 250, 709), |cor| {
        distancia(cor, theme::ACCENT) < 30
    });
    assert!(
        botao > 2000,
        "botão Resolver preenchido no fallback ({botao})"
    );
    clicar(&h, Button::Example);
    clicar(&h, Button::Solve);
    let resolvida = capturar(h.main);
    let tile = resolvida.conta((702, 406, 1085, 464), |cor| {
        distancia(cor, theme::TILE_FILL) < 20
    });
    assert!(tile > 5000, "tile de valor no fallback ({tile})");
    fechar(&h);
}

#[test]
fn alto_contraste_reage_ao_vivo() {
    let h = janela_com(Launch {
        high_contrast: Some(false),
        ..Launch::background(FORA_DA_TELA, CLIENTE)
    });
    assert!(
        !diagnostics(h.main)
            .expect("diagnóstico")
            .palette
            .high_contrast
    );
    force_high_contrast(h.main, Some(true));
    pump(h.main);
    let sistema = system_colors();
    let paleta = diagnostics(h.main).expect("diagnóstico").palette;
    assert!(paleta.high_contrast);
    assert_eq!(paleta.canvas, sistema.window);
    assert_eq!(paleta.ink, sistema.window_text);
    assert_eq!(paleta.accent_fill, sistema.highlight);
    assert_eq!(paleta.link, sistema.hot_light);
    let imagem = capturar(h.main);
    assert_eq!(
        imagem.cor(4, 4),
        sistema.window,
        "fundo com COLOR_WINDOW via {}",
        imagem.metodo
    );
    assert_eq!(
        imagem.cor(900, 320),
        sistema.window,
        "card de resultado sem lilás no alto contraste"
    );
    assert_eq!(
        imagem.cor(240, 688),
        sistema.highlight,
        "botão primário com COLOR_HIGHLIGHT"
    );
    let campo = imagem.cor(110, 470);
    assert_eq!(campo, sistema.window, "interior do campo com COLOR_WINDOW");
    let brush = unsafe {
        SendMessageW(
            h.canvas,
            WM_CTLCOLOREDIT,
            Some(WPARAM(0)),
            Some(LPARAM(h.inputs[0][0].0 as isize)),
        )
    };
    let _ = brush;
    force_high_contrast(h.main, Some(false));
    pump(h.main);
    let escura = capturar(h.main);
    assert_eq!(
        escura.cor(4, 4),
        theme::CANVAS,
        "volta ao tema escuro sem reiniciar"
    );
    assert_eq!(escura.cor(900, 320), theme::ACCENT);
    force_high_contrast(h.main, None);
    unsafe {
        SendMessageW(h.main, WM_SYSCOLORCHANGE, None, None);
        SendMessageW(
            h.main,
            WM_SETTINGCHANGE,
            Some(WPARAM(SPI_SETHIGHCONTRAST.0 as usize)),
            None,
        );
        SendMessageW(h.main, WM_THEMECHANGED, None, None);
    }
    pump(h.main);
    assert_eq!(
        diagnostics(h.main)
            .expect("diagnóstico")
            .palette
            .high_contrast,
        system_high_contrast(),
        "segue o estado real do Windows"
    );
    let _ = HBRUSH::default();
    fechar(&h);
}

thread_local! {
    static EVENTOS: RefCell<Vec<isize>> = const { RefCell::new(Vec::new()) };
}

unsafe extern "system" fn ao_evento(
    _: HWINEVENTHOOK,
    evento: u32,
    hwnd: HWND,
    objeto: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if evento == EVENT_OBJECT_LIVEREGIONCHANGED && objeto == OBJID_CLIENT.0 {
        EVENTOS.with(|eventos| eventos.borrow_mut().push(hwnd.0 as isize));
    }
}

fn esperar_evento(h: &sistemas_lineares::Handles, alvo: HWND) -> bool {
    let limite = Instant::now() + Duration::from_secs(3);
    while Instant::now() < limite {
        pump(h.main);
        if EVENTOS.with(|eventos| eventos.borrow().contains(&(alvo.0 as isize))) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn leitor_de_tela_nomes_e_regioes_vivas() {
    let h = janela();
    assert_eq!(
        nome_acessivel(h.inputs[0][0]),
        "Equação 1, coeficiente de x"
    );
    assert_eq!(
        nome_acessivel(h.inputs[1][3]),
        "Equação 2, termo independente"
    );
    assert_eq!(
        nome_acessivel(h.inputs[2][2]),
        "Equação 3, coeficiente de z"
    );
    assert_eq!(nome_acessivel(h.values[1]), "Valor de y");
    assert_eq!(nome_acessivel(h.author), "Feito por Lucas Lima · RA 185964");
    assert_eq!(descricao_acessivel(h.author), model::AUTHOR_DESCRIPTION);
    assert_eq!(nome_acessivel(h.solve), "Resolver sistema");
    let diag = diagnostics(h.main).expect("diagnóstico");
    assert_eq!(
        diag.live_regions, 4,
        "helper, título, nota e dimensão são regiões vivas"
    );

    let gancho = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_LIVEREGIONCHANGED,
            EVENT_OBJECT_LIVEREGIONCHANGED,
            None,
            Some(ao_evento),
            GetCurrentProcessId(),
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    assert!(!gancho.is_invalid(), "gancho de eventos de acessibilidade");
    EVENTOS.with(|eventos| eventos.borrow_mut().clear());
    clicar(&h, Button::Solve);
    assert!(esperar_evento(&h, h.helper), "erro anunciado pelo helper");
    clicar(&h, Button::Example);
    clicar(&h, Button::Solve);
    assert!(
        esperar_evento(&h, h.title),
        "resultado anunciado pelo título"
    );
    assert_eq!(nome_acessivel(h.title), "Solução encontrada: x = 2, y = 1.");
    EVENTOS.with(|eventos| eventos.borrow_mut().clear());
    clicar(&h, Button::Three);
    assert!(
        esperar_evento(&h, h.dimension),
        "troca de tamanho anunciada"
    );
    unsafe {
        let _ = UnhookWinEvent(gancho);
    }

    let helper = h.helper.0 as isize;
    let concluido = Arc::new(AtomicBool::new(false));
    let sinal = concluido.clone();
    let (envio, recebe) = mpsc::channel();
    let trabalhador = std::thread::spawn(move || {
        let resultado = unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let automacao: windows::core::Result<IUIAutomation> =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER);
            match automacao {
                Ok(automacao) => automacao
                    .ElementFromHandle(HWND(helper as *mut _))
                    .and_then(|elemento| {
                        elemento.GetCurrentPropertyValue(UIA_LiveSettingPropertyId)
                    })
                    .map(|valor| {
                        (
                            valor.Anonymous.Anonymous.vt,
                            valor.Anonymous.Anonymous.Anonymous.lVal,
                        )
                    })
                    .ok(),
                Err(_) => None,
            }
        };
        let _ = envio.send(resultado);
        sinal.store(true, Ordering::SeqCst);
    });
    let limite = Instant::now() + Duration::from_secs(15);
    while !concluido.load(Ordering::SeqCst) && Instant::now() < limite {
        pump(h.main);
        std::thread::sleep(Duration::from_millis(5));
    }
    match recebe.recv_timeout(Duration::from_secs(1)) {
        Ok(Some((tipo, valor))) => {
            assert_eq!(tipo, VT_I4, "LiveSetting via UI Automation");
            assert_eq!(valor, 1, "LiveSetting = Polite");
        }
        Ok(None) => eprintln!(
            "aviso: UI Automation indisponível neste ambiente; LiveSetting conferido só pelos eventos"
        ),
        Err(_) => eprintln!(
            "aviso: UI Automation não respondeu a tempo; LiveSetting conferido só pelos eventos"
        ),
    }
    let _ = trabalhador.join();
    fechar(&h);
}

#[test]
fn dpi_por_monitor_recria_fontes_e_layout() {
    let h = janela();
    let antes = diagnostics(h.main).expect("diagnóstico");
    let (_, altura_antes) = face_e_altura(h.inputs[0][0]);
    let mut janela_antes = RECT::default();
    unsafe {
        let _ = GetWindowRect(h.main, &mut janela_antes);
    }
    let sugerido = RECT {
        left: janela_antes.left,
        top: janela_antes.top,
        right: janela_antes.left + 1764,
        bottom: janela_antes.top + 1274,
    };
    unsafe {
        SendMessageW(
            h.main,
            WM_DPICHANGED,
            Some(WPARAM(144 | (144 << 16))),
            Some(LPARAM((&sugerido as *const RECT) as isize)),
        );
    }
    pump(h.main);
    let depois = diagnostics(h.main).expect("diagnóstico");
    assert_eq!(depois.dpi, 144);
    let (face, altura_depois) = face_e_altura(h.inputs[0][0]);
    assert_eq!(face, "Space Grotesk");
    assert!(
        altura_depois * 2 >= altura_antes * 3 - 2,
        "fonte GDI recriada para 144 DPI: {altura_antes} -> {altura_depois}"
    );
    assert_eq!(depois.layout.solve.w, d(1.5, 192));
    assert_eq!(depois.layout.author.h, d(1.5, 42));
    let mut janela_depois = RECT::default();
    unsafe {
        let _ = GetWindowRect(h.main, &mut janela_depois);
    }
    assert_eq!(janela_depois, sugerido, "usa o retângulo sugerido");
    assert_ne!(antes.layout.solve, depois.layout.solve);
    let imagem = capturar(h.main);
    assert_eq!(imagem.cor(6, 6), theme::CANVAS);
    unsafe {
        SendMessageW(
            h.main,
            WM_DPICHANGED,
            Some(WPARAM(96 | (96 << 16))),
            Some(LPARAM((&janela_antes as *const RECT) as isize)),
        );
    }
    pump(h.main);
    assert_eq!(
        diagnostics(h.main).expect("diagnóstico").layout.solve,
        antes.layout.solve,
        "volta ao layout de 96 DPI"
    );
    fechar(&h);
}

#[test]
fn janela_minima_rola_e_mantem_o_foco_visivel() {
    let h = janela();
    clicar(&h, Button::Three);
    unsafe {
        let _ = SetWindowPos(
            h.main,
            None,
            0,
            0,
            640,
            480,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    pump(h.main);
    let estilo = unsafe { GetWindowLongW(h.main, GWL_STYLE) } as u32;
    assert!(
        estilo & WS_VSCROLL.0 != 0 && estilo & WS_HSCROLL.0 != 0,
        "barras de rolagem visíveis"
    );
    let diag = diagnostics(h.main).expect("diagnóstico");
    let mut cliente = RECT::default();
    unsafe {
        let _ = GetClientRect(h.main, &mut cliente);
    }
    let campo = diag.layout.fields[0][0];
    assert!(diag.scroll.1 > 0, "rolou até o campo focado");
    assert!(
        campo.y - diag.scroll.1 >= 0 && campo.bottom() - diag.scroll.1 <= cliente.bottom,
        "campo focado visível: {campo:?} rolagem {:?} cliente {cliente:?}",
        diag.scroll
    );
    let vertical = diag.scroll.1;
    unsafe {
        SendMessageW(
            h.main,
            WM_MOUSEWHEEL,
            Some(WPARAM((120usize) << 16)),
            Some(LPARAM(0)),
        );
    }
    pump(h.main);
    assert!(
        diagnostics(h.main).expect("diagnóstico").scroll.1 < vertical,
        "roda do mouse rola para cima"
    );
    unsafe {
        SendMessageW(
            h.main,
            WM_MOUSEHWHEEL,
            Some(WPARAM((120usize) << 16)),
            Some(LPARAM(0)),
        );
    }
    pump(h.main);
    assert!(
        diagnostics(h.main).expect("diagnóstico").scroll.0 > 0,
        "rolagem horizontal"
    );
    unsafe {
        let _ = SetFocus(Some(h.author));
    }
    pump(h.main);
    let diag = diagnostics(h.main).expect("diagnóstico");
    assert!(
        diag.layout.author.right() - diag.scroll.0 <= cliente.right
            && diag.layout.author.y - diag.scroll.1 >= 0,
        "foco no autor rola até ele"
    );
    fechar(&h);
}

#[test]
fn reativar_devolve_o_foco() {
    let h = janela();
    unsafe {
        let _ = SetFocus(Some(h.example));
        SendMessageW(
            h.main,
            WM_ACTIVATE,
            Some(WPARAM(WA_INACTIVE as usize)),
            Some(LPARAM(0)),
        );
        let _ = SetFocus(None);
        SendMessageW(
            h.main,
            WM_ACTIVATE,
            Some(WPARAM(WA_ACTIVE as usize)),
            Some(LPARAM(0)),
        );
    }
    pump(h.main);
    assert_eq!(foco(), h.example, "foco devolvido ao último controle");
    nao_e_primeiro_plano(&h);
    fechar(&h);
}

#[test]
fn hover_cursor_de_mao_e_foco_so_pelo_teclado() {
    let h = janela();
    let antes = capturar(h.main);
    let ponto = (262 + 6, 667 + 21);
    assert_eq!(
        antes.cor(ponto.0, ponto.1),
        theme::SURFACE,
        "botão Usar exemplo em repouso"
    );
    unsafe {
        SendMessageW(
            h.example,
            WM_MOUSEMOVE,
            Some(WPARAM(0)),
            Some(LPARAM(5 | (5 << 16))),
        );
    }
    let sobre = capturar(h.main);
    assert_eq!(
        sobre.cor(ponto.0, ponto.1),
        theme::SOFT,
        "hover pinta o botão com Soft"
    );
    pump(h.main);
    unsafe {
        SendMessageW(h.example, WM_MOUSELEAVE, None, None);
    }
    pump(h.main);
    assert_eq!(
        capturar(h.main).cor(ponto.0, ponto.1),
        theme::SURFACE,
        "sair do botão remove o hover"
    );
    let resultado = unsafe {
        SendMessageW(
            h.example,
            WM_SETCURSOR,
            Some(WPARAM(h.example.0 as usize)),
            Some(LPARAM(
                (HTCLIENT as isize) | ((WM_MOUSEMOVE as isize) << 16),
            )),
        )
    };
    assert_eq!(resultado.0, 1, "WM_SETCURSOR tratado no botão habilitado");
    let mao = unsafe { LoadCursorW(None, IDC_HAND) }.unwrap_or_default();
    assert_eq!(unsafe { GetCursor() }, mao, "cursor de mão");
    let desabilitado = unsafe {
        SendMessageW(
            h.copy,
            WM_SETCURSOR,
            Some(WPARAM(h.copy.0 as usize)),
            Some(LPARAM(
                (HTCLIENT as isize) | ((WM_MOUSEMOVE as isize) << 16),
            )),
        )
    };
    assert_eq!(desabilitado.0, 0, "botão desabilitado não troca o cursor");
    let mut dicas = windows::core::BOOL(0);
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_GETKEYBOARDCUES,
            0,
            Some((&mut dicas as *mut windows::core::BOOL).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        let _ = SetFocus(Some(h.example));
    }
    pump(h.main);
    let anel = |imagem: &Imagem| {
        imagem.conta((262 + 4, 667 + 4, 262 + 151, 667 + 38), |cor| {
            distancia(cor, theme::ACCENT) < 40
        })
    };
    let sem_teclado = anel(&capturar(h.main));
    if dicas.as_bool() {
        eprintln!(
            "aviso: o Windows está configurado para sempre mostrar o foco; verificado só o anel visível"
        );
        assert!(sem_teclado > 20, "anel de foco visível ({sem_teclado})");
    } else {
        assert_eq!(sem_teclado, 0, "sem anel de foco antes de usar o teclado");
    }
    unsafe {
        SendMessageW(
            h.main,
            WM_CHANGEUISTATE,
            Some(WPARAM(
                (UIS_CLEAR as usize) | ((UISF_HIDEFOCUS as usize) << 16),
            )),
            None,
        );
    }
    pump(h.main);
    let com_teclado = anel(&capturar(h.main));
    assert!(
        com_teclado > 20,
        "anel de foco pontilhado aparece com o teclado ({com_teclado})"
    );
    fechar(&h);
}

#[test]
fn janela_aparece_ja_pintada_e_sem_camuflagem() {
    let h = janela();
    let mut camuflada = 0u32;
    let lido = unsafe {
        windows::Win32::Graphics::Dwm::DwmGetWindowAttribute(
            h.main,
            windows::Win32::Graphics::Dwm::DWMWA_CLOAKED,
            (&mut camuflada as *mut u32).cast(),
            4,
        )
    };
    if lido.is_ok() {
        assert_eq!(
            camuflada, 0,
            "a janela é revelada depois da primeira pintura"
        );
    } else {
        eprintln!("aviso: DWM indisponível neste ambiente; camuflagem não verificada");
    }
    assert!(visivel(h.main));
    fechar(&h);
}

#[test]
fn tamanho_explicito_ignora_o_limite_da_tela() {
    let pedido = (2600, 1600);
    let h = janela_com(Launch::background(FORA_DA_TELA, pedido));
    let mut cliente = RECT::default();
    unsafe {
        let _ = GetClientRect(h.main, &mut cliente);
    }
    assert_eq!(
        (cliente.right - cliente.left, cliente.bottom - cliente.top),
        pedido,
        "janela de teste com o tamanho pedido mesmo em tela menor, como a do GitHub Actions"
    );
    fechar(&h);
}
