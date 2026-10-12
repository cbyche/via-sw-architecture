import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {PresentationFile,FileBlob} from '@oai/artifact-tool';

const [repoArg,buildArg,mode='finalize']=process.argv.slice(2);
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const python=process.env.VIA_RUNTIME_PYTHON,skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const final=path.join(build,'final','VIA-architecture-presentation_v8.pptx');
if(mode==='finalize'){
  const manifest=JSON.parse(await fs.readFile(path.join(build,'manifest-v8.json'),'utf8'));
  const owners=manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page);
  await fs.mkdir(path.dirname(final),{recursive:true});
  const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
  await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate-v8.pptx'),finalPath:final,explicitTotalSlideCount:63,
    pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
    layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
    layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
    requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation-v8.json')});
}
const p=await PresentationFile.importPptx(await FileBlob.load(final));
const content=await p.inspect({select:'$.slides[*]',include:['id','index','title','elements.id','elements.kind','elements.name','elements.text','elements.position','notes.text']});
await fs.writeFile(path.join(build,'final-inspection.ndjson'),content.ndjson);
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for(const [i,slide] of p.slides.items.entries()){
  const png=await p.export({slide,format:'png',scale:1});
  await fs.writeFile(`${build}/renders/${String(i+1).padStart(2,'0')}.png`,new Uint8Array(await png.arrayBuffer()));
  if([12,13,14,15,17,18,20,22,23,24,25,62,63].includes(i+1)){
    const layout=await slide.export({format:'layout'});await fs.writeFile(`${build}/final-${i+1}.layout.json`,await layout.text());
  }
  console.log(`Rendered ${i+1}/63`);
}
console.log(JSON.stringify({final,repo,rendered:63}));
