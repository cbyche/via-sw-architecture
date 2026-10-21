# 공통 QA 합성 원장과 설명서

`COMMON-QA-SYN-1.0` / `SYNTHETIC_AUTHORED_NOT_MEASURED`

- [대화형 설명서](outputs/01a15d51/VIA-QA-common-guide.html): 비교 결과, 사례군 및 입력과 오류를 고르는 원장.
- [Excel 원장](outputs/01a15d51/VIA-QA-common-synthetic.xlsx): 요약, 공통가정, 사례군, 시간, 목표, 지칭판정, 의미판정, 비용, 메모리, 변경과제, 입력, 원본, 단계의 13개 시트.
- [전체 원본](synthetic-data.json), [집계](summary.json), [판본과 해시](manifest.json), [계산 검증](validation.json), [발행 확인](publication-validation.json).
- [공통 계약과 해설](../../architecture/12-decisions/decision-packages/qa-common-synthetic-20261021/README.md).

120개 에피소드에 각 2개 입력이 있다. 세 구성은 같은 입력과 정답을 사용한다. 정확도는 작성한 최초 채택 사례를 집계하고, 시간과 비용 및 RAM은 단계별 가정을 계산한 값이다. 실제 실행 또는 OS 계측 원본을 포함하지 않는다.

## 재생성

저장소 루트에서 Node.js로 다음 명령을 실행한다.

```sh
node scripts/presentations/build_common_qa_synthetic.mjs --data-only
node scripts/presentations/build_common_qa_synthetic.mjs
```

첫 명령은 외부 의존성 없이 JSON과 Markdown 및 HTML을 생성한다. 두 번째 명령은 `@oai/artifact-tool`을 사용해 수식이 포함된 xlsx를 만들고, 13개 시트 미리보기와 수식 재계산을 검증한다. `@oai/artifact-tool`의 설치 경로는 `VIA_ARTIFACT_MODULE_BASE`로 지정할 수 있다. 인수는 해당 `node_modules`를 찾을 수 있는 디렉터리의 절대 경로다.

원본은 [시나리오](../../../scripts/presentations/common_qa_synthetic_scenarios.mjs), [계산과 workbook 생성기](../../../scripts/presentations/build_common_qa_synthetic.mjs), [설명서 생성기](../../../scripts/presentations/render_common_qa_synthetic_report.mjs) 및 [변경 과제](../../architecture/12-decisions/decision-packages/qa-common-synthetic-20261021/change-spec.json)에 있다. 산출값을 직접 고치지 않고 해당 원본을 수정한 뒤 다시 생성한다. 발행 후 사례나 가정을 바꿀 때는 새 판본과 변경 사유를 남긴다.

`previews/`는 검토한 시트 상단의 PNG다. 전체 메모리 경계는 JSON에 보존하고 Excel에는 각 에피소드의 가장 큰 두 동시 상태를 싣는다. 최초 표시와 전체 의미 확정은 서로 다른 열이며, 목표의 처리 종료와 최종 의미 일치도 구분한다.
