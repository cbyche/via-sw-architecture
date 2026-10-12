# v3 A1의 두 실제 Request 해결 경로 검토

상태: **READ_ONLY_OBSERVATION / NO_QA_VERDICT**. A1의 완료된 `(A, 1, S2-02)`와 `(A, 1, S2-05)`만 검토했다. 모델 호출, 후보 실행, 테스트 실행과 후보/fixture/정답 수정은 수행하지 않았다. 이후 B1과 반복 결과를 보지 않은 사례 설명이며 우열이나 QA 점수를 산출하지 않는다.

기준은 [campaign manifest](../manifest.json)의 `DP41-P21-MAC-TEXT-v3`, frozen source digest `f521d6bea196342dab6c0f66268134e9c33f25dfa86a0aac874afc160cb6a20a`이다. 사건과 실제 모델 호출은 측정 기록이고, 입력 전사와 Voice 재생은 공통 모형이다. 실제 음성 모델의 정확성/속도나 Windows 실행에 대한 증거로 사용할 수 없다.

동결된 [ASR-QA-v6 원문](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 S2-02/S2-05 및 §8.2를 함께 적용한다. S2-02 본문의 접수 checkpoint는 유지하며 §8.2의 접수+8초 최종 tail도 요구된다. `OBSERVED`, `COMPLETED` Task 상태, 실제 전달 receipt는 서로 다른 사실이고 어느 것도 의미 정답을 자동 보증하지 않는다.

## 1. 판독과 원인 분류의 범위

이 검토는 원문 요구 → 첫 모델 Context → 모델이 선택한 조회 → 실제 owner receipt → 출력 검증 → 채택 의미 → 실제 payload/효과 → Text/Voice 전달을 연결한다. 근거가 없는 원인 추정은 아래 한계로 남긴다.

| 분류 | 이 문서에서의 의미 |
| --- | --- |
| 요구 표현 가능성 | 동결된 연산/selector/argument로 목표와 필요한 실제 근거를 전달할 수 있는지 |
| 모델의 잘못된 선택 | 제공된 handle/지원 연산이 존재하는데 모델이 다른 목표/조회/조건을 출력한 관측 |
| 지원 한계 | 공통 owner 검색, 모형, 유한 언어가 제공하지 않는 동작이나 미입증 동작 |
| 코드 결함 | 합법적으로 표현된 의미와 계약에 대해 코드가 다른 효과를 낸 증거. 단순한 오답 또는 query 비관련 결과만으로 단정하지 않는다. |

공통 [prompt 계약](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py)에는 `NEW_TASK/CREATE_PROPOSAL`, `TASK_RESULT(source_ref:task_id)`, `PAST_DELIVERED(source_ref:publication_id)`, `KNOWN_REF`가 있다. 평가 기준과 가격 평가 제외는 제안서 작성이라는 Downstream Agent의 domain instruction에 담을 수 있다. source 참조/권한/Task/의존 관계와 같은 VIA 책임은 해당 typed 항목에 남아야 한다. 이 두 사례에는 임의 실행 표현식이나 새 relation 연산이 필요하지 않다. 이 사실은 올바른 모델 출력이나 완성된 제안서 품질을 보증하지 않는다.

## 2. `(A, 1, S2-02)` — 들은 기준을 확보하지 못한 새 제안서 위임

[trial 원장](../workers/A1/trials-unjudged.jsonl)의 line 4, `E3746`; Request `R:1a699de1-d4f2-4ff5-85aa-958c6fb45de6`, input `I:0a03d6a5-627a-4f5f-8bc9-cd481ed8a32e`, Conversation `C현재`이다. trial 상태는 `EPISODE_TIMEOUT`이다. raw JSONL 한 행의 UTF-8 bytes는 마지막 LF를 포함하며 SHA-256 `40d2eb3d7e850ad993997725d9089ee80b08ac266cf938c7d13942418aaa2d33`다.

### 2.1 원천과 첫 Context

원문 요구는 실제 들은 E1의 첫 두 문장, 현재 D2@v2, D3@v1을 사용한 새 제안서 위임이다. 유지비/설치 기간이 평가 기준이고 가격은 평가에서 제외한다. E1의 Text 전체에만 있는 보안 문장은 실제 들은 조건이 아니다. D2@v1은 현재 제품 비교 결과가 아니다.

[첫 Context](../workers/A1/model-contexts/c47ed4b9-5d23-425b-bfe2-2b274ce6b87c.json)는 현재 사용자 발화와 E1@v1/D2@v1/D2@v2/D3@v1의 source metadata를 받았다. T비교/T견적 metadata에는 각각 현재 D2@v2/D3@v1 handle이 있었고, E1의 interaction metadata에는 `text_presented=true`, `delivered_receipt=true`, `interrupted=true`가 있었다. 설명 원문과 실제 들은 문장 자체는 초기 Context에 들어 있지 않았다. owner read가 필요한 상태였다.

### 2.2 실제 조회와 추가 질문

아래 event는 [events 원장](../workers/A1/events.jsonl) 및 같은 trial의 `events`에서 확인된다. 시간은 scene 시작 기준이며 점수나 VIA 귀속 시간을 계산한 값이 아니다.

| Event / scene seconds | 모델 선택 또는 실제 receipt | 관측 의미 |
| --- | --- | --- |
| E2104 / 4.809 | `TASK_RESULT(query="T비교", limit=1)` | 실제 T비교 locator를 지정하지 않은 bounded query |
| E2106 / 4.812 | read `a901a6b1-2194-4cee-808c-d4be0f45f6e6`: D2@v2, “A: 설치 2일, B: 설치 4일” | 실제 현재 비교 원문 |
| E2110 / 4.814 | `TASK_RESULT(query="T견적", limit=1)`의 read `68c7cf7f-f7bc-4011-ad57-5aaa0b33c055`: 다시 D2@v2 | D3 견적 원문은 이 모델 read로 확보되지 않았다. 두 후보 결과 가운데 D2가 선택됐고 두 번째는 cache hit다. |
| E2114 / 4.822 | `RECENT_INTERACTION(query="criteria", limit=5)` → H0001~H0005 | E1 원문/실제 Voice 범위를 읽지 못했다. 반환 후보 총수 2001, 반환 5건은 실제 배경 기록이다. |
| E2172 / 7.170 | CLARIFY: 두 완료 Task 사용 여부와 “들은 기준”을 추가 질문 | 질문에 H0001~H0005가 언급된다. 실제 조회 실패를 모델이 질문으로 넘겼다. |
| E3182 / 45.800 | 실제 QUESTION_PRESENTED | Text 7.177초부터, Voice 7.449~45.790초의 전체 질문이 전달됐다. |
| E3269/E3272 / 약 49.806 | 실제 후속 전사 “새로운 조건이나 권한은 추가하지 않았어. 처음 요청대로 처리해줘.” | 공통 unexpected clarification 응답이다. 정답 기준/추가 권한을 제공하지 않는다. 실제 질문 전달 뒤 1초 대기와 3초 발화가 적용됐다. |

`TASK_RESULT(query=...)`를 Task ID의 정확한 조회로 해석하면 안 된다. 동결된 [owners.py](../frozen-source/prototypes/candidates/dp41_45_poc/shared/owners.py)의 186~228행은 명시적인 `task_id/source_ref/ref`가 있을 때 해당 Task 결과를 좁힌다. query만 있으면 전체 Task에서 실제 결과 artifact를 모은 뒤 artifact 객체에 lexical term 점수를 매긴다. parent Task ID/설명은 이 점수 객체에 포함되지 않으므로 두 query 모두 같은 첫 artifact가 반환될 수 있다. 현재 계약은 query의 semantic Task 검색 성공을 보장하지 않는다. 이 결과는 **공통 검색의 관련성 한계와 모델의 locator 미사용**으로 나눠 기록한다. 잘못된 견적 내용을 생성하거나 정답으로 고친 것은 아니다. 두 후보 모두 명시적인 Task locator를 쓸 수 있으므로 이 한계를 DP41A/B 고유의 한계로 귀속하지 않는다.

### 2.3 출력 검증, 채택과 실제 효과

E3337 출력은 `dp41-meaning-runtime-2`였다. E3338은 명시된 v1 계약에 따라 `version must be dp41-meaning-runtime-1`로 거부했다. 다음 실제 호출의 E3382는 v1으로 바뀌었으며 동일한 목표와 누락된 기준을 유지했다. 코드가 버전 오류를 직접 치환하지 않았다. v3 prompt 버전 2는 meaning schema 버전 1과 별개다.

E3386 / 54.383초에 채택한 의미의 핵심은 다음과 같다.

```json
{
  "version": "dp41-meaning-runtime-1",
  "goal": {
    "id": "G-create-proposal",
    "kind": "NEW_TASK",
    "operation": "CREATE_PROPOSAL",
    "target": null,
    "source_refs": ["D2@v2", "D3@v1"],
    "instruction": "제품 설치 기간 비교 결과(D2@v2)와 제품 견적(D3@v1)을 바탕으로 새 제안서를 작성하세요."
  },
  "constraints": [],
  "preservations": [],
  "relations": [],
  "unresolved": []
}
```

위는 읽기 편의를 위한 핵심 필드 투영이며 원본 `goal.arguments.source_refs`는 source_id/revision 객체다. 채택 meaning 전체의 canonical SHA-256은 `fa62faa8704616d4912a6ca768761d2450d83373f700ed99110185a90c6a8996`이다. `evidence_refs`는 앞의 D2 read 두 건이며 E1/들은 조건은 포함되지 않았다. D3 참조는 초기 Task/source metadata에 실제 있었으므로 그 handle 자체가 가공된 ID는 아니다.

E3392 / 54.390초의 실제 명령은 `CMD:92695e1f-bd33-47b3-9b71-ae52e0fa8343`, 새 Task `T:fe578462-2007-42f2-8d1e-798c06fad936`, Execution `X:cd60802b-dd81-4232-b0b5-e4e3a59f8632`이다. [effects 원장](../workers/A1/effects.jsonl)의 payload에는 실제 D2@v2/D3@v1 본문만 있고, E1/들은 기준/가격 제외가 추가되지 않았다. payload canonical SHA-256은 `7c741ed6fa85c9a10b82dc9807da39ef0b9b6b52522196dede024e232514f40d`이다. E3400 / 54.594초에 실제 외부 접수 receipt가 왔다.

E3609 / 62.400초에 artifact `A:beb07765-d670-4908-bbea-c6c4ebcf9e99@v1`가 실제 전달 payload의 두 source 본문을 연결해 생성됐다. 모사 Agent는 제안서 기준이나 제외 조건을 정답으로 보충하지 않았다. 새 Task는 실제 최종 상태 `COMPLETED`이고 T비교/T견적은 완료 상태를 유지했다. 이 stub 결과는 보고서의 domain 품질을 검증하지 않는다.

### 2.4 Text 표시와 Voice 전달

| 실제 publication | Text 표시 | 실제 Voice | 전달 receipt |
| --- | --- | --- | --- |
| `P:1:d590d463-22e0-4177-be58-19db245c6ff4` 질문 | 7.177초 | 7.449~45.790초, 전체 질문 | true |
| `P:2:68bae07b-e139-4d0d-90b5-38f5066cf56c` 접수 안내 | 55.599초; “CREATE_PROPOSAL 작업은 접수(ACCEPTED)되었으며, 완료되지 않았습니다.” | 55.867~64.041초, 전체 안내 | true |
| `P:3:78c32cf5-7db0-4bca-ad2e-3e8088dd8dba` 최종 결과 | 63.580초; 실제 두 source 값 포함 완료 안내 전체 | 64.321초 시작; 67.032초까지 “CREATE_PROPOSAL 작”만 재생 | false, `LOCAL_CANCELLED` |

E3741/E3745의 자동 episode timeout과 E3743 OUTPUT_CANCELLED가 실제 부분 Voice를 남겼다. 최종 Text가 표시됐다는 사실을 전체 Voice 전달로 바꾸거나, 부분 Voice를 빈 문자열로 바꾸면 안 된다. 위 시간은 [deliveries 원장](../workers/A1/deliveries.jsonl)의 sink 기록을 사용했고 event emit 시간의 미세한 차이와 구분했다. `end_s`는 Voice 종료를 뜻하지 않으므로 `voice_end_s`를 읽었다.

### 2.5 원인과 미입증 부분

- **표현 가능:** NEW_TASK/CREATE_PROPOSAL, D2/D3 명시 locator, E1 actual delivered-history read와 domain 기준 instruction으로 요구를 표현할 수 있다.
- **모델 선택 관측:** E1을 읽지 않았고 기준/가격 제외를 채택 의미와 명령에 넣지 않았다. schema 버전 오류는 거부 후 별도 실제 호출에서 수정됐다.
- **공통 지원 한계:** parent Task query의 관련성 검색 및 영문 `criteria`의 한국어 설명 검색을 보장하지 않는다. 이를 A 고유 오류율이나 B의 자동 정답 이점으로 일반화할 수 없다.
- **코드 동작 증거:** 채택한 새 Task/두 source payload가 그대로 실행됐다. 누락된 기준을 코드가 정답으로 채우거나 합법적 CREATE_PROPOSAL을 다른 연산으로 바꾼 증거는 없다. deadline/긴 질문/출력 queue의 시간 귀속은 이후 인과 DAG 분석 대상이며 여기서 구조 결함이나 QA 실패 수로 확정하지 않는다.

### 2.6 실제 호출 원장

| 원장 line / role | 실제 호출과 출력 요약 | Context / Output 원본 |
| --- | --- | --- |
| 11 / CINTERPRET | `c47ed4b9-5d23-425b-bfe2-2b274ce6b87c`<br>READ; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/c47ed4b9-5d23-425b-bfe2-2b274ce6b87c.json)<br>[Output](../workers/A1/model-outputs/c47ed4b9-5d23-425b-bfe2-2b274ce6b87c.json) |
| 12 / CINTERPRET | `e8279a1c-5a00-4fb7-9315-c6973565f4be`<br>CLARIFY; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/e8279a1c-5a00-4fb7-9315-c6973565f4be.json)<br>[Output](../workers/A1/model-outputs/e8279a1c-5a00-4fb7-9315-c6973565f4be.json) |
| 13 / CINTERPRET | `a9fa4a3a-84f2-4ce5-963a-1383a0ecaf55`<br>PROPOSE / dp41-meaning-runtime-2; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/a9fa4a3a-84f2-4ce5-963a-1383a0ecaf55.json)<br>[Output](../workers/A1/model-outputs/a9fa4a3a-84f2-4ce5-963a-1383a0ecaf55.json) |
| 14 / CINTERPRET | `97d1b20e-32ed-4a93-ae24-4dc786b3fdd5`<br>PROPOSE / dp41-meaning-runtime-1; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/97d1b20e-32ed-4a93-ae24-4dc786b3fdd5.json)<br>[Output](../workers/A1/model-outputs/97d1b20e-32ed-4a93-ae24-4dc786b3fdd5.json) |
| 15 / CRESPONSE | `9b63ecb0-72e4-42f9-a364-0aec53580805`<br>응답 문장; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/9b63ecb0-72e4-42f9-a364-0aec53580805.json)<br>[Output](../workers/A1/model-outputs/9b63ecb0-72e4-42f9-a364-0aec53580805.json) |
| 16 / CRESPONSE | `1f22f3e8-0ce3-4baf-8e42-88837c605fa1`<br>응답 문장; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/1f22f3e8-0ce3-4baf-8e42-88837c605fa1.json)<br>[Output](../workers/A1/model-outputs/1f22f3e8-0ce3-4baf-8e42-88837c605fa1.json) |

CINTERPRET 4회와 CRESPONSE 2회가 모두 HTTP 200, attempt 1이다. 스키마 거부로 추가 의미 호출이 발생했지만 API transport 재시도는 아니었다. CLARIFY는 모델 생성 출력 그 자체를 재생했고 별도 CRESPONSE 호출로 계산하지 않는다.

## 3. `(A, 1, S2-05)` — 실제 새 문서를 읽은 뒤 목표를 직접 비교로 변경

[trial 원장](../workers/A1/trials-unjudged.jsonl)의 line 7, `E7103`; Request `R:ec402211-57cc-488f-b5c3-bc55d79ebacb`, input `I:fd5591cd-2fd6-4cca-a5a2-63f6d3e08d52`이다. trial 상태는 `OBSERVED`다. raw JSONL 한 행(LF 포함) SHA-256은 `2250c16583c02a234ce235594a9344ad4f57fac59ac1155bd4582cc2435bb2d1`다.

### 3.1 실제 availability와 Context

원문 요구는 실제 들은 납기 2주/가격 제외 기준과 새 비교/견적을 사용한 새 제안서 위임이다. canonical 원천 사건은 2.97/2.98/2.99초이고 실제 기록은 E6541/E6545/E6549의 2.970747/2.981733/2.990509초다. 최초 전사 확정 사건 E6556은 3.001113초다. 미래 계획을 선주입한 것이 아니라 실제 availability 이후 원천 handle이 첫 모델 Context에 들어왔다.

[첫 Context](../workers/A1/model-contexts/1ce78ffa-55aa-4091-973e-54e053bb0833.json)는 E새@v1/D새비교@v1/D새견적@v1 handle과 E새의 실제 전달 interaction metadata를 받았다. 설명 본문은 owner read 이전에는 Context에 없었고 현재 요청에 연결된 제안서 Task도 없었다.

| Event / scene seconds | 실제 선택과 반환 |
| --- | --- |
| E6608 / 5.134 | D새비교와 D새견적만 EXPLICIT_SOURCE로 READ. 모델의 query 설명은 D새비교를 “previous standard or baseline criteria” 확인 대상으로 썼다. |
| E6610 / 5.135 | read `fbb0ca30-64a4-4921-99f1-09f2b35510cb`: D새비교@v1, 실제 납기 10일/20일 |
| E6614 / 5.135 | read `fd2859ed-f228-4944-b457-55ca344979c5`: D새견적@v1, 실제 가격 30백만원/40백만원 |
| E6674/E6675 / 7.586 | PROPOSE meaning version 2; v1 계약으로 거부 |
| E6715/E6719 / 9.136~9.140 | 모델이 v1으로 다시 출력; 같은 DIRECT_RESPONSE/COMPARE를 채택 |

실제 read는 두 문서를 정확히 반환했다. E새의 실제 들은 기준은 읽지 않았으며 모델의 query 문구가 비교 문서에 없는 2주 조건을 생산하거나 문서 body를 바꾸지는 않았다.

### 3.2 채택 의미, 실제 효과와 전달

실제 채택 goal은 `G-compare-baseline`, `kind=DIRECT_RESPONSE`, `operation=COMPARE`, `target=null`, `arguments.source_refs=["D새비교@v1","D새견적@v1"]`다. instruction은 두 제품 납기와 가격을 비교하되 가격 우열은 평가하지 않고 인용하라고 했다. E새나 납기 2주 기준은 포함되지 않았다. 전체 meaning canonical SHA-256은 `0f6f51ec191d1c4806c502391d0db9ad7ad9e82c7157183427de541efb761130`이다.

trial `effects=[]`이고 새 Task/Agent 명령/artifact는 생성되지 않았다. 직접 COMPARE의 실제 sources는 조회한 두 문서 그대로였다. 공통 실행기가 제안서 작성 지시를 추가하거나 모사 Agent에게 대신 위임하지 않았다.

publication `P:1:3847a97b-d9a1-46fa-a6bf-53e3fab749e4`는 Text 10.209502초, Voice 10.475740~19.648605초로 “제품 A 납기 10일, 제품 B 납기 20일. 제품 A 가격 30백만원, 제품 B 가격 40백만원.” 전체를 실제 전달했다. `delivered_receipt=true`는 그 문장이 전달됐다는 사실이다. 새 제안서 위임/2주 기준 반영/최종 ProposalFresh 완료를 의미하지 않는다. T보호는 RUNNING으로 유지됐다.

### 3.3 원인과 미입증 부분

- **표현 가능:** 동일한 finite 계약에 NEW_TASK/CREATE_PROPOSAL과 E새 실제 delivered-history read가 존재한다. 요청의 작성 목표와 들은 기준을 나타내는 데 새로운 연산은 필요하지 않다.
- **모델 선택 관측:** “제안서 만들어줘”를 직접 COMPARE로 바꿨고 E새를 조회하지 않았다. 사용자 발화에 명시된 가격 평가 제외는 instruction에 보존했으나, 들은 2주 기준은 확보하지 못했다.
- **지원 한계:** 현재 모사 domain producer는 실제 payload/source 결합까지만 검증한다. 실제 제안서 내용의 domain 품질이나 표현 언어 전체의 완전성은 이 두 사례로 검증할 수 없다. 이 사례에서는 Task를 만들지 않아 제안서 stub 경로 자체가 발생하지 않았다.
- **코드 동작 증거:** DIRECT_RESPONSE/COMPARE는 합법적인 일반 연산이며 실행 결과는 채택한 연산과 일치한다. 의미와 사용자 요구가 다르다는 이유만으로 runtime이 목표를 고쳐 주는 것이 정답 보정이다. 합법적으로 명시한 새 작성 목표를 runtime이 비교로 바꾼 증거는 없다.

### 3.4 실제 호출 원장

| 원장 line / role | 실제 호출과 출력 요약 | Context / Output 원본 |
| --- | --- | --- |
| 26 / CINTERPRET | `1ce78ffa-55aa-4091-973e-54e053bb0833`<br>READ; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/1ce78ffa-55aa-4091-973e-54e053bb0833.json)<br>[Output](../workers/A1/model-outputs/1ce78ffa-55aa-4091-973e-54e053bb0833.json) |
| 27 / CINTERPRET | `54b2a364-4beb-4f0d-891d-168379df9c4a`<br>PROPOSE / dp41-meaning-runtime-2; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/54b2a364-4beb-4f0d-891d-168379df9c4a.json)<br>[Output](../workers/A1/model-outputs/54b2a364-4beb-4f0d-891d-168379df9c4a.json) |
| 28 / CINTERPRET | `beb00c90-75e1-4102-ae63-3c934c83b314`<br>PROPOSE / dp41-meaning-runtime-1; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/beb00c90-75e1-4102-ae63-3c934c83b314.json)<br>[Output](../workers/A1/model-outputs/beb00c90-75e1-4102-ae63-3c934c83b314.json) |
| 29 / CRESPONSE | `c55d751e-ff34-4d47-b795-2ccb6faaaea0`<br>응답 문장; HTTP 200, attempt 1 | [Context](../workers/A1/model-contexts/c55d751e-ff34-4d47-b795-2ccb6faaaea0.json)<br>[Output](../workers/A1/model-outputs/c55d751e-ff34-4d47-b795-2ccb6faaaea0.json) |

CINTERPRET 3회와 CRESPONSE 1회, 모두 HTTP 200, attempt 1이다. 이 문서는 해당 호출의 비용/시간 점수도 산출하지 않는다.

## 4. 증거 SHA-256과 후속 사용 한계

선택한 두 trial의 stable raw line, 그 안의 실제 payload와 meaning, 관련 10개 호출의 Context/Output artifact 20개를 읽어 검증했다. 아래 `context_artifact_sha256`은 호출 원장의 `context.sha256`이며, 별도로 기록된 caller fingerprint `context_sha256`와 혼동하지 않는다. artifact hash는 JSON을 `ensure_ascii=False`, `sort_keys=True`, `separators=(',', ':')`로 canonical 직렬화한 bytes를 사용했다. append 중인 원장 전체를 완성된 고정 hash로 주장하지 않는다.

```text
S2-02 CINTERPRET c47ed4b9-5d23-425b-bfe2-2b274ce6b87c
  context_artifact_sha256=272a5206fc16c661c4d0ea9bb2f7e3f590efdbdd7e02594d13534620041b611b
  output_artifact_sha256=141c75a72b1f9d3ef0dfb40bf912c4f8f5fb5d6f79a42fdc26e0b43ba374c82a
S2-02 CINTERPRET e8279a1c-5a00-4fb7-9315-c6973565f4be
  context_artifact_sha256=095140e4e690632a288762cce00dc9f8fa6760a12d02eb179190c14d7aaeb964
  output_artifact_sha256=18a8fe1099cb434347b8b0d31d859a1b42adb0c1b249ce53e47b221b1b4b13f6
S2-02 CINTERPRET a9fa4a3a-84f2-4ce5-963a-1383a0ecaf55
  context_artifact_sha256=f82cad84fcc56e7294bf3abf578241fc25544ab596060126d37336ba6db7e2f2
  output_artifact_sha256=ea6ca42d92f4f29e138e95b7a33a461f0a87ff45e80fc8ce41ff70cf16143d3e
S2-02 CINTERPRET 97d1b20e-32ed-4a93-ae24-4dc786b3fdd5
  context_artifact_sha256=e31fa0af48440c0954abeb661ff2910a33ea81e6d0008e14c5fa4d007e22fc3f
  output_artifact_sha256=39c3fbe85c5e8e08e0c4f9dad4a3ee2a0d3937cb94b5842b803d8abe83e04f23
S2-02 CRESPONSE 9b63ecb0-72e4-42f9-a364-0aec53580805
  context_artifact_sha256=bcc524c2ad8c1e44ba4fcf7ae8d467ebf34d864e7134d7d824f18929052bc161
  output_artifact_sha256=850ec8804836e9b0b222a42ae4450218e0688f3ef1eca7a8f09921fa51f04a31
S2-02 CRESPONSE 1f22f3e8-0ce3-4baf-8e42-88837c605fa1
  context_artifact_sha256=358d0dc6e6617c543e1001084fefadbd2d566cb0281fa49e71e9c4010c6f564d
  output_artifact_sha256=00544a598b5bb816b9cfdddbda5d4fa71319d91b8b037547e953338c83814613
S2-05 CINTERPRET 1ce78ffa-55aa-4091-973e-54e053bb0833
  context_artifact_sha256=3cebe4b08e083c1da7ef7c02af59781fcfbb7a4c86f8113200b4daacda6f80d3
  output_artifact_sha256=97daecb55749e5d37c35ebf678c1967a1989930d5bf3a2015288beedae9a8db6
S2-05 CINTERPRET 54b2a364-4beb-4f0d-891d-168379df9c4a
  context_artifact_sha256=6f9e8bfb6d7625cb6e8e0436c3d26a8c3f573589fa157ea9dc00d8b123e2d974
  output_artifact_sha256=637f7819ba245199c42d08cfb07bf03acb090a6ccb671d999d38ae24c285f71f
S2-05 CINTERPRET beb00c90-75e1-4102-ae63-3c934c83b314
  context_artifact_sha256=6cfab764c48de3d483314b0e237b3f0dd4a4f50e3b96ea0d07f103886b0689c7
  output_artifact_sha256=72611976ac4a36fee41e75eaffc522ddd17f3936245f1b83dc13f443bf51180a
S2-05 CRESPONSE c55d751e-ff34-4d47-b795-2ccb6faaaea0
  context_artifact_sha256=5386fc5b15b8113725a0738f7ec4af430cfb0aa30ae3aef68aabe01453477c41
  output_artifact_sha256=0f9984bd90e7668e00da066390e2239c539fe88fc067151e7f5a44a6b62f8622
```

이 두 단일 반복 사례로 A/B 우열, 구조 고유 의미 오류율, 최종 QA 점수 또는 생산 제품 수준의 성능을 주장할 수 없다. 여기서 확인한 것은 실패/전달 부족까지 보존된 구체적인 실행 인과다. 이후 같은 frozen 계약의 B1/반복과 실제 판정을 대조할 때 모델의 내용 선택, 공통 retrieval 관련성, 스키마 오류 후 추가 호출, 실제 Task/효과, 부분 Voice 전달을 각각 유지해야 한다. 점수 판정과 인간 분쟁 검토는 별도 후처리 작업이다.
