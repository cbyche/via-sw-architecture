"""N-H contract oracle. Receiver draft quality is a separate external diagnostic."""
def score(result, oracle):
    actual = result.get('handoffs', [])
    expected = oracle.get('handoffs', [])
    details = []
    by_id = {}
    duplicate_contracts = []
    for h in actual:
        hid = h.get('mandatory', {}).get('delegation_id')
        if hid in by_id:duplicate_contracts.append({'delegation_id':hid,'first_admission':by_id[hid],'duplicate_admission':h})
        else:by_id[hid]=h
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
        # Actual model paraphrases are not compared to the old pre-adopted prose.
        # Report required meaning separately from structure/rights; this finite lexical
        # diagnostic is not a general semantic judge or receiver-domain-quality score.
        mandatory=h.get('mandatory',{}) if h else {}
        text=''.join(str(mandatory.get(k,'')) for k in ('purpose','completion_condition')).lower()
        text=''.join(text.split())
        groups=wanted.get('purpose_contract_groups',[])
        missing_groups=[g for g in groups if not any(''.join(term.lower().split()) in text for term in g)]
        details.append({'delegation_id':hid,'correct':not failures,'failures':failures,
                        'purpose_contract_correct':h is not None and not missing_groups,
                        'missing_purpose_groups':missing_groups,
                        'combined_handoff_contract_correct':not failures and not missing_groups,
                        'original_expected_prose':wanted.get('original_expected_prose')})
    known={e['delegation_id'] for e in expected}
    unexpected_contracts=[h for h in actual if h.get('mandatory',{}).get('delegation_id') not in known]
    from score_dp01 import score as content_score
    return {'receiver_content':content_score(result,oracle),'correct':sum(d['correct'] for d in details),'total':len(details),'details':details,
            'unexpected_contracts':unexpected_contracts,'duplicate_contracts':duplicate_contracts,
            'functional_regression':bool(unexpected_contracts or duplicate_contracts),
            'purpose_contract_correct':sum(d.get('purpose_contract_correct',False) for d in details),
            'combined_handoff_contract_correct':sum(d.get('combined_handoff_contract_correct',False) for d in details),
            'scope':'review-v2 N-H structure/currentness/rights and usable provenance; actual purpose/completion paraphrase token-group diagnostics separate; receiver draft/domain quality excluded'}

if __name__=='__main__':
    expected={'handoffs':[{'delegation_id':'h','mandatory':{'delegation_revision':1}}]}
    first={'mandatory':{'delegation_id':'h','delegation_revision':0}}
    later={'mandatory':{'delegation_id':'h','delegation_revision':1}}
    extra={'mandatory':{'delegation_id':'extra'}}
    r=score({'handoffs':[first,later,extra]},expected)
    assert r['total']==1 and r['correct']==0
    assert len(r['duplicate_contracts'])==1 and len(r['unexpected_contracts'])==1
    assert r['functional_regression'] and r['details'][0]['failures']==['mandatory.delegation_revision']
    empty=score({'handoffs':[extra]},{'handoffs':[]})
    assert empty['total']==0 and empty['functional_regression']
    print('score_dp05 fixed denominator/first admission/extra regressions passed')
