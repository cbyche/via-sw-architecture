import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Presentation, PresentationFile, FileBlob } from '@oai/artifact-tool';

// Copy this source into a fresh private build directory with runtime node_modules.
const [repoArg, buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_quality_metrics.mjs REPO BUILD_DIR');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill ?? '') || !path.isAbsolute(python ?? '')) throw new Error('Set absolute skill and Python paths.');
const out = path.join(repo, 'docs/presentations_files/quality-metrics');
const data = JSON.parse(await fs.readFile(path.join(out,'quality-metrics.json'),'utf8'));
const baseline = JSON.parse(await fs.readFile(path.join(out,data.targetSource),'utf8'));
const methods = await fs.readFile(path.join(repo,'docs/presentations_files/quality-attributes/measurement-design.md'),'utf8');
if (JSON.stringify(data.rows.map(r=>r.id)) !== JSON.stringify(baseline.rows.map(r=>r.id))) throw new Error('Ten reviewed QAs required.');
for(const [i,r] of data.rows.entries()) {
  for(const key of ['id','priority','name','description','target','metric']) {
    if(JSON.stringify(r[key]) !== JSON.stringify(baseline.rows[i][key])) throw new Error(`${r.id}: altered ${key}`);
  }
}
if (/[\u00b7\u2027\u2219\u30fb]/u.test(JSON.stringify(data))) throw new Error('Middle dots prohibited.');
const strict = r => ['missing','events'].includes(r.kind);
const filename = r => `qa-metric-${r.id.toLowerCase().replace('-','')}.png`;
function bands(r) {
  const b=r.bounds, s=r.symbol;
  if(r.kind==='high') return [
    [6,`${s} = ${b[0]}%`,true],
    ...b.slice(1).map((x,i)=>[5-i,`${x}% ≤ ${s} < ${b[i]}%`,true]),
    [0,`0% ≤ ${s} < ${b[5]}%`,false]
  ];
  if(r.kind==='low') return [
    [6,`0 ≤ ${s} ≤ ${b[0]}`,true],
    ...b.slice(1).map((x,i)=>[5-i,`${b[i]} < ${s} ≤ ${x}`,true]),
    [0,`${s} > ${b[5]}`,false]
  ];
  const variable=r.kind==='events'?s:'m';
  return b.map((x,i)=>[6-i,`${variable} = ${x}${r.unit}`,i===0]).concat([[0,`${variable} ≥ 6${r.unit}`,false]]);
}
const common = `## 공통 채점 규칙

이 자료는 기존 10개 QA 목표를 설명하는 발표용 제안이다. Architecture baseline이나 구현 및 측정 계약을 변경하지 않으며 아직 동결 또는 실측하지 않았다.

1. 각 페이지의 숫자는 0~6점의 **7개 진단 등급**이다. 문헌이 검증한 효용 함수나 통계적 유의성의 척도가 아니다. QA 사이에서 점수 크기를 비교하거나 평균 및 가중합으로 후보를 선정하지 않는다.
2. 개선형 지표는 **1~6점이 목표 충족**, 0점이 미달이다. 완전 지원, 전수 분석과 기밀성의 엄격 계약은 **6점만 목표 충족**이다. 0~5점은 모두 미달이며 실패 규모를 압축해 표시할 뿐이다. 보안 미달은 다른 QA로 상쇄할 수 없다.
3. 개선형의 기존 목표와 상향 참조값 사이를 5등분해 여섯 통과 등급을 만든다. 상향 참조는 구현 가능성이 입증된 최저값이 아니다. 엄격 계약은 한 개 또는 한 건을 자연스러운 진단 단위로 사용한다. 6개 이상을 0점으로 묶어도 실제 누락 수와 사건 수 및 ID는 반드시 병기한다.
4. 반올림 전 합계와 분모로 채점한다. 비율과 평균은 분수로 비교하고 메모리는 bytes로 비교한다. 경계의 ≤, <, =를 그대로 적용하며 표시에 사용한 반올림 숫자로 판정하지 않는다. 음수 시간이나 범위를 벗어난 비율은 유효한 측정값이 아니다.
5. **실패와 측정 미실시를 구별한다.** V-01은 독립 harness가 실행과 timeout을 확인하면 미전달도 실패로 분모에 남긴다. V-03은 지원 심사를 수행해 미지원이나 실행 증거 부족을 확인한 변형을 누락으로 센다. V-09는 유효하게 수행한 분석에서 로그 유실, 오답과 판단 불가 및 5분 내 미제출을 모두 미확인 실패로 센다. 이 실패들은 채점에서 빠지지 않는다. 반면 V-04/05/07의 실제 종료 endpoint가 없거나 V-08 과제가 미완료이면 **UNMEASURED / 채점 보류**다. 전체 registry 자체를 확인할 수 없거나 심사 및 분석 세션 자체가 미실시 또는 판정 기록 소실이면 해당 평가의 수치를 보류한다. V-10은 필수 sink를 관측할 수 없으면 정확한 사건 수의 채점과 0건 충족을 보류하되 이미 관측한 위반은 공개하고 목표 미달로 남긴다. UNMEASURED는 0점과 다르다. 시간 실패를 무한대로 바꾸지 않으며 관측 종료까지의 실제 경과시간과 미전달 또는 미복구 표식을 보고한다. 빠른 오답의 시간은 그대로 두고 V-01에서 실패로 판정한다.
6. 후보별 같은 원래 요구, 입력, 출력 계약, 장치, 모델 inventory와 외부 event schedule을 사용한다. [기존 측정 설계](quality-attributes/measurement-design.md)와 [시험 계획](quality-attributes/evaluation-plan.json)을 그대로 따른다. DP별 모집단은 같은 DP 안에서만 비교하고 통합 모집단과 섞지 않는다.
7. 실제 결과에는 원값, 합계와 N, 목표 충족 여부, 등급, 실패 ID와 반복 변동을 함께 공개한다. 비율과 평균의 불확실성은 같은 concrete case의 반복을 군집으로 보존한 bootstrap 95% 구간으로 보고할 계획이며 이는 시간 지표를 p95로 바꾸는 것이 아니다. 경계 여러 개를 가로지르는 구간은 **등급 차이 불확실**로 표시한다. 서로 다른 등급이 곧 구조의 유의한 우열이라는 주장은 하지 않는다. 메모리 최대값은 3개 세션 각각의 최대와 polling 및 allocation 계측 오차를 공개한다. 기밀성은 사건 수와 관측 범위를 보고하며 0건으로 모집단 전체의 안전을 보증하지 않는다.
8. U_min, 실제 model config, hash, fixture, 각 책임 ID의 대응표와 분석 registry의 N은 아직 채울 항목이 있다. 후보 결과를 보기 전에 모두 동결하고 물리 한도 및 상향 참조의 실현 가능성을 확인한다. 변경이 필요하면 사유와 이전 generation을 보존하며 결과에 맞춰 경계를 옮기지 않는다.

## 자료 구성

레퍼런스의 상단 요약표와 왼쪽 근거 및 계산, 오른쪽 7단계 표 구조를 16:9로 재구성했다. 장식용 그림 대신 계산표 자체를 편집 가능한 PowerPoint 표로 제공한다. 출처의 사실, VIA의 판단과 작성자 등급 규칙을 구분하고 상세 측정 절차는 발표자 노트에도 수록했다.
`;
const md = '# VIA QA 목표 근거와 7단계 채점 기준\n\n'+
`> ${data.date} / 발표용 제안 / 동결 및 실측 결과 없음\n\n`+
'[편집 가능한 PowerPoint](quality-metrics/VIA-quality-metrics.pptx) | [전체 이미지 미리보기](quality-metrics/index.html) | [기존 QA 요약](quality-attributes.md) | [문안과 채점 데이터](quality-metrics/quality-metrics.json)\n\n'+common+'\n'+
data.rows.map(r=>`## ${r.id} ${r.name}\n\n**설명:** ${r.description}\n\n**단일 목표:** ${r.target}\n\n`+
`### 목표를 정한 근거\n\n${r.fact.replaceAll('\n',' ')}\n\n${r.choice.replaceAll('\n',' ')}\n\n${r.detail}\n\n`+
`### 계산과 측정\n\n${r.calculation}\n\n| 항목 | 기준 |\n| --- | --- |\n`+r.calculationRows.map(c=>`| ${c[0]} | ${c[1]} |`).join('\n')+'\n\n'+
`### 7단계 구간\n\n${r.scoringBasis}\n\n`+
(strict(r)?'**6점만 목표 충족이며 나머지는 모두 미달 진단이다.**\n\n':'**1~6점은 목표 충족이며 0점은 미달이다.**\n\n')+
`범위의 단위: ${r.unit}. ${r.kind==='missing'?'m은 원래 비율과 동치인 미지원 또는 미확인 개수다. ':''}실제 결과와 점수는 아직 없다.\n\n`+
'| 점수 | 정확한 범위 | 목표 충족 |\n| --- | --- | --- |\n'+bands(r).map(([score,range,pass])=>`| ${score} | ${range} | ${pass?'충족':'미달'} |`).join('\n')+'\n\n'+
`### 출처와 상세 절차\n\n`+r.sources.map(id=>`- ${id}: [${data.references[id][0]}](${data.references[id][1]})`).join('\n')+
'\n\n상세 시험 조건과 원래 지표의 endpoint, 실패 및 제외 규칙은 [측정 설계](quality-attributes/measurement-design.md)의 해당 V 절을 따른다.\n\n'+
`![${r.id} 목표 근거와 채점표](quality-metrics/${filename(r)})\n`).join('\n');
await fs.writeFile(path.join(repo,'docs/presentations_files/quality-metrics.md'),md);

const { finalizePresentation } = await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const font='Apple SD Gothic Neo', ink='#202B35', blue='#2873B9', red='#AC353A';
const p=Presentation.create({slideSize:{width:1920,height:1080}});
function text(slide,name,value,x,y,w,h,size,color=ink,bold=false) {
  const shape=slide.shapes.add({name,geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
  shape.text=value;
  shape.text.style={typeface:font,fontSize:size,color,bold,wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
  return shape;
}
function table(slide,values,x,y,width,widths,heights,size=27,headerFill=ink) {
  const t=slide.tables.add({rows:values.length,columns:widths.length,left:x,top:y,width,height:heights.reduce((a,b)=>a+b,0),columnWidths:widths,values});
  t.styleOptions={headerRow:false,bandedRows:false};
  t.borders.assign({style:'solid',fill:'#D4DBE3',width:1.2});
  for(let row=0;row<values.length;row++) {
    t.rows[row].height=heights[row];
    for(let col=0;col<widths.length;col++) t.cells.block({row,column:col,rowCount:1,columnCount:1}).assign({fill:row===0?headerFill:row%2?'#F0F4F8':'#F8FAFC',margins:{left:14,right:12,top:3,bottom:3},anchor:'center',textStyle:{typeface:font,fontSize:size,bold:row===0,color:row===0?'#FFFFFF':ink,alignment:'left'}});
  }
  return t;
}
const categories={'V-01':'기능 적합성','V-02':'기능 적합성','V-03':'기능 적합성','V-04':'성능 효율성','V-05':'성능 효율성','V-08':'유지보수성','V-06':'성능 효율성','V-07':'신뢰성','V-09':'유지보수성','V-10':'보안성'};
const shortSources={S00:'VIA 고정 UC 계약',S01:'NexusRaven-13B 공식 README',S02:'tau-bench (2024)',S03:'CoA (2026 preprint)',S04:'Nielsen (2019)',S05:'SEI Architectural Tactics (2003)',S06:'Samsung NP960UJH-XG3IN',S07:'AWQ (2023)',S08:'Microreboot (2004)',S09:'OpenTelemetry SDK 문서',S10:'NIST SP 800-53 Rev.5',S11:'BFCL v3 (2024)',S12:'Qwen3-14B config'};
const strictLabels={'V-03':'미지원 변형 수 (개)','V-09':'미확인 시험 수 (개)','V-10':'위반 사건 수 (건)'};
for(const [index,r] of data.rows.entries()) {
  const slide=p.slides.add();slide.background.fill='#FFFFFF';
  text(slide,'title',`QA Metric 기준  |  ${r.id} ${r.name}`,48,30,1650,80,58,ink,true);
  text(slide,'page',`${index+1} / 10`,1760,55,110,40,27,'#647484');
  const summary=table(slide,[['분류','QA명','설명 / 단일 목표'],[categories[r.id],r.name,[
    {runs:[{run:r.description,textStyle:{typeface:font,fontSize:'30px',color:ink}}]},
    {runs:[{run:r.target,textStyle:{typeface:font,fontSize:'33px',color:blue,bold:true}}]},
    {runs:[{run:'측정: '+r.calculation,textStyle:{typeface:font,fontSize:'25px',color:'#566574'}}]},
  ]]],48,132,1824,[240,310,1274],[46,130],28);
  for(let row=0;row<2;row++) for(let col=0;col<2;col++) summary.cells.block({row,column:col,rowCount:1,columnCount:1}).assign({textStyle:{typeface:font,fontSize:28,bold:row===0,color:row===0?'#FFFFFF':ink,alignment:'center'}});
  text(slide,'basis-title','목표값의 근거',48,351,1120,43,34,ink,true);
  text(slide,'score-title','7단계 채점표 (0~6점)',1220,351,652,43,33,blue,true);
  text(slide,'external-fact',r.fact,48,407,1120,72,31,ink);
  text(slide,'via-choice',r.choice,48,489,1120,72,31,ink);
  text(slide,'calculation-title','계산과 측정 기준',48,579,1120,33,28,blue,true);
  table(slide,[['항목','계산 또는 판정 기준'],...r.calculationRows],48,620,1120,[265,855],[42,...r.calculationRows.map(()=>44)],25,blue);
  text(slide,'method-detail',r.slideMethod,48,941,1120,66,26,'#465464');
  const rubric=bands(r);
  const scoring=table(slide,[['점수',strictLabels[r.id]??'범위 ('+r.unit+')','목표'],...rubric.map(([s,range,pass])=>[String(s),range,pass?'충족':'미달'])],1220,407,652,[80,432,140],[51,...rubric.map(()=>62)],28,blue);
  for(const [i,[score,,pass]] of rubric.entries()) {
    for(let col=0;col<3;col++) {
      scoring.cells.block({row:i+1,column:col,rowCount:1,columnCount:1}).assign({fill:pass?(score===(strict(r)?6:1)?'#D9EAF8':'#EDF4FA'):'#FAEBEC',textStyle:{typeface:font,fontSize:col===1?27:29,bold:col!==1||score===(strict(r)?6:1),color:pass?'#1E507D':red,alignment:col===1?'left':'center'}});
    }
  }
  const threeLines=r.slideNote.split('\n').length===3;
  text(slide,'scoring-caution',r.slideNote,1220,threeLines?918:926,652,threeLines?87:72,threeLines?24:27,strict(r)?red:'#465464',strict(r));
  text(slide,'source-footer','출처  '+r.sources.map(s=>shortSources[s]).join(' / ')+'  |  전체 링크와 상세 절차: 발표자 노트',48,1016,1824,27,21,'#536575');
  text(slide,'status-footer','발표용 채점 제안. 등급 간격은 작성자 규칙이며 실측 결과 없음. QA 간 점수의 평균 또는 가중합을 사용하지 않음.',48,1049,1824,26,21,'#536575');
  const chapters=methods.split(/(?=^## \d+\. )/m);
  const chapter=chapters.find(c=>c.startsWith('## ')&&c.split('\n')[0].includes(r.id));
  const conditions=chapters.find(c=>c.startsWith('## 2. '));
  if(!chapter || !conditions) throw new Error(`${r.id}: detailed measurement chapter missing`);
  slide.speakerNotes.textFrame.setText(`${r.id} ${r.name}\n${r.description}\n목표: ${r.target}\n\n${r.fact.replaceAll('\n',' ')}\n${r.choice.replaceAll('\n',' ')}\n${r.detail}\n\n등급 설정 근거\n${r.scoringBasis}\n\n`+rubric.map(([s,range,pass])=>`${s}점: ${range}, 목표 ${pass?'충족':'미달'}`).join('\n')+`\n\n공통 채점 규칙\n${common}\n\n출처\n`+r.sources.map(s=>`${s}: ${data.references[s][0]} ${data.references[s][1]}`).join('\n')+`\n\n공통 시험 조건\n${conditions}\n\n상세 측정 절차\n${chapter}`);
}
const candidate=path.join(build,'candidate.pptx'), final=path.join(build,'final','VIA-quality-metrics.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await (await PresentationFile.exportPptx(p)).save(candidate);
const owners=Array.from({length:10},(_,i)=>i+1);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:10,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit','--validate-bullet-geometry',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
await fs.copyFile(final,path.join(out,'VIA-quality-metrics.pptx'));
const loaded=await PresentationFile.importPptx(await FileBlob.load(final));
for(const [i,slide] of loaded.slides.items.entries()) {
  const png=await loaded.export({slide,format:'png',scale:1});
  await fs.writeFile(path.join(out,filename(data.rows[i])),new Uint8Array(await png.arrayBuffer()));
}
await fs.writeFile(path.join(out,'index.html'),'<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA QA Metric</title><style>body{margin:24px;background:#eaf0f3;color:#202b35;font-family:system-ui}main{max-width:1600px;margin:auto}img{width:100%;display:block;margin:18px 0;box-shadow:0 2px 12px #0002}a{color:#2873b9}p{line-height:1.6}</style><main><h1>VIA QA 목표 근거와 7단계 채점 기준</h1><p>10개 QA, 각 1장. 발표용 제안이며 실측 결과가 아닙니다.</p><p><a href="VIA-quality-metrics.pptx">편집 가능한 PowerPoint</a> / <a href="../quality-metrics.md">전체 근거와 채점 규칙</a></p>'+data.rows.map(r=>`<section id="${r.id}"><h2>${r.id} ${r.name}</h2><a href="${filename(r)}">PNG 원본</a><img src="${filename(r)}" alt="${r.id} ${r.name} 목표 근거와 채점표"></section>`).join('')+'</main></html>');
console.log(JSON.stringify({slides:10,nativeTables:30,final,path:out},null,2));
