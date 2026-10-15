import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {PresentationFile,FileBlob} from '@oai/artifact-tool';
const [repo,build]=process.argv.slice(2);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const python=process.env.VIA_RUNTIME_PYTHON;
if(!repo||!build||!skill||!python)throw new Error('Supply REPO BUILD and bundled runtime environment');
await fs.mkdir(build,{recursive:true});
const specs=[
 {tag:'necessity10',file:'VIA-project-introduction_v10.pptx',pages:[2],targets:[2],marker:'v6-header-page'},
 {tag:'necessity9',file:'VIA-project-introduction_v9.pptx',pages:[2],targets:[3],marker:'v6-header-page'},
 {tag:'reference',file:'VIA-DP-comparisons-reference_v1.pptx',pages:[1,2,3,4],targets:[21,22,23,24],marker:'page'},
];
for(const spec of specs){
 const source=path.join(repo,'docs/presentations_files',spec.file);
 const p=await PresentationFile.importPptx(await FileBlob.load(source));
 for(const[index,page]of spec.pages.entries()){
  const s=p.slides.items[page-1];
  await fs.writeFile(path.join(build,`before-${spec.tag}-${page}.png`),new Uint8Array(await(await p.export({slide:s,format:'png',scale:1})).arrayBuffer()));
  const marker=s.shapes.items.find(x=>x.name===spec.marker);
  if(!marker)throw new Error('Missing source page marker');
  marker.text=`${spec.targets[index]} / 49`;
  if(spec.tag==='reference')marker.position={left:1784,top:1052,width:120,height:24};
 }
 await(await PresentationFile.exportPptx(p)).save(path.join(build,`edited-${spec.tag}.pptx`));
}
execFileSync(python,[path.join(repo,'scripts/presentations/package_reference_comparisons_v17.py'),repo,build],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(path.join(build,'manifest-v17.json'),'utf8'));
const owners=manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page);
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')));
await fs.mkdir(path.join(build,'final'),{recursive:true});
const final=path.join(build,'final/VIA-architecture-presentation_v17.pptx');
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:49,
 pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
 requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation-v17.json')});
manifest.output_sha256=createHash('sha256').update(await fs.readFile(final)).digest('hex');
await fs.writeFile(path.join(build,'manifest-v17.json'),JSON.stringify(manifest,null,2)+'\n');
const deck=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for(const[i,s]of deck.slides.items.entries()){
 await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await(await deck.export({slide:s,format:'png',scale:i===1||i===2||(i>=20&&i<=23)?1:.5})).arrayBuffer()));
 console.log('Rendered',i+1);
}
console.log(final);
