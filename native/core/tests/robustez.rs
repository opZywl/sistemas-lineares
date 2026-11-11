mod comum;

use std::fmt::Write as _;
use std::fs;

use comum::{RandomDotNet, codificar_base64};
use num_bigint::BigInt;
use num_traits::Zero;
use sistemas_lineares_core::{Rational, SolutionKind, format, solve};

struct Auditoria {
    verificacoes: u64,
    sistemas: u64,
    fixtures: Option<String>,
}

fn r(valor: i32) -> Rational {
    Rational::new(BigInt::from(valor), BigInt::from(1))
}

fn det(a: &[Vec<BigInt>]) -> BigInt {
    match a.len() {
        1 => a[0][0].clone(),
        2 => &a[0][0] * &a[1][1] - &a[0][1] * &a[1][0],
        _ => {
            &a[0][0] * (&a[1][1] * &a[2][2] - &a[1][2] * &a[2][1])
                - &a[0][1] * (&a[1][0] * &a[2][2] - &a[1][2] * &a[2][0])
                + &a[0][2] * (&a[1][0] * &a[2][1] - &a[1][1] * &a[2][0])
        }
    }
}

fn bits(mascara: u32) -> usize {
    mascara.count_ones() as usize
}

fn posto(a: &[Vec<BigInt>]) -> usize {
    let linhas = a.len();
    let colunas = a[0].len();
    for k in (1..=linhas.min(colunas)).rev() {
        for mascara_linhas in 1..(1u32 << linhas) {
            if bits(mascara_linhas) != k {
                continue;
            }
            for mascara_colunas in 1..(1u32 << colunas) {
                if bits(mascara_colunas) != k {
                    continue;
                }
                let menor: Vec<Vec<BigInt>> = (0..linhas)
                    .filter(|&i| mascara_linhas & (1 << i) != 0)
                    .map(|i| {
                        (0..colunas)
                            .filter(|&j| mascara_colunas & (1 << j) != 0)
                            .map(|j| a[i][j].clone())
                            .collect()
                    })
                    .collect();
                if !det(&menor).is_zero() {
                    return k;
                }
            }
        }
    }
    0
}

impl Auditoria {
    fn check(&mut self, condicao: bool, mensagem: &str) {
        self.verificacoes += 1;
        assert!(condicao, "{mensagem}");
    }

    fn auditar(&mut self, inteiros: &[Vec<i32>]) {
        self.sistemas += 1;
        let n = inteiros.len();
        let entrada: Vec<Vec<Rational>> = inteiros
            .iter()
            .map(|linha| linha.iter().map(|&valor| r(valor)).collect())
            .collect();
        let coeficientes: Vec<Vec<BigInt>> = inteiros
            .iter()
            .map(|linha| {
                linha[..n]
                    .iter()
                    .map(|&valor| BigInt::from(valor))
                    .collect()
            })
            .collect();
        let aumentada: Vec<Vec<BigInt>> = inteiros
            .iter()
            .map(|linha| linha.iter().map(|&valor| BigInt::from(valor)).collect())
            .collect();
        let resposta = solve(&entrada).unwrap();
        let ra = posto(&coeficientes);
        let rab = posto(&aumentada);
        if let Some(fixtures) = self.fixtures.as_mut() {
            let entradas: Vec<String> = inteiros.iter().flatten().map(i32::to_string).collect();
            let tipo = match resposta.kind {
                SolutionKind::Unique => "unique",
                SolutionKind::Infinite => "infinite",
                SolutionKind::None => "none",
            };
            let texto = format(&resposta).replace("\r\n", "\n");
            write!(
                fixtures,
                "{n}|{}|{tipo}|{}\r\n",
                entradas.join(","),
                codificar_base64(texto.as_bytes())
            )
            .unwrap();
        }
        let esperado = if ra < rab {
            SolutionKind::None
        } else if ra == n {
            SolutionKind::Unique
        } else {
            SolutionKind::Infinite
        };
        self.check(resposta.kind == esperado, "classificação independente");
        for i in 0..n {
            for j in 0..=n {
                self.check(entrada[i][j] == r(inteiros[i][j]), "entrada preservada");
            }
        }
        if resposta.kind == SolutionKind::None {
            return;
        }
        if resposta.kind == SolutionKind::Unique {
            let denominador = det(&coeficientes);
            for coluna in 0..n {
                let mut substituida = coeficientes.clone();
                for linha in 0..n {
                    substituida[linha][coluna] = BigInt::from(inteiros[linha][n]);
                }
                self.check(
                    resposta.reduced[coluna][n]
                        == Rational::new(det(&substituida), denominador.clone()),
                    "solução por Cramer",
                );
            }
        }
        let mut linha_do_pivo = vec![None; n];
        for (linha, &coluna) in resposta.pivot_columns.iter().enumerate() {
            linha_do_pivo[coluna] = Some(linha);
        }
        for amostra in 0..4i32 {
            let mut x = vec![Rational::zero(); n];
            for j in 0..n {
                if linha_do_pivo[j].is_none() {
                    x[j] = Rational::new(
                        BigInt::from((amostra - 1) * (j as i32 + 1)),
                        BigInt::from(amostra + 1),
                    );
                }
            }
            for j in 0..n {
                if let Some(linha) = linha_do_pivo[j] {
                    x[j] = resposta.reduced[linha][n].clone();
                    for livre in 0..n {
                        if linha_do_pivo[livre].is_none() {
                            x[j] = &x[j] - &(&resposta.reduced[linha][livre] * &x[livre]);
                        }
                    }
                }
            }
            for linha in &entrada {
                let mut total = r(0);
                for (coeficiente, valor) in linha.iter().zip(&x) {
                    total = &total + &(coeficiente * valor);
                }
                self.check(total == linha[n], "substituição nas equações originais");
            }
        }
    }
}

fn primeira_linha_diferente(gerado: &str, referencia: &[u8]) -> Option<usize> {
    let referencia = String::from_utf8_lossy(referencia);
    let mut gerado = gerado.split("\r\n");
    let mut referencia = referencia.split("\r\n");
    let mut numero = 0;
    loop {
        numero += 1;
        match (gerado.next(), referencia.next()) {
            (None, None) => return None,
            (a, b) if a != b => return Some(numero),
            _ => {}
        }
    }
}

#[test]
fn porte_fiel_de_robustness_tests() {
    let saida = std::env::var_os("SL_FIXTURES_SAIDA");
    let referencia = std::env::var_os("SL_FIXTURES_CSHARP");
    let mut auditoria = Auditoria {
        verificacoes: 0,
        sistemas: 0,
        fixtures: (saida.is_some() || referencia.is_some()).then(String::new),
    };
    for code in 0..15625 {
        let mut value = code;
        let mut a = vec![vec![0; 3]; 2];
        for linha in a.iter_mut() {
            for valor in linha.iter_mut() {
                *valor = value % 5 - 2;
                value /= 5;
            }
        }
        auditoria.auditar(&a);
    }
    for code in 0..19683 {
        let mut value = code;
        let mut a = vec![vec![0; 4]; 3];
        for linha in a.iter_mut() {
            for valor in linha[..3].iter_mut() {
                *valor = value % 3 - 1;
                value /= 3;
            }
        }
        for b in 0..3 {
            for (i, linha) in (0i32..).zip(a.iter_mut()) {
                linha[3] = if b == 0 {
                    0
                } else if b == 1 {
                    i - 1
                } else {
                    i + 1
                };
            }
            auditoria.auditar(&a);
        }
    }
    let mut random = RandomDotNet::new(71291);
    for k in 0..2000 {
        let n = k % 2 + 2;
        let mut a = vec![vec![0; n + 1]; n];
        for linha in a.iter_mut() {
            for valor in linha.iter_mut() {
                *valor = random.next(-1000, 1001);
            }
        }
        auditoria.auditar(&a);
    }
    auditoria.check(
        Rational::parse("1e-20").unwrap().to_string() == "1/100000000000000000000",
        "expoente negativo",
    );
    auditoria.check(
        Rational::parse("\u{2212}2,5E+3").unwrap().to_string() == "-2500",
        "sinal unicode e expoente",
    );
    auditoria.check(
        Rational::parse("1e100/1e100").unwrap() == r(1),
        "fração grande",
    );
    let longo = "1".repeat(101);
    for bad in [
        "1e",
        "1e1001",
        "1e-2147483648",
        "NaN",
        "Infinity",
        "1e1e1",
        "1/0",
        "1,2.3",
        longo.as_str(),
    ] {
        auditoria.check(
            Rational::parse(bad).is_err(),
            &format!("entrada inválida: {bad}"),
        );
    }
    let tiny = Rational::parse("1e-50").unwrap();
    let one = r(1);
    let near = solve(&[
        vec![one.clone(), one.clone(), r(2)],
        vec![one.clone(), &one + &tiny, &r(2) + &tiny],
    ])
    .unwrap();
    auditoria.check(
        near.kind == SolutionKind::Unique && near.reduced[0][2] == one && near.reduced[1][2] == one,
        "quase singular, cálculo exato",
    );
    let huge = Rational::parse(&"9".repeat(100)).unwrap();
    let large = solve(&[
        vec![huge.clone(), one.clone(), &huge + &one],
        vec![one.clone(), -&huge, &one - &huge],
    ])
    .unwrap();
    auditoria.check(
        large.kind == SolutionKind::Unique
            && large.reduced[0][2] == one
            && large.reduced[1][2] == one,
        "100 dígitos",
    );
    if let Some(conteudo) = &auditoria.fixtures {
        if let Some(caminho) = &saida {
            fs::write(caminho, conteudo).expect("gravar SL_FIXTURES_SAIDA");
            println!(
                "fixtures gravadas: {} ({} bytes)",
                caminho.to_string_lossy(),
                conteudo.len()
            );
        }
        if let Some(caminho) = &referencia {
            let esperado = fs::read(caminho).expect("ler SL_FIXTURES_CSHARP");
            assert!(
                conteudo.as_bytes() == esperado.as_slice(),
                "fixtures diferentes do C# (Rust {} bytes, C# {} bytes); primeira linha divergente: {:?}",
                conteudo.len(),
                esperado.len(),
                primeira_linha_diferente(conteudo, &esperado)
            );
            println!(
                "fixtures byte a byte idênticas às do C#: {} bytes",
                esperado.len()
            );
        }
    }
    assert_eq!(
        (auditoria.sistemas, auditoria.verificacoes),
        (76_674, 1_703_886),
        "a contagem deve ser idêntica à do RobustnessTests.cs"
    );
    println!(
        "PASS: {} sistemas auditados; {} verificações independentes",
        auditoria.sistemas, auditoria.verificacoes
    );
}
