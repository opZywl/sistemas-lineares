use sistemas_lineares_core::{Rational, Solution, SolutionKind, VARIABLE_NAMES, format, solve};

type Linhas<'a> = &'a [&'a [&'a str]];

fn resolver(linhas: Linhas) -> Solution {
    let matriz: Vec<Vec<Rational>> = linhas
        .iter()
        .map(|linha| {
            linha
                .iter()
                .map(|valor| Rational::parse(valor).unwrap())
                .collect()
        })
        .collect();
    solve(&matriz).unwrap()
}

const APRESENTACAO: &[(&str, Linhas, SolutionKind, &str)] = &[
    (
        "Única 2×2",
        &[&["2", "1", "5"], &["1", "-1", "1"]],
        SolutionKind::Unique,
        "Solução única\r\n\r\nx = 2\r\ny = 1",
    ),
    (
        "Única 3×3",
        &[
            &["1", "1", "1", "6"],
            &["2", "-1", "1", "3"],
            &["1", "2", "-1", "2"],
        ],
        SolutionKind::Unique,
        "Solução única\r\n\r\nx = 1\r\ny = 2\r\nz = 3",
    ),
    (
        "Infinitas",
        &[&["1", "1", "2"], &["2", "2", "4"]],
        SolutionKind::Infinite,
        "Infinitas soluções\r\n\r\nx = 2 - t1\r\ny = t1\r\n\r\nt1, t2, ... podem ser quaisquer números reais.",
    ),
    (
        "Sem solução",
        &[&["1", "1", "2"], &["2", "2", "5"]],
        SolutionKind::None,
        "Este sistema não tem solução.\r\nAs equações são incompatíveis.",
    ),
    (
        "Troca de linhas",
        &[&["0", "1", "2"], &["1", "0", "3"]],
        SolutionKind::Unique,
        "Solução única\r\n\r\nx = 3\r\ny = 2",
    ),
    (
        "Fração exata",
        &[&["3", "0", "1"], &["0", "2", "1"]],
        SolutionKind::Unique,
        "Solução única\r\n\r\nx = 1/3\r\ny = 1/2",
    ),
    (
        "Todas livres",
        &[&["0", "0", "0"], &["0", "0", "0"]],
        SolutionKind::Infinite,
        "Infinitas soluções\r\n\r\nx = t1\r\ny = t2\r\n\r\nt1, t2, ... podem ser quaisquer números reais.",
    ),
];

#[test]
fn os_sete_casos_de_apresentacao_md() {
    for &(nome, linhas, tipo, texto) in APRESENTACAO {
        let solucao = resolver(linhas);
        assert_eq!(solucao.kind, tipo, "{nome}");
        assert_eq!(format(&solucao), texto, "{nome}");
    }
    println!(
        "PASS: {} casos de APRESENTACAO.md com o texto exato, inclusive \\r\\n",
        APRESENTACAO.len()
    );
}

type EstadoCompleto = (
    Linhas<'static>,
    SolutionKind,
    &'static [usize],
    &'static [&'static str],
    &'static str,
);

const ESTADO_COMPLETO_DO_CSHARP: &[EstadoCompleto] = &[
    (
        &[&["2", "1", "5"], &["1", "-1", "1"]],
        SolutionKind::Unique,
        &[0, 1],
        &["1", "0", "2", "0", "1", "1"],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = 2\r\ny = 1",
    ),
    (
        &[
            &["1", "1", "1", "6"],
            &["2", "-1", "1", "3"],
            &["1", "2", "-1", "2"],
        ],
        SolutionKind::Unique,
        &[0, 1, 2],
        &["1", "0", "0", "1", "0", "1", "0", "2", "0", "0", "1", "3"],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = 1\r\ny = 2\r\nz = 3",
    ),
    (
        &[&["1", "1", "2"], &["2", "2", "4"]],
        SolutionKind::Infinite,
        &[0],
        &["1", "1", "2", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = 2 - t1\r\ny = t1\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["1", "1", "2"], &["2", "2", "5"]],
        SolutionKind::None,
        &[0],
        &["1", "1", "2", "0", "0", "1"],
        "Este sistema n\u{E3}o tem solu\u{E7}\u{E3}o.\r\nAs equa\u{E7}\u{F5}es s\u{E3}o incompat\u{ED}veis.",
    ),
    (
        &[&["0", "1", "2"], &["1", "0", "3"]],
        SolutionKind::Unique,
        &[0, 1],
        &["1", "0", "3", "0", "1", "2"],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = 3\r\ny = 2",
    ),
    (
        &[&["3", "0", "1"], &["0", "2", "1"]],
        SolutionKind::Unique,
        &[0, 1],
        &["1", "0", "1/3", "0", "1", "1/2"],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = 1/3\r\ny = 1/2",
    ),
    (
        &[&["0", "0", "0"], &["0", "0", "0"]],
        SolutionKind::Infinite,
        &[],
        &["0", "0", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = t1\r\ny = t2\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["0", "0", "0", "0"],
            &["0", "0", "0", "0"],
            &["0", "0", "0", "0"],
        ],
        SolutionKind::Infinite,
        &[],
        &["0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = t1\r\ny = t2\r\nz = t3\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["1", "2", "-3", "4"],
            &["2", "4", "-6", "8"],
            &["-1", "-2", "3", "-4"],
        ],
        SolutionKind::Infinite,
        &[0],
        &["1", "2", "-3", "4", "0", "0", "0", "0", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = 4 - 2 \u{B7} t1 + 3 \u{B7} t2\r\ny = t1\r\nz = t2\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["2", "3", "1"], &["4", "6", "2"]],
        SolutionKind::Infinite,
        &[0],
        &["1", "3/2", "1/2", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = 1/2 - 3/2 \u{B7} t1\r\ny = t1\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["1", "1", "0"], &["2", "2", "0"]],
        SolutionKind::Infinite,
        &[0],
        &["1", "1", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = -t1\r\ny = t1\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["1", "-2", "0"], &["0", "0", "0"]],
        SolutionKind::Infinite,
        &[0],
        &["1", "-2", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = 2 \u{B7} t1\r\ny = t1\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["1", "0", "0"], &["0", "1", "-3/4"]],
        SolutionKind::Unique,
        &[0, 1],
        &["1", "0", "0", "0", "1", "-3/4"],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = 0\r\ny = -3/4",
    ),
    (
        &[
            &["1", "1", "1", "3"],
            &["0", "0", "1", "1"],
            &["1", "1", "2", "4"],
        ],
        SolutionKind::Infinite,
        &[0, 2],
        &["1", "1", "0", "2", "0", "0", "1", "1", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = 2 - t1\r\ny = t1\r\nz = 1\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["1", "1", "1", "1"],
            &["1", "1", "1", "2"],
            &["0", "0", "0", "0"],
        ],
        SolutionKind::None,
        &[0],
        &["1", "1", "1", "1", "0", "0", "0", "1", "0", "0", "0", "0"],
        "Este sistema n\u{E3}o tem solu\u{E7}\u{E3}o.\r\nAs equa\u{E7}\u{F5}es s\u{E3}o incompat\u{ED}veis.",
    ),
    (
        &[&["0", "1", "2"], &["0", "2", "4"]],
        SolutionKind::Infinite,
        &[1],
        &["0", "1", "2", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = t1\r\ny = 2\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["0", "0", "1", "5"],
            &["0", "0", "2", "10"],
            &["0", "0", "0", "0"],
        ],
        SolutionKind::Infinite,
        &[2],
        &["0", "0", "1", "5", "0", "0", "0", "0", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = t1\r\ny = t2\r\nz = 5\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["1", "2", "3", "4"],
            &["2", "3", "4", "5"],
            &["3", "4", "5", "6"],
        ],
        SolutionKind::Infinite,
        &[0, 1],
        &["1", "0", "-1", "-2", "0", "1", "2", "3", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = -2 + t1\r\ny = 3 - 2 \u{B7} t1\r\nz = t1\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["0", "0", "0"], &["0", "0", "1"]],
        SolutionKind::None,
        &[],
        &["0", "0", "0", "0", "0", "1"],
        "Este sistema n\u{E3}o tem solu\u{E7}\u{E3}o.\r\nAs equa\u{E7}\u{F5}es s\u{E3}o incompat\u{ED}veis.",
    ),
    (
        &[&["1e50", "1", "2"], &["1", "-1", "0"]],
        SolutionKind::Unique,
        &[0, 1],
        &[
            "1",
            "0",
            "2/100000000000000000000000000000000000000000000000001",
            "0",
            "1",
            "2/100000000000000000000000000000000000000000000000001",
        ],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = 2/100000000000000000000000000000000000000000000000001\r\ny = 2/100000000000000000000000000000000000000000000000001",
    ),
    (
        &[
            &["2", "-4", "6", "-2"],
            &["-1", "2", "-3", "1"],
            &["1/2", "-1", "3/2", "-1/2"],
        ],
        SolutionKind::Infinite,
        &[0],
        &["1", "-2", "3", "-1", "0", "0", "0", "0", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = -1 + 2 \u{B7} t1 - 3 \u{B7} t2\r\ny = t1\r\nz = t2\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["0", "3", "-6", "9"],
            &["0", "0", "0", "0"],
            &["0", "-1", "2", "-3"],
        ],
        SolutionKind::Infinite,
        &[1],
        &["0", "1", "-2", "3", "0", "0", "0", "0", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = t1\r\ny = 3 + 2 \u{B7} t2\r\nz = t2\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[
            &["1", "0", "0", "1"],
            &["0", "0", "1", "2"],
            &["0", "0", "0", "0"],
        ],
        SolutionKind::Infinite,
        &[0, 2],
        &["1", "0", "0", "1", "0", "0", "1", "2", "0", "0", "0", "0"],
        "Infinitas solu\u{E7}\u{F5}es\r\n\r\nx = 1\r\ny = t1\r\nz = 2\r\n\r\nt1, t2, ... podem ser quaisquer n\u{FA}meros reais.",
    ),
    (
        &[&["0,5", "-2,25", "1e-3"], &["-1/3", "\u{2212}7", "2"]],
        SolutionKind::Unique,
        &[0, 1],
        &["1", "0", "-4493/4250", "0", "1", "-3001/12750"],
        "Solu\u{E7}\u{E3}o \u{FA}nica\r\n\r\nx = -4493/4250\r\ny = -3001/12750",
    ),
];

#[test]
fn estado_completo_identico_ao_csharp() {
    for &(linhas, tipo, pivos, reduzida, texto) in ESTADO_COMPLETO_DO_CSHARP {
        let solucao = resolver(linhas);
        let obtida: Vec<String> = solucao
            .reduced
            .iter()
            .flatten()
            .map(Rational::to_string)
            .collect();
        assert_eq!(solucao.kind, tipo, "classificação de {linhas:?}");
        assert_eq!(solucao.pivot_columns, pivos, "colunas pivô de {linhas:?}");
        assert_eq!(obtida, reduzida, "matriz reduzida de {linhas:?}");
        assert_eq!(format(&solucao), texto, "texto de {linhas:?}");
    }
    println!(
        "PASS: {} sistemas com classificação, pivôs, matriz reduzida e texto idênticos ao C#",
        ESTADO_COMPLETO_DO_CSHARP.len()
    );
}

#[test]
fn texto_termina_sem_quebra_e_usa_crlf() {
    assert_eq!(VARIABLE_NAMES, ["x", "y", "z"]);
    for &(_, linhas, _, _) in APRESENTACAO {
        let texto = format(&resolver(linhas));
        assert!(!texto.ends_with(char::is_whitespace), "{texto:?}");
        assert!(
            !texto.replace("\r\n", "").contains(['\r', '\n']),
            "{texto:?}"
        );
    }
}
