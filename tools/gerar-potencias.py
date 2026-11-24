from pathlib import Path
from fractions import Fraction

PE = "Use um expoente inteiro de -1000 a 1000, como 2^3 ou 10^-2."
PC = "Use parênteses para potência de potência, como (2^3)^2."
PT = "Use números menores: com potências, o limite é de 300 algarismos."
PR = "Faça a multiplicação antes: para 1,5×10³, digite 1,5e3 ou 1500."
ZZ = "0 elevado a 0 é indeterminado."
ZN = "0 não pode ser elevado a expoente negativo."
PA = "Confira os parênteses."
INV = "Número inválido."
ZD = "O denominador não pode ser zero."
EXP = "Use um expoente inteiro entre -1000 e 1000."
LONG = "Use números com no máximo 100 caracteres."


def show(value):
    value = Fraction(value)
    if value.denominator == 1:
        return str(value.numerator)
    return f"{value.numerator}/{value.denominator}"


def ok(value):
    return "OK:" + show(value)


def erro(message):
    return "ERRO:" + message


F = Fraction
cases = [
    ("2^3", ok(8)), ("2^0", ok(1)), ("5^0", ok(1)), ("2^1", ok(2)), ("2^10", ok(1024)),
    ("10^3", ok(1000)), ("10^-3", ok(F(1, 1000))), ("2^-1", ok(F(1, 2))), ("2^-3", ok(F(1, 8))),
    ("2^(-2)", ok(F(1, 4))), ("2^(+2)", ok(4)), ("2^+3", ok(8)), ("2^(3)", ok(8)), ("2^( 3 )", ok(8)),
    ("2 ^ 3", ok(8)), ("2 ^3", ok(8)), ("2^ 3", ok(8)), (" 2^3 ", ok(8)), ("2**3", ok(8)),
    ("2 ** 3", ok(8)), ("2**-2", ok(F(1, 4))), ("2*3", erro(PR)), ("2***3", erro(PE)), ("2* *3", erro(PR)),
    ("2\u00b2", ok(4)), ("2\u00b3", ok(8)), ("10\u00b3", ok(1000)), ("2\u00b9", ok(2)), ("2\u2070", ok(1)),
    ("2\u00b9\u2070", ok(1024)), ("2\u207b\u00b9", ok(F(1, 2))), ("2\u207b\u00b2", ok(F(1, 4))),
    ("2\u207a\u00b3", ok(8)), ("(1/2)\u00b2", ok(F(1, 4))), ("(1/2)\u00b3", ok(F(1, 8))),
    ("(-2)\u00b2", ok(4)), ("(-2)\u00b3", ok(-8)), ("-2\u00b2", ok(-4)), ("\u22122\u00b2", ok(-4)),
    ("2 \u00b2", ok(4)), ("\u00b2", erro(INV)), ("2\u207b", erro(PE)), ("2\u207a", erro(PE)),
    ("2\u00b2\u00b3", ok(2 ** 23)), ("2\u00b2 \u00b3", erro(PC)), ("2\u00b2^3", erro(PC)), ("2^3\u00b2", erro(PC)),
    ("2^3^2", erro(PC)), ("2^3^", erro(PC)), ("(2^3)^2", ok(64)), ("(2\u00b3)\u00b2", ok(64)), ("((2))^3", ok(8)),
    ("((2^2))^2", ok(16)), ("10\u207b\u00b3", ok(F(1, 1000))), ("1\u00b2", ok(1)), ("(1)", ok(1)),
    ("-2^2", ok(-4)), ("+2^2", ok(4)), ("(-2)^2", ok(4)), ("-(2)^2", ok(-4)), ("-(-2)^2", ok(-4)),
    ("(-2)^3", ok(-8)), ("-2^3", ok(-8)), ("-(2^3)", ok(-8)), ("--2^2", erro(INV)), ("-+2^2", erro(INV)),
    ("- 2^2", erro(INV)), ("-(2", erro(PA)), ("\u2212(1/2)\u00b2", ok(F(-1, 4))), ("(\u22121/2)\u00b2", ok(F(1, 4))),
    ("2^\u22121", ok(F(1, 2))), ("2^(\u22121)", ok(F(1, 2))),
    ("1/2^2", ok(F(1, 4))), ("2^2/3", ok(F(4, 3))), ("2^2/2^3", ok(F(1, 2))), ("(1/2)^-1", ok(2)),
    ("(2/3)^2", ok(F(4, 9))), ("(2/3)^-2", ok(F(9, 4))), ("(-2/3)^3", ok(F(-8, 27))), ("(1/2)/(3/4)", ok(F(2, 3))),
    ("(1/2)/3", ok(F(1, 6))), ("1/(2)", ok(F(1, 2))), ("1/(2/3)", ok(F(3, 2))), ("(1)/(2)", ok(F(1, 2))),
    ("1/2/3^2", erro(INV)), ("(1/2/3)^2", erro(INV)), ("1/0^2", erro(ZD)), ("1/(0)", erro(ZD)),
    ("1/(2-2)", erro(INV)), ("0/(1)", ok(0)), ("(0)/0", erro(ZD)), ("(1/0)", erro(ZD)), ("2^3/0", erro(ZD)),
    ("0.5^2", ok(F(1, 4))), ("0,5^2", ok(F(1, 4))), ("1,5\u00b2", ok(F(9, 4))), (".5^2", ok(F(1, 4))),
    ("5.^2", ok(25)), ("2.5^2", ok(F(25, 4))), ("-0,5^2", ok(F(-1, 4))), ("(-0,5)^2", ok(F(1, 4))),
    ("1.000^3", ok(1)), ("1e3^2", ok(10 ** 6)), ("2e3^2", ok(4 * 10 ** 6)), ("1e-3^2", ok(F(1, 10 ** 6))),
    ("2.5e1^2", ok(625)), ("1e3\u00b2", ok(10 ** 6)), ("(1e2)^-1", ok(F(1, 100))), ("2E2^2", ok(40000)),
    ("1e^2", erro(EXP)), ("1e1001^2", erro(EXP)), ("1e5)", erro(PA)), ("(1e5", erro(PA)),
    ("2^", erro(PE)), ("2^0.5", erro(PE)), ("2^0,5", erro(PE)), ("2^(1/2)", erro(PE)), ("2^x", erro(PE)),
    ("2^1e3", erro(PE)), ("2^(", erro(PE)), ("2^()", erro(PE)), ("2^(2", erro(PA)), ("2^(2))", erro(PA)),
    ("2^--1", erro(PE)), ("2^ - 1", erro(PE)), ("2^1001", erro(PE)), ("2^-1001", erro(PE)), ("2^99999", erro(PE)),
    ("2^00003", ok(8)), ("2^0000000000003", ok(8)), ("2^3x", erro(PE)), ("2^3,5", erro(PE)), ("2^3 x", erro(INV)),
    ("x^2", erro(INV)), ("\u221a2^2", erro(INV)),
    ("2^1000", erro(PT)), ("2^-1000", erro(PT)), ("10^1000", erro(PT)),
    ("10^-1000", erro(PT)), ("9^1000", erro(PT)), ("99^1000", erro(PT)),
    ("101^1000", erro(PT)), ("(1/99)^1000", erro(PT)), ("(1/101)^1000", erro(PT)),
    ("(10^1000)^2", erro(PT)), ("(1/3)^-1000", erro(PT)), ("(-1)^1000", ok(1)), ("(-1)^999", ok(-1)),
    ("(-1)^-1", ok(-1)), ("1^1000", ok(1)), ("1^-1000", ok(1)), ("0^1000", ok(0)),
    ("(10^1000)/(10^-1000)", erro(PT)), ("99999999999999999999^2", ok(99999999999999999999 ** 2)),
    ("0^0", erro(ZZ)), ("0^5", ok(0)), ("0^-1", erro(ZN)), ("0^-5", erro(ZN)), ("0\u2070", erro(ZZ)),
    ("0,0^0", erro(ZZ)), ("(0)^0", erro(ZZ)), ("-0^0", erro(ZZ)), ("(1-1)^0", erro(INV)),
    ("(2)", ok(2)), ("((3))", ok(3)), ("(-3)", ok(-3)), ("-(3)", ok(-3)), ("+(3)", ok(3)), ("( 3 )", ok(3)),
    ("(1/2)", ok(F(1, 2))), ("(2", erro(PA)), ("2)", erro(PA)), ("()", erro(PA)), ("(", erro(PA)), (")", erro(PA)),
    ("(()", erro(PA)), ("(2))", erro(PA)), ("((2)", erro(PA)), ("(2)(3)", erro(INV)), ("2(3)", erro(INV)),
    ("(2)3", erro(INV)), ("(2 3)", erro(INV)), ("-", erro(INV)) if False else ("-(", erro(PA)),
    ("2\u3000^\u30003", ok(8)), ("2\u00a0^2", ok(4)), ("2\u0085^2", ok(4)), ("2\u2009^\u20092", ok(4)),
    ("2\ufeff^2", erro(INV)), ("2\u200b^2", erro(INV)), ("\u00a02^2\u00a0", ok(4)), ("\u00852^2\u0085", ok(4)),
    ("\u0085" + "1", ok(1)), ("\ufeff" + "1", erro(INV)), ("1\ufeff", erro(INV)), ("\u20281", ok(1)),
    ("2^" + "1" * 98, erro(PE)), ("(" * 50 + "2" + ")" * 50, erro(LONG)), ("(" * 49 + "2" + ")" * 49, ok(2)),
    ("2^3" + " " * 200, ok(8)), ("((((((((((2^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)", erro(PT)),
    ("(((((((((((2^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)", erro(PT)),
    ("((((((((((((2^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)", erro(PT)),
    ("(((((((((((((2^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)^2)", erro(PT)),
]

cases += [
    ("2\u0085^\u00853", ok(8)), ("2﻿^3", erro(INV)), ("1\t^\t2", ok(1)),
    ("(-1e1000/1e-999)^1", erro(PT)), ("(1e1000/1e-1000)", ok(10 ** 2000)),
    ("(1e1000/1e-1000)^1", erro(PT)), ("0^1001", erro(PE)), ("0^-1001", erro(PE)), ("2^2²", erro(PC)),
    ("(2x", erro(INV)), ("2^(- 2)", erro(PE)), ("1" * 99 + "²", ok(int("1" * 99) ** 2)),
    ("1" * 100 + "²", erro(LONG)), (" " * 200 + "2^3" + " " * 200, ok(8)),
    ("1\t/\t2", ok(F(1, 2))), ("\\", erro(INV)), ("2\\3", erro(INV)),
]

cases += [
    ("(((((((((2^2)^2)^2)^2)^2)^2)^2)^2)^2)", ok(2 ** 512)),
    ("2^996", ok(2 ** 996)), ("2^997", erro(PT)), ("2^-996", ok(F(1, 2 ** 996))), ("2^-997", erro(PT)),
    ("10^299", ok(10 ** 299)), ("10^300", erro(PT)), ("(1/10)^299", ok(F(1, 10 ** 299))),
    ("(1/10)^300", erro(PT)), ("(9.999e99)^3", ok(9999 ** 3 * 10 ** 288)), ("(1e100)^3", erro(PT)),
    ("(-9.999e99)^3", ok(-(9999 ** 3) * 10 ** 288)), ("3^628", ok(3 ** 628)), ("3^629", erro(PT)),
    ("(2/3)^628", ok(F(2 ** 628, 3 ** 628))), ("(2/3)^629", erro(PT)),
    ("(10^200)/(10^-99)", ok(10 ** 299)), ("(10^200)/(10^-100)", erro(PT)),
    ("2^500/2^499", ok(2)), ("(10^150)*1", erro(PR)), ("1e1000", ok(10 ** 1000)),
    ("(1e1000/1e-1000)^0", ok(1)), ("((1e1000))", ok(10 ** 1000)), ("(1e1000)^1", erro(PT)),
    ("2\u00b710^3", erro(PR)), ("1,5\u00d710\u00b3", erro(PR)), ("2\u00d73", erro(PR)),
    ("2 * 3", erro(PR)), ("(2*3)", erro(PR)), ("1/2*3", erro(PR)), ("2^3*2", erro(PR)),
    ("0^0*2", erro(ZZ)), ("2*", erro(PR)), ("*2", erro(INV)), ("2\u22c53", erro(PR)),
    ("2**3*2", erro(PR)), ("2^(3*4)", erro(PE)), ("\u00d7", erro(INV)),
    ("2/3^2", ok(F(2, 9))), ("3/2\u00b2", ok(F(3, 4))), ("-1/2^2", ok(F(-1, 4))),
    ("1/-2^2", ok(F(-1, 4))), ("1/(-2)^2", ok(F(1, 4))), ("2\u00b3\u00b2", ok(2 ** 32)),
    ("2\u2071", erro(INV)), ("2\u00b2\u2071", erro(INV)), ("2^1001/0", erro(PE)),
    ("1/0^2 x", erro(ZD)), ("(2/)", erro(PA)), ("(-)", erro(PA)), ("(/2)", erro(INV)),
    ("(\u00b2)", erro(INV)), ("2^((3))", erro(PE)), ("2^-(3)", erro(PE)),
    ("2\u00b2\u207b\u00b3", erro(PE)),
]


def escape(texto):
    saida = []
    for caractere in texto:
        codigo = ord(caractere)
        if caractere == "\\":
            saida.append("\\\\")
        elif 32 <= codigo < 127:
            saida.append(caractere)
        else:
            saida.append("\\u{%X}" % codigo)
    return "".join(saida)


lines = []
seen = set()
for entrada, esperado in cases:
    assert entrada not in seen, entrada
    seen.add(entrada)
    lines.append(f"{escape(entrada)}\t{esperado}")
path = Path(__file__).resolve().parent.parent / "tests" / "potencias.txt"
with open(path, "w", encoding="utf-8", newline="\n") as handle:
    handle.write("\n".join(lines) + "\n")
print(len(lines), "casos")
