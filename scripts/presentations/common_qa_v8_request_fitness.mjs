import assert from 'node:assert/strict';

export function addQa07Protocol(wb,d){
 const q=d.qa07,caseSheet=wb.worksheets.add('QA07v3사례'),obs=wb.worksheets.add('QA07v3의무'),spec=wb.worksheets.add('QA07v3명세');
 const col=i=>String.fromCharCode(65+i);
 function style(s,end,widths){s.showGridLines=false;s.getRange(`A1:${col(widths.length-1)}${end}`).format={font:{name:'Arial',size:11,color:'#23344B'},verticalAlignment:'center',rowHeight:34};widths.forEach((w,i)=>s.getRange(`${col(i)}1:${col(i)}${end}`).format.columnWidth=w);}
 function header(s,row,values){s.getRange(`A${row}:${col(values.length-1)}${row}`).values=[values];s.getRange(`A${row}:${col(values.length-1)}${row}`).format={fill:'#253C59',font:{name:'Arial',size:11,color:'#FFFFFF',bold:true},rowHeight:46,wrapText:true,horizontalAlignment:'center',verticalAlignment:'center'};}
 function title(s,text,note,last){s.getRange(`A2:${last}2`).merge();s.getRange('A2').values=[[text]];s.getRange('A2').format.font={name:'Arial',size:16,bold:true};s.getRange(`A3:${last}3`).merge();s.getRange('A3').values=[[note]];s.getRange('A3').format={rowHeight:42,wrapText:true};}
 style(spec,37,[14,39,65,68,56,14]);title(spec,'QA07 v3의 32사례','작성자 제안: 8유형 × 4사례 × 5반복 = 160회/구성. 실제 원음·화면 fixture 및 실행 동결 전.','F');
 header(spec,5,['사례','유형','입력과 사건','필수 해석·응답','금지 조건','입력 채널']);
 spec.getRange('A6:F37').values=q.manifest.cases.map(c=>[c.case_id,c.type,c.input_and_events,c.required,c.forbidden,c.modality]);spec.getRange('A6:F37').format={rowHeight:138,wrapText:true};spec.freezePanes.freezeRows(5);spec.freezePanes.freezeColumns(1);
 const obligationRows=[],ranges=new Map();
 for(const r of q.records){const c=q.manifest.cases.find(c=>c.case_id===r.case_id),start=obligationRows.length+6;for(const o of c.obligations)obligationRows.push([r.config_id,r.case_id,r.repeat_id,o.stage,o.requirement,o.forbidden,'','','','','','','','','']);ranges.set([r.config_id,r.case_id,r.repeat_id].join('|'),[start,obligationRows.length+5]);}
 style(obs,obligationRows.length+5,[18,14,10,30,66,53,15,28,16,24,16,24,28,28,16]);
 title(obs,'QA07 v3 단계별 판정 근거','각 행은 해당 반복의 모든 revision과 의무를 판정한다. 두 평가자·출력 근거 필요. 과거 입증 실패 참조가 있으면 FAIL 유지.','F');
 header(obs,5,['구성','사례','반복','단계','필수 내용 전체','금지 조건','최종 판정','trace/출력 근거','평가자1 판정','평가자1 참조','평가자2 판정','평가자2 참조','불일치 조정 참조','과거 입증 실패 참조','상태']);
 obs.getRange(`A6:O${obligationRows.length+5}`).values=obligationRows;obs.getRange(`A6:O${obligationRows.length+5}`).format={rowHeight:120,wrapText:true};obs.getRange(`G6:N${obligationRows.length+5}`).format.fill='#FFF0CB';
 for(const c of ['G','I','K'])obs.getRange(`${c}6:${c}${obligationRows.length+5}`).dataValidation={rule:{type:'list',values:['PASS','FAIL','UNKNOWN']}};
 obs.getRange(`O6:O${obligationRows.length+5}`).formulas=obligationRows.map((_,i)=>{const r=i+6;return [`=IF(N${r}<>"","FAIL",IF(AND(H${r}<>"",J${r}<>"",L${r}<>"",J${r}<>L${r},OR(G${r}="PASS",G${r}="FAIL"),OR(AND(I${r}=G${r},K${r}=G${r}),AND(OR(I${r}="PASS",I${r}="FAIL"),OR(K${r}="PASS",K${r}="FAIL"),I${r}<>K${r},M${r}<>""))),G${r},"UNKNOWN"))`];});obs.freezePanes.freezeRows(5);obs.freezePanes.freezeColumns(3);
 const end=q.records.length+16;style(caseSheet,end,[18,15,10,38,12,12,14,18,27,29,29,15,16,26,29]);
 title(caseSheet,'QA07 v3 사례와 반복','목표 ≥90%, 144/160 성공, 2%p 점수 구간은 작성자 제안. 계획 160회/구성, 실제 실행 0회. 결과는 미산출.','L');
 header(caseSheet,5,['구성','계획 반복','성공 S','실패 F','미판정 U','공통 무효','확정 N','QA07 %','하한 %','상한 %','점수 /6','실행 동결']);
 header(caseSheet,16,['구성','사례','반복','유형','충족 단계','위반 단계','미판정 단계','fixture 완비','실행 증거 키','종료 관측 근거','실행 조건 증거','사례 상태','키 중복 수','공통 무효 근거','최초 채택 trace']);
 caseSheet.getRange(`A17:O${end}`).values=q.records.map(r=>{const c=q.manifest.cases.find(c=>c.case_id===r.case_id);return [r.config_id,r.case_id,r.repeat_id,c.type,0,0,3,'NO','','','','',0,'',''];});caseSheet.getRange(`A17:O${end}`).format={rowHeight:64,wrapText:true};
 caseSheet.getRange(`I17:K${end}`).format.fill='#FFF0CB';caseSheet.getRange(`O17:O${end}`).format.fill='#FFF0CB';
 caseSheet.getRange('Q5:R5').values=[['사례','공통 자료 무효 근거']];caseSheet.getRange('Q5:R5').format={fill:'#253C59',font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},rowHeight:46};caseSheet.getRange('Q6:R37').values=q.manifest.cases.map(c=>[c.case_id,'']);caseSheet.getRange('Q1:Q37').format.columnWidth=18;caseSheet.getRange('R1:R37').format.columnWidth=46;
 q.records.forEach((r,i)=>{const row=i+17,[a,b]=ranges.get([r.config_id,r.case_id,r.repeat_id].join('|'));caseSheet.getRange(`E${row}:G${row}`).formulas=[['PASS','FAIL','UNKNOWN'].map(status=>`=COUNTIF('QA07v3의무'!O${a}:O${b},"${status}")`)];caseSheet.getRange(`N${row}`).formulas=[[`=IF(R${6+Math.floor(i%160/5)}="","",R${6+Math.floor(i%160/5)})`]];caseSheet.getRange(`M${row}`).formulas=[[`=IF(I${row}="",0,COUNTIFS($A$17:$A$${end},A${row},$I$17:$I$${end},I${row}))`]];caseSheet.getRange(`L${row}`).formulas=[[`=IF(N${row}<>"","INVALID",IF(F${row}>0,"FAIL",IF(AND(G${row}=0,H${row}="YES",I${row}<>"",J${row}<>"",K${row}<>"",O${row}<>"",M${row}=1),"PASS","UNKNOWN")))`]];});
 d.configs.forEach((cfg,i)=>{const r=i+6,a=i*160+17,b=a+159;caseSheet.getRange(`A${r}:B${r}`).values=[[cfg.id,160]];caseSheet.getRange(`L${r}`).values=[['NO']];caseSheet.getRange(`C${r}:F${r}`).formulas=[['PASS','FAIL','UNKNOWN','INVALID'].map(s=>`=COUNTIF(L${a}:L${b},"${s}")`)];caseSheet.getRange(`G${r}:K${r}`).formulas=[[
 `=IF(OR(L${r}<>"YES",COUNTIF(H${a}:H${b},"YES")<>160,COUNTIF(M${a}:M${b},">1")>0,F${r}>0),"미동결",160)`,
 `=IF(OR(G${r}="미동결",E${r}>0),"미산출",100*C${r}/G${r})`,
 `=IF(G${r}="미동결","미산출",100*C${r}/G${r})`,
 `=IF(G${r}="미동결","미산출",100*(C${r}+E${r})/G${r})`,
 `=IF(ISNUMBER(H${r}),IF(H${r}=100,6,IF(H${r}<90,0,IF(H${r}<92,1,IF(H${r}<94,2,IF(H${r}<96,3,IF(H${r}<98,4,5)))))),"미산출")`]];});
 caseSheet.getRange('A14:L14').merge();caseSheet.getRange('A14').values=[['의무 한도 60초, 사례 한도 300초 + 후속 10초, 종료 후 5초에 구판 재검사. 원음/화면 해시 고정과 실행 동결은 별도 필요.']];
 caseSheet.getRange('A15:L15').merge();caseSheet.getRange('A15').values=[['fixture 완비와 실행 동결은 검증된 manifest에만 기록한다. 노란 관측 칸을 채운 것만으로 YES로 바꾸지 않는다. 원시 판정 이력은 보존.']];
 caseSheet.getRange('H6:K12').setNumberFormat('0.00');caseSheet.freezePanes.freezeRows(16);caseSheet.freezePanes.freezeColumns(3);
 header(spec,5,['사례','유형','입력과 사건','필수 해석·응답','금지 조건','입력 채널']);
 spec.getRange('H5:O5').values=[['구성','유형','사례 수','계획 반복','성공','실패','미판정','전체 충족률 %']];spec.getRange('H5:O5').format={fill:'#253C59',font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},rowHeight:46,wrapText:true};
 [18,44,13,15,15,15,15,20].forEach((w,i)=>spec.getRange(`${String.fromCharCode(72+i)}1:${String.fromCharCode(72+i)}61`).format.columnWidth=w);
 d.appendices.qa07_types.forEach((t,i)=>{const r=i+6,ci=d.configs.findIndex(c=>c.id===t.config_id),a=17+160*ci,b=a+159;spec.getRange(`H${r}:K${r}`).values=[[t.config_id,t.type,4,20]];spec.getRange(`L${r}:N${r}`).formulas=[['PASS','FAIL','UNKNOWN'].map(v=>`=COUNTIFS('QA07v3사례'!D${a}:D${b},I${r},'QA07v3사례'!L${a}:L${b},"${v}")`)];spec.getRange(`O${r}`).formulas=[[`=IF(OR(NOT(ISNUMBER('QA07v3사례'!G${6+ci})),N${r}>0,L${r}+M${r}<>K${r}),"미산출",100*L${r}/K${r})`]];});spec.getRange('H6:O61').format={font:{name:'Arial',size:11,color:'#23344B'},wrapText:true,rowHeight:138};
 const main=wb.worksheets.getItem('전체비교'),cmp=['BASE','DP2-B','BASE','DP3-B-R1','DP3-B-R2','DP3-B-R3','BASE','DP5-B','BASE','DP44-B'];
 cmp.forEach((id,i)=>main.getRange(`G${21+i}`).formulas=[[`='QA07v3사례'!H${6+d.configs.findIndex(c=>c.id===id)}`]]);
 main.getRange('A32').values=[['QA07은 새 32사례 × 5회 전체 충족률. QA07v3사례/의무/명세 참조. 목표 90%는 제안, 실제 실행 0회.']];
 wb.worksheets.getItem('QA07사례').getRange('A3').values=[['v6 보존 24묶음 검토안. 현재 QA07 대표값에 사용하지 않는다. 새 32사례는 QA07v3사례/명세.']];
 wb.worksheets.getItem('QA07의무').getRange('A3').values=[['v6 보존 의무 원장. 현재 QA07 대표값에 사용하지 않는다. 새 판정은 QA07v3의무.']];
 // Evidence gates and two-reviewer disagreement probes, restored before export.
 obs.getRange('G6:L6').values=[['PASS','test-trace','PASS','reviewer-1','','']];assert.equal(obs.getRange('O6').values[0][0],'UNKNOWN');
 obs.getRange('K6:L6').values=[['FAIL','reviewer-2']];assert.equal(obs.getRange('O6').values[0][0],'UNKNOWN');
 obs.getRange('M6').values=[['test-adjudication']];assert.equal(obs.getRange('O6').values[0][0],'PASS');assert.equal(caseSheet.getRange('L17').values[0][0],'UNKNOWN');
 obs.getRange('N6').values=[['test-proven-prior-fail']];assert.equal(caseSheet.getRange('L17').values[0][0],'FAIL');
 caseSheet.getRange('R6').values=[['test-common-invalid']];for(let i=0;i<7;i++)assert.equal(caseSheet.getRange(`L${17+i*160}`).values[0][0],'INVALID');
 caseSheet.getRange('R6').clear({applyTo:'contents'});obs.getRange('G6:N6').clear({applyTo:'contents'});
 // Exercise an entire planned denominator and score in a disposable authoring state.
 obs.getRange('G6:L485').values=Array.from({length:480},()=>['PASS','test-trace','PASS','test-r1','PASS','test-r2']);
 caseSheet.getRange('H17:H176').values=Array.from({length:160},()=>['YES']);
 caseSheet.getRange('I17:K176').values=Array.from({length:160},(_,i)=>['test-run-'+i,'test-close','test-protocol']);
 caseSheet.getRange('O17:O176').values=Array.from({length:160},()=>['test-first-adoption']);
 caseSheet.getRange('L6').values=[['YES']];assert.equal(caseSheet.getRange('H6').values[0][0],100);assert.equal(caseSheet.getRange('K6').values[0][0],6);assert.equal(spec.getRange('O6').values[0][0],100);
 obs.getRange('N6').values=[['test-prior-fail']];assert.equal(caseSheet.getRange('H6').values[0][0],99.375);assert.equal(caseSheet.getRange('K6').values[0][0],5);
 obs.getRange('N9').values=[['test-second-prior-fail']];assert.equal(spec.getRange('N6').values[0][0],0);
 caseSheet.getRange('I18').values=[['test-run-0']];assert.equal(caseSheet.getRange('G6').values[0][0],'미동결');assert.equal(spec.getRange('N6').values[0][0],0);assert.equal(spec.getRange('O6').values[0][0],'미산출');
 obs.getRange('G6:N485').clear({applyTo:'contents'});caseSheet.getRange('H17:H176').values=Array.from({length:160},()=>['NO']);caseSheet.getRange('I17:K176').clear({applyTo:'contents'});caseSheet.getRange('O17:O176').clear({applyTo:'contents'});caseSheet.getRange('L6').values=[['NO']];
 assert.equal(caseSheet.getRange('E6').values[0][0],160);
 return {views:[['QA07v3사례','A1:L12','qa07-v3-summary.png'],['QA07v3명세','A1:F8','qa07-v3-spec.png'],['QA07v3명세','H5:O9','qa07-v3-types.png'],['QA07v3사례','A16:H19','qa07-v3-cases.png'],['QA07v3의무','A1:F8','qa07-v3-obligations.png'],['QA07v3의무','G5:O8','qa07-v3-evidence.png']],obligationRows:obligationRows.length};
}
