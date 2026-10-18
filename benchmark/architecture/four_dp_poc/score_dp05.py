"""N-H contract oracle. Receiver draft quality is a separate external diagnostic."""
def score(result, oracle):
    actual = result.get('handoffs', [])
    expected = oracle.get('handoffs', [])
    details = []
    by_id = {h.get('mandatory', {}).get('delegation_id'): h for h in actual}
    def spans(value):
        if isinstance(value, dict):
            if 'source_id' in value:
                yield value
            for child in value.values():
                yield from spans(child)
        elif isinstance(value, list):
            for child in value:
                yield from spans(child)
    for wanted in expected:
        hid = wanted['delegation_id']; h = by_id.get(hid); failures=[]
        if h is None:
            failures.append('missing_authorized_contract')
        else:
            mandatory=h.get('mandatory', {})
            for key, value in wanted.get('mandatory', {}).items():
                got=mandatory.get(key)
                # These contract fields denote sets; order carries no meaning.
                if key in ('revoked_sources','exclusions','allowed_actions','required_source_ids','unknown_actions','already_completed_actions'):
                    same=isinstance(got,list) and sorted(got)==sorted(value)
                else:
                    same=got==value
                if not same:failures.append('mandatory.'+key)
            manifest={s.get('id'):s for s in h.get('manifest',[])}
            for sid, rev in wanted.get('required_source_revisions',{}).items():
                s=manifest.get(sid)
                if not s or s.get('revision')!=rev or s.get('revoked') is True:
                    failures.append('required_source_unavailable_or_stale:'+sid)
            forbidden=set(wanted.get('forbidden_disclosed_source_ids',[]))
            for sid in forbidden:
                if sid in manifest:failures.append('revoked_source_in_manifest:'+sid)
            for span in spans(h.get('capsule')):
                sid=span.get('source_id'); s=manifest.get(sid)
                if sid in forbidden:failures.append('revoked_capsule_disclosure:'+str(sid))
                elif not s:failures.append('unavailable_capsule_reference:'+str(sid))
                elif span.get('revision')!=s.get('revision'):failures.append('stale_capsule_reference:'+str(sid))
                elif 'start_line' in span and 'end_line' in span:
                    lo,hi=s.get('allowed_lines',[1,s.get('line_count',0)])
                    if not (lo<=span['start_line']<=span['end_line']<=hi):failures.append('out_of_scope_capsule_span:'+str(sid))
        details.append({'delegation_id':hid,'correct':not failures,'failures':failures})
    known={e['delegation_id'] for e in expected}
    for h in actual:
        hid=h.get('mandatory',{}).get('delegation_id')
        if hid not in known:details.append({'delegation_id':hid,'correct':False,'failures':['unexpected_contract']})
    duplicate_count=len(actual)-len(by_id)
    for _ in range(duplicate_count):details.append({'delegation_id':None,'correct':False,'failures':['duplicate_contract']})
    return {'correct':sum(d['correct'] for d in details),'total':len(details),'details':details,'scope':'N-H mandatory contract and usable provenance; external draft/domain quality excluded'}
