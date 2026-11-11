use std::collections::HashSet;

use num_bigint::BigInt;
use sistemas_lineares_core::{ParseError, Rational, ShapeError, solve};

fn q(numerador: i64, denominador: i64) -> Rational {
    Rational::new(BigInt::from(numerador), BigInt::from(denominador))
}

fn b(valor: i64) -> BigInt {
    BigInt::from(valor)
}

fn partes(valor: &Rational) -> (BigInt, BigInt) {
    (valor.numerator().clone(), valor.denominator().clone())
}

fn potencia_de_dez(expoente: u32) -> BigInt {
    BigInt::from(10).pow(expoente)
}

#[test]
fn sinal_fica_sempre_no_numerador() {
    assert_eq!(partes(&q(1, -2)), (b(-1), b(2)));
    assert_eq!(partes(&q(-1, -2)), (b(1), b(2)));
    assert_eq!(partes(&q(-1, 2)), (b(-1), b(2)));
    assert_eq!(partes(&q(3, -1)), (b(-3), b(1)));
    assert!(q(1, -2).is_negative());
    assert!(!q(-1, -2).is_negative());
}

#[test]
fn reduz_pelo_mdc() {
    assert_eq!(partes(&q(6, 4)), (b(3), b(2)));
    assert_eq!(partes(&q(-6, 4)), (b(-3), b(2)));
    assert_eq!(partes(&q(6, -4)), (b(-3), b(2)));
    assert_eq!(partes(&q(12, 18)), (b(2), b(3)));
    assert_eq!(partes(&q(100, 25)), (b(4), b(1)));
    assert_eq!(partes(&q(17, 17)), (b(1), b(1)));
    assert_eq!(partes(&q(-17, -17)), (b(1), b(1)));
    assert_eq!(partes(&q(1, 3)), (b(1), b(3)));
    assert_eq!(partes(&q(i64::MAX, i64::MAX)), (b(1), b(1)));
    assert_eq!(partes(&q(i64::MIN, 2)), (b(i64::MIN / 2), b(1)));
    let grande = Rational::new(potencia_de_dez(60) * 6, potencia_de_dez(60) * -4);
    assert_eq!(partes(&grande), (b(-3), b(2)));
}

#[test]
fn zero_e_sempre_0_sobre_1() {
    let zeros = [
        q(0, 5),
        q(0, -7),
        q(0, 1),
        Rational::zero(),
        Rational::integer(0),
        &q(1, 2) - &q(1, 2),
        &q(3, 4) * &Rational::zero(),
        &Rational::zero() / &q(-9, 4),
        -Rational::zero(),
        Rational::parse("-0").unwrap(),
        Rational::parse("0/-3").unwrap(),
        Rational::parse("-0.000e5").unwrap(),
    ];
    for zero in &zeros {
        assert_eq!(partes(zero), (b(0), b(1)));
        assert!(zero.is_zero());
        assert!(!zero.is_negative());
        assert!(!zero.is_one());
        assert_eq!(zero.to_string(), "0");
        assert_eq!(zero, &Rational::zero());
    }
    assert_eq!(zeros.iter().cloned().collect::<HashSet<_>>().len(), 1);
}

#[test]
fn exibicao_igual_ao_tostring_do_csharp() {
    assert_eq!(q(5, 1).to_string(), "5");
    assert_eq!(q(-5, 1).to_string(), "-5");
    assert_eq!(q(3, 2).to_string(), "3/2");
    assert_eq!(q(-3, 2).to_string(), "-3/2");
    assert_eq!(q(3, -2).to_string(), "-3/2");
    assert_eq!(q(10, 5).to_string(), "2");
    assert_eq!(q(1, 100).to_string(), "1/100");
    assert_eq!(format!("{}", q(-7, 3)), "-7/3");
    assert_eq!(
        Rational::new(potencia_de_dez(30) + 1, b(-7)).to_string(),
        "-1000000000000000000000000000001/7"
    );
}

#[test]
fn operacoes_exatas() {
    assert_eq!(&q(1, 2) + &q(1, 3), q(5, 6));
    assert_eq!(&q(1, 2) - &q(1, 3), q(1, 6));
    assert_eq!(&q(1, 3) - &q(1, 2), q(-1, 6));
    assert_eq!(&q(2, 3) * &q(3, 4), q(1, 2));
    assert_eq!(&q(-2, 3) * &q(-3, 4), q(1, 2));
    assert_eq!(&q(1, 2) / &q(-1, 4), q(-2, 1));
    assert_eq!(&q(-1, 2) / &q(-1, 4), q(2, 1));
    assert_eq!(-&q(3, 4), q(-3, 4));
    assert_eq!(-q(-3, 4), q(3, 4));
    assert_eq!(q(1, 2) + q(1, 2), Rational::one());
    assert_eq!(q(1, 2) + &q(1, 2), Rational::one());
    assert_eq!(&q(1, 2) + q(1, 2), Rational::one());
    assert_eq!(q(5, 6) - q(1, 3), q(1, 2));
    assert_eq!(q(5, 6) * q(6, 5), Rational::one());
    assert_eq!(q(5, 6) / q(5, 6), Rational::one());
    assert_eq!((&q(1, 10) + &q(2, 10)).to_string(), "3/10");
    let terco = q(1, 3);
    let soma = (0..3).fold(Rational::zero(), |total, _| &total + &terco);
    assert!(soma.is_one());
    let pequeno = Rational::parse("1e-50").unwrap();
    assert_eq!((&(&Rational::one() + &pequeno) - &Rational::one()), pequeno);
}

#[test]
fn constantes_e_predicados() {
    assert_eq!(partes(&Rational::zero()), (b(0), b(1)));
    assert_eq!(partes(&Rational::one()), (b(1), b(1)));
    assert_eq!(partes(&Rational::integer(-42)), (b(-42), b(1)));
    assert_eq!(
        partes(&Rational::integer(potencia_de_dez(40))),
        (potencia_de_dez(40), b(1))
    );
    assert!(Rational::one().is_one());
    assert!(q(7, 7).is_one());
    assert!(!q(-1, 1).is_one());
    assert!(!q(1, 2).is_one());
    assert!(!Rational::one().is_zero());
    assert!(q(-1, 3).is_negative());
    assert!(!q(1, 3).is_negative());
}

#[test]
fn igualdade_e_hash_consistentes() {
    let iguais = [
        q(1, 2),
        q(2, 4),
        q(-1, -2),
        q(50, 100),
        Rational::parse("0,5").unwrap(),
        Rational::parse("5e-1").unwrap(),
        Rational::parse("\u{2212}1/\u{2212}2").unwrap(),
    ];
    assert_eq!(iguais.iter().cloned().collect::<HashSet<_>>().len(), 1);
    assert_ne!(q(1, 2), q(-1, 2));
    assert_ne!(q(1, 2), q(1, 3));
}

#[test]
#[should_panic(expected = "denominador zero")]
fn denominador_zero_entra_em_panico() {
    let _ = q(1, 0);
}

#[test]
#[should_panic(expected = "denominador zero")]
fn divisao_por_zero_entra_em_panico() {
    let _ = &q(1, 2) / &Rational::zero();
}

#[test]
fn mensagens_de_erro_iguais_ao_csharp() {
    let mensagens = [
        (ParseError::Empty, "Preencha todos os campos."),
        (
            ParseError::TooLong,
            "Use números com no máximo 100 caracteres.",
        ),
        (ParseError::Invalid, "Número inválido."),
        (
            ParseError::Exponent,
            "Use um expoente inteiro entre -1000 e 1000.",
        ),
        (
            ParseError::ZeroDenominator,
            "O denominador não pode ser zero.",
        ),
    ];
    for (erro, mensagem) in mensagens {
        assert_eq!(erro.message(), mensagem);
        assert_eq!(erro.to_string(), mensagem);
    }
    assert_eq!(ShapeError.to_string(), "O sistema deve ser 2×2 ou 3×3.");
}

#[test]
fn formatos_de_sistema_invalidos_sao_rejeitados() {
    let linha = |tamanho: usize| vec![Rational::one(); tamanho];
    assert_eq!(solve(&[]), Err(ShapeError));
    assert_eq!(solve(&[linha(2)]), Err(ShapeError));
    assert_eq!(solve(&[linha(2), linha(2)]), Err(ShapeError));
    assert_eq!(solve(&[linha(4), linha(4)]), Err(ShapeError));
    assert_eq!(solve(&[linha(3), linha(4)]), Err(ShapeError));
    assert_eq!(solve(&[linha(3), linha(3), linha(3)]), Err(ShapeError));
    assert_eq!(solve(&[linha(4), linha(4), linha(3)]), Err(ShapeError));
    assert_eq!(
        solve(&[linha(5), linha(5), linha(5), linha(5)]),
        Err(ShapeError)
    );
    assert!(solve(&[linha(3), linha(3)]).is_ok());
    assert!(solve(&[linha(4), linha(4), linha(4)]).is_ok());
}
