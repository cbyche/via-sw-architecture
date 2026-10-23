import assert from 'node:assert/strict';

// This is an evidence-entry/aggregation table, not a model runner or semantic adapter.
export function addOrderLink(wb,d){
 const q=d.qa07,m=q.manifest,s=wb.worksheets.add('QA07순서연결'),p=wb.worksheets.add('QA07선정입력');
 const col=i=>String.fromCharCode(65+i),end=996;
 function style(sh,last,widths){sh.showGridLines=false;sh.getRange(`A1:${col(widths.length-1)}${last}`).format={font:{name:'Arial',size:11,color:'#23344B'},verticalAlignment:'center',rowHeight:34};widths.forEach((w,i)=>sh.getRange(`${col(i)}1:${col(i)}${last}`).format.columnWidth=w);}
 function header(sh,row,labels,start=0){sh.getRange(`${col(start)}${row}:${col(start+labels.length-1)}${row}`).values=[labels];sh.getRange(`${col(start)}${row}:${col(start+labels.length-1)}${row}`).format={fill:'#253C59',font:{name:'Arial',size:11,bold:true,color:'#FFFFFF'},wrapText:true,rowHeight:50,horizontalAlignment:'center',verticalAlignment:'center'};}
 function title(sh,text,note,last){sh.getRange(`A2:${last}2`).merge();sh.getRange('A2').values=[[text]];sh.getRange('A2').format.font={name:'Arial',size:16,bold:true};sh.getRange(`A3:${last}3`).merge();sh.getRange('A3').values=[[note]];sh.getRange('A3').format={wrapText:true,rowHeight:48};}
 style(p,94,[16,62,54,48,17,20,72,20]);
 title(p,'QA07 v4 선정 입력과 정답','84입력 중 28개 선정·56개 제외. 모든 대표 입력은 Text. 정답/설명 문맥은 평가자용이며 후보 입력 fixture는 미준비.','H');
 header(p,5,['원전 입력','사용자 원문','NEW / EXISTING 정답','사용자 지정 선후 관계','순서 유형','원전 입력 채널','문맥 보완과 해설','공통 무효 근거']);
 p.getRange('A6:H33').values=m.cases.map(c=>[c.source_event_id,c.input,c.units.map(u=>`${u.anchor}: ${u.link}`).join('\n'),c.before.map(e=>e.join(' → ')).join('\n')||'없음',c.order_type,c.source_input_mode,c.context_note,'']);
 p.getRange('A6:H33').format={wrapText:true,rowHeight:112};p.getRange('H6:H33').format.fill='#FFF0CB';
 header(p,37,['제외 입력','제외 이유']);p.getRange('A38:B93').values=m.population.excluded_user_events.map(c=>[c.source_event_id,c.reason]);p.getRange('A38:B93').format={wrapText:true,rowHeight:62};
 p.getRange('J2:Q2').merge();p.getRange('J2').values=[['순서 유형별 부록']];p.getRange('J2').format.font={name:'Arial',size:16,bold:true};
 p.getRange('J3:Q3').merge();p.getRange('J3').values=[['순서 있음 2입력·10반복, 없음 26입력·130반복. S17-U01 독립 요청 5반복은 없음 그룹의 부분집합.']];p.getRange('J3').format={rowHeight:48,wrapText:true};
 header(p,5,['구성','유형','입력 수','계획 N','성공 S','실패 F','미판정 U','정확도 %'],9);
 [18,23,12,12,12,12,14,18].forEach((w,i)=>p.getRange(`${col(9+i)}1:${col(9+i)}33`).format.columnWidth=w);
 const groups=[...d.appendices.qa07_types,...d.appendices.qa07_explicit_independence.map(t=>({...t,type:'명시적 독립 요청',input_count:1}))];
 groups.forEach((t,i)=>{const r=i+6,ci=d.configs.findIndex(c=>c.id===t.config_id),a=17+ci*140,b=a+139,ind=t.type==='명시적 독립 요청';p.getRange(`J${r}:M${r}`).values=[[t.config_id,t.type,t.input_count,t.planned]];p.getRange(`N${r}:P${r}`).formulas=[['PASS','FAIL','UNKNOWN'].map(v=>`=COUNTIFS('QA07순서연결'!${ind?'B':'D'}${a}:${ind?'B':'D'}${b},${ind?'"S17-U01"':`K${r}`},'QA07순서연결'!R${a}:R${b},"${v}")`)];p.getRange(`Q${r}`).formulas=[[`=IF(OR(NOT(ISNUMBER('QA07순서연결'!G${ci+6})),P${r}>0,N${r}+O${r}<>M${r}),"미산출",100*N${r}/M${r})`]];});
 p.getRange('J6:Q26').format={font:{name:'Arial',size:11,color:'#23344B'},rowHeight:40,wrapText:true};
 p.getRange('J29:Q29').merge();p.getRange('J29').values=[['출처: QA07-ORDER-LINK-v4 / QA07-ORDER-LINK-POP-v4. 목표 85%, 3%p 구간은 작성자 제안.']];
 p.getRange('J30:Q30').merge();p.getRange('J30').values=[['TaskBench 2작업 graph accuracy 84.53% 참조. 전체 graph 67.25% / SMCalFlow 73.751%와 다른 수치.']];
 p.getRange('J31:Q31').merge();p.getRange('J31').values=[['서로 다른 평가셋 숫자 차이를 VIA 개선 성적으로 사용하지 않는다. 근거와 링크는 고정 정의 snapshot을 참조한다.']];
 p.getRange('J29:Q31').format={wrapText:true,rowHeight:44};p.freezePanes.freezeRows(5);p.freezePanes.freezeColumns(1);
 style(s,end,[18,16,10,17,16,16,29,22,22,30,29,29,17,29,29,18,29,16,14,29]);
 title(s,'QA07 업무 순서 및 연결 정확도','목표 85% = 119/140, 3%p 점수는 작성자 제안. 계획 140회/구성. 실제 실행 0회, 정확도·점수 미산출.','L');
 header(s,5,['구성','계획 N','성공 S','실패 F','미판정 U','공통 무효','확정 N','QA07 %','하한 %','상한 %','점수 /6','실행 동결']);
 s.getRange('A14:L14').merge();s.getRange('A14').values=[['최초 사용한 순서와 연결만 판정. 전체 관측 60초 무판단은 실패. 응답 내용·음성 전달·Agent 실행 성공은 점수에서 제외.']];s.getRange('A14').format.rowHeight=30;
 s.getRange('A15:T15').merge();s.getRange('A15').values=[['노란 칸은 관측/평가자 근거 입력. 두 판정은 모든 요청 절·최초 부분 채택을 검토한 결과여야 함. 원시 이력과 정규화 구조는 JSON 참조로 보존. 실행/fixture 동결은 별도 검증 후 기록.']];s.getRange('A15').format.rowHeight=30;
 header(s,16,['구성','원전 입력','반복','순서 유형','순서 판정','연결 판정','판정/원시 증거','평가자1 참조','평가자2 참조','정규화·불일치 검토','입력 제공 증거','최초 사용 증거','최초 사용 초','60초 무판단 완전관측','실행 증거 키','fixture 완비','이전 입증 오류','상태','키 중복 수','공통 무효 근거']);
 s.getRange(`A17:T${end}`).values=q.records.map(r=>[r.config_id,r.source_event_id,r.repeat_id,m.cases.find(c=>c.source_event_id===r.source_event_id).order_type,'','','','','','','','','','','','NO','','',0,'']);
 s.getRange(`A17:T${end}`).format={rowHeight:42,wrapText:true};s.getRange(`E17:O${end}`).format.fill='#FFF0CB';s.getRange(`Q17:Q${end}`).format.fill='#FFF0CB';
 for(const c of ['E','F'])s.getRange(`${c}17:${c}${end}`).dataValidation={rule:{type:'list',values:['PASS','FAIL','UNKNOWN']}};
 q.records.forEach((_,i)=>{const r=i+17,pr=6+Math.floor(i%140/5);s.getRange(`S${r}`).formulas=[[`=IF(O${r}="",0,COUNTIFS($A$17:$A$${end},A${r},$O$17:$O$${end},O${r}))`]];s.getRange(`T${r}`).formulas=[[`=IF('QA07선정입력'!H${pr}="","",'QA07선정입력'!H${pr})`]];
 const reviewed=`AND(G${r}<>"",H${r}<>"",I${r}<>"",H${r}<>I${r},J${r}<>"",K${r}<>"",L${r}<>"",ISNUMBER(M${r}),M${r}>=0,M${r}<=60,O${r}<>"")`;
 s.getRange(`R${r}`).formulas=[[`=IF(T${r}<>"","INVALID",IF(AND(Q${r}<>"",K${r}<>"",O${r}<>""),"FAIL",IF(AND(N${r}<>"",K${r}<>"",O${r}<>"",OR(L${r}="",AND(ISNUMBER(M${r}),M${r}>60))),"FAIL",IF(${reviewed},IF(OR(E${r}="FAIL",F${r}="FAIL"),"FAIL",IF(AND(E${r}="PASS",F${r}="PASS",P${r}="YES",S${r}=1),"PASS","UNKNOWN")),"UNKNOWN"))))`]];});
 d.configs.forEach((c,i)=>{const r=i+6,a=17+i*140,b=a+139;s.getRange(`A${r}:B${r}`).values=[[c.id,140]];s.getRange(`L${r}`).values=[['NO']];s.getRange(`C${r}:F${r}`).formulas=[['PASS','FAIL','UNKNOWN','INVALID'].map(v=>`=COUNTIFS(R${a}:R${b},"${v}")`)];s.getRange(`G${r}:K${r}`).formulas=[[
 `=IF(OR(L${r}<>"YES",COUNTIFS(P${a}:P${b},"YES")<>B${r},COUNTIFS(S${a}:S${b},">1")>0,F${r}>0),"미동결",B${r})`,
 `=IF(OR(NOT(ISNUMBER(G${r})),E${r}>0),"미산출",100*C${r}/G${r})`,
 `=IF(ISNUMBER(G${r}),100*C${r}/G${r},"미산출")`,
 `=IF(ISNUMBER(G${r}),100*(C${r}+E${r})/G${r},"미산출")`,
 `=IF(ISNUMBER(H${r}),IF(H${r}=100,6,IF(H${r}<85,0,IF(H${r}<88,1,IF(H${r}<91,2,IF(H${r}<94,3,IF(H${r}<97,4,5)))))),"미산출")`]];});
 s.getRange('G6:K12').format.horizontalAlignment='center';p.getRange('N6:Q26').format.horizontalAlignment='center';s.getRange('H6:K12').setNumberFormat('0.00');s.freezePanes.freezeRows(16);s.freezePanes.freezeColumns(3);
 const main=wb.worksheets.getItem('전체비교'),cmp=['BASE','DP2-B','BASE','DP3-B-R1','DP3-B-R2','DP3-B-R3','BASE','DP5-B','BASE','DP44-B'];cmp.forEach((id,i)=>main.getRange(`G${21+i}`).formulas=[[`='QA07순서연결'!H${6+d.configs.findIndex(c=>c.id===id)}`]]);
 main.getRange('A3').values=[['분석 모델 v9. QA07은 업무 순서/연결의 별도 28입력·5회 평가안. 실제 실행 0회. 나머지 QA 분석 값/미측정 보존.']];main.getRange('A32').values=[['QA07순서연결/QA07선정입력 참조. 85%는 제안. QA07 입력은 기존 24입력과 다르며 시간·비용 값을 전용하지 않는다.']];
 for(const name of ['QA07사례','QA07의무','QA07v3사례','QA07v3의무','QA07v3명세'])wb.worksheets.getItem(name).getRange('A3').values=[['이전 QA07 범위의 보존 자료. 새 업무 순서/연결 대표값에는 사용하지 않음. QA07순서연결/QA07선정입력 참조.']];
 wb.worksheets.getItem('QA상세부록').getRange('A3').values=[['이전 QA07 유형은 보존. 새 순서 있음 2 / 없음 26 입력 및 독립 요청 부록은 QA07선정입력 J:Q를 확인한다.']];
 // Disposable evidence/denominator probes. All observations are cleared before export.
 const good=i=>['PASS','PASS','test-evidence','reviewer-1','reviewer-2','test-normalization','test-input','test-first',1,'',`test-run-${i}`,'YES',''];
 s.getRange('E17:Q156').values=Array.from({length:140},(_,i)=>good(i));s.getRange('L6').values=[['YES']];
 assert.equal(s.getRange('H6').values[0][0],100);assert.equal(s.getRange('K6').values[0][0],6);assert.equal(p.getRange('Q6').values[0][0],100);
 s.getRange('Q17:Q37').values=[['test-proven-error']];assert.equal(s.getRange('H6').values[0][0],85);assert.equal(s.getRange('K6').values[0][0],1);
 s.getRange('O18').values=[['test-run-0']];assert.equal(s.getRange('G6').values[0][0],'미동결');assert.equal(p.getRange('Q7').values[0][0],'미산출');s.getRange('O18').values=[['test-run-1']];
 s.getRange('Q17:Q37').clear({applyTo:'contents'});s.getRange('I17').values=[['reviewer-1']];assert.equal(s.getRange('R17').values[0][0],'UNKNOWN');assert.equal(s.getRange('H6').values[0][0],'미산출');s.getRange('I17').values=[['reviewer-2']];
 s.getRange('L17:M17').clear({applyTo:'contents'});s.getRange('N17').values=[['test-complete-60s-no-adoption']];assert.equal(s.getRange('R17').values[0][0],'FAIL');s.getRange('L17:M17').values=[['test-late-first',61]];assert.equal(s.getRange('R17').values[0][0],'FAIL');s.getRange('N17').clear({applyTo:'contents'});s.getRange('M17').values=[[1]];s.getRange('E17:F17').values=[['FAIL','UNKNOWN']];assert.equal(s.getRange('R17').values[0][0],'FAIL');s.getRange('E17:F17').values=[['UNKNOWN','FAIL']];assert.equal(s.getRange('R17').values[0][0],'FAIL');s.getRange('K17').clear({applyTo:'contents'});assert.equal(s.getRange('R17').values[0][0],'UNKNOWN');
 p.getRange('H6').values=[['test-common-invalid']];for(let i=0;i<7;i++)assert.equal(s.getRange(`R${17+i*140}`).values[0][0],'INVALID');p.getRange('H6').clear({applyTo:'contents'});
 s.getRange('E17:Q156').clear({applyTo:'contents'});s.getRange('P17:P156').values=[['NO']];s.getRange('L6').values=[['NO']];assert.equal(s.getRange('E6').values[0][0],140);
 return {views:[['QA07순서연결','A1:L12','qa07-summary.png'],['QA07순서연결','A16:F20','qa07-records.png'],['QA07순서연결','G16:N19','qa07-evidence.png'],['QA07선정입력','A5:G8','qa07-inputs.png'],['QA07선정입력','A12:G14','qa07-order.png'],['QA07선정입력','J2:Q12','qa07-types.png'],['QA07선정입력','A37:B40','qa07-excluded.png']],recordRows:q.records.length};
}
