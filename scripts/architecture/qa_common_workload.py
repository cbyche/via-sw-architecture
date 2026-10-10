#!/usr/bin/env python3
"""Compile/check an event-count workload blueprint, without running a candidate."""
from collections import Counter
from fractions import Fraction as F
from pathlib import Path
import argparse,hashlib,json,re
ROOT=Path(__file__).resolve().parents[2]
DIR=ROOT/'docs/architecture/12-decisions/decision-packages'
INPUT=DIR/'03-02-common-workload.json'
SOURCE=DIR/'03-02-quality-attribute-definitions.md'
OUTPUT=DIR/'03-02-workload-manifest.json'

def compile_workload(data,source):
    assert data['contract_version']=='ASR-QA-v5' and data['workload_id']=='W100'
    assert data['independent_repeats']==3 and data['blocks_per_activity']==5 and data['scored_run']=='R1'
    assert data['initial']['active_task_ceiling']==10 and data['initial']['cloud_call_ceiling']==4
    assert data['source_changes_per_block']*data['blocks_per_activity']==20
    assert data['maintenance_ticks_per_block']*data['blocks_per_activity']==10
    assert (data['drain_deadline_seconds'],data['idle_probe_seconds'])==(30,10)
    assert len(data['cases'])==18
    cases={r['case_id']:r for r in data['cases']}
    specs={i:(name,block.strip()) for i,name,block in re.findall(r'##### (S[1-6]-0[1-3]) ([^\n]+)\n(.*?)(?=\n<a id=|\n#### 3\.2\.3)',source,re.S)}
    assert set(cases)==set(specs)=={f'S{s}-0{n}' for s in range(1,7) for n in range(1,4)}
    table_rows={m[0]:m[1:] for m in re.findall(r'^\| (S[1-6]-0[1-3]) \| (\d+) \| (사용자|Agent) \| (\d+) \| (\d+) \| (\d+) \| (I-0[1-6]|없음) \| (T-0[1-6]|없음) \|$',source,re.M)}
    assert set(table_rows)==set(cases)
    for cid,row in cases.items():
        assert table_rows[cid]==tuple(map(str,[row['occurrences'],'사용자' if row['trigger']=='USER' else 'Agent',row['root_voice_seconds'],row['planned_followup_seconds'],row['voice_output_reference_seconds'],row['reaction'] or '없음',row['cycle'] or '없음'])),cid
        spec=specs[cid][1]
        if cid not in ['S5-01','S5-02','S5-03']:
            length=re.search(r'(?:발화 |최초[^\n]*?\()0~(\d+)초|종료/확정 전사(?:는)? t=(\d+)초|t=(\d+)초 발화 종료',spec)
            assert length and int(next(v for v in length.groups() if v))==row['root_voice_seconds'],cid
    reaction_map={cid:obs for obs,cid in re.findall(r'^\| (I-0[1-6]) / (S[1-6]-0[1-3]) \|',source,re.M)}
    cycle_map={cid:obs for obs,cid in re.findall(r'^\| (T-0[1-6]) / (S[1-6]-0[1-3]) \|',source,re.M)}
    assert {v for v in reaction_map.values()}=={f'I-0{i}' for i in range(1,7)}
    assert {v for v in cycle_map.values()}=={f'T-0{i}' for i in range(1,7)}
    for cid,row in cases.items():
        assert row['reaction']==reaction_map.get(cid) and row['cycle']==cycle_map.get(cid),cid
    assert cases['S5-01']['root_voice_seconds']==cases['S5-02']['root_voice_seconds']==0
    assert cases['S5-03']['root_voice_seconds']==2
    assert cases['S6-01']['planned_followup_seconds']==cases['S6-02']['planned_followup_seconds']==3
    occurrences=[]
    for b in range(1,6):
        for phase,sequence in [('CORE',list(cases)),('REUSE',data['extra_user_by_block'][b-1]),('BURST',data['burst_cases'])]:
            for n,cid in enumerate(sequence,1):
                spec=specs[cid][1];row=cases[cid]
                occurrences.append(dict(occurrence_id=f'B{b}-{phase}-{n:02d}',block=b,phase=phase,case_id=cid,trigger=row['trigger'],canonical_case_sha256=hashlib.sha256(spec.encode()).hexdigest(),accuracy_sample=phase=='CORE',reaction=row['reaction'] if phase=='CORE' else None,cycle=row['cycle'] if phase=='CORE' else None,burst_offset_seconds=data['burst_offsets_seconds'][n-1] if phase=='BURST' else None))
    counts=Counter(r['case_id'] for r in occurrences)
    assert counts==Counter({r['case_id']:r['occurrences'] for r in data['cases']}),(counts,data['cases'])
    assert len(occurrences)==data['user_input_scenes']+data['agent_triggered_scenes']==130
    assert sum(r['trigger']=='USER' for r in occurrences)==100
    assert sum(r['trigger']=='AGENT' for r in occurrences)==30
    assert sum(r['accuracy_sample'] for r in occurrences)==data['primary_accuracy_trials']==90
    assert 6*data['primary_accuracy_trials']==data['primary_accuracy_conditions']==540
    assert sum(bool(r['reaction']) for r in occurrences)==data['primary_reaction_trials']==30
    assert sum(bool(r['cycle']) for r in occurrences)==data['primary_cycle_trials']==30
    assert all(sum(r['case_id']==c and r['accuracy_sample'] for r in occurrences)==5 for c in cases)
    root=sum(r['occurrences']*r['root_voice_seconds'] for r in data['cases'])
    answers=sum(r['occurrences']*r['planned_followup_seconds'] for r in data['cases'])
    voice_out=sum(r['occurrences']*r['voice_output_reference_seconds'] for r in data['cases'])
    assert (root,answers,voice_out)==(234,30,914)
    calls=data['reference_semantic_calls'];assert sum(c['count'] for c in calls)==300
    return dict(contract_version=data['contract_version'],workload_id=data['workload_id'],evidence_level='DEFINED_INPUT_BLUEPRINT_NOT_EXECUTION_TRACE',source_sha256=hashlib.sha256(source.encode()).hexdigest(),input_sha256=hashlib.sha256(INPUT.read_bytes()).hexdigest(),summary=dict(user_input_scenes=100,agent_triggered_scenes=30,total_scenes=130,blocks=5,independent_repeats=3,accuracy_trials=90,accuracy_conditions=540,reaction_trials=30,cycle_trials=30,planned_voice_utterances=110,voice_input_root_seconds=root,voice_input_followup_seconds=answers,voice_input_seconds=root+answers,voice_input_minutes=str(F(root+answers,60)),voice_output_reference_seconds=voice_out,voice_output_reference_minutes=str(F(voice_out,60)),reference_semantic_calls=300,source_changes=20,maintenance_ticks=10),occurrences=occurrences)

def main():
    p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');p.add_argument('--check',action='store_true');args=p.parse_args()
    data=json.loads(INPUT.read_text());result=compile_workload(data,SOURCE.read_text());payload=json.dumps(result,ensure_ascii=False,indent=2)+'\n'
    if args.write:OUTPUT.write_text(payload)
    else:assert OUTPUT.read_text()==payload,'Stale shared workload blueprint'
    print('PASS: W100 100 user/30 Agent scenes, 18 exact case links, 90/540 accuracy, 30+30 time observations and voice 264/914 seconds')
if __name__=='__main__':main()
