use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use egui_kittest::kittest::{By, NodeT, Queryable};
use egui_kittest::{Harness, Node};
use sistemas_lineares_core::Method;
use sistemas_lineares_linux::app::{
    self, App, IDLE_DELAY, KEY_DIMENSION, KEY_METHOD, Launch, Target,
};
use sistemas_lineares_linux::eframe::egui::epaint::{ColorMode, RectShape};
use sistemas_lineares_linux::eframe::egui::{self, Color32, Key, Modifiers, accesskit};
use sistemas_lineares_linux::eframe::{self, Storage};
use sistemas_lineares_linux::paint::{color, tint};
use sistemas_lineares_linux::{contrast, fonts};
use sistemas_lineares_ui::graph;
use sistemas_lineares_ui::layout;
use sistemas_lineares_ui::model::{self, Card, Waiting};
use sistemas_lineares_ui::text::margins_for;
use sistemas_lineares_ui::theme::{self, Font};

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
            animations: false,
            open_url: abre_sem_navegador,
        },
        1160.0,
        810.0,
    )
}

fn janela_animada() -> Harness<'static, App> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1160.0, 810.0))
        .with_max_steps(400)
        .with_step_dt(1.0 / 60.0)
        .build_eframe(|context| {
            App::new(
                context,
                Launch {
                    high_contrast: false,
                    animations: true,
                    open_url: abre_sem_navegador,
                },
            )
        });
    harness.run();
    harness
}

fn agora(harness: &Harness<'_, App>) -> f64 {
    harness.input().time.unwrap_or_default()
}

fn relogio(harness: &Harness<'_, App>) -> f64 {
    harness.ctx.input(|input| input.time)
}

fn repinta_ja(harness: &Harness<'_, App>) -> bool {
    harness
        .output()
        .viewport_output
        .get(&egui::ViewportId::ROOT)
        .is_some_and(|viewport| viewport.repaint_delay.is_zero())
}

fn grafico<'a>(harness: &'a Harness<'_, App>) -> Node<'a> {
    harness.get_by_label(graph::GRAPH_NAME)
}

fn descricao_do_grafico(harness: &Harness<'_, App>) -> Option<String> {
    grafico(harness).accesskit_node().description()
}

fn formas_do_grafico(harness: &Harness<'_, App>) -> Vec<egui::Shape> {
    let area = grafico(harness).rect();
    harness
        .output()
        .shapes
        .iter()
        .filter(|forma| {
            (forma.clip_rect.min - area.min).length() < 0.01
                && (forma.clip_rect.max - area.max).length() < 0.01
        })
        .map(|forma| forma.shape.clone())
        .collect()
}

fn cores(forma: &egui::Shape) -> Vec<Color32> {
    let cores = match forma {
        egui::Shape::Circle(circulo) => vec![circulo.fill, circulo.stroke.color],
        egui::Shape::Path(caminho) => {
            let traco = match caminho.stroke.color {
                ColorMode::Solid(cor) => cor,
                ColorMode::UV(_) => Color32::TRANSPARENT,
            };
            vec![caminho.fill, traco]
        }
        egui::Shape::LineSegment { stroke, .. } => vec![stroke.color],
        egui::Shape::Text(texto) => vec![texto.fallback_color],
        _ => Vec::new(),
    };
    cores
        .into_iter()
        .filter(|cor| *cor != Color32::TRANSPARENT)
        .collect()
}

fn preenchimentos(formas: &[egui::Shape]) -> Vec<Color32> {
    formas
        .iter()
        .filter_map(|forma| match forma {
            egui::Shape::Path(caminho) if caminho.closed => Some(caminho.fill),
            _ => None,
        })
        .collect()
}

fn textos(formas: &[egui::Shape]) -> Vec<String> {
    formas
        .iter()
        .filter_map(|forma| match forma {
            egui::Shape::Text(texto) => Some(texto.galley.text().to_string()),
            _ => None,
        })
        .collect()
}

fn cor_do_numero(harness: &Harness<'_, App>, numero: &str) -> Option<Color32> {
    harness
        .output()
        .shapes
        .iter()
        .find_map(|forma| match &forma.shape {
            egui::Shape::Text(texto) if texto.galley.text() == numero => Some(texto.fallback_color),
            _ => None,
        })
}

fn layout_padrao(dimensao: usize) -> layout::Layout {
    layout::compute(1.0, 1160, 810, dimensao, app::metrics())
}

fn na_tela(harness: &Harness<'_, App>, area: layout::Rect) -> egui::Rect {
    let plot = layout_padrao(harness.state().model().dimension()).plot;
    let origem = grafico(harness).rect().min - egui::vec2(plot.x as f32, plot.y as f32);
    egui::Rect::from_min_size(
        origem + egui::vec2(area.x as f32, area.y as f32),
        egui::vec2(area.w as f32, area.h as f32),
    )
}

fn mesmo_retangulo(primeiro: egui::Rect, segundo: egui::Rect) -> bool {
    (primeiro.min - segundo.min).length() < 0.01 && (primeiro.max - segundo.max).length() < 0.01
}

struct LinhaDaDica {
    topo: f32,
    esquerda: f32,
    texto: String,
    cor: Color32,
}

fn linhas_da_dica(harness: &Harness<'_, App>) -> Vec<LinhaDaDica> {
    let dimensao = harness.state().model().dimension();
    let area = na_tela(harness, layout_padrao(dimensao).helper);
    let deslocamento =
        fonts::metrics(Font::Small).ascent as f32 - fonts::baseline(&harness.ctx, Font::Small);
    let mut linhas: Vec<LinhaDaDica> = harness
        .output()
        .shapes
        .iter()
        .filter(|forma| mesmo_retangulo(forma.clip_rect, area))
        .filter_map(|forma| match &forma.shape {
            egui::Shape::Text(texto) => Some(LinhaDaDica {
                topo: texto.pos.y - deslocamento - area.min.y,
                esquerda: texto.pos.x - area.min.x,
                texto: texto.galley.text().replace('\u{a0}', " "),
                cor: texto.fallback_color,
            }),
            _ => None,
        })
        .collect();
    linhas.sort_by(|primeira, segunda| primeira.topo.total_cmp(&segunda.topo));
    linhas
}

fn conferir_dica(harness: &Harness<'_, App>, esperado: &str, cor: Color32) {
    let dimensao = harness.state().model().dimension();
    let altura = layout_padrao(dimensao).helper.h as f32;
    let metrica = fonts::metrics(Font::Small);
    let margem = margins_for(metrica).0 as f32;
    let linhas = linhas_da_dica(harness);
    assert!(!linhas.is_empty(), "a dica aparece: {esperado}");
    let textos: Vec<&str> = linhas.iter().map(|linha| linha.texto.as_str()).collect();
    assert_eq!(textos.join(" "), esperado);
    for linha in &linhas {
        assert_eq!(linha.cor, cor, "cor de \"{}\"", linha.texto);
        assert!(
            (linha.esquerda - margem).abs() < 0.51,
            "alinhada à esquerda: \"{}\" começa em {}",
            linha.texto,
            linha.esquerda
        );
    }
    for par in linhas.windows(2) {
        assert!(
            (par[1].topo - par[0].topo - metrica.height as f32).abs() < 0.51,
            "linhas seguidas"
        );
    }
    let acima = linhas[0].topo;
    let abaixo = altura - (linhas[linhas.len() - 1].topo + metrica.height as f32);
    assert!(
        acima > 0.0 && (acima - abaixo).abs() <= 2.0,
        "centralizada na vertical: {acima} acima e {abaixo} abaixo de \"{esperado}\""
    );
}

fn esboco_em_dia(harness: &Harness<'_, App>) {
    assert_eq!(
        *harness.state().graph_sketch(),
        harness.state().model().sketch(),
        "o esboço guardado acompanha o modelo"
    );
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
    assert_eq!(
        harness.state().model().helper(),
        graph::INTRO_TWO,
        "a dica vira a legenda do gráfico"
    );
    assert!(harness.state().model().helper_is_caption());
    assert!(texto_visivel(&harness, graph::INTRO_TWO));
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
    assert_eq!(foco(&harness), Some(Target::Example));
    for esperado in [
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
fn enter_com_tudo_preenchido_anuncia_mesmo_repetido() {
    let mut harness = janela();
    assert!(
        harness.query_by_label("Resolver sistema").is_none(),
        "sem botão Resolver: a resposta sai sozinha"
    );
    preencher_exemplo_sem_ultimo(&mut harness);
    digitar(&mut harness, 1, 3, "1");
    apertar(&mut harness, Target::Field(1, 3));
    for _ in 0..2 {
        let antes = harness.state().announcements().len();
        tecla(&mut harness, Key::Enter);
        assert_eq!(harness.state().announcements().len(), antes + 1);
        assert_eq!(
            ultimo_anuncio(&harness).as_deref(),
            Some("Solução encontrada: x = 2, y = 1."),
            "Enter anuncia na hora, mesmo repetido"
        );
        assert_eq!(foco(&harness), Some(Target::Field(1, 3)));
    }
}

#[test]
fn autor_abre_o_site_ou_avisa() {
    let mut harness = janela_com(
        Launch {
            high_contrast: false,
            animations: false,
            open_url: conta_aberturas,
        },
        1160.0,
        810.0,
    );
    clicar(&mut harness, model::AUTHOR_LABEL);
    assert_eq!(ABERTURAS.load(Ordering::SeqCst), 1);
    assert_eq!(harness.state().model().helper(), graph::INTRO_TWO);
    let mut sem_navegador = janela_com(
        Launch {
            high_contrast: false,
            animations: false,
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
            animations: false,
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
    harness.get_by_label(model::EXAMPLE_LABEL).scroll_to_me();
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
            animations: false,
            open_url: abre_sem_navegador,
        },
        1160.0,
        810.0,
    );
    assert_eq!(*harness.state().palette(), contrast::palette(true));
    assert!(harness.state().palette().high_contrast);
    assert!(harness.state().palette().focus_glow.is_none());
}

#[test]
fn grafico_e_uma_imagem_descrita_pela_legenda() {
    let mut harness = janela();
    let no = grafico(&harness).accesskit_node();
    assert_eq!(no.role(), accesskit::Role::Image);
    assert_eq!(no.description().as_deref(), Some(graph::INTRO_TWO));
    clicar(&mut harness, model::EXAMPLE_LABEL);
    let legenda = "As retas se cruzam em (2, 1): esse ponto é a solução do sistema.";
    assert_eq!(harness.state().model().helper(), legenda);
    assert_eq!(descricao_do_grafico(&harness).as_deref(), Some(legenda));
    assert!(
        texto_visivel(&harness, legenda),
        "a legenda continua legível como a dica"
    );
    assert!(
        !regioes_vivas(&harness).contains(&legenda.to_string()),
        "a legenda não é anunciada como alerta"
    );
    assert!(
        !harness
            .state()
            .announcements()
            .contains(&legenda.to_string()),
        "o anúncio do exemplo fala do resultado, não da legenda"
    );
    clicar(&mut harness, "Sistema 3 × 3");
    assert_eq!(
        descricao_do_grafico(&harness).as_deref(),
        Some(graph::INTRO_THREE)
    );
    clicar(&mut harness, model::EXAMPLE_LABEL);
    let legenda = "Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema.";
    assert_eq!(descricao_do_grafico(&harness).as_deref(), Some(legenda));
    assert!(texto_visivel(&harness, legenda));
    harness.check_accessibility();
}

#[test]
fn erros_e_avisos_continuam_no_lugar_da_legenda() {
    let mut harness = janela();
    digitar(&mut harness, 0, 0, "2x");
    pausa(&mut harness);
    let erro = "Equação 1, coeficiente de x: Número inválido.";
    assert_eq!(harness.state().model().helper(), erro);
    assert!(texto_visivel(&harness, erro));
    assert_eq!(
        descricao_do_grafico(&harness).as_deref(),
        Some(erro),
        "a descrição é o que a legenda mostra"
    );
    digitar(&mut harness, 0, 0, "2^3");
    assert_eq!(
        descricao_do_grafico(&harness).as_deref(),
        Some("Este campo vale 8.")
    );
    apertar(&mut harness, Target::Example);
    assert!(harness.state().model().helper_is_caption());
    assert_eq!(
        descricao_do_grafico(&harness).as_deref(),
        Some("As retas se cruzam em (2, 1): esse ponto é a solução do sistema.")
    );
}

#[test]
fn legenda_centralizada_a_esquerda_e_na_cor_de_cada_caso() {
    let mut harness = janela();
    let paleta = *harness.state().palette();
    conferir_dica(&harness, graph::INTRO_TWO, color(paleta.muted));
    digitar(&mut harness, 0, 0, "2x");
    pausa(&mut harness);
    assert!(harness.state().model().helper_is_error());
    conferir_dica(
        &harness,
        "Equação 1, coeficiente de x: Número inválido.",
        color(paleta.error_text),
    );
    digitar(&mut harness, 0, 0, "2^3");
    assert!(harness.state().model().helper_is_reading());
    conferir_dica(&harness, "Este campo vale 8.", color(paleta.ink));
    apertar(&mut harness, Target::Example);
    conferir_dica(
        &harness,
        "As retas se cruzam em (2, 1): esse ponto é a solução do sistema.",
        color(paleta.muted),
    );
    clicar(&mut harness, "Sistema 3 × 3");
    conferir_dica(&harness, graph::INTRO_THREE, color(paleta.muted));
}

#[test]
fn legenda_quebra_a_linha_sem_separar_as_coordenadas() {
    let mut harness = janela();
    for (linha, valores) in [(0, ["1/3", "-2/7", "5/11"]), (1, ["3/5", "1/9", "-4/13"])] {
        for (coluna, valor) in [0, 1, 3].into_iter().zip(valores) {
            digitar(&mut harness, linha, coluna, valor);
        }
    }
    let legenda =
        "As retas se cruzam em (-5055/28171, -50715/28171): esse ponto é a solução do sistema.";
    assert_eq!(harness.state().model().helper(), legenda);
    conferir_dica(&harness, legenda, color(harness.state().palette().muted));
    let linhas: Vec<String> = linhas_da_dica(&harness)
        .into_iter()
        .map(|linha| linha.texto)
        .collect();
    assert!(linhas.len() > 1, "a legenda quebra: {linhas:?}");
    assert_eq!(
        linhas.join(" ").replace('\u{a0}', " "),
        legenda,
        "nenhuma palavra se perde: {linhas:?}"
    );
    assert!(
        linhas.iter().any(|linha| linha.contains("(-5055/28171,"))
            && linhas.iter().any(|linha| linha.contains("-50715/28171):")),
        "cada coordenada fica inteira numa linha: {linhas:?}"
    );
    assert_eq!(
        descricao_do_grafico(&harness).as_deref(),
        Some(legenda),
        "o texto acessível mantém os espaços comuns"
    );
    assert!(texto_visivel(&harness, legenda));
}

#[test]
fn painel_tem_fundo_contorno_de_um_pixel_e_cantos_redondos() {
    for alto_contraste in [false, true] {
        let harness = janela_com(
            Launch {
                high_contrast: alto_contraste,
                animations: false,
                open_url: abre_sem_navegador,
            },
            1160.0,
            810.0,
        );
        let paleta = *harness.state().palette();
        let layout = layout_padrao(2);
        let painel = na_tela(&harness, layout.graph);
        let esperado = egui::Rect::from_min_max(
            painel.min + egui::vec2(0.5, 0.5),
            painel.max - egui::vec2(0.5, 0.5),
        );
        let paineis: Vec<RectShape> = harness
            .output()
            .shapes
            .iter()
            .filter_map(|forma| match &forma.shape {
                egui::Shape::Rect(retangulo) if mesmo_retangulo(retangulo.rect, esperado) => {
                    Some(retangulo.clone())
                }
                _ => None,
            })
            .collect();
        assert_eq!(paineis.len(), 1, "um painel em layout.graph");
        let retangulo = &paineis[0];
        assert_eq!(retangulo.fill, color(paleta.graph_fill));
        assert_eq!(
            retangulo.stroke,
            egui::Stroke::new(1.0, color(paleta.graph_stroke)),
            "contorno de 1 px"
        );
        assert_eq!(
            retangulo.corner_radius,
            egui::CornerRadius::same(theme::GRAPH_RADIUS as u8)
        );
        assert!(
            painel.contains_rect(grafico(&harness).rect()),
            "a área do gráfico fica dentro do painel"
        );
        assert!(
            painel.contains_rect(na_tela(&harness, layout.helper)),
            "a legenda fica sobre o fundo do painel"
        );
    }
}

#[test]
fn grafico_pinta_retas_ponto_e_rotulos_dentro_da_area() {
    let mut harness = janela();
    let paleta = *harness.state().palette();
    let vazio = formas_do_grafico(&harness);
    let pontinhos = vazio
        .iter()
        .filter(|forma| {
            matches!(forma, egui::Shape::Circle(circulo)
                if circulo.fill == color(paleta.graph_texture) && circulo.radius < 1.0)
        })
        .count();
    assert_eq!(pontinhos, 33 * 16, "textura cobre a área de 267 × 128");
    assert_eq!(textos(&vazio), ["x", "y"]);
    clicar(&mut harness, model::EXAMPLE_LABEL);
    let formas = formas_do_grafico(&harness);
    let cheios = preenchimentos(&formas);
    for linha in 0..2 {
        assert!(
            cheios.contains(&color(paleta.graph_rows[linha])),
            "reta {linha} na cor do seu número"
        );
        assert!(
            cheios.contains(&tint(paleta.graph_rows[linha], 0.14)),
            "brilho da reta {linha}"
        );
    }
    assert!(
        !cheios.contains(&color(paleta.graph_rows[2])),
        "o 2×2 não tem terceira reta"
    );
    let e_ponto = |forma: &egui::Shape| {
        matches!(forma, egui::Shape::Circle(circulo)
            if circulo.fill == color(paleta.graph_point) && (circulo.radius - 4.25).abs() < 1e-3)
    };
    assert!(formas.iter().any(e_ponto), "o ponto da solução aparece");
    let halo = formas.iter().any(|forma| {
        matches!(forma, egui::Shape::Circle(circulo)
            if circulo.fill == tint(paleta.graph_halo, 0.2))
    });
    assert!(halo, "o halo envolve o ponto");
    assert_eq!(textos(&formas), ["x", "y", "2", "1"]);
    let guias = formas
        .iter()
        .filter(|forma| {
            matches!(forma, egui::Shape::Path(caminho)
                if caminho.closed && caminho.fill == tint(paleta.graph_guide, 0.85))
        })
        .count();
    assert!(guias > 4, "linhas-guia tracejadas: {guias}");
    let area = grafico(&harness).rect();
    let painel = layout_padrao(2).graph;
    assert!(
        area.min.x > painel.x as f32 && area.max.x < painel.right() as f32,
        "a área do gráfico fica dentro do painel"
    );
    let pontos: Vec<egui::Rect> = harness
        .output()
        .shapes
        .iter()
        .filter(|forma| e_ponto(&forma.shape))
        .map(|forma| forma.clip_rect)
        .collect();
    assert_eq!(pontos.len(), 1);
    assert!(
        pontos.iter().all(|recorte| area.contains_rect(*recorte)),
        "o desenho do gráfico fica recortado à sua área"
    );
}

#[test]
fn tres_por_tres_mostra_os_planos_e_as_retas_de_encontro() {
    let mut harness = janela();
    let paleta = *harness.state().palette();
    clicar(&mut harness, "Sistema 3 × 3");
    clicar(&mut harness, model::EXAMPLE_LABEL);
    let formas = formas_do_grafico(&harness);
    let cheios = preenchimentos(&formas);
    for linha in 0..3 {
        assert!(
            cheios.contains(&tint(paleta.graph_rows[linha], 0.26)),
            "plano {linha} translúcido"
        );
    }
    let contornos = formas
        .iter()
        .filter(|forma| {
            matches!(forma, egui::Shape::Path(caminho)
            if caminho.closed
                && caminho.fill == Color32::TRANSPARENT
                && (0..3).any(|linha| {
                    caminho.stroke.color
                        == ColorMode::Solid(tint(paleta.graph_rows[linha], 0.95))
                }))
        })
        .count();
    assert_eq!(contornos, 3, "cada plano tem contorno na sua cor");
    let encontros = formas
        .iter()
        .filter(|forma| {
            matches!(forma, egui::Shape::Path(caminho)
                if caminho.closed && caminho.fill == tint(paleta.graph_meet, 0.5))
        })
        .count();
    assert_eq!(encontros, 3, "três retas de encontro entre os planos");
    assert_eq!(textos(&formas), ["x", "z", "y"]);
    assert!(formas.iter().any(|forma| {
        matches!(forma, egui::Shape::Circle(circulo) if circulo.fill == color(paleta.graph_point))
    }));
    for (numero, linha) in [("01", 0), ("02", 1), ("03", 2)] {
        assert_eq!(
            cor_do_numero(&harness, numero),
            Some(color(paleta.graph_rows[linha])),
            "{numero} na cor do seu plano"
        );
    }
}

#[test]
fn numeros_das_equacoes_ganham_a_cor_quando_completas() {
    let mut harness = janela();
    let paleta = *harness.state().palette();
    assert_eq!(cor_do_numero(&harness, "01"), Some(color(paleta.muted)));
    for (coluna, valor) in [(0, "2"), (1, "1"), (3, "5")] {
        digitar(&mut harness, 0, coluna, valor);
    }
    assert_eq!(
        cor_do_numero(&harness, "01"),
        Some(color(paleta.graph_rows[0]))
    );
    assert_eq!(cor_do_numero(&harness, "02"), Some(color(paleta.muted)));
    assert_eq!(
        harness.state().model().helper(),
        "Preencha a equação 2 para ver onde as retas se cruzam."
    );
    let formas = formas_do_grafico(&harness);
    assert!(preenchimentos(&formas).contains(&color(paleta.graph_rows[0])));
    assert!(!preenchimentos(&formas).contains(&color(paleta.graph_rows[1])));
}

#[test]
fn alto_contraste_no_grafico_sem_transparencia_nem_textura() {
    let mut harness = janela_com(
        Launch {
            high_contrast: true,
            animations: false,
            open_url: abre_sem_navegador,
        },
        1160.0,
        810.0,
    );
    let paleta = *harness.state().palette();
    clicar(&mut harness, model::EXAMPLE_LABEL);
    let formas = formas_do_grafico(&harness);
    assert!(!formas.is_empty());
    for forma in &formas {
        for cor in cores(forma) {
            assert_eq!(cor.a(), 255, "sem alfa no alto contraste: {forma:?}");
        }
    }
    let pequenos = formas
        .iter()
        .filter(|forma| matches!(forma, egui::Shape::Circle(circulo) if circulo.radius < 1.0))
        .count();
    assert_eq!(pequenos, 0, "sem textura de pontinhos");
    let cheios = preenchimentos(&formas);
    assert!(cheios.contains(&color(paleta.graph_rows[0])));
    assert!(cheios.contains(&color(paleta.graph_rows[1])));
    assert_eq!(
        cor_do_numero(&harness, "01"),
        Some(color(paleta.graph_rows[0]))
    );
}

#[test]
fn janela_baixa_sem_painel_mantem_a_legenda() {
    let mut harness = janela_com(
        Launch {
            high_contrast: false,
            animations: false,
            open_url: abre_sem_navegador,
        },
        640.0,
        480.0,
    );
    assert!(
        harness.query_by_label(graph::GRAPH_NAME).is_some(),
        "o 2×2 tem espaço para o gráfico"
    );
    apertar(&mut harness, Target::Three);
    assert!(
        harness.query_by_label(graph::GRAPH_NAME).is_none(),
        "sem espaço, o 3×3 fica sem painel"
    );
    assert!(texto_visivel(&harness, graph::INTRO_THREE));
    apertar(&mut harness, Target::Example);
    assert!(texto_visivel(
        &harness,
        "Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema."
    ));
    esboco_em_dia(&harness);
}

#[test]
fn esboco_acompanha_cada_mudanca_do_modelo() {
    let mut harness = janela();
    esboco_em_dia(&harness);
    digitar(&mut harness, 0, 0, "2");
    esboco_em_dia(&harness);
    for (coluna, valor) in [(1, "1"), (3, "5")] {
        digitar(&mut harness, 0, coluna, valor);
        esboco_em_dia(&harness);
    }
    digitar(&mut harness, 1, 0, "1");
    digitar(&mut harness, 1, 1, "-1");
    digitar(&mut harness, 1, 3, "1");
    esboco_em_dia(&harness);
    assert_eq!(
        harness.state().graph_sketch().point(),
        Some([2.0, 1.0, 0.0])
    );
    escrever(&mut harness, "/");
    esboco_em_dia(&harness);
    pausa(&mut harness);
    esboco_em_dia(&harness);
    tecla(&mut harness, Key::Tab);
    esboco_em_dia(&harness);
    clicar(&mut harness, model::EXAMPLE_LABEL);
    esboco_em_dia(&harness);
    digitar(&mut harness, 1, 3, "1e999");
    pausa(&mut harness);
    esboco_em_dia(&harness);
    assert!(harness.state().graph_sketch().huge() || harness.state().model().is_empty());
    clicar(&mut harness, "Sistema 3 × 3");
    esboco_em_dia(&harness);
    assert_eq!(harness.state().graph_sketch().dimension, 3);
    clicar(&mut harness, model::EXAMPLE_LABEL);
    esboco_em_dia(&harness);
    clicar(&mut harness, model::CLEAR_LABEL);
    esboco_em_dia(&harness);
    assert_eq!(
        *harness.state().graph_sketch(),
        graph::Sketch::blank(3),
        "limpar volta ao gráfico vazio"
    );
}

#[test]
fn sem_animacoes_o_grafico_vai_direto_ao_repouso() {
    let mut harness = janela();
    assert!(!harness.state().animates());
    harness.get_by_label(model::EXAMPLE_LABEL).click();
    let passos = harness.run();
    assert!(passos < 10, "nada de laço de quadros: {passos}");
    assert!(!harness.state().graph_busy(relogio(&harness)));
    assert!(!repinta_ja(&harness));
    let formas = formas_do_grafico(&harness);
    let pulsos = formas
        .iter()
        .filter(|forma| {
            matches!(forma, egui::Shape::Circle(circulo)
                if circulo.fill == Color32::TRANSPARENT && circulo.radius > 3.0)
        })
        .count();
    assert_eq!(pulsos, 0, "sem pulsos parados no repouso");
    clicar(&mut harness, model::CLEAR_LABEL);
    for (linha, coluna, valor) in [(0, 0, "1"), (0, 1, "1"), (0, 3, "2")] {
        digitar(&mut harness, linha, coluna, valor);
    }
    for (linha, coluna, valor) in [(1, 0, "2"), (1, 1, "2"), (1, 3, "4")] {
        digitar(&mut harness, linha, coluna, valor);
    }
    assert_eq!(harness.state().graph_sketch().verdict, graph::Verdict::Line);
    assert!(!harness.state().graph_busy(relogio(&harness)));
    assert!(
        !repinta_ja(&harness),
        "a reta solução não marcha sem animação"
    );
}

#[test]
fn animacao_do_grafico_termina_e_para_de_pedir_quadros() {
    let mut harness = janela_animada();
    assert!(harness.state().animates());
    assert!(!repinta_ja(&harness), "parado, o app não pede quadros");
    assert!(!harness.state().graph_busy(relogio(&harness)));
    harness.get_by_label(model::EXAMPLE_LABEL).click();
    harness.run_steps(4);
    assert!(
        harness.state().graph_busy(relogio(&harness)),
        "o exemplo anima as retas e o ponto"
    );
    assert!(
        repinta_ja(&harness),
        "durante a animação pede o próximo quadro"
    );
    let antes = formas_do_grafico(&harness);
    harness.run_steps(6);
    let depois = formas_do_grafico(&harness);
    assert_ne!(antes, depois, "o gráfico muda de um quadro para outro");
    let passos = harness.run();
    let segundos = (passos + 10) as f64 / 60.0;
    assert!(
        (1.5..3.5).contains(&segundos),
        "a animação dura uns dois segundos: {segundos}"
    );
    assert!(!harness.state().graph_busy(relogio(&harness)));
    assert!(
        !repinta_ja(&harness),
        "terminada a animação, não pede mais quadros"
    );
    let parado = formas_do_grafico(&harness);
    harness.step();
    assert_eq!(
        formas_do_grafico(&harness),
        parado,
        "em repouso o desenho não muda"
    );
    let ctx = harness.ctx.clone();
    harness.state_mut().press(&ctx, Target::Clear);
    harness.run();
    for (linha, coluna, valor) in [
        (0, 0, "1"),
        (0, 1, "1"),
        (0, 3, "2"),
        (1, 0, "2"),
        (1, 1, "2"),
        (1, 3, "4"),
    ] {
        harness.state_mut().write(&ctx, linha, coluna, valor);
    }
    let inicio = relogio(&harness);
    harness.run();
    assert_eq!(harness.state().graph_sketch().verdict, graph::Verdict::Line);
    assert!(
        relogio(&harness) - inicio >= graph::MARCH - 0.05,
        "os pontinhos marcham até o fim"
    );
    assert!(!harness.state().graph_busy(relogio(&harness)));
    assert!(!repinta_ja(&harness));
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
