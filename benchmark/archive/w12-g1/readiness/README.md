# Measurement Readiness Audit

> **HISTORICAL / SUPERSEDED:** This audit validates the former W12-G1 timing contract, not the current Voice responsiveness contract. Keep it for provenance; do not interpret a pass as current measurement readiness.

> 2026-09-22 / 비교 성능 측정 전의 코드·계약 검증. 이 디렉터리는 p95·0~5 score·승자를 산출하지 않는다.

## 역할

- `timing_contract.py`: W-01 대화 반응성 / W-02 Task 인계 반응성 / W-03 Feedback 반응성의 단일 raw sample endpoint 검증. S2S 지연 이중 합산, 다른 process clock 차감, 로컬 enqueue를 Agent 접수로 대체하는 오류를 거부한다.
- `freeze.py`: 실제 host OS/메모리/검출된 chip 및 source/fixture/schema digest를 기록한다. 비교 승인이 없거나 코드가 바뀌면 실행 허가 검증이 실패한다. 승인 객체는 인간 승인에 대한 명시적 assertion이지 전자서명·인증체계는 아니다.
- `ir_contract.py`: 실제 모델을 부르지 않고 통합/단계형 semantic 요청 body를 작성한다. 세 TaskRelation과 별도의 pending interaction을 유지하고, 단계 결과를 손실 없이 최종 Decision으로 합친다.
- `canonical_adapter.py`: 승인된 94개 canonical TC를 하나도 누락하지 않고 IR/Task/Agent/EXEC/Voice/UI/Policy/Memory 등 필요한 executable slice에 매핑한다. 현재 prototype이 실제 구현한 경계와 fixture-only/미구현 경계를 구분하며 candidate observation이나 score를 만들지 않는다.
- `test_*.py`: 위 도구를 테스트한다. 테스트의 hand-written output은 테스트용 정답이며 후보 모델에 전달하지 않는다.

## 실행

```bash
python3 -m venv .venv  # 없을 때만, repository 루트에서
.venv/bin/python -m unittest discover -s benchmark/rebaseline/readiness -p 'test_*.py' -v
.venv/bin/python benchmark/rebaseline/readiness/freeze.py --out results/readiness-manifest-NEW
```

모듈은 Python standard library만 사용한다. 시스템 Python에 package를 설치하지 않는다. `freeze.py`는 설치나 benchmark를 시작하지 않으며 기존 output 디렉터리를 덮어쓰지 않는다.

## S2S 시간의 정확한 의미

`user_input_end`에서 실제 test sink까지의 wall-clock endpoint에 replay sleep이 이미 포함됐다면 S2S delay를 다시 더하지 않는다. 원천 delay 설정값, 실제 sleep 경과, model/Context/IPC의 raw spans는 각각 보존하되 nested/parallel span을 단순 합산하지 않는다.

`post_core_ready_window_ns`는 Core 입력 도착 후의 elapsed 구간이며 **순수 VIA CPU 시간이라고 부르지 않는다.** 이 구간에도 외부 의존성 대기가 있을 수 있다. 실제 S2S trace를 replay해도 조합 시스템은 `SIMULATED_E2E`; headless sink면 추가로 `HEADLESS`를 명시한다. Text에는 S2S 지연을 부과하지 않는다.

S2S의 first-audio packet benchmark는 입력 전사 준비 시간이나 Core routing 시간과 같지 않다. 공식 234ms를 모든 voice 경로의 고정 지연으로 대입하지 않는다. 현재 짧은 1/2/3ms fixture는 계측 smoke용이며 제품 latency profile이 아니다.

mock은 shared GPU contention, 메모리 점유, 실제 음성·전사 오류를 재현하지 못한다. 동일 delay replay의 A/B 결과는 이 고정 의존성 조건에서만 유효하다.

## IR 계약 정합성

`ir_contract.schemas()`를 구조 검증 원천으로 쓰며 `prototype/gate2/prompts/*-schema.json`과 같은 의미를 유지한다. TaskRelation은 `no_tracked_task / new_task / existing_task` 세 가지다. 질문/승인 응답은 독립 `pending_interaction_id`이며 TaskRelation 네 번째 값이 아니다.

단계 2/3의 body는 **실제로 나온 이전 단계 output**을 사용해야 한다. 미리 정답 중간 output을 넣으면 안 된다. `merge_stages`는 deterministic merge이지 네 번째 LLM 호출이 아니다. 구조/ID 검증을 통과해도 지칭·의도 의미가 올바르다는 증거는 아니며 W-05는 실제 동일 모델 실행과 reviewed oracle로 평가한다.

API body를 만드는 것과 실제 runtime이 schema/chat template를 지원하는 것을 검증하는 것은 다르다. 정확한 tokenizer/chat-template token count, actual Qwen inference, source snapshot/version은 여전히 실행 준비 항목이다.

## GitHub CI와 사용자 Mac의 구분

CI correctness 성공은 해당 source revision의 컴파일·회귀 증거다. 사용자의 MacBook Air 성능 측정이 아니다. 실제 Mac의 칩은 preflight로 검출하며 M5로 가정하지 않는다. Windows 11은 product target이고 macOS 실행은 macOS로 기록한다.

GitHub의 prototype 기본 구현은 아직 일부 AF-v1 기능, surviving external Agent, 전체 canonical replay 및 UI endpoint 연결이 부족할 수 있다. 다음 리뷰는 구현 coverage 원장의 미완료 항목까지 보고 benchmark 승인 여부를 판단해야 한다.
