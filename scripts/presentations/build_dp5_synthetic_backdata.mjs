// SYNTHETIC illustrative backdata. No candidate/model/benchmark is executed.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {Workbook,SpreadsheetFile} from '@oai/artifact-tool';
const ROOT='/Users/changbae/Workspace/via-sw-architecture';
const OUT=path.join(ROOT,'docs/presentations_files/dp5-synthetic-backdata-20261021');
const DELIVER=path.join(OUT,'outputs/01a154e4');
await fs.mkdir(DELIVER,{recursive:true});
await fs.mkdir(path.join(OUT,'previews'),{recursive:true});
const provenance='설명용 합성 데이터. 목표 수치에 맞춰 작성한 사례·응답·시간·비용이며 실제 관측/LLM 실행/통계 표본이 아님.';
const projects=['새봄','해솔','가온','누리','다온','마루','이든','라온','한결','온유'];
const topics=['협업 도구 도입','고객지원 시스템 교체','물류 현황 보고','매장 운영 개선','교육 과정 개편','제품 출시 계획','연구 장비 도입','사무공간 이전','서비스 품질 개선','파트너 제휴 검토'];
const referentPhrases=['고객용 구성이 좋다고 했던 자료','지난번 고객 설명에 쓰기 좋다고 한 구성의 자료','고객 앞에서 보여주기 낫다고 평가했던 쪽','내가 고객용 구성을 칭찬했던 자료','고객 설명에 맞는다고 골랐던 구성의 자료','고객에게 보여주자고 했던 구성의 자료','고객용은 구성이 더 좋다고 했던 쪽','지난 피드백에서 고객용으로 선호한 구성의 자료','고객에게 설명하기 편하다고 했던 자료','고객용으로 채택하자고 말했던 구성의 자료'];
const families=[
 ['G01','명시적 ID 조회',.55,.65,1.2,1.2,'정확한 자료 ID와 확정 조건. 의미 관계를 새로 해석할 필요가 적다.','공통 사실 직접 조회','동일 사실 조회; 관계 검사 추가 이득 없음'],
 ['G02','오래된 피드백 최초 재참조',3.1,.85,1.8,1.1,'고객용 구성 칭찬과 여러 결과의 연결을 처음 다시 묻는다.','표시·피드백·자료 원본 조합','준비된 대상·측면·상황 관계 조회'],
 ['G03','비슷한 자료 중 선택',3.7,1.05,1.9,1.15,'고객용과 내부용 자료의 이름·내용이 비슷하지만 칭찬 대상은 하나다.','유사한 원본 사이 관계 판단','피드백과 판본의 연결 활용'],
 ['G04','실제 본 판본과 최신 판본',2.9,.9,1.8,1.1,'사용자는 D2를 본 뒤 평가했다. 뒤에 생성된 D3는 아직 보지 않았다.','열람·생성 시각을 구분','실제 표시 판본을 포함한 관계 사용'],
 ['G05','피드백 정정과 무효화',2.6,1.65,1.6,1.35,'F1의 선호를 F2가 D3로 정정했다. 과거 파생 관계는 dirty다.','정정 원문과 현재 판본 조회','무효 관계 제외 후 원본 보완; 갱신 비용 발생'],
 ['G06','대상 유지·사용 목적 변경',2.8,1.05,1.7,1.15,'고객용으로 좋았던 D2를 내부회의용으로 바꾸고 단점부터 보여달라 한다.','현재 목적과 원본 관계를 함께 해석','대상은 재사용하되 새 목적 적용에 오류 가능'],
 ['G07','평가 측면·선호 범위 제한',2.7,.95,1.6,1.15,'구성만 좋았고 가격·문체는 평가하지 않았다. 과거 P 위임은 이번 업무 범위다.','범위 제한을 원문에서 판단','관계의 측면·상황을 활용; 전체 선호로 확대 위험'],
 ['G08','이미 확정한 관계 재사용',.7,.8,1.2,1.2,'G02에서 확인된 동일 관계를 다시 쓴다. A에도 확정 관계가 제공된다.','확정 사실 재사용','기존 관계 재사용; 큰 추가 이득 없음'],
 ['G09','방금 받은 사건·미생산 관계',2.45,2.15,1.6,1.5,'새 피드백은 원본에 있으나 B의 처리 cursor 뒤에 있어 파생 관계가 없다.','새 원본 해석','미확인 범위를 검출하고 같은 원본 해석'],
 ['G10','형식과 내용 출처 분리',2.5,.95,1.6,1.1,'D2의 구성만 재사용하고 내용은 신규 조사 N으로 바꾸며 가격을 제외한다.','형식·내용·제외 조건의 관계 구성','형식 관계 재사용; 과거 내용까지 가져올 위험']
].map(([id,name,a,b,pa,pb,background,apath,bpath])=>({id,name,a,b,pa,pb,background,apath,bpath}));
const refFails={A:{G02:[1,6],G03:[1,8],G04:[2,9],G05:[3],G07:[2]},B:{G03:[8],G04:[9],G05:[3],G09:[4]}};
const meaningFails={A:{G05:[8],G07:[8]},B:{G05:[7],G06:[0,3,6,9],G07:[1,4,7],G09:[8],G10:[6]}};
const cases=[],responses=[],times=[],sources=[],costs=[];
const r3=n=>Math.round(n*1000)/1000;
for(const f of families)for(let j=0;j<10;j++){
 const p=projects[j],id=`SYN-${f.id}-${String(j+1).padStart(2,'0')}`,sg=f.id==='G08'?'G02':f.id;
 const base=`P${j+1}-${sg}`,d2=`${base}-D2@v1`,d3=`${base}-D3@v1`,fresh=`${base}-N@v1`;
 const target=f.id==='G05'?d3:d2,alternative=f.id==='G05'?d2:d3;
 const title2=`${p} ${topics[j]} 고객 제안서`,title3=`${p} ${topics[j]} 내부 검토본`;
 const title=f.id==='G05'?title3:title2,wrongTitle=f.id==='G05'?title2:title3;
 const expression=f.id==='G01'?target:f.id==='G05'?'정정해서 좋다고 한 자료':f.id==='G08'?'앞서 확인한 자료':referentPhrases[j];
 const audience=f.id==='G06'?'내부회의':'외부 고객';
 const condition=(f.id==='G06'?'단점을 먼저 제시':f.id==='G07'?'구성만 재사용; 가격·문체·장기 Agent 선호는 계승하지 않음':f.id==='G10'?'기존 내용 대신 신규 조사 사용; 가격 제외':'신규 조사로 수치 교체')+`; 표는 ${j+2}개 이내`;
 const goal=`${topics[j]} 후속 발표자료 작성`,agent=`발표 Agent P${j+1}`,relation='별도 발표 작업 생성; 기존 조사 작업 유지';
 const input=`${p}의 ${topics[j]} 건에서 ${expression}의 구성을 활용해 ${audience}용 발표자료를 ${agent}에게 맡겨줘. ${fresh}를 사용하고, ${condition}. 기존 조사 작업은 계속해줘.`;
 const feedback=f.id==='G05'?'처음 것이 좋다고 한 말을 정정할게. 고객용으로 쓰려던 구성은 나중에 보여준 수정본이야.':f.id==='G07'?`먼저 보여준 쪽은 고객용 구성만 좋아. 가격과 문체는 그대로 쓰지 말고, P${j+1}는 이번 작업에만 맡긴 거야.`:f.id==='G04'?'방금 본 쪽의 구성이 고객용으로 좋네.':'고객에게 보여줄 구성은 먼저 보여준 쪽이 좋고 내부 검토에는 나중에 본 수정본을 쓰자.';
 const evidence=[`${base}-DISPLAY`,`${base}-F1`,`${base}-EXEC`,...(f.id==='G05'?[`${base}-F2`]:[]),...(f.id==='G08'?[`${base}-CONFIRM`]:[])];
 const oracle={goal,aspect:'구성',audience,condition,data:fresh,agent,taskRelation:relation};
 const storedTarget=(refFails.B[f.id]||[]).includes(j)?alternative:target;
 const bstate=f.id==='G05'?`dirty: F1의 ${d2} 관계는 F2로 무효화. 현재 요청은 원본 보완`:f.id==='G09'?'unproduced: 새 사건은 cursor 뒤':f.id==='G01'?'공통 명시 사실 직접 조회':f.id==='G08'?'양안에 확정 관계 존재':`valid/supported: ${storedTarget}, 구성, 외부 고객, F1·DISPLAY 의존; valid는 의미 정답 보증 아님`;
 const history=f.id==='G08'?`SYN-G02-${String(j+1).padStart(2,'0')} 후 공동 확인된 정답 관계를 동일 제공. 이전 오답은 QA02/07에 남김.`:f.background;
 cases.push({id,group:f.id,project:p,history,input,expression,expectedRef:JSON.stringify([target]),expectedMeaning:JSON.stringify(oracle),evidence:evidence.join(', '),bstate,target,alternative,title,wrongTitle,oracle,feedback});
 if(f.id!=='G08'){
  sources.push([`${base}-EXEC`,p,'공통 실행 사실',`조사 Agent R${j+1}가 조사 N 작성. ${agent}가 D2 작성, 수정 Agent Q${j+1}가 D3 작성. 현재 P${j+1}는 접근 가능하며 발표 작성 capability 공개.`,`D2=${d2}; D3=${d3}; N=${fresh}`]);
  sources.push([`${base}-DISPLAY`,p,'공통 표시 사실',f.id==='G04'?`${d2}만 표시 완료. ${d3}는 생성됐으나 아직 미표시.`:`${d2}와 ${d3}를 순서대로 실제 표시했다고 가정.`,`${d2}, ${d3}`]);
  sources.push([`${base}-F1`,p,'합성 피드백 원문',f.id==='G05'?'먼저 보여준 쪽의 고객용 구성이 좋아.':feedback,`${d2}, ${d3}`]);
  if(f.id==='G05')sources.push([`${base}-F2`,p,'합성 정정 원문',feedback,`${d2}, ${d3}`]);
  sources.push([`${base}-NEW`,p,'공통 신규 내용',`신규 조사 ${fresh}는 최신 수치의 출처. VIA는 공개 참조를 연결하고 내용 분석·발표 작성은 Downstream Agent에 위임.`,fresh]);
 }
 if(f.id==='G08')sources.push([`${base}-CONFIRM`,p,'합성 사용자 확인',`G02 이후 사용자가 “맞아, ${d2}의 구성을 말한 거야”라고 확인했다고 가정. 양안의 확정 사실에 동일 저장.`,d2]);
 for(const opt of ['A','B']){
  const rf=(refFails[opt][f.id]||[]).includes(j),mf=(meaningFails[opt][f.id]||[]).includes(j);
  assert(!(rf&&mf));
  const chosen=rf?alternative:target,chosenTitle=rf?wrongTitle:title,adopted={...oracle};
  if(mf){
   if(f.id==='G06') {adopted.audience='외부 고객';adopted.condition='장점을 먼저 제시';}
   else if(f.id==='G07')adopted.condition='가격·문체도 그대로 계승; 향후 모든 업무에서 P 우선';
   else if(f.id==='G10'){adopted.data=target;adopted.condition='기존 가격 포함';}
   else adopted.taskRelation='기존 조사 작업 중단';
  }
  const firstUnit=`${chosenTitle}(${chosen})의 ${adopted.aspect}을 활용해 ${adopted.audience}용 발표자료 작성을 ${adopted.agent}에게 맡기겠습니다.`;
  const first=`${firstUnit} 내용은 ${adopted.data}를 사용하고, ${adopted.condition}. ${adopted.taskRelation}.`;
  const reason=rf?(opt==='A'?'원본의 표시·피드백 연결을 놓쳐 대안 판본 채택':['G05','G09'].includes(f.id)?'원본 보완 중 대상 연결 오류':'파생 관계의 잘못된 대상 연결을 채택'):mf?(opt==='A'?'현재 조건 중 일부를 해석에서 누락':'과거 적용 범위 재사용 또는 현재 조건 연결 누락'):'필수 의미와 지칭 집합 모두 일치';
  responses.push({id,opt,expectedRef:JSON.stringify([target]),adoptedRef:JSON.stringify([chosen]),expectedMeaning:JSON.stringify(oracle),adoptedMeaning:JSON.stringify(adopted),firstUnit,first,reason,refOk:rf?0:1,meaningOk:rf||mf?0:1,error:rf?'TARGET':mf?'MEANING_ONLY':'NONE'});
  const firstSec=r3((opt==='A'?f.a:f.b)+(j-4.5)*.02);
  const capture=.08,display=.10,lookup=r3(firstSec*(opt==='A'?.20:.14));
  const hist=r3(['G01','G08'].includes(f.id)?0:firstSec*(opt==='A'?.43:f.id==='G09'?.42:f.id==='G05'?.25:.08));
  const meaning=r3(firstSec-capture-display-lookup-hist);
  const after=r3((opt==='A'?f.pa:f.pb)+(j-4.5)*.01);
  const details=r3(after-1),dispatch=.3,serving=.25,result=.45,external=12+j*1.4+Number(f.id.slice(1))*.3;
  times.push({id,opt,group:f.id,capture,lookup,hist,meaning,display,first:firstSec,details,dispatch,serving,result,via:r3(firstSec+after),external:r3(external),wall:r3(firstSec+after+external)});
 }
}
function allocate(total,weights){const sum=weights.reduce((a,b)=>a+b,0),raw=weights.map(w=>total*w/sum),res=raw.map(Math.floor);let extra=total-res.reduce((a,b)=>a+b,0);const order=raw.map((x,i)=>[x-res[i],i]).sort((a,b)=>b[0]-a[0]||a[1]-b[1]);for(let k=0;k<extra;k++)res[order[k][1]]++;return res.map(x=>x/1000);}
for(const opt of ['A','B']){
 const rows=times.filter(t=>t.opt===opt),alloc=allocate(opt==='A'?75000:30000,rows.map(t=>t.meaning+t.hist+t.details));
 rows.forEach((t,i)=>{costs.push([`SYN-COST-${opt}-${i+1}`,t.id,opt,'请求'.replace('请求','요청'),'VIA 요청 처리','현재 요청 해석·적용·조건부 원본 보완',alloc[i],'사용','합성 비용 단위; 호출·토큰·단가를 실제로 관측하지 않음']);costs.push([`SYN-REC-${opt}-${i+1}`,t.id,opt,'수신자','Downstream 처리','공통 수신자 비용 가정',.25,'사용','동일 비용 가정; 실제 업무 실행 없음']);});
}
for(let j=0;j<10;j++){
 for(const [kind,amount,use] of [['생산',5,'사용'],['생산',3.5,'미사용'],['갱신',2,'사용'],['갱신',1.5,'미사용']])costs.push([`SYN-PRE-${j+1}-${kind}-${use}`,`EP-P${j+1}`,'B',kind,'사전 관계 관리',`${projects[j]} 업무 이력의 ${use==='사용'?'후속 요청에서 활용한':'끝내 재참조하지 않은'} 관계`,amount,use,'프로젝트 episode에 한 번만 배분. 요청별 재합산 금지']);
}
for(const opt of ['A','B']){const rr=responses.filter(x=>x.opt===opt),tt=times.filter(x=>x.opt===opt);assert.equal(rr.reduce((s,x)=>s+x.refOk,0),opt==='A'?92:96);assert.equal(rr.reduce((s,x)=>s+x.meaningOk,0),opt==='A'?90:86);assert(Math.abs(tt.reduce((s,x)=>s+x.first,0)/100-(opt==='A'?2.4:1.1))<1e-9);assert(Math.abs(tt.reduce((s,x)=>s+x.via,0)/100-(opt==='A'?4:2.3))<1e-9);assert(Math.abs(costs.filter(x=>x[2]===opt).reduce((s,x)=>s+x[6],0)-(opt==='A'?100:175))<1e-9);}
assert.equal(new Set(cases.map(x=>x.id)).size,100);
const sourceIDs=new Set(sources.map(x=>x[0]));for(const c of cases)for(const id of c.evidence.split(', '))assert(sourceIDs.has(id));
await fs.writeFile(path.join(OUT,'synthetic-data.json'),JSON.stringify({status:'SYNTHETIC_ILLUSTRATIVE_NOT_MEASURED',created:'2026-10-21',method:'Author-designed outcomes and timings fitted to agreed illustrative totals; no randomness, statistical inference or model execution',families,cases,responses,times,costs,sources},null,2));
const wb=Workbook.create();
const names=['요약','평가기준','사례군','사례100','공통원본','응답판정','시간내역','비용원장'];
const sheets=Object.fromEntries(names.map(n=>[n,wb.worksheets.add(n)]));
const navy='#24435C',pale='#EAF1F5',muted='#5D6570',orange='#A05A00';
function col(n){let x='';for(n++;n>0;n=Math.floor((n-1)/26))x=String.fromCharCode(65+(n-1)%26)+x;return x;}
function base(name,title,headers,rows,widths,rowHeight=48){const s=sheets[name],end=col(headers.length-1);s.showGridLines=false;s.getRange(`A1:${end}${rows.length+6}`).format.font={name:'Arial',size:11,color:'#222222'};s.getRange('A2').values=[[title+' (합성)']];s.getRange('A2').format.font={size:15,bold:true,color:navy};s.getRange('A3').values=[[provenance]];s.getRange('A3').format.font={size:10,color:orange};s.getRange(`A5:${end}5`).values=[headers];s.getRange(`A5:${end}5`).format={fill:navy,font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},wrapText:true,rowHeight:38,verticalAlignment:'center'};if(rows.length)s.getRange(`A6:${end}${rows.length+5}`).values=rows;s.getRange(`A6:${end}${rows.length+5}`).format.rowHeight=rowHeight;s.getRange(`A6:${end}${rows.length+5}`).format.wrapText=true;s.getRange(`A6:${end}${rows.length+5}`).format.verticalAlignment='top';widths.forEach((w,i)=>s.getRange(`${col(i)}1:${col(i)}${rows.length+6}`).format.columnWidthPx=w);s.freezePanes.freezeRows(5);s.freezePanes.freezeColumns(2);return s;}
const def=[
 ['证据'.replace('证据','근거 수준'),'SYNTHETIC_ILLUSTRATIVE_NOT_MEASURED','수치·오답·문장은 목표 합계에 맞춰 작성','모델 실행·관측 timestamp·실험 로그 없음','실제 결과나 통계적 성능 근거로 사용하지 않음'],
 ['모집단','10개 가상 프로젝트 × 10개 사례군 = 100개 입력','각 입력에 A/B 합성 출력 한 개씩. 200개 비교 행','요청마다 평가 지칭 표현 하나, 필수 의미 전체 하나','100건은 독립 시행 100회가 아님. G08은 G02의 확정 사실을 재사용'],
 ['QA01','텍스트 제출 t=0','첫 채택된 완결된 내용 단위가 표시된 가정 시각','100개 입력의 시간 산술평균; 오답도 포함','앞선 채팅의 “첫 올바른 응답” 설명을 현재 QA-C01-v3 정의로 정정. ack/spinner/내부 proposal 제외'],
 ['첫 응답 예','대상 ID·작업·목적·수신 Agent를 구체적으로 안내','아직 작성 완료나 전송 완료를 주장하지 않는 수행 의도','응답판정의 QA01 종료 문장. 이어지는 상세 본문과 구분','첫 토큰이 아니라 첫 완결 문장 표시. QA07은 그 전에 최초 채택한 전체 의미 필드로 평가'],
 ['QA02','지정한 과거자료 지칭 표현 하나와 입력 revision','처음 채택한 canonical 대상 집합이 정답과 정확히 일치','정확히 일치한 집합 수 / 100','신규 조사 ID·Agent명은 공통 명시값. 모든 표현을 망라한 정확도가 아님. 최적 Agent 선택 점수로 확대하지 않음'],
 ['QA07','입력 하나와 revision','지칭 집합 + 작업·측면·목적·조건·내용 출처·Agent·기존 Task 관계 전체','필수 의미 전체가 맞은 입력 수 / 100','부분 항목 평균 아님. 이후 정정해도 최초 채택 실패 유지. 인계 이후 누락·Agent 실행 실패 제외'],
 ['QA03','텍스트 제출 t=0부터 합성 terminal 표시까지','입력·검색·관계 해석·현재 의미·표시·근거 보완·전달·serving·결과 연결','전체경과 - 외부 Agent 내부 처리 구간','합성 직렬 경로를 가정하므로 단계 합산 가능. 실제 병렬 제품에서는 interval union과 critical path로 검증 필요'],
 ['QA03 terminal','Downstream 공개 결과가 같은 Task에 연결되고 최종 결과 카드 표시','실제 도메인 업무 수행은 미실행. 결과 연결 단계까지 모두 완료했다고 가정','100개 모두 형식상 terminal 존재 가정, 내용 정확성과 별개','오답에도 실제 경과에 해당하는 합성 시간을 유지. timeout 벌점·실패 제외 없음'],
 ['후속 0.4초 차이','A 평균 0.6초 / B 평균 0.2초의 근거 보완','첫 안내 후 인계 manifest에 필요한 판본·근거 참조를 같은 계약으로 확보','B가 저장한 provenance를 써서 추가 읽기를 줄인다는 별도 가정','DP4 인계 정책·Downstream 지능 차이를 넣지 않음. 이 효과가 없으면 QA03 B는 2.7초가 되어 격차 축소'],
 ['요청 전 시간','B 생산·갱신은 요청 이전에 실행한다고 가정','수치가 정해진 준비 시간은 없음; 가상 timestamp를 꾸며 넣지 않음','QA03 요청 평균에 사전 시간 가산하지 않음','QA05에는 전부 포함. 요청과 경합하지 않는 준비 여유를 가정하므로 부하 우위 주장 불가'],
 ['QA05','각 프로젝트 준비부터 10개 후속 요청 종료까지','생산·갱신·미사용 관계·요청·수신자 비용 모두 포함','합성 비용 단위 합계; A=100으로 정규화','실제 달러·토큰·모델 가격 없음. B 사전 비용을 요청마다 중복 배분하지 않음'],
 ['검색과 모델','명시 ID·시간·판본·권한은 공통 코드 조회','모호한 관계와 현재 요청 의미만 동일 모델 자원을 사용한다고 가정','모델명·호출 횟수·token rate는 설정하지 않음','A=1회 B=여러 회로 강제하지 않음. Embedding 도입 차이도 없음'],
 ['공정성 가정','원본·확정 사실·권한·접속·capability는 양안 동일','A도 확정 사실 재사용. B는 dirty·미생산이면 원본 보완','같은 Downstream 인계 계약과 완료 조건','B를 유리하게 하는 안정된 과거 관계가 많은 분포를 의도적으로 선택. 실제 트래픽 대표성 없음'],
 ['오류 배정','A: 대상 8건 + 의미만 2건 / B: 대상 4건 + 의미만 10건','작성자가 실패 위치와 문장을 직접 정함','QA07 오류는 QA02 오류를 포함','LLM 관측 오류율이 아님. 분포·모델이 바뀌면 방향도 바뀔 수 있음'],
 ['범위','QA01·02·03·05·새 QA07만 합성 구성','QA04/06·새 QA07 점수와 우선순위는 만들지 않음','기존 발표 점수·평가 계약·실측 결과를 대체하지 않음','QA01 2.4초/1.1초는 기존 1초 목표를 모두 초과. 상대 우위가 목표 충족을 뜻하지 않음'],
 ['사례 흐름','프로젝트별 G01→G10 순서','G02 이후 공통 확인 사실을 양안에 제공하고 G08에서 재사용','G02 오답은 최초 정확도에 보존','확인 대화의 추가 입력은 100건에 미포함. 이 합성 모집단의 한계이며 비용은 요청 비용 가정 안에 포함'],
 ['출처','사용자와 합의한 가상 수치 및 DP5 설계','QA-C01-v3(2026-10-15), 14-corrected-poc-contract §3, INTENT QA02/07','기존 계약에서 지표 의미만 참조','기존 결과·상용 모델 성능·실제 사용자 대화를 가져온 자료가 아님']
];
base('평가기준','평가 경계와 합성 가정',['항목','시작·대상','종료·처리','집계·단위','주의 및 적용 범위'],def,[155,320,370,320,480],80);
base('사례군','10개 사례군과 예상 경로',['사례군','주제','건수','공통 배경','A 경로','B 경로'],families.map(f=>[f.id,f.name,10,f.background,f.apath,f.bpath]),[90,210,75,500,340,360],60);
base('사례100','입력과 정답 기준 100건',['사례 ID','사례군','프로젝트','공통 사전 조건','사용자 입력','QA02 지칭 표현','정답 대상 집합','QA07 필수 의미','근거 source ID','B 관계 상태'],cases.map(c=>[c.id,c.group,c.project,c.history,c.input,c.expression,c.expectedRef,c.expectedMeaning,c.evidence,c.bstate]),[155,85,85,390,620,240,240,700,390,390],85);
base('공통원본','양안에 제공하는 합성 원본',['Source ID','프로젝트','종류','내용','연결 자료'],sources,[240,90,160,850,500],50);
const responseSheet=base('응답판정','가상 최초 응답과 정확도',['사례 ID','방안','정답 대상 집합','최초 대상 집합','QA02 성공','정답 필수 의미','최초 채택 의미','QA07 성공','오류 분류','QA01 종료 문장(합성)','이어지는 본문 포함 응답(합성)','합성 오류 원인'],responses.map(x=>[x.id,x.opt,x.expectedRef,x.adoptedRef,null,x.expectedMeaning,x.adoptedMeaning,null,x.error,x.firstUnit,x.first,x.reason]),[155,70,240,240,110,700,700,110,150,620,900,380],95);
for(let i=0;i<200;i++){let r=i+6;responseSheet.getRange(`E${r}`).formulas=[[`=IF(C${r}=D${r},1,0)`]];responseSheet.getRange(`H${r}`).formulas=[[`=IF(AND(E${r}=1,F${r}=G${r}),1,0)`]];}
for(const c of ['E','H'])responseSheet.getRange(`${c}6:${c}205`).conditionalFormats.add('cellIs',{operator:'equal',formula:0,format:{fill:'#FCE9E7',font:{color:'#A52121',bold:true}}});
const timing=base('시간내역','VIA 처리시간의 단계 분해',['사례 ID','방안','사례군','입력 접수 s','검색·조회 s','과거 관계·보완 s','현재 의미 해석 s','채택·첫 표시 s','QA01 s','후속 근거 참조 s','전달·접수 s','원본 serving s','결과 연결·표시 s','QA03 s','외부 처리 제외 s','전체경과 s'],times.map(t=>[t.id,t.opt,t.group,t.capture,t.lookup,t.hist,t.meaning,t.display,null,t.details,t.dispatch,t.serving,t.result,null,t.external,null]),[155,70,90,120,120,145,140,140,115,150,125,140,155,115,150,130],35);
for(let i=0;i<200;i++){let r=i+6;timing.getRange(`I${r}`).formulas=[[`=SUM(D${r}:H${r})`]];timing.getRange(`N${r}`).formulas=[[`=SUM(I${r}:M${r})`]];timing.getRange(`P${r}`).formulas=[[`=N${r}+O${r}`]];}
timing.getRange('D6:P205').setNumberFormat('0.000');
const cost=base('비용원장','전체 episode의 합성 비용',['항목 ID','사례·episode','방안','단계','처리 주체','내용','비용 단위','사용 여부','산정 가정'],costs,[235,160,70,100,160,500,125,100,500],40);cost.getRange(`G6:G${costs.length+5}`).setNumberFormat('0.000');
const s=sheets['요약'];s.showGridLines=false;s.tabColor=navy;
s.getRange('A1:E48').format.font={name:'Arial',size:11,color:'#222222'};s.getRange('A1:E48').format.rowHeight=30;[310,100,130,130,620].forEach((w,i)=>s.getRange(`${col(i)}1:${col(i)}48`).format.columnWidthPx=w);
s.getRange('A2').values=[['DP5 가상 비교 백데이터']];s.getRange('A2').format.font={size:15,bold:true,color:navy};s.getRange('A3').values=[[provenance]];s.getRange('A3').format.font={color:orange,size:10};
function header(row,values){s.getRange(`A${row}:E${row}`).values=[values];s.getRange(`A${row}:E${row}`).format={fill:navy,font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},rowHeight:32};}
header(5,['지표','단위','A','B','가상 결과의 해석']);
const summaryRows=[['QA01 첫 응답','초',null,null,'오답 포함 최초 내용 표시. 성공 응답만 고르지 않음'],['QA02 지칭 정확도','%',null,null,'단일 대상 집합의 최초 채택. A 92/100, B 96/100'],['QA03 VIA 처리','초',null,null,'외부 Agent 내부 시간 제외. 모든 합성 terminal 포함'],['QA05 전체 모델 비용','비용 단위',null,null,'요청 전 생산·갱신·미사용·수신자 포함'],['QA07 요청 의미 정확도','%',null,null,'대상과 필수 의미 전체. A 90/100, B 86/100']];s.getRange('A6:E10').values=summaryRows;
for(const [opt,c] of [['A','C'],['B','D']]){
 s.getRange(`${c}6`).formulas=[[`=AVERAGEIF('시간내역'!$B$6:$B$205,"${opt}",'시간내역'!$I$6:$I$205)`]];
 s.getRange(`${c}7`).formulas=[[`=AVERAGEIF('응답판정'!$B$6:$B$205,"${opt}",'응답판정'!$E$6:$E$205)`]];
 s.getRange(`${c}8`).formulas=[[`=AVERAGEIF('시간내역'!$B$6:$B$205,"${opt}",'시간내역'!$N$6:$N$205)`]];
 s.getRange(`${c}9`).formulas=[[`=SUMIF('비용원장'!$C$6:$C$445,"${opt}",'비용원장'!$G$6:$G$445)`]];
 s.getRange(`${c}10`).formulas=[[`=AVERAGEIF('응답판정'!$B$6:$B$205,"${opt}",'응답판정'!$H$6:$H$205)`]];
}
s.getRange('C6:D10').setNumberFormat('0.00');s.getRange('C7:D7').setNumberFormat('0%');s.getRange('C10:D10').setNumberFormat('0%');
s.getRange('A12').values=[['100개 입력, A/B 각 100개 출력. 시간·비용은 합계에 맞춘 설정값이고 정확도는 작성자가 배정한 오류다.']];s.getRange('A12').format.font={size:10,color:muted};
header(14,['평균 시간 분해','단위','A','B','포함 경로']);
const stageCols=['D','E','F','G','H','J','K','L','M'];const stageNames=['입력 접수','검색·조회','과거 관계 해석·원본 보완','현재 요청 의미 해석','채택·첫 내용 표시','후속 근거 참조','Agent 전달·접수','원본 serving','결과 연결·표시'];
for(let i=0;i<9;i++){let r=15+i;s.getRange(`A${r}:E${r}`).values=[[stageNames[i],'초',null,null,i<5?'QA01 및 QA03에 포함':'QA03에만 포함']];for(const [opt,c]of[['A','C'],['B','D']])s.getRange(`${c}${r}`).formulas=[[`=AVERAGEIF('시간내역'!$B$6:$B$205,"${opt}",'시간내역'!$${stageCols[i]}$6:$${stageCols[i]}$205)`]];}
s.getRange('A24:B24').values=[['VIA 합계','초']];s.getRange('C24:D24').formulas=[['=SUM(C15:C23)','=SUM(D15:D23)']];s.getRange('C15:D24').setNumberFormat('0.000');s.getRange('E24').values=[['요청 전 작업 시간은 미설정. 비용은 아래 전체 비용에 포함']];
header(27,['전체 비용 분해','단위','A','B','비용 범위']);
const costTypes=['생산','갱신','요청','수신자'];for(let i=0;i<4;i++){let r=28+i;s.getRange(`A${r}:E${r}`).values=[[['사전 관계 생산','파생 관계 갱신','요청 시 VIA 처리','공통 수신자'][i],'비용 단위',null,null,i<2?'사용·미사용 관계 모두 포함':'해당 100건 전체 포함']];for(const [opt,c]of[['A','C'],['B','D']])s.getRange(`${c}${r}`).formulas=[[`=SUMIFS('비용원장'!$G$6:$G$445,'비용원장'!$C$6:$C$445,"${opt}",'비용원장'!$D$6:$D$445,"${costTypes[i]}")`]];}
s.getRange('A32:B32').values=[['전체 합계','비용 단위']];s.getRange('C32:D32').formulas=[['=SUM(C28:C31)','=SUM(D28:D31)']];s.getRange('C28:D32').setNumberFormat('0.000');
header(35,['최초 오류 분해','단위','A','B','QA02와 QA07의 관계']);
for(let i=0;i<3;i++){let r=36+i;s.getRange(`A${r}:E${r}`).values=[[['대상부터 오류','대상은 맞고 의미만 오류','대상·의미 모두 정답'][i],'건',null,null,['QA02 실패 및 QA07 실패','QA02 성공 및 QA07 실패','QA02 성공 및 QA07 성공'][i]]];for(const[opt,c]of[['A','C'],['B','D']])s.getRange(`${c}${r}`).formulas=[[`=COUNTIFS('응답판정'!$B$6:$B$205,"${opt}",'응답판정'!$I$6:$I$205,"${['TARGET','MEANING_ONLY','NONE'][i]}")`]];}
s.getRange('A41').values=[['이 파일은 합성 설계 예시다. 수치가 실제 구조 우위를 검증하지 않으며 기존 PPTX·실측 결과는 변경하지 않았다.']];s.getRange('A41').format.font={color:orange,size:10};
s.getRange('C6:D38').format.horizontalAlignment='right';
for(const r of [5,14,27,35])s.getRange(`B${r}:D${r}`).format.horizontalAlignment='center';
for(let r=5;r<=38;r++){const v=s.getRange(`E${r}`).values[0]?.[0];if(typeof v==='string')s.getRange(`E${r}`).values=[['   '+v]];}
wb.recalculate();
const got=s.getRange('C6:D10').values;const expected=[[2.4,1.1],[.92,.96],[4,2.3],[100,175],[.9,.86]];for(let i=0;i<5;i++)for(let j=0;j<2;j++)assert(Math.abs(got[i][j]-expected[i][j])<1e-8,`summary mismatch ${i},${j}: ${got[i][j]}`);
// A disposable change proves both a row formula and its summary dependency recalculate.
const old=timing.getRange('D6').values[0][0];timing.getRange('D6').values=[[old+.1]];assert(Math.abs(s.getRange('C6').values[0][0]-2.401)<1e-8);timing.getRange('D6').values=[[old]];
wb.recalculate();
const inspection=await wb.inspect({kind:'table',range:'요약!A5:E10',include:'values,formulas',tableMaxRows:6,tableMaxCols:5,maxChars:5000});await fs.writeFile(path.join(OUT,'summary-inspection.ndjson'),inspection.ndjson);
const errors=await wb.inspect({kind:'match',searchTerm:'#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A|#NUM!|#NULL!|#SPILL!|#CALC!',options:{useRegex:true,maxResults:50},maxChars:3000});await fs.writeFile(path.join(OUT,'formula-scan.ndjson'),errors.ndjson);
const previews=[['요약','A1:E12'],['요약','A14:E38'],['평가기준','A1:E8'],['사례군','A1:F9'],['사례100','A1:F7'],['사례100','G5:J7'],['공통원본','A1:E8'],['응답판정','A1:E8'],['응답판정','H5:K8'],['시간내역','A1:I10'],['시간내역','J5:P10'],['비용원장','A1:I10']];
for(let i=0;i<previews.length;i++){const[n,range]=previews[i];const blob=await wb.render({sheetName:n,range,scale:1,format:'png'});await fs.writeFile(path.join(OUT,'previews',`${String(i+1).padStart(2,'0')}-${n}.png`),new Uint8Array(await blob.arrayBuffer()));}
await(await SpreadsheetFile.exportXlsx(wb)).save(path.join(DELIVER,'DP5-synthetic-backdata.xlsx'));
await fs.writeFile(path.join(OUT,'validation.json'),JSON.stringify({status:'SYNTHETIC_NOT_MEASURED',case_count:100,response_rows:200,timing_rows:200,cost_rows:costs.length,source_rows:sources.length,unique_case_ids:true,source_refs_resolve:true,totals_match_authored_targets:true,formula_recalculation_perturbation_checked:true,summary:got,actual_model_calls:0,real_windows_excel_verified:false,qa01_definition_correction:'First substantive adopted content, including wrong content; not first correct answer',cost_note:'Arbitrary normalized synthetic units, no real token/price attribution'},null,2));
console.log(JSON.stringify({file:path.join(DELIVER,'DP5-synthetic-backdata.xlsx'),summary:got,cases:cases.length,costRows:costs.length,sources:sources.length}));
