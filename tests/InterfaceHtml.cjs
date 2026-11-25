const fs=require('node:fs');
const os=require('node:os');
const path=require('node:path');
const zlib=require('node:zlib');
const assert=require('node:assert/strict');
const {spawn,execFileSync}=require('node:child_process');
const {pathToFileURL}=require('node:url');
const solver=require('../portable/solver.js');
const {TEXTS,Model,inputColumn}=require('../portable/app.js');
const G=require('../portable/graph.js');
const HTML=path.join(__dirname,'..','SistemasLineares.html');
const SEM_CHROME=3,PRAZO_TOTAL=240000;
const METHOD_TIPS={'gauss-jordan':'Gauss-Jordan: operações de linha até a matriz identidade.',cramer:'Regra de Cramer: determinantes, com Sarrus no 3×3.',gauss:'Escalonamento: zera abaixo dos pivôs e resolve de baixo para cima.'};
const FIELD_DESCRIPTION='Aceita 2 · -1,5 · 3/4 · 2^3 · 10². Use 0 se a variável não aparecer. Enter vai ao próximo campo vazio; as setas para cima e para baixo mudam de equação.';
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
 const tempoReal=performance.now.bind(performance);
 let desvio=0,parado=null;
 performance.now=()=>parado??tempoReal()+desvio;
 window.__tempo={parar(){parado=performance.now();return parado;},ir(ms){parado=ms;},soltar(){desvio=parado-tempoReal();parado=null;}};
 const pedirQuadro=window.requestAnimationFrame.bind(window),cancelarQuadro=window.cancelAnimationFrame.bind(window),quadros=new Set();
 window.__quadros={pedidos:0,pendentes:()=>quadros.size};
 window.requestAnimationFrame=funcao=>{window.__quadros.pedidos++;const id=pedirQuadro(tempo=>{quadros.delete(id);funcao(tempo);});quadros.add(id);return id;};
 window.cancelAnimationFrame=id=>{quadros.delete(id);cancelarQuadro(id);};
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

const GRAFICO=`(()=>{
 const svg=document.getElementById('plot'),camada=svg.querySelector('g'),retangulo=elemento=>{const caixa=elemento.getBoundingClientRect();return {esquerda:caixa.left,topo:caixa.top,direita:caixa.right,base:caixa.bottom,largura:caixa.width,altura:caixa.height};};
 const regua=document.createElement('canvas').getContext('2d');
 regua.font='12px '+getComputedStyle(svg).fontFamily;
 const itens=[...camada.children].map(no=>{const atributos={};for(const atributo of no.attributes)atributos[atributo.name]=atributo.value;const texto=no.localName==='text'&&no.getNumberOfChars()>0;return {tag:no.localName,atributos,texto:no.textContent,caixa:texto?{x:no.getStartPositionOfChar(0).x,largura:no.getComputedTextLength(),medida:regua.measureText(no.textContent).width}:null};});
 return {
  itens,
  assinatura:itens.map(item=>item.tag+'|'+Object.keys(item.atributos).sort().map(nome=>nome+'='+item.atributos[nome]).join(' ')+'|'+item.texto).join('\\n'),
  plot:retangulo(svg),
  figura:retangulo(document.getElementById('graph')),
  legenda:retangulo(document.getElementById('help')),
  acoes:retangulo(document.querySelector('.actions')),
  cartao:retangulo(document.querySelector('.entry')),
  tres:document.getElementById('graph').classList.contains('three'),
  pendentes:window.__quadros.pendentes(),
  pedidos:window.__quadros.pedidos,
  ajuste:getComputedStyle(svg).forcedColorAdjust,
  algarismos:regua.measureText('0123456789').actualBoundingBoxAscent,
  texturas:[...svg.querySelectorAll('pattern')].map(no=>({id:no.id,largura:no.getAttribute('width'),altura:no.getAttribute('height'),raio:no.firstChild.getAttribute('r'),cor:no.firstChild.getAttribute('fill'),opacidade:no.firstChild.getAttribute('fill-opacity')}))
 };
})()`;
const retasDoGrafico=grafico=>grafico.itens.filter(item=>item.tag==='line'&&item.atributos['stroke-width']==='2');
const planosDoGrafico=grafico=>grafico.itens.filter(item=>item.tag==='polygon'&&item.atributos['fill-opacity']==='0.26');
const pontoDoGrafico=grafico=>grafico.itens.filter(item=>item.tag==='circle'&&item.atributos.r==='4.25');
const textosDoGrafico=grafico=>grafico.itens.filter(item=>item.tag==='text').map(item=>item.texto);
const comprimento=item=>Math.hypot(Number(item.atributos.x2)-Number(item.atributos.x1),Number(item.atributos.y2)-Number(item.atributos.y1));
const esbocoDe=(dimensao,linhas)=>{const modelo=new Model();modelo.changeDimension(dimensao);if(linhas)linhas.forEach((valores,linha)=>valores.forEach((texto,coluna)=>modelo.edit(linha,inputColumn(dimensao,coluna),texto)));else modelo.fillExample();return modelo.sketch();};
const emRepouso=(esboco,grafico,contraste=false)=>{const motion=new G.Motion();motion.update(esboco,0,false);return motion.scene(1e9,grafico.plot.largura,grafico.plot.altura,1,contraste);};
const TAG_DA_FORMA={dots:'rect',segment:'line',polygon:'polygon',outline:'polygon',disc:'circle',ring:'circle',text:'text'};
const PONTAS_REDONDAS=new Set(['r0','r1','r2','m','u','s']);
const perto=(texto,valor,folga=0.006)=>texto!==undefined&&Math.abs(Number(texto)-valor)<=folga;
const mesmosPontos=(texto,pontos)=>{const lidos=String(texto).trim().split(/\s+/).map(par=>par.split(',').map(Number));return lidos.length===pontos.length&&lidos.every(([x,y],indice)=>Math.abs(x-pontos[indice].x)<=0.006&&Math.abs(y-pontos[indice].y)<=0.006);};
function conferirDesenho(grafico,itens,contraste,descricao){
 igual(grafico.itens.map(item=>item.tag),itens.map(item=>TAG_DA_FORMA[item.shape.kind]),`${descricao}: um elemento do SVG para cada item da cena, na mesma ordem`);
 const largura=grafico.plot.largura,altura=grafico.plot.altura;
 let textura=0;
 itens.forEach((item,indice)=>{
  const no=grafico.itens[indice],a=no.atributos,forma=item.shape,cor=G.inkColor(item.ink,contraste),onde=`${descricao}, item ${indice} (${item.ink} ${forma.kind})`;
  if(forma.kind==='dots'){
   const padrao=grafico.texturas[textura++];
   check(!!padrao&&a.fill===`url(#${padrao.id})`,`${onde}: textura desenhada pelo seu padrão`);
   check(perto(padrao.largura,forma.spacing)&&perto(padrao.altura,forma.spacing)&&perto(padrao.raio,forma.radius),`${onde}: espaçamento e raio dos pontinhos`);
   check(padrao.cor===cor&&perto(padrao.opacidade,item.alpha),`${onde}: pontinhos ${padrao.cor} com opacidade ${padrao.opacidade}, esperado ${cor} com ${item.alpha}`);
   for(const [lado,limite] of [[Number(a.width),largura],[Number(a.height),altura]]){
    const ultimo=lado-forma.spacing/2;
    check(a.x==='0'&&a.y==='0'&&Math.abs(lado/forma.spacing-Math.round(lado/forma.spacing))<1e-6&&ultimo+forma.radius<=limite+1e-6&&ultimo+forma.spacing+forma.radius>limite,`${onde}: só pontinhos inteiros, até a borda da área (${lado} px de ${limite} px)`);
   }
  }else if(forma.kind==='segment'){
   check(perto(a.x1,forma.from.x)&&perto(a.y1,forma.from.y)&&perto(a.x2,forma.to.x)&&perto(a.y2,forma.to.y)&&perto(a['stroke-width'],forma.width),`${onde}: posição e espessura`);
   igual([a.stroke,a['stroke-linecap']],[cor,PONTAS_REDONDAS.has(item.ink)?'round':'butt'],`${onde}: cor e pontas`);
   check(perto(a['stroke-opacity'],item.alpha),`${onde}: opacidade ${a['stroke-opacity']}, esperado ${item.alpha}`);
  }else if(forma.kind==='polygon'){
   check(mesmosPontos(a.points,forma.points),`${onde}: vértices`);
   check(a.fill===cor&&a.stroke==='none'&&perto(a['fill-opacity'],item.alpha),`${onde}: preenchimento ${a.fill} com opacidade ${a['fill-opacity']}, esperado ${cor} com ${item.alpha}`);
  }else if(forma.kind==='outline'){
   check(mesmosPontos(a.points,forma.points)&&perto(a['stroke-width'],forma.width),`${onde}: contorno`);
   check(a.fill==='none'&&a.stroke===cor&&perto(a['stroke-opacity'],item.alpha),`${onde}: contorno ${a.stroke} com opacidade ${a['stroke-opacity']}, esperado ${cor} com ${item.alpha}`);
  }else if(forma.kind==='disc'||forma.kind==='ring'){
   const anel=forma.kind==='ring';
   check(perto(a.cx,forma.center.x)&&perto(a.cy,forma.center.y)&&perto(a.r,forma.radius)&&(!anel||perto(a['stroke-width'],forma.width)),`${onde}: centro e raio`);
   check(anel?a.fill==='none'&&a.stroke===cor&&perto(a['stroke-opacity'],item.alpha):a.fill===cor&&perto(a['fill-opacity'],item.alpha),`${onde}: cor ${anel?a.stroke:a.fill} com opacidade ${a[anel?'stroke-opacity':'fill-opacity']}, esperado ${cor} com ${item.alpha}`);
  }else{
   igual(no.texto,forma.text,`${onde}: texto`);
   check(a.fill===cor&&perto(a['fill-opacity'],item.alpha),`${onde}: "${forma.text}" em ${a.fill} com opacidade ${a['fill-opacity']}, esperado ${cor} com ${item.alpha}`);
   const largo=no.caixa.medida,alto=G.TEXT_HEIGHT;
   const esquerda=forma.horizontal==='s'?forma.at.x:forma.horizontal==='m'?forma.at.x-largo/2:forma.at.x-largo;
   const topo=forma.vertical==='s'?forma.at.y:forma.vertical==='m'?forma.at.y-alto/2:forma.at.y-alto;
   const x=Math.max(0,Math.min(esquerda,largura-largo)),y=Math.max(0,Math.min(topo,altura-alto));
   check(Math.abs(no.caixa.x-x)<=0.02,`${onde}: "${forma.text}" começa em x = ${no.caixa.x}, esperado ${x}`);
   check(perto(a.y,y+(alto+grafico.algarismos)/2,0.02),`${onde}: linha de base de "${forma.text}" em y = ${a.y}, esperado ${y+(alto+grafico.algarismos)/2}`);
   check(no.caixa.x>=-0.01&&no.caixa.x+no.caixa.largura<=largura+0.25,`${onde}: "${forma.text}" desenhado de ${no.caixa.x} a ${no.caixa.x+no.caixa.largura}, fora da largura ${largura}`);
  }
 });
}
function lerPng(dados){
 let posicao=8,largura=0,altura=0,canais=4;
 const blocos=[];
 while(posicao<dados.length){
  const tamanho=dados.readUInt32BE(posicao),tipo=dados.toString('latin1',posicao+4,posicao+8),corpo=dados.subarray(posicao+8,posicao+8+tamanho);
  if(tipo==='IHDR'){
   largura=corpo.readUInt32BE(0);
   altura=corpo.readUInt32BE(4);
   canais=corpo[9]===6?4:3;
   check(corpo[8]===8&&(corpo[9]===6||corpo[9]===2)&&corpo[12]===0,'captura em PNG de 8 bits por canal, sem entrelaçamento');
  }
  if(tipo==='IDAT')blocos.push(corpo);
  posicao+=12+tamanho;
 }
 const bruto=zlib.inflateSync(Buffer.concat(blocos)),passo=largura*canais,pixels=Buffer.alloc(passo*altura);
 for(let y=0;y<altura;y++){
  const filtro=bruto[y*(passo+1)],inicio=y*(passo+1)+1,linha=y*passo;
  for(let x=0;x<passo;x++){
   const a=x>=canais?pixels[linha+x-canais]:0,b=y?pixels[linha-passo+x]:0,c=x>=canais&&y?pixels[linha-passo+x-canais]:0;
   let previsto=0;
   if(filtro===1)previsto=a;
   else if(filtro===2)previsto=b;
   else if(filtro===3)previsto=(a+b)>>1;
   else if(filtro===4){const p=a+b-c,pa=Math.abs(p-a),pb=Math.abs(p-b),pc=Math.abs(p-c);previsto=pa<=pb&&pa<=pc?a:pb<=pc?b:c;}
   pixels[linha+x]=(bruto[inicio+x]+previsto)&255;
  }
 }
 return {cor:(x,y)=>{const indice=y*passo+x*canais;return `rgb(${pixels[indice]},${pixels[indice+1]},${pixels[indice+2]})`;}};
}
function conferirNitidez(ler,centro,escala,cor,descricao){
 const base=Math.floor(centro)-4,faixa=Array.from({length:9},(_,indice)=>ler(base+indice));
 const cheios=faixa.map((valor,indice)=>valor===cor?indice:-1).filter(indice=>indice>=0);
 check(cheios.length===escala&&cheios[cheios.length-1]-cheios[0]===escala-1&&faixa[cheios[0]-1]===G.PALETTE.fill&&faixa[cheios[cheios.length-1]+1]===G.PALETTE.fill,`${descricao}: a linha deveria cobrir ${escala} pixel(s) inteiros em ${cor}, entre pixels do fundo; obtido ${faixa.join(' ')}`);
}

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
  await pagina('Accessibility.enable');
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
  const grafico=()=>avaliar(GRAFICO);
  const preencher=linhas=>avaliar(`(()=>{const linhas=${JSON.stringify(linhas)},dimensao=linhas.length;linhas.forEach((valores,linha)=>valores.forEach((valor,coluna)=>{const entrada=document.getElementById('a-'+linha+'-'+(coluna===dimensao?3:coluna));entrada.focus();entrada.value=valor;entrada.dispatchEvent(new Event('input',{bubbles:true}));}));document.activeElement.blur();return true;})()`);
  const imagem=async()=>lerPng(Buffer.from((await pagina('Page.captureScreenshot',{format:'png'})).data,'base64'));
  const linhasDaLegenda=()=>avaliar(`(()=>{const faixa=document.createRange();faixa.selectNodeContents(document.getElementById('help'));return new Set([...faixa.getClientRects()].filter(retangulo=>retangulo.width>0).map(retangulo=>Math.round(retangulo.top))).size;})()`);
  const acessivel=async seletor=>{
   const resposta=await pagina('Runtime.evaluate',{expression:`document.querySelector(${JSON.stringify(seletor)})`});
   const {nodes}=await pagina('Accessibility.getPartialAXTree',{objectId:resposta.result.objectId,fetchRelatives:false});
   const no=nodes[0];
   return {papel:no.role?.value??null,nome:no.name?.value??null,descricao:no.description?.value??null};
  };
  const coresDoSistema=()=>avaliar(`(()=>{const sonda=document.createElement('div');document.body.append(sonda);const cor=nome=>{sonda.style.cssText='forced-color-adjust:none;color:'+nome;return getComputedStyle(sonda).color;};const cores={Canvas:cor('Canvas'),CanvasText:cor('CanvasText'),Highlight:cor('Highlight'),LinkText:cor('LinkText'),GrayText:cor('GrayText')};sonda.remove();return cores;})()`);
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
   igual(inicial.ajuda,G.INTRO_TWO,'a legenda do gráfico ocupa o lugar da dica');
   igual([inicial.titulo,inicial.descricao],[TEXTS.EMPTY_TITLE,TEXTS.EMPTY_DESCRIPTION],'card vazio');
   igual(inicial.copiar,{desabilitado:'true',titulo:TIP.disabled,descricao:TIP.disabled,rotulo:'Copiar'},'Copiar desabilitado');
   igual(inicial.pensar,{desabilitado:'true',titulo:TIP.disabled,descricao:TIP.disabled},'Modo pensar desabilitado');
   igual(inicial.nota,TEXTS.NOTE_DEFAULT,'nota inicial');
   check(inicial.teclasVisiveis&&!inicial.teclasPensarVisiveis&&inicial.linksVisiveis&&inicial.rodapeDica===null,'rodapé normal');
   igual(inicial.falas,[],'nada anunciado ao abrir');
  });

  await etapa('painel do gráfico',async()=>{
   const painel=await avaliar(`(()=>{
    const figura=document.getElementById('graph'),svg=document.getElementById('plot'),legenda=document.getElementById('help'),estilo=getComputedStyle(figura);
    return {
     ordem:[...document.getElementById('system-form').children].map(item=>item.id||item.className),
     filhos:[...figura.children].map(item=>item.localName+'#'+item.id),
     figura:figura.localName,
     svg:[svg.getAttribute('role'),svg.getAttribute('aria-label'),svg.getAttribute('aria-describedby')],
     legenda:[legenda.localName,legenda.getAttribute('aria-live'),legenda.getAttribute('role')],
     visual:[estilo.backgroundColor,estilo.borderTopColor,estilo.borderTopWidth,estilo.borderTopLeftRadius],
     cortado:getComputedStyle(svg).overflow
    };
   })()`);
   igual(painel.ordem,['eyebrow','equations','graph','actions'],'painel entre as equações e os botões');
   igual([painel.figura,painel.filhos],['figure',['svg#plot','figcaption#help']],'figura com o gráfico e a legenda');
   igual(painel.svg,['img',G.GRAPH_NAME,'help'],'gráfico é uma imagem com nome e descrição');
   igual(painel.legenda,['figcaption',null,null],'a legenda não é região viva');
   igual(painel.visual,['rgb(17, 16, 16)','rgb(53, 49, 55)','1px','12px'],'painel com o fundo e a borda do tema');
   igual(painel.cortado,'hidden','nada do gráfico vaza da área');
   igual(await acessivel('#plot'),{papel:'image',nome:G.GRAPH_NAME,descricao:G.INTRO_TWO},'nome e descrição acessíveis do gráfico');
   const vazio=await grafico();
   check(vazio.itens.some(item=>item.tag==='rect'&&item.atributos.fill==='url(#graph-dots-0)'),'textura de pontinhos');
   igual(vazio.texturas.map(textura=>[textura.cor,textura.opacidade]),[[G.PALETTE.t,'1']],'cor da textura');
   igual(textosDoGrafico(vazio),['x','y'],'eixos x e y');
   conferirDesenho(vazio,emRepouso(G.blank(2),vazio),false,'gráfico vazio');
   igual(retasDoGrafico(vazio).length,0,'nenhuma reta sem equação');
   check(Math.abs(vazio.plot.altura-168)<0.5&&vazio.plot.largura>200,`área do gráfico 2×2: ${vazio.plot.largura}×${vazio.plot.altura}`);
   check(vazio.legenda.esquerda>=vazio.plot.direita+15,'legenda à direita do gráfico');
   igual(vazio.pendentes,0,'nenhum quadro pedido parado');
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
   igual(atual.numerosCor.slice(0,2),['rgb(206, 169, 243)','rgb(125, 211, 192)'],'cada número na cor da sua reta');
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
   igual([atual.ajuda,atual.campos[0][0].erro,atual.campos[0][0].invalido,atual.titulo],[G.INTRO_TWO,false,null,'Faltam 5 campos · 1 inválido'],'o erro espera a saída do campo');
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
   igual([atual.ativo,atual.valores,atual.ajuda],['example',['2','1'],'As retas se cruzam em (2, 1): esse ponto é a solução do sistema.'],'exemplo resolvido sem mudar o foco, com a legenda do gráfico');
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

  await etapa('sem botão Resolver',async()=>{
   await clicar('#clear');
   igual(await avaliar(`[document.getElementById('solve')===null,[...document.querySelectorAll('.actions button')].map(item=>item.id).join(',')]`),[true,'example,clear'],'a resposta sai sozinha, sem botão Resolver');
   await focar(0,0);
   await tecla('Enter');
   const atual=await estado();
   igual([atual.ativo,atual.ajuda,atual.ajudaErro],['a-0-1',G.INTRO_TWO,false],'Enter num campo vazio vai ao próximo sem acusar erro');
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

  let repouso2x2='',botoes2x2=0;
  await etapa('gráfico do exemplo 2×2',async()=>{
   await clicar('#clear');
   const antes=(await grafico()).pedidos;
   await clicar('#example');
   await esperar(async()=>(await grafico()).pendentes===0,'terminar o único quadro do cartão',2000);
   const atual=await grafico(),texto='As retas se cruzam em (2, 1): esse ponto é a solução do sistema.';
   igual((await estado()).ajuda,texto,'legenda do exemplo');
   igual(await acessivel('#plot'),{papel:'image',nome:G.GRAPH_NAME,descricao:texto},'a descrição acessível acompanha a legenda');
   igual(retasDoGrafico(atual).map(item=>[item.atributos.stroke,item.atributos['stroke-linecap'],item.atributos['stroke-opacity']]),[[G.PALETTE.r0,'round','1'],[G.PALETTE.r1,'round','1']],'uma reta por equação, na cor do número');
   igual(pontoDoGrafico(atual).map(item=>[item.atributos.fill,item.atributos['fill-opacity']]),[[G.PALETTE.o,'1']],'o ponto da solução');
   igual(textosDoGrafico(atual),['x','y','2','1'],'eixos e coordenadas do ponto');
   check(atual.itens.some(item=>item.tag==='line'&&item.atributos.stroke===G.PALETTE.u&&item.atributos['stroke-linecap']==='round'),'linhas-guia tracejadas até os eixos');
   check(atual.itens.some(item=>item.tag==='line'&&item.atributos.stroke===G.PALETTE.a&&item.atributos['stroke-linecap']==='butt'),'eixos com pontas retas');
   check(atual.itens.some(item=>item.tag==='circle'&&item.atributos.fill===G.PALETTE.h&&item.atributos['fill-opacity']==='0.2'),'halo em volta do ponto');
   for(const item of atual.itens.filter(item=>item.tag==='line'))for(const [x,y] of [['x1','y1'],['x2','y2']])check(Number(item.atributos[x])>=-0.01&&Number(item.atributos[x])<=atual.plot.largura+0.01&&Number(item.atributos[y])>=-0.01&&Number(item.atributos[y])<=atual.plot.altura+0.01,`linha dentro da área: ${JSON.stringify(item.atributos)}`);
   igual(atual.pendentes,0,'sem quadros pendentes com movimento reduzido');
   check(atual.pedidos-antes<=2,`nenhum laço de quadros com movimento reduzido (${atual.pedidos-antes} pedidos)`);
   conferirDesenho(atual,emRepouso(esbocoDe(2),atual),false,'exemplo 2×2');
   const rotulo=texto=>atual.itens.find(item=>item.tag==='text'&&item.texto===texto),centro=pontoDoGrafico(atual)[0].atributos;
   const eixoX=atual.itens.find(item=>item.tag==='line'&&item.atributos.stroke===G.PALETTE.a&&item.atributos.y1===item.atributos.y2).atributos,eixoY=atual.itens.find(item=>item.tag==='line'&&item.atributos.stroke===G.PALETTE.a&&item.atributos.x1===item.atributos.x2).atributos;
   const dois=rotulo('2'),um=rotulo('1');
   check(Math.abs(dois.caixa.x+dois.caixa.medida/2-Number(centro.cx))<0.05&&Number(dois.atributos.y)>Number(eixoX.y1),'o 2 fica centrado embaixo do ponto, abaixo do eixo x');
   check(um.caixa.x+um.caixa.medida<=Number(eixoY.x1)-4.95&&Math.abs(Number(um.atributos.y)-(atual.algarismos/2)-Number(centro.cy))<0.05,'o 1 fica à esquerda do eixo y, na altura do ponto');
   repouso2x2=atual.assinatura;
   botoes2x2=atual.acoes.topo-atual.cartao.topo;
  });

  await etapa('gráfico do exemplo 3×3',async()=>{
   await clicar('#size3');
   await clicar('#example');
   const atual=await grafico(),texto='Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema.';
   igual((await estado()).ajuda,texto,'legenda do 3×3');
   igual(await acessivel('#plot'),{papel:'image',nome:G.GRAPH_NAME,descricao:texto},'descrição acessível do 3×3');
   check(atual.tres&&Math.abs(atual.plot.altura-112)<0.5,`área do gráfico 3×3: ${atual.plot.largura}×${atual.plot.altura}`);
   igual(planosDoGrafico(atual).map(item=>item.atributos.fill).sort(),[G.PALETTE.r0,G.PALETTE.r1,G.PALETTE.r2].sort(),'três planos translúcidos nas cores das equações');
   igual(atual.itens.filter(item=>item.tag==='line'&&item.atributos.stroke===G.PALETTE.m).length,3,'as retas de encontro dos planos');
   igual(pontoDoGrafico(atual).length,1,'o ponto da solução');
   igual(textosDoGrafico(atual),['x','z','y'],'eixos da caixa');
   conferirDesenho(atual,emRepouso(esbocoDe(3),atual),false,'exemplo 3×3');
   const y=atual.itens.find(item=>item.tag==='text'&&item.texto==='y');
   check(Number(y.atributos.y)-(G.TEXT_HEIGHT+atual.algarismos)/2>=-0.01,'o rótulo y no alto da caixa é empurrado para dentro da área');
   igual((await estado()).numerosCor,['rgb(206, 169, 243)','rgb(125, 211, 192)','rgb(247, 178, 140)'],'números nas cores dos planos');
   igual(atual.acoes.topo-atual.cartao.topo,botoes2x2,'os botões não saem do lugar ao trocar 2×2 e 3×3');
   await clicar('#size2');
   igual((await grafico()).assinatura,repouso2x2,'o 2×2 volta igual');
  });

  await etapa('animação do gráfico',async()=>{
   await clicar('#clear');
   await midia([{name:'prefers-reduced-motion',value:'no-preference'}]);
   await esperar(async()=>(await grafico()).pendentes===0,'assentar o gráfico vazio',5000);
   const antes=await grafico();
   const logo=await avaliar(`(()=>{document.getElementById('example').click();return ${GRAFICO};})()`);
   check(logo.pendentes>=1,'o gráfico pede quadros enquanto anima');
   await esperar(async()=>(await grafico()).pendentes===0,'terminar a animação do gráfico',8000);
   const fim=await grafico();
   check(fim.pedidos-antes.pedidos>=5,`a animação desenhou vários quadros (${fim.pedidos-antes.pedidos})`);
   igual(fim.assinatura,repouso2x2,'a animação termina exatamente na cena de repouso');
   const inicio=retasDoGrafico(logo).map(comprimento),final=retasDoGrafico(fim).map(comprimento);
   igual(inicio.length,2,'as retas surgem logo no clique');
   check(inicio.every((valor,indice)=>valor<final[indice]/2),`as retas crescem a partir do meio (${inicio} → ${final})`);
   await clicar('#clear');
   await avaliar(`document.getElementById('example').click()`);
   check((await grafico()).pendentes>=1,'animando de novo');
   await midia(movimentoReduzido);
   await esperar(async()=>(await grafico()).pendentes===0,'parar ao pedir menos movimento',3000);
   igual((await grafico()).assinatura,repouso2x2,'menos movimento leva direto ao estado final');
  });

  await etapa('quadros da animação desenhados como a cena',async()=>{
   await midia([{name:'prefers-reduced-motion',value:'no-preference'}]);
   const quadro=ms=>avaliar(`(window.__tempo.ir(${ms}),new Promise(resolver=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolver(true)))))`);
   const inicio=await avaliar('window.__tempo.parar()');
   for(const dimensao of [2,3]){
    const motion=new G.Motion(),troca=inicio+(dimensao-2)*30000,limpeza=troca+10000,clique=limpeza+10000;
    await avaliar(`window.__tempo.ir(${troca})`);
    await clicar(`#size${dimensao}`);
    motion.update(esbocoDe(dimensao),troca/1000,true);
    await avaliar(`window.__tempo.ir(${limpeza})`);
    await clicar('#clear');
    motion.update(G.blank(dimensao),limpeza/1000,true);
    await quadro(clique);
    igual((await grafico()).pendentes,0,`o ${dimensao}×${dimensao} vazio fica parado`);
    await avaliar(`document.getElementById('example').click()`);
    motion.update(esbocoDe(dimensao),clique/1000,true);
    let parciais=0;
    for(const passo of [40,150,300,450,800,1500]){
     await quadro(clique+passo);
     const atual=await grafico(),itens=motion.scene((clique+passo)/1000,atual.plot.largura,atual.plot.altura,1,false);
     check(atual.pendentes>=1,`o laço de quadros continua aos ${passo} ms`);
     conferirDesenho(atual,itens,false,`${dimensao}×${dimensao} aos ${passo} ms`);
     parciais+=itens.filter(item=>item.alpha>0&&item.alpha<1&&(item.shape.kind==='text'||item.ink==='m')).length;
    }
    check(parciais>0,`quadros do ${dimensao}×${dimensao} com rótulos ou retas de encontro ainda surgindo`);
    await quadro(clique+5000);
    const fim=await grafico();
    igual(fim.pendentes,0,`a animação do ${dimensao}×${dimensao} termina`);
    conferirDesenho(fim,emRepouso(esbocoDe(dimensao),fim),false,`${dimensao}×${dimensao} em repouso depois da animação`);
   }
   await avaliar('window.__tempo.soltar()');
   await midia(movimentoReduzido);
   await clicar('#size2');
   await clicar('#example');
   igual((await grafico()).assinatura,repouso2x2,'o 2×2 volta ao exemplo em repouso');
  });

  await etapa('o gráfico para de pedir quadros com o painel oculto',async()=>{
   await midia([{name:'prefers-reduced-motion',value:'no-preference'}]);
   await clicar('#clear');
   await esperar(async()=>(await grafico()).pendentes===0,'assentar o gráfico vazio',5000);
   await clicar('#example');
   check((await grafico()).pendentes>=1,'o gráfico anima depois do exemplo');
   await clicar('#think-open');
   igual(await avaliar(`[document.getElementById('think').hidden,document.getElementById('plot').getClientRects().length]`),[false,0],'o Modo pensar esconde o painel');
   await avaliar('new Promise(resolver=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolver(true))))');
   igual((await grafico()).pendentes,0,'nenhum quadro pedido com o painel oculto');
   await tecla('Escape');
   await esperar(async()=>(await grafico()).pendentes===0,'terminar a animação depois de voltar',8000);
   igual((await grafico()).assinatura,repouso2x2,'o gráfico volta inteiro ao sair do Modo pensar');
   await midia(movimentoReduzido);
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
   await focar(0,0);
   await digitar('1/0');
   await tecla('Enter');
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

  await etapa('gráfico em alto contraste',async()=>{
   await clicar('#example');
   igual((await grafico()).assinatura,repouso2x2,'exemplo 2×2 nas cores do tema');
   await midia([...movimentoReduzido,{name:'forced-colors',value:'active'}]);
   await esperar(async()=>!(await grafico()).itens.some(item=>item.tag==='rect'),'redesenhar o gráfico em alto contraste',3000);
   const atual=await grafico(),cores=await coresDoSistema();
   igual(atual.ajuste,'none','o gráfico pinta com as cores do sistema sem ajuste automático');
   igual(atual.texturas,[],'sem textura no alto contraste');
   check(atual.itens.every(item=>['fill-opacity','stroke-opacity'].every(nome=>item.atributos[nome]===undefined||item.atributos[nome]==='1')),'sem transparência no alto contraste');
   igual(retasDoGrafico(atual).map(item=>item.atributos.stroke),['Highlight','LinkText'],'retas nas cores do sistema');
   igual(pontoDoGrafico(atual).map(item=>item.atributos.fill),['CanvasText'],'ponto na cor do texto');
   igual(atual.itens.filter(item=>item.tag==='circle'&&item.atributos.r==='6').map(item=>item.atributos.fill),['Canvas'],'borda do ponto na cor do fundo do sistema');
   conferirDesenho(atual,emRepouso(esbocoDe(2),atual,true),true,'exemplo 2×2 em alto contraste');
   const sistema=new Set(['Canvas','CanvasText','Highlight','LinkText','GrayText','none']);
   check(atual.itens.every(item=>[item.atributos.fill,item.atributos.stroke].every(cor=>cor===undefined||sistema.has(cor))),'só cores do sistema no gráfico');
   igual(await avaliar(`[...document.querySelectorAll('#plot line')].filter(item=>item.getAttribute('stroke-width')==='2').map(item=>getComputedStyle(item).stroke)`),[cores.Highlight,cores.LinkText],'cores do sistema aplicadas de verdade');
   igual((await estado()).numerosCor.slice(0,2),[cores.Highlight,cores.LinkText],'números das equações nas cores do sistema');
   await midia(movimentoReduzido);
   await esperar(async()=>(await grafico()).assinatura===repouso2x2,'voltar às cores do tema',3000);
  });

  await etapa('gráfico acompanha o tamanho da janela',async()=>{
   await pagina('Emulation.setDeviceMetricsOverride',{width:960,height:900,deviceScaleFactor:1,mobile:false});
   const eixoNaBorda=async()=>{const atual=await grafico(),eixo=atual.itens.find(item=>item.tag==='line'&&item.atributos.stroke===G.PALETTE.a&&item.atributos.y1===item.atributos.y2);return !!eixo&&Math.abs(Number(eixo.atributos.x2)-atual.plot.largura)<0.01;};
   await esperar(eixoNaBorda,'redesenhar no novo tamanho',3000);
   const atual=await grafico();
   check(atual.plot.largura<240,`a área encolheu com a janela (${atual.plot.largura} px)`);
   await pagina('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
   await esperar(async()=>(await grafico()).assinatura===repouso2x2,'voltar ao tamanho original',3000);
  });

  await etapa('textura acesa quando tudo é solução',async()=>{
   const linhas=[['0','0','0'],['0','0','0']];
   await clicar('#clear');
   await preencher(linhas);
   const atual=await grafico();
   igual((await estado()).ajuda,G.EVERYTHING_TWO,'legenda do plano inteiro');
   igual(atual.texturas.map(textura=>[textura.cor,textura.opacidade]),[[G.PALETTE.t,'1'],[G.PALETTE.s,'0.55']],'uma segunda textura acende os pontinhos');
   conferirDesenho(atual,emRepouso(esbocoDe(2,linhas),atual),false,'tudo é solução');
  });

  await etapa('rótulos perto da borda ficam dentro da área',async()=>{
   for(const linhas of [[['1','1','1'],['1','1,0001','2']],[['1','0','123'],['0','1','-4567']],[['3','0','-7'],['0','3','-11']]]){
    await clicar('#clear');
    await preencher(linhas);
    const atual=await grafico();
    conferirDesenho(atual,emRepouso(esbocoDe(2,linhas),atual),false,`sistema ${JSON.stringify(linhas)}`);
    check(textosDoGrafico(atual).length===4,`eixos e as duas coordenadas em ${JSON.stringify(linhas)}`);
   }
  });

  await etapa('grade e eixos nítidos no pixel',async()=>{
   for(const escala of [1,2]){
    await pagina('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:escala,mobile:false});
    await avaliar(`document.getElementById('graph').removeAttribute('style')`);
    await clicar('#clear');
    await avaliar(`(()=>{const caixa=document.getElementById('plot').getBoundingClientRect(),fracao=valor=>valor-Math.floor(valor);Object.assign(document.getElementById('graph').style,{position:'relative',left:(0.4-fracao(caixa.left+scrollX))+'px',top:(0.4-fracao(caixa.top+scrollY))+'px'});document.getElementById('plot').scrollIntoView({block:'center'});return true;})()`);
    const atual=await grafico(),foto=await imagem();
    const linhas=atual.itens.filter(item=>item.tag==='line').map(item=>({x1:Number(item.atributos.x1),y1:Number(item.atributos.y1),x2:Number(item.atributos.x2),y2:Number(item.atributos.y2),cor:item.atributos.stroke}));
    const horizontais=linhas.filter(linha=>linha.y1===linha.y2),verticais=linhas.filter(linha=>linha.x1===linha.x2);
    const livre=(ocupadas,limite)=>{for(let valor=16;valor<limite-24;valor+=8)if(ocupadas.every(posicao=>Math.abs(posicao-valor-0.5)>=3))return valor;return null;};
    const coluna=livre(verticais.map(linha=>linha.x1),atual.plot.largura),fileira=livre(horizontais.map(linha=>linha.y1),atual.plot.altura);
    const eixoX=horizontais.find(linha=>linha.cor===G.PALETTE.a),eixoY=verticais.find(linha=>linha.cor===G.PALETTE.a),grade=horizontais.find(linha=>linha.cor===G.PALETTE.g);
    check(!!eixoX&&!!eixoY&&!!grade&&coluna!==null&&fileira!==null,'eixos, grade e um trecho livre para conferir');
    for(const borda of [atual.plot.esquerda,atual.plot.topo])check(Math.abs(borda*escala-Math.round(borda*escala))>0.1,`o painel começa entre dois pixels (${borda} px com escala ${escala})`);
    const x=Math.floor((atual.plot.esquerda+coluna+0.5)*escala),y=Math.floor((atual.plot.topo+fileira+0.5)*escala);
    conferirNitidez(linha=>foto.cor(x,linha),(atual.plot.topo+eixoX.y1)*escala,escala,G.PALETTE.a,`eixo x com escala ${escala}`);
    conferirNitidez(linha=>foto.cor(x,linha),(atual.plot.topo+grade.y1)*escala,escala,G.PALETTE.g,`grade horizontal com escala ${escala}`);
    conferirNitidez(indice=>foto.cor(indice,y),(atual.plot.esquerda+eixoY.x1)*escala,escala,G.PALETTE.a,`eixo y com escala ${escala}`);
   }
   await avaliar(`document.getElementById('graph').removeAttribute('style')`);
   await pagina('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
  });

  await etapa('legenda longa cabe no painel',async()=>{
   await pagina('Emulation.setDeviceMetricsOverride',{width:851,height:900,deviceScaleFactor:1,mobile:false});
   const medir=()=>avaliar(`(()=>{const ajuda=document.getElementById('help'),caixa=ajuda.getBoundingClientRect(),estilo=getComputedStyle(ajuda),faixa=document.createRange();faixa.selectNodeContents(ajuda);const linhas=[...faixa.getClientRects()].filter(retangulo=>retangulo.width>0);return {texto:ajuda.textContent,coluna:[caixa.left+parseFloat(estilo.paddingLeft),caixa.right-parseFloat(estilo.paddingRight)],esquerda:Math.min(...linhas.map(retangulo=>retangulo.left)),direita:Math.max(...linhas.map(retangulo=>retangulo.right)),juntos:[...ajuda.querySelectorAll('.keep')].map(trecho=>[trecho.textContent,getComputedStyle(trecho).whiteSpace]),quebras:ajuda.querySelectorAll('wbr').length};})()`);
   for(const [linhas,legenda,quebras] of [
    [[['7654321','0','-123456'],['0','7654321','234567']],'As retas se cruzam em (-123456/7654321, 234567/7654321): esse ponto é a solução do sistema.',0],
    [[['987654321011','0','-123456789012'],['0','7','5']],'As retas se cruzam em (-123456789012/987654321011, 5/7): esse ponto é a solução do sistema.',1],
    [[['1','1','2'],['0','0','-123456789012/987654321011']],'A equação 2 diz que 0 = -123456789012/987654321011, o que nunca acontece: o sistema não tem solução.',1]
   ]){
    await clicar('#clear');
    await preencher(linhas);
    const medida=await medir();
    igual(medida.texto,legenda,'legenda com valores longos');
    check(medida.esquerda>=medida.coluna[0]-0.5&&medida.direita<=medida.coluna[1]+0.5,`a legenda fica dentro da sua coluna: texto de ${medida.esquerda} a ${medida.direita} px, coluna de ${medida.coluna[0]} a ${medida.coluna[1]} px`);
    igual([medida.juntos,medida.quebras],[[],quebras],'coordenadas longas quebram entre si, e as frações enormes depois da barra');
   }
   await clicar('#example');
   igual((await medir()).juntos,[['(2, 1)','nowrap']],'coordenadas curtas ficam na mesma linha');
   await pagina('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
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
   const apresentacao=await grafico();
   igual(await linhasDaLegenda(),3,'no celular a apresentação ocupa três linhas');
   await clicar('#example');
   igual((await estado()).resumo,'x = 2 · y = 1','resumo com o resultado');
   const painel=await grafico();
   igual(await linhasDaLegenda(),2,'no celular a legenda do exemplo ocupa duas linhas');
   igual([painel.figura.altura,painel.acoes.topo-painel.cartao.topo],[apresentacao.figura.altura,apresentacao.acoes.topo-apresentacao.cartao.topo],'o painel reserva três linhas de legenda: os botões não pulam quando ela muda');
   check(Math.abs(painel.plot.largura-(painel.figura.largura-10))<1&&Math.abs(painel.plot.altura-168)<0.5,`no celular o gráfico ocupa a largura do painel: ${JSON.stringify(painel.plot)}`);
   check(painel.legenda.topo>=painel.plot.base-0.5&&painel.legenda.base<=painel.figura.base,'no celular a legenda fica embaixo do gráfico');
   igual(retasDoGrafico(painel).length,2,'as retas no celular');
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
