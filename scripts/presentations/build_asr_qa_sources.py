#!/usr/bin/env python3
"""Project the authoritative ASR-QA Markdown into presentation sources."""
from pathlib import Path
import re,json,hashlib,sys

def emit(path, value):
 if "--check" in sys.argv:
  assert path.read_text()==value, f"Stale derived file: {path}"
 else:
  path.write_text(value)

root=Path(__file__).resolve().parents[2]
src=root/'docs/architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md'
s=src.read_text();h=hashlib.sha256(src.read_bytes()).hexdigest()
clean=lambda x:re.sub(r'\[([^]]+)\]\([^)]*\)',r'\1',x).replace('**','').strip()
rows=[]
for line in s.splitlines():
 if line.startswith('| **ASR-QA-'):
  c=[clean(x) for x in line.split('|')[1:-1]]
  rows.append(dict(id=c[0],priority=len(rows)+1,name=c[1],description=c[2],method=c[3],target=c[4],targetBasis=c[5]))
cat=['Accuracy','Performance','Performance','Cost Efficiency','Maintainability','Resource Efficiency']
shortgoal=['필수 정답 조건 충족률 ≥ 98%','평균 상호작용 응답시간 ≤ 1초','평균 VIA 요청 처리시간 ≤ 10초','8시간 총 모델 비용 ≤ $1.50','평균 변경 책임 단위 수 ≤ 3개','최대 전체 commit 메모리 ≤ 4GiB']
shortmetric=['18개 Voice 사례 × 5회 × 6개 조건 = 540개','6개 사건 × 5회 = 30개 유효 반응시간','6개 전체 사이클 × 5회 = 30개 귀속 시간','동일 8시간 활동의 청구 사용량 × 공개 단가','6개 변경 과제의 고유 수정 책임 수 평균','8시간/3session 중 가장 큰 동시 commit량']
basis=['50개 조건당 오류 1개 이하, 전체 요청 실패 20회당 1회 이하','대화 흐름 1초와 끼어들기 local stop 0.1초를 관리','의미 6초 + 연결 1초 + 전달 1초 + 재검증 여유 2초','기본 $1.1325 + 추가 생산/재시도 여유 $0.3675','일상 변경을 adapter/생산/소비 약 3개 책임에 국소화','32GB급 PC의 사용자 앱과 공존하는 VIA 4GiB 예산']
facts=[
'VIA는 여러 업무와 조건, 실제 전달을 같은 대화로 연결한다.\n잘못 연결하면 사용자가 업무를 다시 식별하고 수정해야 한다.',
'Nielsen의 응답시간 참조는 즉각 제어 0.1초,\n대화 흐름 1초, 주의 유지 10초다.',
'외부 Agent의 업무 시간은 VIA가 책임지는 처리 시간이 아니다.\nVIA의 조회/해석/인계/결과 전달은 평가 경로에 남는다.',
'공식 단가: 의미 입력 $0.80 / 출력 $4.00, 1M token 기준.\n음성 입력 $0.005 / 출력 $0.018, 1분 근사 기준.',
'모델/Agent/source wire 교체와 의미/상태 계약 확장은 다르다.\n같은 책임 registry로 강한 A/B의 실제 수정 범위를 센다.',
'삼성 reference PC: Windows 11, RAM 32GB급.\nWindows PrivateUsage는 resident RAM과 다른 commit 값이다.']
choices=[
'필수 조건 오류 ≤ 2%, 전체 요청 실패 ≤ 5%를 PoC 예산으로 선택.\n평균이 높아도 중요 상황/조건 실패와 필수 위반은 따로 검사.',
'평균 1초, 모든 음성 stop 0.1초 이내를 과제 예산으로 선택.\n2초 이내 29/30회와 미전달 0건도 요구.',
'의미 6초 + 연결 1초 + 전달 1초 + 재검증 2초 = 10초.\n첫 접수 멘트 이후의 전체 후속 처리까지 포함.',
'하루 100개 목표의 기본 계산 $1.1325에 $0.3675 여유를 둠.\n배경 갱신/미사용 생산/재시도와 청구 context를 함께 관리.',
'평균 3개, 과제별 최대 5개로 변경 전파 범위를 제한.\n파일/Component 수가 아니라 수정된 기능 책임으로 판정.',
'OS 8 + 사용자 앱 16 + VIA 4 + 여유 4 = 32GiB 계획 가정.\n실제 RAM 가용량이나 측정 peak를 주장하는 수치는 아님.']
calcs=[
[['대표 분모','18 × 5 × 6 = 540개, 항목 안 모든 사실 통과 시 1'],['대표 목표','정답 530개 이상 / 오답 최대 10개'],['요청 전체 보호','여섯 조건 모두 정답 86/90회 이상'],['S / C 보호','각 90개 중 정답 86개 이상'],['필수 위반','무관 업무/중복 효과/잘못된 승인/무허용 사용 0건'],['A/B 차이','1pp 이상과 paired 95% 구간, 사례 묶음 bootstrap']],
[['시작','발화 끝 / Agent source availability / 끼어들기 onset'],['종료','필요한 실제 제어 효과 또는 Text + 첫 핵심 음성'],['대표 집계','I-01~06 각 5회, 총 30개 산술평균'],['예산 예시','0.10 + 0.30 + 100/450 + 0.08 + 0.25 = 0.952초'],['실패 처리','5초 cap + 미전달 기록, 누락을 분모에서 제외하지 않음'],['A/B 차이','0.10초 이상, 실제 반복 구간 또는 계산 민감도']],
[['전체 사이클','요청/확인/인계/결과 수신/실제 결과 전달'],['대표 집계','T-01~06 각 5회, 총 30개 산술평균'],['귀속 계산','같은 causal DAG의 Agent/사람 대기만 0으로 계산'],['병렬 처리','독립 branch는 max, 순차는 합산, 외부 시간 중복 차감 금지'],['실패 처리','30초 cap + 미전달 기록, 15초 이내 29/30회'],['A/B 차이','0.50초 이상, 접수 인사로 최종 결과를 대체하지 않음']],
[['의미 모델','600k 입력 / 60k 출력 = $0.7200'],['음성 audio','입력 30분 / 출력 10분 = $0.3300'],['음성 Text/context','50k 입력 / 10k 출력 = $0.0825'],['기본 총액','$0.7200 + $0.3300 + $0.0825 = $1.1325'],['추가 예산','$1.5000 - $1.1325 = $0.3675'],['A/B 차이','$0.15/8h 이상, cache miss/생산 빈도 민감도']],
[['공통 과제','provider / Agent / source 교체, 관계 추가, 질문 token, 철회'],['공통 단위','U01~18 기능 책임 registry, 동일 수정의 중복 금지'],['대표 집계','여섯 수정 집합의 크기 합 / 6'],['설명 예시','M-04 A: U01/U03/U07, B: U01/U02/U03/U07'],['완료 조건','새 요구 충족 + 기존 기능/권한/중복 회귀 유지'],['A/B 차이','평균 0.5개 이상, 여섯 과제 합계 차이 3개 이상']],
[['대표 관측','전체 process private commit + 고유 shared commit'],['공통 부하','8시간, Task 10개, cloud call 4개, 동일 capture/원본'],['기본 할당 예산','동시 8개 할당 항목의 합 = 1,792MiB (전체 내역 §7.3)'],['추가/변동 여유','4,096 - 1,792 = 2,304MiB'],['관측/보호','50ms + allocation event, 3session 최대 / working set 6GiB'],['A/B 차이','128MiB 이상, 동일 기능/보관과 process copy 포함']]]
methods=[
'18개 사례/108개 초기 정답 명세는 03-02 §3에 수록.\n실제 모델 출력으로 채점하며 가정 정답률을 생성하지 않음.',
'I-01 음성 중단, I-02 정정, I-03 취소, I-04 유효 질문, I-05 결과, I-06 접근 거부.\nlocal stop는 공통 경로이며 다른 반응의 의미 처리를 생략하지 않음.',
'정확성 원본의 접수 완료와 시간 파생 장면의 최종 결과를 구별.\n실제 wall-clock과 제외 구간/DAG도 함께 보고.',
'역할별 call ID로 중복 청구를 제거하고 청구된 폐기 출력은 포함.\n외부 Agent 내부 모델 비용은 별도 책임 원장.',
'구체 설계 patch/실제 diff와 완료 oracle로 변경 집합을 검수.\n구현 개발 시간이나 박스 수로 점수를 만들지 않음.',
'commit과 실제 RAM residency/VRAM을 구별해 진단을 병기.\n초기 생산/반복/철회/유휴까지 포함하고 조기 종료로 peak를 낮추지 않음.']
guards=['완전 정답/각 S/C ≥95%, 필수 위반 0건','stop ≤0.1초, 29/30 ≤2초, 미전달 0건','29/30 ≤15초, 미전달 0건','재시도/폐기/배경/청구 context 누락 없음','과제별 ≤5개, 6개 완료/필수 회귀 충족','working-set 합 ≤6GiB, 유휴 증가 ≤16MiB']
references={
 'scope':{'title':'VIA 고정 기능/UC','url':'../../architecture/05-representative-use-cases.md'},
 'nielsen':{'title':'Nielsen Norman Group 응답시간','url':'https://www.nngroup.com/articles/website-response-times/'},
 'groq':{'title':'Groq 공식 모델 단가/생성률','url':'https://console.groq.com/docs/models'},
 'google':{'title':'Google Live 공식 가격표','url':'https://ai.google.dev/gemini-api/docs/pricing'},
 'samsung':{'title':'Samsung NP960UJH-XG3IN 공식 사양','url':'https://www.samsung.com/in/computers/galaxy-book/galaxy-book6-ultra-ultra-7-32gb-1tb-np960ujh-xg3in/'},
 'windows':{'title':'Microsoft PrivateUsage/commit 정의','url':'https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex'},
 'contract':{'title':'03-02 공통 QA 계약 v4','url':'../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md'}}
refsets=[['scope','contract'],['nielsen','groq','contract'],['nielsen','contract'],['groq','google','contract'],['scope','contract'],['samsung','windows','contract']]
bands=json.loads((src.parent/'03-02-poc-inputs.json').read_text())['score_bands']
chapters={i:s.split(f'## {i}. ',1)[1].split('\n## ',1)[0] for i in range(3,9)}
for i,r in enumerate(rows):
 r.update(category=cat[i],displayTarget=shortgoal[i],shortMetric=shortmetric[i],shortBasis=basis[i],fact=facts[i],choice=choices[i],calculationRows=calcs[i],slideMethod=methods[i],guards=guards[i],sources=refsets[i],score=bands[i],detail=chapters[3 if i==0 else 4 if i<3 else i+2],definitionSection=3 if i==0 else 4 if i<3 else i+2)
data={'source':str(src.relative_to(root)),'source_sha256':h,'contract_version':'ASR-QA-v4','date':'2026-10-11','status':'AUTHOR_DEFINED_POC_CONTRACT_NO_CANDIDATE_MEASUREMENT','rows':rows,'references':references,'reference_style':{'summary':'07-품질요구사항.png','metrics':'39~44-Appendix-QAMetric*.png','interpretation':'스타일/구조만 적용. 외부 과제 요구/수치/순위를 VIA로 이관하지 않음'}}
for folder,file in [('quality-attributes','quality-attributes.json'),('quality-metrics','quality-metrics.json')]:
 emit((root/'docs/presentations_files'/folder/file), json.dumps(data,ensure_ascii=False,indent=2)+'\n')
caseblocks=re.findall(r'##### (S[1-6]-0[1-3]) ([^\n]+)\n(.*?)(?=\n<a id=|\n#### 3\.2\.3)',s,re.S)
assert len(caseblocks)==18,len(caseblocks)
case_registry={'contract_version':'ASR-QA-v4','accuracy_version':'Voice-PoC-v3','source_sha256':h,'cases':[{'id':i,'name':name,'specification':text.strip(),'required_conditions':[f'C{j}' for j in range(1,7)]} for i,name,text in caseblocks],'trials':90,'binary_conditions':540}
emit((root/'docs/presentations_files/quality-attributes/functional-coverage.json'), json.dumps(case_registry,ensure_ascii=False,indent=2)+'\n')
plan={'source':data['source'],'source_sha256':h,'contract_version':'ASR-QA-v4','status':data['status'],'priority':[r['id'] for r in rows],'populations':{'accuracy':{'cases':18,'repeats':5,'trials':90,'denominator':540},'reaction':{'events':6,'repeats':5,'trials':30},'cycle':{'cycles':6,'repeats':5,'trials':30},'cost_memory':{'hours':8,'sessions':3,'goals':100},'change':{'fixtures':6,'responsibilities':18}},'targets':dict(zip([r['id'] for r in rows],[98,1,10,1.5,3,4096])),'summary':[{k:r[k] for k in ['id','name','description','method','target','targetBasis','guards']} for r in rows]}
emit((root/'docs/presentations_files/quality-attributes/evaluation-plan.json'), json.dumps(plan,ensure_ascii=False,indent=2)+'\n')
emit((root/'docs/presentations_files/quality-attributes/measurement-design.md'), '# ASR-QA 평가 설계 안내\n\n이 파일은 [03-02 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 발표용 연결 안내다. 정의/정답/분모/목표를 이곳에서 독립 개정하지 않는다.\n\n| QA | 원본 | 평가 단위 |\n| --- | --- | --- |\n'+ '\n'.join(f'| {r["id"]} {r["name"]} | 03-02 §{r["definitionSection"]} | {r["shortMetric"]} |' for r in rows)+'\n\n정확성 18개 상세 명세는 [사례 투영 JSON](functional-coverage.json), 공통 규모/목표는 [평가 안내 JSON](evaluation-plan.json)에 원본 hash와 함께 수록했다. [검산 보고](../../architecture/12-decisions/decision-packages/03-02-poc-calculation-report.md)는 공개 단가/가정 계산이며 후보 성능 결과가 아니다.\n')


base=root/'docs/presentations_files'
archive='../archive/qa-presentations-before-asr-20261011/README.md'
source_url='../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md'
overview=['# VIA 중요 품질속성 발표 요약', '',
 f'[정의 원본 03-02]({source_url})의 ASR-QA-01~06을 두 장으로 요약한다. 번호순 우선순위는 01 → 02 → 03 → 04 → 05 → 06이며 낮은 순위의 약점은 tactic으로 보완한다.', '',
 '> ASR-QA-v4 / 2026-10-11 / 작성자 정의와 참조 계산. 후보 성능과 Windows 실측 결과는 아직 없다.', '',
 '[편집 가능한 PPTX, 2장](quality-attributes/VIA-quality-attributes.pptx) / [PNG 미리보기](quality-attributes/index.html) / [QA별 근거와 등급, 6장](quality-metrics.md)', '',
 '| ID | 이름 | 정의 | 대표 측정 지표 | 목표값 | 과제 차원의 목표 근거 |', '| --- | --- | --- | --- | --- | --- |']
overview += ['| '+' | '.join(r[k] for k in ('id','name','description','method','target','targetBasis'))+' |' for r in rows]
overview += ['', '외부 Agent의 업무 수행 시간은 시간 지표에서 제외하며 VIA 모델/통신/연결/결과 전달은 남긴다. Text 입력을 추가하지 않고 공통 Voice replay를 사용한다. 정확성 18개는 각 6개 조건을 5회씩 채점하는 540개 항목이며 시간은 각각 30개 관측이다. 비용/메모리는 같은 8시간 활동의 3session, 변경은 여섯 공통 과제다.', '',
 '표의 대표 목표 외에도 보호 조건을 검사한다. 정확성 전체/상황별/조건별 정답과 필수 위반, 시간 미전달/음성 stop, 변경 완료/회귀, 메모리 working set/유휴 증가를 원본대로 보고한다. 목표 충족 등급만으로 보호 조건 실패를 숨기지 않는다.', '',
 '## 원문과 자료 관리', '',
 '[18개 상세 정답 투영](quality-attributes/functional-coverage.json), [평가 규모/목표 투영](quality-attributes/evaluation-plan.json)과 [측정 설계 연결](quality-attributes/measurement-design.md)은 03-02의 원본 hash로 추적한다. 상세 정의는 03-02에서 수정하고 투영/발표 파일을 재생성한다.', '',
 f'이전 V-01~10 요약/10장 부록/시험 초안/생성 코드는 [archive]({archive})에 원래 bytes와 hash로 보존했다. 기존 QA catalog/V ID/ADR/참조 설계를 이번 발표 ID로 바꾸지 않는다.', '',
 '![정확성과 응답시간](quality-attributes/quality-attributes-01.png)', '', '![비용, 변경과 메모리](quality-attributes/quality-attributes-02.png)', '']
emit(base/'quality-attributes.md','\n'.join(overview))
metrics=['# VIA QA 목표 근거와 7단계 등급', '',
 f'[03-02 원본]({source_url})을 여섯 장으로 설명한다. [편집 가능한 PPTX, 6장](quality-metrics/VIA-quality-metrics.pptx) / [PNG 미리보기](quality-metrics/index.html) / [요약표](quality-attributes.md).', '',
 '제공한 레퍼런스의 상단 요약표, 목표 근거/계산과 오른쪽 등급표 구성을 따른다. 타 과제의 수치/요구사항은 가져오지 않았다. 목표와 등급 경계는 작성자의 VIA PoC 예산이며 출처의 보장 수치와 구별한다.', '',
 '등급 6~0의 표시는 ●●● / ●●◐ / ●●○ / ●◐○ / ●○○ / ◐○○ / ○○○다. 원지표의 개선 여유를 나타내며 QA 간 합산하지 않는다. 1~6점은 대표 목표를 만족하지만 보호 조건 실패가 있으면 전체 목표는 미달이다. 실측 전은 0점이 아니며 현재 후보 점수는 생성하지 않았다.', '']
for r in rows:
 metrics += [f'## {r["id"]} {r["name"]}', '', '**정의:** '+r['description'], '', '**대표 지표:** '+r['method'], '', '**목표:** '+r['target'], '', '**목표 근거:** '+r['targetBasis'], '', '| 항목 | 계산/평가 규칙 |', '| --- | --- |']
 metrics += ['| '+' | '.join(row)+' |' for row in r['calculationRows']]
 metrics += ['', '**보호 조건:** '+r['guards'], '', r['slideMethod'].replace('\n',' '), '', '| 점수/표시 | 원지표 범위 | 대표 목표 |', '| --- | --- | --- |']
 b=r['score']['bounds'];high=r['score']['direction']=='high';unit=r['score']['unit']
 ranges=([f'x = {b[0]}%']+[f'{b[i+1]}% ≤ x < {b[i]}%' for i in range(5)]+[f'0% ≤ x < {b[-1]}%']) if high else ([f'0 ≤ x ≤ {b[0]} {unit}']+[f'{b[i]} < x ≤ {b[i+1]} {unit}' for i in range(5)]+[f'x > {b[-1]} {unit}'])
 for j,rg in enumerate(ranges):
  n=6-j;metrics.append(f'| {n} / {["○○○","◐○○","●○○","●◐○","●●○","●●◐","●●●"][n]} | {rg} | {"충족" if n else "미달"} |')
 citations=['['+references[k]['title']+']('+references[k]['url'].replace('../../architecture','../architecture')+')' for k in r['sources']]
 metrics += ['', '출처: '+', '.join(citations)+'. 숫자/목표 선택과 상세 경계는 03-02 §'+str(r['definitionSection'])+' 및 §8.9.', '', f'![{r["id"]} 목표 근거와 등급](quality-metrics/qa-metric-asr-qa-{r["priority"]:02d}.png)', '']
metrics += ['## 변경과 재생성', '', f'이전 자료는 [archive]({archive})에 보존했다. [입력 원장](../architecture/12-decisions/decision-packages/03-02-poc-inputs.json), [검산 보고](../architecture/12-decisions/decision-packages/03-02-poc-calculation-report.md)와 [재생성 안내](quality-metrics/README.md)를 따른다. 상세 정의는 03-02 하나에서 유지하고 문안/JSON/PPTX의 원본 hash를 함께 갱신한다.', '']
emit(base/'quality-metrics.md','\n'.join(metrics))
for kind,count in [('attributes',2),('metrics',6)]:
 md=f'''# VIA ASR-QA {'품질 요구사항 요약' if kind=='attributes' else '목표 근거와 등급 부록'}

[03-02 정의 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 여섯 QA를 {'3개씩 두 페이지에' if kind=='attributes' else '각 QA별 한 페이지에'} 정리했다. 정의/지표/목표/과제 근거, 보호 조건과 번호순 우선순위를 유지한다. 후보 성능은 아직 측정하지 않았다.

- [편집 가능한 PowerPoint, {count}장](VIA-quality-{kind}.pptx)
- [최종 PPTX에서 렌더링한 PNG](index.html)
- [발표 문안과 원본 연결](../quality-{kind}.md)
- [투영 JSON](quality-{kind}.json): 원본 SHA-256, 여섯 핵심 정의와 상세 평가 명세, 출처
- [확인 범위](verification.md)
- [이전 발표 archive](../../archive/qa-presentations-before-asr-20261011/README.md): 원래 bytes/hash 보존

PowerPoint native text와 table로 편집 가능하며 슬라이드 PNG를 붙인 파일이 아니다. 16:9, 1920×1080이다. 한글은 Apple SD Gothic Neo, 등급 기호는 Arial Unicode MS를 사용했다. Microsoft PowerPoint 앱에서 저장/재열기는 확인하지 않았다. 글꼴이 다른 환경의 배치는 확인이 필요하다.

## 원본 관리와 재생성

[03-02](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)를 수정한 뒤 아래 투영/검산을 재생성한다. 별도 발표 정의를 JSON에서 독립 개정하지 않는다.

```sh
python3 scripts/architecture/qa_poc_reference.py --write
python3 scripts/presentations/build_asr_qa_sources.py
python3 scripts/presentations/build_asr_qa_sources.py --check
```

Codex workspace dependency 도구로 bundled Node/Python/node_modules 경로를 확인한다. Presentations 스킬의 재생성 절차에 따라 새 임시 build 폴더에 [통합 생성기](../../../scripts/presentations/generate_asr_qa_presentations.mjs)를 복사하고 `node_modules`를 bundled modules에 연결한다. `RUNTIME_NODE_MODULES`, `VIA_PRESENTATION_SKILL_DIR`, `VIA_RUNTIME_PYTHON`을 각 환경의 절대 경로로 지정하고 bundled Node로 `generate_asr_qa_presentations.mjs REPO_ABSOLUTE BUILD_ABSOLUTE {kind}`를 실행한다. candidate와 별도 final 파일을 사용하며 기존 final 폴더를 재사용하지 않는다. package/layout/native table/font/Artifact Tool 재가져오기를 검증하고 최종 파일에서 PNG를 렌더링한다.

```sh
python3 scripts/presentations/check_quality_attributes.py
python3 scripts/presentations/check_quality_metrics.py
```

등급은 대표 지표의 목표를 표시한다. 보호 조건을 함께 검사하며 서로 다른 QA의 등급을 합산하지 않는다. 목표/계산의 작성자 가정과 공개 단가/장치 사양을 구별하며 실제 A/B 우열이나 제품 목표 달성을 주장하지 않는다.
'''
 if kind=='attributes':md+='\n정확성은 [18개 상세 사례](functional-coverage.json), 평가 규모는 [evaluation-plan](evaluation-plan.json), 원문 절 연결은 [measurement-design](measurement-design.md)을 따른다.\n'
 emit(base/f'quality-{kind}'/'README.md',md)
print('PASS: six ASR rows, 18 exact case specifications, evaluation plan and presentation prose synchronized')
