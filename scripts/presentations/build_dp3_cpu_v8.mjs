import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
import {Scene,C,validateScene} from './dp3_v8_scene.mjs';
import {groundingPage} from './dp3_v8_grounding_page.mjs';
import {executionPages} from './dp3_v8_execution_pages.mjs';
const [ROOT,BUILD]=process.argv.slice(2);
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1017.22135/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const FONT='Calibri',W=1920,H=1080,src=path.join(BUILD,'source-v7.pptx');
await fs.mkdir(path.join(BUILD,'diagrams'),{recursive:true});await fs.mkdir(path.join(BUILD,'previews'),{recursive:true});
const reference=await PresentationFile.importPptx(await FileBlob.load(src));
await fs.writeFile(path.join(BUILD,'reference-inspection.ndjson'),(await reference.inspect({select:'$.slides[1,5,16]',include:['id','index','layout','elements.text','elements.position','elements.style']})).ndjson);
const original=await fs.readFile(path.join(ROOT,'scripts/presentations/generate_dp3_reference_style.mjs'),'utf8');
const helpers=original.slice(original.indexOf('const line='),original.indexOf('function notes('));
const hp=path.join(BUILD,'helpers-v8.mjs');await fs.writeFile(hp,`export function helpers(ctx){const {FONT,W,H,C}=ctx;\n${helpers}\nreturn {nativeTable,highlightHeader};}`);
const h=(await import(pathToFileURL(hp))).helpers({FONT,W,H,C});
const pres=Presentation.create({slideSize:{width:W,height:H}});
const scenes=[groundingPage(Scene),...executionPages(Scene)];
// The comparison palette represents structural scope, never QA scores.
for(const d of scenes){
 for(const n of Object.values(d.nodes)){
  n.stroke=C.black;
  if(n.kind==='proc')n.stroke='#72777D';
  if(d.page===10&&n.id==='ainterpret')n.stroke=C.blue;
  if(d.page===10&&['bp','bassemble','bcandidates','binterpret','bcontrol','bwma','bipc','bexec','bmodel','bepisode'].includes(n.id))n.stroke='#32824A';
 }
 for(const e of d.items.filter(x=>x.kind==='edge')){
  if(d.page===10&&['a-infer','a-model-return'].includes(e.id))e.color=C.blue;
  if(d.page===10&&e.id.startsWith('b-')&&!['b-cloud','b-cloud-return'].includes(e.id))e.color='#32824A';
 }
}
const geometryChecks=scenes.map(validateScene);
await fs.writeFile(path.join(BUILD,'scene-model.json'),JSON.stringify(scenes,null,2));
await fs.writeFile(path.join(BUILD,'scene-checks.json'),JSON.stringify(geometryChecks,null,2));
function shape(sl,geom,id,x,y,w,hh,fill,stroke,width=1.7){return sl.shapes.add({name:id,geometry:geom,position:{left:x,top:y,width:w,height:hh},fill,line:{fill:stroke,width}});}
function textOn(q,t,size=24,opt={}){q.text=t;q.text.style={typeface:FONT,fontSize:size,color:opt.color||C.black,bold:!!opt.bold,alignment:opt.align||'center',verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:opt.insets||{left:5,right:5,top:2,bottom:2}};}
function text(sl,t,x,y,w,hh,size,opt={}){const q=shape(sl,'textbox','label '+t,x,y,w,hh,opt.fill||'none','none',0);textOn(q,t,size,opt);return q;}
function poly(sl,pts,id,{color=C.black,width=1.8,dashed=false}={}){const xs=pts.map(p=>p[0]),ys=pts.map(p=>p[1]),x=Math.min(...xs),y=Math.min(...ys),w=Math.max(.1,Math.max(...xs)-x),hh=Math.max(.1,Math.max(...ys)-y);return sl.shapes.add({name:id,geometry:'custom',position:{left:x,top:y,width:w,height:hh},fill:'none',line:{fill:color,width,style:dashed?'dashed':'solid'},customPaths:[{width:w,height:hh,commands:pts.map(([a,b],i)=>({[i?'lineTo':'moveTo']:{x:a-x,y:b-y}}))}]});}
function header(sl,d){text(sl,d.title,60,29,1140,68,44,{bold:true,align:'left'});['과제 소개','요구사항 분석','설계','구현 및 검증','결론'].forEach((v,i)=>{const q=shape(sl,'chevron','stage '+i,1210+i*134,33,143,44,i===2?C.cyan:C.gray,C.white);textOn(q,v,19,{bold:true,color:C.white});});poly(sl,[[58,108],[1860,108]],'header line',{color:'#A7A7A7',width:3});text(sl,d.subtitle,60,129,1800,47,27,{bold:true,align:'left'});text(sl,String(d.page),1835,1053,38,20,17,{align:'right',color:C.muted});}
for(const d of scenes){
 const sl=pres.slides.add();sl.background.fill=C.white;header(sl,d);
 for(const o of d.items){
  if(['comp','mod','store','proc','ext','doc','band'].includes(o.kind)){
   const geom={comp:'rect',mod:'roundRect',store:'can',proc:'rect',ext:'hexagon',doc:'foldedCorner',band:'rect'}[o.kind];
   const fill=o.kind==='comp'?C.cream:o.kind==='proc'?'none':o.kind==='ext'?C.pale:o.kind==='band'?C.green:C.white;
   const q=shape(sl,geom,'node:'+o.id,o.x,o.y,o.w,o.h,fill,o.stroke||C.grid,o.kind==='band'?1:1.7);
   if(o.kind==='proc')q.line={fill:o.stroke,width:1.8,style:'dashed'};
   const insets=o.kind==='comp'?{left:5,right:5,top:2,bottom:o.h-36}:o.kind==='proc'?{left:12,right:8,top:2,bottom:o.h-32}:o.kind==='store'?{left:6,right:6,top:8,bottom:3}:{left:5,right:5,top:2,bottom:2};
   if(o.kind==='store')q.adjustmentList=[{name:'adj',formula:'val 16000'}];
   textOn(q,o.text,o.size,{bold:['comp','proc','band','ext'].includes(o.kind),align:o.kind==='proc'?'left':'center',color:o.kind==='proc'?'#555B61':C.black,insets});
   if(o.kind==='comp')poly(sl,[[o.x,o.y+38],[o.x+o.w,o.y+38]],'divider '+o.id,{width:1});
  }else if(o.kind==='text'){text(sl,o.text,o.x,o.y,o.w,o.h,o.size,{...o,align:o.align||'left'});}
  else if(o.kind==='edge'||o.kind==='rule'){poly(sl,o.points,(o.kind==='edge'?'edge:':'rule:')+(o.id||''),o);if(o.kind==='edge'){const [ex,ey]=o.points.at(-1),[px,py]=o.points.at(-2),len=Math.hypot(ex-px,ey-py),ux=(ex-px)/len,uy=(ey-py)/len;const pts=[[ex,ey],[ex-9*ux+4*uy,ey-9*uy-4*ux],[ex-9*ux-4*uy,ey-9*uy+4*ux]];const xx=Math.min(...pts.map(p=>p[0])),yy=Math.min(...pts.map(p=>p[1])),ww=Math.max(...pts.map(p=>p[0]))-xx,hh=Math.max(...pts.map(p=>p[1]))-yy;sl.shapes.add({name:'arrow:'+o.id,geometry:'custom',position:{left:xx,top:yy,width:ww,height:hh},fill:o.color||C.black,line:{fill:o.color||C.black,width:.4},customPaths:[{width:ww,height:hh,commands:[...pts.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-xx,y:y-yy}})),{close:{}}]}]});}if(o.text&&o.label)text(sl,o.text,...o.label,o.size||21,{fill:C.white,align:'center'});}
  else if(o.kind==='table'){const t=h.nativeTable(sl,o.rows,o.x,o.y,o.widths,o.heights,o.size);h.highlightHeader(t,o.rows[0].length);t.cells.block({row:o.rows.length-1,column:0,rowCount:1,columnCount:o.rows[0].length}).assign({fill:C.pale,textStyle:{typeface:FONT,fontSize:26,bold:true,color:C.blue}});}
 }
 sl.speakerNotes.textFrame.setText(`DP3 v8 guide correction. SYNTHETIC_AUTHORED_NOT_MEASURED. Source: docs/architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/presentation-architecture-contract.md\n${d.notes}`);
}
// SVG and draw.io exports preserve node identity, containment and connected edges.
const esc=t=>String(t).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
function svgText(t,x,y,w,hh,size,{bold=false,align='center',color=C.black}={}){const lines=t.split('\n'),lh=size*1.12,xx=align==='left'?x+5:x+w/2,yy=y+(hh-lh*lines.length)/2+size*.9;return `<text fill="${color}" font-family="Calibri,Arial,sans-serif" font-size="${size}" font-weight="${bold?700:400}" text-anchor="${align==='left'?'start':'middle'}">${lines.map((v,i)=>`<tspan x="${xx}" y="${yy+i*lh}">${esc(v)}</tspan>`).join('')}</text>`;}
function svgNode(o){let sh='',{x,y,w,h:hh}=o;const stroke=o.stroke||C.grid,fill=o.fill|| (o.kind==='comp'?C.cream:o.kind==='proc'?'none':o.kind==='ext'?C.pale:o.kind==='band'?C.green:C.white);const common=`fill="${fill}" stroke="${stroke}" stroke-width="1.7"`;
 if(o.kind==='store'){const r=Math.min(8,hh*.08);sh=`<path d="M${x},${y+r}a${w/2},${r} 0 0 1 ${w},0v${hh-2*r}a${w/2},${r} 0 0 1 -${w},0Z" ${common}/><ellipse cx="${x+w/2}" cy="${y+r}" rx="${w/2}" ry="${r}" ${common}/>`;}
 else if(o.kind==='ext')sh=`<polygon points="${x+20},${y} ${x+w-20},${y} ${x+w},${y+hh/2} ${x+w-20},${y+hh} ${x+20},${y+hh} ${x},${y+hh/2}" ${common}/>`;
 else sh=`<rect x="${x}" y="${y}" width="${w}" height="${hh}" rx="${o.kind==='mod'?7:0}" ${common} ${o.kind==='proc'?'stroke-dasharray="8 6"':''}/>`;
 if(o.kind==='comp')sh+=`<path d="M${x} ${y+38}H${x+w}" stroke="${stroke}"/>`;
 return `<g id="${o.id}">${sh}${svgText(o.text,x,y+(o.kind==='store'?Math.min(16,hh*.16):0),w,['comp','proc'].includes(o.kind)?36:hh-(o.kind==='store'?Math.min(16,hh*.16):0),o.size,{bold:o.bold??['comp','proc','band','ext'].includes(o.kind),align:o.kind==='proc'?'left':'center',color:o.color||(o.kind==='proc'?'#555B61':C.black)})}</g>`;
}
function svg(d){let s=`<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080"><defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse"><path d="M0,0L10,5L0,10Z" fill="context-stroke"/></marker></defs><rect width="1920" height="1080" fill="white"/>${svgText(d.title,60,29,1140,68,44,{bold:true,align:'left'})}<path d="M58 108H1860" stroke="#A7A7A7" stroke-width="3"/>${svgText(d.subtitle,60,129,1800,47,27,{bold:true,align:'left'})}`;
 for(const o of d.items){if(d.nodes[o.id]||o.kind==='band')s+=svgNode(o);else if(o.kind==='text')s+=svgText(o.text,o.x,o.y,o.w,o.h,o.size,{...o,align:o.align||'left'});else if(o.kind==='edge'||o.kind==='rule'){s+=`<polyline id="${o.id||''}" points="${o.points.map(p=>p.join(',')).join(' ')}" fill="none" stroke="${o.color||C.black}" stroke-width="${o.width||1.8}" ${o.dashed?'stroke-dasharray="7 5"':''} ${o.kind==='edge'?'marker-end="url(#arrow)"':''}/>`;if(o.text&&o.label){const [x,y,w,hh]=o.label;s+=`<rect x="${x}" y="${y}" width="${w}" height="${hh}" fill="white"/>`+svgText(o.text,x,y,w,hh,o.size||21);}}else if(o.kind==='table'){let y=o.y;for(let r=0;r<o.rows.length;r++){let x=o.x;for(let c=0;c<o.rows[r].length;c++){s+=svgNode({kind:'band',fill:r===0?C.green:r===o.rows.length-1?C.pale:C.white,text:o.rows[r][c],x,y,w:o.widths[c],h:o.heights[r],size:r===o.rows.length-1?26:o.size,bold:r===0||r===o.rows.length-1,color:r===o.rows.length-1?C.blue:C.black});x+=o.widths[c];}y+=o.heights[r];}}}
 return s+svgText(`${d.page} / 29`,1790,1050,110,24,17,{color:C.muted})+'</svg>';
}
function drawio(d){let cs=['<mxCell id="0"/><mxCell id="1" parent="0"/>'],serial=0;const cell=(id,value,x,y,w,hh,style,parent='1')=>cs.push(`<mxCell id="${id}" value="${esc(value).replaceAll('\n','&#xa;')}" vertex="1" parent="${parent}" style="${style};fontFamily=Calibri;whiteSpace=wrap;html=0;"><mxGeometry x="${x}" y="${y}" width="${w}" height="${hh}" as="geometry"/></mxCell>`);
 cell('title',d.title,60,29,1140,68,'text;strokeColor=none;fillColor=none;fontSize=44;fontStyle=1;align=left');cell('subtitle',d.subtitle,60,129,1800,47,'text;strokeColor=none;fillColor=none;fontSize=27;fontStyle=1;align=left');
 for(const o of d.items){if(d.nodes[o.id]||o.kind==='band'){const p=o.parent?d.nodes[o.parent]:null;const parent=o.parent||'1',x=o.x-(p?.x||0),y=o.y-(p?.y||0);const geom=o.kind==='store'?'cylinder':o.kind==='ext'?'hexagon':'rectangle';const fill=o.kind==='comp'?C.cream:o.kind==='proc'?'none':o.kind==='ext'?C.pale:o.kind==='band'?C.green:C.white;cell(o.id||'band'+serial++,o.text,x,y,o.w,o.h,`shape=${geom};rounded=${o.kind==='mod'?1:0};fillColor=${fill};strokeColor=${o.stroke||C.grid};strokeWidth=1.7;fontSize=${o.size};fontStyle=${['comp','proc','band','ext'].includes(o.kind)?1:0};verticalAlign=${['comp','proc'].includes(o.kind)?'top':'middle'};spacingTop=${['comp','proc'].includes(o.kind)?5:0};dashed=${o.kind==='proc'?1:0}`,parent);if(o.kind==='comp'){cs.push(`<mxCell id="divider-${o.id}" edge="1" parent="${o.id}" style="endArrow=none;strokeColor=${o.stroke||C.black};strokeWidth=1;"><mxGeometry relative="1" as="geometry"><mxPoint x="0" y="38" as="sourcePoint"/><mxPoint x="${o.w}" y="38" as="targetPoint"/></mxGeometry></mxCell>`);}
 }else if(o.kind==='text')cell('text'+serial++,o.text,o.x,o.y,o.w,o.h,`text;strokeColor=none;fillColor=none;fontSize=${o.size};align=${o.align||'left'};fontStyle=${o.bold?1:0}`);
 else if(o.kind==='edge'||o.kind==='rule'){const [a,b]=[d.nodes[o.from],d.nodes[o.to]],first=o.points[0],last=o.points.at(-1);const anchor=a?`exitX=${(first[0]-a.x)/a.w};exitY=${(first[1]-a.y)/a.h};entryX=${(last[0]-b.x)/b.w};entryY=${(last[1]-b.y)/b.h};exitPerimeter=0;entryPerimeter=0;`:'';cs.push(`<mxCell id="${o.id||'rule'+serial++}" edge="1" parent="1" ${a?`source="${o.from}" target="${o.to}"`:''} style="${anchor}edgeStyle=none;endArrow=${a?'block':'none'};strokeColor=${o.color||C.black};dashed=${o.dashed?1:0};strokeWidth=1.8;"><mxGeometry relative="1" as="geometry"><mxPoint x="${first[0]}" y="${first[1]}" as="sourcePoint"/><mxPoint x="${last[0]}" y="${last[1]}" as="targetPoint"/><Array as="points">${o.points.slice(1,-1).map(([x,y])=>`<mxPoint x="${x}" y="${y}"/>`).join('')}</Array></mxGeometry></mxCell>`);if(o.text&&o.label)cell('flowlabel'+serial++,o.text,...o.label,`text;strokeColor=none;fillColor=#FFFFFF;fontSize=${o.size||21};align=center`);
 }else if(o.kind==='table'){let y=o.y;for(let r=0;r<o.rows.length;r++){let x=o.x;for(let c=0;c<o.rows[r].length;c++){cell('cell'+serial++,o.rows[r][c],x,y,o.widths[c],o.heights[r],`shape=rectangle;fillColor=${r===0?C.green:r===o.rows.length-1?C.pale:C.white};strokeColor=${C.grid};fontSize=${r===o.rows.length-1?26:o.size};fontStyle=${r===0||r===o.rows.length-1?1:0};fontColor=${r===o.rows.length-1?C.blue:C.black}`);x+=o.widths[c];}y+=o.heights[r];}}}
 return `<mxfile host="app.diagrams.net"><diagram name="${d.name}" id="${d.name}"><mxGraphModel dx="1920" dy="1080" page="1" pageWidth="1920" pageHeight="1080"><root>${cs.join('')}</root></mxGraphModel></diagram></mxfile>`;
}
for(const d of scenes){await fs.writeFile(path.join(BUILD,'diagrams',d.name+'.svg'),svg(d));await fs.writeFile(path.join(BUILD,'diagrams',d.name+'.drawio'),drawio(d));}
if(process.env.DP3_EXPORT_FIGURES_ONLY==='1')process.exit(0);
await(await PresentationFile.exportPptx(pres)).save(path.join(BUILD,'dp3-cpu-v8-authored.pptx'));
execFileSync(PY,[path.join(ROOT,'scripts/presentations/package_dp3_cpu_v8.py'),ROOT,BUILD],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')));
const final=path.join(BUILD,'output','VIA-DP1-DP5-reference-style-29slides-v8-dp3-guide-fixed-windows.pptx');
await finalizePresentation({workspaceDir:BUILD,candidatePath:path.join(BUILD,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:29,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...[12,13,14,15].flatMap(n=>['--require-native-table-slide',String(n)])],requiredNativeTableOwnerSlides:[12,13,14,15],fontPolicy:{basis:'reference',families:['Calibri'],referencePath:src,referenceSha256:createHash('sha256').update(await fs.readFile(src)).digest('hex')},verifyArtifactToolImport:true,receiptPath:path.join(BUILD,'validation.json')});
const result=await PresentationFile.importPptx(await FileBlob.load(final));
const pages=process.env.DP3_RENDER_ALL==='1'?Array.from({length:29},(_,i)=>i):[9,10,11];
for(const i of pages){const sl=result.slides.items[i];await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await(await result.export({slide:sl,format:'png',scale:1})).arrayBuffer()));if(i>=9&&i<=11)await fs.writeFile(path.join(BUILD,'previews',`slide-${i+1}.layout.json`),await(await sl.export({format:'layout'})).text());console.log('Rendered '+(i+1));}
console.log(final);
