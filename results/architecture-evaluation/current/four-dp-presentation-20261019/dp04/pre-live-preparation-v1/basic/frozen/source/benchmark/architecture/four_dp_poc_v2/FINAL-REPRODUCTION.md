# 최종 evidence 재현 진입점

## Clean clone에서 저장된 증거 복원

발행용 구성은 작은 보고서·검토·manifest를 직접 읽을 수 있게 두고, 두 캠페인의 전체 비캐시 기록을 [분할 evidence 패키지](../../../results/architecture-evaluation/current/four-dp-poc-evidence-20261019/README.md)에 보존한다. 이 설명은 발행 구성과 재현 방법이며 commit/push 완료를 주장하지 않는다. 초기 결과와 최종 결과, 실패·미표시·중단 이력은 구별해 보존한다.

새 clone의 저장소 루트에서 먼저 패키지를 검증하고 원래 경로에 복원한다. Python3.12 이상과 표준 라이브러리만 사용하며 API 키·모델 호출·Rust 빌드는 필요하지 않다. 실제 패키징 검증 환경은 Python3.14.7이다.

```sh
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py verify --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py restore --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination .
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py verify-restored --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination .
```

보고서에 연결된 큰 replay HTML, raw 요청·응답·trace, frozen binary/source와 세부 수용검사 기록은 복원 후 원래 경로에서 사용할 수 있다. 직접 보존한 보고서만 읽는 데 복원은 필요하지 않지만, 아래 전체 재분석 명령에는 복원이 필요하다. 파일별 SHA·원래 권한·bundle/part SHA는 패키지 manifest에 있다. 다른 내용의 기존 파일을 덮어쓰거나 기존 symlink를 따라 쓰지 않는다. 기록된 budget symlink16개는 안전한 대상의 원래 bytes를 일반 파일로 복원하며 원래 link 정보는 manifest에 남긴다. 이 위상 차이를 원래 실행의 symlink 상태와 동일하다고 주장하지 않는다. 원래 symlink16개가 남은 당시 작업 트리에는 restore를 반복하지 말고 같은 도구의 `verify-live --package <package> --destination <original-tree>`를 사용한다. clean checkout과 당시 원본 작업 트리를 구분한다.

## 다른 clone 경로에서 API 없이 수치·재생 재현

다음 두 포터블 명령은 Python3.12 이상이 필요하다. 복원한 저장소 루트에서 사용하며 출력은 복원 트리 밖의 새 경로여야 한다. 원래 JSON의 절대경로는 provenance로 보존하고, 도구가 읽는 동안 원래 저장소 prefix만 검증된 복원 경로로 연결한다.

```sh
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py portable-numeric --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination . --output /tmp/via-poc-numeric.json
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py portable-replay --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination . --output /tmp/via-poc-replayed.html
```

`portable-numeric`은 보존한 당시 row 분석기와 동결 oracle로 기본120의 QA01~06, 보완27, 전체229의 호출·비용·실패를 재계산한다. QA04는 기존 실제 patch·수용증거의 모듈 수와 평균을 대조하며 새 코드 변경/수용실행을 하지 않는다. QA06도 저장된 RSS 표본의 재계산이지 새로운 RAM 측정이 아니다. `portable-replay`는 보존한 당시 builder로229회·1,274개 실제 request/response와8개 PNG를 재생한다. 두 명령은 모델/후보를 실행하지 않으며 원시/frozen JSON을 재작성하지 않는다. 초기 캠페인의 기록도 검증·복원하지만 이 포터블 수치·재생 모집단은 보완 캠페인의229회다.

발표용 비교는 `results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/effective-main-selection.json`의 명시적120선택이다. 원래01/02/04 84회는 `basic-v1`, DP05 전체36회는 `basic-dp05-reuse-union-corrected-v2`이다. 원래 DP05 36회와 calibration26·보완27·중간교정11·번호진단9를 포함한 actual229은 모두 별도로 보존한다.

## 정확한 후보 소스와 binary

- 원래 BASIC: `prototypes/candidates/four_dp_poc_v2/`; exact measured source/binary는 `basic-v1/frozen/`.
- 교정 DP05: `prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/`; exact source pin/binary는 `basic-dp05-reuse-union-corrected-v2/freeze.json`의 effective_candidate_source와 binary.
- DP04 A4/B4: BASIC과 동일한 frozen binary, 실제 input.variant는 A4/B4다. `tactic-dp04-A4`, `tactic-dp04-B4`의 plan/freeze 확인. --tactic만으로 이 경로가 활성화되지 않는다.
- DP01 A2: `tactic-dp01-A2/effective-candidate-source/` 및 frozen binary. exact BASIC+dp01.rs만의 승인변경, variant A+tactic A2.
- DP05 번호진단: 교정 DP05 binary 그대로, `diagnostic-dp05-numbered-lines/frozen/harness/`의 deep-copy 입력 formatter/worker. guard/oracle/caps는 유지되고 actual original-line 표기+공통설명만 추가.

## 당시 전체 파생 명령과 포터블 재현의 구분

아래는 당시 실행환경에서 사용한 전체 파생 명령이다. API 키가 필요하지 않지만 기록 안의 원래 저장소 절대경로와 `/private/tmp` 준비 경로를 가정하는 부분이 있어 다른 위치의 clean clone에서 이 전체 finalizer/audit가 그대로 재실행된다고 보장하지 않는다. 원시/frozen JSON의 경로를 고쳐 SHA를 바꾸지 않는다. 다른 clone의 수치·재생 재현에는 패키지 안내의 `portable-numeric`/`portable-replay` 경로를 사용한다. QA04는 보존한 patch·hash·수용 결과의 변경 모듈 수를 대조하며 새 patch 실행을 뜻하지 않는다.

```sh
python3 benchmark/architecture/four_dp_poc_v2/reports/finalize_exports.py
python3 benchmark/architecture/four_dp_poc_v2/reports/audit_campaign.py --final
```

재생은 campaign `demo.html`에 실제 requests/responses/PNG를 내장한 recorded replay다. 서버나 추가 API 호출이 필요 없고 actual display timing을 재측정하지 않는다. 원시50ms RSS 전수 표본은 별도 CSV/raw SHA 링크로 제공한다. 브라우저 로컬 파일 정책이 막으면 읽기 전용 HTTP 서버로 같은 파일을 제공할 수 있다.

최종 파생 계산기 정본은 `reports/`, 후속 측정 정본은 `post_basic/`다. 실제 실행 시 사용한 snapshot/SHA는 각 phase의 freeze와 final-analysis-source-manifest.json에 있다. results/analysis의 과거 준비본은 provenance이며 실행 정본이 아니다.

## 보존된 실제 실행 명령

`post_basic/tactics_control.py run dp04-A4`, `run dp04-B4`, `run dp01-A2`; `post_basic/corrected_dp05_union_control.py run`; `post_basic/line_number_diagnostic.py run --root <campaign>`를 사용했다. 준비 단계가 생성한 각 MODEL-CALLS-AUTHORIZED.json 및 plan의 고정 job만 실행했다. 이 명령은 완료 job을 건너뛰지만 새 모델 재평가는 새 결과 디렉터리·명시적 승인·전역 비용한도를 요구한다. 기존 실패행을 지우거나 같은 freeze로 조용히 재실행하면 안 된다.

calibration-v1의 당시 파생분석 provenance는 UNKNOWN으로 보존했다. 후속 CURRENT-REANALYSIS는 새이름/현재 계산기 hash/원시 hash를 가진 별도 재분석이며 당시 버전을 추정하지 않는다.
