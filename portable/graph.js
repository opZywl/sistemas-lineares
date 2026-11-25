(function(root){
 'use strict';
 const LIMIT=1e100,MARGIN=1.3,MIN_HALF=2.5,BOX_HEIGHT=0.6,APPEAR=0.6,MORPH=0.35,VANISH=0.25,VIEW=0.45,POP=0.45,PULSE=1.2,PULSE_DELAY=0.15,PULSE_GAP=0.4,PULSES=3,MARCH=2.4,MARCH_SPEED=18,LIGHT_UP=0.5,LABEL_LIMIT=7,TEXT_HEIGHT=14,CHAR_WIDTH=7;
 const SX_X=Math.SQRT1_2,SX_Z=-Math.SQRT1_2,SY_X=0.29883623873011983,SY_Y=-0.9063077870366499,SY_Z=0.29883623873011983,DEPTH_X=0.6408563820557885,DEPTH_Y=0.42261826174069944,DEPTH_Z=0.6408563820557885;
 const INTRO_TWO='Cada equação é uma reta no plano. Preencha uma equação e ela aparece aqui, na cor do seu número.';
 const INTRO_THREE='Cada equação é um plano no espaço. Preencha uma equação e ele aparece aqui, na cor do seu número.';
 const WAITING='Pressione Enter em um dos campos para resolver e marcar a solução no gráfico.';
 const HUGE='Os valores são grandes demais para desenhar o gráfico; a resposta exata continua no resultado.';
 const COINCIDENT_LINES='As retas coincidem: cada ponto delas é uma solução.';
 const SHARED_LINE='Os planos se cruzam numa mesma reta: cada ponto dela é uma solução.';
 const COINCIDENT_PLANES='Os planos coincidem: cada ponto deles é uma solução.';
 const EVERYTHING_TWO='Todas as equações valem para qualquer ponto (0 = 0): o plano inteiro é solução.';
 const EVERYTHING_THREE='Todas as equações valem para qualquer ponto (0 = 0): o espaço inteiro é solução.';
 const PARALLEL_LINES='As retas são paralelas e nunca se cruzam: o sistema não tem solução.';
 const PARALLEL_PLANES='Os planos são paralelos e nunca se encontram: o sistema não tem solução.';
 const NO_COMMON_POINT='Não existe um ponto comum aos três planos: o sistema não tem solução.';
 const GRAPH_NAME='Gráfico do sistema';
 const NEVER=-Infinity;
 const PALETTE=Object.freeze({fill:'rgb(17,16,16)',stroke:'rgb(53,49,55)',t:'rgb(50,46,55)',g:'rgb(36,33,39)',a:'rgb(99,92,107)',l:'rgb(170,165,177)',p:'rgb(26,24,29)',e:'rgb(64,59,69)',r0:'rgb(206,169,243)',r1:'rgb(125,211,192)',r2:'rgb(247,178,140)',m:'rgb(243,240,247)',u:'rgb(170,165,177)',s:'rgb(243,240,247)',h:'rgb(206,169,243)',o:'rgb(243,240,247)',b:'rgb(17,16,16)'});
 const CONTRAST_PALETTE=Object.freeze({fill:'Canvas',stroke:'CanvasText',t:'GrayText',g:'GrayText',a:'CanvasText',l:'CanvasText',p:'Canvas',e:'CanvasText',r0:'Highlight',r1:'LinkText',r2:'CanvasText',m:'CanvasText',u:'CanvasText',s:'CanvasText',h:'Highlight',o:'CanvasText',b:'Canvas'});
 const inkColor=(ink,contrast)=>(contrast?CONTRAST_PALETTE:PALETTE)[ink];
 const rowInk=row=>row===0?'r0':row===1?'r1':'r2';

 const min=(first,second)=>first!==first?second:second!==second?first:first<second?first:second;
 const max=(first,second)=>first!==first?second:second!==second?first:first>second?first:second;
 function clamp(value,low,high){
  let result=value;
  if(result<low)result=low;
  if(result>high)result=high;
  return result;
 }
 const clamp01=value=>value!==value?1:clamp(value,0,1);
 const integer=value=>Math.ceil(value)||0;
 function totalCmp(first,second){
  if(first<second)return -1;
  if(first>second)return 1;
  if(first===second){
   if(first!==0)return 0;
   const negativeFirst=Object.is(first,-0),negativeSecond=Object.is(second,-0);
   return negativeFirst===negativeSecond?0:negativeFirst?-1:1;
  }
  const nanFirst=first!==first,nanSecond=second!==second;
  return nanFirst===nanSecond?0:nanFirst?1:-1;
 }

 const add=(first,second)=>[first[0]+second[0],first[1]+second[1],first[2]+second[2]];
 const sub=(first,second)=>[first[0]-second[0],first[1]-second[1],first[2]-second[2]];
 const scale=(vector,amount)=>[vector[0]*amount,vector[1]*amount,vector[2]*amount];
 const dot=(first,second)=>first[0]*second[0]+first[1]*second[1]+first[2]*second[2];
 const cross=(first,second)=>[first[1]*second[2]-first[2]*second[1],first[2]*second[0]-first[0]*second[2],first[0]*second[1]-first[1]*second[0]];
 const lerp3=(first,second,amount)=>[first[0]+(second[0]-first[0])*amount,first[1]+(second[1]-first[1])*amount,first[2]+(second[2]-first[2])*amount];

 function plane(coefficients,constant){
  const [a,b,c]=coefficients,length=Math.sqrt(a*a+b*b+c*c);
  const drawable=length>0&&Number.isFinite(length)&&Number.isFinite(constant);
  if(!drawable)return null;
  const offset=constant/length;
  if(Math.abs(offset)>LIMIT)return null;
  return {normal:[a/length,b/length,c/length],offset};
 }
 const foot=entry=>scale(entry.normal,entry.offset);

 const sameTriple=(first,second)=>first===second||(!!first&&!!second&&first[0]===second[0]&&first[1]===second[1]&&first[2]===second[2]);
 const samePlane=(first,second)=>first===second||(!!first&&!!second&&first.offset===second.offset&&sameTriple(first.normal,second.normal));
 const sameRow=(first,second)=>first.kind===second.kind&&(first.kind==='drawn'?samePlane(first.plane,second.plane):first.kind==='impossible'?first.constant===second.constant:true);
 const sameRows=(first,second)=>first.length===second.length&&first.every((row,index)=>sameRow(row,second[index]));
 const sameStrings=(first,second)=>first.length===second.length&&first.every((text,index)=>text===second[index]);
 const sameVerdict=(first,second)=>first.kind===second.kind&&(first.kind==='point'?sameTriple(first.at,second.at)&&sameStrings(first.coordinates,second.coordinates):first.kind==='nothing'?first.rank===second.rank:true);
 const sameSketch=(first,second)=>first.dimension===second.dimension&&sameRows(first.rows,second.rows)&&sameVerdict(first.verdict,second.verdict);
 const sameBounds=(first,second)=>sameTriple(first.center,second.center)&&sameTriple(first.half,second.half);
 const copyRow=row=>row.kind==='drawn'?{kind:'drawn',plane:{normal:row.plane.normal.slice(),offset:row.plane.offset}}:{...row};
 const copyVerdict=verdict=>verdict.kind==='point'?{kind:'point',at:verdict.at?verdict.at.slice():null,coordinates:verdict.coordinates.slice()}:{...verdict};
 const copySketch=sketch=>({dimension:sketch.dimension,rows:sketch.rows.map(copyRow),verdict:copyVerdict(sketch.verdict)});

 function blank(dimension){
  const size=clamp(dimension,2,3);
  return {dimension:size,rows:Array.from({length:size},()=>({kind:'blank'})),verdict:{kind:'pending'}};
 }
 const sketchPlane=(sketch,row)=>{const entry=sketch.rows[row];return entry&&entry.kind==='drawn'?entry.plane:null;};
 const sketchHuge=sketch=>sketch.rows.some(row=>row.kind==='huge');
 const sketchPoint=sketch=>sketch.verdict.kind==='point'?sketch.verdict.at:null;
 const sketchCoordinates=sketch=>sketch.verdict.kind==='point'?sketch.verdict.coordinates:[];

 function numbered(rows){
  if(!rows.length)return '';
  if(rows.length===1)return `a equação ${rows[0]}`;
  return `as equações ${rows.slice(0,-1).join(', ')} e ${rows[rows.length-1]}`;
 }
 function caption(sketch){
  const three=sketch.dimension===3,rows=sketch.rows;
  const picked=wanted=>rows.map((row,index)=>wanted(row)?index+1:0).filter(Boolean);
  if(sketchHuge(sketch))return HUGE;
  const verdict=sketch.verdict;
  switch(verdict.kind){
   case 'pending':{
    const missing=picked(row=>row.kind==='blank'||row.kind==='partial');
    if(!missing.length||missing.length===rows.length)return three?INTRO_THREE:INTRO_TWO;
    return `Preencha ${numbered(missing)} para ver onde ${three?'os planos':'as retas'} se cruzam.`;
   }
   case 'waiting':return WAITING;
   case 'point':{
    const label=`(${verdict.coordinates.join(', ')})`;
    if(!verdict.at)return `A solução é ${label}, longe demais para caber no gráfico.`;
    if(three)return `Os três planos se cruzam em ${label}: esse ponto é a solução do sistema.`;
    return `As retas se cruzam em ${label}: esse ponto é a solução do sistema.`;
   }
   case 'line':{
    if(three)return SHARED_LINE;
    const trivial=picked(row=>row.kind==='trivial'),drawn=picked(row=>row.kind==='drawn');
    if(trivial.length===1&&drawn.length===1)return `A equação ${trivial[0]} vale para qualquer ponto (0 = 0), então toda a reta da equação ${drawn[0]} é solução.`;
    return COINCIDENT_LINES;
   }
   case 'plane':{
    const drawn=picked(row=>row.kind==='drawn');
    if(drawn.length===1)return `As outras equações valem para qualquer ponto (0 = 0), então todo o plano da equação ${drawn[0]} é solução.`;
    return COINCIDENT_PLANES;
   }
   case 'everything':return three?EVERYTHING_THREE:EVERYTHING_TWO;
   default:{
    const index=rows.findIndex(row=>row.kind==='impossible');
    if(index>=0)return `A equação ${index+1} diz que 0 = ${rows[index].constant}, o que nunca acontece: o sistema não tem solução.`;
    if(!three)return PARALLEL_LINES;
    if(verdict.rank<=1)return PARALLEL_PLANES;
    return NO_COMMON_POINT;
   }
  }
 }

 const pt=(x,y)=>({x,y});
 const lerpPt=(from,to,amount)=>pt(from.x+(to.x-from.x)*amount,from.y+(to.y-from.y)*amount);
 function distance(from,to){
  const dx=to.x-from.x,dy=to.y-from.y;
  return Math.sqrt(dx*dx+dy*dy);
 }
 const item=(shape,ink,alpha)=>({shape,ink,alpha});
 const dots=(spacing,radius)=>({kind:'dots',spacing,radius});
 const segment=(from,to,width)=>({kind:'segment',from,to,width});
 const polygon=points=>({kind:'polygon',points});
 const outline=(points,width)=>({kind:'outline',points,width});
 const disc=(center,radius)=>({kind:'disc',center,radius});
 const ring=(center,radius,width)=>({kind:'ring',center,radius,width});
 const label=(at,text,horizontal,vertical)=>({kind:'text',at,text,horizontal,vertical});

 const quantize=value=>Math.floor(value*100+0.5);
 const describePoints=points=>points.map(point=>`${quantize(point.x)},${quantize(point.y)}`).join(' ');
 function describeShape(shape){
  switch(shape.kind){
   case 'dots':return `D${quantize(shape.spacing)},${quantize(shape.radius)}`;
   case 'segment':return `S${quantize(shape.from.x)},${quantize(shape.from.y)},${quantize(shape.to.x)},${quantize(shape.to.y)},${quantize(shape.width)}`;
   case 'polygon':return `P${describePoints(shape.points)}`;
   case 'outline':return `O${describePoints(shape.points)}/${quantize(shape.width)}`;
   case 'disc':return `C${quantize(shape.center.x)},${quantize(shape.center.y)},${quantize(shape.radius)}`;
   case 'ring':return `R${quantize(shape.center.x)},${quantize(shape.center.y)},${quantize(shape.radius)},${quantize(shape.width)}`;
   default:return `T${quantize(shape.at.x)},${quantize(shape.at.y)},${shape.horizontal}${shape.vertical},${shape.text}`;
  }
 }
 const describe=items=>items.map(entry=>`${entry.ink}${Math.floor(entry.alpha*1000+0.5)}:${describeShape(entry.shape)}`).join(';');

 const easeOut=value=>{const rest=1-clamp01(value);return 1-rest*rest*rest;};
 function easeInOut(value){
  const amount=clamp01(value);
  if(amount<0.5)return 4*amount*amount*amount;
  const rest=-2*amount+2;
  return 1-rest*rest*rest/2;
 }
 function easeOutBack(value){
  const amount=clamp01(value),c1=1.70158,c3=c1+1,rest=amount-1;
  return 1+c3*rest*rest*rest+c1*rest*rest;
 }

 function lerpPlane(from,to,amount){
  const flip=dot(from.normal,to.normal)<0,normal=flip?scale(to.normal,-1):to.normal,offset=flip?-to.offset:to.offset;
  const blended=lerp3(from.normal,normal,amount),length=Math.sqrt(dot(blended,blended));
  if(length<1e-9)return to;
  const mixed=from.offset+(offset-from.offset)*amount;
  return {normal:scale(blended,1/length),offset:mixed/length};
 }

 function nice(raw){
  if(raw!==raw||raw<=0||raw===Infinity)return 1;
  let power=1;
  while(power>raw)power/=10;
  while(power*10<=raw)power*=10;
  for(const multiple of [1,2,5])if(power*multiple>=raw)return power*multiple;
  return power*10;
 }

 const lerpBounds=(from,to,amount)=>({center:lerp3(from.center,to.center,amount),half:lerp3(from.half,to.half,amount)});
 const lowCorner=view=>sub(view.center,view.half);
 const highCorner=view=>add(view.center,view.half);

 function lineOf(first,second){
  const direction=cross(first.normal,second.normal),length=dot(direction,direction);
  if(length<1e-10)return null;
  const point=scale(add(scale(cross(second.normal,direction),first.offset),scale(cross(direction,first.normal),second.offset)),1/length);
  return [point,direction];
 }
 function solutionLine(dimension,planes){
  if(dimension===2){
   const entry=planes.find(Boolean);
   if(!entry)return null;
   return [foot(entry),[-entry.normal[1],entry.normal[0],0]];
  }
  for(let first=0;first<planes.length;first++)for(let second=first+1;second<planes.length;second++){
   const one=planes[first],two=planes[second];
   if(one&&two){
    const line=lineOf(one,two);
    if(line)return line;
   }
  }
  return null;
 }

 function bounds(sketch){
  const three=sketch.dimension===3,anchors=[[0,0,0]],planes=sketch.rows.map((_,row)=>sketchPlane(sketch,row));
  for(const entry of planes)if(entry)anchors.push(foot(entry));
  const point=sketchPoint(sketch);
  if(point)anchors.push(point);
  if(three&&sketch.verdict.kind==='line'){
   const line=solutionLine(3,planes);
   if(line)anchors.push(line[0]);
  }
  const low=anchors[0].slice(),high=anchors[0].slice();
  for(const anchor of anchors)for(let axis=0;axis<3;axis++){
   low[axis]=min(low[axis],anchor[axis]);
   high[axis]=max(high[axis],anchor[axis]);
  }
  const center=scale(add(low,high),0.5),extent=scale(sub(high,low),0.5);
  let half;
  if(three){
   const size=max(max(max(MIN_HALF,extent[0]*MARGIN),extent[2]*MARGIN),extent[1]*MARGIN/BOX_HEIGHT);
   half=[size,size*BOX_HEIGHT,size];
  }else half=[max(MIN_HALF,extent[0]*MARGIN),max(MIN_HALF,extent[1]*MARGIN),0];
  return {center:three?center:[center[0],center[1],0],half};
 }

 function clip(point,direction,low,high,axes){
  let start=-Infinity,end=Infinity;
  for(let axis=0;axis<axes;axis++){
   if(Math.abs(direction[axis])<1e-12){
    if(point[axis]<low[axis]||point[axis]>high[axis])return null;
    continue;
   }
   const first=(low[axis]-point[axis])/direction[axis],second=(high[axis]-point[axis])/direction[axis];
   start=max(start,min(first,second));
   end=min(end,max(first,second));
  }
  if(start!==start||end!==end||start>=end||!Number.isFinite(start)||!Number.isFinite(end))return null;
  return [add(point,scale(direction,start)),add(point,scale(direction,end))];
 }

 function pseudoAngle(x,y){
  if(y>=0)return x>=0?y/(x+y):1-x/(y-x);
  return x<0?2-y/(-x-y):3+x/(x-y);
 }

 function section(entry,low,high){
  const corner=index=>[index&1?high[0]:low[0],index&2?high[1]:low[1],index&4?high[2]:low[2]];
  const size=sub(high,low),tolerance=1e-9*dot(size,size),points=[];
  const push=candidate=>{
   const fresh=points.every(existing=>{const gap=sub(existing,candidate);return dot(gap,gap)>tolerance;});
   if(fresh)points.push(candidate);
  };
  for(const bit of [1,2,4])for(let index=0;index<8;index++){
   if(index&bit)continue;
   const start=corner(index),end=corner(index|bit),first=dot(entry.normal,start)-entry.offset,second=dot(entry.normal,end)-entry.offset;
   if(first===0)push(start);
   if(second===0)push(end);
   if((first<0&&second>0)||(first>0&&second<0))push(add(start,scale(sub(end,start),first/(first-second))));
  }
  if(points.length<3)return [];
  const count=points.length,center=scale(points.reduce((sum,point)=>add(sum,point),[0,0,0]),1/count),normal=entry.normal;
  const axis=Math.abs(normal[0])<=Math.abs(normal[1])&&Math.abs(normal[0])<=Math.abs(normal[2])?[1,0,0]:Math.abs(normal[1])<=Math.abs(normal[2])?[0,1,0]:[0,0,1];
  const projected=sub(axis,scale(normal,dot(axis,normal))),along=scale(projected,1/Math.sqrt(dot(projected,projected))),across=cross(normal,along);
  const keyed=points.map(point=>{const offset=sub(point,center);return [pseudoAngle(dot(offset,along),dot(offset,across)),point];});
  keyed.sort((first,second)=>totalCmp(first[0],second[0]));
  return keyed.map(entry=>entry[1]);
 }

 class Track{
  constructor(target=null){
   this.target=target;
   this.appear=NEVER;
   this.from=null;
   this.morph=NEVER;
   this.ghost=null;
   this.vanish=NEVER;
  }
  shown(now){
   if(this.target){
    const shownPlane=this.from&&now<this.morph+MORPH?lerpPlane(this.from,this.target,easeInOut((now-this.morph)/MORPH)):this.target;
    return {plane:shownPlane,progress:easeOut((now-this.appear)/APPEAR),alpha:1};
   }
   if(!this.ghost)return null;
   const fade=(now-this.vanish)/VANISH;
   return fade<1?{plane:this.ghost.plane,progress:this.ghost.progress,alpha:1-clamp01(fade)}:null;
  }
  busy(now){
   if(this.target)return now<this.appear+APPEAR||(this.from!==null&&now<this.morph+MORPH);
   return this.ghost!==null&&now<this.vanish+VANISH;
  }
  retarget(target,now,animate){
   if(samePlane(this.target,target))return;
   const shown=this.shown(now);
   if(!animate){
    Object.assign(this,new Track(target));
    return;
   }
   if(target){
    if(shown){
     if(!this.target)this.appear=NEVER;
     this.from=shown.plane;
     this.morph=now;
    }else{
     this.appear=now;
     this.from=null;
    }
    this.ghost=null;
    this.target=target;
   }else{
    this.ghost=shown?{plane:shown.plane,progress:shown.progress}:null;
    this.vanish=now;
    this.target=null;
    this.from=null;
   }
  }
 }

 class Marker{
  constructor(target=null){
   this.target=target;
   this.from=null;
   this.moved=NEVER;
   this.popped=NEVER;
   this.pulsed=NEVER;
   this.ghost=null;
   this.vanish=NEVER;
  }
  shown(now){
   if(this.target){
    const at=this.from&&now<this.moved+MORPH?lerp3(this.from,this.target,easeInOut((now-this.moved)/MORPH)):this.target;
    const grow=(now-this.popped)/POP;
    return {at,size:easeOutBack(grow),alpha:1,reveal:clamp01(grow)};
   }
   if(!this.ghost)return null;
   const fade=(now-this.vanish)/VANISH;
   return fade<1?{at:this.ghost,size:1,alpha:1-clamp01(fade),reveal:1-clamp01(fade)}:null;
  }
  rings(now){
   if(!this.target)return [];
   const rings=[];
   for(let index=0;index<PULSES;index++){
    const start=this.pulsed+PULSE_DELAY+index*PULSE_GAP,progress=(now-start)/PULSE;
    if(progress>=0&&progress<1){
     const rest=1-progress;
     rings.push([easeOut(progress),0.6*rest*rest]);
    }
   }
   return rings;
  }
  busy(now,contrast){
   if(this.target)return now<this.popped+POP||(this.from!==null&&now<this.moved+MORPH)||(!contrast&&now<this.pulsed+PULSE_DELAY+(PULSES-1)*PULSE_GAP+PULSE);
   return this.ghost!==null&&now<this.vanish+VANISH;
  }
  retarget(target,now,animate){
   if(sameTriple(this.target,target))return;
   const shown=this.shown(now);
   if(!animate){
    Object.assign(this,new Marker(target));
    return;
   }
   if(target){
    if(shown){
     if(!this.target)this.popped=NEVER;
     this.from=shown.at;
     this.moved=now;
    }else{
     this.from=null;
     this.popped=now;
    }
    this.pulsed=now;
    this.ghost=null;
    this.target=target;
   }else{
    this.ghost=shown?shown.at:null;
    this.vanish=now;
    this.target=null;
    this.from=null;
   }
  }
 }

 function textArea(at,horizontal,vertical,wide,tall){
  const left=horizontal==='s'?at.x:horizontal==='m'?at.x-wide/2:at.x-wide;
  const top=vertical==='s'?at.y:vertical==='m'?at.y-tall/2:at.y-tall;
  return {left,top,right:left+wide,bottom:top+tall};
 }
 const spanArea=(from,to,pad)=>({left:min(from.x,to.x)-pad,top:min(from.y,to.y)-pad,right:max(from.x,to.x)+pad,bottom:max(from.y,to.y)+pad});
 const grow=(area,pad)=>({left:area.left-pad,top:area.top-pad,right:area.right+pad,bottom:area.bottom+pad});
 const meets=(first,second)=>first.left<second.right&&second.left<first.right&&first.top<second.bottom&&second.top<first.bottom;
 function crossed(area,line){
  const side=(x,y)=>line[0]*x+line[1]*y-line[2];
  const corners=[side(area.left,area.top),side(area.right,area.top),side(area.left,area.bottom),side(area.right,area.bottom)];
  const above=corners.every(value=>value>0),below=corners.every(value=>value<0);
  return !above&&!below;
 }
 const blocks=(obstacles,area)=>obstacles.lines.some(line=>crossed(area,line))||obstacles.areas.some(other=>meets(area,other));
 function pick(candidates,wide,tall,unit,obstacles){
  const found=candidates.find(([at,horizontal,vertical])=>!blocks(obstacles,grow(textArea(at,horizontal,vertical,wide,tall),unit)));
  return found||candidates[0]||null;
 }

 function xLabel(center,axis,text,stage,obstacles){
  const count=Array.from(text).length;
  if(count>LABEL_LIMIT)return null;
  const unit=stage.unit,tall=TEXT_HEIGHT*unit,wide=count*CHAR_WIDTH*unit,gap=4*unit;
  const below=[axis+gap+tall<=stage.height,axis+gap,'s'],above=[axis-gap-tall>=0,axis-gap,'e'];
  const [far,near]=center.y<axis?[below,above]:[above,below];
  const middle=stage.width>wide+4*unit?clamp(center.x,wide/2+2*unit,stage.width-wide/2-2*unit):stage.width/2;
  const east=center.x+5*unit+wide<=stage.width-2*unit,west=center.x-5*unit-wide>=2*unit;
  const candidates=[];
  if(far[0])candidates.push([pt(middle,far[1]),'m',far[2]]);
  for(const [fits,y,vertical] of [far,near]){
   if(!fits)continue;
   if(east)candidates.push([pt(center.x+5*unit,y),'s',vertical]);
   if(west||!east)candidates.push([pt(center.x-5*unit,y),'e',vertical]);
  }
  return pick(candidates,wide,tall,unit,obstacles);
 }

 function yLabel(center,axis,text,stage,obstacles){
  const count=Array.from(text).length;
  if(count>LABEL_LIMIT)return null;
  const unit=stage.unit,tall=TEXT_HEIGHT*unit,wide=count*CHAR_WIDTH*unit,gap=5*unit;
  const west=[axis-gap-wide>=0,axis-gap,'e'],east=[axis+gap+wide<=stage.width,axis+gap,'s'];
  const [far,near]=center.x>axis?[west,east]:[east,west];
  const middle=stage.height>tall?clamp(center.y,tall/2,stage.height-tall/2):stage.height/2;
  const up=center.y-3*unit-tall>=0,down=center.y+3*unit+tall<=stage.height;
  const candidates=[];
  if(far[0])candidates.push([pt(far[1],middle),far[2],'m']);
  for(const [fits,x,horizontal] of [far,near]){
   if(!fits)continue;
   if(up)candidates.push([pt(x,center.y-3*unit),horizontal,'e']);
   if(down||!up)candidates.push([pt(x,center.y+3*unit),horizontal,'s']);
  }
  return pick(candidates,wide,tall,unit,obstacles);
 }

 function dashes(from,to,unit,alpha,items){
  const length=distance(from,to);
  if(length!==length||length<=0.5)return;
  const on=3*unit,period=6*unit;
  for(let start=0;start<length;start+=period){
   const end=min(start+on,length);
   items.push(item(segment(lerpPt(from,to,start/length),lerpPt(from,to,end/length),unit),'u',alpha));
  }
 }

 function march(from,to,phase,unit,contrast,items){
  const length=distance(from,to);
  if(length!==length||length<=0.5)return;
  const spacing=10*unit;
  let offset=phase%spacing;
  if(offset<0)offset+=spacing;
  for(;offset<=length;offset+=spacing){
   const center=lerpPt(from,to,offset/length);
   if(contrast)items.push(item(disc(center,3.2*unit),'b',1));
   items.push(item(disc(center,1.7*unit),'s',contrast?1:0.95));
  }
 }

 class Motion{
  constructor(){
   this.state=blank(2);
   this.tracks=[new Track(),new Track(),new Track()];
   this.marker=new Marker();
   this.view=null;
   this.viewFrom=null;
   this.viewStart=NEVER;
   this.setStart=NEVER;
  }
  sketch(){return this.state;}
  update(sketch,now,animate){
   if(sameSketch(sketch,this.state)&&this.view!==null)return;
   const next=copySketch(sketch),moving=animate&&this.view!==null&&next.dimension===this.state.dimension;
   if(!moving&&next.dimension!==this.state.dimension){
    this.tracks=[new Track(),new Track(),new Track()];
    this.marker=new Marker();
   }
   const huge=sketchHuge(next);
   this.tracks.forEach((track,row)=>track.retarget(huge?null:sketchPlane(next,row),now,moving));
   this.marker.retarget(huge?null:sketchPoint(next),now,moving);
   const target=huge?bounds(blank(next.dimension)):bounds(next);
   if(this.view===null||!sameBounds(this.view,target)){
    if(moving&&this.view!==null){
     this.viewFrom=this.shownView(now);
     this.viewStart=now;
    }else{
     this.viewFrom=null;
     this.viewStart=NEVER;
    }
    this.view=target;
   }
   const setChanged=next.verdict.kind!==this.state.verdict.kind||!sameRows(next.rows,this.state.rows);
   if(setChanged)this.setStart=moving?now:NEVER;
   this.state=next;
  }
  settle(){
   this.tracks=this.tracks.map(track=>new Track(track.target));
   this.marker=new Marker(this.marker.target);
   this.viewFrom=null;
   this.viewStart=NEVER;
   this.setStart=NEVER;
  }
  shownView(now){
   const target=this.view??bounds(this.state);
   if(this.viewFrom&&now<this.viewStart+VIEW)return lerpBounds(this.viewFrom,target,easeInOut((now-this.viewStart)/VIEW));
   return target;
  }
  settling(contrast){
   const kind=this.state.verdict.kind;
   if(kind==='line')return MARCH;
   if(kind==='everything'&&!contrast)return LIGHT_UP;
   return 0;
  }
  busy(now,contrast){
   return (this.viewFrom!==null&&now<this.viewStart+VIEW)||this.tracks.some(track=>track.busy(now))||this.marker.busy(now,contrast)||now<this.setStart+this.settling(contrast);
  }
  scene(now,width,height,unit,contrast){
   if(width!==width||height!==height||width<=0||height<=0)return [];
   const stage={width,height,unit,contrast},items=[];
   const everything=this.state.verdict.kind==='everything'&&!sketchHuge(this.state);
   const fade=everything?clamp01((now-this.setStart)/LIGHT_UP):0;
   if(!contrast)items.push(item(dots(8*unit,0.9*unit),'t',1));
   if(everything)items.push(item(dots(8*unit,0.9*unit),'s',contrast?1:0.55*fade));
   const view=this.shownView(now),shown=[];
   for(let row=0;row<this.state.dimension;row++)shown.push(this.tracks[row].shown(now));
   const phase=MARCH_SPEED*unit*min(now-this.setStart,MARCH);
   if(this.state.dimension===3)this.sceneThree(now,stage,view,shown,phase,items);
   else this.sceneTwo(now,stage,view,shown,phase,items);
   return items;
  }
  spotItems(now,stage,center,alpha,size,items){
   const unit=stage.unit;
   if(!stage.contrast){
    items.push(item(disc(center,11*unit*max(size,0)),'h',0.2*alpha));
    for(const [growth,fade] of this.marker.rings(now))items.push(item(ring(center,(4+13*growth)*unit,1.5*unit),'h',fade*alpha));
   }
   const radius=4.25*unit*max(size,0);
   if(radius>0){
    items.push(item(disc(center,radius+1.75*unit),'b',stage.contrast?1:alpha));
    items.push(item(disc(center,radius),'o',stage.contrast?1:alpha));
   }
  }
  sceneTwo(now,stage,view,shown,phase,items){
   const {width,height,unit,contrast}=stage;
   const aspect=width/height,halfHeight=max(view.half[1],view.half[0]/aspect),halfWidth=halfHeight*aspect;
   const left=view.center[0]-halfWidth,right=view.center[0]+halfWidth,bottom=view.center[1]-halfHeight,top=view.center[1]+halfHeight;
   const screenX=x=>(x-left)/(right-left)*width,screenY=y=>(top-y)/(top-bottom)*height,screen=point=>pt(screenX(point[0]),screenY(point[1]));
   const opaque=alpha=>contrast?1:alpha,step=nice(2*halfHeight/5);
   const even=Math.floor(unit)%2===0;
   const snap=(value,limit)=>{const snapped=even?Math.floor(value+0.5):Math.floor(value)+0.5;return snapped>=0&&snapped<=limit?snapped:null;};
   for(let index=integer(left/step);index*step<=right;index++){
    const x=snap(screenX(index*step),width);
    if(index!==0&&x!==null)items.push(item(segment(pt(x,0),pt(x,height),unit),'g',1));
   }
   for(let index=integer(bottom/step);index*step<=top;index++){
    const y=snap(screenY(index*step),height);
    if(index!==0&&y!==null)items.push(item(segment(pt(0,y),pt(width,y),unit),'g',1));
   }
   const axisY=bottom<0&&top>0?snap(screenY(0),height):null,axisX=left<0&&right>0?snap(screenX(0),width):null;
   const tall=TEXT_HEIGHT*unit,letter=CHAR_WIDTH*unit;
   const lines=this.tracks.slice(0,this.state.dimension).filter(track=>track.target).map(track=>{
    const target=track.target;
    return [target.normal[0]*(right-left)/width,-target.normal[1]*(top-bottom)/height,target.offset-target.normal[0]*left-target.normal[1]*top];
   });
   const clear=area=>!lines.some(line=>crossed(grow(area,unit),line));
   const areas=[];
   if(axisY!==null){
    items.push(item(segment(pt(0,axisY),pt(width,axisY),unit),'a',1));
    items.push(item(polygon([pt(width-unit,axisY),pt(width-7*unit,axisY-3.5*unit),pt(width-7*unit,axisY+3.5*unit)]),'a',1));
    const over=pt(width-3*unit,axisY-5*unit),under=pt(width-3*unit,axisY+5*unit);
    const flip=axisY+5*unit+tall<=height&&!clear(textArea(over,'e','e',letter,tall))&&clear(textArea(under,'e','s',letter,tall));
    const at=flip?under:over,vertical=flip?'s':'e';
    areas.push(textArea(at,'e',vertical,letter,tall));
    areas.push(spanArea(pt(width-7*unit,axisY-3.5*unit),pt(width-unit,axisY+3.5*unit),0));
    areas.push(spanArea(pt(0,axisY),pt(width,axisY),0.5*unit));
    items.push(item(label(at,'x','e',vertical),'l',1));
   }
   if(axisX!==null){
    items.push(item(segment(pt(axisX,0),pt(axisX,height),unit),'a',1));
    items.push(item(polygon([pt(axisX,unit),pt(axisX-3.5*unit,7*unit),pt(axisX+3.5*unit,7*unit)]),'a',1));
    const east=pt(axisX+6*unit,2*unit),west=pt(axisX-6*unit,2*unit);
    const flip=axisX-6*unit-letter>=0&&!clear(textArea(east,'s','s',letter,tall))&&clear(textArea(west,'e','s',letter,tall));
    const at=flip?west:east,horizontal=flip?'e':'s';
    areas.push(textArea(at,horizontal,'s',letter,tall));
    areas.push(spanArea(pt(axisX-3.5*unit,unit),pt(axisX+3.5*unit,7*unit),0));
    areas.push(spanArea(pt(axisX,0),pt(axisX,height),0.5*unit));
    items.push(item(label(at,'y',horizontal,'s'),'l',1));
   }
   const low=[left,bottom,0],high=[right,top,0];
   const segments=shown.map(entry=>{
    if(!entry)return null;
    const shownPlane=entry.plane,direction=[-shownPlane.normal[1],shownPlane.normal[0],0],line=clip(foot(shownPlane),direction,low,high,2);
    if(!line)return null;
    const start=screen(line[0]),end=screen(line[1]),middle=lerpPt(start,end,0.5);
    return [lerpPt(middle,start,entry.progress),lerpPt(middle,end,entry.progress),entry.alpha];
   });
   if(!contrast)segments.forEach((entry,row)=>{if(entry)items.push(item(segment(entry[0],entry[1],7*unit),rowInk(row),0.14*entry[2]));});
   segments.forEach((entry,row)=>{if(entry)items.push(item(segment(entry[0],entry[1],2*unit),rowInk(row),opaque(entry[2])));});
   if(this.state.verdict.kind==='line'){
    const line=solutionLine(2,shown.map(entry=>entry?entry.plane:null));
    if(line){
     const clipped=clip(line[0],line[1],low,high,2);
     if(clipped)march(screen(clipped[0]),screen(clipped[1]),phase,unit,contrast,items);
    }
   }
   const spot=this.marker.shown(now);
   if(!spot)return;
   const center=screen(spot.at),coordinates=sketchCoordinates(this.state),guide=spot.reveal*spot.alpha;
   if(guide>0){
    const coordinate=index=>index<coordinates.length&&coordinates[index]!=='0'?coordinates[index]:null;
    const textX=axisY!==null?coordinate(0):null,textY=axisX!==null?coordinate(1):null;
    const obstacles={lines,areas};
    obstacles.areas.push(spanArea(center,center,6*unit));
    if(textX!==null)obstacles.areas.push(spanArea(center,pt(center.x,axisY),0.5*unit));
    if(textY!==null)obstacles.areas.push(spanArea(center,pt(axisX,center.y),0.5*unit));
    if(textX!==null){
     dashes(center,pt(center.x,axisY),unit,opaque(0.85*guide),items);
     const placed=xLabel(center,axisY,textX,stage,obstacles);
     if(placed){
      obstacles.areas.push(textArea(placed[0],placed[1],placed[2],Array.from(textX).length*CHAR_WIDTH*unit,tall));
      items.push(item(label(placed[0],textX,placed[1],placed[2]),'l',opaque(guide)));
     }
    }
    if(textY!==null){
     dashes(center,pt(axisX,center.y),unit,opaque(0.85*guide),items);
     const placed=yLabel(center,axisX,textY,stage,obstacles);
     if(placed)items.push(item(label(placed[0],textY,placed[1],placed[2]),'l',opaque(guide)));
    }
   }
   this.spotItems(now,stage,center,spot.alpha,spot.size,items);
  }
  sceneThree(now,stage,view,shown,phase,items){
   const {width,height,unit,contrast}=stage;
   const opaque=alpha=>contrast?1:alpha,low=lowCorner(view),high=highCorner(view);
   const project=point=>[SX_X*point[0]+SX_Z*point[2],SY_X*point[0]+SY_Y*point[1]+SY_Z*point[2]];
   const corner=index=>[index&1?high[0]:low[0],index&2?high[1]:low[1],index&4?high[2]:low[2]];
   let minU=Infinity,maxU=-Infinity,minV=Infinity,maxV=-Infinity;
   for(let index=0;index<8;index++){
    const [u,v]=project(corner(index));
    minU=min(minU,u);
    maxU=max(maxU,u);
    minV=min(minV,v);
    maxV=max(maxV,v);
   }
   const padSide=12*unit,padTop=15*unit,padBottom=4*unit;
   const roomW=max(width-2*padSide,1),roomH=max(height-padTop-padBottom,1);
   const zoom=min(roomW/(maxU-minU),roomH/(maxV-minV));
   const originX=padSide+(roomW-zoom*(maxU-minU))/2-zoom*minU,originY=padTop+(roomH-zoom*(maxV-minV))/2-zoom*minV;
   const screen=point=>{const [u,v]=project(point);return pt(originX+zoom*u,originY+zoom*v);};
   const depth=point=>DEPTH_X*point[0]+DEPTH_Y*point[1]+DEPTH_Z*point[2];
   const panes=[[corner(0),corner(1),corner(5),corner(4)],[corner(0),corner(2),corner(6),corner(4)],[corner(0),corner(1),corner(3),corner(2)]];
   if(!contrast)for(const pane of panes)items.push(item(polygon(pane.map(screen)),'p',1));
   const step=nice(view.half[0]/2);
   const grid=(axis,span)=>{
    for(let index=integer(low[axis]/step);index*step<=high[axis];index++){
     const start=low.slice();
     start[axis]=index*step;
     const end=start.slice();
     end[span]=high[span];
     items.push(item(segment(screen(start),screen(end),unit),index===0?'a':'g',1));
    }
   };
   for(const [axis,span] of [[0,2],[2,0],[1,2],[2,1],[1,0],[0,1]])grid(axis,span);
   for(const pane of panes)items.push(item(outline(pane.map(screen),unit),'e',1));
   const xEnd=screen(corner(1)),zEnd=screen(corner(4)),yEnd=screen(corner(2));
   for(const [at,text,horizontal,vertical] of [[pt(xEnd.x+4*unit,xEnd.y),'x','s','m'],[pt(zEnd.x-4*unit,zEnd.y),'z','e','m'],[pt(yEnd.x,yEnd.y-2*unit),'y','m','e']])items.push(item(label(at,text,horizontal,vertical),'l',1));
   const sections=[];
   shown.forEach((entry,row)=>{
    if(!entry)return;
    const cut=section(entry.plane,low,high);
    if(!cut.length)return;
    const count=cut.length,middle=scale(cut.reduce((sum,point)=>add(sum,point),[0,0,0]),1/count),flat=cut.map(screen);
    const total=flat.reduce((sum,point)=>pt(sum.x+point.x,sum.y+point.y),pt(0,0)),center=pt(total.x/count,total.y/count);
    sections.push([depth(middle),row,flat.map(point=>lerpPt(center,point,entry.progress)),entry.alpha]);
   });
   sections.sort((first,second)=>totalCmp(first[0],second[0])||first[1]-second[1]);
   for(const [,row,points,alpha] of sections){
    if(!contrast)items.push(item(polygon(points.slice()),rowInk(row),0.26*alpha));
    items.push(item(outline(points.slice(),1.25*unit),rowInk(row),opaque(0.95*alpha)));
   }
   if(this.state.verdict.kind==='plane'){
    let chosen=null;
    for(const entry of sections)if(sketchPlane(this.state,entry[1])&&(chosen===null||entry[1]<chosen[1]))chosen=entry;
    if(chosen)items.push(item(outline(chosen[2].slice(),2*unit),'s',opaque(chosen[3])));
   }
   for(let first=0;first<shown.length;first++)for(let second=first+1;second<shown.length;second++){
    const one=shown[first],two=shown[second];
    if(!one||!two)continue;
    const line=lineOf(one.plane,two.plane);
    if(!line)continue;
    const clipped=clip(line[0],line[1],low,high,3);
    if(!clipped)continue;
    const strength=min(one.alpha,two.alpha)*min(one.progress,two.progress);
    items.push(item(segment(screen(clipped[0]),screen(clipped[1]),unit),'m',opaque(0.5*strength)));
   }
   if(this.state.verdict.kind==='line'){
    const line=solutionLine(3,shown.map(entry=>entry?entry.plane:null));
    if(line){
     const clipped=clip(line[0],line[1],low,high,3);
     if(clipped)march(screen(clipped[0]),screen(clipped[1]),phase,unit,contrast,items);
    }
   }
   const spot=this.marker.shown(now);
   if(!spot)return;
   const center=screen(spot.at),guide=spot.reveal*spot.alpha;
   if(guide>0){
    const floor=screen([spot.at[0],low[1],spot.at[2]]);
    dashes(center,floor,unit,opaque(0.85*guide),items);
    items.push(item(ring(floor,2.5*unit,unit),'u',opaque(0.85*guide)));
   }
   this.spotItems(now,stage,center,spot.alpha,spot.size,items);
  }
 }

 const api={LIMIT,MARGIN,MIN_HALF,BOX_HEIGHT,APPEAR,MORPH,VANISH,VIEW,POP,PULSE,PULSE_DELAY,PULSE_GAP,PULSES,MARCH,MARCH_SPEED,LIGHT_UP,LABEL_LIMIT,TEXT_HEIGHT,CHAR_WIDTH,
  INTRO_TWO,INTRO_THREE,WAITING,HUGE,COINCIDENT_LINES,SHARED_LINE,COINCIDENT_PLANES,EVERYTHING_TWO,EVERYTHING_THREE,PARALLEL_LINES,PARALLEL_PLANES,NO_COMMON_POINT,GRAPH_NAME,
  PALETTE,CONTRAST_PALETTE,inkColor,plane,foot,blank,sameSketch,sketchPlane,sketchHuge,sketchPoint,sketchCoordinates,caption,describe,nice,bounds,clip,section,lineOf,solutionLine,lerpPlane,easeOut,easeInOut,easeOutBack,totalCmp,Motion};
 if(typeof module!=='undefined'&&module.exports)module.exports=api;
 else root.LinearGraph=api;
})(typeof globalThis!=='undefined'?globalThis:this);
