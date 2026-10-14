# DP-47 작성자 리뷰와 개선 기록

> 2026-10-13 / 작성자 반복 리뷰 / 독립 리뷰, 실제 Windows/API 시험, 모델 평가 아님
> [설계](./04-47-screen-voice-grounding.md) / [사고실험](./dp47-design-experiments.md) / [구조 조사](./dp47-structure-references.md)

## 1. 리뷰 범위와 판정 기준

사용자는 다양한 사고실험과 다른 SW 구조/구현을 조사하여 강하게 경쟁하는 두 설계를 완성하도록 요청했다. 세 기준은 지칭 근거의 형태와 보존, 말과 화면 판단의 협력, 일부 정정의 유지 및 재판단이다. 검토는 설계, 문서의 모순을 찾아 수정하는 작성자 작업이다. 별도 리뷰 Agent나 독립 검증자를 실행하지 않았다.

**현재 판정:** 공통 입력과 지원 목표를 유지한 두 SW 대안을 사용자 검토에 제시할 수 있다. 정상 경로의 중간 후보 계약, 의미 생산과 재생산 책임과 version 결합이 실제로 다르다. Major DP의 최종 자격, 측정 우위와 구현 전환비를 입증했다는 판정은 하지 않는다.

## 2. 1차 — 단순 구현과 reference 대조

| 지적, 반론 | 보완과 남는 조건 |
| --- | --- |
| 마지막 screenshot, pointer, foreground로는 과거 페이지와 circle을 잃음 | frame/path/selection/source 시점, 오차, gap을 양안의 공통 계약으로 보존 |
| raw 입력 대 미리 scene 게시라는 pair는 45/46과 겹침 | 양안 모두 현재 발화/request의 지칭 처리. B의 candidate를 current request 임시 자료로 한정, 전역 screen inventory와 독립 recorder 선택을 제외 |
| source 시각이 있으면 지칭까지 맞는 것처럼 보임 | 기술적 시간 대응과 사용자 의도 판단을 분리. 단어 span 없는 경우 불확실 구간과 확인을 명시 |
| separate inference가 단순 prompt 분할에 머무름 | B에 producer가 관리하는 candidate ID/coverage/revision, 필수 소비, 새 target의 원 생산자 재생산과 결합 contract를 정의 |
| GUI control parser나 QuickSet가 모든 PC 지칭을 해결했다는 확대 | 원 자료의 실제 범위, 가져온 구조 원리와 VIA 적용 추론을 분리. pen map/모델 학습/추가 helper/업무 Action은 가져오지 않음 |

## 3. 2차 — 강한 반론과 16개 사고실험

| 지적, 반론 | 적용한 개선 |
| --- | --- |
| A를 whole replay, one-shot, 매번 전체 재호출로 약화하면 B가 과장됨 | A에 bounded read, crop/cache, source와 binding dependency, targeted rejudge와 patch를 허용. B만 부분 수정한다는 주장 제거 |
| B가 1위 screen target을 고정하면 navigation, circle에서 회복 불가 | navigation/designation 경쟁 후보, set/region/unknown coverage와 peer 정보를 받은 재판단 추가 |
| 후보에 원래 대상이 없으면 복수 후보도 소용없음 | resolver가 후보 확장을 요청하고 Screen Target Producer가 원본에서 새 target/extent를 생산. 못 만들면 해당 지칭 확인 |
| B의 resolver가 raw에서 target을 직접 만들면 A+B hybrid가 숨겨짐 | pure B의 생산자 관문과 hybrid 우회를 분리. pure B 성능/coverage로 우회 성공을 계산하지 않음 |
| 일부 정정의 dependency가 완전하다고 가정 | 영향 불확실 시 전체 관계 확대 점검. “앞 페이지 아래 표” 변형 E14 포함 |
| source가 늦으면 항상 기다리고 화면 완료를 아는 것으로 보임 | source별 coverage와 bounded 기다림, gap/미완료/미해결을 구별. button up과 안정 frame도 rendering 완료의 의미 증명 아님 |
| cloud API가 원음+image/span을 모두 지원한다고 가정 | 양안 동일 capability profile, 미지원이면 둘 다 전사/알려진 구간 사용. dependency 지원과 정확성은 미확인 |
| 후보가 작아 B의 비용, 메모리가 자동 감소함 | 같은 raw 한도, producer의 media read/호출, output, 왕복, metadata, staging을 비용에 포함 |
| 삭제 후 후보를 쓰거나 서로 다른 source를 독립 증거로 중복 계산 | 원본/파생/사본 사용권 철회, 같은 원본 파생 참조 구별. E13/E15와 수명 계약에 반영 |
| 내부 엔진 교체로 가능함에도 전체 제품 전환이 어렵다고 과장 | 공통 proposal와 주변 Component 유지, request 종료 후 migration 제거, 양방향 최소 전환, hybrid 가능성 공개 |
| 43의 일반적인 기능별 해석을 새 이름으로 복사한 것 아닌가 | 47은 음성과 관측의 실제 자료와 referent 후보 coverage, 원본 재생산에 한정. 전체 Task/업무 의미 권위 분할은 제외 |

## 4. 3차 — MAIN과 문서 계약 대조

| 발견한 표현, 도식 문제 | 수정 |
| --- | --- |
| Model Access를 Request Interpreter 박스 안에 두면 내부 Component처럼 읽힘 | Model Access를 밖으로 이동. 내부 모듈, 임시 상태만 실제 owner 안에 배치 |
| B의 재판단 결과와 의미 model 반환이 일부 연결에서 보이지 않음 | producer/resolver별 의미 요청, 점선 결과, Coordinator를 통한 candidate read, 결합 요청, 재판단 경로 표시 |
| 공통 Request Controller를 입력/채택 위치에 반복하여 둘로 읽힐 수 있음 | 두 위치에 같은 owner의 책임 확대 표시. 실제 추가 Component가 아님 |
| text끼리 겹침은 없지만 선이 제목, 주석을 지남 | 원음, GroundingInput 경로를 이름을 피하도록 변경, cost 주석을 adoption 경로 양옆에 배치. PNG에서 다시 확인 |
| 문서의 이전 “대안 미정” 상태가 새 내용과 충돌 | 현재 top checkpoint, 읽기 안내는 두 대안 구체화/사용자 검토로 갱신. 이전 지정 기록은 당시 미정 이력으로 보존 |

MAIN은 [SVG](./diagrams/choice47-structure.svg), [draw.io](./diagrams/choice47-structure.drawio), [PNG](./diagrams/choice47-structure.png)로 제공한다. 생성기와 renderer는 문서 도구이며 DP 후보의 실행 구현이 아니다.

## 5. 검사 기록

| 실제 검사 | 결과, 범위 |
| --- | --- |
| `check_active_markdown_links.py` | PASS / active Markdown 397개 local link 검사. 이후 concurrent 문서 추가에 따라 파일 수는 바뀔 수 있음 |
| `check_active_terminology.py` | PASS / active 용어와 정식 target Component 이름 |
| `check_qa_catalog.py` | PASS / 기존 core-ASR registry 보존. 이 검사는 새 47 QA 실행이나 목표 달성이 아님 |
| `generate_screen_voice_grounding_diagram.py --check` | PASS / SVG/draw.io deterministic 일치, XML, ID, owner 포함 좌표, canvas, 직교 edge endpoint, 무관한 내부 node 관통 검사. DP-47 SVG 2개가 본문에 삽입되고 편집 원본도 연결된 것을 검사 |
| `render_screen_voice_grounding_diagram.cjs` | PASS / 실제 Chrome에서 MAIN 2560×1440 및 문제 그림 1920×1080 PNG, 두 그림 모두 text width/overlap/bounds 오류 0. [MAIN SHA 기록](./diagrams/choice47-structure-render.json), [문제 그림 SHA 기록](./diagrams/screen-voice-grounding-background-render.json) |
| PNG를 열어 흐름 재대조 | 공통 voice/capture, 양안 의미 요청/반환, 후보 교환, 재판단, 현재 채택과 외부 위임 확인. 선이 제목, 주석을 가리킨 부분 수정 후 다시 확인 |
| `git diff --check` | PASS / whitespace 검사. 다른 세션의 변경은 이 작업의 구현, 검증 결과로 포함하지 않음 |

링크, 용어, XML, render 검사는 의미 판단 성능을 검증하지 않는다. 원본 시각, native 자료를 실제 취득하거나 cloud 모델에 보내지 않았다. renderer의 최초 제한된 실행은 Chrome 시작에 실패했고, 로컬 렌더 실행 권한으로 같은 도구를 수행하여 실제 PNG와 text geometry를 검사했다.

## 6. 사용자 검토에서 남기는 실질적인 한계

- Windows의 실제 source 노출, 여러 monitor/DPI 조건, provider의 원음, 단어 span 및 media capability는 시험하지 않았다.
- 모든 필수 gesture/정정의 실제 성공, candidate miss와 재생산 빈도, model queue와 총비용은 측정하지 않았다.
- A의 값싼 helper/cache/patch가 B의 중요한 이익을 얻는지, B의 추가 계약이 주요 품질 차이를 만드는지가 최종 선택의 핵심이다.
- 전환비는 Request Interpreter 내부의 새 후보, 재판단 계약에 남는다. 주변 시스템이나 진행 중 업무의 대규모 migration 비용을 만들어 주장하지 않는다.

완료한 것은 두 대안의 설계, 설명, 반론 검토 자료다. 사용자의 대안 승인, 참조 Architecture 변경, 실행 검증 또는 winner 선정은 별도다. 기존 PPTX와 42 강화 문서는 이 작업에서 수정하지 않는다.

## 7. 본문에서 모든 그림을 확인하도록 정리

사용자 요청에 따라 [47 본문 §0](./04-47-screen-voice-grounding.md#0-이-문서에서-모든-dp-47-그림을-확인한다)에 전체 그림 목록과 추가 규칙을 기록했다. 초기 문제 그림은 §1, MAIN은 §6에 직접 삽입하고 각각 draw.io와 PNG를 연결했다. 새 관련 SVG가 본문에서 빠지면 생성기의 `--check`가 실패한다. AGENTS.md에도 같은 관리 규칙을 반영했다.

DP-47 본문, 다섯 보조 문서, 그림과 생성 원본의 가운데 점을 제거했다. 초기 문제 그림의 번호와 A/B 미정 문구를 현재 본문 참조로 바꾸고 SVG, draw.io와 PNG 및 통합 편집 원본의 해당 페이지를 맞췄다. 두 PNG를 열어 확인하고 위 문서 검사와 생성 검사를 통과했다. 이는 표기와 문서 일치 검사이며 SW 구현이나 API 검증은 아니다.

## 8. 구현 수준의 안별 구조와 사건도 검수 / 2026-10-13

사용자는 DP-44 §3.7보다 구체적인 안별 실행 구조와 주요 flow를 DP-47 본문에 넣고 commit/push하도록 요청했다. [47 §8.5](./04-47-screen-voice-grounding.md#85-구현을-가정한-상세-구조와-실행-계약)에 별도 A/B 구조도와 별도 A/B 사건도, 총 네 장의 SVG/draw.io와 3840×2160 PNG를 추가했다. 기존 MAIN과 배경 그림은 유지했다.

| 작성 중 검토한 문제 | 최종 반영 |
| --- | --- |
| Module 이름만 달라지고 실제 실행 준비 조건은 같아 보임 | A의 원본 → 직접 binding/patch와 B의 producer별 게시 → version 결합 → candidate miss → 원 producer 확장을 상태와 실제 왕복으로 연결 |
| A를 한 번의 큰 모델 호출, B를 항상 정확한 후보 구조로 약화/강화할 위험 | A의 bounded read, cache와 부분 patch 및 추가 판단을 유지. B는 t0 하나로 부족한 경우 원본을 다시 읽어 t7/t8을 생산해야 하는 사건을 포함 |
| 입력 등록이 의미 해석용 read 반환 이후로 오해될 수 있음 | Timeline & Buffer에서 Input Revision Control로 직접 등록. `EvidenceReadReceipt`는 안별 read 호출자에게 별도 반환 |
| 취소 요청으로 늦은 callback까지 없어지는 것처럼 보임 | 실제 반환 owner에서 attempt/generation을 검사해 상태 게시 전에 거절. 마지막 채택 검사와 중간 상태의 조건부 게시를 구별 |
| 후보 갱신의 peer 의존과 결합 재시작이 사라짐 | B 사건도에 생산 완료, m3/t0 충돌, 확장 후 새 read-set, r2의 m4와 target revision, 재결합을 순서대로 표시 |
| raw를 cloud에 바로 제공한 뒤 최종 채택에서만 사용권 검사 | 공통 Model Use Gate와 Policy Manager 호출/반환을 표시. 원본 조회, 전송 전, 결과 사용 전의 사용권 조건을 본문에서 구별 |
| 구조를 다르게 보이게 하려고 공통 배치를 바꾸거나 callback 선이 Module 안을 통과함 | 공통 27개 요소의 좌표와 크기를 일치 검사. source/target도 내부 관통을 허용하지 않고 포트에서 바깥으로 출입하도록 배선 수정 |
| grounding 결과에서 바로 외부 업무 실행으로 읽힘 | 그림과 본문에 전체 요청 해석과 admission을 거친 후속 응답/업무 경계 명시. Task Manager 내부나 Agent 실행 구조를 이번 비교로 다시 설계하지 않음 |

전용 생성기는 XML, owner 포함 관계, 직교 배선과 node 관통, 동일 공통 좌표, SVG/draw.io 재생성 일치를 검사한다. 전용 renderer로 네 PNG를 생성하고 실제 글자 폭, 겹침과 canvas 이탈 오류 0을 확인했다. PNG를 열어 호출/반환, 자료 수명과 정정 경로를 본문과 대조했다. render manifest의 SVG/PNG 해시를 검사해 source만 바뀐 뒤 옛 PNG가 남는 것을 막는다.

기존 `generate_screen_voice_grounding_diagram.py --check`에서 상세도 검사를 함께 실행한다. 여섯 관련 그림의 본문 삽입, draw.io 및 PNG 링크, 본문 가운데 점 금지도 검사한다. 이 검사를 architecture CI에 추가했다. active Markdown 링크, 용어, QA registry 검사를 통과했다. whitespace 검사는 DP-47 commit 대상으로 범위를 제한하며 다른 세션의 미완료 변경은 포함하지 않는다.

이 기록은 작성자의 문서/도식 검수다. 새 독립 리뷰, Windows/앱/API 실험, 제품 구현, 모델 실행, 비용 측정이나 A/B 선택을 수행하지 않았다. 42/45의 동시 작업, 44 원본과 기존 PPTX, 참조 Architecture와 QA/ADR은 이번 commit 범위에서 제외한다.

## 9. reference를 반영한 queue, data model과 pattern 검수 / 2026-10-14

사용자는 실제 SW 구현 요소와 architectural style/pattern까지 드러나도록 §8.5의 도식을 구체화하도록 요청했다. 기존 원본 공동 해석과 후보 생산/결합의 의미 경계를 유지하면서 네 그림과 해당 본문을 갱신했다. 이 절은 작성자의 설계 및 도식 검수 기록이다.

| 발견한 문제 | 반영한 구체화 |
| --- | --- |
| Module과 원통만으로는 실행 함수, queue, RAM 상태와 DB를 구별하기 어려움 | 코드 사각형, FIFO 기호, RAM Map 카드, 영속 DB 원통과 immutable DTO를 구별. 각 queue의 자료/수신자와 RAM model의 의미 writer를 표기 |
| A/B 차이가 모델 호출 개수나 상자 개수로 읽힐 위험 | A의 async workflow와 직접 binding aggregate, B의 producer별 queue/repository와 Fork–Join 및 feedback을 스타일과 실제 필수 경로로 표시 |
| 모든 worker의 긴 read/model 배선이 B의 중심 흐름을 가림 | 동일 adapter를 가리키는 R/M 명명 port와 async 요청/반환 범례를 사용. queue, candidate 게시, 결합과 재판단 경로를 중앙에 남김 |
| Coordinator가 언제 결합을 시작할지 추상적임 | 생산 경로별 응답 상태 또는 deadline, 호환 snapshot과 남은 예산으로 dispatch. 의미상 충분 여부는 Resolver에 남김 |
| 정정 검사와 상태 쓰기가 별개이면 옛 callback이 새 상태를 덮을 수 있음 | BeginRevision과 TryApply/TryPublish가 요청별 RevisionGate의 같은 짧은 lock을 사용. lock 안에서 I/O, queue와 모델을 await하지 않음 |
| 작업 queue가 가득 찬 동안 제어 loop가 완료를 못 읽으면 순환 대기 가능 | 제어 loop의 TryWrite와 유한 pending, 별도 Completion Inbox 및 두 inbox 처리 계약을 명시. capture callback은 의미 작업 대기를 기다리지 않음 |
| 후보 게시만 보여주면 Coordinator의 다음 실행이 저절로 시작되는 것처럼 보임 | 생산자별 RAM 게시 이후 Completion Inbox 통지와 Binding Work Queue 실행을 사건도에 표시 |
| reference가 해당 queue와 lock을 그대로 검증한 것처럼 확대될 위험 | QuickSet/Google/OmniParser에서 확인한 구조와 VIA 구현 가정을 분리. Channels와 Point-to-Point Channel은 구현 수단 참고로 연결 |

`generate_screen_voice_grounding_diagram.py --check`가 네 상세도의 SVG/draw.io 및 렌더 해시, 공통 30개 요소의 같은 배치, owner/직교 배선과 여섯 그림의 본문 등록을 통과했다. Chrome으로 네 3840×2160 PNG를 렌더링하여 text width/overlap/bounds 오류 0을 확인하고 각각 열어 읽었다. active Markdown 397개 local link, 용어와 QA registry 검사도 통과했다. DP-47 관련 원본과 editable figure 13개에서 가운데 점이 없음을 확인했고 대상 diff의 whitespace 검사도 통과했다.

구현 가능한 실행 계약을 설명하는 문서 작업이다. queue 용량, worker 수와 실제 provider capability를 측정하거나 SDK/DB engine을 최종 선정하지 않았다. local in-process task 가정은 Request Interpreter 내부에 적용하며 DP-42의 주변 서비스 수명을 결정하지 않는다. 제품 코드, Windows 연동, 모델 실행, 독립 리뷰나 품질 우위의 실증은 없다.

## 10. 구현 구조 한 페이지 비교와 기존 그림 보존 / 2026-10-15

사용자는 기존 비교 페이지에서 SW 구조가 충분히 보이지 않는다고 지적하고, §8.5의 구현 수준 A/B를 한 장에서 비교하도록 요청했다. 이전 그림을 지우지 말라는 후속 지시에 따라 `dp47-implementation-comparison-v1`을 별도 추가했다. 기존 MAIN, 배경과 네 상세도의 SVG/draw.io/PNG는 변경하지 않는다. 통합 PPTX의 기존 페이지도 수정하지 않고 편집 가능한 독립 PPTX 한 장을 추가한다.

상단의 같은 원본과 GroundingInput, 중단의 같은 위치에 놓인 제어 inbox 및 registry, 하단의 공통 채택을 기준으로 두 실행 경로를 비교한다. A의 직접 binding aggregate와 B의 생산자별 후보 repository가 정상 실행의 다른 중간 자료임을 보인다. B의 Coordinator를 의미 판단자로 읽지 않도록 코드의 실행 관리와 Resolver의 판단을 구별했다. 저장소 의존 선은 pub/sub가 아니며 완료 inbox와 Coordinator가 후속 실행을 지시한다.

기존 상세 생성물과의 동기화 검사, 새 scene의 owner 및 배선 검사, SVG/PNG 렌더 해시와 본문 삽입 검사를 적용한다. PPTX는 같은 scene에서 native 도형과 도형 자체 텍스트를 만들며 Calibri, 정수 pt와 한국어 설정을 적용한다. 실제 생성 및 렌더 확인은 문서 표현의 검증이다. 새로운 API 실험, 모델 호출, 구현, QA 측정이나 독립 설계 리뷰를 수행한 기록은 아니다.

로컬 검증에서 일곱 그림의 본문 등록과 생성물 동기화, 링크, 용어와 QA registry 검사가 통과했다. 새 SVG의 text width/overlap/bounds 오류는 0건이며, 최종 PPTX를 다시 읽어 3840×2160으로 렌더링하고 직접 확인했다. PPTX의 package와 geometry 검사 및 Calibri/정수 pt/한국어 검사도 통과했다. 그림을 통째로 붙인 raster picture 없이 실행 모듈 이름이 native 텍스트로 남는지 확인했다. 기존 DP-47 자산과 PPTX 61개가 이 변경 직전 `6901a1deb`의 파일과 byte 단위로 동일했다. PowerPoint 프로그램 자체의 실행 및 native 배치는 이 로컬 검사에 포함하지 않는다.
