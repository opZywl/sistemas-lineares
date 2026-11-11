mod comum;

use std::time::{SystemTime, UNIX_EPOCH};

use comum::{clicar, fechar, janela_com};
use sistemas_lineares::registry::{self, Key};
use sistemas_lineares::settings::{self, Bounds, Placement, Saved};
use sistemas_lineares::window::startup_show;
use sistemas_lineares::{Button, Launch, diagnostics, pump};
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONULL, MONITOR_DEFAULTTOPRIMARY, MONITORINFO,
    MonitorFromPoint, MonitorFromRect,
};
use windows::Win32::UI::WindowsAndMessaging::*;

const RAIZ: &str = r"Software\Lucas Lima\Testes Sistemas Lineares";

struct Chave(String);

impl Chave {
    fn nova(nome: &str) -> Chave {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |tempo| tempo.as_nanos());
        Chave(format!(r"{RAIZ}\{nome}-{}-{nanos}", std::process::id()))
    }
}

impl Drop for Chave {
    fn drop(&mut self) {
        registry::delete_tree(&self.0);
        registry::delete_if_empty(RAIZ);
        registry::delete_if_empty(r"Software\Lucas Lima");
    }
}

fn oculta(chave: &Chave) -> Launch {
    Launch {
        position: None,
        client_size: None,
        show: SW_HIDE,
        quit_on_close: false,
        settings_key: Some(chave.0.clone()),
        activate: false,
        ..Launch::default()
    }
}

fn area_principal() -> Bounds {
    unsafe {
        let monitor = MonitorFromPoint(Default::default(), MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        Bounds {
            left: info.rcWork.left,
            top: info.rcWork.top,
            width: info.rcWork.right - info.rcWork.left,
            height: info.rcWork.bottom - info.rcWork.top,
        }
    }
}

fn retangulo(hwnd: windows::Win32::Foundation::HWND) -> RECT {
    let mut rect = RECT::default();
    unsafe {
        let _ = GetWindowRect(hwnd, &mut rect);
    }
    rect
}

#[test]
fn salva_e_restaura_posicao_tamanho_e_dimensao() {
    let chave = Chave::nova("restaura");
    let area = area_principal();
    let alvo = RECT {
        left: area.left + 37,
        top: area.top + 41,
        right: area.left + 37 + 1000,
        bottom: area.top + 41 + 700,
    };
    let h = janela_com(oculta(&chave));
    assert!(!comum::visivel(h.main), "janela de teste nunca aparece");
    clicar(&h, Button::Three);
    unsafe {
        let _ = SetWindowPos(
            h.main,
            None,
            alvo.left,
            alvo.top,
            1000,
            700,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    pump(h.main);
    fechar(&h);
    let salvo = settings::load(&chave.0);
    let lugar = salvo.placement.expect("posição salva no fechamento");
    assert_eq!(
        lugar.bounds,
        Bounds {
            left: alvo.left,
            top: alvo.top,
            width: 1000,
            height: 700
        }
    );
    assert!(!lugar.maximized);
    assert_eq!(salvo.dimension, Some(3));
    let chave_aberta = Key::open(&chave.0).expect("chave HKCU de teste");
    assert_eq!(chave_aberta.dword("Largura"), Some(1000));
    assert_eq!(chave_aberta.dword("Dimensao"), Some(3));
    drop(chave_aberta);
    let h = janela_com(oculta(&chave));
    assert_eq!(retangulo(h.main), alvo, "reabre no mesmo lugar e tamanho");
    assert_eq!(
        diagnostics(h.main).expect("diagnóstico").dimension,
        3,
        "reabre em 3x3"
    );
    clicar(&h, Button::Two);
    fechar(&h);
    assert_eq!(settings::load(&chave.0).dimension, Some(2));
}

#[test]
fn retangulo_sem_monitor_volta_ao_padrao() {
    let chave = Chave::nova("sem-monitor");
    let longe = Placement {
        bounds: Bounds {
            left: -60_000,
            top: -60_000,
            width: 900,
            height: 700,
        },
        maximized: false,
        dpi: 96,
    };
    assert!(settings::save(
        &chave.0,
        &Saved {
            placement: Some(longe),
            dimension: Some(2)
        }
    ));
    let h = janela_com(oculta(&chave));
    let rect = retangulo(h.main);
    assert_ne!(rect.left, -60_000);
    let monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) };
    assert!(
        !monitor.is_invalid(),
        "janela criada sobre um monitor existente: {rect:?}"
    );
    fechar(&h);
}

#[test]
fn nunca_restaura_minimizada() {
    let chave = Chave::nova("minimizada");
    let area = area_principal();
    let h = janela_com(oculta(&chave));
    unsafe {
        let _ = SetWindowPos(
            h.main,
            None,
            area.left + 10,
            area.top + 10,
            900,
            650,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        let mut lugar = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        let _ = GetWindowPlacement(h.main, &mut lugar);
        lugar.showCmd = SW_SHOWMINNOACTIVE.0 as u32;
        lugar.flags = WPF_RESTORETOMAXIMIZED;
        let _ = SetWindowPlacement(h.main, &lugar);
    }
    pump(h.main);
    assert!(unsafe { IsIconic(h.main).as_bool() });
    fechar(&h);
    let salvo = settings::load(&chave.0).placement.expect("posição salva");
    assert!(
        salvo.maximized,
        "minimizada a partir de maximizada volta maximizada"
    );
    assert_eq!(
        (salvo.bounds.width, salvo.bounds.height),
        (900, 650),
        "salva o retângulo normal, não o minimizado"
    );
    assert!(salvo.bounds.left > -30_000 && salvo.bounds.top > -30_000);
    assert_eq!(startup_show(true, SW_SHOWNORMAL), SW_SHOWMAXIMIZED);
    assert_eq!(startup_show(true, SW_SHOWMINIMIZED), SW_SHOWMAXIMIZED);
    assert_eq!(startup_show(false, SW_SHOWMINNOACTIVE), SW_SHOWNORMAL);
    assert_eq!(startup_show(false, SW_MINIMIZE), SW_SHOWNORMAL);
    assert_eq!(startup_show(true, SW_HIDE), SW_HIDE);
    assert_eq!(startup_show(false, SW_SHOWNOACTIVATE), SW_SHOWNOACTIVATE);
    let h = janela_com(oculta(&chave));
    assert!(
        !unsafe { IsIconic(h.main).as_bool() },
        "nunca reabre minimizada"
    );
    fechar(&h);
}

#[test]
fn testes_e_capturas_ignoram_a_persistencia() {
    let launch = Launch::background((-4000, 0), (1160, 810));
    assert_eq!(launch.settings_key, None);
    assert_eq!(
        Launch::default().settings_key.as_deref(),
        Some(settings::KEY)
    );
    assert_eq!(settings::KEY, r"Software\Lucas Lima\Sistemas Lineares");
    let chave = Chave::nova("invalida");
    let gravada = Key::create(&chave.0).expect("chave");
    assert!(
        gravada.set_dword("Largura", 0)
            && gravada.set_dword("Altura", 500)
            && gravada.set_dword("Esquerda", 1)
            && gravada.set_dword("Topo", 1)
            && gravada.set_dword("Dpi", 96)
            && gravada.set_dword("Dimensao", 9)
    );
    drop(gravada);
    assert_eq!(
        settings::load(&chave.0),
        Saved::default(),
        "valores inválidos são ignorados"
    );
}
