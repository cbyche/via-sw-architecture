import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [repoArg,buildArg,kindArg]=process.argv.slice(2);
const kind=kindArg??process.env.VIA_QA_DECK_KIND;
if(!repoArg||!buildArg||!['attributes','metrics'].includes(kind))throw new Error('Usage: REPO BUILD_DIR attributes|metrics');
const repo=await fs.realpath(repoArg),build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR,python=process.env.VIA_RUNTIME_PYTHON;
if(!path.isAbsolute(skill??'')||!path.isAbsolute(python??''))throw new Error('Absolute runtime skill/Python paths required');
const out=path.join(repo,`docs/presentations_files/quality-${kind}`);
const data=JSON.parse(await fs.readFile(path.join(out,`quality-${kind}.json`),'utf8'));
const source=await fs.readFile(path.join(repo,data.source),'utf8');
if(crypto.createHash('sha256').update(source).digest('hex')!==data.source_sha256)throw new Error('Stale projection of authoritative QA document');
if(crypto.createHash('sha256').update(await fs.readFile(path.join(repo,'docs/architecture/12-decisions/decision-packages/03-02-common-workload.json'))).digest('hex')!==data.common_workload_sha256)throw new Error('Stale shared workload');
if(data.rows.length!==6||data.rows.some((r,i)=>r.id!==`ASR-QA-0${i+1}`||r.priority!==i+1))throw new Error('Six ordered ASR IDs required');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const font='Apple SD Gothic Neo',symbolsFont='Arial Unicode MS';
const ink='#222A35',blue='#4472C4',cyan='#43AACC',muted='#56606B',red='#AF2E33';
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const symbols=['○○○','◐○○','●○○','●◐○','●●○','●●◐','●●●'];
function text(slide,name,value,x,y,w,h,size,color=ink,bold=false){
 const sh=slide.shapes.add({name,geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
 sh.text=value;sh.text.style={typeface:font,fontSize:size,color,bold,wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};return sh;
}
function table(slide,values,x,y,width,widths,heights,size=27,headerFill=ink,cream=false){
 const t=slide.tables.add({rows:values.length,columns:widths.length,left:x,top:y,width,height:heights.reduce((a,b)=>a+b,0),columnWidths:widths,values});
 t.styleOptions={headerRow:false,bandedRows:false};t.borders.assign({style:'solid',fill:cream?blue:'#FFFFFF',width:cream?1.3:1.4});
 for(let r=0;r<values.length;r++){
  t.rows[r].height=heights[r];
  for(let c=0;c<widths.length;c++)t.cells.block({row:r,column:c,rowCount:1,columnCount:1}).assign({fill:r===0?headerFill:cream?'#FFF6DE':r%2?'#E8ECF5':'#F3F5FA',margins:{left:14,right:12,top:6,bottom:6},anchor:'center',textStyle:{typeface:font,fontSize:size,bold:r===0,color:r===0?'#FFFFFF':ink,alignment:'left'}});
 }return t;
}
function paragraph(run,size=27,color=ink,bold=false){return {runs:[{run,textStyle:{typeface:font,fontSize:`${size}px`,color,bold}}]};}
function header(slide,title,page,total){
 slide.background.fill='#FFFFFF';text(slide,'title',title,48,26,1650,84,59,ink,true);
 const rule=slide.shapes.add({name:'header-rule',geometry:'rect',position:{left:48,top:112,width:1824,height:2},fill:'#9E9E9E',line:{fill:'none',width:0}});
 text(slide,'page',`${page} / ${total}`,1744,53,128,35,25,muted);
}
function notes(slide,rows){
 slide.speakerNotes.textFrame.setText(`원본: ${data.source}\n계약: ${data.contract_version}\n공통 활동: P21 사용자 19/Agent 단독 2장면, 내부 Agent 4사건, 5회 독립 반복\n활동 SHA-256: ${data.common_workload_sha256}\n작성일: ${data.date}\n원본 SHA-256: ${data.source_sha256}\n우선순위: ASR-QA-01부터 번호순. 목표는 작성자 PoC 예산이며 실제 제품/모델/Windows 성능 결과가 아님. 제공한 PNG 레퍼런스의 스타일만 적용하고 타 과제의 요구/수치는 가져오지 않음.\n\n`+rows.map(r=>`${r.id} ${r.name}\n정의: ${r.description}\n측정 지표: ${r.method}\n목표: ${r.target}\n목표 근거: ${r.targetBasis}\n보호 조건: ${r.guards}\n출처:\n`+r.sources.map(k=>`${data.references[k].title}: ${data.references[k].url}`).join('\n')+`\n\n상세 평가 원문 §${r.definitionSection}\n${r.detail}\n`).join('\n')+`\n전체 사례의 QA 대응과 선정 근거\n${JSON.stringify(data.case_mapping,null,2)}\n공통 활동 상세 §8.0\n${data.common_workload_detail}\n모델 reference 상세 §8.4\n${data.model_reference_detail}\n`+'\n등급은 원지표를 표시하는 작성자 규칙이며 QA 간 합산하지 않음. 측정 전에는 점수를 생성하지 않음.');
}
const wrapNames={'ASR-QA-01':'요청 처리\n정확성','ASR-QA-02':'상호작용\n응답시간','ASR-QA-03':'VIA 요청\n처리시간','ASR-QA-04':'VIA 모델\n사용 비용','ASR-QA-05':'VIA 변경\n용이성','ASR-QA-06':'로컬 VIA\n메모리 사용량'};
function ranges(row){
 const b=row.score.bounds.map(Number),unit=row.score.unit;
 if(row.score.direction==='high')return [{score:6,range:'정확히 100%'},...b.slice(1).map((v,i)=>({score:5-i,range:`${v}% ≤ x < ${b[i]}%`})),{score:0,range:'0% ≤ x < 98%'}];
 const fmt=v=>row.id==='ASR-QA-06'?`${v/1024}GiB`:row.id==='ASR-QA-04'?`$${v.toFixed(2)}`:`${v}${unit==='개/과제'?'개':unit}`;
 return [{score:6,range:`0 ≤ x ≤ ${fmt(b[0])}`},...b.slice(1).map((v,i)=>({score:5-i,range:`${fmt(b[i])} < x ≤ ${fmt(v)}`})),{score:0,range:`x > ${fmt(b[5])}`}];
}
if(kind==='attributes'){
 for(const [index,group] of [data.rows.slice(0,3),data.rows.slice(3)].entries()){
  const slide=p.slides.add();header(slide,'품질 요구사항',index+1,6);
  text(slide,'headline',index===0?'핵심 품질속성: 정확성과 응답시간':'핵심 품질속성: 비용, 변경 용이성과 메모리',48,135,1800,51,42,blue,true);
  text(slide,'scope','같은 P21: 21개 사례를 5회 독립 반복. 번호순 우선 선택, 낮은 순위 약점은 tactic으로 보완',48,191,1800,34,28,ink);
  const values=[['ID','분류','Title','Scenario / 지표 / 목표 근거','우선순위'],...group.map(r=>[r.id,r.category.replace(' ','\n'),wrapNames[r.id],[paragraph(r.description,28),paragraph(r.displayTarget,31,cyan,true),paragraph('측정: '+r.shortMetric,26,ink),paragraph('근거: '+r.shortBasis,26,muted)],String(r.priority)])];
  const t=table(slide,values,48,240,1824,[155,215,290,1034,130],[50,245,245,245],28,blue,true);
  for(let row=0;row<values.length;row++)for(const col of [0,1,2,4])t.cells.block({row,column:col,rowCount:1,columnCount:1}).assign({textStyle:{typeface:font,fontSize:row===0?27:col===0?25:col===2?34:col===4?36:28,bold:row===0||col===2||col===4,color:row===0?'#FFFFFF':col===4?red:ink,alignment:'center'}});
  text(slide,'footer','P21: 정확성 630조건 / 반응 135관측 / 처리 95사이클 / 비용·메모리도 같은 5회 전체 활동',48,1040,1824,30,22,muted);
  notes(slide,group);
 }
 const slide=p.slides.add();header(slide,'공통 평가 활동',3,6);
 text(slide,'headline','P21: 모든 QA가 같은 사례와 후속 사건을 평가',48,135,1824,51,42,blue,true);
 text(slide,'scope','DP-41/42/44/45의 실제 기제와 오류 경로로 선정한 사례를 동일 조건에서 평가',48,191,1824,36,29,ink);
 const values=[['활동 / 관측','고정 조건','QA에서 읽는 값'],
 ['사례 선정','S1~S6: 2 / 5 / 3 / 3 / 4 / 4개 = 21개\n기존 17 유지, 선호 등록은 회귀, 민감 사례 4개 추가','DP-41/42/44/45의 실제 기제 경로\n공통 기준 경로와 차이를 활성화하는 경로 포함'],
 ['같은 활동 5회','각 활동에서 21개를 한 번씩, 전체 5회 독립 초기화\nS2-02 → S2-04 실제 재사용의 선후는 고정','사용자 목표 19 + Agent 단독 2장면\n네 Agent 동시 사건은 S5-04 내부에 포함'],
 ['모든 정확성과 시간','정확성: 모든 21사례의 6조건 × 5회\n시간: 같은 사례의 첫 반응과 전체 완료','정확성 105시나리오 / 630조건\n반응 27×5=135 / 처리 19×5=95'],
 ['같은 비용과 메모리','P21 전체 5회: 비용 평균 / 메모리 최대\n음성 입력 50초 / 출력 179초 / 의미 62호출 예산','비용 ≤$0.50/P21 / commit ≤4GiB\n실제 호출·청구·할당은 후보 경로의 결과'],
 ['초기 자료와 집중 부하','과거 2,000turn / 완료 Task 200 / 원본 30개\nTask 최대 10 / cloud 호출 최대 4','초기/미사용 생산, 재사용, 첫 사용, 집중 도착\n동일 원천·허용·외부 응답·모델/기능 조건'],
 ['회수와 보호','원본 갱신 4건 / maintenance 2회 / B04 철회\ndrain ≤30초 + idle 10초, 실제 실패는 보존','일반 ≥110/115 ≤2초 / 집중 30/30 ≤5초\nlocal stop 모두 ≤0.1초 / 유휴 증가 ≤16MiB']];
 table(slide,values,48,240,1824,[280,840,704],[50,122,122,122,142,122,106],27,blue,true);
 text(slide,'footer','03-02 §3.2 선정/대응표, §8 공통 실행 / 다음 3장은 21개 전체 대응표 / 작성자 설계 PoC, 미실측',48,1040,1824,30,22,muted);
 notes(slide,[]);
 const shortReaction={
 'S5-03':'결과 도착 → Text 전달 (1)\n재연결 발화 끝 → Voice 전달 (1)',
 'S5-04':'끼어들기 → local stop (1)\n정정 → 접수 (1), 질문 2 / 결과 2 → 제시 (4)'
 };
 for(let page=0;page<3;page++){
  const group=data.case_mapping.slice(page*7,page*7+7),sl=p.slides.add();header(sl,'공통 사례와 QA 대응',page+4,6);
  text(sl,'headline',`21개 전체 사례: ${page*7+1}~${page*7+7} / 21`,48,135,1824,51,42,blue,true);
  text(sl,'scope','모든 행: QA-01의 C1~C6 + QA-04 비용 + QA-06 메모리 / QA-02는 첫 반응, QA-03는 목표 완료',48,191,1824,36,27,ink);
  const vals=[['사례','주제 / 선정 이유','민감 DP','QA-02 유효 반응','QA-03 완료 / 적용 제외'],...group.map(c=>[c.case_id,c.name+'\n'+c.selection_reason,c.sensitive_dps.join('/')+'\n'+(c.selection_role==='CONTROL'?'공통 기준':'구조 민감'),shortReaction[c.case_id]??c.reactions.map(r=>r.start+' → '+r.end).join('\n'),c.cycle?c.cycle.end:'새 사용자 목표 없음\nQA-02로 도착→전달 관측'])];
  table(sl,vals,48,245,1824,[140,510,160,550,464],[55,...group.map(()=>99)],25,blue,true);
  text(sl,'footer','시작/종료의 전체 경계·초기 상태·Voice 입력·사건·정답 규칙: 03-02 §3.2 / 각 활동 내 사례 1회, 전체 5회',48,1040,1824,30,22,muted);
  notes(sl,[]);
 }
}else{
 for(const [index,r] of data.rows.entries()){
  const slide=p.slides.add();header(slide,`QA Metric (●◐○) 기준`,index+1,6);
  const summary=table(slide,[['분류','Title','Scenario'],[r.category,wrapNames[r.id],[paragraph(r.description,28),paragraph(r.displayTarget,31,cyan,true),paragraph('측정: '+r.shortMetric,25,muted)]]],48,132,1824,[240,330,1254],[49,169],28,ink);
  for(let row=0;row<2;row++)for(const col of [0,1])summary.cells.block({row,column:col,rowCount:1,columnCount:1}).assign({textStyle:{typeface:font,fontSize:row===0?29:col===1?33:28,bold:row===0,color:row===0?'#FFFFFF':ink,alignment:'center'},fill:row===0?ink:'#FFFFFF'});
  summary.cells.block({row:1,column:2,rowCount:1,columnCount:1}).assign({fill:'#FFFFFF'});
  text(slide,'qa-id',r.id,48,373,300,38,29,blue,true);
  text(slide,'basis-label','목표값의 근거',340,372,828,43,34,ink,true);
  text(slide,'fact',r.fact,48,435,1120,86,30,ink);
  text(slide,'choice',r.choice,48,533,1120,86,30,ink);
  text(slide,'calculation-title','계산과 평가 방법',48,636,1120,37,30,blue,true);
  const calculation=table(slide,[['항목','계산 또는 판정 기준'],...r.calculationRows],48,680,1120,[250,870],[42,...r.calculationRows.map(()=>43)],25,blue);
  calculation.cells.block({row:0,column:0,rowCount:7,columnCount:2}).assign({margins:{left:14,right:12,top:2,bottom:2}});
  text(slide,'method',r.slideMethod,48,992,1120,58,23,muted);
  text(slide,'score-label','7단계 등급과 목표 충족',1220,374,652,43,34,blue,true);
  const rubric=ranges(r);
  const scoretable=table(slide,[['표시','원지표 범위','목표'],...rubric.map(x=>[symbols[x.score],x.range,x.score>0?'충족':'미달'])],1220,435,652,[145,369,138],[54,...rubric.map(()=>69)],26,blue);
  for(const [j,b] of rubric.entries())for(let c=0;c<3;c++)scoretable.cells.block({row:j+1,column:c,rowCount:1,columnCount:1}).assign({fill:b.score===0?'#FAECEC':b.score===1?'#D9E9F8':'#E8ECF5',textStyle:{typeface:c===0?symbolsFont:font,fontSize:c===0?32:26,bold:c===2||b.score===1,color:b.score===0?red:ink,alignment:c===1?'left':'center'}});
  text(slide,'guard-note','보호 조건: '+r.guards+'\n대표 등급과 보호 조건을 함께 판정',1220,991,652,61,22,muted);
  const sourceNames=r.sources.filter(k=>k!=='contract').map(k=>data.references[k].title).join(' / ');
  text(slide,'source-footer',sourceNames+' / 목표와 등급은 작성자 PoC 규칙, 실측 전',48,1054,1824,23,20,muted);
  notes(slide,[r]);
 }
}
const total=6,owners=Array.from({length:total},(_,i)=>i+1);
const candidate=path.join(build,'candidate.pptx'),final=path.join(build,'final',`VIA-quality-${kind}.pptx`);
await fs.mkdir(path.dirname(final),{recursive:true});
await (await PresentationFile.exportPptx(p)).save(candidate);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:total,pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit','--validate-bullet-geometry',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'design',families:[font,...kind==='metrics'?[symbolsFont]:[]]},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
await fs.copyFile(final,path.join(out,`VIA-quality-${kind}.pptx`));
const loaded=await PresentationFile.importPptx(await FileBlob.load(final));
const imgs=[];
for(const [i,sl] of loaded.slides.items.entries()){
 const file=kind==='attributes'?`quality-attributes-0${i+1}.png`:`qa-metric-asr-qa-0${i+1}.png`;
 const png=await loaded.export({slide:sl,format:'png',scale:1});await fs.writeFile(path.join(out,file),new Uint8Array(await png.arrayBuffer()));imgs.push(file);
}
await fs.writeFile(path.join(out,'index.html'),`<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA ASR-QA ${kind}</title><style>body{margin:24px;background:#eff2f6;color:#222a35;font-family:system-ui}main{max-width:1600px;margin:auto}img{width:100%;display:block;margin:24px 0}a{color:#4472c4}</style><main><h1>VIA ASR-QA ${kind==='attributes'?'품질 요구사항':'목표 근거와 등급'}</h1><p>03-02의 여섯 QA, 번호순 우선순위. 작성자 평가안이며 실제 후보 성능은 아직 측정하지 않았습니다.</p><p><a href="VIA-quality-${kind}.pptx">편집 가능한 PowerPoint</a>, <a href="../quality-${kind}.md">문안과 원본 연결</a></p>${imgs.map((f,i)=>`<section><h2>${kind==='attributes'?`요약 ${i+1}`:data.rows[i].id+' '+data.rows[i].name}</h2><img src="${f}" alt="VIA ASR QA ${i+1}"></section>`).join('')}</main></html>`);
console.log(JSON.stringify({kind,slides:total,final:path.join(out,`VIA-quality-${kind}.pptx`),previews:imgs}));
