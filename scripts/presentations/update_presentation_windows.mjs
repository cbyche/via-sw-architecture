import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {PresentationFile, FileBlob} from '@oai/artifact-tool';

const [repoArg, buildArg] = process.argv.slice(2);
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(v => path.isAbsolute(v ?? ''))) throw new Error('Set bundled runtime paths');
const source = path.join(repo, 'docs/presentations_files/VIA-architecture-presentation_v4.pptx');
const previousPath = path.join(repo, 'docs/presentations_files/full-presentation/manifest-v4.json');
const previous = JSON.parse(await fs.readFile(previousPath, 'utf8'));
const hash = value => createHash('sha256').update(value).digest('hex');
const sourceHash = hash(await fs.readFile(source));
if (sourceHash !== previous.output_sha256) throw new Error('Source changed; rebase this edit');
const p = await PresentationFile.importPptx(await FileBlob.load(source));
const snapshot = await p.inspect({kind:'textbox,table', include:'id,slide,rows,cols', maxChars:2000000});
const records = snapshot.ndjson.split('\n').filter(Boolean).map(JSON.parse);
const seen = new Set();
let textboxes = 0, cells = 0;
for (const row of records) {
  const object = p.resolve(row.id);
  if (row.kind === 'textbox') {
    object.text.style = {typeface:'Calibri'};
    seen.add(row.slide); textboxes++;
  } else if (row.kind === 'table') {
    for (let r=0;r<row.rows;r++) for (let c=0;c<row.cols;c++) {
      object.getCell(r,c).text.style = {typeface:'Calibri'};
      cells++;
    }
    seen.add(row.slide);
  }
}
if (seen.size !== 61 || textboxes === 0 || cells === 0) throw new Error('Incomplete font edit coverage');
const authored = path.join(build, 'authored.pptx');
await (await PresentationFile.exportPptx(p)).save(authored);
await fs.writeFile(path.join(build,'artifact-font-edit.json'), JSON.stringify({sourceHash,textboxes,cells,slides:seen.size},null,2));
const candidate = path.join(build,'candidate.pptx'), manifestPath = path.join(build,'manifest-v5.json');
const policyScript = path.join(repo,'scripts/presentations/presentation_windows_policy.py');
execFileSync(python, [policyScript,'normalize',source,authored,previousPath,candidate,manifestPath], {stdio:'inherit'});
const manifest = JSON.parse(await fs.readFile(manifestPath,'utf8'));
const owners = manifest.slides.filter(r=>r.native_tables>0).map(r=>r.page);
const final = path.join(build,'final','VIA-architecture-presentation_v5.pptx');
await fs.mkdir(path.dirname(final), {recursive:true});
const {finalizePresentation} = await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:61,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',
    ...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'user_request',families:['Calibri']},
  verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
manifest.output_sha256 = hash(await fs.readFile(final));
await fs.writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
execFileSync(python, [policyScript,'check',final], {stdio:'inherit'});
const rendered = await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
await fs.mkdir(path.join(build,'layouts'),{recursive:true});
for (const [i,slide] of rendered.slides.items.entries()) {
  const png = await rendered.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
  await fs.writeFile(path.join(build,'layouts',`${String(i+1).padStart(2,'0')}.json`),await (await slide.export({format:'layout'})).text());
  if ((i+1)%10===0) console.log(`Rendered ${i+1}/61`);
}
if (sourceHash !== hash(await fs.readFile(source))) throw new Error('Source changed during edit');
console.log(JSON.stringify({final,manifestPath,slides:61,sourceUnchanged:true}));
