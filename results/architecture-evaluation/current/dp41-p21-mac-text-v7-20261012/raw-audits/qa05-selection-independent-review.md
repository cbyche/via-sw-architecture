# QA05 선택 원장 독립 검수

상태: **PASS_WITH_DECLARED_EVIDENCE_SCOPE**. UTC 2026-10-12T00:13:42.092923+00:00. API 0회, 원본 변경 0개.

선택 manifest의 reviewed-result 12개와 기존 `selected_change_records`가 반환하는 12개가 일치한다. hash로 지정된 두 LLM agent의 책임 검토, 변경 전/후 53개 source, patch, 원시 result와 회귀를 대조했다. 이 검수의 작성자는 변경 patch 작성자이므로 기존 두 독립 리뷰의 개수 판정에 추가 투표하지 않았다.

| 방안 | M01 | M02 | M03 | M04 | M05 | M06 | 합계 | 평균 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| A | 1 | 1 | 1 | 4 | 2 | 0 | 9 | 1.5 |
| B | 1 | 1 | 1 | 4 | 2 | 0 | 9 | 1.5 |

대조 1296개 중 실패 0개. 같은 baseline에서 독립 생성된 12개 변경 사본이며, 초기 result의 AWAITING 상태와 count=None은 보존되고 새 reviewed-result만 선택됐다. 원래 M04 사본과 revision collision 실패는 보존됐고 선택/집계에 중복되지 않는다. Typed B의 binder b.py는 baseline과 byte 단위로 동일하다.

M04는 수정 코드의 지정 모델 port 기능 검사, 성공/실패 두 분기, AFTER/USE_RESULT 및 실제 Request revision/Task/Execution lineage 회귀가 통과했다. 별도 실제 Qwen 신규 자연어 probe는 A/B 모두 **FAIL**이다. manifest와 두 책임 리뷰가 이를 별도 실제 모델 실패로 유지하며 P21 정확성/비용에 더하지 않는다. 따라서 평균 1.5개는 검토한 기능 책임 변경 수이며 신규 자연어 기능의 실제 모델 end-to-end 성공을 뜻하지 않는다.

M05는 실제 새 Task 질문 owner가 발급한 token을 실제 게시하고 snapshot한 뒤 다른 PID의 fresh process로 같은 token을 복원했다. A/B 모두 actual ANSWER 명령 1회, 손상 token/중복 거절을 확인했다. generic snapshot/restore를 그대로 쓰므로 U14를 추가하지 않은 기존 두 리뷰와 일치한다.

M06은 before/after source 53개와 빈 patch가 완전히 같다. 원본/파생물/대기 후보의 사용 차단, 정직한 처리 receipt, 다른 자료/선호 유지, cache/late publication/consent 복원 회귀가 통과했다. exact S1-03 입력의 C1~C6과 사용자 partition/원본 M언어/owner ref+revision/철회 삭제를 A/B baseline과 unchanged job source에서 모두 확인했다. API 0회의 지정 모델 출력 기능 증거이며 실제 모델의 자유문 정확성이나 P21 모집단 추가로 사용하지 않았다.

각 파일의 SHA, reviewer ID, patch/source digest, 역할별 증거 한계와 PID/token hash는 JSON에 보존했다. 확정 P1/P2는 없다.
