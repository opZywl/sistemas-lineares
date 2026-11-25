use std::fmt::Write as _;

use sistemas_lineares_ui::graph::{self, Item, Motion, Shape, Verdict};
use sistemas_lineares_ui::model::{self, Model};

const TAMANHOS_2D: [(f64, f64, f64); 4] = [
    (267.0, 128.0, 1.0),
    (233.0, 111.0, 1.0),
    (400.0, 192.0, 1.5),
    (534.0, 256.0, 2.0),
];
const TAMANHOS_3D: [(f64, f64, f64); 3] =
    [(163.0, 78.0, 1.0), (128.0, 61.0, 1.0), (244.0, 117.0, 1.5)];
const VALORES: [&str; 26] = [
    "0", "0", "1", "-1", "2", "-2", "3", "-3", "4", "5", "-5", "7", "1/2", "-3/4", "2/3", "1,5",
    "-0,25", "10", "12", "-8", "6", "9", "1e3", "2^3", "(1/2)^2", "-2²",
];

struct Sorteio(u64);

impl Sorteio {
    fn proximo(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn ate(&mut self, limite: u64) -> usize {
        (self.proximo() % limite) as usize
    }
}

fn casos_fixos() -> Vec<(usize, Vec<Vec<&'static str>>)> {
    vec![
        (2, vec![vec!["", "", ""], vec!["", "", ""]]),
        (
            3,
            vec![
                vec!["", "", "", ""],
                vec!["", "", "", ""],
                vec!["", "", "", ""],
            ],
        ),
        (2, vec![vec!["2", "1", "5"], vec!["", "", ""]]),
        (2, vec![vec!["2", "1", ""], vec!["1", "-1", "1"]]),
        (2, vec![vec!["2", "1", "5"], vec!["1", "-1", "1"]]),
        (2, vec![vec!["4", "1", "5"], vec!["1", "-1", "1"]]),
        (2, vec![vec!["1", "1", "2"], vec!["2", "2", "4"]]),
        (2, vec![vec!["1", "1", "2"], vec!["1", "1", "3"]]),
        (2, vec![vec!["1", "1", "2"], vec!["0", "0", "0"]]),
        (2, vec![vec!["1", "1", "2"], vec!["0", "0", "5/2"]]),
        (2, vec![vec!["0", "0", "0"], vec!["0", "0", "0"]]),
        (2, vec![vec!["0", "1", "0"], vec!["1", "0", "0"]]),
        (2, vec![vec!["1", "0", "3"], vec!["0", "1", "-2"]]),
        (2, vec![vec!["1", "1", "1000"], vec!["1", "-1", "0"]]),
        (2, vec![vec!["1", "1", "1"], vec!["1", "1,0001", "2"]]),
        (
            2,
            vec![vec!["2e1000", "1e1000", "5e1000"], vec!["1", "-1", "1"]],
        ),
        (2, vec![vec!["1", "1", "1e200"], vec!["1", "-1", "0"]]),
        (2, vec![vec!["1e-400", "0", "1"], vec!["0", "1", "0"]]),
        (
            2,
            vec![vec!["1/3", "-2/7", "5/11"], vec!["3/5", "1/9", "-4/13"]],
        ),
        (2, vec![vec!["1/0", "1", "2"], vec!["1", "1", "1"]]),
        (2, vec![vec!["2^10", "3", "1"], vec!["-1", "(1/2)^3", "2"]]),
        (
            3,
            vec![
                vec!["1", "1", "1", "6"],
                vec!["2", "-1", "1", "3"],
                vec!["1", "2", "-1", "2"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "1", "6"],
                vec!["", "", "", ""],
                vec!["", "", "", ""],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "1", "6"],
                vec!["2", "-1", "1", "3"],
                vec!["", "2", "", ""],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "1", "1"],
                vec!["1", "-1", "0", "0"],
                vec!["2", "0", "1", "1"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "1", "1"],
                vec!["2", "2", "2", "2"],
                vec!["0", "0", "0", "0"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "1", "1"],
                vec!["0", "0", "0", "0"],
                vec!["0", "0", "0", "0"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "1", "1"],
                vec!["1", "1", "1", "2"],
                vec!["1", "1", "1", "3"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "0", "0", "0"],
                vec!["0", "1", "0", "0"],
                vec!["1", "1", "0", "1"],
            ],
        ),
        (
            3,
            vec![
                vec!["0", "0", "0", "0"],
                vec!["0", "0", "0", "0"],
                vec!["0", "0", "0", "0"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "0", "0", "0"],
                vec!["0", "1", "0", "0"],
                vec!["0", "0", "1", "0"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "0", "0", "100"],
                vec!["0", "1", "0", "-50"],
                vec!["0", "0", "1", "25"],
            ],
        ),
        (
            3,
            vec![
                vec!["1", "1", "0", "2"],
                vec!["0", "0", "1", "3"],
                vec!["0", "0", "0", "7"],
            ],
        ),
        (
            3,
            vec![
                vec!["3/4", "-1/2", "2", "1"],
                vec!["1,5", "2", "-1", "0"],
                vec!["1", "1", "1", "1/3"],
            ],
        ),
    ]
}

fn sistema_sorteado(sorteio: &mut Sorteio, dimension: usize) -> Vec<Vec<String>> {
    let mut linhas: Vec<Vec<String>> = (0..dimension)
        .map(|_| {
            (0..=dimension)
                .map(|_| match sorteio.ate(100) {
                    0..=3 => String::new(),
                    4..=5 => "1/0".to_string(),
                    _ => VALORES[sorteio.ate(VALORES.len() as u64)].to_string(),
                })
                .collect()
        })
        .collect();
    match sorteio.ate(20) {
        0..=2 => {
            let fator = [2i64, -1, 3][sorteio.ate(3)];
            let base: Vec<i64> = (0..=dimension).map(|_| sorteio.ate(9) as i64 - 4).collect();
            linhas[0] = base.iter().map(ToString::to_string).collect();
            linhas[1] = base
                .iter()
                .map(|valor| (valor * fator).to_string())
                .collect();
            if sorteio.ate(2) == 0 {
                linhas[1][dimension] = (sorteio.ate(9) as i64 - 4).to_string();
            }
        }
        3 => {
            let linha = sorteio.ate(dimension as u64);
            for valor in linhas[linha].iter_mut().take(dimension) {
                *valor = "0".to_string();
            }
        }
        _ => {}
    }
    linhas
}

fn preencher(dimension: usize, linhas: &[Vec<String>]) -> Model {
    let mut model = Model::new();
    model.change_dimension(dimension);
    for (row, valores) in linhas.iter().enumerate() {
        for (column, valor) in valores.iter().enumerate() {
            model.edit(row, model::input_column(dimension, column), valor);
        }
    }
    model
}

fn conferir(contexto: &str, items: &[Item], width: f64, height: f64, estrito: bool) {
    let dentro = |x: f64, y: f64| {
        !estrito || (x >= -0.01 && y >= -0.01 && x <= width + 0.01 && y <= height + 0.01)
    };
    for item in items {
        assert!(
            item.alpha >= 0.0 && item.alpha <= 1.0,
            "{contexto}: {item:?}"
        );
        let pontos: Vec<(f64, f64)> = match &item.shape {
            Shape::Segment { from, to, .. } => vec![(from.x, from.y), (to.x, to.y)],
            Shape::Polygon { points } | Shape::Outline { points, .. } => {
                points.iter().map(|point| (point.x, point.y)).collect()
            }
            Shape::Disc { center, .. } | Shape::Ring { center, .. } => vec![(center.x, center.y)],
            Shape::Text { at, .. } => vec![(at.x, at.y)],
            Shape::Dots { .. } => Vec::new(),
        };
        for (x, y) in pontos {
            assert!(x.is_finite() && y.is_finite(), "{contexto}: {item:?}");
            assert!(dentro(x, y), "{contexto}: {item:?}");
        }
    }
}

fn linha_do_corpus(
    saida: &mut String,
    dimension: usize,
    linhas: &[Vec<String>],
    tamanho: (f64, f64, f64),
    contraste: bool,
) {
    let model = preencher(dimension, linhas);
    let sketch = model.sketch();
    let mut motion = Motion::new();
    motion.update(&sketch, 0.0, false);
    let (width, height, unit) = tamanho;
    let items = motion.scene(1e9, width, height, unit, contraste);
    let contexto = format!("{dimension}x{dimension} {linhas:?} {tamanho:?}");
    conferir(&contexto, &items, width, height, true);
    let legenda = model.helper();
    assert!(!legenda.is_empty(), "{contexto}");
    assert_eq!(legenda, graph::caption(&sketch), "{contexto}");
    if let Verdict::Point { at: Some(at), .. } = sketch.verdict {
        assert!(at.iter().all(|value| value.is_finite()), "{contexto}");
        assert!(
            items.iter().any(|item| item.ink == graph::Ink::Point),
            "{contexto}"
        );
    }
    let celulas: Vec<String> = linhas.iter().flatten().cloned().collect();
    let _ = writeln!(
        saida,
        "{dimension}\t{}\t{width}\t{height}\t{unit}\t{}\t{legenda}\t{}",
        celulas.join("|"),
        u8::from(contraste),
        graph::describe(&items)
    );
}

#[test]
fn corpus_de_graficos_compartilhado_com_o_html() {
    let mut saida = String::new();
    let mut linhas_geradas = 0;
    for (dimension, linhas) in casos_fixos() {
        let linhas: Vec<Vec<String>> = linhas
            .into_iter()
            .map(|linha| linha.into_iter().map(str::to_string).collect())
            .collect();
        let tamanhos: &[(f64, f64, f64)] = if dimension == 2 {
            &TAMANHOS_2D
        } else {
            &TAMANHOS_3D
        };
        for &tamanho in tamanhos {
            for contraste in [false, true] {
                linha_do_corpus(&mut saida, dimension, &linhas, tamanho, contraste);
                linhas_geradas += 1;
            }
        }
    }
    let mut sorteio = Sorteio(0x5EED_6A7F_1C0D_E5A1);
    for indice in 0..1600 {
        let dimension = 2 + indice % 2;
        let linhas = sistema_sorteado(&mut sorteio, dimension);
        let tamanhos: &[(f64, f64, f64)] = if dimension == 2 {
            &TAMANHOS_2D
        } else {
            &TAMANHOS_3D
        };
        let tamanho = tamanhos[(indice / 2) % tamanhos.len()];
        linha_do_corpus(&mut saida, dimension, &linhas, tamanho, indice % 7 == 0);
        linhas_geradas += 1;
    }
    assert!(linhas_geradas > 1700);
    if let Ok(destino) = std::env::var("SL_GRAFICOS_SAIDA") {
        std::fs::write(&destino, saida).expect("gravar o corpus de gráficos");
        println!("corpus de gráficos: {linhas_geradas} cenas em {destino}");
    }
}

#[test]
fn animacao_completa_chega_na_cena_de_repouso() {
    let mut sorteio = Sorteio(0xA11C_E5ED_0000_0001);
    for indice in 0..300 {
        let dimension = 2 + indice % 2;
        let antes = preencher(dimension, &sistema_sorteado(&mut sorteio, dimension)).sketch();
        let depois = preencher(dimension, &sistema_sorteado(&mut sorteio, dimension)).sketch();
        let (width, height, unit) = if dimension == 2 {
            TAMANHOS_2D[0]
        } else {
            TAMANHOS_3D[0]
        };
        let contraste = indice % 5 == 0;
        let mut motion = Motion::new();
        motion.update(&antes, 0.0, false);
        motion.update(&depois, 1.0, true);
        let mut agora = 1.0;
        while motion.busy(agora, contraste) {
            let items = motion.scene(agora, width, height, unit, contraste);
            conferir(&format!("quadro {agora}"), &items, width, height, false);
            agora += 1.0 / 30.0;
            assert!(agora < 10.0, "a animação precisa terminar");
        }
        let mut repouso = Motion::new();
        repouso.update(&depois, 0.0, false);
        assert_eq!(
            motion.scene(agora, width, height, unit, contraste),
            repouso.scene(1e9, width, height, unit, contraste),
            "{depois:?}"
        );
    }
}
