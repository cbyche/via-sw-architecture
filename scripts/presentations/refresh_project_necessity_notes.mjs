import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {FileBlob,PresentationFile} from '@oai/artifact-tool';

export const CURRENT_ROLES_NOTES = [
  'VIA는 지속적인 Voice/Text/screen 상호작용과 요청 및 업무 연결을 담당하고, Downstream Agent는 도메인 추론, 계획, 도구 선택과 실제 업무 실행을 담당한다. 이 페이지의 책임 영역은 개념도이며 Component 배치나 대안 선정 결과가 아니다.',
  '현재 발표의 모델 배치는 docs/architecture/12-decisions/decision-packages/04-40-common-execution-contract.md를 따른다. VIA와 VAD는 사용자 PC에서 실행하고 음성 모델과 의미 LLM은 별도 클라우드 의존성이며 Model Access로 연동한다. Request, Task 및 실제 전달의 권위는 로컬 VIA에 남는다. 공급자/checkpoint는 품질 문서의 PoC reference이며 구현 완료나 후보 우위를 뜻하지 않는다.',
  '기능 근거: docs/architecture/01-system-mission-and-boundary.md; 03-fixed-architecture-scope.md; 05-representative-use-cases.md. 현재 비교는 04-41, 04-44, 04-45, 04-42이며 최종 A/B 선택과 실측은 아직 없다.',
  '화면의 원래 책임 구분은 source commit a2acb638001f15b920c5ab1903053ea3c2892711의 target-architecture/architecture.md, control-and-lifecycle.md, memory-and-context-lifecycle.md, shared-omni-runtime.md 및 design-completeness.md에서 유래했다. 해당 참조 Architecture와 당시 공유 Omni 배치는 보존된 검토 이력이다. 현재 발표의 모델 제약으로 사용하지 않는다.',
].join('\n');

async function main() {
  const [repoArg,buildArg]=process.argv.slice(2);
  if(!repoArg||!buildArg)throw new Error('Usage: refresh_project_necessity_notes.mjs REPO BUILD');
  const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
  const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
  if(![skill,python].every(v=>path.isAbsolute(v??'')))throw new Error('Set bundled skill and Python paths');
  const relative='docs/presentations_files/VIA_과제배경_필요성_검토반영_v7.pptx';
  const source=path.join(repo,relative),filename=path.basename(source);
  const sourceBytes=await fs.readFile(source),sourceSha=createHash('sha256').update(sourceBytes).digest('hex');
  const JSZip=createRequire(import.meta.url)('jszip');
  const original=await JSZip.loadAsync(sourceBytes);
  const notesXML=await original.file('ppt/notesSlides/notesSlide3.xml').async('string');
  const noteBody=[...notesXML.matchAll(/<p:sp\b[^>]*>[\s\S]*?<\/p:sp>/g)].find(m=>/type="body"/.test(m[0]))?.[0];
  const decode=s=>s.replaceAll('&lt;','<').replaceAll('&gt;','>').replaceAll('&quot;','"').replaceAll('&apos;',"'").replaceAll('&amp;','&');
  const old=[...(noteBody??'').matchAll(/<a:p\b[^>]*>[\s\S]*?<\/a:p>/g)]
    .map(m=>[...m[0].matchAll(/<a:t\b[^>]*>([\s\S]*?)<\/a:t>/g)].map(t=>decode(t[1])).join('')).join('\n');
  const p=await PresentationFile.importPptx(await FileBlob.load(source));
  p.slides.items[1].speakerNotes.textFrame.setText(CURRENT_ROLES_NOTES);
  if(!old.includes('문안 근거:'))throw new Error('Missing source notes');
  const updated=old.replace('04-43-request-interpretation.md; ','')
    .replace('decision-packages/04-41-request-resolution-control.md;',
      'decision-packages/04-40-common-execution-contract.md; 04-41-request-resolution-control.md;');
  p.slides.items[2].speakerNotes.textFrame.setText(updated);
  const authored=path.join(build,'notes-authored.pptx');
  await(await PresentationFile.exportPptx(p)).save(authored);
  const generated=await JSZip.loadAsync(await fs.readFile(authored));
  const changed=[];
  for(const n of [2,3]){
    const key=`ppt/notesSlides/notesSlide${n}.xml`;
    const before=await original.file(key).async('string'),next=await generated.file(key).async('string');
    const body=s=>[...s.matchAll(/<p:sp\b[^>]*>[\s\S]*?<\/p:sp>/g)].find(m=>/type="body"/.test(m[0]));
    const a=body(before),b=body(next);
    if(!a||!b)throw new Error('Missing notes body');
    const tx=b[0].match(/<p:txBody>[\s\S]*?<\/p:txBody>/)?.[0];
    if(!tx)throw new Error('Missing generated notes text');
    const replacement=a[0].replace(/<p:txBody>[\s\S]*?<\/p:txBody>/,()=>tx);
    original.file(key,before.slice(0,a.index)+replacement+before.slice(a.index+a[0].length),{createFolders:false});
    changed.push(key);
  }
  const candidate=path.join(build,'candidate.pptx'),final=path.join(build,'final',filename);
  await fs.writeFile(candidate,await original.generateAsync({type:'nodebuffer',compression:'DEFLATE'}));
  await fs.mkdir(path.dirname(final),{recursive:true});
  const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
  await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:3,
    requiredNativeTableOwnerSlides:[3],pythonExecutable:python,
    integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
    layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
    layoutArgs:['--expected-slide-size-emu','12192000,6858000','--validate-heading-fit','--validate-bullet-geometry','--require-native-table-slide','3'],
    fontPolicy:{basis:'reference',families:['Apple SD Gothic Neo'],referencePath:source,referenceSha256:sourceSha},
    verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
  const loaded=await PresentationFile.importPptx(await FileBlob.load(final));
  await fs.mkdir(path.join(build,'renders'),{recursive:true});
  for(const [i,slide]of loaded.slides.items.entries()){
    const png=await loaded.export({slide,format:'png',scale:1});
    await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
  }
  await fs.copyFile(final,source);
  await fs.writeFile(path.join(build,'preservation.json'),JSON.stringify({source:relative,source_sha256:sourceSha,
    changed_parts:changed,all_slide_and_media_parts_preserved:true},null,2)+'\n');
  console.log('PASS: project introduction notes updated; all three slide XMLs and media preserved');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(path.resolve(process.argv[1])).href)await main();
