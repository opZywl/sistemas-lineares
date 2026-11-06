using System;
using System.Globalization;
using System.Numerics;
using System.Collections.Generic;

namespace SistemasLineares {
    public struct Rational : IEquatable<Rational> {
        private readonly BigInteger n, d;
        public BigInteger Numerator { get { return n; } }
        public BigInteger Denominator { get { return d.IsZero ? BigInteger.One : d; } }
        public bool IsZero { get { return n.IsZero; } }
        public Rational(BigInteger numerator, BigInteger denominator) {
            if (denominator.IsZero) throw new DivideByZeroException();
            if (denominator.Sign < 0) { numerator = -numerator; denominator = -denominator; }
            BigInteger gcd = BigInteger.GreatestCommonDivisor(BigInteger.Abs(numerator), denominator);
            n = numerator / gcd; d = denominator / gcd;
        }
        public static Rational Parse(string input) {
            string s = (input ?? "").Trim().Replace(',', '.').Replace('−','-');
            if (s.Length == 0) throw new FormatException("Preencha todos os campos.");
            if (s.Length > 100) throw new FormatException("Use números com no máximo 100 caracteres.");
            string[] parts = s.Split('/');
            if (parts.Length > 2) throw new FormatException("Número inválido.");
            Rational a = ParseDecimal(parts[0]);
            if (parts.Length == 1) return a;
            Rational b = ParseDecimal(parts[1]);
            if (b.IsZero) throw new FormatException("O denominador não pode ser zero.");
            return a / b;
        }
        private static Rational ParseDecimal(string input) {
            string s = input.Trim();
            if (s.Length == 0) throw new FormatException("Número inválido.");
            int exponent=0, e=s.IndexOfAny(new char[]{'e','E'});
            if(e>=0) {
                if(!int.TryParse(s.Substring(e+1),NumberStyles.AllowLeadingSign,CultureInfo.InvariantCulture,out exponent) || Math.Abs((long)exponent)>1000) throw new FormatException("Use um expoente inteiro entre -1000 e 1000.");
                s=s.Substring(0,e);
                if(s.Length==0) throw new FormatException("Número inválido.");
            }
            int sign = 1, pos = 0;
            if (s[0] == '-' || s[0] == '+') { if (s[0] == '-') sign = -1; pos++; }
            bool dot = false, digit = false; int decimals = 0;
            System.Text.StringBuilder digits = new System.Text.StringBuilder();
            for (; pos < s.Length; pos++) {
                char c = s[pos];
                if (c == '.' && !dot) { dot = true; continue; }
                if (c < '0' || c > '9') throw new FormatException("Número inválido.");
                digit = true; digits.Append(c); if (dot) decimals++;
            }
            if (!digit) throw new FormatException("Número inválido.");
            BigInteger numerator=BigInteger.Parse(digits.ToString(),CultureInfo.InvariantCulture)*sign;
            int power=decimals-exponent;
            return power>=0 ? new Rational(numerator,BigInteger.Pow(10,power)) : new Rational(numerator*BigInteger.Pow(10,-power),1);
        }
        public static Rational operator +(Rational a, Rational b) { return new Rational(a.n * b.Denominator + b.n * a.Denominator, a.Denominator * b.Denominator); }
        public static Rational operator -(Rational a, Rational b) { return new Rational(a.n * b.Denominator - b.n * a.Denominator, a.Denominator * b.Denominator); }
        public static Rational operator -(Rational a) { return new Rational(-a.n, a.Denominator); }
        public static Rational operator *(Rational a, Rational b) { return new Rational(a.n * b.n, a.Denominator * b.Denominator); }
        public static Rational operator /(Rational a, Rational b) { return new Rational(a.n * b.Denominator, a.Denominator * b.n); }
        public bool Equals(Rational other) { return n == other.n && Denominator == other.Denominator; }
        public override bool Equals(object other) { return other is Rational && Equals((Rational)other); }
        public override int GetHashCode() { return n.GetHashCode() ^ Denominator.GetHashCode(); }
        public override string ToString() { return Denominator.IsOne ? n.ToString() : n + "/" + Denominator; }
    }
    public enum SolutionKind { Unique, Infinite, None }
    public sealed class Solution {
        public SolutionKind Kind; public Rational[,] Reduced; public int[] PivotColumns;
    }
    public static class Solver {
        public static Solution Solve(Rational[,] input) {
            if (input == null) throw new ArgumentNullException("input");
            int rows = input.GetLength(0), cols = input.GetLength(1) - 1;
            if (rows != cols || (cols != 2 && cols != 3)) throw new ArgumentException("O sistema deve ser 2×2 ou 3×3.");
            Rational[,] a = (Rational[,])input.Clone();
            List<int> pivots = new List<int>(); int pivotRow = 0;
            for (int col = 0; col < cols && pivotRow < rows; col++) {
                int selected = pivotRow;
                while (selected < rows && a[selected, col].IsZero) selected++;
                if (selected == rows) continue;
                if (selected != pivotRow) for (int j = 0; j <= cols; j++) { Rational tmp = a[pivotRow,j]; a[pivotRow,j] = a[selected,j]; a[selected,j] = tmp; }
                Rational pivot = a[pivotRow,col];
                for (int j = 0; j <= cols; j++) a[pivotRow,j] = a[pivotRow,j] / pivot;
                for (int i = 0; i < rows; i++) if (i != pivotRow && !a[i,col].IsZero) {
                    Rational factor = a[i,col];
                    for (int j = 0; j <= cols; j++) a[i,j] = a[i,j] - factor * a[pivotRow,j];
                }
                pivots.Add(col); pivotRow++;
            }
            for (int i = 0; i < rows; i++) {
                bool zero = true; for (int j = 0; j < cols; j++) if (!a[i,j].IsZero) zero = false;
                if (zero && !a[i,cols].IsZero) return new Solution { Kind = SolutionKind.None, Reduced = a, PivotColumns = pivots.ToArray() };
            }
            return new Solution { Kind = pivots.Count == cols ? SolutionKind.Unique : SolutionKind.Infinite, Reduced = a, PivotColumns = pivots.ToArray() };
        }
        public static string Format(Solution solution) {
            if (solution.Kind == SolutionKind.None) return "Este sistema não tem solução.\r\nAs equações são incompatíveis.";
            int count = solution.Reduced.GetLength(1) - 1;
            string[] names = { "x", "y", "z" }; int[] pivotForColumn = new int[count];
            for (int i = 0; i < count; i++) pivotForColumn[i] = -1;
            for (int i = 0; i < solution.PivotColumns.Length; i++) pivotForColumn[solution.PivotColumns[i]] = i;
            int[] parameters = new int[count]; int next = 1;
            for (int i = 0; i < count; i++) if (pivotForColumn[i] < 0) parameters[i] = next++;
            System.Text.StringBuilder b = new System.Text.StringBuilder();
            b.AppendLine(solution.Kind == SolutionKind.Unique ? "Solução única" : "Infinitas soluções"); b.AppendLine();
            for (int col = 0; col < count; col++) {
                b.Append(names[col]).Append(" = "); int row = pivotForColumn[col];
                if (row < 0) b.Append("t").Append(parameters[col]);
                else {
                    Rational constant = solution.Reduced[row,count]; bool hasTerm = !constant.IsZero;
                    if (hasTerm) b.Append(constant.ToString());
                    for (int j = 0; j < count; j++) if (parameters[j] != 0) {
                        Rational coeff = -solution.Reduced[row,j]; if (coeff.IsZero) continue;
                        bool negative = coeff.Numerator.Sign < 0; Rational magnitude = negative ? -coeff : coeff;
                        if (hasTerm) b.Append(negative ? " - " : " + "); else if (negative) b.Append("-");
                        if (!magnitude.Equals(new Rational(1,1))) b.Append(magnitude.ToString()).Append(" · ");
                        b.Append("t").Append(parameters[j]); hasTerm = true;
                    }
                    if (!hasTerm) b.Append("0");
                }
                b.AppendLine();
            }
            if (solution.Kind == SolutionKind.Infinite) b.AppendLine("\r\nt1, t2, ... podem ser quaisquer números reais.");
            return b.ToString().TrimEnd();
        }
    }
}
