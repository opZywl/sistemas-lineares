mod comum;

use comum::*;
use sistemas_lineares::Button;

#[test]
fn wm_print_reproduz_a_tela() {
    let h = janela();
    clicar(&h, Button::Example);
    clicar(&h, Button::Solve);
    let tela = capturar(h.main);
    let impressa = imprimir(h.main);
    let mut diferentes = 0usize;
    for y in 0..tela.altura {
        for x in 0..tela.largura {
            if distancia(tela.cor(x, y), impressa.cor(x, y)) > 60 {
                diferentes += 1;
            }
        }
    }
    let total = (tela.largura * tela.altura) as usize;
    eprintln!(
        "{} x WM_PRINT: {diferentes} de {total} pixels diferentes",
        tela.metodo
    );
    assert!(
        diferentes * 1000 < total * 5,
        "WM_PRINT diverge da tela: {diferentes} pixels"
    );
    fechar(&h);
}
