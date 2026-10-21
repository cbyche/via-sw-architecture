// Presentation adapter only. Preserves three separate authored profiles and their values.
import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {fileURLToPath,pathToFileURL} from 'node:url';

const ROOT=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
const OUT=path.join(ROOT,'docs/presentations_files/qa-common-synthetic-v3-20261022');
const DEL=path.join(OUT,'outputs/01a15dfe');
const PREVIEW=path.join(OUT,'previews');
const sourceBytes=await fs.readFile(path.join(OUT,'normalized-data.json'));
const d=JSON.parse(sourceBytes);
assert.equal(d.aggregation_policy,'WITHIN_PROFILE_ONLY_NO_POOLED_HEADLINE');
assert.equal(d.configs.length,9);
const KIND=d.evidence_kind;
const canon=v=>Array.isArray(v)?v.map(canon).sort((a,b)=>JSON.stringify(a).localeCompare(JSON.stringify(b))):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).sort(([a],[b])=>a.localeCompare(b)).map(([k,x])=>[k,canon(x)])):v;
const json=v=>JSON.stringify(canon(v));
const ordered=v=>Array.isArray(v)?v.map(ordered):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).sort(([a],[b])=>a.localeCompare(b)).map(([k,x])=>[k,ordered(x)])):v;
const meaningJson=(value,stratum)=>stratum==='DELIVERY'?JSON.stringify(ordered(value)):json(value);
assert.notEqual(meaningJson({reading_order:['first','second']},'DELIVERY'),meaningJson({reading_order:['second','first']},'DELIVERY'));
assert.equal(json(['first','second']),json(['second','first']));
const col=n=>{let s='';for(n++;n;n=Math.floor((n-1)/26))s=String.fromCharCode(65+(n-1)%26)+s;return s;};
const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const labels={'T-BASE':'DP2 A + DP5 A','T-EXEC':'DP2 B + DP5 A','T-REL':'DP2 A + DP5 B','V-A':'DP3 A 원본 직접 해석','V-R1':'DP3 B-R1 모델 회수','V-R2':'DP3 B-R2 모델 상주','V-R3':'DP3 B-R3 worker 분리','D-A':'DP44 A 응답별 생성 유지','D-B':'DP44 B 현재 발화에 생성 연결'};
const qaLabels=['QA01 화면 첫 응답','QA02 지칭 집합','QA03 VIA 처리','QA04 예상 변경 모듈','QA05 VIA 비용','QA06 최대 RAM','QA07 요청 의미'];
const units=['초','정확도','초','모듈','USD/활동 묶음','MB','정확도'];
const fmt=(v,k)=>v===null||v===undefined?'미모델링':[1,6].includes(k)?(v*100).toFixed(2)+'%':k===4?'$'+v.toFixed(6):v.toFixed(k===3?3:3);
const scenarios=new Map(d.scenarios.map(s=>[s.case_id,s]));
const inputText=r=>{const s=scenarios.get(r.case_id);return s?.inputs.find(i=>r.input_id.endsWith(i.id))?.text??'';};

// Quantity and price are retained as separate editable columns. Aggregate only equal buckets/rates.
const usageMap=new Map();
for(const u of d.usage){const key=[u.config_id,u.phase,u.role,u.bucket,u.rate_usd].join('|');let g=usageMap.get(key);if(!g){g={config_id:u.config_id,stratum:u.stratum,phase:u.phase,role:u.role,bucket:u.bucket,rate_usd:u.rate_usd,amount:0,n:0,ids:[]};usageMap.set(key,g);}g.amount+=u.amount;g.n++;g.ids.push(u.id);}
const usage=[...usageMap.values()];
// One true simultaneous peak per case/config preserves every possible global maximum.
const peakMap=new Map();
for(const m of d.memory){const key=m.config_id+'|'+m.case_id;const old=peakMap.get(key);if(!old||m.total_mb>old.total_mb)peakMap.set(key,m);}
const memory=[...peakMap.values()];
const components=[...new Set(memory.flatMap(m=>Object.keys(m.components_mb)))];
const req=createRequire(path.join(process.env.VIA_ARTIFACT_MODULE_BASE??'/private/tmp/via-qa-delivery-v3-01a15dfe','entry.cjs'));
const {Workbook,SpreadsheetFile}=await import(pathToFileURL(req.resolve('@oai/artifact-tool')));
const wb=Workbook.create();
const names=['비교요약','계산기준','첫응답','지칭판정','의미판정','목표시간','비용계산','메모리상태','변경과제'];
const sheets=Object.fromEntries(names.map(n=>[n,wb.worksheets.add(n)]));
const tables={};
const navy='#263C55',ink='#233548',muted='#627185';
function table(name,title,headers,rows,widths,{height=34,context='가상 작성 원장. 실제 모델 또는 제품 실행 기록이 아니다.'}={}){
 const sh=sheets[name],last=col(headers.length-1),end=rows.length+5;
 sh.showGridLines=false;
 sh.getRange(`A1:${last}${Math.max(end,8)}`).format.font={name:'Arial',size:10,color:ink};
 sh.getRange('A2').values=[[title]];sh.getRange('A2').format.font={name:'Arial',size:15,bold:true,color:navy};
 sh.getRange('A3').values=[[context]];sh.getRange('A3').format.font={size:10,color:muted};
 sh.getRange(`A5:${last}5`).values=[headers];sh.getRange(`A5:${last}5`).format={fill:navy,font:{name:'Arial',size:10,bold:true,color:'#FFFFFF'},rowHeight:35,horizontalAlignment:'center',verticalAlignment:'center',wrapText:true};
 sh.getRange(`A6:${last}${end}`).values=rows;
 sh.getRange(`A6:${last}${end}`).format={rowHeight:height,verticalAlignment:'top',wrapText:true};
 widths.forEach((w,i)=>sh.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidthPx=w);
 sh.freezePanes.freezeRows(5);sh.freezePanes.freezeColumns(2);
 sh.tables.add(`A5:${last}${end}`,true,`Stratified${names.indexOf(name)}`);
 tables[name]={last,end};return sh;
}
const range=(name,c)=>`'${name}'!$${c}$6:$${c}$${tables[name].end}`;
table('첫응답','화면 첫 응답과 별도 음성 진단',['구성','층','사례','입력 ID','유형','revision','입력 t','첫 표시 t','QA01 화면 초','사용자 입력','최초 표시 내용','첫 음성 t','음성 진단 초','음성 근거 상태'],d.responses.map(r=>[r.config_id,r.stratum,r.case_id,r.input_id,r.family_id,r.revision,r.start_s,r.first_display_s,null,inputText(r),r.first_content,r.qa01_voice_s==null?null:(r.first_voice_s??r.start_s+r.qa01_voice_s),null,r.qa01_voice_s==null?'NOT_MODELED':'AUTHORED_AVAILABLE']),[95,90,190,245,85,75,100,100,110,630,690,110,110,190],{height:85,context:'QA01 대표값은 화면 표시다. 음성은 DELIVERY만 별도 계산하며 나머지 층의 빈칸은 NOT_MODELED다.'});
sheets['첫응답'].getRange(`I6:I${tables['첫응답'].end}`).formulas=d.responses.map((r,i)=>[`=H${i+6}-G${i+6}`]);
sheets['첫응답'].getRange(`G6:I${tables['첫응답'].end}`).setNumberFormat('0.000');
// Unsupported blank-return IF cells are unnecessary: unmodeled source rows remain blank.
for(const [i,r]of d.responses.entries())if(r.qa01_voice_s!==null&&r.qa01_voice_s!==undefined)sheets['첫응답'].getRange(`M${i+6}`).formulas=[[`=L${i+6}-G${i+6}`]];
sheets['첫응답'].getRange(`L6:M${tables['첫응답'].end}`).setNumberFormat('0.000');
table('지칭판정','표현별 최초 지칭 집합',['구성','층','입력 ID','표현 ID','정답 집합','채택 집합','QA02 성공'],d.referents.map(r=>[r.config_id,r.stratum,r.input_id,r.expression_id,json(r.expected),json(r.adopted),null]),[95,75,260,285,470,470,95],{height:55,context:'정답과 채택 집합의 순서를 정규화해 완전 일치 여부를 계산한다.'});
sheets['지칭판정'].getRange(`G6:G${tables['지칭판정'].end}`).formulas=d.referents.map((r,i)=>[`=IF(E${i+6}=F${i+6},1,0)`]);
table('의미판정','입력별 최초 요청 의미',['구성','층','입력 ID','정답 의미','채택 의미','QA07 성공','오류 설명'],d.responses.map(r=>[r.config_id,r.stratum,r.input_id,meaningJson(r.expected,r.stratum),meaningJson(r.adopted,r.stratum),null,r.error_reason||'필수 의미 전체 일치']),[95,75,260,760,760,95,410],{height:145,context:'입력별 필수 의미 전체 일치다. DELIVERY 배열 순서를 보존한다. QA02와 같은 대상 오류가 중복될 수 있다.'});
sheets['의미판정'].getRange(`F6:F${tables['의미판정'].end}`).formulas=d.responses.map((r,i)=>[`=IF(D${i+6}=E${i+6},1,0)`]);
table('목표시간','원래 목표의 VIA 처리 경로',['구성','층','사례','목표 ID','VIA 초','전체 경과 초','경과와 투영의 차이','최종 의미 일치','종료 종류','QA03 종료 노드','입력 ID','전체 terminal 목록'],d.goals.map(r=>[r.config_id,r.stratum,r.case_id,r.goal_id,r.via_s,r.wall_s,null,r.terminal_content_correct,r.status,r.terminal_node,r.input_ids.join(', '),(r.terminal_nodes??[r.terminal_node]).join(', ')]),[95,90,190,265,105,115,140,105,210,340,480,480],{height:52,context:'VIA 초는 외부 duration=0인 인과 DAG 투영값. 경과시간과 같은 단축량이 아니다. DELIVERY는 원래 보고서 10+새 목표 80이다.'});
sheets['목표시간'].getRange(`G6:G${tables['목표시간'].end}`).formulas=d.goals.map((r,i)=>[`=F${i+6}-E${i+6}`]);
sheets['목표시간'].getRange(`E6:G${tables['목표시간'].end}`).setNumberFormat('0.000');
table('비용계산','같은 비용 bucket의 사용량과 단가',['구성','층','구간','역할','bucket','합계 사용량','USD/단위','비용 USD','원장 행 수','원본 위치'],usage.map(u=>[u.config_id,u.stratum,u.phase,u.role,u.bucket,u.amount,u.rate_usd,null,u.n,'normalized-data.json#/usage']),[95,75,125,105,180,130,130,130,100,315],{height:36,context:'같은 구성, 구간, 역할, bucket, 단가만 합쳤다. 단가와 사용량을 바꾸면 요약 비용도 바뀐다.'});
sheets['비용계산'].getRange(`H6:H${tables['비용계산'].end}`).formulas=usage.map((u,i)=>[`=F${i+6}*G${i+6}`]);
sheets['비용계산'].getRange(`G6:H${tables['비용계산'].end}`).setNumberFormat('$0.000000');
sheets['비용계산'].getRange(`F6:G${tables['비용계산'].end}`).format.font={color:'#1E5D96'};
table('메모리상태','사례별 최대 동시 RAM 상태',['구성','층','사례','상태 ID','가상 t','구간',...components,'동시 합계 MB'],memory.map(m=>[m.config_id,m.stratum,m.case_id,m.id,m.time_s,m.phase,...components.map(k=>m.components_mb[k]??0),null]),[95,75,190,150,100,135,...components.map(()=>110),125],{height:36,context:'각 사례와 구성의 실제로 작성된 동일 시점 peak 상태다. TEXT 238MB, VOICE와 DELIVERY 각 560MB 기반이지만 서로 다른 profile이다.'});
const memLast=col(6+components.length),memFinal=col(5+components.length);
sheets['메모리상태'].getRange(`${memLast}6:${memLast}${tables['메모리상태'].end}`).formulas=memory.map((m,i)=>[`=SUM(G${i+6}:${memFinal}${i+6})`]);
sheets['메모리상태'].getRange(`E6:${memLast}${tables['메모리상태'].end}`).setNumberFormat('0.00');
table('변경과제','공통 C01~C06 변경의 예상 운영 모듈',['구성','층','과제','변경 요구','예상 모듈 수','모듈 목록','수용 조건','범위 이유','근거 종류'],d.changes.map(c=>[c.config_id,c.stratum,c.change_id,c.title,c.module_count,c.modules.join('\n'),c.acceptance,c.reason,c.evidence_kind]),[95,75,70,320,110,440,680,600,300],{height:130,context:'concept:: 식별자는 설계 예상이다. 실제 Rust diff나 변경 수행 결과가 아니다.'});
const sourceRows=[];
for(const [stratum,p]of Object.entries(d.profiles)){
 sourceRows.push([stratum,'profile',p.id,'고정',p.parent_dataset]);
 sourceRows.push([stratum,'QA01 시작',p.start_event,'고정','TEXT 제출과 발화 종료는 서로 다른 시작 사건']);
 for(const k of ['episodes','inputs','referents','goals','prior_user_activities'])sourceRows.push([stratum,k,p[k],'고정','평가 준비 활동은 QA01/02/03/07 분모 밖, 비용과 RAM 포함']);
 for(const [k,v]of Object.entries(p.assumptions??{}))sourceRows.push([stratum,k,typeof v==='object'?JSON.stringify(v):v,'작성 가정','부모 profile 보존']);
}
for(const c of d.configs)sourceRows.push([c.stratum,c.id,labels[c.id],'구성',`DP2=${c.dp2}; DP3=${c.dp3}; DP5=${c.dp5}; DP44=${c.dp44}`]);
for(const p of d.parent_sources)sourceRows.push([p.stratum,'원본 SHA-256',p.sha256,'보존본',p.snapshot_path]);
sourceRows.push(['공통','근거 종류',KIND,'고정','통합 평균, 층 사이 절대 순위, 실측 차이 주장을 생성하지 않음']);
sourceRows.push(['DELIVERY','QA02/07','FIXED_CORRECT_SEMANTIC_FIXTURE','제어 fixture','같은 정답 의미를 고정한 제어 fixture. 모델 정확도나 차별화 coverage 근거가 아니다.']);
sourceRows.push(['DELIVERY','QA04','DESIGN_CHANGE_REVIEW_NOT_RUST_DIFF','예상','공통 인터페이스에 따라 A도 2.00이 되어 B와 동점 가능. 실제 Rust diff가 아니다.']);
sourceRows.push(['DELIVERY','receiver / total','NOT_MODELED','결측','수신 Agent 내부 비용 미모델링. 합계 0 또는 VIA만의 전체 비용으로 채우지 않는다.']);
table('계산기준','세 profile과 계산 근거',['층','항목','값','상태','설명 및 출처'],sourceRows,[90,300,760,135,730],{height:52,context:'각 profile 안에서만 비교한다. VOICE와 DELIVERY의 주변 DP는 FIXED_UNMAPPED이며 기존 A/B로 자동 대응하지 않는다.'});

const sh=sheets['비교요약'];sh.showGridLines=false;sh.tabColor=navy;
sh.getRange('A1:H68').format.font={name:'Arial',size:11,color:ink};
sh.getRange('A2').values=[['DP 공통 QA 비교 v3']];sh.getRange('A2').format.font={size:16,bold:true,color:navy};
sh.getRange('A3').values=[['TEXT, VOICE, DELIVERY를 같은 층 안에서 비교한다. QA01은 화면이고 음성은 별도 진단이다.']];
sh.getRange('A4').values=[[`근거: ${KIND}`]];sh.getRange('A4').format.font={size:10,color:muted};
const summaryCells=[];
const summaryFormula=(id,q)=>{
 if(q===0)return `=AVERAGEIFS(${range('첫응답','I')},${range('첫응답','A')},"${id}")`;
 if(q===1)return `=AVERAGEIFS(${range('지칭판정','G')},${range('지칭판정','A')},"${id}")`;
 if(q===2)return `=AVERAGEIFS(${range('목표시간','E')},${range('목표시간','A')},"${id}")`;
 if(q===3)return `=AVERAGEIFS(${range('변경과제','E')},${range('변경과제','A')},"${id}")`;
 if(q===4)return `=SUMIFS(${range('비용계산','H')},${range('비용계산','A')},"${id}",${range('비용계산','D')},"VIA")`;
 if(q===5)return `=_xlfn.MAXIFS(${range('메모리상태',memLast)},${range('메모리상태','A')},"${id}")`;
 return `=AVERAGEIFS(${range('의미판정','F')},${range('의미판정','A')},"${id}")`;
};
function summaryBlock(stratum,start){
 const configs=d.configs.filter(c=>c.stratum===stratum),voice=stratum==='VOICE',delivery=stratum==='DELIVERY';
 sh.getRange(`A${start}`).values=[[delivery?'DELIVERY 음성 전달: DP44':voice?'VOICE 화면: DP3':'TEXT 이력: DP2와 DP5']];sh.getRange(`A${start}`).format.font={bold:true,color:navy};
 const header=start+2;
 sh.getRange(`A${header}:H${header}`).values=[['QA','단위',...configs.map(c=>c.id),...Array(4-configs.length).fill(''),'분모/관측 범위','해석']];
 sh.getRange(`A${header}:H${header}`).format={fill:navy,font:{name:'Arial',size:10,color:'#FFFFFF',bold:true},rowHeight:34,horizontalAlignment:'center',verticalAlignment:'center',wrapText:true};
 for(let q=0;q<7;q++){
  const row=header+q+1;
  const denom=delivery?['80 입력','120 표현','90 목표','6 과제','80 평가 + 80 준비','전체 동시 구간','80 입력'][q]:[voice?'120 입력':'240 입력',voice?'120 표현':'250 표현',voice?'120 목표':'180 목표','6 과제',voice?'120 평가 + 60 준비':'120 에피소드','전체 동시 구간',voice?'120 입력':'240 입력'][q];
  const note=delivery?['화면 표시. 음성과 구분','고정 정답 제어 fixture','외부 duration=0 인과 경로','예상. 인터페이스 동점 가능','준비/폐기/재개 문맥 포함','로컬 동시 합계. decimal MB','고정 정답 제어 fixture'][q]:['구체적인 최초 채택 표시','표현별 완전 일치','외부 Agent 업무 제외','설계 예상 운영 모듈','준비와 후속 갱신 포함','같은 시점의 합계 중 최대','필수 의미 전체 일치'][q];
  sh.getRange(`A${row}:H${row}`).values=[[qaLabels[q],units[q],null,null,null,null,denom,note]];
  configs.forEach((c,i)=>{const address=`${col(i+2)}${row}`;sh.getRange(address).formulas=[[summaryFormula(c.id,q)]];sh.getRange(address).setNumberFormat([1,6].includes(q)?'0.00%':q===4?'$0.000000':'0.000');summaryCells.push({address,config:c.id,qa:`qa0${q+1}`,expected:d.summary[c.id][`qa0${q+1}`]});});
 }
 sh.getRange(`A${header+1}:H${header+7}`).format={rowHeight:38,verticalAlignment:'center',wrapText:true};
}
summaryBlock('TEXT',6);summaryBlock('VOICE',18);summaryBlock('DELIVERY',30);
sh.getRange('A41:H41').values=[['QA01 음성 진단','초',null,null,null,null,'80 입력','화면 QA01과 평균/대체 안 함']];
const voiceCells=[];
for(const [i,c]of d.configs.filter(c=>c.stratum==='DELIVERY').entries()){const address=`${col(i+2)}41`;sh.getRange(address).formulas=[[`=SUMIFS(${range('첫응답','M')},${range('첫응답','A')},"${c.id}")/COUNTIFS(${range('첫응답','A')},"${c.id}")`]];sh.getRange(address).setNumberFormat('0.000');voiceCells.push({address,config:c.id,qa:'qa01_voice',expected:d.summary[c.id].qa01_voice});}
sh.getRange('A41:H41').format={rowHeight:38,verticalAlignment:'center',wrapText:true};
sh.getRange('A43').values=[['비용과 분모를 함께 읽기']];sh.getRange('A43').format.font={bold:true,color:navy};
sh.getRange('A45:H45').values=[['구성','평가 VIA USD','준비 VIA USD','유지 VIA USD','VIA 총 USD','수신 Agent USD','전체 USD','활동 묶음']];
sh.getRange('A45:H45').format={fill:navy,font:{name:'Arial',size:10,bold:true,color:'#FFFFFF'},rowHeight:36,wrapText:true,horizontalAlignment:'center'};
for(const [i,c]of d.configs.entries()){
 const r=i+46;
 sh.getRange(`A${r}:H${r}`).values=[[c.id,null,null,null,null,null,null,c.stratum==='TEXT'?'120 에피소드':c.stratum==='VOICE'?'120 평가 + 60 준비':'80 평가 + 80 준비; 수신/전체 미모델링']];
 const base=`${range('비용계산','H')},${range('비용계산','A')},A${r}`;
 sh.getRange(`B${r}:E${r}`).formulas=[[...['scored','preparation','maintenance'].map(p=>`=SUMIFS(${base},${range('비용계산','C')},"${p}",${range('비용계산','D')},"VIA")`),`=SUM(B${r}:D${r})`]];
 if(d.summary[c.id].receiver_cost!==null)sh.getRange(`F${r}:G${r}`).formulas=[[`=SUMIFS(${base},${range('비용계산','D')},"RECEIVER")`,`=SUM(E${r}:F${r})`]];
}
sh.getRange('B46:G54').setNumberFormat('$0.000000');sh.getRange('A46:H54').format={rowHeight:38,wrapText:true,verticalAlignment:'center'};
const notes=[['개별 조건부 권고','DP2 B, DP3 A, DP5 B, DP44 A. 각 층의 독립 비교에 대한 권고이며 결합 구성의 성능 권고가 아니다.'],['구성','T-BASE는 두 비교의 같은 기준 행. T-EXEC는 DP2만, T-REL은 DP5만 바꾼다.'],['VOICE의 주변 조건','V-A를 2A+3A+5A라고 표시하지 않는다. DP2/DP5는 FIXED_UNMAPPED다.'],['메모리','TEXT 공통 기반 238MB와 VOICE 공통 기반 560MB는 별도 profile이다.'],['정확도','QA02와 QA07의 같은 대상 오류는 중복될 수 있다. 독립 효과로 합산하지 않는다.'],['준비 활동','VOICE의 60개 준비 활동은 평가 분모 밖이며 비용과 RAM에는 포함한다.'],['재계산 범위','비용은 사용량×단가, RAM은 동시 항목합, 시간과 정확도는 해당 원장의 평균으로 계산한다.'],['시간 원장','QA03은 원본 DAG에서 계산한 VIA 경로다. 이 파일에서 부모 DAG 전체를 재실행하지 않는다.']];
notes.push(['DP44 권고','A는 화면/고정 정확도 동점 후 QA03 이익과 음성 진단 이익을 조건부 권고 근거로 삼는다.'],['DP44 가정','QA02/07은 고정 정답 제어 fixture. QA04는 인터페이스에 따라 A=2.00 동점 가능.'],['DP44 비용','수신 Agent/전체 비용은 NOT_MODELED 빈칸. 무과금 자체 모델에는 청구액 차이가 적용되지 않는다.'],['근거 한계','SYNTHETIC_AUTHORED_NOT_MEASURED. 실제 모델/제품 실행 0회. Windows Excel 미검증.']);
for(const [i,n]of notes.entries()){sh.getRange(`A${57+i}:H${57+i}`).values=[[n[0],n[1],null,null,null,null,null,null]];sh.getRange(`A${57+i}`).format.font={bold:true};sh.getRange(`B${57+i}:H${57+i}`).merge();}
sh.getRange('A57:H68').format={rowHeight:34,verticalAlignment:'center',wrapText:true};
[[0,210],[1,145],[2,125],[3,125],[4,125],[5,130],[6,165],[7,300]].forEach(([i,w])=>sh.getRange(`${col(i)}1:${col(i)}68`).format.columnWidthPx=w);

wb.recalculate();
for(const c of summaryCells)assert(Math.abs(sh.getRange(c.address).values[0][0]-c.expected)<1e-7,`summary mismatch ${c.config}/${c.qa}`);
for(const [i,c]of d.configs.entries())for(const [j,key]of ['scored_usd','preparation_usd','maintenance_usd','qa05','receiver_cost','total_cost'].entries()){const value=sh.getRange(`${col(j+1)}${i+46}`).values[0][0],expected=d.summary[c.id][key];assert(expected===null?(value===null||value===''):Math.abs(value-expected)<1e-7,`cost ${c.id}/${key}`);}
for(const c of voiceCells)assert(Math.abs(sh.getRange(c.address).values[0][0]-c.expected)<1e-7,`voice ${c.config}: ${sh.getRange(c.address).values[0][0]} != ${c.expected}`);
const tariffChecks=[];
for(const stratum of ['TEXT','VOICE','DELIVERY']){const i=usage.findIndex(u=>u.stratum===stratum&&u.role==='VIA'&&u.amount>0&&u.rate_usd>0),price=sheets['비용계산'].getRange(`G${i+6}`),oldPrice=price.values[0][0],affected=usage[i].config_id;const costAddress=summaryCells.find(x=>x.config===affected&&x.qa==='qa05').address,original=sh.getRange(costAddress).values[0][0];price.values=[[oldPrice*2]];wb.recalculate();const changed=sh.getRange(costAddress).values[0][0];assert(Math.abs(changed-original-usage[i].amount*oldPrice)<1e-7);price.values=[[oldPrice]];wb.recalculate();assert(Math.abs(sh.getRange(costAddress).values[0][0]-original)<1e-7);tariffChecks.push({stratum,config:affected,rate_cell:`비용계산!G${i+6}`,summary_cell:`비교요약!${costAddress}`,original,changed,restored:true});}
for(const c of summaryCells)assert(Math.abs(sh.getRange(c.address).values[0][0]-c.expected)<1e-7);
const orderIndex=d.responses.findIndex(r=>r.config_id==='D-A'&&r.family_id==='D-S07');
assert(orderIndex>=0);
const orderSource=d.responses[orderIndex],orderCell=sheets['의미판정'].getRange(`E${orderIndex+6}`),orderOriginal=orderCell.values[0][0],orderChanged=structuredClone(orderSource.adopted);
orderChanged.conditions.order.reverse();orderCell.values=[[meaningJson(orderChanged,'DELIVERY')]];wb.recalculate();
const orderSummary=summaryCells.find(x=>x.config==='D-A'&&x.qa==='qa07').address;
assert.equal(sheets['의미판정'].getRange(`F${orderIndex+6}`).values[0][0],0);
assert(Math.abs(sh.getRange(orderSummary).values[0][0]-79/80)<1e-7);
orderCell.values=[[orderOriginal]];wb.recalculate();assert.equal(sh.getRange(orderSummary).values[0][0],1);
await fs.mkdir(DEL,{recursive:true});await fs.mkdir(PREVIEW,{recursive:true});
await fs.writeFile(path.join(PREVIEW,'workbook-summary-inspection.ndjson'),(await wb.inspect({kind:'table',range:'비교요약!A32:H41',include:'values,formulas',tableMaxRows:20,tableMaxCols:8,maxChars:10000})).ndjson);
await fs.writeFile(path.join(PREVIEW,'workbook-formula-scan.ndjson'),(await wb.inspect({kind:'match',searchTerm:'#REF!|#DIV/0!|#VALUE!|#NAME\\?|#N/A|#NUM!|#NULL!|#SPILL!|#CALC!',options:{useRegex:true,maxResults:30},maxChars:2000})).ndjson);
for(const name of names){
 if(process.argv.includes('--skip-previews'))continue;
 if(process.argv.includes('--changed-previews')&&!['비교요약','의미판정','메모리상태'].includes(name))continue;
 if(process.argv.includes('--summary-preview-only')&&name!=='비교요약')continue;
 const regions=name==='비교요약'?['A1:H28','A30:H41','A43:H54','A57:H68']:name==='메모리상태'?['A1:M10',`N5:${memLast}10`]:name==='첫응답'?['A1:I9','J5:N9',`A${d.responses.findIndex(r=>r.stratum==='DELIVERY')+6}:N${d.responses.findIndex(r=>r.stratum==='DELIVERY')+8}`]:[`A1:${tables[name].last}${Math.min(tables[name].end,name==='의미판정'||name==='변경과제'?8:11)}`];
 for(const [i,area]of regions.entries()){const blob=await wb.render({sheetName:name,range:area,scale:1,format:'png'});await fs.writeFile(path.join(PREVIEW,`${name}-${i+1}.png`),new Uint8Array(await blob.arrayBuffer()));}
}
const xlsx=path.join(DEL,'VIA-QA-stratified-v3.xlsx');await(await SpreadsheetFile.exportXlsx(wb)).save(xlsx);await fs.rm(xlsx+'.inspect.ndjson',{force:true});
const profileCard=stratum=>{const cfg=d.configs.filter(c=>c.stratum===stratum),voice=stratum==='VOICE',delivery=stratum==='DELIVERY';return `<section class="profile"><div class="eyebrow">${delivery?'VOICE DELIVERY':voice?'VOICE SCREEN':'TEXT HISTORY'}</div><h2>${delivery?'응답 생성과 음성 전달의 실행 구조':voice?'화면과 음성 근거의 처리 구조':'대화 이력과 후속 입력의 처리 구조'}</h2><p>${delivery?'80개 평가 입력, 120개 지칭, 90개 목표. 별도 준비 활동 80개를 비용과 RAM에 포함한다. QA02/07은 고정 정답 제어 fixture이며 정확도 우위 근거가 아니다.':voice?'120개 음성 입력, 120개 지칭, 120개 목표. 60개 사전 활동의 비용과 메모리를 함께 계상한다.':'120개 에피소드, 240개 입력, 250개 지칭, 180개 목표. 두 DP가 같은 T-BASE를 공유한다.'}</p><div class="overflow"><table><thead><tr><th>QA</th><th>단위</th>${cfg.map(c=>`<th>${c.id}</th>`).join('')}<th>분모/범위</th></tr></thead><tbody>${qaLabels.map((q,k)=>`<tr><th>${q}</th><td>${units[k]}</td>${cfg.map(c=>`<td class="num">${fmt(d.summary[c.id][`qa0${k+1}`],k)}</td>`).join('')}<td>${delivery?['80 입력','120 표현','90 목표','6 과제','80 평가 + 80 준비','전체 동시 구간','80 입력'][k]:[voice?'120 입력':'240 입력',voice?'120 표현':'250 표현',voice?'120 목표':'180 목표','6 과제',voice?'120 평가 + 60 준비':'120 에피소드','전체 동시 구간',voice?'120 입력':'240 입력'][k]}</td></tr>`).join('')}</tbody></table></div>${delivery?'<p><b>별도 음성 진단:</b> D-A '+fmt(d.summary['D-A'].qa01_voice,0)+'초 / D-B '+fmt(d.summary['D-B'].qa01_voice,0)+'초. 화면 QA01을 대체하거나 평균하지 않는다.</p><p>QA03은 외부 duration=0 인과 경로다. S08 음성의 실제 경과 이익 50ms와 투영 QA03 차이를 같은 wall-clock 단축으로 읽지 않는다.</p>':''}<div class="configline">${cfg.map(c=>`<span><b>${c.id}</b> ${esc(labels[c.id])}</span>`).join('')}</div></section>`;};
const caseUsageMap=new Map();for(const u of d.usage){const key=[u.config_id,u.case_id,u.phase,u.role].join('|');if(!caseUsageMap.has(key))caseUsageMap.set(key,{config_id:u.config_id,case_id:u.case_id,phase:u.phase,role:u.role,usd:0});caseUsageMap.get(key).usd+=u.usd;}
const mini={sources:d.sources??[],jobs:d.jobs??[],playback:d.playback??[],history:d.history??[],preparation:d.preparation_activities??[],configs:d.configs,labels,summary:d.summary,responses:d.responses.map(({config_id,stratum,case_id,family_id,input_id,qa01_s,qa01_voice_s,expected,adopted,qa07_success,first_content,error_reason})=>({config_id,stratum,case_id,family_id,input_id,qa01_s,qa01_voice_s,expected,adopted,qa07_success,first_content,error_reason,input_text:inputText({case_id,input_id})})),scenarios:d.scenarios.map(({case_id,inputs,source_locator})=>({case_id,inputs:inputs.map(({id,text})=>({id,text})),source_locator})),referents:d.referents.map(({config_id,input_id,expression_id,expected,adopted,correct})=>({config_id,input_id,expression_id,expected,adopted,correct})),goals:d.goals.map(({config_id,input_ids,goal_id,via_s,wall_s,terminal_content_correct,start_s,terminal_node,terminal_nodes})=>({config_id,input_ids,goal_id,via_s,wall_s,terminal_content_correct,start_s,terminal_node,terminal_nodes})),memory,stages:d.stages.map(({id,config_id,case_id,kind,role,start_s,end_s,parents})=>({id,config_id,case_id,kind,role,start_s,end_s,parents})),usage:[...caseUsageMap.values()]};
const html=`<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>VIA 공통 QA 비교 v3</title><style>
:root{--ink:#22364c;--muted:#637185;--line:#dce3ea;--navy:#263c55;--accent:#2d6d91}*{box-sizing:border-box}body{margin:0;background:#f4f6f9;color:var(--ink);font:15px/1.65 -apple-system,BlinkMacSystemFont,"Apple SD Gothic Neo",Arial,sans-serif}main{max-width:1440px;margin:auto;padding:50px 32px 70px}h1{font-size:34px;letter-spacing:-1px;margin:6px 0 14px}h2{font-size:24px;letter-spacing:-.5px;margin:3px 0 10px}h3{font-size:18px;margin:20px 0 10px}p{margin:8px 0;color:var(--muted)}.eyebrow{font-size:12px;font-weight:700;letter-spacing:1.5px;color:var(--accent)}.lead{font-size:17px;max-width:980px}.notice{border-left:3px solid var(--accent);padding:10px 16px;background:#eaf0f6;margin:24px 0}.profile,.panel{background:white;border:1px solid var(--line);border-radius:12px;padding:26px;margin:24px 0}.overflow{overflow:auto}table{border-collapse:collapse;width:100%;font-size:14px}th,td{padding:12px 14px;border-bottom:1px solid var(--line);vertical-align:top;text-align:left}thead th{background:var(--navy);color:white;font-size:13px;white-space:nowrap}tbody th{font-weight:600}.num{text-align:right;font-variant-numeric:tabular-nums}.configline{display:flex;flex-wrap:wrap;gap:12px 24px;font-size:13px;margin-top:17px;color:var(--muted)}.configline b{color:var(--ink)}.controls{display:flex;flex-wrap:wrap;gap:14px;align-items:end;margin:15px 0}label{font-size:13px;color:var(--muted)}select,input{display:block;min-width:160px;padding:10px;border:1px solid #bac7d4;border-radius:6px;background:white;color:var(--ink);font:inherit}label.check{display:flex;align-items:center;gap:8px;padding-bottom:8px}input[type=checkbox]{min-width:0}.split{display:grid;grid-template-columns:minmax(270px,1fr) minmax(500px,2fr);gap:22px}.list{max-height:670px;overflow:auto;border:1px solid var(--line);border-radius:7px}.case{display:block;width:100%;text-align:left;border:0;border-bottom:1px solid var(--line);padding:14px;background:white;color:var(--ink);cursor:pointer;font:inherit}.case:hover,.case.active{background:#edf3f8}.case small{display:block;color:var(--muted)}.fail{color:#a54036}.ok{color:#2a695e}.content{padding:15px;background:#f4f7fa;border-radius:6px;white-space:pre-wrap;overflow-wrap:anywhere}.pair{display:grid;grid-template-columns:1fr 1fr;gap:16px}pre{margin:0;font:12px/1.7 ui-monospace,monospace;white-space:pre-wrap;overflow-wrap:anywhere}.fine{font-size:12px;color:var(--muted)}details{margin:16px 0}summary{font-weight:600;cursor:pointer}a{color:var(--accent)}@media(max-width:950px){main{padding:24px 14px}.split,.pair{grid-template-columns:1fr}.list{max-height:250px}.profile,.panel{padding:18px}h1{font-size:28px}}
</style><main><div class="eyebrow">VIA ARCHITECTURE / COMMON QA v3</div><h1>같은 QA, 각 profile 안의 공정한 비교</h1><p class="lead">기존 DP2/DP5 TEXT와 DP3 VOICE를 보존하고 DP44 DELIVERY를 별도 층으로 추가했습니다. 공통 정의와 근거 추적 형식을 맞추고, 서로 다른 입력과 장치 가정은 별도 층으로 보존했습니다.</p><div class="notice">가상 시나리오 기반 설계 평가 <b>${KIND}</b><br>세 층을 합산한 평균이나 DP 사이의 절대 성능 순위는 제시하지 않습니다. 모델 및 제품 실제 실행은 0회입니다.</div>${profileCard('TEXT')}${profileCard('VOICE')}${profileCard('DELIVERY')}<section class="panel"><h2>각 층의 개별 조건부 권고</h2><p><b>DP2 B, DP3 A, DP5 B, DP44 A</b>를 각각 권고합니다. QA01 우선순위와 이 자료의 작업량 및 장치 가정에 따른 독립 비교의 판단입니다. 네 안을 함께 적용한 제품 구성의 성능이나 우위를 뜻하지 않습니다.</p><p>DP2 B와 DP5 B는 첫 응답 이익과 함께 의미 오류 및 비용 부담을 받아들이는 선택입니다. DP3 A는 이 CPU 가정에서 첫 응답과 RAM을 우선한 선택입니다. DP44 A는 화면과 고정 정확도 동점 후 QA03 및 음성 진단의 이익에 모델 비용 약 11.5% 증가를 수용하는 조건부 권고입니다. QA04는 예상 concept 모듈 수이며 공통 인터페이스에 따라 A도 2.00으로 동점이 가능합니다. RAM 차이는 약 0.152 MB로 작습니다. 결합 구성은 별도의 같은 workload에서 계산해야 합니다.</p></section><section class="panel"><h2>비용에 포함한 활동</h2><p>준비와 종료 후 갱신을 VIA 비용에 포함합니다. 수신 Agent의 업무 비용은 별도이며, 아래 전체 비용은 같은 구성 안에서만 합산합니다. DELIVERY 수신 Agent와 전체는 미모델링입니다. 폐기비는 VIA 총액의 부분집합이며 더하지 않습니다. 자체 무과금 모델에는 이 청구액 비교가 적용되지 않습니다.</p><div class="overflow"><table><thead><tr><th>구성</th><th>평가 VIA</th><th>준비 VIA</th><th>갱신 VIA</th><th>VIA 총액</th><th>수신 Agent</th><th>전체 USD</th></tr></thead><tbody>${d.configs.map(c=>{const s=d.summary[c.id];return `<tr><th>${c.id}</th>${['scored_usd','preparation_usd','maintenance_usd','qa05','receiver_cost','total_cost'].map(k=>`<td class="num">${s[k]===null?'미모델링':('$'+s[k].toFixed(6))}</td>`).join('')}</tr>`;}).join('')}</tbody></table></div></section><section class="panel"><h2>비교 범위의 경계</h2><div class="pair"><div><h3>공유한 것</h3><p>첫 구체적 표시, 최초 지칭 집합, 목표의 VIA 경로, C01~C06 변경 요구, 전체 활동 비용과 동시 RAM, 입력별 필수 의미 판정입니다.</p><p>QA02와 QA07은 대상 오류가 겹칩니다. 같은 개선을 독립적인 두 효과로 더하지 않습니다.</p></div><div><h3>별도로 유지한 것</h3><p>TEXT는 입력 제출, VOICE는 발화 종료에서 시작합니다. TEXT 공통 기반은 238 MB, VOICE 공통 기반은 560 MB입니다.</p><p>DELIVERY의 DP2/DP3/DP5는 FIXED_UNMAPPED입니다. 공유 RAM/요율이 있어도 DP3와 같은 전체 구성은 아닙니다. 음성 진단은 TEXT/VOICE에서 NOT_MODELED입니다.</p><p>VOICE의 DP2/DP5는 FIXED_UNMAPPED입니다. V-A를 2A+3A+5A로 해석하지 않으며 T-BASE와 동일한 구현이라고 가정하지 않습니다.</p></div></div></section><section class="panel"><h2>입력과 채택 결과 살펴보기</h2><p>같은 사례 안에서 구성을 바꾸어 첫 응답, 최초 의미, 비용과 RAM 근거를 확인할 수 있습니다.</p><div class="controls"><label>Profile<select id="profile"><option>TEXT</option><option>VOICE</option><option>DELIVERY</option></select></label><label>구성<select id="config"></select></label><label>유형<select id="family"></select></label><label>검색<input id="query" placeholder="입력 또는 사례 ID"></label><label class="check"><input type="checkbox" id="errors">의미 오류만</label></div><p id="count" class="fine"></p><div class="split"><div id="list" class="list"></div><div id="detail"></div></div></section><p class="fine">${esc(d.dataset_id)} / ${esc(d.contract_id)}. 원본 snapshot의 SHA와 원장은 두 단계 상위 폴더의 manifest.json, normalized-data.json에 보존했습니다. <a href="VIA-QA-stratified-v3.xlsx">Excel 계산표 열기</a></p></main><script>
const D=${JSON.stringify(mini).replace(/</g,'\\u003c')};
const $=id=>document.getElementById(id),E=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
let selected='';
function configOptions(){const p=$('profile').value;$('config').innerHTML=D.configs.filter(c=>c.stratum===p).map(c=>'<option value="'+c.id+'">'+E(c.id+' '+D.labels[c.id])+'</option>').join('');$('family').innerHTML='<option value="">전체 유형</option>'+[...new Set(D.responses.filter(r=>r.stratum===p).map(r=>r.family_id))].sort().map(f=>'<option>'+f+'</option>').join('');render();}
function render(){const c=$('config').value,f=$('family').value,q=$('query').value.toLowerCase();const rows=D.responses.filter(r=>r.config_id===c&&(!f||r.family_id===f)&&(!$('errors').checked||!r.qa07_success)&&(!q||JSON.stringify(r).toLowerCase().includes(q)));$('count').textContent=rows.length+'개 입력 표시. 전체 분모는 위 요약표를 유지합니다.';$('list').innerHTML=rows.map(r=>'<button class="case" data-id="'+E(r.input_id)+'"><b>'+E(r.input_id)+'</b><small>'+r.qa01_s.toFixed(3)+'초 화면 / <span class="'+(r.qa07_success?'ok':'fail')+'">'+(r.qa07_success?'의미 일치':'의미 오류')+'</span></small></button>').join('');if(!rows.some(r=>r.input_id===selected))selected=rows[0]?.input_id??'';document.querySelectorAll('.case').forEach(b=>{b.onclick=()=>{selected=b.dataset.id;show(rows.find(r=>r.input_id===selected));};});show(rows.find(r=>r.input_id===selected));}
function show(r){document.querySelectorAll('.case').forEach(b=>b.classList.toggle('active',b.dataset.id===selected));if(!r){$('detail').innerHTML='<p>조건에 맞는 입력이 없습니다.</p>';return;}const sc=D.scenarios.find(s=>s.case_id===r.case_id),input=sc.inputs.find(i=>r.input_id.endsWith(i.id));const refs=D.referents.filter(x=>x.config_id===r.config_id&&x.input_id===r.input_id),goals=D.goals.filter(x=>x.config_id===r.config_id&&x.input_ids.includes(r.input_id));const mem=D.memory.find(x=>x.config_id===r.config_id&&x.case_id===r.case_id);const stages=D.stages.filter(x=>x.config_id===r.config_id&&x.case_id===r.case_id);const costs=D.usage.filter(x=>x.config_id===r.config_id&&x.case_id===r.case_id);const sums={};costs.forEach(u=>{const k=u.phase+' / '+u.role;sums[k]=(sums[k]??0)+u.usd;});$('detail').innerHTML='<h3>'+E(r.config_id+' / '+r.input_id)+'</h3><div class="content">'+E(input?.text??'')+'</div><h3>QA01 화면 첫 구체적 표시</h3><p>'+r.qa01_s.toFixed(3)+'초. '+(r.qa07_success?'최초 의미 전체 일치.':'최초 의미 오류. '+E(r.error_reason))+'</p>'+(r.qa01_voice_s==null?'<p>음성 진단: NOT_MODELED</p>':'<p>별도 음성 진단 '+r.qa01_voice_s.toFixed(3)+'초. 화면 값과 평균하지 않음.</p>')+(r.stratum==='DELIVERY'?'<p>QA02/07은 고정 정답 제어 fixture. 모델 정확도나 차별화 coverage의 증거가 아니다.</p>':'')+'<div class="content">'+E(r.first_content)+'</div><h3>지칭 판정</h3>'+(refs.length?refs.map(x=>'<p class="'+(x.correct?'ok':'fail')+'">'+E(x.expression_id)+'<br>정답 '+E(JSON.stringify(x.expected))+'<br>채택 '+E(JSON.stringify(x.adopted))+'</p>').join(''):'<p>이 입력에는 채점할 지칭 표현이 없습니다.</p>')+'<details open><summary>필수 요청 의미</summary><div class="pair"><div><h3>정답</h3><pre class="content">'+E(JSON.stringify(r.expected,null,2))+'</pre></div><div><h3>최초 채택</h3><pre class="content">'+E(JSON.stringify(r.adopted,null,2))+'</pre></div></div></details><h3>연결된 목표</h3>'+goals.map(g=>'<p>'+E(g.goal_id)+'<br>VIA '+g.via_s.toFixed(3)+'초 / 전체 경과 '+g.wall_s.toFixed(3)+'초 / 최종 의미 '+(g.terminal_content_correct?'일치':'불일치')+'<br>시작 '+E(g.start_s)+' / terminal '+E((g.terminal_nodes??[g.terminal_node]).join(', '))+'</p>').join('')+'<h3>이 사례 전체의 비용</h3><p>'+Object.entries(sums).map(([k,v])=>E(k)+': $'+v.toFixed(6)).join('<br>')+'</p>'+(mem?'<details><summary>사례의 최대 동시 RAM '+mem.total_mb.toFixed(2)+' MB</summary><p>'+E(mem.id)+' / t='+mem.time_s.toFixed(3)+' / '+E(mem.phase)+'</p><pre class="content">'+E(JSON.stringify(mem.components_mb,null,2))+'</pre></details>':'')+ (r.stratum==='DELIVERY'?'<details><summary>응답 소스와 생성 및 전달 이력</summary><pre class="content">'+E(JSON.stringify({sources:D.sources.filter(x=>x.case_id===r.case_id&&x.config_id===r.config_id),jobs:D.jobs.filter(x=>x.case_id===r.case_id&&x.config_id===r.config_id),playback:D.playback.filter(x=>x.case_id===r.case_id&&x.config_id===r.config_id),history:D.history.filter(x=>x.case_id===r.case_id&&x.config_id===r.config_id)},null,2))+'</pre></details>':'')+'<details><summary>이 사례의 처리 단계 '+stages.length+'개</summary><div class="overflow"><table><thead><tr><th>처리 ID / 종류</th><th>시작</th><th>끝</th><th>역할</th></tr></thead><tbody>'+stages.map(x=>'<tr><td>'+E(x.id)+'<br>'+E(x.kind)+'<br>의존 '+E((x.parents??[]).join(', '))+'</td><td>'+x.start_s.toFixed(3)+'</td><td>'+x.end_s.toFixed(3)+'</td><td>'+E(x.role)+'</td></tr>').join('')+'</tbody></table></div></details><p class="fine">사례 원본: '+E(sc.source_locator)+'<br>비용과 RAM에는 준비 상태가 포함될 수 있습니다.</p>';}
$('profile').onchange=configOptions;['config','family','errors'].forEach(k=>$(k).onchange=render);$('query').oninput=render;configOptions();
</script></html>`;
await fs.writeFile(path.join(DEL,'guide.html'),html);
await fs.writeFile(path.join(OUT,'workbook-validation.json'),JSON.stringify({dataset:d.dataset_id,evidence_kind:KIND,source_sha256:sha(sourceBytes),sheet_count:names.length,qa_cells_checked:summaryCells.length,voice_diagnostic_cells_checked:voiceCells.length,voice_cells:voiceCells,summary_cells:summaryCells,cost_numeric_cells_checked:50,cost_missing_cells_checked:4,tariff_perturbation_and_restore:tariffChecks,delivery_order_only_perturbation_and_restore:{input_id:orderSource.input_id,edited_cell:`의미판정!E${orderIndex+6}`,original_qa07:1,changed_qa07:79/80,restored_qa07:1},memory_source_rows:d.memory.length,memory_case_peak_rows:memory.length,usage_source_rows:d.usage.length,usage_equal_bucket_rows:usage.length,rendered_sheets:names,preview_policy:process.argv.includes('--skip-previews')?'REUSED_UNCHANGED_VISIBLE_CONTENT':'RENDERED_OR_RETAINED_UNCHANGED_SHEETS',native_excel_verified:false,actual_model_calls:0},null,2));
console.log(JSON.stringify({xlsx,html:path.join(DEL,'guide.html'),sheets:names.length,qa_cells:summaryCells.length,voice_cells:voiceCells.length,usage_rows:usage.length,memory_rows:memory.length}));
