import fs from 'node:fs/promises';
import path from 'node:path';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

// Run a copy from a private build directory with bundled node_modules linked.
const [repo, build] = process.argv.slice(2);
const data = JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/qa-metrics-v11/qa-pages.json'),'utf8'));
const p = Presentation.create({slideSize:{width:1920,height:1080}});
const GREEN='#C6E0B4', CREAM='#FFF8E4', BLUE='#3379B8';
function text(s,name,value,x,y,w,h,pt=20,bold=false,color='#000000'){
  const sh=s.shapes.add({name,geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
  sh.text=value;sh.text.style={typeface:'Calibri',fontSize:pt*4/3,bold,color,autoFit:'none',wrap:'word',verticalAlignment:'middle',insets:0};return sh;
}
function table(s,name,values,x,y,widths,heights,pt=18){
  const t=s.tables.add({rows:values.length,columns:widths.length,left:x,top:y,width:widths.reduce((a,b)=>a+b),height:heights.reduce((a,b)=>a+b),columnWidths:widths,values});
  t.name=name;t.borders.assign({style:'solid',fill:'#666666',width:1});
  for(let r=0;r<values.length;r++){
    t.rows[r].height=heights[r];
    for(let c=0;c<widths.length;c++){
      const cell=t.getCell(r,c);cell.fill=r===0?GREEN:'#FFFFFF';
      cell.text.style={typeface:'Calibri',fontSize:pt*4/3,bold:r===0,color:'#000000',alignment:'center',verticalAlignment:'middle',autoFit:'none',insets:{left:12,right:12,top:5,bottom:5}};
    }
  }return t;
}
function scenario(q){return q.summary.split('\n').map((line,i)=>({runs:[{run:line,style:{typeface:'Calibri',fontSize:(i===1?'19pt':i===0?'17pt':'15pt'),bold:i<2,color:i===1?BLUE:'#000000'}}],spaceAfter:3}));}
function notes(s,q){
  const base='https://github.com/cbyche/via-sw-architecture/blob/main/docs/architecture/12-decisions/decision-packages/qa-candidate-package/';
  const md=path.join(repo,q.source);
  return fs.readFile(md,'utf8').then(async raw=>{
    const plan=await fs.readFile(path.join(repo,'docs/architecture/12-decisions/decision-packages/qa-candidate-package/slide-page-plan.md'),'utf8');
    const citation=[...plan.matchAll(/\*\*출처 각주:\*\* (.+)/g)][Number(q.id.slice(2))-1][1];
    const urls=[...new Set([...raw.matchAll(/https?:\/\/[^\s)<>]+/g)].map(m=>m[0]))];
    s.speakerNotes.textFrame.setText(q.summary+'\n\n'+q.why+'\n\n'+q.formula+'\n'+q.guard+'\n\n'+q.result+'\n계산 예시는 실측 결과가 아니다.\n\n'+q.band+'\n\n'+q.foot+'\n\n'+q.notes.replace(/\]\(\.\//g,']('+base)+'\n\n상세 정의: https://github.com/cbyche/via-sw-architecture/blob/main/'+q.source+'\n목표 출처: '+citation.replace(/\]\(\.\//g,']('+base)+'\n원문 출처:\n'+urls.join('\n'));
  });
}
for(const q of data){
  const s=p.slides.add();s.background.fill='#FFFFFF';
  const top=table(s,q.id+'-scenario',[['분류','Title','Scenario'],[q.category,q.title,scenario(q)]],48,132,[230,290,1304],[44,168],20);
  top.getCell(1,1).text.style={bold:true};top.getCell(1,2).text.style={alignment:'left'};top.getCell(1,2).value=scenario(q);
  text(s,q.id+'-rationale-label','목표 수치의 근거',55,366,1100,40,22,true);
  text(s,q.id+'-rationale',q.why,55,410,1110,114,20);
  text(s,q.id+'-metric-label','측정 지표와 계산 예시',55,537,1110,40,22,true);
  text(s,q.id+'-formula',q.formula,55,581,1110,72,20,true);
  table(s,q.id+'-example',q.example,55,669,q.exampleWidths,q.example.map((_,i)=>i===0?44:46),18);
  text(s,q.id+'-example-result',q.result,55,827,1110,78,20,true);
  text(s,q.id+'-guard',q.guard,55,913,1110,66,17);
  text(s,q.id+'-dp',q.dp,55,989,1110,34,16,true);
  text(s,q.id+'-scoring-title','0~6점 평가 기준',1224,366,648,40,22,true);
  const scores=q.scores.map(([n,g,v])=>[n,g,v]);
  // Compact visible infinity rule; full failure rule remains in the notes.
  if(q.id==='QA09')scores[6][2]='x > 500 또는 미출력';
  const st=table(s,q.id+'-score',[['점수','기호',q.unit?'기준 ('+q.unit+')':'기준'],...scores],1224,417,[80,170,398],Array(8).fill(59),18);
  for(let c=0;c<3;c++)st.getCell(6,c).fill=CREAM;
  text(s,q.id+'-band',q.band,1224,917,648,99,18);
  text(s,q.id+'-source',q.foot+'  예시는 계산 설명용.',55,1031,1815,28,13);
  await notes(s,q);
}
// Main presentation pages 9 and 10: update current core QA names and targets.
for(const start of [0,3]){
  const s=p.slides.add();s.background.fill='#FFFFFF';
  text(s,'qa-summary-subtitle','공통 28개 시나리오로 같은 기능을 평가하고 QA별 대상 사건을 집계',55,133,1815,46,24,true);
  const subset=data.slice(start,start+3);
  const vals=[['ID','분류','Title','Scenario','중요도 / 난이도'],...subset.map(q=>[q.id,q.category,q.title,scenario(q),q.importance])];
  const t=table(s,'current-quality-summary',vals,48,200,[110,215,280,1034,185],[52,224,224,224],19);
  for(let r=1;r<=3;r++){
    for(let c=0;c<5;c++)t.getCell(r,c).fill=CREAM;
    t.getCell(r,3).text.style={alignment:'left'};t.getCell(r,3).value=scenario(subset[r-1]);
    t.getCell(r,2).text.style={bold:true};
  }
  text(s,'qa-summary-common','기본 시나리오 S01~S28 / 같은 입력과 외부 조건 / 각 5회 반복 / 세부 정의와 점수는 Appendix',55,935,1815,44,20,true);
  text(s,'qa-summary-note','QA04는 공통 기능에서 파생한 변경 6건을 평가한다. QA07~QA09는 추가 후보이며 별도 상세 페이지에 정리한다.',55,989,1815,45,18);
  s.speakerNotes.textFrame.setText(subset.map(q=>q.summary+'\n출처: '+q.source).join('\n\n')+'\n\n본 표는 최신 qa-candidate-package의 정의를 옮겼다. 실측 결과와 최종 DP 선정은 없다.');
}
// Main page 11: make three different response endpoints explicit.
{
  const s=p.slides.add();s.background.fill='#FFFFFF';
  text(s,'response-subtitle','첫 안내와 VIA 처리 완료 및 음성 전달을 서로 다른 시점에서 평가',55,134,1815,48,24,true);
  const vals=[['지표','시작과 종료','포함하는 대기','목표'],
    ['QA01\n첫 응답 지연시간','실제 발화 종료 / Text 제출 / Agent source 제공\n첫 완결된 내용의 실제 UI 표시까지','VAD와 전사 및 polling 대기\n오답도 실제 시간을 포함하고 오류는 별도 판정','평균 1초'],
    ['QA03\nVIA 처리시간','실제 사건부터 목표 완료까지의 실행 경로\n외부 대기를 0으로 바꾼 경로 길이','모델과 내부 대기 및 출력 준비\n오답도 실제 시간을 포함하고 오류는 별도 판정','평균 5초'],
    ['QA09\n음성 전달 지연','첫 응답 내용 채택부터 실제 첫 요약 음성까지\n들을 수 없는 차례의 순수 대기는 제외','생성과 준비 및 VIA 출력 대기\nVIA가 자의적으로 미룬 시간도 포함','평균 500 ms']];
  table(s,'three-time-boundaries',vals,48,214,[330,740,520,234],[56,160,160,160],21);
  text(s,'response-example-heading','예: 구체적인 수행 안내를 Agent 위임 전에 보여주는 경우',55,799,1815,44,23,true);
  table(s,'response-timeline',[['0.0초','0.1초','0.8초','1.3초','2.5초'],['발화 종료','“네” 표시','“표와 그래프를\n비교하겠습니다”','Agent 전송','음성 요약 재생']],55,858,[260,260,620,320,355],[45,78],20);
  text(s,'response-example-result','대상과 수행 범위를 실제로 채택했다면 QA01은 0.8초다. Agent 전송과 실제 음성 재생은 첫 UI 응답의 종료점이 아니다.',55,1002,1815,42,19,true);
  s.speakerNotes.textFrame.setText('QA01/QA03/QA09는 서로 다른 목적과 endpoint를 가진다.\n'+data.filter(q=>['QA01','QA03','QA09'].includes(q.id)).map(q=>q.summary+'\n'+q.guard+'\n'+q.notes).join('\n\n')+'\n계산 예시는 실측이 아니다.');
}
await(await PresentationFile.exportPptx(p)).save(path.join(build,'qa-authored.pptx'));
console.log(JSON.stringify({authoredSlides:p.slides.items.length,qaCount:data.length}));
