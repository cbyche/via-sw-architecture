// Calibri's Korean fallback uses different metrics. Give the dense comparison
// tables a clear lower margin while keeping their editable structure intact.
export function adjustWindowsLayout(source,dp){
 let body=source;
 const replace=(from,to)=>{
  if(!body.includes(from))throw new Error(`Missing DP${dp} layout pattern: ${from.slice(0,80)}`);
  body=body.replace(from,to);
 };
 if(dp===2){
  replace("shape(s,'rect',1046,174,850,824", "shape(s,'rect',1046,174,850,842");
  replace("28,1007,1810,27,21","28,1023,1810,24,19");
  replace("28,1007,1830,27,21","28,1023,1830,24,19");
  body=body.replaceAll("28,1040,1805,27,20","28,1052,1805,22,17");
 }
 if(dp===3){
  replace("shape(s,'rect',196,174,850,836", "shape(s,'rect',196,174,850,854");
  body=body.replaceAll("28,1016,1810,27,20","28,1035,1810,21,17");
  body=body.replaceAll("28,1016,1840,27,20","28,1035,1840,21,17");
  replace("28,1047,1805,25,19","28,1058,1805,19,16");
 }
 if(dp===4){
  replace("shape(s,'rect',1328,174,568,852", "shape(s,'rect',1328,174,568,874");
  replace(
   "txt(s,'C1은 조건 누락 위험의 보완. 적용 QA의 상대적 열세는 입증되지 않음. 별도 E3 수치는 전체 기본값과 직접 연결하지 않음.',28,1032,1840,22,18,{color:C.blue});",
   "txt(s,'C1은 조건 누락 보완 / 적용 QA 열세 미확인 / E3는 별도 실험 / PoC 4사례×3',28,1053,1810,20,16,{color:C.blue});"
  );
  replace(
   "txt(s,'QA01 우열 미확정. C는 실제 범위 읽기·권한·판본 검사를 지원하는 수신자 조건. 본문 충족은 QA02와 별도 진단.',28,1032,1840,22,18,{bold:true});",
   "txt(s,'QA01 우열 미확정 / 본문 충족은 QA02와 별도 진단 / PoC 4사례×3',28,1053,1810,20,16,{bold:true});"
  );
  replace(
   "txt(s,'기존 완료 PoC: 안별 4사례×3 / 시간 괄호: 관측·예정 / 비용: 12회 합계(수신자 포함) / RSS: 로컬 실행기 / QA03은 수신자 HTTP 시간 제외',28,1057,1805,19,16,{color:C.muted});",
   ""
  );
 }
 return body;
}
