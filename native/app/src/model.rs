use sistemas_lineares_core::{Rational, SolutionKind, VARIABLE_NAMES, format, solve};

pub const WINDOW_TITLE: &str = "Sistemas Lineares · Lucas Lima";
pub const BRAND: &str = "Sistemas Lineares";
pub const AUTHOR_LABEL: &str = "Feito por Lucas Lima · RA 185964";
pub const AUTHOR_NAME: &str = "Feito por Lucas Lima";
pub const AUTHOR_SEPARATOR: &str = "·";
pub const AUTHOR_RA: &str = "RA 185964";
pub const AUTHOR_DESCRIPTION: &str = "Abre https://lucas-lima.xyz/ no navegador padrão.";
pub const AUTHOR_TOOLTIP: &str = "Visitar lucas-lima.xyz";
pub const AUTHOR_URL: &str = "https://lucas-lima.xyz/";
pub const INTRO: &str = "SISTEMAS\nLINEARES.";
pub const SUBTITLE: &str =
    "Insira os coeficientes. Confira a solução.\nCálculo exato por eliminação de Gauss-Jordan.";
pub const ENTRY_TITLE: &str = "Seu sistema";
pub const ENTRY_DESCRIPTION: &str = "Escolha o tamanho e preencha cada equação.";
pub const TWO_LABEL: &str = "2 × 2";
pub const THREE_LABEL: &str = "3 × 3";
pub const GRID_LABEL: &str = "COEFICIENTES E TERMOS INDEPENDENTES";
pub const SOLVE_LABEL: &str = "Resolver sistema";
pub const EXAMPLE_LABEL: &str = "Usar exemplo";
pub const CLEAR_LABEL: &str = "Limpar";
pub const RESULT_LABEL: &str = "RESULTADO";
pub const EMPTY_TITLE: &str = "Aguardando os valores";
pub const EMPTY_DESCRIPTION: &str =
    "Preencha o sistema ao lado ou use um exemplo.\nA resposta será exibida neste painel.";
pub const FOOTER: &str = "Gauss-Jordan  /  aritmética racional exata";
pub const SHORTCUTS: &str = "ENTER  para resolver     ·     TAB  para navegar";
pub const DEFAULT_HELPER: &str = "Aceita 2, -1,5 e 3/4. Use 0 se a variável não aparecer.";
pub const EXAMPLE_HELPER: &str = "Exemplo pronto. Clique em Resolver sistema ou pressione Enter.";
pub const AUTHOR_HELPER: &str = "Abra lucas-lima.xyz no seu navegador para visitar o autor.";
pub const NOTE_DEFAULT: &str = "Resultados exatos, incluindo frações.";
pub const NOTE_UNIQUE: &str = "Solução única · valores exatos";
pub const NOTE_INFINITE: &str = "Os parâmetros t1, t2, … são números reais livres.";
pub const NOTE_NONE: &str = "Confira os coeficientes se esse não era o esperado.";
pub const NOTE_COPY_FAILED: &str = "Não foi possível copiar agora. Tente novamente.";
pub const COPY_LABEL: &str = "Copiar resultado";
pub const COPIED_LABEL: &str = "Resultado copiado";
pub const TITLE_UNIQUE: &str = "Solução encontrada";
pub const TITLE_INFINITE: &str = "Infinitas soluções";
pub const TITLE_NONE: &str = "Sem solução";
pub const DESCRIPTION_UNIQUE: &str = "Estes valores satisfazem todas\nas equações do sistema.";
pub const DESCRIPTION_INFINITE: &str =
    "As equações são dependentes.\nVeja a forma geral da resposta:";
pub const DESCRIPTION_NONE: &str = "As equações são incompatíveis.";
pub const GENERAL_NONE: &str =
    "Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo.";

const EXAMPLE_TWO: [[&str; 3]; 2] = [["2", "1", "5"], ["1", "-1", "1"]];
const EXAMPLE_THREE: [[&str; 4]; 3] = [
    ["1", "1", "1", "6"],
    ["2", "-1", "1", "3"],
    ["1", "2", "-1", "2"],
];

pub const fn input_column(dimension: usize, column: usize) -> usize {
    if column == dimension { 3 } else { column }
}

pub fn term_name(column: usize) -> String {
    match VARIABLE_NAMES.get(column) {
        Some(name) => format!("coeficiente de {name}"),
        None => "termo independente".to_string(),
    }
}

pub fn accessible_input_name(row: usize, column: usize) -> String {
    format!("Equação {}, {}", row + 1, term_name(column))
}

pub fn accessible_value_name(index: usize) -> String {
    format!(
        "Valor de {}",
        VARIABLE_NAMES.get(index).copied().unwrap_or("x")
    )
}

pub fn row_number(row: usize) -> String {
    format!("{:02}", row + 1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Empty,
    Unique(Vec<String>),
    Infinite(String),
    NoSolution(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Field {
        row: usize,
        column: usize,
        select_all: bool,
    },
    Solve,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    dimension: usize,
    inputs: [[String; 4]; 3],
    helper: String,
    helper_error: bool,
    error_field: Option<(usize, usize)>,
    outcome: Outcome,
    last_result: Option<String>,
    note: String,
    copied: bool,
}

impl Default for Model {
    fn default() -> Self {
        Model::new()
    }
}

impl Model {
    pub fn new() -> Model {
        Model {
            dimension: 2,
            inputs: Default::default(),
            helper: DEFAULT_HELPER.to_string(),
            helper_error: false,
            error_field: None,
            outcome: Outcome::Empty,
            last_result: None,
            note: NOTE_DEFAULT.to_string(),
            copied: false,
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn input(&self, row: usize, column: usize) -> &str {
        self.inputs
            .get(row)
            .and_then(|values| values.get(column))
            .map_or("", String::as_str)
    }

    pub fn helper(&self) -> &str {
        &self.helper
    }

    pub fn helper_is_error(&self) -> bool {
        self.helper_error
    }

    pub fn field_has_error(&self, row: usize, column: usize) -> bool {
        self.error_field == Some((row, column))
    }

    pub fn error_field(&self) -> Option<(usize, usize)> {
        self.error_field
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    pub fn is_empty(&self) -> bool {
        self.outcome == Outcome::Empty
    }

    pub fn title(&self) -> Option<&'static str> {
        match self.outcome {
            Outcome::Empty => None,
            Outcome::Unique(_) => Some(TITLE_UNIQUE),
            Outcome::Infinite(_) => Some(TITLE_INFINITE),
            Outcome::NoSolution(_) => Some(TITLE_NONE),
        }
    }

    pub fn description(&self) -> Option<&'static str> {
        match self.outcome {
            Outcome::Empty => None,
            Outcome::Unique(_) => Some(DESCRIPTION_UNIQUE),
            Outcome::Infinite(_) => Some(DESCRIPTION_INFINITE),
            Outcome::NoSolution(_) => Some(DESCRIPTION_NONE),
        }
    }

    pub fn values(&self) -> &[String] {
        match &self.outcome {
            Outcome::Unique(values) => values,
            _ => &[],
        }
    }

    pub fn general(&self) -> Option<&str> {
        match &self.outcome {
            Outcome::Infinite(text) | Outcome::NoSolution(text) => Some(text),
            _ => None,
        }
    }

    pub fn last_result(&self) -> Option<&str> {
        self.last_result.as_deref()
    }

    pub fn note(&self) -> &str {
        &self.note
    }

    pub fn copy_enabled(&self) -> bool {
        self.last_result.is_some()
    }

    pub fn copy_label(&self) -> &'static str {
        if self.copied {
            COPIED_LABEL
        } else {
            COPY_LABEL
        }
    }

    pub fn announcement(&self) -> Option<String> {
        match &self.outcome {
            Outcome::Empty => None,
            Outcome::Unique(values) => {
                let parts: Vec<String> = values
                    .iter()
                    .zip(VARIABLE_NAMES)
                    .map(|(value, name)| format!("{name} = {value}"))
                    .collect();
                Some(format!("{TITLE_UNIQUE}: {}.", parts.join(", ")))
            }
            Outcome::Infinite(general) => {
                let parts: Vec<&str> = general
                    .lines()
                    .map(str::trim)
                    .filter(|line| line.contains(" = "))
                    .collect();
                Some(format!("{TITLE_INFINITE}: {}.", parts.join(", ")))
            }
            Outcome::NoSolution(_) => Some(format!("{TITLE_NONE}. {DESCRIPTION_NONE}")),
        }
    }

    pub fn dimension_text(&self) -> String {
        format!("{0} equações · {0} incógnitas", self.dimension)
    }

    pub fn set_input(&mut self, row: usize, column: usize, text: &str) {
        if self.store_input(row, column, text) {
            self.reset_errors();
            self.reset_result();
        }
    }

    pub fn store_input(&mut self, row: usize, column: usize, text: &str) -> bool {
        match self
            .inputs
            .get_mut(row)
            .and_then(|values| values.get_mut(column))
        {
            Some(slot) => {
                if **slot != *text {
                    *slot = text.to_string();
                }
                true
            }
            None => false,
        }
    }

    pub fn change_dimension(&mut self, dimension: usize) {
        self.dimension = dimension.clamp(2, 3);
        self.reset_errors();
        self.reset_result();
    }

    pub fn solve(&mut self) -> Option<Focus> {
        self.reset_errors();
        let dimension = self.dimension;
        let mut matrix = Vec::with_capacity(dimension);
        for row in 0..dimension {
            let mut values = Vec::with_capacity(dimension + 1);
            for column in 0..=dimension {
                let target = input_column(dimension, column);
                match Rational::parse(self.input(row, target)) {
                    Ok(value) => values.push(value),
                    Err(error) => {
                        self.helper = format!(
                            "Equação {}, {}: {}",
                            row + 1,
                            term_name(if column == dimension { 3 } else { column }),
                            error.message()
                        );
                        self.helper_error = true;
                        self.error_field = Some((row, target));
                        return Some(Focus::Field {
                            row,
                            column: target,
                            select_all: true,
                        });
                    }
                }
            }
            matrix.push(values);
        }
        let Ok(solution) = solve(&matrix) else {
            return None;
        };
        let text = format(&solution);
        self.copied = false;
        self.outcome = match solution.kind {
            SolutionKind::Unique => {
                self.note = NOTE_UNIQUE.to_string();
                Outcome::Unique(
                    (0..dimension)
                        .map(|row| {
                            solution
                                .reduced
                                .get(row)
                                .and_then(|values| values.get(dimension))
                                .map(ToString::to_string)
                                .unwrap_or_default()
                        })
                        .collect(),
                )
            }
            SolutionKind::Infinite => {
                self.note = NOTE_INFINITE.to_string();
                Outcome::Infinite(
                    text.find("\r\n\r\n")
                        .map_or_else(|| text.clone(), |start| text[start + 4..].to_string()),
                )
            }
            SolutionKind::None => {
                self.note = NOTE_NONE.to_string();
                Outcome::NoSolution(GENERAL_NONE.to_string())
            }
        };
        self.last_result = Some(text);
        None
    }

    pub fn fill_example(&mut self) -> Focus {
        let dimension = self.dimension;
        for row in 0..dimension {
            for column in 0..=dimension {
                let value = if dimension == 2 {
                    EXAMPLE_TWO[row][column]
                } else {
                    EXAMPLE_THREE[row][column]
                };
                self.store_input(row, input_column(dimension, column), value);
            }
        }
        self.reset_errors();
        self.reset_result();
        self.helper = EXAMPLE_HELPER.to_string();
        Focus::Solve
    }

    pub fn clear(&mut self) -> Focus {
        for row in self.inputs.iter_mut() {
            for value in row.iter_mut() {
                value.clear();
            }
        }
        self.reset_errors();
        self.reset_result();
        Focus::Field {
            row: 0,
            column: 0,
            select_all: false,
        }
    }

    pub fn copy(&mut self, write: impl FnOnce(&str) -> bool) {
        let Some(text) = self.last_result.as_deref() else {
            return;
        };
        if write(text) {
            self.copied = true;
        } else {
            self.note = NOTE_COPY_FAILED.to_string();
        }
    }

    pub fn author_unavailable(&mut self) {
        self.helper = AUTHOR_HELPER.to_string();
    }

    fn reset_errors(&mut self) {
        self.helper = DEFAULT_HELPER.to_string();
        self.helper_error = false;
        self.error_field = None;
    }

    fn reset_result(&mut self) {
        self.outcome = Outcome::Empty;
        self.last_result = None;
        self.note = NOTE_DEFAULT.to_string();
        self.copied = false;
    }
}
