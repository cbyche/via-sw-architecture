import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const parent=path.join(ROOT,'docs/architecture/12-decisions/decision-packages/current/evaluation');
const base=path.join(parent,'common-qa-v6'),dir=path.join(parent,'common-qa-v7');
const out=path.join(dir,'artifacts/outputs/01a15dfe-v7'),pre=path.join(dir,'artifacts/previews');
const bytes=await fs.readFile(path.join(dir,'artifacts/ledger.json')),d=JSON.parse(bytes),ap=d.appendices;
const req=createRequire(path.join(process.env.VIA_ARTIFACT_MODULE_BASE??'/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node','entry.cjs'));
const {FileBlob,SpreadsheetFile}=await import(pathToFileURL(req.resolve('@oai/artifact-tool')));
const sourceBook=path.join(base,'artifacts/outputs/01a15dfe-v6/VIA-QA-common-v6.xlsx');
const wb=await SpreadsheetFile.importXlsx(await FileBlob.load(sourceBook));
await fs.mkdir(out,{recursive:true});await fs.mkdir(pre,{recursive:true});
const render=async(sheet,range,file)=>{const b=await wb.render({sheetName:sheet,range,scale:1,format:'png'});await fs.writeFile(path.join(pre,file),new Uint8Array(await b.arrayBuffer()));};
await render('전체비교','A18:I33','source-v6-summary.png');
const summary=wb.worksheets.getItem('전체비교'),old=wb.worksheets.getItem('변경과제');
const changes=wb.worksheets.add('QA04변경'),appendix=wb.worksheets.add('QA상세부록');
const col=i=>String.fromCharCode(65+i);
function style(sheet,end,widths){sheet.showGridLines=false;sheet.getRange(`A1:${col(widths.length-1)}${end}`).format={font:{name:'Arial',size:11,color:'#23344B'},verticalAlignment:'center',rowHeight:34};widths.forEach((w,i)=>sheet.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidth=w);}
function header(sheet,row,values){sheet.getRange(`A${row}:${col(values.length-1)}${row}`).values=[values];sheet.getRange(`A${row}:${col(values.length-1)}${row}`).format={fill:'#253C59',font:{name:'Arial',size:11,color:'#FFFFFF',bold:true},rowHeight:44,wrapText:true,horizontalAlignment:'center',verticalAlignment:'center'};}
function title(sheet,text,note,last='H'){sheet.getRange(`A2:${last}2`).merge();sheet.getRange('A2').values=[[text]];sheet.getRange('A2').format.font={name:'Arial',size:16,bold:true};sheet.getRange(`A3:${last}3`).merge();sheet.getRange('A3').values=[[note]];sheet.getRange('A3').format={rowHeight:42,wrapText:true};}
style(changes,72,[17,17,36,57,62,72,24,24,24,16,26,26,24]);
title(changes,'QA04 공통 8개 변경 과제','실제 patch와 운영 모듈 등록 후 JSON을 검증하고 재생성한다. 56개 모두 미측정. 빈 수치는 0개가 아니다.','F');
header(changes,5,['구성','공통 과제 수','완료 과제 수','평균 모듈 수','점수 /6','근거 상태']);
header(changes,16,['구성','과제','변경 대상','변경 전','변경 후','완료 조건','기준 commit','실제 patch','운영 모듈 등록','수정 모듈 수','수용 증거','회귀 증거','상태']);
const taskMap=new Map(d.qa04.tasks.map(t=>[t.task_id,t]));
const changeRows=d.qa04.records.map(r=>{const t=taskMap.get(r.task_id);return [r.config_id,r.task_id,t.title,t.before,t.after,t.completion,r.baseline_commit??'',r.patch_ref??'',r.registry_ref??'',r.module_count??'',r.acceptance_ref??'',r.regression_ref??'',r.status];});
changes.getRange('A17:M72').values=changeRows;changes.getRange('A17:M72').format={rowHeight:132,wrapText:true};
d.configs.forEach((cfg,i)=>{const r=i+6,a=17+i*8,b=a+7;changes.getRange(`A${r}:B${r}`).values=[[cfg.id,8]];changes.getRange(`C${r}:E${r}`).formulas=[[
 `=COUNTIF(M${a}:M${b},"IMPLEMENTED_CHANGE_REVIEW")`,
 `=IF(AND(C${r}=8,COUNT(J${a}:J${b})=8),AVERAGE(J${a}:J${b}),"미측정")`,
 `=IF(ISNUMBER(D${r}),IF(D${r}<=1,6,IF(D${r}<=1.4,5,IF(D${r}<=1.8,4,IF(D${r}<=2.2,3,IF(D${r}<=2.6,2,IF(D${r}<=3,1,0)))))),"미산출")`]];changes.getRange(`F${r}`).values=[['실제 patch 없음']];});
changes.getRange('B6:F12').format.horizontalAlignment='center';changes.getRange('D6:D12').setNumberFormat('0.00');changes.freezePanes.freezeRows(16);changes.freezePanes.freezeColumns(2);
changes.getRange('A14:F14').merge();changes.getRange('A14').values=[['원전: qa/change-scenarios.md, qa04-change-manifest.json. 실제 증거 확인은 생성기의 사전 검증 책임이다.']];
changes.getRange('A14').format={rowHeight:34,wrapText:true};
old.getRange('A2').values=[['v6 보존: 이전 6과제 모듈 영향 추정']];old.getRange('A3').values=[['구형 concept 추정. 새 QA04 8과제의 평균/점수에는 사용하지 않는다. 새 표는 QA04변경.']];
const cmp=['BASE','DP2-B','BASE','DP3-B-R1','DP3-B-R2','DP3-B-R3','BASE','DP5-B','BASE','DP44-B'];
summary.getRange('A3').values=[['분석 모델 v7. QA04 새 8과제는 미측정. 시간·비용·메모리 추정은 유지. QA상세부록 참조.']];
summary.getRange('D20').values=[['QA04 개/과제']];summary.getRange('E20').values=[['QA05 $/묶음']];summary.getRange('H20').values=[['QA08 GB']];
cmp.forEach((id,i)=>{summary.getRange(`D${21+i}`).formulas=[[`='QA04변경'!D${6+d.configs.findIndex(c=>c.id===id)}`]];const oldFormula=summary.getRange(`H${21+i}`).formulas[0][0];summary.getRange(`H${21+i}`).formulas=[[oldFormula+'/1000']];});
summary.getRange('H21:H30').setNumberFormat('0.000');
summary.getRange('A58:I58').merge();summary.getRange('A58').values=[['QA06 전체 제품 예산 512 MB, QA08 추가 모델 1 GB 미만. 두 메모리는 합산하지 않는다. 현재 전 구성 QA06 추정은 예산 초과.']];summary.getRange('A58').format={rowHeight:42,wrapText:true,font:{name:'Arial',size:11,color:'#23344B'}};
summary.getRange('A59:I59').merge();summary.getRange('A59').values=[['QA05는 24입력 분석 묶음 비용. 실제 반복/청구 관측이 없으며 COMMON-28 목표 $2.55를 적용하지 않는다.']];summary.getRange('A59').format={rowHeight:34,wrapText:true,font:{name:'Arial',size:11,color:'#23344B'}};
// Appendix numeric aggregates link directly to the existing event/cost calculation rows.
const eventRows=new Map();let er=6;for(const c of d.cases)for(const e of c.events)eventRows.set(c.config_id+'|'+e.id,er++);
const costRows=new Map();let cr=6;for(const c of d.cases){const cancelled=new Set(c.audio.filter(a=>a.discard_at!=null).map(a=>a.node));for(const u of c.usage){const k=[c.config_id,cancelled.has(u.node)||u.node==='relation_unused'?'CANCELLED_OR_UNUSED':u.phase.toUpperCase(),u.bucket].join('|');if(!costRows.has(k))costRows.set(k,[]);costRows.get(k).push(cr++);}}
const rows=[],formulas=[];
const add=(row,formula)=>{rows.push(row);if(formula)formulas.push([rows.length+5,formula]);};
for(const a of ap.qa01_types){const matches=ap.qa01_event_labels.filter(e=>['config_id','source_kind','primary_type','profile'].every(k=>e[k]===a[k]));const refs=matches.map(e=>`'사건계산'!O${eventRows.get(e.config_id+'|'+e.event_id)}`);add(['QA01',a.config_id,`${a.source_kind} ${a.primary_type}`,a.profile,a.planned,a.analytical_onsets,'', '초',a.physical_observations,`철회 ${a.withdrawn}. 분석 음성 시작의 평균; 물리 관측 없음.`,`사건계산 O열, appendix.qa01_event_labels`],`=IF(COUNT(${refs.join(',')})=0,"미산출",AVERAGE(${refs.join(',')}))`);}
for(const a of ap.qa02_tags)add(['QA02',a.config_id,a.tag,'중복 태그',a.planned,'','미산출','%',a.observed,'태그 합계를 대표 분모로 사용하지 않음. 전체 고유 지칭 33개.','appendix.qa02_expressions, 지칭판정']);
for(const a of ap.qa05_buckets){const refs=costRows.get([a.config_id,a.phase,a.bucket].join('|')).map(r=>`'비용원장'!H${r}`);add(['QA05',a.config_id,a.phase,a.bucket,a.items,'','','USD',0,'분석 묶음 비용. 각 청구 항목을 한 번만 집계.','비용원장 H열'],`=SUM(${refs.join(',')})`);}
for(const a of ap.qa07_types)add(['QA07',a.config_id,a.type,'검토 사례',a.draft_cases,'','미산출','%',a.observed,`미판정 ${a.unknown}. 해석/구성/전달 실패 원인 미관측. 원인은 중복 가능.`, 'QA07사례, QA07의무']);
for(const a of ap.memory_peaks){const cfgIndex=d.configs.findIndex(c=>c.id===a.config_id),start=6+cfgIndex*24,end=start+23;add([a.qa.toUpperCase(),a.config_id,'동시 자원 최고량',a.case_id,24,'','',a.qa==='qa06'?'MB':'GB',0,`${a.qa==='qa06'?'전체 제품 512 MB 이하':'추가 모델 1 GB 미만'}. QA06/08 합산 금지. 구성요소는 입력별 원장 참조.`, `목표계산, ledger.cases[].${a.qa==='qa06'?'memory_peak':'qa08_peak'}`],`=MAX('목표계산'!${a.qa==='qa06'?'H':'I'}${start}:${a.qa==='qa06'?'H':'I'}${end})${a.qa==='qa08'?'/1000':''}`);}
style(appendix,rows.length+5,[12,18,40,24,15,16,18,12,17,66,54]);
title(appendix,'QA 세부 유형과 원장 연결','계획 건수와 실제 관측을 구분한다. QA02 태그/QA07 원인은 중복 가능. QA03 상세는 목표계산·처리단계.','K');
header(appendix,5,['QA','구성','유형','조건/단위','계획/항목 수','분석 출력 수','값','단위','실제 관측 수','해석과 한계','원전']);
appendix.getRange(`A6:K${rows.length+5}`).values=rows;appendix.getRange(`A6:K${rows.length+5}`).format={rowHeight:72,wrapText:true};
for(const [r,f] of formulas)appendix.getRange(`G${r}`).formulas=[[f]];
appendix.getRange(`G6:G${rows.length+5}`).setNumberFormat('0.000000');appendix.freezePanes.freezeRows(5);appendix.freezePanes.freezeColumns(2);
// Disposable formula probes. Evidence validation stays in the JSON generator; this is not product evidence.
changes.getRange('J17:J24').values=Array.from({length:8},()=>[0]);assert.equal(changes.getRange('D6').values[0][0],'미측정');
changes.getRange('M17:M24').values=Array.from({length:8},()=>['IMPLEMENTED_CHANGE_REVIEW']);assert.equal(changes.getRange('D6').values[0][0],0);assert.equal(changes.getRange('E6').values[0][0],6);
changes.getRange('J24').clear({applyTo:'contents'});assert.equal(changes.getRange('D6').values[0][0],'미측정');
changes.getRange('J17:J24').clear({applyTo:'contents'});changes.getRange('M17:M24').values=Array.from({length:8},()=>['NOT_MEASURED']);
const cost=wb.worksheets.getItem('비용원장'),oldRate=cost.getRange('G6').values[0][0],costAppendix=rows.findIndex(r=>r[0]==='QA05'&&r[1]==='BASE'&&r[2]==='REQUEST'&&r[3]==='text_input')+6;
const beforeCost=appendix.getRange(`G${costAppendix}`).values[0][0];cost.getRange('G6').values=[[oldRate*2]];assert(appendix.getRange(`G${costAppendix}`).values[0][0]>beforeCost);cost.getRange('G6').values=[[oldRate]];
wb.recalculate();
cmp.forEach((id,i)=>{assert.equal(summary.getRange(`D${21+i}`).values[0][0],'미측정');assert(Math.abs(summary.getRange(`H${21+i}`).values[0][0]-d.summary[id].qa08_gb)<1e-9);});
for(const sheet of wb.worksheets.items)for(const row of sheet.getUsedRange().values)for(const v of row)assert(!/^#(REF!|DIV\/0!|VALUE!|NAME\?|N\/A|NUM!|NULL!|SPILL!|CALC!)/.test(String(v)));
console.log((await wb.inspect({kind:'region',sheetId:'QA04변경',range:'A5:F12',maxChars:1800,tableMaxRows:8,tableMaxCols:6})).ndjson);
const book=path.join(out,'VIA-QA-common-v7.xlsx');await(await SpreadsheetFile.exportXlsx(wb)).save(book);await fs.rm(book+'.inspect.ndjson',{force:true});
const views=[['전체비교','A18:I33','summary.png'],['전체비교','A58:I59','scope-notes.png'],['QA04변경','A1:F12','qa04-summary.png'],['QA04변경','A16:D19','qa04-before.png'],['QA04변경','E16:F19','qa04-after.png'],['QA04변경','G16:M19','qa04-evidence.png'],['QA상세부록','A1:K10','appendix.png'],['변경과제','A1:E8','legacy-changes.png']];
for(const args of views)await render(...args);
const escape=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const table=(heads,rows)=>`<div class="scroll"><table><thead><tr>${heads.map(h=>`<th>${escape(h)}</th>`).join('')}</tr></thead><tbody>${rows.map(r=>`<tr>${r.map(c=>`<td>${escape(c)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
const fmt=(x,n=4)=>x==null?'미산출':x.toFixed(n);
let html=await fs.readFile(path.join(base,'artifacts/outputs/01a15dfe-v6/guide.html'),'utf8');
html=html.replaceAll('공통 QA v6','공통 QA v7').replaceAll('공통 QA 비교 v6','공통 QA 비교 v7').replaceAll('VIA-QA-common-v6.xlsx','VIA-QA-common-v7.xlsx');
html=html.replace(/(<script id="qa-data" type="application\/json">)[\s\S]*?(<\/script>)/,(_,a,b)=>a+JSON.stringify(d).replace(/</g,'\\u003c')+b);
const mainRows=cmp.map((id,i)=>{const s=d.summary[id];return [i<2?'DP2':i<6?'DP3':i<8?'DP5':'DP44',id==='BASE'?'A':id.replace(/^DP\d+-/,''),fmt(s.qa01),s.qa01_score,fmt(s.wait.L),fmt(s.wait.B),fmt(s.wait.R),fmt(s.qa03),s.qa03_score,'미산출','미측정',fmt(s.qa05,6),fmt(s.qa06),'미산출',fmt(s.qa08_gb,3)];});
html=html.replace(/<section><h2>모든 방안의 수치<\/h2>[\s\S]*?<\/section>/,`<section id="main-summary"><h2>모든 방안의 수치</h2><p>NORMAL 36건, WAIT 6예정(5출력·1철회), QA03 전체 24목표. 네 A행은 동일 BASE입니다. QA04는 새 8과제입니다.</p>${table(['DP','방안','QA01 NORMAL 초','점수 /6','WAIT 총 초','WAIT B 초','WAIT R 초','QA03 초','점수 /6','QA02 %','QA04 개/과제','QA05 $/묶음','QA06 MB','QA07 %','QA08 GB'],mainRows)}<p class="note">QA04 실제 patch가 없어 모두 미측정입니다. QA02/07은 실제 출력이 없어 미산출입니다. 시간/비용/메모리는 분석 추정입니다. QA06 전체 제품 예산은 512 MB이며 현재 모든 추정값이 초과합니다. QA08은 추가 모델 1 GB 미만이며 QA06과 합산하지 않습니다. 제품 적합성은 실측 미판정입니다.</p><p>QA05는 24입력 분석 묶음 1개의 비용입니다. 실제 반복과 청구 종료 관측이 없으며 COMMON-28 목표 $2.55를 적용하지 않습니다.</p></section>`);
html=html.replaceAll('href="../../../02-qa07-case-mapping.md"','href="../../../../common-qa-v6/02-qa07-case-mapping.md"').replaceAll('href="../../../01-qa07-contract.md"','href="../../../../common-qa-v6/01-qa07-contract.md"');
const extra=`<section id="qa04-current"><h2>QA04 같은 8개 변경 과제</h2><p>7구성 × 8과제는 모두 NOT_MEASURED입니다. 실제 기준 코드와 모듈 registry, patch 및 수용/회귀 증거를 등록한 뒤 계산합니다. 기존 6과제 추정값은 v6에 보존했습니다.</p><p><a href="../../../02-qa04-changes.md">전체 완료 조건과 증거 계약</a></p>${table(['과제','대상','변경 전','변경 후','완료 조건'],d.qa04.tasks.map(t=>[t.task_id,t.title,t.before,t.after,t.completion]))}</section><section id="qa-appendices"><h2>QA별 세부 유형과 관측 범위</h2><p>QA01은 사건마다 주유형 하나이며 QA02 태그와 QA07 실패 원인은 중복 가능합니다. 세부 비율을 평균내어 대표 비율을 만들지 않습니다. 실제 관측 수는 모두 0입니다.</p>${[['QA01 요청 유형',['구성','입력 종류','주유형','조건','예정','분석 출력','철회','평균 초'],ap.qa01_types.map(a=>[a.config_id,a.source_kind,a.primary_type,a.profile,a.planned,a.analytical_onsets,a.withdrawn,fmt(a.mean_s)])],['QA02 지칭 태그',['구성','태그','계획 지칭','관측','완전 일치율 %'],ap.qa02_tags.map(a=>[a.config_id,a.tag,a.planned,a.observed,'미산출'])],['QA05 비용 항목',['구성','단계','과금 종류','항목 수','분석 USD'],ap.qa05_buckets.map(a=>[a.config_id,a.phase,a.bucket,a.items,fmt(a.usd,6)])],['QA07 사례 유형',['구성','유형','검토 사례','관측','전체 충족률 %'],ap.qa07_types.map(a=>[a.config_id,a.type,a.draft_cases,a.observed,'미산출'])],['QA06/08 최고량',['QA','구성','입력','값','단위'],ap.memory_peaks.map(a=>[a.qa.toUpperCase(),a.config_id,a.case_id,fmt(a.qa==='qa08'?a.value_mb/1000:a.value_mb,3),a.qa==='qa08'?'GB':'MB'])]].map(([name,heads,r])=>`<details><summary>${name}</summary>${table(heads,r)}</details>`).join('')}<p>QA03의 전체 완료 DAG와 QA06/08의 동시 구성 자원은 위 입력별 원장에서 확인합니다. QA07 단계별 실패 원인도 실제 판정이 없어 미산출입니다.</p><p>QA05 준비/취소/미사용 생성은 분석에 포함했습니다. 재시도는 분석에 예정하지 않았고 후속 청구 종료 관측도 없습니다. 실제 발생 0회나 청구 완료로 읽지 않습니다.</p></section>`;
html=html.replace('<script id="qa-data"',extra+'<script id="qa-data"');
await fs.writeFile(path.join(out,'guide.html'),html);
await fs.writeFile(path.join(out,'render-verification.json'),JSON.stringify({version:d.version,parent_workbook_sha256:crypto.createHash('sha256').update(await fs.readFile(sourceBook)).digest('hex'),ledger_sha256:crypto.createHash('sha256').update(bytes).digest('hex'),previews:views.map(v=>v[2]),formula_scan:'PASS',recalculation_probes:'PASS_RESTORED',native_excel:'NOT_RUN',actual_measurement:false},null,2)+'\n');
console.log(JSON.stringify({workbook:book,qa04_records:changeRows.length,appendix_rows:rows.length}));
