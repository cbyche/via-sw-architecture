# V7 초기 실제 경로 독립 관찰

상태: READ_ONLY_ACTUAL_TRACE_REVIEW. 2026-10-12 진행 중인 formal A1에서 이미 완료된 S1-01/S1-02/S2-01/S2-02와 완료된 final source preflight S6-01 A/B를 읽었다. 후보/동결/정답 변경, API와 테스트 실행, CPU/메모리 측정은 하지 않았다. 아직 judge 판정이 없으므로 C1~C6 결과, QA 점수와 후보 우열은 확정하지 않는다. preflight는 formal 모집단에 포함되지 않는다.

V7 frozen digest는 `44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce`이다. Context/output hash는 실제 model-calls가 제공한 artifact digest를 아래에 그대로 참조한다. 이번 검수에서 추가 파일 hash 작업은 하지 않았다.

| 기록 | trial evidence | Request | 실제 상태 | 실제 호출/명령/전달 |
| --- | --- | --- | --- | --- |
| A1 S1-01 | E500 | `R:a57d490b-716d-4d4b-a22a-6190d295fc1e` | OBSERVED | 3/0/1 |
| A1 S1-02 | E1528 | `R:b316182d-922b-4991-8888-4e8c1f67e84a` | OBSERVED | 5/1/2 |
| A1 S2-01 | E3337 | `R:37c55c26-9e6f-4a3e-b2b3-c3f3584d5b30` | EPISODE_TIMEOUT | 6/0/3 |
| A1 S2-02 | E3995 | `R:ee237827-5ab5-4a3e-97b6-97af44007af7` | OBSERVED | 3/0/1 |
| v7-final-source-preflight-A S6-01 | E1673 | `R:724a83cd-c1e6-42fc-a3cb-aa9e40e8db33` | OBSERVED | 6/1/3 |
| v7-final-source-preflight-B S6-01 | E1020 | `R:f2290a99-1ab5-4667-9ecb-9858ffd86850` | OBSERVED | 3/0/2 |

## A1 S1-01

E100에서 D1@v1의 실제 마감 원문을 조회했다. 뒤 CINTERPRET가 goals=[]/response=null을 PROPOSE하여 NO_ACTION facts의 CRESPONSE와 실제 Voice/Text 전달로 이어졌다. 필요한 원문과 지원 연산 READ_FIELD가 있었지만 해당 Goal/응답을 만들지 않은 모델 의미 선택이다.

- call `b649d79f-2dc1-4b65-b1a6-16aaae8494e0`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/b649d79f-2dc1-4b65-b1a6-16aaae8494e0.json), digest `8de91d038741d647a4be088cb692404456065d546683ba098f768e6b2d75af33`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/b649d79f-2dc1-4b65-b1a6-16aaae8494e0.json), digest `a00171bee37f63022a697fe8de76581b963896df108955c2af9453affe31a5ca`.
- call `7227f088-efd2-45e2-be36-370fd9c0823a`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/7227f088-efd2-45e2-be36-370fd9c0823a.json), digest `eca0b5d642563e5ca3758233e7d553290190426626a539d59a44293ee72543f6`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/7227f088-efd2-45e2-be36-370fd9c0823a.json), digest `4d14485ef8b91130316030512c090e1d41dd761c574039bc8844ac8bacfc5207`.

실제 Voice/Text는 “해당 요청에 대한 실제 효과나 상태 변경은 없었으며, NO_ACTION으로 처리되었습니다.”였다. 원문 조회 실패로 분류하지 않는다.

## A1 S1-02

Sales2026Q3@v1 원문 조회 E615 후 모델이 meaning version2를 반환하여 E670에서 거절됐다. 다음 출력이 version1로 재판단되었다. 채택된 DIRECT_RESPONSE+CREATE_TABLE은 실제 source payload를 Agent에 위임했다. command의 실제 operation/kind와 보고서 요구의 충족 정도는 이후 조건별 판정 대상이다. 코드가 잘못된 version을 무료 수정하거나 정답 업무로 바꾸지 않았다.

- call `f4185e8b-f56b-4b88-8a31-8e171dbe213f`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/f4185e8b-f56b-4b88-8a31-8e171dbe213f.json), digest `bac7127b6ba6bdd99dd3217bcbf3bae51964a216db2e46335a0bfdfbce74b318`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/f4185e8b-f56b-4b88-8a31-8e171dbe213f.json), digest `2e71441e50af94da65e3abb385031c4528415103edee17d6dd956f278318c6d1`.
- call `cb0dc6c9-aba4-46db-b87f-a0d4c00d828f`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/cb0dc6c9-aba4-46db-b87f-a0d4c00d828f.json), digest `69c88af62dbfcaf6a422dc6409159357d21d3a80751b7033ae877415ffe2334f`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/cb0dc6c9-aba4-46db-b87f-a0d4c00d828f.json), digest `441cd7a0b5f8d670dbd5347ba57bfda11c82717fb8ceec30fdc2c3b79c9817e5`.
- call `268905d3-5431-4f37-b3e3-4c82ce820913`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/268905d3-5431-4f37-b3e3-4c82ce820913.json), digest `65b65b61f8ea59a33b93debc54c5c046e5299bbe3c5906ef16b7cd8655971721`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/268905d3-5431-4f37-b3e3-4c82ce820913.json), digest `b767a2307f8791a252ce47c61e81aa21558b7ce776503728dce5e4dbf065e25f`.

## A1 S2-01

INPUT_SELECTION E1630/E1663의 candidate_count는3인데 모델이 limit1로 두 번 조회하여 A@v1만 받았다. 모델은 다른 대상이 없다고 판단해 질문 두 번을 선택했다. 원래 없는 추가 조건을 주지 않는 neutral 답변 두 번이 실제 보완 입력으로 돌아왔다. 최종 단일 A COMPARE 및 기존 Task 목록을 받은 CRESPONSE가 무관한 상태 문장을 전달했다. 이후 E3335/E3336 automatic episode timeout이 기록됐다. 조회 coverage/limit 선택과 모델 재판단을 구분해야 하며 공통 조회가 발화 span의 의미 binding을 대신 보장하는 계약은 아니다.

- call `ee723622-247a-474d-80aa-97f26b25b3e4`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/ee723622-247a-474d-80aa-97f26b25b3e4.json), digest `5941faeeb255819184db4ded50f9be8afa58594d2b3fadb206e3fa7fbdb3e51d`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/ee723622-247a-474d-80aa-97f26b25b3e4.json), digest `fe0dd338757825c67500831711cf27fb59508f05535d87263f6fbb80684b8355`.
- call `3928d6f5-d62e-4dab-84d1-051485613c4a`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/3928d6f5-d62e-4dab-84d1-051485613c4a.json), digest `2dbb39706e54a1190110e317f0fa5f44e1d847a602a7f0a58525ab1699dacb48`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/3928d6f5-d62e-4dab-84d1-051485613c4a.json), digest `7446c98ce313449155ba0fd2c4d7724841925556a1fe1efad7fde6b44aafb410`.
- call `b525407c-ce07-4b96-a094-f93fdaaaa523`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/b525407c-ce07-4b96-a094-f93fdaaaa523.json), digest `1967b35ef17f9a469389ad06918d9f38876aa6eae220dea95dff330d0be99a62`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/b525407c-ce07-4b96-a094-f93fdaaaa523.json), digest `3a2e4743a41fcb5d63b72814adaa5eb97b30cadab40e644fc7626a0f4409a47a`.
- call `698e4350-29e6-40c9-b642-ae5272dd865c`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/698e4350-29e6-40c9-b642-ae5272dd865c.json), digest `6e2c8afafe93088b5dfe4dd22584a20423d89fc7bf6d3ff62caba59d4fcca4bf`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/698e4350-29e6-40c9-b642-ae5272dd865c.json), digest `6609c636775873ee0f726fba777b1e382ee3763e4923bdbfaf7cec14dcf68dfb`.
- call `af6fe1ed-2fd8-4da1-b7f9-045a2c3723fb`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/af6fe1ed-2fd8-4da1-b7f9-045a2c3723fb.json), digest `8469f4fd5189f2a916c597429de94e51e93b7802dae1d8881aa5469d33f19b70`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/af6fe1ed-2fd8-4da1-b7f9-045a2c3723fb.json), digest `faa1a9f95c3f30d5af05bf39c96e7154c705e94bd320a3685a70080b8a80ab6b`.

실제 duration은 70.056948583초이고 timeout 원장은 automatic=60.049804833초, manual input pause=8.006324542초를 구분한다. 초기 발화와 neutral 답변 음성/전사도 원장에 남았다. 이는 후보의 전체 처리 활동 timeout이며 QA03 cap30초의 계산값이나 정상 완료 endpoint가 아니다.

## A1 S2-02

TASK_RESULT의 두 lexical query가 limit1로 모두 D2@v2를 반환했고 모델은 B01/B02 배경 source도 조회했다(E3590/3594/3598/3602). 명시 task locator를 쓰지 않은 query 선택과 기존 lexical 검색의 relevance 한계를 구분한다. 모델은 실제 현재 질문이 아닌 Q-clarify-criteria scope를 CLARIFY에 지정했다. E3650의 no_answer/QUESTION_NOT_CURRENT 뒤 NO_CLARIFICATION_ANSWER와 NO_ACTION을 채택했다. 실제 과거 전달 evidence를 조회하거나 제안서 Goal을 보낸 경로는 발생하지 않았다.

- call `aa9b5193-3e2d-4745-8db1-a584fd7f044d`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/aa9b5193-3e2d-4745-8db1-a584fd7f044d.json), digest `7eed6c98331c15bd3097a9163141b4158023f45dba224fb15eebc68cbb57e148`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/aa9b5193-3e2d-4745-8db1-a584fd7f044d.json), digest `43a958c4253e39a94597761f9dc70d9a05de3463b3b5284984e1abd1f20f4d1f`.
- call `ea8296a1-54e2-42f6-ac40-d4a1ce6e7172`: [Context](../dp41-p21-mac-text-v7-20261012/workers/A1/model-contexts/ea8296a1-54e2-42f6-ac40-d4a1ce6e7172.json), digest `87dd32aeafb56d5575003fb0cc64a4c4eb2c64129d241ade743d0740f34b43b4`; [실제 output](../dp41-p21-mac-text-v7-20261012/workers/A1/model-outputs/ea8296a1-54e2-42f6-ac40-d4a1ce6e7172.json), digest `a78bd3b91313923d80eff72728e169b9242554dc4910f7d167d102a223c83845`.

## v7-final-source-preflight-A S6-01

실제 후보 질문 후 원래 예정한 수정 답변을 받았다(E804/805/807). 모델은 version2(E865)와 지원하지 않는 FORBID_ACTION MODIFY(E902)를 차례로 반환해 추가 의미 호출로 재판단했다. 마지막에는 T3분기 SET_PROPERTY FORMAT=TABLE과 T4분기 NO_NEW_COMMAND를 채택했다. 실제 SalesQ3 원천/기존 Task 상태를 포함한 payload와 접수/완료 전달이 보존됐다.

- call `8f845d39-8b9a-464a-89fa-762e1ea9b465`: [Context](v7-final-source-preflight-A/model-contexts/8f845d39-8b9a-464a-89fa-762e1ea9b465.json), digest `c8eba498bffbf215887ef7842542b6241169448be5b383f734732f6d8f78ee00`; [실제 output](v7-final-source-preflight-A/model-outputs/8f845d39-8b9a-464a-89fa-762e1ea9b465.json), digest `2ada6cef5102f14daca3720a0fbf2221bd07a18ea381da9e6726d5094270f682`.
- call `917f4999-30d1-4577-9040-0d1f862afd41`: [Context](v7-final-source-preflight-A/model-contexts/917f4999-30d1-4577-9040-0d1f862afd41.json), digest `81b6c0703af71bde1ed4edd06f1f1bb7206e3c33c60526612251b7fa7d01a13a`; [실제 output](v7-final-source-preflight-A/model-outputs/917f4999-30d1-4577-9040-0d1f862afd41.json), digest `808eada3e6fa36f4ddf6f4f9549e54b455bdc7a26f7fb436a3ed3974791a1914`.
- call `fde328a3-6cf2-487c-9b22-a039595d1ecf`: [Context](v7-final-source-preflight-A/model-contexts/fde328a3-6cf2-487c-9b22-a039595d1ecf.json), digest `7b817928a595973b52c5f6133830e65b53de3f4c3e737ec43358cf6e781d0500`; [실제 output](v7-final-source-preflight-A/model-outputs/fde328a3-6cf2-487c-9b22-a039595d1ecf.json), digest `279470d58ee8a9baefc3ad731235569753741e5965cf0495533c201fca76532c`.
- call `3d95e6ef-b58f-4953-9d58-3a5fb5f74ba3`: [Context](v7-final-source-preflight-A/model-contexts/3d95e6ef-b58f-4953-9d58-3a5fb5f74ba3.json), digest `d6a1d3cb85683cb625407b193a83bc2e9f62c610ad00051dda007a2f744a44c3`; [실제 output](v7-final-source-preflight-A/model-outputs/3d95e6ef-b58f-4953-9d58-3a5fb5f74ba3.json), digest `311f5b6cf79dc74f0d9f1e15734ffadcc01e4d6fe5cf6877a53866c604f75901`.

## v7-final-source-preflight-B S6-01

첫 finite Frame은 후보 두 Task와 미결정 변경 항목을 만들고 code clarification을 선택했다. 실제 예정 답변 E564/565/567 뒤 두 번째 Frame이 현재 root input_id 대신 actual_clarification_answer.input_id를 frame.input_ref로 반환했다. 현재 input_ref 검사에서 ContractError가 발생하여 미결정 Meaning 및 해당 오류 전달로 이어졌고 명령은 없었다. 동일 Request 보완 중 모델이 답변 입력 ID를 고른 locator/contract 실패이며 미지원 SET_PROPERTY를 요구한 사례가 아니다.

- call `4622d4c6-418a-4c69-b9cb-ffe48467d93e`: [Context](v7-final-source-preflight-B/model-contexts/4622d4c6-418a-4c69-b9cb-ffe48467d93e.json), digest `a0cc394efb0f1920c22f32892763d3d482d6348a69a949f4515b5d1436e84ff7`; [실제 output](v7-final-source-preflight-B/model-outputs/4622d4c6-418a-4c69-b9cb-ffe48467d93e.json), digest `dda8fa2ed5f6d473259b0016fe85eafd55ff17e5d7e134e944379c68bb7d74d6`.
- call `ac68dfcc-e6cb-4f3a-98e7-1f0b895e4447`: [Context](v7-final-source-preflight-B/model-contexts/ac68dfcc-e6cb-4f3a-98e7-1f0b895e4447.json), digest `706626a176cb07fc0daac722037951a14663ed8a24849cf88f8ee443eea82de2`; [실제 output](v7-final-source-preflight-B/model-outputs/ac68dfcc-e6cb-4f3a-98e7-1f0b895e4447.json), digest `a34ef595569bce12a8b49997755b6d0496f757a37f6cc76e9ce36e41f0d3f70f`.

## S6-01 B 현재 Input 경계의 좁은 확인

두 번째 actual API Context `ac68dfcc-e6cb-4f3a-98e7-1f0b895e4447`의 첫 user는 현재 input_id=`I:011c4f96-a119-4c46-b5ec-c64e509f14bf`, Request=`R:f2290a99-1ab5-4667-9ecb-9858ffd86850`, revision1이다. 실제 SYSTEM은 frame.input_ref를 current input_id로 지정한다. 뒤 user의 actual_clarification_answer에만 `input:f609b979-52ea-4285-989b-7e5a33ee7dd7` 및 원래 예정된 답변 내용이 있다. 모델 output은 이 답변 ID를 input_ref로 사용했다.

Frozen B resolve는 초기 interpretation_input을 inp로 복사하고 CLARIFY 뒤 transcript만 추가하여 같은 Request를 보완한다. input_id는 바꾸지 않으며 현재 inp.input_id와 Frame input_ref를 비교한다. 공통 Ports.clarify는 CLARIFICATION mode로 동작하여 이 private 보완 입력으로 새 Request를 생성하지 않는다. 따라서 이 실제 trace는 새 Request의 현재 입력을 validator가 오래된 root와 비교한 공통 연결 오류가 아니다. 수정 값 자체는 SET_PROPERTY로 표현할 수 있었지만 해당 Frame이 그 binding 단계에 도달하지 않았다. 성공한 B 명령을 추정하여 기록하지 않는다.

## 판정과 원장 검수에 넘길 경계

이번 경로에서 신규 공통 계약 P1/P2는 확인하지 않았다. 모델의 미실행 Goal, version/enum 거절, 잘못된 locator와 미결정/timeout은 후보 기록으로 보존한다. bounded lexical 검색의 relevance와 후보 query/limit 선택을 분리한다. 해당 한계에서 DP41 A/B 구조 전체의 정확성 상한을 추정하지 않는다.

검토한 각 call에는 actual HTTP200 및 알려진 provider usage가 있었다. 원시 모델 출력, 실제 조회 receipt, adoption, command payload와 final delivery를 연결할 수 있다. TEXT_PRESENTED 당시의 미완료 snapshot과 final delivered_receipt/voice_text는 서로 다른 시점의 기록이므로 이후 judge는 final 실제 전달 범위를 사용해야 한다. preflight B의 짧은 오류 전달과 effect0을 정상 처리의 빠른 성공으로 해석하지 않는다. formal 이후 cap과 조건별 정확성은 고정 QA 정의로 계산해야 한다.
