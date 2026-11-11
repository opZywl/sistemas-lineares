use sistemas_lineares::layout::{self, Layout, Metrics, Rect, d};

const SCALES: [f32; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5];
const SIZES: [(i32, i32); 4] = [(1040, 760), (1143, 793), (1600, 1000), (2400, 1400)];

fn metrics(scale: f32) -> Metrics {
    let input = (16.0 * scale * 1.276).ceil() as i32;
    let value = (32.0 * scale * 1.276).ceil() as i32;
    Metrics {
        input_height: input,
        value_height: value,
    }
}

fn each() -> impl Iterator<Item = (f32, (i32, i32), usize, Layout)> {
    SCALES.into_iter().flat_map(|scale| {
        SIZES.into_iter().flat_map(move |size| {
            [2usize, 3].into_iter().map(move |dimension| {
                (
                    scale,
                    size,
                    dimension,
                    layout::compute(scale, size.0, size.1, dimension, metrics(scale)),
                )
            })
        })
    })
}

fn assert_disjoint(context: &str, items: &[(&str, Rect)]) {
    for (index, (first_name, first)) in items.iter().enumerate() {
        for (second_name, second) in &items[index + 1..] {
            assert!(
                !first.intersects(*second),
                "{context}: {first_name} {first:?} sobrepõe {second_name} {second:?}"
            );
        }
    }
}

#[test]
fn ultima_linha_acima_do_helper() {
    for (scale, size, dimension, layout) in each() {
        let context = format!("escala {scale} tamanho {size:?} {dimension}x{dimension}");
        for row in 0..dimension {
            for column in 0..4 {
                if layout::field_visible(dimension, row, column) {
                    assert!(
                        layout.fields[row][column].bottom() <= layout.helper.y,
                        "{context}: campo ({row},{column})"
                    );
                    assert!(
                        layout.entry.contains(layout.fields[row][column]),
                        "{context}: campo fora do card"
                    );
                    assert!(
                        layout.fields[row][column].contains(layout.editors[row][column]),
                        "{context}: EDIT fora do campo ({row},{column})"
                    );
                }
            }
        }
    }
}

#[test]
fn botoes_dentro_dos_cards() {
    for (scale, size, dimension, layout) in each() {
        let context = format!("escala {scale} tamanho {size:?} {dimension}x{dimension}");
        for (name, rect) in [
            ("2x2", layout.two),
            ("3x3", layout.three),
            ("resolver", layout.solve),
            ("exemplo", layout.example),
            ("limpar", layout.clear),
        ] {
            assert!(
                layout.entry.contains(rect),
                "{context}: {name} {rect:?} fora do card {:?}",
                layout.entry
            );
        }
        assert!(
            layout.solution.contains(layout.copy),
            "{context}: copiar fora do card de resultado"
        );
        assert!(
            layout.author.right() <= layout.content_w && layout.author.x >= 0,
            "{context}: autor fora do conteúdo"
        );
        assert!(
            (layout.author.right() - (layout.content_w - d(scale, 32))).abs() <= 1,
            "{context}: borda direita do autor {:?}",
            layout.author
        );
        assert!(
            layout.entry.bottom() <= layout.footer.y,
            "{context}: card invade o rodapé"
        );
        assert!(
            layout.solution.bottom() <= layout.shortcut.y,
            "{context}: resultado invade o rodapé"
        );
        assert!(
            layout.entry.right() < layout.solution.x,
            "{context}: cards se sobrepõem"
        );
        assert!(
            layout.solution.right() <= layout.content_w,
            "{context}: resultado fora do conteúdo"
        );
    }
}

#[test]
fn tiles_dentro_do_card_de_resultado() {
    for (scale, size, dimension, layout) in each() {
        let context = format!("escala {scale} tamanho {size:?} {dimension}x{dimension}");
        for index in 0..3 {
            assert!(
                layout.solution.contains(layout.tiles[index]),
                "{context}: tile {index}"
            );
            assert!(
                layout.tiles[index].contains(layout.tile_names[index]),
                "{context}: nome do tile {index}"
            );
            let value = layout.tile_values[index];
            assert!(
                value.x >= layout.tile_names[index].right()
                    && value.right() <= layout.tiles[index].right(),
                "{context}: valor do tile {index} {value:?}"
            );
            assert!(
                layout.tiles[index].bottom() <= layout.note.y,
                "{context}: tile {index} invade a nota"
            );
        }
        assert!(
            layout.solution.contains(layout.general),
            "{context}: resultado geral"
        );
        assert!(
            layout.general.bottom() <= layout.note.y,
            "{context}: resultado geral invade a nota"
        );
        assert!(
            layout.note.bottom() <= layout.copy.y,
            "{context}: nota invade o botão copiar"
        );
    }
}

#[test]
fn irmaos_visiveis_nao_se_sobrepoem() {
    for (scale, size, dimension, layout) in each() {
        let context = format!("escala {scale} tamanho {size:?} {dimension}x{dimension}");
        let mut entry: Vec<(String, Rect)> = vec![
            ("icone".into(), layout.entry_icon),
            ("titulo".into(), layout.entry_title),
            ("descricao".into(), layout.entry_description),
            ("2x2".into(), layout.two),
            ("3x3".into(), layout.three),
            ("dimensao".into(), layout.dimension_text),
            ("rotulo".into(), layout.grid_label),
            ("helper".into(), layout.helper),
            ("resolver".into(), layout.solve),
            ("exemplo".into(), layout.example),
            ("limpar".into(), layout.clear),
        ];
        for row in 0..dimension {
            entry.push((format!("numero {row}"), layout.row_numbers[row]));
            for column in 0..4 {
                if layout::field_visible(dimension, row, column) {
                    entry.push((format!("campo {row},{column}"), layout.fields[row][column]));
                }
            }
            for column in 0..3 {
                if layout::operator_visible(dimension, row, column) {
                    entry.push((
                        format!("operador {row},{column}"),
                        layout.operators[row][column],
                    ));
                }
            }
        }
        let entry: Vec<(&str, Rect)> = entry
            .iter()
            .map(|(name, rect)| (name.as_str(), *rect))
            .collect();
        assert_disjoint(&context, &entry);
        let common = [
            ("rotulo", layout.solution_label),
            ("nota", layout.note),
            ("copiar", layout.copy),
        ];
        let empty = [
            ("icone vazio", layout.empty_icon),
            ("titulo vazio", layout.empty_title),
            ("descricao vazia", layout.empty_description),
        ];
        let unique: Vec<(&str, Rect)> = [
            ("titulo", layout.solution_title),
            ("descricao", layout.solution_description),
        ]
        .into_iter()
        .chain((0..dimension).map(|index| ("tile", layout.tiles[index])))
        .collect();
        let general = [
            ("titulo", layout.solution_title),
            ("descricao", layout.solution_description),
            ("geral", layout.general),
        ];
        assert_disjoint(&context, &[&common[..], &empty[..]].concat());
        assert_disjoint(&context, &[&common[..], &unique[..]].concat());
        assert_disjoint(&context, &[&common[..], &general[..]].concat());
        assert_disjoint(
            &context,
            &[
                ("logo", layout.logo),
                ("marca", layout.brand),
                ("autor", layout.author),
            ],
        );
        assert_disjoint(
            &context,
            &[
                ("intro", layout.intro),
                ("subtitulo", layout.subtitle),
                ("entrada", layout.entry),
                ("resultado", layout.solution),
                ("rodape", layout.footer),
                ("atalhos", layout.shortcut),
            ],
        );
    }
}

#[test]
fn referencia_a_96_dpi() {
    let layout = layout::compute(
        1.0,
        1143,
        793,
        2,
        Metrics {
            input_height: 21,
            value_height: 41,
        },
    );
    assert_eq!(layout.author, Rect::new(1143 - 344, 28, 312, 42));
    assert_eq!(layout.entry, Rect::new(32, 238, 624, 493));
    assert_eq!(layout.solution, Rect::new(676, 238, 435, 493));
    assert_eq!(layout.separator, Rect::new(32, 86, 1080, 1));
    assert_eq!(layout.fields[0][0], Rect::new(87, 432, 163, 46));
    assert_eq!(layout.editors[0][0], Rect::new(95, 444, 127, 21));
    assert_eq!(layout.fields[0][3], Rect::new(465, 432, 163, 46));
    assert_eq!(layout.solve, Rect::new(58, 667, 192, 42));
    assert_eq!(layout.clear, Rect::new(541, 667, 89, 42));
    assert_eq!(layout.copy, Rect::new(702, 667, 383, 42));
    assert_eq!(layout.tile_values[0], Rect::new(770, 413, 295, 41));
    assert_eq!(layout.footer, Rect::new(33, 750, 470, 25));
    assert_eq!(layout.shortcut, Rect::new(658, 750, 450, 25));
    let three = layout::compute(
        1.0,
        1143,
        793,
        3,
        Metrics {
            input_height: 21,
            value_height: 41,
        },
    );
    assert_eq!(three.fields[0][0].w, 116);
    assert_eq!(
        three.fields[2][3],
        Rect::new(32 + 55 + 3 * (116 + 26), 238 + 194 + 100, 116, 46)
    );
}

#[test]
fn area_rolavel_como_no_csharp() {
    assert_eq!(layout::content_size(1.0, 1160, 810, 17, 17), (1143, 793));
    assert_eq!(layout::content_size(1.0, 607, 424, 17, 17), (1040, 760));
    assert_eq!(layout::content_size(1.5, 1740, 1215, 26, 26), (1142, 792));
    let full = layout::viewport(1.0, 1160, 810, 17, 17);
    assert!(!full.horizontal && !full.vertical);
    assert_eq!((full.width, full.height), (1143, 793));
    let small = layout::viewport(1.0, 624, 441, 17, 17);
    assert!(small.horizontal && small.vertical);
    assert_eq!((small.client_w, small.client_h), (607, 424));
    assert_eq!((small.content_w, small.content_h), (1040, 760));
    assert_eq!(small.max_scroll(), (1040 - 607, 760 - 424));
}
