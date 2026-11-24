(function(root) {
  'use strict';
  const abs = x => x < 0n ? -x : x;
  function gcd(a,b) { a=abs(a); b=abs(b); while(b) [a,b]=[b,a%b]; return a; }
  const MAX_INPUT_LENGTH=100,MAX_EXPONENT=1000,MAX_POWER_DIGITS=300,LIVE_BUDGET_BITS=8000,PREVIEW_DIGIT_LIMIT=12;
  const POWER_LIMIT_BITS=Math.ceil(MAX_POWER_DIGITS*Math.log2(10));
  const PARSE_MESSAGES=Object.freeze({
    Empty:'Preencha todos os campos.',
    TooLong:'Use números com no máximo 100 caracteres.',
    Invalid:'Número inválido.',
    Exponent:'Use um expoente inteiro entre -1000 e 1000.',
    ZeroDenominator:'O denominador não pode ser zero.',
    PowerExponent:'Use um expoente inteiro de -1000 a 1000, como 2^3 ou 10^-2.',
    PowerChain:'Use parênteses para potência de potência, como (2^3)^2.',
    PowerTooLarge:'Use números menores: com potências, o limite é de 300 algarismos.',
    ZeroToZero:'0 elevado a 0 é indeterminado.',
    ZeroNegativePower:'0 não pode ser elevado a expoente negativo.',
    Parenthesis:'Confira os parênteses.',
    Product:'Faça a multiplicação antes: para 1,5×10³, digite 1,5e3 ou 1500.'
  });
  class ParseError extends Error {
    constructor(kind) { super(PARSE_MESSAGES[kind]); this.name='ParseError'; this.kind=kind; }
  }
  const PARSE_ERRORS=Object.freeze(Object.fromEntries(Object.keys(PARSE_MESSAGES).map(kind=>[kind,Object.freeze(new ParseError(kind))])));
  const fail=kind=>{ throw PARSE_ERRORS[kind]; };
  const WHITESPACE=new Set([0x9,0xA,0xB,0xC,0xD,0x20,0x85,0xA0,0x1680,0x2000,0x2001,0x2002,0x2003,0x2004,0x2005,0x2006,0x2007,0x2008,0x2009,0x200A,0x2028,0x2029,0x202F,0x205F,0x3000]);
  const isWhitespace=character=>character!==undefined&&WHITESPACE.has(character.codePointAt(0));
  function trim(text) {
    let start=0,end=text.length;
    while(start<end&&WHITESPACE.has(text.charCodeAt(start))) start++;
    while(end>start&&WHITESPACE.has(text.charCodeAt(end-1))) end--;
    return text.slice(start,end);
  }
  const normalize=input=>trim(String(input??'')).replaceAll(',','.').replaceAll('−','-');
  const isAsciiDigit=character=>character!==undefined&&character.length===1&&character>='0'&&character<='9';
  const ALPHANUMERIC=/^[\p{Alphabetic}\p{N}]$/u;
  const isAlphanumeric=character=>character!==undefined&&ALPHANUMERIC.test(character);
  const SUPERSCRIPT_DIGITS=['⁰','¹','²','³','⁴','⁵','⁶','⁷','⁸','⁹'];
  const SUPERSCRIPT_PLUS='⁺',SUPERSCRIPT_MINUS='⁻',PRODUCT_SIGNS=['×','·','⋅'];
  const superscriptDigit=character=>SUPERSCRIPT_DIGITS.indexOf(character);
  const isSuperscript=character=>character!==undefined&&(superscriptDigit(character)>=0||character===SUPERSCRIPT_PLUS||character===SUPERSCRIPT_MINUS);
  function bitLength(value) {
    if(value<0n) value=-value;
    if(value===0n) return 0;
    const hex=value.toString(16);
    return (hex.length-1)*4+32-Math.clz32(parseInt(hex[0],16));
  }
  class Rational {
    constructor(n=0n,d=1n) {
      if(d===0n) throw new Error('O denominador não pode ser zero.');
      if(d<0n) { n=-n; d=-d; }
      const g=gcd(n,d); this.n=n/g; this.d=d/g;
    }
    add(b) { return new Rational(this.n*b.d+b.n*this.d,this.d*b.d); }
    sub(b) { return new Rational(this.n*b.d-b.n*this.d,this.d*b.d); }
    mul(b) { return new Rational(this.n*b.n,this.d*b.d); }
    div(b) { return new Rational(this.n*b.d,this.d*b.n); }
    neg() { return new Rational(-this.n,this.d); }
    eq(b) { return this.n===b.n && this.d===b.d; }
    get zero() { return this.n===0n; }
    toString() { return this.d===1n?String(this.n):`${this.n}/${this.d}`; }
    static parse(input) {
      const text=normalize(input);
      if(!text) fail('Empty');
      if(text.length>MAX_INPUT_LENGTH) fail('TooLong');
      if(usesExtension(text)) return new PowerParser(text).value();
      const parts=text.split('/');
      if(parts.length>2) fail('Invalid');
      const value=parseDecimal(parts[0]);
      if(parts.length===1) return value;
      const divisor=parseDecimal(parts[1]);
      if(divisor.zero) fail('ZeroDenominator');
      return value.div(divisor);
    }
  }
  function parseDecimal(input) {
    let text=trim(input);
    if(!text) fail('Invalid');
    let exponent=0;
    const position=text.search(/[eE]/);
    if(position>=0) {
      const parsed=parseExponent(text.slice(position+1));
      if(parsed===null) fail('Exponent');
      exponent=parsed;
      text=text.slice(0,position);
      if(!text) fail('Invalid');
    }
    const negative=text[0]==='-',body=text[0]==='-'||text[0]==='+'?text.slice(1):text;
    let digits='',dot=false,decimals=0;
    for(const character of body) {
      if(character==='.'&&!dot) { dot=true; continue; }
      if(!isAsciiDigit(character)) fail('Invalid');
      digits+=character;
      if(dot) decimals++;
    }
    if(!digits) fail('Invalid');
    const numerator=negative?-BigInt(digits):BigInt(digits),power=decimals-exponent;
    return power>=0?new Rational(numerator,10n**BigInt(power)):new Rational(numerator*10n**BigInt(-power));
  }
  function parseExponent(text) {
    const negative=text[0]==='-';
    const digits=(negative||text[0]==='+'?text.slice(1):text).replace(/\0+$/,'');
    if(!digits||!/^[0-9]+$/.test(digits)) return null;
    const significant=digits.replace(/^0+/,'');
    if(significant.length>4) return null;
    const value=significant?Number(significant):0;
    if(value>MAX_EXPONENT) return null;
    return negative?-value:value;
  }
  const usesExtension=text=>Array.from(text).some(character=>character==='^'||character==='('||character===')'||character==='*'||PRODUCT_SIGNS.includes(character)||isSuperscript(character));
  function bounded(digits,negative) {
    const significant=digits.replace(/^0+/,'');
    if(significant.length>4) fail('PowerExponent');
    const value=significant?Number(significant):0;
    if(value>MAX_EXPONENT) fail('PowerExponent');
    return negative?-value:value;
  }
  let powerLimit=null;
  const tooLarge=value=>abs(value)>=(powerLimit??=10n**BigInt(MAX_POWER_DIGITS));
  function capped(value,powered) {
    if(powered&&(tooLarge(value.n)||tooLarge(value.d))) fail('PowerTooLarge');
    return [value,powered];
  }
  function raise(base,exponent) {
    if(Math.abs(exponent)>MAX_EXPONENT) fail('PowerExponent');
    if(base.zero) {
      if(exponent===0) fail('ZeroToZero');
      if(exponent<0) fail('ZeroNegativePower');
      return new Rational(0n);
    }
    if(exponent===0) return new Rational(1n);
    const power=Math.abs(exponent),widest=Math.max(bitLength(base.n),bitLength(base.d));
    if(Math.max(widest-1,0)*power>POWER_LIMIT_BITS) fail('PowerTooLarge');
    const numerator=base.n**BigInt(power),denominator=base.d**BigInt(power);
    if(tooLarge(numerator)||tooLarge(denominator)) fail('PowerTooLarge');
    return exponent>0?new Rational(numerator,denominator):new Rational(denominator,numerator);
  }
  class PowerParser {
    constructor(text) { this.characters=Array.from(text); this.position=0; this.depth=0; }
    peek() { return this.characters[this.position]; }
    peekAt(offset) { return this.characters[this.position+offset]; }
    skipSpaces() { while(isWhitespace(this.peek())) this.position++; }
    atPower() {
      const character=this.peek();
      if(character==='^') return true;
      if(character==='*') return this.peekAt(1)==='*';
      return isSuperscript(character);
    }
    atProduct() {
      const character=this.peek();
      if(character==='*') return this.peekAt(1)!=='*';
      return character!==undefined&&PRODUCT_SIGNS.includes(character);
    }
    value() {
      const [value]=this.quotient();
      this.skipSpaces();
      const character=this.peek();
      if(character===undefined) return value;
      return fail(character===')'?'Parenthesis':'Invalid');
    }
    quotient() {
      const [numerator,poweredNumerator]=this.signed();
      this.skipSpaces();
      if(this.peek()!=='/') return [numerator,poweredNumerator];
      this.position++;
      const [denominator,poweredDenominator]=this.signed();
      this.skipSpaces();
      if(this.peek()==='/') fail('Invalid');
      if(denominator.zero) fail('ZeroDenominator');
      return capped(numerator.div(denominator),poweredNumerator||poweredDenominator);
    }
    signed() {
      this.skipSpaces();
      const sign=this.peek();
      if(sign!=='-'&&sign!=='+') return this.power();
      this.position++;
      const next=this.peek();
      if(!(isAsciiDigit(next)||next==='.'||next==='(')) fail(this.missingOperand());
      const [value,powered]=this.power();
      return [sign==='-'?value.neg():value,powered];
    }
    missingOperand() { return (this.peek()===undefined&&this.depth>0)||this.peek()===')'?'Parenthesis':'Invalid'; }
    power() {
      const [base,powered]=this.atom();
      this.skipSpaces();
      if(!this.atPower()) {
        if(this.atProduct()) fail('Product');
        return [base,powered];
      }
      const exponent=this.exponent();
      this.skipSpaces();
      if(this.atPower()) fail('PowerChain');
      const value=raise(base,exponent);
      if(this.atProduct()) fail('Product');
      return [value,true];
    }
    atom() {
      this.skipSpaces();
      const character=this.peek();
      if(character==='(') {
        this.position++;
        this.depth++;
        const parsed=this.quotient();
        this.skipSpaces();
        if(this.peek()!==')') fail(this.peek()===undefined?'Parenthesis':'Invalid');
        this.position++;
        this.depth--;
        return parsed;
      }
      if(isAsciiDigit(character)||character==='.') return [this.number(),false];
      return fail(this.missingOperand());
    }
    number() {
      const start=this.position;
      while(isAsciiDigit(this.peek())||this.peek()==='.') this.position++;
      if(this.peek()==='e'||this.peek()==='E') {
        this.position++;
        if(this.peek()==='+'||this.peek()==='-') this.position++;
        while(isAsciiDigit(this.peek())) this.position++;
      }
      return parseDecimal(this.characters.slice(start,this.position).join(''));
    }
    exponent() {
      const character=this.peek();
      if(character==='^') this.position+=1;
      else if(character==='*') this.position+=2;
      else return this.superscript();
      this.skipSpaces();
      if(this.peek()!=='(') return this.integer();
      this.position++;
      this.skipSpaces();
      const value=this.integer();
      this.skipSpaces();
      const close=this.peek();
      if(close===undefined) fail('Parenthesis');
      if(close!==')') fail('PowerExponent');
      this.position++;
      return value;
    }
    integer() {
      const sign=this.peek();
      if(sign==='-'||sign==='+') this.position++;
      const start=this.position;
      while(isAsciiDigit(this.peek())) this.position++;
      const next=this.peek();
      if(this.position===start||next==='.'||next==='e'||next==='E'||(isAlphanumeric(next)&&!isSuperscript(next))) fail('PowerExponent');
      return bounded(this.characters.slice(start,this.position).join(''),sign==='-');
    }
    superscript() {
      const sign=this.peek();
      if(sign===SUPERSCRIPT_MINUS||sign===SUPERSCRIPT_PLUS) this.position++;
      let digits='';
      for(let digit=superscriptDigit(this.peek());digit>=0;digit=superscriptDigit(this.peek())) { digits+=String(digit); this.position++; }
      if(!digits||isSuperscript(this.peek())) fail('PowerExponent');
      return bounded(digits,sign===SUPERSCRIPT_MINUS);
    }
  }
  function looksIncomplete(input) {
    const characters=Array.from(normalize(input));
    if(!characters.length) return false;
    const last=characters[characters.length-1];
    const open=characters.filter(character=>character==='(').length,close=characters.filter(character=>character===')').length;
    return open>close||['+','-','/','^','*','(','e','E','.',SUPERSCRIPT_PLUS,SUPERSCRIPT_MINUS].includes(last);
  }
  const withinLiveBudget=matrix=>matrix.flat().reduce((bits,value)=>bits+bitLength(value.n)+bitLength(value.d),0)<=LIVE_BUDGET_BITS;
  function solve(input) {
    const n=input.length;
    if((n!==2&&n!==3)||input.some(row=>row.length!==n+1)) throw new Error('O sistema deve ser 2×2 ou 3×3.');
    const a=input.map(row=>row.slice()),pivots=[];
    let r=0;
    for(let col=0;col<n&&r<n;col++) {
      let selected=r; while(selected<n&&a[selected][col].zero) selected++;
      if(selected===n) continue;
      [a[r],a[selected]]=[a[selected],a[r]];
      const pivot=a[r][col]; a[r]=a[r].map(v=>v.div(pivot));
      for(let i=0;i<n;i++) if(i!==r&&!a[i][col].zero) { const factor=a[i][col]; a[i]=a[i].map((v,j)=>v.sub(factor.mul(a[r][j]))); }
      pivots.push(col); r++;
    }
    const inconsistent=a.some(row=>row.slice(0,n).every(v=>v.zero)&&!row[n].zero);
    return {kind:inconsistent?'none':(pivots.length===n?'unique':'infinite'),rref:a,pivots};
  }
  function format(solution) {
    if(solution.kind==='none') return 'Este sistema não tem solução.\nAs equações são incompatíveis.';
    const n=solution.rref.length,names=['x','y','z'],rowFor=Array(n).fill(-1),parameters=Array(n).fill(0);
    solution.pivots.forEach((col,row)=>rowFor[col]=row);
    let next=1; parameters.forEach((_,i)=>{if(rowFor[i]<0) parameters[i]=next++;});
    const rows=names.slice(0,n).map((name,col)=>{
      if(rowFor[col]<0) return `${name} = t${parameters[col]}`;
      const row=solution.rref[rowFor[col]],constant=row[n];
      let text=constant.zero?'':String(constant);
      for(let j=0;j<n;j++) if(parameters[j]) {
        const coeff=row[j].neg(); if(coeff.zero) continue;
        const negative=coeff.n<0n, magnitude=negative?coeff.neg():coeff;
        text+=text?(negative?' - ':' + '):(negative?'-':'');
        text+=(magnitude.eq(new Rational(1n))?'':`${magnitude} · `)+`t${parameters[j]}`;
      }
      return `${name} = ${text||'0'}`;
    });
    return [solution.kind==='unique'?'Solução única':'Infinitas soluções','',...rows,...(solution.kind==='infinite'?['','t1, t2, ... podem ser quaisquer números reais.']:[])].join('\n');
  }
  const VARIABLES=['x','y','z'],ROWS=['L₁','L₂','L₃'],MINUS='−',ZERO=new Rational(0n),ONE=new Rational(1n);
  const METHODS=Object.freeze([
    Object.freeze({key:'gauss-jordan',name:'Gauss-Jordan',label:'Gauss-Jordan'}),
    Object.freeze({key:'cramer',name:'Regra de Cramer',label:'Cramer'}),
    Object.freeze({key:'gauss',name:'Escalonamento de Gauss',label:'Escalonamento'})
  ]);
  const CLASSIFICATIONS={
    unique:'Sistema possível e determinado (SPD): solução única.',
    infinite:'Sistema possível e indeterminado (SPI): infinitas soluções.',
    none:'Sistema impossível (SI): não tem solução.'
  };
  const isOne=v=>v.n===1n&&v.d===1n,isInteger=v=>v.d===1n,isNegative=v=>v.n<0n,magnitude=v=>isNegative(v)?v.neg():v;
  const number=v=>String(v).replaceAll('-',MINUS);
  const factor=v=>isNegative(v)||!isInteger(v)?`(${number(v)})`:number(v);
  const dividend=v=>isInteger(v)?number(v):`(${number(v)})`;
  const coefficientPrefix=size=>isOne(size)?'':isInteger(size)?String(size):`(${size})`;
  const sum=values=>values.reduce((total,value)=>total.add(value),ZERO);
  const list=items=>items.length===0?'':items.length===1?items[0]:`${items.slice(0,-1).join(', ')} e ${items[items.length-1]}`;
  const names=count=>list(VARIABLES.slice(0,count));
  const rowsList=(start,end)=>list(ROWS.slice(start,end));
  const line=(kind,text)=>({kind,text});
  const step=(title,reasoning)=>({title,reasoning,lines:[],matrix:null,matrixFirst:false});
  const view=(matrix,options)=>({label:null,rows:matrix.map(row=>row.slice()),augmented:true,markedRows:[],markedColumn:null,pivot:null,...options});
  const swapRows=(matrix,first,second)=>{[matrix[first],matrix[second]]=[matrix[second],matrix[first]];};
  const findPivot=(matrix,from,column)=>{let row=from;while(row<matrix.length&&matrix[row][column].zero)row++;return row<matrix.length?row:-1;};
  const freeColumns=(count,pivots)=>VARIABLES.slice(0,count).map((_,column)=>column).filter(column=>!pivots.includes(column));
  function leftSide(coefficients) {
    let text='';
    coefficients.forEach((coefficient,column)=>{
      if(coefficient.zero) return;
      const negative=isNegative(coefficient);
      if(!text) { if(negative) text+=MINUS; }
      else text+=negative?' − ':' + ';
      text+=coefficientPrefix(magnitude(coefficient))+VARIABLES[column];
    });
    return text||'0';
  }
  const equation=row=>`${leftSide(row.slice(0,-1))} = ${number(row[row.length-1])}`;
  const Entry=Object.freeze({Missing:Object.freeze({kind:'missing'}),Invalid:Object.freeze({kind:'invalid'}),value:value=>({kind:'value',value})});
  const abbreviateDigits=digits=>digits.length<=PREVIEW_DIGIT_LIMIT?digits:`${digits.slice(0,4)}…${digits.slice(-4)}`;
  function abbreviate(text) {
    let output='',digits='';
    for(const character of text) {
      if(isAsciiDigit(character)) { digits+=character; continue; }
      output+=abbreviateDigits(digits)+character;
      digits='';
    }
    return output+abbreviateDigits(digits);
  }
  function previewTerm(coefficient,column,text) {
    if(coefficient.zero) return text;
    const negative=isNegative(coefficient),size=magnitude(coefficient);
    if(!text) { if(negative) text+=MINUS; }
    else text+=negative?' − ':' + ';
    if(!isOne(size)) { const shown=abbreviate(String(size)); text+=isInteger(size)?shown:`(${shown})`; }
    return text+VARIABLES[column];
  }
  function previewEquation(row) {
    if(!row.length) return '';
    const constant=row[row.length-1];
    let text='';
    row.slice(0,-1).forEach((entry,column)=>{
      if(entry.kind==='value') text=previewTerm(entry.value,column,text);
      else text+=`${text?' + ':''}?${VARIABLES[column]}`;
    });
    return `${text||'0'} = ${constant.kind==='value'?abbreviate(number(constant.value)):'?'}`;
  }
  const vector=row=>`(${row.slice(0,-1).map(number).join(', ')} | ${number(row[row.length-1])})`;
  const signedSum=values=>values.map((value,index)=>index===0?number(value):isNegative(value)?` − ${number(value.neg())}`:` + ${number(value)}`).join('');
  function operationText(target,source,value) {
    const size=magnitude(value),multiple=isOne(size)?ROWS[source]:`${factor(size)}·${ROWS[source]}`;
    return `${ROWS[target]} ← ${ROWS[target]} ${isNegative(value)?'+':MINUS} ${multiple}`;
  }
  function eliminationText(before,source,value,after) {
    const size=magnitude(value),multiple=isOne(size)?'':`${factor(size)}·`;
    return `${vector(before)} ${isNegative(value)?'+':MINUS} ${multiple}${vector(source)} = ${vector(after)}`;
  }
  function subtractMultiple(matrix,target,source,value) {
    matrix[target]=matrix[target].map((entry,column)=>entry.sub(value.mul(matrix[source][column])));
  }
  function firstInconsistent(matrix) {
    const count=matrix.length;
    return matrix.findIndex(row=>row.slice(0,count).every(value=>value.zero)&&!row[count].zero);
  }
  function noPivotStep(matrix,pivotRow,column) {
    const count=matrix.length,name=VARIABLES[column],next=column+1<count?'Passamos para a próxima coluna. ':'';
    const result=step(`Coluna de ${name}: sem pivô`,`Em ${rowsList(pivotRow,count)}, a coluna de ${name} só tem zeros: não há como escolher um pivô nela. ${next}Se o sistema tiver solução, ${name} será uma variável livre, que pode assumir qualquer valor.`);
    result.matrix=view(matrix,{markedColumn:column});
    return result;
  }
  function swapStep(matrix,pivotRow,selected,column) {
    const top=ROWS[pivotRow],bottom=ROWS[selected],name=VARIABLES[column];
    const result=step(`Trocar ${top} e ${bottom}`,`Na coluna de ${name}, o pivô deveria ficar em ${top}, mas ali o valor é 0. Trocamos ${top} com ${bottom}, a primeira linha abaixo com valor diferente de zero nessa coluna. Mudar a ordem das equações não muda a solução.`);
    result.lines.push(line('operation',`${top} ↔ ${bottom}`));
    result.matrix=view(matrix,{markedRows:[pivotRow,selected],pivot:[pivotRow,column]});
    return result;
  }
  function openingStep(matrix,method) {
    const goal=method==='gauss'
      ?'No escalonamento, zeramos tudo o que fica abaixo dos pivôs, coluna por coluna, e depois resolvemos o sistema de baixo para cima. Trocar linhas e somar a uma linha um múltiplo de outra não mudam as soluções.'
      :'A meta é transformar a parte da esquerda na matriz identidade (1 na diagonal e 0 no resto), ou chegar o mais perto possível dela, usando operações de linha. Trocar linhas, dividir uma linha por um número diferente de zero e somar a uma linha um múltiplo de outra não mudam as soluções.';
    const result=step('Montar a matriz aumentada',`Cada linha é uma equação: à esquerda da barra ficam os coeficientes de ${names(matrix.length)} e, à direita, os termos independentes. ${goal}`);
    result.matrix=view(matrix);
    return result;
  }
  function gaussJordanElimination(steps,input) {
    const count=input.length,matrix=input.map(row=>row.slice()),pivots=[];
    steps.push(openingStep(matrix,'gauss-jordan'));
    let pivotRow=0;
    for(let column=0;column<count&&pivotRow<count;column++) {
      const name=VARIABLES[column],rowName=ROWS[pivotRow],selected=findPivot(matrix,pivotRow,column);
      if(selected<0) { steps.push(noPivotStep(matrix,pivotRow,column)); continue; }
      const first=steps.length,swapped=selected!==pivotRow;
      if(swapped) { swapRows(matrix,pivotRow,selected); steps.push(swapStep(matrix,pivotRow,selected,column)); }
      const pivot=matrix[pivotRow][column],normalized=!isOne(pivot);
      if(normalized) {
        const before=matrix[pivotRow];
        matrix[pivotRow]=before.map(value=>value.div(pivot));
        const result=step(`Fazer o pivô de ${name} valer 1`,`O pivô da coluna de ${name} é ${number(pivot)}, em ${rowName}. Dividimos a linha inteira por ${number(pivot)} para que ele vire 1. Dividir uma equação inteira por um número diferente de zero não muda a solução.`);
        result.lines.push(line('operation',`${rowName} ← ${rowName} ÷ ${factor(pivot)}`),line('calculation',`${vector(before)} ÷ ${factor(pivot)} = ${vector(matrix[pivotRow])}`));
        result.matrix=view(matrix,{markedRows:[pivotRow],pivot:[pivotRow,column]});
        steps.push(result);
      }
      const targets=[];
      for(let row=0;row<count;row++) if(row!==pivotRow&&!matrix[row][column].zero) targets.push(row);
      if(!targets.length) {
        if(steps.length>first) steps[steps.length-1].reasoning+=` As outras linhas já têm 0 na coluna de ${name}.`;
        else {
          const result=step(`Coluna de ${name} pronta`,`O pivô de ${name}, em ${rowName}, já vale 1 e as outras linhas já têm 0 nessa coluna: não há conta a fazer aqui.`);
          result.matrix=view(matrix,{pivot:[pivotRow,column]});
          steps.push(result);
        }
      } else {
        const note=normalized?'':swapped?`Depois da troca, o pivô de ${name} já vale 1. `:`O pivô de ${name}, em ${rowName}, já vale 1. `;
        const result=step(`Zerar a coluna de ${name}`,`${note}Agora usamos ${rowName} para zerar ${name} nas outras linhas: de cada uma subtraímos (valor de ${name} nela) × ${rowName}. Como o pivô vale 1, o ${name} dessas linhas vira 0.`);
        for(const target of targets) {
          const value=matrix[target][column],before=matrix[target];
          subtractMultiple(matrix,target,pivotRow,value);
          result.lines.push(line('operation',operationText(target,pivotRow,value)),line('calculation',eliminationText(before,matrix[pivotRow],value,matrix[target])));
        }
        result.matrix=view(matrix,{markedRows:targets,pivot:[pivotRow,column]});
        steps.push(result);
      }
      pivots.push(column); pivotRow++;
    }
    return {matrix,pivots};
  }
  function contradictionStep(matrix,row) {
    const count=matrix.length,constant=number(matrix[row][count]),zeros=VARIABLES.slice(0,count).map(name=>`0${name}`);
    const result=step('Achar a contradição',`A linha ${ROWS[row]} ficou com todos os coeficientes iguais a 0 e termo independente ${constant}. Ela diz que 0 = ${constant}, o que é impossível. Nenhum valor de ${names(count)} satisfaz todas as equações ao mesmo tempo: o sistema é impossível (SI).`);
    result.lines.push(line('calculation',`${ROWS[row]}: ${zeros.join(' + ')} = ${constant}  →  0 = ${constant}`),line('result','Sem solução'));
    return result;
  }
  function answerLines(solution) {
    if(solution.kind==='none') return ['Sem solução'];
    return format(solution).split('\n').slice(2,2+solution.rref.length).map(text=>text.replaceAll('-',MINUS));
  }
  function freeSentence(free) {
    const items=free.map(column=>VARIABLES[column]);
    return free.length===1?`${items[0]} ficou sem pivô e é uma variável livre`:`${list(items)} ficaram sem pivô e são variáveis livres`;
  }
  function parameterSentence(free,others) {
    const parameters=free.map((_,index)=>`t${index+1}`);
    const head=free.length===1?`A variável livre vira o parâmetro ${parameters[0]}`:`As variáveis livres viram os parâmetros ${list(parameters)}`;
    if(others) return `${head}, e ${others}.`;
    return free.length===1?`${head}, que pode valer qualquer número.`:`${head}, que podem valer qualquer número.`;
  }
  function readingStep(reduced,solution) {
    const count=reduced.matrix.length,answer=answerLines(solution);
    if(solution.kind==='unique') {
      const result=step('Ler a solução','A parte da esquerda virou a matriz identidade, então cada linha diz diretamente o valor de uma incógnita. O sistema é possível e determinado (SPD): tem uma única solução.');
      for(const text of answer) result.lines.push(line('result',text));
      return result;
    }
    const free=freeColumns(count,reduced.pivots);
    const zeroNote=reduced.matrix.some(row=>row.every(value=>value.zero))?' As linhas só com zeros dizem 0 = 0: valem sempre e não impõem condição.':'';
    const result=step('Escrever a solução geral',`Não deu para ter um pivô em cada coluna: ${freeSentence(free)}.${zeroNote} ${parameterSentence(free,reduced.pivots.length?'as outras incógnitas são isoladas nas linhas com pivô':null)} O sistema é possível e indeterminado (SPI): tem infinitas soluções.`);
    free.forEach((column,index)=>result.lines.push(line('calculation',`${VARIABLES[column]} = t${index+1}  (livre)`)));
    reduced.pivots.forEach((column,row)=>result.lines.push(line('calculation',`${ROWS[row]}: ${equation(reduced.matrix[row])}  →  ${answer[column]}`)));
    for(const text of answer) result.lines.push(line('result',text));
    return result;
  }
  function solutionValues(solution) {
    const count=solution.rref.length,values=Array(count).fill(ZERO);
    solution.pivots.forEach((column,row)=>{values[column]=solution.rref[row][count];});
    return values;
  }
  const assignments=values=>list(values.map((value,index)=>`${VARIABLES[index]} = ${number(value)}`));
  function verificationStep(input,solution) {
    if(solution.kind==='none') return null;
    const count=input.length,values=solutionValues(solution);
    let result;
    if(solution.kind==='unique') result=step('Conferir nas equações originais',`Substituímos ${assignments(values)} em cada equação do sistema original. Se todas fecharem, a resposta está certa.`);
    else {
      const parameters=freeColumns(count,solution.pivots).map((_,index)=>`t${index+1} = 0`);
      result=step('Conferir uma solução particular',`Cada escolha dos parâmetros dá uma solução. Com ${list(parameters)}, ficamos com ${assignments(values)}. Substituindo nas equações originais:`);
    }
    input.forEach((row,index)=>{
      const coefficients=row.slice(0,count),products=coefficients.map((coefficient,column)=>coefficient.mul(values[column]));
      const terms=coefficients.map((coefficient,column)=>`${factor(coefficient)}·${factor(values[column])}`),total=sum(products);
      const verdict=total.eq(row[count])?'(confere)':`≠ ${number(row[count])} (não confere)`;
      result.lines.push(line('calculation',`${index+1}ª equação: ${terms.join(' + ')} = ${signedSum(products)} = ${number(total)} ${verdict}`));
    });
    return result;
  }
  function finishGaussJordan(steps,input,reduced,solution) {
    const row=firstInconsistent(reduced.matrix);
    let kind;
    if(row>=0) { steps.push(contradictionStep(reduced.matrix,row)); kind='none'; }
    else { kind=reduced.pivots.length===input.length?'unique':'infinite'; steps.push(readingStep(reduced,solution)); }
    const check=verificationStep(input,solution);
    if(check) steps.push(check);
    return kind;
  }
  const DOWN=[[[0,0],[1,1],[2,2]],[[0,1],[1,2],[2,0]],[[0,2],[1,0],[2,1]]];
  const UP=[[[0,2],[1,1],[2,0]],[[0,0],[1,2],[2,1]],[[0,1],[1,0],[2,2]]];
  function determinantLines(name,matrix) {
    if(matrix.length===2) {
      const main=matrix[0][0].mul(matrix[1][1]),secondary=matrix[0][1].mul(matrix[1][0]),value=main.sub(secondary);
      return {value,lines:[
        line('calculation',`${name} = ${factor(matrix[0][0])}·${factor(matrix[1][1])} − ${factor(matrix[0][1])}·${factor(matrix[1][0])}`),
        line('calculation',`${name} = ${number(main)} − ${factor(secondary)}`),
        line('result',`${name} = ${number(value)}`)
      ]};
    }
    const expand=diagonals=>({
      text:diagonals.map(diagonal=>diagonal.map(([row,column])=>factor(matrix[row][column])).join('·')).join(' + '),
      products:diagonals.map(diagonal=>diagonal.reduce((total,[row,column])=>total.mul(matrix[row][column]),ONE))
    });
    const down=expand(DOWN),up=expand(UP),downTotal=sum(down.products),upTotal=sum(up.products),value=downTotal.sub(upTotal);
    return {value,lines:[
      line('calculation',`${name} = (${down.text}) − (${up.text})`),
      line('calculation',`${name} = (${signedSum(down.products)}) − (${signedSum(up.products)})`),
      line('calculation',`${name} = ${number(downTotal)} − ${factor(upTotal)}`),
      line('result',`${name} = ${number(value)}`)
    ]};
  }
  function cramer(steps,input,solution,determinants) {
    const count=input.length,coefficients=input.map(row=>row.slice(0,count)),constants=input.map(row=>row[count]);
    const opening=step('Separar a matriz dos coeficientes',`A regra de Cramer usa determinantes. A matriz A guarda só os coeficientes de ${names(count)}, e os termos independentes formam a coluna b.`);
    opening.matrix=view(coefficients,{label:'A',augmented:false});
    opening.matrixFirst=true;
    opening.lines.push(line('calculation',`b = (${constants.map(number).join(', ')})`));
    steps.push(opening);
    const main=determinantLines('D',coefficients),determinant=main.value;
    const mainStep=step('Calcular D = det(A)',count===2
      ?'Numa matriz 2×2, o determinante é o produto da diagonal principal menos o produto da diagonal secundária.'
      :'Numa matriz 3×3, usamos a regra de Sarrus: somamos os produtos das três diagonais que descem da esquerda para a direita e subtraímos os produtos das três que sobem.');
    mainStep.lines=main.lines;
    steps.push(mainStep);
    determinants.push({name:'D',value:determinant});
    if(determinant.zero) {
      steps.push(step('D = 0: Cramer não decide','Com D = 0 não dá para dividir por D, então o sistema não tem solução única: ele pode ter infinitas soluções (SPI) ou nenhuma (SI). Para descobrir qual, escalonamos a matriz aumentada por Gauss-Jordan nos passos a seguir.'));
      return finishGaussJordan(steps,input,gaussJordanElimination(steps,input),solution);
    }
    const values=[];
    for(let column=0;column<count;column++) {
      const name=VARIABLES[column],label=`D${name}`;
      const replaced=coefficients.map((row,index)=>row.map((value,other)=>other===column?constants[index]:value));
      const {lines,value}=determinantLines(label,replaced);
      const intro=column===0?`Como D = ${number(determinant)} é diferente de 0, o sistema tem solução única (SPD) e a regra de Cramer se aplica. `:'';
      const result=step(`Calcular ${label}`,`${intro}Para ${name}, trocamos a coluna de ${name} em A pelos termos independentes b e calculamos o determinante ${label}.`);
      result.matrix=view(replaced,{augmented:false,markedColumn:column});
      result.matrixFirst=true;
      result.lines=lines;
      steps.push(result);
      determinants.push({name:label,value});
      values.push(value);
    }
    const division=step('Dividir pelos determinantes','Pela regra de Cramer, cada incógnita é o seu determinante dividido por D.');
    values.forEach((value,column)=>{
      const name=VARIABLES[column];
      division.lines.push(line('result',`${name} = D${name} ÷ D = ${dividend(value)} ÷ ${factor(determinant)} = ${number(value.div(determinant))}`));
    });
    steps.push(division);
    const check=verificationStep(input,solution);
    if(check) steps.push(check);
    return 'unique';
  }
  class Affine {
    constructor(constant,terms) { this.constant=constant; this.terms=terms; }
    static of(value,parameters) { return new Affine(value,Array(parameters).fill(ZERO)); }
    static parameter(index,parameters) { const terms=Array(parameters).fill(ZERO); terms[index]=ONE; return new Affine(ZERO,terms); }
    get isNumber() { return this.terms.every(term=>term.zero); }
    get isParameter() { return this.constant.zero&&this.terms.filter(term=>!term.zero).length===1&&this.terms.some(isOne); }
    add(other) { return new Affine(this.constant.add(other.constant),this.terms.map((term,index)=>term.add(other.terms[index]))); }
    scale(value) { return new Affine(this.constant.mul(value),this.terms.map(term=>term.mul(value))); }
    text() {
      let text='',hasTerm=!this.constant.zero;
      if(hasTerm) text+=number(this.constant);
      this.terms.forEach((coefficient,index)=>{
        if(coefficient.zero) return;
        const negative=isNegative(coefficient),size=magnitude(coefficient);
        if(hasTerm) text+=negative?' − ':' + ';
        else if(negative) text+=MINUS;
        if(!isOne(size)) text+=`${size} · `;
        text+=`t${index+1}`;
        hasTerm=true;
      });
      return hasTerm?text:'0';
    }
    operand() { return this.isNumber?factor(this.constant):this.isParameter?this.text():`(${this.text()})`; }
    dividend() { return this.isNumber?dividend(this.constant):this.isParameter?this.text():`(${this.text()})`; }
    subtrahend() {
      const singlePositive=this.constant.zero&&this.terms.filter(term=>!term.zero).length===1&&this.terms.every(term=>!isNegative(term));
      return this.isNumber?factor(this.constant):singlePositive?this.text():`(${this.text()})`;
    }
  }
  const pivotTerm=(pivot,column)=>(isNegative(pivot)?MINUS:'')+coefficientPrefix(magnitude(pivot))+VARIABLES[column];
  function backSubstitutionStep(matrix,pivotRow,column,values,parameters,first) {
    const count=matrix.length,row=matrix[pivotRow],name=VARIABLES[column],rowName=ROWS[pivotRow],pivot=row[column],constant=row[count];
    const known=[];
    for(let other=column+1;other<count;other++) if(!row[other].zero&&values[other]) known.push({other,coefficient:row[other],value:values[other]});
    const knownNames=known.map(item=>VARIABLES[item.other]);
    const replaced=known.length===1?`trocamos ${knownNames[0]} pelo valor que já temos`:`trocamos ${list(knownNames)} pelos valores que já temos`;
    const opening=first?`Começamos de baixo, por ${rowName}`:`Subimos para ${rowName}`;
    const direct=!known.length&&isOne(pivot);
    const reasoning=direct?`${opening}, que já diz diretamente o valor de ${name}.`
      :!known.length?`${opening}, que só tem a incógnita ${name}: basta dividir pelo coeficiente dela.`
      :`${opening}: ${replaced} e isolamos ${name}.`;
    const result=step(`Encontrar ${name}`,reasoning);
    if(direct) {
      const value=Affine.of(constant,parameters);
      result.lines.push(line('result',`${rowName}: ${name} = ${value.text()}`));
      return {step:result,value};
    }
    result.lines.push(line('calculation',`${rowName}: ${equation(row)}`));
    const lead=pivotTerm(pivot,column);
    let remainder=Affine.of(constant,parameters);
    if(known.length) {
      let substituted=lead,total=Affine.of(ZERO,parameters);
      const products=[];
      for(const {coefficient,value} of known) {
        substituted+=(isNegative(coefficient)?' − ':' + ')+`${factor(magnitude(coefficient))}·${value.operand()}`;
        const product=value.scale(coefficient);
        total=total.add(product);
        products.push(product);
      }
      result.lines.push(line('calculation',`${substituted} = ${number(constant)}`));
      remainder=Affine.of(constant,parameters).add(total.scale(ONE.neg()));
      let text;
      if(total.isNumber&&products.length>1) text=`${lead} = ${number(constant)} − (${signedSum(products.map(product=>product.constant))}) = ${number(constant)} − ${factor(total.constant)} = ${remainder.text()}`;
      else {
        const difference=`${number(constant)} − ${total.subtrahend()}`;
        text=difference===remainder.text()?`${lead} = ${difference}`:`${lead} = ${difference} = ${remainder.text()}`;
      }
      result.lines.push(line(isOne(pivot)?'result':'calculation',text));
    }
    const value=remainder.scale(ONE.div(pivot));
    if(!isOne(pivot)) result.lines.push(line('result',`${name} = ${remainder.dividend()} ÷ ${factor(pivot)} = ${value.text()}`));
    return {step:result,value};
  }
  function gauss(steps,input,solution) {
    const count=input.length,matrix=input.map(row=>row.slice()),pivots=[];
    steps.push(openingStep(matrix,'gauss'));
    let pivotRow=0;
    for(let column=0;column<count&&pivotRow<count;column++) {
      const name=VARIABLES[column],rowName=ROWS[pivotRow],selected=findPivot(matrix,pivotRow,column);
      if(selected<0) { steps.push(noPivotStep(matrix,pivotRow,column)); continue; }
      const first=steps.length;
      if(selected!==pivotRow) { swapRows(matrix,pivotRow,selected); steps.push(swapStep(matrix,pivotRow,selected,column)); }
      const pivot=matrix[pivotRow][column],targets=[];
      for(let row=pivotRow+1;row<count;row++) if(!matrix[row][column].zero) targets.push(row);
      if(!targets.length) {
        if(steps.length>first) steps[steps.length-1].reasoning+=' Abaixo do pivô já está tudo zerado.';
        else if(pivotRow+1<count) {
          const result=step(`Coluna de ${name} pronta`,`O pivô da coluna de ${name} é ${number(pivot)}, em ${rowName}, e abaixo dele já está tudo zerado: não há conta a fazer aqui.`);
          result.matrix=view(matrix,{pivot:[pivotRow,column]});
          steps.push(result);
        }
      } else {
        const result=step(`Zerar ${name} abaixo do pivô`,`O pivô da coluna de ${name} é ${number(pivot)}, em ${rowName}. Para cada linha abaixo dele, calculamos o multiplicador m = (valor de ${name} na linha) ÷ pivô e fazemos linha ← linha − m·${rowName}. Assim o ${name} dessas linhas vira 0.`);
        for(const target of targets) {
          const entry=matrix[target][column],multiplier=entry.div(pivot),before=matrix[target];
          subtractMultiple(matrix,target,pivotRow,multiplier);
          result.lines.push(
            line('calculation',`m = ${dividend(entry)} ÷ ${factor(pivot)} = ${number(multiplier)}`),
            line('operation',operationText(target,pivotRow,multiplier)),
            line('calculation',eliminationText(before,matrix[pivotRow],multiplier,matrix[target]))
          );
        }
        result.matrix=view(matrix,{markedRows:targets,pivot:[pivotRow,column]});
        steps.push(result);
      }
      pivots.push([pivotRow,column]); pivotRow++;
    }
    const echelon=step('Forma escalonada','Abaixo de cada pivô só há zeros: a matriz está escalonada. Escrevendo cada linha de novo como equação:');
    matrix.forEach((row,index)=>echelon.lines.push(line('calculation',`${ROWS[index]}: ${equation(row)}`)));
    steps.push(echelon);
    const inconsistent=firstInconsistent(matrix);
    if(inconsistent>=0) { steps.push(contradictionStep(matrix,inconsistent)); return 'none'; }
    const free=freeColumns(count,pivots.map(([,column])=>column)),parameters=free.length,values=Array(count).fill(null);
    if(free.length) {
      const result=step('Variáveis livres',`Na forma escalonada, ${freeSentence(free)}. ${parameterSentence(free,pivots.length?'as outras incógnitas saem por substituição, de baixo para cima':null)} O sistema é possível e indeterminado (SPI).`);
      free.forEach((column,index)=>{
        values[column]=Affine.parameter(index,parameters);
        result.lines.push(line('calculation',`${VARIABLES[column]} = t${index+1}`));
      });
      steps.push(result);
    }
    for(let position=pivots.length-1;position>=0;position--) {
      const [row,column]=pivots[position],back=backSubstitutionStep(matrix,row,column,values,parameters,position+1===pivots.length);
      values[column]=back.value;
      steps.push(back.step);
    }
    const kind=free.length?'infinite':'unique',answer=step('Juntar a resposta',CLASSIFICATIONS[kind]);
    values.forEach((value,column)=>answer.lines.push(line('result',`${VARIABLES[column]} = ${value?value.text():'0'}`)));
    steps.push(answer);
    const check=verificationStep(input,solution);
    if(check) steps.push(check);
    return kind;
  }
  function walkthrough(input,methodKey) {
    const method=METHODS.find(item=>item.key===methodKey);
    if(!method) throw new Error('Método desconhecido.');
    const solution=solve(input),steps=[],determinants=[];
    const kind=method.key==='cramer'?cramer(steps,input,solution,determinants)
      :method.key==='gauss'?gauss(steps,input,solution)
      :finishGaussJordan(steps,input,gaussJordanElimination(steps,input),solution);
    return {method,kind,system:input.map(row=>equation(row)),steps,answer:answerLines(solution),classification:CLASSIFICATIONS[kind],determinants};
  }
  const matrixCells=matrix=>matrix.rows.map(row=>row.map(number));
  function matrixLines(matrix) {
    const cells=matrixCells(matrix),columns=cells.length?cells[0].length:0,width=text=>[...text].length;
    const widths=Array.from({length:columns},(_,column)=>Math.max(0,...cells.map(row=>width(row[column]))));
    const lines=matrix.label!=null?[`${matrix.label} =`]:[];
    for(const row of cells) {
      let text='[';
      row.forEach((cell,column)=>{
        if(matrix.augmented&&column+1===columns) text+=' |';
        text+=' '+' '.repeat(widths[column]-width(cell))+cell;
      });
      lines.push(text+' ]');
    }
    return lines;
  }
  function walkthroughText(walk) {
    let out=`Modo pensar · ${walk.method.name}\n\nSistema\n`;
    for(const text of walk.system) out+=text+'\n';
    walk.steps.forEach((item,index)=>{
      out+=`\nPasso ${index+1} · ${item.title}\n${item.reasoning}\n`;
      const matrix=item.matrix?matrixLines(item.matrix):[];
      if(item.matrixFirst) for(const text of matrix) out+=text+'\n';
      for(const entry of item.lines) out+=entry.text+'\n';
      if(!item.matrixFirst) for(const text of matrix) out+=text+'\n';
    });
    out+='\nResposta\n';
    for(const text of walk.answer) out+=text+'\n';
    return out+walk.classification;
  }
  const api={Rational,ParseError,PARSE_MESSAGES,solve,format,METHODS,walkthrough,walkthroughText,matrixLines,matrixCells,
    VARIABLES,MAX_INPUT_LENGTH,MAX_EXPONENT,MAX_POWER_DIGITS,LIVE_BUDGET_BITS,PREVIEW_DIGIT_LIMIT,
    trim,isWhitespace,usesExtension,raise,looksIncomplete,withinLiveBudget,Entry,abbreviate,previewEquation,equation};
  if(typeof module!=='undefined'&&module.exports) module.exports=api;
  else root.LinearSolver=api;
})(typeof globalThis!=='undefined'?globalThis:this);
