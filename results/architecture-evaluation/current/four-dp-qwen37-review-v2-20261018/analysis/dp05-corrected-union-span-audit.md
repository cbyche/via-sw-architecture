# 교정 B E3 r1: 인접 원문 span 집계의 host 제한

실제 h3-meeting은 **NOT_DISPLAYED**이며 실패와 비용·시간을 유지한다. 미표시 초안을 의미 성공으로 세지 않는다. 후보/평가기/원시를 고치거나 재실행하지 않았다.

## 실제 공급과 거절

- actual_evidence에는 `public-comparison v1 2–3행`과 `4–4행`이 각각 공급돼 있었다. 같은 원본/판본의2–4행 전체에 빈틈이 없다.
- facts는2–3행의 앱/보안 문장과4행의 추천미확정 문장을 각각 정확한quote/range로 표시했다.
- 모델은 집계 필드 `source_spans`에서 이를2–4행 하나로 합쳤다. guide1–3행과quote1–3행도 실제공급돼 있었다.
- host의 `available.iter().any(|given|contained(p,given))`는 한 passage 안에 전체범위가 있어야 통과한다.2–4행은2–3이나4–4한개에는 포함되지 않으므로 repair후에도 거절됐다.

## prompt 명시 여부

select/result/repair의 실제 전송 시스템 prompt는 fact quote가 실제 supplied evidence.text의 현재 cited span에 정확히 있어야 한다고 요구한다. **집계 source_spans가 단일 passage 안에 있어야 한다거나, 실제 공급한 인접한 동일원본·동일판본 범위의 합집합을 금지하는 조항은 없다.** 따라서 이를 B의 의미오답 또는 미공급 원문 인용으로 설명하지 않는다. 현 host가 인정하는 coverage 표현이 더 좁았던 제한으로 분류한다.

이 행의 실제 input→failure는 55011.701625ms다. 초안의 문장 자체를 성공으로 치환하지 않고, 최종선택 해석에서는 이 실패원인과 실제 내용오류를 분리해야 한다.

## 코드와 원시 근거

- 실제 실행 소스는 phase의 `effective-candidate-source`다. `frozen/source`의 기본 하네스 맵과 구분한다.
- [dp05_sources.rs:8](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-corrected-v1/effective-candidate-source/src/dp05_sources.rs:8): `pub fn contained(span:&Value,p:&Value)->bool{span["source_id"]==p["source_id"]&&span["revision"]==p["revision"]&&p["start_line"].as_u64().unwrap_or(u64::MAX)<=span["start_line"].as_u64().unwrap_or(0)&&p["end_line"].as_u64().unwrap_or(0)>=span["end_line"].as_u64().unwrap_or(u64::MAX)}`
- [dp05_sources.rs:57](/Users/changbae/Workspace/via-sw-architecture/results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/basic-dp05-reuse-corrected-v1/effective-candidate-source/src/dp05_sources.rs:57): ` if !spans.iter().all(|p|valid_span(p,s)&&available.iter().any(|given|contained(p,given))){e.push("result_span_not_actually_supplied_or_current".into());}e`
- [모델 실제 요청/응답·범위·prompt 전문 JSON](dp05-corrected-union-span-audit.json).
- [원시 trace](../basic-dp05-reuse-corrected-v1/raw/runs/basic_corrected-dp05-e3-B-r1-a19d4b/trace.json).
