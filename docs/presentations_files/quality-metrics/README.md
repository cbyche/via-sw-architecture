# QA 목표 근거와 채점 부록

기존 [품질 요구사항 요약](../quality-attributes.md)의 10개 QA를 한 장씩 설명하는 발표용 부록이다. 기존 목표와 순위를 보존하며 Architecture baseline, 실제 측정 계약과 DP 선택을 변경하지 않는다.

- [편집 가능한 PowerPoint, 10장](VIA-quality-metrics.pptx)
- [각 장의 PNG와 미리보기](index.html)
- [전체 근거와 정확한 범위 및 측정 규칙](../quality-metrics.md)
- [문안과 구간의 편집 원본](quality-metrics.json)
- [기존 상세 시험 설계](../quality-attributes/measurement-design.md)
- [제작 검사와 독립 검토 기록](verification.md)

모든 표, 설명과 계산은 PowerPoint의 native text와 table이다. PPTX에 슬라이드 PNG를 붙인 방식이 아니다. PNG는 동일한 최종 PPTX를 다시 가져와 렌더링한 미리보기다. 16:9, 1920 × 1080이며 글꼴은 Apple SD Gothic Neo를 사용했다. 같은 글꼴이 없는 장치의 배치는 실제 PowerPoint에서 확인해야 한다.

## 읽는 방법

개선형 지표의 1~6점은 목표 충족이다. V-03, V-09와 V-10은 6점만 목표 충족이며 나머지는 모두 실패 규모의 진단이다. 표의 붉은 칸과 목표 열이 이 차이를 나타낸다. 등급 수치를 QA 사이에서 직접 비교하거나 평균 및 가중합으로 합치지 않는다.

목표의 출처와 등급 구간을 나누는 작성자 규칙은 구분했다. V-02의 0.5회, V-05의 4초, V-06의 12GB와 다른 상향 참조는 실제 달성 가능한 최저값으로 입증된 수치가 아니다. 논문이 정한 점수 경계로 소개하지 않는다. 시험 전에 하한과 자원 조건을 동결하며 이후 결과에 맞춰 구간을 이동하지 않는다.

## 생성과 확인

저장소 원본은 `scripts/presentations/generate_quality_metrics.mjs`다. Presentations 스킬의 bundled Node와 artifact-tool을 사용하는 새 private build 디렉터리에 생성기를 복사하고 `node_modules`를 연결한다. `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`과 `RUNTIME_NODE_MODULES`에 해당 환경의 절대 경로를 설정한 뒤 생성기에 저장소 경로와 build 경로를 전달한다. finalizer는 candidate와 별개의 최종 PPTX를 만들고 package, layout, fonts 및 재가져오기를 검증한다.

다음 검사는 범위의 중복 및 누락, 목표 보존, 경계 반올림 사례, 실제 7개 행과 native table 및 발표자 노트와 이미지의 일치를 확인한다.

```sh
python3 scripts/presentations/check_quality_metrics.py
python3 scripts/presentations/check_quality_attributes.py
```

최종 10개 PNG를 각각 확인했다. 파일 구조와 렌더 검사는 실제 후보 성능, 목표의 실현 가능성 또는 Microsoft PowerPoint 앱에서의 동작을 입증하지 않는다. 현재 구현 및 측정 결과는 없다.
