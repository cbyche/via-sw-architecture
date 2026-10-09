import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Run a copy in a private build directory linked to the bundled node_modules.
// The existing draw.io/SVG scenes remain the content and geometry source.
const [repoArg,buildArg,scope,subset] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_dp_editable_pptx.mjs REPO BUILD_DIR');
if (scope && !['--comparison-only','--dp41-only','--dp41-with-background','--dp41-42-only','--dp44-only','--dp45-only','--intro-flow-only','--list-selected'].includes(scope)) throw new Error('Optional scope: --comparison-only, --dp41-only, --dp41-with-background, --dp41-42-only, --dp44-only, --dp45-only, --intro-flow-only or --list-selected');
if (subset && !(['--dp45-only','--list-selected'].includes(scope) && subset==='--comparison-only')) throw new Error('Subset requires --dp45-only (or --list-selected) --comparison-only');
const repo=await fs.realpath(repoArg), build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const python=process.env.VIA_RUNTIME_PYTHON;
const modules=process.env.RUNTIME_NODE_MODULES;
if (![skill,python,modules].every(value=>path.isAbsolute(value??''))) {
  throw new Error('Set absolute VIA_PRESENTATION_SKILL_DIR, VIA_RUNTIME_PYTHON and RUNTIME_NODE_MODULES paths.');
}
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const font='Apple SD Gothic Neo';
const extraction=String.raw`
import sys,json,hashlib,re
from pathlib import Path
root=Path(sys.argv[1]);sys.path.insert(0,str(root/'scripts/architecture'))
import generate_dp_background_slides as bg
import generate_dp_comparison_slides as cmp
scenes=[]
# The introduction shares the existing background scene, not a new source.
o=bg.overview();source='docs/architecture/12-decisions/decision-packages/diagrams/'+o.slug+'.svg'
assert (root/source).read_text()==o.svg(),source+' is out of sync'
scenes.append(dict(number='overview',kind='overview',slug=o.slug,title=o.caption,items=o.items,source=source,notes=o.notes,sha256=hashlib.sha256((root/source).read_bytes()).hexdigest()))
# Python 3.12's compensated sum changes last-bit annotation widths only.
def normalized_svg(value):
    return re.sub(r'-?\d+\.\d+',lambda m:format(float(m[0]),'.8f').rstrip('0').rstrip('.'),value)
for n in range(41,46):
    for kind in ['background','comparison']:
        s=getattr(bg,'slide'+str(n))() if kind=='background' else cmp.Comparison(n)
        if kind=='comparison':
            for side,x in enumerate([140,1010]):getattr(cmp,'graph'+str(n))(s,x,side)
        prefix='docs/architecture/12-decisions/decision-packages/diagrams' if kind=='background' else 'docs/presentations_files/dp-comparison'
        source=prefix+'/'+s.slug+'.svg'
        assert normalized_svg((root/source).read_text())==normalized_svg(s.svg()),source+' is out of sync'
        scenes.append(dict(number=n,kind=kind,slug=s.slug,title=s.caption,items=s.items,source=source,transition=bg.TRANSITIONS.get(n),sha256=hashlib.sha256((root/source).read_bytes()).hexdigest()))
print(json.dumps(scenes,ensure_ascii=False))
`;
const scenes=JSON.parse(execFileSync(python,['-c',extraction,repo],{encoding:'utf8',maxBuffer:8*1024*1024}));
await fs.writeFile(path.join(build,'source-scenes.json'),JSON.stringify(scenes,null,2));

const stroke=(color,width=1.7,dashed=false)=>({fill:color==='none'?'none':color,width:color==='none'?0:width,style:dashed?'dashed':'solid'});
function polyline(slide,name,pts,color,width,dashed=false) {
  const xs=pts.map(p=>p[0]),ys=pts.map(p=>p[1]);
  const left=Math.min(...xs),top=Math.min(...ys),w=Math.max(.1,Math.max(...xs)-left),h=Math.max(.1,Math.max(...ys)-top);
  return slide.shapes.add({name,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',line:stroke(color,width,dashed),
    customPaths:[{width:w,height:h,commands:pts.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
}
function arrowhead(slide,name,pts,color,width) {
  const [ex,ey]=pts.at(-1);
  let previous=pts.length-2;
  while(previous>=0 && pts[previous][0]===ex && pts[previous][1]===ey) previous--;
  if(previous<0)return;
  const [px,py]=pts[previous],length=Math.hypot(ex-px,ey-py),u=(ex-px)/length,v=(ey-py)/length;
  const back=width*6.4,side=width*3.2;
  polyline(slide,name,[[ex-u*back-v*side,ey-v*back+u*side],[ex,ey],[ex-u*back+v*side,ey-v*back-u*side]],color,width*1.28);
}
function nativeScene(presentation,scene) {
  const slide=presentation.slides.add();slide.background.fill='#FFFFFF';
  for(const item of scene.items) {
    const name=`${scene.slug} / ${item.id}`;
    if(item.kind==='line') {
      polyline(slide,name+' / path',item.points,item.color,item.width,item.dashed);
      if(item.arrow)arrowhead(slide,name+' / arrow',item.points,item.color,item.width);
      continue;
    }
    if(item.kind==='text') {
      const left=item.x-(item.align==='center'?item.w/2:item.align==='right'?item.w:0);
      // Separate lines keep the exact authored line breaks and baseline spacing.
      for(const [index,content] of item.lines.entries()) {
        const shape=slide.shapes.add({name:name+` / text ${index+1}`,geometry:'textbox',
          position:{left,top:item.y+index*item.leading,width:item.w,height:item.size*1.45},fill:'none',line:stroke('none',0)});
        shape.text=content;
        shape.text.style={typeface:font,fontSize:item.size,bold:item.bold,color:item.color,
          alignment:item.align,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
      }
      continue;
    }
    const position={left:item.x,top:item.y,width:item.w,height:item.h};
    if(item.kind==='note') {
      const w=item.w,h=item.h;
      slide.shapes.add({name,geometry:'custom',position,fill:item.fill,line:stroke(item.stroke,item.line_width??1.7),
        customPaths:[{width:w,height:h,commands:[{moveTo:{x:0,y:0}},{lineTo:{x:w-18,y:0}},
          {lineTo:{x:w,y:18}},{lineTo:{x:w,y:h}},{lineTo:{x:0,y:h}},{close:{}}]}]});
      polyline(slide,name+' / fold',[[item.x+item.w-18,item.y],[item.x+item.w-18,item.y+18],[item.x+item.w,item.y+18]],item.stroke,item.line_width??1.7);
    } else if(item.kind==='store') {
      slide.shapes.add({name,geometry:'can',position,fill:item.fill,line:stroke(item.stroke,item.line_width??1.7),
        adjustmentList:[{name:'adj',formula:'val 20000'}]});
    } else if(item.kind==='rect') {
      slide.shapes.add({name,geometry:item.flow_decision?'diamond':item.external?'hexagon':item.rounded?'roundRect':'rect',position,fill:item.fill,line:stroke(item.stroke,item.line_width??1.7,item.dashed)});
    } else throw new Error(`Unsupported scene object: ${item.kind}`);
  }
  const base='https://github.com/cbyche/via-sw-architecture/blob/main/';
  const documents={41:'04-41-request-resolution-control.md',42:'04-42-lifecycle-ownership.md',
    43:'04-43-request-interpretation.md',44:'04-44-continuous-interaction.md',45:'04-45-memory-and-context.md'};
  let notesText=`${scene.title}\n${base}docs/architecture/12-decisions/decision-packages/${documents[scene.number]}\n${base}${scene.source}\n`+
    (scene.number===45 && scene.kind==='comparison'?'조건별 정성 trade-off이며 최종 선정 또는 품질 측정이 아니다. 첫 요청, 반복 R24, 정정 직후와 표현 밖 새 관계를 구별한다.':scene.kind==='comparison'?'수치와 원형 점수는 형식 검토용 예상 예시이며 실측 또는 대안 선정 결과가 아니다. V-04는 평균 반응시간, V-05는 평균 VIA 처리시간이다. V-05는 외부 작업이나 사용자 답변만 기다리는 구간을 제외한다. 기존 시간 수치는 평균 시간의 형식 예시이며 p95 측정값을 변환한 결과가 아니다. 양안 조건은 본문과 그림의 비교 예시 조건을 따른다.':'공통 문제와 설계 고려 사항의 배경이며 특정 설계안의 선택을 뜻하지 않는다.');
  if(scene.number===41 && scene.kind==='comparison') notesText=`${scene.title}
${base}docs/architecture/12-decisions/decision-packages/04-41-request-resolution-control.md
${base}${scene.source}
조건별 정성 비교 / 미선정 / 미측정
사용자 요청은 “이 표를 아까 보고서에 넣고, 메일은 보내지 말고 결과를 바탕으로 초안만 만들어.”이다. 결과는 표를 추가한 보고서의 결과이며 그것으로 메일 초안을 작성한다. Task1은 예산 보고서 업무, Task2는 실적 보고서 업무다. “어느 보고서인가요?”라는 질문을 실제 전달하고 사용자의 “예산 보고서” 답변을 해석한 뒤 Task1을 연결한다. 초기 입력에 정답 Task를 미리 넣지 않는다.
A는 모델이 조회 요청(ReadRequest)을 선택하고 조회 결과(EvidenceBundle)를 받아 재판단하여 전체 의미 제안(MeaningProposal)을 만든다. 모델 주도 해석 제어기는 클라우드 LLM을 호출하고 그 제안을 제한하여 실행하는 로컬 코드다. 모델의 다음 행동이 조회이면 도구 실행 후 근거를 받아 모델을 다시 호출한다. 이 순환은 필요한 만큼 반복하고 모델이 완성 의미나 확인 질문을 제안하면 검증 경로로 나간다. A도 정해진 형식의 출력, 부분 수정과 cache를 허용한다.
B는 모델이 요청 틀(RequestFrame)을 만들고 Request Interpreter 내부 Request Resolution Engine이 필요한 조회와 항목 해결, 전체 의미 결합을 결정한다. Engine은 항목 검사와 조회 결과 반영을 코드 루프로 반복한다. 조회 결과를 코드만으로 처리할 수 있으면 LLM을 호출하지 않는다. 항목 의미 해석이 필요하면 요청 틀 해석기를 통해 부분 모델 호출을 하고 그 결과를 같은 코드 루프에 돌려준다. 필요한 항목만 모델에 다시 해석시킬 수 있으므로 항상 한 번 호출로 끝나는 구조가 아니다. 전체 관계를 모델이 만들고 코드가 복사하는 경우는 순수 B가 아닌 혼합안이다. 코드로 연결했다고 의미가 항상 옳은 것은 아니다.
읽기 도구 실행기는 Context Manager와 Task Manager에 각각 직접 조회한다. 화면/대화 근거와 업무 후보, 출처와 조회한 자료의 변경 번호를 받는다. 문서형 도형은 교환 내용의 설명용 예시이며 Component, Module, 저장 상태 또는 확정 API schema가 아니다.
이 발표 그림은 사용자 발화 → Interaction Manager → Request Controller → Request Interpreter 시작 경로를 간략히 표시하고 요청 해석을 확대했다. 마름모는 제어 흐름 분기이며 Component나 Module이 아니다. 반복은 완성 의미 제안, 확인 질문 대기, 실패, 취소 또는 한도 도달에서 종료한다. 확인 질문의 실제 전달과 답변 뒤 새 해석은 공통 후속 경로다. 공통 음성 처리 내부, Model Access adapter 내부, Policy Manager 권한, Request Controller 검증/저장/채택, State Store, Response Manager 출력과 실제 전달, Task Manager/Agent Gateway 위임은 전체 MAIN에 남아 있다. 발표에서 생략은 설계 삭제가 아니다. 임시 해석 상태는 Request Interpreter, 채택 의미/질문은 Request Controller, 실제 전달 원본은 Response Manager다. 입력/자료/업무/질문이 바뀌었는지와 권한을 검사하고 저장 성공 후 채택한다.
revision/version은 자료나 상태의 변경 번호다. v2는 두 번째 버전이며 최신임을 보증하지 않는다. 그림은 이를 조회 시점과 자료 변경 확인으로 풀어 썼다. 자료가 바뀌면 옛 해석을 그대로 채택하지 않는 검사는 유지한다.
도구 조회 자체는 모델 호출이 아니다. Context 의미 가공, 응답 구성, 음성/전사, 실패/재시도/폐기된 호출의 실제 사용량도 비용에 포함한다. 조건별 trade-off는 가설이며 품질 우열을 측정하지 않았다.`;
  if(scene.number===41 && scene.kind==='background') notesText+='\nTask1은 예산 보고서, Task2는 실적 보고서 업무다. 선택한 표와 이전 업무 두 후보가 있으므로 아까라는 표현만으로 정답을 미리 결정하지 않는다. 보고서 수정 결과를 메일 초안에 사용하고 메일은 보내지 않는 조건을 유지한다. 보고서 수정과 메일 작성은 외부 업무 수행 Agent가 맡는다.';
  if(scene.number===44 && scene.kind==='comparison') notesText+='\nA 중앙 조정 방식: Request Controller 내부 Dialogue Dispatcher와 Dialogue Progress State. B 이벤트 흐름을 연결하는 방식: Request Controller 내부 Input Resolution Stage·Task Notice Stage와 Window, Response Manager 내부 Publication Join·Publication Window. 같은 정식 10개 Component와 원본 owner, 흰색/검정 공통 및 살구색/짙은 테두리 차이 표기.\nVoice Runtime에는 Interaction Manager의 음성 입출력·즉시 중단 기능과 Model Access client가 배치된다. 동시 실행 수가 제한된 비동기 executor와 Blocking worker pool, Model Access 별도 scheduler를 사용한다. A 사건 대기열·비동기 작업·완료 반환은 실행 기반이, B 이벤트 채널·구독·단계별 수용량·취소 전달은 실행 라이브러리가 지원한다. 수치·우선순위·pool 크기는 미정이며 별도 Component·process·모델을 추가하지 않는다.';
  if(scene.number===45) notesText+='\n공통 사례: C20/E20 VIA 직접 평가 기준 설명과 실제 전달 P20, C21/T21/D21@v2 제품 비교, C22/T22/D22@v1 견적. 현재 C23/R23의 제안서 작성 Task T23 연결은 현재 해석/채택의 결과다. VIA는 허용된 발췌와 참조를 연결하고, 평가 기준 적용과 제안서 작성은 Downstream Agent가 수행한다. 결과 참조는 본문 전체 보관을 뜻하지 않으며 Agent 내부 reasoning/모든 tool 기록은 전제하지 않는다.\nA의 요약/index/cache/관계 cache/부분 갱신/병렬 조회와 기존 Agent 실행 맥락 재사용을 허용한다. B의 과거 관계는 검증/게시 범위만 읽고 원본 확인, 미게시/표현 미지원, 오류/갱신/삭제와 생산 비용을 유지한다. R23의 정답 관계를 미리 게시하지 않는다. 필요성과 사례 보완이며 최종 DP/A/B 선정 또는 새 사례의 품질 검증이 아니다. 기존 수치와 점수는 동일한 형식 예시다.';
  if(scene.number===45 && scene.kind==='comparison') {
    notesText=notesText.replace('기존 수치와 점수는 동일한 형식 예시다.','45의 수치와 원형 점수는 조건별 정성 표로 교체했다.');
    notesText+='\nA: Context Composer가 owner 읽기와 cache를 조합하여 직접 EvidenceBundle/receipt를 반환한다. Request Evidence Set은 요청 수명의 데이터다. B: 변경 알림, 실제 원본 읽기, 조건부 과거 의미 생산, 출처/권한/지원 종류 검증, 관계+coverage 게시, Evidence Reader의 게시 계약 소비를 나눈다. 미래 R23의 정답이나 T23 binding을 미리 생산하지 않는다.\nNOT_COVERED는 지원 종류의 미생산으로 Publisher 생산 후 revision/상태를 반환하고 Reader가 Repository를 다시 읽는다. UNSUPPORTED_RELATION은 표현 변경 없이는 반복 생산으로 해결되지 않는다. NO_MATCH는 읽은 범위의 무일치다. DENIED/UNAVAILABLE도 분리한다. 현재 metadata 검사와 exact quote/조건/충돌 원문 확인은 남고, 원본 조합 우회는 명시적 B+다. 파생 오류 EvidenceDispute는 원본 revision 변화 없이도 차단/재검증한다. 삭제/철회 fence가 후속 정리보다 먼저이며 옛 job 게시를 거부한다.\n첫 요청/정정 직후: B 생산/갱신 대기가 추가될 수 있다. 반복 조회: 유효 게시가 owner 교차 조합을 실제 대체하면 B 요청 경로가 줄지만 A warm 관계 cache가 같은 효과를 낼 수 있다. 새 관계/미묘한 부정/예외: A 원문 구성이 유연하나 검색 누락 가능, B 표현/추출 한계가 남는다. 정확성의 전체 우열을 고정하지 않는다.\n전체 모델 호출: A cache 생산/유지+요청 구성/재검증, B 초기/갱신/backfill/미사용/실패 생산을 모두 포함한다. 현재 판단은 공통이고 명시 ID/위치 연결은 코드일 수 있다. 고정 우열 없음. Source 형식 변경은 adapter에 흡수할 수 있고, 관계 의미/schema 변경은 B producer/독자 호환/재생산, A cache 무효화도 함께 본다.\n공식 Reference (본문 §9):\nA https://docs.langchain.com/oss/python/deepagents/retrieval — 현재 질문의 원본 획득 선례. VIA Task/실제 전달/정정 계약 전체 구현 사례 아님.\nB https://langchain-ai.github.io/langmem/background_quickstart/ 및 https://langchain-ai.github.io/langmem/reference/memory/ — 기억 생산/저장/소비 분리 선례. 자동 선호 추출을 VIA User Memory 정책으로 채택하지 않고 revision/coverage/삭제 fence는 VIA 별도 계약.\n실행 https://langchain-ai.github.io/langmem/guides/delayed_processing/ — 대기 생산/재예약 선례. thread/지연값/배치를 채택하지 않는다.\n연결 참고 https://docs.mem0.ai/platform/features/graph-memory — 공통 개체 연결 선례이며 typed 정정/대체/실제 전달 관계 구현 증거 아님. cloud/embedding/helper를 도입하지 않는다. 공식 사례는 VIA 성능 우위의 증거가 아니다.';
  }
  if(scene.kind==='overview') notesText=scene.notes+'\n\n본 발표: 도입 → 41 → 44 → 45 → 42. 43은 부록이며 후보 자격 변경을 뜻하지 않는다. 시간축은 상대적인 설명용이며 구간 길이는 측정값이 아니다. Agent 내부 기록 전체를 보유한다고 가정하지 않는다.\n'+base+scene.source;
  if(scene.presentationTransition) notesText+='\n다음 주제로 연결\n'+scene.transition;
  slide.speakerNotes.textFrame.setText(notesText);
  return slide;
}

const mainOrder=[41,44,45,42,43];
const integratedScenes=[scenes.find(s=>s.kind==='overview'),...mainOrder.flatMap(n=>['background','comparison'].map(kind=>({...scenes.find(s=>s.number===n&&s.kind===kind),presentationTransition:kind==='comparison'&&[41,44,45].includes(n)})))];
const plans=[
  {slug:'VIA-DP-background-and-comparison-41-45',scenes:integratedScenes,destination:'docs/presentations_files/VIA-DP-background-and-comparison-41-45.pptx'},
  {slug:'VIA-DP-background-41-45',scenes:scenes.filter(s=>s.kind==='background'),destination:'docs/presentations_files/dp-background/VIA-DP-background-41-45.pptx'},
  {slug:'VIA-DP-comparison-41-45',scenes:scenes.filter(s=>s.kind==='comparison'),destination:'docs/presentations_files/dp-comparison/VIA-DP-comparison-41-45.pptx'},
];
const checkPackage=String.raw`
import sys,json,zipfile,re
import xml.etree.ElementTree as E
sys.path.insert(0,sys.argv[3]);from dp_pptx_package import ordered_slides
scenes=json.loads(open(sys.argv[2]).read());ns={'p':'http://schemas.openxmlformats.org/presentationml/2006/main','a':'http://schemas.openxmlformats.org/drawingml/2006/main'}
with zipfile.ZipFile(sys.argv[1]) as z:
    parts={n:z.read(n) for n in z.namelist()}
    files=[s['part'] for s in ordered_slides(parts)]
    assert len(files)==len(scenes),(len(files),len(scenes))
    stats=[]
    for file,scene in zip(files,scenes):
        root=E.fromstring(z.read(file));texts=[n.text or '' for n in root.findall('.//a:t',ns)]
        required=[line for item in scene['items'] if item['kind']=='text' for line in item['lines']]
        assert texts==required,(scene['slug'],'text order or content changed')
        assert not root.findall('.//p:pic',ns),scene['slug']+' contains a flattened image'
        native=len(root.findall('.//p:sp',ns)); paths=len(root.findall('.//a:custGeom',ns))
        assert paths>0 and native>50,(scene['slug'],native,paths)
        stats.append(dict(slug=scene['slug'],nativeShapes=native,nativeTextLines=len(texts),nativePaths=paths,images=0))
    print(json.dumps(stats))
`;
// Scene identity is resolved through relationships by dp_pptx_package.py.
function selectedScenes(plan, requested=scope) {
  return plan.scenes.filter(s=>requested==='--intro-flow-only'?s.kind==='overview':
    requested==='--dp45-only'?s.number===45&&(!subset||s.kind==='comparison'):
    requested==='--dp44-only'?s.number===44&&s.kind==='comparison':
    requested==='--dp41-only'?s.number===41&&s.kind==='comparison':
    requested==='--dp41-with-background'?s.number===41:
    requested==='--dp41-42-only'?[41,42].includes(s.number)&&s.kind==='comparison':
    requested==='--comparison-only'?s.kind==='comparison':true);
}
const activePlans=plans.filter(p=>scope==='--intro-flow-only'?p===plans[0]:
  !scope||scope==='--dp41-with-background'||(scope==='--dp45-only'&&!subset)||p.scenes.some(s=>s.kind==='comparison'));
if(scope==='--list-selected') {
  console.log(JSON.stringify(plans.map(plan=>({file:plan.destination,selections:Object.fromEntries(
    ['--intro-flow-only','--dp41-only','--dp41-with-background','--dp41-42-only','--dp44-only','--dp45-only','--comparison-only'].map(option=>[option,selectedScenes(plan,option).map(s=>({slug:s.slug,position:plan.scenes.indexOf(s)+1}))]))})),null,2));
  process.exit(0);
}
const stats=[];
for(const plan of activePlans) {
  const presentation=Presentation.create({slideSize:{width:1920,height:1080}});
  for(const scene of plan.scenes)nativeScene(presentation,scene);
  const draft=path.join(build,`${plan.slug}.candidate.pptx`),final=path.join(build,'final',`${plan.slug}.pptx`);
  await fs.mkdir(path.dirname(final),{recursive:true});
  await (await PresentationFile.exportPptx(presentation)).save(draft);
  if(['--dp41-only','--dp41-with-background','--dp41-42-only','--dp44-only','--dp45-only','--comparison-only','--intro-flow-only'].includes(scope)) {
    const selected=scope==='--intro-flow-only'?[]:selectedScenes(plan).map(s=>s.slug);
    execFileSync(python,[path.join(repo,'scripts/presentations/dp_pptx_package.py'),draft,
      path.join(repo,plan.destination),JSON.stringify(selected),...(scope==='--intro-flow-only'?['--intro']:[])]);
  }
  if(!scope && plan===plans[0]) execFileSync(python,[path.join(repo,'scripts/presentations/dp_pptx_package.py'),draft,draft,'[]','--sections-only']);
  await finalizePresentation({workspaceDir:build,candidatePath:draft,finalPath:final,explicitTotalSlideCount:plan.scenes.length,
    pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
    layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
    layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit'],
    requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,
    receiptPath:path.join(build,`${plan.slug}.validation.json`)});
  const selected=path.join(build,`${plan.slug}.scenes.json`);await fs.writeFile(selected,JSON.stringify(plan.scenes));
  const editability=JSON.parse(execFileSync(python,['-c',checkPackage,final,selected,path.join(repo,'scripts/presentations')],{encoding:'utf8'}));
  const imported=await PresentationFile.importPptx(await FileBlob.load(final));
  for(const [index,slide] of imported.slides.items.entries()) {
    const png=await imported.export({slide,format:'png',scale:1});
    const preview=path.join(build,'renders',plan.slug,`${String(index+1).padStart(2,'0')}-${plan.scenes[index].slug}.png`);
    await fs.mkdir(path.dirname(preview),{recursive:true});
    await fs.writeFile(preview,new Uint8Array(await png.arrayBuffer()));
  }
  const destination=path.join(repo,plan.destination);
  await fs.copyFile(final,destination);
  stats.push({file:plan.destination,slides:plan.scenes.length,editability});
  console.log(`PASS: ${plan.slug}, ${plan.scenes.length} editable slides, no slide images`);
}
await fs.writeFile(path.join(build,'editability.json'),JSON.stringify(stats,null,2));
