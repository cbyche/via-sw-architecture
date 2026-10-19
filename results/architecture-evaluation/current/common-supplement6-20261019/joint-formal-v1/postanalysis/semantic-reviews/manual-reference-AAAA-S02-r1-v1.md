# Manual semantic reference: AAAA-S02-r1-fea606

Reviewer: GPT-6 Astra Ultra, `/root/method_astra_review`. Date: 2026-10-19.

## Scope

Exactly one closed COMMON9 run, six response obligations and six goal relations. Trace SHA256 `fa652e4b255c42f34136e2704b0039270c2eeaff694fab33d52a332afe64acd8`. This is independent manual raw-evidence review. The reporting helper and evaluator were not executed. No timing, endpoint, denominator, frozen file or running experiment changed.

## Conclusion

All six displayed answers satisfy their actual spoken requests and match the supplied source facts. The six NEW/DIRECT goal associations with revision1 and task_id=null are supported by adoption, receipt, delivery-state and final request records. Authorized reads are supported. No approval decision or actual Agent/handoff occurs; those dimensions are not applicable and receive no PASS.

RU02 and RU05 contain additional oracle wording that is not required by the actual input or supplied common output prompt. Preserve literal nonmatch separately as `ORACLE_INPUT_MISMATCH` with additional requirement `UNRESOLVED`. Do not label these omissions model semantic FAIL.

| Response / goal | Actual display event (index) | Actual-request body | Goal association | Oracle alignment |
| --- | --- | --- | --- | --- |
| S02-RU01 / G01 | event-37 (31) | PASS | PASS | MATCH |
| S02-RU02 / G02 | event-79 (68) | PASS | PASS | ORACLE_INPUT_MISMATCH |
| S02-RU03 / G03 | event-121 (105) | PASS | PASS | MATCH |
| S02-RU04 / G04 | event-163 (142) | PASS | PASS | MATCH |
| S02-RU05 / G05 | event-205 (179) | PASS | PASS | ORACLE_INPUT_MISMATCH |
| S02-RU06 / G06 | event-248 (217) | PASS | PASS | MATCH |

## Exact content and state evidence

### S02-RU01 / S02-G01

Actual input: F01 결론을 두 문장으로 요약해줘.

Frozen required_content: F01 결론의 두 문장 요약

Actual displayed text:

```text
작은 읽기 작업을 병합하면 준비 비용을 줄일 수 있다. 기존 권한과 판본 검사는 유지해야 한다.
```

The two displayed sentences preserve both supplied F01 conclusions: merging small reads reduces preparation cost, and existing authority/version checks must remain.

Evidence: input=event-7[6], source_read=event-20[17], adoption_commit=event-31[27], adoption=event-33[29], display=event-37[31], candidate_receipt=event-39[33], delivery_commit=event-41[35], goal_complete=event-42[36], delivered_source_registration=event-44[38].

Goal: `goal:S02-U01`, revision1, task_id=null. Final request is HOST_COMPLETE at trace `/events/228/data/requests/goal:S02-U01`. This is recorded relation/state evidence, not a blanket semantic completion certificate.

### S02-RU02 / S02-G02

Actual input: 어제 민수가 보낸 M01 메일의 제목을 알려줘.

Frozen required_content: M01 제목과 발신 시각

Actual displayed text:

```text
10월 검토 일정
```

The actual user asks for the title of the M01 email sent by Minsu yesterday. The displayed title is exactly supported by M01 line2. The source also confirms sender and date. Reporting its send time is not requested.

Evidence: input=event-49[43], source_read=event-62[54], adoption_commit=event-73[64], adoption=event-75[66], display=event-79[68], candidate_receipt=event-81[70], delivery_commit=event-83[72], goal_complete=event-84[73], delivered_source_registration=event-86[75].

Goal: `goal:S02-U02`, revision1, task_id=null. Final request is HOST_COMPLETE at trace `/events/228/data/requests/goal:S02-U02`. This is recorded relation/state evidence, not a blanket semantic completion certificate.

Frozen RU02 adds sent time; actual U02 asks only title. M01 contains 2026-10-18 16:30, but the supplied common prompt and generic G02 completion add no requirement to display it. Do not turn this literal field nonmatch into model semantic FAIL.

### S02-RU03 / S02-G03

Actual input: 오늘 오후 CAL01 일정을 알려줘.

Frozen required_content: CAL01 시간과 제목

Actual displayed text:

```text
제목: 구조 검토 회의
시간: 2026-10-19 15:00-16:00
```

The displayed meeting title and full 2026-10-19 15:00-16:00 interval match CAL01 and answer the afternoon schedule request.

Evidence: input=event-91[80], source_read=event-104[91], adoption_commit=event-115[101], adoption=event-117[103], display=event-121[105], candidate_receipt=event-123[107], delivery_commit=event-125[109], goal_complete=event-126[110], delivered_source_registration=event-128[112].

Goal: `goal:S02-U03`, revision1, task_id=null. Final request is HOST_COMPLETE at trace `/events/228/data/requests/goal:S02-U03`. This is recorded relation/state evidence, not a blanket semantic completion certificate.

### S02-RU04 / S02-G04

Actual input: WEB01 가격과 기준 시각을 읽어줘.

Frozen required_content: WEB01에 표시된 가격과 기준 시각

Actual displayed text:

```text
제품 가격: 129000원
기준 시각: 2026-10-19 09:00
```

The displayed 129000원 price and 2026-10-19 09:00 reference time both match WEB01 and the explicit request.

Evidence: input=event-133[117], source_read=event-146[128], adoption_commit=event-157[138], adoption=event-159[140], display=event-163[142], candidate_receipt=event-165[144], delivery_commit=event-167[146], goal_complete=event-168[147], delivered_source_registration=event-170[149].

Goal: `goal:S02-U04`, revision1, task_id=null. Final request is HOST_COMPLETE at trace `/events/228/data/requests/goal:S02-U04`. This is recorded relation/state evidence, not a blanket semantic completion certificate.

### S02-RU05 / S02-G05

Actual input: PUB01에 게시된 오늘 운영 시간을 확인해줘.

Frozen required_content: PUB01의 운영 시간과 출처

Actual displayed text:

```text
오늘 운영 시간: 09:00-18:00
```

The displayed 09:00-18:00 operating hours match PUB01 and answer the actual request. PUB01 is retained in targets/source_spans; visible citation text is not established or explicitly required by this input.

Evidence: input=event-175[154], source_read=event-188[165], adoption_commit=event-199[175], adoption=event-201[177], display=event-205[179], candidate_receipt=event-207[181], delivery_commit=event-209[183], goal_complete=event-210[184], delivered_source_registration=event-212[186].

Goal: `goal:S02-U05`, revision1, task_id=null. Final request is HOST_COMPLETE at trace `/events/228/data/requests/goal:S02-U05`. This is recorded relation/state evidence, not a blanket semantic completion certificate.

Frozen RU05 adds source. Actual U05 asks the hours published in PUB01. targets/source_spans carry PUB01 but text does not repeat a citation. No supplied common contract mandates visible citation text; actual UI rendering of metadata is outside this evidence. Do not turn this field/presentation ambiguity into model semantic FAIL.

### S02-RU06 / S02-G06

Actual input: 북마크 BK01의 제품 설명을 알려줘.

Frozen required_content: 허용한 BK01의 지정 제품 설명

Actual displayed text:

```text
제품명: Cedar 도킹 스테이션
설명: USB-C와 외부 화면 두 개를 연결하는 허브
```

The displayed Cedar docking station name and USB-C/two-external-display hub description match BK01 and answer the product-description request.

Evidence: input=event-217[191], source_read=event-231[203], adoption_commit=event-242[213], adoption=event-244[215], display=event-248[217], candidate_receipt=event-250[219], delivery_commit=event-252[221], goal_complete=event-253[222], delivered_source_registration=event-255[224].

Goal: `goal:S02-U06`, revision1, task_id=null. Final request is HOST_COMPLETE at trace `/events/228/data/requests/goal:S02-U06`. This is recorded relation/state evidence, not a blanket semantic completion certificate.

## Applicability and limits

- Authority PASS is limited to actual permitted source reads. Each relevant revision1 source has local and external access, and each adopted plan has empty permission/control operations and no completed external-action claim.
- Approval and handoff: NOT_APPLICABLE, stored with UNREVIEWED status and excluded from applicable PASS/FAIL counts. No Agent ingress, Agent role call or actual Agent fact was observed.
- Generic goal completion includes necessary voice-summary wording, but this PoC trace has voice=OFF and contract19 excludes ASR/TTS/device implementation. Full goal completion is not independently certified here.
- Response and goal rows reuse the same six bodies. They are not twelve independent semantic successes.
- All six bindings.expression values copy source/answer text rather than an exact spoken substring, contrary to the supplied meaning prompt. Preserve this model-instruction deviation as an auxiliary diagnostic. S02 has no frozen QA02 binding obligations; add no cases or failures to that metric.
- No certified QA01 endpoint is newly selected. No evaluator, DAG, timing, deadline or aggregate was recomputed.

## Campaign decision

No technical stop is warranted by this bounded review. The two oracle/input disagreements affect separate diagnostic interpretation, not frozen primary callback timing, QA03 recorded host completion or S02 QA02 membership. Preserve the mismatch, raw outputs and original metrics. No oracle, prompt or runtime edit and no rerun.

## Reproducible references

- [Structured review](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/postanalysis/semantic-reviews/manual-reference-AAAA-S02-r1-v1.json)
- [Original trace](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAA-S02-r1-fea606/trace.json)
- [Frozen oracle](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/oracles.json)
- [Frozen fixtures](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/benchmark/architecture/common9_poc_20261019/fixtures.json)
- [Actual candidate input](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/raw/runs/AAAA-S02-r1-fea606/candidate-initial.json)
- [Frozen contract19 section5](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/common-supplement6-20261019/joint-formal-v1/frozen/source/docs/architecture/12-decisions/decision-packages/seven-problem-review/19-common9-evaluation-contract.md:88)

The JSON contains exact file SHA256/size, zero-based event indices, JSON pointers, raw model request/response identities, source passages and per-dimension applicability. It retains unresolved oracle requirements rather than silently redefining them.
