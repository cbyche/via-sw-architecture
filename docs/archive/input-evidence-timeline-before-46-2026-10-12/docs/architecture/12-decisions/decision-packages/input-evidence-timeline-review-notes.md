# 관측 타임라인 초안 검수와 재개 기록

> 2026-10-09 / 작성자 자체 검토 / 독립 reviewer·사용자 승인·구현·측정 없음
> [본문](./input-evidence-timeline-review.md) · [04-40](./04-40-common-execution-contract.md)

## 작업 범위와 재개

사용자는 새 관측 타임라인 구조 탐색을 04-40의 공통 실행 계약에 맞춰 검토 가능한 형태로 정제하도록 요청했다. 새 DP 번호/승격, 구현·측정 또는 commit/push는 요청하지 않았다. 이 초안, 별도 SVG/draw.io/PNG와 전용 생성·렌더 도구만 추가한다. 다른 세션의 40/41·작업계획·공유 생성기·발표 package와 초기 50/`work/` 변경에는 쓰지 않는다.

다시 시작할 때 04-40의 실제 현재 내용과 이 본문을 읽는다. 새 전제는 로컬 VIA·로컬 VAD·cloud voice/transcription/semantic이다. 이전 on-device 모델 배치와 32의 제외 결론을 새 선정으로 바꾸지 않는다. 현재 원본/조회 A/B는 **사용자 검토 전 후보**이며, 새 주요 DP로 유지할 중요한 품질 효과는 미확인이다.

## 설계 대조와 실제 수정

- 04-40 §1/2/4/5/7/8/12에 맞춰 정식 Component 소속, 임시 상태와 원본, code/model 경계, 호출 역할·usage, 정상/정정/취소/실패/늦은 결과와 V-01~13을 대조했다.
- 앞선 채팅의 입력별 복합 기록을 독립 C로 고정하지 않았다. A/B의 view tactic 또는 A의 소유 인수로 환원될 수 있음을 기록했다. 모델 직접 시간 결합은 capability 미확인/41·43 중복으로 주 비교에서 분리했다.
- A에 zero-copy·분할 저장·부분 조회, B에 facade·batch reference·cache를 허용했다. 동일 process/내구성 조건과 총예산으로 원본 생존을 비교하며 단순 process 분리의 이익을 재승격하지 않는다.
- 단어 span timing, resampling mapping·capture 구간·clock 오차·watermark/gap을 구별했다. provider 수신 시각이나 VAD 종료를 단어 시각으로 쓰지 않는다. helper 추가와 source/API 지원은 미결이다.
- E1~11에서 인수/읽기·재지칭·late revision·전송 후 정정·철회·원본 소실을 본문과 메인에 대조했다. 원본 확보와 의미 제안/채택을 구별했다.
- 초기 렌더는 A의 Module 중앙 이름과 내부 설명 두 곳이 겹쳤다. 이름 전용 Module로 수정하고 설명을 흐름/본문으로 옮겼다. 최종 렌더에서 해당 겹침을 재확인했다.

## 검증과 한계

생성기와 renderer는 다음 파일만 갱신한다. 기존 다른 DP의 source/generator를 import하거나 통합 PPTX를 생성하지 않는다.

```bash
.venv/bin/python scripts/architecture/generate_input_evidence_timeline_diagram.py --check
NODE_PATH=<workspace dependency node_modules> node scripts/architecture/render_input_evidence_timeline_diagram.cjs
```

[생성기](../../../../scripts/architecture/generate_input_evidence_timeline_diagram.py)의 검사는 SVG/draw.io source 일치, XML, ID, Module/상태 owner 포함과 직교 연결의 실제 끝점이다. [렌더 도구](../../../../scripts/architecture/render_input_evidence_timeline_diagram.cjs)는 로컬 headless Chrome으로 2560×1440 PNG, 글자 폭/캔버스/서로의 겹침과 [SVG/PNG hash 기록](./diagrams/input-evidence-timeline-structure-render.json)을 만든다. PNG를 직접 열어 한글, 상태 소속과 흐름을 육안 점검했다. 자동 검사는 모든 선 교차나 구조 중요성의 증명이 아니다.

실행 결과: active Markdown local link 154개 파일, 정식 Component/용어, QA catalog 검사를 통과했다. 생성물 일치/owner/연결 끝점 검사와 최종 2560×1440 렌더의 글자 geometry 검사도 통과했다. Git의 tracked diff whitespace 검사도 통과했다. concurrent writer가 수정하는 다른 자료의 오류는 이 초안의 수정 범위로 흡수하지 않는다. API 실행, provider 문서/가격 검증, 독립 Architecture review, 실제 시간 정렬/semantic 품질·요금 측정은 수행하지 않았다.
