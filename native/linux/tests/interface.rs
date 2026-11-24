use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui_kittest::kittest::{By, NodeT, Queryable};
use egui_kittest::{Harness, Node};
use sistemas_lineares_core::Method;
use sistemas_lineares_linux::app::{App, IDLE_DELAY, KEY_DIMENSION, KEY_METHOD, Launch, Target};
use sistemas_lineares_linux::contrast;
use sistemas_lineares_linux::eframe::egui::{self, Key, Modifiers, accesskit};
use sistemas_lineares_linux::eframe::{self, Storage};
use sistemas_lineares_ui::model::{self, Card, Waiting};

static ABERTURAS: AtomicUsize = AtomicUsize::new(0);

fn abre_sem_navegador(_: &str) -> bool {
    true
}

fn conta_aberturas(url: &str) -> bool {
    assert_eq!(url, model::AUTHOR_URL);
    ABERTURAS.fetch_add(1, Ordering::SeqCst);
    true
}

fn navegador_indisponivel(_: &str) -> bool {
    false
}

fn janela_com(launch: Launch, largura: f32, altura: f32) -> Harness<'static, App> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(largura, altura))
        .with_max_steps(200)
        .build_eframe(move |context| App::new(context, launch));
    harness.input_mut().time = Some(100.0);
    harness.run();
    harness
}

fn janela() -> Harness<'static, App> {
    janela_com(
        Launch {
            high_contrast: false,
            open_url: abre_sem_navegador,
        },
        1160.0,
        810.0,
    )
}

fn agora(harness: &Harness<'_, App>) -> f64 {
    harness.input().time.unwrap_or_default()
}

fn esperar(harness: &mut Harness<'_, App>, segundos: f64) {
    let tempo = agora(harness) + segundos;
    harness.input_mut().time = Some(tempo);
    harness.run();
}

fn pausa(harness: &mut Harness<'_, App>) {
    esperar(harness, IDLE_DELAY + 0.05);
}

fn rotulado(nome: String) -> By<'static> {
    By::new().predicate(move |node| node.label().as_deref() == Some(nome.as_str()))
}

fn com_valor(texto: String) -> By<'static> {
    By::new().predicate(move |node| node.value().as_deref() == Some(texto.as_str()))
}

fn campo<'a>(harness: &'a Harness<'_, App>, linha: usize, coluna: usize) -> Node<'a> {
    harness.get(rotulado(model::accessible_input_name(linha, coluna)))
}

fn existe_campo(harness: &Harness<'_, App>, linha: usize, coluna: usize) -> bool {
    harness
        .query(rotulado(model::accessible_input_name(linha, coluna)))
        .is_some()
}

fn texto_visivel(harness: &Harness<'_, App>, texto: &str) -> bool {
    harness.query(com_valor(texto.to_string())).is_some()
}

fn regioes_vivas(harness: &Harness<'_, App>) -> Vec<String> {
    harness
        .root()
        .children_recursive()
        .map(|node| node.accesskit_node())
        .filter(|node| node.live() != accesskit::Live::Off)
        .filter_map(|node| node.value().or_else(|| node.label()))
        .collect()
}

fn selecionar_tudo(harness: &mut Harness<'_, App>) {
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.run();
}

fn escrever(harness: &mut Harness<'_, App>, texto: &str) {
    for caractere in texto.chars() {
        harness.event(egui::Event::Text(caractere.to_string()));
        harness.run();
    }
}

fn digitar(harness: &mut Harness<'_, App>, linha: usize, coluna: usize, texto: &str) {
    campo(harness, linha, coluna).click();
    harness.run();
    selecionar_tudo(harness);
    escrever(harness, texto);
}

fn tecla(harness: &mut Harness<'_, App>, key: Key) {
    harness.key_press(key);
    harness.run();
}

fn tecla_com(harness: &mut Harness<'_, App>, modifiers: Modifiers, key: Key) {
    harness.key_press_modifiers(modifiers, key);
    harness.run();
}

fn clicar(harness: &mut Harness<'_, App>, rotulo: &str) {
    harness.get_by_label(rotulo).click();
    harness.run();
}

fn apertar(harness: &mut Harness<'_, App>, alvo: Target) {
    let ctx = harness.ctx.clone();
    harness.state_mut().press(&ctx, alvo);
    harness.run();
}

fn foco(harness: &Harness<'_, App>) -> Option<Target> {
    harness.state().focused()
}

fn cartao(harness: &Harness<'_, App>) -> Card {
    harness.state().model().card()
}

fn valores(harness: &Harness<'_, App>) -> Vec<String> {
    harness.state().model().values().to_vec()
}

fn texto_do_campo(harness: &Harness<'_, App>, linha: usize, coluna: usize) -> String {
    harness.state().input_text(linha, coluna).to_string()
}

fn ultimo_anuncio(harness: &Harness<'_, App>) -> Option<String> {
    harness.state().announcements().last().cloned()
}

fn preencher_exemplo_sem_ultimo(harness: &mut Harness<'_, App>) {
    for (linha, coluna, valor) in [
        (0, 0, "2"),
        (0, 1, "1"),
        (0, 3, "5"),
        (1, 0, "1"),
        (1, 1, "-1"),
    ] {
        digitar(harness, linha, coluna, valor);
    }
}

#[test]
fn estado_inicial_com_foco_no_primeiro_campo() {
    let harness = janela();
    assert_eq!(foco(&harness), Some(Target::Field(0, 0)));
    assert!(campo(&harness, 0, 0).is_focused());
    assert_eq!(cartao(&harness), Card::Waiting(Waiting::Empty));
    harness.get_by_label("Sistema 2 × 2, selecionado");
    harness.get_by_label("Sistema 3 × 3");
    for (linha, coluna) in [(0, 0), (0, 1), (0, 3), (1, 0), (1, 1), (1, 3)] {
        let no = campo(&harness, linha, coluna).accesskit_node();
        assert_eq!(no.role(), accesskit::Role::TextInput);
        assert_eq!(no.description().as_deref(), Some(model::FIELD_DESCRIPTION));
    }
    assert!(!existe_campo(&harness, 0, 2), "o 2×2 não mostra a coluna z");
    assert!(texto_visivel(&harness, model::EMPTY_TITLE));
    assert!(texto_visivel(&harness, model::DEFAULT_HELPER));
    let pensar = harness.get_by_label(model::THINK_LABEL).accesskit_node();
    assert!(pensar.is_disabled());
    assert_eq!(pensar.description().as_deref(), Some(model::TIP_DISABLED));
    let copiar = harness.get_by_label(model::COPY_LABEL).accesskit_node();
    assert!(copiar.is_disabled());
    assert_eq!(copiar.description().as_deref(), Some(model::TIP_DISABLED));
    assert_eq!(
        harness
            .get_by_label(model::CLEAR_LABEL)
            .accesskit_node()
            .description()
            .as_deref(),
        Some(model::TIP_CLEAR)
    );
    assert_eq!(
        harness
            .get_by_label(model::EXAMPLE_LABEL)
            .accesskit_node()
            .description()
            .as_deref(),
        Some(model::TIP_EXAMPLE)
    );
    let autor = harness.get_by_label(model::AUTHOR_LABEL).accesskit_node();
    assert_eq!(autor.role(), accesskit::Role::Link);
    assert_eq!(
        autor.description().as_deref(),
        Some(model::AUTHOR_DESCRIPTION)
    );
    for metodo in Method::ALL {
        let link = harness
            .get_by_label(model::method_link_name(metodo))
            .accesskit_node();
        assert_eq!(link.role(), accesskit::Role::Link);
        assert_eq!(
            link.description().as_deref(),
            Some(model::method_tip(metodo))
        );
    }
    harness.check_accessibility();
}

#[test]
fn digitar_o_exemplo_resolve_ao_vivo() {
    let mut harness = janela();
    preencher_exemplo_sem_ultimo(&mut harness);
    assert_eq!(
        cartao(&harness),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        })
    );
    assert!(texto_visivel(&harness, "Falta 1 campo"));
    let titulo = harness
        .get(com_valor("Falta 1 campo".to_string()))
        .accesskit_node();
    assert_eq!(
        titulo.description().as_deref(),
        Some("Faltam: equação 2, termo independente."),
        "a prévia é lida pela descrição"
    );
    assert_eq!(
        harness.state().model().preview(),
        ["2x + y = 5", "x \u{2212} y = ?"]
    );
    digitar(&mut harness, 1, 3, "1");
    assert_eq!(cartao(&harness), Card::Result { stale: false });
    assert_eq!(valores(&harness), ["2", "1"]);
    assert!(texto_visivel(&harness, "Valor de x: 2"));
    assert!(texto_visivel(&harness, "Valor de y: 1"));
    assert!(
        !harness
            .get_by_label(model::THINK_LABEL)
            .accesskit_node()
            .is_disabled()
    );
    assert_eq!(
        harness
            .get_by_label(model::THINK_LABEL)
            .accesskit_node()
            .description()
            .as_deref(),
        Some(model::TIP_THINK)
    );
    assert_eq!(
        harness
            .get_by_label(model::COPY_LABEL)
            .accesskit_node()
            .description()
            .as_deref(),
        Some(model::TIP_COPY)
    );
    assert_ne!(
        ultimo_anuncio(&harness).as_deref(),
        Some("Solução encontrada: x = 2, y = 1."),
        "digitação só anuncia depois da pausa"
    );
    pausa(&mut harness);
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some("Solução encontrada: x = 2, y = 1.")
    );
    assert!(
        regioes_vivas(&harness).contains(&"Solução encontrada: x = 2, y = 1.".to_string()),
        "o título do resultado é região viva"
    );
    let anuncios = harness.state().announcements().len();
    pausa(&mut harness);
    assert_eq!(
        harness.state().announcements().len(),
        anuncios,
        "o mesmo texto não é anunciado de novo"
    );
}

#[test]
fn esmaece_no_menos_e_atualiza_no_menos_tres() {
    let mut harness = janela();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    assert_eq!(cartao(&harness), Card::Result { stale: false });
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some(
            "Exemplo resolvido. Mude qualquer valor e a resposta se atualiza na hora. Solução encontrada: x = 2, y = 1."
        )
    );
    assert_eq!(
        foco(&harness),
        Some(Target::Example),
        "o foco não vai para um campo"
    );
    digitar(&mut harness, 1, 3, "-");
    assert_eq!(cartao(&harness), Card::Result { stale: true });
    assert_eq!(harness.state().model().note(), model::NOTE_STALE);
    assert!(
        harness
            .get_by_label(model::COPY_LABEL)
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        harness
            .get_by_label(model::THINK_LABEL)
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !harness.state().model().helper_is_error(),
        "prefixo não é erro"
    );
    escrever(&mut harness, "3");
    assert_eq!(cartao(&harness), Card::Result { stale: false });
    assert_eq!(valores(&harness), ["2/3", "11/3"]);
    escrever(&mut harness, "/");
    assert_eq!(cartao(&harness), Card::Result { stale: true });
    pausa(&mut harness);
    assert_eq!(
        cartao(&harness),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        }),
        "a pausa assenta o card; prefixo com foco conta como vazio"
    );
    assert!(!harness.state().model().helper_is_error());
}

#[test]
fn erro_aparece_na_pausa_e_ao_sair_do_campo() {
    let mut harness = janela();
    digitar(&mut harness, 0, 0, "2x");
    assert!(!harness.state().model().helper_is_error(), "o erro espera");
    assert!(harness.state().idle_pending());
    pausa(&mut harness);
    assert!(harness.state().model().field_has_error(0, 0));
    let mensagem = "Equação 1, coeficiente de x: Número inválido.";
    assert_eq!(harness.state().model().helper(), mensagem);
    assert!(regioes_vivas(&harness).contains(&mensagem.to_string()));
    assert!(
        harness
            .state()
            .announcements()
            .contains(&mensagem.to_string())
    );
    let no = campo(&harness, 0, 0).accesskit_node();
    assert_eq!(no.data().invalid(), Some(accesskit::Invalid::True));
    assert_eq!(no.description().as_deref(), Some("Número inválido."));
    digitar(&mut harness, 0, 1, "1/");
    assert!(
        harness.state().model().field_has_error(0, 0),
        "editar outro campo não apaga o erro"
    );
    pausa(&mut harness);
    assert!(
        !harness.state().model().field_has_error(0, 1),
        "prefixo com foco não é revelado"
    );
    tecla(&mut harness, Key::Tab);
    assert_eq!(foco(&harness), Some(Target::Field(0, 3)));
    assert!(
        harness.state().model().field_has_error(0, 1),
        "sair do campo revela o erro"
    );
    digitar(&mut harness, 0, 0, "2");
    assert!(!harness.state().model().field_has_error(0, 0));
    assert_eq!(
        harness.state().model().helper(),
        "Equação 1, coeficiente de y: Número inválido."
    );
}

#[test]
fn enter_vai_ao_proximo_vazio_e_resolve() {
    let mut harness = janela();
    tecla(&mut harness, Key::Enter);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(0, 1)),
        "Enter vai ao próximo vazio"
    );
    preencher_exemplo_sem_ultimo(&mut harness);
    campo(&harness, 0, 0).click();
    harness.run();
    tecla(&mut harness, Key::Enter);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(1, 3)),
        "pula os preenchidos"
    );
    tecla(&mut harness, Key::Enter);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(1, 3)),
        "único vazio: fica"
    );
    assert_eq!(harness.state().model().helper(), model::ONLY_THIS_FIELD);
    assert!(regioes_vivas(&harness).contains(&model::ONLY_THIS_FIELD.to_string()));
    escrever(&mut harness, "1");
    assert_eq!(cartao(&harness), Card::Result { stale: false });
    let antes = harness.state().announcements().len();
    tecla(&mut harness, Key::Enter);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(1, 3)),
        "com tudo preenchido resolve e fica"
    );
    assert_eq!(harness.state().announcements().len(), antes + 1);
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some("Solução encontrada: x = 2, y = 1.")
    );
    digitar(&mut harness, 0, 0, "1/0");
    tecla(&mut harness, Key::Enter);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(0, 0)),
        "inválido segura o foco"
    );
    assert_eq!(
        harness.state().model().helper(),
        "Equação 1, coeficiente de x: O denominador não pode ser zero."
    );
}

#[test]
fn tab_percorre_campos_e_botoes() {
    let mut harness = janela();
    apertar(&mut harness, Target::Example);
    apertar(&mut harness, Target::Field(1, 3));
    assert_eq!(foco(&harness), Some(Target::Field(1, 3)));
    tecla(&mut harness, Key::Tab);
    assert_eq!(foco(&harness), Some(Target::Solve));
    for esperado in [
        Target::Example,
        Target::Clear,
        Target::Think,
        Target::Copy,
        Target::Link(Method::GaussJordan),
        Target::Link(Method::Cramer),
        Target::Link(Method::Gauss),
        Target::Author,
    ] {
        tecla(&mut harness, Key::Tab);
        assert_eq!(foco(&harness), Some(esperado));
    }
    assert!(harness.state().focus_visible());
    tecla_com(&mut harness, Modifiers::SHIFT, Key::Tab);
    assert_eq!(foco(&harness), Some(Target::Link(Method::Gauss)));
    assert_eq!(harness.state().footer_tip(), Some(Method::Gauss));
}

#[test]
fn enter_seleciona_o_texto_do_destino() {
    let mut harness = janela();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    campo(&harness, 0, 1).click();
    harness.run();
    selecionar_tudo(&mut harness);
    tecla(&mut harness, Key::Backspace);
    assert_eq!(texto_do_campo(&harness, 0, 1), "");
    campo(&harness, 0, 0).click();
    harness.run();
    tecla(&mut harness, Key::Enter);
    assert_eq!(foco(&harness), Some(Target::Field(0, 1)));
    escrever(&mut harness, "7");
    assert_eq!(texto_do_campo(&harness, 0, 1), "7");
    tecla(&mut harness, Key::ArrowDown);
    assert_eq!(foco(&harness), Some(Target::Field(1, 1)));
    escrever(&mut harness, "9");
    assert_eq!(
        texto_do_campo(&harness, 1, 1),
        "9",
        "seta seleciona todo o texto"
    );
}

#[test]
fn setas_mudam_de_equacao_na_mesma_coluna() {
    let mut harness = janela();
    apertar(&mut harness, Target::Three);
    campo(&harness, 0, 1).click();
    harness.run();
    tecla(&mut harness, Key::ArrowDown);
    assert_eq!(foco(&harness), Some(Target::Field(1, 1)));
    tecla(&mut harness, Key::ArrowDown);
    assert_eq!(foco(&harness), Some(Target::Field(2, 1)));
    tecla(&mut harness, Key::ArrowDown);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(2, 1)),
        "na última equação fica"
    );
    tecla(&mut harness, Key::ArrowUp);
    assert_eq!(foco(&harness), Some(Target::Field(1, 1)));
    tecla_com(&mut harness, Modifiers::SHIFT, Key::ArrowUp);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(1, 1)),
        "Shift+seta segue o normal"
    );
    campo(&harness, 0, 3).click();
    harness.run();
    tecla(&mut harness, Key::ArrowUp);
    assert_eq!(
        foco(&harness),
        Some(Target::Field(0, 3)),
        "na primeira equação fica"
    );
    tecla(&mut harness, Key::ArrowDown);
    assert_eq!(foco(&harness), Some(Target::Field(1, 3)));
}

#[test]
fn altgr_digita_potencias_sem_atalhos_de_ctrl() {
    let mut harness = janela();
    escrever(&mut harness, "12 34");
    let altgr = Modifiers {
        alt: true,
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    tecla_com(&mut harness, altgr, Key::A);
    escrever(&mut harness, "\u{e1}");
    assert_eq!(
        texto_do_campo(&harness, 0, 0),
        "12 34\u{e1}",
        "AltGr+A não seleciona tudo"
    );
    tecla_com(&mut harness, altgr, Key::Backspace);
    assert_eq!(
        texto_do_campo(&harness, 0, 0),
        "12 34",
        "AltGr+Backspace apaga só um caractere"
    );
    let ctrl = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    tecla_com(&mut harness, ctrl, Key::Backspace);
    assert_eq!(
        texto_do_campo(&harness, 0, 0),
        "12 ",
        "Ctrl+Backspace apaga a palavra"
    );
    selecionar_tudo(&mut harness);
    escrever(&mut harness, "2\u{b2}");
    assert_eq!(texto_do_campo(&harness, 0, 0), "2\u{b2}");
    for (linha, coluna, valor) in [
        (0, 1, "1"),
        (0, 3, "5"),
        (1, 0, "1"),
        (1, 1, "-1"),
        (1, 3, "1"),
    ] {
        digitar(&mut harness, linha, coluna, valor);
    }
    assert_eq!(valores(&harness), ["6/5", "1/5"], "2² vale 4");
    digitar(&mut harness, 0, 0, "2^3");
    assert_eq!(harness.state().model().helper(), "Este campo vale 8.");
    assert!(harness.state().model().helper_is_reading());
}

#[test]
fn links_do_rodape() {
    let mut harness = janela();
    clicar(&mut harness, model::method_link_name(Method::Cramer));
    assert!(!harness.state().model().is_thinking());
    assert_eq!(
        harness.state().model().helper(),
        model::method_needs_system(Method::Cramer)
    );
    assert!(!harness.state().model().helper_is_error(), "dica neutra");
    assert_eq!(
        foco(&harness),
        Some(Target::Field(0, 0)),
        "foco no primeiro vazio"
    );
    clicar(&mut harness, model::EXAMPLE_LABEL);
    harness
        .get_by_label(model::method_link_name(Method::Cramer))
        .hover();
    harness.run();
    assert_eq!(
        harness.state().footer_tip(),
        Some(Method::Cramer),
        "hover mostra a dica"
    );
    harness
        .get_by_label(model::method_link_name(Method::Gauss))
        .hover();
    harness.run();
    assert_eq!(harness.state().footer_tip(), Some(Method::Gauss));
    harness
        .get_by_label(model::method_link_name(Method::Cramer))
        .focus();
    harness.run();
    assert_eq!(
        harness.state().footer_tip(),
        Some(Method::Gauss),
        "o mouse vence o foco"
    );
    harness.event(egui::Event::PointerGone);
    harness.run();
    assert_eq!(
        harness.state().footer_tip(),
        Some(Method::Cramer),
        "volta ao link focado"
    );
    tecla(&mut harness, Key::Enter);
    assert!(
        harness.state().model().is_thinking(),
        "o link abre o Modo pensar"
    );
    assert_eq!(harness.state().model().method(), Method::Cramer);
    assert_eq!(foco(&harness), Some(Target::Tab(Method::Cramer)));
    assert_eq!(harness.state().footer_tip(), None);
    assert!(
        harness
            .query_by_label(model::method_link_name(Method::Cramer))
            .is_none(),
        "os links somem no Modo pensar"
    );
    tecla(&mut harness, Key::Escape);
    assert!(!harness.state().model().is_thinking());
    assert_eq!(
        foco(&harness),
        Some(Target::Link(Method::Cramer)),
        "Esc devolve o foco ao link"
    );
    digitar(&mut harness, 0, 0, "abc");
    clicar(&mut harness, model::method_link_name(Method::GaussJordan));
    assert!(!harness.state().model().is_thinking());
    assert_eq!(
        harness.state().model().helper(),
        "Equação 1, coeficiente de x: Número inválido.",
        "o link revela o campo inválido"
    );
    assert_eq!(foco(&harness), Some(Target::Field(0, 0)));
    for (linha, coluna, valor) in [
        (0, 0, "1e999"),
        (0, 1, "1e998"),
        (0, 3, "1e997"),
        (1, 3, "1e996"),
    ] {
        digitar(&mut harness, linha, coluna, valor);
    }
    pausa(&mut harness);
    assert_eq!(cartao(&harness), Card::Waiting(Waiting::Large));
    assert!(texto_visivel(&harness, model::LARGE_TITLE));
    clicar(&mut harness, model::method_link_name(Method::Gauss));
    assert!(
        harness.state().model().is_thinking(),
        "sistema grande: o link resolve e abre"
    );
    assert_eq!(harness.state().model().method(), Method::Gauss);
    assert!(
        harness
            .state()
            .announcements()
            .iter()
            .any(|texto| texto.starts_with(model::TITLE_UNIQUE) && texto.contains("algarismos")),
        "o link que resolve anuncia na hora"
    );
}

#[test]
fn modo_pensar_troca_metodo_e_volta() {
    let mut harness = janela();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    clicar(&mut harness, model::THINK_LABEL);
    assert!(harness.state().model().is_thinking());
    assert_eq!(foco(&harness), Some(Target::Tab(Method::GaussJordan)));
    let status = "Gauss-Jordan · 7 passos";
    assert!(regioes_vivas(&harness).contains(&status.to_string()));
    let abas = harness
        .get_all_by_role(accesskit::Role::Tab)
        .collect::<Vec<_>>();
    assert_eq!(abas.len(), 3);
    for (aba, metodo) in abas.iter().zip(Method::ALL) {
        assert_eq!(
            aba.accesskit_node().label().as_deref(),
            Some(metodo.label())
        );
        assert_eq!(
            aba.accesskit_node().is_selected(),
            Some(metodo == Method::GaussJordan)
        );
    }
    harness.get_by_role(accesskit::Role::TabList);
    assert!(texto_visivel(
        &harness,
        &format!("{}\n2x + y = 5\nx \u{2212} y = 1", model::SYSTEM_LABEL)
    ));
    let passos = harness
        .root()
        .children_recursive()
        .filter(|node| {
            node.accesskit_node()
                .value()
                .is_some_and(|texto| texto.starts_with("Passo "))
        })
        .count();
    assert_eq!(passos, 7);
    harness.check_accessibility();
    clicar(&mut harness, "Cramer");
    assert_eq!(harness.state().model().method(), Method::Cramer);
    assert!(regioes_vivas(&harness).contains(&"Regra de Cramer · 6 passos".to_string()));
    assert_eq!(
        harness
            .get_by_label("Cramer")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
    tecla(&mut harness, Key::Escape);
    assert!(!harness.state().model().is_thinking());
    assert_eq!(
        foco(&harness),
        Some(Target::Think),
        "Esc devolve o foco ao Modo pensar"
    );
    assert_eq!(
        harness.state().model().method(),
        Method::Cramer,
        "método lembrado"
    );
    clicar(&mut harness, model::THINK_LABEL);
    assert_eq!(foco(&harness), Some(Target::Tab(Method::Cramer)));
    clicar(&mut harness, model::BACK_LABEL);
    assert!(!harness.state().model().is_thinking());
    assert_eq!(
        foco(&harness),
        Some(Target::Think),
        "Voltar também devolve o foco"
    );
}

#[test]
fn copiar_resultado_e_passos() {
    let mut harness = janela();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    harness.get_by_label(model::COPY_LABEL).click();
    harness.step();
    let copiado =
        harness
            .output()
            .platform_output
            .commands
            .iter()
            .find_map(|comando| match comando {
                egui::OutputCommand::CopyText(texto) => Some(texto.clone()),
                _ => None,
            });
    assert_eq!(copiado.as_deref(), Some("Solução única\n\nx = 2\ny = 1"));
    harness.run();
    assert_eq!(harness.state().model().copy_label(), model::COPIED_LABEL);
    harness.get_by_label(model::COPIED_LABEL);
    clicar(&mut harness, model::THINK_LABEL);
    harness.get_by_label(model::COPY_STEPS_LABEL).click();
    harness.step();
    let passos = harness
        .output()
        .platform_output
        .commands
        .iter()
        .find_map(|comando| match comando {
            egui::OutputCommand::CopyText(texto) => Some(texto.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(passos.starts_with("Modo pensar · Gauss-Jordan\n\nSistema\n2x + y = 5\n"));
    assert!(!passos.contains('\r'));
    harness.run();
    harness.get_by_label(model::COPIED_STEPS_LABEL);
}

#[test]
fn tamanhos_guardam_sistemas_separados() {
    let mut harness = janela();
    clicar(&mut harness, "Sistema 3 × 3");
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some("3 equações · 3 incógnitas. Aguardando os valores."),
    );
    digitar(&mut harness, 0, 0, "1");
    clicar(&mut harness, "Sistema 2 × 2");
    clicar(&mut harness, "Sistema 3 × 3");
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some("3 equações · 3 incógnitas. Faltam 11 campos."),
        "a troca anuncia na hora, com o progresso"
    );
    harness.get_by_label("Sistema 3 × 3, selecionado");
    assert!(existe_campo(&harness, 2, 2));
    clicar(&mut harness, model::EXAMPLE_LABEL);
    assert_eq!(valores(&harness), ["1", "2", "3"]);
    clicar(&mut harness, "Sistema 2 × 2");
    assert_eq!(
        cartao(&harness),
        Card::Waiting(Waiting::Empty),
        "o 2×2 começa vazio"
    );
    assert_eq!(texto_do_campo(&harness, 0, 0), "");
    assert!(!existe_campo(&harness, 2, 2));
    clicar(&mut harness, "Sistema 3 × 3");
    assert_eq!(
        texto_do_campo(&harness, 2, 3),
        "2",
        "o 3×3 volta como estava"
    );
    assert_eq!(cartao(&harness), Card::Result { stale: false });
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some("3 equações · 3 incógnitas. Solução encontrada: x = 1, y = 2, z = 3.")
    );
}

#[test]
fn limpar_volta_ao_primeiro_campo() {
    let mut harness = janela();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    clicar(&mut harness, model::CLEAR_LABEL);
    assert_eq!(foco(&harness), Some(Target::Field(0, 0)));
    assert_eq!(cartao(&harness), Card::Waiting(Waiting::Empty));
    for (linha, coluna) in [(0, 0), (0, 1), (0, 3), (1, 0), (1, 1), (1, 3)] {
        assert_eq!(texto_do_campo(&harness, linha, coluna), "");
    }
    assert!(
        !harness.state().focus_visible(),
        "clique não mostra anel de teclado"
    );
    tecla(&mut harness, Key::Tab);
    assert!(harness.state().focus_visible(), "Tab mostra o anel de foco");
}

#[test]
fn resolver_revela_e_foca_o_primeiro_problema() {
    let mut harness = janela();
    clicar(&mut harness, model::SOLVE_LABEL);
    assert_eq!(foco(&harness), Some(Target::Field(0, 0)));
    assert_eq!(
        harness.state().model().helper(),
        "Equação 1, coeficiente de x: Preencha todos os campos."
    );
    assert!(harness.state().model().field_has_error(0, 0));
    preencher_exemplo_sem_ultimo(&mut harness);
    digitar(&mut harness, 1, 3, "1");
    let antes = harness.state().announcements().len();
    clicar(&mut harness, model::SOLVE_LABEL);
    assert_eq!(harness.state().announcements().len(), antes + 1);
    assert_eq!(
        ultimo_anuncio(&harness).as_deref(),
        Some("Solução encontrada: x = 2, y = 1."),
        "Resolver anuncia na hora, mesmo repetido"
    );
}

#[test]
fn autor_abre_o_site_ou_avisa() {
    let mut harness = janela_com(
        Launch {
            high_contrast: false,
            open_url: conta_aberturas,
        },
        1160.0,
        810.0,
    );
    clicar(&mut harness, model::AUTHOR_LABEL);
    assert_eq!(ABERTURAS.load(Ordering::SeqCst), 1);
    assert_eq!(harness.state().model().helper(), model::DEFAULT_HELPER);
    let mut sem_navegador = janela_com(
        Launch {
            high_contrast: false,
            open_url: navegador_indisponivel,
        },
        1160.0,
        810.0,
    );
    clicar(&mut sem_navegador, model::AUTHOR_LABEL);
    assert_eq!(sem_navegador.state().model().helper(), model::AUTHOR_HELPER);
    assert!(regioes_vivas(&sem_navegador).contains(&model::AUTHOR_HELPER.to_string()));
}

#[test]
fn janela_minima_rola_nos_dois_eixos() {
    let mut harness = janela_com(
        Launch {
            high_contrast: false,
            open_url: abre_sem_navegador,
        },
        640.0,
        480.0,
    );
    assert_eq!(harness.state().content_size(), (1040, 760));
    harness.get_by_label(model::EXAMPLE_LABEL).scroll_to_me();
    harness.run();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    assert_eq!(cartao(&harness), Card::Result { stale: false });
    harness.get_by_label(model::SOLVE_LABEL).scroll_to_me();
    harness.run();
    let fora = harness.get_by_label(model::THINK_LABEL).rect();
    assert!(fora.min.x > 640.0, "o botão começa fora da janela");
    harness.get_by_label(model::THINK_LABEL).scroll_to_me();
    harness.run();
    let visivel = harness.get_by_label(model::THINK_LABEL).rect();
    assert!(
        visivel.max.x <= 640.0 + 0.5,
        "a rolagem horizontal mostra o botão: {visivel:?}"
    );
    assert!(
        visivel.max.y <= 480.0 + 0.5,
        "a rolagem vertical mostra o botão: {visivel:?}"
    );
    clicar(&mut harness, model::THINK_LABEL);
    assert!(
        harness.state().model().is_thinking(),
        "{visivel:?} {:?}",
        harness.state().focused()
    );
    let (largura, altura) = harness.state().content_size();
    assert_eq!(largura, 1040);
    assert!(altura > 1500, "{altura}");
    harness.check_accessibility();
}

#[test]
fn alto_contraste_usa_a_paleta_do_sistema() {
    let harness = janela_com(
        Launch {
            high_contrast: true,
            open_url: abre_sem_navegador,
        },
        1160.0,
        810.0,
    );
    assert_eq!(*harness.state().palette(), contrast::palette(true));
    assert!(harness.state().palette().high_contrast);
    assert!(harness.state().palette().focus_glow.is_none());
}

#[derive(Default)]
struct Memoria(HashMap<String, String>);

impl Storage for Memoria {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.to_string(), value);
    }

    fn remove_string(&mut self, key: &str) {
        self.0.remove(key);
    }

    fn flush(&mut self) {}
}

#[test]
fn preferencias_guardam_metodo_e_tamanho() {
    let mut harness = janela();
    clicar(&mut harness, "Sistema 3 × 3");
    clicar(&mut harness, model::EXAMPLE_LABEL);
    clicar(&mut harness, model::THINK_LABEL);
    clicar(&mut harness, "Escalonamento");
    let mut memoria = Memoria::default();
    eframe::App::save(harness.state_mut(), &mut memoria);
    assert_eq!(memoria.get_string(KEY_METHOD).as_deref(), Some("gauss"));
    assert_eq!(memoria.get_string(KEY_DIMENSION).as_deref(), Some("3"));
    let mut novo = App::with_launch(Launch::default());
    novo.restore(&memoria);
    assert_eq!(novo.model().dimension(), 3);
    assert_eq!(novo.model().method(), Method::Gauss);
    let mut estragado = Memoria::default();
    estragado.set_string(KEY_METHOD, "nenhum".to_string());
    estragado.set_string(KEY_DIMENSION, "x".to_string());
    let mut padrao = App::with_launch(Launch::default());
    padrao.restore(&estragado);
    assert_eq!(padrao.model().dimension(), 2);
    assert_eq!(padrao.model().method(), Method::GaussJordan);
}
