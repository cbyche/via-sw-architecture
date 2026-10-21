from pathlib import Path
import json, html
HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'INTENT.md').exists())
OUT=ROOT/'docs/presentations_files/dp44-synthetic-backdata-20261022'
d=json.loads((OUT/'synthetic-data.json').read_text());a=d['summary']['A'];b=d['summary']['B']
rel='../../../../../../presentations_files/dp44-synthetic-backdata-20261022'
metrics=[('QA01 화면 표시','qa01_display_s','초'),('QA01 음성 출력','qa01_voice_s','초'),('QA02 지칭 대상','qa02','비율'),('QA03 VIA 처리','qa03_s','초'),('QA04 예상 변경 모듈','qa04_modules','개'),('QA05 전체 VIA 모델 비용','qa05_usd','USD'),('QA06 최대 로컬 RAM','qa06_mb','MB'),('QA07 요청 의미','qa07','비율')]
def fmt(v,k):return f'{v*100:.2f}%' if k in ['qa02','qa07'] else f'{v:.6f}' if k=='qa05_usd' else f'{v:.3f}'
table='\n'.join(f'| {label} | {fmt(a[key],key)} | {fmt(b[key],key)} | {unit} |' for label,key,unit in metrics)
ft=[]
for f in d['families']:
    aa=next(x for x in d['family_summary'] if x['family']==f['id'] and x['option']=='A');bb=next(x for x in d['family_summary'] if x['family']==f['id'] and x['option']=='B')
    ft.append(f"| {f['id']} {f['name']} | {aa['voice_s']:.3f} / {bb['voice_s']:.3f} | {aa['via_s']:.3f} / {bb['via_s']:.3f} | {aa['cost_usd']:.5f} / {bb['cost_usd']:.5f} |")
scenes='\n'.join(f"| {f['id']} | {f['input']} | {f['name']} |" for f in d['families'])
change='\n'.join(f"| {cid} | {next(x['count'] for x in d['changes'] if x['id']==cid and x['option']=='A')} | {next(x['count'] for x in d['changes'] if x['id']==cid and x['option']=='B')} |" for cid in ['C01','C02','C03','C04','C05','C06'])
sample_nodes={o:{n['name']:n for n in d['nodes'] if n['episode']=='S03-05' and n['option']==o} for o in ['A','B']}
walk=[]
for name,label in [('meaning_adoption','정정 의미 채택'),('apply_relation','판본과 전달 순서 반영'),('evaluation_first_display','정정 수행 의도 화면 표시'),('question_first','메일 질문 첫 음성 준비'),('question_device','메일 질문 실제 첫 음성'),('source_report_v2','6월 결과 도착'),('display_report_v2','6월 결과 화면 표시'),('report_v2_tail','6월 결과 전체 음성 준비')]:
    walk.append(f"| {label} | {sample_nodes['A'][name]['end_s']:.3f} | {sample_nodes['B'][name]['end_s']:.3f} |")
report=f'''# DP44 시나리오와 가상 QA 비교

2026-10-22 / `{d['version']}` / **{d['evidence_kind']}**

**이 가정과 기존 우선순위에서는 기본안 A를 권고한다.** 첫 음성은 평균 {b['qa01_voice_s']-a['qa01_voice_s']:.3f}초 빠르고 QA03은 {b['qa03_s']-a['qa03_s']:.3f}초 작다. 대가는 전체 활동 모델 비용 {a['qa05_usd']-b['qa05_usd']:.6f} USD와 예상 변경 범위 {a['qa04_modules']-b['qa04_modules']:.2f}모듈 증가다. RAM 차이는 {a['qa06_mb']-b['qa06_mb']:.3f} MB로 작아 주요 선택 근거가 아니다. 수치는 작성한 사건과 자원 가정의 계산 결과이며, 제품 시험이나 실측 우위를 뜻하지 않는다.

[A/B 설계와 다섯 구조 그림](../README.md), [계산 원장]({rel}/synthetic-data.json), [Excel]({rel}/outputs/01a154de/DP44-synthetic-backdata.xlsx), [사례 탐색기]({rel}/outputs/01a154de/guide.html), [통합 인계](handoff.md).

## 1. 비교하는 구조와 고정 조건

A는 Response Registry가 받는 유효한 응답마다 Speech Preparation Coordinator가 S2S 출력을 준비하고 Prepared Audio Store에 보관한다. Speech Playback Coordinator가 실제 차례에 준비된 음성을 읽는다. B는 Speech Turn Controller가 선택한 발화에 생성 실행을 붙이며, 대기 응답은 텍스트 상태로 보관한다. B도 생성한 유효 잔여 음성을 보존한다.

Downstream Agent가 실제 질문과 결과 문안을 만든다. DP44는 문안 생성 LLM을 추가하지 않는다. 사용자 입력과 DP2 의미 채택, 판본과 권한 검사, UI 게시, 발화 감지 및 80ms의 로컬 재생 중단은 공통이다. 의미 해석이 끝나기 전에는 어느 쪽도 정정 범위를 추측해 출력하지 않는다. A는 이미 유효한 응답의 음성을 준비할 뿐 새 사용자 요청을 미리 해석하지 않는다.

모델은 동일한 가상 원격 S2S와 의미 모델이다. 로컬 SW가 준비, 중단, 재개와 실제 전달 기록을 소유한다. 출력 생성은 최대 4개, RAM audio는 16 MB로 제한하며 이 자료에서는 한도를 넘지 않는다. 모델 가중치와 원격 KV를 로컬 메모리에 넣지 않는다. 이는 자체 모델 내부 동작이나 현재 OpenAI 모델의 실측 특성이 아니다. 로컬 모델 배치로 옮기면 새 profile과 비용 정의 검토가 필요하다.

## 2. 같은 입력과 평가 단위

8개 사례군에 각각 10개의 작성 변형을 둔다. **평가 입력 80개, 지칭 표현 120개, 목표 90개**다. 10변형은 내용 구조는 유지하고 입력 token 수와 음성 길이를 조금 바꾼 합성 행이다. 10회 모델 실행이나 통계적 독립 표본이 아니다. 유형 비중은 각 12.5%의 설계 coverage이며 실사용 빈도 조사값이 아니다.

각 에피소드에는 앞선 업무와 원본을 만드는 준비 활동 한 개가 있다. 80개 준비 활동의 입력 처리와 모델 비용, 모든 음성 준비 및 RAM을 포함한다. 준비 활동의 정확도를 평가 입력 80개에 더하지 않는다. 진행 중인 원래 보고서 요청 10개는 QA03의 시작점을 -6초로 보존한다. 다른 80개 목표는 사용자가 새로 요청한 읽기, 순서 변경, 재개와 음성 철회의 완료다. 메일 질문을 읽었다고 메일 발송 업무가 완료된 것으로 세지 않는다.

| 사례 | 평가 발화 | 관찰할 차이 |
| --- | --- | --- |
{scenes}

S02, S03, S07의 통지는 발화 중에 도착하여 UI에 독립 표시된다. 사용자 입력 근거는 발화 시작 snapshot에 발화 중 제시 사건을 연결한다. 모든 발화 단어가 시작 시점에 이미 확정된 것으로 가정하지 않는다. S05는 이미 선택되어 생성과 재생 중인 긴 설명을 실제로 중단한다. S06은 이미 생성한 음성의 재사용과 이후 짧은 오감지이며 양쪽 모두 새 생성 호출을 하지 않는다.

## 3. QA 정의와 숫자

| QA | A | B | 단위 |
| --- | ---: | ---: | --- |
{table}

- **QA01 화면:** 실제 발화 종료에 대응하는 가상 사건 0에서 새 요청의 구체적 수행 의도 표시까지. 기존에 표시된 Agent 질문을 새 입력의 첫 응답 시각으로 재사용하지 않는다.
- **QA01 음성:** 같은 0에서 해당 입력의 첫 구체적 음성 출력까지. 생성 callback과 enqueue가 아닌 장치 출력 사건을 작성한다. 실제 장치를 관측한 것은 아니다. 화면과 음성은 합산하거나 평균하지 않는다.
- **QA02 / QA07:** 120/120 표현과 80/80 입력이다. **DP2가 동일한 올바른 의미를 공급하는 고정 fixture**이므로 100%는 DP44 처리 계약의 동점이며 LLM 정확도 예측이 아니다. 이전 DP의 작성 오답률을 이 자료에 이식하지 않았다.
- **QA03:** 원래 목표별 90개 인과 경로 평균. `finish=max(parent finish)+VIA duration`으로 다시 계산하고 외부 업무, 사용자 대기와 순수 재생 duration은 0으로 한다. 실제 선행 관계와 장치 준비는 남긴다. 단순히 화면 시간에 상수를 더하지 않는다. 새 목표 이전에 완료된 준비는 초기 상태로 두며 그 준비 비용은 QA05에 남는다.
- **QA04:** 같은 C01~C06의 예상 운영 Rust 모듈 수 평균이다. 실제 Rust diff가 아니며 `concept::` 소유 경계를 가정한다.
- **QA05:** 80평가와 80준비 및 모든 폐기, 취소 tail, 재개를 포함한 VIA 모델 비용이다. 수신 Agent 내부 업무 비용은 모델링하지 않았고 시스템 전체 비용으로 표현하지 않는다.
- **QA06:** 준비와 평가 전체에서 같은 시점의 소유 RAM 합계 중 최고값이다. 개별 buffer peak를 더하지 않는다.

공통 v2의 QA01 대표값은 화면 표시다. 음성 값은 사용자 요청으로 병기한 별도 종료점이며 기존 QA01 정의를 덮어쓰지 않는다. Agent 원본 도착부터 음성까지의 지연은 `deliveries.csv`에 별도 진단으로 남기며 입력 80개의 평균에 섞지 않는다.

## 4. 시간과 비용 가정

| 항목 | 작성 가정 |
| --- | --- |
| 입력 종료 후 전사 잔여 | 0.30초 |
| 의미 호출 | 0.20 + 입력 token/6000 + 출력 token/80초 |
| 판본 반영, UI 표시, 장치 첫 출력 | 각각 0.04초 |
| 음성 첫 준비 | 0.38 + 음성 호출 text token/6000초 |
| 첫 chunk와 후속 생산 | 0.20초 분량, 이후 wall-clock 1초당 audio 4초 |
| 취소 tail | cancel 요청 후 최대 0.10초의 생성까지 비용과 상태에 포함 |
| 텍스트 가상 요율 | 입력 1 USD/M, 출력 4 USD/M |
| 음성 가상 요율 | 입력 0.001 USD/초, 출력 0.002 USD/초 |
| 로컬 공통 RAM | 560 MB = core/UI 180 + ASR/audio 220 + 원본 이력 96 + 요청/이력 64 |
| 가변 RAM | PCM 24 kHz, 16 bit, mono = 48,000 byte/초. job metadata 4096 byte/개 |

요율과 의미 시간식은 공통 v2에서 사용한 계산 convention을 따른다. 음성 생성 속도와 지연, RAM은 작성 가정이며 공급자의 가격, 모델 성능 또는 파일 크기로부터 추정한 값이 아니다. 공통 기반 560 MB를 가져왔다는 사실만으로 DP3와 같은 workload나 같은 기준 구성이라는 뜻은 아니다. 압축 오디오, local KV, 여러 프로세스 복제 비용을 숨겨 넣지 않았다.

## 5. 같은 정정 장면을 끝까지 따라가기

S03-05의 평가 발화는 “보고서는 6월 기준으로 다시 해줘. 메일 질문부터 들려줘.”다. 원래 보고서 요청의 시작은 -6초, 현재 발화는 -4.4~0초다. 메일 질문은 -1.4초, 5월 결과는 -1초에 도착한다. 두 안 모두 질문과 결과를 UI에 바로 표시하고, 음성은 사용자 발화와 정정 판단 때문에 보류한다.

| 사건, 평가 발화 종료 기준 초 | A | B |
| --- | ---: | ---: |
{chr(10).join(walk)}

첫 화면 문구는 “보고서를 6월 기준으로 다시 요청하고, 메일 질문부터 읽겠습니다.”다. 첫 음성은 실제로 요청한 “수신자는 김 대리인가요?”다. 이 질문을 읽는 것은 사용자가 명시한 하위 요청의 실질 응답이며 접수 인사가 아니다. 보고서 수정 완료를 주장하지 않는다. 양쪽 모두 새 보고서 업무는 Downstream Agent에 맡긴다.

A는 질문 음성을 -0.351초에 완성해 두었으므로 판본과 순서 적용 뒤 바로 재생한다. B는 적용 뒤 생성한다. 동시에 A는 5월 결과의 생성도 시작했으므로 정정 시 구판을 폐기한다. 취소는 출력 차단 뒤 비동기로 하며 0.10초 tail도 비용에 남는다. 새 보고서 표시 시각은 양쪽이 같고 음성 생성 때문에 늦추지 않는다.

원장에는 R0가 재생 중인 S05도 별도로 있다. 사용자가 -3.6초에 말하기 시작하면 양쪽 모두 -3.52초까지 재생을 멈춘다. 의미 채택 때까지 유효 생성은 유지한다. 질문으로 실제 전환하면 A의 이전 생성은 계속되고 B는 종료한다. 복귀 시 B도 기존 buffer를 먼저 재생하며 **미생성 후반만** 새로 요청한다. 전체 설명을 처음부터 재생성하지 않는다. 생성된 전체 문구를 전달 이력에 올리지 않으며 실제 출력 구간과 UI 제시를 분리한다.

## 6. 유형별 이익과 비용

| 사례군, 각 10개 | 첫 음성 A / B, 초 | QA03 A / B, 초 | 전체 유형 비용 A / B, USD |
| --- | ---: | ---: | ---: |
{chr(10).join(ft)}

QA03의 S03 행은 보고서 원래 목표 10개와 질문 읽기 목표 10개를 함께 평균한다. 전체 QA03은 유형 평균 8개를 다시 평균하지 않고 목표 90개로 계산한다. S08은 물리적으로 준비를 50ms 앞당긴 경우다. QA03에서는 외부 도착 대기를 0으로 투영하면서 A의 생성과 의미 채택이 병행되고 B의 직렬 의존은 남는다. 따라서 S08의 QA03 차이를 실제 wall-clock 단축량으로 읽으면 안 된다.

S01과 S06의 시간 및 비용은 동점이다. S03과 S04에서 A가 사용하지 않을 구판을 생성한다. S05에서는 반대로 B의 suffix 재개 호출에 추가 문맥 비용이 든다. 균등 묶음에서 A의 구판 비용 {a['discarded_cost_usd']:.6f} USD 중 일부를 A의 재개 호출 절약이 상쇄하여 순차이는 {a['qa05_usd']-b['qa05_usd']:.6f} USD가 된다. B 비용 기준 A의 증가는 {(a['qa05_usd']/b['qa05_usd']-1)*100:.2f}%다.

## 7. QA04의 변경 근거와 QA06의 실제 크기

| 변경 요구 | A 예상 모듈 | B 예상 모듈 |
| --- | ---: | ---: |
{change}

C01~03은 공통 adapter에서 끝난다. C05도 같은 실제 전달 이력과 표시 모듈을 바꾼다. C04에서 A는 응답 문단, 전달 이력, 재생, 준비 실행, 보관 음성의 5개 소유 계약을 바꾼다는 가정이다. B도 한 모듈로 뭉개지 않고 문단, 이력, Turn Execution, Turn Audio Buffer의 4개로 센다. C06에서는 A의 준비 의존성 무효화 한 개가 추가된다. 기존 필수 판본 검사 부재를 새 기능으로 포장한 것이 아니며, 응답 단위에서 범위 단위로 기능을 확장하는 과제다.

공통 section/extent 인터페이스가 A의 준비와 보관에도 이미 충분히 일반화되어 있으면 C04 추가 변경과 C06 추가 변경이 사라져 두 안 모두 2.00이 될 수 있다. 이는 QA04 가정의 민감도이며 현재 구조만으로 0.33모듈 차이가 보장되지는 않는다. 선택은 이 작은 차이에 의존하지 않는다.

A peak는 {a['peak_witness']['episode']}의 {a['peak_witness']['at_s']:.3f}초, B peak는 {b['peak_witness']['episode']}의 {b['peak_witness']['at_s']:.3f}초다. 각각 base + audio + metadata를 합한다. A의 더 큰 audio 보관량은 사실이지만 전체 차이는 {a['qa06_mb']-b['qa06_mb']:.3f} MB뿐이다. 별도 복제 runtime이나 큰 per-job 임의 버퍼를 넣어 차이를 확대하지 않았다.

## 8. 선택과 반대 조건

**기본안 A를 조건부 권고한다.** 사용자가 요구한 음성 QA01을 우선하면 평균 {b['qa01_voice_s']-a['qa01_voice_s']:.3f}초의 이익이 첫 근거다. 공통 v2의 화면 QA01을 그대로 적용해도 화면과 QA02는 구조적 동점이고 다음 QA03에서 A가 유리하므로 이번 작성 묶음에서는 같은 결론이다. QA07의 순위를 임의로 추가하거나 가중 총점으로 합산하지 않는다. 100% 고정 정확도 fixture를 실제 모델의 자격 통과 근거로 쓰지는 않는다.

받아들이는 비용은 약 {(a['qa05_usd']/b['qa05_usd']-1)*100:.1f}%의 모델 비용 증가와 QA04의 더 넓을 수 있는 변경 범위다. RAM 차이는 작다. B는 음성 비용 제한이 우선이거나 사용 전 철회가 많고 유효 선행 준비가 드문 환경에서 선택할 이유가 있다.

- S01과 S04, S06만 남는 사용 범위에서는 시간은 동점이고 S04의 폐기 비용을 피하는 B가 유리하다. 사례 비중을 바꾼 별도 조건이며 원래 평균을 덮어쓰지 않는다.
- S05만 남으면 A가 시간과 재개 문맥 비용에 유리하다. B의 동일 생성 상태 pause/resume가 지원되면 추가 호출 비용과 시간 일부가 줄 수 있으며 보존 KV와 runtime 계약 비용을 새로 평가해야 한다.
- 첫 준비 지연이 작아지면 음성 이익 상한도 작아진다. 원래 장면에서 유효 선행 여유를 L, 첫 준비를 f라 하면 첫 음성 이익은 최대 min(L,f)다.
- 로컬 무과금 모델에서는 현재 청구액 QA05의 차이가 없다. 연산량을 임의 USD로 바꾸지 않으며 새 배치 profile에서 전체 RAM과 실행 의존을 다시 계산한다.

선택 후 A1로 다음 응답 한 개만 준비하는 제한을 검토할 수 있다. 폐기 비용은 줄지만 다른 응답이 선택되면 선행 이익을 잃는다. 이번 기본안 표에는 적용하지 않았다.

## 9. 재생성과 검증 범위

저장소 루트에서 `.venv/bin/python {HERE.relative_to(ROOT)}/build.py`, 이어서 같은 폴더의 `validate.py`, `report.py`를 실행한다. 이는 작성한 데이터의 오프라인 계산이며 제품 구현이나 모델 호출이 아니다. Workbook은 `workbook.mjs`로 내보낸다. 마지막에 `finalize.py`로 발행 검사와 SHA256 목록을 갱신한다.

[validation.json]({rel}/validation.json)은 DAG, 원래 목표 시작, 같은 입력, 사용량과 단가, 동시 RAM, 재생 겹침, 80ms 정지, 구판 미재생 및 집계를 다시 확인한다. 정확도는 고정 fixture이고 숫자의 현실성이나 실제 장치 출력은 검증하지 않았다. 독립 검토와 공통판 통합은 인계받는 세션에서 수행하도록 범위와 미확인 사항을 전달한다.

[발행 검증 기록]({rel}/publication-validation.json)은 변경 범위의 링크와 그림 XML, HTML 문법 및 저장소 검사 상태를 남긴다. Excel 9개 시트는 렌더 검토했으나 native Windows Excel은 실행하지 않았다. HTML 브라우저 시각 검증은 로컬 listener 권한과 file URL 보안 정책으로 완료하지 못했으며 문법 검사만 통과했다.
'''
(HERE/'README.md').write_text(report)

payload=json.dumps(d,ensure_ascii=False).replace('</','<\\/')
htmltext='''<!doctype html><html lang="ko"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>DP44 가상 비교</title>
<style>body{font:16px system-ui,sans-serif;margin:32px;color:#17212b;background:#f5f7fa}main{max-width:1250px;margin:auto}h1{font-size:30px}h2{margin-top:32px;font-size:22px}p{line-height:1.7}table{border-collapse:collapse;width:100%;background:white;margin:16px 0}th,td{padding:10px 12px;text-align:left;border-bottom:1px solid #d9e2eb;vertical-align:top}th{background:#e4edf5}td.num{text-align:right;font-variant-numeric:tabular-nums}select{font:inherit;padding:8px;margin-right:20px}pre{white-space:pre-wrap;overflow-wrap:anywhere;background:#fff;padding:16px;line-height:1.6}small{color:#52606d}.scroll{overflow:auto}a{color:#155aa0}.card{background:white;padding:16px;border-left:4px solid #32689a}</style>
<main><h1>DP44 음성 준비 구조의 가상 비교</h1><p>2026-10-22 | SYNTHETIC_AUTHORED_NOT_MEASURED<br>80개 평가 입력, 120개 지칭 표현, 90개 목표. 모델과 제품을 실행하지 않은 설계 계산입니다.</p><div class="card"><b>조건부 권고: A</b><p>화면 응답과 고정 의미 fixture는 동점입니다. A는 음성 시작과 전체 준비를 앞당기고, B는 구판 생성비와 예상 변경 범위를 줄입니다. RAM 차이는 작습니다.</p></div><div id="summary"></div><h2>사례 탐색</h2><label>사례 <select id="episode"></select></label><label>대안 <select id="option"><option>A</option><option>B</option></select></label><div id="detail"></div><h2>읽는 방법</h2><p>음성 시간은 실제 출력에 해당하는 가상 사건입니다. 화면 시간과 평균하지 않습니다. QA02와 QA07의 100%는 동일한 올바른 DP2 fixture를 고정한 결과이며 모델 정확도 예측이 아닙니다. QA03은 외부 업무와 순수 재생 시간을 0으로 바꾼 인과 경로입니다. 과거 준비 비용은 전체 비용에 남깁니다.</p><p>원장: <a href="../../synthetic-data.json">JSON</a> / <a href="DP44-synthetic-backdata.xlsx">Excel</a></p></main>
<script>const D=PAYLOAD;const el=id=>document.getElementById(id);const esc=x=>String(x??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const metrics=[['QA01 화면 (초)','qa01_display_s'],['QA01 음성 (초)','qa01_voice_s'],['QA02 지칭 대상','qa02'],['QA03 VIA 처리 (초)','qa03_s'],['QA04 예상 모듈 (개)','qa04_modules'],['QA05 활동 비용 (USD)','qa05_usd'],['QA06 peak RAM (MB)','qa06_mb'],['QA07 요청 의미','qa07']];
const table=(hs,rows)=>'<div class="scroll"><table><thead><tr>'+hs.map(h=>'<th>'+esc(h)+'</th>').join('')+'</tr></thead><tbody>'+rows.map(r=>'<tr>'+r.map(c=>'<td>'+esc(c)+'</td>').join('')+'</tr>').join('')+'</tbody></table></div>';
el('summary').innerHTML=table(['지표','A','B'],metrics.map(([n,k])=>[n,...['A','B'].map(o=>['qa02','qa07'].includes(k)?(D.summary[o][k]*100).toFixed(2)+'%':D.summary[o][k].toFixed(k==='qa05_usd'?6:3))]));
el('episode').innerHTML=D.inputs.map(i=>'<option value="'+i.episode+'">'+esc(i.episode+' '+D.families.find(f=>f.id===i.family).name)+'</option>').join('');el('episode').value='S03-05';
function render(){const ep=el('episode').value,o=el('option').value,i=D.inputs.find(x=>x.episode===ep),r=D.responses.find(x=>x.episode===ep&&x.option===o),filter=arr=>arr.filter(x=>x.episode===ep&&x.option===o);let h='<p><b>'+esc(i.text)+'</b></p><p>화면: '+esc(r.first_ui_text)+' ('+r.first_ui_s.toFixed(3)+'초)<br>음성: '+esc(r.first_voice_text)+' ('+r.first_voice_s.toFixed(3)+'초)</p><h3>최초 채택 의미와 정답</h3><pre>'+esc(JSON.stringify(r.adopted_meaning,null,2))+'</pre>';h+='<h3>원래 목표별 QA03</h3>'+table(['목표','시작','VIA 초','완료 범위'],filter(D.goals).map(g=>[g.id,g.start_s,g.qa03_s.toFixed(3),g.description]));h+='<h3>처리 순서와 생성</h3>'+table(['단계','시작 초','종료 초','QA03 포함'],filter(D.nodes).sort((a,b)=>a.start_s-b.start_s).map(n=>[n.name,n.start_s.toFixed(3),n.end_s.toFixed(3),n.via?'포함':'외부/대기/재생 제외']));h+='<h3>사용량</h3>'+table(['역할','단계','text 입력','생성 audio 초','비용 USD'],filter(D.usage).map(u=>[u.role,u.phase,u.text_input,u.audio_output_s.toFixed(3),u.amount_usd.toFixed(6)]));h+='<h3>실제 출력에 대응하는 작성 구간</h3>'+table(['응답','시작','종료','audio offset'],filter(D.playback).sort((a,b)=>a.start_s-b.start_s).map(p=>[p.source_id,p.start_s.toFixed(3),p.end_s.toFixed(3),p.source_offset_s.toFixed(3)]));el('detail').innerHTML=h;}el('episode').onchange=render;el('option').onchange=render;render();</script></html>'''.replace('PAYLOAD',payload)
outdir=OUT/'outputs/01a154de';outdir.mkdir(parents=True,exist_ok=True);(outdir/'guide.html').write_text(htmltext)
print('README and HTML generated')
