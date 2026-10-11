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
w=json.loads((src.parent/'03-02-common-workload.json').read_text());wm=json.loads((src.parent/'03-02-workload-manifest.json').read_text());u=wm['summary']
assert wm['source_sha256']==h
clean=lambda x:re.sub(r'\[([^]]+)\]\([^)]*\)',r'\1',x).replace('**','').strip()
rows=[]
for line in s.splitlines():
 if line.startswith('| **ASR-QA-'):
  c=[clean(x) for x in line.split('|')[1:-1]]
  rows.append(dict(id=c[0],priority=len(rows)+1,name=c[1],description=c[2],method=c[3],target=c[4],targetBasis=c[5]))
cat=['Accuracy','Performance','Performance','Cost Efficiency','Maintainability','Resource Efficiency']
shortgoal=['필수 정답 조건 충족률 ≥ 98%','평균 상호작용 응답시간 ≤ 1초','평균 VIA 요청 처리시간 ≤ 10초','P21 1묶음 평균 모델 비용 ≤ $0.50','평균 변경 책임 단위 수 ≤ 3개','최대 전체 commit 메모리 ≤ 4GiB']
shortmetric=['21개 Voice/Agent 사례 × 5회 × 6조건 = 630개','모든 사례의 지정 반응 27개 × 5회 = 135관측','사용자 목표 19개 × 5회 = 95사이클','동일 P21 전체 5회 총 청구량의 평균 × 단가','6개 변경 과제의 고유 수정 책임 수 평균','동일 P21 전체 5회 중 최대 동시 commit량']
basis=['50개 조건당 오류 1개 이하, 전체 시나리오 실패 ≤5%','대화 흐름 1초, 끼어들기 local stop 0.1초 관리','의미 6초 + 연결 1초 + 전달 1초 + 재검증 2초','신규 audio·Text 기본 $0.416320 + 추가 여유 $0.083680','일상 변경을 adapter/생산/소비 약 3개 책임에 국소화','32GB급 PC의 사용자 앱과 공존하는 VIA 4GiB 예산']
facts=[
'VIA는 업무, 근거, 조건과 실제 전달을 연결한다.\n잘못 연결하면 사용자가 업무를 다시 식별하고 수정해야 한다.',
'Nielsen 응답시간 참조: 즉각 제어 0.1초,\n대화 흐름 1초, 주의 유지 10초.',
'외부 Agent 업무와 사람의 입력/답변 대기는 제외한다.\nVIA 해석, 연결, queue와 실제 전달 시간은 남긴다.',
'공식 단가(1M token): 의미 Gemini $0.50 / $3.00.\n음성 GPT Realtime: audio $32 / $64, Text $4 / $16.',
'모델/Agent/source 교체와 의미/상태 계약 확장을 구별.\n같은 책임 registry로 양안의 실제 수정 범위를 센다.',
'삼성 reference PC: Windows 11, RAM 32GB급.\nWindows commit과 실제 resident RAM을 구별한다.']
choices=[
'필수 조건 오류 ≤2%, 전체 시나리오 실패 ≤5%를 선택.\n21개 모든 사례와 재사용/집중 도착/최종 tail을 채점.',
'평균 1초, local stop 0.1초를 과제 예산으로 선택.\n생성률 100/200/400 token/s는 가정, 목표 달성 미입증.',
'의미 6 + 연결 1 + 전달 1 + 재검증 2 = 10초.\n첫 접수 안내 이후의 전체 후속 처리까지 포함.',
'동일 P21 기본 소계 $0.416320에 $0.083680 여유.\n추가 audio 이력, 배경 생산·재시도까지 실제 청구.',
'평균 3개, 과제별 최대 5개로 변경 전파를 제한.\n파일/박스 수 대신 수정된 기능 책임으로 판정.',
'OS 8 + 사용자 앱 16 + VIA 4 + 여유 4 = 32GiB 가정.\n장치의 실측 가용량이나 후보 peak가 아니다.']
calcs=[
[['대표 분모','21 × 5 × 6 = 630조건, 모든 필수 사실 통과 시 1'],['대표 목표','정답 618개 이상 / 오답 최대 12개'],['시나리오 보호','6조건 모두 정답인 시나리오 ≥100/105'],['S / C 보호','상황군별 / 조건 유형별 각각 ≥95%'],['필수 위반','무관 업무/중복 효과/잘못된 승인/무허용 사용 0건'],['A/B 차이','평균 ≥1pp + 5쌍 같은 방향; 원값/최소/최대 공개']],
[['시작','발화 끝 / Agent 원천 도착 / 끼어들기 onset'],['종료','실제 제어 효과 또는 Text + 첫 핵심 음성'],['대표 집계','21개 사례의 27반응 × 5회 = 135관측'],['일반 / 집중','일반 ≥110/115회 ≤2초; 집중 30회 전부 ≤5초'],['실패 / stop','미전달 0건, 실패 5초 cap; stop 10회 모두 ≤0.1초'],['A/B 차이','평균 ≥0.10초 + 5쌍 같은 방향; 귀속 DAG 공개']],
[['전체 사이클','요청 → 해석 → 인계 → 결과 수신 → 실제 전달'],['대표 집계','19개 사용자 목표 × 5회 = 95사이클'],['적용 제외','Agent 단독 S5-01/02: 새 사용자 목표 시작 없음'],['귀속 계산','같은 DAG의 외부 업무/사람 입력·대기만 0'],['실패 처리','30초 cap + 미전달 기록; ≥91/95회 ≤15초'],['A/B 차이','평균 ≥0.50초 + 5쌍 같은 방향; 최종 결과 관측']],
[['의미 모델','62호출 예산: 124k 입력 / 12.4k 출력 = $0.0992'],['음성 신규 audio','입력 50초 / 출력 179초 예산 = $0.245120'],['음성 Text/context','총 청구 입력 10k / 출력 2k = $0.072000'],['기본 소계','$0.0992 + $0.24512 + $0.072 = $0.416320'],['추가 예산','$0.50 - $0.41632 = $0.08368; 이력 audio 등'],['A/B 차이','≥$0.03/P21; 실제 cache/생산/재시도 청구 비교']],
[['공통 과제','provider / Agent / source 교체, 관계/질문/철회'],['공통 단위','U01~18 기능 책임 registry, 동일 수정 중복 금지'],['대표 집계','여섯 수정 집합의 크기 합 / 6'],['설명 예시','M-04 A: U01/U03/U07, B: U01/U02/U03/U07'],['완료 조건','새 요구 충족 + P21/선호/권한/중복 회귀 유지'],['A/B 차이','평균 ≥0.5개, 여섯 과제 합계 차이 ≥3개']],
[['대표 관측','전체 process private + 고유 shared commit'],['공통 부하','동일 P21, Task 최대 10 / cloud 호출 최대 4'],['기본 할당 예산','동시 8개 할당 항목 합 = 1,792MiB (§7.3)'],['추가/변동 여유','4,096 - 1,792 = 2,304MiB'],['관측/보호','50ms + allocation event, 5회 최대 / WS ≤6GiB'],['A/B 차이','≥128MiB; 동일 기능/보관, 실제 process copy 포함']]]
methods=[
'21개 명세/6조건/최종 tail은 03-02 §3·8.2.\n실제 출력으로 채점하며 가정 정답률을 생성하지 않음.',
'모든 사례 → 반응 mapping은 요약 4~6장과 03-02 §3.2.\n사건 평균의 비중과 사례별 값/voice queue도 공개.',
'19개 전체 사이클을 동일 사례에서 관측한다.\nwall-clock / 제외 구간 / 병렬 DAG를 함께 보고.',
'역할별 call ID로 중복 제거, 청구된 폐기 출력 포함.\n179초와 62호출은 예산; 실제 사용량으로 대체.',
'구체 patch/diff와 완료 oracle로 수정 책임 검수.\n개발 시간이나 Component 수로 점수를 만들지 않음.',
'초기 생산/재사용/집중/철회/drain/idle 모두 포함.\n미완료 상태를 지워 비용과 peak를 낮추지 않음.']
guards=['전체/S/C ≥95%, 필수 위반 0건','일반 ≥110/115 ≤2초; 집중 30/30 ≤5초; stop ≤0.1초','≥91/95 ≤15초, 미전달 0건','초기/재시도/폐기/배경/context 누락 없음','과제별 ≤5개, 6개 완료/필수 회귀 충족','working-set ≤6GiB, 유휴 증가 ≤16MiB']
references={
 'scope':{'title':'VIA 고정 기능/UC','url':'../../architecture/05-representative-use-cases.md'},
 'nielsen':{'title':'Nielsen Norman Group 응답시간','url':'https://www.nngroup.com/articles/website-response-times/'},
 'gemini':{'title':'Google Gemini 3 Flash 공식 단가','url':'https://ai.google.dev/gemini-api/docs/pricing#gemini-3-flash-preview'},
 'openai':{'title':'OpenAI GPT Realtime 공식 단가','url':'https://developers.openai.com/api/docs/models/gpt-realtime'},
 'voice-billing':{'title':'OpenAI Realtime 청구','url':'https://developers.openai.com/api/docs/guides/voice-latency-cost'},
 'samsung':{'title':'Samsung NP960UJH-XG3IN 공식 사양','url':'https://www.samsung.com/in/computers/galaxy-book/galaxy-book6-ultra-ultra-7-32gb-1tb-np960ujh-xg3in/'},
 'windows':{'title':'Microsoft PrivateUsage/commit 정의','url':'https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex'},
 'contract':{'title':'03-02 공통 QA 계약 v6','url':'../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md'}}
refsets=[['scope','contract'],['nielsen','gemini','contract'],['nielsen','contract'],['gemini','openai','voice-billing','contract'],['scope','contract'],['samsung','windows','contract']]
bands=json.loads((src.parent/'03-02-poc-inputs.json').read_text())['score_bands']
chapters={i:s.split(f'## {i}. ',1)[1].split('\n## ',1)[0] for i in range(3,9)}
for i,r in enumerate(rows):
 r.update(category=cat[i],displayTarget=shortgoal[i],shortMetric=shortmetric[i],shortBasis=basis[i],fact=facts[i],choice=choices[i],calculationRows=calcs[i],slideMethod=methods[i],guards=guards[i],sources=refsets[i],score=bands[i],detail=chapters[3 if i==0 else 4 if i<3 else i+2],definitionSection=3 if i==0 else 4 if i<3 else i+2)
data={'source':str(src.relative_to(root)),'source_sha256':h,'contract_version':'ASR-QA-v6','date':'2026-10-11','status':'AUTHOR_DEFINED_POC_CONTRACT_NO_CANDIDATE_MEASUREMENT','rows':rows,'references':references,'common_workload_sha256':wm['input_sha256'],'workload_manifest_sha256':hashlib.sha256((src.parent/'03-02-workload-manifest.json').read_bytes()).hexdigest(),'common_workload':w,'workload_summary':u,'common_workload_detail':chapters[8].split('### 8.1',1)[0],'case_mapping':w['cases'],'category_cases':wm['category_cases'],'reference_style':{'summary':'07-품질요구사항.png','metrics':'39~44-Appendix-QAMetric*.png','interpretation':'스타일/구조만 적용. 외부 과제 요구/수치/순위를 VIA로 이관하지 않음'}}
for folder,file in [('quality-attributes','quality-attributes.json'),('quality-metrics','quality-metrics.json')]:
 emit((root/'docs/presentations_files'/folder/file), json.dumps(data,ensure_ascii=False,indent=2)+'\n')
caseblocks=re.findall(r'##### (S[1-6]-\d{2}) ([^\n]+)\n(.*?)(?=\n<a id=|\n#### 3\.2\.3)',s,re.S)
assert len(caseblocks)==21,len(caseblocks)
case_registry={'contract_version':'ASR-QA-v6','accuracy_version':'Voice-PoC-v4','source_sha256':h,'cases':[{'id':i,'name':name,'specification':text.strip(),'required_conditions':[f'C{j}' for j in range(1,7)]} for i,name,text in caseblocks],'trials':105,'binary_conditions':630}
emit((root/'docs/presentations_files/quality-attributes/functional-coverage.json'), json.dumps(case_registry,ensure_ascii=False,indent=2)+'\n')
plan={'source':data['source'],'source_sha256':h,'contract_version':'ASR-QA-v6','status':data['status'],'priority':[r['id'] for r in rows],'populations':{'accuracy':{'cases':21,'repeats':5,'trials':105,'denominator':630},'reaction':{'events':27,'repeats':5,'trials':135},'cycle':{'cycles':19,'repeats':5,'trials':95},'cost_memory':{'workload':'P21','user_scenes':19,'agent_scenes':2,'internal_agent_events':4,'independent_runs':5},'change':{'fixtures':6,'responsibilities':18}},'targets':dict(zip([r['id'] for r in rows],[98,1,10,0.50,3,4096])),'summary':[{k:r[k] for k in ['id','name','description','method','target','targetBasis','guards']} for r in rows]}
emit((root/'docs/presentations_files/quality-attributes/evaluation-plan.json'), json.dumps(plan,ensure_ascii=False,indent=2)+'\n')
emit((root/'docs/presentations_files/quality-attributes/measurement-design.md'), '# ASR-QA 평가 설계 안내\n\n이 파일은 [03-02 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 발표용 연결 안내다. 정의/정답/분모/목표를 이곳에서 독립 개정하지 않는다.\n\n| QA | 원본 | 평가 단위 |\n| --- | --- | --- |\n'+ '\n'.join(f'| {r["id"]} {r["name"]} | 03-02 §{r["definitionSection"]} | {r["shortMetric"]} |' for r in rows)+'\n\n정확성 21개 상세 명세는 [사례 투영 JSON](functional-coverage.json), 공통 규모/목표는 [평가 안내 JSON](evaluation-plan.json)에 원본 hash와 함께 수록했다. [검산 보고](../../architecture/12-decisions/decision-packages/03-02-poc-calculation-report.md)는 공개 단가/가정 계산이며 후보 성능 결과가 아니다.\n')


base=root/'docs/presentations_files'
archive='../archive/qa-presentations-before-asr-20261011/README.md'
source_url='../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md'
overview=['# VIA 중요 품질속성 발표 요약', '',
 f'[정의 원본 03-02]({source_url})의 ASR-QA-01~06을 세 장으로 요약한다. 번호순 우선순위는 01 → 02 → 03 → 04 → 05 → 06이며 낮은 순위의 약점은 tactic으로 보완한다.', '',
 '> ASR-QA-v6 / 2026-10-11 / 작성자 정의와 참조 계산. 후보 성능과 Windows 실측 결과는 아직 없다.', '',
 '[편집 가능한 PPTX, 6장](quality-attributes/VIA-quality-attributes.pptx) / [PNG 미리보기](quality-attributes/index.html) / [QA별 근거와 등급, 6장](quality-metrics.md)', '',
 '| ID | 이름 | 정의 | 대표 측정 지표 | 목표값 | 과제 차원의 목표 근거 |', '| --- | --- | --- | --- | --- | --- |']
overview += ['| '+' | '.join(r[k] for k in ('id','name','description','method','target','targetBasis'))+' |' for r in rows]
overview += ['', '**공통 P21:** 메커니즘과 오류 경로로 선정한 21개(상황군별 2/5/3/3/4/4)를 5회 독립 초기화한다. 모든 실행을 정확성에 채점하고 같은 장면의 27반응/19사이클을 관측한다. 비용은 같은 5개 활동 평균, 메모리는 같은 5개 활동 최대다. 실제 일일 사용 빈도를 주장하지 않는다.', '',
'정확성 105시나리오/630조건, 반응 135관측, 처리 95사이클이다. 입력 음성 50초/출력 179초와 의미 62호출은 예산이고 실제 사용량으로 대체한다. 초기 자료, 재사용, 네 사건 집중 도착, 미사용 생산/철회도 모두 포함한다.', '',
'## 모든 사례의 QA 대응표', '',
'전부 정확성 C1~C6와 비용/메모리에 포함한다. 아래 QA-02는 사건별 첫 반응, QA-03는 사용자 목표의 완료다. S5-01/02만 새 사용자 목표가 없어 QA-03에 적용하지 않는다. [21개 상세 명세](quality-attributes/functional-coverage.json), [평가 규모](quality-attributes/evaluation-plan.json), [원문 연결](quality-attributes/measurement-design.md)을 따른다.', '',
'| 사례 | 주제 | 선정 이유 / DP | QA-02 | QA-03 |', '| --- | --- | --- | --- | --- |']
for c in w['cases']:
 overview += ['| '+' | '.join([c['case_id'],c['name'],c['selection_reason']+'; '+ '/'.join(c['sensitive_dps']),'<br>'.join(x['start']+' → '+x['end'] for x in c['reactions']),c['cycle']['end'] if c['cycle'] else c['cycle_exclusion']])+' |']
overview += ['', f'이전 자료는 [archive]({archive})에 보존했다. 기존 QA catalog/V ID/ADR/참조 설계는 유지한다. 지표와 보호 조건은 원본을 함께 개정하며 QA 등급은 합산하지 않는다.', '']
for n in range(1,7):overview += [f'![요약 및 사례 대응 {n}](quality-attributes/quality-attributes-{n:02d}.png)', '']
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
 metrics += ['', '출처: '+', '.join(citations)+'. 숫자/목표 선택과 상세 경계는 03-02 §'+str(r['definitionSection'])+' 및 §8.7.', '', f'![{r["id"]} 목표 근거와 등급](quality-metrics/qa-metric-asr-qa-{r["priority"]:02d}.png)', '']
metrics += ['## 변경과 재생성', '', f'이전 자료는 [archive]({archive})에 보존했다. [입력 원장](../architecture/12-decisions/decision-packages/03-02-poc-inputs.json), [검산 보고](../architecture/12-decisions/decision-packages/03-02-poc-calculation-report.md)와 [재생성 안내](quality-metrics/README.md)를 따른다. 상세 정의는 03-02 하나에서 유지하고 문안/JSON/PPTX의 원본 hash를 함께 갱신한다.', '']
emit(base/'quality-metrics.md','\n'.join(metrics))
for kind,count in [('attributes',6),('metrics',6)]:
 md=f'''# VIA ASR-QA {'품질 요구사항 요약' if kind=='attributes' else '목표 근거와 등급 부록'}

[03-02 정의 원본](../../architecture/12-decisions/decision-packages/03-02-quality-attribute-definitions.md)의 여섯 QA를 {'핵심 표 2장, 공통 활동 1장, 전체 사례 대응표 3장에' if kind=='attributes' else '각 QA별 한 페이지에'} 정리했다. 정의/지표/목표/과제 근거, 보호 조건과 번호순 우선순위를 유지한다. 후보 성능은 아직 측정하지 않았다.

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
python3 scripts/architecture/qa_common_workload.py --write
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
 if kind=='attributes':md+='\n정확성은 [21개 상세 사례](functional-coverage.json), 평가 규모는 [evaluation-plan](evaluation-plan.json), 원문 절 연결은 [measurement-design](measurement-design.md)을 따른다.\n'
 emit(base/f'quality-{kind}'/'README.md',md)
print('PASS: six ASR rows, 21 exact case specifications, evaluation plan and presentation prose synchronized')
