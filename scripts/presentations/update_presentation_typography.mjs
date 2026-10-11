import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {PresentationFile, FileBlob} from '@oai/artifact-tool';

// Author focused text edits with Artifact Tool, then preserve the source package
// while normalizing native font sizes in points, not CSS pixels.
const [repoArg, buildArg] = process.argv.slice(2);
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(v => path.isAbsolute(v ?? ''))) throw new Error('Set bundled runtime paths');
const source = path.join(repo,'docs/presentations_files/VIA-architecture-presentation_v3.pptx');
const previous = JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/full-presentation/manifest-v3.json'),'utf8'));
const sourceHash = createHash('sha256').update(await fs.readFile(source)).digest('hex');
if (sourceHash !== previous.output_sha256) throw new Error('Source changed; rebase this edit');
const p = await PresentationFile.importPptx(await FileBlob.load(source));
const questions = [
  {dp:41,page:15,appendix:58,old:'다음 조회와 전체 의미를 모델이 결정할까, 코드가 항목 해결 상태로 진행할까?',
    oldAppendix:'설계 질문  다음 조회와 전체 의미 완성을 모델이 결정할까, 코드가 항목 해결 상태로 진행할까?',
    text:'조회와 정정이 반복되어도, 요청의 대상과 순서 및 조건을 어떻게 정확히 확정할까?',
    oldCost:'모델의 유연한 재판단과 반복 비용 / 코드의 추적 가능성, 유한 표현의 지원 범위와 추가 해석 비용',
    context:'해석이 바뀌어도 이미 정한 조건을 보존하고, 추가 조회와 재해석에 드는 시간과 비용을 관리해야 합니다.',
    tableOld:'41  모델이 다음 조회를 정할까, 코드가 요청 항목을 해결할까?',
    tableText:'41  요청의 대상과 순서, 조건을 어떻게 정확히 확정할까?'},
  {dp:44,page:17,appendix:59,old:'완료와 대기를 중앙이 연결할까, 단계별 조건으로 후속 처리를 이어갈까?',
    oldAppendix:'설계 질문  완료마다 중앙이 다음 실행을 지시할까, 단계가 입력과 게시 조건으로 실행될까?',
    text:'새 발화와 업무 사건이 겹쳐도, 유효한 응답을 적절한 시점에 어떻게 전달할까?',
    oldCost:'양안 모두 비동기 실행과 지속 입력 가능. 중앙 회수/지시 비용과 단계별 Window/Join 상태 비용을 비교',
    context:'입력을 계속 받고, 늦은 응답의 현재성을 확인하며 실제 전달 뒤에도 필요한 처리를 이어가야 합니다.',
    tableOld:'44  중앙이 다음 실행을 지시할까, 단계 조건이 실행을 이끌까?',
    tableText:'44  겹치는 사건 속에서 유효한 응답을 언제 전달할까?'},
  {dp:45,page:19,appendix:60,old:'요청마다 원본 읽기를 조합할까, 공통 과거 관계를 생산하고 유지해 조회할까?',
    oldAppendix:'설계 질문  필요할 때 과거 원본을 조합할까, 여러 요청이 읽을 관계를 공통 생산할까?',
    text:'흩어진 과거 근거를 어떻게 연결하고, 최신 상태와 권한에 맞게 다시 사용할까?',
    oldCost:'A도 유효 cache와 부분 갱신 가능. B는 초기/미사용 생산, 삭제/철회와 게시 범위 관리 비용을 부담',
    context:'원본의 수정이나 삭제를 반영하면서, 과거의 실제 전달 범위와 사용 가능한 근거를 구분해야 합니다.',
    tableOld:'45  요청 때 조합할까, 재사용할 관계를 공통 생산할까?',
    tableText:'45  흩어진 과거 근거를 어떻게 연결하고 재사용할까?'},
  {dp:42,page:21,appendix:61,old:'관련 대화/업무 상태를 함께 확정할까, 독립 상태의 접수 결과로 맞출까?',
    oldAppendix:'설계 질문  대화와 업무의 관련 상태를 함께 확정할까, 독립 상태 권위의 접수로 맞출까?',
    text:'대화와 업무의 수명이 달라도, 요청과 실행의 연결 및 상태를 어떻게 일관되게 유지할까?',
    oldCost:'A도 논리 수명 분리와 모듈화 가능. B의 독립 운영 조건과 인계/재확인, 추가 process와 상태 비용을 비교',
    context:'음성 연결과 개별 요청이 끝나도 같은 업무를 이어가며, 접수 결과와 변경 이력을 맞춰야 합니다.',
    tableOld:'42  관련 상태를 함께 확정할까, 독립 접수로 맞출까?',
    tableText:'42  수명이 다른 대화와 업무 상태를 어떻게 일관되게 유지할까?'}
];
const replacements = [];
async function editShape(page, oldText, newText) {
  const snapshot = await p.inspect({kind:'textbox',search:oldText,maxChars:30000});
  const rows = snapshot.ndjson.split('\n').filter(Boolean).map(JSON.parse);
  const matches = rows.filter(r => r.slide===page && r.text===oldText);
  if (matches.length!==1) throw new Error(`Expected one exact textbox on ${page}: ${oldText}`);
  p.resolve(matches[0].id).text.replace(oldText,newText);
  replacements.push({page,old:oldText,text:newText});
}
for (const q of questions) {
  await editShape(q.page,q.old,q.text);
  await editShape(q.page,q.oldCost,q.context);
  await editShape(q.appendix,q.oldAppendix,'설계 질문  '+q.text);
}
const tables = (await p.inspect({kind:'table',maxChars:200000})).ndjson.split('\n').filter(Boolean).map(JSON.parse);
const tableRows = tables.filter(r => r.kind==='table' && r.slide===12);
if (tableRows.length!==1) throw new Error('Expected the design-driver table on page 12');
const table = p.resolve(tableRows[0].id);
table.cells.set(0,2,'설계 질문');
replacements.push({page:12,old:'구조 선택 질문',text:'설계 질문'});
for (const [index,q] of questions.entries()) {
  table.cells.set(index+1,2,q.tableText);
  replacements.push({page:12,old:q.tableOld,text:q.tableText});
}
const authored = path.join(build,'authored.pptx');
await (await PresentationFile.exportPptx(p)).save(authored);
const planPath=path.join(build,'typography-plan.json');
await fs.writeFile(planPath,JSON.stringify({source_sha256:sourceHash,replacements,
  notes_append:questions.map(q=>({page:q.page,text:'구조 비교의 비용과 조건: '+q.oldCost}))},null,2)+'\n');
const candidate=path.join(build,'candidate.pptx'),manifestPath=path.join(build,'manifest-v4.json');
execFileSync(python,[path.join(repo,'scripts/presentations/package_typography_revision.py'),source,authored,
  path.join(repo,'docs/presentations_files/full-presentation/manifest-v3.json'),planPath,candidate,manifestPath],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
const owners=manifest.slides.filter(r=>r.native_tables>0).map(r=>r.page);
const final=path.join(build,'final','VIA-architecture-presentation_v4.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:61,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'reference',families:['Apple SD Gothic Neo','Arial Unicode MS'],referencePath:source,referenceSha256:sourceHash},
  verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
manifest.output_sha256=createHash('sha256').update(await fs.readFile(final)).digest('hex');
await fs.writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
const rendered=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.mkdir(path.join(build,'renders'),{recursive:true});
await fs.mkdir(path.join(build,'layouts'),{recursive:true});
for (const [i,slide] of rendered.slides.items.entries()) {
  const png=await rendered.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
  await fs.writeFile(path.join(build,'layouts',`${String(i+1).padStart(2,'0')}.json`),await (await slide.export({format:'layout'})).text());
}
if(sourceHash!==createHash('sha256').update(await fs.readFile(source)).digest('hex'))throw new Error('Source changed during edit');
console.log(JSON.stringify({final,manifestPath,slides:61,sourceUnchanged:true}));
