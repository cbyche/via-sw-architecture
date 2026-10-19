"""OFFLINE protocol test double. No model quality or latency claim; no oracle import."""
from offline_dp01 import _answer

def _result(passages):
    a=_answer(passages)
    return {'text':a['text'],'used_source_ids':a['targets'],'source_spans':[{k:f[k] for k in ('source_id','revision','start_line','end_line')} for f in a['facts']], 'facts':a['facts'],'constraints_respected':[],'actions_performed':[],'uncertainties':[]}

def respond(payload,meta):
    stage=meta['stage'];d=payload.get('input',{})
    if stage=='dp05_produce_work_record':
        return {'blocks':[{'block_id':'offline-'+p['source_id'],'explanation':p['text'].splitlines()[0], 'source_spans':[{k:p[k] for k in ('source_id','revision','start_line','end_line')}]} for p in d['public_evidence']], 'conditions':[],'unknowns':[],'coverage':[p['source_id'] for p in d['public_evidence']]}
    if stage in ('dp05_interpret_user_request','dp05_repair_user_interpretation'):
        close='없으니' in d['user_utterance'];targets=[s['id'] for s in d['source_catalog'] if not s.get('revoked')]
        return {'action':'CLOSE' if close else 'DELEGATE','purpose':'오프라인 프로토콜 검사용 초안','completion_condition':'실제로 공급한 근거의 작은 초안', 'recipient':None if close else 'minimal-draft-agent', 'targets':targets,'handling':'위임 없이 이 대화를 마칩니다.' if close else '허용된 자료로 짧은 초안을 준비하겠습니다.','artifact_type':'draft'}
    if stage in ('dp05_select_handoff_details','dp05_extract_record','dp05_repair_handoff_selection'):
        blocks=d.get('work_record') or {};chosen=[b['block_id'] for b in blocks.get('blocks',[])]
        refs=[{'source_id':s['id'],'revision':s['revision'],'start_line':s['allowed_lines'][0],'end_line':s['allowed_lines'][1]} for s in d['manifest'] if not s.get('revoked')]
        return {'mode':'DIRECT','summary':'OFFLINE source references or actual produced blocks','references':[] if chosen else refs,'selected_evidence_spans':[],'selected_record_blocks':chosen,'reads':[],'unknowns':[]}
    if stage=='dp05_compose_capsule':return {'summary':'OFFLINE actual source-backed capsule','unknowns':[]}
    if stage=='dp05_receiver_select_or_complete':
        capsule=d.get('capsule') or {};passages=list(capsule.get('passages',[]))
        for b in capsule.get('record_blocks',[]):passages.extend(b.get('supporting_passages',[]))
        if passages:return {'mode':'COMPLETE','result':_result(passages),'reads':[]}
        wanted=set(d['mandatory']['required_source_ids']);return {'mode':'READ','result':None,'reads':[{'source_id':s['id'],'start_line':s['allowed_lines'][0],'end_line':s['allowed_lines'][1]} for s in d['manifest'] if s['id'] in wanted]}
    if stage=='dp05_receiver_recover_read_selection':
        c=d['contract'];return {'reads':[{'source_id':s['id'],'start_line':s['allowed_lines'][0],'end_line':s['allowed_lines'][1]} for s in c['manifest'] if s['id'] in c['mandatory']['required_source_ids']]}
    if stage=='dp05_receiver_result' and meta.get('repetition') in (2,3):return {'text':None,'facts':[]}
    if stage=='dp05_receiver_repair_result' and meta.get('repetition')==3:return {'text':None,'facts':[]}
    if stage in ('dp05_receiver_result','dp05_receiver_repair_result'):return _result(d['actual_evidence'])
    raise ValueError('unhandled OFFLINE dp05 stage '+stage)
