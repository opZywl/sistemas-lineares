if(typeof BigInt!=='undefined') (function(){
 'use strict';
 const {Rational,solve,format}=LinearSolver;
 const $=id=>document.getElementById(id), names=['x','y','z'];
 let dimension=2,lastResult=null; const stored=Array.from({length:3},()=>Array(4).fill(''));
 const icon=name=>`<svg aria-hidden="true"><use href="#icon-${name}"/></svg>`;
 const text=(tag,content,className)=>{const node=document.createElement(tag);node.textContent=content;if(className)node.className=className;return node;};
 function resetHelp(){ $('help').className=''; $('help').textContent='Aceita 2, -1,5 e 3/4. Use 0 se a variável não aparecer.';document.querySelectorAll('.field.error').forEach(n=>n.classList.remove('error'));document.querySelectorAll('input[aria-invalid]').forEach(n=>n.removeAttribute('aria-invalid')); }
 function resetResult(){lastResult=null;$('copy').disabled=true;$('copy').querySelector('span').textContent='Copiar resultado';$('status').textContent='';$('result-note').textContent='Resultados exatos, incluindo frações.';$('output').className='empty';$('output').innerHTML=icon('matrix');$('output').append(text('h2','Aguardando os valores'),text('p','Preencha o sistema ao lado ou use um exemplo. A resposta será exibida neste painel.'));}
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
  const solution=solve(matrix);lastResult=format(solution);$('copy').disabled=false;$('output').className='results';$('output').replaceChildren();
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
 $('copy').onclick=async()=>{
  if(!lastResult)return;
  try{if(navigator.clipboard?.writeText)await navigator.clipboard.writeText(lastResult);else throw Error();$('status').textContent='Resultado copiado.';}
  catch(_){const area=document.createElement('textarea');area.value=lastResult;area.style.position='fixed';area.style.opacity='0';document.body.append(area);area.select();const ok=document.execCommand('copy');area.remove();$('status').textContent=ok?'Resultado copiado.':'Selecione a resposta e use Ctrl+C para copiar.';}
 };
 build();
})();
