# DP41 B의 실제 지원 범위: 조회 후 기준 값과 전달 범위

상태: **READ_ONLY_SCOPE_AUDIT / NO_QA_VERDICT**. 공식 v3 frozen source `f521d6bea196342dab6c0f66268134e9c33f25dfa86a0aac874afc160cb6a20a`와 실제 `(B, 1, S2-02)`, `(B, 1, S2-05)`의 작은 raw 행을 읽었다. API, 후보 실행, 테스트, 재채점과 후보/fixture/정답 변경을 수행하지 않았다.

**판단:** 최초 finite Frame이 새 제안서 Goal과 세 근거 참조를 표현할 수 있다는 사실과, 조회 후 실제로 들은 기준 범위가 실행 payload에 보존된다는 사실은 다르다. 현재 B는 일반적인 Goal/조건 값의 조회 후 부분 추출을 지원하지 않는다. S2-02의 E1 조회는 actual heard scope를 보존하지만 binding 뒤 공통 source 조합은 그 scope를 보존하지 않는다. S2-05는 E새 원문과 heard text가 같아 이 범위 손실로 내용이 달라지지 않는다. 두 실제 B1 trial에는 제안서 command가 없으므로 대체 경로를 실제 성공으로 주장하지 않는다.

## 구현된 경계

| 단계 | 실제 지원 | 실제 한계 |
| --- | --- | --- |
| 최초 B_FRAME | Goal/operation/instruction/유한 조건과 unresolved reference 생산 | owner read 전에는 source body와 실제 heard 문장이 없음 |
| owner read | PAST_DELIVERED/RECENT_INTERACTION이 전체 Text와 actual delivered_voice_text/receipt를 분리해 반환 | source/interaction이 같은 E1@v1 identity를 사용하며 참조에 전달 facet이 없음 |
| B_PARTIAL_SELECT | 단일 제공 reference 선택, 불명확하면 unresolved | `{field_ref,candidate_ref,evidence_refs,uncertain}`만 반환. Goal/instruction/조건 값 변경 불가 |
| _build_meaning | 선언된 target/source/question/recipient/body/preservation/WHEN reference 위치를 코드가 binding | instruction과 일반 domain 값은 최초 Frame에서 복사. read에서 기준 값/범위를 자동 추출하지 않음 |
| 확인 질문 뒤 B_FRAME | 이전 Frame, 실제 read_evidence, 실제 답변으로 새 unbound Frame 생산 | 추가 질문/호출 필요. read 직후 일반 partial Goal/value call이 있는 것은 아님 |
| source_payload | 권한 검사 뒤 actual owner source 조합 | read content를 조합하지 않고 lookup(ref)로 재조회. 같은 ID/revision에서는 source가 interaction보다 먼저 선택됨 |

근거: [b.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py)의 `_build_meaning`(77~115행), candidate selection(173~204행), clarification 뒤 `read_evidence`(233~241행); [prompts.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py)의 B_SYSTEM/SELECT_SYSTEM; [owners.py](../frozen-source/prototypes/candidates/dp41_45_poc/shared/owners.py)의 lookup(109~125행)와 전달 content(249~253행); [runtime.py](../frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py)의 source_payload(535~540행), PUBLICATION_ORIGINAL/DELIVERY_RECEIPT(1058~1069행).

## S2-02: read에는 heard scope가 있고 Agent source에는 없다

[현재 명세](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md#s2-02)는 E1의 마지막 보안 등급 문장이 재생되지 않았다는 조건을 둔다. 실제 준비 전후 owner 상태와 read는 이에 맞다.

- tuple `(B, 1, S2-02)`; Request `R:589156a6-6909-4509-a8fe-66feeda8f498`; root input `I:7ec8634e-ac62-4280-b444-8fd621c9cd42`.
- E2776, scene 6.823834초: actual RECENT_INTERACTION read `15cb963c-10d6-48ca-9590-f1a0356e03d5`가 E1@v1을 반환했다. PAST_DELIVERED도 같은 interaction pool과 전달 content 변환을 쓰지만 이 trial에서는 호출되지 않았다.
- E2822/E2823: partial이 E1@v1을 uncertain=false로 선택하고 코드가 BOUND로 기록했다. 이는 history identity 선택의 증거이며 기준 값 추출 증거는 아니다.

| actual record | 실제 내용 |
| --- | --- |
| E2776 content.text | 평가 기준은 연간 유지비와 설치 기간입니다. 가격은 평가에서 제외합니다. 보안 등급도 최우선 기준으로 추가할 수 있습니다. |
| E2776 content.delivered_voice_text | 평가 기준은 연간 유지비와 설치 기간입니다. 가격은 평가에서 제외합니다. |
| actual initial E1 source | 전체 Text, revision=1, read/model/agent=ALLOWED, metadata={}. delivered_voice_text 없음 |
| actual initial E1 interaction | 실제 첫 두 문장, interrupted=true, receipt=true |

**정적 조합 경로의 제한:** E1 binding을 채택하면 source_refs는 `E1@v1` 문자열이 된다. 실제 owner 상태에서 source와 interaction의 ID/revision이 같아 lookup은 source-first 동률 규칙으로 E1 original source를 반환한다. source_payload는 이 object를 deep-copy하므로 whole Text만 주고 actual heard 문장을 주지 않는다. 실제 B1은 command를 보내지 않았으므로 이 가상 payload를 raw evidence로 생성하지 않았다.

최초 instruction에 “들은 기준을 사용”이라고 적고 E1 source만 선택하는 경로는 현재 Agent payload에서 heard 범위를 확인할 수 없다. reference identity만으로 보안 등급 문장을 배제했음을 보증하지 못한다. 이는 **이 PoC의 partial-value 지원 범위와 공통 delivery-scoped source 계약의 제한**이다. B 일반 구조의 필연적 불능으로 일반화하지 않는다. 같은 source 조합은 A에도 적용된다. A는 read 뒤 모델이 조건 값을 새 instruction에 명시할 수 있지만 그 사실만으로 Agent stub의 도메인 제안서 품질이 입증되지는 않는다.

**현재 지원하는 다른 경로:** 실제 clarification 뒤 새 Frame에서는 read body를 볼 수 있다. 두 번째 actual Context `a052615b-7bb3-4035-90d5-6b424aec904e` 마지막 user message의 read_evidence에는 E1 whole Text와 heard 첫 두 문장이 그대로 있었다. 새 Frame에서 기준 값/제외를 domain instruction에 명시하고 reference를 unbound로 유지하는 것은 정적으로 허용된다. 최종 bound Meaning을 모델이 만드는 hybrid와 다르지만 추가 질문/호출이 필요하다. 실제 두 번째 output은 잘못된 Task refs/instruction을 유지하고 input_ref를 답변 input ID로 바꿔 E3443에서 거절됐다. 이 대체 경로의 성공 증거는 없다.

Actual primary failure는 잘못된 THT193/THT194 선택/NO_MATCH와 뒤이은 input_ref 오류다. 잠재적인 heard-scope 손실을 관측된 이 failure의 직접 원인으로 바꾸지 않는다. 해당 오류를 고쳐도 최초 Frame + E1 identity만으로 모든 조건이 충족된다고 단정할 수 없다는 별도 한계다.

## S2-05: 원문과 heard 범위가 같아 표현 경로는 있으나 미실행

- tuple `(B, 1, S2-05)`; Request `R:dcc13b20-c481-4134-872d-7ee417f7e43f`; root input `I:eacc5691-1309-4c06-8990-cd82078a16da`.
- 실제 availability E5140, scene 2.971379초: E새 source와 historical publication 모두 “납기 2주 이내가 우선이고 가격은 평가에서 제외”다. revision=1, 세 권한 ALLOWED. 이는 실제 외부 사건이며 미래/정답 선주입이 아니다.
- B1은 E새를 PAST_DELIVERED/RECENT_INTERACTION으로 조회하지 않았다. r-baseline을 RECENT_TASK로 조회해 HT001~HT010을 받았고 partial은 세 번 uncertain=true였다. E6493 UNRESOLVED_FIELDS, command/effects 없음.

이 사례에는 미재생 추가 문장이 없다. **정적 표현 경로**로 최초 Frame이 NEW_TASK/CREATE_PROPOSAL, E새/D새비교/D새견적의 reference fields, “가격 평가 제외” instruction을 선언하고 actual owner read로 받은 reference를 코드가 결합해 세 actual source를 Agent에 줄 수 있다. 납기 2주 값은 actual E새 source에 있다. 이 domain 기준을 해석하는 것은 외부 Agent의 domain 작업이다. 숫자를 최초 Frame에 선주입하거나 partial이 Goal 값을 다시 만들어야 하는 사례는 아니다.

이는 operation/데이터 경로의 표현 가능성이며 actual B1 성공이나 stub의 제안서 판단 품질 증명이 아니다. actual B1은 잘못된 source pool과 현재 Goal의 future comparison을 기존 TASK_RESULT로 찾는 selector를 사용했다. 이를 “유한 Frame으로 새 제안서/납기 기준을 표현할 수 없다”로 일반화하지 않는다.

## 측정 해석과 후속 범위

이번 결과는 reference-selection 중심 partial을 가진 **구현된 B**의 비교다. “조회 후 부분 모델이 일반 Goal/조건 값을 갱신할 수 있는 B 전체”를 측정했다고 쓰지 않는다. S2-02의 actual delivery scope를 read에서 execution source로 보존하지 못하는 source-first 조합은 공통 P2 결함이며 actual Task/input_ref 의미 오류와 별개다. 지금 QA 판정을 바꾸거나 frozen source를 수정하지 않았다.

후속 계약에서는 delivery-scoped reference/facet을 실제 read content에서 검증해 물질화하거나, 선언된 condition/value slot만 갱신하는 bounded partial producer를 고려할 수 있다. 현재 구현되지 않은 별도 설계/측정 범위다. 정답 문장 무료 주입이나 마지막 whole Meaning 재생성으로 이번 B의 실패를 보정하지 않는다. Agent stub은 실제 받은 source의 텍스트를 결합하므로 proposal domain quality 자체도 입증하지 못한다.

## 근거 hash

[trials-unjudged.jsonl](../workers/B1/trials-unjudged.jsonl): S2-02 line 4 SHA-256 `85bbbec2f1228a2153268d3dcdac3a6293f6ed0f5dc9c60bdbe237d1dc494f3f`; S2-05 line 7 SHA-256 `b3561c1e0a4dba11f038d0b9914105f8ebf9db0537d2e03c191d1034085ce733`.

| selected actual object | canonical SHA-256 |
| --- | --- |
| E2776 E1 read item | `a6bfd07a88986a0bc2aa221828f1ec602d8ce5a6621a526ccba7cf7f18fad7dc` |
| S2-02 actual initial E1 source | `3e524100c3b6ff0bb8a5f41cd10bdfa0b6340603d99a94c8bd69101774ecec43` |
| E5140 E새 actual payload | `fd0fb91ac922cc1a2d6f85a896f8f18d3ae09f371968ad6b021ef29d7eb99c52` |

| actual model call | Context artifact hash | Output artifact hash |
| --- | --- | --- |
| `59089d54-9c5d-4378-a42d-075f3cba5b38` | [Context](../workers/B1/model-contexts/59089d54-9c5d-4378-a42d-075f3cba5b38.json) `9b8a6783e3ce83af283505228d465420749df30ff237b6b829aa602683485da4` | [Output](../workers/B1/model-outputs/59089d54-9c5d-4378-a42d-075f3cba5b38.json) `29899cc4c6ee45784834e7270f4ea19997553f7de48a5e19a2f04d3cc860e076` |
| `3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f` | [Context](../workers/B1/model-contexts/3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f.json) `7f836cac4c5e24b22cdc9a0b561b0b64f496e1b4441e745b11a39abaebdc6eee` | [Output](../workers/B1/model-outputs/3c0f7b3e-7a3b-48d3-baf8-d74d66d2398f.json) `0ea74f9d76736a3328f53ccca0e0a416bd8ecd54177d61dc8699a8b92d3c4024` |
| `a052615b-7bb3-4035-90d5-6b424aec904e` | [Context](../workers/B1/model-contexts/a052615b-7bb3-4035-90d5-6b424aec904e.json) `4eb78d50265cb032ab432c23fda69e74a5655423876a05c286e3ae5d656c1da2` | [Output](../workers/B1/model-outputs/a052615b-7bb3-4035-90d5-6b424aec904e.json) `65e201c077b3dae09fe00b4773db94349a85f13e5a53ccf7ec030b4fc56638e2` |
| `115d3965-d24b-4397-abdf-6215a64e8c07` | [Context](../workers/B1/model-contexts/115d3965-d24b-4397-abdf-6215a64e8c07.json) `f863cc41387f2b8a2ccda6665aac37ced6a76d7d479e960b791e0eed19e50e08` | [Output](../workers/B1/model-outputs/115d3965-d24b-4397-abdf-6215a64e8c07.json) `e74a09a30c2eb31df834f46623eb1c0ff5f310ab12f425fc65d8dd3ff525bf82` |

| frozen file | byte SHA-256 |
| --- | --- |
| [b.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/b.py) | `4637588c85e745c290334c3bf7471e752d2c1946bd2c1baee649c6f60b2b73e2` |
| [prompts.py](../frozen-source/prototypes/candidates/dp41_45_poc/dp41/prompts.py) | `be366f35926aebc73ba6ce9f1c9ba526366145518090b99d2aa28eee4ee8590c` |
| [owners.py](../frozen-source/prototypes/candidates/dp41_45_poc/shared/owners.py) | `268a172ddc85d6188bd00f89e14ae84726a3dfa96964d50b976cfd3e02e6adae` |
| [runtime.py](../frozen-source/prototypes/candidates/dp41_45_poc/shared/runtime.py) | `182e6f7537117a1216e9e50766b12e5a2cc30888632e1190c9489da7d5aecfe5` |
| [inputs.json](../frozen-source/benchmark/architecture/dp41_45_poc/fixtures/inputs.json) | `8024baf9ecefdc797bce532268ccb3f789fe788cdfaa418765dc730587728bd4` |

Object/artifact hash는 JSON `ensure_ascii=False,sort_keys=True,separators=(',',':')`, raw 행은 LF 포함 byte hash다. 8개 Context/Output artifact를 원장 hash와 대조했다. append 중인 전체 원장의 최종 hash를 주장하지 않는다. 상세 actual path는 [B1 관찰](./observed-resolution-paths-B1.md), 공통 실행의 성공한 부분과 한계는 [단순 조회/producer-consumer 관찰](./observed-simple-resolution-paths.md)을 따른다.
