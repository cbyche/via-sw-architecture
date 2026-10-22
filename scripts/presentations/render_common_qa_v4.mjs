// Read-only adapter for the versioned simulation ledger. No product runs.
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const DIR=path.join(ROOT,'docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v4');
const OUT=path.join(DIR,'artifacts/outputs/01a15dfe-v4'), PRE=path.join(DIR,'artifacts/previews');
const bytes=await fs.readFile(path.join(DIR,'artifacts/ledger.json'));
const d=JSON.parse(bytes),spec=JSON.parse(await fs.readFile(path.join(DIR,'specification.json'))),oracle=JSON.parse(await fs.readFile(path.join(DIR,'oracles.json')));
const req=createRequire(path.join(process.env.VIA_ARTIFACT_MODULE_BASE??'/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node','entry.cjs'));
const {Workbook,SpreadsheetFile}=await import(pathToFileURL(req.resolve('@oai/artifact-tool')));
const wb=Workbook.create(),names=['전체비교','사례별계산','처리단계','비용원장','메모리상태','변경과제','지칭판정','의미판정','공통입력','공통가정'];
const sheets=Object.fromEntries(names.map(n=>[n,wb.worksheets.add(n)])),tables={};
const qa=['qa01','qa02','qa03','qa04','qa05','qa06','qa07','first_voice'];
const labels=['QA01 화면 (s)','QA02 지칭 (%)','QA03 VIA 시간 (s)','QA04 모듈 (개)','QA05 묶음 비용 ($)','QA06 최대 RAM (MB)','QA07 의미 (%)','첫 음성 (s)'];
const sizes=[3,2,3,3,6,3,2,3];
const cmp=[];for(const [dp,ids] of [['DP2',['DP2-B']],['DP3',['DP3-B-R1','DP3-B-R2','DP3-B-R3']],['DP5',['DP5-B']],['DP44',['DP44-B']]])for(const id of ['BASE',...ids])cmp.push({dp,id,alt:id==='BASE'?'A':id.slice(dp.length+1)});
const col=n=>{let s='';for(n++;n;n=Math.floor((n-1)/26))s=String.fromCharCode(65+(n-1)%26)+s;return s;};
const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
function canon(v,p=''){
 if(Array.isArray(v)){const a=v.map(x=>canon(x,p+'[]'));if(p==='conditions.order')return a;assert(p==='targets'||p==='referents',p);return [...new Set(a.map(x=>JSON.stringify(x)))].sort().map(x=>JSON.parse(x));}
 if(v&&typeof v==='object')return Object.fromEntries(Object.keys(v).sort().map(k=>[k,canon(v[k],p?p+'.'+k:k)]));return v;
}
const meaning=v=>JSON.stringify(canon(v)),referents=v=>JSON.stringify(canon(v,'referents'));
assert.notEqual(meaning({conditions:{order:['Q1','R0']}}),meaning({conditions:{order:['R0','Q1']}}));
assert.equal(meaning({targets:['B','A','A']}),meaning({targets:['A','B']}));
assert.notEqual(meaning({targets:['A']}),meaning({targets:['a']}));
function fmt(v,k){return v==null?'미산출':Number(v.toFixed(12)).toLocaleString('en-US',{minimumFractionDigits:sizes[k],maximumFractionDigits:sizes[k],useGrouping:false});}
function table(name,title,heads,rows,widths,note,{height=34}={}){
 const sh=sheets[name],last=col(heads.length-1),end=5+rows.length;
 sh.showGridLines=false;sh.freezePanes.freezeRows(5);sh.freezePanes.freezeColumns(name==='전체비교'?2:1);
 sh.getRange(`A1:${last}${end}`).format.font.name='Arial';sh.getRange(`A1:${last}${end}`).format.font.size=11;
 sh.getRange(`A1:${last}${end}`).format.font.color='#24364B';sh.getRange(`A1:${last}${end}`).format.verticalAlignment='center';
 sh.getRange(`A1:${last}1`).merge();sh.getRange('A1').values=[[title]];sh.getRange(`A1:${last}1`).format.fill='#183A52';sh.getRange(`A1:${last}1`).format.font.color='#FFFFFF';sh.getRange(`A1:${last}1`).format.font.size=18;sh.getRange(`A1:${last}1`).format.rowHeight=38;
 sh.getRange(`A2:${last}3`).merge();sh.getRange('A2').values=[[note]];sh.getRange(`A2:${last}3`).format.wrapText=true;sh.getRange(`A2:${last}3`).format.fill='#EDF4F7';sh.getRange(`A2:${last}3`).format.rowHeight=23;
 sh.getRange(`A5:${last}5`).values=[heads];sh.getRange(`A5:${last}5`).format.fill='#DDEBF0';sh.getRange(`A5:${last}5`).format.font.bold=true;sh.getRange(`A5:${last}5`).format.wrapText=true;sh.getRange(`A5:${last}5`).format.rowHeight=38;
 if(rows.length){sh.getRange(`A6:${last}${end}`).values=rows;sh.getRange(`A6:${last}${end}`).format.rowHeight=height;sh.getRange(`A6:${last}${end}`).format.wrapText=true;for(let r=6;r<=end;r+=2)sh.getRange(`A${r}:${last}${r}`).format.fill='#F2F6F8';}
 widths.forEach((w,i)=>sh.getRange(`${col(i)}:${col(i)}`).format.columnWidth=w);
 tables[name]={end,cols:heads.length};return sh;
}
function formulas(name,column,values){if(values.length)sheets[name].getRange(`${column}6:${column}${5+values.length}`).formulas=values.map(v=>[v]);}
const usage=[],memory=[],stages=[],changes=d.changes.map(c=>[c.config_id,c.change_id,c.title,c.modules.join('\n'),c.module_count,c.evidence]);
const caseRows=[],stageRow=new Map(),caseIndex=new Map(),costRanges=new Map(),memRanges=new Map();
for(const c of d.cases){
 const key=c.config_id+'|'+c.case_id;caseIndex.set(key,6+caseRows.length);
 for(const n of c.nodes){stageRow.set(key+'|'+n.id,6+stages.length);stages.push([c.config_id,c.case_id,n.id,n.label,n.kind,n.start_s,n.finish_s,n.duration_s,c.goal_origin_s,n.causal_parents.join(', '),n.resource??'none',0]);}
 const ustart=6+usage.length;for(const u of c.usage)usage.push([c.config_id,c.case_id,u.node,u.phase,u.bucket,u.amount,u.rate_usd,0]);costRanges.set(key,[ustart,5+usage.length]);
 const mstart=6+memory.length;for(const [owner,mb] of Object.entries(c.memory_peak.components_mb))memory.push([c.config_id,c.case_id,c.memory_peak.time_s,owner,mb]);memRanges.set(key,[mstart,5+memory.length]);
 const input=spec.scenarios.find(s=>s.id===c.case_id);caseRows.push([c.config_id,c.case_id,input.text,c.goal_origin_s,0,c.qa01,0,0,0,0,c.first_voice,c.status]);
}
table('처리단계','처리 순서와 QA03 시간 투영',['구성','사례','단계 ID','내용','VIA/외부','시작 s','종료 s','duration s','목표 원점 s','인과 부모','자원','QA03 누적 s'],stages,[16,8,24,42,12,12,12,12,12,42,12,15],'종료 시각은 Python scheduler 계산값. L열은 외부·사용자 대기·순수 재생을 0으로 한 DAG 투영 수식이다. 병렬 경로는 MAX로 연결한다.',{height:42});
const pf=[];for(const c of d.cases)for(const n of c.nodes){const r=stageRow.get(c.config_id+'|'+c.case_id+'|'+n.id);const parents=n.causal_parents.map(p=>'L'+stageRow.get(c.config_id+'|'+c.case_id+'|'+p));pf.push(`=IF(G${r}<=I${r},0,MAX(0${parents.length?','+parents.join(','):''})+IF(E${r}="via",MAX(0,G${r}-MAX(F${r},I${r})),0))`);}
formulas('처리단계','L',pf);sheets['처리단계'].getRange(`F6:I${tables['처리단계'].end}`).setNumberFormat('0.000');sheets['처리단계'].getRange(`L6:L${tables['처리단계'].end}`).setNumberFormat('0.000');
table('비용원장','요청 전·중·후의 VIA 모델 비용',['구성','사례','단계 ID','단계 범위','과금 종류','사용량','단가 $','계산 비용 $'],usage,[16,8,25,16,18,14,14,17],'H=F×G. 사전/미사용/취소 비용도 포함. G열 단가 변경은 전체비교에 반영된다. 수신 Agent 내부 비용은 미모델링이며 0이 아니다.');
formulas('비용원장','H',usage.map((_,i)=>`=F${i+6}*G${i+6}`));sheets['비용원장'].getRange(`F6:H${tables['비용원장'].end}`).setNumberFormat('0.000000');sheets['비용원장'].getRange(`G6:G${tables['비용원장'].end}`).format.font.color='#1769AA';
table('메모리상태','각 사례의 동시 최대 메모리 상태',['구성','사례','peak 시각 s','중복 없는 소유 항목','MB'],memory,[16,8,16,50,16],'전체 구간을 다시 계산한 최대 상태의 소유 항목 합이다. 서로 다른 시각의 최대값을 더하지 않는다. interval과 audio ramp 원본은 ledger.json에 있다.');sheets['메모리상태'].getRange(`C6:C${tables['메모리상태'].end}`).setNumberFormat('0.000');sheets['메모리상태'].getRange(`E6:E${tables['메모리상태'].end}`).setNumberFormat('0.000');
table('변경과제','같은 여섯 변경 과제의 모듈 영향',['구성','과제','요청','예상 concept 모듈 집합','개수','근거 수준'],changes,[16,8,40,38,10,42],'모듈 집합의 크기를 과제별로 한 번 센다. 실제 Rust diff가 없는 설계 추정이며 DP별 기존 평균을 합산하지 않는다.',{height:92});
table('사례별계산','같은 입력에서 구성별 값이 나온 과정',['구성','사례','공통 입력','목표 원점 s','입력 종료 s','첫 화면 s','QA01 s','QA03 s','QA05 $','QA06 MB','첫 음성 s','근거'],caseRows,[16,8,58,13,13,13,13,13,13,15,13,44],'입력 종료는 모두 0초. 기존 목표 정정 U08/U16/U24의 QA03 원점은 -6초. 시간표·메모리 가정을 바꾸려면 specification 수정 후 생성기를 다시 실행한다.',{height:56});
for(let i=0;i<d.cases.length;i++){
 const c=d.cases[i],r=i+6,key=c.config_id+'|'+c.case_id,[us,ue]=costRanges.get(key),[ms,me]=memRanges.get(key),terminal=stageRow.get(key+'|'+c.terminal_node);
 sheets['사례별계산'].getRange(`G${r}:J${r}`).formulas=[[`=F${r}-E${r}`,`='처리단계'!L${terminal}`,`=SUM('비용원장'!H${us}:H${ue})`,`=SUM('메모리상태'!E${ms}:E${me})`]];
}
sheets['사례별계산'].getRange(`D6:K${tables['사례별계산'].end}`).setNumberFormat('0.000');sheets['사례별계산'].getRange(`I6:I${tables['사례별계산'].end}`).setNumberFormat('0.000000');
const refs=[],meanings=[];for(const config of d.configs)for(const o of oracle.items){o.referents.forEach((ref,i)=>refs.push([config.id,o.id,i+1,referents(ref),'','']));meanings.push([config.id,o.id,meaning(o.meaning),'','']);}
table('지칭판정','QA02 공통 정답과 최초 채택 출력',['구성','사례','표현 번호','정답 canonical set','관측 출력 canonical set','일치 1/0'],refs,[16,8,12,38,38,14],'후보 출력은 모두 비어 있다. 평가자가 검증한 최초 채택 집합을 canonical 형식으로 입력해야 한다. 33개가 모두 채워지기 전에는 집계가 미산출이다.',{height:48});
formulas('지칭판정','F',refs.map((_,i)=>`=IF(E${i+6}="","",IF(EXACT(D${i+6},E${i+6}),1,0))`));
table('의미판정','QA07 공통 정답과 최초 채택 의미',['구성','사례','정답 canonical meaning','관측 출력 canonical meaning','전체 일치 1/0'],meanings,[16,8,68,68,15],'검증된 최초 의미를 입력한다. 필수 의미 전체의 완전 일치. targets는 집합, conditions.order는 순서를 보존한다. 정답을 후보 출력으로 복사해 실측값을 만들지 않는다.',{height:82});
formulas('의미판정','E',meanings.map((_,i)=>`=IF(D${i+6}="","",IF(EXACT(C${i+6},D${i+6}),1,0))`));
sheets['지칭판정'].getRange(`E6:E${tables['지칭판정'].end}`).format.fill='#FFF5D9';sheets['의미판정'].getRange(`D6:D${tables['의미판정'].end}`).format.fill='#FFF5D9';
table('공통입력','모든 구성에 동일하게 주는 24개 입력',['사례','그룹','입력 방식','사용자 입력','공통 초기 사실 / 고정 사건 시각','기존 사례 출처','지칭 수'],spec.scenarios.map(s=>[s.id,s.group,s.modality,s.text,s.facts,s.source_family,oracle.items.find(o=>o.id===s.id).referents.length]),[8,15,12,56,70,26,10],'각 사례 1회인 구조 커버리지 묶음. 실사용 빈도 표본이나 통계 반복이 아니다. 원본 화면/WAV와 실제 후보 adapter는 후속 실행 때 준비해야 한다.',{height:85});
const assumptions=[...Object.entries(spec.profile).map(([k,v])=>['profile',k,v]),...Object.entries(spec.token_budget).map(([k,v])=>['tokens',k,v]),...Object.entries(spec.policy).map(([k,v])=>['policy',k,v])];
table('공통가정','환경·산정식·상태를 공개한 가정표',['구분','키','값 또는 설명'],assumptions,[12,38,104],'specification.json과 같은 작성 가정이다. 실측 교정값이나 클라우드의 실제 견적이 아니다. 시간·RAM 가정 변경은 DAG와 원장을 재생성해야 한다.',{height:42});
const displayRows=cmp.map(r=>[r.dp,r.alt,r.id,...qa.map(()=>0)]);
table('전체비교','DP2 · DP3 · DP5 · DP44 — 같은 입력, 같은 환경',['DP','방안','비교 구성',...labels],displayRows,[10,12,18,17,17,19,18,21,22,17,17],'분석 모델 v4 · 실측 아님 | 24입력 / 24목표 / 33지칭 / 6변경 과제 | A 네 행은 같은 BASE 원장이다. QA02·07 미산출. 첫 음성은 QA01과 별도 진단.',{height:36});
const summaryFormulas=(id)=>{
 const ci=d.configs.findIndex(c=>c.id===id),s=6+ci*24,e=s+23,cs=6+ci*6,ce=cs+5,rs=6+ci*33,re=rs+32;
 return [`=AVERAGE('사례별계산'!G${s}:G${e})`,`=IF(COUNT('지칭판정'!F${rs}:F${re})<33,"미산출",SUM('지칭판정'!F${rs}:F${re})/33*100)`,`=AVERAGE('사례별계산'!H${s}:H${e})`,`=AVERAGE('변경과제'!E${cs}:E${ce})`,`=SUM('사례별계산'!I${s}:I${e})`,`=MAX('사례별계산'!J${s}:J${e})`,`=IF(COUNT('의미판정'!E${s}:E${e})<24,"미산출",SUM('의미판정'!E${s}:E${e})/24*100)`,`=AVERAGE('사례별계산'!K${s}:K${e})`];
};
sheets['전체비교'].getRange('D6:K15').formulas=cmp.map(r=>summaryFormulas(r.id).map((f,i)=>[1,6].includes(i)?f:`=ROUND(${f.slice(1)},12)`));
for(let i=0;i<8;i++)sheets['전체비교'].getRange(`${col(i+3)}6:${col(i+3)}15`).setNumberFormat('0.'+'0'.repeat(sizes[i]));
const summary=sheets['전체비교'];summary.getRange('A18:K18').merge();summary.getRange('A18').values=[['기준 A 대비 차이 · 음수는 해당 수치 감소이며 선택 권고가 아님']];summary.getRange('A18:K18').format.fill='#183A52';summary.getRange('A18:K18').format.font.color='#FFFFFF';summary.getRange('A18:K18').format.rowHeight=32;
const deltaRows=cmp.map((r,i)=>({...r,row:i+6})).filter(r=>r.id!=='BASE');summary.getRange('A20:K20').values=[['DP','방안','기준',...labels]];summary.getRange('A20:K20').format.wrapText=true;summary.getRange('A20:K20').format.rowHeight=40;summary.getRange('A20:K20').format.fill='#DDEBF0';
deltaRows.forEach((v,i)=>{const r=i+21;summary.getRange(`A${r}:C${r}`).values=[[v.dp,v.alt,'공통 A']];summary.getRange(`D${r}:K${r}`).formulas=[qa.map((q,j)=>q==='qa02'||q==='qa07'?'="미산출"':`=ROUND(${col(j+3)}${v.row}-${col(j+3)}6,12)`)];});
for(let i=0;i<8;i++)summary.getRange(`${col(i+3)}21:${col(i+3)}26`).setNumberFormat('+0.'+'0'.repeat(sizes[i])+';-0.'+'0'.repeat(sizes[i])+';0.'+'0'.repeat(sizes[i]));
summary.getRange('A21:K26').format.rowHeight=30;summary.getRange('A28:K30').merge();summary.getRange('A28').values=[['읽는 순서: 전체비교 → 사례별계산 → 비용원장/메모리상태/처리단계. QA04는 개념 모듈 추정. QA05는 24개 전체 비용 합. QA06은 동시 최대. 기존 COMMON-28 점수표를 이 묶음에 적용하지 않는다. 정확도는 실제 최초 출력 확보 후 판정한다.']];summary.getRange('A28:K30').format.wrapText=true;summary.getRange('A28:K30').format.fill='#FFF5D9';
summary.getRange('A18:K30').format.font.name='Arial';summary.getRange('A18:K30').format.font.size=11;summary.getRange('A18:K30').format.verticalAlignment='center';
for(const range of ['E6:E15','J6:J15','E21:E26','J21:J26'])summary.getRange(range).format.horizontalAlignment='center';
// Verify formula chains and blank guards with reversible probes, then restore.
const near=(a,b)=>assert(Math.abs(a-b)<1e-7,`${a} != ${b}`);
for(let i=0;i<cmp.length;i++)summary.getRange(`D${i+6}:K${i+6}`).values[0].forEach((v,k)=>d.summary[cmp[i].id][qa[k]]==null?assert.equal(v,'미산출'):near(v,d.summary[cmp[i].id][qa[k]]));
const initialCost=summary.getRange('H6').values[0][0],rate=sheets['비용원장'].getRange('G6').values[0][0],amount=sheets['비용원장'].getRange('F6').values[0][0];sheets['비용원장'].getRange('G6').values=[[rate*2]];near(summary.getRange('H6').values[0][0],initialCost+rate*amount);sheets['비용원장'].getRange('G6').values=[[rate]];
for(let r=6;r<=38;r++)sheets['지칭판정'].getRange(`E${r}`).values=[[sheets['지칭판정'].getRange(`D${r}`).values[0][0]]];near(summary.getRange('E6').values[0][0],100);sheets['지칭판정'].getRange('E6').values=[['wrong']];near(summary.getRange('E6').values[0][0],32/33*100);sheets['지칭판정'].getRange('E6:E38').clear({applyTo:'contents'});
for(let r=6;r<=29;r++)sheets['의미판정'].getRange(`D${r}`).values=[[sheets['의미판정'].getRange(`C${r}`).values[0][0]]];near(summary.getRange('J6').values[0][0],100);sheets['의미판정'].getRange('D6').values=[['wrong']];near(summary.getRange('J6').values[0][0],23/24*100);sheets['의미판정'].getRange('D6:D29').clear({applyTo:'contents'});
wb.recalculate();assert.equal(summary.getRange('E6').values[0][0],'미산출');assert.equal(summary.getRange('J6').values[0][0],'미산출');near(summary.getRange('H6').values[0][0],initialCost);
let formulaCount=0;for(const [name,t] of Object.entries(tables)){const range=sheets[name].getRange(`A1:${col(t.cols-1)}${name==='전체비교'?30:t.end}`);for(const row of range.values)for(const v of row)assert(!/^#(REF!|DIV\/0!|VALUE!|N\/A|NAME\?|NUM!|SPILL!|CALC!)/.test(String(v)),`${name}: ${v}`);for(const row of range.formulas)for(const v of row)if(v)formulaCount++;}
console.log((await wb.inspect({kind:'region',sheetId:'전체비교',range:'A5:K15',maxChars:2500,tableMaxRows:11,tableMaxCols:11})).ndjson);
await fs.mkdir(OUT,{recursive:true});await fs.mkdir(PRE,{recursive:true});
const workbookPath=path.join(OUT,'VIA-QA-common-v4.xlsx');await(await SpreadsheetFile.exportXlsx(wb)).save(workbookPath);
await fs.rm(workbookPath+'.inspect.ndjson',{force:true});
const previews=[];for(const n of names){const last=col(tables[n].cols-1);const range=`A1:${last}${n==='전체비교'?30:Math.min(tables[n].end,10)}`;const filename=n+'.png';const image=await wb.render({sheetName:n,range,scale:1,format:'png'});await fs.writeFile(path.join(PRE,filename),new Uint8Array(await image.arrayBuffer()));previews.push(filename);}
const matrix=rows=>`<table><thead><tr><th>DP</th><th>방안</th>${labels.map(x=>'<th>'+esc(x)+'</th>').join('')}</tr></thead><tbody>${rows.map(r=>`<tr data-config="${r.id}"><th>${r.dp}</th><td>${r.alt}</td>${qa.map((q,k)=>`<td class="num" data-qa="${q}">${fmt(d.summary[r.id][q],k)}</td>`).join('')}</tr>`).join('')}</tbody></table>`;
const payload=JSON.stringify({configs:d.configs,cases:d.cases.map(c=>({config_id:c.config_id,case_id:c.case_id,qa01:c.qa01,qa03:c.qa03,qa05:c.qa05,qa06:c.qa06,first_voice:c.first_voice,nodes:c.nodes,usage:c.usage,memory_peak:c.memory_peak})),summary:d.summary,deltas:d.deltas,scenarios:spec.scenarios,oracles:oracle.items}).replace(/</g,'\\u003c');
const html=`<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>공통 QA v4 — 네 DP 전체 비교</title><style>:root{font:16px/1.65 system-ui,sans-serif;color:#183247;background:#f2f6f8}body{max-width:1500px;margin:32px auto;padding:0 24px}h1{font-size:32px;line-height:1.3}h2{font-size:23px}section{background:white;padding:24px;margin:22px 0;border-radius:10px}.tag{color:#986000;background:#fff1cd;padding:5px 12px;border-radius:20px;display:inline-block}.overflow{overflow:auto}table{border-collapse:collapse;width:100%;font-size:14px}th,td{padding:11px 12px;border-bottom:1px solid #d8e3e9;text-align:left;vertical-align:top}thead{background:#e6f0f4}tbody tr:nth-child(even){background:#f7fafb}.num{text-align:right;white-space:nowrap;font-variant-numeric:tabular-nums}select{font:inherit;padding:8px;max-width:100%;margin:8px}pre{white-space:pre-wrap;word-break:break-word;font-size:13px}a{color:#125b8c}.note{background:#fff5d9;padding:14px}details{margin:12px 0}summary{cursor:pointer;font-weight:600}</style>
<header><span class="tag">분석 모델 값 · 실측 아님</span><h1>네 DP, 모든 방안, 같은 QA</h1><p>24개 입력 · 24개 목표 · 33개 지칭 · 6개 변경 과제 · 동일 CPU/모델 가정</p><p><a href="VIA-QA-common-v4.xlsx">Excel 다운로드</a> · <a href="../../../README.md">계약과 검토 기록</a></p></header>
<section><h2>전체 비교</h2><p>각 DP의 A는 같은 A/A/A/A 구성입니다. B 행은 해당 DP만 바꾼 구성입니다. QA01은 화면 표시이며 첫 음성은 별도 진단입니다.</p><div class="overflow">${matrix(cmp)}</div><p class="note">QA02와 QA07은 실제 최초 채택 출력이 없어 미산출입니다. 0%나 100%를 부여하지 않았습니다. 시간·비용·RAM은 조건부 계산이고 QA04는 실제 Rust 변경이 없는 개념 모듈 추정입니다.</p></section>
<section><h2>A 대비 차이</h2><div class="overflow"><table><thead><tr><th>구성</th>${labels.map(l=>'<th>'+esc(l)+'</th>').join('')}</tr></thead><tbody>${d.configs.slice(1).map(c=>`<tr><th>${esc(c.label)}</th>${qa.map((q,k)=>`<td class="num">${fmt(d.deltas[c.id][q],k)}</td>`).join('')}</tr>`).join('')}</tbody></table></div><p>음수는 수치 감소입니다. 수신 Agent 내부 비용은 미모델링이며 결합 구성의 성능이나 최종 승자를 뜻하지 않습니다.</p></section>
<section><h2>같은 사례를 직접 비교</h2><label>사례 <select id="case">${spec.scenarios.map(s=>`<option value="${s.id}">${s.id} · ${esc(s.text)}</option>`).join('')}</select></label><label>비교 구성 <select id="config">${d.configs.slice(1).map(c=>`<option value="${c.id}">${esc(c.label)}</option>`).join('')}</select></label><div id="case-detail"></div></section>
<section><h2>해석할 때 지켜야 할 범위</h2><p>모든 수치는 같은 24개 대표 사례 묶음 안에서 비교합니다. 반복 실행 표본이 아니므로 p95나 신뢰구간을 만들지 않았습니다. QA05는 준비·미사용·취소 활동까지 포함한 전체 묶음 합계이고 QA06은 동시 메모리 합의 최댓값입니다. 기존 COMMON-28의 점수표를 새 묶음에 자동 적용하지 않았습니다.</p><p>가정과 입력을 고정한 계산 구조는 준비됐지만 실제 WAV·화면·후보 프로그램 및 채택 출력은 없습니다. 실제 성능을 보고하려면 같은 계약으로 관측값을 수집한 새 판본이 필요합니다. v3는 보존됐습니다.</p></section>
<script id="qa-data" type="application/json">${payload}</script><script>
const data=JSON.parse(document.getElementById('qa-data').textContent);const e=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));const f=(n,k=3)=>n.toLocaleString('en-US',{minimumFractionDigits:k,maximumFractionDigits:k,useGrouping:false});
function renderCase(){const id=document.getElementById('case').value,cfg=document.getElementById('config').value,s=data.scenarios.find(x=>x.id===id),o=data.oracles.find(x=>x.id===id),a=data.cases.find(x=>x.case_id===id&&x.config_id==='BASE'),b=data.cases.find(x=>x.case_id===id&&x.config_id===cfg);let h='<p><b>'+e(s.text)+'</b></p><p>'+e(s.facts)+'</p><div class="overflow"><table><tr><th>구성</th><th>화면 QA01 s</th><th>VIA QA03 s</th><th>QA05 $</th><th>QA06 MB</th><th>첫 음성 s</th></tr>';for(const r of [a,b])h+='<tr><th>'+e(r.config_id)+'</th>'+['qa01','qa03','qa05','qa06','first_voice'].map(k=>'<td class="num">'+f(r[k],k==='qa05'?6:3)+'</td>').join('')+'</tr>';h+='</table></div>';for(const r of [a,b]){h+='<details><summary>'+e(r.config_id)+' 처리·비용·메모리 근거</summary><div class="overflow"><table><tr><th>단계</th><th>내용</th><th>시작 s</th><th>종료 s</th><th>인과 부모</th></tr>'+r.nodes.map(n=>'<tr><td>'+e(n.id)+'</td><td>'+e(n.label)+'</td><td>'+f(n.start_s)+'</td><td>'+f(n.finish_s)+'</td><td>'+e(n.causal_parents.join(', '))+'</td></tr>').join('')+'</table></div><h3>전체 과금 원장</h3><pre>'+e(JSON.stringify(r.usage,null,2))+'</pre><h3>동시 최대 메모리 상태</h3><pre>'+e(JSON.stringify(r.memory_peak,null,2))+'</pre></details>';}h+='<details><summary>평가자 정답 · 후보 출력은 아직 없음</summary><pre>'+e(JSON.stringify(o,null,2))+'</pre></details>';document.getElementById('case-detail').innerHTML=h;}
document.getElementById('case').addEventListener('change',renderCase);document.getElementById('config').addEventListener('change',renderCase);renderCase();
</script></html>`;
await fs.writeFile(path.join(OUT,'guide.html'),html);
const meta={version:d.version,ledger_sha256:crypto.createHash('sha256').update(bytes).digest('hex'),workbook:'VIA-QA-common-v4.xlsx',sheets:names,formula_count:formulaCount,formula_error_scan:'PASS',cost_perturb_restore:'PASS',accuracy_blank_and_complete_probes:'PASS_RESTORED_EMPTY',previews,physical_browser_render:'NOT_COMPLETED',native_excel:'NOT_RUN',product_measurement:false};
await fs.writeFile(path.join(OUT,'render-verification.json'),JSON.stringify(meta,null,2)+'\n');console.log(JSON.stringify({workbookPath,formulaCount,previews:previews.length}));
