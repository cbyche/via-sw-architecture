# v3 지표 산출 계약 독립 리뷰

상태: `READ_ONLY_SOURCE_REVIEW`. 리뷰어: `poc_dp41_contract`. 기록일: 2026-10-12.

이 리뷰에서 읽은 범위에는 QA 모집단, cap 계산, 실제 핵심 응답 onset, 외부 시간 귀속, 금지 효과 판정 및 source provenance를 훼손하는 확정 P1/P2를 추가로 발견하지 않았다. 이는 소스와 일부 raw 자료를 검토한 결론이다. 최종 210개 trial의 의미 판정이나 여섯 QA의 최종 측정값을 검증했다는 뜻은 아니다.

## 1. 고정된 검토 대상

캠페인: `results/architecture-evaluation/current/dp41-p21-mac-text-v3-20261012/`.

원본 [manifest.json](../manifest.json)의 `frozen_source_digest`와 [evaluation-tool-manifest.json](../evaluation-tool-manifest.json)의 `baseline_source_digest`, `source_digest`는 모두 다음 값이다.

```text
f521d6bea196342dab6c0f66268134e9c33f25dfa86a0aac874afc160cb6a20a
```

manifest의 동결 소스 목록은 49개 파일이다. 아래 일곱 검토 파일은 저장 시점에 `frozen-source/`와 `evaluator-tools/`의 바이트가 같은지 읽기 전용으로 재확인했다. 이 문서는 동결된 49개 파일 밖의 리뷰 산출물이며 후보나 평가 도구를 수정하지 않았다.

| 검토 파일, 동결 소스 기준 | SHA-256 |
| --- | --- |
| [finalize.py](../frozen-source/benchmark/architecture/dp41_45_poc/finalize.py) | `7d2420e0142bb199cf5b56cd5f61bba12584ae198a7937a0e227fa3b04273296` |
| [normalize.py](../frozen-source/benchmark/architecture/dp41_45_poc/normalize.py) | `e172fcfafea6700cfd4e9e774b5f0289de51dc3df28965e6f8f1346d214fdb83` |
| [metrics.py](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py) | `fe7aadac13ab74e36222eae951c9d0199c9e9b61a13229bf9a2cb4893a11ccc1` |
| [judge.py](../frozen-source/benchmark/architecture/dp41_45_poc/judge.py) | `e96fee7a06176fdbc53fef1c7e9a3cfbfe8cb4485382432e5e6d152f89902119` |
| [evaluate.py](../frozen-source/benchmark/architecture/dp41_45_poc/evaluate.py) | `6985590abae400b8a251762143d863f67875849f61fab39061a5c64489fa3bb1` |
| [worker.py](../frozen-source/benchmark/architecture/dp41_45_poc/worker.py) | `f144078d212a159e3a3827266bc74aa54fcfef2b2fd126e635efec9ea0ad4dfa` |
| [shared/timing.py](../frozen-source/prototypes/candidates/dp41_45_poc/shared/timing.py) | `6f5275313103926dbbc701bef691f5fc5fb649c48dca2e98aa41b97371b59356` |

판정 의미의 대조 기준은 동결된 [ASR-QA-v6 정의](../frozen-source/docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 P21, QA01~06, 시간 경계 및 합성 tail 계약이다. 아래 링크의 줄 번호는 위 해시의 동결 파일에 대한 것이다.

## 2. 실제로 읽은 범위

소스에서는 `finalize.py` L73~119, `normalize.py` L14~91, `metrics.py`의 계약 검증, 모집단 및 여섯 QA 집계, `judge.py`의 판정 프롬프트와 evidence view 및 결과 검증, `evaluate.py`의 calibration 및 불일치 조정, `timing.py`의 DAG 귀속 계산을 읽었다. `worker.py`에서는 실제 입력과 source availability 기록, trial 저장 및 causal timeline 저장 경로를 읽었다.

실제 raw에서는 [A1/trials-unjudged.jsonl](../workers/A1/trials-unjudged.jsonl)의 21개 trial 식별자, 실제 입력 기록, 반응 및 사이클 blueprint, effect operation 목록을 확인했다. 특히 S5-03 재접속의 `FOLLOWUP_SPEECH_END`와 root 발화 후 추가 질문이 발생한 trial의 `observed_inputs` 순서를 확인했다. 이 확인은 각 trial의 C1~C6를 의미적으로 전수 판정한 것이 아니다.

이 리뷰에서는 API 호출, 후보 실행, 테스트 실행, normalization 또는 finalization 실행을 하지 않았다. 다른 작업자가 수행한 테스트나 실행의 PASS를 이 리뷰의 실측 증거로 사용하지 않았다.

## 3. 모집단과 실패 보존

[`metrics.py` L303~334](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L303-L334)는 trial의 variant, repeat, case 식별자 중복을 거절하며, 예정된 실패도 condition/reaction/cycle 분모에 남기는 계약을 가진다. [`metrics.py` L341~410](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L341-L410)는 시나리오별 C1~C6와 예정된 반응 및 사이클 ID를 검사한다. `strict_coverage`에서 누락이나 중복은 집계 오류이고, 일부 대표 사례만으로 최종 점수를 내지 않는다.

한 방안의 전체 모집단은 P21 21개 시나리오를 5회 수행한 105개 trial, 정확성 630조건, 반응 135관측, 사이클 95관측이다. [`finalize.py` L115~118](../frozen-source/benchmark/architecture/dp41_45_poc/finalize.py#L115-L118)는 A/B 합계 210개 trial, 1,260조건, 270반응, 190사이클 및 QA05 12개 job의 수가 맞는지 확인한 뒤 `completion.json`을 저장한다. [`finalize.py` L83~100](../frozen-source/benchmark/architecture/dp41_45_poc/finalize.py#L83-L100)는 실제 worker raw를 정규화하고 numeric completion 및 여섯 점수 누락을 거절한다.

따라서 후보의 의미 오류나 timeout은 원래 예정된 관측을 제거하는 이유가 아니다. 이 리뷰는 최종 파일에서 위 수가 실제로 모두 충족됐음을 확인한 것은 아니다.

## 4. QA02 및 QA03의 시간 계산

[`metrics.py` L84~109](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L84-L109)와 [L424~450](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L424-L450)는 QA02에 세 평균을 별도로 둔다.

- `cap_mean`: 처음 핵심 반응이 잘못됐거나 유효 반응이 없으면 5초를 넣는다. 유효한 반응 자체가 6초 걸렸으면 6초를 유지하며 5초로 잘라서 개선하지 않는다.
- `correct_only_mean`: 처음 반응이 정확하고 유효한 관측만 포함한다.
- `actual_including_wrong_mean`: 잘못된 반응도 실제 onset이 있으면 그 실측 시간을 포함한다. 실제 onset이 없는 관측은 이 진단 평균에 넣지 않으며, 그 누락 수는 별도로 보고한다.

오답 후 정정된 시간은 `correction_seconds`로 남는다. 오답이나 유효 endpoint 누락은 `missed_endpoint_zero` 보호 조건을 통과시키지 않는다. 잘못된 빠른 반응이 실제 시간 진단 평균을 낮출 수는 있지만, 그 값으로 QA02 점수를 산출하지 않는다.

[`metrics.py` L112~127](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L112-L127)과 [L451~462](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L451-L462)는 QA03에 다음을 둔다.

- `cap_mean`: VIA 귀속 시간이 30초 안에 완료되면 그 시간을 넣고, 미완료 또는 30초 초과 완료는 30초를 넣는다.
- `completed_only_mean`: 실제 유효 완료가 있는 관측만 포함하며, 30초 초과 완료의 실제 귀속 시간도 보존한다.
- 완료, 늦은 완료, 미완료 수와 wall-clock 시간을 별도 보존한다. 늦은 완료도 endpoint 보호 조건을 통과시키지 않는다.

이 리뷰에서 확인한 것은 계산식과 raw 연결 경로다. 아직 최종 관측 전체의 시간값을 재계산하거나 평균을 검산하지 않았다.

## 5. 실제 핵심 반응과 causal DAG 연결

[`normalize.py` L14~30](../frozen-source/benchmark/architecture/dp41_45_poc/normalize.py#L14-L30)는 발화, source event 및 후속 발화 시작 경계를 실제 기록에서 찾는다. A1의 S5-03에는 root 발화가 없고 재접속 후 실제 발화가 `observed_inputs[0]`에 있으므로 `FOLLOWUP_SPEECH_END`의 참조가 해당 관측과 맞았다. A1의 S2-02, S2-03, S6-01처럼 root 발화 뒤 추가 답이 있는 경우에는 root 기록이 첫 번째로 유지되어 기존 `SPEECH_END` 경계를 후속 답 시각으로 바꾸지 않았다. source event 경계는 예정 offset 대신 같은 event ID의 실제 availability를 찾는다.

[`judge.py` L121~135](../frozen-source/benchmark/architecture/dp41_45_poc/judge.py#L121-L135)는 핵심 문구가 실제 제시된 Text 및 Voice에 존재하는지 검증한다. [`normalize.py` L33~52](../frozen-source/benchmark/architecture/dp41_45_poc/normalize.py#L33-L52)는 Voice ON에서 그 정확한 문구의 시작 문자와 실제 `voice_char_events`의 onset 및 causal node를 연결한다. 처음 인사의 onset을 자동으로 핵심 onset으로 사용하지 않는다. Text 시각과 핵심 음성 문자 시각 중 늦은 값을 endpoint로 사용하며, Voice OFF는 실제 Text node를 사용한다. 로컬 중단은 [L57~62](../frozen-source/benchmark/architecture/dp41_45_poc/normalize.py#L57-L62)의 실제 `LOCAL_STOP` 기록에서 계산한다.

[`normalize.py` L70~87](../frozen-source/benchmark/architecture/dp41_45_poc/normalize.py#L70-L87)는 실제 endpoint node와 공통 절대 좌표의 scene origin을 동일 DAG에 전달한다. [`timing.py` L16](../frozen-source/prototypes/candidates/dp41_45_poc/shared/timing.py#L16)은 제외할 시간을 `EXTERNAL_AGENT_WORK`, `EXTERNAL_USER_WAIT`, `EXTERNAL_USER_SPEECH`로 한정한다. [L80~142](../frozen-source/prototypes/candidates/dp41_45_poc/shared/timing.py#L80-L142)는 실제 DAG의 wall-clock 재구성을 검산하고, 해당 외부 duration만 0으로 만들어 재계산한다. 모델 대기, 코드 gap, 출력 queue 등을 외부 시간처럼 일괄 차감하는 계산은 아니다.

음성 문자 시각은 공통 모사 backend의 실제 emission 기록이다. 음성 API의 실제 발음이나 오디오 파형을 측정했다는 결론은 이 데이터로 낼 수 없다. exact quote 검증 역시 문구가 실제 전달됐음을 검증할 뿐, 자유문의 의미 판정이 항상 맞았음을 증명하지 않는다.

## 6. 정확성, 금지 효과 및 source 근거

[`judge.py` L12~54](../frozen-source/benchmark/architecture/dp41_45_poc/judge.py#L12-L54)의 판정 지침은 C1~C6를 실제 evidence로 독립 판단하고, 단순 접수를 완료로 해석하거나 후보의 잘못된 의미를 정답으로 보정하지 않도록 한다. 동적 Task와 artifact ID는 실제 producer, source, revision 관계로 비교하고, 내부 거절 시도와 외부에 실제 발생한 효과를 구분한다.

[`judge.py` L57~79](../frozen-source/benchmark/architecture/dp41_45_poc/judge.py#L57-L79)의 evaluator view는 후보 제어 경로를 가리는 한편 실제 초기 owner 상태, 실제 사용자 입력 및 후속 답, 채택 의미, 효과, 전달 기록, 최종 owner 상태를 제공한다. oracle는 이 평가 view에 들어간다. 이 경로가 oracle를 후보 model context로 주입하는 경로는 아니다. [`worker.py` L266~289](../frozen-source/benchmark/architecture/dp41_45_poc/worker.py#L266-L289)는 실제 초기 상태와 fixture seed를 구분하고 실제 입력, source availability, effects 및 DAG를 trial에 보존한다.

[`judge.py` L82~136](../frozen-source/benchmark/architecture/dp41_45_poc/judge.py#L82-L136)는 C1~C6와 reaction ID의 완전성, boolean 판정, 알려진 실제 evidence ID, PASS의 근거, 금지 효과 종류 및 실제 publication/핵심 문구 참조를 검사한다. [`evaluate.py` L20~69](../frozen-source/benchmark/architecture/dp41_45_poc/evaluate.py#L20-L69)는 두 판정의 불일치에 추가 판정을 요청하고 원판정들을 보존한다. [L74~115](../frozen-source/benchmark/architecture/dp41_45_poc/evaluate.py#L74-L115)는 calibration 조건과 실제 trial 판정 진행 및 retry 기록을 가진다.

이 리뷰는 위 정적 경계와 A1 raw 일부를 확인했다. 최종 판정에서 모든 source lineage와 금지 효과를 올바르게 판별했는지 전수 검토한 것은 아니다. 다수결 LLM 판정은 독립 인간 심사와 같다고 주장하지 않는다.

## 7. QA04~06의 확인 범위

[`metrics.py` L208~268](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L208-L268)의 비용 집계와 [L479~487](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L479-L487)의 점수 경로는 실제 provider usage/cost와 음성 참조 추정을 구분한다. [`finalize.py` L90~92](../frozen-source/benchmark/architecture/dp41_45_poc/finalize.py#L90-L92)는 worker 호출과 evaluator 호출의 원장을 읽는다. 이 리뷰는 실제 모든 호출의 usage 완전성, 계정 청구 금액 또는 provider invoice를 대조하지 않았다.

[`metrics.py` L271~300](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L271-L300)의 QA05 준비 판정은 실제 completed job, baseline hash, patch 및 result 참조, regression 결과와 독립 리뷰를 요구한다. 파일 수를 QA05 값으로 사용하지 않는다. 아직 12개 실제 변경 diff와 U01~U18 책임 단위 분류를 독립 검토하지 않았으며, 이 리뷰로 QA05 완료를 승인하지 않는다.

[`metrics.py` L130~192](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L130-L192)의 메모리 집계는 같은 sample에서 실제 대상 PID들의 값을 합산하고, 누락 또는 OS 수집 실패를 완전한 sample로 취급하지 않는다. [L488~510](../frozen-source/benchmark/architecture/dp41_45_poc/metrics.py#L488-L510)는 Mac `phys_footprint`를 주 지표로 사용하며 `resident_size_rss`와 repeat coverage도 보존한다. `windows_commit_equivalence`는 false다. 실제 다섯 반복과 다섯 activity 전체의 메모리 raw 완전성은 이 리뷰에서 확인하지 않았다. Mac 측정치를 Windows 실측으로 표현할 근거는 없다.

## 8. 후속 검토 대상

최종 산출 후에는 전체 판정과 raw endpoint의 연결, 실패 및 cap 관측 수, 210개 trial 모집단, 호출 원장의 usage 완전성, 메모리 activity coverage를 실제 산출물로 확인해야 한다. QA05는 별도로 실제 12개 변경 사본의 diff, 실행 근거 및 U01~U18 분류를 리뷰한다. 이 문서는 그 후속 확인을 수행했다는 기록을 대신하지 않는다.
