mod comum;

use std::fs;

use comum::decodificar_base64;
use sistemas_lineares_core::{MAX_EXPONENT, MAX_INPUT_LENGTH, ParseError, Rational};

const CASOS_DE_BORDA: &[(&str, Result<&str, ParseError>)] = &[
    ("", Err(ParseError::Empty)),
    (" ", Err(ParseError::Empty)),
    ("\t", Err(ParseError::Empty)),
    ("\u{A0}", Err(ParseError::Empty)),
    ("\u{2003}", Err(ParseError::Empty)),
    ("\u{3000}", Err(ParseError::Empty)),
    ("  \t\u{A0}\u{3000} ", Err(ParseError::Empty)),
    ("\r\n", Err(ParseError::Empty)),
    ("\n", Err(ParseError::Empty)),
    ("\u{85}", Err(ParseError::Empty)),
    ("\u{2028}", Err(ParseError::Empty)),
    ("0", Ok("0")),
    ("-0", Ok("0")),
    ("+0", Ok("0")),
    ("\u{2212}0", Ok("0")),
    ("00", Ok("0")),
    ("007", Ok("7")),
    ("-007", Ok("-7")),
    ("+007", Ok("7")),
    ("1", Ok("1")),
    ("-1", Ok("-1")),
    ("+1", Ok("1")),
    ("\u{2212}1", Ok("-1")),
    ("12345678901234567890", Ok("12345678901234567890")),
    ("-98765432109876543210", Ok("-98765432109876543210")),
    (".", Err(ParseError::Invalid)),
    ("+.", Err(ParseError::Invalid)),
    ("-.", Err(ParseError::Invalid)),
    ("\u{2212}.", Err(ParseError::Invalid)),
    (".5", Ok("1/2")),
    ("+.5", Ok("1/2")),
    ("-.5", Ok("-1/2")),
    ("\u{2212}.5", Ok("-1/2")),
    ("5.", Ok("5")),
    ("-5.", Ok("-5")),
    ("5.0", Ok("5")),
    ("5.50", Ok("11/2")),
    ("0.5", Ok("1/2")),
    (",5", Ok("1/2")),
    ("5,", Ok("5")),
    ("1,5", Ok("3/2")),
    ("-1,5", Ok("-3/2")),
    ("\u{2212}1,5", Ok("-3/2")),
    ("1.2.3", Err(ParseError::Invalid)),
    ("1,2.3", Err(ParseError::Invalid)),
    ("1.2,3", Err(ParseError::Invalid)),
    ("1,2,3", Err(ParseError::Invalid)),
    ("1..2", Err(ParseError::Invalid)),
    ("..5", Err(ParseError::Invalid)),
    ("5..", Err(ParseError::Invalid)),
    (".,5", Err(ParseError::Invalid)),
    ("0.000", Ok("0")),
    ("-0.000", Ok("0")),
    ("000.000100", Ok("1/10000")),
    ("-000.000100", Ok("-1/10000")),
    (
        "3.14159265358979323846264338327950288419716939937510",
        Ok(
            "31415926535897932384626433832795028841971693993751/10000000000000000000000000000000000000000000000000",
        ),
    ),
    ("1/2", Ok("1/2")),
    ("-1/2", Ok("-1/2")),
    ("1/-2", Ok("-1/2")),
    ("-1/-2", Ok("1/2")),
    ("+1/+2", Ok("1/2")),
    ("2/4", Ok("1/2")),
    ("6/-4", Ok("-3/2")),
    ("0/5", Ok("0")),
    ("0/-5", Ok("0")),
    ("-0/5", Ok("0")),
    ("1/0", Err(ParseError::ZeroDenominator)),
    ("0/0", Err(ParseError::ZeroDenominator)),
    ("-1/0", Err(ParseError::ZeroDenominator)),
    ("1/0.0", Err(ParseError::ZeroDenominator)),
    ("1/-0", Err(ParseError::ZeroDenominator)),
    ("1/0e5", Err(ParseError::ZeroDenominator)),
    ("1/0.000e-3", Err(ParseError::ZeroDenominator)),
    ("0/0e1001", Err(ParseError::Exponent)),
    ("1/2/3", Err(ParseError::Invalid)),
    ("1/2/0", Err(ParseError::Invalid)),
    ("/", Err(ParseError::Invalid)),
    ("1/", Err(ParseError::Invalid)),
    ("/2", Err(ParseError::Invalid)),
    ("//", Err(ParseError::Invalid)),
    ("1//2", Err(ParseError::Invalid)),
    (" 1/2 ", Ok("1/2")),
    ("1 / 2", Ok("1/2")),
    ("1/ 2", Ok("1/2")),
    ("1 /2", Ok("1/2")),
    ("1\t/\t2", Ok("1/2")),
    ("1\u{3000}/\u{3000}2", Ok("1/2")),
    ("1\u{A0}/\u{A0}2", Ok("1/2")),
    (" / ", Err(ParseError::Invalid)),
    ("2,5/0,5", Ok("5")),
    ("0.5/0.25", Ok("2")),
    ("1.5/-0.5", Ok("-3")),
    ("-.5/.25", Ok("-2")),
    ("5./2.", Ok("5/2")),
    ("1e2/1e-2", Ok("10000")),
    ("1e100/1e100", Ok("1")),
    ("1e-1000/1e-1000", Ok("1")),
    ("1e1000/1e1000", Ok("1")),
    ("3/1e10", Ok("3/10000000000")),
    ("\u{2212}3/\u{2212}4", Ok("3/4")),
    ("\u{2212}3/-4", Ok("3/4")),
    ("1/2e", Err(ParseError::Exponent)),
    ("1/e2", Err(ParseError::Invalid)),
    ("1/2 3", Err(ParseError::Invalid)),
    ("x/0", Err(ParseError::Invalid)),
    ("1/x", Err(ParseError::Invalid)),
    ("1e1001/0", Err(ParseError::Exponent)),
    ("1/1e1001", Err(ParseError::Exponent)),
    ("0/1e1001", Err(ParseError::Exponent)),
    ("1e5/0", Err(ParseError::ZeroDenominator)),
    ("1/0e1001", Err(ParseError::Exponent)),
    ("1e0", Ok("1")),
    ("1e+0", Ok("1")),
    ("1e-0", Ok("1")),
    ("1e5", Ok("100000")),
    ("1E5", Ok("100000")),
    ("1e+5", Ok("100000")),
    ("1e-5", Ok("1/100000")),
    ("1e05", Ok("100000")),
    ("1e-05", Ok("1/100000")),
    ("1e0005", Ok("100000")),
    ("1e+00005", Ok("100000")),
    ("2.5e3", Ok("2500")),
    ("\u{2212}2,5E+3", Ok("-2500")),
    ("-2.5e-3", Ok("-1/400")),
    (".5e1", Ok("5")),
    ("5.e1", Ok("50")),
    ("0e5", Ok("0")),
    ("0e-1000", Ok("0")),
    ("-0e1000", Ok("0")),
    ("1e-20", Ok("1/100000000000000000000")),
    ("1e1001", Err(ParseError::Exponent)),
    ("1e-1001", Err(ParseError::Exponent)),
    ("1e1002", Err(ParseError::Exponent)),
    ("1e00001001", Err(ParseError::Exponent)),
    ("1e2147483647", Err(ParseError::Exponent)),
    ("1e2147483648", Err(ParseError::Exponent)),
    ("1e-2147483648", Err(ParseError::Exponent)),
    ("1e-2147483649", Err(ParseError::Exponent)),
    ("1e4294967296", Err(ParseError::Exponent)),
    ("1e99999999999999999999", Err(ParseError::Exponent)),
    ("1e", Err(ParseError::Exponent)),
    ("1e+", Err(ParseError::Exponent)),
    ("1e-", Err(ParseError::Exponent)),
    ("1E", Err(ParseError::Exponent)),
    ("1E-", Err(ParseError::Exponent)),
    ("e", Err(ParseError::Exponent)),
    ("E", Err(ParseError::Exponent)),
    ("e5", Err(ParseError::Invalid)),
    ("E5", Err(ParseError::Invalid)),
    ("-e5", Err(ParseError::Invalid)),
    ("+e5", Err(ParseError::Invalid)),
    (".e5", Err(ParseError::Invalid)),
    ("e-5", Err(ParseError::Invalid)),
    ("1e5e3", Err(ParseError::Exponent)),
    ("1e1e1", Err(ParseError::Exponent)),
    ("1ee5", Err(ParseError::Exponent)),
    ("1eE5", Err(ParseError::Exponent)),
    ("1Ee5", Err(ParseError::Exponent)),
    ("1e5.0", Err(ParseError::Exponent)),
    ("1e5,0", Err(ParseError::Exponent)),
    ("1e.5", Err(ParseError::Exponent)),
    ("1e 5", Err(ParseError::Exponent)),
    ("1e+ 5", Err(ParseError::Exponent)),
    ("1e--5", Err(ParseError::Exponent)),
    ("1e+-5", Err(ParseError::Exponent)),
    ("1e-+5", Err(ParseError::Exponent)),
    ("1e++5", Err(ParseError::Exponent)),
    ("1e\u{2212}5", Ok("1/100000")),
    ("1e\u{2212}\u{2212}5", Err(ParseError::Exponent)),
    ("1 e5", Err(ParseError::Invalid)),
    ("1e5 ", Ok("100000")),
    (" 1e5", Ok("100000")),
    ("1e5/", Err(ParseError::Invalid)),
    ("1e/5", Err(ParseError::Exponent)),
    ("1e\u{663}", Err(ParseError::Exponent)),
    ("1e\u{FF15}", Err(ParseError::Exponent)),
    ("1e5\u{663}", Err(ParseError::Exponent)),
    ("1e5\u{0}", Ok("100000")),
    ("1e5\u{0}\u{0}", Ok("100000")),
    ("1e-5\u{0}", Ok("1/100000")),
    ("1e+5\u{0}", Ok("100000")),
    ("1e0\u{0}", Ok("1")),
    ("1e1001\u{0}", Err(ParseError::Exponent)),
    ("1e\u{0}", Err(ParseError::Exponent)),
    ("1e\u{0}5", Err(ParseError::Exponent)),
    ("1e+\u{0}", Err(ParseError::Exponent)),
    ("1e-\u{0}", Err(ParseError::Exponent)),
    ("1e5\u{0}x", Err(ParseError::Exponent)),
    ("1e5\u{0} ", Ok("100000")),
    ("1e5 \u{0}", Err(ParseError::Exponent)),
    ("1e5\u{0}\u{A0}", Ok("100000")),
    ("\u{0}1e5", Err(ParseError::Invalid)),
    ("1\u{0}", Err(ParseError::Invalid)),
    ("\u{0}1", Err(ParseError::Invalid)),
    ("1\u{0}e5", Err(ParseError::Invalid)),
    ("1e5\u{0}/2", Ok("50000")),
    ("2/1e5\u{0}", Ok("1/50000")),
    ("1e5\u{0}/0", Err(ParseError::ZeroDenominator)),
    ("1e5\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}", Ok("100000")),
    ("1e\u{A0}5", Err(ParseError::Exponent)),
    ("1\u{A0}e5", Err(ParseError::Invalid)),
    ("--2", Err(ParseError::Invalid)),
    ("++2", Err(ParseError::Invalid)),
    ("+-2", Err(ParseError::Invalid)),
    ("-+2", Err(ParseError::Invalid)),
    ("\u{2212}\u{2212}2", Err(ParseError::Invalid)),
    ("-\u{2212}2", Err(ParseError::Invalid)),
    ("- 2", Err(ParseError::Invalid)),
    ("+ 2", Err(ParseError::Invalid)),
    ("\u{2212} 2", Err(ParseError::Invalid)),
    ("-\u{A0}2", Err(ParseError::Invalid)),
    ("\u{A0}-\u{A0}1", Err(ParseError::Invalid)),
    ("2-", Err(ParseError::Invalid)),
    ("2+", Err(ParseError::Invalid)),
    ("2\u{2212}", Err(ParseError::Invalid)),
    ("1-2", Err(ParseError::Invalid)),
    ("1+2", Err(ParseError::Invalid)),
    ("-", Err(ParseError::Invalid)),
    ("+", Err(ParseError::Invalid)),
    ("\u{2212}", Err(ParseError::Invalid)),
    ("-/-", Err(ParseError::Invalid)),
    ("+/+", Err(ParseError::Invalid)),
    ("x", Err(ParseError::Invalid)),
    ("NaN", Err(ParseError::Invalid)),
    ("nan", Err(ParseError::Invalid)),
    ("Infinity", Err(ParseError::Invalid)),
    ("-Infinity", Err(ParseError::Invalid)),
    ("\u{221E}", Err(ParseError::Invalid)),
    ("1x", Err(ParseError::Invalid)),
    ("x1", Err(ParseError::Invalid)),
    ("0x10", Err(ParseError::Invalid)),
    ("1_000", Err(ParseError::Invalid)),
    ("1 000", Err(ParseError::Invalid)),
    ("1.000,5", Err(ParseError::Invalid)),
    ("1'000", Err(ParseError::Invalid)),
    ("(1)", Err(ParseError::Invalid)),
    ("$1", Err(ParseError::Invalid)),
    ("1%", Err(ParseError::Invalid)),
    ("abc", Err(ParseError::Invalid)),
    ("true", Err(ParseError::Exponent)),
    ("\u{661}", Err(ParseError::Invalid)),
    ("\u{661}\u{662}\u{663}", Err(ParseError::Invalid)),
    ("\u{6F1}", Err(ParseError::Invalid)),
    ("\u{967}", Err(ParseError::Invalid)),
    ("\u{FF11}\u{FF12}\u{FF13}", Err(ParseError::Invalid)),
    ("\u{1D7CE}", Err(ParseError::Invalid)),
    ("\u{1D7CF}\u{1D7D0}", Err(ParseError::Invalid)),
    ("1\u{1D7CE}", Err(ParseError::Invalid)),
    ("\u{BD}", Err(ParseError::Invalid)),
    ("\u{B2}", Err(ParseError::Invalid)),
    ("1\u{B2}", Err(ParseError::Invalid)),
    ("\u{221A}2", Err(ParseError::Invalid)),
    ("\u{661}/\u{662}", Err(ParseError::Invalid)),
    ("1\u{200B}", Err(ParseError::Invalid)),
    ("\u{200B}1", Err(ParseError::Invalid)),
    ("\u{FEFF}1", Err(ParseError::Invalid)),
    ("1\u{FEFF}", Err(ParseError::Invalid)),
    ("\u{180E}1", Err(ParseError::Invalid)),
    ("1\u{180E}", Err(ParseError::Invalid)),
    ("\u{85}1\u{85}", Ok("1")),
    ("1\u{2028}", Ok("1")),
    ("\u{2029}1", Ok("1")),
    ("\u{1680}1\u{1680}", Ok("1")),
    ("\u{202F}1\u{205F}", Ok("1")),
    ("\u{1C}1", Err(ParseError::Invalid)),
    ("1\u{1F}", Err(ParseError::Invalid)),
    ("\u{B}1\u{C}", Ok("1")),
    ("\u{2000}\u{200A}1\u{2007}\u{2008}", Ok("1")),
    ("\r\n1\r\n", Ok("1")),
    ("1\n2", Err(ParseError::Invalid)),
    ("1\r", Ok("1")),
];

fn interpretar(entrada: &str) -> Result<String, ParseError> {
    Rational::parse(entrada).map(|valor| valor.to_string())
}

#[test]
fn casos_de_borda_identicos_ao_csharp() {
    let divergencias: Vec<String> = CASOS_DE_BORDA
        .iter()
        .filter_map(|&(entrada, esperado)| {
            let obtido = interpretar(entrada);
            (obtido.as_deref().map_err(|erro| *erro) != esperado)
                .then(|| format!("{entrada:?}: C# {esperado:?}, Rust {obtido:?}"))
        })
        .collect();
    assert!(
        divergencias.is_empty(),
        "{} divergências:\n{}",
        divergencias.len(),
        divergencias.join("\n")
    );
    println!(
        "PASS: {} casos de borda idênticos ao C#",
        CASOS_DE_BORDA.len()
    );
}

#[test]
fn limites_de_comprimento_contados_em_utf16() {
    assert_eq!(MAX_INPUT_LENGTH, 100);
    assert_eq!(MAX_EXPONENT, 1000);
    let repetir = |texto: &str, vezes: usize| texto.repeat(vezes);
    let um = |vezes: usize| repetir("1", vezes);
    let zeros = |vezes: usize| repetir("0", vezes);
    let casos: Vec<(String, Result<String, ParseError>)> = vec![
        (um(99), Ok(um(99))),
        (um(100), Ok(um(100))),
        (um(101), Err(ParseError::TooLong)),
        (format!("-{}", um(99)), Ok(format!("-{}", um(99)))),
        (format!("-{}", um(100)), Err(ParseError::TooLong)),
        (format!("\u{2212}{}", um(99)), Ok(format!("-{}", um(99)))),
        (format!("\u{2212}{}", um(100)), Err(ParseError::TooLong)),
        (format!(" {} ", um(100)), Ok(um(100))),
        (format!("\u{3000}\u{3000}{}\t", um(100)), Ok(um(100))),
        (format!("{}\u{1D7CE}", um(98)), Err(ParseError::Invalid)),
        (format!("{}\u{1D7CE}", um(99)), Err(ParseError::TooLong)),
        (
            format!("{}/{}", um(50), um(49)),
            Ok(format!("{}/{}", um(50), um(49))),
        ),
        (format!("{}/{}", um(50), um(50)), Err(ParseError::TooLong)),
        (
            format!("{} / {}", um(49), um(48)),
            Ok(format!("{}/{}", um(49), um(48))),
        ),
        (format!("{} / {}", um(49), um(49)), Err(ParseError::TooLong)),
        (format!("0.{}1", zeros(97)), Ok(format!("1/1{}", zeros(98)))),
        (format!("0.{}1", zeros(98)), Err(ParseError::TooLong)),
        (format!("1e{}", zeros(98)), Ok("1".to_string())),
        (format!("1e{}", zeros(99)), Err(ParseError::TooLong)),
        (
            format!("1e{}1000", zeros(94)),
            Ok(format!("1{}", zeros(1000))),
        ),
        (format!("1e{}1001", zeros(94)), Err(ParseError::Exponent)),
        (format!("{}/0", um(100)), Err(ParseError::TooLong)),
        (repetir("/", 101), Err(ParseError::TooLong)),
        (repetir("/", 100), Err(ParseError::Invalid)),
        (repetir("9", 100), Ok(repetir("9", 100))),
        (
            format!("-{}", repetir("9", 99)),
            Ok(format!("-{}", repetir("9", 99))),
        ),
        (
            format!("{}7{}", repetir(" ", 200), repetir(" ", 200)),
            Ok("7".to_string()),
        ),
        ("1e999".to_string(), Ok(format!("1{}", zeros(999)))),
        ("1e1000".to_string(), Ok(format!("1{}", zeros(1000)))),
        ("-1e1000".to_string(), Ok(format!("-1{}", zeros(1000)))),
        ("1e-999".to_string(), Ok(format!("1/1{}", zeros(999)))),
        ("1e-1000".to_string(), Ok(format!("1/1{}", zeros(1000)))),
        (
            "1e1000/1e-1000".to_string(),
            Ok(format!("1{}", zeros(2000))),
        ),
        (
            "1e-1000/1e1000".to_string(),
            Ok(format!("1/1{}", zeros(2000))),
        ),
        ("7e1000/3".to_string(), Ok(format!("7{}/3", zeros(1000)))),
        ("3/7e-1000".to_string(), Ok(format!("3{}/7", zeros(1000)))),
        (
            format!("{}e-1000", repetir("9", 100)),
            Err(ParseError::TooLong),
        ),
    ];
    for (entrada, esperado) in &casos {
        assert_eq!(
            &interpretar(entrada),
            esperado,
            "entrada com {} unidades UTF-16",
            entrada.encode_utf16().count()
        );
    }
    println!(
        "PASS: {} limites de comprimento e expoente idênticos ao C#",
        casos.len()
    );
}

#[test]
fn diferencial_completo_contra_csharp() {
    let Some(caminho) = std::env::var_os("SL_PARSER_CSHARP") else {
        eprintln!("SL_PARSER_CSHARP não definido; diferencial completo do parser ignorado");
        return;
    };
    let conteudo = fs::read_to_string(&caminho).expect("saída do helper C#");
    let mut comparadas = 0usize;
    let mut aceitas = 0usize;
    let mut divergencias = Vec::new();
    for linha in conteudo.lines() {
        let mut campos = linha.splitn(3, '\t');
        let (Some(codificada), Some(status), Some(resultado)) =
            (campos.next(), campos.next(), campos.next())
        else {
            panic!("linha malformada na saída do C#: {linha:?}");
        };
        let entrada =
            String::from_utf8(decodificar_base64(codificada)).expect("corpus em UTF-8 válido");
        let (status_rust, resultado_rust) = match Rational::parse(&entrada) {
            Ok(valor) => ("OK", valor.to_string()),
            Err(erro) => ("ERRO", erro.message().to_string()),
        };
        comparadas += 1;
        if status_rust == "OK" {
            aceitas += 1;
        }
        if status_rust != status || resultado_rust != resultado {
            divergencias.push(format!(
                "{entrada:?}: C# {status} {resultado:?} | Rust {status_rust} {resultado_rust:?}"
            ));
        }
    }
    let primeiras: Vec<&str> = divergencias.iter().take(30).map(String::as_str).collect();
    assert!(
        divergencias.is_empty(),
        "{} divergências em {comparadas} entradas; primeiras:\n{}",
        divergencias.len(),
        primeiras.join("\n")
    );
    assert!(
        comparadas >= 200_000,
        "corpus pequeno demais: {comparadas} entradas"
    );
    println!(
        "PASS: {comparadas} entradas do parser idênticas ao C# ({aceitas} aceitas, {} rejeitadas); 0 divergências",
        comparadas - aceitas
    );
}
