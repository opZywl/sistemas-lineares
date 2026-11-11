mod comum;

use comum::*;
use sistemas_lineares::Button;
use windows::Win32::System::Threading::{
    GR_GDIOBJECTS, GR_USEROBJECTS, GetCurrentProcess, GetGuiResources,
};

#[test]
fn gdi_e_user_estaveis_apos_200_ciclos() {
    let h = janela();
    let ciclo = |h: &sistemas_lineares::Handles, indice: usize| {
        clicar(
            h,
            if indice % 2 == 0 {
                Button::Three
            } else {
                Button::Two
            },
        );
        clicar(h, Button::Example);
        clicar(h, Button::Solve);
        escrever(h, 0, 0, "0");
        escrever(h, 1, 0, "0");
        escrever(h, 0, 1, if indice % 3 == 0 { "0" } else { "1" });
        clicar(h, Button::Solve);
        escrever(h, 0, 3, "1/0");
        clicar(h, Button::Solve);
        clicar(h, Button::Clear);
        let _ = capturar(h.main);
    };
    for indice in 0..20 {
        ciclo(&h, indice);
    }
    let processo = unsafe { GetCurrentProcess() };
    let gdi_antes = unsafe { GetGuiResources(processo, GR_GDIOBJECTS) };
    let user_antes = unsafe { GetGuiResources(processo, GR_USEROBJECTS) };
    for indice in 0..200 {
        ciclo(&h, indice);
    }
    let gdi_depois = unsafe { GetGuiResources(processo, GR_GDIOBJECTS) };
    let user_depois = unsafe { GetGuiResources(processo, GR_USEROBJECTS) };
    assert!(
        gdi_depois <= gdi_antes + 4,
        "objetos GDI cresceram: {gdi_antes} -> {gdi_depois}"
    );
    assert!(
        user_depois <= user_antes + 4,
        "objetos USER cresceram: {user_antes} -> {user_depois}"
    );
    fechar(&h);
}
