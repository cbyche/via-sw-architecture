# 문제 3 교차 리뷰: 동시 사건 조사 담당

> 이 기록은 2026-10-16 문서 판본의 검토 이력이다. 기본 구조와 tactic을 나눈 2026-10-17 개정의 검토 상태는 [현재 리뷰](user-review-20261017.md)를 따른다. 아래의 이전 대안 번호나 강한 보완 비교 요구를 최신 기준으로 적용하지 않는다.

2026-10-16 / 독립 문서 및 공개 소스 표본 대조 / 제품 실행과 측정 없음

## 검토 범위와 판정

- [문제 3 본문](../03-screen-speech-grounding.md) 전체와 [source 원장](../../../../../../references/five-problem-study/screen-speech-grounding.md) 전체를 읽었다.
- 기존 [DP-47](../../../04-47-screen-voice-grounding.md)의 strong A/B, producer 소유와 원본 확장, cheap hybrid와 부분 정정 계약을 대조했다.
- 공개 source는 Google Live Console `ControlTray.tsx`, OmniParser server, Psi `Fuse.cs`, ReTiCo `abstract.py`의 문서에 고정된 commit을 직접 재조회했다. QuickSet §3과 InproTKS §2~5 원 논문도 다시 읽었다. 나머지 모든 source를 재조회한 전수 검토는 아니다.

**현재 P1/P2 발견 없음. P3 두 건도 작성자 수정 후 재확인하여 닫았다.** 기존 47을 다른 이름의 새 DP로 중복 승격하지 않았고, A의 원본 read/cache/patch와 B의 competing/unknown 후보 및 원 생산자 확장 경로를 유지한다. 증분 실행과 시간 대기를 A/B와 조합 가능한 축으로 분리한 판단이 타당하다. 아래 두 가지는 범위를 더 명확히 한 P3 문서 보완 기록이다.

## 발견과 수정 권고

| ID | 우선도 | 위치 | 관찰과 영향 | 권고 | 상태 |
| --- | --- | --- | --- | --- | --- |
| G-R1 | P3 | 본문 §2.1 조건 1, §4 ObservationWindow | ‘발화 시작 이후’와 ‘발화 구간’으로만 입력 범위를 읽으면 Select then Speak에서 발화 직전의 선택 또는 pre-roll 관측을 배제한 것처럼 보인다. 다른 문단은 실제 selection과 이전 binding을 허용하므로 설계 자체의 기능 누락으로 판정하지 않았다 | 첫 조회에서 여전히 유효한 발화 전 선택과 실제 허용/보존한 pre-roll을 양안 공통 입력에 명시. 상시 수집이나 미관측 과거 복원을 요구하지 않음 | 수정 재확인, 닫음 |
| G-R2 | P3 | 본문 §7의 Inpro commit 설명 | InproTKS §2는 add/revoke/commit을 소개하지만, 확인한 논문 본문에서 ‘미래 수정이 없음’의 구체 규약을 직접 설명하지 않는다. ReTiCo 코드의 committed flag는 VIA 영속 채택과 다르다는 source 원장 설명이 더 엄밀하다 | ‘해당 처리자의 가설 확정 표시이며 VIA의 영속 채택이나 이후 사용자 정정 금지가 아니다’로 한정 | 수정 재확인, 닫음 |

## 중점 검토 결과

1. **원본 기록과 정렬 및 의미 판단:** ObservationWindow는 source 시각, 도착 시각과 gap을 보존하며 정답 target을 미리 넣지 않는다. Psi의 `OriginatingTime` 보간과 자료 대기를 의미 정답의 증거로 확대하지 않는다. ReTiCo 생성 시각을 sensor 시각으로 쓰지 않는 한계도 명시했다.
2. **source의 실제 범위:** Google Live Console은 별도 audio/frame의 같은 client 공급만 확인한다. temporal binding 완성 사례로 쓰지 않는다. OmniParser는 screen 요소 목록의 interface 근거이며 모든 PDF object나 원형 집합 지원을 주장하지 않는다. QuickSet의 pen/map와 VIA의 free pointing을 구별한다.
3. **대안 강도:** A의 helper와 부분 patch, B의 source 확대와 재생산, native 경로의 모델 생략을 허용한다. B의 원본에서 final target을 직접 만드는 경로는 hybrid로 공개한다. process와 helper 수를 구조적 품질 이익으로 바꾸지 않는다.
4. **QA01/03:** 실제 첫 내용 표시 전에 필요한 판단과 상세 비교 후 처리를 구별한다. 후보 준비와 재생산이 첫 응답 경로에 있는 조건에서만 차이를 주장하고, 두 지연 지표의 중첩을 합산하지 않는다.
5. **QA02:** source coverage와 시간 정렬을 정답으로 취급하지 않는다. 모호한 입력의 제외와 불필요한 질문의 실패를 구별하고 B 확장 뒤 동점 가능성을 남겼다.
6. **QA05/06:** producer 원본 판단과 출력, 결합과 폐기 비용을 포함한다. B도 raw를 유지하며 cloud memory를 로컬 RAM으로 옮기지 않는다. 반복과 cache hit, 관측 부족과 철회 장면이 동일 조건에 남아 있다.

## 검토 한계

공개 코드의 해당 경로와 설계 인과를 읽은 결과다. native API 기능, 시계 오차, 한국어 지칭 성공, 실제 모델 동시성, 512 MB 충족과 품질 우열을 확인하지 않았다. P1/P2 없음은 주요 DP의 차별화 요건이나 품질 가설의 실증을 뜻하지 않는다.
