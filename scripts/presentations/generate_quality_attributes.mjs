import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Presentation, PresentationFile, FileBlob } from '@oai/artifact-tool';

// Run a copy inside a private build directory with the bundled node_modules linked.
// The source JSON is shared by Markdown, native PowerPoint tables and notes.
const [repoArg, buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_quality_attributes.mjs REPO BUILD_DIR');
const repo = await fs.realpath(repoArg);
const build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill ?? '') || !path.isAbsolute(python ?? '')) {
  throw new Error('Set VIA_PRESENTATION_SKILL_DIR and VIA_RUNTIME_PYTHON to absolute runtime paths.');
}
const out = path.join(repo, 'docs/presentations_files/quality-attributes');
const data = JSON.parse(await fs.readFile(path.join(out, 'quality-attributes.json'), 'utf8'));
const rows = data.rows;
const expected = ['V-01','V-02','V-03','V-04','V-05','V-08','V-06','V-07','V-09','V-10','V-11','V-12','V-13'];
const priorities = [1,1,1,2,2,3,4,5,6,6,6,6,6];
if (JSON.stringify(rows.map(r => r.id)) !== JSON.stringify(expected) ||
    JSON.stringify(rows.map(r => r.priority)) !== JSON.stringify(priorities)) {
  throw new Error('All thirteen source V IDs and shared priorities must be preserved.');
}
for (const row of rows) {
  for (const key of ['name','description','target','method','detail','targetBasis','boundary','category']) {
    if (!row[key]?.trim()) throw new Error(`${row.id}: missing ${key}`);
  }
  if ((row.description.match(/[.!?。]/g) ?? []).length !== 1 || !row.description.endsWith('.')) {
    throw new Error(`${row.id}: the explanation must be exactly one sentence.`);
  }
}
const rank = p => [1,2,6].includes(p) ? `공동 ${p}순위` : `${p}순위`;
const src = '../architecture/12-decisions/decision-packages/03-00-quality-scenarios.md';
const table = rows.map(r => `| ${rank(r.priority)} | ${r.id} | ${r.name} | ${r.description} | ${r.target} | ${r.method} |`).join('\n');
const details = rows.map(r => `### ${r.id} ${r.name}\n\n- 측정 초안: ${r.detail}\n- 목표 상태: ${r.targetBasis}\n- 책임 경계: ${r.boundary}\n- 품질 분류: ${r.category}\n`).join('\n');
const markdown = `# VIA 품질 요구사항 발표자료\n\n` +
`> 작성일: ${data.date} / 발표 문안 및 측정 방법 초안 / 구현·측정 결과 없음\n> 원문: [03-00 품질 시나리오 §2·§2.1](${src})\n\n` +
`## 작성 계획과 구성\n\n` +
`1. V-01~13의 이름, 의미와 공동 순위를 원문에 대조한다.\n` +
`2. 각 행을 한 문장 설명, 목표, 측정 방법으로 작성하고 묶인 부특성의 차이를 유지한다.\n` +
`3. 참고 페이지처럼 표를 사용하되 13개를 읽을 수 있도록 1~5순위 8개와 공동 6순위 5개를 두 장으로 나눈다.\n` +
`4. Markdown과 PowerPoint를 같은 원문 데이터로 생성하고 13개 누락 여부, 동순위, 편집 가능한 표, 렌더링과 링크를 검증한다.\n\n` +
`참고 이미지의 표 구성과 설명·목표·측정 기준의 표현을 적용했다. 참고 제품의 숫자, H/M/L 난이도, 상위 5개 ASR 선정은 VIA에 옮기지 않았다. V 표식은 현재 품질 검토 관점이며 확정 QA ID 또는 최종 ASR 선정이 아니다.\n\n` +
`## 목표 수치의 상태\n\n` +
`현재 V 원문은 목표 수치, 시험 모집단과 정확한 지표·집계를 동결하지 않았다. 따라서 미확정 수치는 **미정**으로 표시했다. 기존 요구에 있는 **다른 업무 오실행·중복 실행·무단 사용 0건**은 금지 행위에 대한 조건이며 전체 정확성·복구율 목표를 뜻하지 않는다. 과거 QA의 95%, 1,500ms, 3-element, 5,000ms 제안을 V에 자동 적용하지 않았다.\n\n` +
`아래 측정 방법은 발표를 위한 구체화 초안이다. 실제 측정 전에 모든 DP에 공통인 정의·지표·절차·모집단·집계·실패 처리를 확정해야 한다. 기준선이나 기존 측정 계약을 변경하지 않았다.\n\n` +
`## 우선순위별 품질 요구사항\n\n` +
`| 우선순위 | ID | QA명 | 설명 — 한 문장 | 목표 수치·상태 | 측정 방법 — 초안 |\n` +
`| --- | --- | --- | --- | --- | --- |\n${table}\n\n` +
`공동 순위 안에 추가 우열을 부여하지 않는다. 6순위도 필수 검토 관점으로 모두 유지하며 높은 순위의 이익으로 권한 위반이나 다른 품질 손실을 상쇄하지 않는다. 한 사건의 효과를 여러 관점에 설명하더라도 독립 표본처럼 중복 합산하거나 가중 총점으로 합치지 않는다.\n\n` +
`## 발표 페이지\n\n` +
`- [편집 가능한 PowerPoint](quality-attributes/VIA-quality-attributes.pptx)\n` +
`- [미리보기](quality-attributes/index.html)\n` +
`- [공통 원문 데이터](quality-attributes/quality-attributes.json)\n\n` +
`### 페이지 1 — 1~5순위\n\n![품질 요구사항 1~5순위](quality-attributes/quality-attributes-01.png)\n\n` +
`### 페이지 2 — 공동 6순위\n\n![품질 요구사항 공동 6순위](quality-attributes/quality-attributes-02.png)\n\n` +
`## 측정 시 빠뜨리면 안 되는 내용\n\n${details}\n` +
`## 공통 측정 조건과 목표 확정 항목\n\n` +
`- 사용자 목표, 완료 조건, 공통 요구 목록과 외부 capability/profile을 같게 유지한다. 각 DP의 영향 경로와 비적용 근거를 기록하고 특정 DP에만 지표나 의미를 바꾸지 않는다.\n` +
`- PC·OS·전력 조건, Omni/ASR/helper 구성, warm/cold 조건, 입력률, 동시 업무 수, 자료 크기, background 작업 및 타 앱 부하를 기록한다. 실제 수치는 이 조건을 정한 뒤 설정한다.\n` +
`- 시간 지표는 실제 사용자/source 사건과 유효한 결과·제어 효과를 연결한다. 물리적 위치가 달라도 VIA 책임 비용은 포함하고 Agent 내부 업무 및 사람의 답변 대기를 구분한다.\n` +
`- 정답 기준, 사례·변형·장애·변경 목록, 적용 판정, 반복 수, 지연 분포·집계, timeout과 실패 처리를 결과 전에 확정한다. 미확인·누락·실패를 성공 표본에서 제거하지 않는다.\n` +
`- 묶인 부특성은 자원/수용량, 결함 허용/복구, 변경/모듈성, 분석/시험, 상호운용/공존, 조작/오류 방지를 각각 확인한다.\n` +
`- 정량 목표가 미정인 항목은 단위·상한/하한·근거·적용 조건을 채운 뒤 발표 확정본에 반영한다. 실제 model/제품을 실행하지 않은 자료를 실측 결과로 표시하지 않는다.\n\n` +
`## 근거와 추적성\n\n` +
`| 내용 | 근거 |\n| --- | --- |\n` +
`| V 이름·정의·13개 범위·공동 순위·미동결 상태 | [03-00 §2·§2.1·§2.2·§2.3·§6](${src}) |\n` +
`| 공통 기능 범위와 책임 경계 | [02-00 요구와 해결 수단](../architecture/12-decisions/decision-packages/02-00-requirements-and-choices.md), [고정 범위](../architecture/03-fixed-architecture-scope.md), [대표 UC](../architecture/05-representative-use-cases.md) |\n` +
`| 실제 입력/source 사건과 유효한 종료점 | [event-boundary-contract](../architecture/11-measurement/event-boundary-contract.md) |\n` +
`| 기존 오실행·중복 실행·무단 사용 금지 | [core-asr-contract §7](../architecture/08-quality-attributes/core-asr-contract.md#7-qa-41과-공통-qualification) |\n` +
`| 자원 합산·장애 상태 확인 원칙 | [reliability-and-resource](../architecture/08-quality-attributes/reliability-and-resource.md) |\n` +
`| 로그 완전성과 근거 재계산의 구분 | [observability](../architecture/08-quality-attributes/observability.md) |\n` +
`| 철회·삭제·허용 사용 | [03-00 V-10 및 P-12](${src}), [02-00](../architecture/12-decisions/decision-packages/02-00-requirements-and-choices.md) |\n\n` +
`기존 QA 자료는 의미 경계와 금지 조건의 보조 근거로만 사용했다. V-01~13을 QA-09/19/29/39로 치환하거나 기존 숫자·집계를 승계하지 않았다.\n\n` +
`## 제작·검증 범위\n\n` +
`13개 ID, 원문 QA명과 순위, 한 문장 설명 및 모든 목표·측정 칸의 존재를 생성 시 검사한다. 각 페이지의 표는 이미지가 아닌 PowerPoint의 native table이다. 최종 PPTX의 구조·레이아웃·글꼴·native table·재가져오기를 검사하고 최종 파일을 렌더링해 페이지별로 확인했다. PowerPoint/Google Slides 앱에서 직접 편집·저장·재열기는 확인하지 않았다.\n`;
await fs.writeFile(path.join(repo, 'docs/presentations_files/quality-attributes.md'), markdown);

const { finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
const font = 'Apple SD Gothic Neo';
const presentation = Presentation.create({slideSize:{width:1920,height:1080}});
const text = (slide, name, content, x,y,w,h,size,color,bold=false) => {
  const shape = slide.shapes.add({name,geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
  shape.text = content;
  shape.text.style = {typeface:font,fontSize:size,color,bold,wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  return shape;
};
const blue = '#2873B9', teal = '#087E8B';
const titleWraps = {
  'V-05':'VIA 귀속 요청\n완료 시간', 'V-06':'자원 활용성과\n수용량', 'V-07':'결함 허용성과\n복구성',
  'V-08':'변경 용이성과\n모듈성', 'V-09':'분석 및\n시험 용이성',
  'V-11':'상호운용성과\n공존성', 'V-12':'조작 용이성과\n사용자 오류 방지',
};
for (const [index, group] of [rows.slice(0,8),rows.slice(8)].entries()) {
  const slide = presentation.slides.add();
  slide.background.fill = '#FFFFFF';
  text(slide,'title','품질 요구사항',48,30,1600,78,64,'#111820',true);
  text(slide,'section',index===0?'VIA 품질 관점과 우선순위  /  1~5순위':'VIA 품질 관점과 우선순위  /  공동 6순위',48,120,1750,45,32,teal,true);
  text(slide,'page',`${index+1} / 2`,1770,50,102,40,26,'#5B6673');
  const headerH = 54, rowH = index===0?96:145;
  const bodySize = index===0?22:24;
  const values = [['ID','QA명','설명 / 목표 / 측정 방법(초안)','우선순위'],...group.map(r=>[
    r.id,titleWraps[r.id]??r.name,
    [
      {runs:[{run:r.description,textStyle:{typeface:font,fontSize:`${bodySize}px`,color:'#17232E'}}]},
      {runs:[{run:`목표  ${r.target}`,textStyle:{typeface:font,fontSize:`${bodySize}px`,bold:true,color:blue}}]},
      {runs:[{run:`측정  ${r.method}`,textStyle:{typeface:font,fontSize:`${bodySize-1}px`,color:'#3E4853'}}]},
    ],
    `${r.priority}${[1,2,6].includes(r.priority)?' (공동)':''}`
  ])];
  const table = slide.tables.add({rows:values.length,columns:4,left:48,top:182,width:1824,height:headerH+rowH*group.length,columnWidths:[120,355,1225,124],values});
  table.styleOptions = {headerRow:false,bandedRows:false};
  table.borders.assign({style:'solid',fill:'#D1DEE5',width:1.1});
  table.rows[0].height = headerH;
  for(let row=0;row<values.length;row++){
    if(row>0)table.rows[row].height=rowH;
    for(let col=0;col<4;col++){
      const range=table.cells.block({row,column:col,rowCount:1,columnCount:1});
      range.assign({fill:row===0?teal:row%2?'#EEF4F5':'#F8FAFB',margins:{left:col===2?15:10,right:10,top:3,bottom:3},anchor:'center',textStyle:{typeface:font,fontSize:row===0?27:col===2?bodySize:col===3?25:29,bold:row===0||col!==2,color:row===0?'#FFFFFF':'#183E4A',alignment:col===2?'left':'center'}});
    }
  }
  text(slide,'footnote','수치 목표 미확정, 측정 방법 초안. 0건은 기존 금지 조건이며 측정 전 모든 DP의 공통 계약을 확정한다.',48,1032,1824,33,24,'#53606C');
  slide.speakerNotes.textFrame.setText(`VIA 품질 요구사항 발표 문안, ${data.date}. ${data.status}.\n공동 순위 안에 추가 우열을 부여하지 않는다. V는 검토 관점이며 최종 QA/ASR 선정이 아니다.\n출처: ${data.source} §2, §2.1, §2.2, §2.3, §6. docs/architecture/11-measurement/event-boundary-contract.md. docs/architecture/08-quality-attributes/core-asr-contract.md §7.\n`+group.map(r=>`${r.id} ${r.name}\n설명: ${r.description}\n목표: ${r.target}\n목표 근거: ${r.targetBasis}\n측정 초안: ${r.detail}\n경계: ${r.boundary}\n품질 분류: ${r.category}`).join('\n\n'));
}
const draft=path.join(build,'candidate.pptx');
const final=path.join(build,'final','VIA-quality-attributes.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await (await PresentationFile.exportPptx(presentation)).save(draft);
const result=await finalizePresentation({workspaceDir:build,candidatePath:draft,finalPath:final,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit','--validate-bullet-geometry','--require-native-table-slide','1','--require-native-table-slide','2'],
  requiredNativeTableOwnerSlides:[1,2],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
await fs.copyFile(final,path.join(out,'VIA-quality-attributes.pptx'));
// Render the finalized, imported file, not an in-memory draft.
const finalized=await PresentationFile.importPptx(await FileBlob.load(final));
for (const [index,slide] of finalized.slides.items.entries()) {
  const img=await finalized.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(out,`quality-attributes-0${index+1}.png`),new Uint8Array(await img.arrayBuffer()));
}
await fs.writeFile(path.join(out,'index.html'),`<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 품질 요구사항</title><style>body{margin:30px;background:#eaf0f3;color:#17232e;font-family:system-ui}main{max-width:1600px;margin:auto}img{width:100%;display:block;box-shadow:0 2px 12px #0002;margin:20px 0}a{color:#087e8b}p{line-height:1.6}</style><main><h1>VIA 품질 요구사항</h1><p>V-01~13 전체와 공동 순위 유지. 목표 수치 미확정, 측정 방법 초안. 0건은 기존 금지 조건.</p><p><a href="VIA-quality-attributes.pptx">편집 가능한 PowerPoint</a> · <a href="../quality-attributes.md">설명과 측정 경계</a></p><img src="quality-attributes-01.png" alt="품질 요구사항 1~5순위"><img src="quality-attributes-02.png" alt="품질 요구사항 공동 6순위"></main></html>`);
console.log(JSON.stringify({rows:rows.length,ids:expected,nativeTableSlides:[1,2],result},null,2));
