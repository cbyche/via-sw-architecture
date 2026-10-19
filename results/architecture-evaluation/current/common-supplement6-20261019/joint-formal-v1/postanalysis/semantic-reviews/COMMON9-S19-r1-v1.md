# COMMON9 S19 r1 독립 의미 검토 v1

검토자: GPT-6 Astra Ultra. 정식 실행 중 승인된 PRIMARY-r1-S19, ordinal 12, job indexes 84–90의 일곱 구성만 대상으로 한 사후 의미 진단이다. 새로운 측정, 사전 승인 또는 전체 실험 결과가 아니다.

전체 행과 원본 연결: [COMMON9-S19-r1-v1.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/COMMON9-S19-r1-v1.json), SHA-256 `bf87962430b8866924f0d6cc73eb946cc117d0370087b8a175ed02f915112c1b`, 604704 bytes.

## 1. 결론과 집계 단위

실제 응답 의무 21개 모두 본문이 관측되었다. 그중 7개는 사용자의 짧은 TCP/UDP 질문에 대한 설명이며 14개는 별도 BG Task의 실제 제공된 진행 및 완료 사실 전달이다. 각각의 범위에서 요청 또는 입력 사실과 일치한다. 이를 21개의 독립 사용자 질문 완료나 14개의 실제 외부 업무 성과로 확대하지 않는다.

| 항목 | 수 | 범위 |
| --- | ---: | --- |
| 계획 응답 의무 | 21 | U01, A01, A02 각 7개 |
| 질문 본문 PASS | 7 | 실제 짧은 일반지식 설명 요청 충족 |
| BG 사실 전달 PASS | 14 | 실제 작성된 Agent 입력과 Task 연결에 대한 충실한 전달 |
| 관측 본문 FAIL / 미표시 UNREVIEWED | 0 / 0 | 모든 응답 행에 실제 본문 존재 |
| 고유 실제 DISPLAY / 응답 callback 참조 | 21 / 21 | 추가 공유 참조, 중복 표시 및 대체된 응답 없음 |
| 사용자 목표 관계 PASS | 7 | NEW/DIRECT, U01의 독립 목표 revision 1 |
| 원시 사용자 HOST_COMPLETE | 7 | 본문/상태 범위, 실제 음성 또는 외부 제품 보증 아님 |
| BG 완료 파일의 실제 생성 검증 | UNREVIEWED 7 | 알림의 진실한 전달과 물리적 파일 검증은 별개 |
| 전체 음성 요약 준비 및 생성 | NOT_EXERCISED 7 | 동결 Voice OFF 범위 |
| 일반지식에 대한 공통 source 문구 적용 | UNRESOLVED 7 | 본문 요청 충족과 별도인 provenance 적용 모호성 |

7개 목표 행은 같은 7개 U01 본문의 관계를 확인한 것이며 추가 응답 성공이 아니다. 본문이 없거나 검토가 부족한 행을 PASS로 채우지 않았고, 본문이 모두 관측된 이번 블록에서는 missing body가 0이다.

## 2. 실제 입력과 해석 기준

실제 U01은 모든 구성에서 `TCP와 UDP의 차이를 짧게 설명해줘.`이다. 사용자와 frozen response oracle은 글자 수, 문장 수 상한, 인용 또는 특정 문서 사용을 요구하지 않는다. 두 문장이라는 실제 출력 형식을 관측 사실로 기록하며, 이를 새 수치 기준으로 만들지 않는다.

별도 입력 A01은 `다른 Task BG의 확인된 진행 50%`, A02는 `다른 Task BG의 최종 파일 완료`이다. 두 입력의 task_id는 BG이고 event_class는 각각 PROGRESS와 RESULT이다. 초기 BG는 RUNNING, allowed_actions=[observe]이다. 모든 구성에서 세 입력의 실제 제공과 INPUT_CAPTURED가 존재한다.

질문 본문은 TCP의 연결 지향 및 신뢰성/순서 기능과 UDP의 비연결 특성을 비교한다. 실제 답변에 있는 `속도가 상대적으로 느립니다`, `전송 속도가 빠르고` 등의 표현은 조건을 생략한 통상적인 짧은 비교다. 이를 모든 환경에서 UDP가 더 빠르다는 보증이나 실측 latency 결론으로 인정하지 않는다. 실제 본문에는 “항상”, “예외 없이” 또는 숫자 지연 보장 표현이 없다. 이 일반화의 한계는 보존하지만 요청하지 않은 정량 성능 검증이나 인용 요구를 사후 추가하지 않는다.

### 실제 U01 본문 일곱 개

| 구성 | 실제 표시 본문 |
| --- | --- |
| AAAC | TCP는 연결 지향적이며 데이터의 순서와 신뢰성을 보장하지만 속도가 상대적으로 느립니다. 반면 UDP는 비연결형으로 신뢰성은 낮지만 전송 속도가 빠르고 실시간 통신에 적합합니다. |
| AAAA | TCP는 연결 지향적이며 데이터의 신뢰성과 순서를 보장하지만 속도가 상대적으로 느립니다. 반면 UDP는 비연결형으로 신뢰성 보장은 없지만 전송 속도가 빠르고 실시간 통신에 적합합니다. |
| BAAA | TCP는 연결 지향적이며 데이터의 신뢰성과 순서를 보장하지만 속도가 상대적으로 느립니다. 반면 UDP는 비연결형으로 신뢰성 보장은 없지만 전송 속도가 빠르고 실시간 서비스에 적합합니다. |
| CAAA | TCP는 연결 기반의 신뢰성 있는 전송 프로토콜로, 데이터 순서 보장과 재전송 기능을 제공합니다. 반면 UDP는 비연결형 프로토콜로, 신뢰성은 낮지만 속도가 빠르고 실시간 통신에 적합합니다. |
| ABAA | TCP는 연결 기반의 신뢰성 있는 전송을 보장하며 순서와 오류 제어를 수행합니다. 반면 UDP는 비연결형으로 신뢰성은 낮지만 전송 속도가 빠르고 실시간 서비스에 적합합니다. |
| AABA | TCP는 연결 지향적이며 데이터의 신뢰성과 순서를 보장하지만 속도가 상대적으로 느립니다. 반면 UDP는 비연결형으로 신뢰성 보장은 없지만 전송 속도가 빠르고 실시간성에 유리합니다. |
| AAAB | TCP는 연결 지향적이며 데이터의 순서와 신뢰성을 보장하지만 속도가 상대적으로 느립니다. 반면 UDP는 비연결형으로 신뢰성은 낮지만 전송 속도가 빠르고 실시간성에 유리합니다. |

## 3. 원문 provenance와 권한

일곱 실제 meaning request 모두 `actually_supplied_originals=[]`이고 채택 계획은 `source_claims=[]`, `source_revisions={}`이다. 모델은 일반지식 설명을 했으며 DOC-A, 과거 보고서 또는 BG 결과를 네트워크 설명의 출처로 허위 인용하지 않았다. 실제 요청은 인용을 요구하지 않았으므로 이것을 본문 실패로 바꾸지 않는다.

동일한 공통 system instruction에는 `Facts must use actually supplied originals with exact quote claims.`가 있다. 이 문구는 일반지식 질문에 대한 예외를 명시하지 않는다. 일반지식 답변에까지 원문 인용을 필수로 적용해야 하는지의 문구상 범위는 별도 UNRESOLVED로 남긴다. “사용자 요청 충족 PASS”를 “모든 공통 instruction의 문자적 준수 PASS”로 확장하지 않는다. 실제 일곱 request의 system 문자열 SHA-256은 `9cb03004c774e384bc1299d83a19319e2ab2b22febc608d93a0d72f6bc1e90c9`이며 전문과 파일 해시는 JSON에 있다.

초기 DOC-A와 CONVERSATION-LOG의 읽기 권한과 source catalog를 확인했다. 실제 U01 채택의 allowed_actions, completed_actions, permission_operations 및 control_operations는 모두 빈 배열이다. 새로운 외부 ingress도 없다. 승인과 handoff는 NOT_APPLICABLE이며 권한 철회나 충돌 회복의 검증 성공으로 세지 않는다.

CAAA에서는 evidence producer의 원문 조회와 관계 준비가 관측되고, AAAB에서는 handoff work-record 준비와 폐기/commit이 관측된다. 이것들은 실제 사용자 handoff ingress가 아니다. 최종 ingress는 일곱 구성 모두 비어 있다. “현재 질문에 원문이 공급되지 않았다”는 사실을 “episode 전체에서 producer 조회나 준비 비용이 없었다”로 바꾸지 않는다. 이 검토는 호출 비용과 시간을 재산출하지 않았다.

## 4. 모든 응답 의무와 원본 위치

`/events/N`은 해당 run의 trace 내 pointer이다. 원본 trace 해시는 §8과 JSON의 raw_ref에 바인딩되어 있다. 이번 검토는 승인된 bundle 및 연결된 작은 model 파일을 읽었으며 trace를 새로 추출하거나 전체 재평가하지 않았다.

| 구성 | 의무 | 실제 capture | 실제 표시 / raw first | 판정과 종류 |
| --- | --- | --- | --- | --- |
| AAAC | S19-RU01 | `/events/16` / `event-19` | `/events/49` / `event-58` | PASS / 질문 설명 |
| AAAC | S19-RA01 | `/events/6` / `event-7` | `/events/10` / `event-13` | PASS / BG 사실 전달 |
| AAAC | S19-RA02 | `/events/23` / `event-27` | `/events/28` / `event-33` | PASS / BG 사실 전달 |
| AAAA | S19-RU01 | `/events/16` / `event-19` | `/events/49` / `event-58` | PASS / 질문 설명 |
| AAAA | S19-RA01 | `/events/6` / `event-7` | `/events/10` / `event-13` | PASS / BG 사실 전달 |
| AAAA | S19-RA02 | `/events/23` / `event-27` | `/events/27` / `event-33` | PASS / BG 사실 전달 |
| BAAA | S19-RU01 | `/events/16` / `event-19` | `/events/46` / `event-55` | PASS / 질문 설명 |
| BAAA | S19-RA01 | `/events/6` / `event-7` | `/events/10` / `event-13` | PASS / BG 사실 전달 |
| BAAA | S19-RA02 | `/events/23` / `event-27` | `/events/28` / `event-33` | PASS / BG 사실 전달 |
| CAAA | S19-RU01 | `/events/25` / `event-31` | `/events/62` / `event-74` | PASS / 질문 설명 |
| CAAA | S19-RA01 | `/events/15` / `event-19` | `/events/19` / `event-25` | PASS / BG 사실 전달 |
| CAAA | S19-RA02 | `/events/32` / `event-39` | `/events/37` / `event-46` | PASS / BG 사실 전달 |
| ABAA | S19-RU01 | `/events/16` / `event-19` | `/events/49` / `event-58` | PASS / 질문 설명 |
| ABAA | S19-RA01 | `/events/6` / `event-7` | `/events/10` / `event-13` | PASS / BG 사실 전달 |
| ABAA | S19-RA02 | `/events/23` / `event-27` | `/events/28` / `event-33` | PASS / BG 사실 전달 |
| AABA | S19-RU01 | `/events/16` / `event-19` | `/events/49` / `event-58` | PASS / 질문 설명 |
| AABA | S19-RA01 | `/events/6` / `event-7` | `/events/10` / `event-13` | PASS / BG 사실 전달 |
| AABA | S19-RA02 | `/events/23` / `event-27` | `/events/28` / `event-33` | PASS / BG 사실 전달 |
| AAAB | S19-RU01 | `/events/27` / `event-33` | `/events/60` / `event-72` | PASS / 질문 설명 |
| AAAB | S19-RA01 | `/events/17` / `event-21` | `/events/21` / `event-27` | PASS / BG 사실 전달 |
| AAAB | S19-RA02 | `/events/34` / `event-41` | `/events/39` / `event-47` | PASS / BG 사실 전달 |

A01과 A02의 실제 표시 문장은 각각 해당 capture 문장과 정확히 같다. 각 표시에는 같은 publication_id의 accepted host receipt와 PUBLICATION_DELIVERY_STATE 기록이 연결된다. BG 기록의 `task_fact_state_recorded=true`는 실제 Task 사실 경로를 나타내며 user goal 완료 또는 delivered-history 등록을 발명하지 않는다. 별도 사용자 U01에는 실제 semantic adoption, 표시, receipt, 현재 goal 완료 기록이 있다. 이 연결은 의미 진단에 사용했으며 clock이나 QA endpoint 적격성을 새로 인증하지 않는다.

BG 완료는 승인된 합성 Agent 사실의 전달이다. 실제 commercial Agent의 파일 생성, 파일 내용 및 외부 작업 성능은 이 bundle로 입증되지 않는다. A02의 문자열을 원문과 동일하게 전달한 PASS와 물리적 파일 생성 UNREVIEWED를 함께 남긴다.

## 5. 사용자 목표와 현재 Task 관계

모든 실제 의미 proposal은 NEW/DIRECT, task_id=null, target_goal_id=null, target_event_id=null이다. 실제 채택은 goal:S19-U01 revision 1을 만들고 origin/last event를 S19-U01로 유지한다. 실제 model response와 채택 plan은 provider usage metadata를 제외하면 일치한다. BG는 별도 Task이며 U01이 BG에 흡수되거나 BG 완료가 U01 본문으로 대체된 증거가 없다.

| 구성 | 실제 semantic adoption | 실제 GOAL_COMPLETE | 최종 request | 관계 판정 |
| --- | --- | --- | --- | --- |
| AAAC | `/events/47` / `event-54` | `/events/54` / `event-63` | `/events/60/data/requests/goal:S19-U01` / `event-69` | PASS / U01 revision 1, HOST_COMPLETE |
| AAAA | `/events/47` / `event-54` | `/events/54` / `event-63` | `/events/60/data/requests/goal:S19-U01` / `event-69` | PASS / U01 revision 1, HOST_COMPLETE |
| BAAA | `/events/44` / `event-51` | `/events/51` / `event-60` | `/events/57/data/requests/goal:S19-U01` / `event-66` | PASS / U01 revision 1, HOST_COMPLETE |
| CAAA | `/events/60` / `event-70` | `/events/67` / `event-79` | `/events/92/data/requests/goal:S19-U01` / `event-108` | PASS / U01 revision 1, HOST_COMPLETE |
| ABAA | `/events/47` / `event-54` | `/events/54` / `event-63` | `/events/60/data/requests/goal:S19-U01` / `event-69` | PASS / U01 revision 1, HOST_COMPLETE |
| AABA | `/events/47` / `event-54` | `/events/54` / `event-63` | `/events/60/data/requests/goal:S19-U01` / `event-69` | PASS / U01 revision 1, HOST_COMPLETE |
| AAAB | `/events/58` / `event-68` | `/events/65` / `event-77` | `/events/96/data/requests/goal:S19-U01` / `event-112` | PASS / U01 revision 1, HOST_COMPLETE |

각 의미 request 시점의 실제 public_state에는 BG.last_progress와 BG.last_result가 실제 공급 문장으로 존재하고 BG.state=COMPLETED이다. 사용자 requests는 채택 이전에 비어 있다. 따라서 초기 snapshot을 이후 ingress로 착각하거나 미래 Agent 사실을 만들어서 관계 PASS를 부여하지 않았다.

동결 goal oracle에는 상세 응답과 상태 확정 및 필요한 전체 음성 요약 준비라는 넓은 표현이 남아 있다. 그러나 계약19 80–82행은 Voice OFF, 전사 입력 및 공통 text callback을 명시한다. 이 검토의 일곱 목표 PASS는 실제 요청-목표 관계와 text 처리 범위에 한정하며 TTS, 실제 audio, 전체 음성 요약 준비를 구현하거나 검증했다고 하지 않는다.

## 6. 원시 실패와 동결 중단 조건

일곱 copied terminal record를 각각 SHA와 bytes에 대조했다. 모두 returncode=0, technical_failure=false, budget_stop=false이고 host failures는 빈 배열이다. 최종 candidate failures와 pending_inputs도 비어 있으며 ingress가 없다. 이 블록에는 missing response, anchor timeout, proposal rejection 또는 실제 wrong-Task 결과가 확인되지 않았다.

동결 common9 harness.py 295–296행은 Voice OFF/text callback과 ordinary anchor failure 대 technical failure를 구분한다. joint_campaign.py 153–194행은 정확한 동결/승인과 HOLD, 예산, nonzero returncode, technical failure 및 budget stop에 반응한다. 이번 관측에서 새 기술 중단 사유는 확인되지 않았다. 일반화된 속도 표현이나 일반지식 source 문구의 모호성을 새 harness/clock 결함으로 바꾸지 않는다.

이는 “기술적으로 전부 인증됨”이라는 판정이 아니다. saved strict evaluation과 certified endpoint는 제공되지 않았고 새 평가기를 실행하지 않았다. contract19 99–111행에 따라 본문/기능 진단을 시간 관측과 별도로 보존한다. 원래 QA01/QA02/QA03 endpoint, 모집단 및 분모를 수정하거나 PASS 본문만 남기지 않는다. 튜닝, 재실행, controller 취소 또는 다음 cursor 진행도 하지 않았다.

## 7. 검토 범위의 제한

- S19 첫 반복, 일곱 구성만 검토했다. 전체 315/126 모집단의 결과나 DP 우열로 확대하지 않는다.
- 모든 본문이 보였다는 사실과 실제 물리적 제품 또는 전체 음성 기능이 완성되었다는 주장을 구별했다.
- actual BG fact, user goal, source-backed claim, 원시 HOST_COMPLETE와 certified QA endpoint는 서로 다른 증거이다.
- 이 사례에는 정정, source 권한 철회, 승인 선택 또는 실제 외부 handoff가 없으므로 해당 경합 기능은 NOT_EXERCISED이다.
- 원본 trace 재읽기, corpus scan, evaluator/analyzer, model/parser, test/build 및 network/API 호출은 0이다. frozen 자료 수정과 자동 다음 cursor 진행도 없다.

## 8. 정확한 파일 연결

| 역할 | 파일 | SHA-256 |
| --- | --- | --- |
| freeze | [freeze.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/freeze.json) | `0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd` |
| manifest | [manifest.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S19-r1-v3-r2/manifest.json) | `4c99803e2a1c0f70d878c7ddcec0d2d0278e52fff690339a850ccc67afc4da96` |
| evidence | [evidence.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S19-r1-v3-r2/evidence.json) | `de0c3f09acc7fabc7fbe78dcbe28e72dbae13d9db44de9734eb7c1417d7aa520` |
| worklist | [worklist.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/blocks/COMMON9-S19-r1-v3-r2/worklist.json) | `f12c1c87c9b7a2a9dad8bc8af69c141e143e7fbe93c71e0ff921b1d1fb982b09` |
| index | [index.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/block-indexes/COMMON9-S19-r1-v3-r2/index.json) | `591ec0f49b6af97ed7ebdf953d78565e051e1496c0b6a9b41585ea6788ecc358` |
| receipt | [EXECUTION-RECORD.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/review-cursors/COMMON9-S19-r1-v3-r2/EXECUTION-RECORD.json) | `01212c611b088c3b2e130efcc92c4ea2a308184450923adae12f8a19932bf1ab` |
| fixtures | [fixtures.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/fixtures.json) | `529d8f41cd72bd861e84475697c5af3cd64e070f559801e71de1d715c83c6d79` |
| oracles | [oracles.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/oracles.json) | `74b434872444c2a303e0f1bcee1ba162e519386a82620c1a74c2666142f5da52` |
| contract19 | [19-common9-evaluation-contract.md](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md) | `7b3db7b8f358ee6cd441ca37ef13f9b83d1f697322729a0f7bde802bbdc162e7` |
| harness | [harness.py](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/harness.py) | `d30750622e97978f769da8366a8c12510e91cbd482ead32d1111ed9cc7e079d9` |
| joint_campaign | [joint_campaign.py](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common_supplement6_20261019/joint_campaign.py) | `9314f4f36f58ae2254e0f2d0cb33d4fe5debba5c43f450b696154068745c8d50` |

### 원본 trace 바인딩

해시는 accepted bundle의 trace 바인딩을 보존한 것이다. 이 검토에서 해당 trace 파일들을 새로 읽지 않았다.

| Run | 원본 trace SHA-256 | bytes |
| --- | --- | ---: |
| [AAAC-S19-r1-08df81](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAC-S19-r1-08df81/trace.json) | `c7b75a073e7f034d02d0240e3572be3c57f357252e929aee23b3bfb079bb9f0e` | 2372455 |
| [AAAA-S19-r1-c73672](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAA-S19-r1-c73672/trace.json) | `944cd2f49cd9cbdc5221ea3c19784439433e62f0d92d9cc4c124400536dbd712` | 2425608 |
| [BAAA-S19-r1-f777c4](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/BAAA-S19-r1-f777c4/trace.json) | `956904ab63fb6488619116b55d0a6591cb83d39acdf008a74b5f28fc1b12163b` | 2432867 |
| [CAAA-S19-r1-ec6811](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/CAAA-S19-r1-ec6811/trace.json) | `a294db0f9a3102c9ceb13f2926554685e660db9486901b1da636dce58db57b0a` | 2702084 |
| [ABAA-S19-r1-e1a0c6](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/ABAA-S19-r1-e1a0c6/trace.json) | `0caeb0188297b34693d391c7cca48ff8f156af12aea734936b301905eac7388e` | 2345815 |
| [AABA-S19-r1-2aed29](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AABA-S19-r1-2aed29/trace.json) | `fbd399204716e44bb3657d5a958778efb6f242fcc8993f7d259b12c2d689aad2` | 2371136 |
| [AAAB-S19-r1-e07f60](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAB-S19-r1-e07f60/trace.json) | `893098876a8c34f0b3a7535cdf8918e44a40b5aac6ae199c438de1bb9a4d6215` | 2779893 |

### 실제 meaning request/response 연결

각 원본의 SHA와 크기를 작은 파일 단위로 확인했다. CAAA는 calls/2, 나머지는 calls/1이다. 다른 producer 호출은 여기서 사용자 meaning 호출로 잘못 세지 않았다.

| 구성 | 실제 request | 실제 response |
| --- | --- | --- |
| AAAC | [09391bfb-e0fd-4fb6-8c25-18e48deebfaf-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/09391bfb-e0fd-4fb6-8c25-18e48deebfaf-request.json) / `8eab5a3974ece9caea2fb7f36ef282369851559efb8cf97f2f9cdd7893fffbc9` | [09391bfb-e0fd-4fb6-8c25-18e48deebfaf-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/09391bfb-e0fd-4fb6-8c25-18e48deebfaf-response.json) / `90c89d65c232302011585436abc8336d9113597868e33368df718ffb739ea183` |
| AAAA | [a42f95bb-2446-40a2-8eba-9bc1429179da-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/a42f95bb-2446-40a2-8eba-9bc1429179da-request.json) / `2a537ff856d392f3d08f737032a091ff7957fc4c7304e6b895e0dfc6bc3485db` | [a42f95bb-2446-40a2-8eba-9bc1429179da-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/a42f95bb-2446-40a2-8eba-9bc1429179da-response.json) / `b531621f3b963dd0fc7a22c1a537008c4202f77c98f5377d8507525df56f14a6` |
| BAAA | [db3cd2ee-59c0-4a52-8945-23b85c231041-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/db3cd2ee-59c0-4a52-8945-23b85c231041-request.json) / `ffeab1a1c010dc73e6b78f9c47699545d56261f676c248ba85ba4d8916c2783c` | [db3cd2ee-59c0-4a52-8945-23b85c231041-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/db3cd2ee-59c0-4a52-8945-23b85c231041-response.json) / `c0ca00f8f5df34cdfdcf3f63db7245037187be4aa78a8e39e46ab208c25900c9` |
| CAAA | [53d236c6-a378-442f-940d-906f1496325d-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/53d236c6-a378-442f-940d-906f1496325d-request.json) / `bcd79b3967d7e1e1f6f3085f703044f777593f4b1fdc036b5a3009c9fca75d10` | [53d236c6-a378-442f-940d-906f1496325d-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/53d236c6-a378-442f-940d-906f1496325d-response.json) / `82a9def43f5a1cb1dbc5119c736050d5836a7d0718e8dc1e487a93e6ca23aad1` |
| ABAA | [de571ae1-51d2-49fd-9288-902c332fc87d-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/de571ae1-51d2-49fd-9288-902c332fc87d-request.json) / `7d4b099edeb48ca2e084119bcd8d07b710286312a6716d09616c174a1983570b` | [de571ae1-51d2-49fd-9288-902c332fc87d-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/de571ae1-51d2-49fd-9288-902c332fc87d-response.json) / `1dc247d7331f43e68c1fc9b78c1fd325853046d8d0a82a08b7dea940b4f82328` |
| AABA | [32967f02-2a20-4c39-888e-f8de73471486-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/32967f02-2a20-4c39-888e-f8de73471486-request.json) / `c9fac36053628279991ee259cc052711563a47f4785a0fd8089cea08661778e3` | [32967f02-2a20-4c39-888e-f8de73471486-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/32967f02-2a20-4c39-888e-f8de73471486-response.json) / `14f861a16855bfd9bae44c97bc4aa186179357a4852169124a9ee03f2bba21a4` |
| AAAB | [84577608-ded4-4df8-91ce-a79aeef057a1-request.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/84577608-ded4-4df8-91ce-a79aeef057a1-request.json) / `d4c8cab56aa70b98b7dc0b88a61060fc11e5e50fb168d943373b466f1d32651f` | [84577608-ded4-4df8-91ce-a79aeef057a1-response.json](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/model/84577608-ded4-4df8-91ce-a79aeef057a1-response.json) / `1e4fc7329221fa35422a08563b400cc56ee7a81cbe1357b2b9ea790280fe73cd` |
