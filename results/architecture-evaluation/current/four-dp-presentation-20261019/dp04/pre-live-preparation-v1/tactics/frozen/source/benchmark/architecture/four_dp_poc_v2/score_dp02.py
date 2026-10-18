"""Frozen finite output markers, not semantic entailment or QA02."""
import re

def contains(text,term):
    text=str(text).replace(',','').lower();term=str(term).replace(',','').lower()
    if term.isdigit():return re.search(r'(?<!\d)'+re.escape(term)+r'(?!\d)',text) is not None
    if term.isascii() and term.isalpha():return re.search(r'(?<![a-z])'+re.escape(term)+r'(?![a-z])',text) is not None
    return ''.join(term.split()) in ''.join(text.split())

def score(result,oracle):
    outputs={o.get('goal_id'):o for o in result.get('outputs',[]) if isinstance(o,dict)};details=[]
    for wanted in oracle.get('content_checks',[]):
        actual=next((o for o in reversed(result.get('outputs',[])) if o.get('goal_id')==wanted['goal_id'] and o.get('revision')==wanted['revision']),{});text=actual.get('text','');missing=[g for g in wanted.get('required_groups',[]) if not any(contains(text,t) for t in g)]
        receipt=actual.get('delivery_receipt',{}).get('accepted') is True if wanted.get('required_receipt') else True
        details.append({'goal_id':wanted['goal_id'],'correct':bool(text) and not missing and receipt,'actual_text':text,'missing_required_groups':missing,'required_receipt_satisfied':receipt,'required_revision':wanted['revision'],'actual_revision':actual.get('revision')})
    return {'correct':sum(d['correct'] for d in details),'total':len(details),'details':details,'scope':'Finite literal groups with numeric/word boundaries. Not full semantic correctness or QA02.'}
if __name__=='__main__':
 assert contains('$120 per month','120') and not contains('1120','120') and not contains('1000','10')
 assert contains('TCP and UDP','TCP') and not contains('TCPIP','TCP')
 print('5 finite DP02 content marker checks PASS')
