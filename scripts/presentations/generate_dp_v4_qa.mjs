import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';

// Run a private copy linked to bundled node_modules. v3 files are read-only.
const [repoArg,buildArg]=process.argv.slice(2);
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
if(![skill,python].every(v=>path.isAbsolute(v??'')))throw new Error('Absolute skill and Python paths required');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const scenesFile=path.join(build,'scenes.json');
execFileSync(python,[path.join(repo,'scripts/presentations/build_dp_v4_qa_scenes.py'),scenesFile]);
const {scenes}=JSON.parse(await fs.readFile(scenesFile,'utf8'));
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const font='Apple SD Gothic Neo',ink='#18232E';
for(const scene of scenes){
 const slide=p.slides.add();slide.background.fill='#FFFFFF';
 for(const i of scene.items){
  const name=`${scene.slug} / ${i.id}`,position={left:i.x,top:i.y,width:i.w,height:i.h};
  if(i.kind==='text'){
   const shape=slide.shapes.add({name,geometry:'textbox',position,fill:'none',line:{fill:'none',width:0}});
   shape.text=i.content;shape.text.style={typeface:font,fontSize:i.size,bold:i.bold,color:i.color,
    alignment:'left',verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  }else if(i.kind==='line'){
   slide.shapes.add({name,geometry:'custom',position,fill:'none',line:{fill:i.color,width:1.2},
    customPaths:[{width:i.w,height:i.h,commands:[{moveTo:{x:0,y:0}},{lineTo:{x:i.w,y:i.h<=.1?0:i.h}}]}]});
  }else if(i.kind==='rect'){
   slide.shapes.add({name,geometry:'rect',position,fill:i.fill,line:{fill:'none',width:0}});
  }else if(i.kind==='score-circle'){
   slide.shapes.add({name:name+' / circle',geometry:'ellipse',position,fill:i.value===2?ink:'#FFFFFF',line:{fill:ink,width:1.1}});
   if(i.value===1){
    // A native half-disc follows the reference's filled/half/open circles.
    const r=i.w/2,commands=[{moveTo:{x:r,y:0}}];
    for(let k=1;k<=32;k++){
     const theta=-Math.PI/2-k*Math.PI/32;
     commands.push({lineTo:{x:r+r*Math.cos(theta),y:r+r*Math.sin(theta)}});
    }
    commands.push({close:{}});
    slide.shapes.add({name:name+' / half',geometry:'custom',position,fill:ink,line:{fill:'none',width:0},
     customPaths:[{width:i.w,height:i.h,commands}]});
   }
  }else throw new Error('Unsupported native object '+i.kind);
 }
 slide.speakerNotes.textFrame.setText(scene.notes);
}
const candidate=path.join(build,'qa-native-candidate.pptx');
await (await PresentationFile.exportPptx(p)).save(candidate);
const plans=[
 {stem:'VIA-DP-background-and-comparison-41-45',folder:'docs/presentations_files',count:16},
 {stem:'VIA-DP-comparison-41-45',folder:'docs/presentations_files/dp-comparison',count:9},
];
const records=[];
for(const plan of plans){
 const source=path.join(repo,plan.folder,plan.stem+'_v3.pptx');
 const draft=path.join(build,plan.stem+'_v4.candidate.pptx');
 const final=path.join(build,'final',plan.stem+'_v4.pptx');
 const destination=path.join(repo,plan.folder,plan.stem+'_v4.pptx');
 await fs.mkdir(path.dirname(final),{recursive:true});
 const preservation=path.join(build,plan.stem+'.preservation.json');
 execFileSync(python,[path.join(repo,'scripts/presentations/dp_v4_qa_package.py'),candidate,source,draft,scenesFile,preservation]);
 await finalizePresentation({workspaceDir:build,candidatePath:draft,finalPath:final,explicitTotalSlideCount:plan.count,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit'],
  requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,
  receiptPath:path.join(build,plan.stem+'.validation.json')});
 const imported=await PresentationFile.importPptx(await FileBlob.load(final));
 const deckDir=path.join(build,'renders',plan.stem);await fs.mkdir(deckDir,{recursive:true});
 for(const [index,slide] of imported.slides.items.entries()){
  const blob=await imported.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(deckDir,`${String(index+1).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 }
 await fs.copyFile(final,destination);
 records.push({file:path.relative(repo,destination),slides:plan.count});
 console.log('PASS:',path.basename(destination),plan.count,'slides / native score circles and editable QA text');
}
await fs.writeFile(path.join(build,'outputs.json'),JSON.stringify(records,null,2));
