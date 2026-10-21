// Append four editable DP5 slides to the user's exact v4 Windows reference.
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [ROOT,BUILD]=process.argv.slice(2);
if(![ROOT,BUILD].every(p=>p&&path.isAbsolute(p)))throw Error('Pass absolute repository and private build paths');
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1015.12040/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const FONT='Calibri',W=1920,H=1080;
const C={black:'#111111',gray:'#98999B',grid:'#B7B9B9',green:'#C9E1B3',greenD:'#70A843',cream:'#FFF6DD',blue:'#4676CB',cyan:'#35A8CD',pale:'#F1F6FC',red:'#C00000',muted:'#50555B',white:'#FFFFFF'};
const BASE=path.join(ROOT,'docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP4-reference-style-17slides-v4-windows.pptx');
const OUT='VIA-DP1-DP5-reference-style-21slides-v5-r2-windows.pptx';
const ref=path.join(ROOT,'docs/architecture/12-decisions/decision-packages/seven-problem-review/dp5-delegation-memory-20261020.md');
await fs.mkdir(BUILD,{recursive:true});
// Inspect the actual template without round-tripping its original slide bodies.
const template=await PresentationFile.importPptx(await FileBlob.load(BASE));
await fs.writeFile(path.join(BUILD,'template-inspect.ndjson'),(await template.inspect({kind:'slide,layout',maxChars:60000})).ndjson);
await fs.writeFile(path.join(BUILD,'template-masters.json'),JSON.stringify({masterIds:template.masters.items.map(x=>x.id),layoutIds:template.layouts.items.map(x=>x.id)},null,2));
const original=await fs.readFile(path.join(ROOT,'scripts/presentations/generate_dp4_reference_style.mjs'),'utf8');
let helpers=original.slice(original.indexOf("const line="),original.indexOf('function notes('));
helpers=helpers.replaceAll('DP4 object','DP5 object').replace('DP4. Agent 간 업무 인계 방안','DP5. 위임을 위한 메모리 관리');
const content=String.raw`
function notes(extra=''){
 return 'DP5 설계 가설, 미측정, 미선택. 2026-10-20 사용자 정의의 QA07 요청 의미 해석 정확도는 과거 QA07 부하 추가 지연과 다르다. QA07 우선순위·목표·점수는 미확정이다.\n'+
 '근거: '+ref+'\n기존 발표 DP1 B/C의 요청 시 원본 구성과 사전 관계 생산, DP4 A/B의 준비·갱신 지식을 위임 관계에 구체화했다. 기존 DP1 C가 같은 관계를 생산하면 공유하고 중복 비용·효과를 계산하지 않는다.\n'+
 'A/B 공통: 원본·실제 표시 판본·공개 결과·Task/Agent 연결·명시적 사실·이미 확정한 조건과 허용 선호를 보존하고 재사용한다. 코드로 ID·시각·연결 필터와 텍스트 검색을 수행하며, 검색 결과는 의미 정답이 아니다. Embedding은 선택적 공통 검색 보완이며 이 두 구조의 정의가 아니다. 필요한 의미 해석만 동일 모델 자원을 쓴다. 고정 LLM 호출 횟수 비교가 아니다.\n'+
 'Context Manager는 허용 근거의 조회·파생 관계의 생산과 저장을 담당하고, Request Interpreter는 현재 요청 의미를 제안하며 Request Controller는 최초 현재 의미 채택을 담당한다. 정식 Component 내부의 한국어 상자는 기능이다. 별도 물리 서비스나 모델 가중치를 요구하지 않는다. DP2 실행 구조·DP3 grounding·DP44 개입 정책·Agent 선택 규칙·후속 인계 방식은 공통이다. 원본 권위, 권한·철회·현재 판본 검사를 유지한다. 상세 도메인 추론·계획·실행은 Downstream Agent 책임이다.\n'+
 'B의 valid는 원본 현재성일 뿐 의미 정답을 보장하지 않는다. 미생산·오래된·미확인 범위와 의미 불일치는 원본 보완한다. 명시적 사실이나 이전 요청에서 이미 확정한 관계는 A도 재사용한다. 새 QA07은 최초 확정한 대상·목적·조건·기존 요청 관계가 모두 맞는지 입력 단위로 판단한다. 지칭 오류가 QA02/07을 함께 실패하게 할 수 있으며 독립 이익으로 합산하지 않는다. 인계 후 조건 누락·실제 Agent 실행 성패는 두 정확도에 포함하지 않는다.\n'+
 'QA01 이익은 관계 해결이 첫 올바른 구체적 내용 표시를 실제로 막을 때만 가능하다. 먼저 표시 가능한 다른 올바른 구체 내용이 있으면 두 안에 동일하게 허용한다. QA03은 요청 임계 경로와 실제 background 경합을 포함하고 사전 생산 시간은 단계별로 드러낸다. QA05는 같은 전체 episode에서 미소비 생산·갱신·재시도·요청·수신자 비용을 포함한다. QA04 실제 변경 모듈과 QA06 로컬 peak RAM은 미측정이다.\n'+extra;
}
function footer(s,t){txt(s,t,28,1025,1775,27,20,{color:C.muted});}
// 18: background, matching the reference's document/Agent event diagram.
{
 const s=pres.slides.add();header(s,' (배경)',18);
 txt(s,'사용자가 기억하는 자료와, 마지막 작업 Agent는 다를 수 있다',66,145,1780,62,42.667,{bold:true});
 txt(s,'VIA는 실제 본 자료, 좋다고 평가한 측면과 적용 상황을 연결해 다음 위임의 의미를 정해야 한다.',68,215,1770,40,28);
 const xs=[90,690,1290];
 const headings=['① R이 조사 D1 작성','② P가 발표 D2 작성','③ Q가 내부회의용 D3 수정'];
 const titles=['조사 결과 D1','발표자료 D2','수정본 D3'];
 const bodies=['실제 생산자 R\n조사 내용의 출처','실제 작성자 P\n외부 고객용 구성의 참조','실제 수정자 Q\n가장 최근 결과와 Agent'];
 xs.forEach((x,i)=>{txt(s,headings[i],x,285,540,42,30.667,{bold:true});doc(s,titles[i],bodies[i],x+20,350,460,185);});
 poly(s,[[570,441],[710,441]],C.black,2,false,true);
 poly(s,[[1170,441],[1310,441]],C.black,2,false,true);
 box(s,'사용자 피드백: “외부 고객에게는 처음 구성이 좋고, 이번 내부회의에는 수정본을 쓰자.”',110,574,1660,64,{fill:C.green,size:29.333,bold:true});
 box(s,'새 요청: “고객용으로 좋다고 한 형식에 이번 조사 수치를 넣고, 그 자료 만든 쪽에 맡겨.”',110,670,1660,76,{fill:C.pale,stroke:C.blue,size:29.333,bold:true});
 txt(s,'연결할 의미: 형식 D2 / 내용은 신규 조사 / 명시적으로 지칭한 작성자 P',110,770,1660,47,30.667,{bold:true,align:'center'});
 txt(s,'최신 수정자 Q로 바꾸거나, “모든 업무에서 P 선호”로 일반화하면 다른 요청이 된다.',110,832,1660,40,28,{color:C.red,align:'center'});
 poly(s,[[65,913],[1830,913]],C.blue,1.5);
 txt(s,'이 DP의 질문',66,942,275,52,34.667,{bold:true,color:C.red});
 txt(s,'위임 관계를 현재 요청에서 해석할까, 미리 정리해 둘까?',355,940,1475,56,37.333,{bold:true});
 txt(s,'원본과 명시적 사실은 공통 보존. 미래 요청에 쓰일 관계의 해석 시점과 관리 책임을 선택한다.',355,1005,1460,37,25.333);
 s.speakerNotes.textFrame.setText(notes('설명용 사례이며 실행 fixture가 아니다. 실제 display 증거가 D2/D3와 발화의 대응을 지지해야 한다. 원본도 모호하면 동일 확인 절차를 사용한다. 과거 feedback가 이미 현재 요청으로 확정되어 공통 상태에 있다면 B만의 생산 이익으로 세지 않는다.'));
}
// 19: two different ownership/lifetime structures, not two sequence diagrams.
{
 const s=pres.slides.add();header(s,' (설계 구조 비교)',19);
 txt(s,'차이의 중심: 요청이 관계 해석을 소유하는가, 독립 갱신 경로가 관계 상태를 유지하는가',28,126,1840,43,30.667,{bold:true});
 box(s,'A  요청 경로에서 원본 관계 해석',40,187,875,50,{fill:C.green,size:30.667,bold:true});
 box(s,'B  사건 갱신 경로 + 관계 조회 경로',1005,187,875,50,{fill:C.green,size:30.667,bold:true});
 // A: one persisted source authority; only request-scoped interpretation workspace.
 storeShape(s,'공통 원본·확정 사실\n표시 판본·피드백\nTask / Agent 연결',65,285,250,132,{size:25.333});
 component(s,'Context Manager',355,270,535,147);
 box(s,'원본 조회 인터페이스\nID·시각·연결 필터 / 텍스트 검색',370,322,505,77,{fill:C.white,size:25.333});
 poly(s,[[315,352],[355,352]],C.black,2,false,true);
 txt(s,'요청과 무관하게 보존',67,429,247,31,22.667,{color:C.muted,align:'center'});
 box(s,'현재 요청',65,537,250,66,{fill:C.white,size:29.333,bold:true});
 shape(s,'rect',355,484,535,357,C.pale,C.blue,3,'A request-scoped interpretation boundary');
 txt(s,'Request Interpreter',369,494,507,38,29.333,{bold:true,align:'center'});
 txt(s,'요청 단위 실행·작업 상태',370,536,505,30,24,{color:C.blue,align:'center'});
 box(s,'원본 관계 해석\n필요한 범위 추가 읽기',370,589,245,83,{fill:C.white,size:24});
 box(s,'임시 해석 상태\n후보·근거·불확실성',639,589,236,83,{fill:C.white,stroke:C.blue,size:24,dash:true});
 poly(s,[[315,570],[340,570],[340,648],[370,648]],C.black,2,false,true);
 poly(s,[[491,417],[491,450],[330,450],[330,610],[370,610]],C.black,2,false,true);
 txt(s,'원본·사실',500,445,180,30,22.667,{color:C.muted});
 poly(s,[[615,630],[639,630]],C.blue,2,false,true);
 box(s,'현재 목적·조건과 함께 의미 제안',370,726,505,62,{fill:C.white,size:25.333,bold:true});
 poly(s,[[757,672],[757,704],[620,704],[620,726]],C.blue,2,false,true);
 txt(s,'채택한 조건·확정 사실은 공통 상태로 보존',365,798,515,30,22.667,{color:C.muted,align:'center'});
 txt(s,'추가 영속 상태: 없음\n미확정 관계는\n요청에서 해석',65,701,260,100,24,{color:C.muted,align:'center'});
 // B: Context Manager owns a producer, versioned projection and read gate.
 shape(s,'rect',1260,267,605,473,C.cream,C.black,2.7,'B Context Manager owned production and query state');
 txt(s,'Context Manager',1270,273,585,36,29.333,{bold:true,align:'center'});
 poly(s,[[1260,315],[1865,315]],C.black,1.4);
 txt(s,'사건 갱신 경로 / 요청 조회 경로',1275,325,570,30,24,{color:C.blue,bold:true});
 storeShape(s,'같은 원본·사실',1020,381,205,87,{size:24});
 box(s,'변경 추적\n생산 대상 / 정정',1280,381,235,87,{fill:C.white,size:24});
 box(s,'관계 생산·게시\n해석 → 판본 재검사',1555,381,290,87,{fill:C.pale,stroke:C.blue,size:24,bold:true});
 poly(s,[[1225,423],[1280,423]],C.black,2,false,true);
 poly(s,[[1515,423],[1555,423]],C.blue,2,false,true);
 storeShape(s,'의존 판본·처리 cursor\ndirty / revoked',1265,516,250,89,{size:24});
 storeShape(s,'영속 파생 관계\n대상·측면·상황·출처',1555,516,290,89,{fill:C.pale,stroke:C.blue,size:24});
 poly(s,[[1700,468],[1700,516]],C.blue,2,false,true);
 poly(s,[[1395,468],[1395,516]],C.black,2,false,true);
 poly(s,[[1515,559],[1555,559]],C.red,2,true,true);
 txt(s,'정정 시 무효화',1248,480,300,29,21.333,{color:C.red,align:'center'});
 box(s,'관계 조회 / 현재성·확인 범위 검사',1280,660,565,58,{fill:C.white,size:25.333,bold:true});
 poly(s,[[1700,605],[1700,660]],C.blue,2,false,true);
 poly(s,[[1395,605],[1395,660]],C.black,2,false,true);
 poly(s,[[1300,660],[1300,633],[1120,633],[1120,468]],C.gray,2,true,true);
 txt(s,'미생산·무효·부족\n→ 원본 보완',1035,530,220,70,22.667,{color:C.muted,align:'center'});
 box(s,'현재 요청',1020,789,205,64,{fill:C.white,size:28,bold:true});
 box(s,'Request Interpreter\n관계 적용 판단 + 부족한 의미 해석',1280,783,585,78,{fill:C.pale,stroke:C.blue,size:25.333,bold:true});
 poly(s,[[1225,821],[1280,821]],C.black,2,false,true);
 poly(s,[[1380,783],[1380,718]],C.black,2,false,true);
 poly(s,[[1740,718],[1740,783]],C.blue,2,false,true);
 txt(s,'조회',1310,746,65,28,21.333,{color:C.muted});
 txt(s,'근거',1760,746,70,28,21.333,{color:C.blue});
 // Shared adoption/dispatch and model resources are explicitly outside the choice.
 box(s,'공통  Request Controller: 현재 의미 채택·표시  →  동일한 Agent 선택·위임 계약',40,891,1840,48,{size:28,bold:true});
 poly(s,[[622,841],[622,891]],C.black,2,false,true);
 poly(s,[[1570,861],[1570,891]],C.black,2,false,true);
 const t=nativeTable(s,[['구조적 대가','A: 원본 조합이 요청 임계 경로에 남음','B: 생산·게시·무효화·조회 계약을 추가 관리']],40,959,[225,807,808],[46],24);
 cellStyle(t.getCell(0,0),24,C.black,true);cellStyle(t.getCell(0,2),24,C.red);
 footer(s,'논리적 실행·상태 경계이며 별도 프로세스·모델 복제를 뜻하지 않는다. 필요한 의미 처리만 같은 Model Access 사용. 파생 관계는 원본 권위를 대체하지 않는다.');
 s.speakerNotes.textFrame.setText(notes('구조도 개정: A는 원본 조회 인터페이스와 요청 단위 Request Interpreter 작업 상태를 갖는다. B는 Context Manager 내부에 사건 기반 생산·게시, source revision/처리 cursor/dirty 상태, 영속 파생 관계, 검증된 조회 인터페이스를 추가한다. 작은 상자는 제안한 내부 책임이며 별도 정식 Component가 아니다. A도 명시적 연결과 채택한 사실을 재사용한다. A 임시 상태는 현재 요청에서 생성한 작업 후보를 뜻하며 확정 사실 삭제를 뜻하지 않는다. B의 변경 추적은 논리적 처리 목록으로 별도 broker나 강제 프로세스 격리가 아니다. source 변경과 의미 불일치는 사용을 무효화하며 갱신이 실패하거나 미완료면 요청은 원본을 보완한다. 의미 적용 중 발견한 부족함도 조회 인터페이스를 통해 같은 원본으로 보완한다. 생산에 필요한 원문은 같은 원본 읽기 경로를 사용한다. QA는 21장에 전체 비교한다.'));
}
// 20: retain the selection/tactic vocabulary, without fabricating a selected winner.
{
 const s=pres.slides.add();header(s,' (선택 기준 및 보완)',20);
 txt(s,'기본안 미선택 / 과거 관계의 재참조 빈도와 첫 응답의 실제 대기로 판단',28,126,1835,42,30.667,{bold:true,color:C.blue});
 const t=nativeTable(s,[
 ['판단 기준','A가 유리한 조건','B를 검토할 조건'],
 ['사용 형태','명시적 사실 조회가 많고 과거 관계 재참조가 적음','처음 다시 부르는 과거 관계가 응답을 자주 막음'],
 ['품질 교환','필요한 요청에만 비용 사용 / 요청 때 조합 대기','요청 경로의 조합 절약 / 미소비 생산·갱신 부담'],
 ['선택을 바꿀 조건','원본 조합이 응답 지연과 관계 누락을 반복 유발','원본 보완이 잦거나 잘못된 관계·범위를 반복 적용']
 ],24,195,[250,811,811],[50,73,73,73],26.667);highlightHeader(t,3);
 const xs=[55,1005];
 const titles=['A1  요청에서 확인한 관계의 재조회 색인','B1  같은 업무의 연속 사건을 묶어 생산'];
 titles.forEach((t,i)=>box(s,t,xs[i],515,850,58,{fill:C.green,stroke:C.blue,size:29.333,bold:true}));
 component(s,'Context Manager',xs[0]+20,606,810,168);
 box(s,'요청에서 확정한 관계를 후속 검색용 색인에 반영',xs[0]+37,650,776,50,{fill:C.pale,stroke:C.blue,size:26.667,color:C.blue});
 box(s,'의존 원본 변경 때 갱신 / 기존 권한·현재성 검사 유지',xs[0]+37,715,776,43,{fill:C.pale,stroke:C.blue,size:25.333,color:C.blue});
 poly(s,[[xs[0]+425,700],[xs[0]+425,715]],C.blue,2,false,true);
 component(s,'Context Manager',xs[1]+20,606,810,168);
 box(s,'호환 사건을 모아 겹치는 관계를 한 번 생산',xs[1]+37,650,776,50,{fill:C.pale,stroke:C.blue,size:26.667,color:C.blue});
 box(s,'게시 전 요청이 오면 미처리 범위를 원본으로 보완',xs[1]+37,715,776,43,{fill:C.pale,stroke:C.blue,size:25.333,color:C.blue});
 poly(s,[[xs[1]+425,700],[xs[1]+425,715]],C.blue,2,false,true);
 txt(s,'기대: 두 번째 유사 요청의 검색·조합 감소',xs[0]+20,799,810,40,28,{color:C.blue,bold:true});
 txt(s,'대가: 색인 갱신·메모리 / 첫 재참조 이익은 없음',xs[0]+20,850,810,42,26.667,{color:C.red});
 txt(s,'기대: 같은 관계의 중복 생산·갱신 감소',xs[1]+20,799,810,40,28,{color:C.blue,bold:true});
 txt(s,'대가: 게시 공백 증가 / 원본 보완 시 요청 지연',xs[1]+20,850,810,42,26.667,{color:C.red});
 poly(s,[[28,939],[1894,939]],C.blue,1.5);
 txt(s,'A의 확정 사실 재사용과 B의 필수 원본 보완은 기본 기능이다. 위 보완은 기본안 선택 후 별도 비교한다.',28,953,1810,53,28,{bold:true});
 footer(s,'보완 효과 미측정. A1/B1을 미리 넣어 기본안 차이를 없애거나, 어느 보완이 반드시 유리하다고 가정하지 않는다.');
 s.speakerNotes.textFrame.setText(notes('현재 기본안 선택은 보류다. 원본에 이미 존재하는 명시적 Task·작성자 관계와 확정 선호는 A도 조회한다. A1은 사실의 기본 보존이 아니라 추가 후속 검색 색인이다. B1은 batching으로 미소비 비용을 항상 없애는 구조가 아니며 게시 지연을 비용으로 가진다.'));
}
// 21: keep all QA comparisons legible after dedicating slide 19 to structure.
{
 const s=pres.slides.add();header(s,' (QA Tradeoff 상세)',21);
 txt(s,'요청 경로의 관계 조합 대기  ↔  요청 전에 발생하는 생산·갱신 비용',28,129,1840,50,34.667,{bold:true,color:C.blue});
 const values=[
 ['QA / 차별화','A: 요청 소유 해석','B: 영속 관계 생산·조회'],
 ['QA01 첫 응답\n주요 조건부 가설','원본 조합이 첫 올바른 내용에 필수면 대기','유효 관계가 그 조합을 대체할 때 대기 절약'],
 ['QA02 지칭 정확도\n양방향 · 우열 미정','흩어진 원본의 연결 누락 위험','잘못 묶인 대상 재사용 위험 / 원본 보완'],
 ['QA03 VIA 처리\n주요 조건부 가설','요청 안에서 원본 탐색·조합 수행','조회·적용으로 단축 가능 / 보완·경합 비용'],
 ['QA04 변경 모듈\n관련 · 미확정','원본 조회와 요청 해석 경로 변경','생산·무효화·조회 계약 변경 / 재사용 가능'],
 ['QA05 전체 모델 비용\n주요 반대 비용','실제 요청된 관계에 해석비용 사용','미소비 생산·정정·재시도 포함 / 재사용 상쇄'],
 ['QA06 로컬 peak RAM\n관련 · 우열 미정','요청의 원본 문맥·임시 상태 부담','파생 상태·생산 작업 추가 / 요청 문맥 축소'],
 ['새 QA07 요청 의미\n양방향 · 우열 미정','현재 목적에서 해석 / 조건 연결 누락 가능','측면·상황 재사용 / 과도한 일반화 가능']
 ];
 const t=nativeTable(s,values,28,207,[335,765,765],[51,73,73,73,73,73,73,73],26.667);highlightHeader(t,3);
 for(let r=1;r<values.length;r++)cellStyle(t.getCell(r,0),24,C.black,true,'left');
 cellStyle(t.getCell(5,2),25.333,C.red);
 box(s,'B가 유리해지는 장면',28,817,900,45,{fill:C.green,size:28,bold:true});
 box(s,'A가 유리해지는 장면',993,817,900,45,{fill:C.green,size:28,bold:true});
 txt(s,'처음 다시 부르는 과거 관계가 자주 첫 응답을 막고,\n미리 만든 유효 관계가 원본 조합을 실제로 줄일 때',44,879,865,78,26.667);
 txt(s,'공개 결과·피드백은 많지만 재참조는 적거나,\n새 조건·정정 때문에 B도 원본을 다시 해석해야 할 때',1009,879,865,78,26.667);
 txt(s,'독립 DP의 한계: 공통 사실·A 재사용으로 충분하거나 DP1 C가 같은 관계를 이미 생산하면 차별화가 약해진다.',28,977,1840,36,25.333,{bold:true,color:C.muted});
 footer(s,'설계 가설·미측정·미선택. QA01/03 같은 대기의 중복 효과 주의. QA02/07 같은 대상 오류를 독립 이익으로 합산하지 않음. QA04는 실제 변경 모듈로 검증.');
 s.speakerNotes.textFrame.setText(notes('QA02는 첫 지칭 대상 집합, 새 QA07은 필수 요청 의미 전체다. 인계 후 조건 누락이나 실제 Agent 수행 성공을 정확성으로 대체하지 않는다. QA07 목표·우선순위·점수 미확정. 동일 원본·모델·예산에서 최초 재참조/반복/미사용/정정/상황 변경을 비교한다. A의 확인 사실 재사용을 허용한다. QA05는 같은 episode의 생산부터 요청·수신자까지 미사용·재시도 포함이다. 별도 모델 가중치를 B 비용으로 가정하지 않는다. B 원본 보완 또는 갱신과 요청 경합이 많으면 지연 이익이 사라질 수 있다. 표의 요청 경로 단축은 구조의 가능성으로, LLM 성능과 사례 분포를 넘는 확정 우위를 뜻하지 않는다.'));
}

`;
await fs.writeFile(path.join(BUILD,'dp5-content.mjs'),`export function build(pres,common){const {ROOT,FONT,W,H,C,ref}=common;\n${helpers}\n${content}\n}`);
const p=Presentation.create({slideSize:{width:W,height:H}});
(await import(pathToFileURL(path.join(BUILD,'dp5-content.mjs')).href)).build(p,{ROOT,FONT,W,H,C,ref});
if(p.slides.items.length!==4)throw Error('Expected exactly four DP5 slides');
await (await PresentationFile.exportPptx(p)).save(path.join(BUILD,'dp5-authored.pptx'));
execFileSync(PY,[path.join(ROOT,'scripts/presentations/package_dp5_reference_v5.py'),ROOT,BUILD],{stdio:'inherit'});
await fs.mkdir(path.join(BUILD,'output'),{recursive:true});
const final=path.join(BUILD,'output',OUT);
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:BUILD,candidatePath:path.join(BUILD,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:21,
 requiredNativeTableOwnerSlides:[19,20,21],
 pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit','--require-native-table-slide','19','--require-native-table-slide','20','--require-native-table-slide','21'],
 fontPolicy:{basis:'reference',families:['Calibri'],referencePath:BASE,referenceSha256:createHash('sha256').update(await fs.readFile(BASE)).digest('hex')},
 verifyArtifactToolImport:true,receiptPath:path.join(BUILD,'validation.json')});
await fs.mkdir(path.join(BUILD,'previews'),{recursive:true});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
for(let i=0;i<21;i++){
 const slide=imported.slides.items[i];
 const blob=await imported.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.layout.json`),await(await slide.export({format:'layout'})).text());
 console.log('Rendered '+(i+1)+'/21');
}
console.log(final);
