import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { FileBlob, PresentationFile } from '@oai/artifact-tool';

// Run a copy in a private build directory linked to the bundled node_modules.
const [repoArg, buildArg, attempt = '01'] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: redesign_project_necessity.mjs REPO BUILD_DIR [ATTEMPT]');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill ?? '') || !path.isAbsolute(python ?? '')) throw new Error('Set skill and Python paths.');
const source = path.join(repo, 'docs/presentations_files/VIA_과제배경_필요성_검토반영_v5.pptx');
const filename = 'VIA_과제배경_필요성_검토반영_v6.pptx';
const presentation = await PresentationFile.importPptx(await FileBlob.load(source));
const inspected = await presentation.inspect({ kind: 'slide,shape,textbox,layout', maxChars: 100000 });
await fs.writeFile(path.join(build, 'source-inspection.ndjson'), inspected.ndjson);
const records = inspected.ndjson.split('\n').filter(Boolean).map(JSON.parse);
const slideRecord = records.find(r => r.kind === 'slide' && r.slide === 3);
if (!slideRecord) throw new Error('Missing source necessity slide');
const slide = presentation.resolve(slideRecord.id);

const C = { navy: '#12264A', ink: '#20262C', blue: '#258FAF', pale: '#D8EAF1', gray: '#697681', light: '#E8EBEE', rule: '#BEC8D0' };
const names = new Set();
function shape(name, geometry, x, y, w, h, fill = 'none', lineFill = 'none', lineWidth = 0) {
  const fullName = `necessity-v6-${name}`;
  if (names.has(fullName)) throw new Error(`Duplicate name ${name}`);
  names.add(fullName);
  return slide.shapes.add({ name: fullName, geometry, position: { left: x, top: y, width: w, height: h }, fill,
    line: { style: 'solid', fill: lineFill, width: lineWidth } });
}
function text(name, value, x, y, w, h, size = 20, color = C.ink, bold = false, align = 'left') {
  if (/[\u00b7\u2022\u2027\u2219\u30fb]/u.test(value)) throw new Error('Middle dot in copy');
  const s = shape(name, 'textbox', x, y, w, h);
  s.text = value;
  s.text.style = { typeface: 'Apple SD Gothic Neo', fontSize: size, color, bold,
    alignment: align, verticalAlignment: 'top', autoFit: 'none', wrap: 'none',
    insets: { top: 0, bottom: 0, left: 0, right: 0 } };
  return s;
}
function rule(name, x, y, w, color = C.rule, thickness = 1) { return shape(name, 'line', x, y, w, 0, 'none', color, thickness); }

// Three editorial columns. Native diagrams describe common requirements,
// without selecting any of the DP alternatives or implying measured benefits.
shape('divider-1', 'line', 426, 108, 0, 451, 'none', C.rule, 1);
shape('divider-2', 'line', 853, 108, 0, 451, 'none', C.rule, 1);
const columns = [40, 456, 884];
const headings = ['사용자 의도에 맞는\n정확한 요청 처리', '빠른 반응과\n신속한 결과 전달', 'Agent 기술 발전을\n수용하는 제품 구조'];
const reasons = ['화면과 대화의 Context를 활용해\n요청 의미와 정정 내용을 정확히 반영',
  '새 발화와 Agent의 질문, 결과가\n겹쳐도 이어지는 상호작용',
  'VIA 내부의 Agent 기술과\nDownstream Agent의 변화를 함께 수용'];
for (let i = 0; i < 3; i++) {
  text(`heading-${i}`, headings[i], columns[i], 109, 355, 66, 28, C.navy, true);
  rule(`accent-${i}`, columns[i], 183, 54, C.blue, 3);
  text(`reason-${i}`, reasons[i], columns[i], 201, 355, 56, 20);
}

// Accuracy: a correction has a precise scope and leaves other conditions intact.
text('initial-turn', '“보고서는 PDF로, 메일은\n보고서 결론으로 초안만 만들어줘”', 40, 280, 355, 55, 20, C.ink);
text('correction-turn', '“아니, 메일에는 이 표를 넣어줘”', 40, 343, 355, 30, 20, C.blue, true);
rule('meaning-table-top', 40, 388, 355);
const meanings = [
  ['지칭 대상', '선택한 표'],
  ['대화/Task 연결', '기존 메일 Task'],
  ['정정 범위', '메일 내용만 교체'],
  ['유지할 조건', '보고서 PDF, 메일 초안'],
];
meanings.forEach(([label, value], i) => {
  text(`meaning-label-${i}`, label, 40, 398 + i * 26, 133, 25, 18, C.gray);
  text(`meaning-value-${i}`, value, 179, 398 + i * 26, 216, 25, 18, C.navy, i === 2 || i === 3);
});

// Speed: the gray segment is external-work-only waiting, excluded by V-05.
// The reaction marker is inside the first VIA segment, not at Agent completion.
text('input-event', '사용자 입력', 456, 280, 120, 27, 18, C.gray);
text('result-event', '결과 전달', 738, 280, 92, 27, 18, C.gray, false, 'right');
shape('via-before', 'rect', 456, 316, 96, 56, C.pale);
shape('external-wait', 'rect', 560, 316, 139, 56, C.light);
shape('via-after', 'rect', 707, 316, 123, 56, C.pale);
text('via-before-label', 'VIA', 456, 329, 96, 32, 22, C.blue, true, 'center');
text('external-label', 'Downstream\nAgent', 560, 322, 139, 48, 18, C.gray, false, 'center');
text('via-after-label', 'VIA', 707, 329, 123, 32, 22, C.blue, true, 'center');
shape('reaction-marker', 'line', 506, 372, 0, 22, 'none', C.blue, 2);
text('first-reaction', '첫 유효 반응', 456, 402, 170, 29, 20, C.blue, true);
rule('processing-before', 456, 447, 96, C.blue, 4);
rule('processing-after', 707, 447, 123, C.blue, 4);
text('processing-total', '결과 전달까지 VIA 처리시간 단축', 456, 455, 374, 29, 21, C.navy, true);
text('external-wait-excluded', '외부 작업만 기다리는 시간은 제외', 456, 484, 374, 22, 17, C.gray);

// Change: common evolution scope, not a selected Component topology.
text('internal-label', 'VIA 내부', 884, 280, 340, 30, 22, C.navy, true);
shape('internal-scope', 'rect', 884, 318, 350, 91, 'none', C.rule, 1);
text('model-area', '모델과 Runtime', 900, 334, 178, 27, 19, C.navy);
text('interpretation-area', '요청 이해', 1090, 334, 130, 27, 19, C.navy);
text('context-area', '기억과 Context', 900, 372, 178, 27, 19, C.navy);
text('interaction-area', '연속 상호작용', 1090, 372, 130, 27, 19, C.navy);
shape('delegation-link', 'line', 1059, 409, 0, 28, 'none', C.gray, 1.5);
text('downstream-area', 'Downstream Agent', 884, 445, 350, 35, 22, C.navy, true, 'center');

// Exact QA names link the rationale to the later quality-requirement slides.
rule('qa-line-0', 40, 507, 355);
rule('qa-line-1', 456, 507, 374);
rule('qa-line-2', 884, 507, 350);
text('qa-accuracy', 'V-01  기능 정확성', 40, 520, 355, 31, 24, C.blue, true);
text('qa-appropriateness-completeness', 'V-02 기능 적절성   V-03 기능 완전성', 40, 558, 355, 26, 17, C.gray);
text('qa-responsiveness', 'V-04  응답 신속성', 456, 520, 374, 31, 24, C.blue, true);
text('qa-completion', 'V-05  요청 완료 신속성', 456, 555, 374, 31, 24, C.blue, true);
text('qa-modifiability', 'V-08  변경 용이성', 884, 520, 350, 31, 24, C.blue, true);
text('localized-change', '변경 영향을 필요한 설계 요소로 제한', 884, 558, 350, 26, 18, C.gray);

rule('foundation-rule', 40, 608, 1194, C.rule, 1.2);
text('foundation-title', 'PC에서 지속적으로 사용하기 위한 공통 기반', 40, 620, 1194, 31, 21, C.navy, true);
const foundations = [
  ['V-06  메모리 효율성', '상시 실행 메모리 제한'],
  ['V-07  복구 용이성', '올바른 처리 재개'],
  ['V-09  분석 용이성', '실행 기록으로 경위 확인'],
  ['V-10  기밀성', '허용된 범위의 정보 사용'],
];
foundations.forEach(([qa, benefit], i) => {
  const x = 40 + i * 307;
  text(`foundation-qa-${i}`, qa, x, 661, 285, 25, 18, C.navy, true);
  text(`foundation-benefit-${i}`, benefit, x, 687, 285, 24, 17, C.gray);
});

const notes = [
  '이 페이지는 VIA 과제의 필요성을 기능 정확성, 신속성, 변경 용이성으로 설명한다.',
  '정확성 사례: 기존 보고서 PDF 조건과 메일 초안만 작성하는 조건을 유지하면서, 정정된 메일 내용의 지칭 대상만 선택한 표로 바꾼다. 실제 변경과 업무 수행은 Downstream Agent의 책임이다.',
  'V-01 기능 정확성이 첫 축의 중심이다. V-02 기능 적절성과 V-03 기능 완전성은 추가 사용자 개입과 필수 기능 지원 관점에서 함께 요구된다.',
  'V-04 응답 신속성은 실제 사용자 입력부터 첫 유효 반응까지다. V-05 요청 완료 신속성은 결과 전달까지의 VIA 처리시간이며 외부 작업만 기다리는 구간은 제외한다. 이 시간축은 개념도이며 길이는 실제 시간이나 측정값이 아니다. 첫 유효 반응은 첫 VIA 구간 안의 대표 위치에 표시했다.',
  '변경 용이성은 Downstream Agent 추가와 연동 변화뿐 아니라 VIA 내부의 모델, Runtime, 요청 이해, 기억과 Context, 연속 상호작용 기술의 발전을 수용하는 데 필요하다. 원천 기술의 학습과 내부 알고리즘은 VIA Architecture 설계 범위 밖이다.',
  'DP 연결: 04-41 요청 의미 확정은 조회와 의미 완성의 진행 책임, 04-43 요청 이해의 판단 책임은 의도, 지칭 대상, 대화/Task 연결, 요청 간 관계, 정정 범위와 처리 방향, 04-45 기억과 Context는 과거 근거의 유지와 조회 및 구성을 다룬다. 첫 축의 Context 활용과 의미 보존에 연결된다.',
  '04-42 서로 다른 대화와 업무 수명은 상태 소유와 실행 경계, 04-44 연속 상호작용은 겹치는 사용자 입력과 Agent 알림의 다음 처리를 조직하는 방식이다. 속도와 지속 사용의 구조적 비용을 검토한다.',
  'V-08 변경 용이성은 각 DP에서 실제 변경 영향을 비교하는 기준이며, 모든 DP가 같은 QA를 개선한다는 뜻은 아니다. V-06 메모리 효율성, V-07 복구 용이성, V-09 분석 용이성, V-10 기밀성은 공통 기반이다.',
  '근거: docs/presentations_files/quality-attributes.md; docs/architecture/03-fixed-architecture-scope.md; docs/architecture/05-representative-use-cases.md; docs/architecture/12-decisions/decision-packages/04-41-request-resolution-control.md; 04-42-lifecycle-ownership.md; 04-43-request-interpretation.md; 04-44-continuous-interaction.md; 04-45-memory-and-context.md',
].join('\n');
slide.speakerNotes.textFrame.setText(notes);
const authoredPath = path.join(build, `artifact-authored-${attempt}.pptx`);
await (await PresentationFile.exportPptx(presentation)).save(authoredPath);

// Preserve every original package part outside slide 3 and its notes. This keeps
// the other two pages, their GIF media, master/theme, relationships and fonts.
const require = createRequire(import.meta.url);
const JSZip = require('jszip');
const original = await JSZip.loadAsync(await fs.readFile(source));
const authored = await JSZip.loadAsync(await fs.readFile(authoredPath));
const slidePath = 'ppt/slides/slide3.xml';
let sourceXML = await original.file(slidePath).async('string');
const authoredXML = await authored.file(slidePath).async('string');
const authoredBlocks = [...authoredXML.matchAll(/<p:(?:sp|cxnSp|graphicFrame)\b[^>]*>[\s\S]*?<\/p:(?:sp|cxnSp|graphicFrame)>/g)]
  .map(m => m[0]).filter(block => [...names].some(name => block.includes(`name="${name}"`)));
if (authoredBlocks.length !== names.size) throw new Error(`Native objects lost: ${authoredBlocks.length}/${names.size}`);
const idMap = new Map(authoredBlocks.map((block, i) => [block.match(/<p:cNvPr\b[^>]*\bid="(\d+)"/)[1], String(1000 + i)]));
const newBlocks = authoredBlocks.map(block => block
  .replace(/(<p:cNvPr\b[^>]*\bid=")(\d+)(")/, (_, a, id, b) => a + idMap.get(id) + b)
  .replace(/(<a:(?:stCnx|endCnx)\b[^>]*\bid=")(\d+)(")/g, (_, a, id, b) => a + (idMap.get(id) ?? id) + b));
const keepNames = new Set(['r3-120', 'line-121', 'r3-132', ...Array.from({ length: 10 }, (_, i) => `r3-${122 + i}`)]);
sourceXML = sourceXML.replace(/<p:sp>[\s\S]*?<\/p:sp>/g, block => {
  const name = block.match(/<p:cNvPr\b[^>]*\bname="([^"]*)"/)?.[1];
  return keepNames.has(name) ? block : '';
}).replace('</p:spTree>', newBlocks.join('') + '</p:spTree>');
original.file(slidePath, sourceXML, { createFolders: false });

// Retain the original notes package and replace only its body placeholder text.
const notesPath = 'ppt/notesSlides/notesSlide3.xml';
let notesXML = await original.file(notesPath).async('string');
const xmlEscape = s => s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
let replacedNotes = false;
notesXML = notesXML.replace(/<p:sp>[\s\S]*?<\/p:sp>/g, block => {
  if (!/<p:ph\b[^>]*type="body"/.test(block)) return block;
  replacedNotes = true;
  const paras = notes.split('\n').map(line => `<a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:r><a:t>${xmlEscape(line)}</a:t></a:r></a:p>`).join('');
  const updated = block.replace(/(<p:txBody>[\s\S]*?<a:lstStyle\b[^>]*\/>)[\s\S]*?(<\/p:txBody>)/, `$1${paras}$2`);
  if (updated === block) throw new Error('Notes body replacement failed');
  return updated;
});
if (!replacedNotes) throw new Error('Missing notes body placeholder');
original.file(notesPath, notesXML, { createFolders: false });
const candidate = path.join(build, `candidate-${attempt}.pptx`);
await fs.writeFile(candidate, await original.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' }));
const final = path.join(build, `final-${attempt}`, filename);
await fs.mkdir(path.dirname(final), { recursive: true });
const { finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({ workspaceDir: build, candidatePath: candidate, finalPath: final,
  explicitTotalSlideCount: 3, pythonExecutable: python,
  integrityValidatorPath: path.join(skill, 'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath: path.join(skill, 'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs: ['--expected-slide-size-emu', '12192000,6858000', '--validate-heading-fit', '--validate-bullet-geometry'],
  fontPolicy: { basis: 'reference', families: ['Apple SD Gothic Neo'], referencePath: source,
    referenceSha256: createHash('sha256').update(await fs.readFile(source)).digest('hex') },
  verifyArtifactToolImport: true, receiptPath: path.join(build, `validation-${attempt}.json`),
});
// A fresh renderer process avoids imported-object preview cache reuse after
// authoring and package-preserving export in this process.
const renderDir = path.join(build, `render-${attempt}`);
execFileSync(process.execPath, [path.join(skill, 'container_tools/render_presentation.mjs'),
  '--input', final, '--output_dir', renderDir, '--scale', '1.25'], { stdio: 'pipe', env: process.env });
const checked = await PresentationFile.importPptx(await FileBlob.load(final));
await fs.writeFile(path.join(build, `final-inspection-${attempt}.ndjson`), (await checked.inspect({ kind: 'slide,shape,textbox,notes', maxChars: 100000 })).ndjson);
console.log(JSON.stringify({ final, slides: 3, editableObjectsOnRevisedSlide: names.size }));
