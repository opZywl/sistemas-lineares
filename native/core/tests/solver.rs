mod comum;

use comum::RandomDotNet;
use sistemas_lineares_core::{Rational, SolutionKind, format, solve};

struct Verificacoes(u32);

impl Verificacoes {
    fn check(&mut self, condicao: bool, mensagem: &str) {
        self.0 += 1;
        assert!(condicao, "{mensagem}");
    }
}

fn r(texto: &str) -> Rational {
    Rational::parse(texto).unwrap()
}

fn matriz(linhas: &[&[&str]]) -> Vec<Vec<Rational>> {
    linhas
        .iter()
        .map(|linha| linha.iter().map(|valor| r(valor)).collect())
        .collect()
}

#[test]
fn porte_fiel_de_solver_tests() {
    let mut v = Verificacoes(0);
    v.check(r("-1,5").to_string() == "-3/2", "comma");
    v.check(r(".25").to_string() == "1/4", "decimal");
    v.check(r("2,5/0,5").to_string() == "5", "fraction of decimals");
    for s in ["", "x", "1/0", "1/2/3", "1.2.3", "--2", "."] {
        v.check(Rational::parse(s).is_err(), &format!("invalid {s}"));
    }
    let a = matriz(&[&["2", "1", "5"], &["1", "-1", "1"]]);
    let sol = solve(&a).unwrap();
    v.check(sol.kind == SolutionKind::Unique, "unique");
    v.check(
        sol.reduced[0][2].to_string() == "2" && sol.reduced[1][2].to_string() == "1",
        "unique values",
    );
    let sol = solve(&matriz(&[&["0", "1", "2"], &["1", "0", "3"]])).unwrap();
    v.check(
        sol.reduced[0][2].to_string() == "3" && sol.reduced[1][2].to_string() == "2",
        "pivot swap",
    );
    let sol = solve(&matriz(&[&["1", "1", "2"], &["2", "2", "4"]])).unwrap();
    v.check(sol.kind == SolutionKind::Infinite, "infinite");
    v.check(format(&sol).contains("x = 2 - t1"), "parametric");
    let sol = solve(&matriz(&[&["1", "1", "2"], &["2", "2", "5"]])).unwrap();
    v.check(sol.kind == SolutionKind::None, "inconsistent");
    let sol = solve(&matriz(&[
        &["0", "0", "0", "0"],
        &["0", "0", "0", "0"],
        &["0", "0", "0", "0"],
    ]))
    .unwrap();
    v.check(
        sol.kind == SolutionKind::Infinite && format(&sol).contains("z = t3"),
        "all free",
    );
    let sol = solve(&matriz(&[
        &["0", "0", "0", "1"],
        &["0", "0", "0", "0"],
        &["0", "0", "0", "0"],
    ]))
    .unwrap();
    v.check(sol.kind == SolutionKind::None, "zero contradiction");
    let sol = solve(&matriz(&[
        &["1", "1", "1", "6"],
        &["2", "-1", "1", "3"],
        &["1", "2", "-1", "2"],
    ]))
    .unwrap();
    v.check(
        sol.kind == SolutionKind::Unique
            && sol.reduced[0][3] == r("1")
            && sol.reduced[1][3] == r("2")
            && sol.reduced[2][3] == r("3"),
        "3x3",
    );
    let mut random = RandomDotNet::new(19);
    for n in 2..=3 {
        for _ in 0..500 {
            let mut matrix = vec![vec![Rational::zero(); n + 1]; n];
            let mut expected = vec![Rational::zero(); n];
            for valor in expected.iter_mut() {
                *valor = Rational::integer(random.next(-5, 6));
            }
            for linha in matrix.iter_mut() {
                for (j, esperado) in expected.iter().enumerate() {
                    linha[j] = Rational::integer(random.next(-5, 6));
                    linha[n] = &linha[n] + &(&linha[j] * esperado);
                }
            }
            let output = solve(&matrix).unwrap();
            v.check(output.kind != SolutionKind::None, "generated consistent");
            if output.kind == SolutionKind::Unique {
                for (linha, esperado) in output.reduced.iter().zip(&expected) {
                    v.check(linha[n] == *esperado, "generated solution");
                }
            }
        }
    }
    assert_eq!(
        v.0, 3421,
        "a contagem deve ser idêntica à do SolverTests.cs"
    );
    println!("PASS: {} verificações", v.0);
}

type Sequencia = (i32, i32, i32, usize, &'static [i32], u64);

const SEQUENCIAS_DOTNET: &[Sequencia] = &[
    (
        19,
        -5,
        6,
        9000,
        &[2, -1, -5, 1, 3, -4, -1, 5, -2, -2, -2, -2],
        0x065A01EC6D2296C4,
    ),
    (
        71291,
        -1000,
        1001,
        18000,
        &[
            899, 848, -358, -764, -849, 215, -368, -951, 730, 817, 96, -238,
        ],
        0x5BD128B4C2E57280,
    ),
    (
        0,
        i32::MIN,
        i32::MAX,
        5000,
        &[
            -1559595548,
            1649316165,
            442452828,
            -1945678310,
            2099272108,
            626863972,
            1358625012,
            -2109153757,
            -1851925805,
            1454235443,
            -1754296377,
            -2130093703,
        ],
        0x28E82F318E924369,
    ),
    (
        i32::MIN,
        0,
        1000,
        5000,
        &[726, 817, 768, 558, 206, 558, 906, 442, 977, 273, 291, 467],
        0x64274F7867EAE147,
    ),
    (
        i32::MAX,
        -10,
        10,
        5000,
        &[4, 6, 5, 1, -6, 1, 8, -2, 9, -5, -5, -1],
        0x90C140FB5E73913C,
    ),
    (
        -161803398,
        i32::MIN,
        0,
        5000,
        &[
            -1893288791,
            -227110172,
            -1125954212,
            -995900781,
            -543427961,
            -941487620,
            -92557974,
            -686641116,
            -1923728137,
            -887514618,
            -1498223037,
            -2090022132,
        ],
        0x4447F971E2748DED,
    ),
    (
        161803398,
        -2000000000,
        2000000000,
        5000,
        &[
            -1526525182,
            1576974340,
            -97253150,
            144990241,
            987786544,
            246342653,
            1827597339,
            721031256,
            -1583222882,
            346875202,
            -790657872,
            -1892969586,
        ],
        0xB872580ABD3D74C3,
    ),
    (
        -1,
        0,
        0,
        100,
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        0x2C1B93DAAFB34265,
    ),
];

const NEXT_DOUBLE_SEMENTE_1: [u64; 8] = [
    0x3FCFD45F463FA8BF,
    0x3FBC59B7A038B36F,
    0x3FDDE380C33BC702,
    0x3FE8B0FB20B161F6,
    0x3FE50A65102A14CA,
    0x3FDBB2B5CBB7656C,
    0x3FD6A94EF22D529E,
    0x3FEE341EA8FC683D,
];

#[test]
fn random_identico_ao_dotnet_framework() {
    let mut valores = 0;
    for &(semente, minimo, maximo, quantidade, primeiros, hash_esperado) in SEQUENCIAS_DOTNET {
        let mut random = RandomDotNet::new(semente);
        let mut hash = 0xCBF2_9CE4_8422_2325u64;
        for posicao in 0..quantidade {
            let valor = random.next(minimo, maximo);
            if let Some(&esperado) = primeiros.get(posicao) {
                assert_eq!(
                    valor, esperado,
                    "new Random({semente}).Next({minimo}, {maximo}), posição {posicao}"
                );
            }
            for byte in valor.to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x0100_0000_01B3);
            }
        }
        assert_eq!(
            hash, hash_esperado,
            "FNV-1a de {quantidade} valores de new Random({semente}).Next({minimo}, {maximo})"
        );
        valores += quantidade;
    }
    let mut random = RandomDotNet::new(1);
    let duplos: Vec<u64> = (0..NEXT_DOUBLE_SEMENTE_1.len())
        .map(|_| random.amostra().to_bits())
        .collect();
    assert_eq!(
        duplos, NEXT_DOUBLE_SEMENTE_1,
        "new Random(1).NextDouble(), bit a bit"
    );
    println!(
        "PASS: {} sequências do System.Random idênticas ao .NET Framework ({valores} valores de Next e {} de NextDouble)",
        SEQUENCIAS_DOTNET.len(),
        NEXT_DOUBLE_SEMENTE_1.len()
    );
}
