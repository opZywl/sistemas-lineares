(function(root) {
  'use strict';
  const abs = x => x < 0n ? -x : x;
  function gcd(a,b) { a=abs(a); b=abs(b); while(b) [a,b]=[b,a%b]; return a; }
  class Rational {
    constructor(n=0n,d=1n) {
      if(d===0n) throw new Error('O denominador não pode ser zero.');
      if(d<0n) { n=-n; d=-d; }
      const g=gcd(n,d); this.n=n/g; this.d=d/g;
    }
    add(b) { return new Rational(this.n*b.d+b.n*this.d,this.d*b.d); }
    sub(b) { return new Rational(this.n*b.d-b.n*this.d,this.d*b.d); }
    mul(b) { return new Rational(this.n*b.n,this.d*b.d); }
    div(b) { return new Rational(this.n*b.d,this.d*b.n); }
    neg() { return new Rational(-this.n,this.d); }
    eq(b) { return this.n===b.n && this.d===b.d; }
    get zero() { return this.n===0n; }
    toString() { return this.d===1n?String(this.n):`${this.n}/${this.d}`; }
    static parse(value) {
      const s=String(value??'').trim().replaceAll(',','.').replaceAll('−','-');
      if(!s) throw new Error('Preencha todos os campos.');
      if(s.length>100) throw new Error('Use números com no máximo 100 caracteres.');
      const parts=s.split('/');
      if(parts.length>2) throw new Error('Número inválido.');
      const decimal=text=>{
        const match=/^([+-]?)(\d+(?:\.\d*)?|\.\d+)(?:[eE]([+-]?\d+))?$/.exec(text.trim());
        if(!match) throw new Error('Número inválido.');
        const exponent=Number(match[3]||0);
        if(!Number.isInteger(exponent)||Math.abs(exponent)>1000) throw new Error('Use um expoente inteiro entre -1000 e 1000.');
        const [whole,frac='']=match[2].split('.');
        let n=BigInt((whole||'0')+frac)*(match[1]==='-'?-1n:1n), power=frac.length-exponent;
        return power>=0?new Rational(n,10n**BigInt(power)):new Rational(n*10n**BigInt(-power));
      };
      const a=decimal(parts[0]); return parts.length===1?a:a.div(decimal(parts[1]));
    }
  }
  function solve(input) {
    const n=input.length;
    if((n!==2&&n!==3)||input.some(row=>row.length!==n+1)) throw new Error('O sistema deve ser 2×2 ou 3×3.');
    const a=input.map(row=>row.slice()),pivots=[];
    let r=0;
    for(let col=0;col<n&&r<n;col++) {
      let selected=r; while(selected<n&&a[selected][col].zero) selected++;
      if(selected===n) continue;
      [a[r],a[selected]]=[a[selected],a[r]];
      const pivot=a[r][col]; a[r]=a[r].map(v=>v.div(pivot));
      for(let i=0;i<n;i++) if(i!==r&&!a[i][col].zero) { const factor=a[i][col]; a[i]=a[i].map((v,j)=>v.sub(factor.mul(a[r][j]))); }
      pivots.push(col); r++;
    }
    const inconsistent=a.some(row=>row.slice(0,n).every(v=>v.zero)&&!row[n].zero);
    return {kind:inconsistent?'none':(pivots.length===n?'unique':'infinite'),rref:a,pivots};
  }
  function format(solution) {
    if(solution.kind==='none') return 'Este sistema não tem solução.\nAs equações são incompatíveis.';
    const n=solution.rref.length,names=['x','y','z'],rowFor=Array(n).fill(-1),parameters=Array(n).fill(0);
    solution.pivots.forEach((col,row)=>rowFor[col]=row);
    let next=1; parameters.forEach((_,i)=>{if(rowFor[i]<0) parameters[i]=next++;});
    const rows=names.slice(0,n).map((name,col)=>{
      if(rowFor[col]<0) return `${name} = t${parameters[col]}`;
      const row=solution.rref[rowFor[col]],constant=row[n];
      let text=constant.zero?'':String(constant);
      for(let j=0;j<n;j++) if(parameters[j]) {
        const coeff=row[j].neg(); if(coeff.zero) continue;
        const negative=coeff.n<0n, magnitude=negative?coeff.neg():coeff;
        text+=text?(negative?' - ':' + '):(negative?'-':'');
        text+=(magnitude.eq(new Rational(1n))?'':`${magnitude} · `)+`t${parameters[j]}`;
      }
      return `${name} = ${text||'0'}`;
    });
    return [solution.kind==='unique'?'Solução única':'Infinitas soluções','',...rows,...(solution.kind==='infinite'?['','t1, t2, ... podem ser quaisquer números reais.']:[])].join('\n');
  }
  const api={Rational,solve,format};
  if(typeof module!=='undefined'&&module.exports) module.exports=api;
  else root.LinearSolver=api;
})(typeof globalThis!=='undefined'?globalThis:this);
