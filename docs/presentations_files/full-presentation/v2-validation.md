# VIA 발표자료 v2 검증 기록 — 2026-10-12

대상은 [VIA-architecture-presentation_v2.pptx](../VIA-architecture-presentation_v2.pptx)다. [독립 리뷰](./review-2026-10-11.md)와 [사용자 구성 지시](./v2-composition.md)를 적용했으며 Architecture/QA/ADR 본문은 변경하지 않았다.

## 구성과 보존

| 항목 | 결과 |
| --- | --- |
| 전체 / 본문 / appendix | 57장 / 24장 / 안내 1장과 원본 32장 |
| 원본 8~17장 이동 | appendix 26~35장 |
| 구조 비교와 여섯 QA 같은 페이지 | 본문 16/18/20/22장, 각각 native 평가표 1개 |
| 원본 32장 및 발표자 노트 | 슬라이드 XML, 관계, 노트 XML과 관계가 원본과 bytes 동일 |
| 원본 media | 25개 모두 bytes 동일. 두 시연 GIF 포함 |
| 원본 보존 장 렌더링 | 32장 모두 최초 통합본 렌더링과 픽셀 동일 |
| 위치와 의미별 key | 57개 고유 key와 실제 슬라이드/Section 대응 확인 |
| 최초 통합본 | 파일 SHA-256 동일, 수정하지 않음 |

원본 내용은 본문에서 대체한 경우에도 appendix에 보존했다. 원본 비교와 DP별 상세 QA/지역 계산은 47~48, 50~51, 53~54, 56~57장에 있다. 현재 위치와 원본 대응은 [manifest-v2.json](./manifest-v2.json)을 따른다.

## 내용과 그림 검토

소개에서 Request/Task/Execution/Conversation/Context와 로컬/클라우드/외부 업무 책임을 설명했다. 필요성에는 모델 비용과 PC 메모리의 원인을 추가하고 QA/DP ID를 직접 언급하지 않았다. 요구사항의 근거, 두 시간 지표의 귀속 경계, 요구/품질/구조 질문의 연결, 각 배경의 선택 질문과 마무리를 추가했다. 참고 PNG의 번호순 흐름을 적용하되 선택·구현·실측 결과는 전용하지 않았다.

본문 24장과 appendix 안내를 렌더링해 전체 화면에서 확인하고 변경 후 재확인했다. DP 구조를 독립 리뷰어가 반복 검토해 다음을 수정했다.

- Request Controller, Request Interpreter, Response Manager와 Context Manager의 실제 Component 경계.
- 41의 Task 조회와 조건부 부분 해석, 모델 밖 코드의 전체 의미 구성.
- 44의 의미 제안 반환/채택, 채택 의미 또는 유효 Notice로 응답 후보 준비, Publication Join의 별도 게시 조건.
- 45의 공통 과거 생산/Reader와 필요 시 모델 호출, Context Manager 외부의 Model Access 경계.
- 42의 State Store 내부 공동 transaction과 독립 상태의 접수/확인 차이.
- 화살표가 설명 글씨를 관통하던 배치와 본문 사본의 오래된 페이지 번호.

최종 DP 재검토에서 미해결 P1/P2는 보고되지 않았다. 이는 독립 LLM의 발표자료 검토이며 사람 심사위원의 이해도 실험이나 성능 측정이 아니다. A의 비동기 실행/cache/부분 갱신/모듈화와 B의 추가 상태·표현 한계·미사용 생산·인계 비용을 유지했다. 제목은 설계 목적이며 A/B 우위의 입증이 아니다.

여섯 QA의 정의, 목표, 모집단, 보호 조건과 등급은 유지했다. 비교 표는 미실측을 `평가 전`, 로컬 메모리를 설계 `예산`으로 표시한다. 기존 지역 계산은 appendix에 보존하며 전체 P21 평가 결과처럼 표시하지 않는다. QA 상세 10장을 본문에서 appendix로 이동한 것은 QA 개정이 아니다.

## 파일 검사

- 57장 전체를 최종 PPTX에서 재가져와 PNG로 렌더링했다.
- package 무결성: 오류 0. Layout/font 검사: 오류 0, 경고 0.
- native 표 43개와 필요한 페이지의 표 소유를 확인했다. 신규 표와 도형은 편집 가능하다.
- 기존 16:9, 20×11.25in 크기와 Apple SD Gothic Neo/Arial Unicode MS 글꼴을 유지했다.
- Artifact Tool import는 57장으로 통과했다. 표 산술 자동 검사는 합계형 표가 없어 대상 0개이며, QA/예산 값의 정합은 활성 정의와 수동 대조했다.
- 실제 PowerPoint에서 글꼴 표시와 GIF 재생을 실행한 검증은 하지 않았다. 모델 실행, 후보 구현과 QA 측정도 하지 않았다.

발표자료 링크 23개, 활성 Markdown 168개와 활성 용어 검사를 통과했다. 신규 JavaScript/Python의 구문과 diff 공백도 확인했다. 저장소의 기존 Architecture CI는 앞선 `88166f70b`에서도 `choice42-structure.svg`/`.drawio`의 generator drift로 실패했다. 이번 작업은 그 Architecture 그림과 generator를 변경하지 않으며 발표자료 검증 통과를 저장소 전체 CI 통과로 표현하지 않는다.

파일 SHA-256:

```text
최초 통합본: 9bd1a17b48343ab9187ebedefb589cef5fdb9d13121d387f6c869dd5f3a1805a
v2 최종본:   0a601940a1ba7eaa05fa642e3f3dd085445f0e59b14788c8f52cb144ad187473
```

v2 최종본은 38,328,654 bytes다. 원본 media를 중복 저장하지 않아 이전 내용과 신규 본문을 함께 담으면서 중복 media 용량을 줄였다. 초기 구성/리뷰 기록과 최종 PPTX/manifest/안내를 작업 단위로 나누어 commit/push하며 다른 세션의 Architecture 변경은 포함하지 않는다.
