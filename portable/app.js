if(typeof BigInt!=='undefined') (function(){
 'use strict';
 const {Rational,solve,format,METHODS,walkthrough,walkthroughText,matrixLines,matrixCells}=LinearSolver;
 const $=id=>document.getElementById(id), names=['x','y','z'], STORAGE='sistemas-lineares.metodo';
 let dimension=2,lastResult=null,lastMatrix=null,current=null,copiedTimer=0,fitFrame=0; const stored=Array.from({length:3},()=>Array(4).fill(''));
 const readMethod=()=>{try{const key=localStorage.getItem(STORAGE);if(METHODS.some(item=>item.key===key))return key;}catch(_){}return METHODS[0].key;};
 let method=readMethod();
 const icon=name=>`<svg aria-hidden="true"><use href="#icon-${name}"/></svg>`;
 const text=(tag,content,className)=>{const node=document.createElement(tag);node.textContent=content;if(className)node.className=className;return node;};
 function resetHelp(){ $('help').className=''; $('help').textContent='Aceita 2, -1,5 e 3/4. Use 0 se a variável não aparecer.';document.querySelectorAll('.field.error').forEach(n=>n.classList.remove('error'));document.querySelectorAll('input[aria-invalid]').forEach(n=>n.removeAttribute('aria-invalid')); }
 function resetResult(){lastResult=null;lastMatrix=null;current=null;$('copy').disabled=true;$('think-open').disabled=true;$('status').textContent='';$('result-note').textContent='Resultados exatos, incluindo frações.';$('output').className='empty';$('output').innerHTML=icon('matrix');$('output').append(text('h2','Aguardando os valores'),text('p','Preencha o sistema ao lado ou use um exemplo. A resposta será exibida neste painel.'));}
 function build(){
  $('equations').replaceChildren();
  for(let i=0;i<dimension;i++){
   const row=text('div','','equation');row.append(text('span',String(i+1).padStart(2,'0'),'number'));
   for(let j=0;j<=dimension;j++){
    const col=j===dimension?3:j,host=text('label','','field'),input=document.createElement('input');
    input.type='text';input.autocomplete='off';input.spellcheck=false;input.id=`a-${i}-${col}`;input.value=stored[i][col];input.setAttribute('aria-label',`Equação ${i+1}, ${j===dimension?'termo independente':`coeficiente de ${names[j]}`}`);input.setAttribute('aria-describedby','help');
    input.addEventListener('input',()=>{stored[i][col]=input.value;resetHelp();resetResult();});host.append(input);if(j<dimension)host.append(text('span',names[j]));row.append(host);if(j<dimension)row.append(text('span',j===dimension-1?'=':'+','operator'));
   } $('equations').append(row);
  }
  $('size2').setAttribute('aria-pressed',String(dimension===2));$('size3').setAttribute('aria-pressed',String(dimension===3));$('size-info').textContent=`${dimension} equações · ${dimension} incógnitas`;resetHelp();resetResult();
 }
 $('size2').onclick=()=>{dimension=2;build();};$('size3').onclick=()=>{dimension=3;build();};
 $('example').onclick=()=>{const a=dimension===2?[[2,1,5],[1,-1,1]]:[[1,1,1,6],[2,-1,1,3],[1,2,-1,2]];for(let i=0;i<dimension;i++)for(let j=0;j<=dimension;j++)stored[i][j===dimension?3:j]=String(a[i][j]);build();$('help').textContent='Exemplo pronto. Clique em Resolver sistema ou pressione Enter.';};
 $('clear').onclick=()=>{stored.forEach(row=>row.fill(''));build();$('a-0-0').focus();};
 $('system-form').onsubmit=event=>{
  event.preventDefault();resetHelp();resetResult();const matrix=[];
  for(let i=0;i<dimension;i++){matrix[i]=[];for(let j=0;j<=dimension;j++){const field=$(`a-${i}-${j===dimension?3:j}`);try{matrix[i][j]=Rational.parse(field.value);}catch(error){$('help').className='error';$('help').textContent=`Equação ${i+1}, ${j===dimension?'termo independente':`coeficiente de ${names[j]}`}: ${error.message}`;field.parentElement.classList.add('error');field.setAttribute('aria-invalid','true');field.focus();field.select();return;}}}
  const solution=solve(matrix);lastResult=format(solution);lastMatrix=matrix;$('copy').disabled=false;$('think-open').disabled=false;$('output').className='results';$('output').replaceChildren();
  const title=solution.kind==='unique'?'Solução encontrada':solution.kind==='infinite'?'Infinitas soluções':'Sem solução';$('output').append(text('h2',title));
  if(solution.kind==='unique'){
   $('output').append(text('p','Estes valores satisfazem todas as equações do sistema.','description'));
   solution.rref.forEach((row,i)=>{const tile=text('div','','value');tile.append(text('span',names[i],'variable'),text('span',String(row[dimension]),'answer'));$('output').append(tile);});$('result-note').textContent='Solução única · valores exatos';
  }else if(solution.kind==='infinite'){
   $('output').append(text('p','As equações são dependentes. Veja a forma geral da resposta:','description'),text('pre',lastResult.split('\n').slice(2).join('\n')));$('result-note').textContent='Os parâmetros t1, t2, … são números reais livres.';
  }else{
   $('output').append(text('p','As equações são incompatíveis.','description'),text('pre','Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo.'));$('result-note').textContent='Confira os coeficientes se esse não era o esperado.';
  }
 };
 async function copyText(value){
  try{if(navigator.clipboard?.writeText){await navigator.clipboard.writeText(value);return true;}}catch(_){}
  const focused=document.activeElement,area=document.createElement('textarea');area.value=value;area.setAttribute('readonly','');area.style.position='fixed';area.style.top='0';area.style.opacity='0';document.body.append(area);area.select();
  let ok=false;try{ok=document.execCommand('copy');}catch(_){}
  area.remove();focused?.focus?.({preventScroll:true});return ok;
 }
 $('copy').onclick=async()=>{if(!lastResult)return;$('status').textContent=await copyText(lastResult)?'Resultado copiado.':'Selecione a resposta e use Ctrl+C para copiar.';};
 function announce(message){const live=$('think-live');live.textContent='';setTimeout(()=>{live.textContent=message;},60);}
 const methodButtons=METHODS.map(item=>{const button=text('button',item.label);button.type='button';button.id=`method-${item.key}`;if(item.name!==item.label)button.setAttribute('aria-label',item.name);button.onclick=()=>chooseMethod(item.key);return button;});
 $('methods').append(...methodButtons);
 function matrixNode(view){
  const scroll=text('div','','matrix-scroll'),figure=text('div','','matrix-figure'),grid=text('div','','matrix'),cells=matrixCells(view),rows=cells.length,columns=cells[0].length,marked=row=>view.markedRows.includes(row),lines=matrixLines(view).map(line=>line.replace(/\s+/g,' '));
  figure.setAttribute('role','img');figure.setAttribute('aria-label',view.label!=null?`${lines[0]} ${lines.slice(1).join(', ')}`:lines.join(', '));
  if(view.label!=null)figure.append(text('span',`${view.label} =`,'matrix-label'));
  grid.style.gridTemplateColumns=`repeat(${columns},auto)`;
  cells.forEach((row,r)=>row.forEach((value,c)=>{
   const cell=text('span',value,'cell'),classes=[];
   if(view.augmented&&c===columns-1)classes.push('bar');
   if(marked(r)){classes.push('mark');if(c===0||c===columns-1){const side=c===0?'l':'r';if(!marked(r-1))classes.push(`t${side}`);if(!marked(r+1))classes.push(`b${side}`);}}
   if(view.markedColumn===c){classes.push('mark');if(r===0)classes.push('tl','tr');if(r===rows-1)classes.push('bl','br');}
   if(view.pivot&&view.pivot[0]===r&&view.pivot[1]===c)classes.push('pivot');
   if(classes.length)cell.classList.add(...classes);grid.append(cell);
  }));
  figure.append(grid);scroll.append(figure);return scroll;
 }
 function stepNode(step,index){
  const item=text('li','','step'),title=text('h3','','step-title'),badge=text('span',String(index+1),'step-number'),math=step.lines.length?text('div','','math'):null,matrix=step.matrix?matrixNode(step.matrix):null;
  badge.setAttribute('aria-hidden','true');title.append(badge,text('span',`Passo ${index+1}: `,'sr-only'),step.title);item.append(title,text('p',step.reasoning,'reasoning'));
  if(math)step.lines.forEach(line=>math.append(text('p',line.text,`math-line ${line.kind}`)));
  for(const node of step.matrixFirst?[matrix,math]:[math,matrix])if(node)item.append(node);
  return item;
 }
 function fitMatrices(){cancelAnimationFrame(fitFrame);fitFrame=requestAnimationFrame(()=>document.querySelectorAll('.matrix-scroll').forEach(node=>{if(node.scrollWidth>node.clientWidth+1)node.tabIndex=0;else node.removeAttribute('tabindex');}));}
 function resetCopySteps(){clearTimeout(copiedTimer);$('copy-steps').querySelector('span').textContent='Copiar passos';}
 function renderThinking(){
  current=walkthrough(lastMatrix,method);
  methodButtons.forEach((button,index)=>button.setAttribute('aria-pressed',String(METHODS[index].key===method)));
  $('think-info').textContent=`${current.method.name} · ${current.steps.length} passos`;
  $('think-system').replaceChildren(...current.system.map(line=>text('p',line)));
  $('think-answer').replaceChildren(...current.answer.map(line=>text('p',line)));
  $('think-classification').textContent=current.classification;
  $('steps').replaceChildren(...current.steps.map(stepNode));
  resetCopySteps();fitMatrices();
 }
 function chooseMethod(key){
  if(key===method||!lastMatrix)return;method=key;
  try{localStorage.setItem(STORAGE,key);}catch(_){}
  renderThinking();announce(`Método: ${current.method.name}`);
 }
 function openThinking(){
  if(!lastMatrix)return;
  renderThinking();
  $('solver').hidden=true;$('hero-solver').hidden=true;$('hero-think').hidden=false;$('think').hidden=false;$('keys').textContent='ESC para voltar · TAB para navegar';
  fitMatrices();window.scrollTo(0,0);$('think-title').focus({preventScroll:true});
 }
 function closeThinking(){
  resetCopySteps();
  $('think').hidden=true;$('hero-think').hidden=true;$('hero-solver').hidden=false;$('solver').hidden=false;$('keys').textContent='ENTER para resolver · TAB para navegar';
  $('think-open').focus();
 }
 $('think-open').onclick=openThinking;$('back').onclick=closeThinking;
 $('copy-steps').onclick=async()=>{
  if(!current)return;const ok=await copyText(walkthroughText(current));
  clearTimeout(copiedTimer);$('copy-steps').querySelector('span').textContent=ok?'Passos copiados':'Erro ao copiar';
  announce(ok?'Passos copiados.':'Não foi possível copiar os passos.');copiedTimer=setTimeout(resetCopySteps,2400);
 };
 document.addEventListener('keydown',event=>{if(event.key==='Escape'&&!$('think').hidden){event.preventDefault();closeThinking();}});
 window.addEventListener('resize',()=>{if(!$('think').hidden)fitMatrices();});
 build();
})();
