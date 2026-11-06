const fs=require('node:fs'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const read=name=>fs.readFileSync(path.join(root,name),'utf8').replace(/^\uFEFF/,'');
let html=read('portable/shell.html');
const font=fs.readFileSync(path.join(root,'assets/SpaceGrotesk.woff2')).toString('base64');
const icons=['matrix','grid','arrow-right','external','copy','reset','sparkles'].map(name=>{
 const content=read(`assets/${name}.svg`).replace(/<svg[^>]*>/,'').replace(/<\/svg>\s*$/,'');return `<symbol id="icon-${name}" viewBox="0 0 24 24">${content}</symbol>`;
}).join('');
const parts={STYLE:read('portable/styles.css').replace('__FONT__',font),ICONS:`<svg xmlns="http://www.w3.org/2000/svg" style="display:none" aria-hidden="true">${icons}</svg>`,SOLVER:read('portable/solver.js'),APP:read('portable/app.js')};
for(const [key,value] of Object.entries(parts))html=html.replace(`__${key}__`,()=>value);
fs.writeFileSync(path.join(root,'SistemasLineares.html'),html);
console.log('Versão offline gerada: SistemasLineares.html');
