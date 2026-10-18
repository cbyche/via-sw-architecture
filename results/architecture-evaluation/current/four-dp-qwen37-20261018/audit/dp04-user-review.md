# DP04 사용자 리뷰 대응 독립 감사

2026-10-18. API 호출, 후보/정답 변경, 실험 재실행 없음. [파생 원시 집계](dp04-user-review.json)는 `python3 results/architecture-evaluation/current/four-dp-qwen37-20261018/audit/dp04-user-review.py`로 재현한다. 아래의 시간은 cloud 내부 추론 시간만이 아니라 bridge dispatch, network, provider 반환을 포함한다.

## 1. 답변에 사용할 결론

**현재 데이터는 ‘이번 Qwen 생산자 구현에서 A가 같은 지칭 정확도로 더 빠르고 저렴했다’를 지지한다. B 구조 일반의 열세나 강한 전용 parser의 무효를 입증하지 않는다.** B의 추가 단계와 후보 출력이 실제 비용/시간을 만들었지만, 화면 생산 능력과 관계 검증도 미흡했다. B의 반대편 QA 이익은 BASIC에서 하나도 확인되지 않았다. 같은 Qwen을 역할별로 나눈 PoC와 설계 문서가 인용한 detector/OCR/caption parser를 구별해야 한다.

## 2. 약 4배 시간·3배 비용의 실제 내역

양쪽 BASIC은 12 episode run, 입력 revision 15개, 최초 지칭 판단 24개다. 부분 정정 episode에 revision이 2개이므로 호출 수 분모와 episode 수가 다르다.

| 항목 | A | B |
| --- | ---: | ---: |
| QA01 표시 proxy 평균 | 5,710.38 ms | 23,785.96 ms (A의 4.165배) |
| QA03 | 5,710.43 ms | 23,786.00 ms |
| 호출 수 | 15 | 39 |
| 입력 token | 21,427 | 36,474 |
| 출력 token | 4,358 | 19,556 |
| 정가 환산 총 비용 | $0.0155436 | $0.0458792 (2.952배) |
| 실제 source 재확장 / 공동 재조회 | 0 / 0 | 0 / 0 |

| B 단계 | 호출 | 입력 / 출력 token | 비용 | 반환까지 총 시간 / 15 revision |
| --- | ---: | ---: | ---: | ---: |
| 화면 생산 | 15 | 19,848 / 13,998 | $0.0303360 (B의 66.1%) | 15,901.47 ms |
| 발화 생산 | 15 | 6,328 / 2,946 | $0.0072448 | 4,352.19 ms |
| 모델 결합 | 9 | 10,298 / 2,612 | $0.0082984 | 3,517.69 ms |
| 합계 | 39 | 36,474 / 19,556 | $0.0458792 | 23,771.35 ms |

A의 같은 계산은 joint 15회, revision당 5,704.89 ms다. 나머지 로컬 관측·채택·표시 차이는 평균 약 A 5.5 ms / B 14.6 ms다. enqueue→send 차이도 B 전체 39회 합계 약 89.77 ms에 불과하다. 따라서 이번 큰 차이를 queue 포화, RAM 회수, 원본 재확장 탓으로 설명하면 틀린다. **B의 매 revision 화면/발화 생산, 9회의 추가 결합, 특히 긴 화면 후보 출력과 그 요청의 반환 대기가 관측된 비용이다.** 출력량이 내부 추론 지연을 얼마나 인과적으로 늘렸는지는 provider 계측 없이 분리하지 못한다.

B는 dense와 page-change에서 typed code 결합으로 6개의 결합 호출을 피했다. free-region과 correction의 총 9 revision은 모델 결합을 했다. 실제 확장/재조회 요청은 0회다. 확장 코드가 존재한다는 사실과 성공적인 확장을 실험했다는 사실을 구별한다.

의미 생산은 모두 해당 revision의 STIMULUS 뒤에 시작한다. 이전부터 만들어 둔 screen 후보를 B에 무료로 주거나 그 준비 시간을 제외한 비교가 아니다. A도 같은 revision의 여러 지칭을 한 번의 joint 호출로 처리했다. 이번 입력은 B가 여러 소비자에게 같은 후보를 공급하여 반복 원본 처리비를 상쇄하는 작업 부하를 검증하지 않는다.

코드: [DP04 dispatch/병행/표시](../../../../../prototypes/candidates/four_dp_poc/src/dp04.rs#L21), [screen 3,500 / speech 1,800 token 한도](../../../../../prototypes/candidates/four_dp_poc/src/dp04/producers.rs#L13), [조건부 code/model 및 expansion](../../../../../prototypes/candidates/four_dp_poc/src/dp04/binding.rs#L64). `#L`은 소스 위치 안내이며 Markdown 파일의 줄 anchor 지원 여부는 viewer에 따른다.

## 3. 62.5% 실패는 무엇인가

양쪽 매 반복 dense 0/2, free-region 0/1, page-change 1/1, correction 4/4다. 따라서 세 번 반복한 24개 expression/revision 판단에서 15개 통과다. correction이 반복당 네 판단을 차지하는 micro 집계이며, 장면 네 종류의 62.5%를 푼 뜻은 아니다. 출처 revision 검사와 늦은 결과 거절은 공통 qualification이고 A/B 고유 정확도 이익으로 세지 않는다.

### PNG와 좌표

직접 확인한 dense/graph PNG는 1000×650이고, dense 카드의 ID는 20px, graph ID는 18px이다. 사람이 확인할 때 글자와 점이 뚜렷하다. 심하게 뭉개져 읽을 수 없는 이미지로 보이지 않는다. pointer/loop 자체는 PNG에 그리지 않고 숫자 좌표/시간을 별도 JSON으로 보냈다. **숫자 좌표와 픽셀 위치를 결합하는 능력이 이 시험의 난점**이다. ‘작은 글자가 읽히지 않아서’라고 단정할 근거가 없다.

bridge는 실제 PNG bytes를 그대로 base64 전송한다. 기존 해시 검사는 요청 이미지와 frozen PNG 일치를 확인했다. provider 내부 resize/patch 처리나 모델의 좌표 추론은 직접 계측하지 않았다. `frame_pixels`, width/height, 좌표 기준은 양쪽에 같다. [이미지/지칭 생성 근거](../../../../../benchmark/architecture/four_dp_poc/assets/screen/build_fixtures.py#L8), [bytes 전송과 공통 모델 설정](../../../../../benchmark/architecture/four_dp_poc/bridge.py#L25).

### 구체 오류

- **Dense 정답:** (375,315)는 D06, (630,480)는 D11의 실제 카드 안이다. A는 세 번 모두 D01/D06을 택하며 실제 위치와 다른 ‘안에 있다’는 설명을 했다.
- **B dense:** 첫 반복 screen이 D03/D07을 자신 있게 게시했다. 그런데 그 screen이 만든 D03 bbox `[530,130,760,290]`에는 (375,315)가 들어가지도 않는다. 하단 bbox의 y 최대 690은 frame 높이 650 밖이다. producer의 자기 보고 `complete/unique`와 실제 기하가 모순된다. typed code는 현재 이 자기 보고와 후보 ID 존재/개수만 검사하고 좌표/bbox 진위를 재계산하지 않아 잘못된 관계를 그대로 채택했다. 이는 ‘분리 구조라 어쩔 수 없는 오류’가 아니라 **이번 생산자 능력 및 검증 구현의 약점**이다.
- **Free-region 정답:** polygon은 P3/P4/P5를 포함한다. A는 P2/P3/P5를 골라 P4를 놓치고 밖의 P2를 넣었다. B screen은 P2/P3/P4/P5/P6/P7 여섯 점을 불완전/비유일/navigation 가능 관계로 남겼다. speech는 count=3을 정확히 남겼다. code 결합을 피한 것은 맞지만 최종 모델 binder가 count보다 screen 관계를 우선한다는 설명으로 여섯 점을 `ready` 채택했다. 최종 모델 결과에는 count 불일치 검사가 적용되지 않는다. 후보 부족 때문에 확장을 못 한 것이 아니라 **이미 있는 후보와 관측 경로/발화 제약을 잘못 결합**했다.

직접 근거: [A dense 응답](../raw/model/89ced193-d082-478e-bc80-44b9cb824db4-response.json), [B dense 화면 응답](../raw/model/c1b80c09-b09b-4510-a2f1-a01e501ec661-response.json), [A graph 응답](../raw/model/ac95f440-9d4f-4c9c-bcb0-b64ddb654ba1-response.json), [B graph 화면](../raw/model/220a11d7-b189-4f36-ae4a-a1ad63b9bd4c-response.json), [B graph 최종 결합](../raw/model/60143bf3-ba28-44ca-b46f-551fe7125401-response.json). [code 결합의 실제 검사](../../../../../prototypes/candidates/four_dp_poc/src/dp04/binding.rs#L8), [모델 결과의 현재 검사 범위](../../../../../prototypes/candidates/four_dp_poc/src/dp04/binding.rs#L88).

### 채택 shape와 채점 공정성

A/B는 같은 expression_id와 revision별 최초 ready target 집합 전체 일치를 평가했다. 부분 일치 점수나 나중 정정으로 최초 오답을 없애지 않았다. 공통 gate는 ready가 아닌 상태를 빈 대상으로 처리한다. B는 실제 게시 후보 밖의 ID를 직접 만들 수 없고, A는 원본에서 extent를 직접 만든다. 이는 비교할 구조 차이이고 B에 정답 후보를 주입한 것이 아니다.

초기 문자열 채점 결함은 V5에서 보존·수정했다. `A2 / Conversion chart`와 `A2`, 같은 실행에서 게시한 후보 alias와 실제 visible header/frame이 정확히 일치하는 경우만 동일 대상으로 해석한다. 정답 집합에 끌어맞추거나 fuzzy 매칭하지 않는다. 이 정규화 후에도 dense/free 오류는 그대로다. [공통 ready 정규화](../../../../../prototypes/candidates/four_dp_poc/src/dp04/gate.rs#L27), [평가용 source-backed 동일성 정규화](../../../../../benchmark/architecture/four_dp_poc/score_dp04.py#L40), [14개 회귀 검사](../../../../../benchmark/architecture/four_dp_poc/check_dp04_identity.py).

## 4. B가 유리할 수 있는 QA와 이번 확인 범위

| QA | B의 조건부 이익 | 이번 데이터 |
| --- | --- | --- |
| QA01 | 빠른 충분한 screen/native 생산 + 안전한 code 결합, 또는 B4로 독립 생산 대기 겹침 | B4가 자체 dense 직렬보다 짧음. A보다 빠르지는 않음. 사전 생산을 숨긴 이익 없음 |
| QA02 | 검증된 전용 parser/영역 표현이 시각 혼동을 줄이고, type/count/관계 검사를 실제 최종 결과에 적용 | 기본 정확도 동점. 일반 Qwen screen parser의 기하 오류와 binder 제약 위반 관측. 전용 parser 효과 미검증 |
| QA03 | 같은 판단 안의 많은 결합이 충분한 후보를 재사용하여 A의 실제 반복 해석을 줄일 때 | A도 한 호출로 복수 지칭 처리. 이번에는 이익 없음. QA01과 거의 같은 endpoint여서 별도 두 이익으로 합산하면 안 됨 |
| QA04 | source 변화가 producer 내부 표현에만 머물고 consumer 계약을 건드리지 않을 때 | C01/C03 공통 어댑터 실변경 평균 1.5, 동점. producer 소유권의 수정성 우위 미입증 |
| QA05 | 더 싸고 정확한 생산, 실제 native/code 경로, 같은 판단 내 반복 비용 상쇄 | code 결합을 썼어도 총 2.952배. cross-revision B1 cache를 기본 이익으로 소급할 수 없음 |
| QA06 | 큰 media의 stage별 staging을 빨리 회수하면서 raw 보존 계약도 충족할 때 | A 38.928MB / B 39.633MB scoped RSS. B 감소 이익 없음. cloud helper 가중치가 local RAM인 척하거나 raw를 버려 이익을 만들지 않음 |

따라서 B는 이름만 바꾼 A가 아니다. 실제 producer ownership, 중간 후보, code/model 분기, 원 producer expansion, read-set과 최종 writer 제한을 구현했다. 그러나 **강한 B의 시각 capability를 입증한 구현이라고 말할 수도 없다.** 문서의 별도 detector/OCR/caption parser는 설치하지 않았고 Qwen vision을 같은 범용 모델로 호출했다. source 확장과 대규모 within-judgment 재사용의 이득을 보여 주는 결과도 없다. 현 증거로 양쪽 모두 기능 정확도가 충분하지 않으며 B의 예상 QA02 반대 이익은 남은 가설이다.

## 5. 모든 보완 목록과 실제 실험

문서 기준: `04-screen-speech-grounding.md` §11. 번호는 누적이 아니다. 아래 DP04 A1/A2는 다른 DP의 같은 번호와 다른 보완이다.

| 보완 | 목적 | 이번 실행 |
| --- | --- | --- |
| A1 | source/extent/revision/권한 key crop/cache로 QA03/05 절약; 문맥 누락·RAM·무효화 비용 | 없음 |
| A2 | 의미 영향 범위에 따른 부분 binding patch; 유지 오류/영향 판단 비용 | 없음 |
| A3 | 발화 중 잠정 revision 증분 판단; 낭비/폐기/재판단 비용 | 없음 |
| A4 | B와 같은 화면 parser 정보를 A joint의 보조 입력으로 공급; 시각 표현 이익 귀속 확인 | dense 3회 TACTIC |
| B1 | 새 완성 입력 사이의 유효 target 후보 재사용; cache/무효화 비용 | 없음 |
| B2 | 정정 영향을 받은 producer 후보/binding만 재생산; 의존 관리 비용 | 없음 |
| B3 | 발화 중 잠정 revision 증분 생산/결합; 낭비/폐기/재판단 비용 | 없음 |
| B4 | 초기 screen/speech 생산을 같은 K=2에서 병행; 경합/staging 비용 | dense 3회 DIAGNOSTIC |

| dense 전후 | 자체 BASIC 3회 | 보완 3회 |
| --- | ---: | ---: |
| A→A4 QA01 | 6,854.31ms, 응답 3 | 35,066.85ms, 응답 1 / 미응답 2 |
| A→A4 QA02 | 0/6 | 0/6 |
| A→A4 비용 | $0.0032388 | $0.0173976 |
| B→B4 QA01 | 27,812.93ms | 22,965.19ms (약 17.43% 감소) |
| B→B4 QA02 | 0/6 | 2/6 |
| B→B4 비용 | $0.0096588 | $0.0096588 |

### A4 잘림을 구조 문제로 일반화하지 않는다

A와 A4의 joint 함수/최종 출력 계약/1,700 token 한도는 같다. A4는 screen 3,500 token 호출을 추가하고 그 결과를 joint 입력에 공급한다. B의 screen/speech/binder는 각각 3,500/1,800/1,800이다. **모든 역할에 같은 출력 token 수를 줬던 실험은 아니다.** 공통 모델/전체 $5/K=2와 역할별 응답 길이 한도를 구별해야 한다.

A BASIC 15회는 유효 JSON으로 끝났다. A4의 joint는 r1/r3에서 `finish_reason=length`, output 1,700으로 잘려 JSONDecodeError가 났고 r2만 698 token의 유효 JSON이었다. r1 실제 content에는 잘못된 parser bbox/관계를 두고 길게 재검토하는 문자열이 있다. `enable_thinking=false`여도 일반 content의 evidence 문자열이 길어질 수 있다. 이 사실은 출력 계약/한도와 모델 응답의 취약성을 보이며 A4 구조의 불가피한 결함을 증명하지 않는다. ‘A4가 QA02 개선 없음’은 **이 동결 조건에서 운영상 0/6**이라는 뜻이고, 표현 보조의 순수 정확도 효과를 잘림 없이 충분히 검증했다는 뜻이 아니다. 완료한 한 번도 D01/D08으로 0/2였다.

같은 1,700을 썼다는 이유만으로 완전 공정 또는 불공정 어느 쪽도 자동 결론내릴 수 없다. 후속 실험에서는 동일 final schema를 간결하게 제한하고 필요한 출력 용량을 사전 calibration한 뒤 A/A4 양쪽에 같은 final 한도를 적용해야 한다. 추가 역할의 모든 비용은 그대로 세야 한다. 현재 실패를 수정·삭제하거나 기존 값을 새 한도 결과로 덮어쓰면 안 된다.

실패 근거: [A4 r1 provider 응답](../tactics/dp04-A4/raw/model/bff53cf0-306f-40f4-a7f6-8313aca7be20-response.json), [joint 공통 함수/한도](../../../../../prototypes/candidates/four_dp_poc/src/dp04/joint.rs#L15). 정확한 r3 및 모든 호출 경로는 JSON에 함께 보존했다.

### B4 시간 효과와 정확도 변동은 다르다

실제 두 HTTP 요청의 겹침은 5,001.96 / 5,466.84 / 5,341.47ms다. dense B의 screen 평균 22,863.58ms + speech 4,940.49ms 직렬이 B4에서는 screen 22,955.78ms와 speech 5,276.48ms의 겹침으로 바뀌었다. 이 경로와 시간은 약 4,847.73ms 감소에 부합한다. 같은 K=2이며 producer 후 결합 의존은 없애지 않았다.

그러나 **B BASIC3/A4 3/B4 3의 dense screen API request body 9개가 모두 같은 canonical SHA256 `92b1cc3a1b30b4e4857375620aa709373d0b88ce366710301f7b3f83884ddad9`**인데 cue target 응답이 달랐다. temperature=0은 실제 반복의 동일 출력을 보장하지 않았다. B4 r1/r3의 D06/D08 중 D06 두 건이 맞아 2/6이 되었고, B 기본은 D03/D07 등으로 모두 틀렸다. 병행이 의미 판단 정보를 개선한 것이 아니다. 정확도 개선을 병행의 인과 효과로 주장할 수 없다. 요청/응답 해시 및 경로는 파생 JSON에 있다.

## 6. 심사위원에게 설명하기 위한 최소 조치

**지금 설명 가능한 범위:** 기존 결과를 그대로 유지하면서 위 단계 분해, 실제 오류, Qwen parser라는 구현 범위, B4의 정책 효과와 응답 변동, A4 잘림을 발표에 명시한다. 그러면 ‘동일 QA02에서 이번 작업 기준 A를 택했다’는 국소 선택은 설명할 수 있다. ‘B의 구조적 QA02 이익을 실증했다’, ‘모든 강한 B보다 A가 낫다’, ‘A4 자체가 해롭다’는 주장은 할 수 없다.

**더 강한 trade-off를 주장하려면 필요한 별도 freeze의 최소 후속 확인(이번 감사에서 실행하지 않음):**

1. **출력 계약 용량:** evidence/reasoning 필드 길이를 제한한 공통 final schema와 충분한 final token 한도를 결과 전 calibration한다. A/A4에 같은 final 조건을 주고 dense/free의 성공·미응답·전체 비용을 함께 반복한다. 잘림과 의미 실패를 분리한다.
2. **관측→기하 capability:** 같은 PNG에서 visible label/bbox, pointer→candidate, polygon→sample을 분리 점검한다. frame bounds, point-in-bbox, 명시 count와 최종 target 수 같은 이미 존재하는 계약의 모순은 oracle 없이 검출할 수 있다. 이상하면 정답을 코드로 채우지 말고 공통 재조회/확인/미해결 경로로 보내며 그 실패와 비용을 센다. A에도 적용 가능한 동등한 최종 제약을 허용한다. 원한다면 pointer/path overlay를 양쪽 동일한 추가 입력 조건으로 별도 비교하여 좌표→픽셀 대응 부담을 분리한다.
3. **B의 의도한 visual dependency:** 전용 parser의 효과를 주장하려면 실제 실행 가능한 parser 또는 검증한 화면 생산 경로를 B와 A4에 똑같이 공급한다. 먼저 정답 없이 candidate coverage/좌표 capability를 확인하고, 같은 두 실패 사례를 포함한 고정 모집단에서 A/B/A4를 비교한다. B의 speech/screen 소유권 자체의 효과와 helper 표현의 효과를 분리한다. 해당 parser를 쓰지 않으면 결론을 현재 Qwen 생산자 B로 제한한다.
4. **병행 인과:** B4의 의미 이익은 주장하지 않는다. 순수 scheduling 설명이 더 필요하면 동일 보존 producer 출력/동일 처리시간 trace로 직렬/병행 critical path의 차이를 별도 제어 진단한다. 이것은 실제 새 cloud 정확도 실험이 아니라 재생 진단이라고 표시한다. 추가 실제 실험은 variant 순서를 교차/무작위화하고 변동을 보고한다.

이는 PoC를 제품으로 완성하자는 요구가 아니다. 현재 실패를 지우지 않고, **강한 반대안의 QA 이익 또는 실패 원인의 인과를 추가 주장할 때** 필요한 최소 분리다. 사용자 허가 전에는 새 모델 호출/후보 변경/재실행을 하지 않는다.
