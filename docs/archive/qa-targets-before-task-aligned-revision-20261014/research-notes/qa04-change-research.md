# QA04 변경 범위 조사 부록

> 조사 확인일 2026-10-14 / [현재 QA04 명세](../qa04-change-scope.md) / [실제 Git 수정 근거](../targets-source-qa04.md)

## 1. 사용자가 제공한 협업 문제

사용자는 여러 개발자가 각각 AI 코딩을 사용하면서 변경이 넓게 퍼져 충돌과 통합 비용이 늘고 다른 담당 영역을 수정하기 주저하는 경험을 설명했다. 이것이 변경 범위와 병행 수정 중복을 품질 후보로 다루는 직접 요구다.

변경 모듈 수는 수정과 검토의 범위를 나타낸다. 작업시간은 모델과 도구 및 개발자 숙련에 더 영향을 받으므로 구조 대안 비교의 주지표로 채택하지 않았다. 모듈 수가 곧 인건비나 실제 Git 충돌 횟수라고 주장하지 않는다.

## 2. Rust의 집계 단위

| 자료 | 확인한 구분 | 적용 |
| --- | --- | --- |
| [Rust Reference: Modules](https://doc.rust-lang.org/reference/items/modules.html) | `mod`는 함수와 자료형 등 item의 이름과 공개 경계를 구성하며 중첩 가능 | 고유 논리 모듈 경로를 센다. 한 파일에 여러 모듈이 있을 수 있음 |
| [Rust Book: Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html) | crate는 컴파일 단위이고 package는 Cargo 관리 단위 | crate나 파일 및 실행 프로세스 수와 모듈 수를 혼동하지 않음 |

식별자는 package와 crate target 및 완전한 module path를 조합한다. 운영 코드를 수정하거나 추가하거나 삭제한 고유 모듈을 센다. 자식만 수정했다고 부모를 자동 가산하지 않는다. 부모의 import나 mod 선언 및 공개 연결이 실제 바뀌면 부모도 센다. 소유 prompt와 schema 및 config 변경은 소유 모듈로 추적하며 테스트 전용 수정은 제외한다.

## 3. 외부 연구의 세부 내용

| 출처와 위치 | 확인한 내용 | 채택 범위 |
| --- | --- | --- |
| [CMU/SEI-2002-TR-025, Illuminating the Fundamental Contributors to Software Architecture Quality](https://www.sei.cmu.edu/documents/689/2002_005_001_14063.pdf) / §3.1~3.2 | 변경 모듈 수를 modifiability 시나리오의 response measure로 사용. 의존성에서 수정 파급을 설명 | 지표의 타당성 근거. 논문에 나오는 난이도 가중치와 모듈 목표 개수를 혼동하지 않음 |
| [Modern Code Review: A Case Study at Google, ICSE-SEIP 2018](https://storage.googleapis.com/gweb-research2023-media/pubtools/4476.pdf) / §5.2와 §7.1 | 약 900만 리뷰에서 35% 이상이 한 파일 변경이고 약 90%가 10개 미만 파일 변경. 작은 변경과 가벼운 검토 관행 설명 | 국소 변경의 협업 이점을 설명하는 보조 자료. 파일 수와 reviewer 수를 Rust 모듈 3개로 환산하지 않음 |
| [Understanding predictive factors for merge conflicts, 2020](https://www.sciencedirect.com/science/article/pii/S095058492030001X) | 더 많은 변경 파일과 개발자 및 commit이 관련된 큰 contribution에서 충돌 관련성을 보고 | 넓은 변경 파급을 줄이려는 방향의 근거. AI 코딩 자체나 3모듈 임계의 검증은 아님 |

코드 수정 benchmark의 파일 수나 함수 수를 Rust 모듈 수로 바꾸는 방식은 채택하지 않았다. 이름이 비슷한 변경 단위라도 측정 의미가 다르면 수치를 그대로 옮기지 않는다.

## 4. 실제 VIA 선례와 숫자 3의 선택

원본 `/Users/changbae/Workspace/via`의 HEAD `b6032a4c472e18ec216b3e8592346ab269495342`에서 Git 이력을 읽었다. 실제 운영 diff와 `mod` 선언을 대조했다.

| commit | 독립 변경 | 운영 모듈 |
| --- | --- | --- |
| `06bfa51bc6373d206f46c4f0ab01bcf5d6b05ab6` | Agent의 재할당 포트를 실제 실행 인자에 반영 | `via-backends / lib / crate::openclaw`, `via-backends / lib / crate::runtime`, `via-process / lib / crate::spawn`: **3개** |
| `6d984b635d755ec03e8d00a0b970ebeff61044aa` | 첫 음성 입력에서 엔진 연결을 시작하고 보관 입력을 전달 | `via-app / lib / crate::realtime::connection`: **1개** |

첫 사례의 gateway 파일은 테스트 assertion만 바뀌어 제외했고 integration test와 문서도 제외했다. 두 번째 사례의 wire payload 시험과 내부 테스트도 운영 모듈로 추가하지 않았다. 파일별 집계와 재현 명령은 [Git 수정 근거](../targets-source-qa04.md)에 보존했다.

두 사례의 평균은 2개다. 목표 3개는 그 평균이 아니라 **확인한 두 국소 수정 중 큰 값 3을 향후 대표 변경의 평균 상한으로 선택**한 제품 개발 요구다. 여러 독립 기능을 묶은 큰 commit을 한 과제로 세지 않았다. 이 선택이 모든 기능 확장의 난이도 분포를 대표한다는 뜻은 아니다.

## 5. 폐기한 산식과 대안

- “데이터 형식 변경은 1개이고 기능 확장은 5개”라는 가정: 실제 필요한 수정 목록에서 도출하지 않아 폐기했다.
- “주담당 1개와 관련 2개이므로 3개”라는 설명: 원하는 숫자를 다시 나눈 것이므로 사용하지 않는다.
- 외부 연구의 파일 수를 Rust 모듈 수로 환산: 집계 단위가 달라 사용하지 않는다.
- 실제 독립 수정 두 건의 관측 상단 3개를 참조: 구체적인 내부 선례로 사용하되 대표성 한계와 제품 판단을 함께 밝힌다.

## 6. 발표와 실제 변경 검증

발표 본문에는 실제 수정 두 사례와 평균 3개 목표를 연결하고 Rust 모듈 정의를 작은 예시로 보여준다. 자세한 SHA와 제외 hunk 및 재현 명령은 발표자 노트와 참고 페이지에 둔다.

핵심 답변은 “이미 가능한 국소 변경 수준을 기술과 기능이 바뀌어도 유지하고 싶다”다. 외부 자료가 3개라는 보편 합격선을 정했다거나 팀이 숫자까지 승인했다고 설명하지 않는다.

현재 C01~C06은 데이터 형식 변화 3건과 기능 확장 3건이다. 실제 후보에 적용한 patch가 없으므로 어느 안이 실제 평균 3개를 만족했는지는 미측정이다. 변경 전에 모듈 경계를 고정하고 각 과제를 같은 코드 버전에서 독립 수행하며 완성 여부를 확인해야 한다. 큰 모듈로 합쳐 숫자만 낮추거나 필요한 prompt와 설정 변경을 빼지 않는다.
