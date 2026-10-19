"""Fixed-population callback/goal timing. No timeout imputation or oracle-based endpoint."""
import json
from pathlib import Path
BASE=Path(__file__).resolve().parent

def population(dp,episode,record=None):
    if record is not None:
        from evidence import read_evaluator
        value=read_evaluator(record,Path("goal_manifests")/("dp"+dp+".json"))
    else:value=json.loads((BASE/'goal_manifests'/('dp'+dp+'.json')).read_text())
    if isinstance(value,list):value={v['id']:v for v in value}
    return value[episode]
def duration(spans,lo,hi):
    total=0;end=lo
    for a,b in sorted((max(a,lo),min(b,hi)) for a,b in spans if b>lo and a<hi):
        if b>max(a,end):total+=b-max(a,end);end=max(end,b)
    return total

def analyze_timing(record,expected=None):
    expected=expected or population(record['dp'],record['episode'],record);events=record['events']
    starts={};first={};terminals={};failures=[];goal_starts={};revision_events={}
    for e in events:
        d=e.get('data',{});kind=e.get('kind');at=e.get('elapsed_ns')
        if at is None:continue
        sid=d.get('stimulus_id');gid=d.get('goal_id')
        if kind in ('INPUT_CAPTURED','STIMULUS') and sid:starts.setdefault(str(sid),d.get('input_elapsed_ns',at))
        if kind=='GOAL_START' and gid:goal_starts[str(gid)]=min(goal_starts.get(str(gid),at),d.get('input_elapsed_ns',at))
        if kind=='DISPLAY_PROXY' and sid and isinstance(d.get('text'),str) and d['text'].strip() and d.get('content_type') in ('ADOPTED_HANDLING','DETAILED_RESULT','TASK_QUESTION'):
            first.setdefault(str(sid),e)
        if kind=='GOAL_COMPLETE' and gid and isinstance(d.get('text'),str) and d['text'].strip() and d.get('content_contract_checked') is True:terminals.setdefault(str(gid),[]).append(e)
        if kind=='GOAL_FAILED':failures.append(e)
        if kind=='GOAL_REVISION' and gid:revision_events.setdefault(str(gid),[]).append(e)
    # Rust durations are exact within one Instant origin. Bridge timestamps require alignment;
    # the minimum receipt-offset is an upper estimate of origin with pipe observation uncertainty.
    offsets=[e['observed_ns']-e['elapsed_ns'] for e in events if 'observed_ns' in e and 'elapsed_ns' in e]
    origin=min(offsets) if offsets else record['start_ns'];alignment_spread=max(offsets)-min(offsets) if offsets else None
    stimuli=[]
    for entry in expected['qa01_stimuli']:
        sid=str(entry.get('stimulus_id',entry.get('id'))) if isinstance(entry,dict) else str(entry)
        start=starts.get(sid);f=first.get(sid)
        stimuli.append({'stimulus_id':sid,'input_elapsed_ns':start,'first_elapsed_ns':f['elapsed_ns'] if f else None,'qa01_ms':(f['elapsed_ns']-start)/1e6 if f and start is not None else None,'status':'OBSERVED' if f and start is not None else 'NOT_OBSERVED','first_content':f.get('data') if f else None})
    goals=[]
    for g in expected['qa03_goals']:
        gid=str(g['goal_id']);sid=str(g['origin_stimulus_id']);start=starts.get(sid);required_revision=g.get('final_revision')
        eligible=[e for e in terminals.get(gid,[]) if required_revision is None or e['data'].get('revision')==required_revision]
        terminal=eligible[-1] if eligible else None;end=terminal['elapsed_ns'] if terminal else None
        external=[];unassigned=[]
        for c in record.get('calls',[]):
            if str(c.get('role','VIA')).upper()!='RECEIVER':continue
            causes=c.get('causal_trace',{}).get('goal_ids',[])
            if gid not in causes:
                if not causes:unassigned.append(c.get('call_uuid'))
                continue
            if c.get('send_ns') is not None and c.get('terminal_ns') is not None:external.append((c['send_ns']-origin,c['terminal_ns']-origin))
        elapsed=(end-start) if end is not None and start is not None else None
        excluded=duration(external,start,end) if elapsed is not None else None
        failed=[e for e in failures if e['data'].get('goal_id')==gid]
        goals.append({**g,'start_elapsed_ns':start,'complete_elapsed_ns':end,'elapsed_ms':elapsed/1e6 if elapsed is not None else None,'qa03_ms':(elapsed-excluded)/1e6 if elapsed is not None else None,'receiver_shim_internal_excluded_ms':excluded/1e6 if excluded is not None else None,'receiver_internal_intervals_elapsed_ns':external,'terminal':terminal.get('data') if terminal else None,'status':'OBSERVED' if terminal and start is not None else 'NOT_OBSERVED','failed_attempts':[{'elapsed_ns':e['elapsed_ns'],'input_to_failure_ms':(e['elapsed_ns']-start)/1e6 if start is not None else None,'reason':e['data']}for e in failed], 'attribution_profile':'ACTUAL_ELAPSED_NO_EXTERNAL' if not external else 'API_BOUNDARY_RECEIVER_SHIM_PROXY','unassigned_receiver_calls':unassigned,'causal_graph_claim':'Actual scoped sequential external spans; no sparse-parent full-DAG claim. Model queues, step waits and local serving remain included.'})
    q1=[x['qa01_ms'] for x in stimuli if x['qa01_ms'] is not None];q3=[x['qa03_ms'] for x in goals if x['qa03_ms'] is not None]
    return {'stimuli':stimuli,'goals':goals,'qa01_ms':q1,'qa03_ms':q3,'qa01_expected':len(stimuli),'qa03_expected':len(goals),'qa01_missing':len(stimuli)-len(q1),'qa03_missing':len(goals)-len(q3),'comparison_status':'COMPLETE' if len(q1)==len(stimuli) and len(q3)==len(goals) else 'INCOMPLETE','clock_origin_alignment_ns':origin,'event_receipt_alignment_spread_ns':alignment_spread,'goal_start_events_are_not_the_denominator':True}
