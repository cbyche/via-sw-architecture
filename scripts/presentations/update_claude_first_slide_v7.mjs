import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {FileBlob, PresentationFile} from '@oai/artifact-tool';

// Run a copy in a private build linked to the bundled node_modules.
const [repoArg, buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: update_claude_first_slide_v7.mjs REPO BUILD');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python, process.env.RUNTIME_NODE_MODULES].every(v => path.isAbsolute(v ?? '')))
  throw new Error('Set bundled runtime paths');
const source = path.join(repo, 'docs/presentations_files/VIA-architecture-presentation_v6.pptx');
const gif = path.join(repo, 'docs/presentations_files/full-presentation/assets/claude-cowork-browser-20260826.gif');
const sourceHash = '78eabff07ee0ba6c98e8abacf3e50bb055f1d526bf47b705a48c8d7f8ede2bf8';
const hash = data => createHash('sha256').update(data).digest('hex');
if (hash(await fs.readFile(source)) !== sourceHash) throw new Error('v6 source changed');
const edits = [
  ['r3-20', '자료를 찾아 업무 앱까지 조작하는 AI', '웹사이트를 찾아 정보를 확인하는 AI'],
  ['r3-21', '화면을 읽고, 필요한 정보를 찾아 신청서에 입력', '요청을 받으면 브라우저를 열어 요금 정보를 확인'],
  ['r3-22', '거래처 검색 → 정보 확인 → 신청서 입력', '요금 조사 요청 → 웹사이트 열기 → 요금제 확인'],
  ['r3-23', '[ Anthropic computer use 공개 시연, 2024.10 / 발췌, 3배속 ]',
    '[ Claude Cowork 공개 시연, 2026.08.26 / 발췌 ]'],
];
const p = await PresentationFile.importPptx(await FileBlob.load(source));
const snapshot = await p.inspect({kind:'slide,textbox,shape,image', maxChars:4000000});
await fs.writeFile(path.join(build,'inspection.ndjson'), snapshot.ndjson);
const records = snapshot.ndjson.split('\n').filter(Boolean).map(JSON.parse);
for (const [name, old, text] of edits) {
  const record = records.find(r => r.slide === 1 && r.name === name && r.id?.startsWith('sh/'));
  if (!record) throw new Error(`Missing first-slide shape: ${name}`);
  p.resolve(record.id).text.replace(old, text);
}
const imageRecord = records.filter(r => r.slide === 1 && r.kind === 'image')
  .sort((a,b) => a.bbox[0]-b.bbox[0])[0];
if (!imageRecord || imageRecord.bbox[0] > 300) throw new Error('Missing left demo image');
const image = p.resolve(imageRecord.id), oldFrame = image.frame, oldCrop = image.crop;
image.replace({blob:new Uint8Array(await fs.readFile(gif)), contentType:'image/gif',
  alt:'Claude Cowork 내장 브라우저가 요청을 받고 Northwind 웹사이트에서 요금제를 확인하는 공식 시연'});
image.frame = oldFrame;
image.crop = oldCrop;
const authoredPath = path.join(build,'authored.pptx');
await (await PresentationFile.exportPptx(p)).save(authoredPath);

// Preserve every source package part except the first slide, its image relation,
// and its demo-source notes. Add a distinct GIF so appendix page 36 stays intact.
const require = createRequire(import.meta.url), JSZip = require('jszip');
const original = await JSZip.loadAsync(await fs.readFile(source));
const authored = await JSZip.loadAsync(await fs.readFile(authoredPath));
const slidePart = 'ppt/slides/slide33.xml', relPart = 'ppt/slides/_rels/slide33.xml.rels';
const notesPart = 'ppt/notesSlides/notesSlide33.xml';
const newMedia = 'ppt/media/v7_claude_cowork_browser_20260826.gif';
const escapeXML = s => s.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;');
const authoredFirstPart = execFileSync(python,['-c',
  'import sys; from pathlib import Path; sys.path.insert(0,sys.argv[1]+"/scripts/presentations"); import full_presentation_package as F; print(F.ordered(F.read(Path(sys.argv[2])))[0][1])',
  repo,authoredPath],{encoding:'utf8'}).trim();
const authoredXml = await authored.file(authoredFirstPart).async('string');
let xml = await original.file(slidePart).async('string');
for (const [name, old, text] of edits) {
  if (!authoredXml.includes(escapeXML(text))) throw new Error(`Authoring export lost ${name}`);
  let count = 0;
  xml = xml.replace(/<p:sp>[\s\S]*?<\/p:sp>/g, block => {
    if (!block.includes(`name="${name}"`)) return block;
    count++;
    if (!block.includes(`<a:t>${escapeXML(old)}</a:t>`)) throw new Error(`Source text changed: ${name}`);
    return block.replace(`<a:t>${escapeXML(old)}</a:t>`, `<a:t>${escapeXML(text)}</a:t>`);
  });
  if (count !== 1) throw new Error(`Shape count ${name}: ${count}`);
}
xml = xml.replace(/<p:pic>[\s\S]*?<\/p:pic>/g, block => block.includes('R413be209bf6b450f')
  ? block.replace('<p:cNvPr id="22" name=""', '<p:cNvPr id="22" name="" descr="Claude Cowork 내장 브라우저 공개 시연, 2026-08-26"') : block);
original.file(slidePart, xml);
const rel = await original.file(relPart).async('string');
if (!rel.includes('Target="/ppt/media/initial01_image.gif"')) throw new Error('Old image relationship changed');
original.file(relPart, rel.replace('Target="/ppt/media/initial01_image.gif"', `Target="/${newMedia}"`));
let notes = await original.file(notesPart).async('string');
const demoNote = 'Claude Cowork 내장 브라우저 공식 발표, 2026-08-26: https://claude.com/resources/articles/cowork-built-in-browser ; 공식 시연: https://www.youtube.com/watch?v=63GVebZvqok . 시연에서 요금 조사 요청을 받고 Northwind 웹사이트를 열어 요금제를 확인하는 장면을 발췌했다. 공식 기사에 삽입된 영상의 브라우저 화면을 캡처하고 무음 반복 GIF로 만들었다. 약 6.5–19.2초 구간이며 원래 재생 시점을 유지한다. 첫 요금제 화면 1초와 마지막 화면 1.8초를 추가해 약 15.4초 동안 반복한다. 이 사례의 범위는 Cowork 내장 브라우저의 웹 작업이다. 모든 데스크톱 앱 조작이나 VIA의 구현 또는 성능을 입증하는 사례로 제시하지 않는다. 첫 장의 이전 2024 시연과 출처는 appendix 36장과 v6에 보존한다.';
let noteCount = 0;
notes = notes.replace(/<a:t>Anthropic original announcement:[\s\S]*?<\/a:t>/g, () => {
  noteCount++; return `<a:t>${escapeXML(demoNote)}</a:t>`;
});
if (noteCount !== 1) throw new Error('Old demo source note missing');
original.file(notesPart, notes);
original.file(newMedia, await fs.readFile(gif));
const contentTypes = await original.file('[Content_Types].xml').async('string');
if (!contentTypes.includes('</Types>')) throw new Error('Unexpected content-type namespace');
original.file('[Content_Types].xml', contentTypes.replace('</Types>',
  `<Override PartName="/${newMedia}" ContentType="image/gif" /></Types>`));
const candidate = path.join(build,'candidate.pptx');
await fs.writeFile(candidate, await original.generateAsync({type:'nodebuffer',compression:'DEFLATE'}));
const previous = JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/full-presentation/manifest-v6.json'),'utf8'));
const owners = previous.slides.filter(r => r.native_tables > 0).map(r => r.page);
const final = path.join(build,'final','VIA-architecture-presentation_v7.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation} = await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:61,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',
    ...owners.flatMap(n => ['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'user_request',families:['Calibri']},
  verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
execFileSync(python,[path.join(repo,'scripts/presentations/presentation_windows_policy.py'),'check',final],{stdio:'inherit'});

const result = await JSZip.loadAsync(await fs.readFile(final));
const allowed = new Set([slidePart, relPart, notesPart, '[Content_Types].xml']);
if (hash(await result.file(newMedia).async('nodebuffer')) !== hash(await fs.readFile(gif)))
  throw new Error('Finalization changed the animated GIF');
for (const [name, file] of Object.entries(original.files)) {
  if (file.dir || name === newMedia || allowed.has(name)) continue;
  if (!result.file(name) || hash(await result.file(name).async('nodebuffer')) !== hash(await file.async('nodebuffer')))
    throw new Error(`Finalization changed a preserved package part: ${name}`);
}
const sourceZip = await JSZip.loadAsync(await fs.readFile(source));
const changedParts = [];
for (const [name, file] of Object.entries(sourceZip.files)) {
  if (file.dir) continue;
  if (hash(await file.async('nodebuffer')) !== hash(await result.file(name).async('nodebuffer'))) changedParts.push(name);
}
if (changedParts.length !== 4 || changedParts.some(n => !allowed.has(n))) throw new Error('Edit escaped first slide');
const demo = JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/full-presentation/assets/claude-cowork-browser-20260826.json'),'utf8'));
const slides = await Promise.all(previous.slides.map(async r => ({...r,
  previous_slide_sha256:r.slide_sha256, slide_sha256:hash(await result.file(r.part).async('nodebuffer')),
  ...(r.page === 1 ? {kind:'claude-cowork-browser-demo-v7'} : {})})));
const manifest = {version:7,published_date:'2026-10-12',source_path:path.relative(repo,source),source_sha256:sourceHash,
  output_path:'docs/presentations_files/VIA-architecture-presentation_v7.pptx',output_sha256:hash(await fs.readFile(final)),
  main_slides:24,appendix_divider_page:25,preserved_v2_background_pages:[58,59,60,61],
  slide_order_ids_and_keys_preserved:true,changed_pages:[1],changed_existing_parts:changedParts,added_media_part:newMedia,
  all_other_package_parts_byte_identical:true,all_original_media_byte_identical:true,
  fixed_qa_tables_text_preserved:true,notes_changed_pages:[1],reference_path:previous.reference_path,
  authoring_policy:previous.authoring_policy,demo_source:demo,slides};
await fs.writeFile(path.join(build,'manifest-v7.json'),JSON.stringify(manifest,null,2)+'\n');
const rendered = await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for (const [i,slide] of rendered.slides.items.entries()) {
  const png = await rendered.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
  if ((i+1)%10 === 0) console.log(`Rendered ${i+1}/61`);
}
if (hash(await fs.readFile(source)) !== sourceHash) throw new Error('Source changed during edit');
console.log(JSON.stringify({final,manifest:path.join(build,'manifest-v7.json'),changedParts,addedMedia:newMedia,slides:61}));
