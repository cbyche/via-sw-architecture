# DP04 최종 발표 근거 — 공통 정규화 판본

## 실제 판정

**동일한 공통 normalized-output 계약에서 A를 이 유한 사례의 기본안으로 조건부 선택한다.** 필수 endpoint를 A는 전부 관측했고 B는 dense3회 미완료다. QA순위는 바꾸지 않았다. 미관측을0초/벌점으로 만들거나 성공만 뽑아 기본안을 선택하지 않았다. 필요한 기능/권한/최소기기 전제는 기존 패키지의 선택 전제다. 이 결론은 이전pixel 기본A의 성공을 소급 선언하는 것이 아니다.

- 기존 pixel48: 기본A/B 둘 다 미완료라 확정 기본 승자 없음. A2+A5만 그 phase의 전체 endpoint/정답집합 통과. A2+A5는 공통관측12개에서 A보다 약17.3% 느렸고 RSS가 증가했다. B4는 같은관측13개에서 B의 지연을 줄였지만 정정 실패를 없애지 못했다.
- 새 공통표현24: A는 첫15/15·최종12/12·정답24/24. B는12/15·9/12·18/24. **추가 tactic 없이 A선택**. 정규화 A+A2/A5, 정규화 B4는 미실행이다.
- 구조 소유권 하나만으로 정확도 차이의 원인을 확정하지 않는다. 동일모델에 전달한 좌표 표현·prompt/schema 변화, 모델 출력 변동, producer/binder 처리 경로가 함께 관련된다. 공식 tool 규약은 고정snapshot API의 필수 출력 보장이 아니다.

## 여섯 QA: 직관 → 실제 → 설명의 범위

|QA|구조상 직관|이번 실제값과 해석|
|---|---|---|
|QA01|A는 공동판단1회, B는 초기 screen→speech 및 조건부 binder/expansion으로 callback 전 직렬의존이 길 수 있다.|A 7.716261초15/15, B 18.774386초12/15(조건부). 전체속도비율 금지. 양쪽관측12개만은 A 7.300215초/B 18.774386초로 경로직관과 일치한다. 사후설명용이며 분모15를 대체하지 않는다.|
|QA02|A는 원본+발화를 한 owner가 같이 해석한다. B는 독립생산 결과의 모순을 검출하고 원 owner에 확장을 요청하지만 생산오판을 반드시 고치지는 못한다.|A24/24, B18/24. B실패6집합은 dense3회×2표현이다. 초기 대상제안은 D06/D11로 맞았지만 D11 bbox가 rawpoint를 포함하지 못해 실제채택은 실패했다. 시각대상인식 자체/전체의미정확도의 구조우위로 확대하지 않는다.|
|QA03|이 scoped grounding 작업은 채택과 상세완료가 거의 같이 발생한다. 별도 business work를 발명하지 않는다.|A 9.645732초12/12, B 25.033006초9/12조건부. 같은9목표 A 9.734087초/B 25.033006초. 외부 receiver 공제0. 정정목표는 최초revision입력부터 최신revision완료까지이며 중간성공만으로 종료하지 않는다.|
|QA04|생산자소유권이 특정미래변경을 국소화할 수 있으나 이는 실제독립patch로 확인해야 한다.|**이번 정규화 판본 미측정**. 이전 A/B 각2.5는 이전4과제 change exercise 결과이며 새adapter/tactic으로 전이하지 않는다. 새원천적개선 주장 없음.|
|QA05|B의 작은 binder input만 보면 안 되며 screen/speech/미사용생산/실패확장 모두 합친다.|A15호출/$.0254572, B42호출/$.0650212. 모두usage확정. A input38803/output6210; B input74185/output22092. 실제단계와 실패비용 포함이다. 아래표 참조.|
|QA06|B는 후보·발화·binding state를 보존하는 경로가 있으나 작은 harnessRSS차이는 architecture winner가 아니다.|A40.140800MB, B41.893888MB 최고동시표본. Apeak의 Python37.109760+Rust3.031040MB, B38.502400+3.391488MB. Fresh worker, cloud/UI제외, shared-page중복 미검증. 표본최대간격 A78.532/B87.316ms. 구조적우월성 증명으로 사용하지 않는다.|

QA01은 실제 ADOPTED_HANDLING DISPLAY_PROXY callback까지의 시간이며 전체설명문장의 의미정확도 자동검증은 아니다. QA03은 동결 host grounding 완료계약이다. 한 A dense 응답의 evidence 설명은 pixelpoint와 normalized bbox 숫자를 섞어 서술했으나, 실제 구조화bbox는 엄격변환 뒤 pixel기하검사를 통과했다. 이 자연어 evidence 설명의 정확성을 별도로 입증했다고 주장하지 않는다.

## 고정 분모

각 안12episode=4사례×3반복. QA01은 dense3+free3+page3+정정revision6=15입력, QA03은 dense3+free3+page3+정정최종3=12목표, QA02는 dense6+free3+page3+정정12=24표현·판본 집합이다. 3반복은 모집단 일반화/통계적유의성의 증명이 아니다.

## B dense3 실패 원값

세 반복의 초기 source-backed target 제안은 D06/D11이었다. D11의 normalized y=[530,690]은 높이650에서 ties-even변환 후 pixel y=[344,448]이다. pointer-2의 원래 pixel좌표는(630,480)이다. x는 r1[490,700],r2/r3[510,720]로 달랐으나 세번 모두 y480을 포함하지 못했다. 초기 typed code binding 뒤 공통 final geometry gate가 거절→screen owner 1회확장→해당cue가 unknown/target_ids=[]→model binder need_expansion→한도소진. 별도의 provider/JSON/단위wire거절은0이다. 실패관측까지 r1 60.926586/r2 60.840100/r3 60.729165초이며 QA01/QA03 성공평균에서 빠져도 원시와 분모에는 남는다. 입력된 rawpoint를 예상정답 bbox로 덮거나 clamp하지 않았다.

정정의 oldproposal 재주입은 저장된 실제이전결과의 **REPLAYED_CONTROL_EVENT** 검증이다. 실제 provider지연/late return을 측정한 것으로 쓰지 않는다.

## 단계 및 역할

A15 joint만, reread0. B screen15+speech15+screen expansion3+binder9=42호출. B typed_code_bind9사건은 모델호출수가 아니며 초기dense3은 이후기하 gate에 거절된다. 모든입력/출력토큰과HTTP시간은 [정본표](normalized-comparison.md) 참조. B의 raw source는 screen owner에 남고 binder는 후보만 받는다. raw→final 우회로 바뀌지 않았다.

### 발표용 구조 텍스트

- A: 원본화면·rawcue·발화 → 공동 grounding owner → strict normalized→pixel adapter → 공통기하/개수/판본 gate → 실제채택 callback·상세완료. 모순이면 동일 owner의 원본 재판단1회.
- B: 원본화면·rawcue → screen producer → 같은 strict normalized→pixel adapter → 화면후보; 발화·rawcue·prior → speech producer → 후보기반 typed/model binding → 공통기하/개수/판본 gate → 실제채택. 모순이면 원 screen owner의 원본확장1회. 기본B는직렬이며 B4초기병행은별도tactic이다.

## 원본·정답·공정성

같은4사례·실제PNG·overlay·transcript·event·revision을 A/B가 받았다. rawcue는pixel불변이며 출력만0..1000. bbox변환은 실제frame폭·높이로 축별계산하고 이상단위/다른frame/중복bbox/역전·범위밖좌표는거절한다. oracle bbox/정답대상은candidate입력에없다. QA02는표현·판본별첫채택target집합이며 bbox IoU평가가아니다. 후보자기정합성검사통과가모든권위bbox의정확성을보장하지않는다. 원래pixel출력지시도명확했으므로 이번변경을 미구현필수어댑터결함수정으로단정하지않는다. 두안의 공통표현 민감도실험이다. 같은원본이라도원격모델출력은변동할수있다.

## 동결·비용·재현

- [정본JSON](normalized-comparison.json), [단계·paired·실패표](normalized-comparison.md), [계획](normalized-plan.json), [freeze](normalized/freeze.json), [raw](normalized/runs.jsonl), [최종확인](normalized-final-audit.json), [분석판본](normalized-analysis-provenance.json).
- 모델qwen3.7-plus-2026-05-26, thinkingfalse, temperature0. 원래caps joint2600/screen3000/speech1400/binder2000 유지. balanced AB순서. 실제24/24unique, 요청57=known57+unknown0, 총$.0904784, 예약0. 요청·usage·local/global원장연결을포함한85/85대조PASS이며품질성공85회라는뜻이아니다.
- DP04 전체72실행=기존48+새24, 실제요청180=known179+UNKNOWN1. 확정환산$.2563308+기존미확정예약상한$.0729404=보수적합계$.3292712(승인$.55내). 실제청구서/현재가격이아니라고정비교단가(input$.40/output$1.60 per1M)×실제usage환산이다.
- 동결 source196개 전부보존SHA일치, 실제후보src/binary불변. 부모요청으로 실행후 report generator만 실제선택/paired/실패설명을추가했고 이전generator·최초파생표는 normalized-analysis-history에보존. 모델/fixtures/oracle/guard 변경없음.
- [Qwen 공식 grounding tool](https://github.com/QwenLM/Qwen-MM-Plugins/blob/main/src/capabilities/api/qwen_mm_plugins_api/vl/grounding.py), [축별pixel변환](https://github.com/QwenLM/Qwen-MM-Plugins/blob/main/src/shared/image.py). 조회한main코드의참고규약이며고정snapshot의보장으로확대하지않는다.

## 남은 한계와 마감

새A에대해추가tactic의열세QA보완효과를검증한것은없다. 기존pixel A2+A5의복구효과/지연·RSS부담과 B4의초기병행효과를별도증거로설명할수있지만, 정규화기본선택과한누적구성처럼합치면안된다. B의구조상후보재사용/복구국소화는조건부기대이고이번finite값으로일반적이점을입증하지못했다. 양쪽모든역할의정확도와보편적architecture우열도입증되지않았다. **승인된공통표현24에서모델실험을종료한다.**
