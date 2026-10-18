"""Deterministic protocol test double ONLY. Never provider/semantic performance evidence.

Intentionally adopts every catalog source rather than implementing a semantic oracle.
Exercises empty-read rejected-completion fallback and final null/repair using real supplied text.
"""
def _answer(passages):
    facts=[];seen=set()
    for p in passages:
        sid=p.get('source_id');text=p.get('text','')
        if not sid or sid in seen or not text.strip():continue
        seen.add(sid);quote=text.splitlines()[0];statement=f'{sid}: {quote}'
        facts.append({k:p[k] for k in ('source_id','revision','start_line','end_line')}|{'statement':statement,'quote':quote})
    return {'targets':[f['source_id'] for f in facts], 'text':'\n'.join(f['statement'] for f in facts), 'facts':facts, 'source_revisions':[{'source_id':f['source_id'],'revision':f['revision']} for f in facts], 'conditions':[], 'uncertainties':[]}

def respond(payload,meta):
    stage=meta['stage'];data=payload.get('input',{})
    if stage=='dp01_publish_relations':
        return {'records':[{'record_id':'offline-'+p['source_id'],'subject':p['source_id'],'predicate':'OBSERVED','object':'actual source','explanation':p['text'].splitlines()[0],'source_spans':[{k:p[k] for k in ('source_id','revision','start_line','end_line')}]} for p in data['observed_evidence']], 'coverage':{'source_ids':[p['source_id'] for p in data['observed_evidence']],'gaps':[]}}
    if stage=='dp01_select_adopt_or_complete':
        targets=[s['id'] for s in data['catalog'] if not s.get('revoked')]
        available=data.get('available_evidence',[])
        # First requests deliberately propose metadata-only completion with empty reads.
        return {'adoption':{'targets':targets,'intent':'테스트 입력의 실제 자료를 정리합니다.'},'mode':'COMPLETE','answer':_answer(available),'reads':[],'reason':'OFFLINE guard exercise; not semantic performance'}
    if stage=='dp01_recover_read_selection':
        return {'adoption':{'targets':[s['id'] for s in data['catalog'] if not s.get('revoked')],'intent':'테스트 자료를 정리합니다.'},'reads':[{'source_id':s['id'],'start_line':s.get('delivered_lines',[1,s['line_count']])[0] if s.get('delivered_lines') else 1,'end_line':s.get('delivered_lines',[1,s['line_count']])[1] if s.get('delivered_lines') else s['line_count']} for s in data['catalog'] if not s.get('revoked')]}
    if stage=='dp01_source_backed_answer' and meta.get('repetition') in (2,3):return {'targets':[],'text':None,'facts':[]}
    if stage=='dp01_repair_source_backed_answer' and meta.get('repetition')==3:return {'text':None,'facts':[]}
    if stage in ('dp01_source_backed_answer','dp01_repair_source_backed_answer'):return _answer(data['actual_evidence'])
    raise ValueError('unhandled OFFLINE dp01 stage '+stage)
