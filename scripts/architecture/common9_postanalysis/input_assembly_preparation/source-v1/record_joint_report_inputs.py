"""Record one explicitly authorized frozen analysis and assemble report inputs.

No episode/controller/model/parser/compiler launch. Never imports frozen code.
The sole subprocess target is the pinned frozen analyze_joint.py, after supplied
quiescence evidence. Actual execution requires a separate root GO; preparation
and synthetic checks do not authorize using this CLI against the campaign.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

import report_adapter as adapter

VERSION = 'JOINT441_REPORT_INPUT_ASSEMBLY_v1'
PLAN_VERSION = 'JOINT441_REPORT_INPUT_ASSEMBLY_PLAN_v1'
ADAPTER_SHA = 'c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0'
FREEZE_SHA = adapter.FREEZE_SHA
APPROVAL_SHA = adapter.APPROVAL_SHA
ANALYZER_RELATIVE = 'benchmark/architecture/common_supplement6_20261019/analyze_joint.py'
ANALYZER_SHA = '469a5c8a57ee1a3e6812b8c85182cc0e4b2fbd59b69fbca8eeb8591a0fcecb49'
MAX_ANALYSIS_BYTES = 128 * 1024 * 1024
HARD_ANALYSIS_BYTES = 512 * 1024 * 1024
MAX_RUNS_BYTES = 4 * 1024 * 1024 * 1024
HARD_RUNS_BYTES = 8 * 1024 * 1024 * 1024
GROUPS = (('COMMON9', 'primary_common9', 315), ('COMMON_SUPPLEMENT6', 'supplement6', 126))
require, InputError = adapter.require, adapter.InputError


def utc():
    return datetime.now(timezone.utc).isoformat()


def safe_path(path, root):
    path = Path(path)
    require(path.is_absolute() and '..' not in path.parts and path.resolve().is_relative_to(root), 'Path outside exact root or traversal')
    require(not any(p.is_symlink() for p in (path, *path.parents)), 'Symlink path/ancestor unsupported')
    return path


def hash_file(path, root, limit):
    """Bounded streaming hash; never parses trace/model/media contents."""
    path = safe_path(path, root)
    before = path.stat()
    require(path.is_file() and before.st_size <= limit, 'Not a bounded regular file: ' + str(path))
    digest = hashlib.sha256()
    count = 0
    with path.open('rb') as handle:
        while chunk := handle.read(1024 * 1024):
            count += len(chunk)
            require(count <= limit, 'Input grew beyond read limit')
            digest.update(chunk)
    after = path.stat()
    require(before.st_size == after.st_size == count and before.st_mtime_ns == after.st_mtime_ns, 'Input changed while hashing: ' + str(path))
    return {'path': str(path), 'sha256': digest.hexdigest(), 'bytes': count}


def check_ref(reference, root, limit):
    require(isinstance(reference, dict) and type(reference.get('bytes')) is int, 'Explicit integer-size file ref required')
    actual = hash_file(reference['path'], root, limit)
    require(all(actual[k] == reference[k] for k in ('path', 'sha256', 'bytes')), 'Source hash/path/bytes mismatch: ' + reference['path'])
    return actual


class ExclusiveOutput:
    def __init__(self, path, root):
        self.root = root
        self.path = safe_path(path, root)
        require(self.path.is_relative_to(root / 'postanalysis'), 'Assembly output must be inside exact formal postanalysis root')
        require(not self.path.exists(), 'Immutable output already exists')
        self.path.mkdir(parents=True, exist_ok=False)
        self.outputs = {}
        self.tool_ref = hash_file(Path(__file__).resolve(), Path(__file__).resolve().parent, 1024*1024)

    def write(self, name, value, *, raw=False):
        require(not Path(name).is_absolute() and '..' not in Path(name).parts, 'Unsafe output member')
        path = self.path / name
        safe_path(path, self.root)
        path.parent.mkdir(parents=True, exist_ok=True)
        data = value if raw else adapter.json_bytes(value)
        with path.open('xb') as handle:
            handle.write(data)
        reference = {'path': str(path), 'sha256': adapter.sha(data), 'bytes': len(data)}
        self.outputs[name] = reference
        return reference

    def adopt(self, name, limit):
        reference = hash_file(self.path / name, self.root, limit)
        self.outputs[name] = reference
        return reference

    def finish(self, status, **metadata):
        manifest = {'version': VERSION, 'status': status, 'outputs': dict(self.outputs),
                    'manifest_written_last': True, 'tool': self.tool_ref,
                    'accepted_adapter_sha256': ADAPTER_SHA, 'spec_sha256': adapter.SPEC_SHA, **metadata}
        self.write('manifest.json', manifest)
        return manifest


def read_plan(plan_path, expected_sha):
    path = Path(plan_path)
    require(path.is_absolute() and '..' not in path.parts and not any(p.is_symlink() for p in (path,*path.parents))
            and path.is_file() and path.stat().st_size <= 2 * 1024 * 1024, 'Explicit bounded nonsymlink plan file required')
    data = path.read_bytes()
    require(adapter.sha(data) == expected_sha, 'Assembly plan hash mismatch')
    plan = json.loads(data)
    require(isinstance(plan, dict) and plan.get('version') == PLAN_VERSION, 'Unsupported assembly plan ABI')
    root = Path(plan['formal_root'])
    require(root.is_absolute() and '..' not in root.parts and not any(p.is_symlink() for p in (root, *root.parents)), 'Unsafe formal root')
    root = root.resolve()
    safe_path(path, root)
    return plan, root, {'path': str(path), 'sha256': expected_sha, 'bytes': len(data)}


def preflight(plan, root):
    require(adapter.sha(Path(adapter.__file__).read_bytes()) == ADAPTER_SHA, 'Accepted adapter source changed')
    reader = adapter.ArtifactReader(root, **plan.get('adapter_read_limits', {}))
    refs = plan['artifacts']
    require(refs['freeze']['path'] == str(root / 'freeze.json') and refs['freeze']['sha256'] == FREEZE_SHA, 'Wrong formal freeze')
    require(refs['approval']['path'] == str(root / 'approval/APPROVAL-BINDING.json') and refs['approval']['sha256'] == APPROVAL_SHA, 'Wrong approval binding')
    freeze, approval = reader.read(refs['freeze']), reader.read(refs['approval'])
    require(freeze.get('version') == 'COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1' and approval.get('formal_freeze_sha256') == FREEZE_SHA, 'Mixed freeze/approval generation')
    require(len(freeze['sources']) == 112 and len({s['relativePath'] for s in freeze['sources']}) == 112, 'Expected exact112 frozen sources')
    q = reader.read(refs['quiescence'])
    require(q.get('version') == 'JOINT441_QUIESCENT_SNAPSHOT_v1' and q.get('controller_state') in ('STOPPED','COMPLETED')
            and q.get('no_active_episode_processes') is True and type(q.get('progress_complete')) is bool and q.get('evidence_refs'), 'Independent quiescence evidence required')
    for reference in q['evidence_refs']:
        reader.read(reference, raw=True)
    for name, original in [('progress', root / 'campaign-progress.jsonl'), ('status', root / 'status.json'), ('ledger', Path(freeze['ledger_path']))]:
        require(refs[name].get('snapshot_of') == str(original) and refs[name]['path'] != str(original)
                and q.get(name + '_snapshot_sha256') == refs[name]['sha256'], 'Immutable quiescence snapshot association differs: ' + name)
    reader.read(refs['progress'], jsonl=True)
    require(reader.read(refs['status']).get('planned') == 441, 'Wrong status plan')
    ledger = reader.read(refs['ledger'])
    require(adapter.decimal(ledger.get('limit_usd')) == adapter.Decimal('40'), 'Wrong budget cap')
    source_map, source_refs = {}, []
    for entry in freeze['sources']:
        relative = entry['relativePath']
        path = root / 'frozen/source' / relative
        require(entry['frozen_path'] == str(path) and '..' not in Path(relative).parts and not Path(relative).is_absolute(), 'Mixed frozen source association')
        reference = {'path': str(path), 'sha256': entry['sha256'], 'bytes': path.stat().st_size}
        reader.read(reference, raw=True)
        source_map[relative] = entry
        source_refs.append(reference)
    require(source_map[ANALYZER_RELATIVE]['sha256'] == ANALYZER_SHA, 'Wrong frozen analyzer source')
    oracles = {}
    for population, relative in [('COMMON9','benchmark/architecture/common9_poc_20261019/oracles.json'), ('COMMON_SUPPLEMENT6','benchmark/architecture/common_supplement6_20261019/oracles.json')]:
        reference, entry = plan['oracles'][population], source_map[relative]
        require(reference['path'] == entry['frozen_path'] and reference['sha256'] == entry['sha256'], 'Wrong population oracle')
        oracles[population] = reader.read(reference)
    adapter.validate_plan(freeze['jobs'], oracles)
    adapter.approved_qa04(reader, plan['qa04'], approval)
    require(refs['runs']['path'] == str(root / 'runs.jsonl'), 'Analyzer input must be the exact formal runs.jsonl')
    runs_limit = plan.get('runs_max_bytes', MAX_RUNS_BYTES)
    require(type(runs_limit) is int and 0 < runs_limit <= HARD_RUNS_BYTES, 'Unsupported streaming runs read limit')
    run_ref = check_ref(refs['runs'], root, runs_limit)
    for name in ('global_call_snapshot','semantic_review_index','registrations'):
        if refs.get(name):
            if name == 'global_call_snapshot':
                require(refs[name].get('snapshot_of') == str(root / 'model-calls.jsonl') and refs[name]['path'] != str(root / 'model-calls.jsonl'), 'Only immutable global call snapshot allowed')
            reader.read(refs[name], jsonl=name != 'registrations')
    limit = plan.get('analysis_max_bytes', MAX_ANALYSIS_BYTES)
    require(type(limit) is int and 0 < limit <= HARD_ANALYSIS_BYTES, 'Unsupported joint-analysis read limit')
    return {'freeze': freeze, 'oracles': oracles, 'reader_inputs': list(reader.inputs.values()), 'source_refs': source_refs,
            'runs_ref': run_ref, 'runs_limit': runs_limit, 'fragment_limit': reader.file_limit, 'oracle_refs': plan['oracles'], 'analysis_limit': limit, 'analyzer_path': source_map[ANALYZER_RELATIVE]['frozen_path']}


def default_runner(argv, stdout, stderr, cwd):
    # This fixed analysis CLI is the only possible subprocess; no shell, extra
    # options, episode runner, report adapter or frozen-module import here.
    return subprocess.run(argv, stdout=stdout, stderr=stderr, cwd=cwd, check=False).returncode


def record_invocation(out, context, root, *, runner=None):
    synthetic = runner is not None
    argv = [str(Path(sys.executable).resolve()), context['analyzer_path'], '--root', str(root), '--output', str(out.path / 'joint-analysis.json')]
    started_ns, started_at = time.monotonic_ns(), utc()
    error, returncode = None, None
    with (out.path / 'analysis.stdout.log').open('xb') as stdout, (out.path / 'analysis.stderr.log').open('xb') as stderr:
        try:
            returncode = (runner or default_runner)(argv, stdout, stderr, str(Path(context['analyzer_path']).parent))
            require(type(returncode) is int, 'Analysis runner returned unsupported exit result')
        except Exception as caught:
            error = {'type': type(caught).__name__, 'message': str(caught)}
    ended_ns, ended_at = time.monotonic_ns(), utc()
    log_refs, log_errors = {}, []
    for name in ('analysis.stdout.log','analysis.stderr.log'):
        try:
            log_refs[name] = out.adopt(name, 64 * 1024 * 1024)
        except (OSError, InputError) as caught:
            log_refs[name] = None
            log_errors.append({'path':str(out.path/name),'type':type(caught).__name__,'message':str(caught)})
    stdout_ref, stderr_ref = log_refs['analysis.stdout.log'], log_refs['analysis.stderr.log']
    output_path = out.path / 'joint-analysis.json'
    output_ref, output_bytes, output_error = None, None, None
    if output_path.exists():
        try:
            safe_path(output_path, root)
            before = output_path.stat()
            require(output_path.is_file() and before.st_size <= context['analysis_limit'], 'Oversized or nonregular joint analysis output')
            output_bytes = output_path.read_bytes()  # sole full read of joint output
            require(output_path.stat().st_mtime_ns == before.st_mtime_ns and len(output_bytes) == before.st_size, 'Analysis output changed while reading')
            output_ref = {'path': str(output_path), 'sha256': adapter.sha(output_bytes), 'bytes': len(output_bytes)}
            out.outputs['joint-analysis.json'] = output_ref
        except (OSError, InputError) as caught:
            output_error = {'type': type(caught).__name__, 'message': str(caught), 'path': str(output_path)}
    receipt = {'version': 'JOINT441_RECORDED_ANALYSIS_EXECUTION_v1', 'invocation_kind': 'SYNTHETIC_FAKE_NO_PROCESS' if synthetic else 'ACTUAL_FROZEN_ANALYZER',
               'actual_invocation_attempted': not synthetic, 'actual_process_invoked': False if synthetic else True if type(returncode) is int else None, 'argv': argv, 'cwd': str(Path(context['analyzer_path']).parent),
               'python_version': sys.version, 'started_monotonic_ns': started_ns, 'ended_monotonic_ns': ended_ns,
               'started_at_utc': started_at, 'ended_at_utc': ended_at, 'elapsed_seconds': (ended_ns-started_ns)/1e9,
               'returncode': returncode, 'exception': error, 'log_errors': log_errors, 'stdout_ref': stdout_ref, 'stderr_ref': stderr_ref,
               'output_ref': output_ref, 'output_error': output_error, 'mode': 'FORMAL_STRICT_FROZEN_JOINT_CLI_NO_OFFLINE',
               'freeze_sha256': FREEZE_SHA, 'binary_sha256': context['freeze']['binary']['sha256'],
               'source_sha256': {s['relativePath']:s['sha256'] for s in context['freeze']['sources']},
               'runs_ref': context['runs_ref'], 'population_oracle_refs': context['oracle_refs'], 'scope': 'No episode/model/parser/build invocation; analysis only after independent quiescence/root GO'}
    receipt_ref = out.write('analysis-execution.json', receipt)
    require(error is None and returncode == 0 and output_ref is not None and output_error is None and not log_errors, 'Recorded frozen analysis failed or output unavailable; receipt/logs preserved')
    return receipt, receipt_ref, output_ref, output_bytes


def validate_joint(joint, freeze, root):
    require(isinstance(joint, dict) and joint.get('version') == 'JOINT441_SEPARATE_POPULATION_ANALYSIS_v1' and joint.get('freeze_sha256') == FREEZE_SHA, 'Unsupported/mixed joint analysis ABI')
    jobs = {adapter.identity(j): j for j in freeze['jobs']}
    traces = {}
    for row in joint['trace_provenance']:
        run = row['run_id']
        require(isinstance(run,str) and run and run not in ('.','..') and '/' not in run and run not in traces, 'Duplicate/unsafe trace run ID')
        require(row['trace_path'] == str(root / 'raw/runs' / run / 'trace.json'), 'Mixed trace path/root')
        traces[run] = row
    identities, runs, observed = set(), set(), []
    for population, group, count in GROUPS:
        value = joint[group]
        require(value.get('planned_episodes') == count and value.get('observed_episodes') == len(value['results']), 'Wrong population counts')
        for index, result in enumerate(value['results']):
            key = (population,result.get('configuration'),result.get('episode'),result.get('rep'))
            run = result.get('run_id')
            require(type(result.get('rep')) is int and key in jobs and key not in identities and run in traces and run not in runs, 'Duplicate/unplanned/mismatched result identity')
            require(result.get('phase') == 'FORMAL' and result.get('version') == 'COMMON9-EVALUATOR-v3' and result.get('evaluation_mode') == 'strict'
                    and ('population' not in result or result['population'] == population), 'Wrong result phase/population/strict evaluator')
            identities.add(key)
            runs.add(run)
            observed.append((key,group,index,result,traces[run]))
        require(isinstance(value['summary'],dict) and all(k in {'FORMAL:'+c for c in adapter.CONFIGS} for k in value['summary']), 'Unsupported summary phase/configuration')
    require(runs == set(traces), 'Unmatched trace provenance/result')
    absent = {k[1:] for k in jobs if k not in identities}
    missing = joint['unexecuted_planned_identities']
    require(all(type(r.get('rep')) is int for r in missing), 'Wrong unexecuted identity repetition type')
    actual = [(r.get('configuration'),r.get('episode'),r.get('rep')) for r in missing]
    require(len(actual) == len(set(actual)) and set(actual) == absent, 'Original analyzer missing-identity list differs; no execution inference is allowed')
    activations = joint['supplement6']['mechanism_activation']
    activation_runs = []
    by_run = {result['run_id']:key for key,_,_,result,_ in observed}
    for value in activations:
        run = value.get('run_id')
        require(value.get('version') == 'CONTRACT20_OBSERVED_ACTIVATION_v1' and run in by_run, 'Unknown mechanism result')
        key = by_run[run]
        require(key[0] == 'COMMON_SUPPLEMENT6' and type(value.get('rep')) is int and (value.get('configuration'),value.get('episode'),value.get('rep')) == key[1:], 'Mechanism identity mismatch')
        activation_runs.append(run)
    require(len(activation_runs) == len(set(activation_runs)) and set(activation_runs) == {run for run,key in by_run.items() if key[0]=='COMMON_SUPPLEMENT6'}, 'Mechanism provenance incomplete/duplicate')
    return observed, identities, by_run


def assemble_fragments(out, plan, context, receipt, receipt_ref, original, joint, root):
    observed, identities, by_run = validate_joint(joint, context['freeze'], root)
    result_index, summary_index, mechanism_index, mappings = [], [], [], []
    def fragment(name, value):
        data = adapter.json_bytes(value)
        require(len(data) <= context['fragment_limit'], 'Fragment exceeds existing explicit adapter read limit: ' + name)
        return out.write(name, data, raw=True)
    for number, (key,group,index,result,trace) in enumerate(observed):
        trace_ref = hash_file(trace['trace_path'], root, 64 * 1024 * 1024)
        require(trace_ref['sha256'] == trace['sha256'], 'Preserved trace differs from actual analysis provenance')
        pointer = '/' + group + '/results/' + str(index)
        result_ref = fragment('results/result-' + str(number).zfill(3) + '.json', result)
        association = {'version':'JOINT441_RESULT_ASSOCIATION_v1','phase':'FORMAL','mode':'strict','evaluator_version':'COMMON9-EVALUATOR-v3',
            'formal_root':str(root),'formal_freeze_sha256':FREEZE_SHA,'binary_sha256':context['freeze']['binary']['sha256'],
            'identity':adapter.key_object(key),'run_id':result['run_id'],'result_ref':result_ref,'trace_ref':trace_ref,'oracle_ref':plan['oracles'][key[0]],
            'source_sha256':receipt['source_sha256'],'recorded_analysis_argv':receipt['argv'],'recorded_by':str(Path(__file__).resolve()),
            'recorded_at':receipt['ended_at_utc'],'analysis_execution_ref':receipt_ref,'original_analysis_ref':original,'original_result_json_pointer':pointer,
            'invocation_kind':receipt['invocation_kind']}
        association_ref=fragment('associations/result-'+str(number).zfill(3)+'.json',association)
        result_index.append({'identity':adapter.key_object(key),'run_id':result['run_id'],'result_ref':result_ref,'trace_ref':trace_ref,'association_ref':association_ref})
        mappings.append({'fragment_ref':result_ref,'original_analysis_ref':original,'original_json_pointer':pointer,'unchanged_data_object':True})
    for population,group,_ in GROUPS:
        for config_key,value in joint[group]['summary'].items():
            cfg=config_key.split(':',1)[1]
            reference=fragment('summaries/'+population+'-'+cfg+'.json',value)
            summary_index.append({'population':population,'configuration':cfg,'ref':reference})
            mappings.append({'fragment_ref':reference,'original_analysis_ref':original,'original_json_pointer':'/'+group+'/summary/'+config_key,'unchanged_data_object':True})
    for index,value in enumerate(joint['supplement6']['mechanism_activation']):
        reference=fragment('mechanisms/activation-'+str(index).zfill(3)+'.json',value)
        mechanism_index.append({'identity':adapter.key_object(by_run[value['run_id']]),'ref':reference})
        mappings.append({'fragment_ref':reference,'original_analysis_ref':original,'original_json_pointer':'/supplement6/mechanism_activation/'+str(index),'unchanged_data_object':True})
    totals={field:joint[field] for field in ('total_actual_recorded_API_cost_usd','unknown_call_count')}
    totals_ref=fragment('frozen-analysis-totals.json',totals)
    mappings.append({'fragment_ref':totals_ref,'original_analysis_ref':original,'original_field_selectors':{field:'/'+field for field in totals},'unchanged_field_values':True})
    def jsonl(name, values):
        return out.write(name,b''.join((json.dumps(v,ensure_ascii=False,sort_keys=True,allow_nan=False)+'\n').encode() for v in values),raw=True)
    planned_rows=[{'identity':adapter.key_object(adapter.identity(job)),'frozen_job':job,
        'analysis_result_present':adapter.identity(job) in identities,'absence_is_not_proof_unexecuted':True} for job in context['freeze']['jobs']]
    jsonl('planned-identities.jsonl',planned_rows)
    out.write('original-unexecuted-labels.json',{'original_analysis_ref':original,'original_json_pointer':'/unexecuted_planned_identities',
        'value':joint['unexecuted_planned_identities'],'interpretation':'Analyzer result-absence labels only; progress determines never-started versus missing evidence'})
    artifacts={name:plan['artifacts'][name] for name in ('freeze','approval','quiescence','progress','status','ledger')}
    artifacts.update(result_index=jsonl('result-index.jsonl',result_index),summary_index=jsonl('summary-index.jsonl',summary_index),
        mechanism_index=jsonl('mechanism-index.jsonl',mechanism_index),frozen_analysis_totals=totals_ref)
    for name in ('global_call_snapshot','semantic_review_index','registrations'):
        if plan['artifacts'].get(name):
            artifacts[name]=plan['artifacts'][name]
    input_manifest={'version':'JOINT441_QUANTITATIVE_INPUT_v1','formal_root':str(root),'artifacts':artifacts,'oracles':plan['oracles'],'qa04':plan['qa04'],
        'assembly_provenance':{'analysis_execution_ref':receipt_ref,'original_analysis_ref':original,'fragment_provenance_file':str(out.path/'fragment-provenance.json'),
            'invocation_kind':receipt['invocation_kind'],'no_re_evaluation_or_semantic_judgment':True,'raw_pairs':'ABSENT_FROM_FROZEN_JOINT_OUTPUT_NOT_INVENTED'}}
    if plan.get('adapter_read_limits'):
        input_manifest['read_limits'] = plan['adapter_read_limits']
    mapping_ref=out.write('fragment-provenance.json',{'original_analysis_ref':original,'analysis_execution_ref':receipt_ref,'fragments':mappings,
        'result_objects':len(result_index),'planned_identities':441,'populations_kept_separate':True,'raw_pairs':'NOT_PROVIDED_BY_FROZEN_JOINT_ANALYZER'})
    input_manifest['assembly_provenance']['fragment_provenance_ref']=mapping_ref
    return out.write('adapter-input.json',input_manifest)


def record_and_assemble(plan_path, plan_sha256, output, *, runner=None):
    plan, root, plan_ref = read_plan(plan_path,plan_sha256)
    out = ExclusiveOutput(output,root)
    started = time.monotonic_ns()
    try:
        context=preflight(plan,root)
        out.write('preflight.json',{'plan_ref':plan_ref,'inputs':context['reader_inputs'],'runs_ref':context['runs_ref'],
            'analysis_max_bytes':context['analysis_limit'],'runs_max_bytes':context['runs_limit'],'fragment_max_bytes':context['fragment_limit'],'quiescence':'CONSUMED_INDEPENDENT_EVIDENCE_NO_PROCESS_CONTROL'})
        receipt,receipt_ref,original,data=record_invocation(out,context,root,runner=runner)
        # Pin all pre-analysis sources/snapshots and runs again after the actual
        # invocation. This is byte identity, not a new evaluator or sample test.
        for reference in context['reader_inputs']:
            check_ref(reference,root,64*1024*1024)
        check_ref(context['runs_ref'],root,context['runs_limit'])
        check_ref(plan_ref,root,2*1024*1024)
        check_ref(out.tool_ref,Path(__file__).resolve().parent,1024*1024)
        require(adapter.sha(Path(adapter.__file__).read_bytes()) == ADAPTER_SHA, 'Accepted adapter changed during analysis')
        joint=json.loads(data,parse_constant=lambda value: (_ for _ in ()).throw(InputError('Nonfinite joint JSON: '+value)))
        del data
        input_ref=assemble_fragments(out,plan,context,receipt,receipt_ref,original,joint,root)
        return out.finish('ASSEMBLED_NOT_REPORTED',plan_ref=plan_ref,adapter_input_ref=input_ref,
            elapsed_seconds=(time.monotonic_ns()-started)/1e9,invocation_kind=receipt['invocation_kind'],actual_report_or_adapter_execution=False)
    except Exception as error:
        out.write('assembly-failure.json',{'status':'ASSEMBLY_FAILED_EVIDENCE_PRESERVED','error_type':type(error).__name__,'message':str(error),
            'plan_ref':plan_ref,'actual_report_or_adapter_execution':False,'elapsed_seconds':(time.monotonic_ns()-started)/1e9})
        return out.finish('ASSEMBLY_FAILED_EVIDENCE_PRESERVED',plan_ref=plan_ref,invocation_kind='SYNTHETIC_FAKE_NO_PROCESS' if runner is not None else 'ACTUAL_FROZEN_ANALYSIS_ATTEMPT_OR_PREFLIGHT_REJECTION',actual_report_or_adapter_execution=False)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan',type=Path,required=True)
    parser.add_argument('--plan-sha256',required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--execute-authorized-frozen-analysis',action='store_true')
    args=parser.parse_args()
    if not args.execute_authorized_frozen_analysis:
        parser.error('No execution: explicit root-authorized frozen-analysis operation flag required')
    result=record_and_assemble(args.plan,args.plan_sha256,args.output)
    print(json.dumps({'status':result['status'],'output':str(args.output)}))
    return 0 if result['status']=='ASSEMBLED_NOT_REPORTED' else 2


if __name__=='__main__':
    raise SystemExit(main())
