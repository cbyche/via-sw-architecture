import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Run a copy from a private directory linked to the bundled node_modules.
// Current source is immutable; native originals are retained in the appendix.
const [repoArg, buildArg] = process.argv.slice(2);
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(v => path.isAbsolute(v ?? ''))) throw new Error('Set bundled runtime paths');
const source = path.join(repo, 'docs/presentations_files/VIA-architecture-presentation.pptx');
const sourceHash = createHash('sha256').update(await fs.readFile(source)).digest('hex');
if (sourceHash !== '9bd1a17b48343ab9187ebedefb589cef5fdb9d13121d387f6c869dd5f3a1805a') throw new Error('Source changed; rebase the composition before editing');
const p = Presentation.create({slideSize:{width:1920,height:1080}});
const C = {ink:'#202B35', muted:'#576773', teal:'#00889A', navy:'#354354', light:'#F5F7F8', line:'#BCCAD2', peach:'#FFF0DE', blue:'#EAF3F7', white:'#FFFFFF'};
const font = 'Apple SD Gothic Neo';
let s, seq=0;
const authored={}, records=[];
function shape(name,geometry,x,y,w,h,fill='none',stroke='none',width=0){
 return s.shapes.add({name:`v2-${seq++}-${name}`,geometry,position:{left:x,top:y,width:w,height:h},fill,line:{fill:stroke,width}});
}
function txt(name,value,x,y,w,h,size=30,color=C.ink,bold=false,align='left'){
 const o=shape(name,'textbox',x,y,w,h);o.text=value;
 o.text.style={typeface:font,fontSize:size,color,bold,alignment:align,verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
 return o;
}
function rect(name,x,y,w,h,fill=C.white,stroke=C.line,width=2){return shape(name,'rect',x,y,w,h,fill,stroke,width);}
function poly(name,points,color=C.muted,width=2.4,arrow=false,dashed=false){
 const xs=points.map(t=>t[0]),ys=points.map(t=>t[1]),x=Math.min(...xs),y=Math.min(...ys),w=Math.max(1,Math.max(...xs)-x),h=Math.max(1,Math.max(...ys)-y);
 s.shapes.add({name:`v2-${seq++}-${name}`,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:color,width,style:dashed?'dashed':'solid'},
 customPaths:[{width:w,height:h,commands:points.map(([px,py],i)=>({[i?'lineTo':'moveTo']:{x:px-x,y:py-y}}))}]});
 if(arrow){const [a,b]=points.slice(-2),ang=Math.atan2(b[1]-a[1],b[0]-a[0]),len=12;
  poly(name+'-head',[[b[0]-len*Math.cos(ang-.5),b[1]-len*Math.sin(ang-.5)],b,[b[0]-len*Math.cos(ang+.5),b[1]-len*Math.sin(ang+.5)]],color,width);}
}
function box(name,x,y,w,h,fill=C.white,size=29,geometry='rect'){
 shape(name,geometry,x,y,w,h,fill,C.muted,2);txt(name+'-name',name,x+8,y,w-16,h,size,C.ink,false,'center');
}
function note(value,x,y,w,h=44,size=26,color=C.muted){txt('note',value,x,y,w,h,size,color);}
function minLabelWidth(label,w){return Math.min(w-28,label.length>23?380:330);}
function boundary(label,x,y,w,h){rect(label,x,y,w,h,'none',C.line,2);txt(label+'-boundary',label,x+14,y+4, minLabelWidth(label,w),38,27,C.navy,true);}
function base(key,title,section,page,size=48){
 s=p.slides.add();s.background.fill=C.white;authored[key]=p.slides.items.length;
 txt('section',`VIA SOFTWARE ARCHITECTURE / ${section}`,52,20,1250,32,24,C.teal,true);
 txt('title',title,52,63,1816,67,size,C.ink,true);poly('rule',[[52,140],[1868,140]],C.line,2);
 txt('page',`v2 / ${String(page).padStart(2,'0')}`,1740,20,130,32,23,C.muted,false,'right');
 return s;
}
function notes(value){s.speakerNotes.textFrame.setText(value+'\n\n출처: docs/architecture/12-decisions/decision-packages/04-40-common-execution-contract.md; 03-02-quality-attribute-definitions.md; 04-41/44/45/42. 발표 편집이며 QA 정의, Architecture 선택, 구현과 측정 결과를 변경하지 않는다.');}
function table(name,values,x,y,w,h,widths,fontSize=27){
 const t=s.tables.add({name,rows:values.length,columns:values[0].length,left:x,top:y,width:w,height:h,columnWidths:widths,values});
 const headerHeight=name.startsWith('qa-')?40:58;
 for(let r=0;r<values.length;r++){
  t.rows[r].height=r?((h-headerHeight)/(values.length-1)):headerHeight;
  for(let c=0;c<values[0].length;c++)t.cells.block({row:r,column:c,rowCount:1,columnCount:1}).assign({fill:r?(r%2?C.white:C.light):C.navy,
   margins:{left:12,right:8,top:name.startsWith('qa-')?1:5,bottom:name.startsWith('qa-')?1:5},anchor:'center',textStyle:{typeface:font,fontSize,color:r?C.ink:C.white,bold:r===0,alignment:'left'}});
 }
 t.borders.assign({style:'solid',fill:C.line,width:1});
 return t;
}
function overlay(key,page,section){s=p.slides.add();s.background.fill=C.white;authored[key]=p.slides.items.length;
 rect('header-white',1120,12,754,74,C.white,C.white,0);
 txt('page',`${section} / v2 ${page}`,1130,20,734,34,24,C.muted,false,'right');}

// 2. Need: causes before formal quality names and DP identifiers.
base('necessity-v2','사용자의 흐름을 끊지 않고, 의도를 보존하며 업무를 이어가야 합니다','과제 필요성',2);
const needs=[
 ['뜻과 조건을 함께 보존','“메일에는 이 표를 넣어줘.”','바꿀 대상은 메일 내용입니다.\n보고서 PDF와 초안만 조건은 유지해야 합니다.'],
 ['일하는 중에도 새 입력에 반응','보고서 작성 중 새 지시와 질문이 도착','처리 완료를 기다리지 않고 말을 받으며,\n옛 응답과 새 지시를 구별해야 합니다.'],
 ['변화하는 기술과 연결을 수용','새 음성 모델, 업무 AI와 정보원이 등장','기존 사용자 기능을 유지하면서\n영향받는 기능 책임을 수정해야 합니다.']
];
needs.forEach(([head,example,body],i)=>{const x=52+i*614;txt('need-title',head,x,198,580,55,37,C.teal,true);note(example,x,277,580,76,31,C.ink);
 poly('need-line',[[x,375],[x+568,375]],C.line,2);txt('need-body',body,x,414,580,154,30);});
poly('resource-rule',[[52,640],[1868,640]],C.line,2);
txt('resource-heading','계속되는 상호작용에는 운영 비용과 PC 자원도 필요합니다',52,674,1770,55,38,C.navy,true);
txt('cost-cause','의미 재판단, 음성 응답과 과거 정보 준비\n→ 쓰이지 않은 준비와 재시도도 모델 사용량을 만듭니다.',52,763,870,124,31);
txt('memory-cause','진행 업무, 대화 기록과 입출력 버퍼\n→ 사용자 앱과 함께 쓰는 PC 메모리를 점유합니다.',1000,763,870,124,31);
note('업무 AI의 실제 작성 시간과 별개로, VIA가 해석하고 연결하며 만드는 기다림도 줄여야 합니다.',52,958,1780,65,31,C.teal);
notes('필요성에서는 QA/DP ID를 직접 언급하지 않는다. 이전 필요성의 정정과 조건 보존, 지속 입력, 변화 수용을 유지하고 모델 비용과 로컬 자원 원인을 추가했다. VIA 내부 기술과 외부 Downstream Agent의 역할을 구별한다.');

// 4. Domain primer with external dependencies and no premature Component drill-down.
base('domain-primer','VIA는 대화를 관리하고, 업무 AI는 실제 작업을 수행합니다','과제 소개',4);
note('Voice Interaction Agent: 음성, 텍스트와 당시 화면을 함께 받아 PC 업무를 이어주는 프로그램',52,164,1800,54,34,C.teal);
box('사용자',52,300,260,85,C.blue,38);box('로컬 VIA',525,300,450,85,C.blue,38);box('Downstream Agent',1335,300,515,85,C.light,36);
poly('user-via',[[312,342],[525,342]],C.teal,3,true);note('입력과 화면',327,274,210,45,29);
poly('via-agent',[[975,342],[1335,342]],C.teal,3,true);note('검사 후 업무 위임',1014,270,310,48,28);
poly('agent-return',[[1335,408],[750,408],[750,385]],C.muted,3,true);note('질문, 확인된 진행과 결과',932,416,680,48,29);
box('Model Access',525,490,450,65,C.white,32);box('클라우드 음성 모델',1150,490,320,65,C.light,28);box('클라우드 의미 LLM',1500,490,350,65,C.light,28);
poly('via-model',[[750,385],[750,490]],C.teal,3,true);poly('model-voice',[[975,524],[1150,524]],C.muted,3,true);poly('model-meaning',[[975,568],[1675,568],[1675,555]],C.muted,3,true);
note('발화 시작/끝 탐지는 로컬 VAD. 전사와 뜻 해석은 별도입니다.',52,604,1790,52,30);
table('domain-terms',[
 ['일반 SW 관점','VIA에서 쓰는 이름','보고서와 메일 사례'],
 ['이번 입력의 처리','Request','“보고서는 이전 형식으로, 메일은 그대로”라는 이번 요청'],
 ['계속 추적하는 사용자 업무','Task','보고서와 메일 초안을 별도 업무로 유지'],
 ['실제 작업 실행','Execution','외부 업무 AI의 작성 실행. 대화 종료와 수명이 다름'],
 ['이어지는 대화 / 해석 근거','Conversation / Context','음성이 꺼져도 대화 기록과 업무 연결을 유지 / 화면과 과거 자료']
],52,685,1816,281,[402,388,1026],27);
note('다음 장면의 시작 상태: 화면에 표가 있고, 이전 보고서에서 수정한 형식이 허용된 기록으로 남아 있습니다.',52,988,1816,53,27,C.teal);
notes('간단한 전체 흐름을 먼저 설명한 뒤 요구사항, QA와 상세 Component 지도 순서로 확대한다. local VAD는 음성 인식 모델이 아니다. Context와 Task는 Component 이름이 아니라 데이터/개념이다. 클라우드 모델 서비스가 영속 업무 상태의 유일 원본이 아니다.');

// 7. Requirements: factual sources, no fabricated stakeholder interviews.
base('requirements-rationale','대표 시나리오에서 필요한 기능을 추리고, 설계 범위로 좁힙니다','요구사항',7);
txt('source-label','요구의 근거',52,196,540,64,41,C.teal,true);
txt('sources','고정 Use Case와 변형 시나리오\n현재 시스템의 책임과 경계\n공통 실행 계약과 품질 정의',52,301,535,247,34);
poly('req-source',[[613,407],[741,407]],C.teal,3,true);
txt('scope-label','발표에서 설명할 기능',777,196,1090,64,41,C.teal,true);
table('requirement-sources',[
 ['사용 중 생기는 일','필요한 기능','연결'],
 ['말하는 중 정정, 늦은 응답 도착','지속 입력과 맥락 기반 해석','FR1 / FR2'],
 ['보고서 결과로 메일 초안 작성','순서, 조건과 결과 의존 보존','FR3'],
 ['여러 업무의 질문과 결과 도착','해당 업무와 연결, 기존 업무 계속','FR4 / FR5'],
 ['“지난번 형식 / 평가 기준” 재사용','허용된 과거 맥락 공급','FR6']
],777,294,1091,360,[420,460,211],28);
txt('constraints-heading','이 기능을 구현할 때 지킬 경계',52,730,1790,60,39,C.navy,true);
txt('constraints','C1  로컬 VIA와 VAD, 별도 클라우드 음성/의미 모델\nC2  도메인 추론, 계획, 도구 선택과 실제 업무 실행은 Downstream Agent',52,830,1770,130,33);
note('다음 표는 네 설계 질문과 직접 관련된 기능 6개와 제약 2개입니다. 전체 VIA 기능 목록은 별도 기준 문서에 보존합니다.',52,1000,1800,48,26,C.muted);
notes('근거는 docs/architecture/03-fixed-architecture-scope.md, 05-representative-use-cases.md, 04-40 공통 계약과 03-02 여섯 품질 정의다. 요구 수집 인터뷰를 수행했다는 주장을 추가하지 않는다. 원래 기능/제약과 DP 직접 영향 매핑은 다음 원본 슬라이드에 그대로 유지된다.');

// 11. Difference between the two time metrics; lengths are explicitly conceptual.
base('time-boundaries','빠르게 반응하는 시간과 VIA가 요청에 쓰는 시간을 구별합니다','품질 요구사항',11);
note('예: “이 자료로 보고서를 만들어줘.” → 업무 인계 → 결과 수신 → 사용자에게 실제 전달',52,174,1790,56,33,C.teal);
const timeboxes=[['Interaction Manager',52,290,315],['Request Interpreter',481,290,340],['Agent Gateway',935,290,290],['Downstream Agent',1337,290,531]];
timeboxes.forEach(([name,x,y,w],i)=>box(name,x,y,w,74,i===3?C.light:C.blue,29));
[[367,481],[821,935],[1225,1337]].forEach(([a,b])=>poly('time-forward',[[a,327],[b,327]],C.teal,3,true));
note('입력 접수',52,381,330,52,28);note('VIA 해석',481,381,340,52,28);note('VIA 인계',935,381,290,52,28);note('외부 AI의 실제 작성',1340,381,510,52,28);
poly('time-return',[[1602,364],[1602,477],[187,477],[187,364]],C.muted,3,true);
note('결과를 받은 뒤 VIA가 준비하고 실제 전달',522,478,1020,60,32);
table('time-qa',[
 ['지표','사용자가 보는 의미','현재 목표와 계산 경계'],
 ['QA-02 상호작용 응답시간','새 입력, 질문이나 결과에 알맞게 반응하기까지','평균 ≤ 1초 / 로컬 재생 중단 ≤ 0.1초'],
 ['QA-03 VIA 요청 처리시간','접수부터 결과 전달까지 VIA가 처리하고 기다린 시간','평균 ≤ 10초 / 외부 업무 실행과 사용자 대기 제외']
],52,610,1816,221,[438,697,681],30);
txt('time-boundary-footnote','포함: VIA 의미 모델 호출, 자료 조회, queue 대기와 실제 전달\n제외: 외부 업무 AI가 보고서를 작성하는 시간, 사용자 대기와 계약상 필요한 사용자 발화',52,885,1790,98,30);
note('그림의 길이는 시간 비율이 아닙니다. 같은 21개 사례를 5회 독립 반복하여 지정된 반응과 처리 사이클을 따로 평가합니다.',52,1005,1790,48,26,C.muted);
notes('QA-02는 135관측, QA-03은 95사이클의 기존 정의를 유지한다. 10초 목표를 외부 보고서 작성 완료 보장으로 설명하지 않는다. 병렬 구간의 중복 합산을 피하는 기존 귀속 계약과 보호 조건은 appendix 원본 metric에 남아 있다.');

// 12. Causal links, not manufactured applicability/winner scores.
base('drivers','필요한 기능은 같아도, 진행과 상태를 조직하는 방식은 다릅니다','설계 도입',12);
table('drivers-map',[
 ['필요한 기능과 제약','비교할 품질 경로','구조 선택 질문'],
 ['FR2 / FR3  대상, 조건과 의존 관계','정확성 / VIA 처리시간 / 모델 비용','41  모델이 다음 조회를 정할까, 코드가 요청 항목을 해결할까?'],
 ['FR1 / FR3 / FR4  지속 입력과 후속 전달','정확성 / 상호작용 응답시간 / VIA 처리시간','44  중앙이 다음 실행을 지시할까, 단계 조건이 실행을 이끌까?'],
 ['FR6  허용된 과거 맥락 제공','정확성 / VIA 처리시간 / 모델 비용','45  요청 때 조합할까, 재사용할 관계를 공통 생산할까?'],
 ['FR4 / FR5  업무 연결과 연속성','정확성 / 변경 용이성 / 로컬 메모리','42  관련 상태를 함께 확정할까, 독립 접수로 맞출까?']
],52,212,1816,458,[485,510,821],30);
txt('drivers-common','공통 제약 C1/C2는 네 비교에서 모두 지킵니다.',52,729,1780,63,38,C.teal,true);
txt('drivers-qa','여섯 QA 모두 검토합니다. 위 표는 대표 인과 경로이며 적용성 전수표나 효과 크기가 아닙니다.\n같은 기능과 입력을 유지한 채, 품질 목표를 달성하는 비용과 지원 한계를 비교합니다.',52,840,1790,139,33);
notes('기존 FR/C → DP 직접 영향 표를 유지하며 앞의 필요성, 고정 QA와 실제 구조 질문을 연결한다. 상호작용 응답/처리/비용/메모리의 직접 경로를 숨기지 않는다. 42의 서비스 분리가 자동 정확성/변경 우위라는 주장으로 사용하지 않는다.');

// Presentation-only overlays: illustration/object sources are native clones.
overlay('overlap-overlay',14,'설계 도입');
note('시작 상태: 보고서와 메일을 지금 위임하며, 공급업체 견적은 그 전에 맡겨 아직 진행 중입니다.',95,294,1710,36,27,C.teal);
notes('원래 20장의 견적 도착에 필요한 선행 상태를 명시했다.');
const backgroundChoices={
 41:['다음 조회와 전체 의미 완성을 모델이 결정할까, 코드가 항목 해결 상태로 진행할까?',
     '모델 재판단의 유연성과 반복 비용 ↔ 유한 표현의 추적 가능성과 지원 범위/추가 해석 비용'],
 44:['완료마다 중앙이 다음 실행을 지시할까, 단계가 입력과 게시 조건으로 실행될까?',
     '중앙 진행 상태와 회수 대기 ↔ 단계별 상태와 Join 조건, 대기 및 수용량 관리 비용'],
 45:['필요할 때 과거 원본을 조합할까, 여러 요청이 읽을 관계를 공통 생산할까?',
     '유효 cache와 요청별 miss 비용 ↔ 공통 조회와 초기/미사용 생산, 철회 및 coverage 비용'],
 42:['대화와 업무의 관련 상태를 함께 확정할까, 독립 상태 권위의 접수로 맞출까?',
     '모듈형 Core의 공동 확정 ↔ 독립 운영 조건과 인계/재확인, process 및 접수 상태 비용']
};
for(const [dp,page]of [[41,15],[44,17],[45,19],[42,21]]){
 overlay(`dp${dp}-background-overlay`,page,`DP${dp} 배경`);
 rect('choice-white',54,940,1814,139,C.white,C.white,0);
 txt('choice-question',`설계 질문  ${backgroundChoices[dp][0]}`,74,948,1790,58,dp===42?30:31,C.teal,true);
 note(backgroundChoices[dp][1],74,1008,1770,52,28,C.ink);
 if(dp===42){
  rect('conversation-white',76,266,1770,34,C.white,C.white,0);
  note('Conversation(대화 기록과 업무 연결)은 음성 연결 종료 뒤에도 이어집니다. 아래의 수명 차이는 양안 공통입니다.',95,266,1760,34,25,C.teal);
 }
 notes(`원본 DP${dp} 배경의 그림과 사용자 장면을 보존한 사본에 선택 질문과 양쪽 비용을 추가했다. 기존 하단 설명 전체는 appendix 원본에 그대로 있다. ${dp===42?'다음 비교는 같은 보고서 업무에서 기간을 묻고 답하는 확대 장면이다. 음성 연결 종료는 Conversation 종료를 뜻하지 않는다.':''}`);
}

const qaNames=['QA-01  요청 처리 정확성','QA-02  상호작용 응답시간','QA-03  VIA 요청 처리시간','QA-04  VIA 모델 사용 비용','QA-05  VIA 변경 용이성','QA-06  로컬 VIA 메모리 사용량'];
const qaRows={
 41:[['평가 전 / 전체 의미와 부분 갱신','평가 전 / 표현 범위와 코드 결합'],['평가 전 / 조회 후 재판단 대기','평가 전 / 항목 해결과 추가 해석 대기'],['평가 전 / 조회와 전체 판단 반복','평가 전 / 항목 해결과 전체 결합'],['평가 전 / 재판단, retry와 폐기 청구','평가 전 / 부분 해석과 전체 실제 청구'],['평가 전 / 지침과 검증의 실제 수정','평가 전 / 요청 틀과 Engine 규칙 수정'],['예산 1,792MiB / 해석 맥락과 buffer','예산 1,760MiB / 틀과 항목 해결 상태']],
 44:[['평가 전 / 현재성 검사와 후속 지시','평가 전 / Window와 Join의 현재성'],['평가 전 / 중앙 queue와 회수 대기','평가 전 / Stage, Join과 수용량 대기'],['평가 전 / 회수와 다음 실행 연결','평가 전 / 단계 활성화와 게시 Join'],['평가 전 / 실제 호출, retry와 폐기','평가 전 / 실제 호출, retry와 폐기'],['평가 전 / 후속 규칙과 사건 계약','평가 전 / 단계와 Join 계약'],['예산 1,792MiB / 중앙 상태와 queue','예산 1,840MiB / Window, credit, copy']],
 45:[['평가 전 / 원문과 cache 조건 보존','평가 전 / coverage와 예외 보존'],['평가 전 / 최초 조합과 cache 대기','평가 전 / 조회와 미생산 보완 대기'],['평가 전 / 첫 사용, 반복과 갱신','평가 전 / 조회, 원문 검증과 backfill'],['평가 전 / cache miss와 추가 해석','평가 전 / 초기, 미사용 생산과 철회'],['평가 전 / 조합과 cache 철회 수정','평가 전 / 생산, 조회와 철회 수정'],['예산 1,792MiB / 원문과 선택 cache','예산 1,920MiB / 관계와 생산 상태']],
 42:[['평가 전 / 의미, 접수와 현재성 검사','평가 전 / 같은 검사와 receipt 반영'],['평가 전 / 공동 commit과 대화 반영','평가 전 / 독립 접수와 receipt 반영'],['평가 전 / 관련 상태 공동 확정','평가 전 / 인계와 재확인의 전체 경로'],['평가 전 / 저장 외 실제 모델 청구','평가 전 / IPC 외 실제 모델 청구'],['평가 전 / 공동 상태 계약의 수정','평가 전 / 독립 인계 계약의 수정'],['예산 1,792MiB / 통합 상태와 queue','예산 2,080MiB / process와 접수 상태']]
};
const titles={41:'요청 처리 정확성을 위한 맥락 기반 요청 해석 설계',44:'상호작용 응답시간을 위한 지속 대화 실행 설계',45:'VIA 요청 처리시간과 모델 사용 비용을 위한 과거 맥락 제공 설계',42:'요청 처리 정확성과 변경 용이성을 위한 대화와 업무 상태 관리 설계'};
function comparison(dp,page,example,a,b){
 base(`dp${dp}-comparison-v2`,`${dp}. ${titles[dp]}`,`DP${dp} 설계 비교`,page,dp===45||dp===42?41:45);
 note(example,52,153,1816,48,29);
 rect('option-a',52,210,881,50,C.blue,C.line,1);rect('option-b',969,210,899,50,C.peach,C.line,1);
 txt('option-a-text',a,65,210,850,50,31,C.navy,true,'center');txt('option-b-text',b,982,210,870,50,31,C.navy,true,'center');
}
function compareFooter(dp,aCost,bCost,common){
 note(common,52,665,1816,40,26,C.teal);
 txt('a-cost',aCost,52,708,879,46,25,C.muted);txt('b-cost',bCost,970,708,898,46,25,C.muted);
 table(`qa-${dp}`,[['고정 품질 / 같은 P21','A: 평가 경로와 현재 상태','B: 평가 경로와 현재 상태'],...qaNames.map((v,i)=>[v,...qaRows[dp][i]])],52,762,1816,266,[424,696,696],23);
 note('실측 없음. 메모리는 설계 예산이며 측정값이 아닙니다. 전체 목표/보호 조건과 지역 계산의 상세 근거는 appendix에 보존합니다.',52,1033,1816,35,22,C.muted);
}

comparison(41,16,'같은 입력: “이 표를 아까 보고서에 넣고, 보내지 말고 그 결과로 메일 초안만 만들어줘.”','A  모델이 조회와 전체 의미를 제안','B  모델의 요청 틀을 코드가 완성');
for(const x of [52,969]){
 box('클라우드 의미 LLM',x+22,275,323,47,C.light,26);box('Model Access',x+402,275,286,47,C.white,27);
 poly('model-call',[[x+402,289],[x+345,289]],C.muted,2,true);
 poly('model-result',[[x+345,310],[x+402,310]],C.muted,2,true);
 boundary('Request Interpreter',x+10,341,851,232);
}
box('모델 주도 해석 제어기',84,403,336,55,C.peach,29);box('읽기 도구 실행기',552,403,323,55,C.white,28);
box('모델에 제공할 해석 맥락',84,505,336,49,C.white,26,'can');box('의미 제안 검증기',552,505,323,49,C.white,28);
poly('a-read',[[420,430],[552,430]],C.teal,2.8,true);note('② 조회 선택',429,392,150,30,22);
poly('a-results',[[713,458],[713,482],[252,482],[252,505]],C.teal,2.8,true);note('③ 결과를 해석 맥락에 반영',320,464,390,34,23);
poly('a-rejudge',[[252,505],[64,505],[64,328],[546,328],[546,322]],C.teal,2.8,true);note('④ 누적 근거로 모델 재호출',70,577,407,40,24);
poly('a-action',[[546,322],[546,388],[252,388],[252,403]],C.teal,2.8,true);note('① 다음 조회 / 의미 / 질문',580,348,311,30,22);
poly('a-proposal',[[420,439],[459,439],[459,530],[552,530]],C.muted,2,true);note('완성 제안',449,543,118,30,22);
box('Context Manager',532,593,227,44,C.light,25);box('Task Manager',774,593,151,44,C.light,24);
poly('a-sources',[[875,430],[921,430],[921,583],[650,583],[650,593]],C.muted,2,true);
poly('a-task-source',[[850,583],[850,593]],C.muted,2,true);
box('요청 구조화기',999,403,298,55,C.peach,28);box('Request Resolution Engine',1390,403,423,55,C.peach,28);
box('항목별 해결 상태',999,505,298,49,C.white,27,'can');box('읽기 도구 실행기',1390,505,423,49,C.white,28);
poly('b-frame',[[1297,430],[1390,430]],C.teal,2.8,true);note('① 요청 틀',1289,373,165,31,23);
poly('b-code-read',[[1601,458],[1601,505]],C.teal,2.8,true);note('② 코드가 조회 선택',1492,463,310,32,23);
poly('b-results',[[1390,530],[1297,530]],C.teal,2.8,true);note('③ 반영',1296,547,120,30,22);
poly('b-recheck',[[1148,505],[1148,479],[1436,479],[1436,458]],C.teal,2.8,true);
poly('b-partial',[[1784,403],[1784,337],[1148,337],[1148,403]],C.muted,2,true,true);note('④ 필요 항목만 조건부 해석',1340,347,480,29,22);
poly('b-cloud',[[1148,403],[1148,331],[1508,331],[1508,322]],C.muted,2,true);
box('Context Manager',1411,593,230,44,C.light,25);box('Task Manager',1662,593,194,44,C.light,24);
poly('b-sources',[[1813,530],[1872,530],[1872,583],[1526,583],[1526,593]],C.muted,2,true);
poly('b-task-source',[[1759,583],[1759,593]],C.muted,2,true);
note('⑤ 코드가 지원 연산으로 전체 의미 구성',984,578,423,43,24);
compareFooter(41,'A도 구조화 출력, cache와 부분 갱신 가능','B는 유한 표현 한계와 추가 해석/확인 비용',
 '공통: 의미 제안 검증 → Request Controller의 최신성/권한 검사와 채택 → 직접 응답 또는 Task Manager / Agent Gateway 위임');
notes('원본 22장의 구조/흐름/schema는 appendix에 보존한다. 이 본문은 실제 다른 실행 책임과 반환 상태를 확대했다. A 모델이 다음 조회/전체 의미를 결정하고 B Engine 코드가 항목 상태로 조회/부분 해석/전체 결합을 결정한다. B는 한 번 호출 보장이 아니다. 전체 QA는 평가 전. A/B 메모리는 공통 계약 예산 1792/1760MiB이다.');

comparison(44,18,'같은 사건: 새 “표부터 설명해줘” + 메일 수신자 질문 + 이전 응답 준비의 늦은 완료','A  중앙이 완료를 회수하고 후속 지시','B  단계와 Join이 조건을 보고 실행');
for(const x of [52,969]){box('Interaction Manager',x+24,279,382,52,C.white,29);box('Task Manager',x+489,279,368,52,C.white,29);}
boundary('Request Controller',63,354,851,163);box('Dialogue Dispatcher',242,402,370,60,C.peach,32);box('Dialogue Progress State',654,407,232,53,C.white,22,'can');
box('Request Interpreter',85,566,340,48,C.white,29);boundary('Response Manager',545,533,360,108);box('Response Composer',556,578,336,45,C.white,28);
poly('a-progress-state',[[612,432],[654,432]],C.muted,2,true);
poly('a-input',[[267,331],[267,383],[427,383],[427,402]],C.teal,3,true);poly('a-task',[[725,331],[725,383],[427,383]],C.teal,3);
poly('a-infer',[[354,462],[354,505],[255,505],[255,566]],C.teal,2.6,true);note('① 지시',86,463,180,32,24);
poly('a-infer-return',[[425,590],[474,590],[474,462]],C.muted,2.6,true);note('② 완료 회수',348,510,221,32,24);
poly('a-compose',[[533,462],[533,501],[724,501],[724,578]],C.teal,2.6,true);note('③ 준비 지시',665,466,215,32,24);
poly('a-comp-return',[[556,601],[496,601],[496,462]],C.muted,2.6,true);
boundary('Request Controller',980,354,879,163);
box('Input Resolution Stage',992,399,405,51,C.peach,28);box('Task Notice Stage',1471,399,370,51,C.peach,29);
box('Input Window',992,466,230,45,C.white,25,'can');box('Notice Window',1611,466,230,45,C.white,25,'can');
poly('b-input',[[1184,331],[1184,399]],C.teal,3,true);poly('b-task',[[1658,331],[1658,399]],C.teal,3,true);
poly('b-input-state',[[1107,450],[1107,466]],C.muted,2,true);poly('b-notice-state',[[1726,450],[1726,466]],C.muted,2,true);
box('Request Interpreter',995,558,405,45,C.white,28);boundary('Response Manager',1453,526,408,117);
box('Response Composer',1470,566,373,32,C.white,25);box('Publication Join',1470,606,373,32,C.peach,26);
poly('b-meaning',[[1397,425],[1424,425],[1424,541],[1197,541],[1197,558]],C.teal,2.5,true);
poly('b-proposal-return',[[1400,581],[1434,581],[1434,385],[1364,385],[1364,399]],C.muted,2.5,true);
poly('b-adopted-compose',[[1397,443],[1445,443],[1445,581],[1470,581]],C.teal,2.5,true);
note('제안 반환 후 채택',995,532,190,22,20);
poly('b-notice-compose',[[1841,430],[1886,430],[1886,581],[1843,581]],C.teal,2.5,true);
note('유효 Notice',1700,361,154,25,20);
poly('b-candidate',[[1656,598],[1656,606]],C.teal,2.5,true);
poly('b-join-input',[[1107,511],[1107,527],[984,527],[984,622],[1470,622]],C.teal,2.5,true);
poly('b-join-notice',[[1726,511],[1872,511],[1872,622],[1843,622]],C.teal,2.5,true);
note('입력/알림 조건 + 준비된 후보 → 게시 가능 여부',995,642,850,23,21);
compareFooter(44,'A도 비동기, 병렬 실행과 우선순위 처리 가능','B는 Stage/Window, credit와 Join 상태 비용',
 '공통: Publication Control의 검사 → Interaction Manager 실제 표시/재생 → Response Manager 전달 기록. 옛 후보 폐기 / 질문은 별도 허용');
notes('B의 Request Interpreter 출력이 바로 게시되는 것이 아니다. 의미 제안은 Input Resolution Stage/Request Controller에서 채택하고, 채택 의미 또는 Task Notice Stage의 유효 Notice를 입력으로 Response Composer가 후보를 생산한다. 이후 입력/알림/전달 조건과 Join한다. 중앙 완료 회수와 단계별 진행이 핵심 대비다. 실제 표시/재생 원장을 생성 완료와 구별한다. 지역 0.060/0.050초는 appendix 계산 가정이며 전체 QA-02 실측 차이가 아니다.');

comparison(45,20,'같은 요청: “지난번 평가 기준으로 제품 비교와 견적 결과를 함께 써서 제안서를 만들어줘.”','A  현재 요청에 맞춰 원본을 조합','B  과거 관계를 공통 생산하고 조회');
note('원본: 실제 전달한 설명과 업무 결과',52,268,671,46,27,C.teal);
box('Model Access',750,276,430,45,C.white,27);box('클라우드 의미 LLM',1410,276,439,45,C.light,27);
poly('common-meaning',[[1180,298],[1410,298]],C.muted,2,true);
note('필요할 때 호출',1203,266,195,27,21);
boundary('Context Manager',63,326,851,234);box('Context Composer',113,390,410,59,C.peach,33);box('Source View Cache',606,389,274,58,C.white,25,'can');
poly('a-query',[[75,375],[318,375],[318,390]],C.teal,2.5,true);note('① 이번 조회',348,335,210,30,23);
poly('a-cache',[[523,419],[606,419]],C.muted,2.5,true);note('② 유효 cache / 부분 갱신',578,451,318,34,23);
box('Request Evidence Set',493,493,385,56,C.white,28,'can');
poly('a-meaning-read',[[523,408],[550,408],[550,298],[750,298]],C.muted,2.5,true,true);note('③ 원본 확인 후 이번 요청의 근거 반환',94,584,807,40,25);
poly('a-evidence',[[523,432],[545,432],[545,477],[685,477],[685,493]],C.teal,2.5,true);
poly('b-semantic-call',[[1010,415],[964,415],[964,321]],C.muted,2.5,true,true);
boundary('Context Manager',980,326,879,312);box('Memory Publisher',1010,390,365,55,C.peach,30);box('Coverage State',1458,391,367,55,C.white,28,'can');
box('Episodic Memory Repository',1197,476,533,55,C.white,28,'can');box('Evidence Reader',1197,577,533,45,C.peach,28);
poly('b-produce',[[1193,445],[1193,457],[1463,457],[1463,476]],C.teal,2.5,true);note('② 관계 게시',1020,454,180,32,23);
poly('b-coverage',[[1375,417],[1458,417]],C.muted,2.5,true);
poly('b-sourcechange',[[987,376],[1193,376],[1193,390]],C.teal,2.5,true);note('① 원본 변경 / 생산 요청',1270,335,572,32,23);
poly('b-query',[[990,599],[1197,599]],C.teal,2.5,true);note('③ 별도 현재 조회',988,534,280,34,23);
poly('b-read',[[1463,577],[1463,531]],C.teal,2.5,true);note('원본 확인 후 반환',1542,533,295,37,24);
note('미생산은 게시 후 재조회 / 지원 밖은 한계 반환',988,641,850,23,22);
compareFooter(45,'A도 유효 관계 cache와 부분 갱신 가능','B는 초기/미사용 생산, coverage와 철회 비용',
 '공통: 원본 owner의 개정/권한/실제 전달 범위를 확인 → 현재 Request Interpreter에 근거 제공. PARTIAL은 전체 의미 완성을 뜻하지 않음');
notes('A current Query 수명의 구성 책임과 B 공통 관계 생산/Repository/Coverage 유지 및 독립 현재 Reader가 다르다. 원본 owner는 그대로다. 예외 문구와 결과 참조 일부만 생산하는 PARTIAL 예시이며 현재 요청의 정답은 저장하지 않는다. Model Access는 공통 cloud adapter로 B Publisher의 필요한 의미 생산에도 사용된다. 지역 과거 처리 비용은 전체 P21 청구가 아니다.');

comparison(42,22,'같은 보고서 업무의 확대 장면: 기간 질문을 실제 제시 → “상반기로 해줘” → 접수와 외부 확인','A  모듈형 Core에서 관련 상태 공동 확정','B  독립 대화/업무 서비스의 별도 접수');
note('Conversation / Request / Task / Execution의 서로 다른 논리 수명은 양안 공통입니다.',52,270,1816,42,29,C.teal);
boundary('VIA Core 실행 경계',63,324,851,311);box('Request Controller',93,389,340,56,C.white,29);box('Task Manager',551,389,334,56,C.white,29);
box('Request Graph State',94,473,337,50,C.white,27,'can');box('Task Command State',551,473,334,50,C.white,27,'can');
boundary('State Store',280,536,415,97);box('트랜잭션 관리자',293,581,388,41,C.peach,29);
poly('a-req-state',[[263,445],[263,473]],C.muted,2,false,true);poly('a-task-state',[[718,445],[718,473]],C.muted,2,false,true);
poly('a-changes-1',[[433,428],[452,428],[452,533],[274,533],[274,601],[293,601]],C.teal,2.8,true);
poly('a-changes-2',[[885,428],[902,428],[902,601],[681,601]],C.teal,2.8,true);
note('검증된 변경',718,543,172,29,23);note('공동 commit',89,558,168,28,22);
poly('a-commit-1',[[293,613],[263,613],[263,523]],C.muted,2.5,true);poly('a-commit-2',[[681,613],[718,613],[718,523]],C.muted,2.5,true);
note('① 관계와 답변 명령을 같은 commit으로 확정',94,356,790,27,24);note('성공: 모두 반영 / 충돌과 실패: 모두 미반영',100,642,800,23,22);
boundary('대화 서비스 실행 경계',980,324,396,311);boundary('Task 서비스 실행 경계',1454,324,405,311);
box('Request Controller',996,389,363,56,C.white,29);box('Task Manager',1470,389,373,56,C.white,29);
box('Request Graph State',997,485,361,55,C.white,27,'can');box('Task Command State',1471,485,371,55,C.white,27,'can');
poly('b-command',[[1359,408],[1470,408]],C.teal,2.8,true);note('② 같은 명령',1299,351,248,33,23);
poly('b-receipt',[[1470,449],[1359,449]],C.teal,2.8,true);note('④ 접수 결과',1358,456,151,30,22);
poly('b-dialogue-commit',[[1177,445],[1177,485]],C.muted,2.5,true);poly('b-task-commit',[[1656,445],[1656,485]],C.muted,2.5,true);
note('① 로컬 의도 저장\n④ 접수 결과 별도 반영',996,552,363,63,25);note('③ 내부 접수와 상태 확정\n외부 전송 가능 상태 저장',1470,552,373,63,25);
compareFooter(42,'A도 논리 수명 분리와 변경 국소화 가능','B의 독립 운영은 조건부. 인계/재확인 비용',
 '공통: 확정된 명령 → Agent Gateway → Downstream Agent의 실제 확인. 내부 접수는 외부 실행/확인 완료가 아님. 의미 모델은 transaction 밖');
notes('DP42 title의 정확성은 semantic model 우위가 아니라 대화/질문/Task 관계와 receipt 반영의 정확성이다. A strong modular local Core retains separate logical lifetimes. B has distinct local state authorities and execution lifetimes; command/receipt and reconciliation are actual costs. No automatic semantic/modifiability win. Local commit 0.010/0.025 seconds in appendix is an assumption, not whole QA03 measurement. ACK 전 DISPATCHING은 UNKNOWN이며 업무 gate 적용 ACK와 hold 요청 시각을 혼동하지 않는다.');

// 23. Verify scope; do not pretend comparison candidates are a selected architecture.
base('coverage-summary','네 설계 질문을 같은 품질 기준과 공통 경계에서 검토합니다','검토 요약',23);
table('coverage-map',[
 ['설계 질문','직접 다루는 기능','확인할 실제 구조 차이'],
 ['41  요청 해석 제어','FR2 / FR3','다음 조회와 전체 의미의 결정 주체, 표현 범위와 추가 해석'],
 ['44  지속 대화 실행','FR1 / FR3 / FR4','완료 회수와 후속 지시, 단계별 상태와 게시 조건'],
 ['45  과거 맥락 제공','FR6','요청별 구성과 공통 관계 생산의 시점, coverage와 철회'],
 ['42  대화와 업무 상태','FR4 / FR5','공동 확정과 독립 접수, 외부 확인과 재연결']
],52,206,1816,441,[385,390,1041],31);
txt('scope-qa','정확성 → 상호작용 응답시간 → VIA 처리시간 → 모델 비용 → 변경 용이성 → 로컬 메모리',52,716,1816,57,31,C.teal,true);
txt('scope-common','로컬 입력과 VAD / 같은 허용 근거 / 최신성 및 권한 검사 / 실제 전달 기록\n별도 클라우드 음성/의미 모델 / 외부 업무 AI의 실제 실행과 확인',52,829,1816,116,32);
note('현재는 네 A/B의 구조와 평가 계약을 설명하는 단계입니다. 채택된 통합 구조나 실측 승자를 표시하지 않습니다.',52,992,1816,50,29,C.muted);
notes('기존 요구 매핑을 재확인한다. 여섯 QA 우선순위는 원본 그대로이며 새 기준을 추가하지 않는다. C1/C2와 공통 실행 계약을 유지한다. 이 요약을 최종 Architecture 선택으로 읽지 않는다.');

base('next-validation','조건과 비용을 확인한 뒤, 동일한 활동에서 대안을 판단합니다','검토 요약',24);
table('next-steps',[
 ['검토할 대상','양쪽에서 확인할 조건','아직 필요한 근거'],
 ['요청 해석','모호한 대상, 부분 정정과 표현 밖 관계','조건 누락, 반복 조회와 확인 질문의 실제 비용'],
 ['지속 대화','새 발화, 늦은 준비와 질문이 겹치는 상황','실제 반응/전달 시각과 대기 상태'],
 ['과거 맥락','첫 사용, 반복, 정정 직후와 미사용 생산','전체 호출 청구, 원본 검증과 coverage'],
 ['상태 관리','공동 변경, 내부 접수와 외부 확인의 차이','실제 수정 책임, 접수 대기와 메모리']
],52,223,1816,466,[321,761,734],31);
txt('next-contract','같은 입력과 외부 조건, 여섯 QA의 목표와 보호 조건을 유지합니다.',52,754,1816,57,37,C.teal,true);
txt('next-status','현재 숫자는 목표, 설계 예산과 지역 가정 계산입니다.\nA/B 선택과 품질 결과는 아직 없습니다. 구현/측정 착수는 별도 승인 단계입니다.',52,867,1816,116,33);
notes('참고 발표자료의 구현/평가/채택 결론을 VIA의 사실로 복사하지 않는다. 현재 단계에 맞는 검토 종료와 향후 확인 사항을 설명한다. 향후 실행을 자동 승인하는 문서가 아니다.');

base('appendix-divider','Appendix','상세 근거와 원본 보존',25,64);
txt('appendix-intro','발표 중 필요한 상세 정의와 설계 계약을 확인할 수 있습니다.',52,260,1816,80,45,C.teal,true);
txt('appendix-order','26~35  원본 8~17장: P21, 21개 사례 대응과 QA-01~06 계산/등급\n36~57  나머지 원본: 소개, 전체 지도, DP 구조와 QA 상세',52,422,1816,172,36);
txt('appendix-preserved','원본 32장의 편집 가능한 개체, 표와 발표자 노트를 모두 보존했습니다.\n원본 파일은 변경하지 않았고, 아래 페이지는 원래 문구와 도형을 그대로 유지합니다.',52,745,1816,153,35);
note('부록의 화면에 남아 있는 옛 번호/제목은 원본 자료의 표기입니다. v2의 실제 위치는 README와 manifest에서 확인할 수 있습니다.',52,996,1816,46,27,C.muted);
notes('기존 내용을 삭제하지 말라는 사용자 지시를 위해 원본 32장을 전부 byte-identical로 보존했다. 원본 8~17은 appendix 첫 상세 구역에 배치한다. 새 본문으로 대체된 원본 비교 22/25/28/31도 여기에 남는다.');

function clone(page,key,section,originalPage,overlayKey){records.push({page,key,section,kind:'clone',original_page:originalPage,...(overlayKey?{overlay:authored[overlayKey]}:{})});}
function newSlide(page,key,section){records.push({page,key,section,kind:'authored',authored_page:authored[key]});}
clone(1,'background-v2','배경과 소개',1);newSlide(2,'necessity-v2','배경과 소개');clone(3,'role-v2','배경과 소개',2);newSlide(4,'domain-primer','배경과 소개');
clone(5,'scenario-v2','배경과 소개',4);clone(6,'responsibility-v2','배경과 소개',19);
newSlide(7,'requirements-rationale','요구사항과 QA');clone(8,'requirements-v2','요구사항과 QA',5);clone(9,'quality-01-03-v2','요구사항과 QA',6);clone(10,'quality-04-06-v2','요구사항과 QA',7);newSlide(11,'time-boundaries','요구사항과 QA');
newSlide(12,'drivers','설계 도입');clone(13,'overview-v2','설계 도입',18);clone(14,'overlap-v2','설계 도입',20,'overlap-overlay');
for(const [dp,bg,page] of [[41,21,15],[44,24,17],[45,27,19],[42,30,21]]){
 clone(page,`dp${dp}-background-v2`,`DP${dp}`,bg,`dp${dp}-background-overlay`);newSlide(page+1,`dp${dp}-comparison-v2`,`DP${dp}`);
}
newSlide(23,'coverage-summary','검토 요약');newSlide(24,'next-validation','검토 요약');newSlide(25,'appendix-divider','Appendix 안내');
for(const originalPage of [...Array.from({length:10},(_,i)=>i+8),...Array.from({length:32},(_,i)=>i+1).filter(i=>i<8||i>17)]){
 records.push({page:records.length+1,key:`original-${String(originalPage).padStart(2,'0')}`,section:originalPage>=8&&originalPage<=17?'Appendix / QA 상세':'Appendix / 원본 보존',kind:'original',original_page:originalPage});
}
for(const row of records.filter(r=>r.kind==='clone'&&!r.overlay)){overlay(`header-${row.key}`,row.page,row.section);row.overlay=authored[`header-${row.key}`];}
if(records.length!==57||p.slides.items.length!==25)throw new Error('Composition mismatch');
const draft=path.join(build,'authored.pptx');await(await PresentationFile.exportPptx(p)).save(draft);
const planPath=path.join(build,'v2-plan.json');await fs.writeFile(planPath,JSON.stringify({version:2,source_path:path.relative(repo,source),source_sha256:sourceHash,main_slides:24,slides:records},null,2));
const candidate=path.join(build,'candidate.pptx'),manifestPath=path.join(build,'manifest-v2.json');
execFileSync(python,[path.join(repo,'scripts/presentations/package_presentation_v2.py'),source,draft,planPath,candidate,manifestPath],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(manifestPath,'utf8'));
const owners=manifest.slides.filter(row=>row.native_tables>0).map(row=>row.page);
const final=path.join(build,'final','VIA-architecture-presentation_v2.pptx');await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:57,
 pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
 layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(i=>['--require-native-table-slide',String(i)])],
 requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'reference',families:['Apple SD Gothic Neo','Arial Unicode MS'],referencePath:source,referenceSha256:sourceHash},
 verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
manifest.output_sha256=createHash('sha256').update(await fs.readFile(final)).digest('hex');await fs.writeFile(manifestPath,JSON.stringify(manifest,null,2)+'\n');
const rendered=await PresentationFile.importPptx(await FileBlob.load(final));
await fs.writeFile(path.join(build,'inspection.ndjson'),(await rendered.inspect({kind:'slide,layout,table,image',maxChars:200000})).ndjson);
await fs.mkdir(path.join(build,'renders'),{recursive:true});
for(const [i,slide] of rendered.slides.items.entries()){
 const png=await rendered.export({slide,format:'png',scale:1});await fs.writeFile(path.join(build,'renders',`${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await png.arrayBuffer()));
 console.log(`Rendered ${i+1}/57`);
}
console.log(JSON.stringify({final,manifestPath,sourceUnchanged:sourceHash===createHash('sha256').update(await fs.readFile(source)).digest('hex')}));
