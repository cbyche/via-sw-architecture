import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {PresentationFile, FileBlob} from '@oai/artifact-tool';

const [build] = process.argv.slice(2);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(v=>path.isAbsolute(v??''))) throw new Error('Set bundled skill and Python paths');
const {finalizePresentation} = await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const manifest = JSON.parse(await fs.readFile(path.join(build,'manifest-v13.json'),'utf8'));
const owners = manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page);
const final = path.join(build,'final','VIA-architecture-presentation_v13.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate.pptx'),finalPath:final,
 explicitTotalSlideCount:37,pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
 layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
 requiredNativeTableOwnerSlides:owners,requiredNativeChartOwnerSlides:[1],materializeLiteralChartWorkbooks:false,nativeChartTargetApplication:'portable',
 fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
const p = await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for (const [i,slide] of p.slides.items.entries()) {
 const png = await p.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
 if (i<7) {const layout=await slide.export({format:'layout'});await fs.writeFile(path.join(build,`${i+1}.layout.json`),await layout.text());}
 console.log(`Rendered ${i+1}/37`);
}
manifest.output_sha256=createHash('sha256').update(await fs.readFile(final)).digest('hex');
await fs.writeFile(path.join(build,'manifest-v13.json'),JSON.stringify(manifest,null,2)+'\n');
console.log(final);
