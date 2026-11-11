using System;
using System.Numerics;
using System.IO;
using System.Text;
using SistemasLineares;

// Oráculo independente: posto por todos os menores e soluções por Cramer.
// Nenhum dos dois utiliza eliminação de Gauss-Jordan.
class RobustnessTests {
    static long assertions, systems;
    static StreamWriter fixtures;
    static void Check(bool condition,string message) { assertions++; if(!condition) throw new Exception(message); }
    static Rational R(int n) { return new Rational(n,1); }
    static BigInteger Det(BigInteger[,] a) {
        int n=a.GetLength(0);
        if(n==1) return a[0,0];
        if(n==2) return a[0,0]*a[1,1]-a[0,1]*a[1,0];
        return a[0,0]*(a[1,1]*a[2,2]-a[1,2]*a[2,1])-a[0,1]*(a[1,0]*a[2,2]-a[1,2]*a[2,0])+a[0,2]*(a[1,0]*a[2,1]-a[1,1]*a[2,0]);
    }
    static int Bits(int mask) { int n=0; while(mask!=0) { n+=mask&1; mask>>=1; } return n; }
    static int Rank(BigInteger[,] a) {
        int rows=a.GetLength(0),cols=a.GetLength(1);
        for(int k=Math.Min(rows,cols);k>=1;k--) for(int r=1;r<(1<<rows);r++) if(Bits(r)==k) for(int c=1;c<(1<<cols);c++) if(Bits(c)==k) {
            BigInteger[,] minor=new BigInteger[k,k]; int ri=0;
            for(int i=0;i<rows;i++) if((r&(1<<i))!=0) { int ci=0; for(int j=0;j<cols;j++) if((c&(1<<j))!=0) minor[ri,ci++]=a[i,j]; ri++; }
            if(Det(minor)!=0) return k;
        }
        return 0;
    }
    static void Audit(int[,] integers) {
        systems++; int n=integers.GetLength(0); Rational[,] input=new Rational[n,n+1]; BigInteger[,] coeff=new BigInteger[n,n],aug=new BigInteger[n,n+1];
        for(int i=0;i<n;i++) for(int j=0;j<=n;j++) { input[i,j]=R(integers[i,j]); aug[i,j]=integers[i,j]; if(j<n) coeff[i,j]=integers[i,j]; }
        Solution answer=Solver.Solve(input); int ra=Rank(coeff),rab=Rank(aug);
        if(fixtures!=null) {
            string[] entries=new string[n*(n+1)]; int k=0; foreach(int value in integers) entries[k++]=value.ToString(System.Globalization.CultureInfo.InvariantCulture);
            fixtures.WriteLine(n+"|"+string.Join(",",entries)+"|"+answer.Kind.ToString().ToLowerInvariant()+"|"+Convert.ToBase64String(Encoding.UTF8.GetBytes(Solver.Format(answer).Replace("\r\n","\n"))));
        }
        SolutionKind expected=ra<rab?SolutionKind.None:(ra==n?SolutionKind.Unique:SolutionKind.Infinite);
        Check(answer.Kind==expected,"classificação independente");
        for(int i=0;i<n;i++) for(int j=0;j<=n;j++) Check(input[i,j].Equals(R(integers[i,j])),"entrada preservada");
        if(answer.Kind==SolutionKind.None) return;
        if(answer.Kind==SolutionKind.Unique) {
            BigInteger denominator=Det(coeff);
            for(int col=0;col<n;col++) {
                BigInteger[,] replaced=(BigInteger[,])coeff.Clone(); for(int row=0;row<n;row++) replaced[row,col]=integers[row,n];
                Check(answer.Reduced[col,n].Equals(new Rational(Det(replaced),denominator)),"solução por Cramer");
            }
        }
        int[] rowFor=new int[n]; for(int i=0;i<n;i++) rowFor[i]=-1;
        for(int i=0;i<answer.PivotColumns.Length;i++) rowFor[answer.PivotColumns[i]]=i;
        for(int sample=0;sample<4;sample++) {
            Rational[] x=new Rational[n]; for(int j=0;j<n;j++) if(rowFor[j]<0) x[j]=new Rational((sample-1)*(j+1),sample+1);
            for(int j=0;j<n;j++) if(rowFor[j]>=0) { int row=rowFor[j]; x[j]=answer.Reduced[row,n]; for(int free=0;free<n;free++) if(rowFor[free]<0) x[j]=x[j]-answer.Reduced[row,free]*x[free]; }
            for(int i=0;i<n;i++) { Rational total=R(0); for(int j=0;j<n;j++) total=total+input[i,j]*x[j]; Check(total.Equals(input[i,n]),"substituição nas equações originais"); }
        }
    }
    static void Main(string[] args) {
        if(args.Length>0) fixtures=new StreamWriter(args[0],false,new UTF8Encoding(false));
        // Todos os sistemas 2x2 com cada entrada inteira de -2 a 2.
        for(int code=0;code<15625;code++) { int value=code; int[,] a=new int[2,3]; for(int i=0;i<2;i++) for(int j=0;j<3;j++) { a[i,j]=value%5-2; value/=5; } Audit(a); }
        // Todas as matrizes 3x3 com entradas -1, 0 e 1, para três vetores b.
        for(int code=0;code<19683;code++) {
            int value=code; int[,] a=new int[3,4]; for(int i=0;i<3;i++) for(int j=0;j<3;j++) { a[i,j]=value%3-1; value/=3; }
            for(int b=0;b<3;b++) { for(int i=0;i<3;i++) a[i,3]=b==0?0:(b==1?i-1:i+1); Audit(a); }
        }
        Random random=new Random(71291);
        for(int k=0;k<2000;k++) { int n=k%2+2; int[,] a=new int[n,n+1]; for(int i=0;i<n;i++) for(int j=0;j<=n;j++) a[i,j]=random.Next(-1000,1001); Audit(a); }
        Check(Rational.Parse("1e-20").ToString()=="1/100000000000000000000","expoente negativo");
        Check(Rational.Parse("−2,5E+3").ToString()=="-2500","sinal unicode e expoente");
        Check(Rational.Parse("1e100/1e100").Equals(R(1)),"fração grande");
        foreach(string bad in new[]{"1e", "1e1001", "1e-2147483648", "NaN", "Infinity", "1e1e1", "1/0", "1,2.3", new string('1',101)}) {
            bool rejected=false; try { Rational.Parse(bad); } catch(FormatException) { rejected=true; } Check(rejected,"entrada inválida: "+bad);
        }
        Rational tiny=Rational.Parse("1e-50"),one=R(1);
        Solution near=Solver.Solve(new Rational[,]{{one,one,R(2)},{one,one+tiny,R(2)+tiny}});
        Check(near.Kind==SolutionKind.Unique && near.Reduced[0,2].Equals(one) && near.Reduced[1,2].Equals(one),"quase singular, cálculo exato");
        Rational huge=Rational.Parse(new string('9',100));
        Solution large=Solver.Solve(new Rational[,]{{huge,one,huge+one},{one,-huge,one-huge}});
        Check(large.Kind==SolutionKind.Unique && large.Reduced[0,2].Equals(one) && large.Reduced[1,2].Equals(one),"100 dígitos");
        if(fixtures!=null) fixtures.Dispose();
        Console.WriteLine("PASS: "+systems+" sistemas auditados; "+assertions+" verificações independentes");
    }
}
