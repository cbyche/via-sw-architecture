import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {PresentationFile, FileBlob} from '@oai/artifact-tool';

const [repoArg, buildArg] = process.argv.slice(2);
const repo=await fs.realpath(repoArg), build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR, python=process.env.VIA_RUNTIME_PYTHON;
if (![skill,python,process.env.RUNTIME_NODE_MODULES].every(v=>path.isAbsolute(v??''))) throw new Error('Set bundled runtime paths');
const source=path.join(repo,'docs/presentations_files/VIA-architecture-presentation_v5.pptx');
const previous=path.join(repo,'docs/presentations_files/full-presentation/manifest-v5.json');
const planPath=path.join(build,'plan.json');
const plan=JSON.parse(await fs.readFile(planPath,'utf8'));
const hash=data=>createHash('sha256').update(data).digest('hex');
if (hash(await fs.readFile(source))!==plan.source_sha256) throw new Error('Source changed');
const p=await PresentationFile.importPptx(await FileBlob.load(source));
const snapshot=await p.inspect({kind:'slide,shape,textbox,table',include:'id,slide,name,text,rows,cols',maxChars:4000000});
const records=snapshot.ndjson.split('\n').filter(Boolean).map(JSON.parse);
const editReceipt={textboxes:0,cells:0,textEdits:0,headers:0};
for (const record of records) {
  if (record.kind==='textbox') {
    const object=p.resolve(record.id);
    object.text.style={typeface:'Calibri',color:'#000000',autoFit:'none'};
    editReceipt.textboxes++;
    const update=plan.pages[record.slide-1].shape_updates.find(u=>u.name===record.name);
    if (update) { object.text.replace(update.old,update.new); editReceipt.textEdits++; }
  } else if (record.kind==='table') {
    const table=p.resolve(record.id);
    for(let r=0;r<record.rows;r++)for(let c=0;c<record.cols;c++) {
      table.getCell(r,c).text.style={typeface:'Calibri',color:'#000000',autoFit:'none'};
      editReceipt.cells++;
    }
  }
}
for (const info of plan.pages) {
  const slideRecord=records.find(r=>r.kind==='slide'&&r.slide===info.page);
  const slide=p.resolve(slideRecord.id);
  for (const cell of info.native_header_cells) {
    const [left,top,width,height]=cell.box, [l,t,r,b]=cell.text_box;
    const node=slide.shapes.add({name:cell.name,geometry:'rect',position:{left,top,width,height},
      fill:'#C6E0B4',line:{fill:'none',width:0}});
    node.text=cell.text;
    node.text.style={typeface:'Calibri',fontSize:cell.font_size_pt*4/3,bold:cell.bold,color:'#000000',
      autoFit:'none',alignment:cell.align==='ctr'?'center':'left',verticalAlignment:'middle',
      insets:{left:l,right:r,top:t,bottom:b}};
  }
  const tableRecords=records.filter(r=>r.kind==='table'&&r.slide===info.page);
  for (const update of info.table_updates) {
    p.resolve(tableRecords[update.table].id).getCell(update.row,update.col).text.replace(update.old,update.new);
    editReceipt.textEdits++;
  }
  const title=slide.shapes.add({name:'v6-header-title',geometry:'textbox',
    position:{left:60,top:20,width:1070,height:66},fill:'none',line:{fill:'none',width:0}});
  title.text=info.title;
  title.text.style={typeface:'Calibri',fontSize:40,bold:true,color:'#000000',autoFit:'none',
    insets:{left:0,right:0,top:0,bottom:0},alignment:'left',verticalAlignment:'middle'};
  slide.shapes.add({name:'v6-header-rule',geometry:'line',
    position:{left:52,top:104,width:1820,height:0},fill:'none',line:{fill:'#A6A6A6',width:2}});
  const stages=['과제 소개','요구사항 분석','설계','검토 요약','Appendix'];
  const active=info.page<=6?0:info.page<=11?1:info.page<=22?2:info.page<=24?3:4;
  for (let i=0;i<5;i++) {
    const node=slide.shapes.add({name:`v6-header-stage-${i}`,geometry:'chevron',
      position:{left:1140+i*146,top:28,width:158,height:44},
      fill:i===active?'#3AAFD0':'#96989A',line:{fill:'#FFFFFF',width:2},
      adjustmentList:[{name:'adj',formula:'val 18000'}]});
    node.text=stages[i];
    node.text.style={typeface:'Calibri',fontSize:56/3,bold:true,color:'#FFFFFF',autoFit:'none',
      alignment:'center',verticalAlignment:'middle',insets:{left:19,right:17,top:0,bottom:0}};
  }
  const pageNumber=slide.shapes.add({name:'v6-header-page',geometry:'textbox',
    position:{left:1750,top:77,width:120,height:23},fill:'none',line:{fill:'none',width:0}});
  pageNumber.text=`${info.page} / 61`;
  pageNumber.text.style={typeface:'Calibri',fontSize:40/3,color:'#000000',autoFit:'none',alignment:'right',
    insets:{left:0,right:0,top:0,bottom:0}};
  editReceipt.headers++;
}
const authored=path.join(build,'authored.pptx');
await (await PresentationFile.exportPptx(p)).save(authored);
await fs.writeFile(path.join(build,'artifact-edit.json'),JSON.stringify(editReceipt,null,2)+'\n');
const candidate=path.join(build,'candidate.pptx'),manifestPath=path.join(build,'manifest-v6.json');
execFileSync(python,[path.join(repo,'scripts/presentations/restyle_presentation_v6.py'),'finish',
  source,authored,planPath,previous,candidate,manifestPath],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
const owners=manifest.slides.filter(r=>r.native_tables>0).map(r=>r.page);
const final=path.join(build,'final','VIA-architecture-presentation_v6.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:61,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',
    ...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'user_request',families:['Calibri']},
  verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
manifest.output_sha256=hash(await fs.readFile(final));
await fs.writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
execFileSync(python,[path.join(repo,'scripts/presentations/presentation_windows_policy.py'),'check',final],{stdio:'inherit'});
const rendered=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
await fs.mkdir(path.join(build,'layouts'),{recursive:true});
for(const [i,slide] of rendered.slides.items.entries()) {
  const png=await rendered.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
  await fs.writeFile(path.join(build,'layouts',`${String(i+1).padStart(2,'0')}.json`),await (await slide.export({format:'layout'})).text());
  if((i+1)%10===0)console.log(`Rendered ${i+1}/61`);
}
if(hash(await fs.readFile(source))!==plan.source_sha256)throw new Error('Source changed during edit');
console.log(JSON.stringify({final,manifestPath,slides:61,...editReceipt}));
