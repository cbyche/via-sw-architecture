// Rebuild the four reference-format DP pages from committed input snapshots.
// Copy this file into a private build directory linked to bundled node_modules.
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

const [repo, build, contentPath] = process.argv.slice(2);
if (!repo || !build || !contentPath) throw Error('Usage: build_reference_dp_comparisons.mjs REPO BUILD CONTENT.json');
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const py = process.env.VIA_RUNTIME_PYTHON;
if (!skill || !py) throw Error('Bundled presentation skill and Python paths are required');
const data = JSON.parse(await fs.readFile(contentPath, 'utf8'));
const p = Presentation.create({slideSize:{width:1920,height:1080}});
const C = {black:'#000000',green:'#C6E0B4',blue:'#376CB5',teal:'#37ACCD',gray:'#969898',grid:'#BFBFBF',cream:'#FFF4D7',red:'#C62525'};
const xml = s => String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
const allScenes=[];
let scene, slide;

function addNode(id,x,y,w,h,text='',opts={}){
  const n={id,x,y,w,h,text,geometry:'rect',fill:'#FFFFFF',stroke:'#000000',strokeWidth:1.1,pt:13,align:'center',valign:'middle',bold:false,color:'#000000',...opts};
  scene.nodes.push(n); return n;
}
function txt(id,x,y,w,h,text,pt=13,opts={}) { return addNode(id,x,y,w,h,text,{geometry:'textbox',fill:'none',stroke:'none',pt,align:'left',...opts}); }
function line(id,points,opts={}) { scene.edges.push({id,points,color:'#000000',width:1.3,arrow:false,dash:false,...opts}); }
function route(id,points,number,label,lx,ly,lw=200,opts={}){
  line(id,points,{arrow:true,...opts});
  if(number!==null&&number!==undefined)addNode(id+'-step',lx,ly,23,23,String(number),{fill:C.green,stroke:'#72AD49',geometry:'roundRect',pt:12,bold:true});
  if(label)txt(id+'-label',lx+(number==null?0:27),ly-2,lw,27,label,12,{fill:'#FFFFFF',color:opts.color??C.black});
}
function nativeShape(n){
  const shape=slide.shapes.add({name:n.id,geometry:n.geometry,position:{left:n.x,top:n.y,width:n.w,height:n.h},
    fill:n.fill,line:{fill:n.stroke,width:n.stroke==='none'?0:n.strokeWidth,style:n.dash?'dashed':'solid'},
    ...(n.geometry==='can'?{adjustmentList:[{name:'adj',formula:'val 9000'}]}:{})});
  if(n.text){
    shape.text=n.text;
    shape.text.style={typeface:'Calibri',fontSize:n.pt*4/3,color:n.color,bold:n.bold,alignment:n.align,
      verticalAlignment:n.valign,autoFit:'none',wrap:'none',insets:n.insets??{left:4,right:4,top:2,bottom:2},lineSpacing:1.0};
  }
}
function nativePath(e,pts=e.points,id=e.id){
  const xs=pts.map(a=>a[0]),ys=pts.map(a=>a[1]),x=Math.min(...xs),y=Math.min(...ys);
  const w=Math.max(.1,Math.max(...xs)-x),h=Math.max(.1,Math.max(...ys)-y);
  slide.shapes.add({name:id,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:e.color,width:e.width,style:e.dash?'dashed':'solid'},
    customPaths:[{width:w,height:h,commands:pts.map(([a,b],i)=>({[i?'lineTo':'moveTo']:{x:a-x,y:b-y}}))}]});
  if(e.arrow){
    const [a,b]=pts.slice(-2),t=Math.atan2(b[1]-a[1],b[0]-a[0]),r=8;
    nativePath({...e,arrow:false,dash:false},[[b[0]-r*Math.cos(t-.45),b[1]-r*Math.sin(t-.45)],b,[b[0]-r*Math.cos(t+.45),b[1]-r*Math.sin(t+.45)]],id+'-arrow');
  }
}
function circleScore(s){return ['○○○','◐○○','●○○','●◐○','●●○','●●◐','●●●'][s];}
function frame(d,idx){
  txt('title',64,29,1060,58,`DP-${d.dp}. ${d.title}`,30,{bold:true});
  const nav=['과제 소개','요구사항 분석','설계','구현 및 검증','결론'];
  nav.forEach((v,i)=>addNode('nav-'+i,1128+i*152,28,152,44,v,{geometry:i?'chevron':'rect',fill:i===2?C.teal:C.gray,stroke:'#FFFFFF',pt:13,bold:true,color:'#FFFFFF'}));
  line('title-rule',[[60,89],[1880,89]],{color:'#A8A8A8',width:2});
  addNode('fr',1440,95,466,24,d.functionalTag,{geometry:'roundRect',fill:'#75AF45',stroke:'none',pt:12,color:'#FFFFFF'});
  const count=d.qas.length,tagW=Math.min(300,1130/Math.max(count,1));
  d.qas.forEach((q,i)=>addNode('qa-tag-'+q.id,1906-tagW*count+i*tagW,123,tagW-7,23,q.id+'. '+q.name,{geometry:'roundRect',fill:C.blue,stroke:'none',pt:11,color:'#FFFFFF'}));
  if(!count)txt('no-qa-tag',1330,122,575,24,'QA01~06: A/B 점수 동점으로 비교 행 제외',12,{align:'right'});
  // The native table retains the reference's five rows and equal A/B columns.
  const rows=[46,544,78,78,148],values=[['설계안',d.options[0],d.options[1]],['구조','',''],['장점',d.pros[0].join('\n'),d.pros[1].join('\n')],['단점',d.cons[0].join('\n'),d.cons[1].join('\n')],['Trade\nOff','','']];
  const table=slide.tables.add({rows:5,columns:3,left:12,top:154,width:1896,height:894,columnWidths:[96,900,900],values});
  table.styleOptions={headerRow:false,bandedRows:false,firstColumn:false};
  table.borders.assign({fill:C.grid,width:1.3,style:'solid'});
  table.rows.forEach((r,i)=>r.height=rows[i]);
  for(let r=0;r<5;r++)for(let c=0;c<3;c++){
    const cell=table.getCell(r,c);cell.fill=r===0?C.green:'#FFFFFF';
    cell.text.style={typeface:'Calibri',fontSize:(r===0?19:c===0?15:14)*4/3,color:C.black,bold:r===0||c===0,
      alignment:r===0||c===0?'center':'left',verticalAlignment:r===1?'top':'middle',autoFit:'none',wrap:'none',lineSpacing:1.1,insets:{left:12,right:8,top:8,bottom:6}};
    if(r>=2&&r<=3&&c>0)for(const match of values[r][c].matchAll(/\(QA[^)]+\)/g))cell.text.get(match[0]).fill=C.red;
  }
  // Keep SVG/draw.io cells tied to the same table values and geometry.
  let yy=154;
  for(let r=0;r<5;r++){let xx=12;for(let c=0;c<3;c++){const w=c===0?96:900;
    scene.tableCells.push({id:`table-${r}-${c}`,x:xx,y:yy,w,h:rows[r],text:values[r][c],geometry:'rect',fill:r===0?C.green:'#FFFFFF',stroke:C.grid,strokeWidth:1.3,pt:r===0?19:c===0?15:14,bold:r===0||c===0,align:r===0||c===0?'center':'left',valign:r===1?'top':'middle',color:C.black});xx+=w;}yy+=rows[r];}
  // Compact shared legend in the left structural column.
  txt('legend-title',16,263,88,26,'Legend',13,{bold:true,align:'center'});
  addNode('legend-component',22,305,76,39,'Component',{pt:9,bold:true});
  addNode('legend-module',22,365,76,39,'Module',{geometry:'roundRect',pt:11});
  addNode('legend-state',22,427,76,39,'상태',{geometry:'can',pt:11});
  addNode('legend-diff',22,493,76,39,'비교 대상',{fill:C.cream,pt:10});
  line('legend-call',[[24,563],[94,563]],{arrow:true});txt('legend-call-label',16,571,88,35,'호출 / 전달',10,{align:'center'});
  line('legend-return',[[24,622],[94,622]],{arrow:true,dash:true});txt('legend-return-label',16,630,88,35,'반환 / 사실',10,{align:'center'});
  addNode('legend-step',42,690,28,26,'1',{fill:C.green,stroke:'#72AD49',pt:12});
  // Numeric data remains editable, with each A/B metric in the same position.
  if(!count){
    [108,1008].forEach((x,i)=>txt('trade-none-'+i,x+22,917,854,110,'QA01~06의 A/B 점수가 모두 같아 비교 행에서 제외\n원값 차이는 원문에 보존하며 점수 차이를 만들어 넣지 않음',14));
  } else {
    const step=Math.min(31,130/count),start=909+(130-step*count)/2;
    d.qas.forEach((q,i)=>['A','B'].forEach((opt,j)=>{
      const x=108+900*j,y=start+i*step;
      txt(`metric-${q.id}-${opt}`,x+17,y,292,29,q.id+' '+q.name,13);
      txt(`score-${q.id}-${opt}`,x+311,y,116,29,circleScore(q[opt].score)+` ${q[opt].score}/6`,13,{bold:true});
      txt(`value-${q.id}-${opt}`,x+434,y,440,29,q[opt].display+(q[opt].annotation?' ['+q[opt].annotation+']':''),12);
    }));
  }
  txt('evidence-status',14,1052,1845,24,'동일 관련 시나리오에 대한 구조 기반 예상값 / 미측정 / A/B 미선정 / ○=0, ◐=1, ●=2 (총 6점)',11);
  txt('page',1865,1052,39,24,String(idx+1),11,{align:'right'});
}

// Layout-specific diagram commands are kept in a separate reviewed content file.
function diagram(d){
  for(const n of d.diagram.nodes)addNode(n.id,n.x,n.y,n.w,n.h,n.text,n);
  for(const e of d.diagram.edges)line(e.id,e.points,e);
  for(const r of d.diagram.routes??[])route(r.id,r.points,r.number,r.label,r.lx,r.ly,r.lw,r);
}

function svgShape(n){
  const s=n.stroke??'none',fill=n.fill??'none',dash=n.dash?' stroke-dasharray="7 4"':'';
  let body='';
  if(n.geometry!=='textbox'){
    if(n.geometry==='can'){
      const r=Math.min(3,n.h*.09);
      body=`<path d="M ${n.x} ${n.y+r} C ${n.x} ${n.y-r/2} ${n.x+n.w} ${n.y-r/2} ${n.x+n.w} ${n.y+r} L ${n.x+n.w} ${n.y+n.h-r} C ${n.x+n.w} ${n.y+n.h+r/2} ${n.x} ${n.y+n.h+r/2} ${n.x} ${n.y+n.h-r} Z M ${n.x} ${n.y+r} C ${n.x} ${n.y+r*2.5} ${n.x+n.w} ${n.y+r*2.5} ${n.x+n.w} ${n.y+r}" fill="${fill}" stroke="${s}" stroke-width="${n.strokeWidth??1.1}"/>`;
    }
    else if(n.geometry==='chevron')body=`<path d="M${n.x},${n.y} L${n.x+n.w-22},${n.y} L${n.x+n.w},${n.y+n.h/2} L${n.x+n.w-22},${n.y+n.h} L${n.x},${n.y+n.h} L${n.x+22},${n.y+n.h/2} Z" fill="${fill}" stroke="${s}"/>`;
    else body=`<rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" rx="${n.geometry==='roundRect'?5:0}" fill="${fill}" stroke="${s}" stroke-width="${n.strokeWidth??1.1}"${dash}/>`;
  }else if(fill!=='none')body=`<rect x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" fill="${fill}"/>`;
  if(!n.text)return body;
  const lines=n.text.split('\n'),size=n.pt*4/3,leading=size*1.12;
  const x=n.align==='center'?n.x+n.w/2:n.align==='right'?n.x+n.w-5:n.x+5;
  const y=n.valign==='top'?n.y+size+3:n.y+(n.h-leading*lines.length)/2+size;
  return body+`<text font-family="Calibri, Noto Sans CJK KR, sans-serif" font-size="${size}" fill="${n.color??C.black}" font-weight="${n.bold?700:400}" text-anchor="${n.align==='center'?'middle':n.align==='right'?'end':'start'}">${lines.map((t,i)=>`<tspan x="${x}" y="${y+i*leading}">${xml(t).replace(/(\(QA[^)]+\))/g,'<tspan fill="'+C.red+'">$1</tspan>')}</tspan>`).join('')}</text>`;
}
function svgEdge(e){return `<polyline points="${e.points.map(a=>a.join(',')).join(' ')}" fill="none" stroke="${e.color}" stroke-width="${e.width}"${e.dash?' stroke-dasharray="6 4"':''}${e.arrow?' marker-end="url(#arrow)"':''}/>`;}
function svg(s){return `<svg xmlns="http://www.w3.org/2000/svg" width="1920" height="1080" viewBox="0 0 1920 1080"><defs><marker id="arrow" markerWidth="7" markerHeight="7" refX="6" refY="3.5" orient="auto"><path d="M0,0 L7,3.5 L0,7" fill="none" stroke="context-stroke"/></marker></defs><rect width="1920" height="1080" fill="white"/>${s.tableCells.map(svgShape).join('')}${s.nodes.filter(n=>n.layer==='back').map(svgShape).join('')}${s.edges.map(svgEdge).join('')}${s.nodes.filter(n=>n.layer!=='back').map(svgShape).join('')}</svg>`;}
function drawio(s){
  function cell(n){
    const style=`shape=${n.geometry==='can'?'cylinder':n.geometry==='textbox'?'text':n.geometry==='chevron'?'chevron':'rectangle'};rounded=${n.geometry==='roundRect'?1:0};whiteSpace=wrap;html=0;fillColor=${n.fill==='none'?'none':n.fill};strokeColor=${n.stroke};strokeWidth=${n.strokeWidth??1.1};fontFamily=Calibri;fontSize=${n.pt*4/3};fontColor=${n.color??C.black};fontStyle=${n.bold?1:0};align=${n.align};verticalAlign=${n.valign};spacing=4;${n.dash?'dashed=1;':''}`;
    return `<mxCell id="${xml(n.id)}" value="${xml(n.text).replaceAll('\n','&#xa;')}" style="${style}" vertex="1" parent="1"><mxGeometry x="${n.x}" y="${n.y}" width="${n.w}" height="${n.h}" as="geometry"/></mxCell>`;
  }
  const edges=s.edges.map(e=>`<mxCell id="${xml(e.id)}" style="edgeStyle=none;endArrow=${e.arrow?'open':'none'};strokeColor=${e.color};strokeWidth=${e.width};dashed=${e.dash?1:0};" edge="1" parent="1"><mxGeometry relative="1" as="geometry"><mxPoint x="${e.points[0][0]}" y="${e.points[0][1]}" as="sourcePoint"/><mxPoint x="${e.points.at(-1)[0]}" y="${e.points.at(-1)[1]}" as="targetPoint"/><Array as="points">${e.points.slice(1,-1).map(a=>`<mxPoint x="${a[0]}" y="${a[1]}"/>`).join('')}</Array></mxGeometry></mxCell>`).join('');
  return `<mxfile><diagram id="dp${s.dp}" name="DP-${s.dp}"><mxGraphModel page="1" pageWidth="1920" pageHeight="1080"><root><mxCell id="0"/><mxCell id="1" parent="0"/>${s.tableCells.map(cell).join('')}${s.nodes.filter(n=>n.layer==='back').map(cell).join('')}${edges}${s.nodes.filter(n=>n.layer!=='back').map(cell).join('')}</root></mxGraphModel></diagram></mxfile>`;
}

await fs.mkdir(build,{recursive:true});
for(const [i,d] of data.slides.entries()){
  slide=p.slides.add();slide.background.fill='#FFFFFF';scene={dp:d.dp,nodes:[],edges:[],tableCells:[]};
  frame(d,i);diagram(d);
  for(const n of scene.nodes.filter(n=>n.layer==='back'))nativeShape(n);
  for(const e of scene.edges)nativePath(e);
  for(const n of scene.nodes.filter(n=>n.layer!=='back'))nativeShape(n);
  slide.speakerNotes.textFrame.setText(d.notes+'\n\n출처\n'+JSON.stringify(data.sources,null,2)+'\n\n동점 제외 규칙: '+data.tieRule);
  allScenes.push(scene);
  await fs.writeFile(path.join(build,`dp${d.dp}-reference-comparison_v1.svg`),svg(scene));
  await fs.writeFile(path.join(build,`dp${d.dp}-reference-comparison_v1.drawio`),drawio(scene));
}
await fs.writeFile(path.join(build,'scene.json'),JSON.stringify(allScenes,null,2));
const draft=path.join(build,'authored.pptx'),candidate=path.join(build,'candidate.pptx');
await(await PresentationFile.exportPptx(p)).save(draft);
execFileSync(py,['-c',`import sys;from pathlib import Path;sys.path.insert(0,sys.argv[1]+'/scripts/presentations');import full_presentation_package as F;import presentation_windows_policy as W;p=F.read(Path(sys.argv[2]));p={k:W.normalize_xml(k,v) if k.endswith('.xml') else v for k,v in p.items()};F.write(Path(sys.argv[3]),p);print(W.policy_report(p))`,repo,draft,candidate],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const out=path.join(build,'final','VIA-DP-comparisons-reference_v1.pptx');await fs.mkdir(path.dirname(out),{recursive:true});
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:out,explicitTotalSlideCount:4,
  requiredNativeTableOwnerSlides:[1,2,3,4],pythonExecutable:py,
  integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...[1,2,3,4].flatMap(n=>['--require-native-table-slide',String(n)])],
  fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(out));
for(const [i,s] of imported.slides.items.entries()){
  const png=await imported.export({slide:s,format:'png',scale:1.5});
  await fs.writeFile(path.join(build,`dp${data.slides[i].dp}-reference-comparison_v1.png`),new Uint8Array(await png.arrayBuffer()));
  await fs.writeFile(path.join(build,`slide-${i+1}.layout.json`),await(await s.export({format:'layout'})).text());
}
console.log(out);
