// Editable DP3 CPU architecture slides, inserted into the exact v5-r2 deck.
import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [ROOT,BUILD]=process.argv.slice(2);
if(!ROOT||!BUILD||![ROOT,BUILD].every(path.isAbsolute))throw Error('Pass absolute repo/build paths');
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1017.22135/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const FONT='Calibri',W=1920,H=1080;
const C={black:'#111111',gray:'#98999B',grid:'#B7B9B9',green:'#C9E1B3',greenD:'#70A843',cream:'#FFF6DD',blue:'#4676CB',cyan:'#35A8CD',pale:'#F1F6FC',red:'#C00000',muted:'#50555B',white:'#FFFFFF'};
const source=path.join(BUILD,'source-v5-r2.pptx');
const sourceDeck=await PresentationFile.importPptx(await FileBlob.load(source));
const sourceInspection=await sourceDeck.inspect({select:'$.slides[8,10,18]',include:['id','index','layout','title','elements.id','elements.kind','elements.text','elements.position','elements.style']});
await fs.writeFile(path.join(BUILD,'source-inspection.ndjson'),sourceInspection.ndjson);
await fs.writeFile(path.join(BUILD,'source-masters.json'),JSON.stringify({masters:sourceDeck.masters.items.map(x=>({id:x.id,name:x.name})),layouts:sourceDeck.layouts.items.map(x=>({id:x.id,name:x.name,placeholders:x.placeholders.summary()}))},null,2));
const original=await fs.readFile(path.join(ROOT,'scripts/presentations/generate_dp3_reference_style.mjs'),'utf8');
const helpers=original.slice(original.indexOf('const line='),original.indexOf('function notes('));
const helperPath=path.join(BUILD,'dp3-helpers.mjs');
await fs.writeFile(helperPath,`export function helpers(ctx){const {FONT,W,H,C}=ctx;\n${helpers}\nreturn {shape,txt,poly,box,storeShape,n,nativeTable,highlightHeader};}`);
const {shape,txt,poly,box,storeShape,n,nativeTable,highlightHeader}=(await import(pathToFileURL(helperPath))).helpers({FONT,W,H,C});
const pres=Presentation.create({slideSize:{width:W,height:H}});
const BASE='docs/architecture/12-decisions/decision-packages/seven-problem-review/dp3-grounding-architecture-20261021/';
const noteBase=`DP3-GROUNDING-CPU-SYN-1.0 / VOICE-SCREEN-CPU-SYN-1.0 / SYNTHETIC_AUTHORED_NOT_MEASURED. 모든 새 수치는 작성한 가정과 오프라인 산식이며 모델 실행·측정·실제 청구 결과가 아니다. 평가 120개 음성 입력·지칭 표현·새 목표 + 별도 준비 활동 60개. 준비는 비용과 동시 RAM 원장에 포함. COMMON-QA-SYN-1.0 Text 모집단과 직접 비교하거나 합산하지 않는다. 동일 ASR·word timestamp, 원본 화면·포인터·선택, cloud S2S 및 최종 의미 LLM. 8 CPU core/16 GB RAM/SSD; visual 총4 thread; 기본 heavy visual stage 순차. 512 MB 임시한계는 이 비교에서 유보.\n출처: ${ROOT}/${BASE}cpu-runtime-architecture.md\n${ROOT}/${BASE}local-producer.md\n${ROOT}/${BASE}synthetic-comparison/README.md\n${ROOT}/${BASE}synthetic-comparison/results.md\n${ROOT}/${BASE}synthetic-comparison/summary.json\n`;
function header(s,suffix,page){
 s.background.fill=C.white;
 txt(s,'DP3. 화면 지칭 해석과 로컬 생산기'+suffix,60,29,1140,68,44,{bold:true});
 ['과제 소개','요구사항 분석','설계','구현 및 검증','결론'].forEach((v,i)=>{const x=1210+i*134;shape(s,'chevron',x,33,143,44,i===2?C.cyan:C.gray,C.white,2);txt(s,v,x+14,33,111,44,19,{bold:true,color:C.white,align:'center'});});
 poly(s,[[58,108],[1860,108]],'#A7A7A7',3);
 txt(s,String(page),1835,1049,38,20,17,{align:'right',color:C.muted,name:'Continuous slide number'});
}
function lead(s,a,b){txt(s,a,65,137,1780,58,38.667,{bold:true});if(b)txt(s,b,65,204,1780,43,26.667);}
function notes(s,t){s.speakerNotes.textFrame.setText(noteBase+t);}
function arrow(s,x,y,yy,color=C.black){poly(s,[[x,y],[x,yy]],color,2,false,true);}
function columns(s,a,b){box(s,a,65,262,870,57,{fill:C.green,size:30.667,bold:true,stroke:C.grid});box(s,b,985,262,870,57,{fill:C.green,size:30.667,bold:true,stroke:C.grid});poly(s,[[960,262],[960,978]],C.grid,1.2);}
function foot(s,t){txt(s,t,65,1007,1740,31,21.333,{color:C.muted});}
function table(s,rows,x,y,widths,heights,size=26.667){const t=nativeTable(s,rows,x,y,widths,heights,size);highlightHeader(t,widths.length);return t;}

// 9. Same captured evidence, but identity is not proved by the timestamp.
{
 const s=pres.slides.add();header(s,' (배경)',9);
 lead(s,'전사와 시각이 정확해도, 두 번 가리킨 대상이 같은지는 남는다','보존한 두 시점의 화면과 동작을 연결해, 실제 대상과 요청 조건을 함께 확정해야 한다.');
 const xs=[85,665,1245];
 ['① t₀: 첫 견적을 가리킴','② t₁: 배치가 바뀐 뒤 다시 가리킴','③ 발화 종료: 이미 다른 앱'].forEach((v,i)=>txt(s,v,xs[i],280,520,43,28,{bold:true}));
 xs.forEach((x,i)=>{shape(s,'rect',x,340,490,290,C.white,C.black,2);shape(s,'rect',x,340,490,40,'#E9EBED',C.black,1);txt(s,i===2?'메일':'견적 목록 — 화면 관측',x+16,341,460,38,24,{bold:true});});
 box(s,'견적 L',120,410,180,120,{fill:C.pale,stroke:C.blue,size:32,bold:true});box(s,'견적 Z',340,410,180,120,{fill:C.white,stroke:C.grid,size:32});
 box(s,'견적 Z',700,410,180,120,{fill:C.white,stroke:C.grid,size:32});box(s,'견적 L',920,410,180,120,{fill:C.pale,stroke:C.blue,size:32,bold:true});
 poly(s,[[210,590],[210,535]],C.red,3,false,true);poly(s,[[1010,590],[1010,535]],C.red,3,false,true);
 txt(s,'포인터 t₀',151,587,240,35,24,{color:C.red,bold:true});txt(s,'포인터 t₁',951,587,240,35,24,{color:C.red,bold:true});
 txt(s,'받은 편지함\n\n새 화면에는 견적이 없음',1280,404,420,155,29.333);
 poly(s,[[582,485],[642,485]],C.black,2.2,false,true);poly(s,[[1162,485],[1222,485]],C.black,2.2,false,true);
 box(s,'“아까와 지금 가리킨 견적들을 중복 없이 묶고, 설치비는 제외해서 요약표로 만들어줘.”',85,674,1650,82,{fill:C.cream,size:30.667,bold:true});
 txt(s,'공통 입력',85,797,250,45,32,{bold:true,color:C.blue});txt(s,'정확한 ASR + 단어 시각 / 당시 frame·pointer·selection / 원본·판본·접근 권한',350,798,1410,43,28);
 txt(s,'해결할 판단',85,870,250,45,32,{bold:true,color:C.blue});txt(s,'시각은 관측을 찾는다. 동일 대상 여부·중복 제거·설치비 제외는 의미 판단이 맡는다.',350,871,1410,43,28,{bold:true});
 foot(s,'S3 설명용 도식. L/Z는 정답 식별용 표기이며 실제 입력에 고유 이름·native ID·단순 viewport 변환은 제공하지 않는 가정.');
 notes(s,'S3는 단순히 과거 화면을 조회하면 끝나는 사례가 아니다. 두 시점에 지칭한 것이 이동한 같은 견적인지 판단하여 중복 제거해야 한다. A도 두 원본과 포인터를 읽고 추가 crop을 요청할 수 있다. B는 UI seed와 SAM의 시각 대응을 사용하지만 그 track도 파생 근거이며 정답 보장은 없다. 실제 혼동과 근거 부족은 보존하고 확인한다.');
}
// 10. Pair one: representation and on-demand lifecycle.
{
 const s=pres.slides.add();header(s,' (방안 비교 1/2)',10);
 lead(s,'A는 원본을 함께 해석하고, B-R1은 화면 후보를 먼저 생산한다','공통: local ASR·단어 timestamp / 당시 화면·동작 보존 / cloud S2S와 최종 의미 LLM');
 columns(s,'A  원본 공동 해석','B-R1  로컬 후보 생산 + 모델 교대 로딩');
 box(s,'보존 원본\n화면·포인터·선택 + 전사·단어 시각',145,350,700,86,{fill:C.cream,size:28});
 arrow(s,495,436,480);n(s,1,505,447);
 box(s,'Grounding Coordinator\n현재 요청에 필요한 원본 범위·판본 구성',145,480,700,85,{size:28});
 arrow(s,495,565,610);n(s,2,505,576);
 box(s,'Cloud 의미 LLM\n대상 범위와 지칭 관계를 함께 해석',145,610,700,104,{fill:C.pale,stroke:C.blue,size:30.667,bold:true});
 poly(s,[[145,662],[100,662],[100,520],[145,520]],C.blue,2,true,true);txt(s,'추가 원본 조회 가능',180,731,620,34,25.333,{color:C.blue,align:'center'});
 arrow(s,495,765,804);
 box(s,'현재성·권한·근거 확인 → 최종 채택',145,804,700,61,{fill:C.white,size:26.667});
 txt(s,'별도 화면 ML runtime 없음\n원본 crop·추가 읽기·확정 이력 재사용 가능',110,894,780,76,28,{bold:true,align:'center'});
 box(s,'당시 화면·포인터·선택 원본',1065,350,700,60,{fill:C.cream,size:28});
 arrow(s,1415,410,446);n(s,1,1425,417);
 shape(s,'rect',1065,446,700,156,C.cream,C.black,2.2);txt(s,'Local CPU worker 1개 — runtime 유지',1078,454,674,38,28,{bold:true,align:'center'});
 box(s,'UI 로딩·추론\n검출 + OCR',1090,512,230,66,{fill:C.white,size:24});
 box(s,'회수·전환',1345,512,150,66,{fill:C.white,size:24});
 box(s,'SAM 로딩·추론\n필요할 때만',1520,512,220,66,{fill:C.white,size:24});
 poly(s,[[1320,545],[1345,545]],C.black,2,false,true);poly(s,[[1495,545],[1520,545]],C.black,2,true,true);
 arrow(s,1415,602,640);n(s,2,1425,609);
 storeShape(s,'후보 bbox·mask·track + 출처·판본·불확실성\n원본 crop과 episode 상태 보존',1065,640,700,88,{size:25.333});
 arrow(s,1415,728,765);
 box(s,'Cloud 의미 LLM\n같은 전사 + 후보 + 원본 crop으로 지칭 결합',1065,765,700,85,{fill:C.pale,stroke:C.blue,size:28,bold:true});
 txt(s,'검증·채택 / 후보가 부족하면 생산기에 확장 요청',1060,866,720,34,24,{align:'center'});
 txt(s,'모델 메모리 회수 ↔ 다음 요청의 로딩·전환 대기',1020,925,800,39,28,{bold:true,align:'center'});
 foot(s,'B의 후보는 파생 해석이며 독립 원본이 아니다. native 선택이면 생산 생략, bbox로 충분하면 SAM 생략.');
 notes(s,'A를 한 번의 LLM 호출로 제한하지 않는다. A의 원본 조회·crop·native ID·확정 상태 재사용을 허용한다. B는 최종 LLM에 원음을 다시 보내지 않는다. local UI는 OmniParser detector+EasyOCR이며 Florence captioner는 이 구성에서 제외한다. R1은 요청마다 runtime을 종료하지 않으며 모델 session만 교대한다. 모델 unload와 episode 삭제는 별개다.');
}
// 11. Pair two: residency versus independent native process failure boundary.
{
 const s=pres.slides.add();header(s,' (방안 비교 2/2)',11);
 lead(s,'B-R2는 한 worker에 상주하고, B-R3는 worker를 분리한다','모델·후보·최종 LLM은 동일하다. 상주·프로세스 분리·병렬 실행·근거 보존은 서로 다른 선택이다.');
 columns(s,'B-R2  한 worker에 사용한 모델 유지','B-R3  UI / SAM worker 별도 상주');
 for(const x of [65,985]){
  box(s,'Grounding Coordinator / 같은 CPU 총량·실행 허가',x+70,350,730,62,{size:26.667});
  if(x===65){arrow(s,x+435,412,456);}else{poly(s,[[1420,412],[1420,432],[1225,432],[1225,456]],C.black,2,false,true);poly(s,[[1420,432],[1615,432],[1615,456]],C.black,2,false,true);}n(s,1,x+445,422);
 }
 shape(s,'rect',135,456,730,205,C.cream,C.black,2.2);txt(s,'CPU worker 1 / Python·PyTorch runtime',147,465,706,43,28,{bold:true,align:'center'});
 box(s,'UI session\n검출·OCR weights',165,529,300,79,{fill:C.white,size:26.667});box(s,'SAM session\nTiny weights',535,529,300,79,{fill:C.white,size:26.667});
 txt(s,'한 번 사용한 모델 유지 / heavy stage 기본 순차',155,619,690,33,25.333,{align:'center'});
 for(const [x,title,body] of [[1055,'UI worker','별도 runtime\n검출·OCR weights'],[1445,'SAM worker','별도 runtime\nTiny weights']]){
  shape(s,'rect',x,456,340,205,C.cream,C.black,2.2);txt(s,title,x+10,465,320,43,28,{bold:true,align:'center'});box(s,body,x+20,529,300,107,{fill:C.white,size:26.667});
 }
 txt(s,'UI seed 확인 후, 필요 시',1455,638,320,20,18.667,{color:C.blue,align:'center'});
 for(const x of [65,985]){
  if(x===65){arrow(s,x+435,661,701);}else{poly(s,[[1225,661],[1225,680],[1420,680],[1420,701]],C.black,2,false,true);poly(s,[[1615,661],[1615,680],[1420,680]],C.black,2);}n(s,2,x+445,670);
  storeShape(s,'동일 후보·episode 상태 + 원본 crop',x+70,701,730,62,{size:26.667});
  arrow(s,x+435,763,798);
  box(s,'Cloud 의미 LLM → 공통 검증·채택',x+70,798,730,63,{fill:C.pale,stroke:C.blue,size:28,bold:true});
 }
 txt(s,'재로딩 대기 감소 / runtime private 메모리 공유\nworker 장애로 두 모델을 함께 재시작',110,894,780,82,28,{bold:true,align:'center'});
 txt(s,'SAM 장애 시 UI worker·상태 보존\nruntime 복제·IPC 증가 / 자동 병렬 이익 없음',1030,894,780,82,28,{bold:true,align:'center'});
 foot(s,'주 비교: 8 CPU core 중 visual 총 4 thread, 한 번에 heavy visual stage 하나. R3에도 같은 제한을 적용한다.');
 notes(s,'R2/R3의 모델 상주는 최초 사용 이후다. cold에서는 로딩이 발생한다. R1/R2/R3는 같은 episode 보존 정책이다. R3의 독립 복구는 native crash/강제 종료에도 다른 worker를 유지하는 process boundary의 차이다. 정상 시나리오 QA04를 복구성 점수로 바꾸지 않는다. R2 역시 명시적 admission/메모리 예산을 갖추면 동시 추론이 가능하므로 병렬성은 R3만의 고유 이익이 아니다. 현재 비교는 동일 total CPU budget과 순차 heavy stage이며 IPC 0.06초 가정만 반영한다.');
}
// 12. Runtime deployment and simultaneous memory, not checkpoint summation.
{
 const s=pres.slides.add();header(s,' (실행 구조·메모리)',12);
 lead(s,'모델 파일은 실행 프로그램이 아니다. worker가 로딩해 CPU 추론한다','Rust VIA → named pipe·공유 버퍼 → 묶음 Python/PyTorch CPU worker → 후보·원본 참조 반환');
 box(s,'VIA 호스트\n요청·보존·허가',85,282,360,93,{size:29.333,bold:true});
 box(s,'worker 실행 프로그램\nPython + CPU runtime',565,282,560,93,{size:29.333,bold:true});
 box(s,'모델 session\nload → infer → retain/release',1245,282,570,93,{size:28,bold:true});
 poly(s,[[445,329],[565,329]],C.black,2.2,false,true);poly(s,[[1125,329],[1245,329]],C.black,2.2,false,true);
 txt(s,'UI: OmniParser icon_detect_v3 + EasyOCR ko/en  /  SAM: SAM2.1 Hiera Tiny 38.9M  /  GPU 사용 없음',85,403,1730,44,28,{bold:true});
 const rows=[['동시에 남는 메모리 · MB','A','B-R1','B-R2','B-R3'],['공통 VIA·ASR·원본·요청 상태','560','560','560','560'],['runtime · shared + private','—','340','340','620'],['UI / SAM 상주 weights·객체','—','170','590','590'],['활성 workspace + episode','—','860','860','860'],['cloud 입력 또는 IPC','32','24','24','32'],['QA06 동시 peak 합계','592','1,954','2,374','2,662']];
 const t=table(s,rows,85,482,[730,240,240,240,280],[54,55,55,55,55,55,64],26.667);
 t.cells.block({row:6,column:0,rowCount:1,columnCount:5}).assign({fill:C.pale,textStyle:{typeface:FONT,fontSize:28,bold:true,color:C.blue}});
 txt(s,'R1 → R2',85,911,230,42,30.667,{bold:true,color:C.blue});txt(s,'UI 모델 객체 420 MB 유지로 로딩 절약. R3는 runtime private 280 MB와 IPC 8 MB 추가.',340,913,1450,40,28);
 foot(s,'가정 RAM 원장, 실측 아님. 이 peak는 SAM 실행 시점이며 UI workspace를 중복 합산하지 않음. 512 MB 한계는 유보.');
 notes(s,'UI detector TorchScript 파일 281 MB(MIT), EasyOCR 별도, SAM2.1 Tiny checkpoint 약156 MB/38.9M. 디스크 파일 크기와 RAM을 동일시하지 않는다. 합성 모델 객체 UI420/SAM170 MB. runtime shared60+private280; R3 shared60 한 번+private280 두 번. 활성 SAMworkspace680+episode180. UIworkspace260과 임시 로딩380/156은 다른 시점 원장에 포함한다. R1 SAM peak에서는 UI session 회수 완료. allocator가 반환하지 않으면 R1 이익이 줄어든다. S2/S4는 SAM을 생략한다. worker executable은 번들 Python/의존 라이브러리를 시작하는 배포 단위이며 model weights만으로 실행하지 않는다.\n모델 참고: https://huggingface.co/microsoft/OmniParser-v2.0 / https://github.com/microsoft/OmniParser / https://github.com/facebookresearch/sam2 / https://github.com/JaidedAI/EasyOCR\n정확한 사용 artifact와 라이선스는 cpu-runtime-architecture.md의 inventory를 따르며 이 슬라이드는 상용 배포 승인 결론이 아니다.');
}
// 13. Same scenario families and one fully decomposed request.
{
 const s=pres.slides.add();header(s,' (동일 시나리오)',13);
 lead(s,'상주는 반복 사용에서 유리하다. 첫 화면과 긴 유휴 뒤에는 다시 로딩한다','가정 기반 합성 비교: 6유형 × 작성 변형 20개 = 120입력. 시간은 발화 종료 → 첫 구체적 표시.');
 const rows=[['동일 시나리오','B의 화면 처리','A','B-R1','B-R2','B-R3'],['S1  native 선택 두 견적','모델 생략','2.300','2.300','2.300','2.300'],['S2  첫 밀집 화면의 두 카드','cold · UI만','3.800','6.933','6.933','6.993'],['S3  두 시점 같은 견적 중복 제거','UI+SAM · R2/3 warm','5.500','10.700','8.800','8.860'],['S4  이어서 새 영역 확인','UI만 · R2/3 warm','3.400','5.667','4.567','4.627'],['S5  같은 장면 후보·확정 이력','모델 생략·재사용','2.833','2.633','2.633','2.633'],['S6  긴 유휴 후 새 화면','cold · UI만','2.967','6.467','6.467','6.527']];
 table(s,rows,65,272,[650,350,200,200,200,200],[55,60,60,60,60,60,60],26.667);
 txt(s,'S3 임계 경로 · 초',65,729,480,45,30.667,{bold:true,color:C.blue});
 txt(s,'A',65,791,130,37,28,{bold:true});txt(s,'공통 1.10 + 원본 해석·추가 읽기 4.40 = 5.50',210,791,1590,37,28);
 txt(s,'B-R2',65,842,130,37,28,{bold:true});txt(s,'공통 1.10 + UI 1.60 + SAM 두 frame 4.10 + 결합 2.00 = 8.80',210,842,1590,37,28);
 txt(s,'R1 / R3',65,893,150,37,28,{bold:true});txt(s,'R1: 위 경로에 로딩·전환 1.90 추가  /  R3: IPC 0.06 추가',240,893,1550,37,28);
 foot(s,'S3~5의 사전 준비 60활동은 비용·RAM에 포함. ASR은 발화 중 수행하고 종료 후 잔여만 계산. QA03은 외부 Agent 업무를 제외.');
 notes(s,'S3 준비는 현재 clip이 아니라 이전 별도 두 시점 견적 중복 제거 요청이다. 그래서 이미 UI/SAM 사용 이력이 있지만 현재 선택 정답은 준비 때 알 수 없다. S4/S5 준비는 UI만 필요하다. R1도 runtime/episode는 유지하지만 weights를 회수한다. S2 bbox로 처리 가능한 두 카드는 SAM을 쓰지 않는다. S5 A도 확정 내용 재사용. QA01 공통1.10=cloud S2S잔여.30+gateLLM.65+원본읽기.08+첫표시.07. QA03은 첫응답 뒤 공통2.5166667초를 이 합성 요청에서만 더한다:dispatch.15+resultlink.10+summaryLLM1.8666667+audio preparation.35+UI.05. 외부 Agent업무8초는 제외한다. 일반적으로 모든 DP의 QA03=QA01+상수라는 정의는 아니다. 작성변형20개는 통계적 모델반복시험20회가 아니다.');
}
// 14. Seven consistent QA meanings; fictional raw values remain explicit.
{
 const s=pres.slides.add();header(s,' (QA01~07 비교)',14);
 lead(s,'A의 응답·메모리 이익과 B의 정확도·cloud 비용 이익을 맞바꾼다','가정 기반 합성 수치 · 실측 아님 / 평가 120입력 + 준비 60활동 / QA07 우선순위는 미확정');
 const rows=[['QA / 동일 지표','A','B-R1','B-R2','B-R3','구조에서 예상한 차이'],['QA01 첫 응답 · 초 ↓','3.467','5.783','5.283','5.323','B는 로컬 생산을 기다림'],['QA02 지칭 정확도 ↑','89.17%\n107/120','95.83%\n115/120','95.83%\n115/120','95.83%\n115/120','B의 범위·대응 후보가 오류 감소'],['QA03 VIA 처리 · 초 ↓','5.983','8.300','7.800','7.840','생산 대기가 목표 완료에도 포함'],['QA04 예상 Rust 모듈 ↓','1.83','2.00','2.00','2.00','후보 의존 무효화 1모듈 가정'],['QA05 VIA 비용 · USD ↓','3.9052','3.4188','3.4188','3.4188','후보 + crop으로 cloud 입력 감소'],['QA06 동시 RAM · MB ↓','592','1,954','2,374','2,662','weights·workspace·runtime 차이'],['QA07 요청 의미 정확도 ↑','85.00%\n102/120','91.67%\n110/120','91.67%\n110/120','91.67%\n110/120','같은 대상 오류 개선이 다시 반영']];
 table(s,rows,65,272,[390,205,205,205,205,590],[56,64,84,64,64,64,64,84],25.333);
 txt(s,'QA01',65,865,160,35,25.333,{bold:true,color:C.blue});txt(s,'첫 구체적 채택 내용 표시. 일반 접수 안내 제외 / 오답의 첫 표시시간도 유지.',240,865,1540,35,25.333);
 txt(s,'QA04~06',65,910,160,35,25.333,{bold:true,color:C.blue});txt(s,'실제 Rust diff·청구액·RAM 측정이 아님. QA04 공통 guard면 동률 가능.',240,910,1540,35,25.333);
 foot(s,'B 세 안은 같은 모델·근거이므로 정확도 동일. QA02·07의 +6.67%p는 중복 효과이며 두 번 합산하지 않는다.');
 notes(s,'QA01/03/02/07의 분모가120인 것은 이번 authored fixture가 입력당 새목표와지칭표현하나이기때문이지 일반계약이같다는뜻아님. QA04 C01..06 A=[1,1,1,3,2,3],B=[1,1,1,3,2,4],실제Rustdiff미실행. C06구간근거철회시candidate_dependencies 추가변경 가정. Python수/프로세스수를모듈수로계산하지않음. QA05 synthetic단가 textin$1/M,textout$4/M,imageequivalent$1/M,audioin$.001/sec,audioout$.002/sec. A평가2.60368+준비1.30152;B평가2.24928+준비1.16952. Agent .324각각별도. imageequivalent는provider실제tokenizer아님. A대상오류13,B5:공통4,A만9,B만1. 대상오류외의미오류5개공통. QA02/07차이는같은대상오류8개감소이다.');
}
// 15. Decision follows the prior priority order, with alternative conditions.
{
 const s=pres.slides.add();header(s,' (선택 및 조건)',15);
 lead(s,'현재 CPU 가정과 QA01 우선순위에서는 A를 선택한다','QA01 → QA02 → QA03 → QA04 → QA05 → QA06 적용 / 합성 가정하의 권고이며 실측 승자 아님');
 const rows=[['선택안','선택할 이유','감수할 비용 / 선택을 바꿀 조건'],['A  현재 권고','B-R2보다 첫 응답 1.82초 단축\nRAM 1,782 MB 감소','지칭 정확도 6.67%p 낮음\nVIA cloud 비용 $0.4864 증가'],['B-R1  메모리 우선','B의 후보 정확도와 cloud 비용 유지\nR2보다 RAM 420 MB 절약','R2보다 첫 응답 평균 0.50초 지연\n실제 allocator 메모리 회수 확인 필요'],['B-R2  B 계열 기본 후보','반복 사용에서 재로딩 절약\nR3보다 RAM 288 MB 절약','정확도를 응답보다 우선할 때 검토\n최초 cold·유휴 후 로딩은 남음'],['B-R3  독립 복구 요구','SAM native crash에도\nUI worker와 상태 유지','R2보다 평균 0.04초·288 MB 증가\n정상 QA 집계 밖 복구 요구가 있어야 함']];
 const t=table(s,rows,65,276,[430,680,690],[56,126,126,126,126],28);
 t.cells.block({row:1,column:0,rowCount:1,columnCount:3}).assign({fill:C.pale});
 txt(s,'선택을 뒤집는 관측',65,883,420,44,30.667,{bold:true,color:C.blue});
 txt(s,'A의 원본 해석이 가정보다 평균 1.82초 더 걸리거나, 정확도 우선 조건을 합의할 때 B-R2 재검토',65,937,1780,43,28,{bold:true});
 foot(s,'모델 상주와 근거 재사용은 별개. 병렬 실행은 R2에도 가능한 별도 tactic이므로 R3의 자동 이익으로 세지 않는다.');
 notes(s,'현재 추천은 가정한 CPU latency와 기존합의QA01우선순위로A를선택한다는분석결론이며acceptedADR나제품채택아님. QA07새우선순위미부여. A보다B-R2정확도6.67pp/비용12.4552%이익은산식가정. R1대R2 .50sec/420MB,R3대R2 .04sec/288MB. QA04작은조건부차이는주선택근거아님. 공통guard면동률. A에추가지연1.8166667초보다더큰차이가실제로생기면R2시간역전. S5재사용비중만높이고나머지비율고정시91.7355%초과에서R2시간역전하지만실제빈도예측아님. B정확도우선은결과를본후A탈락용임계값조작이아니라사전합의필요. R3독립복구는현재7개정상QA외 별도필수요구로검토한다.');
}
// 24. Old five slides follow unchanged; historical measurements stay historical.
{
 const s=pres.slides.add();header(s,' (Appendix)',24);
 txt(s,'Appendix — 이전 DP3 설계와 관측',85,280,1730,84,56,{bold:true});
 poly(s,[[85,402],[1815,402]],C.blue,3);
 txt(s,'이전 발표의 DP3 5장을 보존',85,460,1730,57,38.667,{bold:true,color:C.blue});
 txt(s,'배경 2장 / 기존 A·B 비교 / 기존 관측 범위 / 별도 복구 관측',85,545,1730,57,32);
 txt(s,'아래 자료는 당시 cloud 생산자·pixel 계약의 설계와 관측이다.\n새 본문의 local CPU 네 대안과 모집단·실행 구조가 다르다.',85,663,1730,112,34.667,{bold:true});
 txt(s,'현행 DP3의 수치와 결론에 혼합하지 않는다.',85,850,1730,66,38.667,{color:C.red,bold:true});
 notes(s,'사용자 요청에 따라 원본21장중DP3 9~13쪽의본문/노트/의존자료를보존하여Appendix25~29쪽으로이동했다. 페이지번호만현행29장에맞춘다. 이전노트의옛페이지참조는역사적문맥으로유지한다. 이구분장은이전관측을무효화하는것이아니라현행새대안과섞지않기위한출처범위설명이다.');
}

await fs.writeFile(path.join(BUILD,'new-slides-content.ndjson'),(await pres.inspect({select:'$.slides[*]',include:['index','elements.text','notes.text']})).ndjson);
await(await PresentationFile.exportPptx(pres)).save(path.join(BUILD,'dp3-cpu-authored.pptx'));
console.log('Authored '+pres.slides.items.length+' editable slides');
execFileSync(PY,[path.join(ROOT,'scripts/presentations/package_dp3_cpu_v6.py'),ROOT,BUILD],{stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')));
const final=path.join(BUILD,'output','VIA-DP1-DP5-reference-style-29slides-v6-dp3-cpu-windows.pptx');
await finalizePresentation({workspaceDir:BUILD,candidatePath:path.join(BUILD,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:29,
 pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),
 layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...[12,13,14,15].flatMap(n=>['--require-native-table-slide',String(n)])],requiredNativeTableOwnerSlides:[12,13,14,15],
 fontPolicy:{basis:'reference',families:['Calibri'],referencePath:source,referenceSha256:createHash('sha256').update(await fs.readFile(source)).digest('hex')},verifyArtifactToolImport:true,receiptPath:path.join(BUILD,'validation.json')});
const imported=await PresentationFile.importPptx(await FileBlob.load(final));
for(let i=0;i<imported.slides.items.length;i++){
 const slide=imported.slides.items[i];
 const blob=await imported.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await blob.arrayBuffer()));
 await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.layout.json`),await(await slide.export({format:'layout'})).text());
 console.log('Rendered '+(i+1)+'/29');
}
console.log(final);
