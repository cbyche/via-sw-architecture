"""Independent read-back checks of the authored ledger, not product tests."""
from pathlib import Path
import json, math
HERE=Path(__file__).resolve().parent
ROOT=next(p for p in HERE.parents if (p/'INTENT.md').exists())
OUT=ROOT/'docs/presentations_files/dp44-synthetic-backdata-20261022'
d=json.loads((OUT/'synthetic-data.json').read_text())
nodes={x['id']:x for x in d['nodes']}; jobs={x['id']:x for x in d['jobs']}
assert len(nodes)==len(d['nodes'])
def eq(a,b): assert math.isclose(a,b,abs_tol=2e-7), (a,b)
for n in nodes.values():
    eq(n['end_s'],n['start_s']+n['duration_s'])
    if n['parents']:eq(n['start_s'],max(nodes[p]['end_s'] for p in n['parents']))
for g in d['goals']:
    memo={}
    for n in sorted((x for x in nodes.values() if x['episode']==g['episode'] and x['option']==g['option']),key=lambda x:(x['end_s'],x['start_s'])):
        # Zero-duration nodes can have the same endpoint as a parent. Resolve recursively.
        def project(nid):
            if nid in memo:return memo[nid]
            row=nodes[nid]
            if row['end_s']<=g['start_s']:v=0
            else:v=max((project(p) for p in row['parents']),default=0)+(max(0,row['end_s']-max(row['start_s'],g['start_s'])) if row['via'] else 0)
            memo[nid]=v;return v
        project(n['id'])
    eq(g['qa03_s'],max(memo[t] for t in g['terminals']))
for u in d['usage']:
    eq(u['amount_usd'],u['text_input']*u['input_rate']+u['text_output']*u['output_rate']+u['audio_input_s']*u['audio_input_rate']+u['audio_output_s']*u['audio_output_rate'])
    assert u['node_id'] in nodes
for m in d['memory']:eq(m['total_mb'],m['base_mb']+m['audio_mb']+m['metadata_mb'])
assert len(d['inputs'])==80
assert sum(len(i['expressions']) for i in d['inputs'])==120
for inp in d['inputs']:
    rs=[x for x in d['responses'] if x['input_id']==inp['id']]
    assert len(rs)==2 and rs[0]['adopted_meaning']==rs[1]['adopted_meaning']==inp['oracle_meaning']
    eq(rs[0]['first_ui_s'],rs[1]['first_ui_s'])
    for opt in ['A','B']:
        plays=sorted((p for p in d['playback'] if p['episode']==inp['episode'] and p['option']==opt),key=lambda p:p['start_s'])
        for p in plays:
            j=jobs[p['job_id']]
            assert not j['discarded']
            assert p['start_s']>=j['first_s']-1e-7
            assert p['source_offset_s']+p['audio_seconds']<=j['generated_audio_s']+1e-7
            if p['start_s']<0:assert p['end_s']<=inp['acoustic_start_s']+.0800001
            else:assert p['start_s']>=next(r for r in rs if r['option']==opt)['first_meaning_s']
        assert all(a['end_s']<=b['start_s']+1e-7 for a,b in zip(plays,plays[1:]))
for opt in ['A','B']:
    s=d['summary'][opt]; rr=[x for x in d['responses'] if x['option']==opt]
    gg=[x for x in d['goals'] if x['option']==opt];uu=[x for x in d['usage'] if x['option']==opt]
    eq(s['qa01_display_s'],sum(x['first_ui_s'] for x in rr)/len(rr));eq(s['qa01_voice_s'],sum(x['first_voice_s'] for x in rr)/len(rr))
    eq(s['qa03_s'],sum(x['qa03_s'] for x in gg)/len(gg));assert len(gg)==90
    eq(s['qa05_usd'],sum(x['amount_usd'] for x in uu))
    eq(s['qa06_mb'],max(x['total_mb'] for x in d['memory'] if x['option']==opt))
    eq(s['qa04_modules'],sum(len(x['modules']) for x in d['changes'] if x['option']==opt)/6)
    assert len([x for x in d['changes'] if x['option']==opt])==6
assert all(x['active_generation']<=4 and x['audio_mb']<16 for x in d['memory'])
assert all(not x['claims_heard'] for x in d['history'])
assert d['actual_model_calls']==d['actual_product_runs']==0
report=dict(evidence_kind=d['evidence_kind'],status='PASS',checks=['DAG dependencies and durations','90 goal projections per option','billing quantities and rates','same-input semantic fixture','80 inputs and 120 expressions','no overlapping playback','80ms input stop boundary','no invalidated audio playback','source and generation availability','same-time RAM and capacity','C01-C06 counts','summary recalculation'],
    limits=['No empirical accuracy or latency validation','No independent human or subagent review','No product or model execution'],counts={k:len(v) for k,v in d.items() if isinstance(v,list)})
(OUT/'validation.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(report,ensure_ascii=False))
