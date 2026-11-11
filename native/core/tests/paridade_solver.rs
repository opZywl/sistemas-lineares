mod comum;

use std::fs;

use comum::decodificar_base64;
use sistemas_lineares_core::{Rational, SolutionKind, format, solve};

#[test]
fn sistemas_racionais_identicos_ao_csharp() {
    let Some(caminho) = std::env::var_os("SL_SOLVER_CSHARP") else {
        eprintln!("SL_SOLVER_CSHARP não definido; paridade de sistemas racionais ignorada");
        return;
    };
    let conteudo = fs::read_to_string(&caminho).expect("saída do helper C#");
    let mut sistemas = 0usize;
    let mut verificacoes = 0usize;
    let mut por_tipo = [0usize; 3];
    for linha in conteudo.lines() {
        sistemas += 1;
        let campos: Vec<&str> = linha.split('|').collect();
        assert_eq!(campos.len(), 6, "linha {sistemas} malformada: {linha:?}");
        let n: usize = campos[0].parse().expect("ordem do sistema");
        let valores: Vec<Rational> = campos[1]
            .split(';')
            .map(|valor| {
                Rational::parse(valor)
                    .unwrap_or_else(|erro| panic!("sistema {sistemas}: {valor:?} -> {erro}"))
            })
            .collect();
        assert_eq!(valores.len(), n * (n + 1), "sistema {sistemas}");
        let matriz: Vec<Vec<Rational>> = valores.chunks(n + 1).map(<[Rational]>::to_vec).collect();
        let solucao = solve(&matriz).unwrap();
        let (tipo, indice) = match solucao.kind {
            SolutionKind::Unique => ("unique", 0),
            SolutionKind::Infinite => ("infinite", 1),
            SolutionKind::None => ("none", 2),
        };
        let pivos: Vec<String> = solucao.pivot_columns.iter().map(usize::to_string).collect();
        let reduzida: Vec<String> = solucao
            .reduced
            .iter()
            .flatten()
            .map(Rational::to_string)
            .collect();
        let texto = String::from_utf8(decodificar_base64(campos[5])).expect("texto do C# em UTF-8");
        assert_eq!(
            tipo, campos[2],
            "classificação, sistema {sistemas}: {}",
            campos[1]
        );
        assert_eq!(
            pivos.join(","),
            campos[3],
            "colunas pivô, sistema {sistemas}: {}",
            campos[1]
        );
        assert_eq!(
            reduzida.join(";"),
            campos[4],
            "matriz reduzida, sistema {sistemas}: {}",
            campos[1]
        );
        assert_eq!(
            format(&solucao),
            texto,
            "texto, sistema {sistemas}: {}",
            campos[1]
        );
        verificacoes += 4;
        por_tipo[indice] += 1;
    }
    assert!(sistemas > 0, "saída do C# vazia");
    println!(
        "PASS: {sistemas} sistemas racionais idênticos ao C# ({} única, {} infinitas, {} sem solução); {verificacoes} verificações",
        por_tipo[0], por_tipo[1], por_tipo[2]
    );
}
