import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';

// Initial migration only. Future edits must start from the canonical deck.
const [repoArg,buildArg]=process.argv.slice(2);
if(!repoArg||!buildArg)throw new Error('Usage: assemble_full_presentation.mjs REPO PRIVATE_BUILD');
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const python=process.env.VIA_RUNTIME_PYTHON,skill=process.env.VIA_PRESENTATION_SKILL_DIR;
if(![python,skill].every(v=>path.isAbsolute(v??'')))throw new Error('Set bundled runtime paths');
const destination=path.join(repo,'docs/presentations_files/VIA-architecture-presentation.pptx');
try{await fs.access(destination);throw new Error('Canonical deck already exists; use scoped replacement instead of reassembling');}
catch(e){if(e.code!=='ENOENT')throw e;}
const helper=path.join(repo,'scripts/presentations/full_presentation_package.py'),planPath=path.join(build,'plan.json');
execFileSync(python,[helper,'inventory',repo,planPath],{stdio:'inherit'});
const plan=JSON.parse(await fs.readFile(planPath,'utf8'));
await fs.mkdir(path.join(build,'source-renders'));
let page=0;
for(const input of plan){
 const p=await PresentationFile.importPptx(await FileBlob.load(input.snapshot));
 const inspection=await p.inspect({kind:'slide,layout,table,image,notes',maxChars:100000});
 await fs.writeFile(path.join(build,`source-${input.index}-inspection.ndjson`),inspection.ndjson);
 const scale=18288000/Number(input.size.cx);
 for(const slide of p.slides.items){
  const blob=await p.export({slide,format:'png',scale});
  await fs.writeFile(path.join(build,'source-renders',`${String(++page).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 }
 console.log(`Inspected source ${input.index}/7; ${page} source slides rendered`);
}
if(page!==32)throw new Error('Expected 32 existing content slides');
const scaffold=Presentation.create({slideSize:{width:1920,height:1080}});
for(let i=0;i<32;i++)scaffold.slides.add();
const scaffoldPath=path.join(build,'scaffold.pptx');
await(await PresentationFile.exportPptx(scaffold)).save(scaffoldPath);
const candidate=path.join(build,'merged.candidate.pptx'),manifestPath=path.join(build,'manifest.json');
execFileSync(python,[helper,'merge',planPath,scaffoldPath,candidate,manifestPath],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
const owners=manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page);
const reference=path.join(repo,'docs/presentations_files/quality-metrics/VIA-quality-metrics.pptx');
const final=path.join(build,'final','VIA-architecture-presentation.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:32,
 pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
 layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit','--validate-bullet-geometry',
  ...owners.flatMap(n=>['--require-native-table-slide',String(n)])],requiredNativeTableOwnerSlides:owners,
 fontPolicy:{basis:'reference',families:['Apple SD Gothic Neo','Arial Unicode MS'],referencePath:reference,
  referenceSha256:createHash('sha256').update(await fs.readFile(reference)).digest('hex')},
 verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
const p=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for(const [i,slide]of p.slides.items.entries()){
 const blob=await p.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 console.log(`Rendered final ${i+1}/32`);
}
// The caller reviews the final slides before publishing this file and manifest.
console.log(JSON.stringify({final,manifest:manifestPath,slides:32,sections:manifest.sections.length}));
