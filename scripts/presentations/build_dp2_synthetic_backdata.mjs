// Authored illustration only: no model calls, candidate execution or measured logs.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {Workbook, SpreadsheetFile} from '@oai/artifact-tool';

const ROOT='/Users/changbae/Workspace/via-sw-architecture';
const OUT=path.join(ROOT,'docs/presentations_files/dp2-synthetic-backdata-20261021');
const DELIVER=path.join(OUT,'outputs/01a154e2');
await fs.mkdir(DELIVER,{recursive:true});
await fs.mkdir(path.join(OUT,'previews'),{recursive:true});
const KIND='SYNTHETIC_AUTHORED_NOT_MEASURED';
const VERSION='DP2-SYN-v1';
const notice='설명용 합성 데이터 · 수치에 맞춰 작성한 예시 · 실제 모델/제품/실험 관측 아님';
const r6=x=>Math.round(x*1e6)/1e6;
const mean=xs=>xs.reduce((s,x)=>s+x,0)/xs.length;
const clone=x=>JSON.parse(JSON.stringify(x));
const canon=x=>JSON.stringify(x);
const projects=['새봄','해솔','가온','누리','다온','마루','이든','라온','한결','온유'];
const subjects=['협업 도구','고객지원 시스템','물류 장비','매장 단말','교육 장비','출시 행사','연구 장비','사무공간','품질 검사 장비','파트너 서비스'];
const families=[
 {id:'G1',name:'결과 의존 정정',first:{A:1.4,B:1.3},via:{A:5.2,B:5.6},read:.65,refs:2,start:-.6,rel:'REVISE_R1',map:'S08/S18/S21',explain:'메일 도착 순서로 대상을 바꾸고 비교를 요약으로 교체. 기존 제외 조건 유지.'},
 {id:'G2',name:'조건 수정',first:{A:1.2,B:.6},via:{A:4.8,B:4.6},read:.75,refs:1,start:-.6,rel:'REVISE_R1',map:'S18/S21',explain:'설치비 포함·납기 제외는 파일 조회 반환 전에도 이해 가능.'},
 {id:'G3',name:'공유 근거 추가 목표',first:{A:1.4,B:.7},via:{A:5.6,B:5.2},read:.85,refs:1,start:0,rel:'ADD_KEEP_R1',map:'S17/S18',explain:'같은 두 파일로 구매팀 질문 목록 목표 추가. 원래 보고서 목표 유지.'},
 {id:'G4',name:'독립 질문',first:{A:1.6,B:.6},via:{A:4.4,B:2.2},read:.95,refs:0,start:0,rel:'INDEPENDENT_KEEP_R1',map:'S01/S17',explain:'TCP/UDP 세 문장 설명. 기존 견적 자료가 새 답변에 필요하지 않음.'}
];
// Five wordings per family, ten project substitutions. 20 templates, not 200 independent trials.
const words={
 G1:['그 두 견적 중 나중에 온 것 하나만 요약하는 초안으로 바꿔줘.','그 두 견적을 비교하지 말고, 나중에 온 것 하나만 요약해줘.','그 두 견적 중 나중에 온 것 하나의 요약 초안만 필요해.','그 두 견적 비교는 취소하고, 나중에 온 것 하나만 요약하는 초안으로 변경해줘.','그 두 견적 중 나중에 온 것 하나만 요약하도록 요청을 수정해줘.'],
 G2:['그 두 견적의 설치비도 포함하고, 납기는 비교에서 빼줘.','그 두 견적 비교 조건을 설치비 포함, 납기 제외로 바꿔줘.','그 두 견적은 설치비까지 비교해줘. 납기는 제외해줘.','그 두 견적 비교에서 빼야 할 건 설치비가 아니라 납기야.','그 두 견적의 비용에는 설치비를 넣고, 납기 항목은 빼줘.'],
 G3:['같은 두 견적에 대해 구매팀에 보낼 질문 목록도 만들어줘. 보고서 작성도 계속해.','같은 두 견적으로 구매팀 질문 목록을 추가로 만들어줘. 원래 보고서도 계속해.','보고서는 계속하고, 같은 두 견적에 관한 구매팀 질문 목록도 작성해줘.','같은 두 견적을 쓰는 구매팀 질문 목록 작업을 하나 더 추가해줘. 보고서는 유지해.','같은 두 견적에 대해 구매팀에 물어볼 항목도 별도로 정리해줘. 보고서는 중단하지 마.'],
 G4:['보고서는 계속하고, TCP와 UDP의 차이를 세 문장으로 설명해줘.','원래 보고서는 유지해. 별개로 TCP와 UDP 차이를 세 문장으로 알려줘.','추가 질문이야. TCP와 UDP 차이를 세 문장으로 말해줘. 보고서는 계속해.','보고서 작업과 별개로 TCP와 UDP를 세 문장으로 비교해줘. 기존 작업은 유지해.','보고서는 중단하지 말고, TCP와 UDP의 차이를 세 문장으로 설명해줘.']
};
// Predetermined illustrative mistakes, never sampled model outcomes.
const err={A:{G1:{target:[3],meaning:[]},G2:{target:[4,24],meaning:[34]},G3:{target:[5,25],meaning:[]},G4:{target:[],meaning:[6,26]}},B:{G1:{target:[3,13,23],meaning:[33,38,43]},G2:{target:[],meaning:[4,24,34]},G3:{target:[5,15,25,35],meaning:[]},G4:{target:[],meaning:[6]}}};
const cases=[],sources=[],responses=[],referents=[],times=[],events=[],companions=[];
for(const f of families)for(let j=0;j<50;j++){
 const project=projects[Math.floor(j/5)],subject=subjects[Math.floor(j/5)],v=j%5;
 const id=`SYN-DP2-${f.id}-${String(j+1).padStart(3,'0')}`,ep=`${id}-EP`,p=`${id}-P`,q=`${id}-Q`,r=`${id}-R`,request=`${id}-R1`;
 const initial=`어제 메일로 받은 ${subject} 견적서 두 개를 비교하는 보고서 초안을 만들어줘. 설치비는 제외해.`;
 const goal=['G1','G2'].includes(f.id)?`${request}-GOAL`:`${id}-NEW-GOAL`;
 const input=words[f.id][v]+(f.id==='G3'?' 설치비는 질문 목록에서도 제외해.':'');
 const required={goal:f.id==='G1'?'SUMMARY_DRAFT':f.id==='G2'?'COMPARISON_REPORT_DRAFT':f.id==='G3'?'PURCHASE_QUESTION_LIST':'EXPLAIN_TCP_UDP',targets:f.id==='G1'?[q]:f.id==='G4'?[]:[p,q],relation:f.rel,base_request:request,installation:f.id==='G2'?'INCLUDE':f.id==='G4'?'NOT_APPLICABLE':'EXCLUDE',delivery:f.id==='G2'?'EXCLUDE':f.id==='G4'?'NOT_APPLICABLE':'UNSPECIFIED',audience:f.id==='G3'?'PURCHASE_TEAM':'USER',output:f.id==='G4'?'THREE_SENTENCES':'DRAFT_ONLY',external_send:false,original_goal:f.id==='G1'?'REPLACE_COMPARISON_WITH_SUMMARY':f.id==='G2'?'KEEP_WITH_UPDATED_CONDITIONS':'KEEP'};
 if(f.id==='G1')required.base_pair=[p,q];
 const refspec=f.id==='G1'?[['E1','그 두 견적',[p,q]],['E2','나중에 온 것 하나',[q]]]:f.id==='G2'?[['E1','그 두 견적',[p,q]]]:f.id==='G3'?[['E1','같은 두 견적',[p,q]]]:[];
 const read=r6(f.read+(v-2)*.015),jitter=r6((j-24.5)*.004);
 const sourceRows=[
  {id:`${id}-U0`,type:'PRIOR_INPUT',payload:initial,available_at_s:-.6},
  {id:p,type:'MAIL_ATTACHMENT',payload:`${project} / ${subject} 공급사 P 견적. 메일 도착 D-1 09:10, 파일 수정 D-1 09:00.`,available_at_s:read},
  {id:q,type:'MAIL_ATTACHMENT',payload:`${project} / ${subject} 공급사 Q 견적. 메일 도착 D-1 15:40, 파일 수정 D-1 15:10.`,available_at_s:read},
  {id:r,type:'DISTRACTOR_ATTACHMENT',payload:`이전 주 공급사 R 견적. 메일 도착 D-7 17:10, 파일 수정 D0 08:00. 메일 도착과 파일 수정 순서는 다름.`,available_at_s:read},
  {id:`${id}-READ`,type:'READ_STATE',payload:`VIA는 이미 허용된 메일 목록/첨부 메타데이터를 조회 중. 새 발화 종료 t=0. 조회는 t=${read}에 반환. 업무 분석은 Downstream 소유.`,available_at_s:-.3},
  {id:`${id}-U1`,type:'FOLLOWUP_INPUT',payload:input,available_at_s:0}
 ];
 sources.push(...sourceRows.map(s=>({...s,case_id:id,evidence_kind:KIND})));
 const c={case_id:id,episode_id:ep,dp_id:'DP2',group:f.id,project,variant:v+1,input_mode:'VOICE_TRANSCRIPT_FIXTURE',original_request_id:request,revision:2,initial_input:initial,input,goal_id:goal,goal_start_s:f.start,read_return_s:read,source_ids:sourceRows.map(s=>s.id),oracle:required,referents:refspec,common_mapping:f.map,mapping_status:'THEMATIC_ONLY_NOT_COMMON28_EXECUTION',evidence_kind:KIND};cases.push(c);
 for(const opt of ['A','B']){
  const targetError=err[opt][f.id].target.includes(j),meaningError=err[opt][f.id].meaning.includes(j),adopted=clone(required);
  if(targetError){adopted.targets=f.id==='G1'?[r]:[p,r];if(f.id==='G1')adopted.base_pair=[p,r];}
  if(meaningError){if(f.id==='G1')adopted.installation='INCLUDE';else if(f.id==='G2')adopted.relation='NEW_INSTEAD_OF_REVISION';else adopted.original_goal='CANCEL';}
  const chosen=adopted.targets.join(', ');
  let first=f.id==='G1'?`“${project} ${subject} 공급사 ${targetError?'R':'Q'} 견적” 하나의 요약 초안 작성으로 변경하겠습니다.`:f.id==='G2'?'비교 조건을 설치비 포함, 납기 제외로 변경하겠습니다.':f.id==='G3'?'기존 보고서는 유지하고 구매팀 질문 목록 작성을 추가하겠습니다.':'TCP는 신뢰성 있는 순서 전달을 제공하고, UDP는 전달과 순서를 보장하지 않는 데이터그램 방식입니다.';
  const full=f.id==='G4'?`${first} TCP는 연결을 설정해 데이터를 주고받습니다. UDP는 연결 설정 없이 데이터그램을 보내며 필요한 신뢰성은 응용에서 보완할 수 있습니다.`:`대상: ${chosen}. 목표: ${adopted.goal}. 설치비: ${adopted.installation}. 납기: ${adopted.delivery}. 기존 목표: ${adopted.original_goal}. 외부 발송은 하지 않고 초안 작성만 위임합니다.`;
  const reason=targetError?'메일 도착과 수정 시각/이전 판본을 혼동한 가상 오류':meaningError?(f.id==='G1'?'유지해야 할 설치비 제외 조건 누락':f.id==='G2'?'조건 수정은 맞지만 기존 요청의 revision으로 연결하지 않음':'독립 질문을 처리하면서 기존 목표를 취소로 잘못 확정'):'필수 의미 전체 일치';
  responses.push({case_id:id,option:opt,group:f.id,first_content:first,final_text:full,oracle:required,adopted,qa07:canon(required)===canon(adopted)?1:0,error:targetError?'TARGET':meaningError?'MEANING_ONLY':'NONE',error_reason:reason,evidence_kind:KIND});
  for(const [rid,phrase,expected]of refspec){const chosenRef=targetError?(f.id==='G1'&&rid==='E2'?[r]:[p,r]):expected;referents.push({case_id:id,option:opt,expression_id:`${id}-${rid}`,phrase,revision:2,expected,adopted:chosenRef,correct:canon(expected)===canon(chosenRef)?1:0,evidence_kind:KIND});}
  const firstAt=r6(f.first[opt]+jitter),via=r6(f.via[opt]+jitter*1.5),end=r6(via+f.start);
  const intake=.08,relation=opt==='B'?.18:0;
  const wait=opt==='A'?r6(read-intake):f.id==='G1'?r6(read-intake-relation):0;
  const interpret=r6(firstAt-intake-relation-wait-.10);
  assert(interpret>0);
  const fullAt=opt==='B'&&['G2','G3'].includes(f.id)?r6(Math.max(firstAt,read)+(f.id==='G2'?.25:.30)):r6(firstAt-.10);
  const late=r6(Math.max(0,fullAt-firstAt));
  const delivery=f.id==='G4'?0:.25,result=f.id==='G4'?.15:.30,voice=f.id==='G4'?.30:.45;
  const remaining=r6(end-firstAt-late-delivery-result-voice);
  assert(remaining>0);
  const ext=f.id==='G4'?0:r6(12+(j%10)*.4);
  const t={case_id:id,option:opt,group:f.id,goal_id:goal,goal_start_s:f.start,prior:r6(-f.start),intake,relation,wait,interpret,display:.10,first:firstAt,full_meaning_at:fullAt,late,remaining,delivery,result,voice,via,external:ext,wall:r6(via+ext),read_return:read,evidence_kind:KIND};times.push(t);
  const prefix=`${id}-${opt}`,nodes=[];
  function node(suffix,label,duration,parents,role='VIA',counts=true){const item={event_id:`${prefix}-${suffix}`,case_id:id,option:opt,goal_id:goal,label,role,duration_s:r6(duration),parents:parents.map(n=>n.event_id),count_in_qa03:counts,evidence_kind:KIND};const max=parents.length?Math.max(...parents.map(x=>x.finish_s)):0;item.start_s=r6(max);item.finish_s=r6(max+item.duration_s);item.projected_finish_s=r6((parents.length?Math.max(...parents.map(x=>x.projected_finish_s)):0)+(counts?item.duration_s:0));nodes.push(item);return item;}
  const start=node('START','목표 시작',0,[]),prior=node('PRIOR','정정 전 VIA 처리 구간',-f.start,[start]);
  const ingest=node('INPUT','후속 입력 접수',intake,[prior]);
  const readNode=node('READ','진행 중 조회 잔여 경로',read,[prior]);
  let decisionParents;
  if(opt==='A')decisionParents=[ingest,readNode];else {const rel=node('RELATION','공통 대화의 관계 판단',relation,[ingest]);decisionParents=f.id==='G1'?[rel,readNode]:[rel];}
  const decision=node('INTENT','첫 내용에 필요한 판단',interpret,decisionParents);
  const displayed=node('DISPLAY','첫 구체적 내용 표시',.1,[decision]);
  let completeMeaning=decision;
  if(opt==='B'&&['G2','G3'].includes(f.id))completeMeaning=node('FULL_MEANING','조회 결과 결합과 전체 의미 확정',f.id==='G2'?.25:.30,[displayed,readNode]);
  const prepare=node('PREPARE',f.id==='G4'?'전체 설명 생성':'후속 근거 확인·위임 명세 준비',remaining,[displayed,completeMeaning]);
  const send=node('RECEIPT','Agent 전달·접수',delivery,[prepare]);
  const external=node('EXTERNAL','외부 Agent 업무 — QA03 제외',ext,[send],'DOWNSTREAM',false);
  const resultNode=node('RESULT','결과 연결·상세 표시',result,[external]);
  const done=node('VOICE','전체 음성 요약 준비',voice,[resultNode]);
  assert(Math.abs(displayed.finish_s+f.start-firstAt)<1e-8);
  assert(Math.abs(completeMeaning.finish_s+f.start-fullAt)<1e-8);
  assert(Math.abs(done.projected_finish_s-via)<1e-8);
  assert(Math.abs(done.finish_s-(via+ext))<1e-8);
  // READ can run concurrently; only dependency paths determine durations, never sum all work.
  events.push(...nodes);
  if(['G3','G4'].includes(f.id))companions.push({case_id:id,option:opt,goal_id:`${request}-GOAL`,status_adopted:adopted.original_goal,read_return_s:read,synthetic_via_completion_s:adopted.original_goal==='CANCEL'?null:r6((opt==='A'?6.2:5.5)+jitter),terminal_kind:adopted.original_goal==='CANCEL'?'WRONGLY_CANCELLED_NOT_COMPLETED':'SYNTHETIC_COMPLETED',headline_population:false,reason:'기존 목표는 유지 여부 및 경합 진단. QA03 주표는 사전 지정한 새 목표 하나만 집계. 잘못 취소한 목표의 완료시간은 비워 둠.',evidence_kind:KIND});
 }
}
// Cost is a separate authored COMMON-28 catalog ledger, not extrapolated from the 200-case set.
const common=JSON.parse(await fs.readFile(path.join(ROOT,'docs/architecture/12-decisions/decision-packages/qa-candidate-package/common-scenarios.json'),'utf8'));
const costTotals={A:{voice:1.12,interpret:.62,relation:.06,join:.06,discard:.10,prepare:.08,tail:.06},B:{voice:1.12,interpret:.54,relation:.20,join:.18,discard:.14,prepare:.08,tail:.09}};
const names={voice:'입출력 음성',interpret:'요청 의미·응답 판단',relation:'후속 입력 관계 판단',join:'문맥 전달·재결합',discard:'폐기·재판단',prepare:'초기 준비',tail:'마지막 입력 이후 처리'};
const costRows=[];
function allocate(total,weights){const units=Math.round(total*1e6),sum=weights.reduce((a,b)=>a+b,0),raw=weights.map(x=>units*x/sum),a=raw.map(Math.floor);let rest=units-a.reduce((x,y)=>x+y,0);const order=raw.map((x,i)=>[x-a[i],i]).sort((x,y)=>y[0]-x[0]||x[1]-y[1]);for(let i=0;i<rest;i++)a[order[i][1]]++;return a;}
for(const opt of ['A','B'])for(const stage of Object.keys(names)){
 const weights=common.scenarios.map(s=>Math.max(1,s.events.length)*(stage==='relation'||stage==='join'?(['S08','S17','S18','S20','S21'].includes(s.id)?3:1):1));
 const amounts=allocate(costTotals[opt][stage],weights);
 common.scenarios.forEach((s,i)=>{
  const micro=amounts[i],base={option:opt,bundle_id:'SYN-COMMON28-BUNDLE-1',catalog_id:s.id,catalog_title:s.title,stage:names[stage],scope:'VIA_ONLY',model:'SYNTHETIC_TARIFF_NO_PROVIDER',evidence_kind:KIND};
  if(stage==='voice')costRows.push({...base,cost_id:`SYN-COST-${opt}-${stage}-${s.id}`,bucket:'audio_seconds',quantity:micro/2000,unit_price:.002,cost:micro/1e6});
  else {const output=Math.floor(micro*.3/4),input=micro-output*4;costRows.push({...base,cost_id:`SYN-COST-${opt}-${stage}-${s.id}-IN`,bucket:'uncached_text_input_token',quantity:input,unit_price:.000001,cost:input/1e6});costRows.push({...base,cost_id:`SYN-COST-${opt}-${stage}-${s.id}-OUT`,bucket:'text_output_token',quantity:output,unit_price:.000004,cost:output*4/1e6});}
 });
}
const changes=[];
const changeSpecs=[['C01','모델 요청과 응답 형식 변경',['model_adapter.rs','usage.rs']],['C02','Agent 요청·접수 형식 변경',['agent_adapter.rs']],['C03','화면·과거 자료 형식 변경',['evidence_adapter.rs','source_ref.rs']],['C04','문단 단위 응답 수정 추가',['response_revision.rs','response_view.rs']],['C05','원본 위치·버전 표시 추가',['source_ref.rs','response_view.rs']],['C06','구간별 사용 허용 철회 추가',['access_scope.rs','pending_output.rs','request_state.rs']]];
for(const opt of ['A','B'])for(const [id,title,mods]of changeSpecs)changes.push({option:opt,change_id:id,title,modules:mods.map(m=>`SYNTHETIC_DESIGN_ONLY/${m}`),count:mods.length,evidence_kind:KIND});
const memory=[];
for(const opt of ['A','B'])for(let k=0;k<4;k++){const context=opt==='A'?[18,24,36,16][k]:[24,50,72,24][k],result=opt==='A'?[6,14,18,8][k]:[8,20,31,8][k],exec=opt==='A'?[4,6,4,4][k]:[4,7,10,4][k];memory.push({snapshot_id:`SYN-RAM-${opt}-${k+1}`,option:opt,stage:['기존 조회','새 입력 판단','겹친 완료','정리'][k],base:240,input_buffer:12,context,result_buffer:result,execution:exec,total:240+12+context+result+exec,profile:'CLOUD_MODEL_LOCAL_VIA_ONLY_NO_LOCAL_MODEL_WEIGHTS',evidence_kind:KIND});}
const summary={};
for(const opt of ['A','B']){
 const ts=times.filter(x=>x.option===opt),rs=responses.filter(x=>x.option===opt),refs=referents.filter(x=>x.option===opt);
 summary[opt]={qa01:mean(ts.map(x=>x.first)),qa02:mean(refs.map(x=>x.correct)),qa03:mean(ts.map(x=>x.via)),qa04:mean(changes.filter(x=>x.option===opt).map(x=>x.count)),qa05:costRows.filter(x=>x.option===opt).reduce((s,x)=>s+x.cost,0),qa06:Math.max(...memory.filter(x=>x.option===opt).map(x=>x.total)),qa07:mean(rs.map(x=>x.qa07)),ref_denominator:refs.length,input_denominator:rs.length};
 const target=opt==='A'?[1.4,.97,5,2,2.1,310,.96]:[.8,.95,4.4,2,2.35,365,.93];
 target.forEach((n,i)=>assert(Math.abs(summary[opt][`qa0${i+1}`]-n)<1e-8,`${opt} QA${i+1}`));
 for(const f of families){const values=rs.filter(x=>x.group===f.id).map(x=>x.qa07);const intended=opt==='A'?{G1:.98,G2:.94,G3:.96,G4:.96}:{G1:.88,G2:.94,G3:.92,G4:.98};assert(Math.abs(mean(values)-intended[f.id])<1e-8);}
}
assert.equal(cases.length,200);assert.equal(responses.length,400);assert.equal(referents.length,400);
const ids=new Set(sources.map(x=>x.id));for(const c of cases)for(const id of c.source_ids)assert(ids.has(id));
for(const response of responses){const rr=referents.filter(x=>x.case_id===response.case_id&&x.option===response.option);if(rr.some(x=>!x.correct))assert.equal(response.qa07,0);}
const schema=[
 ['evidence_kind','모든 표','SYNTHETIC_AUTHORED_NOT_MEASURED','실측으로 전환하지 않음. 실측은 별도 dataset ID'],
 ['schema_version / dataset_id','메타','DP-BACKDATA-0.1 / DP2-SYN-v1','DP5 원본은 미변경. 후속 변환기에서 명시적 매핑'],
 ['dp_id / option','모든 비교','DP2 / A 또는 B','DP 번호 재편 이력과 대안 의미를 함께 보존'],
 ['case_id / episode_id','입력','SYN-DP2-G1-001 / 독립 fixture','반복 seed나 실제 실험 ID로 가장하지 않음'],
 ['request_id / revision / goal_id','입력·시간','원래 요청과 후속 revision / 새 목표','정정은 원래 목표 시작점 유지'],
 ['expression_id / expected / adopted','지칭판정','지칭 표현·revision별 정답과 최초 집합','입력 수와 다른 분모. 집합은 canonical 정렬'],
 ['oracle / adopted / first_content','응답판정','필수 의미 전체 / 최초 확정 / 첫 표시','QA07은 전체 일치. 첫 문장이 전부를 말할 필요 없음'],
 ['full_meaning_at / first','시간내역','전체 의미 확정 / 첫 구체적 표시','부분 처리 의도와 전체 의미 확정을 분리'],
 ['event_id / parents / role','실행단계','의존 DAG / VIA 또는 DOWNSTREAM','노드 합이 아니라 완료 경로. 외부 시간은 투영에서 0'],
 ['goal_start_s / via / wall','시간내역','초 / 귀속 시간 / 전체 경과','정정 원래 시작점과 입력별 QA01 시작점 구별'],
 ['bundle_id / catalog_id','비용원장','SYN-COMMON28-BUNDLE-1 / S01~S28','200개 의미 fixture와 별도 모집단. 환산 금지'],
 ['bucket / quantity / unit_price','비용원장','서로 중복 없는 가상 사용량과 가상 단가','실제 provider 요금·usage 아님'],
 ['scope','비용원장','VIA_ONLY','DP5 normalized 비용은 Downstream 포함: 바로 비교 금지'],
 ['change_id / modules','변경범위','C01~C06 / 가상 경로','실제 diff나 구현 완료를 뜻하지 않음'],
 ['snapshot_id / profile','메모리','동시 시점 가상 합 / cloud profile','시점별 total의 MAX; 구성요소별 MAX 합산 금지'],
 ['mapping_status','사례','THEMATIC_ONLY_NOT_COMMON28_EXECUTION','S 참조는 주제 대응이며 COMMON-28 실행/coverage 아님'],
 ['error / reason','응답판정','TARGET / MEANING_ONLY / NONE','설정한 오류. 구조의 필연성·유의성 주장 금지'],
 ['headline_population','기존목표진단','false','새 목표 선택 시 기존 목표 결과는 별도 진단. 전체 평균에 몰래 혼합 금지']
];
const rules=[
 ['출처와 상태','모든 값은 합성','이전 대화의 숫자에 맞춰 오류·시간·비용을 역구성. 후보 실행·모델 호출·계측 없음.','실측/통계적 우위 주장 불가'],
 ['대안 고정','A 동일 실행 / B 별도 입력 판단','A는 다음 step에서 새 입력을 해석. B는 공통 대화에서 먼저 해석하고 기존 실행/새 목표에 연결.','B도 공유 근거 사용. 원래 요청 전체를 A가 기다리게 하지 않음'],
 ['QA01','후속 발화 종료 t=0 → 첫 구체적 내용','오답도 원시간 유지. 접수 인사·미해결 지칭 반복 제외. G2/G3 B는 조건/새 목표 의도를 먼저 채택.','QA01-v3 + 14 보완계약. 앞선 대화의 첫 올바른 응답 표현 정정'],
 ['QA02','200개 표현/안','G1 100개(입력당 2), G2 50, G3 50, G4 0. A 194/200, B 190/200.','G4는 비적용. QA07 입력 수와 우연히 같은 분모이나 다른 단위'],
 ['QA03','200개 사전 지정 목표/안','G1/G2 원래 요청 t=-0.6 시작; G3/G4 새 목표 t=0. 외부 Agent 구간만 0으로 투영한 DAG 완료경로.','200건 전체 오답 포함. 기존 목표 진단은 별도. 전체 제품 평균 아님'],
 ['QA04','동일 C01~C06','가상 Rust 모듈 목록의 수로 평균 2.0/2.0. 실제 수정 없음.','기존 QA04 정의의 측정 결과가 아닌 설계 예시'],
 ['QA05','별도 COMMON-28 명목 비용 묶음','S01~S28 비용을 명시적으로 배분. 가상 입력 $1/M·출력 $4/M·음성 $0.002/s. VIA만 포함.','실제 단가가 아님. 200건에서 28건으로 환산하지 않음. 5회 실험도 아님'],
 ['QA06','동시 시점 로컬 MB 합의 최고값','cloud 모델 프로필의 로컬 VIA 상태만 설정. A 310 / B 365. 모델 가중치 복제 없음.','온디바이스 Omni/RSS 측정·512MB 준수 증거 아님'],
 ['QA07','200개 입력·revision/안','목표·대상·조건·연결·기존 목표 상태·발송 범위를 최초 확정 때 모두 비교. A192/200 B186/200.','항목 평균 아님. 후속 인계 누락/Agent 업무 품질 제외'],
 ['정확도 가정','오류를 사전에 배정','A 대상오류 입력5+의미만3, B 대상오류 입력7+의미만7. G1은 한 입력이 두 지칭 오류를 만들 수 있음.','오류율의 실증 근거 없음. CI/p-value를 만들지 않음'],
 ['시간 가정','그룹 평균 + 대칭 오프셋','j=0..49, QA01 오프셋=(j-24.5)*0.004초, QA03는 그 1.5배. 평균을 보존하는 작성 규칙.','무작위 네트워크 변동이나 독립 반복 측정 아님'],
 ['QA03 추가 가정','G4의 첫 표시 이후 1.2초 차이','가상 A의 전체 설명 생성 2.35초, B 1.15초. 큰 공통 문맥과 좁은 문맥의 후속 생성 차이를 추가로 설정했다.','step 대기만으로 설명 불가. 이 차이를 제거하면 G4 A=3.20초, 전체 A=4.70초/B=4.40초. 구조 필연 아님'],
 ['종료 조건','위임 결과 연결과 상세 표시/음성 준비','G1~G3는 합성 Agent terminal을 연결. G4는 세 문장 답변의 전체 생성·표시와 음성 준비.','모든 terminal은 설정값; 성공/오류와 시간 분리'],
 ['자원 조건','B의 별도 판단 자원 여유','모델 슬롯 2개를 사용할 수 있는 가정. 입력 수신·조회는 의미 추론과 분리.','K=1 포화에서는 B의 선행 대기 이익 감소. 이 표가 K=1 측정은 아님'],
 ['DP5와 통합','형식 참고 / 숫자 합산 금지','DP5:100입력·정규화 비용(수신자 포함). DP2:200입력·USD 명목묶음(VIA만).','QA 단위·범위·분모·프로필·판본 일치 후에만 합치기'],
 ['정의 출처','로컬 계약','INTENT.md §4.1; qa01-first-response-latency.md; qa03-via-processing-time.md; qa05-model-cost.md; 14-corrected-poc-contract.md','DP5 참고 채팅 codex://threads/01a154e4-11ce-7582-841b-347900047d61']
];
const data={schema_version:'DP-BACKDATA-0.1',dataset_id:VERSION,evidence_kind:KIND,actual_model_calls:0,generation_method:'AUTHORED_TO_MATCH_PREVIOUS_ILLUSTRATIVE_TOTALS',families,cases,sources,responses,referents,times,events,companions,costRows,changes,memory,summary,schema,rules};
await fs.writeFile(path.join(OUT,'synthetic-data.json'),JSON.stringify(data,null,2));

const wb=Workbook.create(),sheets={};
for(const n of ['요약','사례군','응답판정','지칭판정','시간내역','실행단계','비용원장','변경범위','메모리','사례200','공통원본','기존목표진단','평가기준','DP통합필드'])sheets[n]=wb.worksheets.add(n);
const col=n=>{let x='';for(n++;n>0;n=Math.floor((n-1)/26))x=String.fromCharCode(65+(n-1)%26)+x;return x;};
const navy='#243F5B',muted='#596673';
function table(name,title,headers,rows,widths,height=48){const s=sheets[name],last=col(headers.length-1),end=rows.length+5;s.showGridLines=false;s.getRange(`A1:${last}${end}`).format.font={name:'Arial',size:11,color:'#202B36'};s.getRange(`A1:${last}${end}`).format.rowHeight=height;for(const [r,h]of[[1,12],[2,30],[3,24],[4,12]])s.getRange(`A${r}:${last}${r}`).format.rowHeight=h;s.getRange('A2').values=[[title]];s.getRange('A2').format.font={size:15,bold:true,color:navy};s.getRange('A3').values=[[notice]];s.getRange('A3').format.font={size:10,color:'#9C5600'};s.getRange(`A5:${last}5`).values=[headers];s.getRange(`A5:${last}5`).format={fill:navy,font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},rowHeight:44,wrapText:true};if(rows.length)s.getRange(`A6:${last}${end}`).values=rows;s.getRange(`A6:${last}${end}`).format.wrapText=true;s.getRange(`A6:${last}${end}`).format.verticalAlignment='center';widths.forEach((w,i)=>s.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidthPx=w);s.freezePanes.freezeRows(5);s.freezePanes.freezeColumns(2);s.tables.add(`A5:${last}${end}`,true,`DP2Table${Object.keys(sheets).indexOf(name)}`);return s;}
table('사례군','네 유형과 판정 경계',['유형','이름','입력 수','지칭 수','QA03 시작','A 처리','B 처리','완료 조건','가정'],families.map(f=>[f.id,f.name,50,50*f.refs,f.start,f.id==='G4'?'현재 조회 step 후 독립 질문 판단':'현재 조회 step 후 같은 문맥에서 새 입력 판단',f.id==='G1'?'먼저 관계 판단 → 조회 결과 대기 → 결합':f.id==='G4'?'공통 대화에서 즉시 독립 답변':'조건/추가 의도 먼저 판단 → 조회 결과와 전체 의미 결합',f.id==='G4'?'세 문장 설명 전체 표시와 음성 준비':'요청 상태·합성 Agent 결과 연결·상세 표시·음성 준비',f.explain]),[80,180,90,90,110,380,410,380,400],110);
const rs=table('응답판정','최초 표시와 최초 채택 의미',['사례 ID','방안','유형','첫 구체적 내용','정답 필수 의미 JSON','최초 채택 의미 JSON','QA07 성공','오류 분류','오류 이유','이후 상세 내용'],responses.map(r=>[r.case_id,r.option,r.group,r.first_content,canon(r.oracle),canon(r.adopted),null,r.error,r.error_reason,r.final_text]),[210,65,65,520,780,780,90,150,380,620],160);
responses.forEach((r,i)=>rs.getRange(`G${i+6}`).formulas=[[`=IF(E${i+6}=F${i+6},1,0)`]]);
rs.getRange('G6:G405').conditionalFormats.add('cellIs',{operator:'equal',formula:0,format:{fill:'#FBE7E5',font:{color:'#A32926'}}});
const refSheet=table('지칭판정','표현별 대상 집합 일치',['사례 ID','방안','표현 ID','지칭 표현','정답 집합','최초 집합','QA02 성공','revision'],referents.map(r=>[r.case_id,r.option,r.expression_id,r.phrase,canon(r.expected),canon(r.adopted),null,r.revision]),[210,65,240,190,480,480,95,85],65);
referents.forEach((r,i)=>refSheet.getRange(`G${i+6}`).formulas=[[`=IF(E${i+6}=F${i+6},1,0)`]]);
refSheet.getRange('G6:G405').conditionalFormats.add('cellIs',{operator:'equal',formula:0,format:{fill:'#FBE7E5',font:{color:'#A32926'}}});
const ts=table('시간내역','첫 표시와 VIA 완료 경로 · 초',['사례 ID','방안','유형','원래 목표 시작 s','정정 전 처리 s','입력 접수 s','별도 관계 판단 s','잔여 대기 s','첫 내용 판단 s','첫 표시 처리 s','QA01 s','전체 의미 확정 t','첫 표시 후 추가 결합 s','후속 준비/전체설명 s','전달·접수 s','상세 연결 s','음성 준비 s','QA03 s','외부 업무 제외 s','목표 전체경과 s','기존 조회 반환 t'],times.map(t=>[t.case_id,t.option,t.group,t.goal_start_s,t.prior,t.intake,t.relation,t.wait,t.interpret,t.display,null,t.full_meaning_at,t.late,t.remaining,t.delivery,t.result,t.voice,null,t.external,null,t.read_return]),[210,65,65,...Array(18).fill(135)],36);
times.forEach((t,i)=>{const r=i+6;ts.getRange(`K${r}`).formulas=[[`=SUM(F${r}:J${r})`]];ts.getRange(`R${r}`).formulas=[[`=E${r}+K${r}+SUM(M${r}:Q${r})`]];ts.getRange(`T${r}`).formulas=[[`=R${r}+S${r}`]];});ts.getRange('D6:U405').setNumberFormat('0.000');
table('실행단계','의존 DAG · 외부 구간은 귀속시간 0',['사건 ID','사례 ID','방안','단계','역할','선행 사건','기간 s','시작 s','끝 s','QA03 포함','투영 완료 s'],events.map(e=>[e.event_id,e.case_id,e.option,e.label,e.role,e.parents.join('\n'),e.duration_s,e.start_s,e.finish_s,e.count_in_qa03?1:0,e.projected_finish_s]),[320,210,65,300,130,620,110,110,110,100,120],70).getRange(`G6:K${events.length+5}`).setNumberFormat('0.000');
const cs=table('비용원장','별도 COMMON-28 명목 비용 · 가상 단가',['항목 ID','방안','묶음 ID','S ID','공통 시나리오','단계','청구 bucket','가상 사용량','가상 단가 USD','가상 비용 USD','범위'],costRows.map(c=>[c.cost_id,c.option,c.bundle_id,c.catalog_id,c.catalog_title,c.stage,c.bucket,c.quantity,c.unit_price,null,c.scope]),[280,65,240,65,340,200,210,130,140,140,110],54);
costRows.forEach((c,i)=>cs.getRange(`J${i+6}`).formulas=[[`=H${i+6}*I${i+6}`]]);cs.getRange(`H6:H${costRows.length+5}`).setNumberFormat('0.00000');cs.getRange(`I6:J${costRows.length+5}`).setNumberFormat('$0.000000');
table('변경범위','가상 변경 범위 · 실제 수정 없음',['방안','변경 ID','변경 과제','가상 모듈 목록','개수','근거 상태'],changes.map(c=>[c.option,c.change_id,c.title,c.modules.join('\n'),c.count,KIND]),[65,90,340,520,90,360],85);
const ms=table('메모리','동시 시점 로컬 사용량 · 가상 MB',['방안','snapshot ID','단계','공통 기반 MB','입력 버퍼 MB','문맥 MB','결과 버퍼 MB','실행 상태 MB','동시 합 MB','프로필'],memory.map(m=>[m.option,m.snapshot_id,m.stage,m.base,m.input_buffer,m.context,m.result_buffer,m.execution,null,m.profile]),[65,200,170,120,120,120,120,130,120,480],48);memory.forEach((m,i)=>ms.getRange(`I${i+6}`).formulas=[[`=SUM(D${i+6}:H${i+6})`]]);
table('사례200','200개 합성 입력과 정답',['사례 ID','유형','프로젝트','표현 변형','원래 사용자 요청','후속 사용자 입력','정답 필수 의미','조회 반환 t','QA03 목표 ID','QA03 시작 t','근거 IDs','COMMON 주제 대응'],cases.map(c=>[c.case_id,c.group,c.project,c.variant,c.initial_input,c.input,canon(c.oracle),c.read_return_s,c.goal_id,c.goal_start_s,c.source_ids.join('\n'),c.common_mapping]),[210,65,95,85,440,520,800,110,300,110,380,180],165);
table('공통원본','양안에 동일한 합성 입력·근거',['근거 ID','사례 ID','종류','합성 원문','사용 가능 t','상태'],sources.map(s=>[s.id,s.case_id,s.type,s.payload,s.available_at_s,s.evidence_kind]),[280,210,180,660,115,350],80);
table('기존목표진단','새 요청 처리 중 원래 목표의 보존',['사례 ID','방안','원래 목표 ID','처음 채택한 유지 상태','조회 반환 t','가상 VIA 완료 s','주표 포함','종료 상태','비고'],companions.map(c=>[c.case_id,c.option,c.goal_id,c.status_adopted,c.read_return_s,c.synthetic_via_completion_s,0,c.terminal_kind,c.reason]),[210,65,280,180,120,140,100,330,550],75);
table('평가기준','단위·분모·경계·가정',['항목','관측 단위/이름','포함·작성 규칙','제외·해석 한계'],rules,[180,290,800,700],90);
table('DP통합필드','후속 DP 통합을 위한 공통 필드 초안',['필드','적용 표','값/의미','통합 시 확인'],schema,[350,180,630,650],70);
const s=sheets['요약'];s.showGridLines=false;s.tabColor=navy;s.getRange('A1:F45').format.font={name:'Arial',size:11,color:'#202B36'};s.getRange('A1:F45').format.rowHeight=32;[280,130,130,130,150,650].forEach((w,i)=>s.getRange(`${col(i)}1:${col(i)}45`).format.columnWidthPx=w);s.getRange('A2').values=[['DP2 합성 백데이터']];s.getRange('A2').format.font={size:16,bold:true,color:navy};s.getRange('A3').values=[[notice]];s.getRange('A3').format.font={size:10,color:'#9C5600'};
function header(row,labels){s.getRange(`A${row}:F${row}`).values=[labels];s.getRange(`A${row}:F${row}`).format={fill:navy,font:{name:'Arial',size:11,color:'#FFFFFF',bold:true},rowHeight:36};}
header(5,['지표','단위','A','B','분모/범위','가상 결과의 설명']);
const summaryRows=[['QA01 첫 응답','초',null,null,'200입력/안','첫 구체적 표시. 오답 포함; 첫 올바른 응답까지가 아님'],['QA02 지칭 정확도','일치율',null,null,'200표현/안','입력과 별도 분모. G1은 2표현, G4는 비적용'],['QA03 VIA 처리','초',null,null,'200목표/안','정정의 원래 시작점 보존. 외부 업무 제외 DAG 완료경로'],['QA04 변경 범위','모듈',null,null,'6가상 변경','실제 diff 없음. 가상 경로 수의 평균'],['QA05 모델 비용','USD/묶음',null,null,'별도28목록','가상 단가와 사용량. VIA만 포함. 200입력과 환산 금지'],['QA06 최대 RAM','MB',null,null,'cloud 프로필','가상 동시 사용량. 온디바이스 Omni 메모리 증거 아님'],['QA07 의미 정확도','전체 일치율',null,null,'200입력/안','최초 필수 의미 전체. A192/200, B186/200']];s.getRange('A6:F12').values=summaryRows;
for(const [o,c]of [['A','C'],['B','D']]){
 const formulas=[`=AVERAGEIF('시간내역'!$B$6:$B$405,"${o}",'시간내역'!$K$6:$K$405)`,`=AVERAGEIF('지칭판정'!$B$6:$B$405,"${o}",'지칭판정'!$G$6:$G$405)`,`=AVERAGEIF('시간내역'!$B$6:$B$405,"${o}",'시간내역'!$R$6:$R$405)`,`=AVERAGEIF('변경범위'!$A$6:$A$17,"${o}",'변경범위'!$E$6:$E$17)`,`=SUMIF('비용원장'!$B$6:$B$${costRows.length+5},"${o}",'비용원장'!$J$6:$J$${costRows.length+5})`,o==='A'?"=MAX('메모리'!I6:I9)":"=MAX('메모리'!I10:I13)",`=AVERAGEIF('응답판정'!$B$6:$B$405,"${o}",'응답판정'!$G$6:$G$405)`];formulas.forEach((f,i)=>s.getRange(`${c}${i+6}`).formulas=[[f]]);
}
s.getRange('C6:D12').setNumberFormat('0.00');for(const r of [7,12])s.getRange(`C${r}:D${r}`).setNumberFormat('0%');s.getRange('C10:D10').setNumberFormat('$0.00');s.getRange('C11:D11').setNumberFormat('0');
header(15,['유형별 비교','지표','A','B','단위','차이가 생기는 곳']);
let row=16;for(const f of families)for(const qa of ['QA01','QA03','QA07']){s.getRange(`A${row}:F${row}`).values=[[f.name,qa,null,null,qa==='QA07'?'일치율':'초',f.explain]];for(const [o,c]of [['A','C'],['B','D']]){const sh=qa==='QA07'?'응답판정':'시간내역',val=qa==='QA07'?'G':qa==='QA01'?'K':'R';s.getRange(`${c}${row}`).formulas=[[`=AVERAGEIFS('${sh}'!$${val}$6:$${val}$405,'${sh}'!$B$6:$B$405,"${o}",'${sh}'!$C$6:$C$405,"${f.id}")`]];}s.getRange(`C${row}:D${row}`).setNumberFormat(qa==='QA07'?'0%':'0.00');row++;}
header(29,['오류 단위','분류','A','B','집계 단위','한 입력의 두 지칭 오류를 분리']);
const errs=[['대상 오류 입력','TARGET'],['의미만 오류 입력','MEANING_ONLY'],['모든 의미 일치','NONE']];errs.forEach(([label,error],i)=>{const rr=30+i;s.getRange(`A${rr}:F${rr}`).values=[[label,error,null,null,'입력','QA07 분모 200; 지칭 오류 표현 수와 다름']];for(const[o,c]of[['A','C'],['B','D']])s.getRange(`${c}${rr}`).formulas=[[`=COUNTIFS('응답판정'!$B$6:$B$405,"${o}",'응답판정'!$H$6:$H$405,"${error}")`]];});
s.getRange('A35').values=[['DP 통합: 정의/분모/비용 범위/프로필을 맞춘 뒤 비교. DP5의 정규화 비용을 이 USD와 합산하지 않음.']];s.getRange('A37').values=[['200입력 = 20개 표현 템플릿 × 10개 프로젝트. 독립 실험 표본 200회가 아님.']];s.getRange('A39').values=[['숫자가 맞는지 검증했지만, 실제 구조 우위나 제품 성능을 검증한 자료는 아님.']];
s.getRange('A41').values=[['G4의 후속 설명 생성 1.2초 차이는 별도 가정. 제거하면 QA03 전체 A=4.70초, B=4.40초.']];
s.getRange('E6:E32').format.horizontalAlignment='center';
for(const r of [5,15,29])s.getRange(`B${r}:E${r}`).format.horizontalAlignment='center';
// Keep long Korean timing headers readable after table creation.
ts.getRange('M5:N5').values=[['추가 의미 결합 s','후속 준비·설명 s']];
ts.getRange('D5:U5').format.wrapText=true;
ts.getRange('L1:U405').format.columnWidthPx=155;
cs.getRange(`K6:K${costRows.length+5}`).format.horizontalAlignment='center';
ms.getRange('J6:J13').values=memory.map(()=>['cloud 모델 / 로컬 VIA / 모델 가중치 제외']);
wb.recalculate();
for(const[o,c]of[['A','C'],['B','D']])for(let i=0;i<7;i++)assert(Math.abs(s.getRange(`${c}${6+i}`).values[0][0]-summary[o][`qa0${i+1}`])<1e-8);
// Verify a nontrivial dependency: input time changes both calculated delay and whole goal time.
const old=ts.getRange('F6').values[0][0];ts.getRange('F6').values=[[old+.1]];assert(Math.abs(s.getRange('C6').values[0][0]-1.4005)<1e-8);assert(Math.abs(s.getRange('C8').values[0][0]-5.0005)<1e-8);ts.getRange('F6').values=[[old]];
const original=refSheet.getRange('F6').values[0][0];refSheet.getRange('F6').values=[['["SYNTHETIC-INVALID-TARGET"]']];assert(Math.abs(s.getRange('C7').values[0][0]-.965)<1e-8);refSheet.getRange('F6').values=[[original]];
wb.recalculate();
await fs.writeFile(path.join(OUT,'summary-inspection.ndjson'),(await wb.inspect({kind:'table',range:'요약!A5:F12',include:'values,formulas',tableMaxRows:8,tableMaxCols:6,maxChars:6000})).ndjson);
await fs.writeFile(path.join(OUT,'formula-scan.ndjson'),(await wb.inspect({kind:'match',searchTerm:'#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A|#NUM!|#NULL!|#SPILL!|#CALC!',options:{useRegex:true,maxResults:50},maxChars:3000})).ndjson);
const previews=[['요약','A1:F13'],['요약','A15:F32'],['사례군','A1:F9'],['응답판정','A1:D8'],['응답판정','G5:J8'],['지칭판정','A1:G8'],['시간내역','A1:K9'],['시간내역','L5:U9'],['실행단계','A1:G10'],['비용원장','A1:F8'],['비용원장','G5:K9'],['변경범위','A1:F9'],['메모리','A1:J13'],['사례200','A1:F8'],['공통원본','A1:F9'],['기존목표진단','A1:H9'],['평가기준','A1:D9'],['DP통합필드','A1:D9']];
for(let i=0;i<previews.length;i++){const [sheetName,range]=previews[i];const blob=await wb.render({sheetName,range,scale:1,format:'png'});await fs.writeFile(path.join(OUT,'previews',`${String(i+1).padStart(2,'0')}-${sheetName}.png`),new Uint8Array(await blob.arrayBuffer()));}
await(await SpreadsheetFile.exportXlsx(wb)).save(path.join(DELIVER,'DP2-synthetic-backdata.xlsx'));

const esc=x=>String(x).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;').replaceAll('"','&quot;');
function htable(headers,rows){return `<table><thead><tr>${headers.map(h=>`<th>${esc(h)}</th>`).join('')}</tr></thead><tbody>${rows.map(r=>`<tr>${r.map(c=>`<td>${esc(c)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;}
const sampleHtml=families.map(f=>{const c=cases.filter(x=>x.group===f.id)[{G1:13,G2:10,G3:15,G4:26}[f.id]],rr=responses.filter(x=>x.case_id===c.case_id),tt=times.filter(x=>x.case_id===c.case_id);return `<section><h2>${f.id} ${f.name}</h2><p><b>원래 요청:</b> ${esc(c.initial_input)}</p><blockquote>${esc(c.input)}</blockquote><p>원본 P: D-1 09:10 / Q: D-1 15:40 / R: D-7 도착이나 파일 수정은 D0. 조회 반환 t=${c.read_return_s}초. A/B는 같은 근거를 받는다.</p><p><b>지칭 정답:</b> ${esc(c.referents.map(x=>x[1]+' → '+x[2].join(', ')).join(' ; ')||'비적용 — 명시된 일반 개념 질문')}</p><pre>${esc(JSON.stringify(c.oracle,null,2))}</pre>${htable(['방안','첫 표시 문장','QA01','전체 의미 확정 t','QA03','QA07','오류'],rr.map((r,i)=>[r.option,r.first_content,tt[i].first.toFixed(3)+'s',tt[i].full_meaning_at.toFixed(3)+'s',tt[i].via.toFixed(3)+'s',r.qa07?'성공':'실패',r.error_reason]))}<p>이 예시의 시각과 응답은 합성이다. 문장에 나타나지 않은 필수 의미도 최초 채택 JSON으로 판정한다.</p></section>`;}).join('');
const timeFields=[['정정 전 처리','prior'],['입력 접수','intake'],['별도 관계 판단','relation'],['첫 판단 선행 대기','wait'],['첫 내용 판단','interpret'],['첫 표시','display'],['첫 표시 후 추가 결합','late'],['후속 준비/전체 설명','remaining'],['전달·접수','delivery'],['결과 연결','result'],['전체 음성 준비','voice']];
const html=`<!doctype html><html lang="ko"><meta charset="utf-8"><title>DP2 합성 백데이터 설명</title><style>body{font:16px/1.65 -apple-system,BlinkMacSystemFont,'Arial',sans-serif;color:#243447;max-width:1280px;margin:40px auto;padding:0 28px}h1,h2{color:#203e59}h1{font-size:32px}h2{margin-top:40px}table{width:100%;border-collapse:collapse;font-size:14px;margin:18px 0}th{background:#243f5b;color:white;text-align:left}th,td{padding:12px;border-bottom:1px solid #dae2e9;vertical-align:top;overflow-wrap:anywhere}tr:nth-child(even){background:#f4f7fa}.notice{padding:16px;background:#fff2d9;border-left:5px solid #b77817}blockquote{border-left:4px solid #527c9c;padding:10px 20px;background:#eef4f8}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#f5f7f9;padding:16px;font-size:13px}section{border-bottom:1px solid #d8e1e8;padding-bottom:24px}a{color:#1467a6}</style><h1>DP2 A/B 합성 백데이터</h1><p class="notice">${notice}. 앞선 가상 평균을 설명하기 위해 상세 행을 역구성했다. 문서 생성일 2026-10-21은 실험일이 아니다. 실측 근거·통계적 유의성·후보 선택을 제공하지 않는다.</p><p>A는 기존 해석 실행의 다음 step에서 새 입력을 함께 판단한다. B는 공통 대화가 별도로 새 입력을 판단하고 기존 실행 또는 새 목표에 연결한다. B에 새 모델 가중치나 매 입력 새 loop를 강제하지 않는다.</p>${htable(['QA','A','B','단위'],Object.keys(summary.A).filter(k=>k.startsWith('qa')).map(k=>[k.toUpperCase(),[ 'qa02','qa07'].includes(k)?(summary.A[k]*100).toFixed(0)+'%':summary.A[k].toFixed(2),['qa02','qa07'].includes(k)?(summary.B[k]*100).toFixed(0)+'%':summary.B[k].toFixed(2),({qa01:'초',qa02:'표현 집합 일치율',qa03:'초',qa04:'가상 모듈',qa05:'가상 USD/명목28묶음',qa06:'가상 MB',qa07:'입력 전체 의미 일치율'})[k]]))}<h2>구성 및 분모</h2><p>네 유형 각 50입력, 총 200입력이다. 20개 표현 템플릿을 10개 가상 프로젝트에 적용했다. 독립적인 실제 시행 200회가 아니다. A/B 응답 400행, 지칭 판정 400행, 시간 내역 400행을 연결했다.</p><p>QA02는 G1의 표현 2개×50 + G2 50 + G3 50 = 200표현/안이다. G4는 비적용이다. QA07은 200입력/안으로 모든 필수 의미가 맞아야 성공이다. A 대상오류 입력 5건이 지칭오류 6건을 만들며 의미만 오류는 3건이다. B 대상오류 입력 7건이 지칭오류 10건을 만들며 의미만 오류는 7건이다.</p><p>QA01은 오답을 포함한 첫 구체적 내용 표시까지다. G2/G3의 B는 변경 조건 또는 추가 목표 의도를 먼저 안내하고, 원본 조회 뒤 대상까지 포함한 전체 의미를 확정할 수 있다. 단순 접수 인사는 세지 않는다.</p>${sampleHtml}<h2>시간 분해와 병행 경로</h2>${htable(['평균 단계','A 초','B 초'],timeFields.map(([label,key])=>[label,mean(times.filter(t=>t.option==='A').map(t=>t[key])).toFixed(3),mean(times.filter(t=>t.option==='B').map(t=>t[key])).toFixed(3)]))}<p>QA01 = 접수 + 별도 관계 판단 + 선행 대기 + 첫 내용 판단 + 표시. QA03 = 정정 전 원래 목표 처리 + QA01 + 이후 결합/준비/전달/결과/음성 경로. 실제 식은 실행단계 DAG에서 외부 업무 노드 시간을 0으로 만든 완료경로로 검산했다. 조회와 관계 판단의 겹친 시간을 두 번 더하지 않는다.</p><p>G1/G2는 원래 목표 t=-0.6부터 QA03를 계산한다. G3/G4는 새 목표 t=0부터 계산한다. 원래 목표의 유지와 경합은 별도 기존목표진단 표에 남겼다. 주표는 사전 지정한 200개 목표만의 합성 평균이며 전체 시스템의 모든 목표 평균이 아니다.</p><p>각 그룹의 시간은 이전 가상 평균에 대칭 오프셋을 더한 것이다. QA01 오프셋=(j−24.5)×0.004초, QA03는 그 1.5배다. 이 변형을 실제 반복 측정의 분산처럼 해석하면 안 된다. 모든 외부 처리와 completion 역시 설정값이다.</p><h2>비용 및 다른 QA</h2>${htable(['비용 단계','A USD','B USD'],Object.keys(names).map(k=>[names[k],costTotals.A[k].toFixed(2),costTotals.B[k].toFixed(2)]))}<p>비용은 200건에서 외삽하지 않았다. 기존 S01~S28 명칭을 읽어 별도 명목 원장을 만들고 합계를 맞췄다. 가상 단가는 입력 $1/M token, 출력 $4/M token, 음성 $0.002/초다. 실제 provider 가격이 아니며 5회 묶음을 실행하지 않았다. 분할 전 목표 금액에서 가상 usage를 역구성했다. Downstream 비용은 포함하지 않는다.</p><p>QA04의 경로는 SYNTHETIC_DESIGN_ONLY 이름의 가상 모듈이며 실제 Rust 수정이 아니다. QA06은 cloud 모델을 사용하는 로컬 프로필의 가상 동시 합이다. 온디바이스 Omni 가중치/KV를 측정하거나 512MB 요구 충족을 증명하지 않는다. 시점별 총합의 최대치를 사용하며 구성요소의 서로 다른 시점 최고치를 합산하지 않는다.</p><h2>DP5 및 향후 DP 통합</h2><p>참고 채팅: <a href="codex://threads/01a154e4-11ce-7582-841b-347900047d61">DP5 = DP4+1+45</a>. 사례→원본→최초 응답/의미→시간→비용 연결 방식을 참고했다. DP5 파일은 변경하지 않았다.</p><p>DP5의 비용은 정규화 단위이고 수신 Agent 비용을 포함한다. DP2는 가상 USD 명목묶음에서 VIA만 포함하므로 숫자를 직접 비교할 수 없다. 두 자료의 입력 수나 적용 표현 수도 다르다. 공통 스키마 DP-BACKDATA-0.1에 evidence_kind, DP/대안, 사례/요청/revision/목표/표현 ID, oracle/최초 채택, DAG, 비용 범위, 프로필을 두고 통합 전에 판본과 분모를 검토하도록 했다.</p><h2>확인한 범위</h2><p>평균·분모·오류 건수·정답/채택 비교·근거 참조·DAG 경로·가상 비용·시점별 메모리 합계와 스프레드시트 수식의 재계산을 확인했다. 모델/제품 실행이나 실측 결과의 유효성을 확인한 것은 아니다. 실제 데이터는 별도 dataset_id와 evidence_kind로 수집해야 한다.</p></html>`;
await fs.writeFile(path.join(DELIVER,'DP2-backdata-guide.html'),html.replace('<h2>비용 및 다른 QA</h2>','<p class="notice"><b>QA03의 추가 가정:</b> 독립 질문 G4의 첫 표시 이후 전체 설명 생성은 A 2.35초, B 1.15초로 설정했다. 큰 공통 문맥과 좁은 문맥의 후속 생성 차이를 가정한 것으로, 기존 step 대기만으로 설명되는 차이가 아니다. 이 1.20초 차이를 제거하면 G4의 A는 3.20초, 전체 A는 4.70초가 되고 B는 4.40초를 유지한다. 실측에서는 이 별도 가정을 검증해야 한다.</p><h2>비용 및 다른 QA</h2>'));
await fs.writeFile(path.join(OUT,'validation.json'),JSON.stringify({evidence_kind:KIND,schema_version:'DP-BACKDATA-0.1',cases:cases.length,response_rows:responses.length,referent_rows:referents.length,time_rows:times.length,event_rows:events.length,cost_rows:costRows.length,source_rows:sources.length,summary,source_refs_resolve:true,canonical_oracle_checks:true,dag_projection_matches:true,formula_perturbation_checked:true,actual_model_calls:0,actual_measurement_runs:0,native_excel_verified:false,notes:'Verification of authored consistency, not evidence of architecture performance.'},null,2));
console.log(JSON.stringify({workbook:path.join(DELIVER,'DP2-synthetic-backdata.xlsx'),guide:path.join(DELIVER,'DP2-backdata-guide.html'),summary,counts:{cases:cases.length,events:events.length,costRows:costRows.length}}));
