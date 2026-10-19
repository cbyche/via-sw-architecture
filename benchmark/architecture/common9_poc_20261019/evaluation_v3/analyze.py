"""Run with python3 -m evaluation_v3.analyze from the common9 directory."""
import argparse
import json
from pathlib import Path
from .evaluate import evaluate, summarize, legacy
from analyze import paired_diagnostics, report_markdown


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('runs', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--repetitions', type=int, default=5)
    parser.add_argument('--mode', choices=('strict', 'legacy_diagnostic'), default='strict')
    args = parser.parse_args()
    content = args.runs.read_text()
    traces = [json.loads(line) for line in content.splitlines() if line.strip()] if args.runs.suffix == '.jsonl' else [json.loads(content)]
    results = [evaluate(trace, mode=args.mode) for trace in traces]
    summary = summarize(results, args.repetitions)
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / 'analysis.json').write_text(json.dumps({'version': 'COMMON9-EVALUATOR-v3', 'mode': args.mode,
        'summary': summary, 'paired_case_level_diagnostics': paired_diagnostics(results), 'results': results}, ensure_ascii=False, indent=2) + '\n')
    rows = [f"| {key} | {r['primary_measurement_eligible_episodes']}/{r['episodes_observed']} | {r['deadline_audit']} | {r['qa01']['unresolved']} | {r['qa02']['status']}; unjudgeable {r['qa02']['technically_unjudgeable']}; semantic incorrect {r['qa02']['semantic_incorrect_judgments']} | {r['qa05']['comparability']} |" for key, r in summary.items()]
    extra = '\n\n## Observation and effect validity\n\nPrimary timing counts and means use technically certified observations with fixed planned denominators. All raw observations and costs remain separately retained; technical unjudgeability is not a model error.\n\n| Phase/config | Certified episodes/attempted | Deadline audit | QA01 unresolved | QA02 validity | QA05 execution comparability |\n| --- | ---: | --- | ---: | --- | --- |\n' + '\n'.join(rows)
    if args.mode == 'legacy_diagnostic':
        extra += '\n\nLEGACY DIAGNOSTIC ONLY: cancellation was not executed. Cost/RSS counterfactuals cannot be reconstructed; no primary pooling.\n'
    (args.output / 'summary.md').write_text(report_markdown(summary) + extra + '\n')


if __name__ == '__main__':
    main()
