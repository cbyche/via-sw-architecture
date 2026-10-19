// User-requested DP3 explanation and parallel-baseline revision, 2026-10-19.
// Uses the existing reference deck's editable objects and preserves DP1/2/4.
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [ROOT,BUILD]=process.argv.slice(2);
if(![ROOT,BUILD].every(p=>p&&path.isAbsolute(p)))throw Error('Pass absolute repository and build paths');
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1015.12040/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const FONT='Calibri',W=1920,H=1080;
const C={black:'#111111',gray:'#98999B',grid:'#B7B9B9',green:'#C9E1B3',greenD:'#70A843',cream:'#FFF6DD',blue:'#4676CB',cyan:'#35A8CD',pale:'#F1F6FC',red:'#C00000',muted:'#50555B',white:'#FFFFFF'};
await fs.mkdir(BUILD,{recursive:true});
await fs.mkdir(path.join(BUILD,'previews'),{recursive:true});
const original=await fs.readFile(path.join(ROOT,'scripts/presentations/generate_dp3_reference_style.mjs'),'utf8');
let helpers=original.slice(original.indexOf('const line='),original.indexOf('// 1. Editable event schematic'));
helpers=helpers.slice(0,helpers.indexOf('function notes('));
helpers+=`function notes(extra=''){return ${JSON.stringify('2026-10-19 사용자 정정 반영. 화면 생산과 발화 생산은 서로의 최초 결과를 소비하지 않는다. 현재 설계 기본 B는 공통 Model Access 호출 한도 K가 허용하면 독립 생산을 병렬 실행하며 둘 다 준비된 뒤 결합한다. K=1 또는 실제 자원 경쟁은 공통 제약으로 공개하며 A/B에 같은 한도를 적용한다. 이전 B4 표기는 과거 pixel 캠페인의 병렬 구성 식별자로 보존한다. 기존 정규화 B는 순차 실행 이력으로만 사용한다. 정규화 병렬 기본 B의 새 실행은 하지 않았으며 최종 A/B 선택을 이 판본에서 확정하지 않는다. 모듈 분리는 별도 모델 가중치나 프로세스 배치를 의미하지 않는다. 현재 input/source/후보 revision 검증, unknown/경쟁 후보와 원본 보존, 원 생산자의 범위 확장 및 공통 최종 채택은 양쪽 기능 계약을 유지한다. 기본 A의 전체 집합 재판단과 영향 표현 patch 보완, 기본 B의 판단 안 후보 재사용과 revision 사이 cache 보완을 구별한다. 기존 수치와 실패는 재채점하거나 병렬 실행값으로 전환하지 않는다.\n출처: 원래 16장 v2-windows PPTX, seven-problem-review/04-screen-speech-grounding.md 및 17-presentation-results.md의 기존 발표 캠페인, dp04/normalized-comparison.md와 comparison.md, normalized_candidate/src/dp04.rs와 dp04/producers.rs. 새 common9 등 다른 캠페인의 수치와 결론은 합치지 않는다.\n') }+extra;}
`;
let background=original.slice(original.indexOf('// 1. Editable event schematic'),original.indexOf('// 2 and 3 share'));
background=background.replace("' (배경)'","' (배경 1/2)'");
background=background.replace("VIA는 발화, 당시 화면, 포인터와 선택 범위를 연결해 실제 대상을 정해야 한다.","앱을 오가며 말하고, 클릭 없이 가리키고, 일부만 고친 요청의 정확한 대상을 찾아야 한다.");
background=background.replace("'② 그래프 두 개 지정'","'② 클릭 없이 두 그래프 지정'");
const lowerStart=background.indexOf(" txt(s,'정정은 현재 메일");
const lowerEnd=background.indexOf(' s.speakerNotes',lowerStart);
background=background.slice(0,lowerStart)+`
 txt(s,'“이 매출”은 파일 전체인가,\\n선택한 셀 범위인가?',85,774,490,72,28,{bold:true,align:'center'});
 txt(s,'포인터의 원은 지칭 동작인가,\\n단순한 마우스 이동인가?',665,774,490,72,28,{bold:true,align:'center'});
 txt(s,'“오른쪽 것”은 현재 메일인가,\\n앞선 PDF의 그래프인가?',1245,774,490,72,28,{bold:true,align:'center'});
 poly(s,[[65,869],[1830,869]],C.blue,1.5);
 txt(s,'VIA가 정리할 요청',85,890,340,70,30,{bold:true,color:C.blue});
 txt(s,'“매출 현황.xlsx의 선택 범위와\\n실적 분석.pdf의 오른쪽 이익 그래프를 비교해줘.”',440,883,1325,82,32,{bold:true});
 txt(s,'Excel 범위는 유지하고, PDF 그래프만 두 개에서 오른쪽 하나로 정정한다.',440,981,1325,37,25.333,{color:C.muted});
`+background.slice(lowerEnd);
const why=String.raw`
{
 const s=pres.slides.add();header(s,' (배경 2/2)',2);
 txt(s,'VIA가 지칭을 잘못 연결하면, Agent가 정확히 일해도 요청은 실패한다',66,145,1785,62,40,{bold:true});
 txt(s,'사용자의 말과 행동을 이어받는 VIA가, 실제 업무를 맡길 대상을 명확하게 만들어야 한다.',68,214,1780,42,28);
 const values=[['VIA의 연속 상호작용','VIA가 해결해야 할 어려움'],
 ['말하면서 앱·페이지를 계속 이동','현재 화면 대신 보존한 당시 화면·행동으로 판단'],
 ['한 문장에서 여러 앱의 자료를 지칭','현재 활성 창을 넘어 자료별 범위와 출처를 연결'],
 ['클릭 없이 가리키거나 둘러쌈','정식 선택 객체가 없어도 지칭 동작·범위를 판단'],
 ['이전 요청의 일부만 정정','유지할 대상과 교체할 대상, 최신 판본을 구분'],
 ['다른 Agent에게 실제 업무 위임','지칭한 자료와 조건을 명확한 요청으로 전달']];
 const t=nativeTable(s,values,85,286,[620,1130],[50,67,67,67,67,67],28);highlightHeader(t,2);
 txt(s,'보존할 근거와, 그 근거에서 의미를 만드는 책임을 함께 설계해야 한다',85,721,1720,46,34,{bold:true,color:C.blue});
 box(s,'A/B 공통 기능\n당시 화면·선택·포인터·발화 보존\n현재 요청과 원본 판본 확인',85,789,520,134,{size:26.667,bold:true});
 box(s,'A의 책임 구조\n원본을 함께 읽고\n대상 범위와 연결 관계를 공동 생산',665,789,520,134,{fill:C.pale,stroke:C.blue,size:26.667,bold:true});
 box(s,'B의 책임 구조\n독립 후보를 병렬 생산한 뒤 결합\n부족한 범위는 원 생산자가 확장',1245,789,520,134,{fill:C.pale,stroke:C.blue,size:26.667,bold:true});
 txt(s,'선택할 것: 중간 상태·실행 의존·재판단 책임  /  달라지는 것: 응답 대기·정확도·비용·메모리',85,954,1740,48,28,{bold:true});
 txt(s,'두 안은 동일한 공통 호출 한도 K를 사용한다. B는 여유가 있으면 독립 생산을 겹치고, 결합은 양쪽 완료를 기다린다.',85,1011,1740,33,24,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('이 장면은 VIA에만 존재하는 문제라는 주장이 아니다. 연속 음성·화면 상호작용과 Agent 위임을 제품 책임으로 가진 VIA에서 결합되어 중요한 문제라는 설명이다. 입력 기록과 시간 대응만으로 지칭 의도를 확정하지 않는다. 근거 부족과 실제 모호함은 보존하고 필요한 확인을 수행한다.'));
}
`;
let comparison=original.slice(original.indexOf('function comparison(selected)'),original.indexOf('comparison(false);comparison(true);'));
comparison=comparison.replace("selected?3:2","3");
comparison=comparison.replace("'A는 원본에서 공동 생산 / B는 생산자별 후보를 게시한 뒤 결합'","'A는 원본에서 공동 판단 / B는 화면·발화 후보를 독립적으로 병렬 생산한 뒤 결합'");
comparison=comparison.replace("'(B) 발화·화면 후보 생산 후 결합'","'(B) 독립 후보 병렬 생산 후 결합'");
comparison=comparison.replace("'유효 후보를 같은 판단 안에서 반복 결합\\n화면·발화의 해석 근거와 오류 지점을 분리'","'화면·발화 생산 대기를 겹침 / 후보를 같은 판단 안에서 재사용\\n화면·발화의 해석 근거와 오류 지점을 분리'");
comparison=comparison.replace("'생산·게시·확장과 결합 의존 및 상태 추가\\n후보의 범위 오류가 최종 결합을 막을 수 있음'","'양쪽 후보 완료 후 결합 / 생산·확장·후보 상태 비용\\n공통 호출 한도와 자원 경쟁에 따라 병렬 이익 제한'");
for(const value of ['18.774 (12/15)','18/24 (75.0%)','25.033 (9/12)','0.0650212','41.894'])comparison=comparison.replace("'"+value+"'","'미측정'");
comparison=comparison.replaceAll("미측정 (이전 판본 2.50)","미측정");
comparison=comparison.replace("'원본별 생산자와 후보 저장, 후속 결합을 분리'","'독립 생산을 병렬 시작 / 양쪽 준비 후 결합'");
comparison=comparison.replace("'역할 queue\\n예산·완료\\n확장 요청'","'동일 호출 한도 K\\n독립 병렬 시작\\n부족한 원본 확장'");
comparison=comparison.replace("1092,432,159,96,23","1092,430,148,99,20");
comparison=comparison.replace("n(s,1,1260,354);n(s,2,1251,438);n(s,3,1251,507);","n(s,1,1260,354);n(s,1,1528,354);n(s,2,1251,438);n(s,2,1528,438);n(s,3,1251,507);");
comparison=comparison.replace("'공통 계약: bbox 0~1000 출력 후 pixel 변환. B 기본은 순차 생산, 충분한 후보는 코드 결합 가능. PoC는 Qwen vision 생산.'","'共通契約PLACEHOLDER'").replace('共通契約PLACEHOLDER','공통 K가 허용하면 B는 독립 생산 병렬 실행. 충분한 후보는 코드 결합. 모호하거나 부족하면 의미 판단·원 생산자 확장.');
comparison=comparison.replace("'정규화 24회: 안별 4사례×3 / 시간 괄호: 관측·예정 / 비용: 12회 합계 / RSS: 로컬 실행기 / B의 미완료 3회 유지'","'A열은 기존 정규화 관측 참고. 새 병렬 기본 B와의 재비교 미실행. 이전 순차 B 결과는 다음 장에 이력으로 보존.'");
comparison=comparison.replace("28,1016,1840,27,20","28,1034,1840,23,18.667").replace("28,1047,1805,25,19","28,1058,1805,19,16");
comparison+='comparison(false);\n';
const evidence=String.raw`
{
 const s=pres.slides.add();header(s,' (기존 관측과 비교 범위)',4);
 txt(s,'병렬 기본 B로 설계를 정정한다. 기존 순차 실측으로 새 A/B 승자를 정하지 않는다.',45,139,1830,61,36,{bold:true,color:C.blue});
 txt(s,'이전 정규화 비교 — 당시 실행한 A와 순차 B의 이력',65,226,1790,46,32,{bold:true});
 const a=[['구성','첫 응답 초 (관측/15)','지칭 집합 (정답/24)','VIA 처리 초 (완료/12)','호출 / 비용 USD'],
 ['A 공동 판단','7.716 (15/15)','24/24','9.646 (12/12)','15 / 0.0254572'],
 ['이전 B 순차 생산','18.774 (12/15)','18/24','25.033 (9/12)','42 / 0.0650212']];
 const t=nativeTable(s,a,65,286,[350,380,350,390,355],[50,76,76],26.667);highlightHeader(t,5);
 txt(s,'B의 밀집 화면 3회 미완료 유지. 정답 대상 ID를 제안했어도 공간 근거가 지시점을 포함하지 못했다.',65,506,1790,40,25.333,{color:C.red});
 txt(s,'별도 pixel 비교 — 이전 B4는 독립 병렬 생산을 실제 실행한 구성',65,574,1790,46,32,{bold:true});
 const b=[['구성','첫 응답 초 (관측/15)','지칭 집합 (정답/24)','VIA 처리 초 (완료/12)','호출 / 비용 USD'],
 ['이전 B 순차','18.922 (14/15)','22/24','22.622 (11/12)','38 / 0.0446804'],
 ['이전 B4 병렬','13.881 (13/15)','20/24','17.604 (11/12)','40 / 0.0459164']];
 const u=nativeTable(s,b,65,635,[350,380,350,390,355],[50,76,76],26.667);highlightHeader(u,5);
 txt(s,'병렬은 독립 대기를 겹친다. 두 구성 모두 미완료가 있어 위 성공 평균으로 전체 속도 우위를 확정하지 않는다.',65,851,1790,41,25.333,{color:C.muted});
 txt(s,'현재 판단',65,929,250,49,32,{bold:true,color:C.blue});
 txt(s,'A의 기존 관측은 보존. 정규화 병렬 기본 B와의 최종 선택은 재비교 전까지 미확정.',320,929,1500,49,30.667,{bold:true});
 txt(s,'B4는 과거 실행 식별자이며 새 기본 B의 수치로 소급하지 않는다. 병렬화만으로 정확도·호출 수·비용·RSS가 같다고 가정하지 않는다.',65,1008,1790,36,24,{color:C.muted});
 s.speakerNotes.textFrame.setText(notes('사용자 정정으로 독립 병렬 생산을 기본 설계에 포함했다. 과거 결과의 수치, 분모, 실패와 당시 조건부 A 판단은 역사로 보존한다. 새 병렬 기본 B와의 우열은 미확정이다. B4 pixel 수치는 공통 정규화 조건과 다르므로 합치지 않는다. K=1은 병렬 이익 제한 조건이며 K>=2에서의 기본 독립 실행을 대체하는 설계 선택 이유가 아니다. 기존 48회 A/B/A2+A5/B4 중 여기서는 B/B4 각각12episode의 값만 표시했다. 정규화 각12episode와 분모를 혼합하지 않는다.'));
}
`;
let recovery=original.slice(original.indexOf('// 4. Only same-contract'),original.indexOf("await fs.mkdir(path.join(BUILD,'previews')"));
recovery=recovery.replace("' (보완 효과 상세)',4","' (별도 복구 관측)',5");
recovery=recovery.replace('현재 선택은 정규화 기본 A. 아래는 별도 pixel 실험의 A와 A2+A5 전후 비교다.','별도 pixel 실험의 A와 A2+A5 비교. 정규화 병렬 기본 B와의 재비교 결과는 아니다.');
recovery=recovery.replace('QA02  19/24 (79.2%) → 24/24 (100%)','QA02 지칭 집합  19/24 → 24/24');
recovery=recovery.replace('4장 수치는','13쪽 수치는');
const p=Presentation.create({slideSize:{width:W,height:H}});
const modulePath=path.join(BUILD,'dp3-content.mjs');
await fs.writeFile(modulePath,`export function build(pres,common){const {ROOT,FONT,W,H,C}=common;\n${helpers}\n${background}\n${why}\n${comparison}\n${evidence}\n${recovery}\n}`);
(await import(pathToFileURL(modulePath).href)).build(p,{ROOT,FONT,W,H,C});
if(p.slides.items.length!==5)throw Error('Expected five DP3 slides');
const authored=path.join(BUILD,'dp3-authored.pptx');
await(await PresentationFile.exportPptx(p)).save(authored);
execFileSync(PY,[path.join(ROOT,'scripts/presentations/package_dp3_parallel_v3.py'),ROOT,BUILD],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')).href);
const final=path.join(BUILD,'output','VIA-DP1-DP4-reference-style-17slides-v3-windows.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await finalizePresentation({workspaceDir:BUILD,candidatePath:path.join(BUILD,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:17,
 pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit'],
 fontPolicy:{basis:'reference',families:['Calibri'],referencePath:path.join(ROOT,'docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP4-reference-style-16slides-v2-windows.pptx'),referenceSha256:createHash('sha256').update(await fs.readFile(path.join(ROOT,'docs/presentations_files/dp1-dp4-reference-style-20261019/VIA-DP1-DP4-reference-style-16slides-v2-windows.pptx'))).digest('hex')},
 verifyArtifactToolImport:true,receiptPath:path.join(BUILD,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
for(let i=0;i<17;i++){
 const slide=imported.slides.items[i];
 const blob=await imported.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.layout.json`),await(await slide.export({format:'layout'})).text());
 console.log('Rendered '+(i+1)+'/17');
}
console.log(final);
