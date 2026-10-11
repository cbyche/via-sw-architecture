import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';

// Maintain only the integrated v5. All older decks and Architecture inputs are read-only.
const [repoArg,buildArg]=process.argv.slice(2);
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
if(![skill,python].every(v=>path.isAbsolute(v??'')))throw new Error('Absolute skill and Python paths required');
const source=path.join(repo,'docs/presentations_files/VIA-DP-background-and-comparison-41-45_v5.pptx');
const referenceSha256=createHash('sha256').update(await fs.readFile(source)).digest('hex');
const scenesFile=path.join(build,'scenes.json');
const snapshot=path.join(repo,'docs/presentations_files/dp-comparison/v5/qa-review.json');
execFileSync(python,[path.join(repo,'scripts/presentations/build_dp_v5_qa_scenes.py'),scenesFile,snapshot],{stdio:'inherit'});
const {scenes}=JSON.parse(await fs.readFile(scenesFile,'utf8'));
const p=Presentation.create({slideSize:{width:1920,height:1080}}),font='Apple SD Gothic Neo';
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
  }else throw new Error('Unsupported native object '+i.kind);
 }
 slide.speakerNotes.textFrame.setText(scene.notes);
}
const nativeCandidate=path.join(build,'qa-native-candidate.pptx');
await(await PresentationFile.exportPptx(p)).save(nativeCandidate);
const candidate=path.join(build,'integrated-v5.candidate.pptx');
const preservation=path.join(build,'preservation.json');
execFileSync(python,[path.join(repo,'scripts/presentations/dp_v5_qa_package.py'),nativeCandidate,source,candidate,preservation],{stdio:'inherit'});
const filename=path.basename(source),final=path.join(build,'final',filename);
await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:14,
 pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
 layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit'],
 requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'reference',families:[font],referencePath:source,referenceSha256},
 verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
const renderDir=path.join(build,'renders');await fs.mkdir(renderDir,{recursive:true});
for(const [index,slide] of imported.slides.items.entries()){
 const blob=await imported.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(renderDir,`${String(index+1).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 console.log('Rendered final '+(index+1)+'/14');
}
await fs.copyFile(final,source);
const previews=path.join(repo,'docs/presentations_files/dp-comparison/v5');
for(const [dp,comparison,detail] of [[41,4,5],[44,7,8],[45,10,11],[42,13,14]]){
 for(const [kind,num] of [['comparison',comparison],['qa-detail',detail]])
  await fs.copyFile(path.join(renderDir,String(num).padStart(2,'0')+'.png'),path.join(previews,`dp${dp}-${kind}_v5.png`));
}
console.log('PASS: single integrated v5, current six QA criteria and P21 references, no invented candidate grades');
