import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {PresentationFile,FileBlob} from '@oai/artifact-tool';
const [build,mode='standalone']=process.argv.slice(2);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const python=process.env.VIA_RUNTIME_PYTHON;
if(![skill,python].every(v=>path.isAbsolute(v??'')))throw new Error('Set bundled presentation skill and Python paths');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const full=mode==='integrated';
const name=full?'VIA-architecture-presentation_v9.pptx':'VIA-project-introduction_v9.pptx';
const final=path.join(build,'final',name);
await fs.mkdir(path.dirname(final),{recursive:true});
const manifest=full?JSON.parse(await fs.readFile(path.join(build,'manifest-v9.json'),'utf8')):null;
const owners=full?manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page):[];
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,mode+'-candidate.pptx'),finalPath:final,
 explicitTotalSlideCount:full?63:2,pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
 layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
 requiredNativeTableOwnerSlides:owners,requiredNativeChartOwnerSlides:[1],materializeLiteralChartWorkbooks:!full,nativeChartTargetApplication:'portable',
 fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,mode+'-validation.json')});
const p=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,mode+'-renders'),{recursive:true});
for(const [i,slide]of p.slides.items.entries()){
 const png=await p.export({slide,format:'png',scale:1});await fs.writeFile(path.join(build,mode+'-renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
 if(i<2){const layout=await slide.export({format:'layout'});await fs.writeFile(path.join(build,`${mode}-${i+1}.layout.json`),await layout.text());}
 console.log(`Rendered ${i+1}/${p.slides.items.length}`);
}
if(full){manifest.output_sha256=createHash('sha256').update(await fs.readFile(final)).digest('hex');await fs.writeFile(path.join(build,'manifest-v9.json'),JSON.stringify(manifest,null,2)+'\n');}
console.log(final);
