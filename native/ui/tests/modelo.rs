use sistemas_lineares_core::Method;
use sistemas_lineares_core::Rational;
use sistemas_lineares_ui::graph::{self, Row, Verdict};
use sistemas_lineares_ui::model::{
    self, Card, Focus, MethodRequest, Model, Notice, Outcome, Waiting,
};

fn fill(model: &mut Model, rows: &[&[&str]]) {
    let dimension = model.dimension();
    for (row, values) in rows.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            model.set_input(row, model::input_column(dimension, column), value);
        }
    }
}

#[test]
fn estado_inicial() {
    let model = Model::new();
    assert_eq!(model.dimension(), 2);
    assert_eq!(model.dimension_text(), "2 equações · 2 incógnitas");
    assert_eq!(
        model.helper(),
        graph::INTRO_TWO,
        "a legenda do gráfico explica o que aparece ali"
    );
    assert!(!model.helper_is_error());
    assert!(model.is_empty());
    assert_eq!(model.card(), Card::Waiting(Waiting::Empty));
    assert_eq!(
        model.waiting_title().as_deref(),
        Some("Aguardando os valores")
    );
    assert!(!model.shows_preview());
    assert_eq!(model.missing(), 6);
    assert!(!model.copy_enabled());
    assert!(!model.think_enabled());
    assert!(!model.is_thinking());
    assert_eq!(model.copy_label(), "Copiar");
    assert_eq!(model.note(), "Resultados exatos, incluindo frações.");
    assert_eq!(model.title(), None);
    assert_eq!(model.announcement(), None);
}

#[test]
fn validacao_inline_aponta_o_campo() {
    let mut model = Model::new();
    let focus = model.solve();
    assert_eq!(
        focus,
        Some(Focus::Field {
            row: 0,
            column: 0,
            select_all: true
        })
    );
    assert!(model.helper().contains("Equação 1"));
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: Preencha todos os campos."
    );
    assert!(model.helper_is_error());
    assert!(model.field_has_error(0, 0));
    assert!(!model.copy_enabled());
    fill(&mut model, &[&["1", "2", "3"], &["4", "5", "x"]]);
    assert_eq!(
        model.solve(),
        Some(Focus::Field {
            row: 1,
            column: 3,
            select_all: true
        })
    );
    assert_eq!(
        model.helper(),
        "Equação 2, termo independente: Número inválido."
    );
    assert_eq!(model.error_field(), Some((1, 3)));
    model.set_input(0, 0, "1/0");
    assert_eq!(
        model.helper(),
        "Equação 2, termo independente: Número inválido.",
        "editar outro campo não apaga um erro já revelado"
    );
    assert_eq!(
        model.solve(),
        Some(Focus::Field {
            row: 0,
            column: 0,
            select_all: true
        })
    );
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: O denominador não pode ser zero."
    );
    model.set_input(0, 0, &"9".repeat(101));
    model.solve();
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: Use números com no máximo 100 caracteres."
    );
    model.set_input(0, 0, "1e5000");
    model.solve();
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: Use um expoente inteiro entre -1000 e 1000."
    );
}

#[test]
fn exemplo_2x2_resolve_e_habilita_copia() {
    let mut model = Model::new();
    assert_eq!(model.fill_example(), None);
    assert_eq!(
        model.helper(),
        "As retas se cruzam em (2, 1): esse ponto é a solução do sistema."
    );
    assert!(model.helper_is_caption());
    assert_eq!(model.notice(), Some(Notice::Example));
    assert_eq!(
        [
            model.input(0, 0),
            model.input(0, 1),
            model.input(0, 3),
            model.input(1, 0),
            model.input(1, 1),
            model.input(1, 3)
        ],
        ["2", "1", "5", "1", "-1", "1"]
    );
    assert!(!model.is_empty(), "o exemplo já sai resolvido");
    assert_eq!(model.card(), Card::Result { stale: false });
    assert_eq!(model.solve(), None);
    let result = model.last_result().unwrap_or_default().to_string();
    assert!(
        result.contains("x = 2") && result.contains("y = 1"),
        "{result}"
    );
    assert_eq!(model.values(), ["2", "1"]);
    assert!(model.copy_enabled());
    assert_eq!(model.title(), Some("Solução encontrada"));
    assert_eq!(
        model.description(),
        Some("Estes valores satisfazem todas\nas equações do sistema.")
    );
    assert_eq!(model.note(), "Solução única · valores exatos");
    assert_eq!(model.general(), None);
    assert_eq!(
        model.announcement().as_deref(),
        Some("Solução encontrada: x = 2, y = 1.")
    );
}

#[test]
fn editar_atualiza_o_resultado_na_hora() {
    let mut model = Model::new();
    model.fill_example();
    assert!(model.copy_enabled());
    model.set_input(0, 0, "4");
    assert_eq!(model.values(), ["6/5", "1/5"]);
    assert!(model.copy_enabled());
    assert_eq!(model.card(), Card::Result { stale: false });
    assert_eq!(
        model.helper(),
        "As retas se cruzam em (6/5, 1/5): esse ponto é a solução do sistema."
    );
}

#[test]
fn exemplo_3x3_resolve() {
    let mut model = Model::new();
    model.change_dimension(3);
    assert_eq!(model.dimension_text(), "3 equações · 3 incógnitas");
    model.fill_example();
    model.solve();
    let result = model.last_result().unwrap_or_default().to_string();
    assert!(
        result.contains("x = 1") && result.contains("y = 2") && result.contains("z = 3"),
        "{result}"
    );
    assert_eq!(model.values(), ["1", "2", "3"]);
}

#[test]
fn zeros_dao_infinitas_solucoes() {
    let mut model = Model::new();
    model.change_dimension(3);
    for row in 0..3 {
        for column in 0..4 {
            model.set_input(row, column, "0");
        }
    }
    model.solve();
    assert_eq!(model.title(), Some("Infinitas soluções"));
    assert_eq!(
        model.description(),
        Some("As equações são dependentes.\nVeja a forma geral da resposta:")
    );
    assert_eq!(
        model.general(),
        Some("x = t1\r\ny = t2\r\nz = t3\r\n\r\nt1, t2, ... podem ser quaisquer números reais.")
    );
    assert_eq!(
        model.note(),
        "Os parâmetros t1, t2, … são números reais livres."
    );
    assert!(model.values().is_empty());
    assert_eq!(
        model.announcement().as_deref(),
        Some("Infinitas soluções: x = t1, y = t2, z = t3.")
    );
    model.set_input(0, 3, "1");
    model.solve();
    assert_eq!(model.title(), Some("Sem solução"));
    assert_eq!(model.description(), Some("As equações são incompatíveis."));
    assert_eq!(
        model.general(),
        Some("Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo.")
    );
    assert_eq!(
        model.note(),
        "Confira os coeficientes se esse não era o esperado."
    );
    assert_eq!(
        model.announcement().as_deref(),
        Some("Sem solução. As equações são incompatíveis.")
    );
}

#[test]
fn infinitas_2x2_mostra_a_forma_geral() {
    let mut model = Model::new();
    fill(&mut model, &[&["1", "1", "2"], &["2", "2", "4"]]);
    model.solve();
    assert_eq!(
        model.outcome(),
        &Outcome::Infinite(
            "x = 2 - t1\r\ny = t1\r\n\r\nt1, t2, ... podem ser quaisquer números reais."
                .to_string()
        )
    );
    assert!(
        model
            .last_result()
            .unwrap_or_default()
            .starts_with("Infinitas soluções\r\n\r\nx = 2 - t1")
    );
}

#[test]
fn valores_longos_sao_exatos() {
    let mut model = Model::new();
    fill(&mut model, &[&["3", "0", "1"], &["0", "7", "1e-30"]]);
    model.solve();
    assert_eq!(model.values(), ["1/3", "1/7000000000000000000000000000000"]);
}

#[test]
fn limpar_reinicia_tudo() {
    let mut model = Model::new();
    model.change_dimension(3);
    model.fill_example();
    model.solve();
    assert_eq!(
        model.clear(),
        Focus::Field {
            row: 0,
            column: 0,
            select_all: false
        }
    );
    for row in 0..3 {
        for column in 0..4 {
            assert_eq!(model.input(row, column), "");
        }
    }
    assert!(!model.copy_enabled());
    assert!(model.is_empty());
    assert_eq!(model.helper(), graph::INTRO_THREE);
    assert_eq!(model.dimension(), 3);
}

#[test]
fn cada_tamanho_guarda_o_proprio_sistema() {
    let mut model = Model::new();
    model.change_dimension(3);
    model.fill_example();
    assert_eq!(model.values(), ["1", "2", "3"]);
    model.change_dimension(2);
    for row in 0..3 {
        for column in 0..4 {
            assert_eq!(model.input(row, column), "", "o 2×2 não herda o 3×3");
        }
    }
    assert_eq!(
        model.card(),
        Card::Waiting(Waiting::Empty),
        "nenhuma resposta para um sistema que ninguém digitou"
    );
    for (row, column, valor) in [
        (0, 0, "1"),
        (0, 1, "1"),
        (0, 3, "2"),
        (1, 0, "1"),
        (1, 1, "-1"),
        (1, 3, "0"),
    ] {
        model.edit(row, column, valor);
    }
    assert_eq!(model.values(), ["1", "1"]);
    model.change_dimension(3);
    assert_eq!(model.input(2, 3), "2", "o 3×3 volta como estava");
    assert_eq!(model.values(), ["1", "2", "3"], "e já resolvido");
    model.change_dimension(2);
    assert_eq!(model.values(), ["1", "1"], "o 2×2 também volta resolvido");
    model.clear();
    model.change_dimension(3);
    assert_eq!(model.input(0, 0), "", "Limpar apaga os dois tamanhos");
    assert_eq!(model.card(), Card::Waiting(Waiting::Empty));
    model.change_dimension(7);
    assert_eq!(model.dimension(), 3);
    model.change_dimension(0);
    assert_eq!(model.dimension(), 2);
}

#[test]
fn erro_revelado_nao_pisca_enquanto_corrige() {
    let mut model = Model::new();
    model.edit(0, 0, "1/0");
    model.leave(0, 0);
    assert!(model.field_has_error(0, 0));
    model.set_focus(Some((0, 0)));
    model.edit(0, 0, "1/");
    assert!(
        model.field_has_error(0, 0),
        "continua inválido: o vermelho fica"
    );
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: Número inválido."
    );
    model.edit(0, 0, "1/2");
    assert!(
        !model.field_has_error(0, 0),
        "ficou válido: o erro some na hora"
    );
    model.edit(0, 0, "1/0");
    assert!(
        !model.field_has_error(0, 0),
        "um erro novo espera a pausa, a saída ou o Enter"
    );
    model.edit(0, 0, "");
    assert!(!model.field_has_error(0, 0));

    let mut model = Model::new();
    model.set_focus(Some((0, 0)));
    assert_eq!(
        model.solve(),
        Some(Focus::Field {
            row: 0,
            column: 0,
            select_all: true
        })
    );
    assert!(model.field_has_error(0, 0), "Resolver aponta o campo vazio");
    model.edit(0, 0, "2^");
    assert!(
        !model.field_has_error(0, 0),
        "começar a digitar no campo vazio apontado não acende outro erro"
    );
    assert_eq!(model.helper(), graph::INTRO_TWO);
}

#[test]
fn dica_mostra_o_valor_lido_do_campo_com_potencia() {
    let mut model = Model::new();
    model.set_focus(Some((0, 0)));
    model.edit(0, 0, "-2\u{B2}");
    assert_eq!(model.helper(), "Este campo vale -4.");
    assert!(model.helper_is_reading());
    assert!(!model.helper_is_error());
    model.edit(0, 0, "2^10");
    assert_eq!(model.helper(), "Este campo vale 1024.");
    model.edit(0, 0, "2^100");
    assert_eq!(model.helper(), "Este campo vale 1267\u{2026}5376.");
    model.edit(0, 0, "(1/2)^3");
    assert_eq!(model.helper(), "Este campo vale 1/8.");
    model.edit(0, 0, "2^");
    assert_eq!(
        model.helper(),
        graph::INTRO_TWO,
        "prefixo ainda não tem valor"
    );
    assert!(!model.helper_is_reading());
    assert!(model.helper_is_caption());
    model.edit(0, 0, "3");
    assert_eq!(
        model.helper(),
        graph::INTRO_TWO,
        "sem potência, a dica volta à legenda"
    );
    model.edit(0, 0, "2^3");
    model.set_focus(Some((0, 1)));
    assert_eq!(
        model.helper(),
        graph::INTRO_TWO,
        "a dica segue o campo focado"
    );
    assert_eq!(
        model::reading(&Rational::parse("3/4").expect("3/4")),
        "Este campo vale 3/4."
    );
}

#[test]
fn copiar_resultado() {
    let mut model = Model::new();
    let mut seen = None;
    model.copy(|text| {
        seen = Some(text.to_string());
        true
    });
    assert_eq!(seen, None);
    model.fill_example();
    model.solve();
    model.copy(|text| {
        seen = Some(text.to_string());
        true
    });
    assert_eq!(seen.as_deref(), Some("Solução única\r\n\r\nx = 2\r\ny = 1"));
    assert_eq!(model.copy_label(), "Copiado");
    model.solve();
    assert_eq!(model.copy_label(), "Copiar");
    model.copy(|_| false);
    assert_eq!(
        model.note(),
        "Não foi possível copiar agora. Tente novamente."
    );
    assert_eq!(model.copy_label(), "Copiar");
}

#[test]
fn autor_indisponivel() {
    let mut model = Model::new();
    model.author_unavailable();
    assert_eq!(
        model.helper(),
        "Abra lucas-lima.xyz no seu navegador para visitar o autor."
    );
}

#[test]
fn nomes_acessiveis() {
    assert_eq!(
        model::accessible_input_name(0, 0),
        "Equação 1, coeficiente de x"
    );
    assert_eq!(
        model::accessible_input_name(2, 2),
        "Equação 3, coeficiente de z"
    );
    assert_eq!(
        model::accessible_input_name(1, 3),
        "Equação 2, termo independente"
    );
    assert_eq!(model::accessible_value_name(1), "Valor de y");
    assert_eq!(model::row_number(0), "01");
    assert_eq!(model::row_number(2), "03");
}

#[test]
fn textos_fixos_identicos_ao_csharp() {
    assert_eq!(model::WINDOW_TITLE, "Sistemas Lineares · Lucas Lima");
    assert_eq!(model::AUTHOR_LABEL, "Feito por Lucas Lima · RA 185964");
    assert_eq!(
        format!(
            "{} {} {}",
            model::AUTHOR_NAME,
            model::AUTHOR_SEPARATOR,
            model::AUTHOR_RA
        ),
        model::AUTHOR_LABEL
    );
    assert_eq!(model::AUTHOR_TOOLTIP, "Visitar lucas-lima.xyz");
    assert_eq!(model::AUTHOR_URL, "https://lucas-lima.xyz/");
    assert_eq!(model::INTRO, "SISTEMAS\nLINEARES.");
    assert_eq!(model::FOOTER_METHODS_LABEL, "Passo a passo por");
    assert_eq!(
        model::KEYS,
        [
            ("Enter", "próximo campo vazio"),
            ("\u{2191} \u{2193}", "mudar de equação")
        ]
    );
    assert_eq!(model::THINK_KEYS, [("Esc", "voltar")]);
    assert_eq!(model::TWO_LABEL, "2 × 2");
    assert_eq!(model::THREE_LABEL, "3 × 3");
    assert!(model::TWO_LABEL.contains('\u{D7}') && model::NOTE_INFINITE.contains('\u{2026}'));
}

fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn textos_coerentes_com_a_versao_html() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("portable");
    let Ok(app) = std::fs::read_to_string(root.join("app.js")) else {
        eprintln!("aviso: portable/app.js indisponível; coerência com o HTML não verificada");
        return;
    };
    let shell = std::fs::read_to_string(root.join("shell.html")).unwrap_or_default();
    let html = normalize(&format!("{app}\n{shell}").replace("<br>", " "));
    let texts = [
        model::WINDOW_TITLE,
        model::BRAND,
        model::AUTHOR_NAME,
        model::AUTHOR_RA,
        model::ENTRY_TITLE,
        model::ENTRY_DESCRIPTION,
        model::TWO_LABEL,
        model::THREE_LABEL,
        model::GRID_LABEL,
        model::EXAMPLE_LABEL,
        model::CLEAR_LABEL,
        model::RESULT_LABEL,
        model::EMPTY_TITLE,
        model::EMPTY_DESCRIPTION,
        model::MISSING_DESCRIPTION,
        model::INVALID_TITLE,
        model::INVALID_DESCRIPTION,
        model::LARGE_TITLE,
        model::LARGE_DESCRIPTION,
        model::ONLY_THIS_FIELD,
        model::NOTE_STALE,
        model::SUBTITLE,
        model::FOOTER_METHODS_LABEL,
        model::TIP_THINK,
        model::TIP_COPY,
        model::TIP_CLEAR,
        model::TIP_EXAMPLE,
        model::TIP_DISABLED,
        "Aceita 2 · -1,5 · 3/4 · 2^3 · 10²",
        "Use 0 se a variável não aparecer.",
        model::EXAMPLE_HELPER,
        model::NOTE_DEFAULT,
        model::NOTE_UNIQUE,
        model::NOTE_INFINITE,
        model::NOTE_NONE,
        model::COPY_LABEL,
        model::THINK_LABEL,
        model::BACK_LABEL,
        model::COPY_STEPS_LABEL,
        model::THINK_SUBTITLE,
        model::TITLE_UNIQUE,
        model::TITLE_INFINITE,
        model::TITLE_NONE,
        model::DESCRIPTION_UNIQUE,
        model::DESCRIPTION_INFINITE,
        model::DESCRIPTION_NONE,
        model::GENERAL_NONE,
        "equações · ",
        " incógnitas",
        "termo independente",
        "coeficiente de ",
        "Faltam ",
        " campos",
        "Falta 1 campo",
        " inválido",
        "número com ",
        " algarismos",
        "próximo campo vazio",
        "mudar de equação",
        "voltar",
        model::FIELD_DESCRIPTION,
        "Este campo vale ",
    ];
    let methods: Vec<&str> = sistemas_lineares_core::Method::ALL
        .into_iter()
        .flat_map(|method| {
            [
                model::method_tip(method),
                model::method_needs_system(method),
                model::method_link_name(method),
            ]
        })
        .collect();
    let texts: Vec<&str> = texts.into_iter().chain(methods).collect();
    for text in texts {
        let wanted = normalize(text);
        assert!(
            html.contains(&wanted),
            "texto ausente na versão HTML: {wanted:?}"
        );
    }
    for example in ["[[2,1,5],[1,-1,1]]", "[[1,1,1,6],[2,-1,1,3],[1,2,-1,2]]"] {
        assert!(
            app.contains(example),
            "exemplo diferente na versão HTML: {example}"
        );
    }
}

#[test]
fn modo_pensar_no_modelo() {
    let mut model = Model::new();
    assert!(!model.think_enabled());
    assert!(!model.open_thinking(), "sem resultado não há passo a passo");
    assert_eq!(model.method(), Method::GaussJordan);
    model.fill_example();
    model.solve();
    assert!(model.think_enabled());
    assert!(!model.is_thinking());
    assert!(model.open_thinking());
    assert!(model.is_thinking());
    assert_eq!(model.thinking_status(), "Gauss-Jordan · 7 passos");
    assert_eq!(
        model.walkthrough().map(|passos| passos.answer.clone()),
        Some(vec!["x = 2".to_string(), "y = 1".to_string()])
    );
    model.set_method(Method::Cramer);
    assert_eq!(model.method(), Method::Cramer);
    assert_eq!(model.thinking_status(), "Regra de Cramer · 6 passos");
    let mut copiado = None;
    model.copy_steps(|texto| {
        copiado = Some(texto.to_string());
        true
    });
    let texto = copiado.expect("passos copiados");
    assert!(texto.starts_with("Modo pensar · Regra de Cramer\r\n\r\nSistema\r\n2x + y = 5\r\n"));
    assert_eq!(texto.matches('\n').count(), texto.matches("\r\n").count());
    assert!(texto.ends_with("Sistema possível e determinado (SPD): solução única."));
    assert_eq!(model.steps_copy_label(), "Passos copiados");
    model.set_method(Method::Gauss);
    assert_eq!(model.steps_copy_label(), "Copiar passos");
    model.copy_steps(|_| false);
    assert_eq!(model.steps_copy_label(), "Copiar passos");
    assert!(model.steps_copy_failed());
    assert!(
        model
            .thinking_status()
            .contains("Não foi possível copiar agora")
    );
    assert!(model.close_thinking());
    assert!(!model.is_thinking());
    assert!(!model.close_thinking());
    assert_eq!(model.method(), Method::Gauss, "método lembrado ao voltar");
    assert!(model.open_thinking());
    assert_eq!(
        model.walkthrough().map(|passos| passos.method),
        Some(Method::Gauss)
    );
    model.set_input(0, 0, "4");
    assert!(!model.is_thinking(), "editar fecha o modo pensar");
    assert!(model.think_enabled(), "o novo sistema já tem passo a passo");
    model.fill_example();
    model.solve();
    model.open_thinking();
    model.change_dimension(3);
    assert!(
        !model.is_thinking(),
        "trocar a dimensão fecha o modo pensar"
    );
    model.fill_example();
    model.solve();
    model.open_thinking();
    model.clear();
    assert!(!model.is_thinking(), "limpar fecha o modo pensar");
    assert!(!model.think_enabled());
}

#[test]
fn modo_pensar_nos_tres_tipos_de_resposta() {
    let casos: [(&[&[&str]], &str); 3] = [
        (&[&["2", "1", "5"], &["1", "-1", "1"]], "x = 2"),
        (&[&["1", "1", "2"], &["2", "2", "4"]], "x = 2 \u{2212} t1"),
        (&[&["1", "1", "2"], &["1", "1", "3"]], "Sem solução"),
    ];
    for (linhas, primeira) in casos {
        let mut model = Model::new();
        fill(&mut model, linhas);
        assert_eq!(model.solve(), None);
        for metodo in Method::ALL {
            model.set_method(metodo);
            assert!(model.open_thinking());
            let passos = model.walkthrough().expect("passo a passo");
            assert_eq!(passos.method, metodo);
            assert_eq!(passos.answer[0], primeira, "{metodo:?}");
            assert!(passos.steps.len() <= sistemas_lineares_ui::thinking::STEP_POOL);
            model.close_thinking();
        }
    }
}

fn campo(row: usize, column: usize) -> Option<Focus> {
    Some(Focus::Field {
        row,
        column,
        select_all: true,
    })
}

#[test]
fn resolve_ao_vivo_ao_completar() {
    let mut model = Model::new();
    let valores = [
        (0, 0, "2"),
        (0, 1, "1"),
        (0, 3, "5"),
        (1, 0, "1"),
        (1, 1, "-1"),
    ];
    for (indice, (row, column, valor)) in valores.into_iter().enumerate() {
        model.set_focus(Some((row, column)));
        model.edit(row, column, valor);
        let faltam = 5 - indice;
        assert_eq!(
            model.card(),
            Card::Waiting(Waiting::Missing {
                missing: faltam,
                invalid: 0
            })
        );
        assert!(model.shows_preview());
        assert!(!model.copy_enabled() && !model.think_enabled());
    }
    assert_eq!(model.waiting_title().as_deref(), Some("Falta 1 campo"));
    assert_eq!(model.preview(), ["2x + y = 5", "x \u{2212} y = ?"]);
    assert_eq!(
        model.preview_spoken(),
        "Faltam: equação 2, termo independente."
    );
    assert!(model.row_complete(0) && !model.row_complete(1));
    model.set_focus(Some((1, 3)));
    model.edit(1, 3, "1");
    assert_eq!(model.card(), Card::Result { stale: false });
    assert_eq!(model.values(), ["2", "1"]);
    assert!(model.copy_enabled() && model.think_enabled());
    assert!(model.row_complete(1));
    assert_eq!(model.waiting_title(), None);
}

#[test]
fn digitar_um_prefixo_nao_pisca_o_card() {
    let mut model = Model::new();
    model.fill_example();
    model.set_focus(Some((0, 3)));
    for parcial in ["-", "1/", "2^", "(", "1e", "2^-"] {
        model.edit(0, 3, parcial);
        assert_eq!(
            model.card(),
            Card::Result { stale: true },
            "{parcial:?} mantém o último resultado esmaecido"
        );
        assert!(!model.helper_is_error(), "{parcial:?} não mostra erro");
        assert!(!model.copy_enabled() && !model.think_enabled());
        assert_eq!(model.last_result(), None);
        assert_eq!(model.announcement(), None);
        assert_eq!(model.note(), model::NOTE_STALE);
    }
    assert!(model.idle(), "a pausa assenta o card");
    assert_eq!(
        model.card(),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        }),
        "prefixo com foco conta como campo ainda em digitação"
    );
    assert!(!model.helper_is_error());
    model.edit(0, 3, "-3");
    assert_eq!(model.card(), Card::Result { stale: false });
    assert_eq!(model.values(), ["-2/3", "-5/3"]);
}

#[test]
fn apagar_um_valor_espera_antes_de_trocar_o_card() {
    let mut model = Model::new();
    model.fill_example();
    model.set_focus(Some((0, 0)));
    model.edit(0, 0, "");
    assert_eq!(model.card(), Card::Result { stale: true });
    model.edit(0, 0, "2");
    assert_eq!(model.card(), Card::Result { stale: false });
    model.edit(0, 0, "");
    assert!(model.idle());
    assert_eq!(
        model.card(),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        })
    );
    assert_eq!(model.preview(), ["?x + y = 5", "x \u{2212} y = 1"]);
    model.edit(0, 0, "2");
    model.edit(0, 0, "");
    assert!(model.leave(0, 0), "sair do campo assenta o card na hora");
    assert_eq!(
        model.card(),
        Card::Waiting(Waiting::Missing {
            missing: 1,
            invalid: 0
        })
    );
}

#[test]
fn erro_revelado_por_campo() {
    let mut model = Model::new();
    model.set_focus(Some((0, 0)));
    model.edit(0, 0, "2x");
    assert!(
        !model.helper_is_error(),
        "o erro espera a pausa ou a saída do campo"
    );
    assert_eq!(
        model.card(),
        Card::Waiting(Waiting::Missing {
            missing: 5,
            invalid: 1
        })
    );
    assert_eq!(
        model.waiting_title().as_deref(),
        Some("Faltam 5 campos · 1 inválido")
    );
    assert!(model.leave(0, 0));
    assert!(model.field_has_error(0, 0));
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: Número inválido."
    );
    model.set_focus(Some((0, 1)));
    model.edit(0, 1, "5");
    assert!(
        model.field_has_error(0, 0),
        "digitar em outro campo não apaga o erro"
    );
    assert!(model.helper_is_error());
    model.set_focus(Some((0, 0)));
    model.edit(0, 0, "2");
    assert!(!model.field_has_error(0, 0));
    assert!(!model.helper_is_error());
    model.edit(0, 0, "1/0");
    assert!(
        model.idle(),
        "sem ser prefixo, a pausa revela o erro mesmo com foco"
    );
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: O denominador não pode ser zero."
    );
    model.set_focus(Some((1, 1)));
    model.edit(1, 1, "1/");
    assert!(!model.idle(), "prefixo com foco não é revelado na pausa");
    assert!(!model.field_has_error(1, 1));
    assert!(model.leave(1, 1));
    assert!(model.field_has_error(1, 1));
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: O denominador não pode ser zero.",
        "sem foco num campo com erro, vale o primeiro na ordem de leitura"
    );
    model.set_focus(Some((1, 1)));
    assert_eq!(
        model.helper(),
        "Equação 2, coeficiente de y: Número inválido.",
        "com foco num campo com erro, vale o dele"
    );
}

#[test]
fn enter_vai_ao_proximo_vazio_e_resolve_quando_completo() {
    let mut model = Model::new();
    assert_eq!(model.enter(0, 0), campo(0, 1));
    model.edit(0, 0, "2");
    assert_eq!(model.enter(0, 0), campo(0, 1));
    model.edit(0, 1, "1");
    model.edit(0, 3, "5");
    model.edit(1, 1, "-1");
    assert_eq!(model.enter(1, 1), campo(1, 3));
    assert_eq!(
        model.enter(1, 3),
        campo(1, 0),
        "dá a volta pela ordem de leitura"
    );
    model.edit(1, 0, "1");
    assert_eq!(model.enter(1, 3), campo(1, 3));
    assert_eq!(model.notice(), Some(Notice::OnlyThisField));
    assert_eq!(model.helper(), "Falta só este campo.");
    assert!(!model.helper_is_error());
    model.edit(1, 3, "x");
    assert_eq!(
        model.enter(1, 3),
        campo(1, 3),
        "Enter num campo inválido revela o erro"
    );
    assert!(model.field_has_error(1, 3));
    model.edit(1, 3, "1");
    assert_eq!(model.enter(1, 3), None);
    assert_eq!(model.values(), ["2", "1"]);
}

#[test]
fn setas_mudam_de_equacao_na_mesma_coluna() {
    let mut model = Model::new();
    assert_eq!(model.neighbor(0, 1, true), campo(1, 1));
    assert_eq!(model.neighbor(1, 1, true), None);
    assert_eq!(model.neighbor(0, 3, false), None);
    assert_eq!(model.neighbor(1, 3, false), campo(0, 3));
    assert_eq!(
        model.neighbor(0, 2, true),
        None,
        "coluna z não existe no 2×2"
    );
    model.change_dimension(3);
    assert_eq!(model.neighbor(1, 2, true), campo(2, 2));
}

#[test]
fn links_de_metodo() {
    let mut model = Model::new();
    model.edit(0, 0, "2");
    assert_eq!(
        model.request_method(sistemas_lineares_core::Method::Cramer),
        MethodRequest::NeedsSystem(campo(0, 1))
    );
    assert_eq!(
        model.helper(),
        "Preencha todos os campos para ver o passo a passo pela regra de Cramer."
    );
    assert!(!model.helper_is_error());
    assert_eq!(model.method(), sistemas_lineares_core::Method::Cramer);
    model.fill_example();
    model.edit(0, 0, "abc");
    assert_eq!(
        model.request_method(sistemas_lineares_core::Method::Gauss),
        MethodRequest::NeedsSystem(campo(0, 0))
    );
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: Número inválido."
    );
    model.edit(0, 0, "2");
    assert_eq!(
        model.request_method(sistemas_lineares_core::Method::Gauss),
        MethodRequest::Opened
    );
    assert!(model.is_thinking());
    assert_eq!(
        model.walkthrough().map(|passos| passos.method),
        Some(sistemas_lineares_core::Method::Gauss)
    );
}

#[test]
fn numeros_grandes_esperam_o_enter() {
    let mut model = Model::new();
    for (row, column, valor) in [
        (0, 0, "1e1000"),
        (0, 1, "3e999"),
        (0, 3, "5"),
        (1, 0, "1"),
        (1, 1, "-2e1000"),
        (1, 3, "7e998"),
    ] {
        model.edit(row, column, valor);
    }
    assert_eq!(model.card(), Card::Waiting(Waiting::Large));
    assert_eq!(model.waiting_title().as_deref(), Some("Números grandes"));
    assert!(model.shows_preview());
    assert!(
        model.preview()[0].starts_with("1000\u{2026}0000x"),
        "{:?}",
        model.preview()
    );
    assert_eq!(model.solve(), None);
    assert!(matches!(model.outcome(), Outcome::Unique(_)));
    let anuncio = model.announcement().unwrap_or_default();
    assert!(anuncio.contains("algarismos"), "{anuncio}");
}

#[test]
fn anuncio_abrevia_valores_longos() {
    assert_eq!(model::spoken_value("2"), "2");
    assert_eq!(model::spoken_value("-3/4"), "-3/4");
    assert_eq!(
        model::spoken_value(&"7".repeat(30)),
        "número com 30 algarismos"
    );
    assert_eq!(
        model::spoken_value(&format!("1/{}", "3".repeat(40))),
        "fração com 1 algarismos sobre 40 algarismos"
    );
}

#[test]
fn exemplos_da_dica_sao_aceitos() {
    for (texto, esperado) in [
        ("2", "2"),
        ("-1,5", "-3/2"),
        ("3/4", "3/4"),
        ("2^3", "8"),
        ("10\u{B2}", "100"),
    ] {
        assert_eq!(
            Rational::parse(texto).map(|valor| valor.to_string()),
            Ok(esperado.to_string()),
            "{texto}"
        );
    }
    assert!(
        model::FIELD_DESCRIPTION.contains("2^3") && model::FIELD_DESCRIPTION.contains("10\u{B2}")
    );
}

#[test]
fn nomes_do_tamanho_e_dos_links() {
    assert_eq!(model::size_name(2, true), "Sistema 2 × 2, selecionado");
    assert_eq!(model::size_name(3, false), "Sistema 3 × 3");
    assert_eq!(
        model::method_link_name(sistemas_lineares_core::Method::Gauss),
        "Passo a passo por escalonamento"
    );
}

fn plano(row: &Row) -> graph::Plane {
    match row {
        Row::Drawn(plane) => *plane,
        other => panic!("esperava um plano desenhado, veio {other:?}"),
    }
}

#[test]
fn esboco_do_exemplo_2x2_tem_retas_e_ponto() {
    let mut model = Model::new();
    let vazio = model.sketch();
    assert_eq!(vazio.rows, [Row::Blank, Row::Blank]);
    assert_eq!(vazio.verdict, Verdict::Pending);
    model.fill_example();
    let sketch = model.sketch();
    assert_eq!(sketch.dimension, 2);
    let primeira = plano(&sketch.rows[0]);
    let raiz = 5f64.sqrt();
    assert!((primeira.normal[0] - 2.0 / raiz).abs() < 1e-15);
    assert!((primeira.normal[1] - 1.0 / raiz).abs() < 1e-15);
    assert!((primeira.offset - raiz).abs() < 1e-15);
    assert_eq!(
        sketch.verdict,
        Verdict::Point {
            at: Some([2.0, 1.0, 0.0]),
            coordinates: vec!["2".to_string(), "1".to_string()],
        }
    );
    model.edit(1, 0, "");
    let editando = model.sketch();
    assert_eq!(editando.rows[1], Row::Partial);
    assert_eq!(editando.verdict, Verdict::Pending);
    assert_eq!(
        model.helper(),
        "Preencha a equação 2 para ver onde as retas se cruzam."
    );
}

#[test]
fn esboco_do_exemplo_3x3_e_dos_casos_degenerados() {
    let mut model = Model::new();
    model.change_dimension(3);
    model.fill_example();
    assert_eq!(
        model.sketch().verdict,
        Verdict::Point {
            at: Some([1.0, 2.0, 3.0]),
            coordinates: vec!["1".to_string(), "2".to_string(), "3".to_string()],
        }
    );
    assert_eq!(
        model.helper(),
        "Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema."
    );

    let mut model = Model::new();
    fill(&mut model, &[&["1", "1", "2"], &["2", "2", "4"]]);
    assert_eq!(model.sketch().verdict, Verdict::Line);
    assert_eq!(model.helper(), graph::COINCIDENT_LINES);
    fill(&mut model, &[&["1", "1", "2"], &["0", "0", "0"]]);
    assert_eq!(model.sketch().rows[1], Row::Trivial);
    assert_eq!(model.sketch().verdict, Verdict::Line);
    fill(&mut model, &[&["1", "1", "2"], &["0", "0", "5/2"]]);
    assert_eq!(model.sketch().rows[1], Row::Impossible("5/2".to_string()));
    assert_eq!(model.sketch().verdict, Verdict::Nothing { rank: 1 });
    assert_eq!(
        model.helper(),
        "A equação 2 diz que 0 = 5/2, o que nunca acontece: o sistema não tem solução."
    );
    fill(&mut model, &[&["1", "1", "2"], &["1", "1", "3"]]);
    assert_eq!(model.helper(), graph::PARALLEL_LINES);
    fill(&mut model, &[&["0", "0", "0"], &["0", "0", "0"]]);
    assert_eq!(model.sketch().verdict, Verdict::Everything);
    assert_eq!(model.helper(), graph::EVERYTHING_TWO);

    let mut model = Model::new();
    model.change_dimension(3);
    fill(
        &mut model,
        &[
            &["1", "1", "1", "1"],
            &["1", "-1", "0", "0"],
            &["2", "0", "1", "1"],
        ],
    );
    assert_eq!(model.sketch().verdict, Verdict::Line);
    assert_eq!(model.helper(), graph::SHARED_LINE);
    fill(
        &mut model,
        &[
            &["1", "1", "1", "1"],
            &["2", "2", "2", "2"],
            &["0", "0", "0", "0"],
        ],
    );
    assert_eq!(model.sketch().verdict, Verdict::Plane);
    assert_eq!(model.helper(), graph::COINCIDENT_PLANES);
    fill(
        &mut model,
        &[
            &["1", "1", "1", "1"],
            &["0", "0", "0", "0"],
            &["0", "0", "0", "0"],
        ],
    );
    assert_eq!(
        model.helper(),
        "As outras equações valem para qualquer ponto (0 = 0), então todo o plano da equação 1 é solução."
    );
    fill(
        &mut model,
        &[
            &["1", "1", "1", "1"],
            &["1", "1", "1", "2"],
            &["1", "1", "1", "3"],
        ],
    );
    assert_eq!(model.sketch().verdict, Verdict::Nothing { rank: 1 });
    assert_eq!(model.helper(), graph::PARALLEL_PLANES);
    fill(
        &mut model,
        &[
            &["1", "0", "0", "0"],
            &["0", "1", "0", "0"],
            &["1", "1", "0", "1"],
        ],
    );
    assert_eq!(model.sketch().verdict, Verdict::Nothing { rank: 2 });
    assert_eq!(model.helper(), graph::NO_COMMON_POINT);
}

#[test]
fn esboco_normaliza_numeros_enormes_e_espera_o_enter() {
    let mut model = Model::new();
    fill(
        &mut model,
        &[&["2e1000", "1e1000", "5e1000"], &["1", "-1", "1"]],
    );
    let sketch = model.sketch();
    let primeira = plano(&sketch.rows[0]);
    let raiz = 5f64.sqrt();
    assert!((primeira.normal[0] - 2.0 / raiz).abs() < 1e-15);
    assert!((primeira.offset - raiz).abs() < 1e-15);
    assert_eq!(model.card(), Card::Waiting(Waiting::Large));
    assert_eq!(sketch.verdict, Verdict::Waiting);
    assert_eq!(model.helper(), graph::WAITING);
    model.enter(0, 0);
    assert!(matches!(
        model.sketch().verdict,
        Verdict::Point { at: Some(_), .. }
    ));

    let mut model = Model::new();
    fill(&mut model, &[&["1", "1", "1e200"], &["1", "-1", "0"]]);
    assert_eq!(model.sketch().rows[0], Row::Huge);
    assert_eq!(model.helper(), graph::HUGE);

    let mut model = Model::new();
    fill(&mut model, &[&["1e-400", "0", "1"], &["0", "1", "0"]]);
    assert_eq!(model.sketch().rows[0], Row::Huge);
    assert!(matches!(
        model.sketch().verdict,
        Verdict::Point { at: None, .. }
    ));
}

#[test]
fn legenda_cede_lugar_ao_erro_e_aos_avisos() {
    let mut model = Model::new();
    model.set_focus(Some((0, 0)));
    model.edit(0, 0, "1/0");
    model.leave(0, 0);
    assert!(model.helper_is_error());
    assert!(!model.helper_is_caption());
    assert_eq!(
        model.helper(),
        "Equação 1, coeficiente de x: O denominador não pode ser zero."
    );
    model.edit(0, 0, "1");
    assert!(model.helper_is_caption());
    assert_eq!(
        model.request_method(Method::Cramer),
        MethodRequest::NeedsSystem(Some(Focus::Field {
            row: 0,
            column: 1,
            select_all: true
        }))
    );
    assert!(!model.helper_is_caption());
    assert_eq!(model.helper(), model::method_needs_system(Method::Cramer));
}
