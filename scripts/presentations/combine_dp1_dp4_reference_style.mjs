import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {Presentation, PresentationFile} from '@oai/artifact-tool';

// Reuse the exact native-object authoring sources of the four approved local decks.
// Only the explicitly listed style/terminology changes below are applied.
const BUILD=process.argv[2];
if(!BUILD || !path.isAbsolute(BUILD)) throw new Error('Pass an absolute private build directory');
const ROOT='/Users/changbae/Workspace/via-sw-architecture';
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1015.12040/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const FONT='Apple SD Gothic Neo',W=1920,H=1080;
const C={black:'#111111',gray:'#98999B',grid:'#B7B9B9',green:'#C9E1B3',greenD:'#70A843',cream:'#FFF6DD',blue:'#4676CB',cyan:'#35A8CD',pale:'#F1F6FC',red:'#C00000',muted:'#50555B',white:'#FFFFFF'};
const pres=Presentation.create({slideSize:{width:W,height:H}});
const SOURCES=[
 {dp:1,source:'generate_dp1_reference_style_v3.mjs',pptx:'dp1-reference-style-20261019/VIA-DP1-reference-style-v3.pptx',title:'현재 요청의 근거 공급 방안'},
 {dp:2,source:'generate_dp2_reference_style_v2.mjs',pptx:'dp2-reference-style-20261019/VIA-DP2-reference-style-v2.pptx',title:'후속 입력의 의미 판단 실행 방안'},
 {dp:3,source:'generate_dp3_reference_style.mjs',pptx:'dp3-reference-style-20261019/VIA-DP3-reference-style.pptx',title:'화면과 발화의 지칭 결합 방안'},
 {dp:4,source:'generate_dp4_reference_style.mjs',pptx:'dp4-reference-style-20261019/VIA-DP4-reference-style.pptx',title:'Agent 간 업무 인계 방안'},
];
const LABELS=new Map([
 ['QA03 처리 (초)','QA03 VIA 처리 (초)'],
 ['QA03 처리시간','QA03 VIA 처리'],
 ['QA06 최고 RSS (MB)','QA06 RSS (MB)'],
 ['QA06 최고 동시 RSS (MB)','QA06 RSS (MB)'],
 ['유지 데이터','상태 / 자료'],
 ['보존 데이터','상태 / 자료'],
 ['보존 상태','상태 / 자료'],
]);
function normalizeLabel(value,dp){
 if(typeof value!=='string')return value;
 return (LABELS.get(value)??value)
  .replaceAll('시간 괄호: 관측·예정','시간 괄호: 관측/예정')
  .replaceAll('4장',`${dp*4}쪽`)
  .replace('모델 비용 $0.01466 → $0.01574','모델 비용 $0.0146552 → $0.0157364');
}
function text(s,t,x,y,w,h,size,opt={}){
 const q=s.shapes.add({name:opt.name??t,geometry:'textbox',position:{left:x,top:y,width:w,height:h},fill:'none',line:{fill:'none',width:0}});
 q.text=t;q.text.style={typeface:FONT,fontSize:size,color:opt.color??C.black,bold:opt.bold??false,alignment:opt.align??'left',verticalAlignment:'middle',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
}
function unifiedHeader(s,dp,title,suffix,page){
 s.background.fill=C.white;
 text(s,`DP${dp}. ${title}${suffix}`,60,29,1200,68,48,{bold:true,name:'Unified slide title'});
 ['과제 소개','요구사항 분석','설계','구현 및 검증','결론'].forEach((v,i)=>{
  const x=1210+i*134;
  s.shapes.add({name:'Workflow '+v,geometry:'chevron',position:{left:x,top:33,width:143,height:44},fill:i===2?C.cyan:C.gray,line:{fill:C.white,width:2,style:'solid'}});
  text(s,v,x+14,33,111,44,19,{bold:true,color:C.white,align:'center'});
 });
 s.shapes.add({name:'Header rule',geometry:'custom',position:{left:58,top:108,width:1802,height:0.1},fill:'none',line:{fill:'#A7A7A7',width:3,style:'solid'},customPaths:[{width:1802,height:0.1,commands:[{moveTo:{x:0,y:0}},{lineTo:{x:1802,y:0}}]}]});
 text(s,`${(dp-1)*4+page} / 16`,1836,1052,62,21,16,{align:'right',color:C.muted,name:'Continuous slide number'});
}
const COMMON_NOTES='\n통합 발표 용어: QA01 첫 응답 지연시간(관측 callback 평균), QA02 지칭 집합 정확도(정답/예정), QA03 VIA 처리시간, QA04 변경 Rust 모듈 수(별도 변경 과제 평균), QA05 해당 실험 전체 모델 비용(고정 단가 USD 합계), QA06 로컬 Python+Rust 최고 동시 RSS(MB). 시간의 괄호는 관측/예정이다. QA05의 미확정 사용량은 해당 장에서 별도 공개한다. N/A, 미측정, 미실행, 미관측, 보류와 우열 미확정은 서로 다른 상태다.\n발표 DP1/2/3/4는 저장소01/02/04/05에 대응한다. DP별 실험 모집단이 달라 DP 간 초 단위 순위나 절감량 합계를 만들지 않는다. DP1-A2, DP2-B1, DP3-A2+A5, DP4-C1은 각각 별개 보완이다. 네 선택과 모든 보완을 합친 실행 구성은 미실행이며 이번 통합은 자료 편집이다.';

await fs.mkdir(path.join(BUILD,'modules'),{recursive:true});
await fs.mkdir(path.join(BUILD,'previews'),{recursive:true});
await fs.mkdir(path.join(BUILD,'output'),{recursive:true});
const inputRecords=[];
for(const spec of SOURCES){
 const sourcePath=path.join(ROOT,'scripts/presentations',spec.source);
 const pptxPath=path.join(ROOT,'docs/presentations_files',spec.pptx);
 const source=await fs.readFile(sourcePath,'utf8');
 const sourcePptx=await fs.readFile(pptxPath);
 inputRecords.push({...spec,sourcePath,pptxPath,sourceSha256:createHash('sha256').update(source).digest('hex'),pptxSha256:createHash('sha256').update(sourcePptx).digest('hex')});
 let body=source.slice(source.indexOf('const line='),source.indexOf("await fs.mkdir(path.join(BUILD,'previews')"));
 if(!body.startsWith('const line=') || !body.includes('function header('))throw new Error('Source boundary changed: '+spec.source);
 body=body.replace(/function header\(s,suffix,page\)\{[\s\S]*?\n\}\n(?=function chip)/,
  `function header(s,suffix,page){\n comparisonOffset=${spec.dp===1?"(suffix===' (설계 방안 비교)'||suffix===' (선택 및 보완)')?-10:0":'0'};\n unifiedHeader(s,${spec.dp},${JSON.stringify(spec.title)},suffix,page);\n}\n`);
 // DP1 comparison body moves as a unit by 10 px to the shared table origin.
 body=body.replaceAll('top:y,','top:alignY(y),').replaceAll('top:ay,','top:alignY(ay),');
 if(spec.dp===1)body=body.replace("txt(s,'✓',202,160,","txt(s,'✓',202,150,");
 body=body.replace('q.text=t;q.text.style=','q.text=normalizeLabel(t);q.text.style=');
 body=body.replace('columnWidths:widths,values});','columnWidths:widths,values:values.map(row=>row.map(normalizeLabel))});');
 body=body.replaceAll("fontSize:19,bold:true,alignment:'left'","fontSize:18,bold:true,alignment:'left'");
 // Restore source precision rather than zero-padding rounded DP1 costs.
 if(spec.dp===1)body=body.replace("'0.06750','0.06992','0.16308'","'0.0674956','0.0699160','0.1630788'");
 const moduleText=`export function build(pres,common){\nconst {ROOT,FONT,W,H,C,normalizeLabel,unifiedHeader}=common;\nlet comparisonOffset=0;\nconst alignY=y=>comparisonOffset && y>=174 && y<991?y+comparisonOffset:y;\n${body}\n}\n`;
 const modulePath=path.join(BUILD,'modules',`dp${spec.dp}.mjs`);
 await fs.writeFile(modulePath,moduleText);
 const before=pres.slides.items.length;
 const mod=await import(pathToFileURL(modulePath).href);
 mod.build(pres,{ROOT,FONT,W,H,C,normalizeLabel:value=>normalizeLabel(value,spec.dp),unifiedHeader});
 if(pres.slides.items.length-before!==4)throw new Error('Expected four slides per DP');
 for(const slide of pres.slides.items.slice(before)){
  slide.speakerNotes.text=slide.speakerNotes.text.replaceAll('4장',`${spec.dp*4}쪽`);
  slide.speakerNotes.append(COMMON_NOTES);
 }
}
if(pres.slides.items.length!==16)throw new Error('Expected exactly 16 slides');
await fs.writeFile(path.join(BUILD,'source-manifest.json'),JSON.stringify({inputs:inputRecords,normalizations:{labels:Object.fromEntries(LABELS),dp1TableOffsetPx:-10,qaRowLabelFontPx:18,currencyPrecision:'DP1 original values restored to 7 decimals',pageNumbers:'1 / 16 through 16 / 16',detailCrossReferences:'4, 8, 12, 16쪽 in visible text and speaker notes',font:FONT,palette:C}},null,2));
for(const [i,slide] of pres.slides.items.entries()){
 const stem='slide-'+String(i+1).padStart(2,'0');
 const png=await pres.export({slide,format:'png',scale:1});
 await fs.writeFile(path.join(BUILD,'previews',stem+'.png'),new Uint8Array(await png.arrayBuffer()));
 const layout=await slide.export({format:'layout'});
 await fs.writeFile(path.join(BUILD,'previews',stem+'.layout.json'),await layout.text());
 console.log('Rendered '+(i+1)+'/16');
}
const draft=path.join(BUILD,'candidate.pptx');
await(await PresentationFile.exportPptx(pres)).save(draft);
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')).href);
const owners=[2,3,4,6,7,8,9,10,11,12,13,14,15,16];
const result=await finalizePresentation({workspaceDir:BUILD,candidatePath:draft,finalPath:path.join(BUILD,'output','VIA-DP1-DP4-reference-style-16slides.pptx'),pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),explicitTotalSlideCount:16,requiredNativeTableOwnerSlides:owners,fontPolicy:{basis:'reference',families:[FONT],referencePath:inputRecords[0].pptxPath,referenceSha256:inputRecords[0].pptxSha256},verifyArtifactToolImport:true,layoutArgs:['--expected-slide-size-emu',String(W*9525)+','+String(H*9525),'--validate-bullet-geometry','--validate-heading-fit',...owners.flatMap(n=>['--require-native-table-slide',String(n)])],receiptPath:path.join(BUILD,'validation.json')});
console.log(JSON.stringify({output:result.finalPath,sha256:result.finalSha256,layoutFindings:result.presentationLayout.finding_count,layoutWarnings:result.presentationLayout.warnings},null,2));
