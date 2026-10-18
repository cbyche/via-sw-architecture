# DP01 근거 공급: 구조와 실제 관측의 연결

2026-10-18. **DP01 BASIC36회 episode·108개 예정 상세응답과108개 실제 progress의 저자 검토 완료.** 동일4episode×ABC×3회, constructed synthetic PoC이며 사용자 연구·제품 검증이 아니다. 공식 지표와 별도로 실제 문안/원문/조건을 읽은 저자 의미판정을 보존했다. 독립 의미심사 완료를 주장하지 않는다.

**발표용 직관:** A는 읽은 근거를 같은 대화에 두고, B는 질문마다 필요한 근거를 만들고, C는 원본 활동 때 관계를 미리 만든다. C는 질문 뒤 추가 추론을 많이 줄였지만, 미리 만드는 비용과 큰 입력을 회수하지 못했다. A/B의 지칭은 모두 맞아도 정정 납기 누락은 남았다.

## 1. 여섯 QA: 구조 → 예상 → 실제 → 해석

| QA | 구조상 예상·조건 | 실제 A / B / C | 실제 원인과 해석 |
| --- | --- | --- | --- |
| QA01 첫 응답 | 유지/게시 근거가 지칭·처리의도 채택을 돕되, 명시적 제목만으로 B도 충분할 수 있다. | **6609.115 / 6266.887 / 11995.458ms**, 각36/36 입력 callback. 잘못된 지칭의 실제시간도 유지. | C의34/36 direct-complete와 A의9/36 direct-complete는 첫 JSON에 상세답변까지 생성한다. 비스트리밍 callback은 전체 JSON 수신 뒤 발생한다. C의 큰 입력/긴 출력과 추가본문 생성이 QA01을 늦출 수 있다. “게시 구조 자체는 본질적으로 느리다”라는 결론은 아니다. |
| QA02 지칭 대상 정확도 | 관계가 오래된 두 원본을 연결하는 데 도움을 주거나 보조자료를 원대상에 잘못 더할 수 있다. | **36/36 / 36/36 / 34/36**, 최초 original-ID exact set·필요판본. | C E4q3r1/E4q2r2에서 old-explanation을 두 원본에 추가했다. 실제 상세내용이 맞더라도 별도 QA02 오답이다. 조건·본문의미 성공으로 이 오류를 덮지 않는다. |
| QA03 VIA 처리시간 | 실제 유지근거가 충분하면 읽기/두 번째 답변 호출을 줄인다. 요청별 짧은 bundle은 더 싼 입력으로 상쇄할 수 있다. | **13617.335 / 14241.594 / 12364.438ms**. A36/36,C36/36; B35/36 **조건부 평균**. | C는 추가 source-backed answer2회, A27회,B33회(+repair1회)였다. C는 A보다 평균9.20% 낮았으나 C 사전생산33회는 이 입력 이후 경과 밖이다. 그 비용·메모리는 다른 QA에 포함한다. B는 완료누락 때문에 이 평균만으로 우열을 정할 수 없다. |
| QA04 실제 변경 범위 | 전달근거 조회/세부 source철회가 lifecycle·authority·producer·consumer에 미치는 범위가 다르다. | 공통C01=1,C03=2. C05 **1/1/1**, C06 **3/3/4**. 4과제 평균 **1.75/1.75/2.00모듈**; 기능2개 평균2/2/2.5. | 동일 기능이 실제 동작하는 독립 scratch patch와 acceptance로 확인했다. C06 C에만 producer의 세부철회 생산 경로가 추가로 바뀌었다. tie를 보존하며 임의 module분할을 하지 않았다. 개발생산성·모든 미래 변경의 대리값은 아니다. |
| QA05 전체 모델 비용 | A view재입력/B요청별구성/C독립생산은 다른 비용을 낸다. 미리 만든 작업이 실제 후속호출을 대체해야 회수된다. | **$0.0674956 / $0.0699160 / $0.1630788**, **63/70/71호출**. Receiver $0. | C는 총비용 A의2.416배, 그중 사전생산 $0.0729348(44.72%)이다. 준비를 제외해도 C요청비용 $0.0901440. 큰 supporting-passage재입력과 실제 장문/영어출력 영향이 함께 있다. A가 B보다 비용3.46% 낮았으나 B실패/문안차이도 포함된 관측이다. |
| QA06 실제 RAM | A view/C publication은 유지되고 B bundle은 요청동안 산다. 작은 상태차보다 bridge증거보관/직렬화가 클 수 있다. | 동시sample peak합계 **44.253 / 45.318 / 50.364MB**. 세 안12/12episode에 sampling결함 없음. | 각 peak의 Python/Rust는 A41.075/3.178MB, B42.025/3.293MB, C46.662/3.703MB. C의 publication논리최대7374byte와 전체 RSS차이를 동일시하면 안 된다. 원격 model weights·UI·on-device Omni메모리는 제외한다. |

### 경계와 실패를 같이 읽기

입력 capture시각을 보존하고 모델이 실제 target+intent를 채택한 뒤 progress를 표시한다. QA01은 구체적 처리의도이고, QA03은 실제 근거검사를 통과한 상세표시까지다. DP01에는 외부 Agent 공제가 없어서 QA03은 실제 입력 이후 경과이다. 완료 guard는 현재판본/권한/실제제공span과 nonempty exactquote를 확인하지만, 본문의 모든 추가주장이나 누락까지 증명하지 않는다.

B E1q2r1은 원문 행2/3을3/4로 잘못 쓴 quote provenance가 repair후에도 통과하지 못했다. **입력→실제 실패 callback은21053.636917ms(21.053637초)**이며 GOAL_COMPLETE는 없다. 이 실패시간을35개 성공평균에 완료값으로 넣지 않았고 예정36분모에서 미완료1건을 보존했다. 맞아 보이는 미표시 draft는 내용성공으로 세지 않는다.

## 2. 실제 호출·토큰·조회·사전생산

|안 / stage|호출|입력 tokens|출력 tokens|표준단가 비용 $|
|---|---:|---:|---:|---:|
|A `dp01_select_adopt_or_complete`|36|39182|11713|0.0344136|
|A `dp01_source_backed_answer`|27|29069|13409|0.0330820|
|**A 전체**|**63**|**68251**|**25122**|**0.0674956**|
|B `dp01_select_adopt_or_complete`|36|34603|11045|0.0315132|
|B `dp01_source_backed_answer`|33|32893|15131|0.0373668|
|B `dp01_repair_source_backed_answer`|1|1102|372|0.0010360|
|**B 전체**|**70**|**68598**|**26548**|**0.0699160**|
|C `dp01_publish_relations`|33|24837|39375|0.0729348|
|C `dp01_select_adopt_or_complete`|36|127180|23189|0.0879744|
|C `dp01_source_backed_answer`|2|2768|664|0.0021696|
|**C 전체**|**71**|**154785**|**63228**|**0.1630788**|

|실제 이벤트|A|B|C|
|---|---:|---:|---:|
|유효 이미제공근거에서 COMPLETE|9|3|34|
|원본근거 구성 후 완료 분기|27|33|2|
|질문 이후 EVIDENCE_READ|39|46|2|
|사전생산 EVIDENCE_READ|0|0|66|
|전체 read_to_string 실행|39|46|68|
|독립 publication|0|0|33|

이 조회 수는 실제 Rust source adapter 실행 횟수다. 매번 파일 전체를 read_to_string한 뒤 범위를 고르므로 OS cachehit/물리 disk I/O 감소를 측정한 값은 아니다. A/B는 미래질문을 위한 독립 relation생산을 하지 않는다. B도 실제표시된 recent답변을 공통으로 쓸 수 있으며3개 direct-complete가 실제 발생했다.

C의33개 publication 중18개는 다음 query에 노출되기 전에 새 publication으로 대체됐다. 이18개 생산호출 비용 **$0.0352844**도 전체에 포함했다. 원본이 전혀 사용되지 않았다는 뜻이 아니며, query가 publication을 입력받았다는 사실만으로 모든 record가 의미적으로 사용됐다는 증명도 아니다. 생산은 미래 utterance 없이 당시 source활동만 보았다. fixture는 생산이 끝난 뒤 질문을 제공하므로 busy producer와 사용자 입력의 경합은 이 실험에서 검증하지 않았다.

### 비스트리밍 완료 분기가 만든 QA01/QA03 교환

현재 bridge는 streaming이 아닌 **완성 JSON 반환 후** Rust가 adoption과 상세완료 callback을 실행한다. 충분한 근거가 있는 select-or-complete 호출은 adoption만 먼저 보내지 않는다. 따라서 한 호출을 줄이면 상세완료는 빨라질 수 있지만 첫 callback은 상세답변 생성까지 기다린다. A의direct9개 QA01/QA03은8638.136/8638.601ms, C의direct34개는12129.765/12130.465ms였다. 서로 다른 질문구성이므로 그 평균차를 독립 구조효과로 추정하지 않는다.

실제 E1r1에서 A q1은 metadata입력483token/출력224token의 READ 호출(원격 왕복4810.795ms) 뒤 답변호출을 했고, q2는 view를 재사용해 입력1245/출력528의 COMPLETE 한 호출(9659.588ms)로 끝났다. C q1은 게시근거를 넣은 입력2942/출력497의 COMPLETE 한 호출(9024.018ms)이었다. C의 첫 응답이 A q1보다 늦더라도 q1상세완료는 추가호출을 피했다. 이 관측에는 입력길이, 실제생성본문/언어, 원격서비스변동이 함께 있으며 구조만의 성능상수로 읽지 않는다.

## 3. 같은 네 episode의 실제 사례

|episode|A QA01 / QA03 ms|B QA01 / QA03 ms|C QA01 / QA03 ms|호출 A/B/C|해석|
|---|---:|---:|---:|---|---|
|dp01-e1|7130.294 / 9558.732|5989.054 / 11903.667|9302.337 / 9302.865|12/19/12|A의첫읽기 후view와 Cpublication이 후속원본/답변호출을 대체. B의span실패1건; B QA03조건부.|
|dp01-e2|5432.747 / 11333.144|5313.153 / 11129.744|8298.153 / 9403.607|24/24/26|주제가계속바뀌어A/B모두24호출. C는4source활동별준비를모두지불해26호출. A의충돌본문에추가날짜오류1건.|
|dp01-e3|6888.312 / 21822.022|6964.339 / 21819.901|18355.040 / 18355.935|12/12/15|A/B정정납기누락각3건. C는최신원문근거에서11월1일을실제로요약했지만영어단위가literal에누락.|
|dp01-e4|7470.297 / 15251.734|7351.396 / 15416.680|15378.599 / 15379.455|15/15/18|새대화의old관계재사용;C2건extra-target QA02오답. ABC모두일부조건/미확인요약누락이있어내용검토별도.|

## 4. 상세문안: 세 진단과 지칭/완료를 혼합하지 않기

|안|예정본문|old literal|표기동등|저자 실제의미 PASS|FAIL|PARTIAL|NOT_DISPLAYED|
|---|---:|---:|---:|---:|---:|---:|---:|
|A|36|30|30|30|5|1|0|
|B|36|29|29|30|3|2|1|
|C|36|27|32|34|0|2|0|

총108예정중107개가 실제 상세표시됐으며 미표시1개도분모에 남겼다. 저자는108개에 대해 실제 질문과 frozen source를 대조했고, 동일본문hash는동일goal조건에서만연결했다. 실제의미PASS는QA02가아니며 독립심사를완료한숫자도아니다.

- A/B E3q2의11월1일실제본문누락은각3건실패다. C의36-month/November1st는실제값이존재하는표기차이로 literal/equivalent를나눴다.
- A E2q3r1의 “해당날짜[10/30] 오전10시는비어”는원문10/31을다른날짜로바꾼추가허위주장이다. 필요한marker가모두있어도manualFAIL이다.
- A E4q3r1의long-term power ratios는장기전력비를다른물리량으로바꾼오역이다. r1의ABC E4q2는동일monitor/절전설정의구체조건이하나이상빠져PARTIAL이며,C r3E4q2는8대·tax·shipping견적조건누락으로PARTIAL이다.
- B r3E4q3는“확인되지않은사항”이라는자연스런표현으로실제미측정범위와계약한계를전달해literal은실패해도manualPASS였다. C E4r1q2도specific hardware settings만으로동일monitor+절전설정을전달했다고보지않았다.
- 108개progress는질문에대응하는구체적처리의도를실제표시했다. 영어/snake_case가섞인proxy문구의제품UX품질을검증한것은아니다. C의잘못추가된target2건의시간은그대로유지했다.

## 5. 상태·변경·보완의 실제 범위

A는같은대화에서실제읽은passage를보관한다. BASIC은정정시whole-view무효화, 새대화시view초기화이다. B는request-localbundle이며원본근거를숨겨장기보관하지않는다. C는현재권한/판본을검증한source-backed관계와실제supportingpassage를유지한다. common recent는실제표시text와별도원문provenance이며미표시facts.statement를다시“전달된답변”으로주입하지않는다.

논리상태직렬화최대는A view1869byte/B view2byte/C publication7374byte, recent A1529/B1828/C1837byte였다. 이는논리UTF-8payload크기이며RSS가아니다. C의RSS증가는Python bridge기여도크므로publication자체에6MB가필요했다고설명하지않는다. 동일시각합계에는공유page중복가능성이있으며Windows commit측정이아니다.

QA04는C05표시된답변근거조회/C06세부범위철회의실제독립patch와수용검사다. 철회된block뿐아니라근거없는ancillary조건/unknown문구도폐기해후속모델payload재주입을막았다. cal-v2→v3→formal의owned9모듈과common4모듈해시가같음을확인해기존acceptance기반을보존했다.

A2는 바뀐 source의 영향 조각만 무효화하는 정의 그대로, BASIC 전체 완료 후 별도 E3×3/동일 실패 분모로 실제 실행했다. 아래 별도 A2 보완 절에 실제 호출·조회·비용과 본문 결과를 기록했다. 기본 결과는 대체하지 않으며 추가 튜닝하지 않는다.

## 6. 근거 링크

- [공식BASIC보고서](../basic-v1/report.md), [episode지표](../basic-v1/episode-metrics.json), [episode별3회요약](../basic-v1/case-summary.json), [개별회차](../basic-v1/individual-trials.md)
- [전수본문·progress·원문행/해시JSON](dp01-basic-content-review.json), [의미실패사례목록](dp01-basic-content-review.md), [호출·토큰·원문조회·publication·RSS파생분해](dp01-basic-breakdown.json)
- [QA04patch/acceptance](../qa04/baseline-v2/README.md), [동결본전이증명](../qa04/baseline-v2/calibration-v3-transition.json), [A2준비](../tactics-preparation/dp01-A2/summary.json)
- [실제해석/표시코드](../basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2/src/dp01_request.rs), [source/완료guard](../basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2/src/dp01_evidence.rs), [C생산](../basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2/src/dp01_producer.rs), [lifecycle](../basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2/src/dp01.rs)

모든값은현재frozen Qwen profile의원격실제호출과Rust최소기능에서얻은관측이다. 전체제품/harness/OmniParser/오디오/온디바이스Omni검증또는보편적구조우위로일반화하지않는다.

## 별도 A2 보완: 의미 검토

기본36행은 그대로 보존하고, A E3 3회만 별도 보완했다. 실제 원문 read는12→9, 모델 호출은12로 같았으며 비용은 $0.0148424→$0.0145860이었다. 입력 token은8118→8433으로 늘고 출력은7247→7008로 줄었다. 원문 재조회 감소와 모델 입력 감소는 같은 효과가 아니다.

실제 표시 본문은 기본/A2 각각6개에서 모두 PASS3/FAIL3이었다. 정정 q2의 납기11월1일 누락3건이 A2에도 남았다. 이 보완은 내용 성공률 개선으로 해석하지 않는다. [전후 실제 본문 검토](dp01-A2-content-comparison.md)와 [호출·토큰·원문 근거](dp01-A2-content-comparison.json)에 보존했다. 공식 동일 사례 비교에서 QA01은6888.312→6394.130ms, QA03은21822.022→21432.660ms였다(관측/host 완료 각각6/6). 시간 감소는 순차 서비스 변동을 제거한 인과 추정이 아니며, 실제 read 감소 및 내용 실패 유지와 함께 해석한다.

독립 담당자도 A2 실제6본문을 원문과 재대조하여 q1 PASS3/q2 FAIL3으로 확인했다. [독립 재확인](dp01-A2-independent-content-review.md)은 기존 저자 판정을 아는 비블라인드 corroboration이며 별도 공식 정확도 지표나 의미 개선 증거가 아니다.
