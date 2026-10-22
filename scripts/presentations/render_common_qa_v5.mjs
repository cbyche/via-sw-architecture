import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';
const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const DIR=path.join(ROOT,'docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v5');
const OUT=path.join(DIR,'artifacts/outputs/01a15dfe-v5'),PRE=path.join(DIR,'artifacts/previews');
const bytes=await fs.readFile(path.join(DIR,'artifacts/ledger.json')),d=JSON.parse(bytes);
const spec=JSON.parse(await fs.readFile(path.join(DIR,'specification.json'))),oracle=JSON.parse(await fs.readFile(path.join(DIR,'oracles.json')));
const req=createRequire(path.join(process.env.VIA_ARTIFACT_MODULE_BASE??'/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node','entry.cjs'));
const {Workbook,SpreadsheetFile}=await import(pathToFileURL(req.resolve('@oai/artifact-tool')));
const wb=Workbook.create(),names=['전체비교','공통가정','사건계산','목표계산','처리단계','비용원장','메모리상태','변경과제','지칭판정','의미판정','공통입력'];
const sh=Object.fromEntries(names.map(n=>[n,wb.worksheets.add(n)])),tables={};
const col=n=>{let s='';for(n++;n;n=Math.floor((n-1)/26))s=String.fromCharCode(65+(n-1)%26)+s;return s;};
const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const fmt=(v,n=4)=>v==null?'미산출':Number(v.toFixed(12)).toLocaleString('en-US',{minimumFractionDigits:n,maximumFractionDigits:n,useGrouping:false});
function table(name,title,heads,rows,widths,note,height=32){
 const s=sh[name],last=col(heads.length-1),end=5+rows.length;
 s.showGridLines=false;s.getRange(`A1:${last}${end}`).format.font={name:'Arial',size:11,color:'#23344B'};
 s.getRange(`A1:${last}${end}`).format.verticalAlignment='center';
 s.getRange('A2').values=[[title]];s.getRange('A2').format.font={name:'Arial',size:16,bold:true};s.getRange('A2').format.rowHeight=28;
 s.getRange('A3').values=[[note]];s.getRange('A3').format.rowHeight=24;
 s.getRange(`A5:${last}5`).values=[heads];s.getRange(`A5:${last}5`).format={fill:'#253C59',font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},rowHeight:42,wrapText:true,horizontalAlignment:'center',verticalAlignment:'center'};
 if(rows.length){s.getRange(`A6:${last}${end}`).values=rows;s.getRange(`A6:${last}${end}`).format.rowHeight=height;s.getRange(`A6:${last}${end}`).format.wrapText=true;for(let i=6;i<=end;i+=2)s.getRange(`A${i}:${last}${i}`).format.fill='#F1F4F8';}
 widths.forEach((w,i)=>s.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidth=w);
 if(rows.length>15){s.freezePanes.freezeRows(5);s.freezePanes.freezeColumns(2);}
 tables[name]={end,cols:heads.length};return s;
}
const put=(name,c,values)=>sh[name].getRange(`${c}6:${c}${values.length+5}`).formulas=values.map(v=>[v]);
const assumptions=[...Object.entries(spec.profile).map(([k,v])=>['profile',k,v]),...Object.entries(spec.token_budget).map(([k,v])=>['tokens',k,v]),...Object.entries(spec.policy).map(([k,v])=>['policy',k,v])];
const scoreRows={};for(const q of ['qa01','qa03']){scoreRows[q]=[];spec.scores[q+'_upper_bounds'].forEach((b,i)=>{scoreRows[q].push(6+assumptions.length);assumptions.push(['score',q+'_'+(6-i)+'점 상한 초',b]);});}
assumptions.push(['source','정의와 원본','COMMON-QA-MODEL-5.0 specification.json; QA01-VOICE-v2; QA03-COMPLETION-v1; QA08-LOCAL-MEM-v1']);
table('공통가정','공통 환경과 채점 기준',['범위','항목','가정값 또는 규칙'],assumptions,[15,38,115],'시간과 메모리 가정을 바꾸면 계산기를 다시 실행한다. 실측 교정값 아님.',44);
const stages=[],usage=[],memory=[],goals=[],events=[],stageMap=new Map(),usageRange=new Map(),memoryRange=new Map();
for(const c of d.cases){
 const key=c.config_id+'|'+c.case_id,ns=Object.fromEntries(c.nodes.map(n=>[n.id,n]));
 for(const n of c.nodes){stageMap.set(key+'|'+n.id,stages.length+6);stages.push([c.config_id,c.case_id,n.id,n.label,n.kind,n.start_s,n.finish_s,n.duration_s,c.goal_origin_s,n.causal_parents.join(', '),n.exclusion_reason,0,0]);}
 const u=usage.length+6;for(const r of c.usage)usage.push([c.config_id,c.case_id,r.node,r.phase,r.bucket,r.amount,r.rate_usd,0]);usageRange.set(key,[u,usage.length+5]);
 for(const [scope,peak] of [['QA06',c.memory_peak],['QA08',c.qa08_peak]]){const start=memory.length+6;const components=Object.entries(peak.components_mb);if(!components.length)components.push(['추가 모델 없음: 공통 ASR + 원격 모델 구성',0]);for(const [owner,mb] of components)memory.push([c.config_id,c.case_id,scope,peak.time_s,owner,mb]);memoryRange.set(key+'|'+scope,[start,memory.length+5]);}
 goals.push([c.config_id,c.case_id,c.routing,c.goal_origin_s,0,0,0,0,0,c.screen_diagnostic,c.required_channel_nodes.join(', ')]);
 for(const e of c.events){const onset=e.first_audio_s,adopt=ns[e.adoption].finish_s;let after=0;
  if(onset!=null)for(const [a,b] of e.blocked_intervals)after+=Math.max(0,Math.min(b,onset)-Math.max(a,e.source_s,adopt));
  events.push([c.config_id,c.case_id,e.id,e.profile,e.source_s,onset,e.U,e.P,e.overlap,e.L==null?null:e.first_audio_s-e.R,adopt,e.L==null?null:after,e.prepared_during_wait_s,e.status,null,null,null,null,null,e.normal_collision?1:0]);
 }
}
table('처리단계','처리 의존 관계와 완료시간 재계산',['구성','입력','단계','내용','소유','시작 초','종료 초','길이 초','목표 원점','인과 부모','투영 기준','VIA 누적 초','원래 누적 초'],stages,[17,8,26,43,12,12,12,12,12,45,27,15,15],'원래 DAG는 완료 경과시간을 재현한다. 외부 업무·사용자 대기·순수 전달만 0으로 투영한다.',42);
for(const [dest,projected] of [['L',true],['M',false]]){let formulas=[];for(const c of d.cases)for(const n of c.nodes){const r=stageMap.get(c.config_id+'|'+c.case_id+'|'+n.id),parents=n.causal_parents.map(k=>dest+stageMap.get(c.config_id+'|'+c.case_id+'|'+k));formulas.push(`=IF(G${r}<=I${r},0,MAX(0${parents.length?','+parents.join(','):''})+${projected?`IF(E${r}="via",MAX(0,G${r}-MAX(F${r},I${r})),0)`:`MAX(0,G${r}-MAX(F${r},I${r}))`})`);}put('처리단계',dest,formulas);}
table('비용원장','준비·요청·취소까지 포함한 VIA 비용',['구성','입력','단계','범위','과금 종류','사용량','단가 $','비용 $'],usage,[17,8,28,18,19,15,15,17],'비용 = 사용량 × 단가. Agent 내부 비용은 미산출이며 0이 아니다.');put('비용원장','H',usage.map((_,i)=>`=F${i+6}*G${i+6}`));sh['비용원장'].getRange(`F6:H${usage.length+5}`).setNumberFormat('0.000000');
table('메모리상태','QA06 전체와 QA08 추가 모델의 각 최고 시점',['구성','입력','범위','피크 시각 초','중복 없는 소유 자원','MB'],memory,[17,8,10,17,62,17],'QA06과 QA08은 각각 같은 시각 합계. 두 지표를 더하거나 피크끼리 차감하지 않는다.');
table('사건계산','음성 사건별 일반·대기 결과',['구성','입력','사건','사전 분류','시작 초','첫 음성 초','사용자 U','이전음성 P','교집합','마지막 대기 끝','내용 채택','채택 뒤 대기','선행 준비 겹침','상태','실제 L','합집합 B','나머지 E','대기 뒤 R','채택 뒤 D','일반 회귀'],events,[17,8,22,20,12,13,12,13,12,16,13,15,16,31,13,13,13,13,13,13],'NORMAL 평균만 채점. WAIT는 별도 보고. 시각·내용은 분석 가정이며 실제 음성 관측은 0건.',36);
for(const [column,fn] of Object.entries({O:r=>`F${r}-E${r}`,P:r=>`G${r}+H${r}-I${r}`,Q:r=>`O${r}-P${r}`,R:r=>`F${r}-J${r}`,S:r=>`F${r}-MAX(E${r},K${r})-L${r}`}))put('사건계산',column,events.map((_,i)=>`=IF(COUNT(F${i+6})=0,"",${fn(i+6)})`));
sh['사건계산'].getRange(`N6:N${events.length+5}`).format.horizontalAlignment='center';
table('목표계산','모든 목표의 처리 완료와 자원',['구성','입력','역할','최초 원점 초','전체 경과 초','QA03 VIA 초','QA05 $','QA06 MB','QA08 MB','첫 화면 초','필수 채널 수용 노드'],goals,[17,8,18,15,16,16,17,17,17,16,65],'24목표 전체. QA01 NORMAL 선택으로 QA03 목표를 줄이지 않는다. 최종 재생이 아닌 전체 준비 수용까지.',44);
for(let i=0;i<d.cases.length;i++){const c=d.cases[i],r=i+6,key=c.config_id+'|'+c.case_id,t=stageMap.get(key+'|'+c.terminal_node),[us,ue]=usageRange.get(key),[ms,me]=memoryRange.get(key+'|QA06'),[as,ae]=memoryRange.get(key+'|QA08');sh['목표계산'].getRange(`E${r}:I${r}`).formulas=[[`='처리단계'!M${t}`,`='처리단계'!L${t}`,`=SUM('비용원장'!H${us}:H${ue})`,`=SUM('메모리상태'!F${ms}:F${me})`,`=SUM('메모리상태'!F${as}:F${ae})`]];}
table('변경과제','동일 여섯 변경 과제의 모듈 영향',['구성','과제','요청','개념 모듈 집합','개수'],d.changes.map(c=>[c.config_id,c.change_id,c.title,c.modules.join('\n'),c.module_count]),[17,8,44,48,12],'실제 Rust diff가 없는 설계 영향 추정. 모듈 집합을 중복 없이 센다.',90);
function canon(v,p=''){if(Array.isArray(v)){const a=v.map(x=>canon(x,p+'[]'));if(p==='conditions.order')return a;assert(p==='targets'||p==='referents',p);return [...new Set(a.map(x=>JSON.stringify(x)))].sort().map(x=>JSON.parse(x));}if(v&&typeof v==='object')return Object.fromEntries(Object.keys(v).sort().map(k=>[k,canon(v[k],p?p+'.'+k:k)]));return v;}
const refs=[],meanings=[];for(const cfg of d.configs)for(const o of oracle.items){o.referents.forEach((r,i)=>refs.push([cfg.id,o.id,i+1,JSON.stringify(canon(r,'referents')),'','']));meanings.push([cfg.id,o.id,JSON.stringify(canon(o.meaning)),'','']);}
table('지칭판정','QA02 지칭 정답과 관측 출력',['구성','입력','표현','정답 집합','관측 최초 출력','일치 1/0'],refs,[17,8,10,43,43,13],'관측 출력이 전부 비어 있다. 실제 최초 채택 출력33개를 확보하기 전 미산출.',46);put('지칭판정','F',refs.map((_,i)=>`=IF(E${i+6}="","",IF(EXACT(D${i+6},E${i+6}),1,0))`));
table('의미판정','이전 요청 의미 정답과 부분 진단',['구성','입력','정답 전체 의미','관측 최초 의미','전체 일치 1/0'],meanings,[17,8,75,75,16],'QA07 응답 구성·전달까지의 확장은 미평가. 이 입력 해석 진단만으로 QA07을 산출하지 않는다.',88);put('의미판정','E',meanings.map((_,i)=>`=IF(D${i+6}="","",IF(EXACT(C${i+6},D${i+6}),1,0))`));
table('공통입력','U01~U24 입력과 책임 경계',['입력','방식','사용자 요청','역할','VIA 책임','Agent 책임','분담 근거','공통 사실'],spec.scenarios.map(s=>[s.id,s.modality,s.text,s.routing,s.via_work,s.agent_work,s.routing_reason,s.facts]),[8,10,60,18,49,30,60,85],'역할은 공통 시험 가정이다. 실제 자동 routing 결과가 아니다. 모든 입력은 Voice ON.',100);
const cmp=[];for(const [dp,ids] of [['DP2',['DP2-B']],['DP3',['DP3-B-R1','DP3-B-R2','DP3-B-R3']],['DP5',['DP5-B']],['DP44',['DP44-B']]])for(const id of ['BASE',...ids])cmp.push({dp,id,alt:id==='BASE'?'A':id.slice(dp.length+1)});
const main=table('전체비교','네 DP의 공통 QA 비교',['DP','방안','QA01 NORMAL 초','점수 /6','WAIT 총 초','WAIT B 초','WAIT R 초','QA03 초','점수 /6'],cmp.map(x=>[x.dp,x.alt,0,0,0,0,0,0,0]),[11,15,18,13,18,18,18,18,13],'분석 모델 v5 · 실측 아님. NORMAL 36건, WAIT 5출력/1철회, QA03 24목표.',34);
function rating(cell,q){let expr='0';for(let j=5;j>=0;j--)expr=`IF(${cell}<='공통가정'!C${scoreRows[q][j]},${6-j},${expr})`;return `=IF(${cell}="미산출","미산출",${expr})`;}
for(let i=0;i<cmp.length;i++){const x=cmp[i],r=i+6,ci=d.configs.findIndex(c=>c.id===x.id),a=6+ci*42,b=a+41,g=6+ci*24,h=g+23;
 const normal=d.cases.filter(c=>c.config_id===x.id).flatMap(c=>c.events).map((e,j)=>({e,row:a+j})).filter(x=>x.e.profile==='NORMAL').map(x=>'\'사건계산\'!O'+x.row);
 const wait=d.cases.filter(c=>c.config_id===x.id).flatMap(c=>c.events).map((e,j)=>({e,row:a+j})).filter(x=>x.e.profile!=='NORMAL'&&x.e.status!=='WITHDRAWN_BEFORE_AUDIO').map(x=>x.row);
 main.getRange(`C${r}:I${r}`).formulas=[[`=IF(COUNT(${normal.join(',')})<36,"미산출",AVERAGE(${normal.join(',')}))`,rating('C'+r,'qa01'),...['O','P','R'].map(k=>`=IF(COUNT(${wait.map(v=>`'사건계산'!${k}${v}`).join(',')})<5,"미산출",AVERAGE(${wait.map(v=>`'사건계산'!${k}${v}`).join(',')}))`),`=AVERAGE('목표계산'!F${g}:F${h})`,rating('H'+r,'qa03')]];
}
main.getRange('A18').values=[['정확도·변경·비용·메모리']];main.getRange('A18').format.font.bold=true;
main.getRange('A20:I20').values=[['DP','방안','QA02 %','QA04 개','QA05 $','QA06 MB','QA07 %','QA08 MB','QA08 판정']];main.getRange('A20:I20').format={fill:'#253C59',font:{color:'#FFFFFF',bold:true},rowHeight:36,horizontalAlignment:'center'};
cmp.forEach((x,i)=>{const r=i+21,ci=d.configs.findIndex(c=>c.id===x.id),a=6+ci*24,b=a+23,u=6+ci*33,v=u+32,m=6+ci*6;main.getRange(`A${r}:B${r}`).values=[[x.dp,x.alt]];main.getRange(`C${r}:I${r}`).formulas=[[`=IF(COUNT('지칭판정'!F${u}:F${v})<33,"미산출",AVERAGE('지칭판정'!F${u}:F${v})*100)`,`=AVERAGE('변경과제'!E${m}:E${m+5})`,`=SUM('목표계산'!G${a}:G${b})`,`=MAX('목표계산'!H${a}:H${b})`,`="미산출"`,`=MAX('목표계산'!I${a}:I${b})`,`="실측 미판정"`]];});
main.getRange('A21:I30').format.rowHeight=32;main.getRange('A18:I33').format.font={name:'Arial',size:11,color:'#23344B'};main.getRange('A20:I20').format.font.color='#FFFFFF';main.getRange('A32').values=[['QA08은 추가 모델 범위. QA02/07은 관측 없음. QA04는 개념 모듈 추정. Agent 비용은 미산출.']];
main.getRange('C21:C30').format.horizontalAlignment='center';main.getRange('G21:G30').format.horizontalAlignment='center';main.getRange('I21:I30').format.horizontalAlignment='center';
main.getRange('A34').values=[['대기 종류별 집계 · 별점 없음. E와 R은 서로 다른 진단이다.']];
main.getRange('A35:I35').values=[['구성','대기 분류','예정','출력','철회','L 초','B 초','E 초','R 초']];
main.getRange('A35:I35').format={fill:'#253C59',font:{name:'Arial',size:11,color:'#FFFFFF',bold:true},rowHeight:40,horizontalAlignment:'center',wrapText:true};
d.configs.forEach((cfg,ci)=>['WAIT_USER','WAIT_PREVIOUS','WAIT_BOTH'].forEach((group,gi)=>{const r=36+ci*3+gi,a=6+ci*42,b=a+41,dr=`'사건계산'!D${a}:D${b}`,sr=`'사건계산'!N${a}:N${b}`;
 main.getRange(`A${r}:B${r}`).values=[[cfg.id,group]];
 main.getRange(`C${r}:I${r}`).formulas=[[`=COUNTIF(${dr},B${r})`,`=COUNTIFS(${dr},B${r},${sr},"ANALYTICAL_ONSET")`,`=COUNTIFS(${dr},B${r},${sr},"WITHDRAWN_BEFORE_AUDIO")`,...['O','P','Q','R'].map(k=>`=IF(D${r}=0,"미산출",IF(D${r}+E${r}<C${r},"미산출",SUMIF(${dr},B${r},'사건계산'!${k}${a}:${k}${b})/D${r}))`)]];
}));
main.getRange('A36:I56').format={font:{name:'Arial',size:11,color:'#23344B'},rowHeight:40,wrapText:true};main.getRange('F36:I56').setNumberFormat('0.0000');main.getRange('F36:I56').format.horizontalAlignment='right';
main.getRange('A34:I56').format.columnWidth=18;main.getRange('B35:B56').format.columnWidth=22;
main.getRange('A33').values=[['점수는 반올림 전 평균으로 계산한다. 표시 소수 자릿수는 실측 정밀도를 뜻하지 않는다.']];
main.getRange('C6:I15').setNumberFormat('0.0000');main.getRange('D6:D15').setNumberFormat('0');main.getRange('I6:I15').setNumberFormat('0');main.getRange('C21:H30').setNumberFormat('0.000');main.getRange('E21:E30').setNumberFormat('0.000000');
for(const [name,range] of [['사건계산',`E6:M${events.length+5}`],['사건계산',`O6:S${events.length+5}`],['목표계산',`D6:J${goals.length+5}`],['처리단계',`F6:I${stages.length+5}`],['처리단계',`L6:M${stages.length+5}`],['메모리상태',`F6:F${memory.length+5}`]])sh[name].getRange(range).setNumberFormat('0.000');sh['목표계산'].getRange(`G6:G${goals.length+5}`).setNumberFormat('0.000000');
const near=(a,b)=>assert(Math.abs(a-b)<1e-7,`${a} != ${b}`);
for(let i=0;i<cmp.length;i++){const s=d.summary[cmp[i].id],r=i+6;const expected=[s.qa01,s.qa01_score,s.wait.L,s.wait.B,s.wait.R,s.qa03,s.qa03_score];main.getRange(`C${r}:I${r}`).values[0].forEach((v,j)=>near(v,expected[j]));}
// Reversible authoring probes, never retained as observations.
const before=main.getRange('C6').values[0][0],onset=sh['사건계산'].getRange('F6').values[0][0];sh['사건계산'].getRange('F6').values=[[onset+1]];near(main.getRange('C6').values[0][0],before+1/36);sh['사건계산'].getRange('F6').values=[[0]];near(main.getRange('C6').values[0][0],before-onset/36);sh['사건계산'].getRange('F6').clear({applyTo:'contents'});assert.equal(main.getRange('C6').values[0][0],'미산출');sh['사건계산'].getRange('F6').values=[[onset]];
const cost=main.getRange('E21').values[0][0],rate=sh['비용원장'].getRange('G6').values[0][0],amount=sh['비용원장'].getRange('F6').values[0][0];sh['비용원장'].getRange('G6').values=[[rate*2]];near(main.getRange('E21').values[0][0],cost+rate*amount);sh['비용원장'].getRange('G6').values=[[rate]];
for(let r=6;r<=38;r++)sh['지칭판정'].getRange(`E${r}`).values=[[sh['지칭판정'].getRange(`D${r}`).values[0][0]]];near(main.getRange('C21').values[0][0],100);sh['지칭판정'].getRange('E6').values=[['wrong']];near(main.getRange('C21').values[0][0],32/33*100);sh['지칭판정'].getRange('E6:E38').clear({applyTo:'contents'});
wb.recalculate();assert.equal(main.getRange('C21').values[0][0],'미산출');assert.equal(main.getRange('G21').values[0][0],'미산출');near(main.getRange('E21').values[0][0],cost);
for(const [name,t] of Object.entries(tables))for(const row of sh[name].getRange(`A1:${col(t.cols-1)}${name==='전체비교'?56:t.end}`).values)for(const v of row)assert(!/^#(REF!|DIV\/0!|VALUE!|NAME\?|N\/A|NUM!|NULL!|SPILL!|CALC!)/.test(String(v)),name+':'+v);
console.log((await wb.inspect({kind:'region',sheetId:'전체비교',range:'A5:I15',maxChars:1800,tableMaxRows:11,tableMaxCols:9})).ndjson);
await fs.mkdir(OUT,{recursive:true});await fs.mkdir(PRE,{recursive:true});
const book=path.join(OUT,'VIA-QA-common-v5.xlsx');await(await SpreadsheetFile.exportXlsx(wb)).save(book);await fs.rm(book+'.inspect.ndjson',{force:true});
const previews=[];for(const n of names){const t=tables[n];const ranges=n==='사건계산'?['A1:J10','K5:T10']:n==='전체비교'?['A1:I33','A34:I56']:[`A1:${col(t.cols-1)}${Math.min(t.end,10)}`];for(let i=0;i<ranges.length;i++){const filename=n+(i?'-detail':'')+'.png';const im=await wb.render({sheetName:n,range:ranges[i],scale:1,format:'png'});await fs.writeFile(path.join(PRE,filename),new Uint8Array(await im.arrayBuffer()));previews.push(filename);}}
const summaryHeaders=['DP','방안','QA01 NORMAL 초','점수 /6','WAIT 총 초','WAIT B 초','WAIT R 초','QA03 초','점수 /6','QA02 %','QA04 개','QA05 $','QA06 MB','QA07 %','QA08 MB'];
const htmlTable=(heads,rows)=>`<div class="scroll"><table><thead><tr>${heads.map(h=>'<th>'+esc(h)+'</th>').join('')}</tr></thead><tbody>${rows.map(r=>'<tr>'+r.map(v=>'<td>'+esc(v)+'</td>').join('')+'</tr>').join('')}</tbody></table></div>`;
const comparisons=cmp.map(x=>{const s=d.summary[x.id];return [x.dp,x.alt,fmt(s.qa01),s.qa01_score,fmt(s.wait.L),fmt(s.wait.B),fmt(s.wait.R),fmt(s.qa03),s.qa03_score,'미산출',fmt(s.qa04),fmt(s.qa05,6),fmt(s.qa06),'미산출',fmt(s.qa08)];});
const waits=d.configs.flatMap(c=>Object.entries(d.summary[c.id].wait_profiles).map(([k,w])=>[c.id,k,`${w.scheduled}/${w.onsets}/${w.withdrawn}`,...['L','U','P','overlap','B','E','R','D','prepared_during_wait_s'].map(q=>fmt(w[q]))]));
const payload=JSON.stringify(d).replace(/</g,'\\u003c');
const html=`<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>공통 QA v5 전체 비교</title><style>body{font:16px/1.6 system-ui,sans-serif;color:#20334b;background:#f3f5f8;max-width:1600px;margin:28px auto;padding:0 24px}section{background:white;padding:24px;margin:24px 0;border-radius:8px}h1{font-size:30px}h2{font-size:22px}.scroll{overflow:auto}table{border-collapse:collapse;width:100%;font-size:14px}td,th{padding:10px;border-bottom:1px solid #dce3eb;text-align:left;vertical-align:top}th{background:#e7edf4}select{font:inherit;padding:8px;max-width:100%}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-size:13px}summary{cursor:pointer;font-weight:600}.note{background:#fff0cb;padding:12px}a{color:#1b5c91}</style><h1>네 DP의 공통 QA 비교 v5</h1><p>분석 모델과 조건부 점수 · 실제 측정값 아님 · QA01-VOICE-v2 / QA03-COMPLETION-v1</p><p><a href="VIA-QA-common-v5.xlsx">Excel 열기</a> · <a href="../../../README.md">계약과 검증</a></p>
<section><h2>모든 방안의 수치</h2><p>NORMAL 36건과 WAIT 6예정(5출력·1철회)을 구분합니다. QA03은 24목표 전체입니다. 네 A행은 동일 BASE를 다시 보여줍니다.</p>${htmlTable(summaryHeaders,comparisons)}<p class="note">QA02/07은 실제 후보 출력이 없어 미산출입니다. QA04는 개념 모듈 영향 추정이며 QA08은 24사례 안의 추가 모델 메모리 추정입니다. 새 QA07 응답 적합성은 미평가이며 기존 의미 정답은 부분 진단입니다. 실제 제품 합격 판정과 QA08 점수는 없습니다.</p></section>
<section><h2>대기 상황</h2>${htmlTable(['구성','사전 분류','예정/출력/철회','L 초','U 초','P 초','교집합','B 초','E 초','R 초','D 초','준비 겹침 초'],waits)}<p>B는 사용자 발화와 앞 음성 재생의 합집합입니다. E는 대기가 아닌 전체 구간, R은 마지막 대기 뒤 지연입니다. WAIT_BOTH는 예정0으로 미산출입니다. WAIT에 별점을 부여하거나 E를 일반 상황 값으로 대신 쓰지 않습니다.</p></section>
<section><h2>입력별 책임과 계산 근거</h2><label>입력 <select id="case">${spec.scenarios.map(s=>`<option value="${s.id}">${s.id} ${esc(s.text)}</option>`).join('')}</select></label><label>비교 구성 <select id="config">${d.configs.slice(1).map(c=>`<option value="${c.id}">${esc(c.label)}</option>`).join('')}</select></label><div id="detail"></div></section>
<section><h2>목표와 해석 범위</h2><p>QA01 목표2초는 연구의4초 참조 대비50% 단축이라는 사용자 합의 목표입니다. 연구의 달성 성과가 아닙니다. QA03 목표5초는 Windows Long wait 상단을 VIA 자체 처리 예산으로 채택했습니다. 모두1초 우수선까지 균등 구간을 적용합니다.</p><p>QA08은 추가 모델에 귀속되는 runtime·weights·workspace·임시 적재·IPC·보존 상태의 같은 시각 합계입니다. QA06 전체 RAM과 더하지 않습니다. 추가 모델 없는 구성의0MB는 공통 ASR과 원격 의미/음성 모델이라는 명세 범위입니다. 최대 지원 부하와5회 장치 실행은 수행하지 않았습니다. 기존 v4와 발표 v10은 보존했습니다.</p></section>
<script id="qa-data" type="application/json">${payload}</script><script>
const data=JSON.parse(document.getElementById('qa-data').textContent),e=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])),f=v=>v==null?'미산출':v.toFixed(3);
function renderCase(){const id=document.getElementById('case').value,cfg=document.getElementById('config').value,s=data.scenarios.find(x=>x.id===id);let h='<p><b>'+e(s.text)+'</b></p><p>역할: '+e(s.routing)+' · VIA: '+e(s.via_work)+' · Agent: '+e(s.agent_work)+'</p><p>'+e(s.routing_reason)+'</p><p>'+e(s.facts)+'</p>';for(const cid of ['BASE',cfg]){const c=data.cases.find(x=>x.case_id===id&&x.config_id===cid);h+='<h3>'+e(cid)+'</h3><p>QA03 '+f(c.qa03)+'초 · QA06 '+f(c.qa06)+'MB · QA08 '+f(c.qa08)+'MB</p><div class="scroll"><table><tr><th>사건</th><th>분류</th><th>상태</th><th>외부/입력 시작</th><th>VIA 수신</th><th>첫 음성</th><th>L</th><th>B</th><th>E</th><th>R</th><th>대응 구간</th></tr>'+c.events.map(x=>'<tr><td>'+e(x.id)+'</td><td>'+e(x.profile)+'</td><td>'+e(x.status)+'</td><td>'+f(x.source_s)+'</td><td>'+f(x.received_s)+'</td><td>'+f(x.first_audio_s)+'</td><td>'+f(x.L)+'</td><td>'+f(x.B)+'</td><td>'+f(x.E)+'</td><td>'+f(x.R)+'</td><td>'+e(x.segment_id)+'</td></tr>').join('')+'</table></div><details><summary>완료 의무·단계·비용·메모리 원장</summary><pre>'+e(JSON.stringify({required_state:c.required_state_nodes,required_external:c.required_external_nodes,required_channels:c.required_channel_nodes,segments:c.segments,nodes:c.nodes,usage:c.usage,qa06_peak:c.memory_peak,qa08_peak:c.qa08_peak,timing_overlap:c.timing_overlap,events:c.events},null,2))+'</pre></details>';}document.getElementById('detail').innerHTML=h;}
document.getElementById('case').addEventListener('change',renderCase);document.getElementById('config').addEventListener('change',renderCase);renderCase();
</script></html>`;
await fs.writeFile(path.join(OUT,'guide.html'),html);
await fs.writeFile(path.join(OUT,'render-verification.json'),JSON.stringify({version:d.version,ledger_sha256:crypto.createHash('sha256').update(bytes).digest('hex'),sheets:names,formula_error_scan:'PASS',onset_blank_and_cost_recalculation:'PASS_RESTORED',accuracy_probe:'PASS_RESTORED_EMPTY',previews,native_excel:'NOT_RUN',physical_browser_render:'NOT_COMPLETED',product_measurement:false},null,2)+'\n');
console.log(JSON.stringify({workbook:book,previews:previews.length}));
