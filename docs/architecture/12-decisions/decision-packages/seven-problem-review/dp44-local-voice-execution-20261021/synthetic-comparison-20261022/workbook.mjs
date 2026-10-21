import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
const runtime=process.env.VIA_ARTIFACT_MODULE || '/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/@oai/artifact-tool/dist/artifact_tool.mjs';
const {Workbook,SpreadsheetFile}=await import(pathToFileURL(runtime).href);
const here=path.dirname(fileURLToPath(import.meta.url));
let root=here;while(true){try{await fs.access(path.join(root,'INTENT.md'));break;}catch{root=path.dirname(root);}}
const out=path.join(root,'docs/presentations_files/dp44-synthetic-backdata-20261022');
const output=path.join(out,'outputs/01a154de');
const d=JSON.parse(await fs.readFile(path.join(out,'synthetic-data.json'),'utf8'));
const w=Workbook.create();
const names=['Summary','Rates','Responses','Goals','Stages','Usage','Memory','Changes','Cases'];
const sheets=Object.fromEntries(names.map(n=>[n,w.worksheets.add(n)]));
const col=n=>{let s='';for(n++;n;n=Math.floor((n-1)/26))s=String.fromCharCode(65+(n-1)%26)+s;return s;};
function tab(name,title,headers,rows){const s=sheets[name],last=col(headers.length-1),titleLast=col(Math.min(headers.length,6)-1),end=rows.length+3;
  s.showGridLines=false;s.getRange(`A1:${last}${end}`).format.font={name:'Arial',size:11,color:'#17212B'};
  s.getRange(`A1:${titleLast}1`).merge();s.getRange('A1').values=[[title]];s.getRange(`A1:${last}1`).format={fill:'#17324D',font:{bold:true,color:'#FFFFFF',size:16},rowHeight:32};
  s.getRange(`A2:${titleLast}2`).merge();s.getRange('A2').values=[['작성 가정 기반 합성 자료. 원본: synthetic-data.json. 실제 모델 실행 및 제품 시험 아님.']];
  s.getRange(`A2:${last}2`).format={font:{color:'#536779',size:10},rowHeight:28};
  s.getRange(`A3:${last}3`).values=[headers];s.getRange(`A3:${last}3`).format={fill:'#DCE7F1',font:{bold:true},wrapText:true,rowHeight:34};
  if(rows.length)s.getRange(`A4:${last}${end}`).values=rows;
  s.getRange(`A3:${last}${end}`).format.verticalAlignment='center';s.getRange(`A4:${last}${end}`).format.rowHeight=24;
  s.getRange(`A:${last}`).format.columnWidth=19;s.getRange(`A4:${last}${end}`).setNumberFormat('0.000');
  s.freezePanes.freezeRows(3);s.tables.add(`A3:${last}${end}`,true,'Table'+name);
  return s;}
tab('Rates','모델 비용의 가상 요율',['항목','USD/단위','단위','출처'],[
 ['text input',1e-6,'token','공통 QA v2 가상 요율'],['text output',4e-6,'token','공통 QA v2 가상 요율'],
 ['audio input',.001,'second','공통 QA v2 가상 요율'],['audio output',.002,'second','공통 QA v2 가상 요율']]);
sheets.Rates.getRange('B4:B7').setNumberFormat('0.000000');sheets.Rates.getRange('B4:B7').format.font.color='#245FA6';
tab('Responses','첫 응답과 고정 의미 판정',['사례','안','화면 초','음성 초','QA02 성공','QA02 표현','QA07 성공','입력 ID','첫 화면 문구','첫 음성 문구'],
 d.responses.map(x=>[x.episode,x.option,x.first_ui_s,x.first_voice_s,x.qa02_correct,x.qa02_n,x.qa07_correct,x.input_id,x.first_ui_text,x.first_voice_text]));
sheets.Responses.getRange('E4:G163').setNumberFormat('0');sheets.Responses.getRange('I:J').format.columnWidth=55;
tab('Goals','원래 목표별 VIA 인과 처리시간',['목표 ID','안','시작 초','QA03 초','terminal node IDs','완료 범위'],d.goals.map(x=>[x.id,x.option,x.start_s,x.qa03_s,x.terminals.join('\n'),x.description]));
sheets.Goals.getRange('A:A').format.columnWidth=32;sheets.Goals.getRange('E:F').format.columnWidth=55;
tab('Stages','선행 관계와 실제 경과시간 가정',['단계 ID','안','단계명','시작 초','종료 초','duration 초','VIA 포함','선행 단계'],d.nodes.map(x=>[x.id,x.option,x.name,x.start_s,x.end_s,x.duration_s,x.via?1:0,x.parents.join('\n')]));
sheets.Stages.getRange('A:A').format.columnWidth=45;sheets.Stages.getRange('C:C').format.columnWidth=27;sheets.Stages.getRange('H:H').format.columnWidth=60;
tab('Usage','준비와 폐기를 포함한 모델 비용',['사례','안','역할','단계','text 입력','text 출력','음성 입력 초','음성 출력 초','비용 USD'],d.usage.map(x=>[x.episode,x.option,x.role,x.phase,x.text_input,x.text_output,x.audio_input_s,x.audio_output_s,null]));
sheets.Usage.getRange(`I4:I${d.usage.length+3}`).formulas=d.usage.map((_,i)=>[`=E${i+4}*Rates!$B$4+F${i+4}*Rates!$B$5+G${i+4}*Rates!$B$6+H${i+4}*Rates!$B$7`]);
sheets.Usage.getRange(`I4:I${d.usage.length+3}`).setNumberFormat('0.000000');
tab('Memory','같은 시각의 로컬 RAM 합계',['사례','안','시각 초','공통 MB','audio MB','metadata MB','합계 MB'],d.memory.map(x=>[x.episode,x.option,x.at_s,x.base_mb,x.audio_mb,x.metadata_mb,null]));
sheets.Memory.getRange(`G4:G${d.memory.length+3}`).formulas=d.memory.map((_,i)=>[`=SUM(D${i+4}:F${i+4})`]);
tab('Changes','공통 C01~C06 예상 변경 모듈',['과제','안','모듈 수','모듈 1','모듈 2','모듈 3','모듈 4','모듈 5','변경 이유'],d.changes.map(x=>[x.id,x.option,null,...[0,1,2,3,4].map(i=>x.modules[i]||null),x.reason]));
sheets.Changes.getRange('C4:C15').formulas=d.changes.map((_,i)=>[`=COUNTA(D${i+4}:H${i+4})`]);sheets.Changes.getRange('C4:C15').setNumberFormat('0');sheets.Changes.getRange('D:H').format.columnWidth=36;sheets.Changes.getRange('I:I').format.columnWidth=70;
tab('Cases','평가 발화와 정답',['사례','사례군','사용자 발화','시작 초','종료 초','정답 대상과 의미','변형 설명'],d.inputs.map(x=>[x.episode,x.family,x.text,x.acoustic_start_s,x.acoustic_end_s,
  '작업: '+x.oracle_meaning.operation+'\n대상: '+x.oracle_meaning.targets.join(', ')+'\n조건: '+Object.entries(x.oracle_meaning.conditions).map(([k,v])=>k+'='+v).join(', ')+'\n무관한 업무 유지: 예\n메일 발송 권한: 없음',
  '같은 발화와 의미를 유지한 작성 변형\n음성 길이와 입력 token 수만 소폭 변경\n독립 모델 실행 또는 실측 반복이 아님']));
sheets.Cases.getRange('C:C').format.columnWidth=55;sheets.Cases.getRange('F:G').format.columnWidth=65;sheets.Cases.getRange('C4:C83').format.wrapText=true;sheets.Cases.getRange('A4:G83').format.rowHeight=60;
for(const [name,cols,height] of [['Responses','I:J',52],['Goals','E:F',112],['Stages','A:H',72],['Changes','D:I',90],['Cases','C:G',136]]){
  sheets[name].getRange(cols).format.wrapText=true;
  sheets[name].getUsedRange().format.wrapText=true;
  sheets[name].getUsedRange().format.rowHeight=height;
  sheets[name].getRange('1:1').format.rowHeight=32;sheets[name].getRange('2:2').format.rowHeight=28;sheets[name].getRange('3:3').format.rowHeight=34;
}
sheets.Stages.getRange(`G4:G${d.nodes.length+3}`).setNumberFormat('0');sheets.Usage.getRange(`E4:F${d.usage.length+3}`).setNumberFormat('0');sheets.Memory.getRange(`C4:C${d.memory.length+3}`).setNumberFormat('0.0000000');
const last={R:d.responses.length+3,G:d.goals.length+3,U:d.usage.length+3,M:d.memory.length+3};
const specs=[['QA01 화면','초','AVERAGEIF',`Responses!$C$4:$C$${last.R}`,'동일 UI 경로'],['QA01 음성','초','AVERAGEIF',`Responses!$D$4:$D$${last.R}`,'실제 출력에 대응하는 가상 사건'],['QA02 지칭 대상','비율','accuracy','','고정 DP2 fixture 120/120'],['QA03 VIA 처리','초','goal','','90목표, 외부 업무와 재생 제외'],['QA04 변경 범위','모듈','change','','실제 Rust diff 아님'],['QA05 모델 비용','USD','cost','','80평가와 80준비, 폐기 포함'],['QA06 로컬 RAM','MB','memory','','동일 시각 peak'],['QA07 요청 의미','비율','meaning','','고정 DP2 fixture 80/80']];
tab('Summary','DP44 A/B 가상 QA 비교',['지표','단위','A','B','B - A','해석'],specs.map(x=>[x[0],x[1],null,null,null,x[4]]));
sheets.Summary.getRange('A:A').format.columnWidth=25;sheets.Summary.getRange('F:F').format.columnWidth=47;
for(let i=0;i<specs.length;i++)for(let oi=0;oi<2;oi++){const option=oi?'B':'A',key=specs[i][2];let f;
 const cond=`Responses!$B$4:$B$${last.R},"${option}"`;
 if(key==='AVERAGEIF')f=`=AVERAGEIF(${cond},${specs[i][3]})`;
 if(key==='accuracy')f=`=SUMIF(${cond},Responses!$E$4:$E$${last.R})/SUMIF(${cond},Responses!$F$4:$F$${last.R})`;
 if(key==='meaning')f=`=AVERAGEIF(${cond},Responses!$G$4:$G$${last.R})`;
 if(key==='goal')f=`=AVERAGEIF(Goals!$B$4:$B$${last.G},"${option}",Goals!$D$4:$D$${last.G})`;
 if(key==='change')f=`=AVERAGEIF(Changes!$B$4:$B$15,"${option}",Changes!$C$4:$C$15)`;
 if(key==='cost')f=`=SUMIF(Usage!$B$4:$B$${last.U},"${option}",Usage!$I$4:$I$${last.U})`;
 if(key==='memory')f=`=_xlfn.MAXIFS(Memory!$G$4:$G$${last.M},Memory!$B$4:$B$${last.M},"${option}")`;
 sheets.Summary.getCell(i+3,oi+2).formulas=[[f]];
}
sheets.Summary.getRange('E4:E11').formulas=specs.map((_,i)=>[`=D${i+4}-C${i+4}`]);
for(const row of [6,11])sheets.Summary.getRange(`C${row}:E${row}`).setNumberFormat('0.00%');
sheets.Summary.getRange('C9:E9').setNumberFormat('0.000000');
sheets.Summary.getRange('A14:F14').merge();sheets.Summary.getRange('A14').values=[['조건부 권고: A. 음성 시작과 QA03 이익을 얻고, 준비 폐기 비용과 변경 범위를 감수한다.']];sheets.Summary.getRange('A14:F14').format={wrapText:true,rowHeight:42,font:{bold:true},fill:'#EDF4F9'};
sheets.Summary.getRange('A16:F17').merge();sheets.Summary.getRange('A16').values=[['QA01 화면과 음성은 별도 종료점이다. 정확도는 구조 비교를 위한 공통 정답 fixture다. 표는 합성 가정의 계산이며 실제 모델 정확도나 제품 성능이 아니다.']];sheets.Summary.getRange('A16:F17').format={wrapText:true,rowHeight:24};
w.recalculate();
const keys=['qa01_display_s','qa01_voice_s','qa02','qa03_s','qa04_modules','qa05_usd','qa06_mb','qa07'];
const vals=sheets.Summary.getRange('C4:D11').values;for(let i=0;i<8;i++)for(let oi=0;oi<2;oi++){const expected=d.summary[oi?'B':'A'][keys[i]];if(Math.abs(vals[i][oi]-expected)>1e-6)throw Error(`Mismatch ${keys[i]} ${vals[i][oi]} ${expected}`);}
// Test that usage and summary respond to the quoted hypothetical audio rate; restore before export.
const base=vals[5][0];sheets.Rates.getRange('B7').values=[[.004]];w.recalculate();
const doubled=sheets.Summary.getRange('C9').values[0][0];if(Math.abs(doubled-base-d.summary.A.audio_output_s*.002)>1e-6)throw Error('Rate recalculation failed');
sheets.Rates.getRange('B7').values=[[.002]];w.recalculate();
const inspect=await w.inspect({kind:'table',range:'Summary!A3:F11',include:'values,formulas',tableMaxRows:9,tableMaxCols:6,maxChars:2500});
const errors=await w.inspect({kind:'match',searchTerm:'#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A|#NUM!|#NULL!|#SPILL!|#CALC!',options:{useRegex:true,maxResults:20},maxChars:2000});
await fs.mkdir(path.join(output,'previews'),{recursive:true});
for(const name of names){const range=name==='Summary'?'A1:F17':name==='Rates'?'A1:D7':name==='Cases'?'A1:G6':name==='Changes'?'A1:I6':name==='Responses'?'A1:J6':name==='Stages'?'A1:H6':'A1:G7';
 const preview=await w.render({sheetName:name,range,scale:1,format:'png'});await fs.writeFile(path.join(output,'previews',name+'.png'),new Uint8Array(await preview.arrayBuffer()));}
const book=await SpreadsheetFile.exportXlsx(w);await book.save(path.join(output,'DP44-synthetic-backdata.xlsx'));
await fs.writeFile(path.join(out,'workbook-validation.json'),JSON.stringify({status:'PASS',summary:vals,rate_recalculation_verified:true,formula_error_scan:errors.ndjson,engine:'artifact-tool',native_windows_excel_tested:false,rendered_sheets:names},null,2));
console.log(inspect.ndjson);console.log(errors.ndjson);console.log('Workbook exported');
