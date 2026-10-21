# DP5 추가 v5

2026-10-20. 사용자가 지정한 정확한 원본은 [17장 v4 Windows 발표](VIA-DP1-DP4-reference-style-17slides-v4-windows.pptx)다. 원본 SHA256은 `8e4bdcbd6d3f279167cf46b4e83a9ceb86848004414bc3d05fe856667763a091`이다.

## 최종 파일

[DP1~DP5 21장 v5 Windows 발표](VIA-DP1-DP5-reference-style-21slides-v5-windows.pptx).

기존 17장 뒤에 동일 서식으로 DP5 4장을 추가했다.

| 장 | 내용 | 미리보기 |
| --- | --- | --- |
| 18 | 위임 메모리 문제와 실제 작성자·수정자·평가 범위 | [PNG](previews-v5/slide-18.png) |
| 19 | A 요청 시 관계 해석 / B 사전 관계 메모리, 구조·장단점·QA01~07 비교 | [PNG](previews-v5/slide-19.png) |
| 20 | 선택 조건과 A1 재조회 색인 / B1 사건 묶음 생산 | [PNG](previews-v5/slide-20.png) |
| 21 | 지연과 생산비용 교환, 정확성 경계, 차별화 반증 조건 | [PNG](previews-v5/slide-21.png) |

DP5는 설계 가설이며 미측정·미선택이다. 기존 DP들의 실측값을 DP5로 전용하지 않았다. [설계 원문](../../architecture/12-decisions/decision-packages/seven-problem-review/dp5-delegation-memory-20261020.md)에 세부 계약과 범위가 있다.

## 보존과 검증

- 원본 v4는 변경하지 않았다. 기존 17장의 본문, 수치, 도형, notes, 관계 파일과 원본 master/layout/theme/media를 유지했다. 기존 슬라이드에서는 페이지 표기의 총수만 17에서 21로 바꿨다.
- 실제 v5 PPTX를 다시 읽어 21장을 렌더했다. 이전 17장은 페이지 footer를 제외한 모든 픽셀이 v4 미리보기와 같았다. 새 4장은 각각 열람하고 19장 범례의 겹침을 수정했다.
- Calibri, 정수 pt, ko-KR 규칙 검사에서 위반 0건이다. 패키지·레이아웃·재가져오기 검사를 통과했다. 실제 Windows PowerPoint에서의 열람·편집이나 새 Windows CI 실행을 수행한 것은 아니다.
- 기록: [보존 범위](dp5-v5-preservation.json), [렌더 보존](dp5-v5-render-preservation.json), [검증 결과](dp5-v5-validation.json).
- 소스: [슬라이드 생성](../../../scripts/presentations/add_dp5_reference_style_v5.mjs), [정확한 원본에 추가하는 패키징](../../../scripts/presentations/package_dp5_reference_v5.py). 기존 reference-style helper를 사용해 새 슬라이드를 만들고 기존 17장은 재작성하지 않는다. `full_presentation_package.py`와 `presentation_windows_policy.py`를 재사용한다.

## 기반 파일 정정

앞서 만든 `five-dp-delegation-memory-20261020/`의 40장 통합본은 다른 `four-dp-final-20261019` 파일을 기반으로 만들었다. 사용자가 요청한 v5는 이 문서에서 연결한 21장 파일이다. 이전 40장 자료는 삭제하지 않고 잘못 지정한 기반의 별도 산출물로 남긴다.
