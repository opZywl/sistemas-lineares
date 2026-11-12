use num_traits::One;

use crate::{Rational, ShapeError, Solution, SolutionKind, VARIABLE_NAMES, format, solve};

pub const ROW_NAMES: [&str; 3] = ["L₁", "L₂", "L₃"];
pub const MINUS: char = '\u{2212}';

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Method {
    #[default]
    GaussJordan,
    Cramer,
    Gauss,
}

impl Method {
    pub const ALL: [Method; 3] = [Method::GaussJordan, Method::Cramer, Method::Gauss];

    pub const fn name(self) -> &'static str {
        match self {
            Method::GaussJordan => "Gauss-Jordan",
            Method::Cramer => "Regra de Cramer",
            Method::Gauss => "Escalonamento de Gauss",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Method::GaussJordan => "Gauss-Jordan",
            Method::Cramer => "Cramer",
            Method::Gauss => "Escalonamento",
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Method::GaussJordan => "gauss-jordan",
            Method::Cramer => "cramer",
            Method::Gauss => "gauss",
        }
    }

    pub fn from_key(key: &str) -> Option<Method> {
        Method::ALL.into_iter().find(|method| method.key() == key)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowOp {
    Swap {
        first: usize,
        second: usize,
    },
    Divide {
        row: usize,
        divisor: Rational,
    },
    Eliminate {
        target: usize,
        source: usize,
        factor: Rational,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineKind {
    Operation,
    Calculation,
    Result,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MathLine {
    pub kind: LineKind,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrixView {
    pub label: Option<String>,
    pub rows: Vec<Vec<Rational>>,
    pub augmented: bool,
    pub marked_rows: Vec<usize>,
    pub marked_column: Option<usize>,
    pub pivot: Option<(usize, usize)>,
}

impl MatrixView {
    pub fn cells(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|row| row.iter().map(number).collect())
            .collect()
    }

    pub fn text_lines(&self) -> Vec<String> {
        let cells = self.cells();
        let columns = cells.first().map_or(0, Vec::len);
        let widths: Vec<usize> = (0..columns)
            .map(|column| {
                cells
                    .iter()
                    .map(|row| row[column].chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let mut lines = Vec::with_capacity(cells.len() + 1);
        if let Some(label) = &self.label {
            lines.push(format!("{label} ="));
        }
        for row in &cells {
            let mut line = String::from("[");
            for (column, cell) in row.iter().enumerate() {
                if self.augmented && column + 1 == columns {
                    line.push_str(" |");
                }
                line.push(' ');
                for _ in cell.chars().count()..widths[column] {
                    line.push(' ');
                }
                line.push_str(cell);
            }
            line.push_str(" ]");
            lines.push(line);
        }
        lines
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub title: String,
    pub reasoning: String,
    pub lines: Vec<MathLine>,
    pub matrix: Option<MatrixView>,
    pub matrix_first: bool,
    pub operations: Vec<RowOp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walkthrough {
    pub method: Method,
    pub kind: SolutionKind,
    pub system: Vec<String>,
    pub steps: Vec<Step>,
    pub answer: Vec<String>,
    pub classification: &'static str,
    pub determinants: Vec<(String, Rational)>,
}

impl Walkthrough {
    pub fn text(&self) -> String {
        let mut out = format!("Modo pensar · {}\n\nSistema\n", self.method.name());
        for equation in &self.system {
            out.push_str(equation);
            out.push('\n');
        }
        for (index, step) in self.steps.iter().enumerate() {
            out.push('\n');
            out.push_str(&format!("Passo {} · {}\n", index + 1, step.title));
            out.push_str(&step.reasoning);
            out.push('\n');
            let matrix: Vec<String> = step
                .matrix
                .as_ref()
                .map(MatrixView::text_lines)
                .unwrap_or_default();
            if step.matrix_first {
                for line in &matrix {
                    out.push_str(line);
                    out.push('\n');
                }
            }
            for line in &step.lines {
                out.push_str(&line.text);
                out.push('\n');
            }
            if !step.matrix_first {
                for line in &matrix {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        out.push_str("\nResposta\n");
        for line in &self.answer {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(self.classification);
        out
    }
}

pub fn number(value: &Rational) -> String {
    value.to_string().replace('-', "\u{2212}")
}

pub fn factor(value: &Rational) -> String {
    if value.is_negative() || !is_integer(value) {
        format!("({})", number(value))
    } else {
        number(value)
    }
}

pub fn dividend(value: &Rational) -> String {
    if is_integer(value) {
        number(value)
    } else {
        format!("({})", number(value))
    }
}

fn is_integer(value: &Rational) -> bool {
    value.denominator().is_one()
}

fn magnitude(value: &Rational) -> Rational {
    if value.is_negative() {
        -value
    } else {
        value.clone()
    }
}

fn coefficient_prefix(magnitude: &Rational) -> String {
    if magnitude.is_one() {
        String::new()
    } else if is_integer(magnitude) {
        magnitude.to_string()
    } else {
        format!("({magnitude})")
    }
}

pub fn left_side(coefficients: &[Rational]) -> String {
    let mut text = String::new();
    for (column, coefficient) in coefficients.iter().enumerate() {
        if coefficient.is_zero() {
            continue;
        }
        let negative = coefficient.is_negative();
        let size = magnitude(coefficient);
        if text.is_empty() {
            if negative {
                text.push(MINUS);
            }
        } else {
            text.push_str(if negative { " \u{2212} " } else { " + " });
        }
        text.push_str(&coefficient_prefix(&size));
        text.push_str(VARIABLE_NAMES[column]);
    }
    if text.is_empty() {
        text.push('0');
    }
    text
}

pub fn equation(row: &[Rational]) -> String {
    let count = row.len() - 1;
    format!("{} = {}", left_side(&row[..count]), number(&row[count]))
}

fn vector(row: &[Rational]) -> String {
    let count = row.len() - 1;
    let left: Vec<String> = row[..count].iter().map(number).collect();
    format!("({} | {})", left.join(", "), number(&row[count]))
}

fn signed_sum(values: &[Rational]) -> String {
    let mut text = String::new();
    for (index, value) in values.iter().enumerate() {
        if index == 0 {
            text.push_str(&number(value));
        } else if value.is_negative() {
            text.push_str(" \u{2212} ");
            text.push_str(&number(&-value));
        } else {
            text.push_str(" + ");
            text.push_str(&number(value));
        }
    }
    text
}

fn sum(values: &[Rational]) -> Rational {
    values
        .iter()
        .fold(Rational::zero(), |total, value| &total + value)
}

fn list(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        count => format!("{} e {}", items[..count - 1].join(", "), items[count - 1]),
    }
}

fn names(count: usize) -> String {
    let items: Vec<String> = VARIABLE_NAMES[..count]
        .iter()
        .map(|name| name.to_string())
        .collect();
    list(&items)
}

fn rows_list(range: std::ops::Range<usize>) -> String {
    let items: Vec<String> = ROW_NAMES[range]
        .iter()
        .map(|name| name.to_string())
        .collect();
    list(&items)
}

fn line(kind: LineKind, text: String) -> MathLine {
    MathLine { kind, text }
}

fn view(matrix: &[Vec<Rational>]) -> MatrixView {
    MatrixView {
        label: None,
        rows: matrix.to_vec(),
        augmented: true,
        marked_rows: Vec::new(),
        marked_column: None,
        pivot: None,
    }
}

fn step(title: String, reasoning: String) -> Step {
    Step {
        title,
        reasoning,
        lines: Vec::new(),
        matrix: None,
        matrix_first: false,
        operations: Vec::new(),
    }
}

fn operation_text(target: usize, source: usize, factor_value: &Rational) -> String {
    let negative = factor_value.is_negative();
    let size = magnitude(factor_value);
    let multiple = if size.is_one() {
        ROW_NAMES[source].to_string()
    } else {
        format!("{}·{}", factor(&size), ROW_NAMES[source])
    };
    format!(
        "{0} ← {0} {1} {2}",
        ROW_NAMES[target],
        if negative { '+' } else { MINUS },
        multiple
    )
}

fn elimination_text(
    before: &[Rational],
    source: &[Rational],
    factor_value: &Rational,
    after: &[Rational],
) -> String {
    let negative = factor_value.is_negative();
    let size = magnitude(factor_value);
    let multiple = if size.is_one() {
        String::new()
    } else {
        format!("{}·", factor(&size))
    };
    format!(
        "{} {} {}{} = {}",
        vector(before),
        if negative { '+' } else { MINUS },
        multiple,
        vector(source),
        vector(after)
    )
}

fn subtract_multiple(matrix: &mut [Vec<Rational>], target: usize, source: usize, value: &Rational) {
    let pivot_row = matrix[source].clone();
    for (entry, pivot_entry) in matrix[target].iter_mut().zip(&pivot_row) {
        *entry = &*entry - &(value * pivot_entry);
    }
}

fn first_inconsistent(matrix: &[Vec<Rational>]) -> Option<usize> {
    let count = matrix.len();
    matrix
        .iter()
        .position(|row| row[..count].iter().all(Rational::is_zero) && !row[count].is_zero())
}

fn no_pivot_step(matrix: &[Vec<Rational>], pivot_row: usize, column: usize) -> Step {
    let count = matrix.len();
    let name = VARIABLE_NAMES[column];
    let next = if column + 1 < count {
        "Passamos para a próxima coluna. "
    } else {
        ""
    };
    let mut result = step(
        format!("Coluna de {name}: sem pivô"),
        format!(
            "Em {}, a coluna de {name} só tem zeros: não há como escolher um pivô nela. {next}Se o sistema tiver solução, {name} será uma variável livre, que pode assumir qualquer valor.",
            rows_list(pivot_row..count)
        ),
    );
    result.matrix = Some(MatrixView {
        marked_column: Some(column),
        ..view(matrix)
    });
    result
}

fn swap_step(matrix: &[Vec<Rational>], pivot_row: usize, selected: usize, column: usize) -> Step {
    let (top, bottom) = (ROW_NAMES[pivot_row], ROW_NAMES[selected]);
    let name = VARIABLE_NAMES[column];
    let mut result = step(
        format!("Trocar {top} e {bottom}"),
        format!(
            "Na coluna de {name}, o pivô deveria ficar em {top}, mas ali o valor é 0. Trocamos {top} com {bottom}, a primeira linha abaixo com valor diferente de zero nessa coluna. Mudar a ordem das equações não muda a solução."
        ),
    );
    result
        .lines
        .push(line(LineKind::Operation, format!("{top} ↔ {bottom}")));
    result.matrix = Some(MatrixView {
        marked_rows: vec![pivot_row, selected],
        pivot: Some((pivot_row, column)),
        ..view(matrix)
    });
    result.operations.push(RowOp::Swap {
        first: pivot_row,
        second: selected,
    });
    result
}

fn opening_step(matrix: &[Vec<Rational>], method: Method) -> Step {
    let count = matrix.len();
    let goal = match method {
        Method::Gauss => {
            "No escalonamento, zeramos tudo o que fica abaixo dos pivôs, coluna por coluna, e depois resolvemos o sistema de baixo para cima. Trocar linhas e somar a uma linha um múltiplo de outra não mudam as soluções."
        }
        _ => {
            "A meta é transformar a parte da esquerda na matriz identidade (1 na diagonal e 0 no resto), ou chegar o mais perto possível dela, usando operações de linha. Trocar linhas, dividir uma linha por um número diferente de zero e somar a uma linha um múltiplo de outra não mudam as soluções."
        }
    };
    let mut result = step(
        "Montar a matriz aumentada".to_string(),
        format!(
            "Cada linha é uma equação: à esquerda da barra ficam os coeficientes de {} e, à direita, os termos independentes. {goal}",
            names(count)
        ),
    );
    result.matrix = Some(view(matrix));
    result
}

struct Reduced {
    matrix: Vec<Vec<Rational>>,
    pivots: Vec<usize>,
}

fn gauss_jordan_elimination(steps: &mut Vec<Step>, input: &[Vec<Rational>]) -> Reduced {
    let count = input.len();
    let mut matrix = input.to_vec();
    steps.push(opening_step(&matrix, Method::GaussJordan));
    let mut pivots = Vec::with_capacity(count);
    let mut pivot_row = 0;
    for column in 0..count {
        if pivot_row == count {
            break;
        }
        let name = VARIABLE_NAMES[column];
        let row_name = ROW_NAMES[pivot_row];
        let Some(selected) = (pivot_row..count).find(|&row| !matrix[row][column].is_zero()) else {
            steps.push(no_pivot_step(&matrix, pivot_row, column));
            continue;
        };
        let first = steps.len();
        let swapped = selected != pivot_row;
        if swapped {
            matrix.swap(pivot_row, selected);
            steps.push(swap_step(&matrix, pivot_row, selected, column));
        }
        let pivot = matrix[pivot_row][column].clone();
        let normalized = !pivot.is_one();
        if normalized {
            let before = matrix[pivot_row].clone();
            for value in &mut matrix[pivot_row] {
                *value = &*value / &pivot;
            }
            let mut result = step(
                format!("Fazer o pivô de {name} valer 1"),
                format!(
                    "O pivô da coluna de {name} é {}, em {row_name}. Dividimos a linha inteira por {} para que ele vire 1. Dividir uma equação inteira por um número diferente de zero não muda a solução.",
                    number(&pivot),
                    number(&pivot)
                ),
            );
            result.lines.push(line(
                LineKind::Operation,
                format!("{row_name} ← {row_name} ÷ {}", factor(&pivot)),
            ));
            result.lines.push(line(
                LineKind::Calculation,
                format!(
                    "{} ÷ {} = {}",
                    vector(&before),
                    factor(&pivot),
                    vector(&matrix[pivot_row])
                ),
            ));
            result.matrix = Some(MatrixView {
                marked_rows: vec![pivot_row],
                pivot: Some((pivot_row, column)),
                ..view(&matrix)
            });
            result.operations.push(RowOp::Divide {
                row: pivot_row,
                divisor: pivot.clone(),
            });
            steps.push(result);
        }
        let targets: Vec<usize> = (0..count)
            .filter(|&row| row != pivot_row && !matrix[row][column].is_zero())
            .collect();
        if targets.is_empty() {
            if steps.len() > first {
                if let Some(last) = steps.last_mut() {
                    last.reasoning
                        .push_str(&format!(" As outras linhas já têm 0 na coluna de {name}."));
                }
            } else {
                let mut result = step(
                    format!("Coluna de {name} pronta"),
                    format!(
                        "O pivô de {name}, em {row_name}, já vale 1 e as outras linhas já têm 0 nessa coluna: não há conta a fazer aqui."
                    ),
                );
                result.matrix = Some(MatrixView {
                    pivot: Some((pivot_row, column)),
                    ..view(&matrix)
                });
                steps.push(result);
            }
        } else {
            let note = if normalized {
                String::new()
            } else if swapped {
                format!("Depois da troca, o pivô de {name} já vale 1. ")
            } else {
                format!("O pivô de {name}, em {row_name}, já vale 1. ")
            };
            let mut result = step(
                format!("Zerar a coluna de {name}"),
                format!(
                    "{note}Agora usamos {row_name} para zerar {name} nas outras linhas: de cada uma subtraímos (valor de {name} nela) × {row_name}. Como o pivô vale 1, o {name} dessas linhas vira 0."
                ),
            );
            for &target in &targets {
                let value = matrix[target][column].clone();
                let before = matrix[target].clone();
                subtract_multiple(&mut matrix, target, pivot_row, &value);
                result.lines.push(line(
                    LineKind::Operation,
                    operation_text(target, pivot_row, &value),
                ));
                result.lines.push(line(
                    LineKind::Calculation,
                    elimination_text(&before, &matrix[pivot_row], &value, &matrix[target]),
                ));
                result.operations.push(RowOp::Eliminate {
                    target,
                    source: pivot_row,
                    factor: value,
                });
            }
            result.matrix = Some(MatrixView {
                marked_rows: targets,
                pivot: Some((pivot_row, column)),
                ..view(&matrix)
            });
            steps.push(result);
        }
        pivots.push(column);
        pivot_row += 1;
    }
    Reduced { matrix, pivots }
}

fn contradiction_step(matrix: &[Vec<Rational>], row: usize) -> Step {
    let count = matrix.len();
    let constant = number(&matrix[row][count]);
    let zeros: Vec<String> = VARIABLE_NAMES[..count]
        .iter()
        .map(|name| format!("0{name}"))
        .collect();
    let mut result = step(
        "Achar a contradição".to_string(),
        format!(
            "A linha {} ficou com todos os coeficientes iguais a 0 e termo independente {constant}. Ela diz que 0 = {constant}, o que é impossível. Nenhum valor de {} satisfaz todas as equações ao mesmo tempo: o sistema é impossível (SI).",
            ROW_NAMES[row],
            names(count)
        ),
    );
    result.lines.push(line(
        LineKind::Calculation,
        format!(
            "{}: {} = {constant}  →  0 = {constant}",
            ROW_NAMES[row],
            zeros.join(" + ")
        ),
    ));
    result
        .lines
        .push(line(LineKind::Result, "Sem solução".to_string()));
    result
}

fn answer_lines(solution: &Solution) -> Vec<String> {
    if solution.kind == SolutionKind::None {
        return vec!["Sem solução".to_string()];
    }
    let count = solution.reduced.len();
    format(solution)
        .split("\r\n")
        .skip(2)
        .take(count)
        .map(|text| text.replace('-', "\u{2212}"))
        .collect()
}

fn classification(kind: SolutionKind) -> &'static str {
    match kind {
        SolutionKind::Unique => "Sistema possível e determinado (SPD): solução única.",
        SolutionKind::Infinite => "Sistema possível e indeterminado (SPI): infinitas soluções.",
        SolutionKind::None => "Sistema impossível (SI): não tem solução.",
    }
}

fn free_columns(count: usize, pivots: &[usize]) -> Vec<usize> {
    (0..count)
        .filter(|column| !pivots.contains(column))
        .collect()
}

fn free_sentence(free: &[usize]) -> String {
    let items: Vec<String> = free
        .iter()
        .map(|&column| VARIABLE_NAMES[column].to_string())
        .collect();
    if free.len() == 1 {
        format!("{} ficou sem pivô e é uma variável livre", items[0])
    } else {
        format!("{} ficaram sem pivô e são variáveis livres", list(&items))
    }
}

fn parameter_sentence(free: &[usize], others: Option<&str>) -> String {
    let parameters: Vec<String> = (1..=free.len()).map(|index| format!("t{index}")).collect();
    let head = if free.len() == 1 {
        format!("A variável livre vira o parâmetro {}", parameters[0])
    } else {
        format!(
            "As variáveis livres viram os parâmetros {}",
            list(&parameters)
        )
    };
    match others {
        Some(others) => format!("{head}, e {others}."),
        None if free.len() == 1 => format!("{head}, que pode valer qualquer número."),
        None => format!("{head}, que podem valer qualquer número."),
    }
}

fn reading_step(reduced: &Reduced, solution: &Solution) -> Step {
    let count = reduced.matrix.len();
    let answer = answer_lines(solution);
    if solution.kind == SolutionKind::Unique {
        let mut result = step(
            "Ler a solução".to_string(),
            "A parte da esquerda virou a matriz identidade, então cada linha diz diretamente o valor de uma incógnita. O sistema é possível e determinado (SPD): tem uma única solução.".to_string(),
        );
        for text in answer {
            result.lines.push(line(LineKind::Result, text));
        }
        return result;
    }
    let free = free_columns(count, &reduced.pivots);
    let zero_rows = reduced
        .matrix
        .iter()
        .any(|row| row.iter().all(Rational::is_zero));
    let zero_note = if zero_rows {
        " As linhas só com zeros dizem 0 = 0: valem sempre e não impõem condição."
    } else {
        ""
    };
    let mut result = step(
        "Escrever a solução geral".to_string(),
        format!(
            "Não deu para ter um pivô em cada coluna: {}.{zero_note} {} O sistema é possível e indeterminado (SPI): tem infinitas soluções.",
            free_sentence(&free),
            parameter_sentence(
                &free,
                (!reduced.pivots.is_empty())
                    .then_some("as outras incógnitas são isoladas nas linhas com pivô")
            )
        ),
    );
    for (index, &column) in free.iter().enumerate() {
        result.lines.push(line(
            LineKind::Calculation,
            format!("{} = t{}  (livre)", VARIABLE_NAMES[column], index + 1),
        ));
    }
    for (row, &column) in reduced.pivots.iter().enumerate() {
        result.lines.push(line(
            LineKind::Calculation,
            format!(
                "{}: {}  →  {}",
                ROW_NAMES[row],
                equation(&reduced.matrix[row]),
                answer[column]
            ),
        ));
    }
    for text in answer {
        result.lines.push(line(LineKind::Result, text));
    }
    result
}

fn solution_values(solution: &Solution) -> Vec<Rational> {
    let count = solution.reduced.len();
    let mut values = vec![Rational::zero(); count];
    for (row, &column) in solution.pivot_columns.iter().enumerate() {
        values[column] = solution.reduced[row][count].clone();
    }
    values
}

fn assignments(values: &[Rational]) -> String {
    let items: Vec<String> = values
        .iter()
        .zip(VARIABLE_NAMES)
        .map(|(value, name)| format!("{name} = {}", number(value)))
        .collect();
    list(&items)
}

fn verification_step(input: &[Vec<Rational>], solution: &Solution) -> Option<Step> {
    let count = input.len();
    let values = solution_values(solution);
    let mut result = match solution.kind {
        SolutionKind::None => return None,
        SolutionKind::Unique => step(
            "Conferir nas equações originais".to_string(),
            format!(
                "Substituímos {} em cada equação do sistema original. Se todas fecharem, a resposta está certa.",
                assignments(&values)
            ),
        ),
        SolutionKind::Infinite => {
            let free = free_columns(count, &solution.pivot_columns);
            let parameters: Vec<String> = (1..=free.len())
                .map(|index| format!("t{index} = 0"))
                .collect();
            step(
                "Conferir uma solução particular".to_string(),
                format!(
                    "Cada escolha dos parâmetros dá uma solução. Com {}, ficamos com {}. Substituindo nas equações originais:",
                    list(&parameters),
                    assignments(&values)
                ),
            )
        }
    };
    for (index, row) in input.iter().enumerate() {
        let products: Vec<Rational> = row[..count]
            .iter()
            .zip(&values)
            .map(|(coefficient, value)| coefficient * value)
            .collect();
        let terms: Vec<String> = row[..count]
            .iter()
            .zip(&values)
            .map(|(coefficient, value)| format!("{}·{}", factor(coefficient), factor(value)))
            .collect();
        let total = sum(&products);
        let verdict = if total == row[count] {
            "(confere)".to_string()
        } else {
            format!("≠ {} (não confere)", number(&row[count]))
        };
        result.lines.push(line(
            LineKind::Calculation,
            format!(
                "{}ª equação: {} = {} = {} {verdict}",
                index + 1,
                terms.join(" + "),
                signed_sum(&products),
                number(&total)
            ),
        ));
    }
    Some(result)
}

fn finish_gauss_jordan(
    steps: &mut Vec<Step>,
    input: &[Vec<Rational>],
    reduced: &Reduced,
    solution: &Solution,
) -> SolutionKind {
    let count = input.len();
    let kind = if let Some(row) = first_inconsistent(&reduced.matrix) {
        steps.push(contradiction_step(&reduced.matrix, row));
        SolutionKind::None
    } else {
        let kind = if reduced.pivots.len() == count {
            SolutionKind::Unique
        } else {
            SolutionKind::Infinite
        };
        steps.push(reading_step(reduced, solution));
        kind
    };
    if let Some(check) = verification_step(input, solution) {
        steps.push(check);
    }
    kind
}

fn determinant_lines(name: &str, matrix: &[Vec<Rational>]) -> (Vec<MathLine>, Rational) {
    let mut lines = Vec::new();
    if matrix.len() == 2 {
        let main = &matrix[0][0] * &matrix[1][1];
        let secondary = &matrix[0][1] * &matrix[1][0];
        let value = &main - &secondary;
        lines.push(line(
            LineKind::Calculation,
            format!(
                "{name} = {}·{} − {}·{}",
                factor(&matrix[0][0]),
                factor(&matrix[1][1]),
                factor(&matrix[0][1]),
                factor(&matrix[1][0])
            ),
        ));
        lines.push(line(
            LineKind::Calculation,
            format!("{name} = {} − {}", number(&main), factor(&secondary)),
        ));
        lines.push(line(
            LineKind::Result,
            format!("{name} = {}", number(&value)),
        ));
        return (lines, value);
    }
    const DOWN: [[(usize, usize); 3]; 3] = [
        [(0, 0), (1, 1), (2, 2)],
        [(0, 1), (1, 2), (2, 0)],
        [(0, 2), (1, 0), (2, 1)],
    ];
    const UP: [[(usize, usize); 3]; 3] = [
        [(0, 2), (1, 1), (2, 0)],
        [(0, 0), (1, 2), (2, 1)],
        [(0, 1), (1, 0), (2, 2)],
    ];
    let expand = |diagonals: &[[(usize, usize); 3]; 3]| -> (String, Vec<Rational>) {
        let mut texts = Vec::with_capacity(3);
        let mut products = Vec::with_capacity(3);
        for diagonal in diagonals {
            let factors: Vec<String> = diagonal
                .iter()
                .map(|&(row, column)| factor(&matrix[row][column]))
                .collect();
            texts.push(factors.join("·"));
            products.push(
                diagonal
                    .iter()
                    .fold(Rational::one(), |total, &(row, column)| {
                        &total * &matrix[row][column]
                    }),
            );
        }
        (texts.join(" + "), products)
    };
    let (down_text, down) = expand(&DOWN);
    let (up_text, up) = expand(&UP);
    let down_total = sum(&down);
    let up_total = sum(&up);
    let value = &down_total - &up_total;
    lines.push(line(
        LineKind::Calculation,
        format!("{name} = ({down_text}) − ({up_text})"),
    ));
    lines.push(line(
        LineKind::Calculation,
        format!("{name} = ({}) − ({})", signed_sum(&down), signed_sum(&up)),
    ));
    lines.push(line(
        LineKind::Calculation,
        format!("{name} = {} − {}", number(&down_total), factor(&up_total)),
    ));
    lines.push(line(
        LineKind::Result,
        format!("{name} = {}", number(&value)),
    ));
    (lines, value)
}

fn cramer(
    steps: &mut Vec<Step>,
    input: &[Vec<Rational>],
    solution: &Solution,
    determinants: &mut Vec<(String, Rational)>,
) -> SolutionKind {
    let count = input.len();
    let coefficients: Vec<Vec<Rational>> = input.iter().map(|row| row[..count].to_vec()).collect();
    let constants: Vec<Rational> = input.iter().map(|row| row[count].clone()).collect();
    let mut opening = step(
        "Separar a matriz dos coeficientes".to_string(),
        format!(
            "A regra de Cramer usa determinantes. A matriz A guarda só os coeficientes de {}, e os termos independentes formam a coluna b.",
            names(count)
        ),
    );
    opening.matrix = Some(MatrixView {
        label: Some("A".to_string()),
        augmented: false,
        ..view(&coefficients)
    });
    opening.matrix_first = true;
    let constant_texts: Vec<String> = constants.iter().map(number).collect();
    opening.lines.push(line(
        LineKind::Calculation,
        format!("b = ({})", constant_texts.join(", ")),
    ));
    steps.push(opening);
    let (lines, determinant) = determinant_lines("D", &coefficients);
    let mut main = step(
        "Calcular D = det(A)".to_string(),
        if count == 2 {
            "Numa matriz 2×2, o determinante é o produto da diagonal principal menos o produto da diagonal secundária.".to_string()
        } else {
            "Numa matriz 3×3, usamos a regra de Sarrus: somamos os produtos das três diagonais que descem da esquerda para a direita e subtraímos os produtos das três que sobem.".to_string()
        },
    );
    main.lines = lines;
    steps.push(main);
    determinants.push(("D".to_string(), determinant.clone()));
    if determinant.is_zero() {
        steps.push(step(
            "D = 0: Cramer não decide".to_string(),
            "Com D = 0 não dá para dividir por D, então o sistema não tem solução única: ele pode ter infinitas soluções (SPI) ou nenhuma (SI). Para descobrir qual, escalonamos a matriz aumentada por Gauss-Jordan nos passos a seguir.".to_string(),
        ));
        let reduced = gauss_jordan_elimination(steps, input);
        return finish_gauss_jordan(steps, input, &reduced, solution);
    }
    let mut values = Vec::with_capacity(count);
    for column in 0..count {
        let name = VARIABLE_NAMES[column];
        let replaced: Vec<Vec<Rational>> = coefficients
            .iter()
            .zip(&constants)
            .map(|(row, constant)| {
                let mut row = row.clone();
                row[column] = constant.clone();
                row
            })
            .collect();
        let label = format!("D{name}");
        let (lines, value) = determinant_lines(&label, &replaced);
        let intro = if column == 0 {
            format!(
                "Como D = {} é diferente de 0, o sistema tem solução única (SPD) e a regra de Cramer se aplica. ",
                number(&determinant)
            )
        } else {
            String::new()
        };
        let mut result = step(
            format!("Calcular {label}"),
            format!(
                "{intro}Para {name}, trocamos a coluna de {name} em A pelos termos independentes b e calculamos o determinante {label}."
            ),
        );
        result.matrix = Some(MatrixView {
            augmented: false,
            marked_column: Some(column),
            ..view(&replaced)
        });
        result.matrix_first = true;
        result.lines = lines;
        steps.push(result);
        determinants.push((label, value.clone()));
        values.push(value);
    }
    let mut division = step(
        "Dividir pelos determinantes".to_string(),
        "Pela regra de Cramer, cada incógnita é o seu determinante dividido por D.".to_string(),
    );
    for (column, value) in values.iter().enumerate() {
        let name = VARIABLE_NAMES[column];
        division.lines.push(line(
            LineKind::Result,
            format!(
                "{name} = D{name} ÷ D = {} ÷ {} = {}",
                dividend(value),
                factor(&determinant),
                number(&(value / &determinant))
            ),
        ));
    }
    steps.push(division);
    if let Some(check) = verification_step(input, solution) {
        steps.push(check);
    }
    SolutionKind::Unique
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Affine {
    constant: Rational,
    terms: Vec<Rational>,
}

impl Affine {
    fn number(value: Rational, parameters: usize) -> Affine {
        Affine {
            constant: value,
            terms: vec![Rational::zero(); parameters],
        }
    }

    fn parameter(index: usize, parameters: usize) -> Affine {
        let mut terms = vec![Rational::zero(); parameters];
        terms[index] = Rational::one();
        Affine {
            constant: Rational::zero(),
            terms,
        }
    }

    fn is_number(&self) -> bool {
        self.terms.iter().all(Rational::is_zero)
    }

    fn is_parameter(&self) -> bool {
        self.constant.is_zero()
            && self.terms.iter().filter(|term| !term.is_zero()).count() == 1
            && self.terms.iter().any(Rational::is_one)
    }

    fn add(&self, other: &Affine) -> Affine {
        Affine {
            constant: &self.constant + &other.constant,
            terms: self
                .terms
                .iter()
                .zip(&other.terms)
                .map(|(first, second)| first + second)
                .collect(),
        }
    }

    fn scale(&self, value: &Rational) -> Affine {
        Affine {
            constant: &self.constant * value,
            terms: self.terms.iter().map(|term| term * value).collect(),
        }
    }

    fn text(&self) -> String {
        let mut text = String::new();
        let mut has_term = !self.constant.is_zero();
        if has_term {
            text.push_str(&number(&self.constant));
        }
        for (index, coefficient) in self.terms.iter().enumerate() {
            if coefficient.is_zero() {
                continue;
            }
            let negative = coefficient.is_negative();
            let size = magnitude(coefficient);
            if has_term {
                text.push_str(if negative { " \u{2212} " } else { " + " });
            } else if negative {
                text.push(MINUS);
            }
            if !size.is_one() {
                text.push_str(&size.to_string());
                text.push_str(" · ");
            }
            text.push_str(&format!("t{}", index + 1));
            has_term = true;
        }
        if !has_term {
            text.push('0');
        }
        text
    }

    fn operand(&self) -> String {
        if self.is_number() {
            factor(&self.constant)
        } else if self.is_parameter() {
            self.text()
        } else {
            format!("({})", self.text())
        }
    }

    fn dividend(&self) -> String {
        if self.is_number() {
            dividend(&self.constant)
        } else if self.is_parameter() {
            self.text()
        } else {
            format!("({})", self.text())
        }
    }

    fn subtrahend(&self) -> String {
        let single_positive = self.constant.is_zero()
            && self.terms.iter().filter(|term| !term.is_zero()).count() == 1
            && self.terms.iter().all(|term| !term.is_negative());
        if self.is_number() {
            factor(&self.constant)
        } else if single_positive {
            self.text()
        } else {
            format!("({})", self.text())
        }
    }
}

fn pivot_term(pivot: &Rational, column: usize) -> String {
    let negative = pivot.is_negative();
    let mut text = String::new();
    if negative {
        text.push(MINUS);
    }
    text.push_str(&coefficient_prefix(&magnitude(pivot)));
    text.push_str(VARIABLE_NAMES[column]);
    text
}

fn back_substitution_step(
    matrix: &[Vec<Rational>],
    pivot_row: usize,
    column: usize,
    values: &[Option<Affine>],
    parameters: usize,
    first: bool,
) -> (Step, Affine) {
    let count = matrix.len();
    let row = &matrix[pivot_row];
    let name = VARIABLE_NAMES[column];
    let row_name = ROW_NAMES[pivot_row];
    let pivot = &row[column];
    let constant = &row[count];
    let known: Vec<(usize, &Rational, &Affine)> = ((column + 1)..count)
        .filter(|&other| !row[other].is_zero())
        .filter_map(|other| {
            values[other]
                .as_ref()
                .map(|value| (other, &row[other], value))
        })
        .collect();
    let known_names: Vec<String> = known
        .iter()
        .map(|(other, _, _)| VARIABLE_NAMES[*other].to_string())
        .collect();
    let replaced = if known.len() == 1 {
        format!("trocamos {} pelo valor que já temos", known_names[0])
    } else {
        format!("trocamos {} pelos valores que já temos", list(&known_names))
    };
    let opening = if first {
        format!("Começamos de baixo, por {row_name}")
    } else {
        format!("Subimos para {row_name}")
    };
    let reasoning = if known.is_empty() && pivot.is_one() {
        format!("{opening}, que já diz diretamente o valor de {name}.")
    } else if known.is_empty() {
        format!("{opening}, que só tem a incógnita {name}: basta dividir pelo coeficiente dela.")
    } else {
        format!("{opening}: {replaced} e isolamos {name}.")
    };
    let mut result = step(format!("Encontrar {name}"), reasoning);
    if known.is_empty() && pivot.is_one() {
        let value = Affine::number(constant.clone(), parameters);
        result.lines.push(line(
            LineKind::Result,
            format!("{row_name}: {name} = {}", value.text()),
        ));
        return (result, value);
    }
    result.lines.push(line(
        LineKind::Calculation,
        format!("{row_name}: {}", equation(row)),
    ));
    let lead = pivot_term(pivot, column);
    let mut remainder = Affine::number(constant.clone(), parameters);
    if !known.is_empty() {
        let mut substituted = lead.clone();
        let mut total = Affine::number(Rational::zero(), parameters);
        let mut products = Vec::with_capacity(known.len());
        for (_, coefficient, known_value) in &known {
            let negative = coefficient.is_negative();
            let size = magnitude(coefficient);
            substituted.push_str(if negative { " \u{2212} " } else { " + " });
            substituted.push_str(&format!("{}·{}", factor(&size), known_value.operand()));
            let product = known_value.scale(coefficient);
            total = total.add(&product);
            products.push(product);
        }
        result.lines.push(line(
            LineKind::Calculation,
            format!("{substituted} = {}", number(constant)),
        ));
        remainder =
            Affine::number(constant.clone(), parameters).add(&total.scale(&-Rational::one()));
        let text = if total.is_number() && products.len() > 1 {
            let parts: Vec<Rational> = products
                .iter()
                .map(|product| product.constant.clone())
                .collect();
            format!(
                "{lead} = {} − ({}) = {} − {} = {}",
                number(constant),
                signed_sum(&parts),
                number(constant),
                factor(&total.constant),
                remainder.text()
            )
        } else {
            let difference = format!("{} − {}", number(constant), total.subtrahend());
            if difference == remainder.text() {
                format!("{lead} = {difference}")
            } else {
                format!("{lead} = {difference} = {}", remainder.text())
            }
        };
        result.lines.push(line(
            if pivot.is_one() {
                LineKind::Result
            } else {
                LineKind::Calculation
            },
            text,
        ));
    }
    let value = remainder.scale(&(&Rational::one() / pivot));
    if !pivot.is_one() {
        result.lines.push(line(
            LineKind::Result,
            format!(
                "{name} = {} ÷ {} = {}",
                remainder.dividend(),
                factor(pivot),
                value.text()
            ),
        ));
    }
    (result, value)
}

fn gauss(steps: &mut Vec<Step>, input: &[Vec<Rational>], solution: &Solution) -> SolutionKind {
    let count = input.len();
    let mut matrix = input.to_vec();
    steps.push(opening_step(&matrix, Method::Gauss));
    let mut pivots: Vec<(usize, usize)> = Vec::with_capacity(count);
    let mut pivot_row = 0;
    for column in 0..count {
        if pivot_row == count {
            break;
        }
        let name = VARIABLE_NAMES[column];
        let row_name = ROW_NAMES[pivot_row];
        let Some(selected) = (pivot_row..count).find(|&row| !matrix[row][column].is_zero()) else {
            steps.push(no_pivot_step(&matrix, pivot_row, column));
            continue;
        };
        let first = steps.len();
        if selected != pivot_row {
            matrix.swap(pivot_row, selected);
            steps.push(swap_step(&matrix, pivot_row, selected, column));
        }
        let pivot = matrix[pivot_row][column].clone();
        let targets: Vec<usize> = ((pivot_row + 1)..count)
            .filter(|&row| !matrix[row][column].is_zero())
            .collect();
        if targets.is_empty() {
            if steps.len() > first {
                if let Some(last) = steps.last_mut() {
                    last.reasoning
                        .push_str(" Abaixo do pivô já está tudo zerado.");
                }
            } else if pivot_row + 1 < count {
                let mut result = step(
                    format!("Coluna de {name} pronta"),
                    format!(
                        "O pivô da coluna de {name} é {}, em {row_name}, e abaixo dele já está tudo zerado: não há conta a fazer aqui.",
                        number(&pivot)
                    ),
                );
                result.matrix = Some(MatrixView {
                    pivot: Some((pivot_row, column)),
                    ..view(&matrix)
                });
                steps.push(result);
            }
        } else {
            let mut result = step(
                format!("Zerar {name} abaixo do pivô"),
                format!(
                    "O pivô da coluna de {name} é {}, em {row_name}. Para cada linha abaixo dele, calculamos o multiplicador m = (valor de {name} na linha) ÷ pivô e fazemos linha ← linha − m·{row_name}. Assim o {name} dessas linhas vira 0.",
                    number(&pivot)
                ),
            );
            for &target in &targets {
                let entry = matrix[target][column].clone();
                let multiplier = &entry / &pivot;
                let before = matrix[target].clone();
                subtract_multiple(&mut matrix, target, pivot_row, &multiplier);
                result.lines.push(line(
                    LineKind::Calculation,
                    format!(
                        "m = {} ÷ {} = {}",
                        dividend(&entry),
                        factor(&pivot),
                        number(&multiplier)
                    ),
                ));
                result.lines.push(line(
                    LineKind::Operation,
                    operation_text(target, pivot_row, &multiplier),
                ));
                result.lines.push(line(
                    LineKind::Calculation,
                    elimination_text(&before, &matrix[pivot_row], &multiplier, &matrix[target]),
                ));
                result.operations.push(RowOp::Eliminate {
                    target,
                    source: pivot_row,
                    factor: multiplier,
                });
            }
            result.matrix = Some(MatrixView {
                marked_rows: targets,
                pivot: Some((pivot_row, column)),
                ..view(&matrix)
            });
            steps.push(result);
        }
        pivots.push((pivot_row, column));
        pivot_row += 1;
    }
    let mut echelon = step(
        "Forma escalonada".to_string(),
        "Abaixo de cada pivô só há zeros: a matriz está escalonada. Escrevendo cada linha de novo como equação:".to_string(),
    );
    for (row, values) in matrix.iter().enumerate() {
        echelon.lines.push(line(
            LineKind::Calculation,
            format!("{}: {}", ROW_NAMES[row], equation(values)),
        ));
    }
    steps.push(echelon);
    if let Some(row) = first_inconsistent(&matrix) {
        steps.push(contradiction_step(&matrix, row));
        return SolutionKind::None;
    }
    let pivot_columns: Vec<usize> = pivots.iter().map(|&(_, column)| column).collect();
    let free = free_columns(count, &pivot_columns);
    let parameters = free.len();
    let mut values: Vec<Option<Affine>> = vec![None; count];
    if !free.is_empty() {
        let mut result = step(
            "Variáveis livres".to_string(),
            format!(
                "Na forma escalonada, {}. {} O sistema é possível e indeterminado (SPI).",
                free_sentence(&free),
                parameter_sentence(
                    &free,
                    (!pivots.is_empty()).then_some(
                        "as outras incógnitas saem por substituição, de baixo para cima"
                    )
                )
            ),
        );
        for (index, &column) in free.iter().enumerate() {
            values[column] = Some(Affine::parameter(index, parameters));
            result.lines.push(line(
                LineKind::Calculation,
                format!("{} = t{}", VARIABLE_NAMES[column], index + 1),
            ));
        }
        steps.push(result);
    }
    for (position, &(row, column)) in pivots.iter().enumerate().rev() {
        let (result, value) = back_substitution_step(
            &matrix,
            row,
            column,
            &values,
            parameters,
            position + 1 == pivots.len(),
        );
        values[column] = Some(value);
        steps.push(result);
    }
    let kind = if free.is_empty() {
        SolutionKind::Unique
    } else {
        SolutionKind::Infinite
    };
    let mut answer = step(
        "Juntar a resposta".to_string(),
        classification(kind).to_string(),
    );
    for (column, value) in values.iter().enumerate() {
        let text = value.as_ref().map_or_else(|| "0".to_string(), Affine::text);
        answer.lines.push(line(
            LineKind::Result,
            format!("{} = {text}", VARIABLE_NAMES[column]),
        ));
    }
    steps.push(answer);
    if let Some(check) = verification_step(input, solution) {
        steps.push(check);
    }
    kind
}

pub fn walkthrough(input: &[Vec<Rational>], method: Method) -> Result<Walkthrough, ShapeError> {
    let solution = solve(input)?;
    let mut steps = Vec::new();
    let mut determinants = Vec::new();
    let kind = match method {
        Method::GaussJordan => {
            let reduced = gauss_jordan_elimination(&mut steps, input);
            finish_gauss_jordan(&mut steps, input, &reduced, &solution)
        }
        Method::Cramer => cramer(&mut steps, input, &solution, &mut determinants),
        Method::Gauss => gauss(&mut steps, input, &solution),
    };
    Ok(Walkthrough {
        method,
        kind,
        system: input.iter().map(|row| equation(row)).collect(),
        steps,
        answer: answer_lines(&solution),
        classification: classification(kind),
        determinants,
    })
}
