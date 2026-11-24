#![cfg(windows)]

mod comum;

use std::time::{Duration, Instant};

use comum::*;
use sistemas_lineares::model::{self, Card, Waiting};
use sistemas_lineares::theme;
use sistemas_lineares::{Button, Handles, Launch, diagnostics, pump};
use sistemas_lineares_core::Method;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Controls::{EM_SETSEL, WM_MOUSELEAVE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SetFocus, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DOWN, VK_ESCAPE, VK_LCONTROL, VK_MENU,
    VK_RETURN, VK_RMENU, VK_SHIFT, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_MOUSEMOVE};

const ALTGR: [VIRTUAL_KEY; 4] = [VK_CONTROL, VK_LCONTROL, VK_MENU, VK_RMENU];

fn cartao(h: &Handles) -> Card {
    diagnostics(h.main).expect("diagnóstico").card
}

fn focar(h: &Handles, alvo: windows::Win32::Foundation::HWND) {
    unsafe {
        let _ = SetFocus(Some(alvo));
    }
    pump(h.main);
}

fn preencher(h: &Handles, valores: &[(usize, usize, &str)]) {
    for (linha, coluna, valor) in valores {
        escrever(h, *linha, *coluna, valor);
    }
}

const EXEMPLO_SEM_ULTIMO: [(usize, usize, &str); 5] = [
    (0, 0, "2"),
    (0, 1, "1"),
    (0, 3, "5"),
    (1, 0, "1"),
    (1, 1, "-1"),
];

#[test]
fn resolve_ao_vivo_esmaece_e_assenta() {
    let h = janela();
    assert_eq!(cartao(&h), Card::Waiting(Waiting::Empty));
    escrever(&h, 0, 0, "2");
    assert_eq!(
        cartao(&h),
        Card::Waiting(Waiting::Missing {
            missing: 5,
            invalid: 0
        })
    );
    assert_eq!(texto(h.empty_title), "Faltam 5 campos");
    assert_eq!(texto(h.empty_description), model::MISSING_DESCRIPTION);
    assert!(
        diagnostics(h.main).expect("diagnóstico").idle_pending,
        "cada edição agenda a pausa"
    );
    preencher(&h, &EXEMPLO_SEM_ULTIMO[1..]);
    assert_eq!(texto(h.empty_title), "Falta 1 campo");
    assert!(
        descricao_acessivel(h.empty_title).contains("Faltam: equação 2, termo independente."),
        "prévia falada: {}",
        descricao_acessivel(h.empty_title)
    );
    escrever(&h, 1, 3, "1");
    assert_eq!(
        cartao(&h),
        Card::Result { stale: false },
        "a resposta aparece sem Enter"
    );
    assert_eq!([texto(h.values[0]), texto(h.values[1])], ["2", "1"]);
    assert!(habilitado(h.copy) && habilitado(h.think));
    assert!(visivel(h.title) && !visivel(h.empty_title));

    escrever(&h, 1, 3, "");
    assert_eq!(
        cartao(&h),
        Card::Result { stale: true },
        "o resultado anterior fica esmaecido"
    );
    assert!(!habilitado(h.copy) && !habilitado(h.think));
    assert_eq!(texto(h.note), model::NOTE_STALE);
    assert_eq!([texto(h.values[0]), texto(h.values[1])], ["2", "1"]);
    let layout = diagnostics(h.main).expect("diagnóstico").layout;
    let tile = layout.tiles[0];
    let regiao = (tile.x + 8, tile.y + 8, tile.right() - 8, tile.bottom() - 8);
    let esmaecida = theme::TILE_FILL.mix(theme::ACCENT, theme::STALE_PERCENT);
    let imagem = capturar(h.main);
    let cheia = imagem.conta(regiao, |cor| distancia(cor, theme::TILE_FILL) < 8);
    let fraca = imagem.conta(regiao, |cor| distancia(cor, esmaecida) < 8);
    assert!(
        fraca > 3000 && cheia < 200,
        "tile esmaecido ({fraca} pixels esmaecidos, {cheia} cheios)"
    );

    ocioso(&h);
    assert_eq!(
        cartao(&h),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        }),
        "a pausa assenta o card"
    );
    assert!(!visivel(h.title) && visivel(h.empty_title));
    assert_eq!(texto(h.note), model::NOTE_DEFAULT);
    assert!(!diagnostics(h.main).expect("diagnóstico").idle_pending);
    fechar(&h);
}

#[test]
fn pausa_real_de_digitacao_assenta_sozinha() {
    let h = janela_com(Launch {
        idle_delay_ms: 30,
        ..Launch::background(FORA_DA_TELA, CLIENTE)
    });
    clicar(&h, Button::Example);
    escrever(&h, 0, 0, "");
    let limite = Instant::now() + Duration::from_secs(5);
    while Instant::now() < limite && cartao(&h) == (Card::Result { stale: true }) {
        pump(h.main);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        cartao(&h),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        }),
        "o temporizador de pausa assenta o card"
    );
    assert!(!diagnostics(h.main).expect("diagnóstico").idle_pending);
    fechar(&h);
}

#[test]
fn prefixo_em_digitacao_e_erro_por_campo() {
    let h = janela();
    focar(&h, h.inputs[0][0]);
    assert_eq!(foco(), h.inputs[0][0]);
    assert_eq!(
        diagnostics(h.main).expect("diagnóstico").focused_field,
        Some((0, 0)),
        "o modelo sabe qual campo tem foco"
    );
    escrever(&h, 0, 0, "2^");
    ocioso(&h);
    assert_eq!(
        texto(h.helper),
        model::DEFAULT_HELPER,
        "prefixo com foco não acusa erro"
    );
    assert_eq!(
        cartao(&h),
        Card::Waiting(Waiting::Missing {
            missing: 6,
            invalid: 0
        }),
        "prefixo conta como campo em digitação"
    );
    escrever(&h, 0, 1, "1/0");
    assert_eq!(
        texto(h.helper),
        model::DEFAULT_HELPER,
        "o erro só aparece depois da pausa"
    );
    ocioso(&h);
    assert_eq!(
        texto(h.helper),
        "Equação 1, coeficiente de y: O denominador não pode ser zero."
    );
    escrever(&h, 1, 0, "3");
    assert!(
        texto(h.helper).starts_with("Equação 1, coeficiente de y"),
        "editar outro campo mantém o erro"
    );
    escrever(&h, 0, 1, "1/2");
    assert_eq!(
        texto(h.helper),
        model::DEFAULT_HELPER,
        "corrigir o próprio campo apaga o erro"
    );
    focar(&h, h.inputs[1][1]);
    assert_eq!(
        texto(h.helper),
        "Equação 1, coeficiente de x: Use um expoente inteiro de -1000 a 1000, como 2^3 ou 10^-2.",
        "sair do campo revela o erro"
    );
    escrever(&h, 0, 0, "2^3");
    assert_eq!(texto(h.helper), model::DEFAULT_HELPER);
    preencher(&h, &[(0, 3, "8"), (1, 1, "1"), (1, 3, "7")]);
    assert_eq!(cartao(&h), Card::Result { stale: false });
    assert_eq!(
        [texto(h.values[0]), texto(h.values[1])],
        ["9/13", "64/13"],
        "2^3 entra como 8: 8x + y/2 = 8 e 3x + y = 7"
    );
    fechar(&h);
}

#[test]
fn enter_e_setas_navegam_entre_campos() {
    let h = janela();
    assert_eq!(foco(), h.inputs[0][0]);
    tecla(&h, h.inputs[0][0], VK_RETURN);
    assert_eq!(foco(), h.inputs[0][1], "Enter vai ao próximo campo vazio");
    preencher(&h, &EXEMPLO_SEM_ULTIMO);
    focar(&h, h.inputs[0][0]);
    tecla(&h, h.inputs[0][0], VK_RETURN);
    assert_eq!(foco(), h.inputs[1][3], "Enter pula os campos preenchidos");
    assert_eq!(selecao(h.inputs[1][3]), (0, 0));
    tecla(&h, h.inputs[1][3], VK_RETURN);
    assert_eq!(foco(), h.inputs[1][3], "único vazio: o foco fica");
    assert_eq!(texto(h.helper), model::ONLY_THIS_FIELD);
    digitar(&h, h.inputs[1][3], "1");
    assert_eq!(
        texto(h.helper),
        model::DEFAULT_HELPER,
        "digitar tira o aviso"
    );
    assert_eq!(cartao(&h), Card::Result { stale: false });
    tecla(&h, h.inputs[1][3], VK_RETURN);
    assert_eq!(
        foco(),
        h.inputs[1][3],
        "Enter com tudo preenchido resolve e fica"
    );
    assert_eq!(
        diagnostics(h.main)
            .expect("diagnóstico")
            .last_announced
            .as_deref(),
        Some("Solução encontrada: x = 2, y = 1.")
    );

    focar(&h, h.inputs[0][1]);
    com_teclas(&[], || tecla(&h, h.inputs[0][1], VK_DOWN));
    assert_eq!(foco(), h.inputs[1][1], "seta para baixo muda de equação");
    assert_eq!(selecao(h.inputs[1][1]), (0, 2), "texto todo selecionado");
    com_teclas(&[], || tecla(&h, h.inputs[1][1], VK_DOWN));
    assert_eq!(foco(), h.inputs[1][1], "na última equação a seta não sai");
    com_teclas(&[], || tecla(&h, h.inputs[1][1], VK_UP));
    assert_eq!(foco(), h.inputs[0][1]);
    assert_eq!(selecao(h.inputs[0][1]), (0, 1));
    com_teclas(&[VK_SHIFT], || tecla(&h, h.inputs[0][1], VK_DOWN));
    assert_eq!(foco(), h.inputs[0][1], "Shift+seta fica no campo");

    focar(&h, h.inputs[0][0]);
    escrever(&h, 0, 0, "1/0");
    tecla(&h, h.inputs[0][0], VK_RETURN);
    assert_eq!(foco(), h.inputs[0][0], "campo inválido segura o foco");
    assert_eq!(
        texto(h.helper),
        "Equação 1, coeficiente de x: O denominador não pode ser zero."
    );
    assert_eq!(selecao(h.inputs[0][0]), (0, 3));
    fechar(&h);
}

#[test]
fn altgr_digita_potencias_sem_atalhos_de_ctrl() {
    let h = janela();
    escrever(&h, 0, 0, "12 34");
    unsafe {
        SendMessageW(h.inputs[0][0], EM_SETSEL, Some(WPARAM(5)), Some(LPARAM(5)));
    }
    com_teclas(&ALTGR, || tecla(&h, h.inputs[0][0], VK_BACK));
    assert_ne!(
        texto(h.inputs[0][0]),
        "12 ",
        "AltGr+Backspace não apaga a palavra"
    );
    escrever(&h, 0, 0, "12 34");
    unsafe {
        SendMessageW(h.inputs[0][0], EM_SETSEL, Some(WPARAM(1)), Some(LPARAM(1)));
    }
    com_teclas(&ALTGR, || {
        tecla(&h, h.inputs[0][0], VIRTUAL_KEY(u16::from(b'A')))
    });
    let (inicio, fim) = selecao(h.inputs[0][0]);
    assert_eq!(inicio, fim, "AltGr+A não seleciona tudo");
    escrever(&h, 0, 0, "");
    focar(&h, h.inputs[0][0]);
    digitar(&h, h.inputs[0][0], "2\u{b2}");
    assert_eq!(texto(h.inputs[0][0]), "2\u{b2}", "AltGr+2 digita ²");
    preencher(
        &h,
        &[
            (0, 1, "1"),
            (0, 3, "5"),
            (1, 0, "1"),
            (1, 1, "-1"),
            (1, 3, "1"),
        ],
    );
    assert_eq!(
        [texto(h.values[0]), texto(h.values[1])],
        ["6/5", "1/5"],
        "2² vale 4"
    );
    escrever(&h, 1, 3, "10^-1");
    assert_eq!(cartao(&h), Card::Result { stale: false });
    fechar(&h);
}

#[test]
fn links_do_rodape_dicas_e_modo_pensar() {
    let h = janela();
    for (indice, metodo) in Method::ALL.into_iter().enumerate() {
        assert_eq!(texto(h.links[indice]), metodo.label());
        assert_eq!(
            nome_acessivel(h.links[indice]),
            model::method_link_name(metodo)
        );
        assert_eq!(
            descricao_acessivel(h.links[indice]),
            model::method_tip(metodo)
        );
        assert_eq!(
            dica(&h, h.links[indice].0 as usize, true),
            model::method_tip(metodo)
        );
    }
    assert_eq!(dica(&h, 1, false), model::TIP_DISABLED);
    assert_eq!(dica(&h, 2, false), model::TIP_DISABLED);
    assert_eq!(descricao_acessivel(h.think), model::TIP_DISABLED);
    assert_eq!(descricao_acessivel(h.copy), model::TIP_DISABLED);
    assert_eq!(dica(&h, h.clear.0 as usize, true), model::TIP_CLEAR);
    assert_eq!(dica(&h, h.example.0 as usize, true), model::TIP_EXAMPLE);
    assert_eq!(descricao_acessivel(h.clear), model::TIP_CLEAR);
    assert_eq!(nome_acessivel(h.two), "Sistema 2 × 2, selecionado");
    assert_eq!(nome_acessivel(h.three), "Sistema 3 × 3");

    clicar(&h, Button::LinkCramer);
    assert!(!diagnostics(h.main).expect("diagnóstico").thinking);
    assert_eq!(texto(h.helper), model::method_needs_system(Method::Cramer));
    assert_eq!(foco(), h.inputs[0][0], "foco no primeiro campo vazio");

    clicar(&h, Button::Example);
    assert_eq!(descricao_acessivel(h.think), model::TIP_THINK);
    assert_eq!(descricao_acessivel(h.copy), model::TIP_COPY);

    focar(&h, h.links[1]);
    let tip = |h: &Handles| diagnostics(h.main).expect("diagnóstico").footer_tip;
    assert_eq!(tip(&h), Some(Method::Cramer), "foco no link mostra a dica");
    unsafe {
        SendMessageW(
            h.links[2],
            WM_MOUSEMOVE,
            Some(WPARAM(0)),
            Some(LPARAM(5 | (5 << 16))),
        );
    }
    assert_eq!(tip(&h), Some(Method::Gauss), "o mouse vence o foco");
    unsafe {
        SendMessageW(h.links[2], WM_MOUSELEAVE, None, None);
    }
    pump(h.main);
    assert_eq!(tip(&h), Some(Method::Cramer), "volta à dica do link focado");

    tecla(&h, h.links[1], VK_RETURN);
    let estado = diagnostics(h.main).expect("diagnóstico");
    assert!(estado.thinking, "o link abre o Modo pensar");
    assert_eq!(estado.method, Method::Cramer);
    assert_eq!(estado.footer_tip, None);
    assert_eq!(foco(), h.methods[1]);
    assert!(
        h.links.iter().all(|link| !visivel(*link)),
        "links somem no Modo pensar"
    );
    tecla(&h, h.methods[1], VK_ESCAPE);
    assert!(!diagnostics(h.main).expect("diagnóstico").thinking);
    assert_eq!(foco(), h.links[1], "Esc devolve o foco ao link que abriu");

    preencher(&h, &[(0, 0, "10^99"), (0, 1, "10^98"), (0, 3, "10^97")]);
    assert_eq!(cartao(&h), Card::Result { stale: false });
    preencher(&h, &[(0, 0, "1e999"), (0, 1, "1e998"), (0, 3, "1e997")]);
    ocioso(&h);
    assert_eq!(cartao(&h), Card::Waiting(Waiting::Large));
    assert_eq!(texto(h.empty_title), model::LARGE_TITLE);
    clicar(&h, Button::LinkGauss);
    let estado = diagnostics(h.main).expect("diagnóstico");
    assert!(estado.thinking, "sistema grande: o link resolve e abre");
    assert_eq!(estado.method, Method::Gauss);
    clicar(&h, Button::Back);

    escrever(&h, 0, 0, "abc");
    clicar(&h, Button::LinkGaussJordan);
    assert!(!diagnostics(h.main).expect("diagnóstico").thinking);
    assert_eq!(
        texto(h.helper),
        "Equação 1, coeficiente de x: Número inválido.",
        "o link revela o campo inválido"
    );
    assert_eq!(foco(), h.inputs[0][0]);
    fechar(&h);
}

#[test]
fn desenho_de_campos_trilho_rodape_e_links() {
    for gdi in [false, true] {
        let h = if gdi { janela_gdi() } else { janela() };
        let nome = if gdi { "GDI" } else { "Direct2D" };
        let layout = diagnostics(h.main).expect("diagnóstico").layout;
        let imagem = capturar(h.main);
        let trilho = layout.size_track;
        assert!(
            distancia(
                imagem.cor(trilho.x + 2, trilho.y + trilho.h / 2),
                theme::FIELD
            ) < 6,
            "{nome}: trilho do tamanho"
        );
        assert!(
            distancia(imagem.cor(trilho.x, trilho.y + trilho.h / 2), theme::BORDER) < 12,
            "{nome}: borda do trilho"
        );
        let tres = layout.three;
        assert!(
            distancia(imagem.cor(tres.x + 12, tres.y + 6), theme::FIELD) < 6,
            "{nome}: segmento não selecionado transparente"
        );
        let dois = layout.two;
        assert_eq!(
            imagem.cor(dois.x + 12, dois.y + 8),
            theme::ACCENT,
            "{nome}: segmento selecionado lilás"
        );
        let teclas = imagem.conta(area(layout.shortcut), |cor| cor == theme::FIELD);
        assert!(teclas > 300, "{nome}: teclas desenhadas ({teclas})");
        let rotulo = imagem.conta(area(layout.footer_label), |cor| {
            distancia(cor, theme::MUTED) < 60
        });
        assert!(rotulo > 40, "{nome}: rótulo do rodapé ({rotulo})");
        for link in layout.method_links {
            let tinta = imagem.conta(area(link), |cor| distancia(cor, theme::INK) < 60);
            assert!(tinta > 30, "{nome}: link desenhado ({tinta})");
        }

        let numero = layout.row_numbers[0];
        let lilas = |imagem: &Imagem| imagem.conta(area(numero), |cor| cor.b > 205);
        assert_eq!(lilas(&imagem), 0, "{nome}: linha incompleta, número neutro");

        let campo = layout.fields[0][0];
        let meio = campo.y + campo.h / 2;
        focar(&h, h.inputs[1][1]);
        unsafe {
            SendMessageW(
                h.inputs[0][0],
                WM_MOUSEMOVE,
                Some(WPARAM(0)),
                Some(LPARAM(2 | (2 << 16))),
            );
        }
        assert_eq!(
            diagnostics(h.main).expect("diagnóstico").hovered_field,
            Some((0, 0))
        );
        let sobre = capturar(h.main);
        assert!(
            distancia(sobre.cor(campo.x + 1, meio), theme::FIELD_HOVER) < 12,
            "{nome}: borda de hover {:?}",
            sobre.cor(campo.x + 1, meio)
        );
        unsafe {
            SendMessageW(h.inputs[0][0], WM_MOUSELEAVE, None, None);
        }
        pump(h.main);
        assert_eq!(
            diagnostics(h.main).expect("diagnóstico").hovered_field,
            None
        );

        focar(&h, h.inputs[0][0]);
        let focado = capturar(h.main);
        assert!(
            distancia(focado.cor(campo.x + 1, meio), theme::ACCENT) < 12,
            "{nome}: borda de foco {:?}",
            focado.cor(campo.x + 1, meio)
        );
        let halo = focado.cor(campo.x + 2, meio);
        if gdi {
            assert_eq!(halo, theme::FIELD, "GDI sem halo");
        } else {
            assert!(
                distancia(halo, theme::FIELD) > 25 && distancia(halo, theme::ACCENT) > 100,
                "halo interno de foco {halo:?}"
            );
        }

        preencher(&h, &[(0, 0, "1"), (0, 1, "1"), (0, 3, "1")]);
        assert!(
            lilas(&capturar(h.main)) > 3,
            "{nome}: linha completa, número lilás"
        );
        escrever(&h, 1, 0, "1/0");
        ocioso(&h);
        let erro = capturar(h.main);
        let errado = layout.fields[1][0];
        assert!(
            distancia(
                erro.cor(errado.x + 1, errado.y + errado.h / 2),
                theme::ERROR
            ) < 12,
            "{nome}: borda de erro"
        );
        fechar(&h);
    }
}

#[test]
fn tamanhos_guardam_sistemas_separados_e_dica_do_valor_lido() {
    let h = janela();
    assert_eq!(
        descricao_acessivel(h.inputs[0][0]),
        model::FIELD_DESCRIPTION
    );
    clicar(&h, Button::Three);
    clicar(&h, Button::Example);
    assert_eq!(
        [texto(h.values[0]), texto(h.values[1]), texto(h.values[2])],
        ["1", "2", "3"]
    );
    clicar(&h, Button::Two);
    for (linha, coluna) in [(0, 0), (0, 1), (0, 3), (1, 0), (1, 1), (1, 3)] {
        assert_eq!(texto(h.inputs[linha][coluna]), "", "o 2×2 começa vazio");
    }
    assert_eq!(cartao(&h), Card::Waiting(Waiting::Empty));
    clicar(&h, Button::Three);
    assert_eq!(texto(h.inputs[2][3]), "2", "o 3×3 volta como estava");
    assert_eq!(cartao(&h), Card::Result { stale: false });
    focar(&h, h.inputs[0][0]);
    escrever(&h, 0, 0, "2^0");
    assert_eq!(texto(h.helper), "Este campo vale 1.");
    assert_eq!(nome_acessivel(h.helper), "Este campo vale 1.");
    escrever(&h, 0, 0, "1");
    assert_eq!(
        texto(h.helper),
        model::DEFAULT_HELPER,
        "sem potência, a dica volta ao normal"
    );
    fechar(&h);
}
#[test]
fn modo_pensar_com_numeros_enormes_responde() {
    use windows::Win32::UI::WindowsAndMessaging::{
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SetWindowPos,
    };
    let h = janela();
    clicar(&h, Button::Three);
    preencher(
        &h,
        &[
            (0, 0, "1e1000"),
            (0, 1, "-3e999"),
            (0, 2, "7e998"),
            (0, 3, "1"),
            (1, 0, "2"),
            (1, 1, "9e1000"),
            (1, 2, "-1"),
            (1, 3, "5e997"),
            (2, 0, "1/3e1000"),
            (2, 1, "4"),
            (2, 2, "-7e999"),
            (2, 3, "2e1000"),
        ],
    );
    clicar(&h, Button::Solve);
    assert_eq!(cartao(&h), Card::Result { stale: false });
    let inicio = Instant::now();
    clicar(&h, Button::Think);
    let abrir = inicio.elapsed();
    assert!(diagnostics(h.main).expect("diagnóstico").thinking);
    let inicio = Instant::now();
    clicar(&h, Button::MethodCramer);
    clicar(&h, Button::MethodGauss);
    let trocar = inicio.elapsed();
    let inicio = Instant::now();
    for largura in [1100, 1220, 1300, 1260, 1180] {
        unsafe {
            let _ = SetWindowPos(
                h.main,
                None,
                0,
                0,
                largura,
                900,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        pump(h.main);
    }
    let redimensionar = inicio.elapsed();
    eprintln!(
        "abrir {abrir:?} · trocar 2 métodos {trocar:?} · 5 redimensionamentos {redimensionar:?}"
    );
    assert!(
        abrir.as_secs_f64() < 6.0
            && trocar.as_secs_f64() < 12.0
            && redimensionar.as_secs_f64() < 12.0,
        "abrir {abrir:?}, trocar {trocar:?}, redimensionar {redimensionar:?}"
    );
    fechar(&h);
}
