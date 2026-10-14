import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile} from '@oai/artifact-tool';
const build=await fs.realpath(process.argv[2]);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
if(!path.isAbsolute(skill??''))throw new Error('VIA_PRESENTATION_SKILL_DIR required');
const {makeNativeBulletParagraphs}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const C={navy:'#082759',sage:'#DDECCD',pale:'#F1F5F9',border:'#D4DBE4',ink:'#000000',muted:'#4A5563',line:'#8398AF',cyan:'#E7F4F8'};
const pt=n=>n*4/3;
function shape(slide,name,text,x,y,w,h,size=22,{fill='none',stroke='none',color=C.ink,bold=false,align='left',geometry='textbox',inset=0}={}){
 const s=slide.shapes.add({name,geometry,position:{left:x,top:y,width:w,height:h},fill,line:{fill:stroke,width:stroke==='none'?0:1.5}});
 if(text){s.text=text;s.text.style={typeface:'Calibri',fontSize:pt(size),color,bold,alignment:align,verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:{left:inset,right:inset,top:0,bottom:0}};}return s;
}
const tx=(s,n,t,x,y,w,h,z=22,o={})=>shape(s,n,t,x,y,w,h,z,o);
const box=(s,n,x,y,w,h,fill=C.pale,stroke='none')=>shape(s,n,'',x,y,w,h,22,{fill,stroke,geometry:'rect'});
const node=(s,n,t,x,y,w,h,z=19,fill=C.pale)=>shape(s,n,t,x,y,w,h,z,{fill,stroke:C.border,bold:true,align:'center',geometry:'roundRect'});
function arrow(s,n,x,y,w,h=20,geometry='rightArrow'){return shape(s,n,'',x,y,w,h,20,{fill:C.line,geometry});}
function rule(s,n,x,y,w,color=C.border){box(s,n,x,y,w,1.5,color);}
// Only these six named elements are transplanted into the original v9 first slide.
// Its native chart, workbook, GIFs and all other shapes remain original package bytes.
const s=p.slides.add();s.background.fill='#FFFFFF';
arrow(s,'user-via-link',1118,982,38,24,'leftRightArrow');
arrow(s,'via-agent-link',1509,982,38,24,'leftRightArrow');
tx(s,'demo-two-source','OpenAI 공식 발표 2026.09.03  /  시연 발췌, 1.5배속',92,1024,770,24,14,{color:C.muted,align:'center'});
tx(s,'market-title','VIA 관련 상위 시장\n음성 비서 애플리케이션',974,222,356,75,20,{bold:true});
tx(s,'market-source','시장: Mordor Intelligence, 2026.08  /  전 산업 음성 비서 SW 및 서비스',972,612,868,25,13,{color:C.muted});
tx(s,'cases-source','PC 전용 시장 규모와 구별  /  기업 사례: 각 사 공식 자료, 발표자 노트 참조',972,639,868,23,13,{color:C.muted});
s.speakerNotes.textFrame.setText(await fs.readFile(path.join(build,'slide1-notes.txt'),'utf8'));
const n=p.slides.add();n.background.fill='#FFFFFF';
const cols=[
 {x:60,title:'자연스러운 대화 경험 확보',a:'사용자 입력과 업무 응답의\n끊김 없는 연결',b:['끼어들기와 정정의 신속한 반영','질문과 결과의 적절한 전달','대화, 요청, 업무와 실행의 연결'],quality:'응답 지연 최소화'},
 {x:670,title:'사용자 맥락의 정확한 이해',a:'발화, 화면과 이력에 근거한\n현재 요청 이해',b:['발화 당시 화면과 조작 정보 결합','과거 대화와 업무 결과의 활용','지칭 대상과 정정 범위의 정확한 식별'],quality:'사용자 의도와 지칭의 정확성 확보'},
 {x:1280,title:'지속적인 서비스 제공과 확장',a:'지속 사용에 따른 운영 부담과\n기술 변경 비용의 관리',b:['반복 추론과 불필요한 호출 최소화','화면 기록과 대화 상태의 메모리 관리','모델 및 Agent 변경의 영향 범위 축소'],quality:'운영 부담과 변경 비용 절감'},
];
for(const [i,c]of cols.entries()){
 shape(n,`need-panel-${i}`,'',c.x,187,580,835,22,{fill:'#FFFFFF',stroke:C.border,geometry:'roundRect'});
 shape(n,`need-heading-${i}`,c.title,c.x+10,153,560,66,24,{fill:C.navy,color:'#FFFFFF',bold:true,align:'center',geometry:'roundRect'});
 tx(n,`need-copy-a-${i}`,c.a,c.x+30,694,520,72,23,{bold:true,align:'center'});
 rule(n,`need-copy-rule-${i}`,c.x+34,788,512);
 const list=tx(n,`need-copy-b-${i}`,' ',c.x+30,808,520,132,20);
 list.text=makeNativeBulletParagraphs(c.b,{marginLeftPoints:15,hangingPoints:9,spaceAfterPoints:10});
 list.text.style={typeface:'Calibri',fontSize:pt(20),color:C.ink,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
 shape(n,`need-quality-${i}`,c.quality,c.x+26,958,528,46,20,{fill:C.sage,bold:true,align:'center',geometry:'roundRect'});
}
// 1. Two event lanes share the same horizontal time axis. Shading identifies overlap.
{
 const x=60;
 tx(n,'continuity-caption','사용자 발화와 Agent 응답의 중첩',x+30,246,520,36,19,{bold:true});
 box(n,'overlap-region',x+281,295,112,176,'#EEF4E8');
 tx(n,'user-lane','사용자 발화',x+30,290,240,30,17);
 arrow(n,'user-time-axis',x+30,353,518,10);
 node(n,'new-request','새 요청',x+30,327,144,50,18,C.cyan);
 node(n,'interruption-correction','끼어들기와 정정',x+285,327,263,50,18,C.cyan);
 tx(n,'agent-lane','Downstream Agent',x+30,395,400,30,17);
 arrow(n,'agent-time-axis',x+30,453,518,10);
 node(n,'agent-question','질문',x+205,427,140,50,18);
 node(n,'agent-result','결과',x+405,427,143,50,18);
 tx(n,'overlap-label','동시 발생',x+283,482,136,26,16,{bold:true,color:C.navy,align:'center'});
 tx(n,'time-label','시간',x+491,482,56,26,14,{color:C.muted,align:'right'});
 // Relationships, rather than lifecycle ordering or implementation ownership.
 tx(n,'relations-label','대화와 업무의 관계',x+30,530,250,30,17,{bold:true});
 node(n,'relation-question','질문',x+155,570,92,42,17,'#FFFFFF');
 box(n,'question-request-link',x+200,612,2,20,C.line);
 const rel=[['대화',30,94],['요청',155,92],['업무',280,92],['Agent 실행',405,143]];
 for(const [i,[t,dx,w]]of rel.entries()){
  node(n,'relation-'+i,t,x+dx,632,w,42,17,i===1?C.sage:C.pale);
  if(i<3)box(n,'relation-edge-'+i,x+dx+w,652,31,2,C.line);
 }
}
// 2. Concrete evidence fields feed request understanding. This does not select a DP alternative.
{
 const x=670;
 tx(n,'context-caption','현재 요청에 필요한 정보의 결합',x+30,246,520,36,19,{bold:true});
 const inputs=[['음성\n발화 내용과 정정',301,C.cyan],['당시 화면\n활성 창, 포인터\n클릭, 선택 범위',423,C.cyan],['과거 이력\n대화와 업무 결과',545,C.pale]];
 for(const [i,[t,y,fill]]of inputs.entries()){
  node(n,'context-input-'+i,t,x+30,y,264,106,18,fill);
  box(n,'context-wire-'+i,x+294,y+52,34,2,C.line);
 }
 box(n,'context-bus',x+326,354,2,244,C.line);
 arrow(n,'context-adoption',x+328,465,28,22);
 node(n,'context-output','현재 요청 이해\n\n대상과 조건\n관련 업무\n정정 범위',x+358,364,192,224,19,C.sage);
}
// 3. Workload and technology conditions explain three different cost drivers.
{
 const x=1280;
 tx(n,'resource-caption','운영 조건에 따른 부담',x+30,246,520,36,19,{bold:true});
 const rows=[['지속적인 대화\n반복적인 맥락 처리','모델 사용 비용'],['화면 기록\n대화와 업무 상태','PC 메모리 점유'],['모델과 Agent의\n기능 및 규격 변화','수정과 재검증']];
 for(const [i,[cause,effect]]of rows.entries()){
  const y=307+i*128;
  node(n,'resource-cause-'+i,cause,x+30,y,266,92,18,C.pale);
  arrow(n,'resource-arrow-'+i,x+300,y+35,37,22);
  node(n,'resource-effect-'+i,effect,x+344,y,206,92,19,i===2?C.sage:C.cyan);
 }
}
n.speakerNotes.textFrame.setText(`과제 필요성\n\n대화 연속성과 관계 유지\n사용자 발화, 끼어들기와 정정, Downstream Agent의 질문과 결과가 시간상 겹치는 조건. 질문과 결과가 도착하더라도 사용자 입력을 계속 수용하고 적절한 응답 시점을 결정할 필요. 아래 관계도는 대화, 요청, 질문, 업무, Agent 실행을 연결하는 정보 관계의 축약 표현. 단일 요청과 질문의 엄격한 소유 관계나 일대일 카디널리티, 처리 순서, 프로세스 분리를 규정하지 않음. FR1과 FR4, DP44와 DP42의 공통 문제 배경.\n\n현재 요청의 맥락 이해\n발화 내용과 정정, 발화 당시 화면의 활성 창과 포인터 및 클릭과 선택 범위, 과거 대화와 업무 결과가 입력 근거. 현재 요청에 관련된 근거를 사용해 대상, 조건, 관련 업무와 정정 범위를 이해할 필요. 화면과 발화의 시간상 근접만으로 의도 확정 불가. 모든 앱에서 객체나 선택 범위의 완전한 수집을 보장하는 그림이 아님. FR2와 FR3, DP47와 DP45의 공통 문제 배경.\n\n지속 제공과 확장\n지속적인 대화와 맥락 처리는 모델 사용 비용, 화면 기록과 대화 및 업무 상태는 PC 메모리 사용, 모델과 Agent의 기능 및 규격 변화는 수정과 재검증 부담의 원인. 비용과 메모리의 수치 목표, 특정 모델 배치, 저장 방식이나 분리 구조를 선택하는 내용이 아님. 필요한 정확도와 응답성을 유지하면서 운영 부담과 변경 범위의 관리 필요.\n\n근거 문서\ndocs/presentations_files/functional-requirements-and-constraints.md\ndocs/architecture/12-decisions/decision-packages/03-03-qa-dp-evaluation-plan.md\n기능 요구사항은 사용자 합의 FR1~4 기준. 품질 목표와 설계 대안의 최종 선정은 별도 검토 대상.`);
await(await PresentationFile.exportPptx(p)).save(path.join(build,'authored.pptx'));
await fs.mkdir(path.join(build,'draft-renders'),{recursive:true});
const img=await p.export({slide:n,format:'png',scale:1});
await fs.writeFile(path.join(build,'draft-renders','2.png'),new Uint8Array(await img.arrayBuffer()));
console.log('Authored source slide patches and editable necessity diagrams');
