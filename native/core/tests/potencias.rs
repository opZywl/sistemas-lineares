mod comum;

use comum::RandomDotNet;
use sistemas_lineares_core::power::{self, MAX_POWER_DIGITS};
use sistemas_lineares_core::{Entry, ParseError, Rational, preview_equation};

const VETOR: &str = include_str!("../../../tests/potencias.txt");

fn decodificar(texto: &str) -> String {
    let mut saida = String::new();
    let mut caracteres = texto.chars();
    while let Some(caractere) = caracteres.next() {
        if caractere != '\\' {
            saida.push(caractere);
            continue;
        }
        match caracteres.next() {
            Some('\\') => saida.push('\\'),
            Some('u') => {
                assert_eq!(
                    caracteres.next(),
                    Some('{'),
                    "escape malformado em {texto:?}"
                );
                let hexadecimal: String = caracteres.by_ref().take_while(|c| *c != '}').collect();
                let codigo = u32::from_str_radix(&hexadecimal, 16).expect("hexadecimal");
                saida.push(char::from_u32(codigo).expect("caractere Unicode"));
            }
            outro => panic!("escape inválido {outro:?} em {texto:?}"),
        }
    }
    saida
}

fn interpretar(entrada: &str) -> String {
    match Rational::parse(entrada) {
        Ok(valor) => format!("OK:{valor}"),
        Err(erro) => format!("ERRO:{}", erro.message()),
    }
}

fn valor(entrada: &str) -> Rational {
    Rational::parse(entrada).unwrap_or_else(|erro| panic!("{entrada:?}: {erro}"))
}

fn sobrescrito(expoente: i64) -> String {
    const DIGITOS: [char; 10] = [
        '\u{2070}', '\u{B9}', '\u{B2}', '\u{B3}', '\u{2074}', '\u{2075}', '\u{2076}', '\u{2077}',
        '\u{2078}', '\u{2079}',
    ];
    let mut texto = String::new();
    if expoente < 0 {
        texto.push('\u{207B}');
    }
    for digito in expoente.unsigned_abs().to_string().chars() {
        texto.push(DIGITOS[digito.to_digit(10).unwrap_or(0) as usize]);
    }
    texto
}

#[test]
fn vetor_compartilhado_com_o_javascript() {
    let mut casos = 0usize;
    let mut divergencias = Vec::new();
    for linha in VETOR.lines() {
        if linha.is_empty() {
            continue;
        }
        let (entrada, esperado) = linha
            .split_once('\t')
            .unwrap_or_else(|| panic!("linha sem tabulação: {linha:?}"));
        casos += 1;
        let entrada = decodificar(entrada);
        let obtido = interpretar(&entrada);
        if obtido != esperado {
            divergencias.push(format!(
                "{entrada:?}\n  esperado {}\n  obtido   {}",
                esperado.chars().take(120).collect::<String>(),
                obtido.chars().take(120).collect::<String>()
            ));
        }
    }
    assert!(casos >= 150, "vetor com poucos casos: {casos}");
    assert!(
        divergencias.is_empty(),
        "{} divergências:\n{}",
        divergencias.len(),
        divergencias.join("\n")
    );
    println!("PASS: {casos} entradas com potência idênticas ao vetor compartilhado");
}

#[test]
fn mensagens_novas() {
    for (erro, mensagem) in [
        (
            ParseError::PowerExponent,
            "Use um expoente inteiro de -1000 a 1000, como 2^3 ou 10^-2.",
        ),
        (
            ParseError::PowerChain,
            "Use parênteses para potência de potência, como (2^3)^2.",
        ),
        (
            ParseError::PowerTooLarge,
            "Use números menores: com potências, o limite é de 300 algarismos.",
        ),
        (ParseError::ZeroToZero, "0 elevado a 0 é indeterminado."),
        (
            ParseError::ZeroNegativePower,
            "0 não pode ser elevado a expoente negativo.",
        ),
        (ParseError::Parenthesis, "Confira os parênteses."),
        (
            ParseError::Product,
            "Faça a multiplicação antes: para 1,5×10³, digite 1,5e3 ou 1500.",
        ),
    ] {
        assert_eq!(erro.message(), mensagem);
    }
    assert_eq!(MAX_POWER_DIGITS, 300);
}

#[test]
fn entradas_sem_potencia_seguem_o_caminho_antigo() {
    for texto in [
        "1", "-2,5", "3/4", "1e-6", "1 / 2", "x", "1+2", "--2", "1/2/3", "\u{85}1",
    ] {
        assert!(
            !power::uses_extension(texto),
            "{texto:?} não deveria usar a extensão"
        );
    }
    for texto in [
        "2^3",
        "2\u{B2}",
        "(1)",
        "2**3",
        "2*3",
        "10\u{207B}\u{B9}",
        "2\u{D7}3",
        "2\u{B7}3",
        "2\u{22C5}3",
    ] {
        assert!(power::uses_extension(texto), "{texto:?}");
    }
}

#[test]
fn propriedades_das_potencias() {
    let mut random = RandomDotNet::new(185964);
    let mut verificacoes = 0usize;
    for _ in 0..4000 {
        let numerador = random.next(-12, 13);
        let denominador = random.next(1, 9);
        if numerador == 0 {
            continue;
        }
        let base = format!("({numerador}/{denominador})");
        let m = i64::from(random.next(-7, 8));
        let n = i64::from(random.next(-7, 8));
        let produto = &valor(&format!("{base}^{m}")) * &valor(&format!("{base}^{n}"));
        assert_eq!(
            produto,
            valor(&format!("{base}^{}", m + n)),
            "{base} m={m} n={n}"
        );
        assert_eq!(
            valor(&format!("({base}^{m})^{n}")),
            valor(&format!("{base}^{}", m * n)),
            "{base} m={m} n={n}"
        );
        assert_eq!(
            valor(&format!("{base}{}", sobrescrito(m))),
            valor(&format!("{base}^{m}")),
            "{base} sobrescrito {m}"
        );
        assert_eq!(
            valor(&format!("{base}**({m})")),
            valor(&format!("{base}^{m}")),
            "{base} ** {m}"
        );
        assert_eq!(
            valor(&format!("-{base}^2")),
            -&valor(&format!("{base}^2")),
            "{base} sinal depois da potência"
        );
        let esperado = {
            let mut acumulado = Rational::one();
            let fator = valor(&base);
            for _ in 0..m.unsigned_abs() {
                acumulado = &acumulado * &fator;
            }
            if m < 0 {
                &Rational::one() / &acumulado
            } else {
                acumulado
            }
        };
        assert_eq!(
            valor(&format!("{base}^{m}")),
            esperado,
            "{base}^{m} pela definição"
        );
        verificacoes += 5;
    }
    println!("PASS: {verificacoes} propriedades de potência conferidas");
}

#[test]
fn limite_de_algarismos_exato() {
    for (base, expoente, cabe) in [
        (10u32, 299u32, true),
        (10, 300, false),
        (2, 996, true),
        (2, 997, false),
        (3, 628, true),
        (3, 629, false),
        (7, 354, true),
        (7, 355, false),
        (99, 150, true),
        (99, 151, false),
        (101, 149, true),
        (2, 1000, false),
    ] {
        let resultado = Rational::parse(&format!("{base}^{expoente}"));
        let algarismos = num_bigint::BigInt::from(base)
            .pow(expoente)
            .to_string()
            .len();
        assert_eq!(
            resultado.is_ok(),
            algarismos <= MAX_POWER_DIGITS,
            "{base}^{expoente} tem {algarismos} algarismos"
        );
        assert_eq!(resultado.is_ok(), cabe, "{base}^{expoente}");
    }
    for (entrada, cabe) in [
        ("(1e1000/1e-1000)", true),
        ("(1e1000/1e-1000)^0", true),
        ("(1e1000)^1", false),
        ("(10^200)/(10^-99)", true),
        ("(10^200)/(10^-100)", false),
        ("((1e1000))", true),
    ] {
        assert_eq!(Rational::parse(entrada).is_ok(), cabe, "{entrada}");
    }
}

#[test]
fn pior_caso_de_potencias_continua_rapido() {
    let entradas = [
        "(101/137)^99",
        "(173/211)^99",
        "(251/283)^99",
        "(337/379)^99",
        "(421/461)^99",
        "(503/563)^99",
        "(601/607)^99",
        "(701/709)^99",
        "(809/811)^99",
        "(907/911)^99",
        "(953/967)^99",
        "(977/983)^99",
    ];
    let inicio = std::time::Instant::now();
    let valores: Vec<Rational> = entradas.iter().map(|entrada| valor(entrada)).collect();
    let matriz: Vec<Vec<Rational>> = valores.chunks(4).map(<[Rational]>::to_vec).collect();
    let solucao = sistemas_lineares_core::solve(&matriz).expect("resolve");
    for metodo in sistemas_lineares_core::Method::ALL {
        let passos = sistemas_lineares_core::walkthrough(&matriz, metodo).expect("passo a passo");
        assert!(!passos.steps.is_empty());
    }
    let decorrido = inicio.elapsed();
    assert!(
        decorrido.as_secs_f64() < 5.0,
        "sistema 3x3 com potências densas levou {decorrido:?}"
    );
    let _ = solucao;
    let parse = std::time::Instant::now();
    for _ in 0..100 {
        let _ = Rational::parse(
            "(((101/137)^99/(173/211)^99)/((251/283)^99/(337/379)^99))/((421/461)^99/(503/563)^99)",
        );
    }
    assert!(
        parse.elapsed().as_secs_f64() < 5.0,
        "100 leituras aninhadas levaram {:?}",
        parse.elapsed()
    );
}

#[test]
fn previa_do_sistema() {
    let v = |texto: &str| Entry::Value(valor(texto));
    let casos: Vec<(Vec<Entry>, &str)> = vec![
        (vec![v("2"), Entry::Missing, v("5")], "2x + ?y = 5"),
        (vec![Entry::Missing, v("-1"), v("1")], "?x \u{2212} y = 1"),
        (vec![v("0"), Entry::Missing, Entry::Missing], "?y = ?"),
        (
            vec![Entry::Missing, Entry::Missing, Entry::Missing],
            "?x + ?y = ?",
        ),
        (vec![v("3/4"), v("-2"), v("0")], "(3/4)x \u{2212} 2y = 0"),
        (vec![v("0"), v("0"), v("-7")], "0 = \u{2212}7"),
        (vec![Entry::Invalid, v("1"), Entry::Invalid], "?x + y = ?"),
        (
            vec![v("-1"), Entry::Missing, v("1"), v("2^3")],
            "\u{2212}x + ?y + z = 8",
        ),
        (
            vec![v("1"), v("-1/2"), Entry::Missing, v("-1,5")],
            "x \u{2212} (1/2)y + ?z = \u{2212}3/2",
        ),
        (
            vec![
                Entry::Missing,
                Entry::Missing,
                Entry::Missing,
                Entry::Missing,
            ],
            "?x + ?y + ?z = ?",
        ),
        (vec![v("0"), v("0"), v("0"), Entry::Missing], "0 = ?"),
    ];
    for (linha, esperado) in casos {
        assert_eq!(preview_equation(&linha), esperado);
    }
}

#[test]
fn digitando_e_orcamento_ao_vivo() {
    use sistemas_lineares_core::{LIVE_BUDGET_BITS, looks_incomplete, within_live_budget};
    for texto in [
        "-",
        "+",
        "1/",
        "2^",
        "2^-",
        "(",
        "(1",
        "((2)",
        "1e",
        "1E-",
        "2**",
        "2*",
        ".",
        ",",
        "-.",
        "2\u{207B}",
        "1/(2",
        " 3/ ",
        "\u{2212}",
    ] {
        assert!(looks_incomplete(texto), "{texto:?} parece em digitação");
    }
    for texto in [
        "", " ", "2", "1/2", "2^3", "x", "2x", "1/0", "0^0", "(2)", "1e5", "abc",
    ] {
        assert!(!looks_incomplete(texto), "{texto:?} não está em digitação");
    }
    let pequeno: Vec<Vec<Rational>> = vec![vec![valor("2"), valor("1"), valor("5")]; 2];
    assert!(within_live_budget(&pequeno));
    let grande: Vec<Vec<Rational>> = vec![vec![valor("1e1000"), valor("1"), valor("5")]; 3];
    assert!(!within_live_budget(&grande));
    assert_eq!(LIVE_BUDGET_BITS, 8000);
}

#[test]
fn previa_abrevia_numeros_longos() {
    use sistemas_lineares_core::steps::{PREVIEW_DIGIT_LIMIT, abbreviate};
    assert_eq!(PREVIEW_DIGIT_LIMIT, 12);
    assert_eq!(abbreviate("123456789012"), "123456789012");
    assert_eq!(abbreviate("1234567890123"), "1234\u{2026}0123");
    assert_eq!(abbreviate("12345678/99999999"), "12345678/99999999");
    assert_eq!(
        abbreviate("\u{2212}1234567890123456/7654321098765432"),
        "\u{2212}1234\u{2026}3456/7654\u{2026}5432"
    );
    let v = |texto: &str| Entry::Value(valor(texto));
    assert_eq!(
        preview_equation(&[v("1e-30"), v("10^20"), v("-1e40")]),
        "(1/1000\u{2026}0000)x + 1000\u{2026}0000y = \u{2212}1000\u{2026}0000"
    );
    assert_eq!(
        preview_equation(&[v("12345677/99999999"), v("2"), v("-1234567/7654321")]),
        "(12345677/99999999)x + 2y = \u{2212}1234567/7654321"
    );
}

#[test]
fn corpus_diferencial_para_o_javascript() {
    let alfabeto: Vec<char> = "0123456789.,eE+-\u{2212}/^*()\u{D7}\u{B7}\u{2070}\u{B9}\u{B2}\u{B3}\u{2074}\u{2075}\u{2076}\u{2077}\u{2078}\u{2079}\u{207A}\u{207B}x \u{A0}\u{85}\u{FEFF}\t"
        .chars()
        .collect();
    let mut random = RandomDotNet::new(410);
    let mut linhas = String::new();
    let mut aceitas = 0usize;
    let total = 200_000;
    for _ in 0..total {
        let tamanho = random.next(1, 15) as usize;
        let entrada: String = (0..tamanho)
            .map(|_| alfabeto[random.next(0, alfabeto.len() as i32) as usize])
            .collect();
        let (status, resultado) = match Rational::parse(&entrada) {
            Ok(valor) => {
                aceitas += 1;
                ("OK", valor.to_string())
            }
            Err(erro) => ("ERRO", erro.message().to_string()),
        };
        linhas.push_str(&comum::codificar_base64(entrada.as_bytes()));
        linhas.push('\t');
        linhas.push_str(status);
        linhas.push('\t');
        linhas.push_str(&resultado);
        linhas.push('\n');
    }
    assert!(
        aceitas > 1_000,
        "corpus com poucas entradas válidas: {aceitas}"
    );
    if let Some(caminho) = std::env::var_os("SL_POTENCIAS_SAIDA") {
        std::fs::write(&caminho, &linhas).expect("gravar SL_POTENCIAS_SAIDA");
        println!(
            "corpus de potências gravado: {} ({total} entradas, {aceitas} aceitas)",
            caminho.to_string_lossy()
        );
    }
}
