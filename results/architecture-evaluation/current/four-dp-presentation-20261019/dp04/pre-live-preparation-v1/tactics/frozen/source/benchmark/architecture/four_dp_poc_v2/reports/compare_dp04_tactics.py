"""Read-only A/A4/B/B4 DP04 comparison; incomplete phases remain pending."""
import collections
import hashlib
import json
import statistics
from pathlib import Path

REPO=next(p for p in Path(__file__).resolve().parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def records(p,v):
    out=[]
    with (p/'runs.jsonl').open() as f:
        for line in f:
            if line.strip():
                r=json.loads(line)
                if r['dp']=='04' and r['variant']==v:out.append(r)
    return out
def body(c):return read(Path(c['request_file']))
def images(value):
    return sum(part.get('type')=='image_url' for message in value['messages'] if isinstance(message.get('content'),list) for part in message['content'])
def user_input(c):
    content=body(c)['messages'][1]['content']
    return json.loads(content[0]['text'] if isinstance(content,list) else content)
def stages(rr):
    out={}
    for r in rr:
        for c in r['calls']:
            s=out.setdefault(c['stage'],dict(calls=0,input_tokens=0,output_tokens=0,usd=0,service_ms=[],queue_ms=[],transmitted_image_occurrences=0))
            s['calls']+=1;s['input_tokens']+=c['input_tokens'];s['output_tokens']+=c['output_tokens'];s['usd']+=c['cost_usd'];s['service_ms'].append((c['terminal_ns']-c['send_ns'])/1e6);s['queue_ms'].append((c['send_ns']-c['enqueue_ns'])/1e6);s['transmitted_image_occurrences']+=images(body(c))
    for s in out.values():s['service_sum_ms']=sum(s['service_ms']);s['service_mean_ms']=statistics.mean(s['service_ms'])
    return out
def main():
    phases={};raw={}
    for v,name in [('A','basic-v1'),('B','basic-v1'),('A4','tactic-dp04-A4'),('B4','tactic-dp04-B4')]:
        p=ROOT/name
        if not (p/'runs.jsonl').is_file():phases[v]={'status':'PENDING','recorded_episodes':0};continue
        rr=records(p,v);raw[v]=rr
        if len(rr)!=12:phases[v]={'status':'PENDING','recorded_episodes':len(rr)};continue
        assert len({(r['episode'],r['repetition']) for r in rr})==12
        s=next(s for s in read(p/'summary.json')['rows'] if s['dp']=='04' and s['variant']==v)
        phases[v]={'status':'COMPLETE','phase_directory':str(p),'recorded_episodes':12,'summary':s,'stages':stages(rr),'cases':[c for c in read(p/'case-summary.json') if c['dp']=='04' and c['variant']==v],'event_counts':dict(collections.Counter(e['kind'] for r in rr for e in r['events'])),'raw_sha256':{r['run_id']:sha(Path(r['input']).with_name('trace.json')) for r in rr}}
    comparison=[];parallel=[];auxiliary=[]
    if all(phases[v]['status']=='COMPLETE' for v in ('A4','B')):
        b={(r['episode'],r['repetition']):r for r in raw['B']}
        for a in raw['A4']:
            br=b[(a['episode'],a['repetition'])]
            ac=[c for c in a['calls'] if c['stage']=='dp04.screen_producer'];bc=[c for c in br['calls'] if c['stage']=='dp04.screen_producer'];assert len(ac)==len(bc)
            for revision,(x,y) in enumerate(zip(ac,bc),1):comparison.append({'episode':a['episode'],'rep':a['repetition'],'revision_index':revision,'A4_call_uuid':x['call_uuid'],'B_call_uuid':y['call_uuid'],'actual_screen_request_json_equal':body(x)==body(y),'actual_screen_response_json_equal':x['response_value']==y['response_value'],'A4_request':x['request_file'],'B_request':y['request_file'],'A4_response':x['response_file'],'B_response':y['response_file']})
    if phases.get('B4',{}).get('status')=='COMPLETE':
        for r in raw['B4']:
            grouped=collections.defaultdict(dict)
            for c in r['calls']:
                if c['stage']=='dp04.screen_producer':grouped[user_input(c)['revision']]['screen']=c
                elif c['stage']=='dp04.speech_producer':grouped[user_input(c)['original']['revision']]['speech']=c
            for revision,pair in grouped.items():
                x,y=pair['screen'],pair['speech'];parallel.append({'run_id':r['run_id'],'revision':revision,'screen_call_uuid':x['call_uuid'],'speech_call_uuid':y['call_uuid'],'actual_send_terminal_overlap_ms':max(0,min(x['terminal_ns'],y['terminal_ns'])-max(x['send_ns'],y['send_ns']))/1e6})
    if phases.get('A4',{}).get('status')=='COMPLETE':
        for r in raw['A4']:
            for c in r['calls']:
                if c['stage']=='dp04.joint_grounding':
                    original=user_input(c)['original'];parser=original.get('auxiliary_screen_parser');auxiliary.append({'run_id':r['run_id'],'call_uuid':c['call_uuid'],'actual_joint_input_has_screen_state':isinstance(parser,dict) and isinstance(parser.get('candidates'),list),'actual_joint_input_has_original_observations':isinstance(original.get('observations'),dict),'transmitted_original_images':images(body(c))})
    out={'scope':'Same four cases ×3 each, phase-separated. Provider send→terminal sums are service spans, not QA01/03 or independently identified structural causality. Actual same helper requests do not force identical remote outputs.','canonical_source':str(Path(__file__).resolve()),'canonical_source_sha256':sha(Path(__file__)),'phases':phases,'A4_vs_B_screen_pairs':comparison,'screen_pair_counts':{'pairs':len(comparison),'equal_actual_requests':sum(x['actual_screen_request_json_equal'] for x in comparison),'equal_actual_responses':sum(x['actual_screen_response_json_equal'] for x in comparison)},'A4_actual_joint_auxiliary_and_pixels':auxiliary,'B4_actual_producer_overlap':parallel,'B4_overlap_summary':{'pairs':len(parallel),'positive_pairs':sum(x['actual_send_terminal_overlap_ms']>0 for x in parallel),'sum_ms':sum(x['actual_send_terminal_overlap_ms'] for x in parallel),'mean_ms':statistics.mean(x['actual_send_terminal_overlap_ms'] for x in parallel) if parallel else None},'provider_calls_created':0,'candidate_runs_created':0}
    target=ROOT/'analysis/dp04-tactic-comparison.json';target.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
    lines=['# DP04 A/A4/B/B4 same-case comparison','',out['scope'],'','|안|상태|QA01 ms 관측/예정|QA03 ms 완료/예정|QA02|calls|USD|RSS MB|','|---|---|---|---|---|---:|---:|---:|']
    for v,p in phases.items():
        if p['status']!='COMPLETE':lines.append(f"|{v}|PENDING {p['recorded_episodes']}/12|—|—|—|—|—|—|");continue
        s=p['summary'];lines.append(f"|{v}|{s['comparison_status']}|{s['qa01_ms']:.3f} {s['qa01_n']}/{s['qa01_expected']}|{s['qa03_ms']:.3f} {s['qa03_n']}/{s['qa03_expected']}|{s['qa02_correct']}/{s['qa02_total']}|{s['model_calls']}|{s['qa05_via_usd']:.7f}|{s['qa06_peak_rss_mb']:.6f}|")
    lines+=['','|안·stage|calls|input/output tokens|USD|HTTP service 총/평균ms|실제 전송 image 횟수|','|---|---:|---:|---:|---:|---:|']
    for v,p in phases.items():
        for stage,s in p.get('stages',{}).items():lines.append(f"|{v} {stage}|{s['calls']}|{s['input_tokens']}/{s['output_tokens']}|{s['usd']:.7f}|{s['service_sum_ms']:.3f}/{s['service_mean_ms']:.3f}|{s['transmitted_image_occurrences']}|")
    lines+=['','Actual helper request/response equality: '+json.dumps(out['screen_pair_counts']),'','B4 actual provider overlap: '+json.dumps(out['B4_overlap_summary']),'', 'Detailed case rows, raw UUID/file references and immutable trace hashes: dp04-tactic-comparison.json. No new model/candidate execution.']
    target.with_suffix('.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'output':str(target),'screen_pair_counts':out['screen_pair_counts'],'phase_status':{v:p['status'] for v,p in phases.items()},'provider_calls_created':0}))
if __name__=='__main__':main()
