"""Evaluator-only parallel diagnostic v2; candidates never import this file.

Preserves the old literal diagnostic and applies uniform, pre-formal month-unit/date
equivalence rules to every alternative. No model calls and no expected answer injection.
Marker presence remains a bounded diagnostic, not a general entailment judgment.
"""
import argparse
import calendar
import json
import re
from pathlib import Path

VERSION = 'BOUNDED_CONTENT_EQUIVALENCE_V2_20261018'
MONTHS = {name.lower(): index for index, name in enumerate(calendar.month_name) if name}
MONTHS.update({name.lower(): index for index, name in enumerate(calendar.month_abbr) if name})
MONTH_PATTERN = '|'.join(sorted(MONTHS, key=len, reverse=True))
MONTH_FIRST = re.compile(r'(?<![a-z0-9])(?P<month>'+MONTH_PATTERN+r')\.?\s*(?P<day>\d{1,2})(?:st|nd|rd|th)?(?![a-z0-9])', re.I)
DAY_FIRST = re.compile(r'(?<![a-z0-9])(?P<day>\d{1,2})(?:st|nd|rd|th)?\s+(?P<month>'+MONTH_PATTERN+r')\.?(?![a-z0-9])', re.I)
DURATION = re.compile(r'(?<![a-z0-9.])(?P<count>\d+)\s*(?:-\s*)?months?(?![a-z0-9])', re.I)

def literal_normalize(text):
    return ''.join(str(text).lower().split()).replace(',', '').replace('_', '').replace('-', '')

def equivalent_normalize(text):
    def date(match):
        month = MONTHS[match['month'].lower()]
        day = int(match['day'])
        # Without a year, February 29 is a possible date; impossible month/day pairs are not mapped.
        maximum = 29 if month == 2 else calendar.monthrange(2024, month)[1]
        return f'{month}월{day}일' if 1 <= day <= maximum else match[0]
    text = MONTH_FIRST.sub(date, str(text))
    text = DAY_FIRST.sub(date, text)
    text = DURATION.sub(lambda m: m['count']+'개월', text)
    return literal_normalize(text)

def has_term(text, term, normalize):
    term = normalize(term)
    # Keep numerical boundaries: 36 is not 136, 1 is not 11, and 1180000 is not 11180000.
    if any(c.isdigit() for c in term):
        return re.search(r'(?<!\d)'+re.escape(term)+r'(?!\d)', text) is not None
    return term in text

def evaluate(result, oracle):
    outputs = {o.get('goal_id'): o for o in result.get('outputs', []) if isinstance(o, dict)}
    details = []
    for wanted in oracle.get('content_checks', []):
        actual = outputs.get(wanted['goal_id'], {})
        raw = actual.get('text')
        present = isinstance(raw, str) and bool(raw.strip())
        answer = actual.get('details', actual.get('receiver', {}))
        provenance = answer.get('source_revisions', answer.get('facts', []))
        versions = ({p.get('source_id'): p.get('revision') for p in provenance if isinstance(p, dict)}
                    if isinstance(provenance, list) else provenance if isinstance(provenance, dict) else {})
        current = all(versions.get(sid) == revision for sid, revision in wanted.get('required_source_revisions', {}).items())
        fields = {}
        for label, normalize in [('literal', literal_normalize), ('equivalent', equivalent_normalize)]:
            text = normalize(raw or '')
            missing = [g for g in wanted.get('required_groups', []) if not any(has_term(text, term, normalize) for term in g)]
            fields[label+'_correct'] = present and not missing and current
            fields[label+'_missing_groups'] = missing
        details.append({'goal_id': wanted['goal_id'], 'actual_detailed_text': raw,
                        'detailed_content_displayed': present, 'source_revisions_match': current,
                        'equivalent_normalized_text': equivalent_normalize(raw or ''), **fields})
    return {'version': VERSION, 'total': len(details),
            'literal_correct': sum(d['literal_correct'] for d in details),
            'equivalent_correct': sum(d['equivalent_correct'] for d in details),
            'details': details, 'model_calls': 0,
            'scope': 'Uniform month/months duration and English month/ordinal date equivalence only; literal values retained. Actual meaning/conditional claims require separately recorded manual review.'}

def checks():
    yes = lambda text, term: has_term(equivalent_normalize(text), term, equivalent_normalize)
    assert yes('36-month maintenance', '36개월')
    assert yes('36 months maintenance', '36개월')
    assert not yes('136-month maintenance', '36개월')
    assert not yes('360 months maintenance', '36개월')
    assert yes('November 1st', '11월 1일')
    assert yes('1 November', '11월 1일')
    assert yes('November1st', '11월 1일')
    assert not yes('November 11th', '11월 1일')
    assert not yes('January 11', '11월 1일')
    assert not yes('11 months and 1 day', '11월 1일')
    assert not yes('November 31', '11월 1일')
    assert not yes('11180000 KRW', '1180000')
    assert yes('1,180,000 KRW', '1180000')
    # Normalization cannot create a date that the actual answer omitted.
    assert not yes('1,180,000 KRW with 36-month maintenance; purchase undecided.', '11월 1일')
    print(VERSION+': boundary/month/day/omission checks passed; no model calls')

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        checks()
