using System;
using SistemasLineares;
class SolverTests {
 static int checks;
 static void Check(bool b, string msg) { checks++; if (!b) throw new Exception(msg); }
 static Rational R(string s) { return Rational.Parse(s); }
 static Rational[,] Matrix(string[,] s) { var a = new Rational[s.GetLength(0), s.GetLength(1)]; for(int i=0;i<s.GetLength(0);i++) for(int j=0;j<s.GetLength(1);j++) a[i,j]=R(s[i,j]); return a; }
 static void Main() {
  Check(R("-1,5").ToString()=="-3/2", "comma"); Check(R(".25").ToString()=="1/4", "decimal"); Check(R("2,5/0,5").ToString()=="5", "fraction of decimals");
  foreach(string s in new[]{"", "x", "1/0", "1/2/3", "1.2.3", "--2", "."}) { bool fail=false; try { R(s); } catch(FormatException) { fail=true; } Check(fail,"invalid "+s); }
  var a=Matrix(new[,]{{"2","1","5"},{"1","-1","1"}}); var sol=Solver.Solve(a); Check(sol.Kind==SolutionKind.Unique, "unique"); Check(sol.Reduced[0,2].ToString()=="2" && sol.Reduced[1,2].ToString()=="1", "unique values");
  sol=Solver.Solve(Matrix(new[,]{{"0","1","2"},{"1","0","3"}})); Check(sol.Reduced[0,2].ToString()=="3" && sol.Reduced[1,2].ToString()=="2", "pivot swap");
  sol=Solver.Solve(Matrix(new[,]{{"1","1","2"},{"2","2","4"}})); Check(sol.Kind==SolutionKind.Infinite, "infinite"); Check(Solver.Format(sol).Contains("x = 2 - t1"), "parametric");
  sol=Solver.Solve(Matrix(new[,]{{"1","1","2"},{"2","2","5"}})); Check(sol.Kind==SolutionKind.None, "inconsistent");
  sol=Solver.Solve(Matrix(new[,]{{"0","0","0","0"},{"0","0","0","0"},{"0","0","0","0"}})); Check(sol.Kind==SolutionKind.Infinite && Solver.Format(sol).Contains("z = t3"), "all free");
  sol=Solver.Solve(Matrix(new[,]{{"0","0","0","1"},{"0","0","0","0"},{"0","0","0","0"}})); Check(sol.Kind==SolutionKind.None, "zero contradiction");
  sol=Solver.Solve(Matrix(new[,]{{"1","1","1","6"},{"2","-1","1","3"},{"1","2","-1","2"}})); Check(sol.Kind==SolutionKind.Unique && sol.Reduced[0,3].Equals(R("1")) && sol.Reduced[1,3].Equals(R("2")) && sol.Reduced[2,3].Equals(R("3")), "3x3");
  Random random = new Random(19);
  for(int n=2; n<=3; n++) for(int k=0;k<500;k++) {
   var matrix=new Rational[n,n+1]; var expected=new Rational[n];
   for(int i=0;i<n;i++) expected[i]=new Rational(random.Next(-5,6),1);
   for(int i=0;i<n;i++) { for(int j=0;j<n;j++) { matrix[i,j]=new Rational(random.Next(-5,6),1); matrix[i,n] = matrix[i,n] + matrix[i,j]*expected[j]; } }
   var output=Solver.Solve(matrix); Check(output.Kind!=SolutionKind.None,"generated consistent");
   if(output.Kind==SolutionKind.Unique) for(int i=0;i<n;i++) Check(output.Reduced[i,n].Equals(expected[i]),"generated solution");
  }
  Console.WriteLine("PASS: " + checks + " verificações");
 }
}
