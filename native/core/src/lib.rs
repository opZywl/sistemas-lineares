use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

pub const MAX_INPUT_LENGTH: usize = 100;
pub const MAX_EXPONENT: i32 = 1000;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rational {
    numerator: BigInt,
    denominator: BigInt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    Empty,
    TooLong,
    Invalid,
    Exponent,
    ZeroDenominator,
}

impl ParseError {
    pub fn message(self) -> &'static str {
        match self {
            ParseError::Empty => "Preencha todos os campos.",
            ParseError::TooLong => "Use números com no máximo 100 caracteres.",
            ParseError::Invalid => "Número inválido.",
            ParseError::Exponent => "Use um expoente inteiro entre -1000 e 1000.",
            ParseError::ZeroDenominator => "O denominador não pode ser zero.",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for ParseError {}

impl Rational {
    pub fn new(numerator: BigInt, denominator: BigInt) -> Rational {
        assert!(!denominator.is_zero(), "denominador zero");
        let (mut numerator, mut denominator) = if denominator.is_negative() {
            (-numerator, -denominator)
        } else {
            (numerator, denominator)
        };
        let divisor = numerator.gcd(&denominator);
        if !divisor.is_one() {
            numerator /= &divisor;
            denominator /= &divisor;
        }
        Rational {
            numerator,
            denominator,
        }
    }

    pub fn integer(value: impl Into<BigInt>) -> Rational {
        Rational {
            numerator: value.into(),
            denominator: BigInt::one(),
        }
    }

    pub fn zero() -> Rational {
        Rational::integer(0)
    }

    pub fn one() -> Rational {
        Rational::integer(1)
    }

    pub fn numerator(&self) -> &BigInt {
        &self.numerator
    }

    pub fn denominator(&self) -> &BigInt {
        &self.denominator
    }

    pub fn is_zero(&self) -> bool {
        self.numerator.is_zero()
    }

    pub fn is_one(&self) -> bool {
        self.numerator.is_one() && self.denominator.is_one()
    }

    pub fn is_negative(&self) -> bool {
        self.numerator.is_negative()
    }

    pub fn parse(input: &str) -> Result<Rational, ParseError> {
        let text = input.trim().replace(',', ".").replace('\u{2212}', "-");
        if text.is_empty() {
            return Err(ParseError::Empty);
        }
        if text.encode_utf16().count() > MAX_INPUT_LENGTH {
            return Err(ParseError::TooLong);
        }
        let parts: Vec<&str> = text.split('/').collect();
        if parts.len() > 2 {
            return Err(ParseError::Invalid);
        }
        let value = parse_decimal(parts[0])?;
        if parts.len() == 1 {
            return Ok(value);
        }
        let divisor = parse_decimal(parts[1])?;
        if divisor.is_zero() {
            return Err(ParseError::ZeroDenominator);
        }
        Ok(value / divisor)
    }
}

fn parse_decimal(input: &str) -> Result<Rational, ParseError> {
    let mut text = input.trim();
    if text.is_empty() {
        return Err(ParseError::Invalid);
    }
    let mut exponent = 0;
    if let Some(position) = text.find(['e', 'E']) {
        exponent = parse_exponent(&text[position + 1..]).ok_or(ParseError::Exponent)?;
        text = &text[..position];
        if text.is_empty() {
            return Err(ParseError::Invalid);
        }
    }
    let (negative, body) = match text.as_bytes()[0] {
        b'-' => (true, &text[1..]),
        b'+' => (false, &text[1..]),
        _ => (false, text),
    };
    let mut digits = String::with_capacity(body.len());
    let mut dot = false;
    let mut decimals = 0;
    for character in body.chars() {
        if character == '.' && !dot {
            dot = true;
            continue;
        }
        if !character.is_ascii_digit() {
            return Err(ParseError::Invalid);
        }
        digits.push(character);
        if dot {
            decimals += 1;
        }
    }
    if digits.is_empty() {
        return Err(ParseError::Invalid);
    }
    let mut numerator: BigInt = digits.parse().map_err(|_| ParseError::Invalid)?;
    if negative {
        numerator = -numerator;
    }
    let power = decimals - exponent;
    let ten = BigInt::from(10u32);
    Ok(if power >= 0 {
        Rational::new(numerator, ten.pow(power.unsigned_abs()))
    } else {
        Rational::integer(numerator * ten.pow(power.unsigned_abs()))
    })
}

fn parse_exponent(text: &str) -> Option<i32> {
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let digits = digits.trim_end_matches('\0');
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let significant = digits.trim_start_matches('0');
    if significant.len() > 4 {
        return None;
    }
    let value: i32 = if significant.is_empty() {
        0
    } else {
        significant.parse().ok()?
    };
    if value > MAX_EXPONENT {
        return None;
    }
    Some(if negative { -value } else { value })
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator.is_one() {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

impl Add<&Rational> for &Rational {
    type Output = Rational;
    fn add(self, other: &Rational) -> Rational {
        Rational::new(
            &self.numerator * &other.denominator + &other.numerator * &self.denominator,
            &self.denominator * &other.denominator,
        )
    }
}

impl Sub<&Rational> for &Rational {
    type Output = Rational;
    fn sub(self, other: &Rational) -> Rational {
        Rational::new(
            &self.numerator * &other.denominator - &other.numerator * &self.denominator,
            &self.denominator * &other.denominator,
        )
    }
}

impl Mul<&Rational> for &Rational {
    type Output = Rational;
    fn mul(self, other: &Rational) -> Rational {
        Rational::new(
            &self.numerator * &other.numerator,
            &self.denominator * &other.denominator,
        )
    }
}

impl Div<&Rational> for &Rational {
    type Output = Rational;
    fn div(self, other: &Rational) -> Rational {
        Rational::new(
            &self.numerator * &other.denominator,
            &self.denominator * &other.numerator,
        )
    }
}

impl Neg for &Rational {
    type Output = Rational;
    fn neg(self) -> Rational {
        Rational {
            numerator: -&self.numerator,
            denominator: self.denominator.clone(),
        }
    }
}

macro_rules! forward_owned {
    ($($trait:ident $method:ident),*) => {$(
        impl $trait<Rational> for Rational {
            type Output = Rational;
            fn $method(self, other: Rational) -> Rational {
                (&self).$method(&other)
            }
        }
        impl $trait<&Rational> for Rational {
            type Output = Rational;
            fn $method(self, other: &Rational) -> Rational {
                (&self).$method(other)
            }
        }
        impl $trait<Rational> for &Rational {
            type Output = Rational;
            fn $method(self, other: Rational) -> Rational {
                self.$method(&other)
            }
        }
    )*};
}

forward_owned!(Add add, Sub sub, Mul mul, Div div);

impl Neg for Rational {
    type Output = Rational;
    fn neg(self) -> Rational {
        -&self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SolutionKind {
    Unique,
    Infinite,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Solution {
    pub kind: SolutionKind,
    pub reduced: Vec<Vec<Rational>>,
    pub pivot_columns: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeError;

impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("O sistema deve ser 2×2 ou 3×3.")
    }
}

impl std::error::Error for ShapeError {}

pub fn solve(input: &[Vec<Rational>]) -> Result<Solution, ShapeError> {
    let rows = input.len();
    let columns = rows;
    if !(rows == 2 || rows == 3) || input.iter().any(|row| row.len() != columns + 1) {
        return Err(ShapeError);
    }
    let mut matrix = input.to_vec();
    let mut pivots = Vec::with_capacity(columns);
    let mut pivot_row = 0;
    for column in 0..columns {
        if pivot_row == rows {
            break;
        }
        let Some(selected) = (pivot_row..rows).find(|&row| !matrix[row][column].is_zero()) else {
            continue;
        };
        matrix.swap(pivot_row, selected);
        let pivot = matrix[pivot_row][column].clone();
        for value in &mut matrix[pivot_row] {
            *value = &*value / &pivot;
        }
        let normalized = matrix[pivot_row].clone();
        for (row, values) in matrix.iter_mut().enumerate() {
            if row == pivot_row || values[column].is_zero() {
                continue;
            }
            let factor = values[column].clone();
            for (value, pivot_value) in values.iter_mut().zip(&normalized) {
                *value = &*value - &(&factor * pivot_value);
            }
        }
        pivots.push(column);
        pivot_row += 1;
    }
    let inconsistent = matrix
        .iter()
        .any(|row| row[..columns].iter().all(Rational::is_zero) && !row[columns].is_zero());
    let kind = if inconsistent {
        SolutionKind::None
    } else if pivots.len() == columns {
        SolutionKind::Unique
    } else {
        SolutionKind::Infinite
    };
    Ok(Solution {
        kind,
        reduced: matrix,
        pivot_columns: pivots,
    })
}

pub const VARIABLE_NAMES: [&str; 3] = ["x", "y", "z"];

pub fn format(solution: &Solution) -> String {
    if solution.kind == SolutionKind::None {
        return "Este sistema não tem solução.\r\nAs equações são incompatíveis.".to_string();
    }
    let count = solution.reduced.len();
    let mut pivot_for_column = vec![None; count];
    for (row, &column) in solution.pivot_columns.iter().enumerate() {
        pivot_for_column[column] = Some(row);
    }
    let mut parameters = vec![0usize; count];
    let mut next = 1;
    for (column, parameter) in parameters.iter_mut().enumerate() {
        if pivot_for_column[column].is_none() {
            *parameter = next;
            next += 1;
        }
    }
    let mut text = String::new();
    text.push_str(if solution.kind == SolutionKind::Unique {
        "Solução única"
    } else {
        "Infinitas soluções"
    });
    text.push_str("\r\n\r\n");
    for column in 0..count {
        text.push_str(VARIABLE_NAMES[column]);
        text.push_str(" = ");
        match pivot_for_column[column] {
            None => {
                text.push('t');
                text.push_str(&parameters[column].to_string());
            }
            Some(row) => {
                let constant = &solution.reduced[row][count];
                let mut has_term = !constant.is_zero();
                if has_term {
                    text.push_str(&constant.to_string());
                }
                for (free, &parameter) in parameters.iter().enumerate() {
                    if parameter == 0 {
                        continue;
                    }
                    let coefficient = -&solution.reduced[row][free];
                    if coefficient.is_zero() {
                        continue;
                    }
                    let negative = coefficient.is_negative();
                    let magnitude = if negative { -coefficient } else { coefficient };
                    if has_term {
                        text.push_str(if negative { " - " } else { " + " });
                    } else if negative {
                        text.push('-');
                    }
                    if !magnitude.is_one() {
                        text.push_str(&magnitude.to_string());
                        text.push_str(" · ");
                    }
                    text.push('t');
                    text.push_str(&parameter.to_string());
                    has_term = true;
                }
                if !has_term {
                    text.push('0');
                }
            }
        }
        text.push_str("\r\n");
    }
    if solution.kind == SolutionKind::Infinite {
        text.push_str("\r\nt1, t2, ... podem ser quaisquer números reais.\r\n");
    }
    text.truncate(text.trim_end().len());
    text
}
