use std::fs;

use sistemas_lineares_core::{Rational, SolutionKind, format, solve};

fn decode_base64(text: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bytes = Vec::with_capacity(text.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0;
    for byte in text.bytes().filter(|&byte| byte != b'=') {
        let value = ALPHABET
            .iter()
            .position(|&symbol| symbol == byte)
            .expect("base64 invalido") as u32;
        buffer = buffer << 6 | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    String::from_utf8(bytes).expect("utf-8 invalido")
}

#[test]
fn respostas_identicas_ao_csharp() {
    let Ok(path) = std::env::var("SL_FIXTURES_CSHARP") else {
        eprintln!("SL_FIXTURES_CSHARP nao definido; paridade com C# ignorada");
        return;
    };
    let content = fs::read_to_string(path).expect("fixtures do C#");
    let mut systems = 0;
    for line in content.lines() {
        systems += 1;
        let fields: Vec<&str> = line.split('|').collect();
        let size: usize = fields[0].parse().unwrap();
        let values: Vec<Rational> = fields[1]
            .split(',')
            .map(|value| Rational::parse(value).unwrap())
            .collect();
        let matrix: Vec<Vec<Rational>> =
            values.chunks(size + 1).map(<[Rational]>::to_vec).collect();
        let solution = solve(&matrix).unwrap();
        let kind = match solution.kind {
            SolutionKind::Unique => "unique",
            SolutionKind::Infinite => "infinite",
            SolutionKind::None => "none",
        };
        assert_eq!(kind, fields[2], "classificacao, sistema {systems}");
        assert_eq!(
            format(&solution).replace("\r\n", "\n"),
            decode_base64(fields[3]),
            "resposta, sistema {systems}"
        );
    }
    assert_eq!(systems, 76_674);
    println!("PASS: {systems} sistemas identicos entre Rust e C#");
}
