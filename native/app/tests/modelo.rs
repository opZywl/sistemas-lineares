use sistemas_lineares::model::{self, Focus, Model, Outcome};
use sistemas_lineares_core::Method;

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
        "Aceita 2, -1,5 e 3/4. Use 0 se a variável não aparecer."
    );
    assert!(!model.helper_is_error());
    assert!(model.is_empty());
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
    assert!(!model.helper_is_error());
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
    assert_eq!(model.fill_example(), Focus::Solve);
    assert_eq!(
        model.helper(),
        "Exemplo pronto. Clique em Resolver sistema ou pressione Enter."
    );
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
    assert!(model.is_empty());
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
fn editar_invalida_o_resultado() {
    let mut model = Model::new();
    model.fill_example();
    model.solve();
    assert!(model.copy_enabled());
    model.set_input(0, 0, "4");
    assert_eq!(model.last_result(), None);
    assert!(!model.copy_enabled());
    assert!(model.is_empty());
    assert_eq!(model.note(), "Resultados exatos, incluindo frações.");
    assert_eq!(model.helper(), model::DEFAULT_HELPER);
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
    assert_eq!(model.helper(), model::DEFAULT_HELPER);
    assert_eq!(model.dimension(), 3);
}

#[test]
fn trocar_dimensao_preserva_campos_ocultos() {
    let mut model = Model::new();
    model.change_dimension(3);
    model.fill_example();
    model.change_dimension(2);
    assert_eq!(model.input(2, 2), "-1");
    assert_eq!(model.input(0, 2), "1");
    assert!(model.is_empty());
    model.change_dimension(3);
    assert_eq!(model.input(2, 3), "2");
    model.change_dimension(7);
    assert_eq!(model.dimension(), 3);
    model.change_dimension(0);
    assert_eq!(model.dimension(), 2);
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
    assert_eq!(
        model::FOOTER,
        "Gauss-Jordan · Cramer · Escalonamento  /  aritmética racional exata"
    );
    assert_eq!(
        model::SHORTCUTS,
        "ENTER  para resolver     ·     TAB  para navegar"
    );
    assert_eq!(model::TWO_LABEL, "2 × 2");
    assert_eq!(model::THREE_LABEL, "3 × 3");
    assert!(
        model::SHORTCUTS.contains('\u{B7}')
            && model::TWO_LABEL.contains('\u{D7}')
            && model::NOTE_INFINITE.contains('\u{2026}')
    );
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
        model::SOLVE_LABEL,
        model::EXAMPLE_LABEL,
        model::CLEAR_LABEL,
        model::RESULT_LABEL,
        model::EMPTY_TITLE,
        model::EMPTY_DESCRIPTION,
        model::SUBTITLE,
        model::FOOTER,
        model::SHORTCUTS,
        model::DEFAULT_HELPER,
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
    ];
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
    assert!(!model.think_enabled());
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
            assert!(passos.steps.len() <= sistemas_lineares::thinking::STEP_POOL);
            model.close_thinking();
        }
    }
}
