import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Run a copy in a private build with bundled node_modules linked there.
const [repoArg, buildArg] = process.argv.slice(2);
const repo=await fs.realpath(repoArg), build=await fs.realpath(buildArg);
const python=process.env.VIA_RUNTIME_PYTHON, skill=process.env.VIA_PRESENTATION_SKILL_DIR;
if(![python,skill].every(v=>path.isAbsolute(v??'')))throw new Error('Set bundled paths');
const source=path.join(repo,'docs/presentations_files/VIA-architecture-presentation_v7.pptx');
const digest=data=>createHash('sha256').update(data).digest('hex');
const sourceHash=digest(await fs.readFile(source));
if(sourceHash!=='80f5762b962da0d732f973a11eda706b6d66f52e530bacdd05a8d98c3519e44e')throw new Error('Rebase on changed source');
const scenes=JSON.parse(await fs.readFile(path.join(build,'scenes.json'),'utf8'));
const authored=Presentation.create({slideSize:{width:1920,height:1080}});
function poly(slide,name,pts,color,width,arrow=false,dashed=false){
  const x=Math.min(...pts.map(p=>p[0])),y=Math.min(...pts.map(p=>p[1]));
  const w=Math.max(1,Math.max(...pts.map(p=>p[0]))-x),h=Math.max(1,Math.max(...pts.map(p=>p[1]))-y);
  slide.shapes.add({name,geometry:'custom',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:color,width,style:dashed?'dashed':'solid'},
    customPaths:[{width:w,height:h,commands:pts.map(([px,py],i)=>({[i?'lineTo':'moveTo']:{x:px-x,y:py-y}}))}]});
  if(arrow){const [a,b]=pts.slice(-2),theta=Math.atan2(b[1]-a[1],b[0]-a[0]),n=13;
    poly(slide,name+'-arrow',[[b[0]-n*Math.cos(theta-.5),b[1]-n*Math.sin(theta-.5)],b,[b[0]-n*Math.cos(theta+.5),b[1]-n*Math.sin(theta+.5)]],color,width);}
}
for(const scene of scenes){
  const slide=authored.slides.add();slide.background.fill='#FFFFFF';
  for(const i of scene.items){
    if(i.kind==='line'){poly(slide,i.id,i.points,i.color,i.width,i.arrow,i.dashed);continue;}
    const shape=slide.shapes.add({name:i.id,geometry:i.kind==='box'?i.geometry:'textbox',position:{left:i.x,top:i.y,width:i.w,height:i.h},
      fill:i.kind==='box'?i.fill:'none',line:{fill:i.kind==='box'?i.stroke:'none',width:i.kind==='box'?2:0}});
    shape.text=i.text;
    shape.text.style={typeface:'Calibri',fontSize:i.sizePt*4/3,color:i.color,bold:i.bold,alignment:i.align,verticalAlignment:'middle',wrap:'none',autoFit:'none',
      insets:{left:i.kind==='box'?14:0,right:i.kind==='box'?14:0,top:0,bottom:0}};
  }
  slide.speakerNotes.textFrame.setText(scene.notes+'\n\n출처: docs/architecture/12-decisions/decision-packages/04-44-continuous-interaction.md; 04-45-memory-and-context.md; 04-42-lifecycle-ownership.md; screen-voice-grounding-problem.md. 2026-10-13 문제 소개 편집. 기존 비교 기제와 미선정 상태를 유지한다.');
}
const nativePath=path.join(build,'intros-authored.pptx');
await(await PresentationFile.exportPptx(authored)).save(nativePath);

const p=await PresentationFile.importPptx(await FileBlob.load(source));
const inspection=await p.inspect({select:'$.slides[*]',include:['id','index','elements.id','elements.kind','elements.name','elements.text','notes.text']});
await fs.writeFile(path.join(build,'metadata-source-inspection.ndjson'),inspection.ndjson);
const records=inspection.records;
const edits=[],tableEdits=[];
function shapeText(page,oldText,newText){
  const matches=records[page-1].elements.filter(e=>e.kind!=='table'&&e.text?.includes(oldText));
  if(matches.length!==1)throw new Error(`Expected one text on ${page}: ${oldText}, got ${matches.length}`);
  const e=matches[0],value=e.text.replace(oldText,newText);p.resolve(e.id).text=value;
  edits.push({page,name:e.name,oldText:e.text,newText:value});
}
function cell(page,row,column,newText){
  const tables=records[page-1].elements.filter(e=>e.kind==='table');
  if(tables.length!==1)throw new Error('Expected native table '+page);
  p.resolve(tables[0].id).cells.set(row,column,newText);
  tableEdits.push({page,name:tables[0].name,row,column,newText});
}
cell(12,1,0,'FR1 / FR2  화면·발화 근거와 지칭');
cell(12,1,2,'화면·음성  말한 순간의 대상을 어떻게 찾을까? (탐색)');
cell(12,2,2,'44  정정·질문·결과가 겹쳐도 사용자 통제를 어떻게 유지할까?');
cell(12,3,2,'45  설명·결과·정정의 과거 맥락을 어떻게 다시 사용할까?');
cell(12,4,2,'42  대화·요청·질문·업무·실행의 수명을 어떻게 연결할까?');
shapeText(12,'공통 제약 C1/C2는 네 비교에서 모두 지킵니다.','공통 제약 C1/C2는 모든 주제에서 지킵니다.');
shapeText(12,'같은 기능과 입력을 유지한 채, 품질 목표를 달성하는 비용과 지원 한계를 비교합니다.','44·45·42의 구조를 비교하고, 화면·음성 지칭의 문제와 대안을 탐색합니다.');
shapeText(13,'04-41. Request 해석 제어','화면·음성 지칭 (문제 탐색)');
shapeText(13,'조회와 의미 완성의 진행 책임','말과 손동작을 당시 화면에 연결');
shapeText(13,'색상 점선: 주요 DP 범위','색상 점선: 기존 A/B 비교 범위');
shapeText(13,'점선은 주요 비교 위치. 모든 DP를 6개 QA로 검토하며, 권위 상태와 VAD는 로컬, 음성과 의미 모델은 클라우드',
  '44·45·42의 기존 A/B 범위입니다. 화면·음성 주제의 구조는 미정이며, VIA의 요청 이해 기능은 유지합니다.');
cell(23,0,2,'구조 비교와 탐색 범위');
cell(23,1,0,'화면·음성 지칭 (문제)');cell(23,1,1,'FR1 / FR2');
cell(23,1,2,'당시 화면과 지칭 연결, 정정 반영 (대안 탐색 전)');
shapeText(23,'현재는 네 A/B의 구조와 평가 계약을 설명하는 단계입니다. 채택된 통합 구조나 실측 승자를 표시하지 않습니다.',
  '44·45·42는 A/B를 검토하고, 화면·음성 주제는 문제를 탐색합니다. 채택안과 실측 승자는 미정입니다.');
cell(24,0,1,'같은 장면에서 확인할 조건');cell(24,1,0,'화면·음성 지칭');
cell(24,1,1,'두 화면의 지칭, 늦은 전사와 “아니, 여기” 정정');
cell(24,1,2,'발화·화면·pointer·선택 기록, 누락 범위');
shapeText(24,'A/B 선택과 품질 결과는 아직 없습니다. 구현/측정 착수는 별도 승인 단계입니다.',
  '44·45·42의 A/B 선정과 품질 결론은 미정입니다. 화면·음성 주제는 문제부터 검토합니다.');
shapeText(25,'이전 버전과 원본 문구, 발표자 노트는 보존했고, 아래 페이지에도 v6의 서식을 적용했습니다.',
  '58~61  v2 배경 보존 · 62~63  주요 DP에서 제외한 41의 기존 배경과 비교');
const metadataPages=[12,13,23,24,25];
for(const page of metadataPages){
  p.slides.items[page-1].speakerNotes.textFrame.setText((records[page-1].notes?.text??'')+
    '\n\n2026-10-13 v8 구성: 41은 사용자 주제 선정으로 주요 비교에서 제외하고 이전 자료를 Appendix로 보존한다. 44·45·42의 기존 A/B 기제 및 QA 정의/목표/평가 상태는 유지한다. 화면·음성 지칭은 DP 번호와 A/B가 미정인 문제 탐색이다. 이 발표 편집은 새 구현, 모델 실행, 측정 또는 최종 선정의 승인이 아니다.');
}
await(await PresentationFile.exportPptx(p)).save(path.join(build,'metadata-authored.pptx'));
const plan={version:8,source_sha256:sourceHash,scenes:scenes.map(s=>({slug:s.slug,title:s.title})),edits,tableEdits,metadataPages,
  removeOverviewNames:['overview-dp41-resolution-control','overview-tag-41-resolution','overview-dp41-tag-link','overview-goal-1-41','overview-goal-4-41']};
await fs.writeFile(path.join(build,'intro-plan.json'),JSON.stringify(plan,null,2)+'\n');
execFileSync(python,[path.join(repo,'scripts/presentations/package_problem_intros_v8.py'),repo,build],{stdio:'inherit'});
const manifest=JSON.parse(await fs.readFile(path.join(build,'manifest-v8.json'),'utf8'));
const owners=manifest.slides.filter(s=>s.native_tables>0).map(s=>s.page);
const final=path.join(build,'final','VIA-architecture-presentation_v8.pptx');await fs.mkdir(path.dirname(final),{recursive:true});
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({workspaceDir:build,candidatePath:path.join(build,'candidate-v8.pptx'),finalPath:final,explicitTotalSlideCount:63,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],
  requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'user_request',families:['Calibri']},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation-v8.json')});
manifest.output_sha256=digest(await fs.readFile(final));
await fs.writeFile(path.join(build,'manifest-v8.json'),JSON.stringify(manifest,null,2)+'\n');
if(sourceHash!==digest(await fs.readFile(source)))throw new Error('Source changed during edit');
console.log(JSON.stringify({final,sourceUnchanged:true}));
