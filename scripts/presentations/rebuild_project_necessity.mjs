import fs from 'node:fs/promises';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { FileBlob, PresentationFile } from '@oai/artifact-tool';

// Run a copy in a private build directory with the bundled node_modules link.
const [repoArg, buildArg, attempt = '01'] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: rebuild_project_necessity.mjs REPO BUILD_DIR [ATTEMPT]');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill ?? '') || !path.isAbsolute(python ?? '')) throw new Error('Set skill and Python paths.');
const source = path.join(repo, 'docs/presentations_files/VIA_과제배경_필요성_검토반영_v6.pptx');
const filename = 'VIA_과제배경_필요성_검토반영_v7.pptx';
const { makeNativeBulletParagraphs, finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
const presentation = await PresentationFile.importPptx(await FileBlob.load(source));
const inspected = await presentation.inspect({ kind: 'slide,shape,textbox,table,layout', maxChars: 100000 });
await fs.writeFile(path.join(build, 'source-inspection.ndjson'), inspected.ndjson);
const records = inspected.ndjson.split('\n').filter(Boolean).map(JSON.parse);
const slideRecord = records.find(r => r.kind === 'slide' && r.slide === 3);
if (!slideRecord) throw new Error('Missing necessity page');
const slide = presentation.resolve(slideRecord.id);
const originalNames = new Set(records.filter(r => r.slide === 3 && r.name).map(r => r.name));
const C = { navy: '#06255C', ink: '#171B20', blue: '#218FAF', pale: '#E7F3F8', gray: '#697783', rule: '#D5DCE5', light: '#F2F4F6', green: '#CDE5BB', white: '#FFFFFF' };
const names = new Set();
const authoredTexts = [];
let connectionCount = 0;
function shape(name, geometry, x, y, w, h, fill = 'none', stroke = 'none', width = 0, radius = 0) {
  const fullName = `necessity-v7-${name}`;
  if (names.has(fullName)) throw new Error(`Duplicate name ${name}`);
  names.add(fullName);
  return slide.shapes.add({ name: fullName, geometry, position: { left: x, top: y, width: w, height: h }, fill,
    line: { style: 'solid', fill: stroke, width }, ...(radius ? { borderRadius: radius } : {}) });
}
function text(name, value, x, y, w, h, size = 18, color = C.ink, bold = false, align = 'left') {
  if (/[\u00b7\u2022\u2027\u2219\u30fb]/u.test(value)) throw new Error('Middle dot in slide text');
  authoredTexts.push(value);
  const s = shape(name, 'textbox', x, y, w, h);
  s.text = value;
  s.text.style = { typeface: 'Apple SD Gothic Neo', fontSize: size, color, bold,
    alignment: align, verticalAlignment: 'middle', autoFit: 'none', wrap: 'none',
    insets: { top: 0, bottom: 0, left: 0, right: 0 } };
  return s;
}
function bullet(name, value, x, y, w, h, size = 21, heading = true) {
  authoredTexts.push(value);
  const s = shape(name, 'textbox', x, y, w, h);
  s.text = makeNativeBulletParagraphs([value], {
    marginLeftPoints: heading ? 15 : 14, hangingPoints: heading ? 12 : 8, spaceAfterPoints: 0,
  }).map(p => ({ ...p, bulletCharacter: heading ? '▪' : '-' }));
  s.text.style = { typeface: 'Apple SD Gothic Neo', fontSize: size, color: C.ink, bold: heading,
    autoFit: 'none', wrap: 'square', verticalAlignment: 'top', insets: { top: 0, bottom: 0, left: 0, right: 0 } };
  return s;
}
function line(name, x1, y1, x2, y2, color = C.rule, width = 1, dashed = false) {
  const s = shape(name, 'line', Math.min(x1, x2), Math.min(y1, y2), Math.abs(x2 - x1), Math.abs(y2 - y1), 'none', color, width);
  if (dashed) s.line = { style: 'dashed', fill: color, width };
  return s;
}
function connect(a, b, from = 'bottom', to = 'top', color = C.blue, dashed = false, arrow = true) {
  connectionCount++;
  return slide.shapes.connect(a, b, { kind: 'straight', fromSide: from, toSide: to,
    line: { style: dashed ? 'dashed' : 'solid', fill: color, width: 1.4 },
    ...(arrow ? { tail: { type: 'triangle', width: 'sm', length: 'sm' } } : {}) });
}
function anchor(name, x, y) { return shape(name, 'rect', x - 0.5, y - 0.5, 1, 1); }
function highlight(name, value, x, y, w, size = 19.5) {
  shape(`${name}-green`, 'rect', x, y + 16, w, 12, C.green);
  text(name, value, x, y, w, 30, size, C.ink, true, 'center');
}

// Match 03-과제필요성.png: three tall outlined areas with overlapping navy tabs.
const panels = [16, 438, 860];
const titles = ['의도에 맞는 정확한 처리 필요', '대화를 이어가는 빠른 처리 필요', 'Agent 기술 발전에 대응 필요'];
panels.forEach((x, i) => {
  shape(`frame-${i}`, 'rect', x, 150, 404, 533, C.white, C.rule, 1.6, 11);
  const tab = shape(`tab-${i}`, 'rect', x + 24, 104, 356, 52, C.navy, C.white, 1.8, 12);
  tab.shadow = '1px 3px 7px #000000/22';
  text(`tab-text-${i}`, titles[i], x + 31, 111, 342, 38, 22, C.white, true, 'center');
});

// 1. Ground the same correction in a selected screen item and prior conditions.
bullet('accuracy-heading-1', '짧은 요청에도 여러 판단이 필요', 29, 178, 374, 31);
bullet('accuracy-desc-1', '화면과 대화, 진행 중인 Task를\n함께 해석', 47, 219, 350, 49, 18, false);
text('screen-label', '화면', 47, 279, 111, 23, 16, C.gray, true);
text('conversation-label', 'Conversation', 181, 279, 217, 23, 16, C.gray, true);
const screen = shape('screen-evidence', 'rect', 47, 307, 113, 73, C.white, '#AEBBC8', 1, 3);
shape('screen-titlebar', 'rect', 47, 307, 113, 12, C.light);
line('table-grid-h-1', 57, 330, 150, 330, '#C4CED6', 1);
line('table-grid-h-2', 57, 344, 150, 344, '#C4CED6', 1);
line('table-grid-v', 96, 322, 96, 371, '#C4CED6', 1);
shape('selected-table', 'rect', 55, 346, 97, 26, C.pale, C.blue, 1.4);
text('selected-table-label', '선택한 표', 57, 347, 93, 24, 15.5, C.blue, true, 'center');
text('previous-request', '“보고서는 PDF로,\n메일은 결론으로 초안만”', 181, 307, 217, 62, 18);
const correction = text('correction', '“메일에는 이 표를 넣어줘”', 70, 390, 322, 33, 20, C.blue, true, 'center');
connect(screen, correction, 'bottom', 'left', C.blue, true, false);
bullet('accuracy-heading-2', '정정 범위와 기존 조건을 함께 반영', 29, 444, 374, 32, 21);
bullet('accuracy-desc-2', '메일 내용은 바꾸고 다른 조건은 보존', 47, 485, 350, 27, 18, false);
const table = slide.tables.add({ rows: 4, columns: 3, left: 44, top: 518, width: 350, height: 110,
  columnWidths: [110, 120, 120], values: [
    ['항목', '정정 전', '정정 후'],
    ['메일 내용', '보고서 결론', '선택한 표'],
    ['보고서 형식', 'PDF', 'PDF 유지'],
    ['메일 처리', '초안만', '초안만 유지'],
  ] });
table.styleOptions = { headerRow: false, firstColumn: false, bandedRows: false, bandedColumns: false };
table.borders.assign({ style: 'solid', fill: '#AEBBC8', width: 0.8 });
table.cells.block({ row: 0, column: 0, rowCount: 4, columnCount: 3 }).assign({ margins: { left: 3, right: 3, top: 2, bottom: 2 } });
for (let r = 0; r < 4; r++) for (let c = 0; c < 3; c++) {
  const cell = table.getCell(r, c);
  cell.fill = r === 0 ? C.light : c === 2 ? (r === 1 ? C.pale : '#EAF3E2') : C.white;
  cell.text.style = { typeface: 'Apple SD Gothic Neo', fontSize: 16.5, bold: r === 0 || c === 2,
    color: r === 1 && c === 2 ? C.blue : C.ink, alignment: 'center', verticalAlignment: 'middle', autoFit: 'none',
    insets: { top: 2, bottom: 2, left: 3, right: 3 } };
}
highlight('accuracy-conclusion', '바꿀 부분과 유지할 조건을 정확히 구분', 47, 645, 348, 19);

// 2. A normal work request continues while VIA answers a later progress query.
bullet('speed-heading-1', '업무 진행 중에도 대화는 계속됨', 451, 178, 374, 31);
bullet('speed-desc-1', '새 발화와 Agent의 질문, 결과가\n서로 다른 시점에 도착', 469, 219, 350, 49, 18, false);
const start = 529, mid = 652, end = 747, delivered = 823;
text('event-request', '업무 요청', start - 41, 281, 82, 25, 16, C.gray, true, 'center');
text('event-query', '진행 확인', mid - 41, 281, 82, 25, 16, C.gray, true, 'center');
text('event-result', '결과 도착', end - 41, 281, 82, 25, 16, C.gray, true, 'center');
for (const [name, x] of [['request', start], ['query', mid], ['result', end]]) line(`guide-${name}`, x, 313, x, 454, '#DCE3E9', 0.8, true);
text('lane-user', '사용자', 457, 319, 66, 26, 17, C.gray, true);
text('lane-via', 'VIA', 457, 365, 66, 26, 17, C.blue, true);
text('lane-agent', 'Agent', 457, 414, 66, 26, 17, C.navy, true);
line('user-lane', start, 331, 829, 331, '#AEBBC8', 1.2);
const u1 = shape('user-request', 'ellipse', start - 4, 327, 8, 8, C.blue);
const u2 = shape('user-query', 'ellipse', mid - 4, 327, 8, 8, C.blue);
const u3 = shape('user-result', 'ellipse', delivered - 4, 327, 8, 8, C.blue);
const via1 = shape('via-request-work', 'rect', start - 8, 365, 93, 27, C.pale, C.blue, 0.8, 3);
const via2 = shape('via-query-work', 'rect', mid - 30, 365, 86, 27, C.pale, C.blue, 0.8, 3);
const via3 = shape('via-result-work', 'rect', end, 365, 81, 27, C.pale, C.blue, 0.8, 3);
text('request-work-label', '해석과 위임', start - 8, 366, 93, 25, 15.5, C.blue, true, 'center');
text('query-work-label', '상태 안내', mid - 30, 366, 86, 25, 15.5, C.blue, true, 'center');
text('result-work-label', '결과 전달', end, 366, 81, 25, 15.5, C.blue, true, 'center');
const work = shape('agent-running-work', 'rect', 607, 417, end - 607, 27, C.navy, 'none', 0, 3);
text('agent-running-label', '보고서 작성', 607, 418, end - 607, 25, 16.5, C.white, true, 'center');
connect(u1, anchor('via-request-in', start, 365), 'bottom', 'top');
connect(u2, anchor('via-query-in', mid, 365), 'bottom', 'top');
connect(anchor('via-result-out', delivered, 365), u3, 'top', 'bottom');
const progressReply = shape('user-progress-reply', 'ellipse', 694, 327, 8, 8, C.blue);
connect(anchor('via-query-out', 698, 365), progressReply, 'top', 'bottom');
const delegateFrom = anchor('delegate-from', 607, 392), delegateTo = anchor('delegate-to', 607, 417);
const resultFrom = anchor('result-from', end, 417), resultTo = anchor('result-to', end, 392);
connect(delegateFrom, delegateTo, 'bottom', 'top', C.navy);
connect(resultFrom, resultTo, 'top', 'bottom', C.navy);
text('work-query-caption', '진행 질문에 답하는 동안에도 업무는 계속', 464, 460, 357, 28, 17, C.gray, false, 'center');
bullet('speed-heading-2', '반응과 결과 전달까지 빠르게', 451, 507, 374, 31, 21);
bullet('speed-desc-2', '입력에 반응하고 준비된 결과를\n전달하는 VIA 처리시간을 줄여야 함', 469, 547, 350, 49, 18, false);
text('external-cost-distinction', '실제 업무 실행시간과 VIA 처리시간을 구분', 469, 607, 350, 25, 16.5, C.gray);
highlight('speed-conclusion', 'VIA 처리시간을 줄여 대화 흐름 유지', 469, 645, 348, 19);

// 3. Change affects both the technologies inside VIA and its Agent integrations.
bullet('change-heading-1', 'VIA 내부의 Agent 기술도 발전', 873, 178, 374, 31);
bullet('change-desc-1', '요청 이해와 기억, Context,\n상호작용 방식의 변화를 수용', 891, 219, 350, 49, 18, false);
text('technology-scope-label', 'VIA', 897, 282, 344, 27, 21, C.navy, true, 'center');
shape('technology-scope', 'rect', 891, 315, 346, 111, C.white, '#AEBBC8', 1.2, 6);
line('scope-middle', 1064, 329, 1064, 412, '#DCE3E9', 1);
line('scope-horizontal', 905, 370, 1223, 370, '#DCE3E9', 1);
text('technology-model', '모델과 Runtime', 901, 330, 160, 30, 18, C.navy, false, 'center');
text('technology-meaning', '요청 이해', 1071, 330, 156, 30, 18, C.navy, false, 'center');
shape('changed-context', 'rect', 899, 377, 162, 34, '#EAF3E2', 'none', 0, 3);
text('technology-context', '기억과 Context', 901, 379, 160, 30, 18, C.navy, true, 'center');
text('technology-interaction', '연속 상호작용', 1071, 379, 156, 30, 18, C.navy, false, 'center');
text('technology-change-caption', '새 기술의 적용 범위를 필요한 부분에 한정', 891, 438, 346, 28, 17, C.gray, false, 'center');
bullet('change-heading-2', '새로운 Agent와 연동 방식도 등장', 873, 474, 374, 31, 21);
bullet('change-desc-2', 'Downstream Agent의 추가와\n연동 방식 변화에 대응', 891, 514, 350, 47, 18, false);
const via = shape('integration-via', 'rect', 902, 577, 76, 43, C.navy, 'none', 0, 4);
text('integration-via-label', 'VIA', 902, 578, 76, 41, 21, C.white, true, 'center');
const junction = anchor('integration-junction', 1007, 598.5);
connect(via, junction, 'right', 'left', C.gray, false, false);
const agentRows = [['Agent A', 561, C.white], ['Agent B', 587, C.white], ['새 Agent', 613, '#EAF3E2']];
agentRows.forEach(([label, y, fill], i) => {
  const agent = shape(`integration-agent-${i}`, 'rect', 1066, y, 161, 23, fill, i === 2 ? C.blue : '#AEBBC8', 1, 3);
  text(`integration-agent-label-${i}`, label, 1066, y + 1, 161, 21, 16.5, i === 2 ? C.blue : C.gray, i === 2, 'center');
  const branch = anchor(`integration-branch-${i}`, 1007, y + 11.5);
  if (i !== 1) line(`integration-spine-${i}`, 1007, 598.5, 1007, y + 11.5, i === 2 ? C.blue : C.gray, 1.2);
  connect(branch, agent, 'right', 'left', i === 2 ? C.blue : C.gray);
});
highlight('change-conclusion', '기존 기능을 유지하며 기술 발전 수용', 891, 645, 348, 19);

const forbidden = ['기능 정확성', '기능 적절성', '기능 완전성', '응답 신속성', '요청 완료 신속성', '변경 용이성', '메모리 효율성', '복구 용이성', '분석 용이성', '기밀성'];
for (const value of authoredTexts) if (forbidden.some(s => value.includes(s)) || /V-\d\d|04-4[1-5]|QA-\d/.test(value)) throw new Error(`QA/DP term shown early: ${value}`);
const notes = [
  'VIA는 화면과 대화, 진행 중인 Task를 함께 해석해 사용자 의도대로 요청을 처리해야 한다. 후속 정정은 바꿀 범위를 구분하고 기존 조건을 유지한다.',
  '첫 사례는 선택한 표와 앞선 Conversation을 현재 정정 Request에 연결한다. 메일 내용은 보고서 결론에서 선택한 표로 바꾸고, 보고서 PDF와 메일 초안 조건은 유지한다. 실제 문서 및 메일 업무 수행은 Downstream Agent 책임이다.',
  '가운데 시간축은 업무 요청, 진행 확인, 결과 도착의 개념도다. 위치와 구간 길이는 실제 시간이나 처리 비율이 아니다. 최초 해석과 위임 이후 외부 보고서 작성이 진행되는 동안 VIA는 확인된 상태로 진행 질문에 답한다. 새 Agent Execution을 만들어야 한다는 뜻이 아니다. 실제 접수와 완료, 결과 도착 및 사용자 전달은 구별한다.',
  '첫 반응과 준비된 결과의 전달에는 VIA의 해석, 조회, 검증 및 출력 시간이 필요하다. 외부 업무만 기다리는 시간은 VIA 처리시간과 구별한다. 입력과 인식의 지속은 해석 및 외부 실행과 병행해야 한다.',
  '오른쪽은 기술 발전의 범위와 연동 확장의 개념도다. 네 영역은 기술 범위이며 Component 배치, 독립 모델 수나 선택된 대안이 아니다. 연두색 기억 영역은 변경 영향의 범위를 설명하는 예시다. 학습과 모델 내부 알고리즘 개발은 본 과제의 Architecture 범위 밖이다. VIA 내부 기술의 발전과 Downstream Agent의 변화가 모두 설계 변경의 동기가 된다.',
  '배경의 세 축은 뒤에서 제시할 품질 요구사항 및 DP 검토의 배경이다. 현재 페이지에서는 품질 이름과 ID를 먼저 표시하지 않는다. 특정 구조의 우열이나 성능 수치, 대안 선택 결과를 주장하지 않는다.',
  '문안 근거: docs/architecture/03-fixed-architecture-scope.md; docs/architecture/05-representative-use-cases.md; docs/presentations_files/quality-attributes.md; docs/architecture/12-decisions/decision-packages/04-41-request-resolution-control.md; 04-42-lifecycle-ownership.md; 04-43-request-interpretation.md; 04-44-continuous-interaction.md; 04-45-memory-and-context.md',
  '레이아웃 참조: docs/presentations_files/presentation_example1/03-과제필요성.png',
].join('\n');
slide.speakerNotes.textFrame.setText(notes);
const authoredPath = path.join(build, `artifact-authored-${attempt}.pptx`);
await (await PresentationFile.exportPptx(presentation)).save(authoredPath);

// Transplant only new slide-3 objects and notes into the original package.
// All other pages, GIF bytes, masters, fonts and relationships remain intact.
const JSZip = createRequire(import.meta.url)('jszip');
const original = await JSZip.loadAsync(await fs.readFile(source));
const authored = await JSZip.loadAsync(await fs.readFile(authoredPath));
const slidePath = 'ppt/slides/slide3.xml';
let sourceXML = await original.file(slidePath).async('string');
const authoredXML = await authored.file(slidePath).async('string');
const blockRE = /<p:(sp|cxnSp|graphicFrame)\b[^>]*>[\s\S]*?<\/p:\1>/g;
const blocks = [...authoredXML.matchAll(blockRE)].map(m => m[0]).filter(block => {
  const name = block.match(/<p:cNvPr\b[^>]*\bname="([^"]*)"/)?.[1];
  return (name && !originalNames.has(name)) || (!name && /^<p:(?:cxnSp|graphicFrame)\b/.test(block));
});
if (blocks.length !== names.size + connectionCount + 1) throw new Error(`Native objects lost: ${blocks.length}/${names.size + connectionCount + 1}`);
const idMap = new Map(blocks.map((block, i) => [block.match(/<p:cNvPr\b[^>]*\bid="(\d+)"/)[1], String(1000 + i)]));
const newBlocks = blocks.map(block => block
  .replace(/(<p:cNvPr\b[^>]*\bid=")(\d+)(")/, (_, a, id, b) => a + idMap.get(id) + b)
  .replace(/(<a:(?:stCxn|endCxn)\b[^>]*\bid=")(\d+)(")/g, (_, a, id, b) => a + (idMap.get(id) ?? id) + b));
const keepNames = new Set(['r3-120', 'line-121', 'r3-132', ...Array.from({ length: 10 }, (_, i) => `r3-${122 + i}`)]);
sourceXML = sourceXML.replace(blockRE, block => {
  const name = block.match(/<p:cNvPr\b[^>]*\bname="([^"]*)"/)?.[1];
  return keepNames.has(name) ? block : '';
}).replace('</p:spTree>', newBlocks.join('') + '</p:spTree>');
original.file(slidePath, sourceXML, { createFolders: false });
const notesPath = 'ppt/notesSlides/notesSlide3.xml';
let notesXML = await original.file(notesPath).async('string');
const escapeXML = s => s.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
let updatedNotes = false;
notesXML = notesXML.replace(/<p:sp>[\s\S]*?<\/p:sp>/g, block => {
  if (!/<p:ph\b[^>]*type="body"/.test(block)) return block;
  const paragraphs = notes.split('\n').map(value => `<a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:r><a:t>${escapeXML(value)}</a:t></a:r></a:p>`).join('');
  const updated = block.replace(/(<p:txBody>[\s\S]*?<a:lstStyle\b[^>]*\/>)[\s\S]*?(<\/p:txBody>)/, `$1${paragraphs}$2`);
  if (updated === block) throw new Error('Notes replacement failed');
  updatedNotes = true;
  return updated;
});
if (!updatedNotes) throw new Error('Missing notes placeholder');
original.file(notesPath, notesXML, { createFolders: false });
const candidate = path.join(build, `candidate-${attempt}.pptx`);
await fs.writeFile(candidate, await original.generateAsync({ type: 'nodebuffer', compression: 'DEFLATE' }));
const final = path.join(build, `final-${attempt}`, filename);
await fs.mkdir(path.dirname(final), { recursive: true });
await finalizePresentation({ workspaceDir: build, candidatePath: candidate, finalPath: final,
  explicitTotalSlideCount: 3, requiredNativeTableOwnerSlides: [3], pythonExecutable: python,
  integrityValidatorPath: path.join(skill, 'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath: path.join(skill, 'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs: ['--expected-slide-size-emu', '12192000,6858000', '--validate-heading-fit', '--validate-bullet-geometry', '--require-native-table-slide', '3'],
  fontPolicy: { basis: 'reference', families: ['Apple SD Gothic Neo'], referencePath: source,
    referenceSha256: createHash('sha256').update(await fs.readFile(source)).digest('hex') },
  verifyArtifactToolImport: true, receiptPath: path.join(build, `validation-${attempt}.json`),
});
execFileSync(process.execPath, [path.join(skill, 'container_tools/render_presentation.mjs'), '--input', final,
  '--output_dir', path.join(build, `render-${attempt}`), '--scale', '1.25'], { stdio: 'pipe', env: process.env });
console.log(JSON.stringify({ final, slides: 3, nativeObjects: blocks.length, nativeTables: 1 }));
