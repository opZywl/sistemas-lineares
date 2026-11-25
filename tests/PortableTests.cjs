const fs=require('node:fs');
const path=require('node:path');
const assert=require('node:assert/strict');
const solver=require('../portable/solver.js');
const {Rational:R,solve,format,METHODS,walkthrough,walkthroughText,matrixLines,PARSE_MESSAGES,MAX_POWER_DIGITS,LIVE_BUDGET_BITS,PREVIEW_DIGIT_LIMIT,usesExtension,looksIncomplete,withinLiveBudget,Entry,abbreviate,previewEquation}=solver;
const {Model,spokenValue,reading,accessibleInputName,accessibleValueName,rowNumber,sizeName,inputColumn,TEXTS}=require('../portable/app.js');
const G=require('../portable/graph.js');
const [fixturesFile,passosFile,corpusFile,graficosFile]=process.argv.slice(2);
let checks=0,systems=0,walkthroughs=0,vetor=0,corpus=0,propriedades=0,cenarios=0,cenas=0,testesGrafico=0,quadros=0;
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
function preencherSistema(dimensao,linhas){
 const model=new Model();
 model.changeDimension(dimensao);
 linhas.forEach((valores,row)=>valores.forEach((texto,column)=>model.edit(row,inputColumn(dimensao,column),texto)));
 return model;
}
if(graficosFile){
 const divergencias=[];
 for(const linha of fs.readFileSync(graficosFile,'utf8').split(/\r?\n/)){
  if(!linha)continue;
  const [dimensao,celulas,largura,altura,unidade,contraste,legenda,esperado]=linha.split('\t'),n=Number(dimensao),valores=celulas.split('|');
  check(valores.length===n*(n+1)&&esperado!==undefined,`linha malformada no corpus de gráficos: ${linha.slice(0,120)}`);
  const model=preencherSistema(n,Array.from({length:n},(_,row)=>valores.slice(row*(n+1),(row+1)*(n+1))));
  const motion=new G.Motion();
  motion.update(model.sketch(),0,false);
  const obtido=G.describe(motion.scene(1e9,Number(largura),Number(altura),Number(unidade),contraste==='1')),texto=model.helper();
  cenas++;
  if(texto===legenda&&obtido===esperado)continue;
  const rust=esperado.split(';'),js=obtido.split(';');
  let item=0;
  while(item<rust.length&&rust[item]===js[item])item++;
  divergencias.push(`${n}x${n} ${celulas} ${largura}x${altura} unidade ${unidade} contraste ${contraste}\n  legenda Rust:       ${legenda}\n  legenda JavaScript: ${texto}\n  item ${item}: Rust ${rust[item]??'(fim)'} / JavaScript ${js[item]??'(fim)'}`);
 }
 if(divergencias.length)console.error(divergencias.slice(0,10).join('\n'));
 check(divergencias.length===0,`${divergencias.length} cenas do gráfico diferentes do Rust`);
 check(cenas>1700,`o corpus de gráficos deveria ter mais de 1700 cenas, tem ${cenas}`);
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
 igual(model.helper(),G.INTRO_TWO,'a legenda do gráfico explica o que aparece ali');
 check(model.helperIsCaption(),'a dica inicial é a legenda do gráfico');
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
 igual(model.helper(),'As retas se cruzam em (2, 1): esse ponto é a solução do sistema.','a legenda do gráfico substitui o aviso do exemplo');
 check(model.helperIsCaption(),'a dica do exemplo é a legenda');
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
 igual(model.helper(),'As retas se cruzam em (6/5, 1/5): esse ponto é a solução do sistema.','a legenda acompanha o novo resultado');
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
 igual(model.helper(),G.INTRO_THREE,'a legenda volta à apresentação do 3×3');
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
 igual(model.helper(),G.INTRO_TWO,'a dica volta à legenda');
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
 igual(model.helper(),G.INTRO_TWO,'prefixo ainda não tem valor');
 check(!model.helperIsReading(),'sem leitura no prefixo');
 check(model.helperIsCaption(),'sem leitura, a dica é a legenda');
 model.edit(0,0,'3');
 igual(model.helper(),G.INTRO_TWO,'sem potência, a dica volta à legenda');
 model.edit(0,0,'2^3');
 model.setFocus([0,1]);
 igual(model.helper(),G.INTRO_TWO,'a dica segue o campo focado');
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
 check(!('DEFAULT_HELPER' in TEXTS),'sem dica parada: a legenda do gráfico ocupa o lugar');
});
cenario('nomes_do_tamanho_e_dos_links',()=>{
 igual(sizeName(2,true),'Sistema 2 × 2, selecionado','2×2 selecionado');
 igual(sizeName(3,false),'Sistema 3 × 3','3×3');
 igual(TEXTS.METHOD_LINK_NAMES.gauss,'Passo a passo por escalonamento','link de escalonamento');
});
const desenhado=row=>{check(row.kind==='drawn',`esperava um plano desenhado, veio ${JSON.stringify(row)}`);return row.plane;};
cenario('esboco_do_exemplo_2x2_tem_retas_e_ponto',()=>{
 const model=new Model();
 const vazio=model.sketch();
 igual(vazio.rows,[{kind:'blank'},{kind:'blank'}],'linhas vazias');
 igual(vazio.verdict,{kind:'pending'},'nada a mostrar ainda');
 model.fillExample();
 const sketch=model.sketch(),primeira=desenhado(sketch.rows[0]),raiz=Math.sqrt(5);
 igual(sketch.dimension,2,'dimensão do esboço');
 check(Math.abs(primeira.normal[0]-2/raiz)<1e-15&&Math.abs(primeira.normal[1]-1/raiz)<1e-15&&Math.abs(primeira.offset-raiz)<1e-15,'reta normalizada');
 igual(sketch.verdict,{kind:'point',at:[2,1,0],coordinates:['2','1']},'ponto da solução');
 model.edit(1,0,'');
 const editando=model.sketch();
 igual(editando.rows[1],{kind:'partial'},'linha incompleta');
 igual(editando.verdict,{kind:'pending'},'sem solução enquanto edita');
 igual(model.helper(),'Preencha a equação 2 para ver onde as retas se cruzam.','legenda pede a equação que falta');
});
cenario('esboco_do_exemplo_3x3_e_dos_casos_degenerados',()=>{
 let model=new Model();
 model.changeDimension(3);
 model.fillExample();
 igual(model.sketch().verdict,{kind:'point',at:[1,2,3],coordinates:['1','2','3']},'ponto do exemplo 3×3');
 igual(model.helper(),'Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema.','legenda do 3×3');
 model=new Model();
 preencher(model,[['1','1','2'],['2','2','4']]);
 igual(model.sketch().verdict,{kind:'line'},'retas coincidentes');
 igual(model.helper(),G.COINCIDENT_LINES,'legenda das coincidentes');
 preencher(model,[['1','1','2'],['0','0','0']]);
 igual(model.sketch().rows[1],{kind:'trivial'},'0 = 0');
 igual(model.sketch().verdict,{kind:'line'},'reta inteira');
 preencher(model,[['1','1','2'],['0','0','5/2']]);
 igual(model.sketch().rows[1],{kind:'impossible',constant:'5/2'},'0 = 5/2');
 igual(model.sketch().verdict,{kind:'nothing',rank:1},'sem solução');
 igual(model.helper(),'A equação 2 diz que 0 = 5/2, o que nunca acontece: o sistema não tem solução.','legenda da equação impossível');
 preencher(model,[['1','1','2'],['1','1','3']]);
 igual(model.helper(),G.PARALLEL_LINES,'paralelas');
 preencher(model,[['0','0','0'],['0','0','0']]);
 igual(model.sketch().verdict,{kind:'everything'},'tudo é solução');
 igual(model.helper(),G.EVERYTHING_TWO,'legenda do plano inteiro');
 model=new Model();
 model.changeDimension(3);
 preencher(model,[['1','1','1','1'],['1','-1','0','0'],['2','0','1','1']]);
 igual(model.sketch().verdict,{kind:'line'},'planos numa reta');
 igual(model.helper(),G.SHARED_LINE,'legenda da reta comum');
 preencher(model,[['1','1','1','1'],['2','2','2','2'],['0','0','0','0']]);
 igual(model.sketch().verdict,{kind:'plane'},'planos coincidentes');
 igual(model.helper(),G.COINCIDENT_PLANES,'legenda dos planos coincidentes');
 preencher(model,[['1','1','1','1'],['0','0','0','0'],['0','0','0','0']]);
 igual(model.helper(),'As outras equações valem para qualquer ponto (0 = 0), então todo o plano da equação 1 é solução.','legenda do plano solução');
 preencher(model,[['1','1','1','1'],['1','1','1','2'],['1','1','1','3']]);
 igual(model.sketch().verdict,{kind:'nothing',rank:1},'planos paralelos');
 igual(model.helper(),G.PARALLEL_PLANES,'legenda dos paralelos');
 preencher(model,[['1','0','0','0'],['0','1','0','0'],['1','1','0','1']]);
 igual(model.sketch().verdict,{kind:'nothing',rank:2},'prisma');
 igual(model.helper(),G.NO_COMMON_POINT,'legenda do prisma');
});
cenario('esboco_normaliza_numeros_enormes_e_espera_o_enter',()=>{
 let model=new Model();
 preencher(model,[['2e1000','1e1000','5e1000'],['1','-1','1']]);
 const sketch=model.sketch(),primeira=desenhado(sketch.rows[0]),raiz=Math.sqrt(5);
 check(Math.abs(primeira.normal[0]-2/raiz)<1e-15&&Math.abs(primeira.offset-raiz)<1e-15,'números enormes normalizados');
 igual(model.card(),{kind:'waiting',waiting:{kind:'large'}},'espera o Enter');
 igual(sketch.verdict,{kind:'waiting'},'gráfico esperando');
 igual(model.helper(),G.WAITING,'legenda pede o Enter');
 model.enter(0,0);
 check(model.sketch().verdict.kind==='point'&&model.sketch().verdict.at!==null,'Enter marca o ponto');
 model=new Model();
 preencher(model,[['1','1','1e200'],['1','-1','0']]);
 igual(model.sketch().rows[0],{kind:'huge'},'grande demais para desenhar');
 igual(model.helper(),G.HUGE,'legenda dos valores grandes');
 model=new Model();
 preencher(model,[['1e-400','0','1'],['0','1','0']]);
 igual(model.sketch().rows[0],{kind:'huge'},'termo grande demais');
 check(model.sketch().verdict.kind==='point'&&model.sketch().verdict.at===null,'ponto longe demais');
});
cenario('legenda_cede_lugar_ao_erro_e_aos_avisos',()=>{
 const model=new Model();
 model.setFocus([0,0]);
 model.edit(0,0,'1/0');
 model.leave(0,0);
 check(model.helperIsError()&&!model.helperIsCaption(),'erro no lugar da legenda');
 igual(model.helper(),'Equação 1, coeficiente de x: O denominador não pode ser zero.','erro mostrado');
 model.edit(0,0,'1');
 check(model.helperIsCaption(),'a legenda volta');
 igual(model.requestMethod('cramer'),{kind:'needsSystem',focus:campo(0,1)},'link sem sistema');
 check(!model.helperIsCaption(),'aviso no lugar da legenda');
 igual(model.helper(),TEXTS.METHOD_NEEDS_SYSTEM.cramer,'aviso mostrado');
});

const grafico=(nome,teste)=>{testesGrafico++;try{teste();}catch(error){error.message=`${nome}: ${error.message}`;throw error;}};
const vetorSub=(a,b)=>[a[0]-b[0],a[1]-b[1],a[2]-b[2]];
const vetorDot=(a,b)=>a[0]*b[0]+a[1]*b[1]+a[2]*b[2];
const vetorCross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const reta=(a,b,d)=>({kind:'drawn',plane:G.plane([a,b,0],d)});
const plano3=(a,b,c,d)=>({kind:'drawn',plane:G.plane([a,b,c],d)});
const exemplo2=()=>({dimension:2,rows:[reta(2,1,5),reta(1,-1,1)],verdict:{kind:'point',at:[2,1,0],coordinates:['2','1']}});
const exemplo3=()=>({dimension:3,rows:[plano3(1,1,1,6),plano3(2,-1,1,3),plano3(1,2,-1,2)],verdict:{kind:'point',at:[1,2,3],coordinates:['1','2','3']}});
const emRepouso=(sketch,largura,altura)=>{const motion=new G.Motion();motion.update(sketch,0,false);return motion.scene(1e9,largura,altura,1,false);};
const distancia=(a,b)=>Math.sqrt((b.x-a.x)**2+(b.y-a.y)**2);
const ehReta=item=>item.ink==='r0'||item.ink==='r1'||item.ink==='r2';
const textos=itens=>itens.filter(item=>item.shape.kind==='text').map(item=>item.shape.text);
function mesmoValor(a,b){
 if(typeof a==='number'||typeof b==='number')return a===b;
 if(Array.isArray(a)||Array.isArray(b))return Array.isArray(a)&&Array.isArray(b)&&a.length===b.length&&a.every((valor,indice)=>mesmoValor(valor,b[indice]));
 if(a&&b&&typeof a==='object'&&typeof b==='object'){const chaves=Object.keys(a);return chaves.length===Object.keys(b).length&&chaves.every(chave=>Object.prototype.hasOwnProperty.call(b,chave)&&mesmoValor(a[chave],b[chave]));}
 return a===b;
}
function pontosDoItem(item){
 const forma=item.shape;
 if(forma.kind==='segment')return [forma.from,forma.to];
 if(forma.kind==='polygon'||forma.kind==='outline')return forma.points;
 if(forma.kind==='disc'||forma.kind==='ring')return [forma.center];
 if(forma.kind==='text')return [forma.at];
 return [];
}
function dentroDaArea(itens,largura,altura,folga){
 for(const item of itens){
  if(item.shape.kind==='text')continue;
  check(pontosDoItem(item).every(ponto=>ponto.x>=-folga&&ponto.y>=-folga&&ponto.x<=largura+folga&&ponto.y<=altura+folga),`item fora da área: ${JSON.stringify(item)}`);
 }
}
class Sorteio{
 constructor(semente){this.estado=semente;}
 proximo(){
  const mascara=(1n<<64n)-1n;
  let valor=this.estado;
  valor^=(valor<<13n)&mascara;
  valor^=valor>>7n;
  valor^=(valor<<17n)&mascara;
  this.estado=valor;
  return valor;
 }
 ate(limite){return Number(this.proximo()%BigInt(limite));}
}
const VALORES_SORTEADOS=['0','0','1','-1','2','-2','3','-3','4','5','-5','7','1/2','-3/4','2/3','1,5','-0,25','10','12','-8','6','9','1e3','2^3','(1/2)^2','-2²'];
function sistemaSorteado(sorteio,dimensao){
 const linhas=Array.from({length:dimensao},()=>Array.from({length:dimensao+1},()=>{
  const sorteado=sorteio.ate(100);
  return sorteado<=3?'':sorteado<=5?'1/0':VALORES_SORTEADOS[sorteio.ate(VALORES_SORTEADOS.length)];
 }));
 const caso=sorteio.ate(20);
 if(caso<=2){
  const fator=[2,-1,3][sorteio.ate(3)],base=Array.from({length:dimensao+1},()=>sorteio.ate(9)-4);
  linhas[0]=base.map(String);
  linhas[1]=base.map(valor=>String(valor*fator));
  if(sorteio.ate(2)===0)linhas[1][dimensao]=String(sorteio.ate(9)-4);
 }else if(caso===3){
  const linha=sorteio.ate(dimensao);
  for(let coluna=0;coluna<dimensao;coluna++)linhas[linha][coluna]='0';
 }
 return linhas;
}
grafico('conversao_para_ponto_flutuante_arredonda_e_limita',()=>{
 const q=(numerador,denominador)=>new R(BigInt(numerador),BigInt(denominador));
 igual([q(0,1).toF64(),q(5,1).toF64(),q(-7,2).toF64(),q(1,3).toF64(),q(-2,3).toF64(),q(1,10).toF64()],[0,5,-3.5,1/3,-2/3,0.1],'conversões simples');
 igual(new R(9007199254740993n,1n).toF64(),9007199254740992,'arredonda para o par mais próximo');
 igual(new R(10n**400n,10n**399n).toF64(),10,'fração com números enormes');
 igual(new R(10n**1000n,1n).toF64(),Infinity,'infinito positivo');
 igual(new R(-(10n**1000n),1n).toF64(),-Infinity,'infinito negativo');
 igual(new R(1n,10n**1000n).toF64(),0,'pequeno demais vira zero');
 igual(new R(10n**100n,1n).toF64(),1e100,'1e100 exato');
 igual(new R(1n,10n**100n).toF64(),1e-100,'1e-100 exato');
 for(let numerador=-50;numerador<=50;numerador++)for(let denominador=1;denominador<=40;denominador++)igual(q(numerador,denominador).toF64(),numerador/denominador,`${numerador}/${denominador}`);
 igual(G.LIMIT,1e100,'limite do desenho');
});
grafico('modulo_e_comparacao_de_magnitude',()=>{
 const q=(numerador,denominador)=>new R(BigInt(numerador),BigInt(denominador));
 igual([String(q(-3,4).abs()),String(q(3,4).abs())],['3/4','3/4'],'módulo');
 igual([q(-3,4).cmpMagnitude(q(1,2)),q(1,3).cmpMagnitude(q(-1,2)),q(-2,4).cmpMagnitude(q(1,2)),q(0,1).cmpMagnitude(q(0,5))],[1,-1,0,0],'comparação de magnitude');
});
grafico('passo_da_grade_e_redondo',()=>{
 igual([1,1.25,3,7,0.03,1500].map(G.nice),[1,2,5,10,0.05,2000],'passos redondos');
});
grafico('plano_normaliza_e_rejeita_o_que_nao_desenha',()=>{
 const plano=G.plane([3,4,0],10);
 igual(plano,{normal:[0.6,0.8,0],offset:2},'plano normalizado');
 igual(G.foot(plano),[1.2,1.6,0],'pé da perpendicular');
 igual([G.plane([0,0,0],1),G.plane([1,0,0],Infinity),G.plane([1,0,0],1e101)],[null,null,null],'planos que não se desenham');
});
grafico('legendas_de_cada_caso',()=>{
 igual(G.caption(G.blank(2)),G.INTRO_TWO,'apresentação 2×2');
 igual(G.caption(G.blank(3)),G.INTRO_THREE,'apresentação 3×3');
 const sketch=G.blank(3);
 sketch.rows[0]=plano3(1,0,0,1);
 igual(G.caption(sketch),'Preencha as equações 2 e 3 para ver onde os planos se cruzam.','duas faltando');
 sketch.rows[1]={kind:'trivial'};
 sketch.rows[2]={kind:'partial'};
 igual(G.caption(sketch),'Preencha a equação 3 para ver onde os planos se cruzam.','uma faltando');
 const duas=G.blank(2);
 duas.rows[1]=reta(1,1,1);
 igual(G.caption(duas),'Preencha a equação 1 para ver onde as retas se cruzam.','a primeira faltando');
 igual(G.caption(exemplo2()),'As retas se cruzam em (2, 1): esse ponto é a solução do sistema.','exemplo 2×2');
 igual(G.caption(exemplo3()),'Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema.','exemplo 3×3');
 igual(G.caption({...exemplo2(),verdict:{kind:'point',at:null,coordinates:['1e200','2']}}),'A solução é (1e200, 2), longe demais para caber no gráfico.','longe demais');
 igual(G.caption({...exemplo2(),verdict:{kind:'waiting'}}),G.WAITING,'esperando o Enter');
 igual(G.caption({dimension:2,rows:[reta(1,1,2),reta(2,2,4)],verdict:{kind:'line'}}),G.COINCIDENT_LINES,'coincidentes');
 igual(G.caption({dimension:2,rows:[{kind:'trivial'},reta(1,1,2)],verdict:{kind:'line'}}),'A equação 1 vale para qualquer ponto (0 = 0), então toda a reta da equação 2 é solução.','reta inteira');
 igual(G.caption({dimension:2,rows:[reta(1,1,2),reta(1,1,3)],verdict:{kind:'nothing',rank:1}}),G.PARALLEL_LINES,'paralelas');
 igual(G.caption({dimension:2,rows:[reta(1,1,2),{kind:'impossible',constant:'5'}],verdict:{kind:'nothing',rank:1}}),'A equação 2 diz que 0 = 5, o que nunca acontece: o sistema não tem solução.','equação impossível');
 igual(G.caption({dimension:3,rows:[{kind:'trivial'},{kind:'trivial'},{kind:'trivial'}],verdict:{kind:'everything'}}),G.EVERYTHING_THREE,'espaço inteiro');
 igual(G.caption({dimension:3,rows:[{kind:'trivial'},plano3(1,1,1,1),{kind:'trivial'}],verdict:{kind:'plane'}}),'As outras equações valem para qualquer ponto (0 = 0), então todo o plano da equação 2 é solução.','plano solução');
 igual(G.caption({...exemplo2(),rows:[{kind:'huge'},reta(1,1,1)]}),G.HUGE,'valores enormes');
 igual(G.caption({dimension:3,rows:[plano3(1,0,0,0),plano3(0,1,0,0),plano3(1,1,0,1)],verdict:{kind:'nothing',rank:2}}),G.NO_COMMON_POINT,'prisma');
 igual(G.GRAPH_NAME,'Gráfico do sistema','nome acessível do gráfico');
});
grafico('janela_contem_origem_ponto_e_pes',()=>{
 let janela=G.bounds(exemplo2());
 check(Math.abs(janela.center[0]-1)<1e-12&&Math.abs(janela.center[1]-0.25)<1e-12&&janela.center[2]===0,'centro da janela 2×2');
 igual(janela.half,[G.MIN_HALF,G.MIN_HALF,0],'meia janela mínima');
 janela=G.bounds(exemplo3());
 const baixo=vetorSub(janela.center,janela.half),alto=janela.center.map((valor,eixo)=>valor+janela.half[eixo]);
 for(const ponto of [[0,0,0],[1,2,3],[2,2,2]])for(let eixo=0;eixo<3;eixo++)check(baixo[eixo]<ponto[eixo]&&ponto[eixo]<alto[eixo],`a caixa contém ${ponto}`);
 igual(janela.half[1],janela.half[0]*G.BOX_HEIGHT,'caixa mais baixa');
 janela=G.bounds({...exemplo2(),verdict:{kind:'point',at:[1000,-400,0],coordinates:['1000','-400']}});
 check(janela.center[0]-janela.half[0]<0&&janela.center[0]+janela.half[0]>1000,'janela cobre o x distante');
 check(janela.center[1]-janela.half[1]<-400&&janela.center[1]+janela.half[1]>0,'janela cobre o y distante');
});
grafico('cena_2d_em_repouso_fica_dentro_da_area',()=>{
 const itens=emRepouso(exemplo2(),252,120);
 dentroDaArea(itens,252,120,0.01);
 const linhas=itens.filter(item=>ehReta(item)&&item.alpha===1),pontos=itens.filter(item=>item.ink==='o');
 igual([linhas.length,pontos.length],[2,1],'duas retas e um ponto');
 const centro=pontos[0].shape.center,meiaLargura=G.MIN_HALF*(252/120);
 check(Math.abs(centro.x-(2-(1-meiaLargura))/(2*meiaLargura)*252)<1e-9&&Math.abs(centro.y-(0.25+G.MIN_HALF-1)/(2*G.MIN_HALF)*120)<1e-9,'ponto na posição da solução');
 igual(textos(itens),['x','y','2','1'],'rótulos dos eixos e das coordenadas');
 linhas.forEach((linha,row)=>{
  const plano=exemplo2().rows[row].plane;
  for(const ponto of [linha.shape.from,linha.shape.to]){
   const x=(1-meiaLargura)+ponto.x/252*2*meiaLargura,y=(0.25+G.MIN_HALF)-ponto.y/120*2*G.MIN_HALF,resto=plano.normal[0]*x+plano.normal[1]*y-plano.offset;
   check(Math.abs(resto)<1e-9,`a reta ${row+1} passa pela equação (${resto})`);
  }
 });
});
grafico('cena_3d_em_repouso_tem_planos_ordenados_e_ponto',()=>{
 const itens=emRepouso(exemplo3(),147,70);
 dentroDaArea(itens,147,70,0.01);
 igual(itens.filter(item=>ehReta(item)&&item.shape.kind==='polygon').length,3,'três planos preenchidos');
 igual(itens.filter(item=>item.ink==='m').length,3,'três retas de encontro');
 igual(itens.filter(item=>item.ink==='o').length,1,'um ponto');
 igual(textos(itens),['x','z','y'],'rótulos dos eixos');
});
grafico('secao_do_plano_no_cubo_e_convexa',()=>{
 const baixo=[-1,-1,-1],alto=[1,1,1];
 const hexagono=G.section(G.plane([1,1,1],0),baixo,alto),quadrado=G.section(G.plane([0,0,1],0),baixo,alto),triangulo=G.section(G.plane([1,1,1],2),baixo,alto),face=G.section(G.plane([1,0,0],1),baixo,alto);
 igual([hexagono.length,quadrado.length,triangulo.length,face.length],[6,4,3,4],'vértices de cada seção');
 igual(G.section(G.plane([1,0,0],5),baixo,alto),[],'plano fora da caixa');
 for(const poligono of [hexagono,quadrado,triangulo,face]){
  const normal=vetorCross(vetorSub(poligono[1],poligono[0]),vetorSub(poligono[2],poligono[1]));
  poligono.forEach((a,indice)=>{
   const b=poligono[(indice+1)%poligono.length],c=poligono[(indice+2)%poligono.length];
   check(vetorDot(vetorCross(vetorSub(b,a),vetorSub(c,b)),normal)>-1e-9,'polígono convexo e em ordem');
  });
 }
 igual([G.totalCmp(-0,0),G.totalCmp(0,-0),G.totalCmp(1,NaN),G.totalCmp(NaN,NaN),G.totalCmp(-Infinity,-1)],[-1,1,-1,0,-1],'ordem total como no Rust');
});
grafico('animacao_comeca_vazia_e_termina_em_repouso',()=>{
 const motion=new G.Motion();
 motion.update(G.blank(2),0,true);
 check(!motion.busy(0,false),'o gráfico vazio não anima');
 motion.update(exemplo2(),1,true);
 check(motion.busy(1,false),'o exemplo anima');
 const comprimentos=itens=>itens.filter(item=>ehReta(item)&&item.alpha>0.5).map(item=>item.shape.kind==='segment'?distancia(item.shape.from,item.shape.to):0);
 const cedo=motion.scene(1.05,252,120,1,false),parcial=comprimentos(cedo);
 const fim=1+Math.max(G.APPEAR,G.POP,G.PULSE_DELAY+2*G.PULSE_GAP+G.PULSE)+0.01;
 check(!motion.busy(fim,false),'a animação termina');
 const pronto=motion.scene(fim,252,120,1,false),inteiro=comprimentos(pronto);
 igual(parcial.length,2,'as duas retas aparecem');
 parcial.forEach((comprimento,indice)=>check(comprimento<inteiro[indice],'as retas crescem'));
 check(mesmoValor(pronto,emRepouso(exemplo2(),252,120)),'termina igual à cena de repouso');
 igual(cedo.filter(item=>item.shape.kind==='ring').length,0,'sem pulsos no começo');
 check(motion.scene(1+G.PULSE_DELAY+0.3,252,120,1,false).some(item=>item.shape.kind==='ring'),'pulsos depois');
});
grafico('linha_que_some_desbota_e_volta_sem_redesenhar',()=>{
 let motion=new G.Motion();
 motion.update(exemplo2(),0,false);
 const editando=exemplo2();
 editando.rows[1]={kind:'partial'};
 editando.verdict={kind:'pending'};
 motion.update(editando,10,true);
 const fantasma=motion.scene(10.1,252,120,1,false).find(item=>item.ink==='r1'&&item.shape.kind==='segment'&&item.shape.width===2).alpha;
 check(fantasma>0&&fantasma<1,'a reta desbota');
 check(motion.busy(10.1,false)&&!motion.busy(10+G.VANISH+G.PULSE*4,false),'o desbotamento termina');
 check(!motion.scene(11,252,120,1,false).some(item=>item.ink==='r1'),'a reta some');
 const movido=exemplo2();
 movido.rows[0]=reta(2,1,6);
 movido.verdict={kind:'point',at:[7/3,4/3,0],coordinates:['7/3','4/3']};
 motion=new G.Motion();
 motion.update(exemplo2(),0,false);
 motion.update(movido,5,true);
 check(motion.scene(5.1,252,120,1,false).some(item=>item.ink==='r0'&&item.alpha===1),'a reta muda sem sumir');
 check(motion.busy(5.1,false),'ocupado enquanto muda');
});
grafico('sem_animacao_e_alto_contraste',()=>{
 const motion=new G.Motion();
 motion.update(exemplo3(),0,true);
 check(!motion.busy(0,false),'a primeira cena não anima');
 const itens=motion.scene(0,147,70,1,true);
 check(itens.every(item=>item.alpha===1),'sem transparência');
 check(!itens.some(item=>item.shape.kind==='dots'),'sem textura');
 check(!itens.some(item=>item.ink==='h'||item.ink==='p'),'sem brilho nem paredes preenchidas');
 igual(itens.filter(item=>ehReta(item)&&item.shape.kind==='polygon').length,0,'planos só com contorno');
});
grafico('conjunto_solucao_marcha_e_para',()=>{
 const sketch={dimension:2,rows:[reta(1,1,2),reta(2,2,4)],verdict:{kind:'line'}},motion=new G.Motion();
 motion.update(G.blank(2),0,false);
 motion.update(sketch,1,true);
 const pontinhos=itens=>itens.filter(item=>item.ink==='s'&&item.shape.kind==='disc').map(item=>item.shape.center);
 const primeiro=pontinhos(motion.scene(1.2,252,120,1,false)),segundo=pontinhos(motion.scene(1.4,252,120,1,false));
 check(primeiro.length>0&&!mesmoValor(primeiro,segundo),'os pontinhos marcham');
 check(motion.busy(1+G.MARCH-0.01,false)&&!motion.busy(1+G.MARCH+1,false),'a marcha para');
 check(mesmoValor(motion.scene(1+G.MARCH+1,252,120,1,false),emRepouso(sketch,252,120)),'para na cena de repouso');
 check(emRepouso({dimension:2,rows:[{kind:'trivial'},{kind:'trivial'}],verdict:{kind:'everything'}},252,120).some(item=>item.ink==='s'&&item.shape.kind==='dots'),'textura acesa quando tudo é solução');
});
grafico('valores_enormes_nao_desenham',()=>{
 const itens=emRepouso({dimension:2,rows:[{kind:'huge'},reta(1,1,1)],verdict:{kind:'nothing',rank:1}},252,120);
 check(!itens.some(item=>ehReta(item)||item.ink==='o'),'nada desenhado com valores enormes');
});
grafico('descricao_textual_e_estavel',()=>{
 const texto=G.describe(emRepouso(exemplo2(),252,120));
 check(texto.startsWith('t1000:D800,90;g1000:S')&&texto.includes('o1000:C'),texto.slice(0,60));
 igual(texto,G.describe(emRepouso(exemplo2(),252,120)),'descrição estável');
});
grafico('ponto_desliza_do_fantasma_quando_a_equacao_volta',()=>{
 const motion=new G.Motion();
 motion.update(exemplo2(),0,false);
 const editando=exemplo2();
 editando.rows[1]={kind:'partial'};
 editando.verdict={kind:'pending'};
 motion.update(editando,10,true);
 const corrigido=exemplo2();
 corrigido.rows[1]=reta(1,-1,-3);
 corrigido.verdict={kind:'point',at:[2/3,11/3,0],coordinates:['2/3','11/3']};
 motion.update(corrigido,10.1,true);
 const ponto=itens=>itens.find(item=>item.ink==='o');
 const inicio=ponto(motion.scene(10.1,252,120,1,false));
 check(inicio!==undefined&&inicio.shape.radius===4.25,'o ponto volta inteiro, sem nascer de novo');
 const meio=ponto(motion.scene(10.1+G.MORPH/2,252,120,1,false)),fim=ponto(emRepouso(corrigido,252,120));
 check(!mesmoValor(meio.shape.center,fim.shape.center),'o ponto desliza até o lugar novo');
 check(mesmoValor(ponto(motion.scene(20,252,120,1,false)),fim),'o ponto para no lugar certo');
});
grafico('alto_contraste_nao_espera_o_que_nao_aparece',()=>{
 let motion=new G.Motion();
 motion.update(G.blank(2),0,false);
 motion.update(exemplo2(),1,true);
 const calmo=1+Math.max(G.APPEAR,G.POP,G.VIEW)+0.01;
 check(motion.busy(calmo,false)&&!motion.busy(calmo,true),'no alto contraste não espera os pulsos');
 motion=new G.Motion();
 motion.update(G.blank(2),0,false);
 motion.update({dimension:2,rows:[{kind:'trivial'},{kind:'trivial'}],verdict:{kind:'everything'}},1,true);
 check(motion.busy(1+G.LIGHT_UP-0.01,false)&&!motion.busy(1+G.LIGHT_UP+0.01,false)&&!motion.busy(1,true),'a textura acende e para');
 const plano={dimension:3,rows:[{kind:'trivial'},plano3(1,1,1,1),{kind:'trivial'}],verdict:{kind:'plane'}};
 motion=new G.Motion();
 motion.update(plano,0,false);
 motion.update({...plano,rows:[{kind:'partial'},plano3(1,1,1,1),{kind:'trivial'}]},1,true);
 check(!motion.busy(1,false),'o plano solução não prende quadros à toa');
});
grafico('rotulos_fogem_das_retas_e_das_letras',()=>{
 const sketch={dimension:2,rows:[reta(3,0,1),reta(0,1,2)],verdict:{kind:'point',at:[1/3,2,0],coordinates:['1/3','2']}};
 const caixa=({at,text,horizontal,vertical})=>{
  const largura=Array.from(text).length*G.CHAR_WIDTH,altura=G.TEXT_HEIGHT;
  const esquerda=horizontal==='s'?at.x:horizontal==='m'?at.x-largura/2:at.x-largura,topo=vertical==='s'?at.y:vertical==='m'?at.y-altura/2:at.y-altura;
  return {esquerda,topo,direita:esquerda+largura,base:topo+altura};
 };
 const cruza=(area,[a,b,c])=>{
  const lados=[[area.esquerda,area.topo],[area.direita,area.topo],[area.esquerda,area.base],[area.direita,area.base]].map(([x,y])=>a*x+b*y-c);
  return !lados.every(lado=>lado>0)&&!lados.every(lado=>lado<0);
 };
 for(const [largura,altura] of [[252,120],[267,128],[233,111]]){
  const itens=emRepouso(sketch,largura,altura);
  const retas=itens.filter(item=>ehReta(item)&&item.alpha===1&&item.shape.kind==='segment').map(({shape:{from,to}})=>{const a=to.y-from.y,b=from.x-to.x;return [a,b,a*from.x+b*from.y];});
  igual(retas.length,2,'duas retas');
  const caixas=itens.filter(item=>item.shape.kind==='text').map(item=>caixa(item.shape));
  igual(caixas.length,4,'quatro rótulos');
  for(const area of caixas)for(const linha of retas)check(!cruza(area,linha),`rótulo sobre uma reta em ${largura}x${altura}`);
  caixas.forEach((uma,indice)=>caixas.slice(indice+1).forEach(outra=>check(!(uma.esquerda<outra.direita&&outra.esquerda<uma.direita&&uma.topo<outra.base&&outra.topo<uma.base),'rótulos sobrepostos')));
 }
});
grafico('grade_nitida_em_qualquer_escala',()=>{
 for(const [unidade,fracao] of [[1,0.5],[1.5,0.5],[2,0],[3,0.5]]){
  const motion=new G.Motion();
  motion.update(exemplo2(),0,false);
  const posicoes=motion.scene(1e9,252*unidade,120*unidade,unidade,false).filter(item=>(item.ink==='g'||item.ink==='a')&&item.shape.kind==='segment').map(({shape:{from,to}})=>from.x===to.x?from.x:from.y);
  check(posicoes.length>4,'grade presente');
  for(const valor of posicoes)igual(valor-Math.floor(valor),fracao,`grade nítida na escala ${unidade}`);
 }
});
grafico('pontinhos_do_conjunto_tem_fundo_no_alto_contraste',()=>{
 const motion=new G.Motion();
 motion.update({dimension:2,rows:[reta(1,1,2),reta(2,2,4)],verdict:{kind:'line'}},0,false);
 const itens=motion.scene(1e9,252,120,1,true);
 const indices=itens.map((item,indice)=>item.ink==='s'?indice:-1).filter(indice=>indice>=0);
 check(indices.length>3,'pontinhos no conjunto solução');
 for(const indice of indices){
  const fundo=itens[indice-1],pontinho=itens[indice];
  check(fundo.ink==='b'&&mesmoValor(fundo.shape.center,pontinho.shape.center)&&fundo.shape.radius>pontinho.shape.radius,'cada pontinho tem fundo no alto contraste');
 }
 check(!motion.scene(1e9,252,120,1,false).some(item=>item.ink==='b'),'sem fundo fora do alto contraste');
});
grafico('cores_do_grafico_iguais_as_do_theme_rs',()=>{
 const ler=nome=>fs.readFileSync(path.join(__dirname,'..','native','ui','src',nome),'utf8').replace(/\r\n/g,'\n');
 const tema=ler('theme.rs'),desenho=ler('graph.rs');
 const corpo=(texto,assinatura)=>{const inicio=texto.indexOf(assinatura),fim=texto.indexOf('\n    }\n',inicio);check(inicio>=0&&fim>inicio,`${assinatura} não encontrado no código Rust`);return texto.slice(inicio,fim);};
 const constantes=new Map([...tema.matchAll(/pub const (\w+): Color = Color::rgb\((\d+), (\d+), (\d+)\);/g)].map(([,nome,r,g,b])=>[nome,`rgb(${r},${g},${b})`]));
 const SISTEMA={window:'Canvas',window_text:'CanvasText',highlight:'Highlight',hot_light:'LinkText',gray_text:'GrayText'};
 const campos=(texto,cor)=>{
  const mapa=new Map([...texto.matchAll(/(graph_\w+): ([\w.]+),/g)].map(([,campo,origem])=>[campo,cor(origem)]));
  const linhas=/graph_rows: \[([^\]]+)\]/.exec(texto);
  check(linhas!==null,'graph_rows no theme.rs');
  linhas[1].split(',').forEach((origem,indice)=>mapa.set(`graph_rows[${indice}]`,cor(origem.trim())));
  return mapa;
 };
 const escuro=campos(corpo(tema,'pub const fn dark('),origem=>constantes.get(origem));
 const contraste=campos(corpo(tema,'pub const fn high_contrast('),origem=>SISTEMA[origem.replace(/^system\./,'')]);
 const codigos=new Map([...corpo(desenho,'pub const fn code(').matchAll(/Ink::([\w()]+) => "(\w+)"/g)].map(([,tinta,codigo])=>[tinta,codigo]));
 const tintas=[...corpo(desenho,'pub const fn color(').matchAll(/Ink::([\w()]+) => palette\.([\w[\]]+),/g)];
 igual(tintas.map(([,tinta])=>codigos.get(tinta)).sort(),Object.keys(G.PALETTE).filter(chave=>chave!=='fill'&&chave!=='stroke').sort(),'as mesmas tintas no Rust e no HTML');
 for(const [,tinta,campo] of tintas){
  const codigo=codigos.get(tinta);
  check(escuro.get(campo)!==undefined&&contraste.get(campo)!==undefined,`cores de ${campo} reconhecidas no theme.rs`);
  igual([G.inkColor(codigo,false),G.inkColor(codigo,true)],[escuro.get(campo),contraste.get(campo)],`tinta ${codigo} (${campo}) com as cores do theme.rs`);
 }
 for(const campo of ['fill','stroke'])igual([G.PALETTE[campo],G.CONTRAST_PALETTE[campo]],[escuro.get(`graph_${campo}`),contraste.get(`graph_${campo}`)],`graph_${campo} igual ao do theme.rs`);
 igual(Object.keys(G.CONTRAST_PALETTE).sort(),Object.keys(G.PALETTE).sort(),'as duas paletas cobrem as mesmas tintas');
});
grafico('animacao_completa_chega_na_cena_de_repouso',()=>{
 const sorteio=new Sorteio(0xA11CE5ED00000001n);
 for(let indice=0;indice<300;indice++){
  const dimensao=2+indice%2,antes=preencherSistema(dimensao,sistemaSorteado(sorteio,dimensao)).sketch(),depois=preencherSistema(dimensao,sistemaSorteado(sorteio,dimensao)).sketch();
  const [largura,altura]=dimensao===2?[267,128]:[163,78],motion=new G.Motion(),contraste=indice%5===0;
  motion.update(antes,0,false);
  motion.update(depois,1,true);
  let agora=1;
  while(motion.busy(agora,contraste)){
   for(const item of motion.scene(agora,largura,altura,1,contraste)){
    check(item.alpha>=0&&item.alpha<=1,`alfa fora de 0 a 1 no quadro ${agora}`);
    check(pontosDoItem(item).every(ponto=>Number.isFinite(ponto.x)&&Number.isFinite(ponto.y)),`coordenada inválida no quadro ${agora}`);
   }
   agora+=1/30;
   quadros++;
   check(agora<10,'a animação precisa terminar');
  }
  const fixo=new G.Motion();
  fixo.update(depois,0,false);
  check(mesmoValor(motion.scene(agora,largura,altura,1,contraste),fixo.scene(1e9,largura,altura,1,contraste)),`a animação ${indice} termina na cena de repouso`);
 }
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
check(!/__STYLE__|__FONT__|__SOLVER__|__GRAPH__|__APP__|__ICONS__/.test(html),'arquivo offline completo');
for(const id of ['icon-bulb','icon-arrow-left','think-open','copy','think','think-title','methods','copy-steps','steps','live','summary','footer-links','tip-disabled','graph','plot','help'])check(ids.includes(id),`elemento na versão offline: #${id}`);
check(html.includes(`<svg id="plot" role="img" aria-label="${G.GRAPH_NAME}" aria-describedby="help"`),'gráfico como imagem com nome e descrição');
check(html.indexOf('id="equations"')<html.indexOf('id="graph"')&&html.indexOf('id="graph"')<html.indexOf('<figcaption id="help"')&&html.indexOf('<figcaption id="help"')<html.indexOf('class="actions"'),'painel do gráfico entre as equações e os botões, com a legenda como figcaption');
check(html.indexOf('root.LinearSolver=api')<html.indexOf('root.LinearGraph=api')&&html.indexOf('root.LinearGraph=api')<html.indexOf('const graph=node?'),'graph.js entre o solver e o app');
check((html.match(/aria-live=/g)||[]).length===1,'uma única região aria-live');
check(html.includes('<title>Sistemas Lineares · Lucas Lima</title>'),'título da janela');
check(html.includes('Aceita 2 · -1,5 · 3/4 · 2^3 · 10². Use 0 se a variável não aparecer.'),'formatos aceitos na descrição dos campos');
check(!ids.includes('solve')&&!html.includes('Resolver sistema'),'sem botão Resolver sistema');
const partes=[systems?`${systems} sistemas idênticos ao Rust`:null,walkthroughs?`${walkthroughs} passo a passo do Modo pensar idênticos ao Rust`:null,`${vetor} potências do vetor compartilhado`,corpus?`${corpus} entradas do corpus diferencial sem divergência`:null,`${propriedades} propriedades de potência`,`${cenarios} cenários do modelo da interface`,cenas?`${cenas} cenas do gráfico idênticas ao Rust`:null,`${testesGrafico} testes do gráfico (${quadros} quadros de animação conferidos)`,`${checks} verificações`].filter(Boolean);
console.log(`PASS: ${partes.join('; ')}`);
