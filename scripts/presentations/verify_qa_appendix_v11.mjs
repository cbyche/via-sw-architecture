import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {PresentationFile,FileBlob} from '@oai/artifact-tool';

const [repo,build]=process.argv.slice(2);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const python=process.env.VIA_RUNTIME_PYTHON;
if(!skill||!python)throw new Error('Set bundled skill and Python paths');
const manifest=JSON.parse(await fs.readFile(path.join(build,'manifest-v11.json'),'utf8'));
const owners=manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page);
const final=path.join(build,'final','VIA-architecture-presentation_v11.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
const{finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate-v11.pptx'),finalPath:final,explicitTotalSlideCount:33,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,requiredNativeChartOwnerSlides:[1],materializeLiteralChartWorkbooks:false,nativeChartTargetApplication:'portable',
  fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation-v11.json')});
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
manifest.output_sha256=hash(await fs.readFile(final));
if(hash(await fs.readFile(path.join(repo,manifest.source_path)))!==manifest.source_sha256)throw new Error('Source changed');
await fs.writeFile(path.join(build,'manifest-v11.json'),JSON.stringify(manifest,null,2)+'\n');
const p=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'final-renders'),{recursive:true});
for(const[index,s]of p.slides.items.entries()){
  const base=path.join(build,'final-renders',String(index+1).padStart(2,'0'));
  await fs.writeFile(base+'.png',new Uint8Array(await(await p.export({slide:s,format:'png',scale:1})).arrayBuffer()));
  await fs.writeFile(base+'.layout.json',await(await s.export({format:'layout'})).text());
  console.log('rendered',index+1);
}
console.log(JSON.stringify({final,slides:33,sha256:manifest.output_sha256}));
