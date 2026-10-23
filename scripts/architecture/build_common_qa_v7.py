#!/usr/bin/env python3
"""QA refinement projection: unchanged timing/resources, unmeasured eight-change QA04."""
import argparse
from collections import defaultdict
import copy
import csv
import hashlib
import json
from pathlib import Path
import re
from statistics import mean

ROOT=Path(__file__).resolve().parents[2]
CURRENT=ROOT/'docs/architecture/12-decisions/decision-packages/current'
BASE=CURRENT/'evaluation/common-qa-v6'
OUT=CURRENT/'evaluation/common-qa-v7'
VERSION='COMMON-QA-MODEL-7.0'
SOURCES=['measurement-appendix.md','qa04-change-scope.md','change-scenarios.md',
         'qa01-first-response-latency.md','qa02-referent-accuracy.md','qa03-via-processing-time.md',
         'qa05-model-cost.md','qa06-local-memory.md','qa07-request-meaning-accuracy.md',
         'qa07-evaluation-plan.md','qa08-on-device-model-memory.md','timing-measurement-contract.md']


def read(p):return json.loads(p.read_text())
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):
    p.parent.mkdir(parents=True,exist_ok=True)
    p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')


def tasks():
    text=(CURRENT/'qa/change-scenarios.md').read_text()
    parts=list(re.finditer(r'^## (Q4-C\d\d) (.+)$',text,re.M))
    result=[]
    legacy={'Q4-C01':'C01','Q4-C02':'C02','Q4-C06':'C04','Q4-C08':'C06'}
    for i,m in enumerate(parts):
        body=text[m.end():parts[i+1].start() if i+1<len(parts) else text.index('## 실행 전 점검')].strip()
        fields={}
        for key,label in [('before','변경 전'),('after','변경 후'),('completion','완료 조건'),('exclusions','제외'),('related_path','관련 경로'),('fixture','시험 자료'),('preserve','유지')]:
            match=re.search(r'^- '+label+r': (.+)$',body,re.M)
            fields[key]=match[1] if match else None
        assert all(fields[k] for k in ['before','after','completion'])
        result.append(dict(task_id=m[1],title=m[2],**fields,source_body=body,
                           category='FORMAT_CHANGE' if i<2 else 'FUNCTION_OR_POLICY_CHANGE',
                           predecessor=legacy.get(m[1]),mapping='REQUIREMENT_LINEAGE_ONLY_NO_RESULT_REUSE' if m[1] in legacy else 'NEW_REQUIREMENT'))
    assert len(result)==8
    return result


def score_qa04(x):
    if x is None:return None
    assert x>=0
    for score,limit in [(6,1),(5,1.4),(4,1.8),(3,2.2),(2,2.6),(1,3)]:
        if x<=limit:return score
    return 0


def evaluate_change(record,registry):
    """Only a complete independently applied patch/acceptance record yields a count."""
    required=['baseline_commit','baseline_verification','fixture_ref','registry_ref','acceptance_ref','regression_ref']
    if any(not record.get(k) for k in required) or record.get('completion_status')!='PASS':return None
    if not registry:return None
    for reg in registry.values():
        if not all(reg.get(k) for k in ['language','source','public_boundary','state_owner','asset_ownership']):return None
        if not isinstance(reg.get('dependencies'),list) or reg.get('baseline_commit')!=record['baseline_commit']:return None
    changes=record.get('module_changes')
    if changes is None:return None
    if not changes:
        return 0 if record.get('already_supported_evidence') else None
    if not record.get('patch_ref'):return None
    ids=[]
    for change in changes:
        mid=change.get('module_id')
        if mid not in registry or not change.get('reason') or not change.get('diff_ref'):return None
        if change.get('operation') not in ['MODIFY','ADD','DELETE','MOVE']:return None
        # A rename/move keeps the registered logical identity instead of counting both paths.
        ids.append(mid)
    return len(set(ids))


def aggregate_changes(records,registry):
    required={f'Q4-C{i:02}' for i in range(1,9)}
    if len(records)!=8 or {r['task_id'] for r in records}!=required:return dict(value=None,score=None,status='NOT_MEASURED')
    values=[evaluate_change(r,registry) for r in records]
    if any(v is None for v in values):return dict(value=None,score=None,status='NOT_MEASURED')
    if len({r['baseline_commit'] for r in records})!=1:return dict(value=None,score=None,status='INCONSISTENT_BASELINE')
    x=mean(values)
    return dict(value=x,score=score_qa04(x),status='IMPLEMENTED_CHANGE_REVIEW')


def source_kind(s,e):
    return s['modality'].upper() if e['kind']=='USER_INPUT' else 'AGENT'


def primary_type(s,e):
    if e['kind']=='AGENT_RESULT':return 'RESULT'
    if e['kind']=='AGENT_QUESTION':return 'CONFIRMATION_QUESTION'
    if e['kind']=='AGENT_NOTICE':return 'NOTICE_WITHDRAWN'
    if s['routing']=='MIXED':return 'COMPOSITE_INTENT'
    if s['routing']=='DELEGATED':return 'DELEGATION_INTENT'
    if int(s['id'][1:])>=17:return 'DELIVERY_CONTROL'
    return 'DIRECT_RESPONSE'


def appendices(d):
    scenarios={s['id']:s for s in d['scenarios']};groups=defaultdict(list);event_labels=[];costs=defaultdict(list)
    for c in d['cases']:
        s=scenarios[c['case_id']]
        for e in c['events']:
            label=dict(config_id=c['config_id'],event_id=e['id'],source_kind=source_kind(s,e),primary_type=primary_type(s,e),profile=e['profile'])
            event_labels.append(label);groups[tuple(label[k] for k in ['config_id','source_kind','primary_type','profile'])].append(e)
        cancelled={a['node'] for a in c['audio'] if a['discard_at'] is not None}
        for u in c['usage']:
            category='CANCELLED_OR_UNUSED' if u['node'] in cancelled or u['node']=='relation_unused' else u['phase'].upper()
            costs[(c['config_id'],category,u['bucket'])].append(u['usd'])
    qa01=[]
    for (cfg,source,kind,profile),events in groups.items():
        onsets=[e['L'] for e in events if e['L'] is not None]
        qa01.append(dict(config_id=cfg,source_kind=source,primary_type=kind,profile=profile,
                         planned=len(events),analytical_onsets=len(onsets),withdrawn=sum(e['status']=='WITHDRAWN_BEFORE_AUDIO' for e in events),
                         mean_s=mean(onsets) if onsets else None,physical_observations=0,evidence='ANALYTICAL_NOT_MEASURED'))
    old=read(CURRENT/'evaluation/common-qa-v5/oracles.json')
    # Labels belong to an expression, never to every referent in its input.
    tags_by_expression={
        'U02:1':['PAST_CONTEXT_OR_SCREEN'], 'U03:1':['EXPLICIT_ID'],
        'U04:1':['ACTIVE_TASK'], 'U05:1':['EXPLICIT_ID'],
        'U06:1':['EXPLICIT_ID'], 'U06:2':['EXPLICIT_ID'], 'U07:1':['EXPLICIT_ID'],
        'U08:1':['PAST_CONTEXT_OR_SCREEN'], 'U09:1':['SELECTION'],
        'U10:1':['POINTING_OR_CIRCLE'],
        'U11:1':['BEFORE_AFTER_SCROLL_OR_FRAME','POINTING_OR_CIRCLE'],
        'U11:2':['BEFORE_AFTER_SCROLL_OR_FRAME','POINTING_OR_CIRCLE'],
        'U12:1':['PAST_CONTEXT_OR_SCREEN'],
        'U13:1':['EXPLICIT_ID'], 'U13:2':['PROVENANCE_LOOKUP'],
        'U14:1':['PAST_CONTEXT_OR_SCREEN'], 'U15:1':['PAST_CONTEXT_OR_SCREEN'],
        'U16:1':['PAST_CONTEXT_OR_SCREEN','CORRECTION'],
        'U17:1':['PENDING_RESPONSE'], 'U18:1':['PENDING_RESPONSE'], 'U18:2':['PENDING_RESPONSE'],
        'U19:1':['PENDING_RESPONSE'], 'U19:2':['PAST_CONTEXT_OR_SCREEN'],
        'U20:1':['PAST_CONTEXT_OR_SCREEN'],
        'U21:1':['SELECTION'], 'U21:2':['PAST_CONTEXT_OR_SCREEN'], 'U21:3':['ACTIVE_TASK'],
        'U22:1':['CURRENT_SCREEN'], 'U22:2':['PENDING_RESPONSE'],
        'U23:1':['PAST_CONTEXT_OR_SCREEN'], 'U23:2':['PENDING_RESPONSE'],
        'U24:1':['BEFORE_AFTER_SCROLL_OR_FRAME','POINTING_OR_CIRCLE'],
        'U24:2':['PAST_CONTEXT_OR_SCREEN','CORRECTION'],
    }
    qa02=[]
    for cfg in d['configs']:
        for item in old['items']:
            uid=item['id']
            for i,refs in enumerate(item['referents']):
                tags=list(tags_by_expression[f'{uid}:{i+1}'])
                if len(refs)>1:tags.append('MULTI_TARGET_SET')
                qa02.append(dict(config_id=cfg['id'],expression_key=f'{uid}:ref{i+1}:r1',tags=tags,
                                 oracle_targets=refs,source_facts=scenarios[uid]['facts'],
                                 correct=None,observed=False,tag_status='AUTHORED_REPORTING_TAGS_NOT_FROZEN_MEASUREMENT'))
    qa02_groups=[]
    for cfg in d['configs']:
        refs=[x for x in qa02 if x['config_id']==cfg['id']]
        for tag in sorted({t for x in refs for t in x['tags']}):
            qa02_groups.append(dict(config_id=cfg['id'],tag=tag,planned=sum(tag in x['tags'] for x in refs),observed=0,correct=None,ratio=None,overlapping=True))
    qa07=[]
    for cfg in d['configs']:
        for routing in ['VIA_LOCAL','DELEGATED','MIXED']:
            cases=[c for c in d['qa07']['manifest']['cases'] if c['routing']==routing]
            qa07.append(dict(config_id=cfg['id'],type=routing,draft_cases=len(cases),unknown=len(cases),observed=0,ratio=None,
                             failure_causes={k:None for k in ['REQUEST_INTERPRETATION','RESPONSE_COMPOSITION','ACTUAL_DELIVERY']},overlapping_causes=True))
    peaks=[]
    for cfg in d['configs']:
        cases=[c for c in d['cases'] if c['config_id']==cfg['id']]
        for qa in ['qa06','qa08']:
            c=max(cases,key=lambda c:c[qa]);peak=c['memory_peak' if qa=='qa06' else 'qa08_peak']
            peaks.append(dict(config_id=cfg['id'],qa=qa,case_id=c['case_id'],peak=peak,value_mb=c[qa],
                              evidence='ANALYTICAL_NOT_MEASURED',product_status='NOT_VERIFIED',budget_mb=512 if qa=='qa06' else 1000,
                              budget_condition='<=' if qa=='qa06' else '<',shared_host_memory_not_additive=True))
    return dict(qa01_event_labels=event_labels,qa01_types=qa01,qa02_expressions=qa02,qa02_tags=qa02_groups,qa07_types=qa07,
                qa05_buckets=[dict(config_id=k[0],phase=k[1],bucket=k[2],items=len(v),usd=sum(v)) for k,v in costs.items()],
                qa05_coverage=dict(bundle='24 common input fixtures, one authored analytical bundle',actual_repetitions=0,
                                   preparation='INCLUDED_IN_MODEL',cancelled_and_unused_generation='INCLUDED_IN_MODEL',
                                   retry='NOT_SCHEDULED_IN_THIS_MODEL_NOT_OBSERVED_ZERO',
                                   post_activity_billing='NO_PROVIDER_LEDGER_OR_BILLING_CLOSE_OBSERVATION',
                                   total_status='ANALYTICAL_ESTIMATE_NOT_ACTUAL_BILL',common28_target_applicable=False),
                memory_peaks=peaks,
                qa03_reference='cases[].required_state_nodes/required_external_nodes/required_channel_nodes/nodes/timing_overlap')


def build(out):
    parent=read(BASE/'artifacts/ledger.json');d=copy.deepcopy(parent);ts=tasks()
    registries=[dict(config_id=c['id'],status='NOT_REGISTERED',baseline_commit=None,modules=[],
                     required_module_fields=['module_id','language','source','public_boundary','state_owner','dependencies','asset_ownership','baseline_commit'],
                     reason='실제 운영 구현/patch가 없어 concept 모듈을 운영 registry로 전용하지 않음') for c in d['configs']]
    records=[]
    for cfg in d['configs']:
        for t in ts:
            records.append(dict(config_id=cfg['id'],task_id=t['task_id'],baseline_commit=None,baseline_verification=None,
                                fixture_ref=None,registry_ref=None,patch_ref=None,module_changes=None,module_count=None,
                                acceptance_ref=None,regression_ref=None,already_supported_evidence=None,completion_status='NOT_MEASURED',status='NOT_MEASURED'))
    d.update(version=VERSION,parent_ledger_sha256=digest(BASE/'artifacts/ledger.json'),generator_sha256=digest(Path(__file__)),
             legacy_changes=d['changes'],legacy_qa04={c:s['qa04'] for c,s in d['summary'].items()},changes=records)
    d['contracts'].update(qa04='QA04-CHANGE-v4',qa04_tasks='Q4-CHANGES-v4',qa04_score='CHG-v4',reporting='QA-REPORTING-v1')
    d['qa04']=dict(tasks=ts,registries=registries,records=records,target=3,target_status='PROPOSAL',
                  status='NOT_MEASURED',summary={c['id']:aggregate_changes([r for r in records if r['config_id']==c['id']],{}) for c in d['configs']})
    for cfg,s in d['summary'].items():
        s.update(qa04=None,qa04_score=None,qa04_status='NOT_MEASURED',change_tasks=8,qa08_gb=s['qa08']/1000)
        d['deltas'][cfg]['qa04']=None
    d['policy']['reporting_v7']='One representative per QA; QA01 WAIT beside main; subtype/cause detail in appendix; no P95'
    d['appendices']=appendices(d)
    d['scope_review']=dict(qa01='UNCHANGED_ENDPOINT_AND_FROZEN_NORMAL_WAIT',qa03='UNCHANGED_FULL_COMPLETION_DAG',
                          qa04='NEW_EIGHT_TASKS_NO_PATCH',qa05='SAME_MODEL_COST_NO_ACTUAL_BILLING_CLOSE',
                          qa06='WHOLE_PRODUCT_INCLUDING_ALL_LOCAL_MODELS_512_MB',qa08='ATTRIBUTABLE_CONCURRENT_PEAK_GB_NO_ADDITION_TO_QA06',
                          qa02='UNOBSERVED_SINGLE_RATIO',qa07='UNFROZEN_UNOBSERVED_SINGLE_RATIO')
    save(out/'artifacts/ledger.json',d);save(out/'artifacts/summary.json',d['summary']);save(out/'artifacts/appendices.json',d['appendices'])
    save(out/'qa04-change-manifest.json',d['qa04'])
    save(out/'preserved-v6-sha256.json',{str(p.relative_to(ROOT)):digest(p) for p in sorted(BASE.rglob('*')) if p.is_file()})
    source_manifest={}
    for name in SOURCES:
        src=CURRENT/'qa'/name;dest=out/'definition-snapshots'/(name+'.txt');dest.parent.mkdir(parents=True,exist_ok=True)
        pinned=OUT/'definition-snapshots'/(name+'.txt')
        source_bytes=pinned.read_bytes() if pinned.exists() else src.read_bytes()
        dest.write_bytes(source_bytes)
        source_manifest[name]=dict(source=str(src.relative_to(ROOT)),snapshot=str(dest.relative_to(out)),sha256=hashlib.sha256(source_bytes).hexdigest())
    save(out/'definition-sources.json',source_manifest)
    with (out/'artifacts/comparison.csv').open('w') as f:
        w=csv.writer(f,lineterminator='\n');w.writerow(['config_id','qa01_s','qa02_pct','qa03_s','qa04_modules_per_change','qa05_usd_per_bundle','qa06_mb','qa07_pct','qa08_gb','evidence'])
        for c in d['configs']:
            s=d['summary'][c['id']];w.writerow([c['id'],s['qa01'],s['qa02'],s['qa03'],s['qa04'],s['qa05'],s['qa06'],s['qa07'],s['qa08_gb'],d['evidence_kind']])
    lines=['# QA04 공통 8개 변경 과제', '', '[계약](01-contract-and-scope.md) / [정본](../../qa/change-scenarios.md) / [운영 모듈 등록과 증거](qa04-change-manifest.json)', '',
           '7구성 모두 같은 8개 변경을 동일 기준 코드에서 각각 독립 적용한다. 현재 56개 기록은 전부 NOT_MEASURED다. 기준 commit, 운영 모듈 registry, 실제 diff와 수용/회귀 증거가 없으며 변경 개수는 null이다. 0개나 구형 concept 추정값으로 채우지 않는다.', '',
           '| 과제 | 변경 전 | 변경 후 | 완료 조건 | 이전 과제와 관계 |', '| --- | --- | --- | --- | --- |']
    for t in ts:lines.append('| '+' | '.join([t['task_id']+' '+t['title'],t['before'],t['after'],t['completion'],(t['predecessor']+' 요구 계보만 연결, 수치 재사용 금지') if t['predecessor'] else '새 요구'])+' |')
    lines += ['', '등록부는 Rust/Python 및 제품 코드와 소유 prompt/schema/config, build/이관 모듈을 포함한다. 사전 등록한 모듈 ID, 소스, 공개 경계, 상태와 자산 소유를 실제 patch에 연결한다. 목록이 비어 있다는 것은 아직 등록하지 않았다는 뜻이며 수정 모듈 0개를 뜻하지 않는다.', '',
              '신규/수정/삭제 모듈을 고유 ID로 센다. 단순 이동은 전후 경로를 같은 ID로 연결한다. 테스트/주석/포맷만 바뀐 것은 운영 개수에 포함하지 않는다. 이미 변경 후 기능을 제공하는 경우도 baseline 확인과 수용/회귀 증거로 0개를 입증해야 한다. 여덟 과제 중 하나라도 미완료면 전체 평균과 점수는 미산출이다.', '',
              '기존 C03 좌표/시각 형식과 C05 원본 표시 과제는 새 ID로 재명명하지 않는다. 기존 6개 과제와 추정값은 v6와 Excel의 보존 변경과제 표에만 남긴다.', '']
    (out/'02-qa04-changes.md').write_text('\n'.join(lines))
    print(json.dumps(dict(version=VERSION,qa04_tasks=8,qa04_records=len(records),actual_patches=0,qa01_events=len(d['appendices']['qa01_event_labels'])),ensure_ascii=False))


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,default=OUT);build(p.parse_args().output)
