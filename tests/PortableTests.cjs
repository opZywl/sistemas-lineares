const fs=require('node:fs');
const {Rational:R,solve,format}=require('../portable/solver.js');
const path=require('node:path');
let checks=0,systems=0;
function check(value,label){checks++;if(!value)throw Error(label);}
const lines=fs.readFileSync(process.argv[2],'utf8').trim().split(/\r?\n/);
for(const line of lines){
 const [size,entries,kind,encoded]=line.split('|'),n=Number(size),values=entries.split(',').map(R.parse),a=Array.from({length:n},(_,i)=>values.slice(i*(n+1),(i+1)*(n+1))),answer=solve(a);systems++;
 check(answer.kind===kind,`classificação, sistema ${systems}`);
 check(format(answer)===Buffer.from(encoded,'base64').toString('utf8'),`resposta exata, sistema ${systems}`);
}
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
console.log(`PASS: ${systems} sistemas idênticos entre a versão nativa e JavaScript; ${checks} verificações da versão portátil`);
