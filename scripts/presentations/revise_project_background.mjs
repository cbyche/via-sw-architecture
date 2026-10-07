import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { FileBlob, PresentationFile } from '@oai/artifact-tool';

// Run a copy in a private build directory linked to the bundled node_modules.
const [repoArg, buildArg, revision = 'v4'] = process.argv.slice(2);
if (!repoArg || !buildArg || !['v4', 'v5'].includes(revision)) throw new Error('Usage: revise_project_background.mjs REPO BUILD_DIR [v4|v5]');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill ?? '') || !path.isAbsolute(python ?? '')) throw new Error('Set skill and Python paths.');
const sourceRevision = revision === 'v5' ? 'v4' : 'v3';
const source = path.join(repo, `docs/presentations_files/VIA_과제배경_필요성_검토반영_${sourceRevision}.pptx`);
const filename = `VIA_과제배경_필요성_검토반영_${revision}.pptx`;
const edits = revision === 'v5' ? [
  [3, 'r3-154', '판단 오류가 실제 업무에 영향을 줌'],
  [3, 'r3-155', '취소 요청을 다른 업무에 적용하거나,\n일부 완료를 전체 완료로 안내할 수 있음'],
  [3, 'r3-157', '처리 오류와 재작업으로 이어질 위험'],
  [3, 'r3-184', '음성 대화에서는 짧은 대기도 반복'],
  [3, 'r3-185', '진행 질문이나 정정, 취소에도\nVIA가 판단하고 답을 준비하는 시간이 필요', { top: 565, height: 60 }],
  [3, 'r3-187', '대화가 이어질수록 사용자 대기가 누적'],
  [3, 'r3-198', '기술 교체가 전체 수정으로 번질 위험'],
  [3, 'r3-199', '음성 모델 하나를 바꾸는 데도\n업무 관리까지 수정하면 검증 범위가 커짐'],
  [3, 'r3-201', '전체 수정과 재검증에 드는 비용 증가'],
] : [
  [1, 'r3-23', '[ Anthropic computer use 공개 시연, 2024.10 / 발췌, 3배속 ]'],
  [1, 'r3-29', '파일과 앱의 자료로 보고서 작성\n자료를 비교해 표와 발표자료 생성'],
  [1, 'r3-37', '[ OpenAI GPT-6 Astra 공개 시연 / 발췌, 1.5배속 ]'],
  [1, 'r3-51', '화면과 대화의 맥락을 기기 경험과 연결하고,'],
  [1, 'r3-53', 'AI가 달라도 요청과 진행, 결과는 같은 대화로 제공'],
  [2, 'r3-68', 'VIA는 사용자의 요청과 AI의 업무 수행을 같은 대화와 업무 맥락으로 연결'],
  [2, 'r3-70', 'VIA: PC에서 대화와 업무 연결을 담당'],
  [2, 'r3-72', '연결된 AI: 맡은 업무를 수행'],
  [2, 'r3-73', '음성 또는 텍스트 + 당시 화면 + 앞선 대화와 진행 업무'],
  [2, 'r3-78', '목표와 조건, 정정 내용 파악\n새 업무인지 기존 업무인지 판단', { top: 299, height: 44 }],
  [2, 'r3-86', '질문과 추가 지시, 취소를 해당 업무에 연결\n진행 상황과 결과도 같은 업무에서 관리', { top: 446, height: 44 }],
  [2, 'r3-87', '목표와 조건'],
  [2, 'r3-94', '질문과 상태'],
  [2, 'r3-100', '자료 조사와 문서 작성'],
  [2, 'r3-101', '앱 조작, 코드 수정과 시험 등'],
  [2, 'r3-119', '본 과제 범위: 요청 판단과 상호작용, 업무 연결과 상태 관리'],
  [3, 'r3-143', '대상과 조건을 해석한 뒤에도,\n업무를 선택하고 처리 결과를 전달'],
  [3, 'r3-145', '목표와 대상, 조건 파악\n기존 업무와의 관계 확인', { top: 308, height: 42 }],
  [3, 'r3-148', '직접 답할지, AI에 맡길지 판단\n담당 AI와 확인 질문 결정', { top: 386, height: 42 }],
  [3, 'r3-150', '상태와 결과 전달', { top: 436, height: 26 }],
  [3, 'r3-151', '진행 상태와 막힌 지점\n부분 완료 여부와 결과 버전', { top: 464, height: 42 }],
  [3, 'r3-157', '요청부터 처리와 전달까지 의미 보존'],
  [3, 'r3-175', '해석과 위임'],
  [3, 'r3-177', '결과를\n음성으로 전달', { top: 373, height: 40 }],
  [3, 'r3-185', '근거 조회와 추론, 검증, 음성 출력의 대기 축소\n직접 답변과 진행 안내,\n정정과 취소에도 적용', { top: 562, height: 72 }],
  [3, 'r3-190', '음성과 요청 해석 기술의 발전과\n새 업무 AI의 등장을 함께 수용해야 함'],
  [3, 'r3-192', '음성과 요청 해석 모델 교체\n문맥 정보와 상태 구조 변경\n실험과 로그 추가'],
  [3, 'r3-195', '업무 AI 추가 또는 교체\n연동 규약 변경', { height: 44 }],
  [3, 'r3-199', '하나의 모델이나 연동 방식을 바꿀 때\n대화와 업무 관리 전체의 수정을 줄여야 함'],
];
const positions = revision === 'v5' ? [] : [
  [2, 'r3-77', { top: 269 }], [2, 'r3-75', { top: 273 }], [2, 'r3-76', { top: 273 }],
  [2, 'r3-81', { top: 347 }], [2, 'r3-82', { top: 379 }],
  [2, 'r3-79', { top: 351 }], [2, 'r3-80', { top: 351 }],
  [2, 'r3-85', { top: 414 }], [2, 'r3-83', { top: 418 }], [2, 'r3-84', { top: 418 }],
  [3, 'r3-144', { top: 280, height: 26 }], [3, 'line-146', { top: 354 }],
  [3, 'r3-147', { top: 358, height: 26 }], [3, 'line-149', { top: 432 }],
];
const presentation = await PresentationFile.importPptx(await FileBlob.load(source));
const snapshot = await presentation.inspect({ kind: 'textbox,shape,layout', maxChars: 100000 });
await fs.writeFile(path.join(build, 'source-inspection.ndjson'), snapshot.ndjson);
const records = snapshot.ndjson.split('\n').filter(Boolean).map(line => JSON.parse(line));
function resolve(slide, name) {
  const record = records.find(r => r.slide === slide && r.name === name && r.id?.startsWith('sh/'));
  if (!record) throw new Error(`Source shape missing: ${slide}/${name}`);
  return presentation.resolve(record.id);
}
for (const [slide, name, text, position] of edits) {
  const shape = resolve(slide, name);
  shape.text = text;
  if (position) shape.position = { ...shape.position, ...position };
}
for (const [slide, name, position] of positions) {
  const shape = resolve(slide, name);
  shape.position = { ...shape.position, ...position };
}
const roundtrip = path.join(build, 'artifact-edited.pptx');
await (await PresentationFile.exportPptx(presentation)).save(roundtrip);

// Keep the original package, including animated GIF bytes and Office metadata.
// Apply the Artifact Tool-authored paragraphs and positions to original objects;
// retain the reference's paragraph/run formatting instead of normalizing fonts.
const require = createRequire(import.meta.url);
const JSZip = require('jszip');
const original = await JSZip.loadAsync(await fs.readFile(source));
const authored = await JSZip.loadAsync(await fs.readFile(roundtrip));
const escapeXML = s => s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
for (let slide = 1; slide <= 3; slide++) {
  const file = `ppt/slides/slide${slide}.xml`;
  let xml = await original.file(file).async('string');
  const authoredXML = await authored.file(file).async('string');
  for (const [owner, name, text, position] of edits.filter(e => e[0] === slide)) {
    // Confirm every requested paragraph survived the authoring export.
    for (const line of text.split('\n')) if (!authoredXML.includes(escapeXML(line))) throw new Error(`Export lost text: ${name}`);
    xml = updateShape(xml, name, block => {
      const body = block.match(/<p:txBody>[\s\S]*?<\/p:txBody>/)?.[0];
      if (!body) throw new Error(`No text body: ${name}`);
      const oldParagraphs = [...body.matchAll(/<a:p\b[^>]*>[\s\S]*?<\/a:p>/g)].map(m => m[0]);
      const newParagraphs = text.split('\n').map((line, index) => {
        const template = oldParagraphs[Math.min(index, oldParagraphs.length - 1)];
        const pPr = template.match(/<a:pPr[\s\S]*?<\/a:pPr>/)?.[0] ?? '';
        const rPr = template.match(/<a:rPr[\s\S]*?<\/a:rPr>/)?.[0] ?? '';
        const opening = template.match(/<a:p\b[^>]*>/)[0];
        return `${opening}${pPr}<a:r>${rPr}<a:t>${escapeXML(line)}</a:t></a:r></a:p>`;
      }).join('');
      const nextBody = body.replace(/<a:p\b[^>]*>[\s\S]*<\/a:p>/, newParagraphs);
      block = block.replace(body, nextBody);
      return position ? updatePosition(block, position) : block;
    });
  }
  for (const [owner, name, position] of positions.filter(e => e[0] === slide)) {
    xml = updateShape(xml, name, block => updatePosition(block, position));
  }
  if (/[\u00b7\u2022\u2027\u2219\u30fb]/u.test(xml)) throw new Error(`Middle dot remains on slide ${slide}`);
  original.file(file, xml);
}
function updateShape(xml, name, update) {
  let found = false;
  const result = xml.replace(/<p:sp>[\s\S]*?<\/p:sp>/g, block => {
    if (!block.includes(`name="${name}"`)) return block;
    if (found) throw new Error(`Duplicate shape: ${name}`);
    found = true;
    return update(block);
  });
  if (!found) throw new Error(`Original shape missing: ${name}`);
  return result;
}
function updatePosition(block, values) {
  return block.replace(/<a:xfrm\b[^>]*>[\s\S]*?<\/a:xfrm>/, xfrm => {
    for (const [key, attribute] of [['left', 'x'], ['top', 'y'], ['width', 'cx'], ['height', 'cy']]) {
      if (values[key] !== undefined) xfrm = xfrm.replace(new RegExp(`\\b${attribute}="[^"]*"`), `${attribute}="${Math.round(values[key] * 9525)}"`);
    }
    return xfrm;
  });
}
const candidate = path.join(build, 'candidate.pptx');
await fs.writeFile(candidate, await original.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' }));
const final = path.join(build, 'final', filename);
await fs.mkdir(path.dirname(final), { recursive: true });
const { finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({ workspaceDir: build, candidatePath: candidate, finalPath: final,
  explicitTotalSlideCount: 3, pythonExecutable: python,
  integrityValidatorPath: path.join(skill, 'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath: path.join(skill, 'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs: ['--expected-slide-size-emu', '12192000,6858000', '--validate-heading-fit', '--validate-bullet-geometry'],
  fontPolicy: { basis: 'reference', families: ['Apple SD Gothic Neo'], referencePath: source,
    referenceSha256: createHash('sha256').update(await fs.readFile(source)).digest('hex') },
  verifyArtifactToolImport: true, receiptPath: path.join(build, 'validation.json'),
});
await fs.copyFile(final, path.join(repo, 'docs/presentations_files', filename));
const checked = await PresentationFile.importPptx(await FileBlob.load(final));
for (let i = 0; i < 3; i++) {
  const png = await checked.export({ slide: checked.slides.items[i], format: 'png', scale: 1.25 });
  await fs.writeFile(path.join(build, `slide-${i + 1}.png`), new Uint8Array(await png.arrayBuffer()));
}
console.log(JSON.stringify({ final, slides: 3, revisedTextShapes: edits.length, middleDots: 0 }));
