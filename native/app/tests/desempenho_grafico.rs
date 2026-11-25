#![cfg(windows)]

mod comum;

use std::time::{Duration, Instant};

use comum::*;
use sistemas_lineares::{Button, diagnostics};
use windows::Win32::Graphics::Gdi::{RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow};

#[test]
fn pintar_um_quadro_do_painel_custa_poucos_milissegundos() {
    for dimensao in [2, 3] {
        let h = janela_animada(false);
        if dimensao == 3 {
            clicar(&h, Button::Three);
        }
        clicar(&h, Button::Example);
        let quadros =
            |h: &sistemas_lineares::Handles| diagnostics(h.main).expect("diagnóstico").graph_frames;
        let antes = quadros(&h);
        let vezes = 41;
        let mut tempos: Vec<Duration> = (0..vezes)
            .map(|_| {
                let inicio = Instant::now();
                unsafe {
                    let _ = RedrawWindow(Some(h.graph), None, None, RDW_INVALIDATE | RDW_UPDATENOW);
                }
                inicio.elapsed()
            })
            .collect();
        tempos.sort();
        let mediana = tempos[vezes / 2];
        let pintados = quadros(&h) - antes;
        eprintln!(
            "{dimensao}x{dimensao}: mediana de {mediana:?} por quadro, pior {:?} ({pintados} quadros)",
            tempos[vezes - 1]
        );
        assert!(pintados >= vezes, "{pintados} quadros pintados");
        assert!(
            mediana < Duration::from_millis(16),
            "{dimensao}x{dimensao}: {mediana:?} por quadro, acima do orçamento de 60 fps"
        );
        fechar(&h);
    }
}
