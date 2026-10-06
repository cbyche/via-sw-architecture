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
const expected = ['V-01','V-02','V-03','V-04','V-05','V-08','V-06','V-07','V-09','V-10'];
const priorities = [1,1,1,2,2,3,4,5,6,6];
if (JSON.stringify(rows.map(r => r.id)) !== JSON.stringify(expected) ||
    JSON.stringify(rows.map(r => r.priority)) !== JSON.stringify(priorities)) {
  throw new Error('Preserve the ten reviewed V IDs and original shared priorities.');
}
for (const row of rows) {
  for (const key of ['name','description','target','method','detail','targetBasis','boundary','category','sourceName']) {
    if (!row[key]?.trim()) throw new Error(`${row.id}: missing ${key}`);
  }
  if ((row.description.match(/[.!?。]/g) ?? []).length !== 1 || !row.description.endsWith('.')) {
    throw new Error(`${row.id}: the explanation must be exactly one sentence.`);
  }
  if (!Number.isFinite(row.metric?.value) || !row.sources.length) throw new Error(`${row.id}: numeric target and evidence required`);
}
const methods = await fs.readFile(path.join(out,'measurement-design.md'),'utf8');
if (/[\u00b7\u2027\u2219\u30fb]/u.test(JSON.stringify(data)+methods)) throw new Error('Middle dots are prohibited.');
const references = Object.fromEntries(methods.split('\n').filter(line=>/^\| S\d\d \|/.test(line)).map(line=>{
  const cells = line.split('|'); return [cells[1].trim(),cells[2].trim()];
}));
const rank = p => [1,2,6].includes(p) ? `공동 ${p}순위` : `${p}순위`;
const table = rows.map(r => `| ${rank(r.priority)} | ${r.id} | ${r.name} | ${r.description} | ${r.target} | ${r.method} |`).join('\n');
const methodsForMain = methods.replaceAll('../../architecture/','../architecture/')
  .replaceAll('](functional-coverage.json)','](quality-attributes/functional-coverage.json)')
  .replaceAll('](evaluation-plan.json)','](quality-attributes/evaluation-plan.json)');
const markdown = `# VIA 품질 요구사항 발표자료\n\n` +
`> 작성일: ${data.date} / 근거 기반 목표 제안 / 구현 및 측정 결과 없음\n\n` +
`## 리뷰 반영 계획과 결과\n\n` +
`1. 각 품질 속성을 하나의 이름과 한 문장 설명 및 단일 수치 목표로 좁힌다.\n` +
`2. 외부 근거의 실제 사실과 VIA 목표를 선택한 계산 및 판단을 구분한다.\n` +
`3. 분모, 정답, 사건 endpoint, 실패 처리와 장치 조건을 측정 설계에 기록한다.\n` +
`4. V-11~13을 제외한 10개를 기존 순위대로 두 페이지에 배치한다.\n` +
`5. 문서와 편집 가능한 PPTX 및 발표자 노트의 내용을 대조하고 최종 렌더링을 확인한다.\n\n` +
`V 표식은 기존 품질 관점을 추적하기 위해 유지한다. 이번 명칭과 목표는 사용자의 발표 리뷰를 반영한 제안이며 기존 Architecture, QA, DP 선택이나 측정 계약을 변경하지 않는다. 공동 순위 내부의 추가 우열도 없다.\n\n` +
`## 우선순위별 품질 요구사항\n\n` +
`| 우선순위 | ID | QA명 | 한 문장 설명 | 단일 목표 수치 | 측정 방법 요약 |\n` +
`| --- | --- | --- | --- | --- | --- |\n${table}\n\n` +
`모든 목표는 제안값이다. 타 시스템의 공개 결과와 VIA의 실제 성능은 구분한다. 정확 처리율 95%의 참조 근거와 VIA 자체 메모리 16GB의 계산 및 전체 시험 조건을 아래에 공개한다.\n\n` +
`## 발표 파일\n\n` +
`- [편집 가능한 PowerPoint](quality-attributes/VIA-quality-attributes.pptx)\n` +
`- [미리보기](quality-attributes/index.html)\n` +
`- [목표와 문안 데이터](quality-attributes/quality-attributes.json)\n` +
`- [94개 기능 단위 목록](quality-attributes/functional-coverage.json)\n` +
`- [공통 조건과 DP별 시험 설계](quality-attributes/evaluation-plan.json)\n` +
`- [측정 설계 편집 원본](quality-attributes/measurement-design.md)\n\n` +
`![품질 요구사항 1페이지](quality-attributes/quality-attributes-01.png)\n\n` +
`![품질 요구사항 2페이지](quality-attributes/quality-attributes-02.png)\n\n` +
`## 목표별 근거 요약\n\n` +
rows.map(r=>`### ${r.id} ${r.name}\n\n${r.targetBasis}\n\n`+r.sources.map(id=>`${id}: ${references[id].replaceAll('../../architecture/','../architecture/')}`).join('\n\n')+'\n').join('\n') +
`\n${methodsForMain.replace(/^# 목표 근거와 측정 설계/,'## 상세 목표 근거와 측정 설계')}\n`;
await fs.writeFile(path.join(repo, 'docs/presentations_files/quality-attributes.md'), markdown.trimEnd()+'\n');

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
const titleWraps = {'V-05':'요청 완료 신속성'};
for (const [index, group] of [rows.slice(0,5),rows.slice(5)].entries()) {
  const slide = presentation.slides.add();
  slide.background.fill = '#FFFFFF';
  text(slide,'title','품질 요구사항',48,30,1600,78,64,'#111820',true);
  text(slide,'section',index===0?'우선순위 1~2  /  기능과 시간':'우선순위 3~6  /  변경, 메모리, 복구와 분석',48,120,1750,45,32,teal,true);
  text(slide,'page',`${index+1} / 2`,1770,50,102,40,26,'#5B6673');
  const headerH = 58, rowH = 154;
  const bodySize = 29;
  const values = [['ID','QA명','설명 / 목표 수치 / 측정 방법','우선순위'],...group.map(r=>[
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
  text(slide,'footnote','근거 기반 목표 제안. 출처와 상세 측정 절차는 발표자 노트 및 Markdown에 수록. 구현 및 측정 결과 없음.',48,1032,1824,33,24,'#53606C');
  slide.speakerNotes.textFrame.setText(`VIA 품질 요구사항 목표 제안, ${data.date}. ${data.status}.\n공동 순위 내부의 추가 우열은 없다. 기존 V 원문: ${data.source}. 삭제: V-11, V-12, V-13.\n`+group.map(r=>`${r.id} ${r.name}\n원래 V 이름: ${r.sourceName}\n설명: ${r.description}\n목표: ${r.target}\n목표 근거: ${r.targetBasis}\n측정: ${r.detail}\n경계: ${r.boundary}\n`+r.sources.map(id=>`${id}: ${references[id]}`).join('\n')).join('\n\n')+`\n\n상세 측정 설계\n${methods}`);

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
await fs.writeFile(path.join(out,'index.html'),`<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 품질 요구사항</title><style>body{margin:30px;background:#eaf0f3;color:#17232e;font-family:system-ui}main{max-width:1600px;margin:auto}img{width:100%;display:block;box-shadow:0 2px 12px #0002;margin:20px 0}a{color:#087e8b}p{line-height:1.6}</style><main><h1>VIA 품질 요구사항</h1><p>사용자 리뷰를 반영한 10개 단일 지표와 근거 기반 목표 제안. 구현 및 측정 결과 없음.</p><p><a href="VIA-quality-attributes.pptx">편집 가능한 PowerPoint</a>, <a href="../quality-attributes.md">목표 근거와 상세 측정 설계</a></p><img src="quality-attributes-01.png" alt="기능과 시간 요구사항"><img src="quality-attributes-02.png" alt="변경과 메모리 및 복구 요구사항"></main></html>`);
console.log(JSON.stringify({rows:rows.length,ids:expected,nativeTableSlides:[1,2],result},null,2));
