const fs=require('node:fs');
const path=require('node:path');
const assert=require('node:assert/strict');
const solver=require('../portable/solver.js');
const {Rational:R,solve,format,METHODS,walkthrough,walkthroughText,matrixLines,PARSE_MESSAGES,MAX_POWER_DIGITS,LIVE_BUDGET_BITS,PREVIEW_DIGIT_LIMIT,usesExtension,looksIncomplete,withinLiveBudget,Entry,abbreviate,previewEquation}=solver;
const {Model,spokenValue,reading,accessibleInputName,accessibleValueName,rowNumber,sizeName,inputColumn,TEXTS}=require('../portable/app.js');
const [fixturesFile,passosFile,corpusFile]=process.argv.slice(2);
let checks=0,systems=0,walkthroughs=0,vetor=0,corpus=0,propriedades=0,cenarios=0;
function check(value,label){checks++;if(!value)throw Error(label);}
function igual(actual,expected,label){
 checks++;
 try{assert.deepStrictEqual(actual,expected);}
 catch(_){throw Error(`${label}\n  esperado: ${JSON.stringify(expected)}\n  obtido:   ${JSON.stringify(actual)}`);}
}
const interpretar=entrada=>{try{return `OK:${R.parse(entrada)}`;}catch(error){return `ERRO:${error.message}`;}};
const valor=texto=>R.parse(texto);
function decodificar(texto){
 let saida='';
 const caracteres=Array.from(texto);
 for(let indice=0;indice<caracteres.length;indice++){
  const caractere=caracteres[indice];
  if(caractere!=='\\'){saida+=caractere;continue;}
  const proximo=caracteres[++indice];
  if(proximo==='\\'){saida+='\\';continue;}
  if(proximo!=='u'||caracteres[++indice]!=='{')throw Error(`escape inválido em ${JSON.stringify(texto)}`);
  let hexadecimal='';
  while(++indice<caracteres.length&&caracteres[indice]!=='}')hexadecimal+=caracteres[indice];
  saida+=String.fromCodePoint(parseInt(hexadecimal,16));
 }
 return saida;
}
class RandomDotNet{
 constructor(semente){
  const MBIG=2147483647,tabela=new Int32Array(56);
  let mj=(161803398-(semente===-2147483648?MBIG:Math.abs(semente)))|0,mk=1;
  tabela[55]=mj;
  for(let i=1;i<55;i++){const ii=21*i%55;tabela[ii]=mk;mk=(mj-mk)|0;if(mk<0)mk=(mk+MBIG)|0;mj=tabela[ii];}
  for(let rodada=1;rodada<5;rodada++)for(let i=1;i<56;i++){tabela[i]=(tabela[i]-tabela[1+(i+30)%55])|0;if(tabela[i]<0)tabela[i]=(tabela[i]+MBIG)|0;}
  Object.assign(this,{tabela,inext:0,inextp:21});
 }
 amostra(){
  let inext=this.inext+1,inextp=this.inextp+1;
  if(inext>=56)inext=1;
  if(inextp>=56)inextp=1;
  let resultado=(this.tabela[inext]-this.tabela[inextp])|0;
  if(resultado===2147483647)resultado--;
  if(resultado<0)resultado=(resultado+2147483647)|0;
  Object.assign(this,{inext,inextp});
  this.tabela[inext]=resultado;
  return resultado*(1/2147483647);
 }
 next(minimo,maximo){return (Math.trunc(this.amostra()*(maximo-minimo))+minimo)|0;}
}

if(fixturesFile){
 const lines=fs.readFileSync(fixturesFile,'utf8').trim().split(/\r?\n/);
 for(const line of lines){
  const [size,entries,kind,encoded]=line.split('|'),n=Number(size),values=entries.split(',').map(R.parse),a=Array.from({length:n},(_,i)=>values.slice(i*(n+1),(i+1)*(n+1))),answer=solve(a);systems++;
  check(answer.kind===kind,`classificação, sistema ${systems}`);
  check(format(answer)===Buffer.from(encoded,'base64').toString('utf8'),`resposta exata, sistema ${systems}`);
 }
}
if(passosFile){
 const passos=fs.readFileSync(passosFile,'utf8').trim().split(/\r?\n/);
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

{
 const divergencias=[];
 for(const linha of fs.readFileSync(path.join(__dirname,'potencias.txt'),'utf8').split(/\r?\n/)){
  if(!linha)continue;
  const tabulacao=linha.indexOf('\t');
  check(tabulacao>=0,`linha sem tabulação em tests/potencias.txt: ${JSON.stringify(linha)}`);
  const entrada=decodificar(linha.slice(0,tabulacao)),esperado=linha.slice(tabulacao+1),obtido=interpretar(entrada);
  vetor++;
  if(obtido!==esperado)divergencias.push(`${JSON.stringify(entrada)}\n  esperado ${esperado.slice(0,120)}\n  obtido   ${obtido.slice(0,120)}`);
 }
 check(vetor>=150,`vetor de potências com poucos casos: ${vetor}`);
 if(divergencias.length)console.error(divergencias.join('\n'));
 check(divergencias.length===0,`${divergencias.length} divergências no vetor compartilhado tests/potencias.txt`);
}
if(corpusFile){
 const divergencias=[];
 for(const linha of fs.readFileSync(corpusFile,'utf8').split(/\r?\n/)){
  if(!linha)continue;
  const [codificada,status,esperado]=linha.split('\t'),entrada=Buffer.from(codificada,'base64').toString('utf8');
  corpus++;
  let obtido,estado='OK';
  try{obtido=String(R.parse(entrada));}catch(error){estado='ERRO';obtido=error.message;}
  if(estado!==status||obtido!==esperado)divergencias.push(`${JSON.stringify(entrada)}\n  Rust:       ${status} ${String(esperado).slice(0,120)}\n  JavaScript: ${estado} ${obtido.slice(0,120)}`);
 }
 if(divergencias.length)console.error(divergencias.slice(0,20).join('\n'));
 check(divergencias.length===0,`${divergencias.length} divergências entre Rust e JavaScript no corpus de potências`);
 check(corpus===200000,`o corpus de potências deveria ter 200000 entradas, tem ${corpus}`);
}

{
 igual(PARSE_MESSAGES.PowerExponent,'Use um expoente inteiro de -1000 a 1000, como 2^3 ou 10^-2.','mensagem PowerExponent');
 igual(PARSE_MESSAGES.PowerChain,'Use parênteses para potência de potência, como (2^3)^2.','mensagem PowerChain');
 igual(PARSE_MESSAGES.PowerTooLarge,'Use números menores: com potências, o limite é de 300 algarismos.','mensagem PowerTooLarge');
 igual(PARSE_MESSAGES.ZeroToZero,'0 elevado a 0 é indeterminado.','mensagem ZeroToZero');
 igual(PARSE_MESSAGES.ZeroNegativePower,'0 não pode ser elevado a expoente negativo.','mensagem ZeroNegativePower');
 igual(PARSE_MESSAGES.Parenthesis,'Confira os parênteses.','mensagem Parenthesis');
 igual(PARSE_MESSAGES.Product,'Faça a multiplicação antes: para 1,5×10³, digite 1,5e3 ou 1500.','mensagem Product');
 igual(MAX_POWER_DIGITS,300,'limite de algarismos');
 for(const texto of ['1','-2,5','3/4','1e-6','1 / 2','x','1+2','--2','1/2/3','\u00851'])check(!usesExtension(texto),`${JSON.stringify(texto)} não deveria usar a extensão`);
 for(const texto of ['2^3','2²','(1)','2**3','2*3','10⁻¹','2\u00D73','2\u00B73','2\u22C53'])check(usesExtension(texto),`${JSON.stringify(texto)} deveria usar a extensão`);
 const sobrescrito=expoente=>(expoente<0?'⁻':'')+Array.from(String(Math.abs(expoente)),digito=>['⁰','¹','²','³','⁴','⁵','⁶','⁷','⁸','⁹'][Number(digito)]).join('');
 const random=new RandomDotNet(185964);
 for(let indice=0;indice<4000;indice++){
  const numerador=random.next(-12,13),denominador=random.next(1,9);
  if(numerador===0)continue;
  const base=`(${numerador}/${denominador})`,m=random.next(-7,8),n=random.next(-7,8);
  check(valor(`${base}^${m}`).mul(valor(`${base}^${n}`)).eq(valor(`${base}^${m+n}`)),`${base} m=${m} n=${n}: produto de potências`);
  check(valor(`(${base}^${m})^${n}`).eq(valor(`${base}^${m*n}`)),`${base} m=${m} n=${n}: potência de potência`);
  check(valor(`${base}${sobrescrito(m)}`).eq(valor(`${base}^${m}`)),`${base}: sobrescrito ${m}`);
  check(valor(`${base}**(${m})`).eq(valor(`${base}^${m}`)),`${base}: ** ${m}`);
  check(valor(`-${base}^2`).eq(valor(`${base}^2`).neg()),`${base}: sinal depois da potência`);
  let acumulado=R.parse('1');
  for(let vez=0;vez<Math.abs(m);vez++)acumulado=acumulado.mul(valor(base));
  check(valor(`${base}^${m}`).eq(m<0?R.parse('1').div(acumulado):acumulado),`${base}^${m} pela definição`);
  propriedades+=5;
 }
 for(const [base,expoente,cabe] of [[10,299,true],[10,300,false],[2,996,true],[2,997,false],[3,628,true],[3,629,false],[7,354,true],[7,355,false],[99,150,true],[99,151,false],[101,149,true],[2,1000,false]]){
  let aceito=true;
  try{R.parse(`${base}^${expoente}`);}catch(_){aceito=false;}
  const algarismos=String(BigInt(base)**BigInt(expoente)).length;
  check(aceito===(algarismos<=MAX_POWER_DIGITS),`${base}^${expoente} tem ${algarismos} algarismos`);
  check(aceito===cabe,`${base}^${expoente}: limite de algarismos`);
 }
 for(const [entrada,cabe] of [['(1e1000/1e-1000)',true],['(1e1000/1e-1000)^0',true],['(1e1000)^1',false],['(10^200)/(10^-99)',true],['(10^200)/(10^-100)',false],['((1e1000))',true]]){
  let aceito=true;
  try{R.parse(entrada);}catch(_){aceito=false;}
  check(aceito===cabe,`${entrada}: limite de algarismos com potências`);
 }
 {
  const inicio=Date.now(),valores=['(101/137)^99','(173/211)^99','(251/283)^99','(337/379)^99','(421/461)^99','(503/563)^99','(601/607)^99','(701/709)^99','(809/811)^99','(907/911)^99','(953/967)^99','(977/983)^99'].map(valor);
  const matriz=[valores.slice(0,4),valores.slice(4,8),valores.slice(8,12)];
  solve(matriz);
  for(const metodo of METHODS)check(walkthrough(matriz,metodo.key).steps.length>0,`passo a passo denso por ${metodo.key}`);
  const decorrido=(Date.now()-inicio)/1000;
  check(decorrido<5,`sistema 3x3 com potências densas levou ${decorrido} s`);
  const leitura=Date.now();
  for(let vez=0;vez<100;vez++)try{R.parse('(((101/137)^99/(173/211)^99)/((251/283)^99/(337/379)^99))/((421/461)^99/(503/563)^99)');}catch(_){}
  check((Date.now()-leitura)/1000<5,`100 leituras aninhadas levaram ${(Date.now()-leitura)/1000} s`);
 }
 const v=texto=>Entry.value(valor(texto));
 for(const [linha,esperado] of [
  [[v('2'),Entry.Missing,v('5')],'2x + ?y = 5'],
  [[Entry.Missing,v('-1'),v('1')],'?x − y = 1'],
  [[v('0'),Entry.Missing,Entry.Missing],'?y = ?'],
  [[Entry.Missing,Entry.Missing,Entry.Missing],'?x + ?y = ?'],
  [[v('3/4'),v('-2'),v('0')],'(3/4)x − 2y = 0'],
  [[v('0'),v('0'),v('-7')],'0 = −7'],
  [[Entry.Invalid,v('1'),Entry.Invalid],'?x + y = ?'],
  [[v('-1'),Entry.Missing,v('1'),v('2^3')],'−x + ?y + z = 8'],
  [[v('1'),v('-1/2'),Entry.Missing,v('-1,5')],'x − (1/2)y + ?z = −3/2'],
  [[Entry.Missing,Entry.Missing,Entry.Missing,Entry.Missing],'?x + ?y + ?z = ?'],
  [[v('0'),v('0'),v('0'),Entry.Missing],'0 = ?'],
  [[v('1e-30'),v('10^20'),v('-1e40')],'(1/1000…0000)x + 1000…0000y = −1000…0000'],
  [[v('12345677/99999999'),v('2'),v('-1234567/7654321')],'(12345677/99999999)x + 2y = −1234567/7654321']
 ])igual(previewEquation(linha),esperado,'prévia do sistema');
 igual(PREVIEW_DIGIT_LIMIT,12,'limite de algarismos da prévia');
 igual(abbreviate('123456789012'),'123456789012','abreviação no limite');
 igual(abbreviate('1234567890123'),'1234…0123','abreviação acima do limite');
 igual(abbreviate('12345678/99999999'),'12345678/99999999','fração curta sem abreviação');
 igual(abbreviate('−1234567890123456/7654321098765432'),'−1234…3456/7654…5432','cada bloco de algarismos abreviado');
 for(const texto of ['-','+','1/','2^','2^-','(','(1','((2)','1e','1E-','2**','2*','.',',','-.','2⁻','1/(2',' 3/ ','−'])check(looksIncomplete(texto),`${JSON.stringify(texto)} parece em digitação`);
 for(const texto of ['',' ','2','1/2','2^3','x','2x','1/0','0^0','(2)','1e5','abc'])check(!looksIncomplete(texto),`${JSON.stringify(texto)} não está em digitação`);
 check(withinLiveBudget([[valor('2'),valor('1'),valor('5')],[valor('2'),valor('1'),valor('5')]]),'sistema pequeno cabe no orçamento');
 check(!withinLiveBudget(Array.from({length:3},()=>[valor('1e1000'),valor('1'),valor('5')])),'sistema grande passa do orçamento');
 igual(LIVE_BUDGET_BITS,8000,'orçamento ao vivo');
}

const campo=(row,column)=>({row,column,selectAll:true});
const preencher=(model,linhas)=>{const dimension=model.dimension();linhas.forEach((valores,row)=>valores.forEach((texto,column)=>model.setInput(row,inputColumn(dimension,column),texto)));};
const resultado=stale=>({kind:'result',stale});
const faltando=(missing,invalid)=>({kind:'waiting',waiting:{kind:'missing',missing,invalid}});
const cenario=(nome,teste)=>{cenarios++;try{teste();}catch(error){error.message=`${nome}: ${error.message}`;throw error;}};
cenario('estado_inicial',()=>{
 const model=new Model();
 igual(model.dimension(),2,'dimensão');
 igual(model.dimensionText(),'2 equações · 2 incógnitas','texto da dimensão');
 igual(model.helper(),'Aceita 2 · -1,5 · 3/4 · 2^3 · 10². Use 0 se a variável não aparecer.','dica inicial da versão HTML');
 check(!model.helperIsError(),'dica inicial sem erro');
 check(model.isEmpty(),'sem resultado');
 igual(model.card(),{kind:'waiting',waiting:{kind:'empty'}},'card inicial');
 igual(model.waitingTitle(),'Aguardando os valores','título inicial');
 check(!model.showsPreview(),'sem prévia');
 igual(model.missing(),6,'campos faltando');
 check(!model.copyEnabled()&&!model.thinkEnabled()&&!model.isThinking(),'ações desabilitadas');
 igual(model.copyLabel(),'Copiar','rótulo Copiar');
 igual(model.note(),'Resultados exatos, incluindo frações.','nota');
 igual(model.title(),null,'título');
 igual(model.announcement(),null,'anúncio');
});
cenario('validacao_inline_aponta_o_campo',()=>{
 const model=new Model();
 igual(model.solve(),campo(0,0),'foco no primeiro vazio');
 check(model.helper().includes('Equação 1'),'dica cita a equação');
 igual(model.helper(),'Equação 1, coeficiente de x: Preencha todos os campos.','dica de campo vazio');
 check(model.helperIsError()&&model.fieldHasError(0,0)&&!model.copyEnabled(),'erro revelado');
 preencher(model,[['1','2','3'],['4','5','x']]);
 igual(model.solve(),campo(1,3),'foco no inválido');
 igual(model.helper(),'Equação 2, termo independente: Número inválido.','dica de inválido');
 igual(model.errorField(),[1,3],'campo com erro');
 model.setInput(0,0,'1/0');
 igual(model.helper(),'Equação 2, termo independente: Número inválido.','editar outro campo não apaga um erro já revelado');
 igual(model.solve(),campo(0,0),'foco no denominador zero');
 igual(model.helper(),'Equação 1, coeficiente de x: O denominador não pode ser zero.','dica de denominador zero');
 model.setInput(0,0,'9'.repeat(101));
 model.solve();
 igual(model.helper(),'Equação 1, coeficiente de x: Use números com no máximo 100 caracteres.','dica de texto longo');
 model.setInput(0,0,'1e5000');
 model.solve();
 igual(model.helper(),'Equação 1, coeficiente de x: Use um expoente inteiro entre -1000 e 1000.','dica de expoente');
});
cenario('exemplo_2x2_resolve_e_habilita_copia',()=>{
 const model=new Model();
 igual(model.fillExample(),null,'foco não muda');
 igual(model.helper(),'Exemplo resolvido. Mude qualquer valor e a resposta se atualiza na hora.','dica do exemplo');
 igual(model.notice(),{kind:'example'},'aviso do exemplo');
 igual([model.input(0,0),model.input(0,1),model.input(0,3),model.input(1,0),model.input(1,1),model.input(1,3)],['2','1','5','1','-1','1'],'campos do exemplo');
 check(!model.isEmpty(),'o exemplo já sai resolvido');
 igual(model.card(),resultado(false),'card de resultado');
 igual(model.solve(),null,'resolver de novo');
 const texto=model.lastResult()??'';
 check(texto.includes('x = 2')&&texto.includes('y = 1'),texto);
 igual(model.values(),['2','1'],'valores');
 check(model.copyEnabled(),'Copiar habilitado');
 igual(model.title(),'Solução encontrada','título');
 igual(model.description(),'Estes valores satisfazem todas as equações do sistema.','descrição');
 igual(model.note(),'Solução única · valores exatos','nota');
 igual(model.general(),null,'sem forma geral');
 igual(model.announcement(),'Solução encontrada: x = 2, y = 1.','anúncio');
});
cenario('editar_atualiza_o_resultado_na_hora',()=>{
 const model=new Model();
 model.fillExample();
 check(model.copyEnabled(),'Copiar habilitado');
 model.setInput(0,0,'4');
 igual(model.values(),['6/5','1/5'],'novo resultado');
 check(model.copyEnabled(),'Copiar continua habilitado');
 igual(model.card(),resultado(false),'card atualizado');
 igual(model.helper(),TEXTS.DEFAULT_HELPER,'dica padrão');
});
cenario('exemplo_3x3_resolve',()=>{
 const model=new Model();
 model.changeDimension(3);
 igual(model.dimensionText(),'3 equações · 3 incógnitas','texto 3×3');
 model.fillExample();
 model.solve();
 const texto=model.lastResult()??'';
 check(texto.includes('x = 1')&&texto.includes('y = 2')&&texto.includes('z = 3'),texto);
 igual(model.values(),['1','2','3'],'valores 3×3');
});
cenario('zeros_dao_infinitas_solucoes',()=>{
 const model=new Model();
 model.changeDimension(3);
 for(let row=0;row<3;row++)for(let column=0;column<4;column++)model.setInput(row,column,'0');
 model.solve();
 igual(model.title(),'Infinitas soluções','título');
 igual(model.description(),'As equações são dependentes. Veja a forma geral da resposta:','descrição');
 igual(model.general(),'x = t1\ny = t2\nz = t3\n\nt1, t2, ... podem ser quaisquer números reais.','forma geral');
 igual(model.note(),'Os parâmetros t1, t2, … são números reais livres.','nota');
 igual(model.values(),[],'sem valores');
 igual(model.announcement(),'Infinitas soluções: x = t1, y = t2, z = t3.','anúncio');
 model.setInput(0,3,'1');
 model.solve();
 igual(model.title(),'Sem solução','título sem solução');
 igual(model.description(),'As equações são incompatíveis.','descrição sem solução');
 igual(model.general(),'Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo.','texto sem solução');
 igual(model.note(),'Confira os coeficientes se esse não era o esperado.','nota sem solução');
 igual(model.announcement(),'Sem solução. As equações são incompatíveis.','anúncio sem solução');
});
cenario('infinitas_2x2_mostra_a_forma_geral',()=>{
 const model=new Model();
 preencher(model,[['1','1','2'],['2','2','4']]);
 model.solve();
 igual(model.outcome(),{kind:'infinite',general:'x = 2 - t1\ny = t1\n\nt1, t2, ... podem ser quaisquer números reais.'},'forma geral');
 check((model.lastResult()??'').startsWith('Infinitas soluções\n\nx = 2 - t1'),'texto copiado');
});
cenario('valores_longos_sao_exatos',()=>{
 const model=new Model();
 preencher(model,[['3','0','1'],['0','7','1e-30']]);
 model.solve();
 igual(model.values(),['1/3','1/7000000000000000000000000000000'],'valores exatos');
});
cenario('limpar_reinicia_tudo',()=>{
 const model=new Model();
 model.changeDimension(3);
 model.fillExample();
 model.solve();
 igual(model.clear(),{row:0,column:0,selectAll:false},'foco no primeiro campo');
 for(let row=0;row<3;row++)for(let column=0;column<4;column++)igual(model.input(row,column),'',`campo ${row},${column} vazio`);
 check(!model.copyEnabled()&&model.isEmpty(),'sem resultado');
 igual(model.helper(),TEXTS.DEFAULT_HELPER,'dica padrão');
 igual(model.dimension(),3,'dimensão mantida');
});
cenario('cada_tamanho_guarda_o_proprio_sistema',()=>{
 const model=new Model();
 model.changeDimension(3);
 model.fillExample();
 igual(model.values(),['1','2','3'],'exemplo 3×3');
 model.changeDimension(2);
 for(let row=0;row<3;row++)for(let column=0;column<4;column++)igual(model.input(row,column),'',`o 2×2 não herda o 3×3 (${row},${column})`);
 igual(model.card(),{kind:'waiting',waiting:{kind:'empty'}},'nenhuma resposta para um sistema que ninguém digitou');
 for(const [row,column,texto] of [[0,0,'1'],[0,1,'1'],[0,3,'2'],[1,0,'1'],[1,1,'-1'],[1,3,'0']])model.edit(row,column,texto);
 igual(model.values(),['1','1'],'2×2 resolvido');
 model.changeDimension(3);
 igual(model.input(2,3),'2','o 3×3 volta como estava');
 igual(model.values(),['1','2','3'],'e já resolvido');
 model.changeDimension(2);
 igual(model.values(),['1','1'],'o 2×2 também volta resolvido');
 model.clear();
 model.changeDimension(3);
 igual(model.input(0,0),'','Limpar apaga os dois tamanhos');
 igual(model.card(),{kind:'waiting',waiting:{kind:'empty'}},'3×3 vazio depois de Limpar');
 model.changeDimension(7);
 igual(model.dimension(),3,'limite superior');
 model.changeDimension(0);
 igual(model.dimension(),2,'limite inferior');
});
cenario('erro_revelado_nao_pisca_enquanto_corrige',()=>{
 let model=new Model();
 model.edit(0,0,'1/0');
 model.leave(0,0);
 check(model.fieldHasError(0,0),'erro revelado ao sair');
 model.setFocus([0,0]);
 model.edit(0,0,'1/');
 check(model.fieldHasError(0,0),'continua inválido: o vermelho fica');
 igual(model.helper(),'Equação 1, coeficiente de x: Número inválido.','dica do prefixo revelado');
 model.edit(0,0,'1/2');
 check(!model.fieldHasError(0,0),'ficou válido: o erro some na hora');
 model.edit(0,0,'1/0');
 check(!model.fieldHasError(0,0),'um erro novo espera a pausa, a saída ou o Enter');
 model.edit(0,0,'');
 check(!model.fieldHasError(0,0),'campo vazio sem erro');
 model=new Model();
 model.setFocus([0,0]);
 igual(model.solve(),campo(0,0),'Resolver aponta o campo vazio');
 check(model.fieldHasError(0,0),'campo vazio apontado');
 model.edit(0,0,'2^');
 check(!model.fieldHasError(0,0),'começar a digitar no campo vazio apontado não acende outro erro');
 igual(model.helper(),TEXTS.DEFAULT_HELPER,'dica padrão');
});
cenario('dica_mostra_o_valor_lido_do_campo_com_potencia',()=>{
 const model=new Model();
 model.setFocus([0,0]);
 model.edit(0,0,'-2²');
 igual(model.helper(),'Este campo vale -4.','leitura de -2²');
 check(model.helperIsReading()&&!model.helperIsError(),'dica de leitura');
 model.edit(0,0,'2^10');
 igual(model.helper(),'Este campo vale 1024.','leitura de 2^10');
 model.edit(0,0,'2^100');
 igual(model.helper(),'Este campo vale 1267…5376.','leitura abreviada');
 model.edit(0,0,'(1/2)^3');
 igual(model.helper(),'Este campo vale 1/8.','leitura de fração');
 model.edit(0,0,'2^');
 igual(model.helper(),TEXTS.DEFAULT_HELPER,'prefixo ainda não tem valor');
 check(!model.helperIsReading(),'sem leitura no prefixo');
 model.edit(0,0,'3');
 igual(model.helper(),TEXTS.DEFAULT_HELPER,'sem potência, a dica volta ao normal');
 model.edit(0,0,'2^3');
 model.setFocus([0,1]);
 igual(model.helper(),TEXTS.DEFAULT_HELPER,'a dica segue o campo focado');
 igual(reading(R.parse('3/4')),'Este campo vale 3/4.','texto da leitura');
});
cenario('copiar_resultado',()=>{
 const model=new Model();
 let visto=null;
 model.copy(texto=>{visto=texto;return true;});
 igual(visto,null,'nada para copiar');
 model.fillExample();
 model.solve();
 model.copy(texto=>{visto=texto;return true;});
 igual(visto,'Solução única\n\nx = 2\ny = 1','texto copiado');
 igual(model.copyLabel(),'Copiado','rótulo depois de copiar');
 model.solve();
 igual(model.copyLabel(),'Copiar','rótulo depois de resolver');
 model.copy(()=>false);
 igual(model.note(),'Não foi possível copiar agora. Tente novamente.','nota de falha');
 igual(model.copyLabel(),'Copiar','rótulo depois da falha');
});
cenario('autor_indisponivel',()=>{
 const model=new Model();
 model.authorUnavailable();
 igual(model.helper(),'Abra lucas-lima.xyz no seu navegador para visitar o autor.','dica do autor');
});
cenario('nomes_acessiveis',()=>{
 igual(accessibleInputName(0,0),'Equação 1, coeficiente de x','nome do campo x');
 igual(accessibleInputName(2,2),'Equação 3, coeficiente de z','nome do campo z');
 igual(accessibleInputName(1,3),'Equação 2, termo independente','nome do termo');
 igual(accessibleValueName(1),'Valor de y','nome do valor');
 igual(rowNumber(0),'01','número da linha 1');
 igual(rowNumber(2),'03','número da linha 3');
});
cenario('textos_fixos',()=>{
 igual(TEXTS.KEYS,[['Enter','próximo campo vazio'],['↑ ↓','mudar de equação']],'teclas');
 igual(TEXTS.THINK_KEYS,[['Esc','voltar']],'teclas do Modo pensar');
 igual(TEXTS.TWO_LABEL,'2 × 2','rótulo 2×2');
 igual(TEXTS.THREE_LABEL,'3 × 3','rótulo 3×3');
 check(TEXTS.TWO_LABEL.includes('×')&&TEXTS.NOTE_INFINITE.includes('…'),'sinal de vezes e reticências');
});
cenario('modo_pensar_no_modelo',()=>{
 const model=new Model();
 check(!model.thinkEnabled()&&!model.openThinking(),'sem resultado não há passo a passo');
 igual(model.method(),'gauss-jordan','método padrão');
 model.fillExample();
 model.solve();
 check(model.thinkEnabled()&&!model.isThinking(),'Modo pensar disponível');
 check(model.openThinking()&&model.isThinking(),'Modo pensar aberto');
 igual(model.thinkingStatus(),'Gauss-Jordan · 7 passos','status Gauss-Jordan');
 igual(model.walkthrough().answer,['x = 2','y = 1'],'resposta do passo a passo');
 model.setMethod('cramer');
 igual(model.method(),'cramer','método Cramer');
 igual(model.thinkingStatus(),'Regra de Cramer · 6 passos','status Cramer');
 let copiado=null;
 model.copySteps(texto=>{copiado=texto;return true;});
 check(copiado.startsWith('Modo pensar · Regra de Cramer\n\nSistema\n2x + y = 5\n'),'início do texto copiado');
 check(!copiado.includes('\r'),'quebras de linha da web');
 check(copiado.endsWith('Sistema possível e determinado (SPD): solução única.'),'fim do texto copiado');
 igual(model.stepsCopyLabel(),'Passos copiados','rótulo depois de copiar');
 model.setMethod('gauss');
 igual(model.stepsCopyLabel(),'Copiar passos','rótulo depois de trocar o método');
 model.copySteps(()=>false);
 igual(model.stepsCopyLabel(),'Copiar passos','rótulo depois da falha');
 check(model.stepsCopyFailed()&&model.thinkingStatus().includes('Não foi possível copiar agora'),'falha ao copiar');
 check(model.closeThinking()&&!model.isThinking()&&!model.closeThinking(),'fechar o Modo pensar');
 igual(model.method(),'gauss','método lembrado ao voltar');
 check(model.openThinking(),'reabrir');
 igual(model.walkthrough().method.key,'gauss','método do passo a passo');
 model.setInput(0,0,'4');
 check(!model.isThinking(),'editar fecha o modo pensar');
 check(model.thinkEnabled(),'o novo sistema já tem passo a passo');
 model.fillExample();
 model.solve();
 model.openThinking();
 model.changeDimension(3);
 check(!model.isThinking(),'trocar a dimensão fecha o modo pensar');
 model.fillExample();
 model.solve();
 model.openThinking();
 model.clear();
 check(!model.isThinking()&&!model.thinkEnabled(),'limpar fecha o modo pensar');
});
cenario('modo_pensar_nos_tres_tipos_de_resposta',()=>{
 for(const [linhas,primeira] of [[[['2','1','5'],['1','-1','1']],'x = 2'],[[['1','1','2'],['2','2','4']],'x = 2 − t1'],[[['1','1','2'],['1','1','3']],'Sem solução']]){
  const model=new Model();
  preencher(model,linhas);
  igual(model.solve(),null,'resolver');
  for(const metodo of METHODS){
   model.setMethod(metodo.key);
   check(model.openThinking(),`abrir ${metodo.key}`);
   const passos=model.walkthrough();
   igual(passos.method.key,metodo.key,'método');
   igual(passos.answer[0],primeira,`resposta por ${metodo.key}`);
   check(passos.steps.length<=24,'cabe no conjunto de passos');
   model.closeThinking();
  }
 }
});
cenario('resolve_ao_vivo_ao_completar',()=>{
 const model=new Model();
 [[0,0,'2'],[0,1,'1'],[0,3,'5'],[1,0,'1'],[1,1,'-1']].forEach(([row,column,texto],indice)=>{
  model.setFocus([row,column]);
  model.edit(row,column,texto);
  igual(model.card(),faltando(5-indice,0),`card depois de ${texto}`);
  check(model.showsPreview()&&!model.copyEnabled()&&!model.thinkEnabled(),'prévia sem ações');
 });
 igual(model.waitingTitle(),'Falta 1 campo','falta 1');
 igual(model.preview(),['2x + y = 5','x − y = ?'],'prévia');
 igual(model.previewSpoken(),'Faltam: equação 2, termo independente.','prévia falada');
 check(model.rowComplete(0)&&!model.rowComplete(1),'linha completa');
 model.setFocus([1,3]);
 model.edit(1,3,'1');
 igual(model.card(),resultado(false),'resultado na hora');
 igual(model.values(),['2','1'],'valores');
 check(model.copyEnabled()&&model.thinkEnabled()&&model.rowComplete(1),'ações habilitadas');
 igual(model.waitingTitle(),null,'sem título de espera');
});
cenario('digitar_um_prefixo_nao_pisca_o_card',()=>{
 const model=new Model();
 model.fillExample();
 model.setFocus([0,3]);
 for(const parcial of ['-','1/','2^','(','1e','2^-']){
  model.edit(0,3,parcial);
  igual(model.card(),resultado(true),`${parcial} mantém o último resultado esmaecido`);
  check(!model.helperIsError(),`${parcial} não mostra erro`);
  check(!model.copyEnabled()&&!model.thinkEnabled(),`${parcial} desabilita as ações`);
  igual(model.lastResult(),null,'sem texto para copiar');
  igual(model.announcement(),null,'sem anúncio');
  igual(model.note(),TEXTS.NOTE_STALE,'nota de atualização');
 }
 check(model.idle(),'a pausa assenta o card');
 igual(model.card(),faltando(1,0),'prefixo com foco conta como campo ainda em digitação');
 check(!model.helperIsError(),'sem erro');
 model.edit(0,3,'-3');
 igual(model.card(),resultado(false),'resultado');
 igual(model.values(),['-2/3','-5/3'],'valores');
});
cenario('apagar_um_valor_espera_antes_de_trocar_o_card',()=>{
 const model=new Model();
 model.fillExample();
 model.setFocus([0,0]);
 model.edit(0,0,'');
 igual(model.card(),resultado(true),'esmaecido');
 model.edit(0,0,'2');
 igual(model.card(),resultado(false),'de volta');
 model.edit(0,0,'');
 check(model.idle(),'pausa');
 igual(model.card(),faltando(1,0),'espera');
 igual(model.preview(),['?x + y = 5','x − y = 1'],'prévia');
 model.edit(0,0,'2');
 model.edit(0,0,'');
 check(model.leave(0,0),'sair do campo assenta o card na hora');
 igual(model.card(),faltando(1,0),'espera depois de sair');
});
cenario('erro_revelado_por_campo',()=>{
 const model=new Model();
 model.setFocus([0,0]);
 model.edit(0,0,'2x');
 check(!model.helperIsError(),'o erro espera a pausa ou a saída do campo');
 igual(model.card(),faltando(5,1),'card com inválido');
 igual(model.waitingTitle(),'Faltam 5 campos · 1 inválido','título');
 check(model.leave(0,0)&&model.fieldHasError(0,0),'sair revela');
 igual(model.helper(),'Equação 1, coeficiente de x: Número inválido.','dica');
 model.setFocus([0,1]);
 model.edit(0,1,'5');
 check(model.fieldHasError(0,0)&&model.helperIsError(),'digitar em outro campo não apaga o erro');
 model.setFocus([0,0]);
 model.edit(0,0,'2');
 check(!model.fieldHasError(0,0)&&!model.helperIsError(),'corrigir apaga o erro');
 model.edit(0,0,'1/0');
 check(model.idle(),'sem ser prefixo, a pausa revela o erro mesmo com foco');
 igual(model.helper(),'Equação 1, coeficiente de x: O denominador não pode ser zero.','dica do denominador');
 model.setFocus([1,1]);
 model.edit(1,1,'1/');
 check(!model.idle(),'prefixo com foco não é revelado na pausa');
 check(!model.fieldHasError(1,1),'prefixo sem erro');
 check(model.leave(1,1)&&model.fieldHasError(1,1),'sair revela o prefixo');
 igual(model.helper(),'Equação 1, coeficiente de x: O denominador não pode ser zero.','sem foco num campo com erro, vale o primeiro na ordem de leitura');
 model.setFocus([1,1]);
 igual(model.helper(),'Equação 2, coeficiente de y: Número inválido.','com foco num campo com erro, vale o dele');
});
cenario('enter_vai_ao_proximo_vazio_e_resolve_quando_completo',()=>{
 const model=new Model();
 igual(model.enter(0,0),campo(0,1),'Enter no vazio');
 model.edit(0,0,'2');
 igual(model.enter(0,0),campo(0,1),'Enter no preenchido');
 model.edit(0,1,'1');
 model.edit(0,3,'5');
 model.edit(1,1,'-1');
 igual(model.enter(1,1),campo(1,3),'próximo vazio');
 igual(model.enter(1,3),campo(1,0),'dá a volta pela ordem de leitura');
 model.edit(1,0,'1');
 igual(model.enter(1,3),campo(1,3),'só falta este campo');
 igual(model.notice(),{kind:'onlyThisField'},'aviso');
 igual(model.helper(),'Falta só este campo.','dica');
 check(!model.helperIsError(),'aviso neutro');
 model.edit(1,3,'x');
 igual(model.enter(1,3),campo(1,3),'Enter num campo inválido revela o erro');
 check(model.fieldHasError(1,3),'erro revelado');
 model.edit(1,3,'1');
 igual(model.enter(1,3),null,'resolve');
 igual(model.values(),['2','1'],'valores');
});
cenario('setas_mudam_de_equacao_na_mesma_coluna',()=>{
 const model=new Model();
 igual(model.neighbor(0,1,true),campo(1,1),'para baixo');
 igual(model.neighbor(1,1,true),null,'borda de baixo');
 igual(model.neighbor(0,3,false),null,'borda de cima');
 igual(model.neighbor(1,3,false),campo(0,3),'para cima');
 igual(model.neighbor(0,2,true),null,'coluna z não existe no 2×2');
 model.changeDimension(3);
 igual(model.neighbor(1,2,true),campo(2,2),'coluna z no 3×3');
});
cenario('links_de_metodo',()=>{
 const model=new Model();
 model.edit(0,0,'2');
 igual(model.requestMethod('cramer'),{kind:'needsSystem',focus:campo(0,1)},'sistema incompleto');
 igual(model.helper(),'Preencha todos os campos para ver o passo a passo pela regra de Cramer.','dica neutra');
 check(!model.helperIsError(),'dica não é erro');
 igual(model.method(),'cramer','método lembrado');
 model.fillExample();
 model.edit(0,0,'abc');
 igual(model.requestMethod('gauss'),{kind:'needsSystem',focus:campo(0,0)},'campo inválido');
 igual(model.helper(),'Equação 1, coeficiente de x: Número inválido.','erro revelado');
 model.edit(0,0,'2');
 igual(model.requestMethod('gauss'),{kind:'opened'},'abre o Modo pensar');
 check(model.isThinking(),'Modo pensar aberto');
 igual(model.walkthrough().method.key,'gauss','método escolhido');
});
cenario('numeros_grandes_esperam_o_enter',()=>{
 const model=new Model();
 for(const [row,column,texto] of [[0,0,'1e1000'],[0,1,'3e999'],[0,3,'5'],[1,0,'1'],[1,1,'-2e1000'],[1,3,'7e998']])model.edit(row,column,texto);
 igual(model.card(),{kind:'waiting',waiting:{kind:'large'}},'espera o Enter');
 igual(model.waitingTitle(),'Números grandes','título');
 check(model.showsPreview()&&model.preview()[0].startsWith('1000…0000x'),JSON.stringify(model.preview()));
 igual(model.solve(),null,'resolver calcula');
 igual(model.outcome().kind,'unique','solução única');
 check((model.announcement()??'').includes('algarismos'),model.announcement());
});
cenario('anuncio_abrevia_valores_longos',()=>{
 igual(spokenValue('2'),'2','curto');
 igual(spokenValue('-3/4'),'-3/4','fração curta');
 igual(spokenValue('7'.repeat(30)),'número com 30 algarismos','número longo');
 igual(spokenValue(`1/${'3'.repeat(40)}`),'fração com 1 algarismos sobre 40 algarismos','fração longa');
});
cenario('exemplos_da_dica_sao_aceitos',()=>{
 for(const [texto,esperado] of [['2','2'],['-1,5','-3/2'],['3/4','3/4'],['2^3','8'],['10²','100']])igual(String(R.parse(texto)),esperado,texto);
 check(TEXTS.DEFAULT_HELPER.includes('2^3')&&TEXTS.DEFAULT_HELPER.includes('10²'),'exemplos na dica');
});
cenario('nomes_do_tamanho_e_dos_links',()=>{
 igual(sizeName(2,true),'Sistema 2 × 2, selecionado','2×2 selecionado');
 igual(sizeName(3,false),'Sistema 3 × 3','3×3');
 igual(TEXTS.METHOD_LINK_NAMES.gauss,'Passo a passo por escalonamento','link de escalonamento');
});

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
for(const id of ['icon-bulb','icon-arrow-left','think-open','copy','think','think-title','methods','copy-steps','steps','live','summary','footer-links','tip-disabled'])check(ids.includes(id),`elemento na versão offline: #${id}`);
check((html.match(/aria-live=/g)||[]).length===1,'uma única região aria-live');
check(html.includes('<title>Sistemas Lineares · Lucas Lima</title>'),'título da janela');
const partes=[systems?`${systems} sistemas idênticos ao Rust`:null,walkthroughs?`${walkthroughs} passo a passo do Modo pensar idênticos ao Rust`:null,`${vetor} potências do vetor compartilhado`,corpus?`${corpus} entradas do corpus diferencial sem divergência`:null,`${propriedades} propriedades de potência`,`${cenarios} cenários do modelo da interface`,`${checks} verificações`].filter(Boolean);
console.log(`PASS: ${partes.join('; ')}`);
