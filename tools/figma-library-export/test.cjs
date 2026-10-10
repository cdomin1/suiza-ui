const assert=require('node:assert/strict');
const fs=require('node:fs');
const {vectorPath,exportLibrary}=require('./code.js');
const p=vectorPath('M 0 0 C 25 50 75 50 100 100',100,100);
assert.deepEqual(p.nodes[0].outgoing,[.25,.5]);assert.deepEqual(p.nodes[1].incoming,[.75,.5]);assert.equal(p.closed,false);
const q=vectorPath('M 0 0 Q 50 100 100 0 Z',100,100);assert.equal(q.closed,true);assert.equal(q.nodes.length,2);assert.ok(Math.abs(q.nodes[0].outgoing[1]-2/3)<1e-6);
assert.throws(()=>vectorPath('M0 0 A10 10 0 0 0 20 20',100,100));
const matrix=(x,y)=>[[1,0,x],[0,1,y]];
const text={id:'t',type:'TEXT',name:'Label',characters:'Continue',width:80,height:20,visible:true,absoluteTransform:matrix(116,218),fills:[{type:'SOLID',color:{r:1,g:1,b:1}}],fontName:{family:'Inter',style:'Bold'},fontSize:14,letterSpacing:{unit:'PIXELS',value:0},lineHeight:{unit:'AUTO'},textAlignHorizontal:'CENTER',textAlignVertical:'CENTER'};
const root={type:'COMPONENT',name:'Button',width:120,height:56,visible:true,absoluteTransform:matrix(100,200),fills:[{type:'SOLID',color:{r:.2,g:.3,b:.8}}],cornerRadius:12,children:[text]};
(async()=>{
 const data=await exportLibrary({root:{name:'Independent test library'},currentPage:{name:'Components',selection:[root]}});
 assert.equal(data.document.pages[0].artboards[0].layers[1].x,16);assert.equal(data.document.pages[0].artboards[0].layers[1].y,18);assert.equal(data.document.pages[0].artboards[0].layers[1].typography.bold,true);assert.equal(text.characters,'Continue');assert.deepEqual(data.warnings,[]);
 if(process.env.SUIZA_BRIDGE_FIXTURE)fs.writeFileSync(process.env.SUIZA_BRIDGE_FIXTURE,JSON.stringify(data));
 const png=[137,80,78,71];const effect={...root,effects:[{type:'DROP_SHADOW',visible:true}],exportAsync:async()=>new Uint8Array(png)};
 const fallback=await exportLibrary({root:{name:'Test'},currentPage:{name:'Components',selection:[effect]}});assert.equal(fallback.document.pages[0].artboards[0].layers[0].kind,'Image');assert.ok(fallback.warnings[0].includes('rendered as an image'));
 console.log('Figma bridge: curve geometry, editable typography, offsets, source preservation and raster fallback passed.');
})().catch(e=>{console.error(e);process.exitCode=1;});
