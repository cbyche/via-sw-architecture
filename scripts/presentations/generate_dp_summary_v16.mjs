import fs from 'node:fs/promises';
import path from 'node:path';
import {Presentation,PresentationFile} from '@oai/artifact-tool';
const [repo,build]=process.argv.slice(2);
const p=Presentation.create({slideSize:{width:1920,height:1080}});
const s=p.slides.add();s.background.fill='#FFFFFF';
const rows=[
 ['DP','풀고자 하는 VIA 문제','설계 주제','두 방안의 주된 차이','평가'],
 ['DP-44\n연속 대화 실행','사용자 정정과 끼어들기,\nAgent 질문과 결과가 겹쳐도\n유효한 처리를 계속 이어가기','다음 실행과 대기,\n취소를 연결하는 방식','A  중앙 조정자가 완료를 받고\n     다음 작업을 지시\nB  연결된 처리 단계가 각자\n     조건에 따라 다음 실행을 진행','응답시간과 변경 부담\n시간 차이는 작을 것으로 예상\nA의 변경 범위와 병행 수정\n중복이 작을 것으로 예상'],
 ['DP-47\n화면과 음성\n지칭 해석','화면 전환과 말의 정정 중에도\n“이것”, “아래 것”이 가리키는\n대상을 정확히 식별하기','말과 화면 근거를\n결합하는 방식','A  말과 화면 원본을 공동 해석\nB  말과 화면 후보를 별도 생산해\n     교환하고 결합\n     필요하면 원본에서 후보 확장','첫 응답 지연과 지칭 정확도\n현재 예상은 A가 유리\nB의 후보 재사용 이익과\n최종 정확도 차이는 검증 필요'],
 ['DP-45\n기억과 Context\n구성','“아까 설명한 것”처럼\n과거 대화와 업무를 참조할 때\n필요한 근거를 찾기','과거 기록의 관계를\n만드는 시점과\n재사용 방식','A  요청 시 원본과 유효한 캐시를\n     모아 해석\nB  관계를 미리 생산하고 저장해\n     요청 시 조회','응답속도와 준비 및 유지 비용\nB는 조회 속도에서 유리할 가능성\nA는 모델 비용과 RAM,\n변경 부담에서 유리할 것으로 예상'],
 ['DP-42\n대화와 업무의\n수명 관리','대화나 요청이 끝난 뒤에도\n남은 업무와 질문, Agent 실행을\n올바르게 연결하기','상태 소유권과\n서비스 실행 경계','A  모듈화된 하나의 Core에서\n     관련 상태를 함께 확정\nB  대화와 업무 서비스가 각각\n     상태를 확정하고 접수 결과로 연결','변경 범위와 RAM, 처리시간\nA의 변경 및 자원 부담이 작을 것으로 예상\nB의 독립 운영이 정상 사용에서\n주는 이익은 추가 검증 필요'],
];
const t=s.tables.add({rows:5,columns:5,left:50,top:168,width:1820,height:800,columnWidths:[190,360,260,500,510],values:rows});
t.borders.assign({fill:'#929292',width:1,style:'solid'});
for(let r=0;r<5;r++){
 t.rows[r].height=r===0?64:184;
 for(let c=0;c<5;c++)t.cells.block({row:r,column:c,rowCount:1,columnCount:1}).assign({fill:r===0?'#C9E0B4':'#FFF8E5',margins:{left:14,right:12,top:12,bottom:12},anchor:'center',textStyle:{typeface:'Calibri',fontSize:r===0?28:80/3,bold:r===0||c===0,color:'#000000',alignment:r===0?'center':'left',autoFit:'none'}});
}
const foot=s.shapes.add({name:'dp-summary-evaluation-status',geometry:'textbox',position:{left:55,top:994,width:1810,height:42},fill:'none',line:{fill:'none',width:0}});
foot.text='평가는 현재 구조와 동일 관련 시나리오에 따른 예상이다. 실측 결과와 최종 A/B 선정은 아직 없다.';
foot.text.style={typeface:'Calibri',fontSize:24,color:'#000000',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
s.speakerNotes.textFrame.setText('DP-44, DP-47, DP-45, DP-42 순서의 요약이다. 문제, 설계 주제, 두 방안의 차이, 평가를 설명한다.\nDP44의 양안 모두 비동기 처리를 지원한다. DP47의 양안 모두 원본과 부분 정정을 지원한다. DP45의 A에도 유효한 캐시를 허용한다. DP42의 A도 대화와 업무의 서로 다른 논리 수명을 지원한다.\n평가 근거: docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa-dp-thought-experiment.md (2026-10-15 v2). 구조 기반 예상이며 측정값이나 최종 선택이 아니다.\n구조 원문: 04-44-continuous-interaction.md, 04-47-screen-voice-grounding.md, 04-45-memory-and-context.md, 04-42-lifecycle-ownership.md.');
await fs.mkdir(build,{recursive:true});
await(await PresentationFile.exportPptx(p)).save(path.join(build,'summary-source.pptx'));
console.log('Created editable summary table');
