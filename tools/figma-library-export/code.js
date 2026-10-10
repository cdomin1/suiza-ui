// Local-only exporter. Reads the current selection/page; never modifies source nodes.
function rgb(c) { return [c.r, c.g, c.b].map(n => Math.round(Math.max(0, Math.min(1,n))*255)); }
function base(n, root, kind) {
  return { name:n.name, label:n.name, kind, x:n.absoluteTransform[0][2]-root.absoluteTransform[0][2], y:n.absoluteTransform[1][2]-root.absoluteTransform[1][2], width:n.width, height:n.height, color:[0,0,0], visible:n.visible, radius:0, group:null, locked:false, effects:[], typography:{size:null,bold:false,align:0,tracking:0,line_height:null}, mask:false };
}
// Figma vector paths use absolute SVG coordinates; retain cubic and quadratic curves.
function vectorPath(data,w,h) {
 const tokens=data.match(/[a-zA-Z]|[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?/g)||[];
 let i=0, cmd='', current=[0,0], contours=[], nodes=[], closed=true;
 const number=()=>{if(i>=tokens.length||/^[a-zA-Z]$/.test(tokens[i]))throw Error('Malformed vector path');const n=Number(tokens[i++]);if(!Number.isFinite(n))throw Error('Malformed vector coordinate');return n;};
 const normalized=p=>[p[0]/w,p[1]/h];
 const node=p=>({point:normalized(p),incoming:normalized(p),outgoing:normalized(p),mode:'Corner'});
 while(i<tokens.length){
  if(/^[a-zA-Z]$/.test(tokens[i]))cmd=tokens[i++];
  const relative=cmd===cmd.toLowerCase();const point=()=>{const p=[number(),number()];return relative?[p[0]+current[0],p[1]+current[1]]:p;};
  switch(cmd.toUpperCase()){
   case 'M': if(nodes.length)contours.push(nodes);nodes=[];current=point();nodes.push(node(current));cmd=relative?'l':'L';break;
   case 'L':current=point();nodes.push(node(current));break;
   case 'H':current=[number()+(relative?current[0]:0),current[1]];nodes.push(node(current));break;
   case 'V':current=[current[0],number()+(relative?current[1]:0)];nodes.push(node(current));break;
   case 'C':{if(!nodes.length)throw Error('Curve without start');const a=point(),b=point(),p=point();nodes[nodes.length-1].outgoing=normalized(a);const end=node(p);end.incoming=normalized(b);nodes.push(end);current=p;break;}
   case 'Q':{if(!nodes.length)throw Error('Curve without start');const q=point(),p=point(),a=current.map((x,j)=>x+(q[j]-x)*2/3),b=p.map((x,j)=>x+(q[j]-x)*2/3);nodes[nodes.length-1].outgoing=normalized(a);const end=node(p);end.incoming=normalized(b);nodes.push(end);current=p;break;}
   case 'Z': if(nodes.length>1){const a=nodes[0],last=nodes[nodes.length-1];if(a.point.every((v,j)=>Math.abs(v-last.point[j])<1e-6)){a.incoming=last.incoming;nodes.pop();}}if(nodes.length)contours.push(nodes);nodes=[];cmd='';break;
   default:throw Error('Unsupported SVG command');
  }
 }
 if(nodes.length){closed=false;contours.push(nodes);}if(!contours.length)throw Error('Empty vector');
 if(contours.flat().length>4096)throw Error('Too many vector nodes');
 return {nodes:contours[0],contours:contours.slice(1),closed,fill:true,stroke:false,stroke_color:[0,0,0],stroke_width:1};
}
async function exportLibrary(api) {
 const warnings=new Map(), warn=(n,s)=>warnings.set(s,(warnings.get(s)||0)+1);
 let visited=0;
 async function raster(n,root,layers,reason){
  warn(n,reason+' (rendered as an image)');const l=base(n,root,'Image');const bounds=n.absoluteRenderBounds||n.absoluteBoundingBox;
  if(bounds){l.x=bounds.x-root.absoluteTransform[0][2];l.y=bounds.y-root.absoluteTransform[1][2];l.width=bounds.width;l.height=bounds.height;}
  if(l.width<=0||l.height<=0)return;
  const scale=Math.min(1,2048/Math.max(l.width,l.height));l.image_data=Array.from(await n.exportAsync({format:'PNG',constraint:{type:'SCALE',value:scale}}));layers.push(l);
 }
 async function walk(n,root,layers,depth){
  if(++visited>100000||depth>64)throw Error('Library is too large; select a smaller component set.');
  if(!n.visible||!n.width||!n.height)return;
  if((n.rotation||0)!==0||n.opacity<0.999||n.isMask||('children' in n && n.children.some(c=>c.isMask))||(n.effects||[]).some(e=>e.visible!==false))return raster(n,root,layers,'Transforms, opacity, masks or effects');
  const paints=Array.isArray(n.fills)?n.fills.filter(p=>p.visible!==false):[];
  const strokes=Array.isArray(n.strokes)?n.strokes.filter(p=>p.visible!==false):[];
  if(paints.some(p=>p.type!=='SOLID'||(p.opacity??1)<0.999)||paints.length>1||strokes.length)return raster(n,root,layers,'Complex fills or borders');
  if(n.type==='TEXT'){
   if(typeof n.fontName!=='object'||typeof n.fontSize!=='number'||typeof n.letterSpacing!=='object'||typeof n.lineHeight!=='object')return raster(n,root,layers,'Mixed text styles');
   const l=base(n,root,'Text');l.name=n.characters;l.color=paints[0]?rgb(paints[0].color):[0,0,0];
   l.typography={...l.typography,size:n.fontSize,family:n.fontName.family,bold:/bold/i.test(n.fontName.style),italic:/italic/i.test(n.fontName.style),align:['LEFT','CENTER','RIGHT','JUSTIFIED'].indexOf(n.textAlignHorizontal),vertical:['TOP','CENTER','BOTTOM'].indexOf(n.textAlignVertical),tracking:n.letterSpacing.unit==='PIXELS'?n.letterSpacing.value:n.letterSpacing.value*n.fontSize/100,underline:n.textDecoration==='UNDERLINE',strike:n.textDecoration==='STRIKETHROUGH',transformation:n.textCase==='UPPER'?1:n.textCase==='LOWER'?2:n.textCase==='TITLE'?3:0};
   if(n.lineHeight.unit!=='AUTO')l.typography.line_height=n.lineHeight.unit==='PIXELS'?n.lineHeight.value:n.lineHeight.value*n.fontSize/100;
   if(n.paragraphSpacing||n.paragraphIndent)warn(n,'Paragraph spacing and indentation are not retained');
   layers.push(l);return;
  }
  if(paints.length){
   if(['RECTANGLE','ELLIPSE','FRAME','COMPONENT','INSTANCE','COMPONENT_SET'].includes(n.type)){
    const l=base(n,root,n.type==='ELLIPSE'?'Ellipse':'Rectangle');l.color=rgb(paints[0].color);l.radius=typeof n.cornerRadius==='number'?n.cornerRadius:0;
    if(n.type==='RECTANGLE'&&typeof n.topLeftRadius==='number')l.corners=[n.topLeftRadius,n.topRightRadius,n.bottomRightRadius,n.bottomLeftRadius];layers.push(l);
   }else if(n.type==='VECTOR'){
    try{const converted=[];for(const p of n.vectorPaths){const l=base(n,root,'Rectangle');l.color=rgb(paints[0].color);l.path=vectorPath(p.data,n.width,n.height);converted.push(l);}if(!n.vectorPaths.length)throw Error('Empty vector');layers.push(...converted);}
    catch(e){return raster(n,root,layers,'Unsupported vector geometry');}
   }else{return raster(n,root,layers,'Unsupported shape type');}
  }
  if('children' in n){
   if(n.clipsContent&&n!==root)return rasterChildren(n,root,layers);
   for(const child of n.children)await walk(child,root,layers,depth+1);
  }else if(!paints.length&&n.type!=='VECTOR')warn(n,'Empty or unsupported layers omitted');
 }
 async function rasterChildren(n,root,layers){return raster(n,root,layers,'Nested clipping');}
 let roots=Array.from(api.currentPage.selection);
 if(!roots.length){
  function collect(n){if(n.type==='COMPONENT'){roots.push(n);return;}if('children' in n)for(const c of n.children)collect(c);}
  for(const n of api.currentPage.children)collect(n);
  if(!roots.length)roots=Array.from(api.currentPage.children);
 }
 if(!roots.length)throw Error('Select components or open a page containing assets.');
 const boards=[];
 for(const root of roots){
  if(!root.width||!root.height||!root.visible)continue;
  if(root.width>8192||root.height>8192){warn(root,'Oversized frames omitted; select their components');continue;}
  const layers=[];await walk(root,root,layers,0);
  if(layers.length)boards.push({name:root.name,frame:{width:root.width,height:root.height,clip:!!root.clipsContent,background:[255,255,255]},layers});
 }
 if(!boards.length)throw Error('No importable components.');
 return {format:'suiza-library',version:1,document:{name:api.root.name,pages:[{name:api.currentPage.name,artboards:boards}]},warnings:Array.from(warnings,([s,n])=>`${s} (${n})`)};
}
if(typeof figma!=='undefined'){
 figma.showUI(__html__,{width:360,height:300});
 figma.ui.onmessage=async msg=>{if(msg.type!=='export')return;try{figma.ui.postMessage({type:'ready',data:await exportLibrary(figma)});}catch(e){figma.ui.postMessage({type:'error',message:String(e.message||e)});}};
}
if(typeof module!=='undefined')module.exports={vectorPath,exportLibrary};
