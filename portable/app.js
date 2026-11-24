(function(root){
 'use strict';
 const node=typeof module!=='undefined'&&module.exports;
 const solver=node?require('./solver.js'):root.LinearSolver;
 const {Rational,ParseError,PARSE_MESSAGES,solve,format,METHODS,walkthroughText,matrixLines,matrixCells,VARIABLES,trim,usesExtension,looksIncomplete,withinLiveBudget,Entry,previewEquation,abbreviate}=solver;

 const TWO_LABEL='2 × 2',THREE_LABEL='3 × 3';
 const EMPTY_TITLE='Aguardando os valores';
 const EMPTY_DESCRIPTION='Preencha o sistema ou use um exemplo. A resposta aparece assim que tudo estiver preenchido.';
 const MISSING_DESCRIPTION='A resposta aparece assim que tudo estiver preenchido. Enter leva ao próximo campo vazio.';
 const INVALID_TITLE='Confira os valores';
 const INVALID_DESCRIPTION='Um dos campos não é um número válido. A dica abaixo dos campos mostra o que corrigir.';
 const LARGE_TITLE='Números grandes';
 const LARGE_DESCRIPTION='Com números tão grandes, a conta é feita quando você pressiona Enter ou Resolver sistema.';
 const ONLY_THIS_FIELD='Falta só este campo.';
 const KEYS=[['Enter','próximo campo vazio'],['↑ ↓','mudar de equação']];
 const THINK_KEYS=[['Esc','voltar']];
 const THINK_FOOTER='Operações de linha e determinantes  /  aritmética racional exata';
 const DEFAULT_HELPER='Aceita 2 · -1,5 · 3/4 · 2^3 · 10². Use 0 se a variável não aparecer.';
 const EXAMPLE_HELPER='Exemplo resolvido. Mude qualquer valor e a resposta se atualiza na hora.';
 const AUTHOR_HELPER='Abra lucas-lima.xyz no seu navegador para visitar o autor.';
 const NOTE_DEFAULT='Resultados exatos, incluindo frações.';
 const NOTE_UNIQUE='Solução única · valores exatos';
 const NOTE_INFINITE='Os parâmetros t1, t2, … são números reais livres.';
 const NOTE_NONE='Confira os coeficientes se esse não era o esperado.';
 const NOTE_STALE='Atualizando: a resposta acompanha o que você digita.';
 const NOTE_COPY_FAILED='Não foi possível copiar agora. Tente novamente.';
 const COPY_LABEL='Copiar',COPIED_LABEL='Copiado',COPY_STEPS_LABEL='Copiar passos',COPIED_STEPS_LABEL='Passos copiados';
 const COPY_STEPS_FAILED='Não foi possível copiar agora. Tente novamente.';
 const TITLE_UNIQUE='Solução encontrada',TITLE_INFINITE='Infinitas soluções',TITLE_NONE='Sem solução';
 const DESCRIPTION_UNIQUE='Estes valores satisfazem todas as equações do sistema.';
 const DESCRIPTION_INFINITE='As equações são dependentes. Veja a forma geral da resposta:';
 const DESCRIPTION_NONE='As equações são incompatíveis.';
 const GENERAL_NONE='Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo.';
 const SPOKEN_LIMIT=24;
 const EXAMPLES={2:[[2,1,5],[1,-1,1]],3:[[1,1,1,6],[2,-1,1,3],[1,2,-1,2]]};
 const METHOD_NEEDS_SYSTEM={
  'gauss-jordan':'Preencha todos os campos para ver o passo a passo por Gauss-Jordan.',
  cramer:'Preencha todos os campos para ver o passo a passo pela regra de Cramer.',
  gauss:'Preencha todos os campos para ver o passo a passo por escalonamento.'
 };
 const METHOD_LINK_NAMES={
  'gauss-jordan':'Passo a passo por Gauss-Jordan',
  cramer:'Passo a passo pela regra de Cramer',
  gauss:'Passo a passo por escalonamento'
 };

 const inputColumn=(dimension,column)=>column===dimension?3:column;
 const layoutVisible=(dimension,row,column)=>row<dimension&&(column<dimension||column===3);
 const termName=column=>column<VARIABLES.length?`coeficiente de ${VARIABLES[column]}`:'termo independente';
 const accessibleInputName=(row,column)=>`Equação ${row+1}, ${termName(column)}`;
 const accessibleValueName=index=>`Valor de ${VARIABLES[index]??'x'}`;
 const sizeName=(dimension,selected)=>`Sistema ${dimension===2?TWO_LABEL:THREE_LABEL}${selected?', selecionado':''}`;
 const rowNumber=row=>String(row+1).padStart(2,'0');
 const missingTitle=missing=>missing===1?'Falta 1 campo':`Faltam ${missing} campos`;
 const invalidSuffix=invalid=>invalid===1?' · 1 inválido':` · ${invalid} inválidos`;
 const fieldFocus=(row,column,selectAll=true)=>({row,column,selectAll});
 const sameField=(first,second)=>first===second||(!!first&&!!second&&first[0]===second[0]&&first[1]===second[1]);
 const blankInputs=()=>Array.from({length:3},()=>['','','','']);
 const blankFlags=()=>Array.from({length:3},()=>[false,false,false,false]);
 const countDigits=text=>(text.match(/[0-9]/g)||[]).length;
 const reading=value=>`Este campo vale ${abbreviate(String(value))}.`;
 function spokenValue(value){
  if(Array.from(value).length<=SPOKEN_LIMIT)return value;
  const slash=value.indexOf('/');
  if(slash<0)return `número com ${countDigits(value)} algarismos`;
  return `fração com ${countDigits(value.slice(0,slash))} algarismos sobre ${countDigits(value.slice(slash+1))} algarismos`;
 }
 function parseCell(text){
  try{return {kind:'value',value:Rational.parse(text)};}
  catch(error){if(error instanceof ParseError)return {kind:'error',error};throw error;}
 }

 class Model{
  constructor(){
   this.state={dimension:2,inputs:[blankInputs(),blankInputs()],cells:[],revealed:blankFlags(),focus:null,notice:null,outcome:{kind:'empty'},stale:false,lastResult:null,note:NOTE_DEFAULT,copied:false,solved:null,method:METHODS[0].key,thinking:null,stepsCopied:false,stepsCopyFailed:false,missing:0,invalid:0,large:false,blank:true,preview:[]};
   this.analyze(false);
  }
  dimension(){return this.state.dimension;}
  grid(){return this.state.inputs[this.state.dimension>=3?1:0];}
  input(row,column){return this.grid()[row]?.[column]??'';}
  cell(row,column){return this.state.cells[row]?.[column];}
  visibleFields(){
   const dimension=this.state.dimension,fields=[];
   for(let row=0;row<dimension;row++)for(let column=0;column<=dimension;column++)fields.push([row,inputColumn(dimension,column)]);
   return fields;
  }
  analyze(explicit){
   const state=this.state,dimension=state.dimension;
   const cells=Array.from({length:3},()=>Array.from({length:4},()=>({kind:'missing'})));
   const matrix=[],preview=[];
   let missing=0,invalid=0,blank=true;
   for(let row=0;row<dimension;row++){
    const values=[],entries=[];
    for(let column=0;column<=dimension;column++){
     const target=inputColumn(dimension,column),text=this.grid()[row][target],parsed=parseCell(text);
     let cell=parsed;
     if(parsed.kind==='error'){
      if(parsed.error.kind==='Empty')cell={kind:'missing'};
      else{
       const typing=!explicit&&!state.revealed[row][target]&&sameField(state.focus,[row,target])&&looksIncomplete(text);
       cell=typing?{kind:'typing'}:{kind:'invalid',error:parsed.error};
      }
     }
     if(trim(text))blank=false;
     if(cell.kind==='value'){values.push(cell.value);entries.push(Entry.value(cell.value));}
     else if(cell.kind==='invalid'){invalid++;entries.push(Entry.Invalid);}
     else{missing++;entries.push(Entry.Missing);}
     cells[row][target]=cell;
    }
    preview.push(previewEquation(entries));
    matrix.push(values);
   }
   Object.assign(state,{cells,missing,invalid,blank,preview});
   const complete=missing===0&&invalid===0;
   state.large=complete&&!withinLiveBudget(matrix);
   return complete?matrix:null;
  }
  showResult(matrix){
   const state=this.state,dimension=state.dimension;
   let solution;
   try{solution=solve(matrix);}catch(_){return;}
   const text=format(solution);
   Object.assign(state,{copied:false,stale:false,solved:matrix,thinking:null,stepsCopied:false,stepsCopyFailed:false});
   if(solution.kind==='unique'){
    state.note=NOTE_UNIQUE;
    state.outcome={kind:'unique',values:Array.from({length:dimension},(_,row)=>{const value=solution.rref[row]?.[dimension];return value===undefined?'':String(value);})};
   }else if(solution.kind==='infinite'){
    state.note=NOTE_INFINITE;
    const start=text.indexOf('\n\n');
    state.outcome={kind:'infinite',general:start<0?text:text.slice(start+2)};
   }else{
    state.note=NOTE_NONE;
    state.outcome={kind:'none',general:GENERAL_NONE};
   }
   state.lastResult=text;
  }
  clearOutcome(){
   Object.assign(this.state,{outcome:{kind:'empty'},stale:false,lastResult:null,note:NOTE_DEFAULT,copied:false,solved:null,thinking:null,stepsCopied:false,stepsCopyFailed:false});
  }
  settleCard(matrix,live){
   const state=this.state;
   if(matrix&&!(live&&state.large))this.showResult(matrix);
   else if(live&&state.outcome.kind!=='empty')Object.assign(state,{stale:true,note:NOTE_STALE,copied:false,thinking:null});
   else this.clearOutcome();
  }
  edit(row,column,text){
   text=String(text);
   if(!(row>=0&&row<3&&column>=0&&column<4)||this.input(row,column)===text)return false;
   const state=this.state,wasMissing=this.cell(row,column)?.kind==='missing';
   this.grid()[row][column]=text;
   state.notice=null;
   let matrix=this.analyze(false);
   const now=this.cell(row,column)?.kind;
   if(state.revealed[row][column]&&(wasMissing||now==='value'||now==='missing')){
    state.revealed[row][column]=false;
    matrix=this.analyze(false);
   }
   this.settleCard(matrix,true);
   return true;
  }
  setInput(row,column,text){this.edit(row,column,text);}
  storeInput(row,column,text){
   const values=this.grid()[row];
   if(!values||!(column>=0&&column<4))return false;
   if(values[column]!==text)values[column]=String(text);
   return true;
  }
  focus(){return this.state.focus;}
  setFocus(field){
   const state=this.state;
   if(sameField(state.focus,field))return false;
   state.focus=field?[field[0],field[1]]:null;
   const missing=state.missing,invalid=state.invalid;
   this.analyze(false);
   return missing!==state.missing||invalid!==state.invalid;
  }
  idle(){
   const state=this.state;
   let changed=false;
   if(state.stale){this.clearOutcome();changed=true;}
   for(const [row,column] of this.visibleFields()){
    const typing=sameField(state.focus,[row,column])&&looksIncomplete(this.grid()[row][column]);
    if(this.cell(row,column)?.kind==='invalid'&&!state.revealed[row][column]&&!typing){state.revealed[row][column]=true;changed=true;}
   }
   if(changed)this.analyze(false);
   return changed;
  }
  leave(row,column){
   const state=this.state;
   let changed=false;
   if(sameField(state.focus,[row,column])){state.focus=null;this.analyze(false);}
   if(this.cell(row,column)?.kind==='invalid'&&!state.revealed[row][column]){state.revealed[row][column]=true;changed=true;}
   if(state.stale){this.clearOutcome();changed=true;}
   if(changed)this.analyze(false);
   return changed;
  }
  helper(){
   const shown=this.shownError();
   if(shown){
    const [row,column]=shown,cell=this.cell(row,column);
    return `Equação ${row+1}, ${termName(column)}: ${cell?.kind==='invalid'?cell.error.message:PARSE_MESSAGES.Empty}`;
   }
   const notice=this.state.notice;
   if(!notice)return this.focusedReading()??DEFAULT_HELPER;
   if(notice.kind==='example')return EXAMPLE_HELPER;
   if(notice.kind==='author')return AUTHOR_HELPER;
   if(notice.kind==='methodNeedsSystem')return METHOD_NEEDS_SYSTEM[notice.method];
   return ONLY_THIS_FIELD;
  }
  focusedReading(){
   const focus=this.state.focus;
   if(!focus||!usesExtension(trim(this.grid()[focus[0]]?.[focus[1]]??'')))return null;
   const cell=this.cell(focus[0],focus[1]);
   return cell?.kind==='value'?reading(cell.value):null;
  }
  helperIsReading(){return this.shownError()===null&&this.state.notice===null&&this.focusedReading()!==null;}
  notice(){return this.state.notice;}
  problem(row,column){const kind=this.cell(row,column)?.kind;return kind==='missing'||kind==='typing'||kind==='invalid';}
  shownError(){
   const state=this.state,revealed=([row,column])=>state.revealed[row][column]&&this.problem(row,column);
   if(state.focus&&revealed(state.focus))return state.focus;
   return this.visibleFields().find(revealed)??null;
  }
  helperIsError(){return this.shownError()!==null;}
  fieldHasError(row,column){return layoutVisible(this.state.dimension,row,column)&&this.state.revealed[row][column]&&this.problem(row,column);}
  errorField(){return this.shownError();}
  rowComplete(row){
   const dimension=this.state.dimension;
   if(row>=dimension)return false;
   for(let column=0;column<=dimension;column++)if(this.cell(row,inputColumn(dimension,column))?.kind!=='value')return false;
   return true;
  }
  outcome(){return this.state.outcome;}
  isEmpty(){return this.state.outcome.kind==='empty';}
  isStale(){return this.state.stale;}
  card(){return this.state.outcome.kind!=='empty'?{kind:'result',stale:this.state.stale}:{kind:'waiting',waiting:this.waiting()};}
  waiting(){
   const state=this.state;
   if(state.blank)return {kind:'empty'};
   if(state.missing>0)return {kind:'missing',missing:state.missing,invalid:state.invalid};
   if(state.invalid>0)return {kind:'invalid'};
   if(state.large)return {kind:'large'};
   return {kind:'empty'};
  }
  waitingTitle(){
   const card=this.card();
   if(card.kind==='result')return null;
   const waiting=card.waiting;
   if(waiting.kind==='empty')return EMPTY_TITLE;
   if(waiting.kind==='missing')return waiting.invalid>0?missingTitle(waiting.missing)+invalidSuffix(waiting.invalid):missingTitle(waiting.missing);
   if(waiting.kind==='invalid')return INVALID_TITLE;
   return LARGE_TITLE;
  }
  waitingDescription(){
   const card=this.card();
   if(card.kind==='result')return null;
   return {empty:EMPTY_DESCRIPTION,missing:MISSING_DESCRIPTION,invalid:INVALID_DESCRIPTION,large:LARGE_DESCRIPTION}[card.waiting.kind];
  }
  showsPreview(){const card=this.card();return card.kind==='waiting'&&card.waiting.kind!=='empty';}
  preview(){return this.state.preview;}
  previewSpoken(){
   const names=wanted=>this.visibleFields().filter(([row,column])=>{const cell=this.cell(row,column);return !!cell&&wanted(cell.kind);}).map(([row,column])=>accessibleInputName(row,column).toLowerCase());
   const missing=names(kind=>kind==='missing'||kind==='typing'),invalid=names(kind=>kind==='invalid'),parts=[];
   if(missing.length)parts.push(`Faltam: ${missing.join('; ')}.`);
   if(invalid.length)parts.push(`Inválidos: ${invalid.join('; ')}.`);
   return parts.join(' ');
  }
  title(){return {empty:null,unique:TITLE_UNIQUE,infinite:TITLE_INFINITE,none:TITLE_NONE}[this.state.outcome.kind];}
  description(){return {empty:null,unique:DESCRIPTION_UNIQUE,infinite:DESCRIPTION_INFINITE,none:DESCRIPTION_NONE}[this.state.outcome.kind];}
  values(){return this.state.outcome.kind==='unique'?this.state.outcome.values:[];}
  general(){const outcome=this.state.outcome;return outcome.kind==='infinite'||outcome.kind==='none'?outcome.general:null;}
  lastResult(){return this.state.stale?null:this.state.lastResult;}
  note(){return this.state.note;}
  copyEnabled(){return this.state.lastResult!==null&&!this.state.stale;}
  copyLabel(){return this.state.copied?COPIED_LABEL:COPY_LABEL;}
  thinkEnabled(){return this.state.solved!==null&&!this.state.stale;}
  method(){return this.state.method;}
  isThinking(){return this.state.thinking!==null;}
  walkthrough(){return this.state.thinking;}
  stepsText(){return this.state.thinking?walkthroughText(this.state.thinking):null;}
  stepsCopyLabel(){return this.state.stepsCopied?COPIED_STEPS_LABEL:COPY_STEPS_LABEL;}
  stepsCopyFailed(){return this.state.stepsCopyFailed;}
  thinkingStatus(){
   const walk=this.state.thinking;
   if(!walk)return '';
   return this.state.stepsCopyFailed?`${walk.method.name} · ${COPY_STEPS_FAILED}`:`${walk.method.name} · ${walk.steps.length} passos`;
  }
  openThinking(){
   const state=this.state;
   if(state.stale||!state.solved)return false;
   try{state.thinking=solver.walkthrough(state.solved,state.method);}catch(_){return false;}
   state.stepsCopied=false;
   state.stepsCopyFailed=false;
   return true;
  }
  closeThinking(){
   const state=this.state,was=state.thinking!==null;
   Object.assign(state,{stepsCopied:false,stepsCopyFailed:false,thinking:null});
   return was;
  }
  setMethod(method){
   const state=this.state;
   if(state.method===method&&state.thinking)return;
   state.method=method;
   if(state.thinking)this.openThinking();
  }
  copySteps(write){
   const text=this.stepsText();
   if(text===null)return;
   const ok=!!write(text);
   this.state.stepsCopied=ok;
   this.state.stepsCopyFailed=!ok;
  }
  announcement(){
   const state=this.state,outcome=state.outcome;
   if(state.stale)return null;
   if(outcome.kind==='unique')return `${TITLE_UNIQUE}: ${outcome.values.map((value,index)=>`${VARIABLES[index]} = ${spokenValue(value)}`).join(', ')}.`;
   if(outcome.kind==='infinite'){
    const parts=outcome.general.split(/\r?\n/).map(trim).filter(line=>line.includes(' = ')).map(line=>{const at=line.indexOf(' = ');return `${line.slice(0,at)} = ${spokenValue(line.slice(at+3))}`;});
    return `${TITLE_INFINITE}: ${parts.join(', ')}.`;
   }
   if(outcome.kind==='none')return `${TITLE_NONE}. ${DESCRIPTION_NONE}`;
   return null;
  }
  dimensionText(){return `${this.state.dimension} equações · ${this.state.dimension} incógnitas`;}
  fieldCount(){const dimension=this.state.dimension;return dimension*(dimension+1);}
  missing(){return this.state.missing;}
  invalidCount(){return this.state.invalid;}
  changeDimension(dimension){
   const state=this.state;
   state.dimension=dimension>=3?3:2;
   state.revealed=blankFlags();
   state.notice=null;
   if(state.focus&&!layoutVisible(state.dimension,state.focus[0],state.focus[1]))state.focus=null;
   this.settleCard(this.analyze(false),false);
  }
  solve(){
   const state=this.state;
   state.notice=null;
   const matrix=this.analyze(true);
   if(matrix){this.showResult(matrix);this.analyze(false);return null;}
   this.clearOutcome();
   let first=null;
   for(const [row,column] of this.visibleFields()){
    const kind=this.cell(row,column)?.kind;
    if(kind==='invalid'){state.revealed[row][column]=true;if(!first)first=[row,column];}
    else if((kind==='missing'||kind==='typing')&&!first){state.revealed[row][column]=true;first=[row,column];}
   }
   this.analyze(false);
   return first?fieldFocus(first[0],first[1]):null;
  }
  enter(row,column){
   const state=this.state;
   if(!layoutVisible(state.dimension,row,column))return null;
   const here=fieldFocus(row,column),parsed=parseCell(this.grid()[row][column]);
   if(parsed.kind==='error'&&parsed.error.kind!=='Empty'){
    state.notice=null;
    state.revealed[row][column]=true;
    this.analyze(false);
    return here;
   }
   this.analyze(true);
   if(state.missing===0)return this.solve();
   const fields=this.visibleFields(),start=Math.max(0,fields.findIndex(field=>sameField(field,[row,column])));
   let target=null;
   for(let offset=1;offset<=fields.length&&!target;offset++){
    const field=fields[(start+offset)%fields.length];
    if(this.problem(field[0],field[1]))target=field;
   }
   this.analyze(false);
   if(!target)return null;
   if(sameField(target,[row,column])){state.notice={kind:'onlyThisField'};return here;}
   return fieldFocus(target[0],target[1]);
  }
  neighbor(row,column,downward){
   const target=downward?row+1:row-1,dimension=this.state.dimension;
   return target>=0&&target<dimension&&layoutVisible(dimension,target,column)?fieldFocus(target,column):null;
  }
  firstIncomplete(){
   const field=this.visibleFields().find(([row,column])=>this.problem(row,column));
   return field?fieldFocus(field[0],field[1]):null;
  }
  requestMethod(method){
   const state=this.state;
   state.method=method;
   if(this.thinkEnabled()){
    state.thinking=null;
    if(this.openThinking())return {kind:'opened'};
   }
   const matrix=this.analyze(true);
   if(matrix){
    this.showResult(matrix);
    this.analyze(false);
    if(this.openThinking())return {kind:'opened'};
   }
   if(state.invalid>0){
    for(const [row,column] of this.visibleFields())if(this.cell(row,column)?.kind==='invalid')state.revealed[row][column]=true;
    state.notice=null;
    this.analyze(false);
    const shown=this.shownError();
    return {kind:'needsSystem',focus:shown?fieldFocus(shown[0],shown[1]):null};
   }
   this.analyze(false);
   state.notice={kind:'methodNeedsSystem',method};
   return {kind:'needsSystem',focus:this.firstIncomplete()};
  }
  fillExample(){
   const state=this.state,dimension=state.dimension,example=EXAMPLES[dimension];
   for(let row=0;row<dimension;row++)for(let column=0;column<=dimension;column++)this.storeInput(row,inputColumn(dimension,column),String(example[row][column]));
   state.revealed=blankFlags();
   this.settleCard(this.analyze(true),false);
   this.analyze(false);
   state.notice={kind:'example'};
   return null;
  }
  clear(){
   const state=this.state;
   state.inputs=[blankInputs(),blankInputs()];
   state.revealed=blankFlags();
   state.notice=null;
   this.analyze(false);
   this.clearOutcome();
   return fieldFocus(0,0,false);
  }
  copy(write){
   const state=this.state;
   if(state.stale||state.lastResult===null)return;
   if(write(state.lastResult))state.copied=true;
   else state.note=NOTE_COPY_FAILED;
  }
  authorUnavailable(){this.state.notice={kind:'author'};}
 }

 const api={Model,spokenValue,reading,termName,accessibleInputName,accessibleValueName,sizeName,rowNumber,missingTitle,invalidSuffix,inputColumn,layoutVisible,EXAMPLES,
  TEXTS:{TWO_LABEL,THREE_LABEL,EMPTY_TITLE,EMPTY_DESCRIPTION,MISSING_DESCRIPTION,INVALID_TITLE,INVALID_DESCRIPTION,LARGE_TITLE,LARGE_DESCRIPTION,ONLY_THIS_FIELD,KEYS,THINK_KEYS,THINK_FOOTER,DEFAULT_HELPER,EXAMPLE_HELPER,AUTHOR_HELPER,NOTE_DEFAULT,NOTE_UNIQUE,NOTE_INFINITE,NOTE_NONE,NOTE_STALE,NOTE_COPY_FAILED,COPY_LABEL,COPIED_LABEL,COPY_STEPS_LABEL,COPIED_STEPS_LABEL,COPY_STEPS_FAILED,TITLE_UNIQUE,TITLE_INFINITE,TITLE_NONE,DESCRIPTION_UNIQUE,DESCRIPTION_INFINITE,DESCRIPTION_NONE,GENERAL_NONE,SPOKEN_LIMIT,METHOD_NEEDS_SYSTEM,METHOD_LINK_NAMES}};
 if(node){module.exports=api;return;}
 if(typeof document==='undefined'||typeof BigInt==='undefined')return;

 const $=id=>document.getElementById(id);
 const STORAGE='sistemas-lineares.metodo',IDLE_MS=700;
 const model=new Model();
 const element=(tag,className,text)=>{const item=document.createElement(tag);if(className)item.className=className;if(text!==undefined&&text!==null)item.textContent=text;return item;};
 const setText=(item,text)=>{if(item.textContent!==text)item.textContent=text;};
 const setAttribute=(item,name,value)=>{
  if(value===null||value===undefined){if(item.hasAttribute(name))item.removeAttribute(name);return;}
  const text=String(value);
  if(item.getAttribute(name)!==text)item.setAttribute(name,text);
 };
 const icon=name=>{const svg=document.createElementNS('http://www.w3.org/2000/svg','svg'),use=document.createElementNS('http://www.w3.org/2000/svg','use');svg.setAttribute('aria-hidden','true');use.setAttribute('href',`#icon-${name}`);svg.append(use);return svg;};
 const reducedMotion=()=>typeof matchMedia==='function'&&matchMedia('(prefers-reduced-motion: reduce)').matches;
 const readMethod=()=>{try{const key=localStorage.getItem(STORAGE);if(METHODS.some(item=>item.key===key))return key;}catch(_){}return METHODS[0].key;};
 const saveMethod=key=>{try{localStorage.setItem(STORAGE,key);}catch(_){}};
 model.setMethod(readMethod());

 const help=$('help'),output=$('output'),live=$('live'),solution=$('solution');
 const inputs=[],fields=[],numbers=[],operators=[],rows=[];
 let composing=null,idleTimer=0,lastAnnounced=null,shownErrors=new Set(),cardKey='',cardKind='',liveToggle=false,opener=null,hoveredMethod=null,focusedMethod=null,fitFrame=0;

 function speak(text){
  if(!text)return;
  liveToggle=!liveToggle;
  live.textContent=liveToggle?text:`${text}\u200B`;
 }
 function resultAnnouncement(){
  const text=model.announcement();
  if(text)lastAnnounced=text;
  return text;
 }
 function revealedFields(){
  const revealed=[];
  for(let row=0;row<3;row++)for(let column=0;column<4;column++)if(model.fieldHasError(row,column))revealed.push([row,column]);
  return revealed;
 }
 function fieldError(row,column){
  const cell=model.cell(row,column);
  return `Equação ${row+1}, ${termName(column)}: ${cell?.kind==='invalid'?cell.error.message:PARSE_MESSAGES.Empty}`;
 }
 function update(spoken){
  render();
  const revealed=revealedFields(),fresh=revealed.find(([row,column])=>!shownErrors.has(`${row}-${column}`));
  shownErrors=new Set(revealed.map(([row,column])=>`${row}-${column}`));
  if(model.card().kind!=='result')lastAnnounced=null;
  const parts=(spoken||[]).filter(Boolean);
  if(fresh)parts.push(model.errorField()&&sameField(model.errorField(),fresh)?model.helper():fieldError(fresh[0],fresh[1]));
  if(parts.length)speak(parts.join(' '));
 }
 function restartIdle(){
  clearTimeout(idleTimer);
  idleTimer=setTimeout(settle,IDLE_MS);
 }
 function stopIdle(){
  clearTimeout(idleTimer);
  idleTimer=0;
 }
 function settle(){
  idleTimer=0;
  model.idle();
  const text=model.announcement();
  const spoken=text&&text!==lastAnnounced?[text]:null;
  if(spoken)lastAnnounced=text;
  update(spoken);
 }
 function typed(row,column,text){
  if(!model.edit(row,column,text))return;
  restartIdle();
  update();
 }
 function finishComposition(){
  if(!composing)return;
  const {input,row,column,before}=composing;
  composing=null;
  model.storeInput(row,column,before);
  typed(row,column,input.value);
 }
 function focusField(focus){
  const input=focus&&inputs[focus.row]?.[focus.column];
  if(!input)return;
  input.focus();
  if(focus.selectAll)input.select();
 }
 function keydown(event,row,column){
  if(event.isComposing||event.keyCode===229)return;
  const plain=!event.shiftKey&&!event.ctrlKey&&!event.altKey&&!event.metaKey;
  if(!plain)return;
  if(event.key==='Enter'){
   event.preventDefault();
   stopIdle();
   const focus=model.enter(row,column);
   if(focus){update(model.notice()?.kind==='onlyThisField'?[model.helper()]:null);focusField(focus);return;}
   const card=model.card();
   update(card.kind==='result'&&!card.stale?[resultAnnouncement()]:null);
   return;
  }
  if(event.key==='ArrowUp'||event.key==='ArrowDown'){
   event.preventDefault();
   const focus=model.neighbor(row,column,event.key==='ArrowDown');
   if(focus)focusField(focus);
  }
 }
 function bind(input,row,column){
  input.addEventListener('focus',()=>{model.setFocus([row,column]);update();});
  input.addEventListener('blur',()=>{finishComposition();model.leave(row,column);update();});
  input.addEventListener('compositionstart',()=>{composing={input,row,column,before:model.input(row,column)};stopIdle();});
  input.addEventListener('compositionend',finishComposition);
  input.addEventListener('input',event=>{
   if(event.isComposing&&composing){model.storeInput(row,column,input.value);return;}
   typed(row,column,input.value);
  });
  input.addEventListener('keydown',event=>keydown(event,row,column));
 }
 function buildFields(){
  const equations=$('equations');
  for(let row=0;row<3;row++){
   const line=element('div','equation'),number=element('span','number',rowNumber(row));
   number.setAttribute('aria-hidden','true');
   line.append(number);
   inputs[row]=[];fields[row]=[];operators[row]=[];
   for(let column=0;column<4;column++){
    const host=element('label','field'),input=element('input');
    input.type='text';
    input.id=`a-${row}-${column}`;
    input.autocomplete='off';
    input.spellcheck=false;
    input.setAttribute('inputmode','text');
    input.setAttribute('autocapitalize','off');
    input.setAttribute('autocorrect','off');
    input.setAttribute('enterkeyhint','next');
    input.setAttribute('aria-label',accessibleInputName(row,column));
    input.setAttribute('aria-describedby','field-description');
    host.append(input);
    if(column<3){const suffix=element('span','suffix',VARIABLES[column]);suffix.setAttribute('aria-hidden','true');host.append(suffix);}
    line.append(host);
    inputs[row][column]=input;
    fields[row][column]=host;
    if(column<3){const operator=element('span','operator');operator.setAttribute('aria-hidden','true');line.append(operator);operators[row][column]=operator;}
    bind(input,row,column);
   }
   numbers[row]=number;
   rows[row]=line;
   equations.append(line);
  }
 }
 function keycaps(container,items){
  container.replaceChildren(...items.map(([key,action])=>{const item=element('span','key');item.append(element('kbd','',key),element('span','key-action',action));return item;}));
 }
 function renderAction(button,enabled,tip){
  const description=enabled?tip:'tip-disabled';
  setAttribute(button,'aria-disabled',enabled?null:'true');
  setAttribute(button,'aria-describedby',description);
  setAttribute(button,'title',$(description).textContent);
 }
 function renderCard(){
  const card=model.card(),stale=card.kind==='result'&&card.stale;
  output.classList.toggle('stale',stale);
  const key=JSON.stringify([card.kind,card.kind==='waiting'?card.waiting:null,model.title(),model.values(),model.general(),model.waitingTitle(),model.preview(),model.previewSpoken()]);
  if(key===cardKey)return;
  cardKey=key;
  const appearing=card.kind==='result'&&cardKind!=='result';
  cardKind=card.kind;
  const children=[];
  let spoken=null;
  if(card.kind==='result'){
   output.className=`results${stale?' stale':''}`;
   const title=element('h2','card-title',model.title()),description=element('p','description',model.description());
   title.id='card-title';
   description.id='card-description';
   children.push(title,description);
   const values=model.values();
   if(values.length)values.forEach((value,index)=>{const tile=element('p','value');tile.append(element('span','variable',VARIABLES[index]),element('span','sr-only',' = '),element('span','answer',value));children.push(tile);});
   else children.push(element('pre','general',model.general()));
  }else{
   output.className=model.showsPreview()?'empty waiting':'empty';
   if(model.showsPreview()){
    const preview=element('div','preview');
    preview.setAttribute('aria-hidden','true');
    preview.append(...model.preview().map(line=>element('p','preview-line',line)));
    children.push(preview);
   }else{
    const art=icon('matrix');
    art.classList.add('empty-icon');
    children.push(art);
   }
   const title=element('h2','card-title',model.waitingTitle()),description=element('p','card-description',model.waitingDescription());
   title.id='card-title';
   description.id='card-description';
   children.push(title,description);
   if(model.showsPreview()){spoken=element('p','sr-only',model.previewSpoken());spoken.id='card-spoken';children.push(spoken);}
  }
  output.replaceChildren(...children);
  if(card.kind==='result')fitScrollers();
  setAttribute(output,'aria-describedby',spoken?'card-description card-spoken':'card-description');
  if(appearing&&!reducedMotion()){output.classList.remove('appear');void output.offsetWidth;output.classList.add('appear');}
 }
 function renderSummary(){
  const card=model.card(),values=model.values();
  let text=model.waitingTitle();
  if(card.kind==='result')text=values.length?values.map((value,index)=>`${VARIABLES[index]} = ${abbreviate(value)}`).join(' · '):model.title();
  setText($('summary-text'),text);
  const summary=$('summary');
  summary.classList.toggle('result',card.kind==='result');
  summary.classList.toggle('stale',card.kind==='result'&&card.stale);
 }
 function renderFooter(){
  const thinking=!$('think').hidden,tipMethod=thinking?null:hoveredMethod??focusedMethod;
  $('footer-methods').hidden=thinking;
  $('think-footer').hidden=!thinking;
  const tip=$('footer-tip');
  tip.hidden=!tipMethod;
  setText(tip,tipMethod?$(`tip-${tipMethod}`).textContent:'');
  $('keys').hidden=thinking||!!tipMethod;
  $('think-keys').hidden=!thinking;
 }
 function render(){
  const dimension=model.dimension(),hint=model.missing()===0?'done':'next';
  for(const size of [2,3]){
   const button=$(`size${size}`);
   setAttribute(button,'aria-pressed',dimension===size);
   setAttribute(button,'aria-label',sizeName(size,dimension===size));
  }
  setText($('size-info'),model.dimensionText());
  for(let row=0;row<3;row++){
   rows[row].hidden=row>=dimension;
   numbers[row].classList.toggle('complete',model.rowComplete(row));
   for(let column=0;column<4;column++){
    const input=inputs[row][column],host=fields[row][column],value=model.input(row,column),error=model.fieldHasError(row,column);
    host.hidden=!layoutVisible(dimension,row,column);
    if(input.value!==value&&!(composing&&composing.input===input))input.value=value;
    host.classList.toggle('error',error);
    setAttribute(input,'aria-invalid',error?'true':null);
    setAttribute(input,'aria-errormessage',error?'help':null);
    setAttribute(input,'aria-describedby',error?'help':'field-description');
    setAttribute(input,'enterkeyhint',hint);
   }
   for(let column=0;column<3;column++){
    operators[row][column].hidden=column>=dimension;
    setText(operators[row][column],column+1===dimension?'=':'+');
   }
  }
  setText(help,model.helper());
  help.classList.toggle('error',model.helperIsError());
  help.classList.toggle('reading',model.helperIsReading());
  renderCard();
  setText($('result-note'),model.note());
  renderAction($('think-open'),model.thinkEnabled(),'tip-think');
  renderAction($('copy'),model.copyEnabled(),'tip-copy');
  setText($('copy-label'),model.copyLabel());
  renderSummary();
  renderFooter();
 }
 async function copyText(value){
  try{if(navigator.clipboard?.writeText){await navigator.clipboard.writeText(value);return true;}}catch(_){}
  const focused=document.activeElement,area=document.createElement('textarea');
  area.value=value;area.setAttribute('readonly','');area.style.position='fixed';area.style.top='0';area.style.opacity='0';
  document.body.append(area);area.select();
  let ok=false;
  try{ok=document.execCommand('copy');}catch(_){}
  area.remove();
  focused?.focus?.({preventScroll:true});
  return ok;
 }

 const methodButtons=METHODS.map(item=>{
  const button=element('button','segment',item.label);
  button.type='button';
  button.id=`method-${item.key}`;
  if(item.name!==item.label)button.setAttribute('aria-label',item.name);
  button.addEventListener('click',()=>chooseMethod(item.key));
  return button;
 });
 $('methods').append(...methodButtons);
 const footerLinks=METHODS.map(item=>{
  const link=element('button','footer-link',item.label);
  link.type='button';
  link.id=`link-${item.key}`;
  link.setAttribute('aria-label',METHOD_LINK_NAMES[item.key]);
  link.setAttribute('aria-describedby',`tip-${item.key}`);
  link.addEventListener('click',()=>requestMethod(item.key,link));
  link.addEventListener('mouseenter',()=>{hoveredMethod=item.key;renderFooter();});
  link.addEventListener('mouseleave',()=>{hoveredMethod=null;renderFooter();});
  link.addEventListener('focus',()=>{focusedMethod=item.key;renderFooter();});
  link.addEventListener('blur',()=>{focusedMethod=null;renderFooter();});
  return link;
 });
 $('footer-links').append(...footerLinks);
 for(const [id,tip] of [['example','tip-example'],['clear','tip-clear']])$(id).title=$(tip).textContent;
 setText($('think-footer'),THINK_FOOTER);
 keycaps($('keys'),KEYS);
 keycaps($('think-keys'),THINK_KEYS);

 function matrixNode(view){
  const scroll=element('div','matrix-scroll'),figure=element('div','matrix-figure'),grid=element('div','matrix'),cells=matrixCells(view),count=cells.length,columns=cells[0].length,marked=row=>view.markedRows.includes(row),lines=matrixLines(view).map(line=>line.replace(/\s+/g,' '));
  figure.setAttribute('role','img');
  figure.setAttribute('aria-label',view.label!=null?`${lines[0]} ${lines.slice(1).join(', ')}`:lines.join(', '));
  if(view.label!=null)figure.append(element('span','matrix-label',`${view.label} =`));
  grid.style.gridTemplateColumns=`repeat(${columns},auto)`;
  cells.forEach((row,r)=>row.forEach((value,c)=>{
   const cell=element('span','cell',value),classes=[];
   if(view.augmented&&c===columns-1)classes.push('bar');
   if(marked(r)){classes.push('mark');if(c===0||c===columns-1){const side=c===0?'l':'r';if(!marked(r-1))classes.push(`t${side}`);if(!marked(r+1))classes.push(`b${side}`);}}
   if(view.markedColumn===c){classes.push('mark');if(r===0)classes.push('tl','tr');if(r===count-1)classes.push('bl','br');}
   if(view.pivot&&view.pivot[0]===r&&view.pivot[1]===c)classes.push('pivot');
   if(classes.length)cell.classList.add(...classes);
   grid.append(cell);
  }));
  figure.append(grid);
  scroll.append(figure);
  return scroll;
 }
 function stepNode(step,index){
  const item=element('li','step'),title=element('h3','step-title'),badge=element('span','step-number',String(index+1)),math=step.lines.length?element('div','math'):null,matrix=step.matrix?matrixNode(step.matrix):null;
  badge.setAttribute('aria-hidden','true');
  title.append(badge,element('span','sr-only',`Passo ${index+1}: `),step.title);
  item.append(title,element('p','reasoning',step.reasoning));
  if(math)step.lines.forEach(line=>math.append(element('p',`math-line ${line.kind}`,line.text)));
  for(const part of step.matrixFirst?[matrix,math]:[math,matrix])if(part)item.append(part);
  return item;
 }
 function fitScrollers(){
  cancelAnimationFrame(fitFrame);
  fitFrame=requestAnimationFrame(()=>document.querySelectorAll('.matrix-scroll,.value .answer').forEach(scroller=>{if(scroller.scrollWidth>scroller.clientWidth+1)scroller.tabIndex=0;else scroller.removeAttribute('tabindex');}));
 }
 function renderThinkingStatus(){
  setText($('think-info'),model.thinkingStatus());
  setText($('copy-steps-label'),model.stepsCopyLabel());
 }
 function renderThinking(){
  const walk=model.walkthrough();
  if(!walk)return;
  methodButtons.forEach((button,index)=>setAttribute(button,'aria-pressed',METHODS[index].key===walk.method.key));
  renderThinkingStatus();
  $('think-system').replaceChildren(...walk.system.map(line=>element('p','',line)));
  $('think-answer').replaceChildren(...walk.answer.map(line=>element('p','',line)));
  setText($('think-classification'),walk.classification);
  $('steps').replaceChildren(...walk.steps.map(stepNode));
  fitScrollers();
 }
 function showThinking(from){
  opener=from;
  renderThinking();
  $('solver').hidden=true;
  $('hero-solver').hidden=true;
  $('hero-think').hidden=false;
  $('think').hidden=false;
  renderFooter();
  fitScrollers();
  window.scrollTo(0,0);
  $('think-title').focus({preventScroll:true});
 }
 function closeThinking(){
  model.closeThinking();
  $('think').hidden=true;
  $('hero-think').hidden=true;
  $('hero-solver').hidden=false;
  $('solver').hidden=false;
  render();
  const target=opener&&opener.isConnected?opener:$('think-open');
  opener=null;
  target.focus();
 }
 function chooseMethod(key){
  if(key===model.method()&&model.isThinking())return;
  model.setMethod(key);
  saveMethod(key);
  renderThinking();
 }
 function requestMethod(key,from){
  stopIdle();
  const resolving=!model.thinkEnabled(),request=model.requestMethod(key);
  saveMethod(key);
  if(request.kind==='opened'){
   update(resolving?[resultAnnouncement()]:null);
   showThinking(from);
   return;
  }
  update(model.notice()?.kind==='methodNeedsSystem'?[model.helper()]:null);
  focusField(request.focus);
 }
 function changeDimension(dimension){
  stopIdle();
  model.changeDimension(dimension);
  const result=resultAnnouncement();
  update([`${model.dimensionText()}.`,result??`${model.waitingTitle()}.`]);
 }

 buildFields();
 $('size2').addEventListener('click',()=>changeDimension(2));
 $('size3').addEventListener('click',()=>changeDimension(3));
 $('system-form').addEventListener('submit',event=>{
  event.preventDefault();
  stopIdle();
  const focus=model.solve();
  if(focus){update();focusField(focus);return;}
  update([resultAnnouncement()]);
 });
 $('example').addEventListener('click',()=>{
  stopIdle();
  model.fillExample();
  update([EXAMPLE_HELPER,resultAnnouncement()]);
 });
 $('clear').addEventListener('click',()=>{
  stopIdle();
  const focus=model.clear();
  lastAnnounced=null;
  update();
  focusField(focus);
 });
 $('copy').addEventListener('click',async()=>{
  if(!model.copyEnabled())return;
  const text=model.lastResult(),ok=await copyText(text);
  if(model.lastResult()!==text)return;
  model.copy(()=>ok);
  render();
 });
 $('think-open').addEventListener('click',()=>{
  if(!model.thinkEnabled())return;
  if(model.openThinking())showThinking($('think-open'));
 });
 $('back').addEventListener('click',closeThinking);
 $('copy-steps').addEventListener('click',async()=>{
  const text=model.stepsText();
  if(text===null)return;
  const ok=await copyText(text);
  if(model.stepsText()!==text)return;
  model.copySteps(()=>ok);
  renderThinkingStatus();
 });
 $('summary-go').addEventListener('click',()=>{
  solution.scrollIntoView({block:'start',behavior:reducedMotion()?'auto':'smooth'});
  solution.focus({preventScroll:true});
 });
 output.addEventListener('animationend',()=>output.classList.remove('appear'));
 document.addEventListener('keydown',event=>{
  if(event.key==='Escape'&&!$('think').hidden&&!event.isComposing){event.preventDefault();closeThinking();}
 });
 window.addEventListener('resize',fitScrollers);
 update();
})(typeof globalThis!=='undefined'?globalThis:this);
