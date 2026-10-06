import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { Presentation, PresentationFile, FileBlob } from '@oai/artifact-tool';

// Run a copy in a fresh private build directory using the bundled node_modules.
const [repoArg, buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_functional_requirements.mjs REPO BUILD_DIR');
const repo=await fs.realpath(repoArg), build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR, python=process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill??'') || !path.isAbsolute(python??'')) throw new Error('Set absolute skill and Python paths.');
const sourcePath=path.join(repo,'docs/presentations_files/functional-requirements-and-constraints.md');
const source=await fs.readFile(sourcePath,'utf8');
const functional=[], constraints=[];
for(const line of source.split('\n')) {
  if(!/^\| (FR[1-6]|C[12]) \|/.test(line)) continue;
  const cells=line.split('|').slice(1,-1).map(s=>s.trim());
  // Optional illustration rows in the same document are not requirement rows.
  if(cells.length!==4) continue;
  const [id,name,description,dp]=cells;
  if(!description.endsWith('.') || (description.match(/[.!?。]/g)??[]).length!==1) throw new Error(`${id}: expected one sentence`);
  (id.startsWith('FR')?functional:constraints).push({id,name,description,dp});
}
if(functional.map(r=>r.id).join()!=='FR1,FR2,FR3,FR4,FR5,FR6' || constraints.map(r=>r.id).join()!=='C1,C2') throw new Error('Expected six functional requirements and two constraints.');
if(/[\u00b7\u2027\u2219\u30fb]/u.test(JSON.stringify([...functional,...constraints]))) throw new Error('Middle dots prohibited.');
const out=path.join(repo,'docs/presentations_files/functional-requirements');
await fs.mkdir(out,{recursive:true});
const wraps={
  FR1:'VIA가 요청을 처리하거나 응답하는 동안에도 사용자의 음성 또는 텍스트 입력을 받아\n대화를 이어갈 수 있어야 한다.',
  FR2:'화면, 자료와 대화 맥락을 이용해 사용자 입력을 목표, 대상, 조건과 관련 업무가\n식별된 요청으로 해석할 수 있어야 한다.',
  FR3:'한 입력에 포함된 여러 요청을 사용자가 명시한 순서, 조건과 결과 의존 관계에 따라\n처리할 수 있어야 한다.',
  FR4:'여러 업무의 질문, 진행 상황, 결과와 사용자 후속 지시를 각각\n해당 업무에 연결할 수 있어야 한다.',
  FR5:'음성 연결 종료, 대화 전환 또는 VIA 재시작 이후에도 기존 업무를\n중복 실행 없이 확인 가능한 상태로 이어갈 수 있어야 한다.',
  FR6:'이전 대화의 정정 내용, 업무 결과와 허용된 사용자 기억을\n현재 요청에 필요한 맥락으로 활용할 수 있어야 한다.',
  C1:'음성 대화와 요청 의미 해석은 사용자 PC에 적재된 하나의\nOmni 모델 가중치를 공유해야 한다.',
  C2:'실제 업무 수행에 필요한 도메인 추론, 계획, 도구 선택과 실행은\nDownstream Agent에 위임해야 한다.',
};
const compact=s=>s.replace(/\s+/g,'');
for(const r of [...functional,...constraints]) if(compact(wraps[r.id])!==compact(r.description)) throw new Error(`${r.id}: layout copy diverged from source`);
const font='Apple SD Gothic Neo', blue='#2873B9', orange='#B96D14', ink='#202B35';
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const slide=p.slides.add();slide.background.fill='#FFFFFF';
function text(name,value,x,y,w,h,size,color=ink,bold=false) {
  const s=slide.shapes.add({name,geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
  s.text=value;s.text.style={typeface:font,fontSize:size,color,bold,wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
}
function wrapDP(value) {
  const ids=value.split(', '), groups=[];
  for(let i=0;i<ids.length;i+=2) groups.push(ids.slice(i,i+2).join(', '));
  return groups.join(',\n');
}
function table(rows,top,rowHeight,color,heading) {
  const values=[['ID',heading,'설명','관련 DP'],...rows.map(r=>[r.id,r.name,wraps[r.id],wrapDP(r.dp)])];
  const t=slide.tables.add({rows:values.length,columns:4,left:48,top,width:1824,height:50+rowHeight*rows.length,columnWidths:[100,350,1110,264],values});
  t.styleOptions={headerRow:false,bandedRows:false};
  t.borders.assign({style:'solid',fill:'#CBD7DF',width:1.1});
  t.rows[0].height=50;
  for(let row=0;row<values.length;row++) {
    if(row>0)t.rows[row].height=rowHeight;
    for(let col=0;col<4;col++) t.cells.block({row,column:col,rowCount:1,columnCount:1}).assign({
      fill:row===0?color:color===blue?(row%2?'#EEF4F9':'#F8FAFC'):(row%2?'#FCF4E8':'#FEFAF3'),
      margins:{left:col===2?16:10,right:col===2?16:10,top:2,bottom:2},anchor:'center',
      textStyle:{typeface:font,fontSize:row===0?27:col===2?29:col===3?26:29,bold:row===0||col===0||col===1,color:row===0?'#FFFFFF':col===0?color:ink,alignment:col===2?'left':'center'}
    });
  }
  return t;
}
text('title','기능 요구사항 및 제약사항',48,32,1824,84,64,ink,true);
text('functional-heading','기능 요구사항',48,133,1824,46,36,blue,true);
table(functional,190,79,blue,'기능 요구사항명');
text('constraints-heading','제약사항',48,752,1824,46,36,orange,true);
table(constraints,807,84,orange,'제약사항명');
text('mapping-boundary','관련 DP는 직접 설계 영향만 표시하며, Omni 공유는 별도 Streaming ASR을 금지하지 않는다.',48,1040,1824,30,23,'#586878');
slide.speakerNotes.textFrame.setText('VIA 기능 요구사항 및 제약사항\n원문: docs/presentations_files/functional-requirements-and-constraints.md\n04-41~45와 직접 연결되는 발표용 요구사항이다. 관련 DP는 직접적인 설계 영향의 연결이며 전체 준수 범위를 뜻하지 않는다. 단일 Omni 공유는 별도 입력 인식용 Streaming ASR을 금지한다는 뜻이 아니다. 특정 설계 대안의 선택 또는 구현 및 측정 결과를 나타내지 않는다.\n\n'+
  [...functional,...constraints].map(r=>`${r.id} ${r.name}\n${r.description}\n관련 DP: ${r.dp}`).join('\n\n')+
  '\n\n관련 설계 문서\n'+source.split('## 관련 설계 문서\n')[1]+
  '\n디자인 참고: 사용자 제공 조제현_09_기능요구사항및제약사항.png의 기능 요구사항 상단 표와 제약사항 하단 표. 관련 DP 열을 추가하여 한 장에 배치했다.');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const candidate=path.join(build,'candidate.pptx'), final=path.join(build,'final','VIA-functional-requirements-and-constraints.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await(await PresentationFile.exportPptx(p)).save(candidate);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:1,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit','--validate-bullet-geometry','--require-native-table-slide','1'],
  requiredNativeTableOwnerSlides:[1],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
await fs.copyFile(final,path.join(out,path.basename(final)));
const loaded=await PresentationFile.importPptx(await FileBlob.load(final));
const png=await loaded.export({slide:loaded.slides.items[0],format:'png',scale:1});
await fs.writeFile(path.join(out,'functional-requirements-and-constraints.png'),new Uint8Array(await png.arrayBuffer()));
await fs.writeFile(path.join(out,'index.html'),'<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 기능 요구사항 및 제약사항</title><style>body{margin:24px;background:#eaf0f3;font-family:system-ui;color:#202b35}main{max-width:1600px;margin:auto}img{display:block;width:100%;margin-top:20px;box-shadow:0 2px 12px #0002}a{color:#2873b9}</style><main><h1>VIA 기능 요구사항 및 제약사항</h1><p><a href="VIA-functional-requirements-and-constraints.pptx">편집 가능한 PowerPoint, 한 장</a> / <a href="../functional-requirements-and-constraints.md">Markdown 원문</a> / <a href="functional-requirements-and-constraints.png">PNG 원본</a></p><img src="functional-requirements-and-constraints.png" alt="VIA 기능 요구사항 6개와 제약사항 2개 및 관련 DP"></main></html>');
console.log(JSON.stringify({slides:1,functional:6,constraints:2,nativeTables:2,final},null,2));
