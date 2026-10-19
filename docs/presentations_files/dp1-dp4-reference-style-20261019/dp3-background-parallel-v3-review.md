# DP3 배경과 병렬 기본 B 정정

2026-10-19 사용자 리뷰 반영. 이 리뷰의 검토본은 `VIA-DP1-DP4-reference-style-17slides-v3-windows.pptx`다. 기존 16장 v2와 그 결과·판단 이력은 보존한다.

후속 사용자 정정은 [v4 검토](dp3-pixel-parallel-v4-review.md)를 따른다. 기존 B4는 병렬 기본 B의 pixel 조건 실측으로 활용할 수 있다. v3의 11쪽이 B를 전부 미측정으로 표시한 부분은 v4에서 같은 pixel 계약의 A/B4 비교로 고쳤다. 정규화 병렬 B만 미측정이라는 경계를 유지한다.

## 사용자 요청과 내용

- 기존 DP3 배경을 9~10쪽 두 장으로 나눴다. 9쪽은 Excel 선택, PDF의 클릭 없는 포인터 지칭, 메일 이동 후 정정이라는 같은 장면을 유지한다. 각 장면에서 무엇이 모호한지 질문으로 보이고, VIA가 정리할 명확한 요청과 유지·교체 범위를 제시한다.
- 10쪽은 앱 이동, 여러 앱의 자료 지칭, 클릭 없는 지칭, 부분 정정, Downstream Agent 연결이 VIA에서 결합되는 이유를 설명한다. 기록 보존과 현재성 확인은 공통 기능으로 두고, 중간 상태·실행 의존·재판단 책임의 A/B 선택으로 연결한다.
- 11쪽 기본 B는 화면·발화 후보의 독립 병렬 생산이다. 양쪽 후보가 준비된 뒤 결합하며, 부족한 범위는 원 생산자가 원본에서 확장한다. 충분한 후보의 코드 결합, 경쟁·미확정 후보, 원본과 revision 검증을 유지한다.

## 순차 기본 정책의 정정 이유

`benchmark/architecture/four_dp_presentation/dp04/normalized_candidate/src/dp04.rs`의 기존 B 경로는 screen 후 speech를 호출하지만, `dp04/producers.rs`의 speech 입력은 observation과 prior이며 screen 생산 결과를 받지 않는다. 현재 정의의 최초 두 생산 사이에는 데이터 의존이 없다. 따라서 공통 Model Access 호출 한도 K가 허용할 때 독립 작업을 겹치는 실행을 기본 B에 포함한다.

K=1이나 실제 자원 경쟁으로 직렬화되는 조건은 동일한 A/B 자원 계약 안의 제약으로 공개한다. 화면 후보를 발화 해석의 입력으로 넘기는 설계라면 실제 screen→speech 의존을 가진 별도 구조로 설명해야 하며, 이번 정정에서 그런 구조를 새로 만들지 않았다. 병렬화가 모델 복제나 독립 프로세스 배치를 뜻하지 않는다.

## 수치와 선택 상태

- 12쪽은 이전 정규화 A/순차 B와 별도 pixel B/B4의 실측 이력이다. 원값·분모·실패를 유지한다. 과거 B4라는 이름도 해당 실행 식별자로 보존한다.
- 정규화 병렬 기본 B는 새로 실행하지 않았다. 11쪽의 B 수치는 미측정이며, A의 값은 기존 정규화 관측 참고라고 명시했다. 이전 순차 결과를 병렬 B의 값으로 바꾸거나 단순히 시간만 빼서 추정하지 않는다.
- 현재 병렬 기본 B와의 최종 A/B 선택은 미확정이다. 과거 조건부 A 판단은 당시 실행 범위의 이력이다. 첫 응답을 포함한 고정 QA 우선순위와 실패·누락 분모를 유지한다.
- 13쪽의 A2+A5는 별도 pixel 복구 관측이다. 정규화 A와의 조합은 미실행이며, 정규화 병렬 B 비교값으로 쓰지 않는다.
- 새 모델 호출, benchmark 실행, QA04 변경 실험, 측정 freeze와 Architecture/ADR 변경은 수행하지 않았다.

## 파일과 확인

- 편집 생성: `scripts/presentations/revise_dp3_background_parallel_v3.mjs`.
- 기존 슬라이드 보존과 패키징: `scripts/presentations/package_dp3_parallel_v3.py`.
- 최종 17장 중 DP1/DP2/DP4의 12장 본문을 보존했다. 페이지 footer를 변경하고 DP4 notes의 16쪽 참조를 17쪽으로 고쳤다.
- `previews-v3/slide-01.png`부터 17까지 최종 PPTX의 Artifact Tool 재가져오기 렌더를 제공한다. 변경한 DP3 다섯 장을 전부 열어 확인했다. 나머지 12장은 원본 본문 복원 비교로 보존을 확인했다.
- `dp3-v3-validation.json`: package/layout finding과 warning 0, Calibri font policy 및 Artifact Tool 재가져오기 통과. 표 산술 자동검사는 이 표의 수치 검증 근거로 사용하지 않는다. 표시 수치는 기존 보고서와 직접 대조했다.
- `dp3-v3-preservation.json`: 원본 SHA와 보존 범위. 원본 16장 파일은 덮어쓰지 않았다.
- 실제 Windows Microsoft PowerPoint 렌더와 새 Windows CI 실행은 수행하지 않았다.

## 재생성

빈 전용 build 폴더에 bundled `node_modules`를 연결하고, 생성 스크립트의 복사본을 그 폴더에서 실행한다. 인자는 저장소와 build 폴더의 절대 경로다. 같은 출력 파일을 덮어쓰지 않으므로 새 build 폴더를 사용한다. 생성에는 모델 호출이 없다.
