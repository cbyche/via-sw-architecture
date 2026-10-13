# DP-47 사고실험과 후보 탐색

> 2026-10-13 / 작성자 사고실험 / 실행, 측정 결과 아님
> [완성 설계](./04-47-screen-voice-grounding.md) / [구조 조사](./dp47-structure-references.md) / [리뷰 기록](./dp47-design-review.md)

## 1. 단순 구현에서 출발한다

가장 단순하게는 발화가 끝나면 원음 또는 전사, 확보한 screenshot들, pointer/button 기록과 앱 정보를 한 request에 넣어 모델에 “사용자가 지정한 대상을 연결하라”고 요청할 수 있다. 코드가 최종 결과를 검사한 뒤 후속 요청에 사용한다.

그 다음 실제 장면을 넣는다. 마지막 screenshot만 넣으면 PDF 8페이지 표가 사라진다. 마지막 pointer 좌표만 넣으면 두 graph를 둘러싼 경로가 사라진다. 전사 도착 당시 foreground만 넣으면 mail이 대상이 된다. 모든 frame을 넣더라도 단어 시점이 불확실하고 UIA가 늦으면 의도한 대상을 틀릴 수 있다. “전체 자료를 보냈다”는 성공 조건이 아니다.

필요한 보완은 원래 시각과 불확실성, 창/좌표 기준, selection snapshot, 원본 범위와 누락, 지칭별 결과 및 정정 범위다. 이는 양안 공통으로 강화한다. 순진한 공동 입력 자체를 약한 비교안으로 남기지 않는다.

## 2. 탐색한 여덟 방향과 남긴 이유

| 탐색 방향 | 실제 메커니즘 | 검토 결과 |
| --- | --- | --- |
| 1. 전사 도착 때 현재 화면 조회 | 현재 screenshot와 foreground에서 대상 결정 | 과거 페이지와 여러 지칭을 잃어 필수 장면을 처리하지 못한다. 채택하지 않음 |
| 2. 원본을 모두 한 번에 전달 | 한 공동 해석에서 전체 binding 생산 | 초기 출발점. 예산, 누락, 부분 정정을 보완한 **A**로 발전 |
| 3. frame 간격, crop, cache 최적화 | acquisition 또는 동일 해석기의 입력 감소 | 양쪽에 허용하는 tactic. 이것만으로 별도 대안 아님 |
| 4. 독립 recorder가 원본 소유 | 원본의 process lifetime과 장애 범위 변경 | 32에서 제외된 축이다. 이번 정상 지칭 문제의 주력 pair로 복원하지 않음 |
| 5. 미리 scene 관계를 게시하고 후속 요청이 조회 | 공유 screen read model의 생산/소비 수명 변경 | 46/이전 screen 탐색에 가까워 이번 새 출발의 pair로 사용하지 않음 |
| 6. screen 1위 후보를 고정한 직렬 pipeline | screen target 확정 → 발화 연결 | circle/navigation과 지칭 범위가 서로 의존한다. 늦은 언어로 후보를 복구할 수 없어 약한 B로 판단 |
| 7. 복수 부분 후보를 교환하며 결합 | 별도 생산자가 근거 있는 후보를 관리, 결합자가 후보에 근거해 지칭 제안, 부족하면 원 생산자 재판단 | **B**로 발전. 원본 보존, 후보 확장, revision 충돌과 왕복 제한을 추가 |
| 8. 사용자에게 먼저 overlay target을 확정받음 | 사용자의 명시적 선택 완료가 대상 입력이 됨 | 모호할 때 공통 확인 tactic으로 유효. 모든 자연 지칭마다 추가 선택을 요구하면 지원/사용자 수고를 바꾸므로 주요 pair로 채택하지 않음 |

두 안은 **같은 발화 구간에서 현재 지칭을 해결**한다. B를 발화 전에 scene을 만드는 공통 memory service로 확대하지 않는다. 전사, 후보 생산을 일찍 시작하는 일은 양안 모두 가능하다.

## 3. 동일 사건을 넣은 열여섯 사고실험

`충족 경로`는 설계에 해당 처리가 있다는 뜻이다. 실제 모델이 성공했다는 PASS 판정이 아니다. 사람이 아는 A/B/C/D 대상 기호는 후보 입력에서 제외한다.

| # | 사건, 기대 결과 | 강한 A의 처리 | 강한 B의 처리 | 검토에서 얻은 결론 |
| --- | --- | --- | --- | --- |
| E01 | 사전 selection이 남은 spreadsheet, PDF가 foreground, 클릭 없이 범위 지칭 | 처음 읽은 selection 상태와 당시 frame/path를 공동 판단 | native 범위 후보와 pointer-window 후보를 screen 생산자가 함께 제출; 언어와 결합 | foreground 하나로 후보를 자르지 않는다. 선택이 실제 노출되지 않으면 visual 범위의 지원 수준을 명시 |
| E02 | PDF 8페이지 표 지칭 후 drag로 9페이지 이동 | 이동 전 원본과 표현을 binding 근거로 남김 | 표 후보는 이동 전 frame을 참조, 후속 후보는 다른 관측을 참조 | 현재 window handle만으로 object identity를 보장하지 않는다 |
| E03 | drag 경로가 graph 위치를 지나감 | button과 화면 변화, 발화를 함께 보아 navigation 후보 판단 | screen 생산자가 navigation/designation 경쟁 후보를 보존; language 정보를 받은 뒤 재판단 | 경로에 포함된 모든 대상이 비교 대상이라는 규칙 금지 |
| E04 | button up 이후에도 render 중 | 완전한 이후 frame이 있는지 read하며 불확실하면 기다림/확인 | 이동 뒤 자료의 coverage가 부족하면 후보를 미완료로 두고 새 frame으로 재생산 | mouse up을 화면 완료로 취급하지 않는다. 안정 frame 감지는 의미상 완료의 증명 아님 |
| E05 | 버튼 없는 circle, “이 두 graph” | 경로 전체와 당시 page에서 집합/경계 판단 | one-of 경쟁 후보와 set 후보를 구별, 두 대상 범위를 screen 생산자가 제시 | native selection이 없다는 이유로 circle을 미지원 처리하지 않는다 |
| E06 | “graph는 아래 것만, 나머지는 그대로” | 이전 binding와 원음/새 근거로 graph 관계 patch, 나머지도 영향 점검 | mention의 변경 범위와 새 target 후보를 재생산; 결합자가 기존 binding별 유지/교체 제안 | B만 부분 정정 가능하다는 주장을 기각. dependency는 의미 판단의 완전성을 증명하지 않는다 |
| E07 | 정정 전 모델 결과가 뒤늦게 도착 | 옛 request/input version 제안 거절 | 옛 producer/binding version 결과도 거절 | capture, 취소와 최신성 adoption은 공통, B의 추가 version 비용 공개 |
| E08 | 마지막 전사 도착 전에 mail 활성화 | 당시 evidence 범위에서 해석 | 후보 source scope를 당시 frame에 묶어 결합 | 늦은 전사와 현재 foreground를 연결하지 않는다 |
| E09 | 단어 시각이 없고 발화 안에서 두 페이지가 바뀜 | 넓은 가능한 audio/frame 구간을 함께 읽고 필요하면 확인 | mention 시간 후보를 넓게 남기고 target 후보 확장, 결합, 미해결이면 확인 | 평균 전사 지연을 단어 시각으로 확정하지 않는다. B가 더 좁은 구간을 공짜로 아는 것은 아님 |
| E10 | UIA에는 PDF container만, 두 graph는 pixels에만 존재 | image 범위를 binding으로 생산, native ID 없이 근거 전달 | screen 생산자가 visual target 후보를 생산, fusion이 그 ID를 참조 | graph 후보/전체 문서 candidate를 구분. 후보 vocabulary 밖의 영역도 생산할 수 있어야 함 |
| E11 | 제목 때문에 필요한 범위가 pointer 근처 crop 밖에 있음 | 주변 context 및 full permitted frame을 추가 read | screen 생산자에 coverage 확장 요구, crop의 원본 좌표를 유지 | crop은 양안 tactic. 잘린 정보로 실패하면 범위 확대 비용과 재호출을 남김 |
| E12 | B가 처음에는 graph 둘을 하나의 figure로만 생산 | 해당 없음. A도 둘을 하나로 오해할 수 있으며 재판단 필요 | “두 graph”와 불일치를 받아 producer가 둘의 extent를 새로 생산; 불가하면 확인 | 복수 후보만으로 정보 손실 해결 불가. 후보 없는 대상의 재생산을 필수 계약으로 추가 |
| E13 | 두 창의 UIA 후보와 pixels가 다른 시점의 객체를 가리킴 | source별 시각/오차를 보고 모순을 해소하거나 확인 | 후보 출처/version의 불일치를 명시, 같은 객체로 병합하지 않고 재생산 | source 합의는 독립 증거의 수가 아니다. 같은 frame 파생 결과도 중복 근거로 세지 않음 |
| E14 | 새 발화 “아니, 앞 페이지 아래 표였어”로 과거 binding 정정 | 관련 과거 frame과 전체 요청 관계 재판단 | 수정된 mention의 의존 범위를 넓히고 과거 target 후보 재생산 | 정정이 항상 직전 graph 하나에만 국한된다는 가정 제거 |
| E15 | frame 누락, memory 한도, source permission 철회 | 필요한 원본/사용권이 없으면 해당 binding 미해결, 원본과 사본 회수 | 같은 처리 + 파생 후보/참조/교환 중 자료의 사용 차단 | B가 작은 후보를 남겨 삭제된 원본의 사용권을 우회하지 않는다 |
| E16 | 후보 왕복이 반복되거나 producer crash, raw evidence owner/PC crash | 제한 내 재호출; 원본 소실은 다시 관측/확인 | 일부 생산자를 원본에서 재생산; 왕복 한도 뒤 질문/중단; 원본 소실은 동일 | 별도 logical producer가 자동 process isolation/내구 복구 이익을 주지는 않는다 |

화면/DPI/monitor 좌표 변화와 시간 불확실성은 E01/E02/E11/E13의 관측 계약에 포함한다. 좌표 변환이 불확실하면 화면의 같은 숫자 좌표를 재사용하지 않는다.

## 4. 가장 싼 전환으로 반박한다

| 시도 | A에 가장 유리한 보완 또는 B에 가장 유리한 축소 | 그래도 남는 차이 또는 기각 조건 |
| --- | --- | --- |
| A에 screen helper 추가 | 필요한 범위의 target 설명을 공동 모델에 제공, cache 재사용, 일부 정정만 patch | helper가 권고이고 공동 해석기가 원본에서 새 target을 직접 만들면 A다. helper 결과만으로 대부분 장면을 같은 비용에 해결하면 B의 추가 계약 이익은 약해진다 |
| A에 binding별 dependency 기록 | source/mention/binding 간 영향 관계를 기록해 부분 재처리 | 이 기록은 A에도 허용한다. B의 근거는 독립 생산 후보의 필수 소비, 재생산 계약이며 dependency 자체가 독점 이익이 아니다 |
| A에 B 엔진을 adapter로 삽입 | 외부 GroundingProposal와 Request Controller를 유지 | 주변 integration은 유지 가능. 내부에는 candidate identity/coverage, 원 생산자 revision, 재생산 요청, 여러 후보 version의 결합 검사가 새로 필요하다. 분할 prompt만으로 충분하면 major 차이를 기각 |
| B의 resolver에 모든 raw를 직접 전달 | candidate miss 때 전체 원음과 frame으로 새 target/binding 생산 | producer의 target 생성, revision을 건너뛰면 A형 직접 해석을 추가한 hybrid다. 유지한다면 후보 생성 경로의 우회 조건, 대안 비용을 별도로 기록해야 함 |
| 요청별 A/B 선택 | 단순 장면 A, 복수 source/정정 장면 B | 가능하다. 외부 계약이 같으므로 전체 시스템 교체를 주장하지 않는다. B의 내부 기제 개발/검증과 route 판단 비용은 남는다 |
| 요청 종료 후 교체 | RAM candidate/state migration을 없앰 | 운용 중 데이터 이전 비용은 작다. 구조 전환비는 interpretation 핵심의 자료 계약과 재판단 경로 설계다. 업무/DB 전체 redesign은 필요 없음 |

**현재 판정:** 요청 해석 Subsystem 안의 실질적인 대안으로 사용자 검토에 제시한다. 시스템 전체의 양립 불가능한 구조라고 주장하지 않는다. 두 안의 조합 가능성은 높으며, 이 점 때문에 major DP로서의 최종 강도는 품질 영향과 구현 전환 검토를 통해 판단해야 한다. 이번에는 충분히 설명할 수 있는 두 설계를 완성했으며 final winner나 되돌림 비용의 실증을 완료하지 않았다.

## 5. 어느 주장부터 반증해야 하는가

1. A의 동일 입력, 선택적 재처리만으로 B의 정확성, 사용자 수고와 재사용 이익을 얻으면 B의 주요 설계 가치가 줄어든다.
2. B의 candidate miss, 재생산과 cross-modal 왕복이 빈번해 공동 해석보다 입력/호출/시간이 늘면 B의 비용 이익 가설이 무너진다.
3. A의 자유로운 공동 원본 해석이 실제로 정보 손실을 줄이지 못하면서 큰 media 전달과 정정 비용만 남기면 A를 선택할 조건이 좁아진다.
4. B의 중간 계약이 ordinary adapter와 같은 수준으로 싸게 추가, 제거 가능하고 중요한 QA 차이도 작으면 major pair 대신 계열 내부 설계 선택으로 다룬다.
5. 양쪽 API가 필수 장면의 근거를 받거나 참조할 수 없으면 구조 비교 전 dependency capability 조건을 고쳐야 한다. 모델 내부의 새 학습을 이미 가능한 기능처럼 적지 않는다.

반증을 실제로 실행하기 위한 모집단, 모델, budget 수치와 평가기는 이번 문서에서 동결하지 않는다. P21 기존 결과나 다른 DP 수치를 이번 pair의 값으로 옮기지 않는다.
