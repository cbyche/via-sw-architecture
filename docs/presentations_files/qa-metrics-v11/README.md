# QA 지표 상세 페이지 v11

[33장 통합 발표자료](../VIA-architecture-presentation_v11.pptx)의 **25~33장**에 QA01~QA09를 한 장씩 정리했다. v10의 기존 Appendix 39장은 삭제했다. 본문 9~11장과 QA를 인용하는 문구도 새 정의와 맞췄다.

## 페이지 찾기

| PPTX 위치 | QA | 미리보기 | 원문 |
| --- | --- | --- | --- |
| 25 | QA01 첫 응답 지연시간 | [PNG](./qa01.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa01-first-response-latency.md) |
| 26 | QA02 지칭 대상 식별 정확도 | [PNG](./qa02.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa02-referent-accuracy.md) |
| 27 | QA03 VIA 처리시간 | [PNG](./qa03.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa03-via-processing-time.md) |
| 28 | QA04 변경 범위 | [PNG](./qa04.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa04-change-scope.md) |
| 29 | QA05 모델 사용 비용 | [PNG](./qa05.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa05-model-cost.md) |
| 30 | QA06 최대 RAM 사용량 | [PNG](./qa06.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa06-local-memory.md) |
| 31 | QA07 부하 추가 지연 | [PNG](./qa07.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa07-load-interference.md) |
| 32 | QA08 병행 수정 중복 | [PNG](./qa08.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa08-parallel-change-overlap.md) |
| 33 | QA09 음성 전달 지연 | [PNG](./qa09.png) | [상세 정의](../../../docs/architecture/12-decisions/decision-packages/qa-candidate-package/qa09-audio-delivery-lag.md) |

각 장은 상단의 분류 / Title / Scenario 표와 목표 수치의 근거, 측정 산식과 계산 예시, 0~6점 기준표, DP 관계 한 문장으로 구성한다. Scenario에는 정의와 목표, 짧은 측정 조건 및 목표 근거를 넣었다. 문구와 표는 PowerPoint에서 편집할 수 있다. 출처 URL과 상세 판정 조건은 발표자 노트에 있다.

## 적용한 정의

- QA01과 QA03은 최신 **QA-C01-v3 / QA-C03-v3의 산술평균**이다. 오답도 실제 처리 시간을 기록하며 정확성은 따로 판정한다. 종료점 미관측은 `NOT_OBSERVED`로 별도 보고하며 일부 관측의 평균을 전체 모집단 평균으로 표시하지 않는다.
- QA02는 입력 표현과 revision별 첫 채택 대상 집합을 정답 집합과 비교한다. 사후 정정으로 최초 오류를 지우지 않는다.
- QA04는 실제 수정 Rust 모듈 수를 센다. 사용자 제공 ARGO PR 145건 평균 5.4개와 40% 개선 요구를 근거로 평균 3개를 적용했다.
- QA05는 공통 28개 전체 묶음당 모델 비용이다. gpt-realtime과 Luna의 참조 견적을 사용하며 월 구독료 역산은 사용하지 않는다.
- QA06은 사용자 지정 **512 MB = 512,000,000바이트**다. PC 플랫폼 및 기술기획 측의 시스템 요구사항으로 기록했다. 전체 VIA가 같은 시점에 사용하는 실제 RAM을 집계하며 공유 페이지를 중복 계산하지 않는다.
- QA07~QA09는 추가 후보다. 목표값의 성격을 각각 간섭 예산, 협업 예산, 음성 전달 예산 제안으로 명시했다.

계산 예시는 지표 설명을 위한 가상 값이다. 목표 수치의 출처와 적용 논리는 각 QA 원문을 따른다. 이 발표자료 제작에서 PoC 실측이나 A/B 최종 선정은 수행하지 않았다.

## 편집 원본과 재생성

- [페이지 문구와 점수표 JSON](./qa-pages.json)
- [Artifact Tool 작성 스크립트](../../../scripts/presentations/generate_qa_appendix_v11.mjs)
- [v10 본문과 자원 보존 및 Appendix 교체](../../../scripts/presentations/package_qa_revision_v11.py)
- [최종 검증과 전체 렌더링](../../../scripts/presentations/verify_qa_appendix_v11.mjs)
- [페이지 manifest](../full-presentation/manifest-v11.json)
- [검수 기록](../full-presentation/review-v11-2026-10-15.md)

재생성은 저장소 밖의 새 private build 디렉터리에서 수행한다. bundled Node의 `node_modules`를 build에 연결하고 두 `.mjs` 파일의 사본을 build에서 실행한다. `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`은 설치된 bundled runtime과 Presentations skill 경로로 설정한다. 다음 순서의 `REPO`와 `BUILD`에는 절대 경로를 전달한다.

```text
node BUILD/generate_qa_appendix_v11.mjs REPO BUILD
python3 REPO/scripts/presentations/package_qa_revision_v11.py REPO BUILD
node BUILD/verify_qa_appendix_v11.mjs REPO BUILD
```

검증 후 `BUILD/final/VIA-architecture-presentation_v11.pptx`와 `final-renders/25.png`~`33.png`를 발행한다. 출처 링크는 실행 시 QA Markdown에서 읽으므로 현재 원문이 바뀌면 노트도 달라질 수 있다. 발행본의 문구와 점수표는 이 폴더의 JSON에 보존한다. 후속 발표 편집은 v11의 사본에서 v12로 저장하며 v10과 v11은 유지한다.
