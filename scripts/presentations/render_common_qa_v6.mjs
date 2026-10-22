import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const parent=path.join(ROOT,'docs/architecture/12-decisions/decision-packages/current/evaluation');
const base=path.join(parent,'common-qa-v5'),dir=path.join(parent,'common-qa-v6');
const out=path.join(dir,'artifacts/outputs/01a15dfe-v6'),pre=path.join(dir,'artifacts/previews');
const bytes=await fs.readFile(path.join(dir,'artifacts/ledger.json')),d=JSON.parse(bytes),q=d.qa07;
const req=createRequire(path.join(process.env.VIA_ARTIFACT_MODULE_BASE??'/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node','entry.cjs'));
const {FileBlob,SpreadsheetFile}=await import(pathToFileURL(req.resolve('@oai/artifact-tool')));
const wb=await SpreadsheetFile.importXlsx(await FileBlob.load(path.join(base,'artifacts/outputs/01a15dfe-v5/VIA-QA-common-v5.xlsx')));
await fs.mkdir(out,{recursive:true});await fs.mkdir(pre,{recursive:true});
// Read-only visual baseline is saved before edits; the original workbook is never overwritten.
const before=await wb.render({sheetName:'전체비교',range:'A18:I33',scale:1,format:'png'});
await fs.writeFile(path.join(pre,'source-v5-summary.png'),new Uint8Array(await before.arrayBuffer()));
const summary=wb.worksheets.getItem('전체비교'),meaning=wb.worksheets.getItem('의미판정');
const cases=wb.worksheets.add('QA07사례'),obs=wb.worksheets.add('QA07의무');
const col=i=>String.fromCharCode(65+i);
const styles=(sheet,end,widths)=>{sheet.showGridLines=false;sheet.getRange(`A1:${col(widths.length-1)}${end}`).format={font:{name:'Arial',size:11,color:'#23344B'},verticalAlignment:'center',rowHeight:34};widths.forEach((w,i)=>sheet.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidth=w);};
const header=(sheet,row,values)=>{sheet.getRange(`A${row}:${col(values.length-1)}${row}`).values=[values];sheet.getRange(`A${row}:${col(values.length-1)}${row}`).format={fill:'#253C59',font:{name:'Arial',size:11,color:'#FFFFFF',bold:true},rowHeight:44,wrapText:true,horizontalAlignment:'center',verticalAlignment:'center'};};
const title=(sheet,text,note)=>{sheet.getRange('A2').values=[[text]];sheet.getRange('A2').format.font={name:'Arial',size:16,bold:true};sheet.getRange('A3').values=[[note]];sheet.getRange('A3').format.rowHeight=28;};
const rows=[],ranges=new Map();
for(const cfg of d.configs)for(const c of q.manifest.cases){const a=rows.length+6;for(const o of c.obligations)rows.push([cfg.id,c.case_id,o.id,o.stage,o.requirement,JSON.stringify(o.expected_meaning??{channels:o.channels??[],event:o.event_ref??null,revision:o.revision}), '', '', '', '', '']);ranges.set(cfg.id+'|'+c.case_id,[a,rows.length+5]);}
styles(obs,rows.length+5,[17,14,27,29,69,62,14,32,27,29,14]);
title(obs,'QA07 의무별 증거와 판정','노란 칸: 최종 조정 판정, 증거, 평가자, 공유 오류 ID. 최초 FAIL은 복구 후에도 유지. 원시 이력은 별도 보존.');
header(obs,5,['구성','사례','의무 ID','판정 단계','충족해야 할 내용','의미/채널/사건 기준','판정 입력','증거 참조','평가자/조정 참조','원인/QA02 공유 ID','증거 기반 상태']);
obs.getRange(`A6:K${rows.length+5}`).values=rows;
obs.getRange(`A6:K${rows.length+5}`).format.wrapText=true;obs.getRange(`A6:K${rows.length+5}`).format.rowHeight=88;
obs.getRange(`G6:J${rows.length+5}`).format.fill='#FFF0CB';
obs.getRange(`G6:G${rows.length+5}`).dataValidation={rule:{type:'list',values:['PASS','FAIL','UNKNOWN']}};
obs.getRange(`K6:K${rows.length+5}`).formulas=rows.map((_,i)=>{const r=i+6;return [`=IF(AND(OR(G${r}="PASS",G${r}="FAIL"),H${r}<>"",I${r}<>""),G${r},"UNKNOWN")`];});
obs.freezePanes.freezeRows(5);obs.freezePanes.freezeColumns(2);
styles(cases,184,[17,15,14,18,12,12,14,16,30,17,40,16,15,28]);
title(cases,'QA07 사례별 전체 충족','24개는 검토용 묶음이며 반복/분모 미동결. 실제 실행 0건. PASS/FAIL/UNKNOWN은 작성 슬롯의 상태다.');
header(cases,5,['구성','작성 슬롯','성공 S','실패 F','미판정 U','공통 무효','확정 N','QA07 %','하한 %','상한 %','점수','모집단 동결']);
header(cases,16,['구성','사례','원전 입력','역할','충족 의무','위반 의무','미판정 의무','사전 계약 완비','실행 증거 키','반복 ID','관측 종료 증거','사례 상태','키 중복 수','공통 무효 근거']);
const caseRows=[];
for(const cfg of d.configs)for(const c of q.manifest.cases)caseRows.push([cfg.id,c.case_id,c.source_case_id,c.routing,0,0,0,c.contract_ready?'YES':'NO','','','','',0,'']);
cases.getRange('A17:N184').values=caseRows;cases.getRange('A17:N184').format.rowHeight=40;cases.getRange('A17:N184').format.wrapText=true;
cases.getRange('I17:K184').format.fill='#FFF0CB';
cases.getRange('P5:Q5').values=[['사례','공통 자료 무효 근거']];
cases.getRange('P5:Q5').format={fill:'#253C59',font:{name:'Arial',size:11,color:'#FFFFFF',bold:true},rowHeight:44,horizontalAlignment:'center',verticalAlignment:'center'};
cases.getRange('P6:Q29').values=q.manifest.cases.map(c=>[c.case_id,c.common_invalid_evidence??'']);
cases.getRange('P1:P29').format.columnWidth=18;cases.getRange('Q1:Q29').format.columnWidth=58;
cases.getRange('P6:Q29').format.font={name:'Arial',size:11,color:'#23344B'};
for(let i=0;i<caseRows.length;i++){const r=i+17,[a,b]=ranges.get(caseRows[i][0]+'|'+caseRows[i][1]);
 cases.getRange(`N${r}`).formulas=[[`=IF(Q${6+i%24}="","",Q${6+i%24})`]];
 cases.getRange(`E${r}:G${r}`).formulas=[['PASS','FAIL','UNKNOWN'].map(s=>`=COUNTIF('QA07의무'!K${a}:K${b},"${s}")`)];
 cases.getRange(`M${r}`).formulas=[[`=IF(I${r}="",0,COUNTIFS($A$17:$A$184,A${r},$I$17:$I$184,I${r}))`]];
 cases.getRange(`L${r}`).formulas=[[`=IF(N${r}<>"","INVALID",IF(F${r}>0,"FAIL",IF(AND(G${r}=0,H${r}="YES",I${r}<>"",J${r}<>"",K${r}<>"",M${r}=1),"PASS","UNKNOWN")))`]];
}
d.configs.forEach((cfg,i)=>{const r=i+6,a=17+i*24,b=a+23;cases.getRange(`A${r}:B${r}`).values=[[cfg.id,24]];cases.getRange(`L${r}`).values=[['NO']];cases.getRange(`C${r}:F${r}`).formulas=[['PASS','FAIL','UNKNOWN','INVALID'].map(s=>`=COUNTIF(L${a}:L${b},"${s}")`)];cases.getRange(`G${r}:K${r}`).formulas=[[
 `=IF(OR(L${r}<>"YES",COUNTIF(M${a}:M${b},">1")>0),"미산출",B${r}-F${r})`,
 `=IF(OR(G${r}="미산출",G${r}=0,E${r}>0),"미산출",100*C${r}/G${r})`,
 `=IF(OR(G${r}="미산출",G${r}=0),"미산출",100*C${r}/G${r})`,
 `=IF(OR(G${r}="미산출",G${r}=0),"미산출",100*(C${r}+E${r})/G${r})`,
 '="미정"']];});
cases.getRange('A14').values=[['사전 계약과 모집단 동결 표시는 manifest 판본의 고정 정보다. 관측 입력만으로 YES로 바꾸지 않는다.']];
cases.getRange('A15').values=[['정의: QA07-REQUEST-RESPONSE-v2. 해석, 응답 구성, 실제 전달을 모두 확인. QA01/03 시간과 별도 판정.']];
cases.getRange('G6:L12').format.horizontalAlignment='center';cases.getRange('H17:H184').format.horizontalAlignment='center';cases.getRange('L17:L184').format.horizontalAlignment='center';cases.getRange('H6:J12').setNumberFormat('0.0');cases.freezePanes.freezeRows(16);cases.freezePanes.freezeColumns(2);
const cmp=['BASE','DP2-B','BASE','DP3-B-R1','DP3-B-R2','DP3-B-R3','BASE','DP5-B','BASE','DP44-B'];
cmp.forEach((id,i)=>summary.getRange(`G${21+i}`).formulas=[[`='QA07사례'!H${6+d.configs.findIndex(x=>x.id===id)}`]]);
summary.getRange('A3').values=[['분석 모델 v6. QA07 사례/응답 의무 갱신. 시간과 자원은 v5 보존값이며 실측 아님.']];
summary.getRange('A32').values=[['QA07은 해석/구성/실제 전달 전체 충족률. QA07사례/QA07의무 시트 참조. 분모 미동결, 관측 없음.']];
meaning.getRange('A3').values=[['구형 입력 해석 부분 진단. 새 QA07 총계는 QA07의무와 QA07사례에서 별도로 집계한다.']];
// Reversible authoring tests, using no product evidence. All probe cells are restored.
const [a,b]=ranges.get('BASE|Q7-U01'),probe=(r,c,v)=>obs.getRange(`${c}${r}`).values=[[v]];
probe(a,'G','PASS');assert.equal(obs.getRange(`K${a}`).values[0][0],'UNKNOWN');
probe(a,'H','TEST_ONLY');probe(a,'I','TEST_REVIEW');assert.equal(obs.getRange(`K${a}`).values[0][0],'PASS');
assert.equal(cases.getRange('L17').values[0][0],'UNKNOWN');
probe(a,'G','FAIL');assert.equal(cases.getRange('L17').values[0][0],'FAIL');
cases.getRange('Q6').values=[['TEST_COMMON_INVALID']];for(let ci=0;ci<7;ci++)assert.equal(cases.getRange(`L${17+ci*24}`).values[0][0],'INVALID');assert.equal(cases.getRange('F17').values[0][0],1);cases.getRange('Q6').clear({applyTo:'contents'});assert.equal(cases.getRange('L17').values[0][0],'FAIL');
for(let r=a;r<=b;r++){probe(r,'G','PASS');probe(r,'H','TEST_ONLY');probe(r,'I','TEST_REVIEW');}
assert.equal(cases.getRange('L17').values[0][0],'UNKNOWN');
cases.getRange('H17:K17').values=[['YES','TEST_RUN','test-repeat','TEST_CLOSE']];assert.equal(cases.getRange('L17').values[0][0],'PASS');
cases.getRange('L6').values=[['YES']];assert.equal(cases.getRange('H6').values[0][0],'미산출');assert.equal(cases.getRange('I6').values[0][0],100/24);assert.equal(cases.getRange('J6').values[0][0],100);
cases.getRange('I18').values=[['TEST_RUN']];assert.equal(cases.getRange('L17').values[0][0],'UNKNOWN');assert.equal(cases.getRange('G6').values[0][0],'미산출');
cases.getRange('I18').clear({applyTo:'contents'});cases.getRange('L6').values=[['NO']];cases.getRange('H17').values=[['NO']];cases.getRange('I17:K17').clear({applyTo:'contents'});obs.getRange(`G${a}:J${b}`).clear({applyTo:'contents'});
wb.recalculate();
assert.equal(cases.getRange('E6').values[0][0],24);assert.equal(summary.getRange('G21').values[0][0],'미산출');
for(let i=0;i<cmp.length;i++){const s=d.summary[cmp[i]],v=summary.getRange(`C${i+6}:I${i+6}`).values[0];[s.qa01,s.qa01_score,s.wait.L,s.wait.B,s.wait.R,s.qa03,s.qa03_score].forEach((x,j)=>assert(Math.abs(v[j]-x)<1e-7));}
for(const s of [cases,obs,summary,meaning])for(const row of s.getUsedRange().values)for(const v of row)assert(!/^#(REF!|DIV\/0!|VALUE!|NAME\?|N\/A|NUM!|NULL!|SPILL!|CALC!)/.test(String(v)));
console.log((await wb.inspect({kind:'region',sheetId:'QA07사례',range:'A5:L12',maxChars:1800,tableMaxRows:8,tableMaxCols:12})).ndjson);
const book=path.join(out,'VIA-QA-common-v6.xlsx');await(await SpreadsheetFile.exportXlsx(wb)).save(book);await fs.rm(book+'.inspect.ndjson',{force:true});
const previews=[];
for(const [sheet,range,file] of [['전체비교','A18:I33','summary.png'],['QA07사례','A1:L12','qa07-summary.png'],['QA07사례','A16:N21','qa07-cases.png'],['QA07의무','A1:F10','qa07-obligations.png'],['QA07의무','G5:K10','qa07-evidence.png'],['의미판정','A1:E7','legacy-meaning.png'],['QA07사례','P5:Q10','qa07-common-invalid.png']]){const blob=await wb.render({sheetName:sheet,range,scale:1,format:'png'});await fs.writeFile(path.join(pre,file),new Uint8Array(await blob.arrayBuffer()));previews.push(file);}
const escape=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
let html=await fs.readFile(path.join(base,'artifacts/outputs/01a15dfe-v5/guide.html'),'utf8');
html=html.replaceAll('공통 QA v5','공통 QA v6').replaceAll('공통 QA 비교 v5','공통 QA 비교 v6').replaceAll('VIA-QA-common-v5.xlsx','VIA-QA-common-v6.xlsx');
html=html.replace('새 QA07 응답 적합성은 미평가이며 기존 의미 정답은 부분 진단입니다.','QA07 사례/응답 의무와 판정 원장을 추가했습니다. 실제 관측과 동결 분모가 없어 전체 충족률은 미산출입니다.');
html=html.replace(/(<script id="qa-data" type="application\/json">)[\s\S]*?(<\/script>)/,(_,a,b)=>a+JSON.stringify(d).replace(/</g,'\\u003c')+b);
const section=`<section><h2>QA07 요청 해석 및 응답 적합성</h2><p>모든 필수 해석, 응답 구성과 실제 전달이 맞아야 사례 하나가 성공입니다. 최초 오류는 복구 후에도 남습니다. 24개는 검토용 사례 묶음이며 확정 분모가 아닙니다. 각 구성은 성공0, 실패0, 미판정24 작성 슬롯입니다. 실제 실행0건. 목표와 점수, 반복과 종료 정책은 미정입니다.</p><p><a href="../../../02-qa07-case-mapping.md">입력별 매핑</a> / <a href="../../../01-qa07-contract.md">판정 계약</a></p><label>QA07 사례 <select id="qa7-case">${q.manifest.cases.map(c=>`<option value="${c.case_id}">${escape(c.source_case_id+' '+c.text)}</option>`).join('')}</select></label><div id="qa7-detail"></div><p>QA02와 동일한 대상 오류는 공통 원인 ID로 연결합니다. 정상 폐기는 허용하지만 다른 필수 응답의 누락은 허용하지 않습니다.</p></section>`;
html=html.replace('<script id="qa-data"',section+'<script id="qa-data"');
const qaScript=`
function renderQa07(){const c=data.qa07.manifest.cases.find(x=>x.case_id===document.getElementById('qa7-case').value);document.getElementById('qa7-detail').innerHTML='<h3>'+e(c.case_id)+'</h3><p>'+e(c.text)+'</p><p>시작 요청: '+e(c.proposed_root_input??'선행 요청 복원 필요')+' / 계약: 검토안 / 실제 판정: UNKNOWN</p><div class="scroll"><table><tr><th>의무</th><th>단계</th><th>충족 기준</th></tr>'+c.obligations.map(o=>'<tr><td>'+e(o.id)+'</td><td>'+e(o.stage)+'</td><td>'+e(o.requirement)+'</td></tr>').join('')+'</table></div><details><summary>미완료 조건과 사례 명세</summary><pre>'+e(JSON.stringify(c,null,2))+'</pre></details>';}
document.getElementById('qa7-case').addEventListener('change',renderQa07);renderQa07();
`;
html=html.replace('</script></html>',qaScript+'</script></html>');
await fs.writeFile(path.join(out,'guide.html'),html);
await fs.writeFile(path.join(out,'render-verification.json'),JSON.stringify({version:d.version,parent_workbook_sha256:crypto.createHash('sha256').update(await fs.readFile(path.join(base,'artifacts/outputs/01a15dfe-v5/VIA-QA-common-v5.xlsx'))).digest('hex'),ledger_sha256:crypto.createHash('sha256').update(bytes).digest('hex'),previews,formula_scan:'PASS',recalculation_probes:'PASS_RESTORED_EMPTY',native_excel:'NOT_RUN',actual_measurement:false},null,2)+'\n');
console.log(JSON.stringify({workbook:book,obligation_rows:rows.length,case_rows:caseRows.length}));
