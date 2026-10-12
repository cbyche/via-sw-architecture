"""Separate evaluator process: blind semantic grades and durable adjudication."""
from __future__ import annotations

import argparse
import asyncio
import copy
import json
from pathlib import Path

from .fixtures import load_oracles
from .judge import calibrate, grade
from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence, canonical
from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient


def read_jsonl(path):
    return [json.loads(line) for line in Path(path).read_text().splitlines() if line.strip()]


def disagreements(a, b):
    values = [key for key in a['conditions'] if a['conditions'][key]['passed'] != b['conditions'][key]['passed']]
    ar, br = {r['id']: r for r in a['reactions']}, {r['id']: r for r in b['reactions']}
    for key in ar:
        if any(ar[key].get(f) != br[key].get(f) for f in ('first_actual_publication_id','valid_publication_id','wrong_first')):
            values.append(key)
    if a['cycle'].get('valid_publication_id') != b['cycle'].get('valid_publication_id'):
        values.append('cycle')
    if a.get('uncertain') or b.get('uncertain'):
        values.append('uncertain')
    return values


async def evaluate_one(trial, oracle, client, evidence):
    first = await grade(trial, oracle, client, evidence, independent_pass=1)
    second = await grade(trial, oracle, client, evidence, independent_pass=2)
    flags = disagreements(first, second)
    audits = [first, second]
    if flags:
        audits.append(await grade(trial, oracle, client, evidence, independent_pass=3))
    # All disputes receive a final evidence-linked disposition. Majority votes
    # are a documented judge protocol; they are not three independent humans.
    chosen = copy.deepcopy(audits[-1])
    for condition in chosen['conditions']:
        passing = sum(a['conditions'][condition]['passed'] for a in audits)
        value = passing > len(audits)/2
        matching = [a['conditions'][condition] for a in audits if a['conditions'][condition]['passed'] == value]
        chosen['conditions'][condition] = copy.deepcopy(matching[-1])
    if len(audits) == 3:
        for index, reaction in enumerate(chosen['reactions']):
            rows = [next(r for r in a['reactions'] if r['id'] == reaction['id']) for a in audits]
            tuples = [(r.get('first_actual_publication_id'),r.get('valid_publication_id'),r['wrong_first']) for r in rows]
            preferred = next((t for t in tuples if tuples.count(t) >= 2), tuples[-1])
            chosen['reactions'][index] = copy.deepcopy(rows[tuples.index(preferred)])
        ends = [a['cycle'].get('valid_publication_id') for a in audits]
        preferred = next((e for e in ends if ends.count(e) >= 2), ends[-1])
        chosen['cycle'] = copy.deepcopy(audits[ends.index(preferred)]['cycle'])
    violation_candidates = {}
    for audit in audits:
        for violation in audit['critical_violations']:
            violation_candidates.setdefault(violation['kind'], []).append(violation)
    chosen['critical_violations'] = [rows[-1] for kind,rows in violation_candidates.items()
        if sum(any(v['kind'] == kind for v in a['critical_violations']) for a in audits) > len(audits)/2]
    record = {'case_id':trial['case_id'],'variant':trial['variant'],'repeat':trial['repeat'],
              'final_judgement':chosen,'judgements':audits,'disagreement_fields':flags,
              'violation_review_candidates':violation_candidates,
              'method':'TWO_BLIND_PROMPT_PASSES_PLUS_THIRD_DISPUTE_AUDIT',
              'same_judge_model_not_statistical_independence':True,'status':'ADJUDICATED'}
    evidence.write('adjudications',record)
    return record


async def main(args):
    evidence = Evidence(args.output)
    client = ModelClient(evidence,'openai',max_cost_usd=args.max_cost,
                         budget_file=args.budget_file or args.output.parent/'openai-budget.json')
    try:
        calibration = await calibrate(client,evidence)
        print(json.dumps({'calibration':calibration['correct'],'total':calibration['total']}),flush=True)
        if calibration['correct'] != calibration['total']:
            raise RuntimeError('Judge calibration failed; grading must not silently continue')
        oracles = load_oracles()
        trials = []
        for path in args.workers:
            trials += read_jsonl(path/'trials-unjudged.jsonl')
        if args.limit:
            trials = trials[:args.limit]
        semaphore = asyncio.Semaphore(4)
        async def run(trial):
            async with semaphore:
                result = await evaluate_one(trial,oracles[trial['case_id']],client,evidence)
                print(json.dumps({'case':trial['case_id'],'variant':trial['variant'],'repeat':trial['repeat'],
                    'passed':sum(c['passed'] for c in result['final_judgement']['conditions'].values()),
                    'audits':len(result['judgements'])}),flush=True)
                return result
        await asyncio.gather(*(run(t) for t in trials))
        evidence.write('evaluation-summary',{'status':'COMPLETE','trial_count':len(trials),
                                             'provider_usage_price_usd':client.spent})
    finally:
        evidence.close()


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--workers',type=Path,nargs='+',required=True)
    parser.add_argument('--max-cost',type=float,default=20.)
    parser.add_argument('--budget-file',type=Path)
    parser.add_argument('--limit',type=int)
    asyncio.run(main(parser.parse_args()))
