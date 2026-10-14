import fs from 'node:fs/promises';
import path from 'node:path';
import {Presentation,PresentationFile} from '@oai/artifact-tool';
const [repo,build]=process.argv.slice(2);
const root=path.join(repo,'docs/presentations_files/necessity-options-v12');
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const C={navy:'#082759',sage:'#DDECCD',gray:'#D4DBE4',ink:'#000000'};
const pt=n=>n*4/3;
function tx(s,name,text,x,y,w,h,size=22,{color=C.ink,bold=false,align='left',fill='none',geometry='textbox'}={}){
 const o=s.shapes.add({name,geometry,position:{left:x,top:y,width:w,height:h},fill,line:{fill:'none',width:0}});
 o.text=text;o.text.style={typeface:'Calibri',fontSize:pt(size),color,bold,alignment:align,verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};return o;
}
function separator(s,name,x,y,w,h){s.shapes.add({name,geometry:'rect',position:{left:x,top:y,width:w,height:h},fill:C.gray,line:{fill:'none',width:0}});}
async function pic(s,file,x,y,w,h){const data=await fs.readFile(path.join(root,'assets',file+'.png'));s.images.add({name:file,blob:data.buffer.slice(data.byteOffset,data.byteOffset+data.byteLength),contentType:'image/png',alt:'VIA 과제 필요성의 설명용 생성 일러스트: '+file,fit:'contain',position:{left:x,top:y,width:w,height:h}});}
const headings=['자연스러운 대화 경험 확보','사용자 맥락의 정확한 이해','지속적인 서비스 제공과 확장'];
const fullCopy=['끼어들기와 정정의 반영\n질문과 결과의 적절한 전달\n대화와 업무의 연결 유지','발화와 당시 화면의 연결\n과거 대화와 업무 이력 활용\n지칭 대상과 정정 범위의 식별','모델 사용 비용 관리\n화면 기록과 대화 상태의 RAM 관리\n모델 및 Agent 변경 영향 최소화'];
const shortCopy=['끼어들기와 정정의 반영\n대화와 업무의 연결 유지','발화, 화면과 과거 이력의 연결\n지칭 대상과 정정 범위의 식별','모델 비용과 RAM 사용 관리\n모델 및 Agent 변경 영향 최소화'];
const versions=[['01-three-scenes','3열 장면형'],['02-connected-desktop','PC 중심 통합형'],['03-horizontal-strips','가로 3단형'],['04-magnified-scene','확대 장면형'],['05-exploded-view','정보 레이어형']];
function headerBand(s,name,title,x,y,w){tx(s,name,title,x,y,w,62,24,{fill:C.navy,color:'#FFFFFF',bold:true,align:'center',geometry:'roundRect'});}
function notes(s,i){s.speakerNotes.textFrame.setText(`과제 필요성 비교안 ${i+1}: ${versions[i][1]}\n\n기준 파일: VIA-architecture-presentation_v11.pptx, 33장 중 필요성인 2장. 세 공통 필요성은 대화 경험 확보, 사용자 맥락의 정확한 이해, 지속적인 서비스 제공과 확장.\n\n첫째, 사용자의 발화와 끼어들기 및 정정이 Downstream Agent의 질문이나 결과와 겹쳐도 대화 지속. 대화, 요청, 질문, 업무와 Agent 실행 관계 유지. FR1과 FR4에 해당.\n둘째, 발화 당시의 화면 지칭과 포인터, 클릭, 선택 범위를 사용자의 말에 연결하고 관련 과거 대화와 업무 결과를 활용해 현재 요청 이해. FR2와 FR3에 해당. 화면과 발화의 시간상 근접만으로 의도 확정 불가.\n셋째, 지속적인 모델 사용에 따른 비용과 화면 기록 및 대화 상태의 RAM 부담 관리. 모델과 Agent 변경에 따른 수정 범위 관리. 필요한 정확도와 응답성 유지가 공통 전제.\n\n그림은 image_gen 내장 도구로 생성한 설명용 합성 일러스트. 실제 제품 스크린샷, 실측 자료나 확정된 SW/HW 배치가 아님. 화면 속 표, 아이콘과 연결은 기능 관계의 시각적 표현이며 특정 업무 사례나 데이터 수치를 뜻하지 않음. RAM, cloud token, plug는 메모리 사용, 모델 사용 비용과 연동 변경의 비유. 기록 보존 방식, 모든 화면의 원문 저장, 전용 RAM, 모델 배치나 하드웨어 교체를 요구하지 않음. 레이어 구성은 필요성을 나눈 시각적 관점이며 선택된 Architecture Component나 처리 단계가 아님.\n\n출처: 사용자 합의 FR1~4 및 v11 과제 필요성, docs/presentations_files/functional-requirements-and-constraints.md.\n그림 생성 지시문: docs/presentations_files/necessity-options-v12/prompts.json\n생성 일자: 2026-10-15.\n문구는 native PowerPoint 텍스트이며 일러스트는 PNG 자원. 선택 후 통합본에 반영할 검토용 다섯 대안으로, 확정 디자인이나 설계 대안 선정 아님.`);}
// 1. Reference-like three columns, with each subject shown by a substantial scene.
{
 const s=p.slides.add();s.background.fill='#FFFFFF';
 for(let i=0;i<3;i++)headerBand(s,'heading-'+i,headings[i],60+i*610,150,580);
 const subs=['발화와 질문 및 결과의 중첩','음성, 화면 지칭과 과거 근거의 결합','반복 처리, 기록 축적과 기술 변화'];
 for(let i=0;i<3;i++)tx(s,'scene-subtitle-'+i,subs[i],70+i*610,233,560,40,20,{bold:true,align:'center'});
 await pic(s,versions[0][0],60,285,1800,600);
 for(let i=0;i<3;i++)tx(s,'copy-'+i,fullCopy[i],75+i*610,909,550,111,20,{align:'center'});
 notes(s,0);
}
// 2. One connected desktop scene with role labels and short requirement captions.
{
 const s=p.slides.add();s.background.fill='#FFFFFF';
 tx(s,'integrated-heading','지속 대화, 맥락 이해와 효율적인 운영의 동시 확보',60,145,1800,64,29,{bold:true});
 tx(s,'role-user','사용자',90,228,300,40,21,{bold:true});
 tx(s,'role-via','VIA 사용자 접점',850,228,450,40,21,{bold:true,align:'center'});
 tx(s,'role-agents','Downstream Agent',1530,228,330,40,21,{bold:true,align:'right'});
 await pic(s,versions[1][0],60,280,1800,600);
 for(let i=0;i<3;i++){
  tx(s,'heading-'+i,headings[i],75+i*610,900,550,42,22,{bold:true});
  tx(s,'copy-'+i,shortCopy[i],75+i*610,953,550,75,20);
 }
 notes(s,1);
}
// 3. Three wide illustrated strips beside concise text, rather than a grid of boxes.
{
 const s=p.slides.add();s.background.fill='#FFFFFF';
 await pic(s,versions[2][0],60,170,1245,830);
 separator(s,'column-divider',1332,180,2,820);
 const hs=['자연스러운\n대화 경험 확보','사용자 맥락의\n정확한 이해','지속적인 서비스\n제공과 확장'];
 for(let i=0;i<3;i++){
  const y=198+i*278;
  tx(s,'heading-'+i,hs[i],1370,y,490,76,25,{bold:true});
  tx(s,'copy-'+i,fullCopy[i],1370,y+97,490,114,20);
 }
 notes(s,2);
}
// 4. The same workstation, with three enlarged visual details.
{
 const s=p.slides.add();s.background.fill='#FFFFFF';
 for(let i=0;i<3;i++){
  headerBand(s,'heading-'+i,headings[i],60+i*610,145,580);
  tx(s,'copy-'+i,shortCopy[i],75+i*610,220,550,69,19,{align:'center'});
 }
 await pic(s,versions[3][0],60,294,1800,768);
 notes(s,3);
}
// 5. A layered product illustration with its three explanatory viewpoints at right.
{
 const s=p.slides.add();s.background.fill='#FFFFFF';
 await pic(s,versions[4][0],45,170,1275,850);
 const hs=['대화의 지속','맥락의 이해','운영 부담의 관리'];
 for(let i=0;i<3;i++){
  const y=214+i*279;
  tx(s,'viewpoint-number-'+i,'0'+(i+1),1370,y,65,50,24,{color:C.navy,bold:true});
  tx(s,'heading-'+i,hs[i],1440,y,420,50,25,{bold:true});
  separator(s,'viewpoint-rule-'+i,1370,y+70,490,2);
  tx(s,'copy-'+i,fullCopy[i],1370,y+93,490,117,20);
 }
 notes(s,4);
}
await(await PresentationFile.exportPptx(p)).save(path.join(build,'authored.pptx'));
console.log('Authored five illustration-led necessity alternatives');
