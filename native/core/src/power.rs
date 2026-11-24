use std::sync::OnceLock;

use num_bigint::BigInt;
use num_traits::Pow;

use crate::{MAX_EXPONENT, ParseError, Rational, parse_decimal};

pub const MAX_POWER_DIGITS: usize = 300;

const SUPERSCRIPT_DIGITS: [char; 10] = [
    '\u{2070}', '\u{B9}', '\u{B2}', '\u{B3}', '\u{2074}', '\u{2075}', '\u{2076}', '\u{2077}',
    '\u{2078}', '\u{2079}',
];
const SUPERSCRIPT_PLUS: char = '\u{207A}';
const SUPERSCRIPT_MINUS: char = '\u{207B}';
const PRODUCT_SIGNS: [char; 3] = ['\u{D7}', '\u{B7}', '\u{22C5}'];

fn superscript_digit(character: char) -> Option<u32> {
    SUPERSCRIPT_DIGITS
        .iter()
        .position(|digit| *digit == character)
        .map(|index| index as u32)
}

fn is_superscript(character: char) -> bool {
    superscript_digit(character).is_some()
        || character == SUPERSCRIPT_PLUS
        || character == SUPERSCRIPT_MINUS
}

pub fn uses_extension(text: &str) -> bool {
    text.chars().any(|character| {
        matches!(character, '^' | '(' | ')' | '*')
            || PRODUCT_SIGNS.contains(&character)
            || is_superscript(character)
    })
}

type Parsed = Result<(Rational, bool), ParseError>;

struct Parser {
    characters: Vec<char>,
    position: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.characters.get(self.position).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.characters.get(self.position + offset).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.position += 1;
        }
    }

    fn at_power(&self) -> bool {
        match self.peek() {
            Some('^') => true,
            Some('*') => self.peek_at(1) == Some('*'),
            Some(character) => is_superscript(character),
            None => false,
        }
    }

    fn at_product(&self) -> bool {
        match self.peek() {
            Some('*') => self.peek_at(1) != Some('*'),
            Some(character) => PRODUCT_SIGNS.contains(&character),
            None => false,
        }
    }

    fn value(&mut self) -> Result<Rational, ParseError> {
        let (value, _) = self.quotient()?;
        self.skip_spaces();
        match self.peek() {
            None => Ok(value),
            Some(')') => Err(ParseError::Parenthesis),
            Some(_) => Err(ParseError::Invalid),
        }
    }

    fn quotient(&mut self) -> Parsed {
        let (numerator, powered_numerator) = self.signed()?;
        self.skip_spaces();
        if self.peek() != Some('/') {
            return Ok((numerator, powered_numerator));
        }
        self.position += 1;
        let (denominator, powered_denominator) = self.signed()?;
        self.skip_spaces();
        if self.peek() == Some('/') {
            return Err(ParseError::Invalid);
        }
        if denominator.is_zero() {
            return Err(ParseError::ZeroDenominator);
        }
        capped(
            numerator / denominator,
            powered_numerator || powered_denominator,
        )
    }

    fn signed(&mut self) -> Parsed {
        self.skip_spaces();
        let negative = match self.peek() {
            Some('-') => true,
            Some('+') => false,
            _ => return self.power(),
        };
        self.position += 1;
        if !self
            .peek()
            .is_some_and(|character| character.is_ascii_digit() || matches!(character, '.' | '('))
        {
            return Err(self.missing_operand());
        }
        let (value, powered) = self.power()?;
        Ok((if negative { -value } else { value }, powered))
    }

    fn missing_operand(&self) -> ParseError {
        if self.peek().is_none() && self.depth > 0 || self.peek() == Some(')') {
            ParseError::Parenthesis
        } else {
            ParseError::Invalid
        }
    }

    fn power(&mut self) -> Parsed {
        let (base, powered) = self.atom()?;
        self.skip_spaces();
        if !self.at_power() {
            if self.at_product() {
                return Err(ParseError::Product);
            }
            return Ok((base, powered));
        }
        let exponent = self.exponent()?;
        self.skip_spaces();
        if self.at_power() {
            return Err(ParseError::PowerChain);
        }
        let value = raise(&base, exponent)?;
        if self.at_product() {
            return Err(ParseError::Product);
        }
        Ok((value, true))
    }

    fn atom(&mut self) -> Parsed {
        self.skip_spaces();
        match self.peek() {
            Some('(') => {
                self.position += 1;
                self.depth += 1;
                let parsed = self.quotient()?;
                self.skip_spaces();
                if self.peek() != Some(')') {
                    return Err(if self.peek().is_none() {
                        ParseError::Parenthesis
                    } else {
                        ParseError::Invalid
                    });
                }
                self.position += 1;
                self.depth -= 1;
                Ok(parsed)
            }
            Some(character) if character.is_ascii_digit() || character == '.' => {
                Ok((self.number()?, false))
            }
            _ => Err(self.missing_operand()),
        }
    }

    fn number(&mut self) -> Result<Rational, ParseError> {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_digit() || character == '.')
        {
            self.position += 1;
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.position += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.position += 1;
            }
            while self
                .peek()
                .is_some_and(|character| character.is_ascii_digit())
            {
                self.position += 1;
            }
        }
        let text: String = self.characters[start..self.position].iter().collect();
        parse_decimal(&text)
    }

    fn exponent(&mut self) -> Result<i64, ParseError> {
        match self.peek() {
            Some('^') => self.position += 1,
            Some('*') => self.position += 2,
            _ => return self.superscript(),
        }
        self.skip_spaces();
        if self.peek() != Some('(') {
            return self.integer();
        }
        self.position += 1;
        self.skip_spaces();
        let value = self.integer()?;
        self.skip_spaces();
        match self.peek() {
            Some(')') => {}
            None => return Err(ParseError::Parenthesis),
            Some(_) => return Err(ParseError::PowerExponent),
        }
        self.position += 1;
        Ok(value)
    }

    fn integer(&mut self) -> Result<i64, ParseError> {
        let negative = match self.peek() {
            Some('-') => {
                self.position += 1;
                true
            }
            Some('+') => {
                self.position += 1;
                false
            }
            _ => false,
        };
        let start = self.position;
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_digit())
        {
            self.position += 1;
        }
        if self.position == start
            || matches!(self.peek(), Some('.' | 'e' | 'E'))
            || self
                .peek()
                .is_some_and(|character| character.is_alphanumeric() && !is_superscript(character))
        {
            return Err(ParseError::PowerExponent);
        }
        let digits: String = self.characters[start..self.position].iter().collect();
        bounded(&digits, negative)
    }

    fn superscript(&mut self) -> Result<i64, ParseError> {
        let negative = match self.peek() {
            Some(SUPERSCRIPT_MINUS) => {
                self.position += 1;
                true
            }
            Some(SUPERSCRIPT_PLUS) => {
                self.position += 1;
                false
            }
            _ => false,
        };
        let mut digits = String::new();
        while let Some(digit) = self.peek().and_then(superscript_digit) {
            digits.push(char::from_digit(digit, 10).unwrap_or('0'));
            self.position += 1;
        }
        if digits.is_empty() || self.peek().is_some_and(is_superscript) {
            return Err(ParseError::PowerExponent);
        }
        bounded(&digits, negative)
    }
}

fn bounded(digits: &str, negative: bool) -> Result<i64, ParseError> {
    let significant = digits.trim_start_matches('0');
    if significant.len() > 4 {
        return Err(ParseError::PowerExponent);
    }
    let value: i64 = if significant.is_empty() {
        0
    } else {
        significant.parse().map_err(|_| ParseError::PowerExponent)?
    };
    if value > i64::from(MAX_EXPONENT) {
        return Err(ParseError::PowerExponent);
    }
    Ok(if negative { -value } else { value })
}

fn too_large(value: &BigInt) -> bool {
    static LIMIT: OnceLock<BigInt> = OnceLock::new();
    let limit = LIMIT.get_or_init(|| BigInt::from(10u32).pow(MAX_POWER_DIGITS as u32));
    value.magnitude() >= limit.magnitude()
}

fn capped(value: Rational, powered: bool) -> Parsed {
    if powered && (too_large(value.numerator()) || too_large(value.denominator())) {
        return Err(ParseError::PowerTooLarge);
    }
    Ok((value, powered))
}

pub fn raise(base: &Rational, exponent: i64) -> Result<Rational, ParseError> {
    if exponent.unsigned_abs() > MAX_EXPONENT.unsigned_abs() as u64 {
        return Err(ParseError::PowerExponent);
    }
    if base.is_zero() {
        return match exponent {
            0 => Err(ParseError::ZeroToZero),
            negative if negative < 0 => Err(ParseError::ZeroNegativePower),
            _ => Ok(Rational::zero()),
        };
    }
    if exponent == 0 {
        return Ok(Rational::one());
    }
    let power = exponent.unsigned_abs() as u32;
    let widest = base.numerator().bits().max(base.denominator().bits());
    let limit_bits = (MAX_POWER_DIGITS as f64 * std::f64::consts::LOG2_10).ceil() as u64;
    if widest.saturating_sub(1).saturating_mul(u64::from(power)) > limit_bits {
        return Err(ParseError::PowerTooLarge);
    }
    let numerator = base.numerator().pow(power);
    let denominator = base.denominator().pow(power);
    if too_large(&numerator) || too_large(&denominator) {
        return Err(ParseError::PowerTooLarge);
    }
    if exponent > 0 {
        Ok(Rational::new(numerator, denominator))
    } else {
        Ok(Rational::new(denominator, numerator))
    }
}

pub fn parse(text: &str) -> Result<Rational, ParseError> {
    Parser {
        characters: text.chars().collect(),
        position: 0,
        depth: 0,
    }
    .value()
}
