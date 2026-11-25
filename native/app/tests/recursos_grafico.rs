#![cfg(windows)]

mod comum;

use comum::*;
use sistemas_lineares::{Button, Handles, Launch, Renderer};
use windows::Win32::Graphics::Gdi::{RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow};
use windows::Win32::System::Threading::{GR_GDIOBJECTS, GetCurrentProcess, GetGuiResources};

fn pintar_o_grafico(h: &Handles, vezes: usize) {
    for _ in 0..vezes {
        unsafe {
            let _ = RedrawWindow(Some(h.graph), None, None, RDW_INVALIDATE | RDW_UPDATENOW);
        }
    }
}

#[test]
fn quadros_do_grafico_nao_vazam_objetos_gdi() {
    for renderer in [Renderer::Gdi, Renderer::Direct2D] {
        let h = janela_com(Launch {
            renderer,
            animations: Some(false),
            ..Launch::background(FORA_DA_TELA, CLIENTE)
        });
        for tamanho in [Button::Three, Button::Two] {
            clicar(&h, tamanho);
            clicar(&h, Button::Example);
            pintar_o_grafico(&h, 2);
        }
        let processo = unsafe { GetCurrentProcess() };
        let antes = unsafe { GetGuiResources(processo, GR_GDIOBJECTS) };
        for tamanho in [Button::Three, Button::Two, Button::Three, Button::Two] {
            clicar(&h, tamanho);
            clicar(&h, Button::Example);
            pintar_o_grafico(&h, 60);
            escrever(&h, 0, 3, "0");
            pintar_o_grafico(&h, 60);
        }
        let depois = unsafe { GetGuiResources(processo, GR_GDIOBJECTS) };
        assert!(
            depois <= antes + 4,
            "{renderer:?}: objetos GDI cresceram de {antes} para {depois} em 480 quadros"
        );
        fechar(&h);
    }
}
