const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const assert=require('node:assert/strict');
const {spawn,execFileSync}=require('node:child_process');
const {pathToFileURL}=require('node:url');
const solver=require('../portable/solver.js');
const {TEXTS}=require('../portable/app.js');
const HTML=path.join(__dirname,'..','SistemasLineares.html');
const SEM_CHROME=3,PRAZO_TOTAL=240000;
const METHOD_TIPS={'gauss-jordan':'Gauss-Jordan: operações de linha até a matriz identidade.',cramer:'Regra de Cramer: determinantes, com Sarrus no 3×3.',gauss:'Escalonamento: zera abaixo dos pivôs e resolve de baixo para cima.'};
const FIELD_DESCRIPTION='Aceita 2 · -1,5 · 3/4 · 2^3 · 10². Enter vai ao próximo campo vazio; as setas para cima e para baixo mudam de equação.';
const TIP={think:'Ver como a resposta foi calculada, passo a passo.',copy:'Copiar a resposta como texto.',clear:'Apagar todos os campos.',example:'Substitui os campos por um sistema de exemplo.',disabled:'Disponível quando todos os campos estiverem preenchidos.'};
let checks=0,etapas=0,etapaAtual='';

function check(valor,descricao){checks++;if(!valor)throw Error(descricao);}
function igual(obtido,esperado,descricao){
 checks++;
 try{assert.deepStrictEqual(obtido,esperado);}
 catch(_){throw Error(`${descricao}\n  esperado: ${JSON.stringify(esperado)}\n  obtido:   ${JSON.stringify(obtido)}`);}
}
function acharChrome(){
 const candidatos=[];
 if(process.env.CHROME_PATH)candidatos.push(process.env.CHROME_PATH);
 const nomes=process.platform==='win32'?['chrome.exe']:['google-chrome','google-chrome-stable','chromium','chromium-browser','chrome'];
 for(const pasta of (process.env.PATH||'').split(path.delimiter))if(pasta)for(const nome of nomes)candidatos.push(path.join(pasta,nome));
 if(process.platform==='win32'){
  for(const base of [process.env.ProgramFiles,process.env['ProgramFiles(x86)'],process.env.LOCALAPPDATA,'C:\\Program Files','C:\\Program Files (x86)'])if(base)candidatos.push(path.join(base,'Google','Chrome','Application','chrome.exe'));
  for(const raiz of ['HKCU','HKLM']){
   try{
    const saida=execFileSync('reg',['query',`${raiz}\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\chrome.exe`,'/ve'],{encoding:'utf8',stdio:['ignore','pipe','ignore']});
    const achado=/REG_(?:EXPAND_)?SZ\s+(.+?)\s*$/m.exec(saida);
    if(achado)candidatos.push(achado[1].replace(/^"|"$/g,'').replace(/%([^%]+)%/g,(_,nome)=>process.env[nome]??''));
   }catch(_){}
  }
 }else if(process.platform==='darwin')candidatos.push('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome');
 else candidatos.push('/usr/bin/google-chrome','/usr/bin/google-chrome-stable','/opt/google/chrome/chrome','/usr/bin/chromium','/usr/bin/chromium-browser','/snap/bin/chromium');
 return candidatos.find(arquivo=>!/msedge|microsoft[\\/ ]edge/i.test(arquivo)&&fs.existsSync(arquivo)&&fs.statSync(arquivo).isFile())??null;
}

const PREPARO=`(()=>{
 let agora=0,sequencia=0;
 const pendentes=new Map();
 window.setTimeout=(funcao,atraso,...argumentos)=>{const id=++sequencia;pendentes.set(id,{quando:agora+Math.max(0,Number(atraso)||0),funcao,argumentos});return id;};
 window.clearTimeout=id=>{pendentes.delete(id);};
 window.__relogio={avancar(ms){const fim=agora+ms;for(;;){let proximo=null;for(const [id,item] of pendentes)if(item.quando<=fim&&(!proximo||item.quando<proximo[1].quando||(item.quando===proximo[1].quando&&id<proximo[0])))proximo=[id,item];if(!proximo)break;pendentes.delete(proximo[0]);agora=proximo[1].quando;if(typeof proximo[1].funcao==='function')proximo[1].funcao(...proximo[1].argumentos);}agora=fim;}};
 window.__copias=[];
 window.__falharCopia=false;
 Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:texto=>window.__falharCopia?Promise.reject(new Error('negado')):(window.__copias.push(texto),Promise.resolve())}});
 const executar=document.execCommand.bind(document);
 document.execCommand=(comando,...resto)=>comando==='copy'&&window.__falharCopia?false:executar(comando,...resto);
 window.__falas=[];
 document.addEventListener('DOMContentLoaded',()=>{const regiao=document.getElementById('live');if(regiao)new MutationObserver(()=>{const texto=regiao.textContent.replace(/\\u200B/g,'');if(texto)window.__falas.push(texto);}).observe(regiao,{childList:true,characterData:true,subtree:true});});
})();`;

const ESTADO=`(()=>{
 const q=seletor=>document.querySelector(seletor),saida=q('#output'),ativo=document.activeElement;
 const campo=(linha,coluna)=>{const entrada=document.getElementById('a-'+linha+'-'+coluna);return {valor:entrada.value,erro:entrada.parentElement.classList.contains('error'),invalido:entrada.getAttribute('aria-invalid'),mensagem:entrada.getAttribute('aria-errormessage'),dica:entrada.getAttribute('enterkeyhint'),visivel:!entrada.parentElement.hidden&&!entrada.closest('.equation').hidden};};
 const campos=[0,1,2].map(linha=>[0,1,2,3].map(coluna=>campo(linha,coluna)));
 const visivel=elemento=>!!elemento&&elemento.getClientRects().length>0&&getComputedStyle(elemento).display!=='none'&&getComputedStyle(elemento).visibility!=='hidden';
 return {
  ativo:ativo?.id||ativo?.tagName,
  selecao:ativo&&ativo.tagName==='INPUT'?[ativo.selectionStart,ativo.selectionEnd,ativo.value.length]:null,
  ajuda:q('#help').textContent,
  ajudaErro:q('#help').classList.contains('error'),
  titulo:q('#card-title')?.textContent??null,
  descricao:q('#card-description')?.textContent??null,
  classe:saida.className,
  esmaecido:saida.classList.contains('stale'),
  opacidade:q('#card-title')?getComputedStyle(q('#card-title')).opacity:null,
  opacidadeDescricao:q('#card-description')?getComputedStyle(q('#card-description')).opacity:null,
  valores:[...saida.querySelectorAll('.value .answer')].map(item=>item.textContent),
  geral:q('#output .general')?.textContent??null,
  previa:[...saida.querySelectorAll('.preview-line')].map(item=>item.textContent),
  previaOculta:q('#output .preview')?.getAttribute('aria-hidden')??null,
  falada:q('#card-spoken')?.textContent??null,
  descritoPor:saida.getAttribute('aria-describedby'),
  nota:q('#result-note').textContent,
  copiar:{desabilitado:q('#copy').getAttribute('aria-disabled'),titulo:q('#copy').title,descricao:document.getElementById(q('#copy').getAttribute('aria-describedby'))?.textContent,rotulo:q('#copy-label').textContent},
  pensar:{desabilitado:q('#think-open').getAttribute('aria-disabled'),titulo:q('#think-open').title,descricao:document.getElementById(q('#think-open').getAttribute('aria-describedby'))?.textContent},
  numeros:[...document.querySelectorAll('.number')].map(item=>item.classList.contains('complete')),
  numerosCor:[...document.querySelectorAll('.number')].map(item=>getComputedStyle(item).color),
  campos,
  operadores:[...document.querySelectorAll('.equation')][0]?[...[...document.querySelectorAll('.equation')][0].querySelectorAll('.operator')].filter(item=>!item.hidden).map(item=>item.textContent):[],
  fala:q('#live').textContent.replace(/\\u200B/g,''),
  falas:window.__falas.slice(),
  resumo:q('#summary-text').textContent,
  resumoClasse:q('#summary').className,
  rodapeDica:visivel(q('#footer-tip'))?q('#footer-tip').textContent:null,
  teclasVisiveis:visivel(q('#keys')),
  teclasPensarVisiveis:visivel(q('#think-keys')),
  linksVisiveis:visivel(q('#footer-links')),
  rodapePensar:visivel(q('#think-footer'))?q('#think-footer').textContent:null,
  pensarVisivel:!q('#think').hidden,
  solverVisivel:!q('#solver').hidden,
  statusPensar:q('#think-info').textContent,
  rotuloPassos:q('#copy-steps-label').textContent,
  metodoPressionado:[...document.querySelectorAll('#methods .segment')].filter(item=>item.getAttribute('aria-pressed')==='true').map(item=>item.id),
  copias:window.__copias.slice(),
  rolagem:scrollY
 };
})()`;

class Cdp{
 constructor(endereco){
  this.socket=new WebSocket(endereco);
  this.proximo=0;
  this.chamadas=new Map();
  this.ouvintes=[];
  this.pronto=new Promise((resolver,rejeitar)=>{this.socket.onopen=resolver;this.socket.onerror=()=>rejeitar(Error('não foi possível conectar ao Chrome pelo DevTools.'));});
  this.socket.onmessage=evento=>{
   const mensagem=JSON.parse(typeof evento.data==='string'?evento.data:Buffer.from(evento.data).toString('utf8'));
   if(mensagem.id===undefined){for(const ouvinte of this.ouvintes)ouvinte(mensagem);return;}
   const chamada=this.chamadas.get(mensagem.id);
   if(!chamada)return;
   this.chamadas.delete(mensagem.id);
   if(mensagem.error)chamada.rejeitar(Error(`${chamada.metodo}: ${mensagem.error.message}`));
   else chamada.resolver(mensagem.result);
  };
  this.socket.onclose=()=>{for(const chamada of this.chamadas.values())chamada.rejeitar(Error('a conexão com o Chrome foi encerrada.'));this.chamadas.clear();};
 }
 enviar(metodo,parametros={},sessao){
  const id=++this.proximo,mensagem={id,method:metodo,params:parametros};
  if(sessao)mensagem.sessionId=sessao;
  return new Promise((resolver,rejeitar)=>{this.chamadas.set(id,{resolver,rejeitar,metodo});this.socket.send(JSON.stringify(mensagem));});
 }
}

async function principal(){
 const navegador=acharChrome();
 if(!navegador){
  console.error(process.env.CI?'ERRO: Google Chrome não encontrado; a interface HTML precisa ser testada na integração contínua.':'AVISO: Google Chrome não encontrado; interface HTML não testada.');
  return SEM_CHROME;
 }
 if(!fs.existsSync(HTML))throw Error('SistemasLineares.html não encontrado: rode node tools/build-portable.cjs antes.');
 const perfil=fs.mkdtempSync(path.join(os.tmpdir(),'sistemas-lineares-chrome-'));
 const argumentos=['--headless=new','--remote-debugging-port=0',`--user-data-dir=${perfil}`,'--no-first-run','--no-default-browser-check','--disable-extensions','--disable-background-networking','--disable-component-update','--disable-sync','--disable-default-apps','--mute-audio','--window-size=1280,900','--force-device-scale-factor=1','about:blank'];
 if(process.platform==='linux'&&typeof process.getuid==='function'&&process.getuid()===0)argumentos.push('--no-sandbox');
 const processo=spawn(navegador,argumentos,{stdio:['ignore','ignore','pipe']});
 let cdp=null;
 const encerrar=()=>{try{cdp?.socket.close();}catch(_){}try{processo.kill();}catch(_){}};
 const relogio=setTimeout(()=>{console.error(`ERRO: tempo esgotado na etapa "${etapaAtual}".`);encerrar();process.exit(1);},PRAZO_TOTAL);
 try{
  const endereco=await new Promise((resolver,rejeitar)=>{
   let texto='';
   const prazo=setTimeout(()=>rejeitar(Error('o Chrome não abriu a porta do DevTools.')),30000);
   processo.stderr.on('data',parte=>{texto+=parte;const achado=/DevTools listening on (ws:\/\/\S+)/.exec(texto);if(achado){clearTimeout(prazo);resolver(achado[1]);}});
   processo.on('exit',codigo=>{clearTimeout(prazo);rejeitar(Error(`o Chrome encerrou antes de abrir o DevTools (código ${codigo}).`));});
  });
  cdp=new Cdp(endereco);
  await cdp.pronto;
  const {targetId}=await cdp.enviar('Target.createTarget',{url:'about:blank'});
  const {sessionId}=await cdp.enviar('Target.attachToTarget',{targetId,flatten:true});
  const pagina=(metodo,parametros)=>cdp.enviar(metodo,parametros,sessionId);
  let cargas=0;
  const erros=[];
  cdp.ouvintes.push(mensagem=>{
   if(mensagem.sessionId!==sessionId)return;
   if(mensagem.method==='Page.loadEventFired')cargas++;
   if(mensagem.method==='Runtime.exceptionThrown')erros.push(mensagem.params.exceptionDetails.exception?.description??mensagem.params.exceptionDetails.text);
   if(mensagem.method==='Runtime.consoleAPICalled'&&mensagem.params.type==='error')erros.push(mensagem.params.args.map(argumento=>argumento.value??argumento.description).join(' '));
  });
  await pagina('Page.enable');
  await pagina('Runtime.enable');
  await pagina('Page.addScriptToEvaluateOnNewDocument',{source:PREPARO});
  const esperar=async(condicao,descricao,prazo=15000)=>{const fim=Date.now()+prazo;while(!(await condicao())){if(Date.now()>fim)throw Error(`tempo esgotado ao ${descricao}`);await new Promise(resolver=>setTimeout(resolver,20));}};
  const avaliar=async expressao=>{
   const resposta=await pagina('Runtime.evaluate',{expression:expressao,returnByValue:true,awaitPromise:true});
   if(resposta.exceptionDetails)throw Error(`erro na página: ${resposta.exceptionDetails.exception?.description??resposta.exceptionDetails.text}`);
   return resposta.result.value;
  };
  const abrir=async()=>{const antes=cargas;await pagina('Page.navigate',{url:pathToFileURL(HTML).href});await esperar(async()=>cargas>antes,'carregar o SistemasLineares.html');await avaliar('document.fonts.ready.then(()=>true)');};
  const midia=async recursos=>pagina('Emulation.setEmulatedMedia',{media:'',features:recursos});
  const movimentoReduzido=[{name:'prefers-reduced-motion',value:'reduce'}];
  const estado=()=>avaliar(ESTADO);
  const avancar=ms=>avaliar(`window.__relogio.avancar(${ms})`);
  const TECLAS={Enter:[13,'Enter','\r'],Tab:[9,'Tab'],Escape:[27,'Escape'],ArrowUp:[38,'ArrowUp'],ArrowDown:[40,'ArrowDown'],Backspace:[8,'Backspace']};
  const tecla=async(nome,modificadores=0)=>{
   const [codigo,code,texto]=TECLAS[nome],base={key:nome,code,windowsVirtualKeyCode:codigo,nativeVirtualKeyCode:codigo,modifiers:modificadores};
   await pagina('Input.dispatchKeyEvent',{type:texto?'keyDown':'rawKeyDown',...base,...(texto?{text:texto,unmodifiedText:texto}:{})});
   await pagina('Input.dispatchKeyEvent',{type:'keyUp',...base});
  };
  const CODIGOS={'-':189,'/':191,'.':190,',':188,'^':54,'*':56,'(':57,')':48,' ':32,'+':187};
  const digitar=async texto=>{
   for(const caractere of texto){
    if(caractere.charCodeAt(0)>127){await pagina('Input.insertText',{text:caractere});continue;}
    const codigo=/[0-9]/.test(caractere)?caractere.charCodeAt(0):/[a-z]/i.test(caractere)?caractere.toUpperCase().charCodeAt(0):CODIGOS[caractere]??0;
    await pagina('Input.dispatchKeyEvent',{type:'keyDown',key:caractere,text:caractere,unmodifiedText:caractere,windowsVirtualKeyCode:codigo});
    await pagina('Input.dispatchKeyEvent',{type:'keyUp',key:caractere,windowsVirtualKeyCode:codigo});
   }
  };
  const ponto=(seletor,rolar=true)=>avaliar(`(()=>{const alvo=document.querySelector(${JSON.stringify(seletor)});if(!alvo)throw new Error('elemento não encontrado: ${seletor}');${rolar?"alvo.scrollIntoView({block:'center',inline:'center'});":''}const caixa=alvo.getBoundingClientRect();return {x:caixa.left+caixa.width/2,y:caixa.top+caixa.height/2};})()`);
  const pairar=async(seletor,rolar=true)=>{const {x,y}=await ponto(seletor,rolar);await pagina('Input.dispatchMouseEvent',{type:'mouseMoved',x,y});};
  const afastar=()=>pagina('Input.dispatchMouseEvent',{type:'mouseMoved',x:2,y:2});
  const clicar=async(seletor,rolar=true)=>{
   const {x,y}=await ponto(seletor,rolar);
   await pagina('Input.dispatchMouseEvent',{type:'mouseMoved',x,y});
   await pagina('Input.dispatchMouseEvent',{type:'mousePressed',x,y,button:'left',buttons:1,clickCount:1});
   await pagina('Input.dispatchMouseEvent',{type:'mouseReleased',x,y,button:'left',buttons:0,clickCount:1});
  };
  const focar=(linha,coluna,selecionar=true)=>avaliar(`(()=>{const entrada=document.getElementById('a-${linha}-${coluna}');entrada.focus();${selecionar?'entrada.select();':''}return true;})()`);
  const ultimaFala=async()=>{const falas=(await estado()).falas;return falas[falas.length-1]??null;};
  const etapa=async(nome,corpo)=>{
   etapaAtual=nome;
   try{await corpo();etapas++;}
   catch(erro){
    try{const imagem=await pagina('Page.captureScreenshot',{format:'png'});const arquivo=path.join(os.tmpdir(),'sistemas-lineares-falha.png');fs.writeFileSync(arquivo,Buffer.from(imagem.data,'base64'));erro.message+=`\n  captura da tela: ${arquivo}`;}catch(_){}
    erro.message=`etapa "${nome}": ${erro.message}`;
    throw erro;
   }
  };

  await pagina('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
  await midia(movimentoReduzido);
  await abrir();

  await etapa('estrutura e textos iniciais',async()=>{
   const estrutura=await avaliar(`(()=>{
    const q=seletor=>document.querySelector(seletor),todos=seletor=>[...document.querySelectorAll(seletor)];
    const entrada=q('#a-0-0'),links=todos('.footer-link');
    return {
     vivas:todos('[aria-live]').map(item=>item.id),
     atomica:q('#live').getAttribute('aria-atomic'),
     educada:q('#live').getAttribute('aria-live'),
     escondida:getComputedStyle(q('#live')).clipPath,
     status:todos('[role=status],[role=alert],[role=log]').length,
     entrada:{modo:entrada.getAttribute('inputmode'),maiusculas:entrada.getAttribute('autocapitalize'),enter:entrada.getAttribute('enterkeyhint'),nome:entrada.getAttribute('aria-label'),descricao:entrada.getAttribute('aria-describedby'),tipo:entrada.type},
     exemplo:{titulo:q('#example').title,descricao:document.getElementById(q('#example').getAttribute('aria-describedby')).textContent},
     limpar:{titulo:q('#clear').title,descricao:document.getElementById(q('#clear').getAttribute('aria-describedby')).textContent},
     tamanhos:['#size2','#size3'].map(seletor=>[q(seletor).getAttribute('aria-pressed'),q(seletor).getAttribute('aria-label'),q(seletor).textContent]),
     trilho:[getComputedStyle(q('.track')).borderTopLeftRadius,getComputedStyle(q('.track')).paddingTop,getComputedStyle(q('.track')).backgroundColor],
     selecionado:[getComputedStyle(q('#size2')).backgroundColor,getComputedStyle(q('#size2')).color],
     naoSelecionado:getComputedStyle(q('#size3')).backgroundColor,
     rotuloRodape:q('.footer-label').textContent,
     links:links.map(link=>({tipo:link.type,texto:link.textContent,nome:link.getAttribute('aria-label'),descricao:document.getElementById(link.getAttribute('aria-describedby')).textContent,sublinhado:getComputedStyle(link).textDecorationStyle,altura:link.getBoundingClientRect().height,cor:getComputedStyle(link).color})),
     teclas:todos('#keys .key').map(item=>[item.querySelector('kbd').textContent,item.querySelector('.key-action').textContent]),
     teclasPensar:todos('#think-keys .key').map(item=>[item.querySelector('kbd').textContent,item.querySelector('.key-action').textContent]),
     kbd:[getComputedStyle(q('kbd')).borderTopStyle,getComputedStyle(q('kbd')).backgroundColor],
     fonte:getComputedStyle(document.body).fontFamily,
     detalhes:q('details').textContent,
     subtitulo:q('#hero-solver p').textContent,
     campoBorda:getComputedStyle(q('.field')).borderTopColor
    };
   })()`);
   igual(estrutura.vivas,['live'],'uma única região aria-live, a #live');
   igual([estrutura.educada,estrutura.atomica],['polite','true'],'região viva educada e atômica');
   check(estrutura.escondida!=='none','região viva visualmente oculta');
   igual(estrutura.status,0,'nenhuma outra região viva (status, alert, log)');
   igual(estrutura.entrada,{modo:'text',maiusculas:'off',enter:'next',nome:'Equação 1, coeficiente de x',descricao:'field-description',tipo:'text'},'atributos do campo');
   igual(await avaliar(`document.getElementById('field-description').textContent`),FIELD_DESCRIPTION,'descrição acessível dos campos');
   igual(estrutura.exemplo,{titulo:TIP.example,descricao:TIP.example},'dica de Usar exemplo');
   igual(estrutura.limpar,{titulo:TIP.clear,descricao:TIP.clear},'dica de Limpar');
   igual(estrutura.tamanhos,[['true','Sistema 2 × 2, selecionado','2 × 2'],['false','Sistema 3 × 3','3 × 3']],'controle de tamanho');
   igual(estrutura.trilho,['13px','4px','rgb(29, 28, 28)'],'trilho do controle segmentado');
   igual(estrutura.selecionado,['rgb(206, 169, 243)','rgb(17, 16, 16)'],'pílula lilás selecionada');
   igual(estrutura.naoSelecionado,'rgba(0, 0, 0, 0)','segmento não selecionado transparente');
   igual(estrutura.rotuloRodape,'Passo a passo por','rótulo do rodapé');
   igual(estrutura.links.map(link=>[link.tipo,link.texto,link.nome,link.descricao,link.sublinhado]),[
    ['button','Gauss-Jordan','Passo a passo por Gauss-Jordan',METHOD_TIPS['gauss-jordan'],'dotted'],
    ['button','Cramer','Passo a passo pela regra de Cramer',METHOD_TIPS.cramer,'dotted'],
    ['button','Escalonamento','Passo a passo por escalonamento',METHOD_TIPS.gauss,'dotted']
   ],'links do rodapé');
   check(estrutura.links.every(link=>link.altura>=28),'links com alvo de pelo menos 28 px');
   check(estrutura.links.every(link=>link.cor==='rgb(243, 240, 247)'),'links na cor do texto');
   igual(estrutura.teclas,TEXTS.KEYS,'teclas do rodapé');
   igual(estrutura.teclasPensar,TEXTS.THINK_KEYS,'tecla do Modo pensar');
   igual(estrutura.kbd,['solid','rgb(29, 28, 28)'],'teclas desenhadas como keycaps');
   check(/^Space,\s*"Segoe UI"/.test(estrutura.fonte),`fonte com alternativa para ⁺⁻ depois da Space: ${estrutura.fonte}`);
   for(const trecho of ['AltGr+2','ABNT2','300 algarismos','(-2)^2 = 4','-1000 a 1000','ao vivo','1,5e3'])check(estrutura.detalhes.includes(trecho),`"Como o cálculo funciona" cita ${trecho}`);
   igual(estrutura.subtitulo,'Preencha os coeficientes: a solução aparece na hora. Frações, decimais e potências, sempre com resultado exato.','subtítulo');
   igual(estrutura.campoBorda,'rgb(53, 49, 55)','borda do campo em repouso');
   const inicial=await estado();
   igual(inicial.ajuda,TEXTS.DEFAULT_HELPER,'dica inicial');
   igual([inicial.titulo,inicial.descricao],[TEXTS.EMPTY_TITLE,TEXTS.EMPTY_DESCRIPTION],'card vazio');
   igual(inicial.copiar,{desabilitado:'true',titulo:TIP.disabled,descricao:TIP.disabled,rotulo:'Copiar'},'Copiar desabilitado');
   igual(inicial.pensar,{desabilitado:'true',titulo:TIP.disabled,descricao:TIP.disabled},'Modo pensar desabilitado');
   igual(inicial.nota,TEXTS.NOTE_DEFAULT,'nota inicial');
   check(inicial.teclasVisiveis&&!inicial.teclasPensarVisiveis&&inicial.linksVisiveis&&inicial.rodapeDica===null,'rodapé normal');
   igual(inicial.falas,[],'nada anunciado ao abrir');
  });

  await etapa('resolução ao vivo digitando o exemplo',async()=>{
   await clicar('#a-0-0');
   await pairar('#a-0-1');
   igual(await avaliar(`getComputedStyle(document.querySelector('#a-0-1').parentElement).borderTopColor`),'rgb(84, 78, 90)','borda do campo sob o mouse');
   igual(await avaliar(`[getComputedStyle(document.querySelector('#a-0-0').parentElement).borderTopColor,getComputedStyle(document.querySelector('#a-0-0').parentElement).boxShadow]`),['rgb(206, 169, 243)','rgba(206, 169, 243, 0.27) 0px 0px 0px 2px inset'],'campo com foco: borda lilás e halo interno');
   await afastar();
   await digitar('2');
   let atual=await estado();
   igual(atual.titulo,'Faltam 5 campos','progresso');
   igual(atual.previa,['2x + ?y = ?','?x + ?y = ?'],'prévia do sistema');
   igual(atual.previaOculta,'true','prévia escondida do leitor de tela');
   igual(atual.falada,'Faltam: equação 1, coeficiente de y; equação 1, termo independente; equação 2, coeficiente de x; equação 2, coeficiente de y; equação 2, termo independente.','prévia falada');
   igual(atual.descritoPor,'card-description card-spoken','descrição acessível do card de espera');
   igual(atual.descricao,TEXTS.MISSING_DESCRIPTION,'descrição do card de espera');
   await tecla('Enter');
   igual((await estado()).ativo,'a-0-1','Enter vai ao próximo campo vazio');
   await digitar('1');
   await tecla('Enter');
   await digitar('5');
   await tecla('Enter');
   igual((await estado()).ativo,'a-1-0','Enter segue a ordem de leitura');
   await digitar('1');
   await tecla('Enter');
   await digitar('-');
   atual=await estado();
   igual([atual.ativo,atual.titulo,atual.ajudaErro],['a-1-1','Faltam 2 campos',false],'sinal sozinho conta como campo em digitação');
   igual(atual.numeros,[true,false,false],'linha completa fica lilás');
   igual(atual.numerosCor.slice(0,2),['rgb(206, 169, 243)','rgb(170, 165, 177)'],'cor do número da linha');
   await digitar('1');
   igual((await estado()).titulo,'Falta 1 campo','falta 1 campo');
   await tecla('Enter');
   atual=await estado();
   igual(atual.ativo,'a-1-3','Enter no último vazio');
   check(atual.campos[0][0].dica==='next','enterkeyhint next enquanto falta campo');
   await tecla('Enter');
   atual=await estado();
   igual([atual.ativo,atual.ajuda,atual.ajudaErro],['a-1-3',TEXTS.ONLY_THIS_FIELD,false],'Enter no único campo vazio avisa sem mover');
   igual(atual.falas,[TEXTS.ONLY_THIS_FIELD],'aviso do único campo anunciado');
   await digitar('1');
   atual=await estado();
   igual([atual.titulo,atual.valores,atual.esmaecido],['Solução encontrada',['2','1'],false],'resultado na hora ao completar');
   igual(atual.numeros,[true,true,false],'as duas linhas completas');
   check(atual.campos.flat().filter(campo=>campo.visivel).every(campo=>campo.dica==='done'),'enterkeyhint done com tudo preenchido');
   igual(atual.copiar,{desabilitado:null,titulo:TIP.copy,descricao:TIP.copy,rotulo:'Copiar'},'Copiar habilitado com dica');
   igual(atual.pensar,{desabilitado:null,titulo:TIP.think,descricao:TIP.think},'Modo pensar habilitado com dica');
   igual(atual.falas,[TEXTS.ONLY_THIS_FIELD],'digitação não anuncia na hora');
   await avancar(699);
   igual((await estado()).falas,[TEXTS.ONLY_THIS_FIELD],'nada anunciado antes de 700 ms');
   await avancar(1);
   igual((await estado()).falas,[TEXTS.ONLY_THIS_FIELD,'Solução encontrada: x = 2, y = 1.'],'anúncio após 700 ms parado');
   await tecla('Enter');
   atual=await estado();
   igual(atual.falas,[TEXTS.ONLY_THIS_FIELD,'Solução encontrada: x = 2, y = 1.','Solução encontrada: x = 2, y = 1.'],'Enter com tudo preenchido anuncia na hora, sem deduplicar');
   igual(atual.ativo,'a-1-3','Enter que resolve não move o foco');
  });

  await etapa('setas, resultado esmaecido e deduplicação',async()=>{
   await tecla('ArrowUp');
   let atual=await estado();
   igual([atual.ativo,atual.selecao],['a-0-3',[0,1,1]],'↑ vai à mesma coluna da equação de cima e seleciona');
   await tecla('ArrowUp');
   atual=await estado();
   igual([atual.ativo,atual.selecao],['a-0-3',[0,1,1]],'↑ na primeira equação é consumida');
   await digitar('-');
   atual=await estado();
   igual([atual.esmaecido,atual.opacidade,atual.opacidadeDescricao,atual.titulo,atual.ajudaErro],[true,'0.7','1','Solução encontrada',false],'"-" mantém o último resultado esmaecido, com a descrição legível');
   igual(atual.nota,TEXTS.NOTE_STALE,'nota de atualização');
   igual(atual.copiar,{desabilitado:'true',titulo:TIP.disabled,descricao:TIP.disabled,rotulo:'Copiar'},'Copiar desabilitado no esmaecido');
   igual(atual.pensar,{desabilitado:'true',titulo:TIP.disabled,descricao:TIP.disabled},'Modo pensar desabilitado no esmaecido');
   await digitar('3');
   atual=await estado();
   igual([atual.esmaecido,atual.valores,atual.opacidade],[false,['-2/3','-5/3'],'1'],'"-3" volta ao resultado');
   await avancar(700);
   igual(await ultimaFala(),'Solução encontrada: x = -2/3, y = -5/3.','novo resultado anunciado');
   const quantas=(await estado()).falas.length;
   await tecla('Backspace');
   await tecla('Backspace');
   igual((await estado()).esmaecido,true,'apagar esmaece');
   await digitar('-3');
   await avancar(700);
   igual((await estado()).falas.length,quantas,'o mesmo texto não é anunciado de novo');
   await tecla('ArrowDown');
   atual=await estado();
   igual([atual.ativo,atual.selecao],['a-1-3',[0,1,1]],'↓ vai à equação de baixo');
   await tecla('ArrowDown');
   igual((await estado()).ativo,'a-1-3','↓ na última equação é consumida');
   await tecla('ArrowUp',8);
   igual((await estado()).ativo,'a-1-3','Shift+↑ segue o comportamento normal');
  });

  await etapa('erro revelado por campo',async()=>{
   await clicar('#clear');
   let atual=await estado();
   igual([atual.ativo,atual.titulo],['a-0-0',TEXTS.EMPTY_TITLE],'Limpar foca o primeiro campo');
   check(atual.campos.flat().every(campo=>campo.valor===''),'Limpar esvazia tudo');
   const falas=atual.falas.length;
   await digitar('2x');
   atual=await estado();
   igual([atual.ajuda,atual.campos[0][0].erro,atual.campos[0][0].invalido,atual.titulo],[TEXTS.DEFAULT_HELPER,false,null,'Faltam 5 campos · 1 inválido'],'o erro espera a saída do campo');
   await tecla('Tab');
   atual=await estado();
   igual(atual.ativo,'a-0-1','Tab segue para o próximo campo');
   igual([atual.campos[0][0].erro,atual.campos[0][0].invalido,atual.campos[0][0].mensagem],[true,'true','help'],'erro revelado ao sair do campo');
   igual([atual.ajuda,atual.ajudaErro],['Equação 1, coeficiente de x: Número inválido.',true],'dica vermelha');
   igual(atual.falas.slice(falas),['Equação 1, coeficiente de x: Número inválido.'],'erro anunciado uma vez');
   igual(await avaliar(`getComputedStyle(document.querySelector('#a-0-0').parentElement).borderTopColor`),'rgb(255, 153, 145)','borda de erro');
   igual(await avaliar(`[document.querySelector('#a-0-0').getAttribute('aria-describedby'),document.querySelector('#a-0-1').getAttribute('aria-describedby')]`),['help','field-description'],'campo com erro descrito pela dica');
   await digitar('5');
   atual=await estado();
   check(atual.campos[0][0].erro,'editar outro campo não apaga o erro');
   igual(atual.falas.length,falas+1,'o erro não é anunciado de novo');
   await focar(0,0);
   await digitar('1/0');
   atual=await estado();
   igual([atual.campos[0][0].erro,atual.campos[0][0].invalido],[false,null],'o erro some quando o próprio campo muda');
   await avancar(699);
   igual((await estado()).campos[0][0].erro,false,'nada revelado antes de 700 ms');
   await avancar(1);
   atual=await estado();
   igual([atual.campos[0][0].erro,atual.ajuda],[true,'Equação 1, coeficiente de x: O denominador não pode ser zero.'],'pausa de 700 ms revela o erro');
   igual(await ultimaFala(),'Equação 1, coeficiente de x: O denominador não pode ser zero.','erro da pausa anunciado');
   await focar(1,1);
   await digitar('1/');
   await avancar(700);
   igual((await estado()).campos[1][1].erro,false,'prefixo com foco não é revelado na pausa');
   await tecla('Tab');
   atual=await estado();
   igual([atual.ativo,atual.campos[1][1].erro],['a-1-3',true],'sair revela o prefixo');
   igual(atual.ajuda,'Equação 1, coeficiente de x: O denominador não pode ser zero.','dica mostra o primeiro erro na ordem de leitura');
   igual(await ultimaFala(),'Equação 2, coeficiente de y: Número inválido.','anuncia o erro recém-revelado');
   await focar(1,1);
   igual((await estado()).ajuda,'Equação 2, coeficiente de y: Número inválido.','dica mostra o erro do campo focado');
   await avaliar(`(()=>{const entrada=document.getElementById('a-0-0');entrada.focus();entrada.setSelectionRange(3,3);return true;})()`);
   await tecla('Backspace');
   atual=await estado();
   igual([atual.campos[0][0].valor,atual.campos[0][0].erro,atual.ajuda],['1/',true,'Equação 1, coeficiente de x: Número inválido.'],'corrigindo, o erro revelado não pisca');
   await digitar('2');
   atual=await estado();
   igual([atual.campos[0][0].valor,atual.campos[0][0].erro],['1/2',false],'ficou válido: o erro some na hora');
  });

  await etapa('Usar exemplo e IME',async()=>{
   await clicar('#example');
   let atual=await estado();
   igual([atual.ativo,atual.valores,atual.ajuda],['example',['2','1'],TEXTS.EXAMPLE_HELPER],'exemplo resolvido sem mudar o foco');
   igual(await ultimaFala(),`${TEXTS.EXAMPLE_HELPER} Solução encontrada: x = 2, y = 1.`,'anúncio único do exemplo');
   await avaliar(`(()=>{const entrada=document.getElementById('a-0-3');entrada.focus();entrada.setSelectionRange(1,1);return true;})()`);
   await pagina('Input.imeSetComposition',{text:'^',selectionStart:1,selectionEnd:1});
   atual=await estado();
   igual([atual.campos[0][3].valor,atual.esmaecido,atual.titulo],['5^',false,'Solução encontrada'],'composição em andamento não recalcula');
   await pagina('Input.insertText',{text:'^'});
   atual=await estado();
   igual([atual.campos[0][3].valor,atual.esmaecido],['5^',true],'composição confirmada recalcula');
   await digitar('1');
   atual=await estado();
   igual([atual.campos[0][3].valor,atual.esmaecido,atual.valores],['5^1',false,['2','1']],'potência digitada depois da composição');
   igual([atual.ajuda,await avaliar(`document.getElementById('help').className`),await avaliar(`getComputedStyle(document.getElementById('help')).color`)],['Este campo vale 5.','reading','rgb(243, 240, 247)'],'dica mostra o valor lido da potência');
   await tecla('Enter');
   igual((await estado()).ativo,'a-0-3','Enter fora da composição resolve');
  });

  await etapa('2×2 e 3×3',async()=>{
   await clicar('#size3');
   let atual=await estado();
   igual(await ultimaFala(),'3 equações · 3 incógnitas. Aguardando os valores.','anúncio da troca para 3×3');
   igual([atual.titulo,atual.operadores],[TEXTS.EMPTY_TITLE,['+','+','=']],'grade 3×3 com o próprio sistema');
   check(atual.campos.flat().every(campo=>campo.valor===''),'o 3×3 não herda o 2×2');
   check(atual.campos[2][0].visivel&&atual.campos[0][2].visivel,'terceira equação e coluna z visíveis');
   igual(await avaliar(`[document.querySelector('#size3').getAttribute('aria-pressed'),document.querySelector('#size3').getAttribute('aria-label')]`),['true','Sistema 3 × 3, selecionado'],'3×3 selecionado');
   await clicar('#example');
   igual((await estado()).valores,['1','2','3'],'exemplo 3×3');
   await clicar('#size2');
   atual=await estado();
   igual([atual.valores,atual.operadores,atual.campos[0][3].valor],[['2','1'],['+','='],'5^1'],'o 2×2 volta como estava');
   igual(await ultimaFala(),'2 equações · 2 incógnitas. Solução encontrada: x = 2, y = 1.','anúncio da troca para 2×2');
   check(!atual.campos[2][0].visivel&&!atual.campos[0][2].visivel,'terceira equação oculta');
   await clicar('#size3');
   igual((await estado()).valores,['1','2','3'],'o 3×3 volta resolvido');
   await clicar('#size2');
  });

  await etapa('Resolver sistema com campos vazios',async()=>{
   await clicar('#clear');
   await clicar('#solve');
   const atual=await estado();
   igual([atual.ativo,atual.ajuda,atual.ajudaErro,atual.campos[0][0].erro],['a-0-0','Equação 1, coeficiente de x: Preencha todos os campos.',true,true],'Resolver aponta o primeiro vazio');
   igual(await ultimaFala(),'Equação 1, coeficiente de x: Preencha todos os campos.','erro anunciado');
  });

  await etapa('links do rodapé',async()=>{
   await pairar('#link-cramer');
   let atual=await estado();
   igual([atual.rodapeDica,atual.teclasVisiveis],[METHOD_TIPS.cramer,false],'a dica do método substitui as teclas');
   igual(await avaliar(`[getComputedStyle(document.querySelector('#link-cramer')).textDecorationStyle,getComputedStyle(document.querySelector('#link-cramer')).color]`),['solid','rgb(206, 169, 243)'],'sublinhado sólido sob o mouse');
   await afastar();
   atual=await estado();
   igual([atual.rodapeDica,atual.teclasVisiveis],[null,true],'as teclas voltam');
   await avaliar(`document.querySelector('#link-gauss').focus()`);
   igual((await estado()).rodapeDica,METHOD_TIPS.gauss,'foco no link mostra a dica');
   await avaliar(`document.querySelector('#link-gauss').blur()`);
   await clicar('#clear');
   await digitar('2');
   await clicar('#link-cramer');
   atual=await estado();
   igual([atual.ajuda,atual.ajudaErro,atual.ativo],[TEXTS.METHOD_NEEDS_SYSTEM.cramer,false,'a-0-1'],'link sem sistema completo');
   igual(await ultimaFala(),TEXTS.METHOD_NEEDS_SYSTEM.cramer,'aviso do link anunciado');
   await clicar('#example');
   await focar(0,0);
   await digitar('abc');
   await clicar('#link-gauss');
   atual=await estado();
   igual([atual.ativo,atual.ajuda,atual.ajudaErro],['a-0-0','Equação 1, coeficiente de x: Número inválido.',true],'link com campo inválido');
   await focar(0,0);
   await digitar('2');
   await clicar('#link-gauss');
   atual=await estado();
   check(atual.pensarVisivel&&!atual.solverVisivel,'o link abre o Modo pensar');
   check(atual.statusPensar.startsWith('Escalonamento de Gauss · '),atual.statusPensar);
   igual(atual.metodoPressionado,['method-gauss'],'aba do método');
   igual([atual.linksVisiveis,atual.rodapePensar,atual.teclasVisiveis,atual.teclasPensarVisiveis],[false,TEXTS.THINK_FOOTER,false,true],'rodapé do Modo pensar');
   igual(atual.ativo,'think-title','foco no título do passo a passo');
   await tecla('Escape');
   atual=await estado();
   igual([atual.pensarVisivel,atual.ativo,atual.linksVisiveis],[false,'link-gauss',true],'Esc devolve o foco ao link');
  });

  await etapa('Modo pensar, Esc e copiar passos',async()=>{
   await clicar('#think-open');
   let atual=await estado();
   igual(atual.metodoPressionado,['method-gauss'],'método lembrado');
   await tecla('Escape');
   igual((await estado()).ativo,'think-open','Esc devolve o foco ao botão Modo pensar');
   await clicar('#think-open');
   await clicar('#method-cramer');
   atual=await estado();
   igual([atual.statusPensar,atual.metodoPressionado],['Regra de Cramer · 6 passos',['method-cramer']],'troca de método');
   await clicar('#copy-steps');
   await esperar(async()=>(await estado()).rotuloPassos===TEXTS.COPIED_STEPS_LABEL,'copiar os passos');
   const exemplo=[['2','1','5'],['1','-1','1']].map(linha=>linha.map(solver.Rational.parse));
   const esperado=solver.walkthroughText(solver.walkthrough(exemplo,'cramer'));
   atual=await estado();
   igual(atual.copias[atual.copias.length-1],esperado,'texto dos passos copiado');
   await avaliar('window.__falharCopia=true');
   await clicar('#method-gauss');
   igual((await estado()).rotuloPassos,TEXTS.COPY_STEPS_LABEL,'rótulo volta ao trocar de método');
   await clicar('#copy-steps');
   await esperar(async()=>(await estado()).statusPensar.includes(TEXTS.COPY_STEPS_FAILED),'mostrar a falha ao copiar');
   igual((await estado()).rotuloPassos,TEXTS.COPY_STEPS_LABEL,'rótulo depois da falha');
   await avaliar('window.__falharCopia=false');
   await clicar('#back');
   atual=await estado();
   igual([atual.pensarVisivel,atual.ativo],[false,'think-open'],'Voltar devolve o foco');
   await clicar('#copy');
   await esperar(async()=>(await estado()).copiar.rotulo===TEXTS.COPIED_LABEL,'copiar o resultado');
   atual=await estado();
   igual(atual.copias[atual.copias.length-1],'Solução única\n\nx = 2\ny = 1','resposta copiada');
  });

  await etapa('valores longos numa linha',async()=>{
   await clicar('#example');
   await focar(0,3);
   await digitar('10^200');
   const medida=async()=>avaliar(`(()=>{const resposta=document.querySelector('#output .value .answer'),caixa=document.querySelector('#output .value').getBoundingClientRect();return {altura:caixa.height,cabe:resposta.scrollWidth<=resposta.clientWidth+1,foco:resposta.getAttribute('tabindex'),algarismos:resposta.textContent.length};})()`);
   await esperar(async()=>(await medida()).foco==='0','tornar o valor longo rolável pelo teclado');
   const longo=await medida();
   check(!longo.cabe&&longo.algarismos>=200,`valor longo maior que o cartão (${longo.algarismos} caracteres)`);
   check(longo.altura<80,`o valor longo fica numa linha (${longo.altura} px)`);
   await focar(0,3);
   await digitar('5');
   await esperar(async()=>(await medida()).foco===null,'voltar ao valor curto');
  });

  await etapa('movimento',async()=>{
   await midia([{name:'prefers-reduced-motion',value:'no-preference'}]);
   await clicar('#clear');
   await clicar('#example');
   igual(await avaliar(`getComputedStyle(document.getElementById('card-title')).transitionDuration`),'0.15s','transição de 150 ms com movimento liberado');
   igual(await avaliar(`[document.getElementById('output').classList.contains('appear'),getComputedStyle(document.getElementById('output')).animationName,getComputedStyle(document.getElementById('output')).animationDuration]`),[true,'appear','0.15s'],'resultado aparece com fade de 150 ms');
   await midia(movimentoReduzido);
   igual(await avaliar(`[getComputedStyle(document.getElementById('card-title')).transitionDuration,getComputedStyle(document.getElementById('output')).animationName]`),['0s','none'],'sem animação com movimento reduzido');
   await clicar('#clear');
   await clicar('#example');
   igual(await avaliar(`document.getElementById('output').classList.contains('appear')`),false,'sem fade com movimento reduzido');
  });

  await etapa('alto contraste',async()=>{
   await clicar('#clear');
   await clicar('#solve');
   const normal=await avaliar(`[getComputedStyle(document.querySelector('#a-0-0').parentElement).borderTopStyle,getComputedStyle(document.querySelector('#size2')).backgroundColor]`);
   igual(normal,['solid','rgb(206, 169, 243)'],'cores normais');
   await midia([...movimentoReduzido,{name:'forced-colors',value:'active'}]);
   const forcado=await avaliar(`(()=>{
    const sonda=document.createElement('div');
    sonda.style.cssText='forced-color-adjust:none;background-color:Highlight;color:HighlightText';
    document.body.append(sonda);
    const realce=getComputedStyle(sonda).backgroundColor,texto=getComputedStyle(sonda).color;
    sonda.remove();
    const campo=getComputedStyle(document.querySelector('#a-0-0').parentElement),botao=getComputedStyle(document.querySelector('#size2'));
    return {ativo:matchMedia('(forced-colors: active)').matches,realce,texto,borda:[campo.borderTopStyle,campo.borderTopWidth],fundo:botao.backgroundColor,cor:botao.color,contorno:[campo.outlineStyle,campo.outlineColor]};
   })()`);
   check(forcado.ativo,'alto contraste emulado');
   igual(forcado.borda,['dashed','2px'],'erro tracejado de 2 px no alto contraste');
   igual([forcado.fundo,forcado.cor],[forcado.realce,forcado.texto],'aria-pressed usa Highlight e HighlightText');
   igual(forcado.contorno,['solid',forcado.realce],'foco do campo com contorno Highlight');
   await midia(movimentoReduzido);
  });

  await etapa('celular 360×740',async()=>{
   await pagina('Emulation.setDeviceMetricsOverride',{width:360,height:740,deviceScaleFactor:2,mobile:true});
   await pagina('Emulation.setTouchEmulationEnabled',{enabled:true,maxTouchPoints:5});
   await abrir();
   const medir=()=>avaliar(`(()=>{const barra=document.getElementById('summary').getBoundingClientRect(),cartao=document.getElementById('solution').getBoundingClientRect();return {largura:innerWidth,altura:innerHeight,rolagemLateral:document.documentElement.scrollWidth,barra:[barra.top,barra.bottom,barra.left,barra.right],exibida:getComputedStyle(document.getElementById('summary')).display,posicao:getComputedStyle(document.getElementById('summary')).position,cartao:[cartao.top,cartao.bottom],teclas:getComputedStyle(document.getElementById('keys')).display,toque:matchMedia('(hover: none) and (pointer: coarse)').matches};})()`);
   let medida=await medir();
   igual([medida.largura,medida.exibida,medida.posicao],[360,'flex','sticky'],'barra de resumo no celular');
   check(medida.rolagemLateral<=360,`sem rolagem horizontal (${medida.rolagemLateral} px)`);
   check(medida.barra[1]<=medida.altura&&medida.barra[0]>=medida.altura-120,`barra de resumo perto da base: ${JSON.stringify(medida.barra)}`);
   check(medida.barra[2]>=0&&medida.barra[3]<=360,'barra de resumo dentro da largura');
   check(medida.cartao[0]>=medida.altura,'o cartão de resultado começa abaixo da dobra');
   check(medida.toque&&medida.teclas==='none','teclas escondidas em tela de toque');
   igual((await estado()).resumo,TEXTS.EMPTY_TITLE,'resumo inicial');
   await clicar('#example');
   igual((await estado()).resumo,'x = 2 · y = 1','resumo com o resultado');
   await avaliar('scrollTo(0,0)');
   await focar(0,0);
   const antes=(await estado()).rolagem;
   await digitar('-');
   let atual=await estado();
   check(atual.resumoClasse.includes('stale'),'resumo esmaecido');
   igual(atual.rolagem,antes,'atualização ao vivo não rola a página');
   await avancar(700);
   igual((await estado()).resumo,'Falta 1 campo','resumo do progresso');
   await digitar('3');
   atual=await estado();
   igual([atual.resumo,atual.rolagem],['x = -3 · y = -4',antes],'resumo atualizado sem rolar');
   await clicar('#summary-go',false);
   await esperar(async()=>(await estado()).ativo==='solution','focar o resultado');
   medida=await medir();
   check(medida.cartao[0]>=-1&&medida.cartao[0]<medida.altura/2,`Ver resultado leva ao cartão: ${JSON.stringify(medida.cartao)}`);
   await avaliar('scrollTo(0,0)');
   await clicar('#size3');
   medida=await medir();
   check(medida.rolagemLateral<=360,`3×3 sem rolagem horizontal (${medida.rolagemLateral} px)`);
   await pagina('Emulation.setTouchEmulationEnabled',{enabled:false});
   await pagina('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
  });

  igual(erros,[],'a página não registrou erros de JavaScript');
  console.log(`PASS: interface HTML no Chrome: ${etapas} etapas, ${checks} verificações`);
  return 0;
 }finally{
  clearTimeout(relogio);
  encerrar();
  await new Promise(resolver=>{if(processo.exitCode!==null)resolver();else{processo.once('exit',resolver);setTimeout(resolver,5000);}});
  try{fs.rmSync(perfil,{recursive:true,force:true,maxRetries:10,retryDelay:200});}catch(_){}
 }
}

principal().then(codigo=>{process.exitCode=codigo;},erro=>{console.error(`ERRO: ${erro.message}`);process.exitCode=1;});
