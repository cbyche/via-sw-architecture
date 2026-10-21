// Authored synthetic design evaluation. This program does not call a model or run VIA.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import {buildScenarios} from './common_qa_synthetic_scenarios.mjs';
import {renderReport} from './render_common_qa_synthetic_report.mjs';

const ROOT=path.resolve(path.dirname(new URL(import.meta.url).pathname),'../..');
const DOC=path.join(ROOT,'docs/architecture/12-decisions/decision-packages/qa-common-synthetic-20261021');
const OUT=path.join(ROOT,'docs/presentations_files/qa-common-synthetic-20261021');
const DELIVERY=path.join(OUT,'outputs/01a15d51');
const KIND='SYNTHETIC_AUTHORED_NOT_MEASURED';
const VERSION='COMMON-QA-SYN-1.0';
const assumptions={
  model_name:'공통 가상 cloud 의미 모델',input_usd_per_million:1,output_usd_per_million:4,audio_usd_per_second:0.002,
  call_startup_s:0.16,prefill_tokens_per_second:3500,decode_tokens_per_second:160,
  intake_s:0.06,publish_s:0.06,external_agent_s:8,overlap_input_gap_s:0.35,sequential_input_gap_s:1.5,
  max_concurrent_semantic_calls:2,source_read_base_s:1.85,source_read_variant_s:0.045,
  audio_generation_fixed_s:0.18,audio_generation_s_per_audio_second:0.055,
  core_resident_mb:238,request_state_mb:11,client_call_base_mb:15,client_mb_per_1000_input_tokens:4,
  local_read_buffer_mb:7,relation_cache_mb:38,relation_producer_state_mb:19,
  production_start_s:-8,update_policy:'관계 갱신은 두 입력의 모든 목표 처리 종료 후 유휴 구간에서 실행. 준비와 갱신의 비용 및 RAM은 포함.',
  source_status:'모든 지연, 사용량, 단가, 상태 크기는 작성 가정. 실제 모델 속도, 청구서, RAM 관측이 아님.'
};
const configs=[{id:'BASE',dp2:'A',dp5:'A',label:'공통 기준 2A 5A'},{id:'EXEC',dp2:'B',dp5:'A',label:'의미 실행 변경 2B 5A'},{id:'REL',dp2:'A',dp5:'B',label:'관계 생산 변경 2A 5B'}];
const corpus=buildScenarios();
const changeSpec=JSON.parse(await fs.readFile(path.join(DOC,'change-spec.json'),'utf8'));
const families=new Map(corpus.families.map(f=>[f.id,f]));
const round=(x,n=6)=>Number(x.toFixed(n));
const clone=x=>JSON.parse(JSON.stringify(x));
const canon=x=>JSON.stringify(x);
const sameSet=(a,b)=>canon([...a].sort())===canon([...b].sort());
const mean=a=>a.reduce((s,x)=>s+x,0)/a.length;
const costs=[],responses=[],referents=[],times=[],goals=[],nodes=[],memory=[],errorRules=[];
const allInputs=corpus.episodes.flatMap(e=>e.inputs.map(i=>({...i,episode_id:e.id,family_id:e.family_id,variant:e.variant})));

// Explicit error assignments are authored hypotheses, not fitted empirical probabilities.
const targetBase={F02:[1,7],F03:[2,8],F04:[3,9],F05:[4],F07:[6],F09:[2,5],F11:[6,8]};
const targetRel={F03:[8],F04:[9],F05:[4],F09:[2,5],F11:[6,8]};
function verdict(ep,input,index,cfg){
  const adopted=clone(input.meaning), refs=clone(input.referents);
  const family=ep.family_id,v=ep.variant;
  let error='NONE',reason='필수 의미와 지칭 집합 일치';
  const targetIndex=['F02','F03','F04','F07'].includes(family)?0:1;
  let failTarget=index===targetIndex&&(cfg.id==='REL'?targetRel:targetBase)[family]?.includes(v);
  if(cfg.id==='EXEC'&&index===1&&['F10','F11'].includes(family)&&[1,5].includes(v))failTarget=true;
  if(failTarget&&refs.length){
    const ref=refs[refs.length-1];
    ref.adopted=clone(ref.alternative);
    adopted.targets=adopted.targets.map(t=>ref.expected.includes(t)?ref.alternative[0]:t);
    for(const expected of ref.expected)adopted.data_source=adopted.data_source.replaceAll(expected,ref.alternative[0]);
    if(sameSet(adopted.targets,input.meaning.targets))adopted.targets=clone(ref.alternative);
    error='TARGET';reason=cfg.id==='REL'?'게시 관계 또는 원본 보완에서 자료 판본을 잘못 연결':'현재 입력과 원본의 대상 관계를 잘못 연결';
  }
  let failMeaning=index===1&&((family==='F05'&&v===7)||(family==='F07'&&v===4)||(cfg.id!=='EXEC'&&family==='F12'&&[2,7].includes(v)));
  if(cfg.id==='EXEC'&&index===1&&['F06','F10','F11'].includes(family)&&[0,3,7].includes(v))failMeaning=true;
  if(cfg.id==='REL'&&index===1&&['F06','F07','F10'].includes(family)&&[0,3,6,9].includes(v))failMeaning=true;
  if(!failTarget&&failMeaning){
    if(family==='F12'){adopted.conditions.push('기존 발표자료의 대상 기간 안에서만 답변');reason='독립 지식 질문에 진행 중인 발표자료의 기간 제한을 불필요하게 적용';}
    else if(family==='F06'){
      adopted.conditions=adopted.conditions.filter(c=>!c.includes('위험 우선')&&!c.includes('미확인 사항 우선')&&!c.endsWith('용'));
      adopted.conditions.push(ep.inputs[0].meaning.conditions.find(c=>c.endsWith('용')),'장점 우선');
      reason='목적 변경을 놓쳐 과거 청중과 장점 우선 순서를 유지';
    }else if(family==='F07'){
      adopted.conditions=adopted.conditions.filter(c=>!c.includes('구성만 재사용')&&!c.includes('가격과 문체')&&!c.includes('장기 선호')&&!c.includes('이번 작업만'));
      adopted.conditions.push('가격과 문체 재사용 허용','문서 Agent를 장기 선호로 저장');
      reason='이번 작업의 구성 선호를 가격과 문체 및 장기 Agent 선호로 확대';
    }else if(family==='F10'){
      adopted.data_source=ep.inputs[0].meaning.targets[0];
      adopted.conditions=adopted.conditions.filter(c=>!c.includes('과거 자료 내용 혼합 금지'));
      adopted.conditions.push('과거 자료 수치도 요약에 포함');
      reason='신규 조사 대상은 알지만 요약에 사용할 내용 출처를 과거 자료와 혼합';
    }else{
      const omitted=adopted.conditions.find(c=>c.includes('제외'))??adopted.conditions.at(-1);
      adopted.conditions=adopted.conditions.filter(c=>c!==omitted);
      reason=`기존 상태와 정정 조건 결합 중 필수 조건 누락: ${omitted}`;
    }
    error='MEANING_ONLY';
  }
  for(const ref of refs){ref.adopted??=clone(ref.expected);ref.correct=Number(sameSet(ref.expected,ref.adopted));}
  const ok=Number(canon(adopted)===canon(input.meaning)&&refs.every(r=>r.correct));
  assert.equal(ok,Number(error==='NONE'));
  if(error!=='NONE')errorRules.push({config:cfg.id,input_id:input.id,error,reason,assigned:true});
  return {adopted,refs,error,reason,ok};
}

function callDuration(inTokens,outTokens){return assumptions.call_startup_s+inTokens/assumptions.prefill_tokens_per_second+outTokens/assumptions.decode_tokens_per_second;}
function usage(ep,cfg,input,stage,inTokens,outTokens,audioSeconds=0,role='VIA',note='공통 가상 단가와 작성 사용량'){
  const item={id:`C${String(costs.length+1).padStart(5,'0')}`,episode_id:ep.id,config:cfg.id,input_id:input?.id??'',stage,role,input_tokens:inTokens,output_tokens:outTokens,audio_seconds:audioSeconds,note};
  item.cost_usd=inTokens*assumptions.input_usd_per_million/1e6+outTokens*assumptions.output_usd_per_million/1e6+audioSeconds*assumptions.audio_usd_per_second;
  costs.push(item);return item;
}
function workload(ep,input,index,cfg){
  const f=families.get(ep.family_id),v=ep.variant;
  const simple=['explicit','confirmed'].includes(f.relation_state),fresh=['dirty','unproduced'].includes(f.relation_state);
  let inTokens=simple?1050:2650+(Number(f.id.slice(1))%4)*280;
  let outTokens=simple?78:116;
  if(cfg.dp5==='B'&&!simple&&!fresh){inTokens=1050+(Number(f.id.slice(1))%3)*90;outTokens=86;}
  if(cfg.dp5==='B'&&fresh){inTokens+=190;outTokens+=12;}
  if(index===1){inTokens=Math.round(inTokens*0.76)+210;outTokens+=5;}
  if(index===1&&f.second_kind==='independent'){inTokens=cfg.dp2==='B'?570:1920;outTokens=62;}
  inTokens+=v*23;outTokens+=v%4;
  return {inTokens,outTokens,lookup_s:simple?0.10:cfg.dp5==='B'&&!fresh?0.12:0.28+(v%3)*0.025};
}

for(const ep of corpus.episodes){
 const f=families.get(ep.family_id);
 for(const cfg of configs){
  const en=[],byId=new Map();let lastInteractiveModel=null;
  function add(label,role,duration,parents=[],meta={}){
    const p=parents.map(x=>typeof x==='string'?byId.get(x):x);
    assert(p.every(Boolean),`missing parent ${label}`);
    const start=p.length?Math.max(...p.map(n=>n.finish_s)):(meta.start_at??0);
    const n={id:`${ep.id}-${cfg.id}-N${String(en.length+1).padStart(2,'0')}`,episode_id:ep.id,config:cfg.id,label,role,duration_s:round(duration),start_s:round(start),finish_s:round(start+duration),parents:p.map(n=>n.id),...meta};
    en.push(n);byId.set(n.id,n);return n;
  }
  function model(input,stage,parents,ins,outs){
    const cost=usage(ep,cfg,input,stage,ins,outs);
    const dependencies=[...parents];
    if(input&&cfg.dp2==='A'&&lastInteractiveModel)dependencies.push(lastInteractiveModel);
    const node=add(stage,input?'VIA_MODEL':'VIA_PRODUCER',callDuration(ins,outs),dependencies,{input_id:input?.id??'',input_tokens:ins,output_tokens:outs,cost_id:cost.id});
    cost.node_id=node.id;
    if(input)lastInteractiveModel=node;
    return node;
  }
  let prepEnd=null;
  if(cfg.dp5==='B'){
    const history=add('요청 전 공통 이력 준비','HISTORY_SOURCE',0,[],{start_at:assumptions.production_start_s});
    const used=model(null,'사용 관계 사전 생산',[history],4200+ep.variant*80,260+ep.variant*5);
    prepEnd=model(null,'미사용 관계 사전 생산',[used],2400+ep.variant*50,170+ep.variant*3);
    assert(prepEnd.finish_s<0);
  }
  const u1=ep.inputs[0],u2=ep.inputs[1];
  const source1=add('첫 사용자 입력','USER_SOURCE',0,[],{input_id:u1.id});
  const cap1=add('입력 접수','VIA',assumptions.intake_s,[source1]);
  const w1=workload(ep,u1,0,cfg),w2=workload(ep,u2,1,cfg);
  const look1=add('허용 원본 또는 관계 조회','VIA_READ',w1.lookup_s,[cap1]);
  const adopt1=model(u1,'첫 요청 관계와 의미 판단',[look1],w1.inTokens,w1.outTokens);
  const first1=add('첫 내용 표시','VIA_UI',assumptions.publish_s,[adopt1]);
  const read1=add('근거 상세 읽기와 현재 step','VIA_READ',assumptions.source_read_base_s+ep.variant*assumptions.source_read_variant_s,[first1]);
  let terminal1,source2;
  function finishGoal(input,parents,suffix){
    const firstIsComplete=input.kind==='independent';
    const detail=firstIsComplete?add(`${suffix} 첫 답변이 전체 답변`,'VIA',0,parents):model(input,`${suffix} 상세 응답 준비`,parents,780+ep.variant*19,94+ep.variant%4);
    const direct=input.handling==='VIA_DIRECT'||['addition','independent'].includes(input.kind);
    let result=direct&&!firstIsComplete?add(`${suffix} 직접 답변 상세 표시`,'VIA_UI',assumptions.publish_s,[detail]):detail;
    if(!direct){
      const ingress=add(`${suffix} 요청 전달과 접수`,'VIA',0.14,[detail]);
      const agent=add(`${suffix} 외부 Agent 업무`,'EXTERNAL_AGENT',assumptions.external_agent_s,[ingress]);
      const cost=usage(ep,cfg,input,'수신 Agent 업무 가정',1400+ep.variant*25,360+ep.variant*4,0,'RECEIVER','VIA QA05에서 분리한 공통 외부 비용');cost.node_id=agent.id;
      result=add(`${suffix} 결과 연결과 상세 표시`,'VIA_UI',0.22,[agent]);
    }
    const secs=3.2+(ep.variant%5)*0.3;
    const cost=usage(ep,cfg,input,'전체 음성 요약 준비',0,0,secs);
    const voice=add(`${suffix} 전체 음성 준비`,'VIA_AUDIO',assumptions.audio_generation_fixed_s+secs*assumptions.audio_generation_s_per_audio_second,[result]);
    cost.node_id=voice.id;
    return voice;
  }
  if(!f.overlap){terminal1=finishGoal(u1,[read1],'첫 목표');source2=add('후속 입력까지 사용자 간격','EXTERNAL_USER',assumptions.sequential_input_gap_s,[terminal1],{input_id:u2.id});}
  else source2=add('첫 표시 후 후속 입력','EXTERNAL_USER',assumptions.overlap_input_gap_s,[first1],{input_id:u2.id});
  const cap2=add('후속 입력 접수','VIA',assumptions.intake_s,[source2]);
  let relation2=cap2;
  if(cfg.dp2==='B')relation2=model(u2,'후속 입력 관계 판단',[cap2],420+ep.variant*12,28+ep.variant%3);
  const gate2=cfg.dp2==='A'&&f.overlap?add('현재 step 경계에서 수용','VIA',0,[cap2,read1]):relation2;
  const look2=add('후속 근거와 관계 조회','VIA_READ',w2.lookup_s,[gate2]);
  const partial2=model(u2,'후속 입력 의미 판단',[look2],w2.inTokens,w2.outTokens);
  const first2=add('후속 첫 내용 표시','VIA_UI',assumptions.publish_s,[partial2]);
  let adopt2=partial2;
  if(cfg.dp2==='B'&&f.overlap&&f.second_kind!=='independent'){
    adopt2=model(u2,'기존 상태와 반환 근거 결합',[partial2,read1],650+ep.variant*17,48+ep.variant%4);
  }
  // A correction replaces one goal. No obsolete revision is dispatched.
  let terminal2=finishGoal(u2,[adopt2,first2],f.second_kind==='correction'?'정정 목표':'추가 목표');
  if(f.overlap&&f.second_kind!=='correction')terminal1=finishGoal(u1,[read1],'기존 목표 유지');
  const terminalGoals=[];
  if(f.second_kind==='correction')terminalGoals.push({id:u1.goal_id,start:source1.finish_s,terminal:terminal2,members:[u1.id,u2.id]});
  else {terminalGoals.push({id:u1.goal_id,start:source1.finish_s,terminal:terminal1,members:[u1.id]},{id:u2.goal_id,start:source2.finish_s,terminal:terminal2,members:[u2.id]});}
  for(const g of terminalGoals){
    // Clip prior operations at this goal's input. Remove only external node durations.
    const projected=new Map();
    for(const n of en){
      const work=n.role.startsWith('EXTERNAL')||n.role==='USER_SOURCE'?0:Math.max(0,n.finish_s-Math.max(n.start_s,g.start));
      const upstream=Math.max(0,...n.parents.map(id=>projected.get(id)??0));
      projected.set(n.id,upstream+work);
    }
    goals.push({episode_id:ep.id,config:cfg.id,family_id:f.id,goal_id:g.id,members:g.members,start_s:g.start,finish_s:g.terminal.finish_s,wall_s:round(g.terminal.finish_s-g.start),via_s:round(projected.get(g.terminal.id)),terminal_node:g.terminal.id,status:'AUTHORED_TERMINAL'});
  }
  for(const [index,input,source,first,adopt,work] of [[0,u1,source1,first1,adopt1,w1],[1,u2,source2,first2,adopt2,w2]]){
    const v=verdict(ep,input,index,cfg);
    const early=adopt.finish_s>first.finish_s;
    let content=v.ok?input.first_response:`${v.adopted.targets.join(', ')||'별도 질문'}에 대해 ${v.adopted.goal}을 진행하겠습니다. 적용 조건: ${v.adopted.conditions.join(', ')}. 내용 출처: ${v.adopted.data_source}.`;
    if(input.kind==='independent'&&!v.ok)content=`기존 발표자료의 대상 기간 안에서만 설명하면, ${input.first_response}`;
    if(early)content=`${v.adopted.goal}에 이번 조건을 적용하겠습니다: ${v.adopted.conditions.join(', ')}. 대상의 최종 연결은 현재 조회 결과와 함께 확정합니다.`;
    responses.push({episode_id:ep.id,config:cfg.id,family_id:f.id,input_id:input.id,revision:input.revision,goal_id:input.goal_id,first_content:content,expected_meaning:input.meaning,adopted_meaning:v.adopted,qa07:v.ok,error:v.error,error_reason:v.reason,first_display_s:first.finish_s,full_meaning_adopted_s:adopt.finish_s,first_is_partial:early});
    for(const r of v.refs)referents.push({episode_id:ep.id,config:cfg.id,family_id:f.id,input_id:input.id,expression_id:r.id,phrase:r.phrase,revision:input.revision,expected:r.expected,adopted:r.adopted,correct:r.correct,adopted_at_s:adopt.finish_s});
    times.push({episode_id:ep.id,config:cfg.id,family_id:f.id,input_id:input.id,start_s:source.finish_s,first_s:first.finish_s,qa01_s:round(first.finish_s-source.finish_s),meaning_s:adopt.finish_s,initial_step_wait_s:index===1?round(Math.max(0,gate2.start_s-cap2.finish_s)):0,lookup_s:work.lookup_s,input_tokens:work.inTokens,output_tokens:work.outTokens,first_node:first.id,adopt_node:adopt.id});
  }
  for(const g of goals.filter(g=>g.episode_id===ep.id&&g.config===cfg.id))g.terminal_content_correct=responses.find(r=>r.config===cfg.id&&r.input_id===g.members.at(-1)).qa07;
  if(cfg.dp5==='B'&&['dirty','unproduced'].includes(f.relation_state))model(null,'정정 또는 새 사건 관계 갱신',terminalGoals.map(g=>g.terminal),2300+ep.variant*30,160);
  // Exact maxima of a fictional piecewise memory profile, not sampled OS RSS.
  const endpoints=[0,...en.flatMap(n=>[n.start_s,n.finish_s])];
  const executionLifetimes=[{start:0,end:f.overlap&&f.second_kind==='correction'?read1.finish_s:terminal1.finish_s},{start:source2.finish_s,end:terminal2.finish_s}];
  for(const t of [...new Set(endpoints)].sort((a,b)=>a-b)){
    const active=en.filter(n=>n.start_s<=t&&n.finish_s>t);
    const activeExecutions=executionLifetimes.filter(g=>g.start<=t&&g.end>t).length;
    const stateCount=cfg.dp2==='B'?activeExecutions:Number(activeExecutions>0);
    const modelBuffers=active.filter(n=>['VIA_MODEL','VIA_PRODUCER'].includes(n.role)).reduce((s,n)=>s+assumptions.client_call_base_mb+n.input_tokens/1000*assumptions.client_mb_per_1000_input_tokens,0);
    const readBuffers=active.filter(n=>n.role==='VIA_READ').length*assumptions.local_read_buffer_mb;
    const cache=cfg.dp5==='B'?assumptions.relation_cache_mb:0;
    const producer=active.some(n=>n.role==='VIA_PRODUCER')?assumptions.relation_producer_state_mb:0;
    const total=assumptions.core_resident_mb+stateCount*assumptions.request_state_mb+modelBuffers+readBuffers+cache+producer;
    memory.push({episode_id:ep.id,config:cfg.id,time_s:t,phase:t<0?'사전 생산':producer?'유휴 갱신':'요청',core_mb:assumptions.core_resident_mb,state_mb:stateCount*assumptions.request_state_mb,model_buffers_mb:round(modelBuffers),read_mb:readBuffers,relation_mb:cache,producer_mb:producer,total_mb:round(total)});
  }
  for(const t of endpoints){const concurrent=en.filter(n=>['VIA_MODEL','VIA_PRODUCER'].includes(n.role)&&n.start_s<=t&&n.finish_s>t);assert(concurrent.length<=assumptions.max_concurrent_semantic_calls);if(cfg.dp2==='A')assert(concurrent.filter(n=>n.role==='VIA_MODEL').length<=1);}
  nodes.push(...en);
 }
}

const summary={};
for(const c of configs){
 const t=times.filter(x=>x.config===c.id),r=referents.filter(x=>x.config===c.id),q=responses.filter(x=>x.config===c.id),g=goals.filter(x=>x.config===c.id),cost=costs.filter(x=>x.config===c.id),m=memory.filter(x=>x.config===c.id);
 summary[c.id]={qa01:mean(t.map(x=>x.qa01_s)),qa02:r.reduce((s,x)=>s+x.correct,0)/r.length,qa03:mean(g.map(x=>x.via_s)),qa04:mean(changeSpec.changes.map(x=>x.configs[c.id].modules.length)),qa05:cost.filter(x=>x.role==='VIA').reduce((s,x)=>s+x.cost_usd,0),qa06:Math.max(...m.map(x=>x.total_mb)),qa07:q.reduce((s,x)=>s+x.qa07,0)/q.length,response_n:t.length,referent_n:r.length,referent_correct:r.reduce((s,x)=>s+x.correct,0),goal_n:g.length,input_n:q.length,meaning_correct:q.reduce((s,x)=>s+x.qa07,0),receiver_cost:cost.filter(x=>x.role==='RECEIVER').reduce((s,x)=>s+x.cost_usd,0)};
 summary[c.id].total_cost=summary[c.id].qa05+summary[c.id].receiver_cost;
}
const familySummary=[];
for(const f of corpus.families)for(const c of configs){const t=times.filter(x=>x.family_id===f.id&&x.config===c.id),g=goals.filter(x=>x.family_id===f.id&&x.config===c.id),q=responses.filter(x=>x.family_id===f.id&&x.config===c.id),r=referents.filter(x=>x.family_id===f.id&&x.config===c.id);familySummary.push({family:f.id,name:f.name,config:c.id,qa01:mean(t.map(x=>x.qa01_s)),qa03:mean(g.map(x=>x.via_s)),qa02:r.length?r.reduce((s,x)=>s+x.correct,0)/r.length:null,qa07:mean(q.map(x=>x.qa07)),response_n:t.length,goal_n:g.length,referent_n:r.length});}
const dataset={version:VERSION,evidence_kind:KIND,created_on:'2026-10-21',actual_model_calls:0,actual_product_runs:0,assumptions,configs,...corpus,changes:changeSpec.changes,responses,referents,times,goals,nodes,costs,memory,error_rules:errorRules,summary,family_summary:familySummary};
await fs.mkdir(DELIVERY,{recursive:true});await fs.mkdir(path.join(OUT,'previews'),{recursive:true});
await fs.writeFile(path.join(OUT,'synthetic-data.json'),JSON.stringify(dataset,null,2));
await fs.writeFile(path.join(OUT,'summary.json'),JSON.stringify({version:VERSION,evidence_kind:KIND,summary,family_summary:familySummary},null,2));
await renderReport(dataset,{docDir:DOC,deliveryDir:DELIVERY});
const sha=x=>crypto.createHash('sha256').update(x).digest('hex');
const manifest={version:VERSION,schema:'COMMON-QA-SYN-SCHEMA-1',contract:'QA-COMMON-SYN-CONTRACT-v1',evidence_kind:KIND,created_on:'2026-10-21',parent_datasets:['dp2-synthetic-backdata-20261021','dp5-synthetic-backdata-20261021'],parent_compatibility:'NEW_CORPUS_NOT_ROW_EQUIVALENT',common_mapping:'THEMATIC_ONLY_NOT_EQUIVALENT_EXECUTION',corpus_sha256:sha(canon(corpus)),assumptions_sha256:sha(canon(assumptions)),dataset_sha256:sha(await fs.readFile(path.join(OUT,'synthetic-data.json'))),generator_sha256:sha(await fs.readFile(new URL(import.meta.url))),scenario_generator_sha256:sha(await fs.readFile(new URL('./common_qa_synthetic_scenarios.mjs',import.meta.url))),counts:{episodes:120,inputs:240,referents:250,goals:180},configs};
await fs.writeFile(path.join(OUT,'manifest.json'),JSON.stringify(manifest,null,2));
const sources=new Set(corpus.sources.map(s=>s.id));
assert.equal(corpus.episodes.length,120);assert.equal(allInputs.length,240);
assert.equal(new Set(allInputs.map(i=>i.id)).size,240);
for(const e of corpus.episodes)for(const id of e.source_ids)assert(sources.has(id),id);
for(const c of configs){assert.equal(summary[c.id].response_n,240);assert.equal(summary[c.id].input_n,240);assert.equal(summary[c.id].referent_n,summary.BASE.referent_n);assert.equal(summary[c.id].goal_n,summary.BASE.goal_n);}
for(const row of times)assert(row.qa01_s>=0);
for(const row of goals)assert(row.via_s>=0&&row.via_s<=row.wall_s+0.00001,JSON.stringify(row));
const nodeIds=new Set(nodes.map(n=>n.id));for(const c of costs)assert(nodeIds.has(c.node_id),`missing cost node ${c.id}`);
for(const r of referents.filter(r=>!r.correct))assert.equal(responses.find(x=>x.input_id===r.input_id&&x.config===r.config).qa07,0);
for(const r of responses.filter(r=>r.family_id==='F12'&&r.input_id.endsWith('U02')))assert(r.first_content.includes(allInputs.find(i=>i.id===r.input_id).first_response));
console.log(JSON.stringify({version:VERSION,summary,rows:{inputs:allInputs.length,responses:responses.length,referents:referents.length,goals:goals.length,nodes:nodes.length,costs:costs.length,memory:memory.length}},null,2));
if(process.argv.includes('--data-only'))process.exit(0);

const moduleBase=process.env.VIA_ARTIFACT_MODULE_BASE??'/private/tmp/via-qa-common-01a15d51';
const req=createRequire(path.join(moduleBase,'entry.cjs'));
const {Workbook,SpreadsheetFile}=await import(pathToFileURL(req.resolve('@oai/artifact-tool')));
const wb=Workbook.create(),navy='#243F5B',muted='#637183',blue='#235C8A';
const sheetNames=['요약','공통가정','사례군','시간','목표','지칭판정','의미판정','비용','메모리','변경과제','입력','원본','단계'];
const sheets=Object.fromEntries(sheetNames.map(n=>[n,wb.worksheets.add(n)]));
function col(n){let s='';for(n++;n;n=Math.floor((n-1)/26))s=String.fromCharCode(65+(n-1)%26)+s;return s;}
const tables={};
function table(name,title,headers,rows,widths,rowHeight=42){
 const sh=sheets[name],last=col(headers.length-1),end=rows.length+5;
 sh.showGridLines=false;sh.getRange(`A1:${last}${Math.max(end,6)}`).format.font={name:'Arial',size:10,color:'#253448'};
 sh.getRange('A2').values=[[title]];sh.getRange('A2').format.font={size:15,bold:true,color:navy};
 sh.getRange('A3').values=[['가상 시나리오 기반 설계 평가. 사례와 성능 수치는 작성 가정이다.']];sh.getRange('A3').format.font={color:muted,size:10};
 sh.getRange(`A5:${last}5`).values=[headers];sh.getRange(`A5:${last}5`).format={fill:navy,font:{name:'Arial',size:10,bold:true,color:'#FFFFFF'},rowHeight:30,horizontalAlignment:'center',verticalAlignment:'center',wrapText:true};
 if(rows.length)sh.getRange(`A6:${last}${end}`).values=rows;
 sh.getRange(`A6:${last}${end}`).format={rowHeight,verticalAlignment:'top',wrapText:true};
 widths.forEach((w,i)=>{sh.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidthPx=w;});
 sh.freezePanes.freezeRows(5);sh.freezePanes.freezeColumns(2);
 sh.tables.add(`A5:${last}${end}`,true,`Common${sheetNames.indexOf(name)}`);
 tables[name]={end,last};return sh;
}
const assumptionsRows=Object.entries(assumptions).map(([k,v])=>[k,v,typeof v==='number'?'작성 가정':'설명']);
table('공통가정','공통 계산 가정',['항목','값','근거 상태'],assumptionsRows,[310,700,170],38);
table('사례군','공통 시나리오별 결과',['유형','내용','구성','QA01 초','QA03 초','QA02','QA07','응답 수','목표 수','표현 수'],familySummary.map(r=>[r.family,r.name,r.config,r.qa01,r.qa03,r.qa02??'비적용',r.qa07,r.response_n,r.goal_n,r.referent_n]),[80,280,90,100,100,100,100,80,80,80],38);
sheets['사례군'].getRange(`D6:E${tables['사례군'].end}`).setNumberFormat('0.000');sheets['사례군'].getRange(`F6:G${tables['사례군'].end}`).setNumberFormat('0.0%');
table('시간','입력별 첫 응답과 최초 의미 시점',['사례','구성','유형','입력','입력 t','첫 표시 t','QA01 초','전체 의미 t','step 대기 초','조회 초','입력 token','출력 token'],times.map(t=>[t.episode_id,t.config,t.family_id,t.input_id,t.start_s,t.first_s,null,t.meaning_s,t.initial_step_wait_s,t.lookup_s,t.input_tokens,t.output_tokens]),[180,80,65,220,95,95,95,105,100,95,100,100],32);
sheets['시간'].getRange(`G6:G${tables['시간'].end}`).formulas=times.map((t,i)=>[`=F${i+6}-E${i+6}`]);sheets['시간'].getRange(`E6:J${tables['시간'].end}`).setNumberFormat('0.000');
table('목표','목표별 처리 종료와 VIA 귀속 경로',['사례','구성','유형','목표','관련 입력','시작 t','종료 t','전체 경과 초','VIA 경로 초','최종 의미 일치','종료 종류'],goals.map(g=>[g.episode_id,g.config,g.family_id,g.goal_id,g.members.join(', '),g.start_s,g.finish_s,null,g.via_s,g.terminal_content_correct,g.status]),[180,80,65,220,390,100,100,115,115,100,230],40);
sheets['목표'].getRange(`H6:H${tables['목표'].end}`).formulas=goals.map((g,i)=>[`=G${i+6}-F${i+6}`]);sheets['목표'].getRange(`F6:I${tables['목표'].end}`).setNumberFormat('0.000');
table('지칭판정','최초 채택 대상 집합',['입력','구성','표현 ID','표현','정답 집합','채택 집합','QA02 성공','채택 t'],referents.map(r=>[r.input_id,r.config,r.expression_id,r.phrase,canon([...r.expected].sort()),canon([...r.adopted].sort()),null,r.adopted_at_s]),[230,80,260,300,330,330,95,100],60);
sheets['지칭판정'].getRange(`G6:G${tables['지칭판정'].end}`).formulas=referents.map((r,i)=>[`=IF(E${i+6}=F${i+6},1,0)`]);
sheets['지칭판정'].getRange(`H6:H${tables['지칭판정'].end}`).setNumberFormat('0.000');
table('의미판정','첫 표시와 최초 채택 의미',['입력','구성','첫 표시 내용','정답 의미','채택 의미','QA07 성공','오류','오류 설명'],responses.map(r=>[r.input_id,r.config,r.first_content,canon(r.expected_meaning),canon(r.adopted_meaning),null,r.error,r.error_reason]),[230,80,650,720,720,95,140,360],150);
sheets['의미판정'].getRange(`F6:F${tables['의미판정'].end}`).formulas=responses.map((r,i)=>[`=IF(D${i+6}=E${i+6},1,0)`]);
table('비용','사용량에서 계산한 가상 비용',['비용 ID','사례','구성','입력','단계','역할','입력 token','출력 token','음성 초','비용 USD'],costs.map(c=>[c.id,c.episode_id,c.config,c.input_id,c.stage,c.role,c.input_tokens,c.output_tokens,c.audio_seconds,null]),[100,180,80,230,260,100,105,105,100,120],38);
const assumptionRow=k=>Object.keys(assumptions).indexOf(k)+6;
sheets['비용'].getRange(`J6:J${tables['비용'].end}`).formulas=costs.map((c,i)=>{let r=i+6;return [`=G${r}*'공통가정'!$B$${assumptionRow('input_usd_per_million')}/1000000+H${r}*'공통가정'!$B$${assumptionRow('output_usd_per_million')}/1000000+I${r}*'공통가정'!$B$${assumptionRow('audio_usd_per_second')}`];});sheets['비용'].getRange(`J6:J${tables['비용'].end}`).setNumberFormat('$0.000000');
// Memory exposes each configuration's peak boundary plus one preceding state per episode.
const memoryView=[];for(const e of corpus.episodes)for(const c of configs){let rows=memory.filter(m=>m.episode_id===e.id&&m.config===c.id).sort((a,b)=>b.total_mb-a.total_mb);memoryView.push(...rows.slice(0,2));}
table('메모리','사례별 상위 동시 RAM 상태',['사례','구성','가상 t','구간','공통 MB','실행 상태 MB','모델 버퍼 MB','읽기 MB','관계 MB','생산 MB','합계 MB'],memoryView.map(m=>[m.episode_id,m.config,m.time_s,m.phase,m.core_mb,m.state_mb,m.model_buffers_mb,m.read_mb,m.relation_mb,m.producer_mb,null]),[180,80,100,100,100,120,120,100,100,100,110],34);
sheets['메모리'].getRange(`K6:K${tables['메모리'].end}`).formulas=memoryView.map((m,i)=>[`=SUM(E${i+6}:J${i+6})`]);sheets['메모리'].getRange(`E6:K${tables['메모리'].end}`).setNumberFormat('0.00');
const changeRows=changeSpec.changes.flatMap(ch=>configs.map(c=>[ch.id,ch.title,c.id,ch.configs[c.id].modules.join('\n'),ch.configs[c.id].modules.length,ch.configs[c.id].reason,ch.acceptance]));
table('변경과제','같은 개발 변경 과제의 예상 범위',['과제','변경 요구','구성','가상 Rust 모듈 목록','예상 수','범위 이유','수용 조건'],changeRows,[80,280,80,450,90,650,600],100);
table('입력','공통 입력과 정답',['사례','유형','입력 ID','종류','revision','사용자 입력','정답 의미','근거 ID'],allInputs.map(i=>[i.episode_id,i.family_id,i.id,i.kind,i.revision,i.text,canon(i.meaning),i.source_ids.join(', ')]),[180,70,230,110,85,680,740,450],130);
table('원본','양안 공통 가상 원자료',['Source ID','사례','종류','원문'],corpus.sources.map(s=>[s.id,s.episode_id,s.kind,s.content]),[290,180,160,950],85);
table('단계','가상 처리 순서와 의존 관계',['노드 ID','구성','단계','역할','시작 t','종료 t','소요 초','선행 노드','입력 ID','비용 ID'],nodes.map(n=>[n.id,n.config,n.label,n.role,n.start_s,n.finish_s,n.duration_s,n.parents.join(', '),n.input_id??'',n.cost_id??costs.find(c=>c.node_id===n.id)?.id??'']),[320,80,290,170,95,95,95,650,230,100],38);
sheets['단계'].getRange(`E6:G${tables['단계'].end}`).setNumberFormat('0.000');

const s=sheets['요약'];s.showGridLines=false;s.tabColor=navy;s.getRange('A1:G36').format.font={name:'Arial',size:11,color:'#253448'};
s.getRange('A2').values=[['DP2와 DP5 공통 QA 비교']];s.getRange('A2').format.font={size:16,bold:true,color:navy};
s.getRange('A3').values=[['가상 시나리오 기반 설계 평가. 공통 120개 에피소드의 동일 입력을 세 구성에 적용했다.']];s.getRange('A3').format.font={size:10,color:muted};
s.getRange('A5:G5').values=[['QA','단위','공통 기준','DP2 B','DP5 B','분모','평가 범위']];s.getRange('A5:G5').format={fill:navy,font:{name:'Arial',color:'#FFFFFF',bold:true},rowHeight:32,horizontalAlignment:'center',wrapText:true};
const summaryLabels=[['QA01 첫 응답','초',240,'오답 포함 첫 구체적 내용'],['QA02 지칭 집합','%',summary.BASE.referent_n,'표현별 최초 대상 집합'],['QA03 VIA 처리','초',summary.BASE.goal_n,'정정과 기존 목표 전체 포함'],['QA04 변경 범위','모듈',6,'같은 변경 요구의 가상 모듈'],['QA05 VIA 모델 비용','USD',120,'전체 에피소드 묶음의 VIA 비용'],['QA06 최대 로컬 RAM','MB',120,'cloud 배치 동시 사용량 가정'],['QA07 요청 의미','%',240,'입력별 필수 의미 전체']];
s.getRange('A6:G12').values=summaryLabels.map(x=>[x[0],x[1],null,null,null,x[2],x[3]]);
for(const [ci,c] of configs.entries()){
 const cc=col(ci+2),range=(sh,k)=>`'${sh}'!$${k}$6:$${k}$${tables[sh].end}`;
 const formulas=[`=AVERAGEIFS(${range('시간','G')},${range('시간','B')},"${c.id}")`,`=AVERAGEIFS(${range('지칭판정','G')},${range('지칭판정','B')},"${c.id}")`,`=AVERAGEIFS(${range('목표','I')},${range('목표','B')},"${c.id}")`,`=AVERAGEIFS(${range('변경과제','E')},${range('변경과제','C')},"${c.id}")`,`=SUMIFS(${range('비용','J')},${range('비용','C')},"${c.id}",${range('비용','F')},"VIA")`,`=_xlfn.MAXIFS(${range('메모리','K')},${range('메모리','B')},"${c.id}")`,`=AVERAGEIFS(${range('의미판정','F')},${range('의미판정','B')},"${c.id}")`];
 s.getRange(`${cc}6:${cc}12`).formulas=formulas.map(x=>[x]);
}
s.getRange('A6:G12').format={rowHeight:42,verticalAlignment:'center',wrapText:true};s.getRange('C6:E12').setNumberFormat('0.000');s.getRange('C7:E7').setNumberFormat('0.0%');s.getRange('C12:E12').setNumberFormat('0.0%');s.getRange('C10:E10').setNumberFormat('$0.0000');s.getRange('C11:E11').setNumberFormat('0.00');
s.getRange('A15').values=[['비교 구성을 읽는 방법']];s.getRange('A15').format.font={bold:true,color:navy};
s.getRange('A17:G19').values=[['공통 기준','2A + 5A',null,null,null,null,'두 DP의 A는 같은 기록을 참조'],['DP2 비교','2A + 5A','2B + 5A',null,null,null,'DP5 A를 고정'],['DP5 비교','2A + 5A','2A + 5B',null,null,null,'DP2 A를 고정']];
s.getRange('A22').values=[['단위와 해석']];s.getRange('A22').format.font={bold:true,color:navy};
s.getRange('A24').values=[['120개는 12개 유형의 작성 변형이다. 실사용 빈도나 독립 실험 횟수로 해석하지 않는다.']];
s.getRange('A26').values=[['QA05는 VIA 비용이다. 수신자 비용과 전체 합계는 설명 페이지와 비용 원장에 별도로 있다.']];
s.getRange('A28').values=[['QA02와 QA07은 대상 오류가 겹친다. 기존 점수와 목표는 새 모집단에 재적용하지 않았다.']];
[[0,235],[1,110],[2,115],[3,115],[4,115],[5,90],[6,320]].forEach(([i,w])=>s.getRange(`${col(i)}1:${col(i)}30`).format.columnWidthPx=w);
wb.recalculate();
const observed=s.getRange('C6:E12').values;
for(let k=0;k<7;k++)for(let c=0;c<3;c++)assert(Math.abs(observed[k][c]-summary[configs[c].id][`qa0${k+1}`])<1e-6,`spreadsheet metric ${k}/${c}: ${observed[k][c]}`);
// Verify that an editable tariff flows through both row costs and the summary, then restore.
const tariffCell=sheets['공통가정'].getRange(`B${assumptionRow('input_usd_per_million')}`);tariffCell.values=[[2]];wb.recalculate();assert(s.getRange('C10').values[0][0]>summary.BASE.qa05);tariffCell.values=[[1]];wb.recalculate();
await fs.writeFile(path.join(OUT,'summary-inspection.ndjson'),(await wb.inspect({kind:'table',range:'요약!A5:G12',include:'values,formulas',tableMaxRows:10,tableMaxCols:7,maxChars:5000})).ndjson);
await fs.writeFile(path.join(OUT,'formula-scan.ndjson'),(await wb.inspect({kind:'match',searchTerm:'#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A|#NUM!|#NULL!|#SPILL!|#CALC!',options:{useRegex:true,maxResults:30},maxChars:3000})).ndjson);
for(const name of sheetNames){const range=name==='요약'?'A1:G29':`A1:${tables[name].last}${Math.min(tables[name].end,name==='의미판정'||name==='입력'?8:11)}`;const blob=await wb.render({sheetName:name,range,scale:1,format:'png'});await fs.writeFile(path.join(OUT,'previews',`${name}.png`),new Uint8Array(await blob.arrayBuffer()));}
await (await SpreadsheetFile.exportXlsx(wb)).save(path.join(DELIVERY,'VIA-QA-common-synthetic.xlsx'));
await fs.writeFile(path.join(OUT,'validation.json'),JSON.stringify({evidence_kind:KIND,version:VERSION,actual_model_calls:0,actual_product_runs:0,common_counts:true,goal_dag_bounds:true,source_links:true,workbook_summary_matches:true,tariff_change_recalculates:true,native_excel_verified:false,dataset_sha256:crypto.createHash('sha256').update(await fs.readFile(path.join(OUT,'synthetic-data.json'))).digest('hex')},null,2));
console.log('Workbook export and formula validation complete');
