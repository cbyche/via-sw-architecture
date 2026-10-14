import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile} from '@oai/artifact-tool';
const build=await fs.realpath(process.argv[2]);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
if(!path.isAbsolute(skill??''))throw new Error('VIA_PRESENTATION_SKILL_DIR required');
const {applyPresentationChartFont}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const C={navy:'#082759',cyan:'#37ACCA',sage:'#DDECCD',green:'#487139',pale:'#F6F8FA',border:'#D4DBE4',ink:'#000000',muted:'#4A5563'};
const pt=n=>n*4/3;
function shape(slide,name,text,x,y,w,h,size=22,{fill='none',stroke='none',color=C.ink,bold=false,align='left',geometry='textbox',inset=0}={}){
 const s=slide.shapes.add({name,geometry,position:{left:x,top:y,width:w,height:h},fill,line:{fill:stroke,width:stroke==='none'?0:1.5}});
 if(text){s.text=text;s.text.style={typeface:'Calibri',fontSize:pt(size),color,bold,alignment:align,verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:{left:inset,right:inset,top:0,bottom:0}};}return s;
}
const tx=(s,n,t,x,y,w,h,z=22,o={})=>shape(s,n,t,x,y,w,h,z,o);
const box=(s,n,x,y,w,h,fill=C.pale,stroke='none')=>shape(s,n,'',x,y,w,h,22,{fill,stroke,geometry:'rect'});
function rule(s,n,x,y,w,color=C.border){box(s,n,x,y,w,1.5,color);}
function band(s,n,t,x,y,w,size=25){return shape(s,n,t,x,y,w,66,size,{fill:C.navy,color:'#FFFFFF',bold:true,align:'center',geometry:'roundRect'});}
async function pic(s,n,file,x,y,w,h){const data=await fs.readFile(path.join(build,'assets',file));s.images.add({name:n,blob:data.buffer.slice(data.byteOffset,data.byteOffset+data.byteLength),contentType:file.endsWith('.gif')?'image/gif':'image/png',alt:n,fit:'contain',position:{left:x,top:y,width:w,height:h}});}
function arrow(s,n,x,y,w){shape(s,n,'',x,y,w,24,20,{fill:'#91A5B7',geometry:'rightArrow'});}
// The matching v8 header, navigation and page field are transplanted during packaging.
const s=p.slides.add();s.background.fill='#FFFFFF';
// Left: two original animated demonstrations.
band(s,'trend-execution-title','PC 업무 실행과 음성 상호작용의 발전',60,140,830,25);
tx(s,'demo-one-label','Claude Cowork  |  브라우저 업무 수행',82,223,790,42,24,{bold:true});
box(s,'demo-one-frame',189,274,574,324,'#FFFFFF',C.border);
await pic(s,'claude-cowork-gif','v7_claude_cowork_browser_20260826.gif',191,276,570,320.625);
tx(s,'demo-one-source','Anthropic 공개 시연 발췌  /  2026.08.26',176,603,600,26,14,{color:C.muted,align:'center'});
rule(s,'demo-divider',82,637,786);
tx(s,'demo-two-label','GPT-6 Astra  |  음성으로 수정과 후속 작업',82,644,790,42,24,{bold:true});
box(s,'demo-two-frame',189,697,574,324,'#FFFFFF',C.border);
await pic(s,'gpt6-astra-gif','initial01_image2.gif',191,699,570,320.625);
tx(s,'demo-two-source','OpenAI 공개 시연 발췌  /  1.5배속',176,1024,600,24,14,{color:C.muted,align:'center'});
// Right: evidence above, strategic direction below.
band(s,'trend-market-title','음성 Agent 시장 확대와 주요 기업의 경쟁',950,140,910,25);
tx(s,'market-title','글로벌 음성 비서\n애플리케이션 시장',974,222,346,75,21,{bold:true});
tx(s,'market-cagr','연평균 15.27% 성장 전망',974,303,360,34,20,{bold:true,color:C.green});
const chart=s.charts.add('bar',{position:{left:969,top:347,width:355,height:229},categories:['2026 추정','2031 전망'],series:[{name:'시장 규모 (십억 달러)',values:[9.02,18.36],fill:C.navy,points:[{idx:0,fill:'#A3B4CA'},{idx:1,fill:C.navy}]}],hasLegend:false,
 barOptions:{direction:'column',grouping:'clustered',gapWidth:80,varyColors:true},xAxis:{visible:true,textStyle:{fontSize:pt(15),fill:C.ink,typeface:'Calibri'},line:{fill:C.border,width:1},majorGridlines:null},
 yAxis:{visible:true,min:0,max:20,majorUnit:10,textStyle:{fontSize:pt(12),fill:C.muted,typeface:'Calibri'},majorGridlines:{fill:'#E7EBEF',width:1},line:{fill:'none',width:0}},
 dataLabels:{showValue:true,position:'outEnd',numberFormatCode:'0.00',textStyle:{fontSize:pt(20),bold:true,fill:C.ink,typeface:'Calibri'}},chartFill:'#FFFFFF',chartLine:{fill:'none',width:0},plotAreaFill:'#FFFFFF',plotAreaLine:{fill:'none',width:0}});
applyPresentationChartFont(chart,{fontFamily:'Calibri'});
tx(s,'market-unit','단위: 십억 달러',974,576,355,24,13,{color:C.muted});
box(s,'market-list-divider',1342,229,1.5,368,C.border);
const cases=[['Google','Gemini Live','음성과 화면 대화, 앱 연동'],['Amazon','Alexa+','자연어 대화, 생활 서비스 실행'],['Apple','Siri AI','개인 맥락과 화면 이해, 앱 연계'],['OpenAI','GPT-6 Astra 시연','음성 기반 수정과 후속 작업 연결']];
for(const [i,[co,product,desc]]of cases.entries()){
 const y=222+i*94;
 tx(s,`company-${i}`,co,1370,y,128,34,20,{bold:true,color:C.navy});
 tx(s,`product-${i}`,product,1505,y,345,34,21,{bold:true});
 tx(s,`case-${i}`,desc,1370,y+38,480,34,18);
 if(i<3)rule(s,`case-rule-${i}`,1370,y+83,470,'#E3E7EC');
}
tx(s,'market-source','시장: Mordor Intelligence, 2026.08  /  전 산업 소프트웨어 및 서비스 기준',972,612,868,25,13,{color:C.muted});
tx(s,'cases-source','사례: 각 사 공식 발표 및 공개 시연  /  상세 출처: 발표자 노트',972,639,868,23,13,{color:C.muted});
band(s,'strategic-title','삼성의 방향: VIA를 통한 사용자 접점 확보',950,693,910,25);
box(s,'quote-highlight',981,854,525,37,C.sage);
tx(s,'cto-quote','“실제 작업을 수행하는 AI Agent가 달라져도\n사용자와의 경험은 당사의\nVoice Interaction Agent(VIA)가 담당”',981,775,845,115,23,{bold:true});
tx(s,'cto-attribution','윤장현 사장(CTO)  |  2026.06',981,889,845,35,19,{align:'right'});
shape(s,'user-role','사용자',975,964,135,62,20,{fill:'#F1F4F8',bold:true,align:'center',geometry:'roundRect'});
arrow(s,'user-via-link',1118,982,38);
shape(s,'via-role','VIA\n사용자 대화와 인터페이스',1164,950,336,91,20,{fill:C.sage,bold:true,align:'center',geometry:'roundRect'});
arrow(s,'via-agent-link',1509,982,38);
shape(s,'agent-role','삼성 및 외부 Agent\n실제 업무 수행',1555,950,291,91,19,{fill:'#EDF1F6',bold:true,align:'center',geometry:'roundRect'});
s.speakerNotes.textFrame.setText(`관련 동향 편집 기준\nPC 업무 수행 Agent의 발전과 음성 사용자 접점 경쟁, 삼성 VIA의 역할을 연결하는 구성. 삼성 및 외부 Downstream Agent의 업무 실행 능력 활용. VIA의 책임은 사용자와의 대화 및 인터페이스, 대화와 업무의 연결.\n\n시장 근거\nMordor Intelligence, Voice Assistant Application Market. 2026 추정 USD 9.02 billion, 2031 전망 USD 18.36 billion, CAGR 15.27%. 2026-08-05 갱신. 전 산업의 음성 비서 애플리케이션 소프트웨어와 관련 서비스 기준. 하드웨어 제외. PC 전용 시장이나 삼성 매출 전망으로 해석 불가.\nhttps://www.mordorintelligence.com/industry-reports/voice-assistant-application-market\n\n기업 사례 출처\nGoogle Gemini Live: 2025-08-20. 카메라/화면과 음성 대화, Calendar/Keep/Tasks 연동. 기능과 지역별 제공 차이 존재.\nhttps://blog.google/products-and-platforms/products/gemini/gemini-live-updates-august-2025/\nAmazon Alexa+: 2026-09-01 갱신. 미국 제공, 자연어 대화와 개인화된 서비스 실행.\nhttps://www.aboutamazon.com/news/devices/alexa-plus-available-free-prime-members-us\nApple Siri AI: 2026-09-14. 개인 맥락, 화면 이해와 앱 동작. 출시 시 영어 베타, 지역/언어별 단계적 제공.\nhttps://www.apple.com/newsroom/2026/09/siri-ai-a-profoundly-more-capable-and-personal-assistant-is-here/\nOpenAI GPT-6 Astra: 기존 v8의 공개 시연 GIF 보존. 공개 시연상의 음성 수정과 후속 작업 연결. 일반 서비스의 전 지역 상용 제공 주장 아님.\nhttps://www.linkedin.com/posts/this-gpt-6-astra-ugcPost-7501364077496221696-H8XE\n\nGIF 출처\nAnthropic Claude Cowork built-in browser 공개 시연. 2026-08-26. 기존 GIF 바이트 보존.\nhttps://claude.com/resources/articles/cowork-built-in-browser\nhttps://www.youtube.com/watch?v=63GVebZvqok\nOpenAI 시연: v8의 128프레임 GIF 보존. 원본 영상 17~41초 발췌, 1.5배속 무음.\n\nCTO 메시지 출처와 구분\n사용자가 제공한 윤장현 사장(CTO), 2026.06 발언 문구를 그대로 인용. 공개 검증된 기사 발언으로 표현하지 않음. 직책 확인 보조 자료는 삼성 뉴스룸의 2025-11-21 사장단 인사이며, 해당 기사에 이 발언은 없음.\nhttps://news.samsung.com/kr/473579-2\n자료 확인일: 2026-10-14`);
// Necessity: three conceptual illustrations without task examples.
const n=p.slides.add();n.background.fill='#FFFFFF';
const cols=[
 {x:60,title:'자연스러운 대화 경험 확보',art:'continuity.png',a:'사용자 입력과 업무 응답의\n연속적 처리',b:'대화와 업무의 관계 유지',quality:'응답 지연 최소화'},
 {x:670,title:'사용자 맥락의 정확한 이해',art:'context.png',a:'음성과 화면 정보의 결합',b:'과거 대화와 업무 이력의 활용',quality:'사용자 의도와 지칭의 정확성 확보'},
 {x:1280,title:'지속적인 서비스 제공과 확장',art:'efficiency.png',a:'모델 비용과\nPC 메모리 부담 최소화',b:'모델 및 Agent 변경에 대한\n수정 범위 최소화',quality:'운영 부담과 변경 비용 절감'},
];
for(const [i,c]of cols.entries()){
 shape(n,`need-panel-${i}`,'',c.x,187,580,805,22,{fill:'#FFFFFF',stroke:C.border,geometry:'roundRect'});
 band(n,`need-heading-${i}`,c.title,c.x+10,153,560,24);
 await pic(n,`need-art-${i}`,c.art,c.x+30,268,520,440);
 tx(n,`need-copy-a-${i}`,c.a,c.x+32,703,516,91,23,{bold:true,align:'center'});
 rule(n,`need-copy-rule-${i}`,c.x+68,811,444);
 tx(n,`need-copy-b-${i}`,c.b,c.x+32,827,516,81,22,{align:'center'});
 shape(n,`need-quality-${i}`,c.quality,c.x+26,927,528,46,20,{fill:C.sage,bold:true,align:'center',geometry:'roundRect'});
}
n.speakerNotes.textFrame.setText(`과제 필요성\n사용자가 요청한 담백한 명사형 문구와 개념 그림 중심 구성. 구체적인 요청/업무 시나리오나 대화 예시 없음.\n\n첫 영역: 끼어들기와 정정 발화, Downstream Agent의 질문 및 결과가 겹쳐도 대화 지속. 대화/요청/질문/업무/실행 관계 유지. FR1과 FR4, 현재 DP44 및 DP42의 문제 배경과 연결. 신속한 응답과 대화 연속성의 필요성 제시.\n둘째 영역: 음성 지칭과 당시 화면 조작 정보 결합, 과거 대화와 업무 기록의 관련 맥락 활용. FR2와 FR3, DP47 및 DP45의 문제 배경과 연결. 사용자 의도와 지칭의 정확성 확보 필요.\n셋째 영역: 모델 사용 비용과 PC 메모리 부담, 모델 및 Agent 변경에 따른 수정 범위 관리. 현재 QA 논의의 운영 비용, 메모리, 변경 용이성 관심사 소개. 특정 수치 목표, 구조의 우월성, QA 확정 또는 설계 대안 선정 주장 없음.\n\nFR 기준: docs/presentations_files/functional-requirements-and-constraints.md\nQA 논의: docs/architecture/12-decisions/decision-packages/03-03-qa-dp-evaluation-plan.md\n그림: OpenAI imagegen 생성 개념 일러스트, 2026-10-14. 실제 제품 화면이나 성능 근거 아님.`);
await(await PresentationFile.exportPptx(p)).save(path.join(build,'authored.pptx'));
await fs.mkdir(path.join(build,'draft-renders'),{recursive:true});
for(const [i,slide]of p.slides.items.entries()){
 const img=await p.export({slide,format:'png',scale:1});await fs.writeFile(path.join(build,'draft-renders',`${i+1}.png`),new Uint8Array(await img.arrayBuffer()));
}
console.log('Authored 2 slides');
