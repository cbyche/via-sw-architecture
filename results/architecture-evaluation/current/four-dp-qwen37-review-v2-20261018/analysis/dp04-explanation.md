# DP04 화면·발화 지칭 결합: 발표용 근거 설명

상태: DP04 BASIC **24/24행 완료**(A/B 각각4사례×3회). 아래 BASIC 수치는 실제 원시24행으로 확정했다. 문서14 §5의 사전 가설, calibration-v1/v2/v3 개발 관측, BASIC 결과를 구분한다. A4 TACTIC와 B4 DIAGNOSTIC도 각4사례×3회가 별도 완료됐고, 아래 BASIC에 합산하지 않는다. 교정 DP05 비교는 별도 예정이다.

최종 BASIC 동결: `basic-v1/freeze.json` SHA256 `50e4116e1fa86807c19abecac6586110e88968087542eececcfc4e379a7cecf2`, binary `8229089b04b66231f58cbb44b35c12950735cfbbad83dfe03993d5d34cd773e9`. DP04 A/B 각각4사례×3반복=12행, 각각 QA01 **15 stimuli**, QA03 **12 goals**, QA02 **24 expression/revision 집합**이 예정 분모다. 실패/미응답을 빼고 이 분모를 줄이지 않는다. 다른 DP와 합친 전체261/243 분모와 혼동하지 않는다.

## 같은 입력과 다른 책임

A는 원본 PNG와 발화·포인터를 한 판단에 함께 넣고 최종 지칭을 직접 채택한다. B는 화면 생산자가 위치·후보를, 발화 생산자가 목적·지칭 제약을 만들고 그 결과를 결합한다. B의 관계가 완전하고 유일하면 코드로 결합하고, 모호하거나 빠진 후보가 있으면 실제 결합 모델/화면 생산자 확장을 쓴다. B의 원본 보존과 모든 선행·실패·복구 호출 비용도 포함한다.

양쪽 모두 같은 원본 PNG와 원래 좌표에서 기계적으로 그린 포인터·경로 overlay를 받는다. overlay에는 사건 ID/좌표만 있고 정답 대상이나 bbox가 없다. 공개된 native object bbox는 이 네 사례에 제공되지 않는다. 후보가 보는 모델 bbox와 실제 화면의 대상 일치는 별개 문제다. 화면 내 유한 bbox, 원문에 명시된 좁은 개수 표현, 포인터/영역과의 자기정합성 검사는 거짓 bbox 자체의 정확성을 증명하지 않는다. 첫 실제 채택 집합의 exact-set QA02가 잘못된 대상을 그대로 실패로 남긴다.

## QA별 직관 → 관측 → 해석

| QA | 실행 전 A/B 직관 | BASIC 실제 관측 | 조건과 해석 |
| --- | --- | --- | --- |
| QA01 첫 응답 | A는 joint 최종 writer까지 단계가 적다. B는 독립 생산 완료를 기다리는 비용이 있다. | A **10/15 관측, 5 미응답**; B **15/15**. 관측값 평균 A11,175.802ms, B18,344.055ms는 관측 집단이 달라 전체 우열 비교에 쓰지 않는다. 양쪽 관측된 같은10입력에서는 A11,175.802/B19,382.408ms다. | 같은 dense/page 사례에서는 A가 짧았다. free A는 모두 미응답이고 정정 최초판본2개도 미응답이므로 평균에0이나 임의 timeout을 넣지 않는다. 이 값은 실제 GUI가 아닌 승인된 DISPLAY_PROXY callback 지연이다. |
| QA02 지칭 집합 | A는 시각·좌표·발화 관계를 한 writer가 함께 판단한다. B는 화면 후보와 발화 제약을 분리해 검사·결합하지만 생산자 오판 가능성은 남는다. | A **17/24=70.833%**, B **24/24=100%**. A의 잘못된7평가단위는 실제 정답아닌 비어 있는 unresolved 채택이다. 성공적으로 비어 있지 않게 채택한17개 집합은 모두 정답이었다. | B의 이번 이점은 지칭집합과 근거 계약을 충족해 실제 채택한 기능 전체에서 관측됐다. 순수 시각대상인식 자체의 구조적 우위라고 확대하지 않는다. 정답제안 뒤 잘못된 bbox로 미채택한 실패를 아래 분리했다. |
| QA03 VIA 처리시간 | A는 중간 생산단계가 적다. B는 후보·발화 생산과 필요한 모델 결합을 부담하며, 유효한 후보 재사용 조건에서는 절약 가능성이 있다. | A **9/12 완료, 3 미완료**, B **12/12**. 관측 평균 A15,623.849/B22,930.461ms는 전체 비교가 아니다. 양쪽 완료한 같은9목표(dense/page/정정)에서는 **A15,623.849/B24,959.682ms**였다. | 정정은 원래 첫 입력부터 최종revision2 완료까지 계산했다. 최초판본의 실패2개도 그 시간에 포함된다. DP04는 grounding·채택·표시가 VIA 완료범위라 callback과 완료가 가까우며, 비교/평균 계산 업무를 임의로 추가하지 않았다. RECEIVER 제외0ms다. |
| QA04 변경 모듈 | 생산·후보·결합 분리의 변경 국소화 이익과 계약 전파 비용 중 어느 쪽이 실제로 생기는지 확인해야 한다. | 독립 A/B scratch의 C01=1, C03=2, C05=2, C06=5. 네 과제 동일가중 평균 **A2.5/B2.5**. 새 기능 pair 평균 **3.5/3.5**도 나란히 보존한다. | C05 실제 전달 지칭 provenance 질의, C06 두 영역 중 일부 철회·픽셀차단·disjoint 보존·옛 proposal 차단은 양쪽 모두 공통 source view/권한/gate와 orchestration 변경이 필요했다. B 국소화 우위는 이 과제에서 미관측이다. |
| QA05 모델 비용 | A는 호출이 적을 가능성, B는 화면·발화 생산 비용이 있지만 코드 결합·유효 재사용이면 상쇄 가능성이 있다. | A **26호출, 66,633 input/9,331 output tokens, $0.0415828**; B **38호출, 50,225/14,392 tokens, $0.0431172**. B/A **1.0369배**. 모든 producer/실패/복구 비용 포함, receiver0, unknown0. | A의 실제11회 reread와 B의 코드결합7회가 비용차를 좁혔다. B의 작은 binder만 계산한 값이 아니다. 병행은 BASIC에 없으며 비용 절약을 자동으로 만들지 않는다. |
| QA06 메모리 | B는 raw 외 후보·발화·결합 상태를 보유한다. 실제 동시 생존 상태와 실행 환경이 RSS를 결정한다. | fresh Python bridge+Rust 동일시점 RSS 최대 **A45.219840MB/B45.350912MB**; 관측 불완전 **0/12,0/12**. B 최대는 정정r2, A 최대는 dense r2다. | 차이0.131072MB는 이 작은 PoC의 표본 추정이다. 클라우드 모델 가중치와 UI 제외, 공유페이지 중복 미제거. 논리 buffer 크기를 RSS나 제품 heap으로 읽지 않는다. |

## BASIC 분모와 사례별 세 번의 실제 결과

각 안은12 episode이지만 한 episode에 지칭식 또는 revision이 여럿 들어 있다. QA01은 dense3+free3+page3+정정6=**15개 입력**의 첫 substantive callback이다. QA03은 dense3+free3+page3+정정3=**12개 목표**의 최종 필수revision 완료다. QA02는 dense 두 지칭×3=6, free 한 집합×3=3, page 한 지칭×3=3, 정정 두 지칭×두revision×3=12로 **24개 exact-set 평가단위**다. 개별 target3개의 recall이나 cardinality 점수가 아니다.

| 사례/안 | QA01 관측/예정·평균ms | QA03 완료/예정·평균ms | QA02 정답/예정 | 호출·input/output tokens | 모든 VIA 비용USD |
| --- | --- | --- | --- | --- | ---: |
| dense A | 3/3 · 17,788.669 | 3/3 · 17,788.926 | 6/6 | 6 · 16,251/2,820 | 0.0110124 |
| free-region A | 0/3 · — | 0/3 · — | 0/3 | 6 · 15,467/1,915 | 0.0092508 |
| page-change A | 3/3 · 10,011.640 | 3/3 · 10,011.850 | 3/3 | 6 · 20,540/1,453 | 0.0105408 |
| partial-correction A | 4/6 · 7,089.272 | 3/3 · 19,070.772 | 8/12 | 8 · 14,375/3,143 | 0.0107788 |
| dense B | 3/3 · 27,580.467 | 3/3 · 27,580.712 | 6/6 | 6 · 8,571/4,650 | 0.0108684 |
| free-region B | 3/3 · 16,842.570 | 3/3 · 16,842.797 | 3/3 | 6 · 8,430/2,731 | 0.0077416 |
| page-change B | 3/3 · 16,475.281 | 3/3 · 16,475.479 | 3/3 | 8 · 13,721/2,489 | 0.0094708 |
| partial-correction B | 6/6 · 15,410.978 | 3/3 · 30,822.855 | 12/12 | 18 · 19,503/4,522 | 0.0150364 |

위 평균은 해당 칸의 관측 endpoint만의 평균이다. partial-correction A의 QA01은 세 episode 평균의 단순평균이 아니라 관측된4입력의 평균이다. 세 번의 개별 값·범위·누락은 `dp04-basic-review.json`의 cases/rows와 frozen phase `individual-trials.md`에서 읽을 수 있다. 통계적 유의성이나 제품 모집단 일반화를 주장하지 않는다.

같은 dense3회는 A/B17,788.669/27,580.467ms, page3회는10,011.640/16,475.281ms에 첫 응답했다. 양쪽 최종 완료한 정정3회는19,070.772/30,822.855ms였다. **A의 관측된 경로가 짧았다는 사실과 B가 모든 예정 입력·목표를 응답/완료했다는 사실을 함께 보고한다.**

## 원시 실패·복구 분류: 제안한 정답과 실제 채택을 구분

A의 첫판본 실패는5개 revision이고, QA02로는7개 지칭집합이다. free3개는 끝까지 미완료다. 정정r2/r3의 첫판본은 각각두 지칭을 미해결로 남겼지만 이후 입력revision2의 C1/C3는 정상 채택·완료했다. 그래서 QA01은5개 미응답이고 QA03은3개 미완료다. 초기 실패를 숨기거나 뒤의 정답으로 대체하지 않는다.

| 원시 run | 실제 실패/복구 경로 | 분류·고정 지표 |
| --- | --- | --- |
| dense A r1/r2/r3 | 최초 D06/D11 ID는 정답. D06 bbox[275,340,475,480]는 pointer y315를 포함하지 않으며 D11 bottom660은650px 화면 밖. 한 실제 reread에서 [275,280,475,420]/[525,460,725,600]로 고쳐 채택. | **(b) 정답제안+기하모순 뒤 복구 성공**. QA02 각각2/2. 초기 오류·추가호출 비용 보존. |
| page A r1/r2/r3 | 최초 A2는 맞지만 bbox[60,380,880,590]가 이전 page_a pointer(470,320)를 포함하지 않음. 실제 reread가 [60,240,880,410]로 고쳐 채택. | **(b) 뒤 복구 성공**. 이전 frame 지칭이 later navigation으로 교체되지 않았고 각각1/1. source/revision canonization 오류가 아니다. |
| free A r1 `4a3fae`, r2 `1edfa6` | 최초 P3/P4/P5는 정답. P5 bbox[380,450,420,490] center(400,470)가 raw loop 밖. reread는 bbox를 고치지 않고 P5를 빼면서 count3 유지·추가근거 요청. | **(b)→(a)+(c)**: 실제 마지막 제안 P3/P4는 불완전하고 원문/모델 count3와 모순. 최종 미채택. QA02 각각0/1. |
| free A r3 `449511` | 최초는 expression_id 대신expression, targets 객체배열을 반환. 실제 reread는 P3/P4/P5를 제안하지만 P3 bbox[280,215,310,245] center(295,230)가 raw loop 밖. | **(c)→(b)**. JSON은 parse되었고 HTTP/provider 오류가 아니다. QA020/1. |
| correction A r2 `fd1aa8` 최초판본 | C1/C2 정답제안이나 bbox bottom820>650. reread도 같은 잘못된 범위를 유지. 최신정정은 C1/C3·유효범위로 채택. | **(b) 초기판본실패, 최신판본완료**. QA022/4, QA01 1/2, QA03 1/1. |
| correction A r3 `8cc650` 최초판본 | C1/C2 정답제안이나 bbox bottom795>650. reread도 유지. 최신정정 C1/C3는 채택. | **(b) 초기판본실패, 최신판본완료**. QA022/4, QA01 1/2, QA03 1/1. |

분류(a)대상집합 오판, (b)정답집합 제안이나 bbox 기하모순, (c)count/type/필수metadata 모순, (d)provider/JSON 실패는 **한 실행의 여러 시도에서 겹칠 수 있다**. 5개 실패revision의 마지막 계약 원인은 (b)3개/(c)2개로 요약할 수 있지만, (c)2개는 빠진 대상(a)과 동시 발생했다. A의 geometry 거절13사건과 schema/count 거절3사건은16개 시도사건이며 실패episode 수와 더하지 않는다. (d)는 A/B 모두0;64개 실제호출 전부HTTP200/finish_reason=stop이었다. cap에 도달해 잘린 출력이 없다.

실패 입력에서 GOAL_FAILED까지의 실제 경과시간(ms)은 free r1/r2/r3 **13,651.190 /13,586.563 /10,040.297**, 정정 최초판본 r2/r3 **14,254.866 /14,598.475**다. 이는 실패 종료시간이며 성공 endpoint의 대체값이 아니다. 정정의 뒤 성공 QA03은 원래 최초 capture부터21,326.173/21,711.693ms로 계산해 앞선 실패시간을 포함한다.

실제 graph PNG/SVG의 점 중심은 P3(350,230), P4(500,320), P5(410,380)이다. 이 권위 geometry는 **평가 설명에서만** 확인하며 후보에 넘기지 않았다. B도 P5를(395,395) 또는(395,405) 등으로 근사했다. B가 정답을 실제 채택한 것은 공통 자기정합성 기준과 exact-set을 통과한 결과이며, bbox 자체가 픽셀에 정확하다는 증명은 아니다. 모델의 좌표 정규화 문제/픽셀 읽기 문제/역할별 prompt 효과는 이 관측만으로 분해하지 못한다. 실제 input·model request·provider response 경로와 rejected proposal 전체는 JSON에 연결했다.

### 부분 정정과 오래된 제안의 검사 범위

A/B 각정정3회에서 **이미 끝난 이전revision 제안(또는 실패 후 unresolved proposal)을 저장했다가 재주입**했다. `REPLAYED_CONTROL_EVENT`는6/6 `accepted:false`, `STALE_INPUT_REVISION`이었다. 네트워크의 실제 지연 응답을 발생시킨 실험이 아니며, 비동기 provider 취소·경쟁 정확도나 latency 이득을 측정했다고 쓰지 않는다. 입력revision2와 source document revision1은 다른 축이다. source가 같은 채 input이 정정돼도 old inputrevision을 거절한다.

## 실제 호출·생산·결합·재사용이 시간과 비용에 기여한 경로

| 안·stage | 호출 | input tokens | output tokens | USD | provider send→terminal 평균ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| A · dp04.joint_grounding | 15 | 34,886 | 5,344 | 0.0225048 | 6,728.162 |
| A · dp04.joint_source_reread | 11 | 31,747 | 3,987 | 0.0190780 | 6,969.859 |
| B · dp04.screen_producer | 15 | 29,829 | 10,383 | 0.0285444 | 12,095.121 |
| B · dp04.speech_producer | 15 | 10,646 | 2,977 | 0.0090216 | 4,441.871 |
| B · dp04.candidate_binding | 8 | 9,750 | 1,032 | 0.0055512 | 3,336.406 |

A는15개 입력revision에 joint15호출+실제 원본 reread11호출이었다. dense/page는 각3/3 복구했고 free3/3 및 정정 최초판본2개는 복구 후에도 실패했다. B는 revision마다 화면15+발화15호출을 **직렬**로 수행했고, 완전·유일한 code binding은 dense3/free3/page1=7회였다. page 나머지2와 정정6revision은 실제 binder8호출을 썼다. B의 source expansion/producer output repair는 이번 BASIC에서0회였다. 최대 실제 모델 output은 screen1327/한도3000, speech242/1400, binder152/2000, A joint/reread470/2600으로 모두 충분히 이하였다. 역할별 필요한 schema를 다르게 제한했으므로 '동일 output token budget 실험'이라고 하지는 않는다.

A/B 각각 prior binding 상태는 정정revision2의 입력으로 전달됐지만, **B가 화면 생산을 재사용해 호출을 생략하지는 않았다**. B는 정정에서도 화면/발화를 다시 생산하고 모델로 상대위치·교체를 결합했다. 두 안 모두 raw frame을 source로18회 읽었다. source 읽기와 모델 reread 호출은 같은 집계단위가 아니다. B1 cache는 BASIC에 없다.

B는 전체 output이 A보다54.24% 많았지만 input은24.62% 적었다. A의 실제11번 repair 입력과 B의7번 코드 결합이 합쳐져 B/A비용은1.0369배로 좁아졌다. 이전 세대의 약3배 비용 또는4배 시간이라는 관측을 이 수정 세대에 옮기지 않는다. 공통 overlay·좌표/type/count 계약·역할별 compact schema/cap·owner recovery 및 실제 code path가 함께 바뀌었으므로 하나의 변경만 비용비율 변화의 원인으로 확정하지 않는다. 시각정보가 원본과 같은 파생생산이라는 점도 유지한다.

위 provider 평균은 실제 서비스 span의 기술값이다. QA01/03, queue를 제외한 '순수 구조 시간', 또는 병행 critical path가 아니다. BASIC queue 최대는 A7.055ms/B5.614ms였고 측정시간에 포함됐다. B의 긴 screen 출력/서비스와 직렬 speech/binder 의존은 실제 지연을 설명하는 자료지만, B4 없이 직렬정책이 시간차의 몇%인지 확정하지 않는다.

## 실제 동시 RSS와 논리 상태

| 안 | 최대 same-sample 합계MB | 그 동일표본 Python/Rust MB | 최초 동시표본 범위MB | 표본수·최대간격ms |
| --- | ---: | --- | --- | --- |
| A | 45.219840 | 42.041344 /3.178496 | 34.439168–34.701312 | 2661 ·311.924 |
| B | 45.350912 | 42.074112 /3.276800 | 34.373632–34.668544 | 4124 ·93.203 |

각 run은 fresh Python+Rust이며 BEFORE_RUST_START의 bridge-only 측정과 두 프로세스가 처음 같이 관측된 값을 구분한다. 실제50ms 명목 polling에는 위 간격이 있어 순간 peak를 놓칠 수 있다. 양쪽12/12 모두 동시 관측은 있었으나 연속 commit memory 측정은 아니다. RSS 합은 한 ps 호출의 같은 시점 역할별 합계이고, 각 역할의 독립 최대를 더한 값이 아니다.

논리 category의 전체run별 최대 serialized bytes는 retained evidence A25,758/B70,626, Rust serialized A8,236/B12,711, image양쪽161,824였다. **서로 다른 시점의 category 최대값이므로 더하지 않는다.** B의 후보·발화·binder trace가 남는 구조는 실제 논리상태에 보이지만, retained evidence에는 Python 측정 harness의 call/event 기록도 포함된다. 이를 제품 retained heap이라고 부르지 않는다. 실제 process RSS의 대부분은 bridge였으며, 이 작은 차이로 제품 메모리 우월성을 입증하지 않는다.

## 과거 예상과 달랐던 결과 및 다음 분리 실험

A가 원본을 한 번에 보면 단계가 짧다는 예상은 dense/page와 최종정정의 실제 완료 경로에서 관측됐다. 하지만 한 writer가 만든 대상이 맞아도 좌표가 모순이면 공통 기준을 통과하지 못했다. B는 이번 Qwen producer 출력이 코드/모델 결합에 충분했고 모든 지칭을 채택했으나 생산자 분리가 본질적으로 시각 정확도를 보장한다는 뜻은 아니다. 최초 모듈변경 국소화 가설은 QA04의 실제 네 과제에서 동점이었다. B의 추가 생산 비용은 A의 실제 repair 비용 때문에 이전 세대에서 예상했던 큰 차이가 재현되지 않았다.

A4는 같은 화면 생산자 도움을 A joint writer에 더해 **helper 효과와 최종 결합 소유권 효과를 분리**한다. B4는 초기 독립 screen/speech만 같은 slot한도에서 병행해 **BASIC 직렬정책의 민감도**를 본다. 각4사례×3회는 아래 별도 phase 결과로 완료됐다. 각자의 BASIC 같은4사례×3회와 성공/누락·복구·stage별tokens/비용·동시RSS를 별도 비교한다. 정확도 변화는 병행 시간단축과 별도 현상이다.

독립 재구성 `review_dp04_basic.py`는 source-visible identity canonicalizer를 재사용하고, 첫 adoption의 exact-set·capture→display·최초capture→필수최종revision complete·동시peak·provider상태를 직접 대조했다. **24행×5=120검사 PASS**이며, 추가로12쌍 A/B의 variant 외 실제입력 동일·64호출의 고정모델/생각OFF/temperature0 및 공개한 stage cap·64개 finish_reason=stop도 대조했다. 초기 검토 스크립트의 provider 상태 상수 오해는 실제 `COMPLETED`/정가환산 cost_state에 맞춰 고쳤으며 원시/공식값은 바꾸지 않았다. API·후보실행·컴파일0. `dp04-basic-review.json`에는 run별raw/input/model request/provider response 경로·원시trace SHA·고정oracle/manifest SHA와 모든 대조값이 있다. 전체120의261/243 및 ledger 감사는 별도 종료 후 진행한다.

## 개발 calibration-v1 관측: formal과 합산하지 않음

| 사례/안 | 호출 경로 | input tokens | output tokens | 비용 USD | 최초표시/완료 ms | 최초 exact-set |
| --- | --- | ---: | ---: | ---: | --- | --- |
| dense A | joint → 실제 geometry 거절 → owner reread | 4743 | 924 | 0.0033756 | 16999.474 / 16999.705 | 2/2 |
| dense B | screen → speech → code binding | 2397 | 1565 | 0.0034628 | 26804.483 / 26804.788 | 2/2 |
| free A | joint | 2148 | 385 | 0.0014752 | 6990.847 / 6990.997 | 1/1 |
| free B | screen → speech → code binding | 2350 | 989 | 0.0025224 | 18002.725 / 18002.951 | 1/1 |

위 호출·token·비용 합계는 실제 원시 stage 추출과 대조했다. 재현 추출기는 `analysis/dp04-call-evidence.py`; 원시 경로/UUID와 합계는 `dp04-calibration-v1-stage-evidence.json`에 보존한다.

Dense A의 첫 답은 대상 ID는 맞았지만 포인터가 bbox 밖에 있거나 bbox가 frame을 넘었다. 실제 common guard가 거절한 뒤 두 번째 모델 호출로 복구했다. 반면 free A는 raw loop인데 relation=point_inside를 반환해 당시 기하검사 분기를 우회했다. B도 검증한 count/cue/relation이 게시된 adoption에서 소실됐다. 이 두 protocol 결함을 calibration-v2에서 공통 type/path guard와 실제 metadata 보존으로 고쳤다. 따라서 v1의 정답 집합 통과를 v2 guard 검증 성과라고 소급하지 않는다. 모든 출력은 finish_reason=stop이었고 당시 JSON-length 실패는 없었다.

모델이 제공한 bbox는 실제 카드/점 범위와 여전히 어긋날 수 있었다. 원래 source card D06의 실제 bounds는 [277,255,502,395], D11은 [519,420,744,560]이고 raw pointer는 [375,315], [630,480]다. 모델이 둘을 포함하는 그럴듯한 다른 bbox를 만들면 자기정합성만으로는 실제 bbox 오판을 잡을 수 없다. 해당 한계를 성공 집합의 내용정확도 증명으로 바꾸지 않는다.

## 개발 calibration-v2: 역방향 결과 보존

`calibration_v2-dp04-dense-A-r1-cae917`는 두 실제 모델 호출 후 unresolved로 종료했다. 새 계약 오류나 JSON 잘림이 아니라 모델이 첫 bbox 모순을 고치지 못한 관측이다. 두 응답 모두 “Pointer-1 (375,315) falls inside D06 card bbox [275,380,475,580]”라고 했지만 **315 < 380**이다. 첫 D11 bbox의 bottom830은650px frame을 넘었고, reread는 D11만 고쳤다. 두 호출 모두 HTTP200/finish_reason=stop, 출력472tokens씩(cap2600). 전송된 원본/overlay 모두1000×650이고 실제 source hash가 일치했다.

공통 guard는 두 번째 거절 뒤 명시적 GOAL_FAILED를 남겼다. 프로그램 returncode는0이지만 기능 성공은 아니다. 정합성 계약은 revision 전체 proposal을 채택하므로 D11 수정만 부분 성공으로 바꾸지 않고 두 expression 모두 unresolved로 남는다. frozen evaluator 확인값은 QA02 **0/2**, QA01 미응답 **1/예정1**, QA03 미완료 **1/예정1**, 비교상태 INCOMPLETE다. 4882input+944output tokens와 **$0.0034632**는 전액 보존한다. 앞선 v1 A성공을 지우거나 이 실패로 대체하지 않는다. 근거: `analysis/dp04-calibration-v2-dense-A-failure.json` 및 그 안의 실제 UUID/응답 경로는 calibration-v2 raw에서 연결한다. formal 분모와 합산하지 않는다.

Free A `a459cb`는 실패 성격이 다르다. 두 응답이 정확한 P3/P4/P5, count3를 제안하고 loop polygon 설명도 했지만 관계를 point_inside라고 두 번 썼다. raw cue는 pointer_path라 frozen guard가 CUE_RELATION_TYPE_CONTRADICTION으로 거절했다. 이는 **정확한 제안집합을 관계 schema 불일치로 미채택한 실패**이며 시각 대상 오판이라고 쓰지 않는다. prompt가 enum과 기하조건을 나열했으나 enum↔event type의 정확한 매핑을 명시표로 설명하지 않았으므로 명칭의 일상적 해석 모호함이 기여했을 가능성을 배제하지 못한다. root가 마지막 공통 정의 명시를 승인했고, v2 raw/실패는 그대로 보존한 채 아래 calibration-v3를 별도 시행했다. 근거: `dp04-calibration-v2-free-A-failure.json`.

| v2 사례/안 | 실제 경로 | input / output tokens | 비용 USD | QA01 / QA03 ms | QA02 | 동시 표본 RSS MB |
| --- | --- | --- | ---: | --- | --- | ---: |
| dense A | joint → 거절 → reread → 거절/실패 | 4882 / 944 | 0.0034632 | 미응답1/1 / 미완료1/1 | 0/2 | 44.745 |
| dense B | screen → speech → code binding | 2421 / 1550 | 0.0034484 | 27165.226 / 27165.567 | 2/2 | 44.171 |
| free A | joint → type 거절 → reread → 같은 type 거절 | 4727 / 748 | 0.0030876 | 미응답1/1 / 미완료1/1 | 0/1 | 43.713 |
| free B | screen → speech → code binding | 2374 / 992 | 0.0025368 | 17959.305 / 17959.621 | 1/1 | 43.139 |

4행 모두 Python/Rust 동시관측은 있었지만 작은 표본의 RSS 차이로 구조의 제품 메모리 우열을 주장하지 않는다. B의 추가 생산단계도 이번 A의 실제 reread보다 비싸지 않을 수 있음이 관측됐다. 이는 **사용 경로에 따른 조건부 비용**이며 A 호출이 늘었기 때문이지 B의 생산이 무료이기 때문이 아니다. B는 이번 두 사례에서 최종 binder 모델을 부르지 않고 완전하고 유일한 후보를 실제 코드로 결합했다. B의 count/cue/relation 보존 정정은 실제 게시된 adoption에서 확인했다. 일반 정확도나 전용 화면 parser 능력으로 확대하지 않는다.

## 최종 계약 명시 후 calibration-v3: naming과 bbox 오류 분리

공통 relation/event 정의를 모든 역할 instruction에 정확히 한 번 넣고, 복구에는 실제 오류와 수정 지침만 보냈다. 검사·정답·cap·최대 복구1회는 바꾸지 않았다. A의 새 응답은 within_region를 정확히 사용했다. 그러나 P5를 bbox `[380,450,420,490]`, center `(400,470)`으로 읽어 raw loop의 아래 경계 약410~420 밖이라고 만들었다. 실제 저작 source의 P5 center는 `(410,380)`이므로 이는 모델이 생성한 위치 추정의 오류다. 첫 제안은 P3/P4/P5라도 잘못된 extent 때문에 기하검사에서 거절됐고, 실제 reread는 그 좌표를 고치지 않고 P5를 제외해 targets2/count3를 반환했다. 최종 실패 원인은 원문 명시 개수와 완전한 집합을 충족하지 못한 것이다. 두 출력은 stop393/351tokens로 cap2600보다 작아 잘림이 아니다.

| 마지막 개발관측 | 경로 | QA02 | QA01 / QA03 | 비용 USD | 같은 표본 RSS peak MB |
| --- | --- | --- | --- | ---: | ---: |
| free A `25c0b8` | joint → bbox 거절 → reread → count/추가근거요청 거절 | 0/1 | 미응답1/1 / 미완료1/1 | 0.0032836 | 43.352 |
| free B `f0906a` | screen → speech → 실제 code binding | 1/1 | 18134.287 / 18134.432ms | 0.0025896 | 43.237 |

이는 마지막 계약 명시가 관계 schema 혼동을 해소했으나 A의 실제 좌표 읽기 오류까지 없앤 것은 아니라는 관측이다. 같은 모델과 관측에서도 역할별 prompt/생성 출력과 필요한 판단이 다르다. 이 소표본만으로 “공동 판단은 원래 정확하지 않다” 또는 “생산자 분리가 항상 정확하다”는 인과를 확정하지 않는다. 모델이 좌표를 잘못 정규화했는지, 그림의 위치를 잘못 읽었는지 등 내부 원인은 원시 수치만으로 분리할 수 없다. 추가 정확도 튜닝 없이 최종 BASIC에서 같은 frozen 조건을 반복했으며 그 결과는 위 표에 별도 집계했다. 근거: `dp04-calibration-v3-review.json`, `dp04-calibration-v3-stage-evidence.json`.

## 최종 코드에서 확인할 책임과 실제 근거

| 책임 | frozen 운영 코드 | 결과를 해석할 때의 주의점 |
| --- | --- | --- |
| 공통 좌표·개수·type·bbox 검사 | `src/dp04/geometry.rs:4` 공통 정의, `:62` final_issues | 모델 bbox와 raw 사건의 자기정합성 검사다. evaluator의 정답 bbox를 후보에 주는 검사가 아니다. |
| A joint와 실제 한 번 reread | `src/dp04/joint.rs:7` ground_impl | reject·retry·실패 비용 전부 포함. 정답 ID를 제안해도 미검증이면 실제 채택으로 세지 않는다. |
| B 화면/발화 독립 생산 | `src/dp04/producers.rs:6` screen, `:38` speech | 전체 producer 비용과 반복 생성 포함. 전용 외부 parser가 아니라 고정 Qwen vision 역할이다. |
| B 완전·유일한 후보의 code binding과 부족 시 model/owner expansion | `src/dp04/binding.rs:8`, `:71`, `:121` | 모든 사례가 code path를 타는 것은 아니다. 실제 호출 stage와 TYPED_CODE_BIND/PRODUCER_EXPANSION_REQUEST를 대조한다. |
| 실제 채택 표시와 grounding 완료 | `src/dp04.rs:77`, `:93`, `:95` | 최초 handling callback과 최종 revision 완료를 분리하지만, 이 PoC의 grounding 업무범위에서는 같은 논리 시점일 수 있다. |
| 원본 읽기와 format adapter | `src/dp04/observations.rs:12` | 실제 original/overlay PNG를 읽으며 현재 source/frame metadata를 normalize한다. |
| 오래된 source/input revision 차단 | `src/dp04/gate.rs:19` | replay는 표시된 실제 이전 모델 제안을 제어 사건으로 재전송하는 검사다. provider latency를 제조하지 않는다. |

위 경로는 모두 `basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2/` 아래다. A와 B는 역할별 output schema가 다르고 공통 raw geometry/count 기준을 최종 채택 전에 만족해야 한다. 같은 원본·동일 모델이라는 사실이 두 내부 출력 형식이나 호출 수까지 같다는 뜻은 아니다. 같은 원본에서 screen helper가 준 효과를 분리하려면 A4, B의 직렬 생산 정책 영향을 분리하려면 B4가 별도 비교로 필요하다. 이 비교들은 BASIC 결과를 본 뒤 승인 범위 내에서만 수행하며 이미 얻은 BASIC 값을 바꾸지 않는다.

## A4/B4 별도 표: 사전에 고정한 목적

| 구분 | 변경 | 식별하려는 효과 | 아직 주장할 수 없는 것 |
| --- | --- | --- | --- |
| A4 TACTIC | B와 같은 screen producer 결과를 A joint writer의 보조입력으로 제공. 최종 writer는 여전히 A. | 화면 생산자의 도움을 A도 받았을 때 정확도/비용/복구가 어떻게 바뀌는가. | A4 이익이 생겨도 B의 producer-owned binding 구조가 우월하다는 결론은 아니다. |
| B4 DIAGNOSTIC | 동일 slot 한도에서 초기 screen/speech만 병행. 후보·결합 의미와 cap은 동일. | B BASIC 직렬정책이 지연에 기여한 정도. | 병행의 시간 변화와 모델 비결정적 정확도 변화를 같은 인과로 해석하지 않는다. 비용 절약도 자동으로 생기지 않는다. |

각4사례×3회의 실제 결과는 아래 표와 분리 JSON에 있다. 실행은 runtime 공유 원장만 사용했다.


## 완료한 별도 A4/B4: 같은 사례와 분모로 비교

A4는 `tactic-dp04-A4`의 **TACTIC12**, B4는 `tactic-dp04-B4`의 **DIAGNOSTIC12**다. BASIC12씩을 다시 실행하거나 덮어쓰지 않았다. 세 구성 모두 QA0115/QA0312/QA0224의 같은 예정 단위를 관측했다. 아래는 실제 phase의 기존 집계와 raw 호출을 읽은 값이다. 보완에 QA04 변경과제를 새로 적용하지 않았으므로 A4/B4의 QA04는 **N/A**이며 BASIC B의2.5를 자동 상속하지 않는다.

| 구성·phase | QA01 ms 관측/예정 | QA03 ms 완료/예정 | QA02 | 실제호출 | 전체VIA USD | 동시 RSS MB |
| --- | --- | --- | --- | ---: | ---: | ---: |
| A4 TACTIC | 19,797.154 ·15/15 | 24,746.810 ·12/12 |24/24|30|0.0561648|45.121536|
| B BASIC |18,344.055 ·15/15|22,930.461 ·12/12|24/24|38|0.0431172|45.350912|
| B4 DIAGNOSTIC |14,467.847 ·15/15|18,085.194 ·12/12|24/24|37|0.0435792|46.972928|

### A4가 회복한 기능과 추가한 부담

원본 joint BASIC A의17/24·QA01 10/15·QA03 9/12가, 동일 screen producer를 보조한 A4에서는24/24·15/15·12/12로 관측됐다. A4는 화면 생산15회 뒤 **joint 최종판단15회**를 실행했고 source reread/owner repair는0회였다.15개 joint 실제 request 모두 `original.auxiliary_screen_parser`와 `original.observations`를 포함하고, 원본/overlay image도 실제 재전송했다. 최종 writer는 producer-owned binder가 아니라 계속 A joint였다.

A4와 BASIC B의 screen producer 실제 HTTP request JSON은 **15/15쌍 동일**했다. 그러나 response JSON은 **8/15쌍만 동일**했다. 따라서 같은 생산자·관측·표현 계약을 쓴 도움의 관측이라고 할 수 있지만, 모든 화면 후보값을 고정해 전달한 인과 실험은 아니다. 공동 판단도 적절한 보조자료를 받으면 이번 정확도·완료 조건을 충족함을 보였으므로 B의 소유권 분리만이 정확도 회복의 원인이라고 할 수 없다. 이24집합은 bbox의 픽셀정확도·미래 모든 화면의 일반정확도도 증명하지 않는다.

| 단계 | calls | input/output tokens | USD | 실제HTTP send→terminal 총/평균ms | 전송image 횟수 |
| --- | ---: | ---: | ---: | ---: | ---: |
| A4 screen |15|29,829/10,951|0.0294532|191,436.550 /12,762.437|27|
| A4 joint 최종 |15|44,619/5,540|0.0267116|105,070.510 /7,004.701|27|
| B screen |15|29,829/10,383|0.0285444|181,426.811 /12,095.121|27|
| B speech |15|10,646/2,977|0.0090216|66,628.059 /4,441.871|0|
| B candidate binder |8|9,750/1,032|0.0055512|26,691.245 /3,336.406|0|

A4의 최종 joint 단계는 B의 speech+binder 합보다 **$0.0121388** 더 들었고, screen 출력 차이는 **$0.0009088**를 더했다. 전체차이는 **$0.0130476(B 대비30.26%)**다. A4는 원본이미지를 screen과 joint 양쪽에 총54번 전송했고 B는 screen에서27번 전송했다. 이는 API에 실제 제공한 입력·출력·과금 부담이며, 모델 내부에서 같은 시각 연산을 반드시 두 번 수행했다는 계측은 아니다. 최종 단계 호출 수는 A4가15로 B의23보다 적어도 원본+parser+언어를 함께 넣는 입력과 출력은 더 컸다.

HTTP 서비스span 총합은 A4 296,507.060ms/B274,746.115ms다. 차이21,760.945ms 중 screen 약10,009.739ms, 최종단계 약11,751.206ms였다. 실제 QA03 합계 차이는21,796.193ms로 경로부담과 일치하는 양상이다. 다만 원격 모델 서비스시간·생성값도 바뀌었으므로 이를 오직 구조의 고정 처리량 차이로 확정하지 않는다. QA01/03은 여전히 callback/host계약 endpoint이고 내용 전체 정답 시각은 아니다.

### B4: 병행 효과와 의미 결과 차이를 구분

B4는 screen15+speech15를 동일 slot한도2에서 병행했다. **15쌍 전부 실제 send→terminal이 중첩**됐고 중첩 합66,745.257ms(쌍 평균4,449.684ms)였다. 화면후보 확장/출력repair0이며 최종 모델binder7회, typed code8회였다. BASIC B의 binder8/code7과는 한 경로가 다르다. 정확도는 B24/24→B4 24/24로 같았으므로 병행으로 정확도를 올렸다고 하지 않는다.

B4의 실제HTTP 서비스span합은283,413.904ms로 BASIC B보다8,667.789ms **늘었다**. 이와 동시에 중첩이 생겨 중첩을 뺀 관측모델span은216,668.647ms였고, 원래B의274,746.115ms보다58,077.469ms 짧았다. 실제QA03합계 감소58,143.203ms와 대체로 맞는다. 이는 이 실행에서 직렬 생산 대기를 실제 겹친 근거이며, 단순히 원격호출이 더 빨라진 관측은 아니다. 각 서비스시간·출력값·한 binder 분기까지 달라진 세 반복 결과이므로 중첩합을 그대로 고정적인 제품 절감량으로 쓰지 않는다.

B4의 QA01/03 관측평균은 B보다 약21.13% 짧았다. 전체비용은 **$0.0435792**로 오히려$0.0004620(1.0715%) 증가했다. screen 출력증가 비용+$0.0009696, speech차이+$0.0000036, binder한회 감소 등을 포함한 비용−$0.0005112가 합쳐진 결과다. 병행 자체가 token을 줄이거나 정확도를 바꾸는 정책은 아니다.

| 같은사례3회 | A4 QA01/QA03 ms | B QA01/QA03 ms | B4 QA01/QA03 ms |
| --- | --- | --- | --- |
| dense |31,372.113 /31,372.397|27,580.467 /27,580.712|22,341.768 /22,341.959|
| free |20,483.486 /20,483.679|16,842.570 /16,842.797|13,367.003 /13,367.302|
| page |18,808.005 /18,808.175|16,475.281 /16,475.479|15,607.886 /15,608.081|
| correction |14,161.084 /28,322.990|15,410.978 /30,822.855|10,511.288 /21,023.434|

A4가 모든 경우 B보다 느린 것은 아니다. 정정의 최종목표는 A4가 B보다 짧았고, B4는 이번네사례 모두에서 B보다 짧았다. 공통전체평균만으로 사례별 실행의존을 지우지 않는다.

B4의 동시RSS 최고46.972928MB는 B보다1.622016MB 높았으며 그 같은 표본의 Python/Rust는43.745280/3.227648MB였다. A4 최고는45.121536MB(Python41.861120/Rust3.260416)였다. 각12행 모두 동시관측이 있었지만 최대표본간격 A4 82.305ms/B4 142.014ms이며 실제연속heap/클라우드메모리를 측정한 값은 아니다. 동시에 살아 있는 request/response·trace 상태가 달라졌다는 조건을 설명하고 작은표본의 통계적 메모리 우열은 확정하지 않는다.

A4/B4 정정도 각각저장한이전제안3회씩을재주입했다. BASIC과 동일한 제어사건 replay이며 실제provider late-return 경쟁실험으로 바꾸어 서술하지 않는다. 단계/UUID/actual request·response 동등성/중첩/모든사례·원시SHA는 `analysis/dp04-tactic-comparison.json`, 표는 `.md`다. 정본 생성기는 `benchmark/architecture/four_dp_poc_v2/reports/compare_dp04_tactics.py`이고 기존QA/scorer 값을 바꾸지 않는 읽기전용 집계다.

## 증빙

- QA04: `qa04/dp04-rebased-calibration-v3/summary.json` 및 C05/C06 A/B별 actual.patch, source-changes.json, changed-source, regression.log, 실제 offline trace/PNG pixel 검사. `basic-v1-applicability.json`에 최종 candidate27파일 완전동일/차이0 증빙. 주지표는 frozen `basic-v1/qa04/summary.json`의 네 과제 평균.
- calibration-v1 최초 집합/좌표/재판단: `analysis/dp04-calibration-v1-review.json`.
- 단계/UUID/token/cost: `analysis/dp04-calibration-v1-stage-evidence.json`.
- 분석기 독립 검토: `analysis/metric-review-v2/review.md` (수정전 발견 보존, 수정후 9+5 checks와 실제 E4 진단 별도).

현재는 여러 보정(공통 raw overlay, schema/역할별 cap, count/type/geometry guard, owner repair)이 함께 적용됐다. 어느 하나가 정확도를 올렸는지 현 데이터만으로 분리하지 않는다. 위 BASIC 결론은 실제24행의 역방향 결과·실패를 포함한다. 별도 A4/B4를 BASIC 성능과 합산하지 않는다.
