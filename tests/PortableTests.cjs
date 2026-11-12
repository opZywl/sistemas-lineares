const fs=require('node:fs');
const {Rational:R,solve,format,METHODS,walkthrough,walkthroughText,matrixLines}=require('../portable/solver.js');
const path=require('node:path');
let checks=0,systems=0,walkthroughs=0;
function check(value,label){checks++;if(!value)throw Error(label);}
const lines=fs.readFileSync(process.argv[2],'utf8').trim().split(/\r?\n/);
for(const line of lines){
 const [size,entries,kind,encoded]=line.split('|'),n=Number(size),values=entries.split(',').map(R.parse),a=Array.from({length:n},(_,i)=>values.slice(i*(n+1),(i+1)*(n+1))),answer=solve(a);systems++;
 check(answer.kind===kind,`classificação, sistema ${systems}`);
 check(format(answer)===Buffer.from(encoded,'base64').toString('utf8'),`resposta exata, sistema ${systems}`);
}
if(process.argv[3]){
 const passos=fs.readFileSync(process.argv[3],'utf8').trim().split(/\r?\n/);
 for(const line of passos){
  const [size,entries,method,encoded]=line.split('|'),n=Number(size),values=entries.split(';').map(R.parse),a=Array.from({length:n},(_,i)=>values.slice(i*(n+1),(i+1)*(n+1)));
  const system=a.map(row=>row.join(' ')).join(' ; '),walk=walkthrough(a,method),expected=Buffer.from(encoded,'base64').toString('utf8'),actual=walkthroughText(walk);walkthroughs++;
  check(walk.kind===solve(a).kind,`Modo pensar (${method}), classificação de ${system}`);
  if(actual!==expected){
   const want=expected.split('\n'),got=actual.split('\n');let index=0;
   while(index<want.length&&index<got.length&&want[index]===got[index])index++;
   console.error(`Método: ${method}\nSistema: ${system}\nPrimeira linha diferente (${index+1}):\n  Rust:       ${JSON.stringify(want[index]??'(fim do texto)')}\n  JavaScript: ${JSON.stringify(got[index]??'(fim do texto)')}`);
  }
  check(actual===expected,`Modo pensar (${method}) diferente do Rust em ${system}`);
 }
}
check(METHODS.map(m=>m.key).join()==='gauss-jordan,cramer,gauss','ordem dos métodos');
check(METHODS.map(m=>m.label).join()==='Gauss-Jordan,Cramer,Escalonamento','rótulos dos métodos');
check(METHODS.map(m=>m.name).join()==='Gauss-Jordan,Regra de Cramer,Escalonamento de Gauss','nomes dos métodos');
{let rejected=false;try{walkthrough([[R.parse('1'),R.parse('2')]],'gauss');}catch(_){rejected=true;}check(rejected,'Modo pensar rejeita sistema fora de 2×2 e 3×3');}
{let rejected=false;try{walkthrough([[R.parse('1'),R.parse('1'),R.parse('2')],[R.parse('1'),R.parse('-1'),R.parse('0')]],'newton');}catch(_){rejected=true;}check(rejected,'Modo pensar rejeita método desconhecido');}
check(matrixLines({label:'A',rows:[[R.parse('-12'),R.parse('1/2')],[R.parse('3'),R.parse('-1')]],augmented:false,markedRows:[],markedColumn:null,pivot:null}).join('\n')==='A =\n[ −12 1/2 ]\n[   3  −1 ]','matriz em texto alinhada por caractere');
check(String(R.parse('−2,5e3'))==='-2500','sinal unicode, vírgula e expoente');
check(String(R.parse('1e100/1e100'))==='1','grandes expoentes');
check(String(R.parse('.25'))==='1/4','decimal exato');
check(String(R.parse('-3/-2'))==='3/2','sinais da fração');
for(const bad of ['', '1/0','1/2/3','1e1001','1e','NaN','Infinity','1,2.3','1'.repeat(101)]){let rejected=false;try{R.parse(bad);}catch(_){rejected=true;}check(rejected,'rejeição: '+bad);}
const one=R.parse('1'),two=R.parse('2'),tiny=R.parse('1e-50');
const near=solve([[one,one,two],[one,one.add(tiny),two.add(tiny)]]);
check(near.kind==='unique'&&near.rref[0][2].eq(one)&&near.rref[1][2].eq(one),'quase singular');
const html=fs.readFileSync(path.join(__dirname,'../SistemasLineares.html'),'utf8');
const ids=[...html.matchAll(/<[a-z][^>]*\sid="([^"]+)"/gi)].map(m=>m[1]);
check(new Set(ids).size===ids.length,'IDs únicos: ícones e controles não podem colidir');
check(!/__STYLE__|__FONT__|__SOLVER__|__APP__|__ICONS__/.test(html),'arquivo offline completo');
for(const id of ['icon-bulb','icon-arrow-left','think-open','copy','think','think-title','methods','copy-steps','steps'])check(ids.includes(id),`Modo pensar na versão offline: #${id}`);
console.log(`PASS: ${systems} sistemas idênticos entre a versão nativa e JavaScript; ${walkthroughs?`${walkthroughs} passo a passo do Modo pensar idênticos ao Rust; `:''}${checks} verificações da versão portátil`);
