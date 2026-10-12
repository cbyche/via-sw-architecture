# V7 최종 원시 재계산 독립 검수

상태는 `PASS_WITH_DECLARED_EVIDENCE_SCOPE`다. 원시 실행과 승인된 최종 판정에서 여섯 QA를 다시 계산한 174,750개 검사가 모두 통과했다. API 호출, 입력 수정과 후보/동결 소스 수정은 모두 0이다. calculator는 후보, evaluator, metrics 모듈을 import하지 않고 표준 라이브러리로 실제 endpoint와 causal DAG, token 원장, OS 동시 메모리, 선정 변경 실험을 재구성했다.

| 관측 | A | B |
| --- | ---: | ---: |
| QA01 조건 정답 | 400/630 (63.492063%) | 359/630 (56.984127%) |
| QA02 오답/미전달 5초 대체 평균 | 6.420109s (n=135) | 6.143911s (n=135) |
| QA02 정답 첫 반응만 평균 | 7.282318s (n=84) | 6.906518s (n=81) |
| QA02 오답 포함 실제 첫 반응 평균 | 5.663224s (n=128) | 5.034161s (n=133) |
| QA03 미완료/30초 초과 30초 대체 평균 | 23.533702s (n=95) | 25.638574s (n=95) |
| QA03 완료 관측만 실제 평균 | 18.328230s (n=46) | 20.535022s (n=35) |
| QA04 P21 1회당 실제 VIA semantic MSRP | $0.132039525 | $0.143066100 |
| QA05 수정 기능 책임 수 평균 | 1.5 | 1.5 |
| QA06 다섯 회 전체 phys_footprint 동시합 peak | 1014.407692MiB | 648.532280MiB |
| RSS 동시합 peak | 1028.218750MiB | 665.359375MiB |

0–6점은 양안 모두 QA01~06 순서로 `0, 0, 0, 6, 4, 6`이다. QA01~04 qualification은 양안 모두 실패다. 낮은 모델 비용의 6점이 정확성 보호 조건 통과를 뜻하지 않는다. QA05 보호 조건과 Mac footprint 참조 예산은 양안 통과다. 올바른 5초 초과 반응의 실측은 clipping하지 않았고, 누락/오답 대체와 정답만 평균의 서로 다른 분모를 검증했다.

검수 모집단은 본 trial 210개, 조건 1,260개, 반응 270개, 사이클 190개, 선정 변경 실험 12개다. 원판정 210개와 승인 override 120개 trial을 연결했다. 다섯 paired 반복의 원값/차이/판정도 모두 일치하며, 현재 계약의 consistent practical difference 판정은 비교된 모든 축에서 false다. QA05는 양안 M01~M06의 책임 수 `1,1,1,4,2,0`이 같아 평균 1.5다.

동결 53개 파일과 네 evaluator 세대의 hash를 확인했다. 첫 SYSTEM/evaluation_view는 같으며 V4 validator는 V3와 같다. A preserved retry에서 병합한 4행의 원형 canonical SHA와 최종 내용, 모든 origin ledger SHA를 대조했다. 추가 M04 실제 자연어 Qwen/판정의 A/B FAIL은 scripted 기능 PASS 및 본 P21과 분리 보존되어 있다.

이 감사는 산술 및 provenance의 일치 검수다. 의미 정답은 고정 oracle와 trace 검수를 거친 판정에 의존하며 추가 독립 인간 판정이 아니다. QA05 patch author의 산출물 무결성 확인은 책임 수의 제3 독립 투표가 아니다. Mac 단일 worker의 OS 원지표 및 모사 음성은 Windows commit 또는 실제 음성 인식/상호작용 성능을 검증하지 않는다.

- [원시 재계산 JSON](final-independent-review.json)
- [실행 스크립트](recompute-final-independent.py)
- [여섯 QA 최종값](../final/qa-results.json)
- [원시 자료 재사용 안내](raw-data-guide.md)

JSON SHA: `d373aea6171da04e07ba25b2921581a3b9f3e3750dbfb0f3b248542fd45e2262`
동결 source digest: `44f725057e7589c35aa59cb23537a9a8c59f48a40944b70f03e3906e931f08ce`
