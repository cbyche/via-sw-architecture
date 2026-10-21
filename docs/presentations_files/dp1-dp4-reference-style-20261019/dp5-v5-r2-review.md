# DP5 구조도 개정 v5-r2

2026-10-20 사용자 요청: 두 방안의 SW Architecture 차이가 보이도록 구조도 재작성.

[21장 v5-r2 발표](VIA-DP1-DP5-reference-style-21slides-v5-r2-windows.pptx)는 사용자가 지정한 [17장 v4](VIA-DP1-DP4-reference-style-17slides-v4-windows.pptx)에 DP5를 붙인 개정본이다. 이전 v5 파일도 보존한다.

## 변경

- 19장을 확대 구조도로 재작성했다. A는 원본 조회와 요청 단위 해석·임시 상태, B는 사건 기반 생산·게시와 영속 파생 관계, 의존 판본·처리 cursor·무효화 및 검증된 조회 인터페이스를 구분한다. 원본 권위와 공통 의미 채택·위임 경계는 유지한다. [19장 미리보기](previews-v5-r2/slide-19.png).
- 구조도의 공간을 확보하기 위해 QA01~07 비교를 21장으로 모았다. 관련성과 조건부 차별화 가설을 구분하고 모델 비용의 반대 부담과 정확성 우열 미정을 표시했다. [21장 미리보기](previews-v5-r2/slide-21.png).
- 추가 프로세스·모델 복제를 요구하는 안으로 바꾸지 않았다. 작은 상자는 기존 Component 내부의 책임을 표현한다. 새 모델 호출·PoC·측정·선택은 수행하지 않았다.
- [설계 문서](../../architecture/12-decisions/decision-packages/seven-problem-review/dp5-delegation-memory-20261020.md) 5.1에 구조 계약을 보강했다.

## 확인

21장 PPTX 재가져오기와 렌더를 완료하고 새 DP5 4장을 열람했다. 첫 구조도 렌더에서 화살표가 제목을 가로지르는 부분과 긴 주석을 수정했다. 기존 17장은 원본과 footer 총수 외 픽셀 동일, 기존 DP5의 18·20장도 이전 v5와 픽셀 동일하다. v4 파일 자체는 변경하지 않았다.

패키지·레이아웃·Calibri 폰트 정책·재가져오기 검사 통과. Windows 형식 정책(Calibri, 정수 pt, ko-KR) 위반 0건. 실제 Windows PowerPoint에서의 열람·편집은 검증하지 않았다.

[패키지 및 레이아웃 검사](dp5-v5-r2-validation.json), [기존 슬라이드 보존](dp5-v5-r2-preservation.json), [렌더 비교](dp5-v5-r2-render-preservation.json).

소스: [슬라이드 생성](../../../scripts/presentations/add_dp5_reference_style_v5.mjs), [v4 보존 패키징](../../../scripts/presentations/package_dp5_reference_v5.py).
