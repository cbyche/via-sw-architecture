import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {FileBlob,Presentation,PresentationFile} from '@oai/artifact-tool';
import {structures} from './structures.mjs';
import {addDP5} from './dp5.mjs';
process.env.RUNTIME_NODE_MODULES||='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../../..');
const WORK=path.join(ROOT,'docs/presentations_files/five-dp-delegation-memory-20261020');
const SKILL=process.env.PRESENTATIONS_SKILL_DIR||'/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1015.12040/skills/presentations';
const REV=process.argv.find(a=>a.startsWith('--revision='))?.split('=')[1]||'draft-v2';
if(!/^[a-z0-9-]+$/.test(REV))throw new Error('Unsafe revision');
const DATA=JSON.parse(await fs.readFile(path.join(WORK,'source/deck-data.json'),'utf8'));
const FONT='Apple SD Gothic Neo';
const C={green:'#387C5B',option:'#C9DEB1',cream:'#FFF7DE',blue:'#186FC1',ink:'#202720',gray:'#606A66',pale:'#EAF3EE',line:'#A7B7AC',white:'#FFFFFF'};
const pres=Presentation.create({slideSize:{width:1280,height:720}});const tableOwners=[];const slideTitles=[];
function rect(s,x,y,w,h,fill=C.cream,line=C.line,lw=1){return s.shapes.add({geometry:'rect',position:{left:x,top:y,width:w,height:h},fill,line:{fill:line,width:lw,style:'solid'}});}
function text(s,t,x,y,w,h,size=24,color=C.ink,bold=false){let q=s.shapes.add({geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});q.text=t;q.text.style={typeface:FONT,fontSize:size,color,bold};for(const m of String(t).matchAll(/→\s*(\$?\d[\d.,/]*)/g))q.text.getRange(m.index+m[0].indexOf(m[1]),m[1].length).fill=C.blue;return q;}
function slide(title,note=''){slideTitles.push(title);let s=pres.slides.add();s.background.fill=C.white;text(s,title,44,30,1192,63,42,C.ink,true);rect(s,44,105,1192,3,C.green,C.green,0);text(s,String(pres.slides.items.length).padStart(2,'0'),1190,676,46,24,17,C.gray);s.speakerNotes.text=note;return s;}
function foot(s,t){text(s,t,46,625,1188,45,20,C.gray);}
function lines(s,t,x=48,y=145,w=1178,size=27,h=420){text(s,t,x,y,w,h,size);}
function connect(s,a,b,dashed=false){return s.shapes.connect(a,b,{kind:'straight',fromSide:'bottom',toSide:'top',line:{style:dashed?'dashed':'solid',fill:C.ink,width:1.8},tail:{type:'arrow',width:'med',length:'med'}});}
function columns(s,cols,{selected=-1,note=''}={}){let gap=25,w=(1188-gap*(cols.length-1))/cols.length;
 cols.forEach((c,i)=>{let x=46+i*(w+gap);if(i===selected)rect(s,x-7,126,w+14,493,'none',C.blue,3);rect(s,x,133,w,47,C.option,C.line,1);text(s,c.title,x+13,141,w-26,37,27,C.ink,true);let prior;
 c.nodes.forEach((n,j)=>{let y=201+j*86;let b=rect(s,x+10,y,w-20,62,c.tactic?'#E9F3FF':C.cream,c.tactic?C.blue:C.ink,1);text(s,n,x+22,y+7,w-44,51,22,c.tactic?C.blue:C.ink);if(prior)connect(s,prior,b);prior=b;});text(s,c.tradeoff,x+3,532,w-6,76,21,C.gray);});if(note)foot(s,note);}
function table(s,values,{top=145,height=390,width=1188,left=46,columns,font=24}={}){tableOwners.push(pres.slides.items.length);let t=s.tables.add({rows:values.length,columns:values[0].length,left,top,width,height,values,columnWidths:columns});t.borders.assign({style:'solid',fill:'#CFD8D2',width:1});
 t.cells.block({row:0,column:0,rowCount:values.length,columnCount:values[0].length}).assign({textStyle:{typeface:FONT,fontSize:font,color:C.ink},margins:{top:8,bottom:8,left:10,right:10}});
 for(let r=0;r<values.length;r++)for(let c=0;c<values[0].length;c++){let cell=t.getCell(r,c);cell.fill=r===0?C.option:(r%2?C.white:'#F4F7F4');cell.text.style={typeface:FONT,fontSize:font,color:r>0&&values[0][c]==='보완 후'?C.blue:C.ink,bold:r===0};if(r>0&&typeof values[r][c]==='string'&&values[r][c].includes('→')){const at=values[r][c].indexOf('→');cell.value={runs:[{run:values[r][c].slice(0,at+1),style:{typeface:FONT,fontSize:`${font}px`}},{run:values[r][c].slice(at+1),style:{typeface:FONT,fontSize:`${font}px`,color:C.blue,bold:true}}]};}else if(r>0&&typeof values[r][c]==='string'&&values[r][c].includes(' / ')){const [a,b]=values[r][c].split(' / ');cell.value={runs:[{run:a+' / ',style:{typeface:FONT,fontSize:`${font}px`}},{run:b,style:{typeface:FONT,fontSize:`${font}px`,color:C.blue,bold:true}}]};}}return t;}
const SOURCES='비용은 실제 provider 사용량에 입력 $0.40/출력 $1.60 per1M token의 고정 비교 단가를 적용한 환산이며 청구서가 아니다.\n모든 semantic/화면/발화 producer는 같은 원격 Qwen3.7-plus와 공통 Model Access를 사용한다. 별도 학습 모델/가중치 복제는 없다.\n설계: docs/architecture/12-decisions/decision-packages/seven-problem-review/01-evidence-supply.md,02-interpretation-execution.md,04-screen-speech-grounding.md,05-agent-handoff.md\n계약:14-corrected-poc-contract.md,16-presentation-selection-contract.md\n기존 결과:15-corrected-poc-results.md / 새 검토:17-presentation-results.md\n원시:results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/ 및 four-dp-presentation-20261019/';
const S=structures({rect,text,C});
const BASIC=DATA.existing_basic;
function metrics(title,rows,comment){let s=slide(title,SOURCES);const shown=rows.map(r=>{const z=[...r];const id=z[0].match(/^(01|02|04|05) ([ABC])$/);if(id)z[0]=title.includes('전체 원값')?`DP${{'01':1,'02':2,'04':3,'05':4}[id[1]]} ${id[2]}`:id[2];return z;});const mt=table(s,[['안','QA01 초 / 관측','QA02','QA03 초 / 완료','QA04 개','QA05 USD','QA06 MB'],...shown],{height:rows.length===4?330:(rows.length>3?270:230),font:rows.length===4?22:23,columns:[110,219,110,219,100,235,195]});shown.forEach((r,i)=>{if(['A2+A5','B4'].includes(r[0]))for(const c of [0,1,2,3,5,6])mt.getCell(i+1,c).text.style={typeface:FONT,fontSize:22,color:C.blue};});lines(s,comment,48,rows.length===4?500:(rows.length>3?451:405),1180,24,rows.length>3?112:188);foot(s,'완료는 호스트 종료 관측, 본문은 별도 저자 조건 검토다. QA01 callback proxy, QA03 조건부 평균. 비용은 12회 전체.');return s;}
// 1
{let s=slide('VIA의 다섯 설계 비교',SOURCES);text(s,'연속 대화의 근거와 위임 메모리',54,179,1170,84,53,C.green,true);lines(s,'DP1  현재 요청에 어떤 근거를 공급할까\nDP2  새 입력의 의미 판단을 언제 실행할까\nDP3  화면과 발화의 지칭을 어떻게 결합할까\nDP4  Agent에게 무엇을 넘기고 누가 상세 자료를 읽을까\nDP5  위임 관계를 요청 때 해석할까, 미리 정리할까',56,292,1160,29,280);foot(s,'DP1–4는 10월 19일 결과를 보존한다. DP5는 10월 20일 추가 설계이며 미측정·미선택이다.');}
// VIA context examples, explicitly separate from the measured authored fixtures.
{
 let s=slide('DP1–4의 기존 사용자 장면',SOURCES+'\n이 장면은 발표 설명 예시이며 새 측정 사례가 아니다.');
 table(s,[['장면','설계에서 다룰 문제'],['DP1  앞서 읽은 자료에 대해 조건을 바꿔 다시 묻는다','근거 재사용과 현재성'],['DP2  분석 도중 추가 질문·정정과 Agent 통지가 들어온다','입력 보존과 의미 판단 대기'],['DP3  화면을 가리켜 “이 둘”이라고 말하고 대상을 고친다','동시 원본·대상 집합·revision'],['DP4  앞서 정한 조건을 유지해 다른 Agent에게 이어 맡긴다','기채택 조건과 상세 자료 읽기']],{height:406,font:26,columns:[805,383]});
 foot(s,'발표 설명 예시다. 새 측정 사례나 전체 VIA 시스템 완성 검증을 뜻하지 않는다.');
}
// 2
{let s=slide('DP1–4의 기존 QA 비교 계약',SOURCES);table(s,[['순서','QA','이번 PoC의 관찰'],['1','첫 응답 지연','채택한 구체적 내용의 실제 출력 callback'],['2','지칭 집합 정확도','대상 집합과 의미 있는 현재 revision'],['3','VIA 처리시간','목표 완료까지 VIA 시간과 별도 전체 경과'],['4','변경 범위','같은 네 변경 과제의 실제 Rust 모듈 수'],['5','모델 비용','준비·실패·수신자를 포함한 실제 사용량'],['6','로컬 메모리','같은 시각 Python+Rust RSS']],{height:440,font:24,columns:[95,300,793]});foot(s,'구조적 동점은 다음 QA로 판단한다. 미확정과 미관측은 동점으로 바꾸지 않는다.');}
//3
{let s=slide('DP1 기본안: 현재 요청의 근거 공급',SOURCES);S.one(s);}
{let s=slide('DP1 C: 별도 생산과 게시 근거 조회',SOURCES);S.oneC(s);}
//4
metrics('DP1 실측: 재사용 이익과 첫 응답의 미확정',BASIC.slice(0,3),'A/B의 첫 호출 위치는 같아도 유지 입력과 완성 답변 분기가 다르다.\n342 ms 차이를 확정 우위나 구조적 동점으로 단정하지 않는다.\n같은 완료 35개 목표 A 13.730초 / B 14.242초. A는 답변 호출 6회 절약.\nC는 호스트 종료 36/36의 QA03 12.364초, 저자 본문 검토 34/36(A/B 30/36). 준비비용과 교환.');
// Common first-handling policy is separate from basic architecture selection.
{
 const rows=['A','B','C'].map(v=>{const a=DATA.dp01_fresh_policy.rows.find(r=>r.variant===v&&r.tactic==='ORIGINAL');const b=DATA.dp01_fresh_policy.rows.find(r=>r.variant===v&&r.tactic==='ADOPTION_ONLY');return [v,`${(a.qa01_ms/1000).toFixed(3)} → ${(b.qa01_ms/1000).toFixed(3)}`,'6/6 → 6/6',`${(a.qa03_ms/1000).toFixed(3)} → ${(b.qa03_ms/1000).toFixed(3)}`,'미측정',`$${a.total_cost_usd.toFixed(7)}
→ $${b.total_cost_usd.toFixed(7)}`,`${a.qa06_peak_rss_mb.toFixed(3)}
→ ${b.qa06_peak_rss_mb.toFixed(3)}`];});
 let s=slide('DP1 공통 첫 안내 분리: 빠른 채택과 추가 비용',SOURCES+'\nDP01 E3 × ABC × original/adoption-only ×3 =18 fresh episodes. 각 정책/안의 first6/6, host종료6/6, QA02 6/6. A/B 첫 payload12쌍 동일; C는 별도 사전생산 변동 포함.\n원문: dp01/fresh-summary.json,first-call-policy-breakdown.json,first-handling-examples.json.\n별도 본문: 저자/동치 A3/6,B3/6,C6/6 각정책 동일. C literal3/6→2/6와 분리. QA04새정책 미측정, RSS소규모 관측변동.');
 table(s,[['안','QA01 초 전 → 후','QA02 집합','QA03 초 전 → 후','QA04','QA05 USD 전 → 후','QA06 MB 전 → 후'],...rows],{height:265,font:21,columns:[65,190,130,190,105,293,215]});
 lines(s,'같은 E3에서 첫 안내 분리는 QA01을 줄였지만 QA03과 비용이 늘었다.\n호출 A/B 12 → 18회, C 15 → 21회. 호스트 종료 각 6/6, 본문 A/B 3/6·C 6/6 불변.\n출력은 출처·대상 채택 안내이며 내부 처리의도 라벨도 노출됐다. 자연스러운 제품 응답은 미검증.\nQA01 우선의 공통 설계 방향이다. 기본 A/B 선택 효과나 네 사례 전체 이익은 아니다.',48,440,1180,24,162);
 foot(s,'파랑은 분리 후 값. 현재 source/target 처리안내 callback이며 상세 사실 답변의 첫 제공 시각과 구별한다.');
}
//5
{let s=slide('DP1 조건부 A 선택과 변경 조각 갱신',SOURCES);columns(s,[{title:'A  유효 근거 유지',nodes:['1. 읽은 원본 발췌와 의존 출처','2. 변경된 source revision 확인','3. 유효 조각 유지·누락 재조회','4. 전체 현재성 재검사·채택'],tradeoff:'후속 자료 재사용을 근거로 조건부 A\n본문 30/36, 조건 미충족 6건 유지\n최상위 QA01 확정 승자는 미입증'},{title:'A2  변경 조각만 갱신',tactic:true,nodes:['1. source별 의존 조각 분리','2. 영향 조각을 무효화','3. 남은 유효 발췌 재사용','4. 재조합 후 동일 최종 검사'],tradeoff:'정정 E3 × 3: 읽기 12회 → 9회\n본문 충족 3/6 → 3/6'}],{selected:0});foot(s,'A2: QA03 21.822 → 21.433초, 호출 12 → 12, 비용 $0.0148424 → $0.0145860. 원문 재조회 절약에 한정 적용.');}
//6
{let s=slide('DP2 기본안: 후속 입력의 의미 판단 실행',SOURCES);S.two(s);}
//7
metrics('DP2 실측: 호출 여유가 있을 때 B 선택',BASIC.slice(3,5),'K=2 독립 후속 질문: QA01 A 13.388초, B 6.270초. 실제 API 중첩 6.261초.\nK=1 같은 질문 완료: A 13.429초, B 13.087초. 중첩 이익이 사라진다.\nN-R 27/27 대 26/27은 별도 연결 진단. QA02는 비적용.\nB의 추가 Task 메타데이터 오류를 실제 잘못된 메일 발송으로 확대하지 않는다.');
//8
{let s=slide('DP2 선택과 B1 불변 문맥 공유',SOURCES);columns(s,[{title:'B  독립 실행 유지',nodes:['1. 공통 원본 input/source','2. worker별 input 깊은 복제','3. source payload·wire 복제','4. 각 요청 상태·revision 독립'],tradeoff:'실제 원본 약 7.5 KB\n모델 가중치 복제는 없음'},{title:'B1  같은 원본을 참조',tactic:true,nodes:['1. immutable input Arc 공유','2. worker는 참조만 유지','3. source·wire를 빌려 직렬화','4. mutable 요청 상태는 독립'],tradeoff:'작은 소유 할당 감소 확인\nN=1 전체 RSS 우세 미확인'}],{selected:0,note:'72회 실제 응답 재생, API 0. 기존 live RSS 45~46 MB와 새 재생 30~32 MB를 전후 개선으로 비교하지 않는다.'});}
//9
{let s=slide('B1 전후: 요청 할당과 전체 RSS',SOURCES+'\nB1은 의미 오류 보완이 아니다. 기존 B N-R26/27 오류도 고정응답 재생에 그대로 남는다.\nDP02 추가실험 dp02/strict-control-v2/summary.json,report.md,trials.csv\n168 session의 actual input/instruction/max_tokens 비교는 독립 HashMap/HashSet 순서만 정규화했다. 원문/의미 데이터/응답은 변경하지 않았다.');table(s,[['동시 N','할당 peak 평균 bytes B / B1','전체 RSS 최고 MB B / B1'],['1','180,134 / 130,440','30.097 / 30.294'],['2','313,253 / 212,126','30.802 / 30.507'],['4','573,372 / 365,413','32.326 / 31.867']],{height:275,font:26,columns:[130,540,518]});lines(s,'통제 72회 최종 결과·callback과 입력 snapshot 168/168 session 동일.\n이전 72회의 상태 순서 차이는 보존하고, 원래 호출 준비 barrier를 강화했다.\nN=1 평균 RSS 동점. N=2·4의 작은 감소를 일반적 큰 개선으로 확대하지 않는다.\n대가: 빌린 직렬화 API와 불변 참조의 수명 관리가 추가된다.',48,457,1180,25,150);foot(s,'파랑은 보완 후 값이다. 할당 bytes는 RSS가 아니다. 새 QA01/03이 아닌 API 0 재생, 네 사례와 세 seed의 관측이다.');}
//10
{let s=slide('DP3 기본안: 화면과 발화의 지칭 결합',SOURCES);S.three(s);}
//11
metrics('DP3 기존 실측: 빠른 관측과 미관측 구별',BASIC.slice(5,7),'이전 A callback 10/15·완료 9/12, B 15/15·12/12의 세대다.\n별도 pixel 기본은 A 12/15·10/12·19/24, B 14/15·11/12·22/24로 양쪽 미완료.\n이 pixel 판본의 전체 승자는 미확정. 빠른 관측과 실패를 함께 본다.');
// Fresh DP04 is a distinct measured generation. Never overwrite original table.
{
 const fp=path.join(ROOT,'results/architecture-evaluation/current/four-dp-presentation-20261019/dp04/basic/summary.json');
 const a=DATA.dp04_basic.rows;
 const vals=a.map(r=>[r.variant,`${(r.qa01_ms/1000).toFixed(3)}  ${r.qa01_n}/${r.qa01_expected}`,`${r.qa02_correct}/${r.qa02_total}`,`${(r.qa03_ms/1000).toFixed(3)}  ${r.qa03_n}/${r.qa03_expected}`,'미측정',`$${(r.qa05_via_usd+r.qa05_receiver_usd).toFixed(7)}${r.unknown_usage_calls?'\n+미확정 1회':''}`,r.qa06_peak_rss_mb.toFixed(3)]);
 metrics('DP3 pixel 기본: 양쪽 모두 미완료',vals,'동일한 pixel 출력 계약의 기본 24회. 엄격한 전체 승자는 미확정.\nA의 provider JSON 생성 400은 사용량 미확정 1회·예약 상한 $0.0729404를 보존했다.\nB 1회는 정정 대상은 맞지만 bbox가 frame 높이를 초과했다. 기존 결과와 섞지 않는다.');
}
//12
{let s=slide('DP3 보완: 실패 범위 재판단과 독립 생산 병행',SOURCES+'\nA2+A5는 원본 PNG와 raw pointer/path overlay를 계속 제공하며, 검증 거절 뒤 좌표 view를 보조 입력으로 추가한다. 원본을 정답 crop으로 교체하거나 최초 호출부터 확대 bbox를 주지 않는다. 기본 B 생산은 순차이며 B4에서만 같은 K 안의 병행을 사용한다.');columns(s,[{title:'A2+A5  실패 때만 원본 재판단',tactic:true,nodes:['1. 공통 guard가 실제 거절','2. 영향 부분 또는 raw cue crop','3. grid view·좌표 affine 확인','4. 같은 의미 책임이 다시 판단'],tradeoff:'정답 bbox를 제공하지 않음\n추가 호출·복구 실패 그대로 보존'},{title:'B4  독립 producer 병행',tactic:true,nodes:['1. 같은 K 호출 슬롯 예산','2. 화면과 발화 producer 병행','3. 양쪽 revision 도착 후 join','4. 기존 binder와 gate 그대로'],tradeoff:'초기 생산 대기를 겹침\n둘의 근거가 있어야 최종 결합'}],{note:'pixel 출력 48회의 보완이다. 이전 A4도 24/24를 회복했으나 시간·비용을 더 썼다. 정규화 A에 A2/A5를 추가한 실험은 없다.'});}
// Pixel-output baseline/tactic population remains separate from normalized sensitivity.
{
 const rows=DATA.dp04_pixel_comparison.summary.map(r=>[r.variant==='A2A5'?'A2+A5':r.variant,`${(r.qa01_conditional_mean_ms/1000).toFixed(3)}  ${r.qa01_observed}/${r.qa01_expected}`,`${r.qa02_correct}/${r.qa02_total}`,`${(r.qa03_conditional_mean_ms/1000).toFixed(3)}  ${r.qa03_completed}/${r.qa03_expected}`,'미측정',`$${r.via_usd.toFixed(7)}${r.unknown_usage_calls?'*':''}`,r.max_concurrent_rss_mb.toFixed(3)]);
 metrics('DP3 원본 pixel 출력의 기본·보완 48회',rows,'A2+A5는 15/15·12/12·24/24를 회복했지만 최고 RSS가 41.452 → 50.037 MB.\nB4는 첫 응답 13/15·완료 11/12·집합 20/24로 전체 회복에 실패했다.\nA 공통 첫 응답 12개는 9.268 → 10.875초. 회복과 첫 응답 부담을 교환했다.\n* A 비용은 사용량 미확정 1회를 제외한 확정액. 평균의 관측 분모가 달라 전체 개선율로 쓰지 않는다.');
}
// Final common normalized-output baseline: independent from pixel-only recovery tactics.
{
 const rows=DATA.dp04_normalized.options.map(r=>[r.variant,`${(r.qa01_ms/1000).toFixed(3)}  ${r.qa01_n}/15`,`${r.qa02_correct}/24`,`${(r.qa03_ms/1000).toFixed(3)}  ${r.qa03_n}/12`,'이번 판본\n미측정',`$${r.qa05_via_usd.toFixed(7)}\n${r.model_calls}회 호출`,r.qa06_peak_rss_mb.toFixed(3)]);
 let s=slide('DP3 공통 정규화 판본: 조건부 기본 A',SOURCES+'\nA joint/B screen에 같은 qwen_normalized_0_1000 bbox_2d→original frame pixel 경계를 사용한다. raw pointer/path는 원래pixel그대로. no clamp/oracle,동일guard/ownerboundedrecovery. A2/A5/B4없음. normalized-comparison.json의 고정24실행/57요청/사용량확정57/UNKNOWN0. 이번QA04미측정(이전A/B각2.5별도).\nA만이번판본 first15/15,goal12/12,QA02 24/24. B dense3회는 초기올바른라벨을제안했으나 bbox와rawpointer기하불일치, 원생산자확장뒤need_evidence/unknown으로끝났다. normalized A+A2/A5 조합실험은없다.');
 table(s,[['안','QA01 초 / 관측','QA02','QA03 초 / 종료','QA04','QA05 USD','QA06 MB'],...rows],{height:230,font:22,columns:[85,200,100,200,190,225,188]});
 text(s,'이 공통 출력 계약에서 A를 조건부 기본안으로 선택한다.',48,415,1180,40,27,C.blue,true);
 lines(s,'A만 첫 응답 15/15·종료 12/12·지칭 24/24를 관측했다. B는 밀집 화면 3회 미완료.\nA/B 모두 0..1000 출력의 같은 변환을 쓴다. 원시 지시점·원본·공통 guard는 유지했다.\nQA04는 이번 판본 미측정이다. 이전 A/B 각 2.5는 별도 원값으로 남겼다.\nA2/A5/B4는 이 판본에 없다. pixel 48회의 보완 성적과 분리한다.',48,468,1180,24,137);
 foot(s,'관측 조건부 평균과 고정 분모를 함께 표시한다. 정규화 A에 A2/A5를 더한 조합은 실행하지 않았다.');
}
//13
{let s=slide('DP4 기본안: Agent 간 업무 인계',SOURCES);S.four(s);}
{let s=slide('DP4 C: VIA와 수신자의 읽기 책임',SOURCES);S.fourC(s);}
//14
metrics('DP4 실측: 첫 응답 미확정과 조건부 C',BASIC.slice(7,10),'같은 최초 채택 함수여도 실제 원문 입력은 다르다. QA01 우열 미확정.\n첫 21호출 입력 token 합계 A 15,318 / B 41,002 / C 13,722. QA02는 비적용.\n공통 완료 16개 목표 VIA 시간 A 10.565 / B 12.299 / C 4.213초.\n전체 경과시간 A 31.372 / B 31.406 / C 23.532초. 완료 20/19/19도 함께 보존.\n저자 본문 충족 B 15/18 / C 12/18 / A 8/18은 B의 실제 반대 이익.');
//15
{let s=slide('DP4 조건부 C와 조건 원문 보완',SOURCES);columns(s,[{title:'C1  기채택 조건 원문 prefetch',tactic:true,nodes:['1. 채택된 조건의 provenance','2. 권한·현재성 확인 후 읽기','3. 실제 원문을 capsule에 제공','4. 수신자가 상세 선택·결과'],tradeoff:'조건 원문 누락 줄이기에 한정 적용\n기존 provenance 보유가 전제'},{title:'C2  같은 범위 읽기 cache',tactic:true,nodes:['1. 수신자의 실제 범위 요청','2. source·revision·range·권한 key','3. 매 hit 현재 권한 재확인','4. 동일 원문 반환·실제 read 절약'],tradeoff:'전체 서비스 지연 이익 없어 보류\n모델 비용 절감으로 해석 금지'}],{note:'QA01 입력 차이로 우열 미확정. C는 읽기 지원 조건부 권고다. C1/C2는 독립 보완이며 조합 결과가 아니다.'});}
// C1 is a condition-provenance availability experiment, not a BASIC replacement.
{
 const rr=DATA.dp05_fresh.rows;const rows=rr.map(r=>[r.tactic||'C',`${(r.qa01_ms/1000).toFixed(3)}  ${r.qa01_n}/9`,'N/A',`${(r.qa03_ms/1000).toFixed(3)}  ${r.qa03_n}/9`,'미측정',`$${r.total_cost_usd.toFixed(7)}\n${r.model_calls}회 호출`,r.qa06_peak_rss_mb.toFixed(3)]);
 let s=slide('C1 전후: 채택 조건 원문을 인계마다 제공',SOURCES+'\n새 C/C1 E3 × 3, 각9목표. 기존 condition_source_refs가 양쪽에 있는 동일조건이며 provenance 생성비용 미측정. C1은 입력이전 현재 원문1–2행을 실제 읽어 첨부. read3회/1,848bytes/0.200750ms, 첨부직렬화peak323bytes; 미래위임목적/정답요약없음. QA04이번판본 미측정.\n고정18본문 저자검토: C4PASS/2FAIL/2UNRESOLVED/1미표시, C1 9PASS. literal/동치4/9→7/9. 독립mail6 일부검토 일치. C r2 deck는 facts15>hostcap14로 미표시; numeric14가 prompt에 없던 계약제약이며 오류와원시보존. 첫prefix9쌍요청동일이나 C조건강화안내1건으로 올바른첫내용9/9확정불가.');
 const ct=table(s,[['구성','QA01 초 / 관측','QA02','QA03 초 / 종료','QA04','QA05 전체 USD','QA06 MB'],...rows],{height:205,font:23,columns:[110,220,100,220,110,233,195]});for(const c of [0,1,3,5,6])ct.getCell(2,c).text.style={typeface:FONT,fontSize:23,color:C.blue};
 lines(s,'N-H 9/9 → 9/9. 저자 본문 4 PASS·2 실패·2 모호·1 미표시 → 9 PASS.\n공통 완료 8개 목표: VIA 4.302 → 4.284초 / 전체 경과 30.532 → 31.519초.\n비용 감소에는 C의 복구 1회가 포함된다. 이를 빼면 $0.0365256 → $0.0376028.\nC의 첫 안내 조건 강화 1건도 보존했다. 9/9 callback은 올바른 내용 9/9의 확정이 아니다.\n조건 provenance가 이미 있을 때 원문 조건 누락을 줄이는 범위에 한정 적용한다.',48,400,1180,24,207);
 foot(s,'본문은 별도 저자 진단. 수신자 API 경계 제외 VIA 시간과 전체 경과를 구별하며 미표시 초안은 성공으로 세지 않는다.');
}
// Native and recorded-response resource measurements, not model latency trials.
{
 let s=slide('보존·읽기 보완의 실제 이익과 대가',SOURCES+'\n새 dp01/native/summary.json,dp01/replay-summary.json,dp05/replay-summary.json\nA3 한계: 작은 실제view12쌍에서는 보존경계에 닿지 않아 A3 미발동. 큰 view→locator 전환 후 실제 Qwen 판단이 직접완료에서 READ+answer로 바뀌면 호출/QA03/QA05가 늘 수 있다. 이 큰 원문 경계는 native 검사뿐이며 전체 모델 경로 이익은 미측정.');
 table(s,[['비교 대상','보완 전','보완 후','교환·판정'],['A3 실제 응답 12쌍의 원문 읽기','39회','39회','이 모집단은 보존경계 미도달'],['A3 큰 원문 native 유지 상태','8,555 bytes','146 bytes','재조회 12회 추가. 이번 기본 적용 보류'],['C2 같은 판본·범위 원문 읽기','34회','22회','12회 cache hit. 결과·전송 입력 동일'],['C2 실제 read 서비스 합계','7.602 ms','8.784 ms','직렬화 cache 최대 14,790 bytes. 적용 보류']],{height:335,font:23,columns:[365,195,195,433]});
 lines(s,'A3는 보존 상태를 줄이는 대신 원본을 다시 읽는다.\nC2는 파일 읽기 호출을 줄였지만 작은 서비스 구간의 시간은 좋아지지 않았다.\n둘 다 API 비용·모델 응답 시간 개선으로 해석하지 않는다.',48,516,1180,25,101);
}
addDP5({slide,text,rect,table,lines,foot,columns,C});
//16
{let s=slide('DP1–4의 기존 권고와 적용 조건',SOURCES);table(s,[['DP','권고','가장 높은 판단 근거','선택을 바꿀 조건'],['1','조건부 A','QA01 미확정. 재사용·완료 관측\n본문 30/36, 조건 미충족 6건','독립 요청 위주 또는 입력 부담 증가'],['2','B','호출 여유가 있는 독립 후속 입력의 QA01','K=1 또는 겹치는 입력이 적음'],['3','조건부 A','정규화 응답·종료 관측\nQA04 이번 미측정','다른 화면·모델에서 A 실패,\nB 근거의 반복 재사용'],['4','조건부 C','QA01 미확정. QA03/05 관측 이익','수신자 읽기 미지원, 실제 내용 조건']],{height:382,font:25,columns:[75,180,505,428]});foot(s,'DP1–4의 기존 조건부 권고다. DP5는 앞 절의 설계 가설이며 기본안 선택과 실측은 아직 없다.');}
//17
metrics('기본안 전체 원값: DP1과 DP2',BASIC.slice(0,5),'시간은 관측 조건부 평균과 고정 분모. 실패를 평균 시간으로 대체하지 않는다.');
//18
metrics('기본안 전체 원값: DP3과 DP4',BASIC.slice(5,10),'이 표는 기존 effective 120이다. 새 live 실험의 기본/보완은 별도 세대로 표시한다.');
//19
{let s=slide('실패와 측정 범위',SOURCES);lines(s,'기본 비교는 작성 사례 4개 × 3회인 유한 PoC\n\n첫 내용 callback은 실제 제품 UI와 모든 문장의 정확성 검증을 대신하지 않는다.\n목표 완료 판정과 별도 본문 충족 진단을 구별한다.\n수신자 API 경계 제외시간과 전체 경과시간을 함께 본다.\nPython/Rust RSS에는 실험 기록 부담이 있고 클라우드 모델 가중치는 없다.\n변경 모듈 수를 인월이나 제품 유지보수 비용으로 바꾸지 않는다.',48,151,1180,29,300);lines(s,'추가 실패: C fresh 초안 1건은 미안내 상한 14개에 사실 15개를 반환해 미표시.\n정규화 B의 밀집 화면 3회는 모두 기하 근거가 미해결이었다.',48,470,1180,23,93);foot(s,'실패, calibration, 중단된 교정, 원본/최종 교정, 보완과 진단의 원시 이력을 보존했다.');}
// Concrete failed endpoints remain evidence, not invented latency penalties.
{
 let s=slide('실패 원문과 끝난 시각',SOURCES+'\n원인 대조: analysis/dp02-explanation.md,dp05-final-provenance-failure-audit.json,dp05-final-provenance-failure-audit-C-r1.json');
 table(s,[['실제 사례','확인한 실패','입력부터 실제 실패'],['DP1 B 원본 q2','잘못된 source span. 본문 미표시','21.054초'],['DP2 A E1 r3 R1','인용·판본 계약 복구 소진. null·충돌 아님','41.759초'],['DP3 pixel A 1회','provider JSON 생성400. 사용량 미확정','성공 지연으로 대체 안 함'],['DP4 B E4 r1','실제 16행 인용을 17행으로 신고','63.338초'],['DP4 C E4 r1','실제 5행 인용을 4행으로 신고','38.173초']],{height:355,font:23,columns:[260,622,306]});
 lines(s,'실패 시간은 별도 attempt endpoint다. 성공 QA01/03 평균에 넣는 임의 벌점이 아니다.\n미표시 초안은 성공 본문으로 세지 않았고 원시·복구·사용 비용을 남겼다.',48,543,1180,25,78);
}
//20
{let s=slide('DP1–4의 기존 근거와 재현 진입점',SOURCES);lines(s,'설계와 선택 계약\nseven-problem-review의 14 계약, 15 기존결과, 16 추가계약, 17 발표검토\n\n기존 전체 원시·실패·재현 패키지\ncurrent/four-dp-poc-evidence-20261019/README.md\n\n새 기본·보완·재생 근거\ncurrent/four-dp-presentation-20261019/ 각 DP의 plan,freeze,summary,raw\n\n추가 실제 요청 331회: 확정 환산 $0.4735528 + 사용량 미확정 1회.\n시간·비용·메모리를 다른 실행 세대와 섞지 않고 원본으로 추적한다.',48,145,1180,28,462);foot(s,'비용은 입력 $0.40·출력 $1.60 / 100만 token의 고정 비교 단가 환산이다. 공급자 청구액은 아니다.');}
// Execution/export only after live measurement window closes.
const config={slideCount:pres.slides.items.length,requiredNativeTableOwnerSlides:tableOwners,fontPolicy:{basis:'reference',families:[FONT],referencePath:'/Users/changbae/Workspace/via-sw-architecture/docs/presentations_files/four-dp-final-20261019/output/VIA-four-DP-20261019-final-v1.pptx',referenceSha256:'7f00e611a6cfea5b3eefee9a8aab7f9ea9ad10006868afe2f8d2ab018f7a6667'},expectedSlideSizeEmu:'12192000,6858000'};
if(process.argv.includes('--export')){
 const out=path.join(WORK,'output');await fs.mkdir(out,{recursive:true});const draft=path.join(WORK,'.build',`candidate-${REV}.pptx`);await(await PresentationFile.exportPptx(pres)).save(draft);
 const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')).href);
 await fs.mkdir(path.join(WORK,'.build','draft-output'),{recursive:true});const final=process.argv.includes('--final')?path.join(out,`VIA-five-DP-with-memory-20261020-${REV}.pptx`):path.join(WORK,'.build','draft-output',`VIA-five-decisions-${REV}.pptx`);await finalizePresentation({workspaceDir:WORK,candidatePath:draft,finalPath:final,explicitTotalSlideCount:config.slideCount,requiredNativeTableOwnerSlides:tableOwners,fontPolicy:config.fontPolicy,verifyArtifactToolImport:true,pythonExecutable:process.env.RUNTIME_PYTHON||'/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3',integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu',config.expectedSlideSizeEmu,'--validate-bullet-geometry','--validate-heading-fit',...tableOwners.flatMap(n=>['--require-native-table-slide',String(n)])],receiptPath:path.join(WORK,'.build',`validation-${REV}.json`)});
 const rendered=await PresentationFile.importPptx(await FileBlob.load(final));await fs.mkdir(path.join(WORK,'.build',`renders-${REV}`),{recursive:true});const pngs=[];const sha=b=>createHash('sha256').update(b).digest('hex');for(let i=0;i<rendered.slides.items.length;i++){let png=await rendered.export({slide:rendered.slides.items[i],format:'png',scale:1});let bytes=new Uint8Array(await png.arrayBuffer());const fp=path.join(WORK,'.build',`renders-${REV}`,`slide-${String(i+1).padStart(2,'0')}.png`);await fs.writeFile(fp,bytes);pngs.push({slide:i+1,title:slideTitles[i],path:fp,bytes:bytes.length,sha256:sha(bytes),native_table_owner:tableOwners.includes(i+1)});}const sourceSha={};for(const n of ['build.mjs','structures.mjs','dp5.mjs','deck-data.json'])sourceSha[`source/${n}`]=sha(await fs.readFile(path.join(WORK,'source',n)));await fs.writeFile(path.join(WORK,'.build',`export-manifest-${REV}.json`),JSON.stringify({revision:REV,created_at_utc:new Date().toISOString(),pptx:final,pptx_sha256:sha(await fs.readFile(final)),slide_count:slideTitles.length,font:FONT,expected_slide_size_emu:config.expectedSlideSizeEmu,source_sha256:sourceSha,required_native_table_owner_slides:tableOwners,render_scope:'All PNGs imported from the actual exported PPTX using Artifact Tool. Not a PowerPoint application screenshot.',slides:pngs},null,2)+'\n');await fs.writeFile(path.join(WORK,'.build',`render-list-${REV}.md`),'# '+REV+' — 전체 PNG\n\n'+pngs.map(r=>`${r.slide}. ${r.title} — [PNG](${r.path})`).join('\n')+'\n');console.log(final);
}
await fs.writeFile(path.join(WORK,'.build','requirements.json'),JSON.stringify(config,null,2));
