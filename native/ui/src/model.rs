use sistemas_lineares_core::steps::abbreviate;
use sistemas_lineares_core::{
    Entry, Method, ParseError, Rational, SolutionKind, VARIABLE_NAMES, Walkthrough, format,
    looks_incomplete, power, preview_equation, solve, walkthrough, within_live_budget,
};

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
pub const SUBTITLE: &str = "Preencha os coeficientes: a solução aparece na hora.\nFrações, decimais e potências, sempre com resultado exato.";
pub const THINK_INTRO: &str = "MODO\nPENSAR.";
pub const THINK_SUBTITLE: &str =
    "Como a resposta foi calculada, passo a passo,\ncom o porquê de cada conta.";
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
    "Preencha o sistema ou use um exemplo.\nA resposta aparece assim que tudo estiver preenchido.";
pub const MISSING_DESCRIPTION: &str =
    "A resposta aparece assim que tudo estiver\npreenchido. Enter leva ao próximo campo vazio.";
pub const INVALID_TITLE: &str = "Confira os valores";
pub const INVALID_DESCRIPTION: &str =
    "Um dos campos não é um número válido.\nA dica abaixo dos campos mostra o que corrigir.";
pub const LARGE_TITLE: &str = "Números grandes";
pub const LARGE_DESCRIPTION: &str =
    "Com números tão grandes, a conta é feita quando\nvocê pressiona Enter ou Resolver sistema.";
pub const ONLY_THIS_FIELD: &str = "Falta só este campo.";
pub const FOOTER_METHODS_LABEL: &str = "Passo a passo por";
pub const KEYS: [(&str, &str); 2] = [
    ("Enter", "próximo campo vazio"),
    ("\u{2191} \u{2193}", "mudar de equação"),
];
pub const THINK_KEYS: [(&str, &str); 1] = [("Esc", "voltar")];
pub const TIP_THINK: &str = "Ver como a resposta foi calculada, passo a passo.";
pub const TIP_COPY: &str = "Copiar a resposta como texto.";
pub const TIP_CLEAR: &str = "Apagar todos os campos.";
pub const TIP_EXAMPLE: &str = "Substitui os campos por um sistema de exemplo.";
pub const TIP_DISABLED: &str = "Disponível quando todos os campos estiverem preenchidos.";
pub const FOOTER: &str = "Gauss-Jordan · Cramer · Escalonamento  /  aritmética racional exata";
pub const THINK_FOOTER: &str = "Operações de linha e determinantes  /  aritmética racional exata";
pub const DEFAULT_HELPER: &str =
    "Aceita 2 · -1,5 · 3/4 · 2^3 · 10² (AltGr+2). Use 0 se a variável não aparecer.";
pub const EXAMPLE_HELPER: &str =
    "Exemplo resolvido. Mude qualquer valor e a resposta se atualiza na hora.";
pub const AUTHOR_HELPER: &str = "Abra lucas-lima.xyz no seu navegador para visitar o autor.";
pub const NOTE_DEFAULT: &str = "Resultados exatos, incluindo frações.";
pub const NOTE_UNIQUE: &str = "Solução única · valores exatos";
pub const NOTE_INFINITE: &str = "Os parâmetros t1, t2, … são números reais livres.";
pub const NOTE_NONE: &str = "Confira os coeficientes se esse não era o esperado.";
pub const NOTE_STALE: &str = "Atualizando: a resposta acompanha o que você digita.";
pub const NOTE_COPY_FAILED: &str = "Não foi possível copiar agora. Tente novamente.";
pub const COPY_LABEL: &str = "Copiar";
pub const COPIED_LABEL: &str = "Copiado";
pub const THINK_LABEL: &str = "Modo pensar";
pub const BACK_LABEL: &str = "Voltar";
pub const COPY_STEPS_LABEL: &str = "Copiar passos";
pub const COPIED_STEPS_LABEL: &str = "Passos copiados";
pub const METHOD_LABEL: &str = "MÉTODO";
pub const SYSTEM_LABEL: &str = "SEU SISTEMA";
pub const ANSWER_LABEL: &str = "RESPOSTA";
pub const COPY_STEPS_FAILED: &str = "Não foi possível copiar agora. Tente novamente.";
pub const TITLE_UNIQUE: &str = "Solução encontrada";
pub const TITLE_INFINITE: &str = "Infinitas soluções";
pub const TITLE_NONE: &str = "Sem solução";
pub const DESCRIPTION_UNIQUE: &str = "Estes valores satisfazem todas\nas equações do sistema.";
pub const DESCRIPTION_INFINITE: &str =
    "As equações são dependentes.\nVeja a forma geral da resposta:";
pub const DESCRIPTION_NONE: &str = "As equações são incompatíveis.";
pub const GENERAL_NONE: &str =
    "Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo.";
pub const SIZE_LABEL: &str = "Tamanho do sistema";
pub const FIELD_DESCRIPTION: &str = "Aceita 2 · -1,5 · 3/4 · 2^3 · 10². Enter vai ao próximo campo vazio; as setas para cima e para baixo mudam de equação.";
pub const SPOKEN_LIMIT: usize = 24;

const EXAMPLE_TWO: [[&str; 3]; 2] = [["2", "1", "5"], ["1", "-1", "1"]];
const EXAMPLE_THREE: [[&str; 4]; 3] = [
    ["1", "1", "1", "6"],
    ["2", "-1", "1", "3"],
    ["1", "2", "-1", "2"],
];

pub const fn method_tip(method: Method) -> &'static str {
    match method {
        Method::GaussJordan => "Gauss-Jordan: operações de linha até a matriz identidade.",
        Method::Cramer => "Regra de Cramer: determinantes, com Sarrus no 3×3.",
        Method::Gauss => "Escalonamento: zera abaixo dos pivôs e resolve de baixo para cima.",
    }
}

pub const fn method_needs_system(method: Method) -> &'static str {
    match method {
        Method::GaussJordan => {
            "Preencha todos os campos para ver o passo a passo por Gauss-Jordan."
        }
        Method::Cramer => "Preencha todos os campos para ver o passo a passo pela regra de Cramer.",
        Method::Gauss => "Preencha todos os campos para ver o passo a passo por escalonamento.",
    }
}

pub const fn method_link_name(method: Method) -> &'static str {
    match method {
        Method::GaussJordan => "Passo a passo por Gauss-Jordan",
        Method::Cramer => "Passo a passo pela regra de Cramer",
        Method::Gauss => "Passo a passo por escalonamento",
    }
}

pub fn missing_title(missing: usize) -> String {
    if missing == 1 {
        "Falta 1 campo".to_string()
    } else {
        format!("Faltam {missing} campos")
    }
}

pub fn invalid_suffix(invalid: usize) -> String {
    if invalid == 1 {
        " · 1 inválido".to_string()
    } else {
        format!(" · {invalid} inválidos")
    }
}

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

pub fn size_name(dimension: usize, selected: bool) -> String {
    let label = if dimension == 2 {
        TWO_LABEL
    } else {
        THREE_LABEL
    };
    if selected {
        format!("Sistema {label}, selecionado")
    } else {
        format!("Sistema {label}")
    }
}

pub fn reading(value: &Rational) -> String {
    format!("Este campo vale {}.", abbreviate(&value.to_string()))
}

pub fn row_number(row: usize) -> String {
    format!("{:02}", row + 1)
}

pub fn spoken_value(value: &str) -> String {
    if value.chars().count() <= SPOKEN_LIMIT {
        return value.to_string();
    }
    let digits = |part: &str| part.chars().filter(char::is_ascii_digit).count();
    match value.split_once('/') {
        Some((numerator, denominator)) => format!(
            "fração com {} algarismos sobre {} algarismos",
            digits(numerator),
            digits(denominator)
        ),
        None => format!("número com {} algarismos", digits(value)),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Empty,
    Unique(Vec<String>),
    Infinite(String),
    NoSolution(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Waiting {
    Empty,
    Missing { missing: usize, invalid: usize },
    Invalid,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Card {
    Result { stale: bool },
    Waiting(Waiting),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    Example,
    Author,
    MethodNeedsSystem(Method),
    OnlyThisField,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MethodRequest {
    Opened,
    NeedsSystem(Option<Focus>),
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
enum Cell {
    Missing,
    Typing,
    Invalid(ParseError),
    Value(Rational),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    dimension: usize,
    inputs: [[[String; 4]; 3]; 2],
    cells: Vec<Vec<Cell>>,
    revealed: [[bool; 4]; 3],
    focus: Option<(usize, usize)>,
    notice: Option<Notice>,
    outcome: Outcome,
    stale: bool,
    last_result: Option<String>,
    note: String,
    copied: bool,
    solved: Option<Vec<Vec<Rational>>>,
    method: Method,
    thinking: Option<Walkthrough>,
    steps_copied: bool,
    steps_copy_failed: bool,
    missing: usize,
    invalid: usize,
    large: bool,
    blank: bool,
    preview: Vec<String>,
}

impl Default for Model {
    fn default() -> Self {
        Model::new()
    }
}

impl Model {
    pub fn new() -> Model {
        let mut model = Model {
            dimension: 2,
            inputs: Default::default(),
            cells: Vec::new(),
            revealed: [[false; 4]; 3],
            focus: None,
            notice: None,
            outcome: Outcome::Empty,
            stale: false,
            last_result: None,
            note: NOTE_DEFAULT.to_string(),
            copied: false,
            solved: None,
            method: Method::GaussJordan,
            thinking: None,
            steps_copied: false,
            steps_copy_failed: false,
            missing: 0,
            invalid: 0,
            large: false,
            blank: true,
            preview: Vec::new(),
        };
        model.analyze(false);
        model
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    fn grid(&self) -> &[[String; 4]; 3] {
        &self.inputs[self.dimension.clamp(2, 3) - 2]
    }

    fn grid_mut(&mut self) -> &mut [[String; 4]; 3] {
        &mut self.inputs[self.dimension.clamp(2, 3) - 2]
    }

    pub fn input(&self, row: usize, column: usize) -> &str {
        self.grid()
            .get(row)
            .and_then(|values| values.get(column))
            .map_or("", String::as_str)
    }

    fn cell(&self, row: usize, column: usize) -> Option<&Cell> {
        self.cells.get(row).and_then(|cells| cells.get(column))
    }

    fn visible_fields(&self) -> Vec<(usize, usize)> {
        (0..self.dimension)
            .flat_map(|row| {
                (0..=self.dimension).map(move |column| (row, input_column(self.dimension, column)))
            })
            .collect()
    }

    fn analyze(&mut self, explicit: bool) -> Option<Vec<Vec<Rational>>> {
        let dimension = self.dimension;
        let mut cells = vec![vec![Cell::Missing; 4]; 3];
        let mut matrix = Vec::with_capacity(dimension);
        let mut preview = Vec::with_capacity(dimension);
        let mut missing = 0;
        let mut invalid = 0;
        let mut blank = true;
        for (row, cells_row) in cells.iter_mut().enumerate().take(dimension) {
            let mut values = Vec::with_capacity(dimension + 1);
            let mut entries = Vec::with_capacity(dimension + 1);
            for column in 0..=dimension {
                let target = input_column(dimension, column);
                let text = self.grid()[row][target].as_str();
                let cell = match Rational::parse(text) {
                    Ok(value) => Cell::Value(value),
                    Err(ParseError::Empty) => Cell::Missing,
                    Err(error) => {
                        let typing = !explicit
                            && !self.revealed[row][target]
                            && self.focus == Some((row, target))
                            && looks_incomplete(text);
                        if typing {
                            Cell::Typing
                        } else {
                            Cell::Invalid(error)
                        }
                    }
                };
                if !text.trim().is_empty() {
                    blank = false;
                }
                match &cell {
                    Cell::Value(value) => {
                        values.push(value.clone());
                        entries.push(Entry::Value(value.clone()));
                    }
                    Cell::Missing | Cell::Typing => {
                        missing += 1;
                        entries.push(Entry::Missing);
                    }
                    Cell::Invalid(_) => {
                        invalid += 1;
                        entries.push(Entry::Invalid);
                    }
                }
                cells_row[target] = cell;
            }
            preview.push(preview_equation(&entries));
            matrix.push(values);
        }
        self.cells = cells;
        self.missing = missing;
        self.invalid = invalid;
        self.blank = blank;
        self.preview = preview;
        let complete = missing == 0 && invalid == 0;
        self.large = complete && !within_live_budget(&matrix);
        complete.then_some(matrix)
    }

    fn show_result(&mut self, matrix: Vec<Vec<Rational>>) {
        let dimension = self.dimension;
        let Ok(solution) = solve(&matrix) else {
            return;
        };
        let text = format(&solution);
        self.copied = false;
        self.stale = false;
        self.solved = Some(matrix);
        self.thinking = None;
        self.steps_copied = false;
        self.steps_copy_failed = false;
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
    }

    fn clear_outcome(&mut self) {
        self.outcome = Outcome::Empty;
        self.stale = false;
        self.last_result = None;
        self.note = NOTE_DEFAULT.to_string();
        self.copied = false;
        self.solved = None;
        self.thinking = None;
        self.steps_copied = false;
        self.steps_copy_failed = false;
    }

    fn settle_card(&mut self, matrix: Option<Vec<Vec<Rational>>>, live: bool) {
        match matrix {
            Some(matrix) if !(live && self.large) => self.show_result(matrix),
            _ if live && self.outcome != Outcome::Empty => {
                self.stale = true;
                self.note = NOTE_STALE.to_string();
                self.copied = false;
                self.thinking = None;
            }
            _ => self.clear_outcome(),
        }
    }

    pub fn edit(&mut self, row: usize, column: usize, text: &str) -> bool {
        if row >= 3 || column >= 4 || self.input(row, column) == text {
            return false;
        }
        let was_missing = matches!(self.cell(row, column), Some(Cell::Missing));
        self.grid_mut()[row][column] = text.to_string();
        self.notice = None;
        let mut matrix = self.analyze(false);
        if self.revealed[row][column]
            && (was_missing
                || matches!(self.cell(row, column), Some(Cell::Value(_) | Cell::Missing)))
        {
            self.revealed[row][column] = false;
            matrix = self.analyze(false);
        }
        self.settle_card(matrix, true);
        true
    }

    pub fn set_input(&mut self, row: usize, column: usize, text: &str) {
        self.edit(row, column, text);
    }

    pub fn store_input(&mut self, row: usize, column: usize, text: &str) -> bool {
        match self
            .grid_mut()
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

    pub fn focus(&self) -> Option<(usize, usize)> {
        self.focus
    }

    pub fn set_focus(&mut self, field: Option<(usize, usize)>) -> bool {
        if self.focus == field {
            return false;
        }
        self.focus = field;
        let missing = self.missing;
        let invalid = self.invalid;
        self.analyze(false);
        missing != self.missing || invalid != self.invalid
    }

    pub fn idle(&mut self) -> bool {
        let mut changed = false;
        if self.stale {
            self.clear_outcome();
            changed = true;
        }
        for (row, column) in self.visible_fields() {
            let typing =
                self.focus == Some((row, column)) && looks_incomplete(&self.grid()[row][column]);
            if matches!(self.cell(row, column), Some(Cell::Invalid(_)))
                && !self.revealed[row][column]
                && !typing
            {
                self.revealed[row][column] = true;
                changed = true;
            }
        }
        if changed {
            self.analyze(false);
        }
        changed
    }

    pub fn leave(&mut self, row: usize, column: usize) -> bool {
        let mut changed = false;
        if self.focus == Some((row, column)) {
            self.focus = None;
            self.analyze(false);
        }
        if matches!(self.cell(row, column), Some(Cell::Invalid(_))) && !self.revealed[row][column] {
            self.revealed[row][column] = true;
            changed = true;
        }
        if self.stale {
            self.clear_outcome();
            changed = true;
        }
        if changed {
            self.analyze(false);
        }
        changed
    }

    pub fn helper(&self) -> String {
        if let Some((row, column)) = self.shown_error() {
            let error = match self.cell(row, column) {
                Some(Cell::Invalid(error)) => *error,
                _ => ParseError::Empty,
            };
            return format!(
                "Equação {}, {}: {}",
                row + 1,
                term_name(column),
                error.message()
            );
        }
        match self.notice {
            Some(Notice::Example) => EXAMPLE_HELPER.to_string(),
            Some(Notice::Author) => AUTHOR_HELPER.to_string(),
            Some(Notice::MethodNeedsSystem(method)) => method_needs_system(method).to_string(),
            Some(Notice::OnlyThisField) => ONLY_THIS_FIELD.to_string(),
            None => self
                .focused_reading()
                .unwrap_or_else(|| DEFAULT_HELPER.to_string()),
        }
    }

    fn focused_reading(&self) -> Option<String> {
        let (row, column) = self.focus?;
        if !power::uses_extension(self.grid()[row][column].trim()) {
            return None;
        }
        match self.cell(row, column) {
            Some(Cell::Value(value)) => Some(reading(value)),
            _ => None,
        }
    }

    pub fn helper_is_reading(&self) -> bool {
        self.shown_error().is_none() && self.notice.is_none() && self.focused_reading().is_some()
    }

    pub fn notice(&self) -> Option<Notice> {
        self.notice
    }

    fn problem(&self, row: usize, column: usize) -> bool {
        matches!(
            self.cell(row, column),
            Some(Cell::Missing | Cell::Typing | Cell::Invalid(_))
        )
    }

    fn shown_error(&self) -> Option<(usize, usize)> {
        let revealed = |&(row, column): &(usize, usize)| {
            self.revealed[row][column] && self.problem(row, column)
        };
        if let Some(field) = self.focus.filter(revealed) {
            return Some(field);
        }
        self.visible_fields().into_iter().find(revealed)
    }

    pub fn helper_is_error(&self) -> bool {
        self.shown_error().is_some()
    }

    pub fn field_has_error(&self, row: usize, column: usize) -> bool {
        layout_visible(self.dimension, row, column)
            && self.revealed[row][column]
            && self.problem(row, column)
    }

    pub fn error_field(&self) -> Option<(usize, usize)> {
        self.shown_error()
    }

    pub fn row_complete(&self, row: usize) -> bool {
        row < self.dimension
            && (0..=self.dimension).all(|column| {
                matches!(
                    self.cell(row, input_column(self.dimension, column)),
                    Some(Cell::Value(_))
                )
            })
    }

    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    pub fn is_empty(&self) -> bool {
        self.outcome == Outcome::Empty
    }

    pub fn is_stale(&self) -> bool {
        self.stale
    }

    pub fn card(&self) -> Card {
        if self.outcome != Outcome::Empty {
            return Card::Result { stale: self.stale };
        }
        Card::Waiting(self.waiting())
    }

    pub fn waiting(&self) -> Waiting {
        if self.blank {
            Waiting::Empty
        } else if self.missing > 0 {
            Waiting::Missing {
                missing: self.missing,
                invalid: self.invalid,
            }
        } else if self.invalid > 0 {
            Waiting::Invalid
        } else if self.large {
            Waiting::Large
        } else {
            Waiting::Empty
        }
    }

    pub fn waiting_title(&self) -> Option<String> {
        match self.card() {
            Card::Result { .. } => None,
            Card::Waiting(waiting) => Some(match waiting {
                Waiting::Empty => EMPTY_TITLE.to_string(),
                Waiting::Missing { missing, invalid } if invalid > 0 => {
                    format!("{}{}", missing_title(missing), invalid_suffix(invalid))
                }
                Waiting::Missing { missing, .. } => missing_title(missing),
                Waiting::Invalid => INVALID_TITLE.to_string(),
                Waiting::Large => LARGE_TITLE.to_string(),
            }),
        }
    }

    pub fn waiting_description(&self) -> Option<&'static str> {
        match self.card() {
            Card::Result { .. } => None,
            Card::Waiting(waiting) => Some(match waiting {
                Waiting::Empty => EMPTY_DESCRIPTION,
                Waiting::Missing { .. } => MISSING_DESCRIPTION,
                Waiting::Invalid => INVALID_DESCRIPTION,
                Waiting::Large => LARGE_DESCRIPTION,
            }),
        }
    }

    pub fn shows_preview(&self) -> bool {
        matches!(
            self.card(),
            Card::Waiting(Waiting::Missing { .. } | Waiting::Invalid | Waiting::Large)
        )
    }

    pub fn preview(&self) -> &[String] {
        &self.preview
    }

    pub fn preview_spoken(&self) -> String {
        let names = |wanted: fn(&Cell) -> bool| -> Vec<String> {
            self.visible_fields()
                .into_iter()
                .filter(|&(row, column)| self.cell(row, column).is_some_and(wanted))
                .map(|(row, column)| accessible_input_name(row, column).to_lowercase())
                .collect()
        };
        let missing = names(|cell| matches!(cell, Cell::Missing | Cell::Typing));
        let invalid = names(|cell| matches!(cell, Cell::Invalid(_)));
        let mut parts = Vec::new();
        if !missing.is_empty() {
            parts.push(format!("Faltam: {}.", missing.join("; ")));
        }
        if !invalid.is_empty() {
            parts.push(format!("Inválidos: {}.", invalid.join("; ")));
        }
        parts.join(" ")
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
        if self.stale {
            None
        } else {
            self.last_result.as_deref()
        }
    }

    pub fn note(&self) -> &str {
        &self.note
    }

    pub fn copy_enabled(&self) -> bool {
        self.last_result.is_some() && !self.stale
    }

    pub fn copy_label(&self) -> &'static str {
        if self.copied {
            COPIED_LABEL
        } else {
            COPY_LABEL
        }
    }

    pub fn think_enabled(&self) -> bool {
        self.solved.is_some() && !self.stale
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn is_thinking(&self) -> bool {
        self.thinking.is_some()
    }

    pub fn walkthrough(&self) -> Option<&Walkthrough> {
        self.thinking.as_ref()
    }

    pub fn steps_text(&self) -> Option<String> {
        self.thinking.as_ref().map(Walkthrough::text)
    }

    pub fn steps_copy_label(&self) -> &'static str {
        if self.steps_copied {
            COPIED_STEPS_LABEL
        } else {
            COPY_STEPS_LABEL
        }
    }

    pub fn steps_copy_failed(&self) -> bool {
        self.steps_copy_failed
    }

    pub fn thinking_status(&self) -> String {
        match &self.thinking {
            Some(walkthrough) if self.steps_copy_failed => {
                format!("{} · {COPY_STEPS_FAILED}", walkthrough.method.name())
            }
            Some(walkthrough) => format!(
                "{} · {} passos",
                walkthrough.method.name(),
                walkthrough.steps.len()
            ),
            None => String::new(),
        }
    }

    pub fn open_thinking(&mut self) -> bool {
        if self.stale {
            return false;
        }
        let Some(matrix) = self.solved.as_ref() else {
            return false;
        };
        match walkthrough(matrix, self.method) {
            Ok(result) => {
                self.thinking = Some(result);
                self.steps_copied = false;
                self.steps_copy_failed = false;
                true
            }
            Err(_) => false,
        }
    }

    pub fn close_thinking(&mut self) -> bool {
        self.steps_copied = false;
        self.steps_copy_failed = false;
        self.thinking.take().is_some()
    }

    pub fn set_method(&mut self, method: Method) {
        if self.method == method && self.thinking.is_some() {
            return;
        }
        self.method = method;
        if self.thinking.is_some() {
            self.open_thinking();
        }
    }

    pub fn copy_steps(&mut self, write: impl FnOnce(&str) -> bool) {
        let Some(text) = self.steps_text() else {
            return;
        };
        if write(&text.replace('\n', "\r\n")) {
            self.steps_copied = true;
            self.steps_copy_failed = false;
        } else {
            self.steps_copied = false;
            self.steps_copy_failed = true;
        }
    }

    pub fn announcement(&self) -> Option<String> {
        if self.stale {
            return None;
        }
        match &self.outcome {
            Outcome::Empty => None,
            Outcome::Unique(values) => {
                let parts: Vec<String> = values
                    .iter()
                    .zip(VARIABLE_NAMES)
                    .map(|(value, name)| format!("{name} = {}", spoken_value(value)))
                    .collect();
                Some(format!("{TITLE_UNIQUE}: {}.", parts.join(", ")))
            }
            Outcome::Infinite(general) => {
                let parts: Vec<String> = general
                    .lines()
                    .map(str::trim)
                    .filter(|line| line.contains(" = "))
                    .map(|line| match line.split_once(" = ") {
                        Some((name, value)) => format!("{name} = {}", spoken_value(value)),
                        None => line.to_string(),
                    })
                    .collect();
                Some(format!("{TITLE_INFINITE}: {}.", parts.join(", ")))
            }
            Outcome::NoSolution(_) => Some(format!("{TITLE_NONE}. {DESCRIPTION_NONE}")),
        }
    }

    pub fn dimension_text(&self) -> String {
        format!("{0} equações · {0} incógnitas", self.dimension)
    }

    pub fn field_count(&self) -> usize {
        self.dimension * (self.dimension + 1)
    }

    pub fn missing(&self) -> usize {
        self.missing
    }

    pub fn invalid_count(&self) -> usize {
        self.invalid
    }

    pub fn change_dimension(&mut self, dimension: usize) {
        self.dimension = dimension.clamp(2, 3);
        self.revealed = [[false; 4]; 3];
        self.notice = None;
        if self
            .focus
            .is_some_and(|(row, column)| !layout_visible(self.dimension, row, column))
        {
            self.focus = None;
        }
        let matrix = self.analyze(false);
        self.settle_card(matrix, false);
    }

    pub fn solve(&mut self) -> Option<Focus> {
        self.notice = None;
        let matrix = self.analyze(true);
        if let Some(matrix) = matrix {
            self.show_result(matrix);
            self.analyze(false);
            return None;
        }
        self.clear_outcome();
        let mut first = None;
        for (row, column) in self.visible_fields() {
            match self.cell(row, column) {
                Some(Cell::Invalid(_)) => {
                    self.revealed[row][column] = true;
                    first.get_or_insert((row, column));
                }
                Some(Cell::Missing | Cell::Typing) if first.is_none() => {
                    self.revealed[row][column] = true;
                    first = Some((row, column));
                }
                _ => {}
            }
        }
        self.analyze(false);
        first.map(|(row, column)| Focus::Field {
            row,
            column,
            select_all: true,
        })
    }

    pub fn enter(&mut self, row: usize, column: usize) -> Option<Focus> {
        if !layout_visible(self.dimension, row, column) {
            return None;
        }
        let here = Focus::Field {
            row,
            column,
            select_all: true,
        };
        let text = self.grid()[row][column].clone();
        if !matches!(Rational::parse(&text), Ok(_) | Err(ParseError::Empty)) {
            self.notice = None;
            self.revealed[row][column] = true;
            self.analyze(false);
            return Some(here);
        }
        self.analyze(true);
        if self.missing == 0 {
            return self.solve();
        }
        let fields = self.visible_fields();
        let start = fields
            .iter()
            .position(|field| *field == (row, column))
            .unwrap_or(0);
        let target = (1..=fields.len())
            .map(|offset| fields[(start + offset) % fields.len()])
            .find(|&(target_row, target_column)| self.problem(target_row, target_column));
        self.analyze(false);
        match target {
            Some(field) if field == (row, column) => {
                self.notice = Some(Notice::OnlyThisField);
                Some(here)
            }
            Some((row, column)) => Some(Focus::Field {
                row,
                column,
                select_all: true,
            }),
            None => None,
        }
    }

    pub fn neighbor(&self, row: usize, column: usize, downward: bool) -> Option<Focus> {
        let target = if downward {
            row.checked_add(1)?
        } else {
            row.checked_sub(1)?
        };
        (target < self.dimension && layout_visible(self.dimension, target, column)).then_some(
            Focus::Field {
                row: target,
                column,
                select_all: true,
            },
        )
    }

    pub fn first_incomplete(&self) -> Option<Focus> {
        self.visible_fields()
            .into_iter()
            .find(|&(row, column)| self.problem(row, column))
            .map(|(row, column)| Focus::Field {
                row,
                column,
                select_all: true,
            })
    }

    pub fn request_method(&mut self, method: Method) -> MethodRequest {
        self.method = method;
        if self.think_enabled() {
            self.thinking = None;
            if self.open_thinking() {
                return MethodRequest::Opened;
            }
        }
        let matrix = self.analyze(true);
        if let Some(matrix) = matrix {
            self.show_result(matrix);
            self.analyze(false);
            if self.open_thinking() {
                return MethodRequest::Opened;
            }
        }
        if self.invalid > 0 {
            for (row, column) in self.visible_fields() {
                if matches!(self.cell(row, column), Some(Cell::Invalid(_))) {
                    self.revealed[row][column] = true;
                }
            }
            self.notice = None;
            self.analyze(false);
            return MethodRequest::NeedsSystem(self.shown_error().map(|(row, column)| {
                Focus::Field {
                    row,
                    column,
                    select_all: true,
                }
            }));
        }
        self.analyze(false);
        self.notice = Some(Notice::MethodNeedsSystem(method));
        MethodRequest::NeedsSystem(self.first_incomplete())
    }

    pub fn fill_example(&mut self) -> Option<Focus> {
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
        self.revealed = [[false; 4]; 3];
        let matrix = self.analyze(true);
        self.settle_card(matrix, false);
        self.analyze(false);
        self.notice = Some(Notice::Example);
        None
    }

    pub fn clear(&mut self) -> Focus {
        for grid in self.inputs.iter_mut() {
            for row in grid.iter_mut() {
                for value in row.iter_mut() {
                    value.clear();
                }
            }
        }
        self.revealed = [[false; 4]; 3];
        self.notice = None;
        self.analyze(false);
        self.clear_outcome();
        Focus::Field {
            row: 0,
            column: 0,
            select_all: false,
        }
    }

    pub fn copy(&mut self, write: impl FnOnce(&str) -> bool) {
        if self.stale {
            return;
        }
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
        self.notice = Some(Notice::Author);
    }
}

const fn layout_visible(dimension: usize, row: usize, column: usize) -> bool {
    row < dimension && (column < dimension || column == 3)
}
