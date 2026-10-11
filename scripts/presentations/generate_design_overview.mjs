import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
import { Presentation, PresentationFile, FileBlob } from '@oai/artifact-tool';

// Run a copy from a private build directory linked to the bundled node_modules.
const [repoArg, buildArg, revision = '01'] = process.argv.slice(2);
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(p => path.isAbsolute(p ?? ''))) throw new Error('Set bundled skill and Python paths.');
const out = path.join(repo, 'docs/presentations_files/design-overview');
await fs.mkdir(out, { recursive: true });
const sourceOverview = path.join(out, 'VIA-design-overview-DP41-45.pptx');
const referenceSha256 = createHash('sha256').update(await fs.readFile(sourceOverview)).digest('hex');
const qaPath = path.join(repo, 'docs/presentations_files/quality-attributes/quality-attributes.json');
const qa = JSON.parse(await fs.readFile(qaPath, 'utf8'));
if (qa.rows.map(row => row.id).join(',') !== 'ASR-QA-01,ASR-QA-02,ASR-QA-03,ASR-QA-04,ASR-QA-05,ASR-QA-06') {
  throw new Error('Expected the six current ASR-QA definitions in priority order.');
}
const qaTargets = [
  '98% 이상', '평균 1초 이하, stop 0.1초 이하', '평균 10초 이하',
  qa.rows[3].target, '평균 3개 이하, 과제별 5개 이하', '4,096MiB 이하 (commit)',
];
const p = Presentation.create({ slideSize: { width: 1920, height: 1080 } });
const s = p.slides.add(); s.background.fill = '#FFFFFF';
const font = 'Apple SD Gothic Neo';
const C = { ink: '#141414', gray: '#95989B', light: '#F6F7F8', external: '#DDE0E4', navy: '#354354',
  41: '#E5A700', 42: '#2463DD', 44: '#00A65A', 45: '#8D49B8' };
const pale = { 41: '#FFF1BE', 42: '#E4EDFF', 44: '#E0F5E8', 45: '#F1E5FA' };
const requiredTexts = [];
function shape(name, geometry, x, y, w, h, fill = 'none', color = 'none', width = 0, dashed = false) {
  return s.shapes.add({ name: `overview-${name}`, geometry, position: { left: x, top: y, width: w, height: h },
    fill, line: { fill: color, width, style: dashed ? 'dashed' : 'solid' } });
}
function text(name, value, x, y, w, h, size = 30, color = C.ink, bold = false, align = 'left') {
  requiredTexts.push(value);
  const obj = shape(name, 'textbox', x, y, w, h);
  obj.text = value;
  obj.text.style = { typeface: font, fontSize: size, color, bold, alignment: align, verticalAlignment: 'middle',
    wrap: 'none', autoFit: 'none', insets: { left: 0, right: 0, top: 0, bottom: 0 } };
  return obj;
}
function poly(name, points, color, width = 3, dashed = false, fill = 'none', close = false) {
  const xs = points.map(p => p[0]), ys = points.map(p => p[1]);
  const x = Math.min(...xs), y = Math.min(...ys), w = Math.max(.1, Math.max(...xs) - x), h = Math.max(.1, Math.max(...ys) - y);
  return s.shapes.add({ name: `overview-${name}`, geometry: 'custom', position: { left: x, top: y, width: w, height: h }, fill,
    line: { fill: color, width, style: dashed ? 'dashed' : 'solid' },
    customPaths: [{ width: w, height: h, commands: [...points.map(([px, py], i) => ({ [i ? 'lineTo' : 'moveTo']: { x: px - x, y: py - y } })), ...(close ? [{ close: {} }] : [])] }] });
}
function rect(name, x, y, w, h, fill = '#FFFFFF', color = C.ink, width = 1.7) {
  return shape(name, 'rect', x, y, w, h, fill, color, width);
}
function component(name, x, y, w, h, fill = '#FFFFFF', geometry = 'rect', size = 32) {
  shape(`component-${name}`, geometry, x, y, w, h, fill, C.ink, 1.7);
  text(`name-${name}`, name, x + 7, y, w - 14, h, size, C.ink, false, 'center');
}
function scope(id, name, x, y, w, h) { shape(`dp${id}-${name}`, 'rect', x, y, w, h, 'none', C[id], 4, true); }
function badge(id, name, x, y, w = 105, h = 36) {
  rect(`tag-${id}-${name}`, x, y, w, h, pale[id], C[id], 3);
  text(`tag-text-${id}-${name}`, `04-${id}`, x, y, w, h, h === 36 ? 28 : 24, C.ink, false, 'center');
}

text('title', '설계 Overview', 52, 24, 700, 74, 64, C.ink, true);
const steps = ['과제 소개', '요구사항 분석', '설계', '구현 및 검증', '결론'];
steps.forEach((label, i) => {
  const x = 1130 + i * 148, y = 29, w = 153, h = 47;
  const pts = i === 0 ? [[x,y],[x+w-19,y],[x+w,y+h/2],[x+w-19,y+h],[x,y+h]]
    : [[x,y],[x+w-19,y],[x+w,y+h/2],[x+w-19,y+h],[x,y+h],[x+19,y+h/2]];
  poly(`step-${i}`, pts, '#FFFFFF', 3, false, i === 2 ? '#38A9C9' : C.gray, true);
  text(`step-label-${i}`, label, x + (i ? 16 : 0), y, w - (i ? 20 : 9), h, 22, '#FFFFFF', true, 'center');
});
poly('header-rule', [[52,106],[1872,106]], '#9A9A9A', 2.5);

// Priority-ordered current QA targets, with selected causal design motivations.
text('left-heading', '품질 목표와 주요 설계 질문', 52, 165, 780, 48, 34, C.navy, true);
text('quality-contract', `${qa.contract_version} / 번호순 우선순위`, 52, 216, 780, 30, 22, '#59616B');
const primaryLinks = [[[41,282],[45,622]], [[44,452]], [[44,452]], [[41,282],[45,622]], [[42,772]], [[42,772]]];
for (const [index, row] of qa.rows.entries()) {
  const y = 253 + index * 97, center = y + 40;
  for (const [id, target] of primaryLinks[index]) poly(`goal-${index + 1}-${id}`, [[340,center],[367,target]], C[id], 2.6);
  rect(`goal-${row.id}`, 52, y, 288, 81, C.navy, '#45699B', 2);
  text(`goal-name-${row.id}`, `${String(index + 1).padStart(2, '0')} ${row.name}`, 61, y + 5, 270, 36, 24, '#FFFFFF', true, 'center');
  text(`goal-target-${row.id}`, qaTargets[index], 58, y + 42, 276, 31, index === 1 || index === 4 ? 19 : 22, '#FFFFFF', false, 'center');
}
const decisions = [
  [41, 250, 'Request 해석 제어', '조회와 의미 완성의 진행 책임'],
  [44, 420, '지속 대화 실행', '입력과 질문 및 결과의 후속 실행'],
  [45, 590, '기억과 Context', '과거 근거의 구성 및 공통 기억 제공'],
  [42, 740, 'Conversation과 Task 관리', '상태 소유와 실행 수명의 분리 방식'],
];
for (const [id,y,title,detail] of decisions) {
  rect(`swatch-${id}`, 344, y + 11, 24, 42, C[id], C[id], 0);
  text(`decision-title-${id}`, `04-${id}. ${title}`, 390, y, 466, 50, id === 42 ? 29 : 33, C.ink, true);
  text(`decision-detail-${id}`, detail, 390, y + 52, 465, 39, 27, '#505760');
}

// Right: exactly the supplied overall architecture's hierarchy and Component names.
rect('VIA-boundary', 886, 135, 984, 769);
text('VIA-label', 'VIA', 909, 145, 220, 40, 35);
rect('Interaction', 905, 195, 945, 145, C.light);
text('Interaction-label', 'Interaction', 924, 198, 450, 40, 29);
component('Interaction Manager', 936, 252, 431, 56);
component('Response Manager', 1396, 252, 431, 56);
rect('Orchestration', 905, 354, 945, 366, C.light);
text('Orchestration-label', 'Orchestration', 924, 357, 245, 43, 29);
component('Request Controller', 936, 428, 431, 52);
component('Request Interpreter', 1396, 428, 431, 52);
component('Context Manager', 936, 523, 431, 52);
component('Policy Manager', 1396, 523, 431, 52);
component('Task Manager', 936, 618, 431, 52);
component('Agent Gateway', 1396, 618, 431, 52);
rect('Shared', 905, 738, 945, 147, C.light);
text('Shared-label', 'Shared', 924, 744, 200, 43, 29);
component('State Store', 936, 817, 431, 47, '#FFFFFF', 'can');
component('Model Access', 1396, 817, 431, 47);

rect('External-boundary', 886, 922, 984, 112);
text('External-label', 'External Components', 909, 927, 600, 37, 28);
component('Context Sources', 908, 976, 229, 40, C.external, 'rect', 24);
component('Downstream Agents', 1154, 976, 243, 40, C.external, 'rect', 24);
rect('Model-runtime', 1414, 959, 433, 65, C.light);
text('Model-runtime-label', 'Cloud Model Runtime', 1426, 960, 400, 24, 21);
component('음성 모델', 1428, 990, 174, 25, C.external, 'rect', 21);
component('의미 LLM', 1620, 990, 212, 25, C.external, 'rect', 21);

// Primary scope markers, not all common consumers/providers or selected alternatives.
poly('dp44-interaction-and-control', [[917,244],[1838,244],[1838,316],[1376,316],[1376,491],[917,491],[917,244]], C[44], 4, true);
badge(44, 'interaction', 1726, 203);
scope(41, 'resolution-control', 924, 416, 914, 71);
badge(41, 'resolution', 1286, 367);
poly('dp41-tag-link', [[1338,403],[1338,416]], C[41], 3);
scope(42, 'dialogue-authority', 930, 421, 442, 63);
badge(42, 'dialogue', 1168, 367);
poly('dp42-dialogue-tag-link', [[1220,403],[1220,421]], C[42], 3);
scope(42, 'work-authority', 923, 609, 915, 69);
badge(42, 'work', 1716, 681);
scope(45, 'context-production', 923, 513, 455, 71);
badge(45, 'context', 1245, 493, 105, 28);
scope(42, 'durable-ownership', 923, 802, 455, 73);
scope(45, 'memory-storage', 930, 809, 441, 59);
badge(42, 'store', 1153, 750);
badge(45, 'store', 1273, 750);
poly('dp42-store-tag-link', [[1205,786],[1205,802]], C[42], 3);
poly('dp45-store-tag-link', [[1325,786],[1325,809]], C[45], 3);

// Legend mirrors the supplied reference while adding the DP scope convention.
rect('legend', 52, 885, 779, 149);
text('legend-title', 'Legend', 52, 887, 779, 35, 27, C.ink, false, 'center');
rect('legend-module', 72, 940, 59, 28);
text('legend-module-text', '기능 모듈', 145, 935, 183, 38, 23);
rect('legend-external', 372, 940, 59, 28, C.external);
text('legend-external-text', '외부 책임 모듈', 445, 935, 305, 38, 23);
shape('legend-store', 'can', 72, 991, 59, 27, '#FFFFFF', C.ink, 1.7);
text('legend-store-text', '영속 저장소', 145, 985, 190, 38, 23);
shape('legend-dp', 'rect', 372, 991, 59, 27, 'none', C[42], 3, true);
text('legend-dp-text', '색상 점선: 주요 DP 범위', 445, 985, 354, 38, 23);
text('scope-footnote', '점선은 주요 비교 위치. 모든 DP를 6개 QA로 검토하며, 권위 상태와 VAD는 로컬, 음성과 의미 모델은 클라우드', 52, 1043, 1818, 30, 21, '#59616B');

const base = 'https://github.com/cbyche/via-sw-architecture/blob/main/docs/architecture/12-decisions/';
const docs = { 41: '04-41-request-resolution-control.md', 42: '04-42-lifecycle-ownership.md',
  44: '04-44-continuous-interaction.md', 45: '04-45-memory-and-context.md' };
s.speakerNotes.textFrame.setText(`설계 Overview: VIA 공통 책임과 41 → 44 → 45 → 42의 주요 비교 위치\n\n` +
  `사용자 제공 VIA overall architecture(1000001540.jpg)의 경계, 영역, Component 이름과 배치를 편집 가능한 도형으로 재구성했다. NOS 레퍼런스(1000001497.png)의 품질 목표, DP 목록, 색상 점선과 반복 번호 표기 방식을 따른다.\n\n` +
  `색상 점선은 참조 구조에 대한 주요 비교 위치다. 모든 의존성 또는 변경 영향의 전수 집합이 아니며, 특정 A/B/C가 채택된 최종 구조나 OS process 경계를 나타내지 않는다. 같은 Component에 여러 점선이 겹치면 서로 다른 결정 질문이 교차한다는 뜻이다. 새 대안의 Request Resolution Engine, Memory Publisher와 Repository 등은 비교 문서에서 읽으며 현 overview에 이미 구현된 Component로 추가하지 않는다.\n\n` +
  `04-41: Request Interpreter의 해석과 Request Controller의 진행/채택 경계. 모델이 조회와 전체 의미를 제안하는 ReAct와 코드 Request Resolution Engine이 표현된 관계를 완성하는 비교다. Context Manager 및 Task Manager의 조회, Policy Manager와 Model Access는 관련 공통 계약이다.\n` +
  `04-42: Request Controller의 Conversation/Request 상태와 Task Manager/Agent Gateway의 업무/실행/전달 상태, State Store의 공동 또는 분리 확정 경계. Response Manager의 내구 전달 상태도 대화 측 관련 계약이다. 모듈화된 통합 Core와 독립 대화/업무 서비스의 상태 소유 및 실행 수명을 비교하며, 의미 생산의 중앙/분산 비교와 구별한다.\n` +
  `04-44: Interaction Manager, Request Controller 및 Response Manager를 잇는 후속 실행의 조직. 중앙 비동기 Orchestration/Mediator와 반응형 Dataflow/Pipes-and-Filters다. Task Manager/Agent Gateway의 확인된 질문/결과도 이 흐름에 들어오고 모델 자원은 공통이다. 반응형이 항상 빠르다는 측정 결론은 없다.\n` +
  `04-45: Context Manager의 근거 생산/조회/폐기와 State Store에 유지하는 기억. 원본 서비스 조합과 공통 파생 기억 저장소를 비교한다. 입력, Request, Task와 실제 전달 기록의 원본 소유자는 유지한다. 저장소만 추가하거나 State Store가 의미 권한을 가져가는 비교가 아니다.\n\n` +
  `품질 목표는 ${qa.contract_version} 원본에서 투영한 여섯 ASR-QA다. 정확성, 상호작용 응답시간, VIA 요청 처리시간, 모델 비용, 변경 용이성, 로컬 commit 메모리 순으로 우선한다. 선은 주된 설계 동기의 일부이며 전체 적용성 목록이나 효과 크기, 대안 우열을 뜻하지 않는다. 여섯 품질과 보호 조건은 네 비교에 모두 확인한다.\n` +
  `모델 배치는 04-40 공통 실행 계약을 따른다. VIA와 Request/Task/실제 전달의 권위 상태는 사용자 PC의 로컬 owner가 관리한다. Interaction Manager 내부 Turn-Taking Control의 로컬 VAD가 발화 경계를 제공하고 즉시 재생 중단을 수행한다. Model Access는 클라우드 음성 모델과 별도 의미 LLM의 adapter, session/call, timeout/cancel/usage를 관리한다. 전사 의존성은 공급자별 inventory와 청구 항목으로 확인하며 별도 Streaming ASR을 공통 필수 모델로 확정하지 않는다. 모델 provider/checkpoint의 최종 채택이나 모델 서버 메모리를 로컬 VIA 메모리로 합산하지 않는다.\n` +
  `현재 P21은 21개 사례를 5회 독립 반복하며 모델 비용 목표는 ${qa.rows[3].target}다. 목표와 점수는 작성자 평가 기준이고 구현/모델 실행/측정 결과가 아니다. 기존 Reference Architecture와 QA catalog 및 ADR는 보존하며 이 발표 그림이 그것들을 일괄 변경하지 않는다.\n\n` +
  `${base}decision-packages/04-40-common-execution-contract.md\n` +
  `${base}decision-packages/03-02-quality-attribute-definitions.md\n` +
  `${base}target-architecture/architecture.md#4-component와-상태-소유권\n` + Object.entries(docs).map(([id,file]) => `04-${id}: ${base}decision-packages/${file}`).join('\n'));

const { finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
const candidate = path.join(build, `candidate-${revision}.pptx`);
const final = path.join(build, 'final', `VIA-design-overview-DP41-45-${revision}.pptx`);
await fs.mkdir(path.dirname(final), { recursive: true });
await (await PresentationFile.exportPptx(p)).save(candidate);
await finalizePresentation({ workspaceDir: build, candidatePath: candidate, finalPath: final, explicitTotalSlideCount: 1,
  pythonExecutable: python, integrityValidatorPath: path.join(skill, 'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath: path.join(skill, 'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs: ['--expected-slide-size-emu', '18288000,10287000', '--validate-heading-fit'],
  requiredNativeTableOwnerSlides: [], fontPolicy: { basis: 'reference', families: [font], referencePath: sourceOverview, referenceSha256 }, verifyArtifactToolImport: true,
  receiptPath: path.join(build, `validation-${revision}.json`) });
const imported = await PresentationFile.importPptx(await FileBlob.load(final));
const preview = await imported.export({ slide: imported.slides.items[0], format: 'png', scale: 1 });
await fs.writeFile(path.join(build, `preview-${revision}.png`), new Uint8Array(await preview.arrayBuffer()));
await fs.writeFile(path.join(build, 'required-texts.json'), JSON.stringify(requiredTexts, null, 2));
await fs.copyFile(final, path.join(out, 'VIA-design-overview-DP41-45.pptx'));
await fs.copyFile(path.join(build, `preview-${revision}.png`), path.join(out, 'design-overview-DP41-45.png'));
console.log(JSON.stringify({ final, preview: path.join(build, `preview-${revision}.png`), nativeObjects: s.shapes.items.length }));
