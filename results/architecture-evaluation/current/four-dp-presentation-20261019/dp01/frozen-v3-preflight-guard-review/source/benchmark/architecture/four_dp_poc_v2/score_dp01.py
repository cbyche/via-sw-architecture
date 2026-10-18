"""Finite authored bounded-content diagnostic, separate from QA02 referent accuracy.

No model judge. Only final actual detailed output is eligible; progress never substitutes.
Numeric formatting/small paraphrase alternatives are frozen before calibration.
"""
import re

def normalized(text):
    return ''.join(str(text).lower().split()).replace(',','').replace('_','').replace('-','')

def has_term(text,term):
    term=normalized(term)
    if any(c.isdigit() for c in term):return re.search(r"(?<!\d)"+re.escape(term)+r"(?!\d)",text) is not None
    return term in text

def score(result,oracle):
    outputs={o.get('goal_id'):o for o in result.get('outputs',[]) if isinstance(o,dict)}
    details=[]
    for wanted in oracle.get('content_checks',[]):
        actual=outputs.get(wanted['goal_id'],{});text=actual.get('text');present=isinstance(text,str) and bool(text.strip())
        norm=normalized(text or '')
        missing=[g for g in wanted.get('required_groups',[]) if not any(has_term(norm,term) for term in g)]
        answer=actual.get('details',actual.get('receiver',{}));raw=answer.get('source_revisions',answer.get('facts',[]))
        revisions={p.get('source_id'):p.get('revision') for p in raw if isinstance(p,dict)} if isinstance(raw,list) else raw if isinstance(raw,dict) else {}
        current=all(revisions.get(k)==v for k,v in wanted.get('required_source_revisions',{}).items())
        details.append({'goal_id':wanted['goal_id'],'correct':present and not missing and current,'detailed_content_displayed':present,'missing_required_groups':missing,'required_source_revisions_match':current,'actual_detailed_text':text})
    return {'correct':sum(d['correct'] for d in details),'total':len(details),'details':details,'scope':'Frozen finite marker groups on actual detailed outputs; not QA02, full factual entailment or product correctness'}

if __name__=='__main__':
    # Contract checks, not campaign/model evidence.
    oracle={'content_checks':[{'goal_id':'g','required_groups':[['1180000','118만원'],['11월1일']]}]}
    assert score({'outputs':[{'goal_id':'g','text':'1,180,000원, 11월 1일'}]},oracle)['correct']==1
    assert score({'outputs':[{'goal_id':'g','text':'요청 자료를 정리하겠습니다.'}]},oracle)['correct']==0
    assert score({'outputs':[]},oracle)['correct']==0
    assert not has_term(normalized('11,180,000원'),'1180000')
    assert not has_term(normalized('360개월'),'36')
    assert has_term(normalized('1,180,000원'),'1180000')
    print('score_dp01 finite diagnostic checks passed including numeric boundaries')
