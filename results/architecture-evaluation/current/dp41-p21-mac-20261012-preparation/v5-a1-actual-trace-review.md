# V5 A1 실제 trace 독립 검토

상태: `ACTUAL_A1_REVIEW_WITH_P2_REPLAY_DEFECT`.

A1의 21개 완료 raw scene을 작은 JSON row로 읽었으며, 아래 실제 경로와 endpoint 위험을 대조했다. QA 점수, 조건별 최종 정답 또는 A/B 우열은 판정하지 않았다. 모델 선택 오류와 schema 오류는 원시 실패로 보존한다. 테스트, API 호출, 후보 코드 수정, 재측정 또는 대규모 메모리 raw 읽기를 하지 않았다. B1 완료 후의 paired 검토는 별도다.

공식 [V5 manifest](../dp41-p21-mac-text-v5-20261012/manifest.json)의 digest는 `2143b39761bd0b74e8d9d973d8b59c1e8ba722ccc24ebafcc0d7e03d85cda4b0`다. 근거는 [A1 trials](../dp41-p21-mac-text-v5-20261012/workers/A1/trials-unjudged.jsonl), 각 Context/output 원장 및 V5 frozen code다. 이 문서의 실제 경로를 다른 source freeze의 성능 결과로 재사용하지 않는다.

## P2: 합법적 QUESTION_PRESENT 전달 뒤 지정 후속 입력이 누락됨

- 원문 [ASR-QA-v6 S6-02](../../../../docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md#s6-02)는 Q1@v2의 실제 재확인 제시 receipt 뒤 1초 대기와 3초 후속 동의 발화를 제공한다고 명시한다. fixture의 match는 Q1/revision2/INFORMATION_CONSENT이며 실제 presentation과 scope가 조건이다.
- 실제 A1/S6-02 request는 `R:dd47f30e-1209-4807-bea0-be8513adff10`이다. 후보는 `ANSWER_QUESTION / QUESTION_PRESENT`, `arguments.question_ref=Q1`을 채택했고 공통 runtime은 현재 Q1@v2를 조회해 전달했다. 최초 입력 `응.`을 Q1@v1 또는 Q2의 승인으로 적용하지 않았다.
- publication `P:1:44634b04-0df2-488b-96a8-1bf87a993b06`의 실제 whole Voice는 `민수의 연락처를 조회해 메일 업무에 제공할까요?`다. Text 시각 4.089117167초, Voice onset 4.340748375초, Voice end 8.676434417초이며 receipt=true다. `QUESTION_PRESENTED E15946`와 최종 Q1@v2 상태 모두 실제 PRESENTED, 정보 목적, T메일, 민수, READ_CONTACT/PROVIDE_TO_TASK scope를 확인한다.
- 그런데 `observed_inputs`에는 최초 입력 하나만 있다. 지정 후속 동의는 제공되지 않았고 실제 ANSWER/주소 제공 명령도 없다. C5/C6 또는 cycle의 부족을 전부 후보 미지원으로 돌릴 수 없다. 후속 입력을 제공했다면 모델이 올바르게 처리했을 것이라고도 주장하지 않는다.
- 원인 위치: V5 frozen `shared/runtime.py:753~757`의 QUESTION_PRESENT는 `response()`만 호출해 반환한다. `response():379~397`는 실제 deliver를 완료하고 반환한다. `AnswerSource`의 호출은 `clarify():335`에만 있어 CLARIFY로 질문을 제시할 때만 후속 입력 replay가 실행된다. `worker.py:54~70`의 실제 receipt와 정보 scope matcher에는 이 QUESTION_PRESENT 경로가 연결되지 않았다.
- `QUESTION_PRESENT` 자체는 선언된 operation이며 lookup 및 actual presentation은 정상 수행됐다. model의 임의 clarification 부족 또는 질문 display wording의 선호 문제가 아니라, 원문이 요구한 실제 presentation 이후의 공통 입력 공급이 실행 경로에 따라 달라진 문제다. 이 판단은 실제 row와 frozen 코드의 제어 흐름을 대조한 것이다.
- 필요한 보완 방향: 실제 유효 질문 전달의 공통 receipt를 중심으로 후속 입력을 한 번 제공해야 한다. 기존 CLARIFY 경로와 중복 소비하지 않고 Q/revision/scope 검사를 유지하며, 최초 `응.`을 새 동의로 재사용하지 않는다. candidate에 정답 동의 결과를 무료로 주거나 미발생 ANSWER 효과를 성공으로 보정해서는 안 된다. 구체 수정과 INVALID 처리 여부는 root의 독립 확인 대상이다.

## 실제 의미와 payload 경로

### S1-01: 실제 source를 읽었지만 잘못된 질문 binding으로 거절

`OWNER_READ E103`은 D1@v1의 실제 날짜와 담당자 원문을 반환했다. 모델의 두 번째 PROPOSE는 `ANSWER_QUESTION / ANSWER_QUESTION`과 `question_ref=I:6703f0fa-ae9e-49a6-afa2-e133301f7a23`를 만들었다. 이 ID는 현재 입력이며 실제 질문 ID가 아니다. `REQUEST_FAILED E166`은 `Binding reference was not provided`로 채택 전에 거절했다. adopted meaning, 효과와 delivery가 없다.

마감일 DIRECT_RESPONSE/READ_FIELD의 지원 경로가 있는데 모델이 다른 operation과 허용되지 않은 질문 참조를 골랐다는 실행 근거다. 날짜를 읽었다는 사실로 응답을 생성하거나 0초 endpoint를 만들면 안 된다. 이 경로의 미전달은 공통 retrieval 결함으로 관측되지 않았다.

### S2-02: 실제 heard scope를 읽고 instruction으로 운반

모델 READ는 실제 Task locator를 사용했다. `E2588`/read `fbb2bd0f-87aa-4656-8d8b-bc45976cf25d`는 T비교의 D2@v2, `E2592`/read `01d7a95f-9636-4160-aa24-52a292b85921`는 T견적의 D3@v1을 반환했다. `E2598`/read `d0e1b7b7-9a3f-41e5-9aa9-6c4550a8e75d`는 E1의 RECENT_INTERACTION opaque channel view와 background 후보를 반환했다.

E1의 실제 Voice는 `평가 기준은 연간 유지비와 설치 기간입니다. 가격은 평가에서 제외합니다.`이고 Text에는 미발화된 보안 등급 문장도 있다. 채택 instruction은 연간 유지비/설치기간과 가격 평가 제외를 포함하며 미발화 보안 등급을 추가하지 않았다. 실제 instruction은 이 근거를 전체 모델의 재해석 결과로 값에 옮긴 것이며 코드가 정답 기준을 채워 넣은 것이 아니다.

명령 `CMD:839feeae-9e19-4c29-be2a-58e169497b68`는 새 Task `T:6ad9a66a-c3b8-4a2c-b825-b20f9b5ec649`에 CREATE_PROPOSAL을 보냈다. Agent sources는 실제 D2/D3 원문 두 개이며 E1 opaque view는 sources에 넣지 않았지만 실제 instruction에 기준을 운반했다. model은 D3의 100/120백만원을 instruction에서 10M/12M으로 바꿨다. runtime은 이를 고치지 않았다. 실제 sources의 100/120백만원은 원문 그대로였다.

Text/Voice 두 publication은 실제 접수와 tail 완료이며 모두 whole Voice/receipt=true다. 이 사실만으로 모든 기준과 실제 payload가 정확했다고 판정하지 않는다. instruction의 수치 오류, 가격 비교 언급과 실제 source 원문 및 원문 요구의 의미를 judge가 구분해야 한다. domain stub의 실제 반환은 제공된 원문에서 만들었으며 모델이 만든 잘못된 instruction 숫자를 정답으로 고쳐준 모델 실행은 아니다.

### S2-05: 새 document는 읽었지만 이전에 들은 기준은 읽지 않음

모델은 `E5656`/read `3abfc19a-d167-4fd9-aeb3-d198e52441b2`로 D새비교@v1, `E5660`/read `61885e50-6d59-49ee-8942-34a6cd373807`로 D새견적@v1을 읽었다. 실제 E새의 전달 근거를 읽는 명령은 없다. 채택 instruction은 두 제품의 납기와 가격, 가격 평가 금지 및 납기 중심 비교를 포함하지만 원래 heard criteria의 2주 이내 우선 기준을 갖지 않는다.

`CMD:72dcf286-63a6-47c6-b88f-395d961a7f33`의 actual Agent payload는 두 새 document 원문과 해당 instruction이며 새로운 Task를 만들었다. 원래 E새를 조회하고 그 실제 delivered scope를 운반하는 지원 경로는 존재한다. 실제 후보가 그 조회와 의미를 생략한 관측을 A 구조의 필수 표현 불가능으로 일반화하지 않는다. 반대로 두 document가 맞는다는 사실로 누락된 세 번째 근거를 무료로 보완해서도 안 된다.

### S3-02: actual current artifact와 후속 소비

모델은 G_SUMMARY와 G_MAIL, `AFTER` 및 `body.result_of=G_SUMMARY`를 제안했다. 모델의 해석 조회를 따로 하지 않았어도 D1/민수의 제공된 metadata ID를 binding했고 실행 시 `SOURCE_COMPOSITION_READ` 3건이 실제 원문과 수신자 조회를 수행했다. 이 Agent 원문 공급을 모델이 이미 body 내용을 알고 있었다고 재라벨하지 않는다.

현재 D1@v1에서 SUMMARIZE 명령 `CMD:3d8048d3-6f26-4350-baa6-4e24c0543733`가 생성한 artifact는 `A:e117638d-8524-45d2-9993-74c13fc03715`다. 실제 text는 `하늘 프로젝트는 10월 15일까지 제출하며 담당자는 민수입니다.`이며 source D1@v1과 현재 Request/G_SUMMARY lineage를 가진다.

SEND_MAIL 명령 `CMD:23627778-9646-40d3-b151-d99309215db7`는 해당 artifact를 body_artifact로 소비하고 실제 recipient `minsu@example.invalid`을 넣었다. 결과 artifact `A:e8bbec78-5b1b-4340-8d85-bfa0c873f01e`의 외부 상태는 SENT이며 body_artifact_id가 현재 producer를 가리킨다. 이전 요약 결과 2건은 RESULT_REJECTED로 거절되고 현재 중복 결과 1건은 `DUPLICATE_IGNORED E7413`으로 버렸다. 문장의 '업무 완료'와 실제 SEND_MAIL receipt 및 수신자/본문 lineage를 같이 대조해야 한다. 이 관측을 Agent의 실제 메일 송신 또는 보고서 domain 품질 검증으로 확대하지 않는다.

## 경합, late return와 판정 경계

- S4-01은 실제 R옛 job이 생성됐다. `HELD_RETURN_RELEASED E9261` 뒤 모델의 3개 실제 반환이 있었고, 최종 old meaning은 `MEANING_DISCARDED E9389/SUPERSEDED_INPUT`으로 버렸다. 해당 branch는 실제 수행됐으며 이전 Request의 명령은 없다. 현재 모델은 UPDATE_TASK kind에 CREATE_TABLE operation을 선택했고 실제 실행은 CREATE_TABLE 신규 위임을 보냈다. 명세의 기존 보고서 FORMAT만 교체와 다르므로 이를 currentness 검사 성공만으로 올바른 변경으로 판정하면 안 된다. SET_PROPERTY 지원 경로와 모델의 operation 선택, 현재 공통 실행의 operation별 branch를 구분해야 한다.
- S4-03은 실제 모델 8회 모두 허용 schema의 operation을 완성하지 못했고 BudgetExceeded를 기록했다. S6-04도 operation을 별도 필드로 주지 않거나 version을 바꿔 8회 뒤 실패했다. 지원 enum/argument와 반복 출력 오류가 원시 근거다. `status=OBSERVED`는 scene 관측 종료이며 성공 판정이 아니다. 전달이 없는 장면을 분모에서 제외하거나 0초로 만들면 안 된다.
- S5-01은 report 질문의 철회 후 mail 질문만 whole Voice로 전달했다. 생성된 이전 질문 문자열이 delivery row에 남아 있어도 actual Text/Voice 표시와 receipt를 함께 확인해야 한다.
- S5-02는 서로 다른 Task의 유효 완료 2건, 중복 1건, 이전 revision 거절 1건이 있다. QA02 지정 endpoint는 latest report 결과이며 다른 Task의 더 빠른 완료를 대신 고르면 안 된다.
- S5-03은 Voice OFF Text 결과와 재연결 후 Voice 결과가 별개의 actual publication이다. I1은 source availability부터 Text, I2와 cycle은 재연결 발화 종료부터 기존 결과 안내다.
- S5-04의 6개 reaction은 각각 source/event와 목적을 갖는다. 안내가 순서대로 나왔다는 사실만으로 6개 endpoint 모두를 첫 publication에 결합하지 않는다. report 정정, 두 질문, 두 결과, local stop를 별도로 연결해야 한다. 모델의 후기 상태 요약에 이미 제시한 질문을 제시되지 않았다고 말하는 문장도 있으므로 final state와 잘못된 delivery를 따로 확인한다.

## 원시 tuple와 모델 근거

모델 Context는 실제 input/Request/revision/spans, 허용 metadata, current adopted meaning과 budget을 가진다. 확인한 초기 semantic Context에 case ID, oracle 또는 future event plan은 없다. owner read 뒤 돌아온 원문과 channel view는 실제 read receipt의 자료다. 아래 Context/output hash는 모델 원장의 canonical JSON hash이며 raw trial hash는 newline 포함 원시 row hash다.

| tuple | actual Request | raw trial SHA-256 |
| --- | --- | --- |
| A/1/S1-01 | `R:903d80df-ea0b-418f-9a8a-ce294696b76e` | `41d8f906f087ee8f70bca6ca01c9e4011b443ef6e5bc0df17163738724146821` |
| A/1/S2-02 | `R:79f98fe6-0e00-4a14-b22e-b5d91834e14f` | `5bdc0b3e51cbdb8137070824a1820bc4f35f2544d96ce1ed0797e3ce5b72efce` |
| A/1/S2-05 | `R:42051092-992e-422b-a172-3afb5aa949f6` | `94014c7ad59c4ebd1beb53b73b20ed15c96421a3a05268c30f15f30440ee0a2b` |
| A/1/S3-02 | `R:4b8d890c-be17-460b-9be8-e53b2c51c82c` | `ecb3c161f6b8b634a7251610e1eeecdecb71aec19a5630a50de7bc336278dadc` |
| A/1/S6-02 | `R:dd47f30e-1209-4807-bea0-be8513adff10` | `36845c9a8b73ca54648f7a8180c56134d7a8a67f23d1d50485612f543d9a59b7` |

| tuple | 역할 / call ID | Context SHA-256 | output SHA-256 |
| --- | --- | --- | --- |
| A/1/S1-01 | CINTERPRET / `4d9a6a00-6abf-434e-89c7-5c7d588677d3` | `7752c2922584c78c3c16006ccf63e2d8881da64b8e45f62c5a4a2c73bae535e4` | `1fa8c8f8c6805821c66f66f98f64afa5f13fc1ec211129f68be1417fb9eedfe0` |
| A/1/S1-01 | CINTERPRET / `1df7a191-dd97-463c-a9be-69b5bb3ff983` | `eec0cc1e7cea8a8db8f35512d2e911fd5b282828674ecb93ad5eee9c584ca904` | `3d32a76fd7239f0cbad49f58538a41282ebdbbc4617a156d098919e0bce4f1a4` |
| A/1/S2-02 | CINTERPRET / `a63f4388-3457-46b5-bd82-eab0f3527839` | `cb4586c9d82f2dcaeb5886d591c5f1c61388f21adab3f20acd66c8f8feb2ccf1` | `d6a1c1eaeee092ce4940b66c3c71029a0e6052b8b09451d424a0a6269674e351` |
| A/1/S2-02 | CINTERPRET / `be474d07-adc8-49be-bde3-e32a4b13c2fc` | `acf6d9b14acb6e2be598323c4fe17d97f558aeb0f1df806652325b344b2fc48c` | `3b7eb355e8c96511fc7f8a050d2865f20f926303f99f4bd0e9dc6758ac71a975` |
| A/1/S2-02 | CRESPONSE / `2b7ec416-286b-4ff1-8a88-5f3d69fe84cc` | `34fc2e0b421e13b0f5ee48903d9c8c0c4227c945d0226c4cedeef68d9d9af4ba` | `05301a74849d196f2f90981d5fabe5843194c2f694e768b92e4df62260e6016d` |
| A/1/S2-02 | CRESPONSE / `bb436104-9d4e-44a8-ad1c-f1136766c3ba` | `58cfa8afe80416601a6a41dc043efc24a6f2139e90884d4f5831971d1545b451` | `deb6c776c21196970088e2da635f8dbaaff4a3933b1db6d43d044a68d26810f3` |
| A/1/S2-05 | CINTERPRET / `e8494ca0-941a-4683-b78f-f880edc48c8b` | `9d5ffdb34c9c1e2256c35a96321f2cc26ec621de2820e8be63fa889d32dad060` | `0f18eafe3620b5d375e444a353aa23d5b4e554d3e222dae656d3aaa5ce29d32a` |
| A/1/S2-05 | CINTERPRET / `8e93e7d3-9108-4efe-b421-49f907cff91b` | `30ad994ace6ba86dbe10218b6bd279d168d214824d550fb33f2cd0c10eabf8ab` | `89c3fc9d9fad9ec845166eb986581404d20526b18c7bdcdfafe3c4ae3cd63e30` |
| A/1/S2-05 | CINTERPRET / `b2e2e6d6-046a-4a80-ad35-887c7493b0d9` | `520c41c270177cc5e8a14dba896707eb8ae48ab437c0ca59d0088c5057570035` | `17ad7a73467cebc95e6ecefe7c72d0233fed106610eb92e74e4c6dd849092502` |
| A/1/S2-05 | CINTERPRET / `dd9abc50-7abd-4bed-aabd-82a4fd7d7f7b` | `f50400aebfb06e4790b42a7906ca4ce4a5197896b5821a2820d75005904963e9` | `42e86505a43b35cc6424c63c1bd01b00a05a8387bdd9abfc20107250f8a675c2` |
| A/1/S2-05 | CRESPONSE / `284fe27f-a9e2-4e03-a31a-c3b10a8ca829` | `501dc25d0bbd44be146717d6734a3e4571e49ea109d1d96f3ab86a1d1362af5b` | `9b96b741f37711a09155f6f01ebbbe7ffd9f57a6777c268e48e4f301695d56e3` |
| A/1/S2-05 | CRESPONSE / `319bb7e1-a54d-4c56-a9dd-58dc2daab515` | `858403cc036f2794c885e58bad97a3d777a5e931d7aa97e52409062545f35035` | `8b9fdbbb687d0be9ef6637c24bc40b99d1ca46234af029850c996afc9dd26f1a` |
| A/1/S3-02 | CINTERPRET / `9c465a1a-a176-46a2-a0f8-1ff1765a4461` | `37a1558731a7041dfcdb0c087f855c990334c4fa75e0ae97cfb2a440fec88413` | `4eb3962e321254e3231318b5264ab8c17cd2cde64388ce14a729c352dceb4f81` |
| A/1/S3-02 | CRESPONSE / `e6419f6a-9a98-4168-bc58-ba2d7cc454e4` | `58b375c56e9fb68ab7156c27e9d2111a32fdd23aee562b2d03b34951ddaea4d1` | `e5b8d0fe2c6c589dc32e012268f23d5ed1dbae1b892feb6e8142d668d5c89126` |
| A/1/S3-02 | CRESPONSE / `6563b254-31fd-4fd1-919d-625dee159a87` | `dd3352caa1e52de5f3c1733656ebeb0aa433664370bd7c5dd5516d578f416888` | `da1c6386b7938a88ebbd071c6c6125ea4552ef4823e23a100d1a665439551808` |
| A/1/S3-02 | CRESPONSE / `01f0489a-0d0a-4c37-9d81-b0ea00c322f2` | `b83dd73753447e6afcd04c0de4ee136a144faa076e6c94a085040671d47769c4` | `fd839bd07f16cf3f1a63fc2e41ee810f9e101422c3f5a5922e8b21f2c383cc04` |
| A/1/S3-02 | CRESPONSE / `462a0b2e-41a5-4e4e-a2d2-e4656181c370` | `02f84894248398690745bbe92c6c474b47fcab8ed303a1c2e1443c6fbb15ef5e` | `a24e2fca2bf3467142ca56e0aa255c7772084ae8c480f823079323f11ad463d5` |
| A/1/S6-02 | CINTERPRET / `7ca8d47d-7ace-4be1-8a27-2bb2e5f28ba3` | `547206c47a845faec6461b7506bfe73d1bfe89ca5ddc4f7a5ce735e853112ab9` | `b4a11d8441541faba67f22bc463d59db242b1e2c34b8fc3c3b4621d0af7048b7` |
| A/1/S6-02 | CRESPONSE / `064e4dea-88e5-405e-adb3-523c29d3b027` | `e99db3b7495caae7802ea2fb52ef8b04a4c88aba9c3b5c66d78812f8c687fadd` | `2e9d9f4f269734a3df47e036380a2d97b625661895cc2f8a19896e64e985b73e` |

## 한계

이 검토는 완료된 실제 A1의 실행 원인과 시간 판정 연결을 위한 문서다. 최종 정확성 또는 5/30초 cap 판정은 아직 하지 않았다. S6-02 입력 재현 P2가 독립 확인되기 전에는 해당 후속 미처리 비용과 시간의 전체 원인을 후보에 귀속시키면 안 된다. 모델의 정상 오류, 공통 bounded lexical 검색 범위 및 유한 B의 partial Goal/value 지원 범위는 이 P2와 분리한다. source, oracle 또는 captured failure를 수정하지 않는다.
