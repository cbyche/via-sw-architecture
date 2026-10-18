# Four-DP PoC 증거 발행 패키지

2026-10-19 / 두 캠페인의 비캐시 기록 보존 구성. 사용자는 세 커밋과 push를 승인했다. 이 문서는 패키지 구성과 복원 방법을 설명하며 실제 Git 발행 성공은 Git 이력으로 확인한다.

## 보존한 캠페인과 읽기 순서

| 캠페인 | 실제 수행과 의미 | 직접 읽는 대표 보고서 |
| --- | --- | --- |
| `four-dp-qwen37-20261018` | 초기 기본120+보완·진단15회, Qwen579호출. 이후 사용자 리뷰로 측정/구현 한계를 확인한 이력이며 현재 선택으로 재사용하지 않음 | [초기 보고서](../../current/four-dp-qwen37-20261018/report.md), [측정 리뷰](../../current/four-dp-qwen37-20261018/metric-review.md) |
| `four-dp-qwen37-review-v2-20261018` | 보정26+원래기본120+보완27+중간교정11+최종교정36+번호진단9=229회, Qwen1,274호출. 최종 비교는 원래01/02/04의84회+최종교정05의36회인 명시적120선택 | [최종 요약](../../current/four-dp-qwen37-review-v2-20261018/final-measurement-summary.md), [전체비용](../../current/four-dp-qwen37-review-v2-20261018/campaign-totals.md), [캠페인 README](../../current/four-dp-qwen37-review-v2-20261018/README.md) |

직접 Git에 보존하는 작은 결과는 **903개/24,223,637bytes**이며 [정확한 목록](direct-reports.txt)과 [선별 정책](publication-list-policy.json)을 제공한다. 모든 캠페인 Markdown207개와 핵심 요약/phase/freeze/계획/수용 기록을 포함하고, raw·JSONL·큰 episode JSON·반복 전체 source·큰 replay는 전체 묶음에서 복원한다. 직접 파일도 원래 캠페인의 bytes를 유지하며 materialized budget symlink는 직접 파일 목록에 넣지 않았다.

먼저 [최종 설계·결과 설명](../../../../docs/architecture/12-decisions/decision-packages/seven-problem-review/15-corrected-poc-results.md)을 읽는다. 작은 Markdown/요약 자료는 복원 없이 읽을 수 있다. 큰 recorded replay와 raw 요청·응답·trace, frozen source/binary, 전체 세부 수용검사 기록은 아래 복원 후 기존 링크를 따른다. 초기·교정 전·중간 결과의 실패/비용을 최종 성공으로 교체하지 않았다.

두 원래 캠페인 트리의 파일 bytes와 동결 JSON은 패키징을 이유로 수정하지 않았다. archive 문서와 이전 계획은 당시 이력으로 보존하며 현재 요구·결과의 대체물이 아니다. 기존 문서의 '미commit/미발행' 문구도 당시 상태를 기록한 것이므로 소급 삭제하지 않았다. `docs/archive/`의 과거 상대링크는 당시 위치의 provenance이며 복원 과정에서 재작성하지 않는다. 현재 탐색은 통합 README와15를 따른다. 문서 archive는 Git의 문서 이력이고 이 묶음에 포함한 두 results 캠페인과 다른 보존 단위다.

## 파일과 무결성

- `manifest.json`: 포함 파일의 저장소 상대경로·크기·SHA256·mode, 원래 파일 종류와 symlink 정보.
- `exclusions.json`: 제외한 cache/build 산출물의 목록과 정책. 원시 모델 실패나 미표시는 제외 이유가 아니다.
- `package.json`: 두 캠페인과 전체 압축 스트림의 크기/SHA256, 순서 있는 part별 크기/SHA256.
- `evidence.tar.gz.part0001`부터 이어지는 part: 각각 최대20MiB. 순서 있는 하나의 gzip tar 스트림을 나눈 것으로 개별 part를 단독 압축 해제하지 않는다.
- [verification.json](verification.json): 패키징 담당자의 전체 검증 기록. [원시 묶음](verification-archive.json), [빈 위치 복원](verification-restore.json), [수치 재계산](verification-numeric.json), [기록 재생](verification-replay.json), [직접 파일903개](verification-direct-files.json)를 나누어 보존한다. 실행 성능 재평가나 새 모델 호출이 아니다.

포함 경로는 **9,877개**, 복원 파일 bytes 합계는 **1,413,700,108**다. 초기 캠페인1,705개/68,670,197bytes와 보완 캠페인8,172개/1,345,029,911bytes를 포함한다. 원래 regular file9,861개와 승인된 materialized symlink16개다. cache/build 등10,674개/1,099,523,495bytes는 명시적으로 제외했다.

압축 전체는 **81,406,773bytes**, SHA256 `726e8262fa80df57fbcc4aff683778bad42d835cce07670c743901eb31bdb53e`다. deterministic 인코딩은 경로정렬, tar uid/gid/mtime0·빈 owner명, gzip mtime0·빈 filename·level6이며 원래 파일 mode는 manifest에 기록한다.

|순서/파일|bytes|SHA256|
|---|---:|---|
|[evidence.tar.gz.part0001](evidence.tar.gz.part0001)|20,971,520|`66a822b7fa68e004b89a8f17101b86eac2537a5fafc73025c2fbc113d3bb2d48`|
|[evidence.tar.gz.part0002](evidence.tar.gz.part0002)|20,971,520|`b9d28babd4530350bba028d5662d425037b5e672d8c2a1944b700350001c138c`|
|[evidence.tar.gz.part0003](evidence.tar.gz.part0003)|20,971,520|`1e1b1cb63c9f22d827585d22448a842cd40d2df10868a6a2a8e80edb76e02639`|
|[evidence.tar.gz.part0004](evidence.tar.gz.part0004)|18,492,213|`a70eda41781dab41d8467b3410d088a7d845415b2e99768bc5016a151942fba8`|

manifest SHA256 `d238f9b16612438e5dc0b08b22880382fa180129ec72e1d30d7358f29c9185a7` / exclusions SHA256 `6ccb906ad4428cf48ff7710a202a8dbea10e3fca7ff69db99313e869212fe76b` / 패키징 도구 SHA256 `43838f7b14483314467ecfeebafcb7929e4f63018cb2a6d70b5170a7a2f6603a`. 전체 part·manifest·제외목록 검증은 [package.json](package.json)을 기준으로 한다. [secret-scan.json](secret-scan.json)의 제한된 token 패턴 검사 finding은0이며 모든 종류의 민감정보 부재를 증명하는 검사는 아니다.

### 공백 검사와 원본 보존

두 불변 캠페인에는 원래 CSV의 CRLF, unified patch의 빈 context 행, 일부 과거 Markdown의 EOF 빈 줄이 있다. `git diff --cached --check`가 이 원문 공백을 표시하더라도 evidence를 정규화해 SHA를 바꾸지 않는다. 두 캠페인 트리는 manifest/직접 파일 SHA 대조로 검증하고, 새 문서·코드·설정에는 일반 공백 검사를 적용한다. 이는 저장된 이력의 보존 범위를 명시한 것이며 전역 공백 경고 억제 설정을 추가한 것이 아니다.

## Clean clone에서 복원

Git clone을 완료한 저장소 루트에서 실행한다. Python3.12 이상과 표준 라이브러리를 사용하며 API 키·모델 호출·Rust 빌드가 필요하지 않다. 패키징 담당자의 실제 검증 환경은 Python3.14.7이다.

```sh
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py verify --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py restore --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination .
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py verify-restored --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination .
```

패키징 담당자는 빈 위치의9,877개 복원·SHA 대조, 기본120/보완27/전체229의 수치 재계산,229회/1,274개 요청·응답/PNG8개 재생을 확인했다. 검증 환경은 Python3.14.7이며 새 후보/모델 실행은 없다.

이미 직접 보존된 동일 regular file은 그대로 허용한다. 기존 파일의 내용이 다르거나 경로에 symlink가 있으면 덮어쓰기를 거절한다. 안전하지 않은 경로와 특수파일도 거절한다. 원래 budget symlink16개가 이미 있는 당시 작업 트리에는 restore를 반복하지 않고 `verify-live --package <package> --destination <original-tree>`로 확인한다. clean checkout의903개 regular file 또는 빈 복원 위치와 구분한다. 복원 전에 다른 내용의 결과가 있다면 별도 빈 디렉터리에 복원해 비교하며 원래 실패 기록을 지우지 않는다.

원래 budget symlink16개는 승인된 내부 대상 bytes를 regular file로 materialize한다. 원래 link 문자열·resolved SHA·파일 종류는 manifest에 보존한다. 원래 실행의 symlink 위상을 정확히 재현했다고 주장하지 않는다. POSIX mode와 파일 bytes 기록은 ACL/xattr·원래 실행환경 전체의 복원과 다르다. 저장된 binary는 당시 플랫폼 산출물이며 보고서를 다시 읽는 데 실행할 필요가 없다.

## API 없는 재분석과 한계

두 포터블 명령은 경로 재기반화에 Python3.12 이상이 필요하며3.11 이하를 지원한다고 주장하지 않는다. 복원한 저장소 루트에서 아래 명령을 실행한다. 출력은 반드시 복원 트리 밖의 새 경로를 사용한다.

```sh
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py portable-numeric --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination . --output /tmp/via-poc-numeric.json
python3 benchmark/architecture/four_dp_poc_v2/reports/evidence_bundle.py portable-replay --package results/architecture-evaluation/current/four-dp-poc-evidence-20261019 --destination . --output /tmp/via-poc-replayed.html
```

`portable-numeric`은 보존한 당시 row 분석기·동결 oracle로 기본120의 QA01~06과 보완27, 전체229의 호출·비용·실패를 재계산한다. QA04는 보존한 실제 수용증거의 모듈 수/평균 대조이고 새 patch 실행이 아니며, QA06은 저장된 RSS 표본 계산이지 새로운 RAM 측정이 아니다. `portable-replay`는 당시 builder로229회·1,274개 실제 request/response·8개 PNG를 재생한다. 초기 캠페인135회의 기록도 검증·복원하되 이 두 포터블 재계산/재생의 모집단은 보완 캠페인의229회다.

[재현 진입점](../../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md)에 당시 전체 `finalize_exports.py`/`audit_campaign.py` 명령도 보존했다. 그 전체 도구는 원래 저장소와 `/private/tmp` 절대경로를 가정하는 부분이 있어 다른 clone 위치에서 전부 재실행된다고 보장하지 않는다. 새 포터블 도구는 읽기 시 원래 저장소 prefix를 검증된 복원 경로로 연결하고 원시/frozen JSON의 bytes·SHA를 바꾸지 않는다. 복원 자체는 새 성능 측정이나 당시 callback/RSS의 재관측이 아니다. 실제 모델을 다시 평가하려면 새 freeze·결과 디렉터리·별도 승인과 비용상한이 필요하다.

로컬 Python/Rust RSS는 클라우드 모델 가중치·제품 UI·Windows commit 전체를 포함하지 않는다. 원문 번호 진단의 약704MB는 선행 대량 기록 파싱을 포함한 Python 시작조건이며 기본안 RAM으로 일반화하지 않는다. 유한 합성4사례×3반복 PoC이고 실제 사용자 연구나 전체 제품/harness 검증이 아니다.
