"""Link adjudicated semantic endpoints to actual clocks and causal graphs."""
from __future__ import annotations

from prototypes.candidates.dp41_45_poc.shared.timing import CausalTimeline, TimingContractError


def timeline_from_snapshot(snapshot):
    timeline = CausalTimeline(clock=lambda:0.)
    timeline.timeline_id = snapshot['timeline_id']
    timeline.nodes = {node['node_id']:node for node in snapshot['nodes']}
    return timeline


def boundary(trial, blueprint):
    kind = blueprint.get('start_kind','SPEECH_END')
    inputs = trial['observed_inputs']
    if kind == 'SPEECH_ONSET':
        matches = [e for e in trial['events'] if e['kind'] == 'INPUT_STARTED']
        return matches[0]['scene_seconds']
    if kind in ('SPEECH_END','FOLLOWUP_SPEECH_END'):
        if not inputs:
            raise TimingContractError('A scheduled user boundary has no actual input receipt')
        return inputs[0]['speech_end_s']
    if kind == 'SOURCE_EVENT_AVAILABLE':
        matches = [r for r in trial['observed_source_availability']
                   if r['event']['event_id'] == blueprint['source_event_id']]
        if len(matches) != 1:
            raise TimingContractError('A scheduled source boundary has no unique actual availability')
        return matches[0]['actual_source_offset_seconds']
    raise TimingContractError('Unsupported actual measurement boundary')


def normalize_trial(trial, adjudication):
    judged = adjudication['final_judgement']
    timeline = timeline_from_snapshot(trial['causal_timeline'])
    origin = trial['scene_origin_evidence_seconds']
    publications = {d['publication_id']:d for d in trial['deliveries'] if d.get('publication_id')}
    def endpoint(publication_id):
        if publication_id is None:
            return None,None
        d = publications[publication_id]
        if d.get('text_presented_s') is None or d['voice_on'] and d.get('voice_onset_s') is None:
            return None,None
        value = max(d['text_presented_s'],d['voice_onset_s']) if d['voice_on'] else d['text_presented_s']
        return value,d['causal_node_id']
    rows, calculations = [], []
    for blueprint in trial['reaction_blueprint']:
        start = boundary(trial,blueprint)
        j = next(r for r in judged['reactions'] if r['id'] == blueprint['id'])
        if blueprint['endpoint_kind'] == 'LOCAL_OUTPUT_STOP':
            stops = [d for d in trial['deliveries'] if d['kind'] == 'LOCAL_STOP']
            first = stops[0] if stops else None
            end,node = (first['end_s'],first['causal_node_id']) if first else (None,None)
            actual,actual_node = end,node
            wrong = False
        else:
            end,node = endpoint(j['valid_publication_id'])
            actual,actual_node = endpoint(j['first_actual_publication_id'])
            wrong = j['wrong_first']
        row = {'id':blueprint['id'],'start_s':start,'end_s':end,'first_actual_s':actual,
               'valid':end is not None,'wrong_first':wrong,'evidence':j['evidence'],
               'reason':j['reason'],'valid_endpoint_node':node,'first_actual_endpoint_node':actual_node}
        for key,endpoint_node in [('attributed_seconds',node),('first_actual_attributed_seconds',actual_node)]:
            if endpoint_node:
                attribution = timeline.attribute(origin+start,endpoint_node)
                row[key] = attribution['attributed_seconds']
                calculations.append({'observation_id':blueprint['id'],'purpose':key,**attribution})
        rows.append(row)
    cycle = None
    if trial['cycle_blueprint']:
        start = boundary(trial,{'start_kind':'SPEECH_END'})
        end,node = endpoint(judged['cycle']['valid_publication_id'])
        cycle = {'id':trial['cycle_blueprint']['id'],'start_s':start,'valid_end_s':end,
                 'completed':end is not None,'evidence':judged['cycle']['evidence'],
                 'reason':judged['cycle']['reason'],'endpoint_node':node}
        if node:
            attribution = timeline.attribute(origin+start,node)
            cycle['attributed_seconds'] = attribution['attributed_seconds']
            cycle['actual_seconds'] = attribution['wall_clock_seconds']
            calculations.append({'observation_id':cycle['id'],'purpose':'cycle',**attribution})
    normalized = {'variant':trial['variant'],'repeat':trial['repeat'],'case_id':trial['case_id'],
                  'conditions':judged['conditions'],'reactions':rows,'cycle':cycle,
                  'mandatory_violations':judged['critical_violations'],
                  'evidence_level':trial['evidence_level'],'judgement_method':adjudication['method']}
    return normalized,calculations
