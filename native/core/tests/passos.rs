mod comum;

use std::fs;

use comum::{RandomDotNet, codificar_base64};
use sistemas_lineares_core::steps::{self, LineKind, number};
use sistemas_lineares_core::{
    Method, Rational, RowOp, SolutionKind, VARIABLE_NAMES, Walkthrough, format, solve, walkthrough,
};

fn r(value: i64) -> Rational {
    Rational::integer(value)
}

fn q(text: &str) -> Rational {
    Rational::parse(text).unwrap_or_else(|erro| panic!("{text}: {erro}"))
}

fn sistema(linhas: &[&[&str]]) -> Vec<Vec<Rational>> {
    linhas
        .iter()
        .map(|linha| linha.iter().map(|valor| q(valor)).collect())
        .collect()
}

fn inteiros(linhas: &[Vec<i64>]) -> Vec<Vec<Rational>> {
    linhas
        .iter()
        .map(|linha| linha.iter().map(|&valor| r(valor)).collect())
        .collect()
}

fn leibniz(matriz: &[Vec<Rational>]) -> Rational {
    let n = matriz.len();
    let permutacoes: Vec<Vec<usize>> = if n == 2 {
        vec![vec![0, 1], vec![1, 0]]
    } else {
        vec![
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ]
    };
    let mut total = Rational::zero();
    for permutacao in permutacoes {
        let mut inversoes = 0;
        for i in 0..n {
            for j in i + 1..n {
                if permutacao[i] > permutacao[j] {
                    inversoes += 1;
                }
            }
        }
        let mut produto = Rational::one();
        for (linha, &coluna) in permutacao.iter().enumerate() {
            produto = &produto * &matriz[linha][coluna];
        }
        total = if inversoes % 2 == 0 {
            &total + &produto
        } else {
            &total - &produto
        };
    }
    total
}

fn aplicar(matriz: &mut [Vec<Rational>], operacao: &RowOp) {
    match operacao {
        RowOp::Swap { first, second } => matriz.swap(*first, *second),
        RowOp::Divide { row, divisor } => {
            assert!(
                !divisor.is_zero(),
                "divisão por zero numa operação de linha"
            );
            for valor in &mut matriz[*row] {
                *valor = &*valor / divisor;
            }
        }
        RowOp::Eliminate {
            target,
            source,
            factor,
        } => {
            assert_ne!(target, source, "linha eliminando a si mesma");
            let fonte = matriz[*source].clone();
            for (valor, base) in matriz[*target].iter_mut().zip(&fonte) {
                *valor = &*valor - &(factor * base);
            }
        }
    }
}

fn respostas_do_format(entrada: &[Vec<Rational>]) -> Vec<String> {
    let solucao = solve(entrada).unwrap();
    if solucao.kind == SolutionKind::None {
        return vec!["Sem solução".to_string()];
    }
    format(&solucao)
        .split("\r\n")
        .skip(2)
        .take(entrada.len())
        .map(|linha| linha.replace('-', "\u{2212}"))
        .collect()
}

fn escalonada(matriz: &[Vec<Rational>]) -> bool {
    let n = matriz.len();
    let mut ultimo: Option<usize> = None;
    let mut zerou = false;
    for linha in matriz {
        match linha[..n].iter().position(|valor| !valor.is_zero()) {
            None => zerou = true,
            Some(coluna) => {
                if zerou {
                    return false;
                }
                if let Some(anterior) = ultimo {
                    if coluna <= anterior {
                        return false;
                    }
                }
                ultimo = Some(coluna);
            }
        }
    }
    true
}

fn texto_limpo(contexto: &str, texto: &str) {
    for proibido in [
        "+ \u{2212}",
        "\u{2212} \u{2212}",
        "\u{2212}\u{2212}",
        "+ +",
        "--",
        "- ",
        "NaN",
        "não confere",
        "  ,",
        "()",
        " 1·L",
        "(1)",
        "\r",
    ] {
        assert!(
            !texto.contains(proibido),
            "{contexto}: texto contém {proibido:?}:\n{texto}"
        );
    }
    for nome in VARIABLE_NAMES {
        for prefixo in [" 1", "\u{2212}1", "(1"] {
            let ruim = format!("{prefixo}{nome}");
            assert!(
                !texto.contains(&ruim),
                "{contexto}: coeficiente 1 explícito {ruim:?}:\n{texto}"
            );
        }
        let zero = format!(" 0{nome}");
        for linha in texto.lines() {
            if linha.contains(&zero) {
                assert!(
                    linha.contains("→  0 ="),
                    "{contexto}: coeficiente 0 fora da contradição: {linha:?}"
                );
            }
        }
    }
}

fn conferir(entrada: &[Vec<Rational>], metodo: Method) -> Walkthrough {
    let contexto = format!("{metodo:?} {:?}", texto_do_sistema(entrada));
    let passos = walkthrough(entrada, metodo).unwrap();
    let solucao = solve(entrada).unwrap();
    let n = entrada.len();
    assert_eq!(passos.kind, solucao.kind, "{contexto}: classificação");
    assert_eq!(passos.method, metodo);
    assert_eq!(
        passos.answer,
        respostas_do_format(entrada),
        "{contexto}: resposta diferente do resultado"
    );
    assert!(!passos.steps.is_empty(), "{contexto}: sem passos");
    for passo in &passos.steps {
        assert!(!passo.title.trim().is_empty(), "{contexto}: título vazio");
        assert!(
            passo.reasoning.trim().len() > 10,
            "{contexto}: raciocínio vazio em {}",
            passo.title
        );
        assert!(
            passo.reasoning.ends_with('.') || passo.reasoning.ends_with(':'),
            "{contexto}: raciocínio sem ponto final em {}: {}",
            passo.title,
            passo.reasoning
        );
        for linha in &passo.lines {
            assert!(!linha.text.trim().is_empty(), "{contexto}: linha vazia");
        }
        if let Some(matriz) = &passo.matrix {
            let largura = if matriz.augmented { n + 1 } else { n };
            assert_eq!(matriz.rows.len(), n, "{contexto}: linhas da matriz");
            assert!(
                matriz.rows.iter().all(|linha| linha.len() == largura),
                "{contexto}: colunas da matriz"
            );
            for &linha in &matriz.marked_rows {
                assert!(linha < n);
            }
            if let Some((linha, coluna)) = matriz.pivot {
                assert!(
                    !matriz.rows[linha][coluna].is_zero(),
                    "{contexto}: pivô zero em {}",
                    passo.title
                );
            }
        }
    }
    let texto = passos.text();
    texto_limpo(&contexto, &texto);
    match metodo {
        Method::GaussJordan => conferir_eliminacao(&contexto, entrada, &passos.steps, true),
        Method::Gauss => conferir_eliminacao(&contexto, entrada, &passos.steps, false),
        Method::Cramer => conferir_cramer(&contexto, entrada, &passos),
    }
    let verificacoes: Vec<&str> = passos
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .filter(|linha| linha.text.contains("ª equação:"))
        .map(|linha| linha.text.as_str())
        .collect();
    match solucao.kind {
        SolutionKind::None => assert!(
            verificacoes.is_empty(),
            "{contexto}: verificação num sistema sem solução"
        ),
        _ => {
            assert_eq!(
                verificacoes.len(),
                n,
                "{contexto}: uma verificação por equação"
            );
            for linha in verificacoes {
                assert!(linha.ends_with("(confere)"), "{contexto}: {linha}");
            }
        }
    }
    let ultimo_resultado: Vec<&str> = passos
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .filter(|linha| linha.kind == LineKind::Result)
        .map(|linha| linha.text.as_str())
        .collect();
    assert!(
        !ultimo_resultado.is_empty(),
        "{contexto}: nenhum resultado destacado"
    );
    passos
}

fn conferir_eliminacao(
    contexto: &str,
    entrada: &[Vec<Rational>],
    passos: &[steps::Step],
    reduzida: bool,
) {
    let n = entrada.len();
    let mut atual = entrada.to_vec();
    let mut operacoes = 0;
    for passo in passos {
        for operacao in &passo.operations {
            if let RowOp::Divide { .. } = operacao {
                assert!(reduzida, "{contexto}: escalonamento não divide linhas");
            }
            if let RowOp::Eliminate { target, source, .. } = operacao {
                if !reduzida {
                    assert!(target > source, "{contexto}: Gauss só zera abaixo do pivô");
                }
            }
            aplicar(&mut atual, operacao);
            operacoes += 1;
        }
        if let Some(matriz) = &passo.matrix {
            if matriz.augmented {
                assert_eq!(
                    matriz.rows, atual,
                    "{contexto}: matriz do passo {:?} não bate com as operações",
                    passo.title
                );
            }
        }
    }
    let solucao = solve(entrada).unwrap();
    if reduzida {
        assert_eq!(
            atual, solucao.reduced,
            "{contexto}: Gauss-Jordan não chegou à forma reduzida do resultado"
        );
    } else {
        assert!(escalonada(&atual), "{contexto}: não ficou escalonada");
        let posto = atual
            .iter()
            .filter(|linha| linha[..n].iter().any(|valor| !valor.is_zero()))
            .count();
        assert_eq!(
            posto,
            solucao.pivot_columns.len(),
            "{contexto}: posto da forma escalonada"
        );
    }
    let _ = operacoes;
}

fn conferir_cramer(contexto: &str, entrada: &[Vec<Rational>], passos: &Walkthrough) {
    let n = entrada.len();
    let a: Vec<Vec<Rational>> = entrada.iter().map(|linha| linha[..n].to_vec()).collect();
    let d = leibniz(&a);
    assert_eq!(passos.determinants[0].0, "D");
    assert_eq!(passos.determinants[0].1, d, "{contexto}: D");
    let linha_d = passos
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .find(|linha| linha.kind == LineKind::Result && linha.text.starts_with("D = "))
        .map(|linha| linha.text.clone());
    assert_eq!(
        linha_d,
        Some(format!("D = {}", number(&d))),
        "{contexto}: linha do determinante"
    );
    if d.is_zero() {
        assert_eq!(passos.determinants.len(), 1, "{contexto}: Dx com D = 0");
        let inicio = passos
            .steps
            .iter()
            .position(|passo| passo.title == "Montar a matriz aumentada")
            .unwrap_or_else(|| panic!("{contexto}: D = 0 sem escalonamento"));
        conferir_eliminacao(contexto, entrada, &passos.steps[inicio..], true);
        assert_ne!(passos.kind, SolutionKind::Unique);
        return;
    }
    assert_eq!(passos.kind, SolutionKind::Unique, "{contexto}: D ≠ 0");
    let solucao = solve(entrada).unwrap();
    for coluna in 0..n {
        let trocada: Vec<Vec<Rational>> = entrada
            .iter()
            .map(|linha| {
                let mut nova = linha[..n].to_vec();
                nova[coluna] = linha[n].clone();
                nova
            })
            .collect();
        let dv = leibniz(&trocada);
        let (nome, valor) = &passos.determinants[coluna + 1];
        assert_eq!(nome, &format!("D{}", VARIABLE_NAMES[coluna]));
        assert_eq!(valor, &dv, "{contexto}: {nome}");
        assert_eq!(
            &(&dv / &d),
            &solucao.reduced[coluna][n],
            "{contexto}: {nome} ÷ D"
        );
        let passo = passos
            .steps
            .iter()
            .find(|passo| passo.title == format!("Calcular {nome}"))
            .unwrap_or_else(|| panic!("{contexto}: passo de {nome}"));
        let matriz = passo.matrix.as_ref().expect("matriz trocada");
        assert_eq!(matriz.rows, trocada, "{contexto}: matriz de {nome}");
        assert_eq!(matriz.marked_column, Some(coluna));
        assert_eq!(
            passo.lines.last().map(|linha| linha.text.clone()),
            Some(format!("{nome} = {}", number(&dv))),
            "{contexto}: valor de {nome}"
        );
    }
}

fn texto_do_sistema(entrada: &[Vec<Rational>]) -> Vec<String> {
    entrada
        .iter()
        .map(|linha| {
            linha
                .iter()
                .map(Rational::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

fn todos_os_metodos(entrada: &[Vec<Rational>]) {
    for metodo in Method::ALL {
        conferir(entrada, metodo);
    }
}

#[test]
fn exemplos_do_app_passo_a_passo() {
    let dois = inteiros(&[vec![2, 1, 5], vec![1, -1, 1]]);
    let tres = inteiros(&[vec![1, 1, 1, 6], vec![2, -1, 1, 3], vec![1, 2, -1, 2]]);
    for entrada in [&dois, &tres] {
        todos_os_metodos(entrada);
    }
    let cramer = walkthrough(&dois, Method::Cramer).unwrap();
    assert_eq!(
        cramer.determinants,
        vec![
            ("D".to_string(), r(-3)),
            ("Dx".to_string(), r(-6)),
            ("Dy".to_string(), r(-3)),
        ]
    );
    let linhas: Vec<&str> = cramer
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .map(|linha| linha.text.as_str())
        .collect();
    for esperado in [
        "D = 2·(−1) − 1·1",
        "D = −2 − 1",
        "D = −3",
        "Dx = 5·(−1) − 1·1",
        "Dx = −6",
        "x = Dx ÷ D = −6 ÷ (−3) = 2",
        "y = Dy ÷ D = −3 ÷ (−3) = 1",
        "1ª equação: 2·2 + 1·1 = 4 + 1 = 5 (confere)",
        "2ª equação: 1·2 + (−1)·1 = 2 − 1 = 1 (confere)",
    ] {
        assert!(
            linhas.contains(&esperado),
            "falta {esperado:?} em {linhas:#?}"
        );
    }
    let sarrus = walkthrough(&tres, Method::Cramer).unwrap();
    let linhas: Vec<&str> = sarrus
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .map(|linha| linha.text.as_str())
        .collect();
    for esperado in [
        "D = (1·(−1)·(−1) + 1·1·1 + 1·2·2) − (1·(−1)·1 + 1·1·2 + 1·2·(−1))",
        "D = (1 + 1 + 4) − (−1 + 2 − 2)",
        "D = 6 − (−1)",
        "D = 7",
        "x = Dx ÷ D = 7 ÷ 7 = 1",
        "y = Dy ÷ D = 14 ÷ 7 = 2",
        "z = Dz ÷ D = 21 ÷ 7 = 3",
    ] {
        assert!(
            linhas.contains(&esperado),
            "falta {esperado:?} em {linhas:#?}"
        );
    }
    let gj = walkthrough(&dois, Method::GaussJordan).unwrap();
    let linhas: Vec<&str> = gj
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .map(|linha| linha.text.as_str())
        .collect();
    for esperado in [
        "L₁ ← L₁ ÷ 2",
        "(2, 1 | 5) ÷ 2 = (1, 1/2 | 5/2)",
        "L₂ ← L₂ − L₁",
        "(1, −1 | 1) − (1, 1/2 | 5/2) = (0, −3/2 | −3/2)",
        "L₂ ← L₂ ÷ (−3/2)",
        "L₁ ← L₁ − (1/2)·L₂",
        "x = 2",
        "y = 1",
    ] {
        assert!(
            linhas.contains(&esperado),
            "falta {esperado:?} em {linhas:#?}"
        );
    }
    let gauss = walkthrough(&dois, Method::Gauss).unwrap();
    let linhas: Vec<&str> = gauss
        .steps
        .iter()
        .flat_map(|passo| passo.lines.iter())
        .map(|linha| linha.text.as_str())
        .collect();
    for esperado in [
        "m = 1 ÷ 2 = 1/2",
        "L₂ ← L₂ − (1/2)·L₁",
        "(1, −1 | 1) − (1/2)·(2, 1 | 5) = (0, −3/2 | −3/2)",
        "L₂: −(3/2)y = −3/2",
        "y = (−3/2) ÷ (−3/2) = 1",
        "L₁: 2x + y = 5",
        "2x + 1·1 = 5",
        "2x = 5 − 1 = 4",
        "x = 4 ÷ 2 = 2",
    ] {
        assert!(
            linhas.contains(&esperado),
            "falta {esperado:?} em {linhas:#?}"
        );
    }
}

#[test]
fn casos_especiais_de_cada_metodo() {
    let casos: Vec<Vec<Vec<Rational>>> = vec![
        inteiros(&[vec![0, 1, 2], vec![1, 0, 3]]),
        inteiros(&[vec![1, 1, 2], vec![2, 2, 4]]),
        inteiros(&[vec![1, 1, 2], vec![1, 1, 3]]),
        inteiros(&[vec![0, 0, 0], vec![0, 0, 0]]),
        inteiros(&[vec![0, 0, 1], vec![0, 0, 1]]),
        inteiros(&[vec![0, 0, 0], vec![0, 0, 5]]),
        inteiros(&[vec![0, 2, 4], vec![0, 3, 6]]),
        inteiros(&[vec![0, 2, 4], vec![0, 3, 7]]),
        inteiros(&[vec![1, 0, 7], vec![0, 1, -3]]),
        inteiros(&[vec![0, 1, 1, 1], vec![1, 0, 1, 2], vec![1, 1, 0, 3]]),
        inteiros(&[vec![1, 2, 3, 4], vec![2, 4, 6, 8], vec![3, 6, 9, 12]]),
        inteiros(&[vec![1, 2, 3, 4], vec![2, 4, 6, 8], vec![3, 6, 9, 13]]),
        inteiros(&[vec![1, 1, 1, 1], vec![1, 1, 1, 2], vec![1, 1, 1, 3]]),
        inteiros(&[vec![0, 0, 0, 0], vec![0, 0, 0, 0], vec![0, 0, 0, 0]]),
        inteiros(&[vec![0, 0, 1, 2], vec![0, 0, 0, 0], vec![0, 0, 0, 0]]),
        inteiros(&[vec![0, 1, 0, 2], vec![0, 0, 1, 3], vec![0, 0, 0, 0]]),
        inteiros(&[vec![0, 1, 2, 3], vec![0, 2, 4, 6], vec![0, 0, 0, 0]]),
        inteiros(&[vec![1, 0, 0, 1], vec![0, 1, 0, 2], vec![0, 0, 1, 3]]),
        inteiros(&[vec![0, 0, 1, 3], vec![0, 1, 0, 2], vec![1, 0, 0, 1]]),
        inteiros(&[vec![1, -1, 2, 5], vec![2, -2, 4, 10], vec![0, 0, 1, 1]]),
        inteiros(&[vec![2, 4, -2, 2], vec![4, 9, -3, 8], vec![-2, -3, 7, 10]]),
        inteiros(&[vec![1, 2, 1, 0], vec![2, 4, 3, 1], vec![3, 6, 4, 1]]),
        sistema(&[&["1/2", "-3/4", "5/6"], &["-7/8", "9/10", "-11/12"]]),
        sistema(&[&["0,5", "1e-3", "2"], &["-1,25", "3/7", "-1e2"]]),
        sistema(&[
            &["1/3", "1/4", "1/5", "1"],
            &["1/4", "1/5", "1/6", "2"],
            &["1/5", "1/6", "1/7", "3"],
        ]),
        sistema(&[
            &["99999999999999999999", "1", "100000000000000000000"],
            &["1", "-99999999999999999999", "-99999999999999999998"],
        ]),
        sistema(&[
            &["1", "1", "2"],
            &["1", "1.00000000000000000001", "2.00000000000000000001"],
        ]),
    ];
    for caso in &casos {
        todos_os_metodos(caso);
    }
    let singular = walkthrough(&casos[1], Method::Cramer).unwrap();
    assert!(
        singular
            .steps
            .iter()
            .any(|passo| passo.title == "D = 0: Cramer não decide"),
        "Cramer com D = 0 precisa explicar e escalonar"
    );
    let impossivel = walkthrough(&casos[2], Method::Gauss).unwrap();
    assert!(
        impossivel
            .steps
            .iter()
            .any(|passo| passo.title == "Achar a contradição")
    );
    assert_eq!(impossivel.answer, vec!["Sem solução".to_string()]);
    let livre = walkthrough(&casos[1], Method::Gauss).unwrap();
    assert_eq!(livre.answer, vec!["x = 2 − t1", "y = t1"]);
    let tudo_livre = walkthrough(&casos[13], Method::GaussJordan).unwrap();
    assert_eq!(tudo_livre.answer, vec!["x = t1", "y = t2", "z = t3"]);
    assert!(
        tudo_livre
            .text()
            .contains("que podem valer qualquer número")
    );
}

#[test]
fn todos_os_2x2_pequenos_em_todos_os_metodos() {
    let mut por_tipo = [0usize; 3];
    for code in 0..15625 {
        let mut value = code;
        let mut a = vec![vec![0i64; 3]; 2];
        for linha in a.iter_mut() {
            for valor in linha.iter_mut() {
                *valor = value % 5 - 2;
                value /= 5;
            }
        }
        let entrada = inteiros(&a);
        for metodo in Method::ALL {
            let passos = conferir(&entrada, metodo);
            por_tipo[match passos.kind {
                SolutionKind::Unique => 0,
                SolutionKind::Infinite => 1,
                SolutionKind::None => 2,
            }] += 1;
        }
    }
    assert!(por_tipo.iter().all(|&total| total > 0), "{por_tipo:?}");
    println!(
        "PASS: 15.625 sistemas 2×2 × 3 métodos ({} única, {} infinitas, {} sem solução)",
        por_tipo[0], por_tipo[1], por_tipo[2]
    );
}

#[test]
fn sistemas_3x3_em_todos_os_metodos() {
    let mut total = 0usize;
    for code in 0..19683 {
        let mut value = code;
        let mut a = vec![vec![0i64; 4]; 3];
        for linha in a.iter_mut() {
            for valor in linha[..3].iter_mut() {
                *valor = value % 3 - 1;
                value /= 3;
            }
        }
        for b in 0..2 {
            for (i, linha) in (0i64..).zip(a.iter_mut()) {
                linha[3] = if b == 0 { 0 } else { i + 1 };
            }
            todos_os_metodos(&inteiros(&a));
            total += 1;
        }
    }
    let mut random = RandomDotNet::new(185964);
    for k in 0..3000 {
        let n = k % 2 + 2;
        let mut a = vec![vec![Rational::zero(); n + 1]; n];
        for linha in a.iter_mut() {
            for valor in linha.iter_mut() {
                let numerador = random.next(-9, 10);
                let denominador = if random.next(0, 4) == 0 {
                    random.next(1, 7)
                } else {
                    1
                };
                *valor = &r(i64::from(numerador)) / &r(i64::from(denominador));
            }
        }
        if k % 7 == 0 {
            let copia = a[0].clone();
            a[n - 1] = copia.iter().map(|valor| valor * &r(2)).collect();
        }
        todos_os_metodos(&a);
        total += 1;
    }
    println!(
        "PASS: {total} sistemas × 3 métodos com operações refeitas e determinantes independentes"
    );
}

#[test]
fn texto_para_copiar_e_paridade_com_javascript() {
    let mut fixtures = String::new();
    let mut random = RandomDotNet::new(2026);
    let mut entradas: Vec<Vec<Vec<Rational>>> = vec![
        inteiros(&[vec![2, 1, 5], vec![1, -1, 1]]),
        inteiros(&[vec![1, 1, 1, 6], vec![2, -1, 1, 3], vec![1, 2, -1, 2]]),
        inteiros(&[vec![1, 1, 2], vec![2, 2, 4]]),
        inteiros(&[vec![1, 1, 2], vec![1, 1, 3]]),
        inteiros(&[vec![0, 0, 0, 0], vec![0, 0, 0, 0], vec![0, 0, 0, 0]]),
        inteiros(&[vec![0, 1, 2, 3], vec![0, 2, 4, 6], vec![0, 0, 0, 0]]),
        inteiros(&[vec![1, 2, 3, 4], vec![2, 4, 6, 8], vec![3, 6, 9, 13]]),
        inteiros(&[vec![0, 0, 1, 3], vec![0, 1, 0, 2], vec![1, 0, 0, 1]]),
        sistema(&[&["1/2", "-3/4", "5/6"], &["-7/8", "9/10", "-11/12"]]),
    ];
    for k in 0..600 {
        let n = k % 2 + 2;
        let mut a = vec![vec![Rational::zero(); n + 1]; n];
        for linha in a.iter_mut() {
            for valor in linha.iter_mut() {
                let numerador = random.next(-6, 7);
                let denominador = if random.next(0, 5) == 0 {
                    random.next(1, 5)
                } else {
                    1
                };
                *valor = &r(i64::from(numerador)) / &r(i64::from(denominador));
            }
        }
        if k % 5 == 0 {
            a[n - 1] = a[0].iter().map(|valor| valor * &r(-3)).collect();
        }
        if k % 11 == 0 {
            for linha in a.iter_mut() {
                linha[k % n] = Rational::zero();
            }
        }
        entradas.push(a);
    }
    for entrada in &entradas {
        for metodo in Method::ALL {
            let texto = conferir(entrada, metodo).text();
            assert!(texto.starts_with(&format!("Modo pensar · {}", metodo.name())));
            assert!(texto.contains("\nResposta\n"));
            let valores: Vec<String> = entrada.iter().flatten().map(Rational::to_string).collect();
            fixtures.push_str(&format!(
                "{}|{}|{}|{}\n",
                entrada.len(),
                valores.join(";"),
                metodo.key(),
                codificar_base64(texto.as_bytes())
            ));
        }
    }
    if let Some(caminho) = std::env::var_os("SL_PASSOS_SAIDA") {
        fs::write(&caminho, &fixtures).expect("gravar SL_PASSOS_SAIDA");
        println!(
            "passos gravados para a paridade com o JavaScript: {} ({} linhas)",
            caminho.to_string_lossy(),
            fixtures.lines().count()
        );
    }
    if std::env::var_os("SL_MOSTRAR_PASSOS").is_some() {
        for metodo in Method::ALL {
            for entrada in entradas.iter().take(9) {
                println!(
                    "==========\n{}\n",
                    walkthrough(entrada, metodo).unwrap().text()
                );
            }
        }
    }
}
