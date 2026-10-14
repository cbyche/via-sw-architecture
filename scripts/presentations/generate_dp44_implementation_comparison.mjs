import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

const [repo, build] = process.argv.slice(2);
if (!repo || !build) throw new Error('Usage: generate_dp44_implementation_comparison.mjs REPO BUILD');
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (!skill || !python) throw new Error('Missing bundled presentation runtime paths');
const {finalizePresentation} = await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const scene = JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/dp-comparison/dp44-implementation-comparison.scene.json'),'utf8'));
const presentation = Presentation.create({slideSize:{width:1920,height:1080}});
const slide = presentation.slides.add();
slide.background.fill = '#FFFFFF';
const lineStyle = (color,width=1.7,dashed=false) => ({fill:color==='none'?'none':color,width:color==='none'?0:width,style:dashed?'dashed':'solid'});
function pathShape(name,points,color,width,dashed=false) {
  const xx=points.map(p=>p[0]),yy=points.map(p=>p[1]);
  const left=Math.min(...xx),top=Math.min(...yy),w=Math.max(.1,Math.max(...xx)-left),h=Math.max(.1,Math.max(...yy)-top);
  slide.shapes.add({name,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',line:lineStyle(color,width,dashed),
    customPaths:[{width:w,height:h,commands:points.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
}
for (const item of scene.items) {
  const name=`${scene.slug} / ${item.id}`;
  if (item.kind==='line') {
    pathShape(name+' / path',item.points,item.color,item.width,item.dashed);
    if (item.arrow) {
      const [x,y]=item.points.at(-1); let i=item.points.length-2;
      while(i>=0 && item.points[i][0]===x && item.points[i][1]===y)i--;
      if(i>=0){const [px,py]=item.points[i],len=Math.hypot(x-px,y-py),u=(x-px)/len,v=(y-py)/len,b=item.width*6.4,s=item.width*3.2;
        pathShape(name+' / arrow',[[x-u*b-v*s,y-v*b+u*s],[x,y],[x-u*b+v*s,y-v*b-u*s]],item.color,item.width*1.28);}
    }
  } else if (item.kind==='text') {
    const left=item.x-(item.align==='center'?item.w/2:item.align==='right'?item.w:0);
    item.lines.forEach((content,index)=>{
      const shape=slide.shapes.add({name:name+` / text ${index+1}`,geometry:'textbox',position:{left,top:item.y+index*item.leading,width:item.w,height:item.size*1.45},fill:'none',line:lineStyle('none',0)});
      shape.text=content;
      shape.text.style={typeface:'Calibri',fontSize:item.size,bold:item.bold,color:item.color,alignment:item.align,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
    });
  } else if (item.kind==='store') {
    slide.shapes.add({name,geometry:'can',position:{left:item.x,top:item.y,width:item.w,height:item.h},fill:item.fill,line:lineStyle(item.stroke,item.line_width??1.7)});
  } else if (item.kind==='rect') {
    slide.shapes.add({name,geometry:item.rounded?'roundRect':'rect',position:{left:item.x,top:item.y,width:item.w,height:item.h},fill:item.fill,line:lineStyle(item.stroke,item.line_width??1.7,item.dashed)});
  } else throw new Error(`Unsupported kind ${item.kind}`);
}
slide.speakerNotes.textFrame.setText('DP44 구현 구조 비교. 기존 DP44 비교 페이지 뒤에 추가한 확대 보기.\n'+
  'A: Request Controller의 Control Lane과 Event Mailbox를 Dialogue Dispatcher가 소비하고, Job Index 및 Dialogue Progress State를 조회해 후속 실행과 결과 폐기를 결정한다. Request Interpreter 및 Response Composer의 비동기 완료는 Event Mailbox로 돌아온다.\n'+
  'B: Input Resolution Stage와 Task Notice Stage는 Request Controller 안에, Response Composer와 Publication Join은 Response Manager 안에 있다. 각 Stage는 입력, 완료, 제어 Queue 및 로컬 RAM 상태를 소유한다. 완료 callback은 생산 Stage의 Done Queue로 돌아오며 Join은 후보와 조건을 ID 및 revision으로 결합한다.\n'+
  '빨간 점선은 InputStarted의 즉시 hold, 채택된 정정의 invalidate, 실행 중 Job 취소 요청이다. 파란 실선은 일반 입력 및 후속 실행, 파란 점선은 비동기 완료 반환이다. 취소 요청만으로 실제 Job 중단을 단정하지 않는다.\n'+
  '양안 모두 Interaction Manager의 실제 출력과 ActualReceipt, Publication Control의 현재성 및 권한 검사를 거친다. 이 장은 미구현, 미측정, 미선정 설계안이다.\n'+
  'https://github.com/cbyche/via-sw-architecture/blob/main/docs/architecture/12-decisions/decision-packages/04-44-continuous-interaction.md');
await fs.mkdir(build,{recursive:true});
const candidate=path.join(build,'candidate.pptx'),final=path.join(build,'final','VIA-DP44-implementation-comparison.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await (await PresentationFile.exportPptx(presentation)).save(candidate);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:1,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','18288000,10287000'],
  requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:['Calibri']},verifyArtifactToolImport:true,
  receiptPath:path.join(build,'validation.json')});
console.log(final);
