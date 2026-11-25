pub static REGULAR: &[u8] = include_bytes!("../../../assets/SpaceGrotesk-Regular.ttf");
pub static BOLD: &[u8] = include_bytes!("../../../assets/SpaceGrotesk-Bold.ttf");
pub const FAMILY: &str = "Space Grotesk";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextMetrics {
    pub height: i32,
    pub ascent: i32,
    pub descent: i32,
    pub external_leading: i32,
}

pub fn line_spacing_ratio(data: &[u8]) -> Option<f32> {
    let read16 = |offset: usize| {
        data.get(offset..offset + 2)
            .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
    };
    let read32 = |offset: usize| {
        data.get(offset..offset + 4)
            .map(|bytes| u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    };
    let tables = read16(4)? as usize;
    let mut head = None;
    let mut hhea = None;
    for index in 0..tables {
        let record = 12 + 16 * index;
        let tag = data.get(record..record + 4)?;
        let offset = read32(record + 8)? as usize;
        match tag {
            b"head" => head = Some(offset),
            b"hhea" => hhea = Some(offset),
            _ => {}
        }
    }
    let units = read16(head? + 18)? as f32;
    let hhea = hhea?;
    let ascender = read16(hhea + 4)? as i16 as f32;
    let descender = read16(hhea + 6)? as i16 as f32;
    let gap = read16(hhea + 8)? as i16 as f32;
    (units > 0.0).then(|| (ascender - descender + gap) / units)
}

pub fn preferred_height(pixels: i32) -> Option<i32> {
    line_spacing_ratio(REGULAR).map(|ratio| (pixels as f32 * ratio).ceil() as i32)
}

pub fn margins_for(metric: TextMetrics) -> (i32, i32) {
    let overhang = metric.height.max(0) as f32 / 6.0;
    (overhang.ceil() as i32, (overhang * 1.5).ceil() as i32)
}

pub fn wrap(paragraph: &str, available: i32, measure: &dyn Fn(&str) -> i32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in paragraph.split(' ') {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if measure(&candidate) <= available {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        if measure(word) <= available {
            current = word.to_string();
            continue;
        }
        let mut pieces = break_word(word, available, measure);
        current = pieces.pop().unwrap_or_default();
        lines.extend(pieces);
    }
    lines.push(current);
    lines
}

fn break_word(word: &str, available: i32, measure: &dyn Fn(&str) -> i32) -> Vec<String> {
    const SAMPLE: usize = 16;
    let characters: Vec<char> = word.chars().collect();
    let mut advances: Vec<(char, f32)> = Vec::new();
    let mut advance = |character: char| -> f32 {
        if let Some((_, width)) = advances.iter().find(|(known, _)| *known == character) {
            return *width;
        }
        let sample: String = std::iter::repeat_n(character, SAMPLE).collect();
        let width = measure(&sample) as f32 / SAMPLE as f32;
        advances.push((character, width));
        width
    };
    let slice = |start: usize, end: usize| characters[start..end].iter().collect::<String>();
    let limit = available as f32;
    let mut pieces = Vec::new();
    let mut start = 0;
    while start < characters.len() {
        let mut end = start;
        let mut width = 0.0;
        while end < characters.len() {
            let next = advance(characters[end]);
            if end > start && width + next > limit {
                break;
            }
            width += next;
            end += 1;
        }
        while end > start + 1 && measure(&slice(start, end)) > available {
            end -= 1;
        }
        while end < characters.len() && measure(&slice(start, end + 1)) <= available {
            end += 1;
        }
        pieces.push(slice(start, end));
        start = end;
    }
    pieces
}

pub fn wrap_lines(text: &str, available: i32, measure: &dyn Fn(&str) -> i32) -> Vec<String> {
    let available = available.max(1);
    text.split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .flat_map(|line| {
            if measure(line) <= available {
                vec![line.to_string()]
            } else {
                wrap(line, available, measure)
            }
        })
        .collect()
}

pub const KEEP_LIMIT: usize = 16;
pub const KEEP_SPACE: char = '\u{a0}';

fn group_end(characters: &[char], start: usize) -> Option<usize> {
    if characters[start] == '(' {
        let inside = characters[start + 1..]
            .iter()
            .position(|character| matches!(character, '(' | ')'));
        if let Some(inside) = inside {
            let close = start + 1 + inside;
            if inside > 0 && characters[close] == ')' {
                return Some(close + 1);
            }
        }
    }
    if characters[start].is_whitespace() {
        return None;
    }
    let word_end = |from: usize| {
        characters[from..]
            .iter()
            .position(|character| character.is_whitespace())
            .map_or(characters.len(), |offset| from + offset)
    };
    let left = word_end(start);
    if characters.get(left..left + 3) != Some(&[' ', '=', ' '][..]) {
        return None;
    }
    let right = word_end(left + 3);
    (right > left + 3).then_some(right)
}

pub fn keep_groups(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut kept = vec![false; characters.len()];
    let mut start = 0;
    while start < characters.len() {
        match group_end(&characters, start) {
            Some(end) => {
                if end - start <= KEEP_LIMIT {
                    kept[start..end].fill(true);
                }
                start = end;
            }
            None => start += 1,
        }
    }
    characters
        .iter()
        .zip(&kept)
        .map(|(character, keep)| {
            if *keep && *character == ' ' {
                KEEP_SPACE
            } else {
                *character
            }
        })
        .collect()
}

pub fn ellipsize(text: &str, available: i32, measure: &dyn Fn(&str) -> i32) -> String {
    if measure(text) <= available {
        return text.to_string();
    }
    let characters: Vec<char> = text.chars().collect();
    let fits = |count: usize| {
        let candidate: String = characters[..count].iter().collect::<String>() + "\u{2026}";
        measure(&candidate) <= available
    };
    let (mut low, mut high) = (0usize, characters.len());
    while low < high {
        let middle = (low + high).div_ceil(2);
        if fits(middle) {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    let kept: String = characters[..low].iter().collect();
    format!("{}\u{2026}", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grupos_curtos_ficam_juntos_como_no_html() {
        let juntos = |texto: &str| texto.replace(' ', "\u{a0}");
        assert_eq!(
            keep_groups("As retas se cruzam em (2, 1): esse ponto"),
            format!("As retas se cruzam em {}: esse ponto", juntos("(2, 1)"))
        );
        assert_eq!(
            keep_groups("qualquer ponto (0 = 0): o plano"),
            format!("qualquer ponto {}: o plano", juntos("(0 = 0)"))
        );
        assert_eq!(
            keep_groups("A equação 2 diz que 0 = 5/2, o que nunca acontece"),
            format!(
                "A equação 2 diz que {} o que nunca acontece",
                juntos("0 = 5/2,")
            )
        );
        let limite = "(1234567, 12345)";
        assert_eq!(limite.chars().count(), KEEP_LIMIT);
        assert_eq!(keep_groups(limite), juntos(limite));
        let longo = "(-5055/28171, -50715/28171)";
        assert_eq!(keep_groups(longo), longo);
        assert_eq!(
            keep_groups("((1, 2) e (3, 4"),
            format!("({} e (3, 4", juntos("(1, 2)"))
        );
        assert_eq!(keep_groups("() e x = "), "() e x = ");
        assert_eq!(keep_groups("(x = 1"), juntos("(x = 1"));
        assert_eq!(keep_groups("sem grupos aqui"), "sem grupos aqui");
        assert_eq!(keep_groups(""), "");
        let medir = |texto: &str| texto.chars().count() as i32;
        assert_eq!(
            wrap(&keep_groups("em (2, 1): esse ponto"), 8, &medir),
            ["em", "(2,\u{a0}1):", "esse", "ponto"]
        );
    }

    #[test]
    fn espacamento_da_space_grotesk() {
        let ratio = line_spacing_ratio(REGULAR).unwrap();
        assert!((ratio - 1.276).abs() < 1e-4);
        assert_eq!(preferred_height(16), Some(21));
        assert_eq!(preferred_height(32), Some(41));
    }

    #[test]
    fn reticencias_por_busca_binaria() {
        let medir = |texto: &str| texto.chars().count() as i32 * 10;
        assert_eq!(ellipsize("curto", 100, &medir), "curto");
        assert_eq!(ellipsize("abcdefghij", 60, &medir), "abcde\u{2026}");
        assert_eq!(ellipsize("ab cd ef", 50, &medir), "ab c\u{2026}");
        assert_eq!(ellipsize("abc", 5, &medir), "\u{2026}");
    }

    #[test]
    fn quebra_palavras_e_palavras_longas() {
        let medir = |texto: &str| texto.chars().count() as i32 * 10;
        assert_eq!(
            wrap_lines("um dois tres", 70, &medir),
            vec!["um dois".to_string(), "tres".to_string()]
        );
        assert_eq!(
            wrap_lines("abcdefghij", 40, &medir),
            vec!["abcd".to_string(), "efgh".to_string(), "ij".to_string()]
        );
        assert_eq!(
            wrap_lines("a\nb", 100, &medir),
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            wrap_lines("ab cdefghijkl mn", 40, &medir),
            vec![
                "ab".to_string(),
                "cdef".to_string(),
                "ghij".to_string(),
                "kl".to_string(),
                "mn".to_string()
            ]
        );
        assert_eq!(
            margins_for(TextMetrics {
                height: 21,
                ascent: 17,
                descent: 4,
                external_leading: 0
            }),
            (4, 6)
        );
    }

    #[test]
    fn numero_enorme_quebra_com_poucas_medidas() {
        use std::cell::Cell;
        let chamadas = Cell::new(0usize);
        let larguras = |caractere: char| match caractere {
            '1' => 7,
            '/' => 5,
            _ => 9,
        };
        let medir = |texto: &str| {
            chamadas.set(chamadas.get() + 1);
            texto.chars().map(larguras).sum::<i32>()
        };
        let numero: String = "1234567890".repeat(300) + "/" + &"7".repeat(500);
        let linhas = wrap_lines(&numero, 600, &medir);
        assert!(
            chamadas.get() < 400,
            "{} medidas para {} caracteres",
            chamadas.get(),
            numero.len()
        );
        assert_eq!(linhas.concat(), numero, "nada se perde na quebra");
        let mut inicio = 0;
        for (indice, linha) in linhas.iter().enumerate() {
            let largura: i32 = linha.chars().map(larguras).sum();
            assert!(largura <= 600, "linha larga demais: {}", linha.len());
            inicio += linha.len();
            if indice + 1 < linhas.len() {
                let proximo = numero[inicio..].chars().next().unwrap_or('1');
                assert!(
                    largura + larguras(proximo) > 600,
                    "a linha {indice} ainda cabia mais um caractere"
                );
            }
        }
    }
}
