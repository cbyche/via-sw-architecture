# 현재 위치의 편집과 원래 평가 세대의 보존

2026-10-22 이관 / 원자료, 계산값, 점수, 계약 변경 없음

이 폴더의 Markdown은 현재 네 DP의 공통 QA v3를 설명하는 편집 정본이다. `artifacts/`에는 기존 `COMMON-QA-SYN-3.0` / `COMMON-QA-SYN-3.0-REVIEW-2`의 JSON, CSV, XLSX, HTML, PNG, 부모 사본, 검증 보고서를 원본 바이트로 가져왔다. 원래 위치의 파일은 그대로 남긴다.

## 무엇을 편집하는가

- QA 정의, 목표, 점수, 출처: [공통 QA 기준](../../qa/README.md)에서 편집한다.
- 세 profile의 적용 경계와 설명: 이 폴더의 `01`~`05` 문서를 함께 고친다. 원자료 가정이나 계약 의미를 바꾸지 않는 설명, 링크 정정과 새로운 평가 판본을 구별한다.
- 현재 수치와 산출물: `artifacts/`는 이관 시점의 동일 평가 세대를 보여 준다. 기존 수치에 맞추려고 부모 원본, raw, 검증 보고서, manifest를 수정하지 않는다. 데이터 또는 계약이 바뀌면 새 dataset과 출력 경로를 만들고 변경 이유, 이전 세대, 양안의 같은 조건을 남긴다.

## 이관 원장과 원래 manifest의 차이

[현재 이관 원장](../../qa-migration.json)은 원래 경로 → 현재 경로와 원본 SHA256을 기록한다. 원래 [manifest](artifacts/manifest.json)는 **원래 발행 세대의 증거**이며 그 안의 `source_path`, `source_commit`, 생성기 경로, 산출물 경로와 해시는 역사적 출처다. `parents/`의 상대 snapshot 경로와 이관한 부모 바이트는 유지했다.

원래 검증 JSON의 PASS는 그 원래 세대와 위치에서 수행한 검증의 기록이다. 새 위치에서 Markdown 링크를 고쳤다는 사실이나 파일이 같다는 사실을 새로운 모델, 장치 검증 PASS로 확대하지 않는다. 이관 검사는 파일 보존, 경로 연결과 기존 점수표 유지 여부를 별도로 확인한다.

## 생성, 검사 명령의 범위

[산출물 안내](artifacts/README.md)에 보존한 `build_common_qa_delivery_v3.py`, `render_common_qa_delivery_v3.mjs`, `check_common_qa_delivery_v3.py`와 `check_common_qa_delivery_v3_html.mjs`는 **기존 경로의 생성, 검사 계약**이다. 이번 정리에서는 생성기와 원래 출력 경로를 변경하지 않았다. 현재 파일을 편집한 후 이 명령을 그대로 실행해 원래 세대를 덮어쓰거나 현재 패키지가 재생성되었다고 판단하지 않는다.

현재 패키지는 `python3 scripts/architecture/check_current_package.py`로 이관 원장, 링크, 자산과 QA 점수 보존을 검사한다. 다음 데이터 개정 시에는 현재 패키지를 대상으로 하는 입력, 출력 경로를 먼저 명시하고 새 dataset으로 생성, 검증한다. 현재 정리 작업에는 새 모델 호출, 장치 측정이나 기존 결과 재채점이 없다.

## 유지할 해석 경계

공통 QA01은 화면 표시이고 첫 음성 지연은 별도 진단이다. QA09는 채택 후 청취 가능한 시간의 다른 지표다. QA02와 새 QA07은 첫 의미 확정을 평가하며 이후 전달 오류를 편입하지 않는다. Delivery 정확도는 고정 정답 fixture다. 수신 Agent 비용의 `null`은 미모델링이며 0이 아니다.

QA04는 예상 concept 모듈 경계이고 실제 Rust 변경이 아니다. Voice/Delivery의 기반 560 MB가 512 MB 요구를 초과한다는 사실을 유지한다. 세 profile의 절대값과 개별 DP 개선량을 합산하지 않는다. 기존 점수와 목표, 우선순위는 그대로이며 새 QA07의 목표, 점수, 순위는 미합의다.
