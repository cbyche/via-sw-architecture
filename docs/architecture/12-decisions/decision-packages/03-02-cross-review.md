# 03-02와 두 QA 발표자료 교차 검수

2026-10-11 사용자 요청에 따라 작성자가 정의 원본, 요약 3장과 QA별 부록 6장을 반복 대조하고 수정했다. 독립 심사위원/별도 에이전트의 리뷰나 실제 후보 성능 시험으로 표현하지 않는다.

원본은 [03-02](./03-02-quality-attribute-definitions.md), 사용량은 [W100 입력](./03-02-common-workload.json), 관측 대응은 [130장면 목록](./03-02-workload-manifest.json)이다. 발표자료는 [요약](../../../presentations_files/quality-attributes/README.md)과 [부록](../../../presentations_files/quality-metrics/README.md)이다. 이전 v4의 원본/두 발표자료/생성기 33개는 [archive](../../../archive/qa-common-workload-before-v5-20261011/README.md)에 보존했다.

## 1차: 사용량과 사건 경계

| 발견 | 수정 | 확인 기준 |
| --- | --- | --- |
| 기간 8시간만으로는 처리량과 비용/메모리의 의미를 설명할 수 없음 | W100 사용자 100/Agent 30장면, 같은 초기 이력/동시 상한/갱신/철회로 통일 | 기존 18개 사례별 횟수 합 130, 사용자/Agent 100/30 |
| 기존 자원 사용량이 정확성/시간의 사례에 연결되지 않음 | 다섯 block의 CORE/REUSE/BURST와 R1 CORE 관측을 source key로 연결 | 정확성 90/540, I/T 각각 30, 자원 동일 활동 3회 |
| 시간 전용 파생 장면이 원본 S1 조회의 0.2초를 8초, S6 답변 대기를 1초에서 2초로 변경 | 원본을 복원하고 최종 결과는 모든 QA 공통 tail로 제공 | S 원문 oracle 보존, §4와 §8.0의 I/T mapping 일치 |
| 음성 30분/10분이 구체 발화에서 산출되지 않음 | 원본 최초 234초+후속 30초, 사례별 출력 예산 합 914초 | 정확한 분 환산과 공개 단가로 $1.0987/$0.4013 검산 |

## 2차: 복원, 원장과 가정

| 발견 | 수정 | 확인 기준 |
| --- | --- | --- |
| Task 10개/4호출의 조건과 반복 간 상태 보존이 모호 | scene 자원 pool과 네 Agent burst를 정의, 완료/대기 history 보존, scope 복원 비용 포함 | 임의 완료/실패 덮어쓰기/매 block history 삭제로 자원 절감 금지 |
| source 변경 20건과 모델 production 20회가 혼동될 수 있음 | 원천 변경과 작성자 call-plan을 구별, 초기/추가/backfill/폐기 청구 전부 포함 | 호출 수는 구조의 결과, 무료 파생 정답 선주입 없음 |
| 초기 2,000turn/200 Task/30 source의 데이터 규모를 재구성하기 어려움 | background source/turn/Task 문자열 규칙, 20MiB payload와 별도 runtime overhead 정의 | JSON과 §8.0의 사용량 조건 동일 |
| 유휴 기간과 기능 종료를 혼동할 수 있음 | 처리 가능한 VIA queue drain 최대 30초, 그 뒤 idle 10초, 정당한 외부 대기 상태 보존 | 미완료/권한 위반/관측 누락은 낮은 비용/peak로 목표 충족하지 못함 |
| 정확성 실행 설명의 W 준비 120초/미래 QA 정의 문구가 최신 원본과 어긋남 | 준비 5초와 120초 실패 상한 구별, 최신 공통 활동/완성된 목표 참조 | 활성 prose에 이전 기간/파생 시험 조건이 현재 정의로 남지 않음 |

## 3차: 발표 표현과 렌더 수정

첫 렌더에서 공통 활동 슬라이드의 표 하단과 footer가 겹쳤다. 마지막 행 높이를 줄여 표를 1,026px에서 끝내고 footer를 1,040px에 유지했다. 비용 부록의 출력 914초는 “출력 예산”으로 표시해 모델 실측값처럼 보이지 않게 했다. 요약의 시간 범위 주석에는 외부 업무/사용자 답변 대기 제외를 표시했다.

수정 후 별도 새 build/final 파일로 두 deck을 다시 export/finalize했다. 최종 PPTX 재가져오기에서 만든 1920×1080 PNG **9장을 각각 확인**했다. 본문/목표/표/7단계 기호/주석의 잘림과 겹침을 확인했고 최종 렌더에 남은 문제는 발견하지 않았다. 레퍼런스의 흰 바탕, 파란 강조, 요약표와 목표 근거/오른쪽 등급표 구조를 유지했다. 모든 슬라이드는 native text/table이며 슬라이드 이미지 한 장으로 대체하지 않았다.

## 4차: 재현성과 회귀 검사

자동 검사는 원본/활동/manifest SHA-256, 여섯 ID의 정의/지표/목표/근거/보호 조건, 두 JSON의 동일 내용, 각 PPTX의 실제 문구와 노트, 모든 링크/PNG/등급 경계와 archive 65개 hash를 검사한다. 18개 원문 사례와 C1~C6 oracle은 v4 archive의 bytes 기반 문자열 비교로 보존을 확인한다.

의도적으로 사례 횟수, 발화 길이, I/T 연결과 cloud 상한을 하나씩 바꾼 입력은 모두 활동 검사에서 거부됐다. 단순히 파일이 생성되었다는 사실만 확인하지 않고 서로 다른 사용량/사건을 같은 QA 비교로 통과시키지 않는지 검수했다. 참조 계산기는 비용 합계, 시간의 병렬/외부 귀속, 공유 allocation의 중복과 경계 점수를 검산한다. 실행된 후보의 정답률이나 peak를 생성하지 않는다.

```sh
python3 scripts/architecture/qa_common_workload.py --check
python3 scripts/architecture/qa_poc_reference.py --check
python3 scripts/presentations/check_quality_attributes.py
python3 scripts/presentations/check_quality_metrics.py
python3 scripts/architecture/check_active_terminology.py
python3 scripts/architecture/check_active_markdown_links.py
python3 scripts/architecture/check_qa_catalog.py
```

최종 PPTX의 package/layout/font/native table/Artifact Tool import 검사도 통과했다. Microsoft PowerPoint 앱에서 저장/재열기, Windows 장치의 메모리, 유료 API/실제 모델/후보 실행은 수행하지 않았다. 목표/허용 예산은 작성자의 구체 평가안이며 달성 사실이나 A/B 최종 선택은 아니다. 이후 허용 예산을 개정하면 03-02 앞 표/상세/활동 원장/계산/두 발표자료를 함께 개정하고 이전 세대를 보존한다.

## 저장소 전체 CI와 QA 범위의 구분

QA 전용 검사와 활성 용어/링크/catalog 검사는 통과했다. 게시할 index만 분리한 사본에서 전체 architecture-ci 명령을 실행했으며 현재 기준선 `48c778484`의 기존 42/44/45 diagram 재생성 검사에서 drift가 재현됐다. 42는 choice42-structure SVG/draw.io, 44는 structure/event 두 쌍, 45는 structure 쌍이다. 해당 문서/생성기/그림은 이번 QA stage에 없고 baseline HEAD와 같은 입력임을 확인했다. QA 변경으로 도입한 실패가 아니며 다른 세션의 DP 그림/기제를 자동 재작성하거나 검사를 제거하지 않았다. 그 외 CI 명령은 통과했다. 전체 CI 성공과 QA 검수 통과를 같은 주장으로 섞지 않는다.
