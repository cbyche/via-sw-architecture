"""Tiny temporary synthetic acceptance only; no real campaign reads/execution."""
import copy
import hashlib
import io
import json
from pathlib import Path
import tempfile
import time
import unittest

import extract_closed_block_v3 as x
import prepare_block_index_v3 as prep
import schema_v3 as schema


def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value) + '\n')
    return reference(path)


def reference(path):
    data = path.read_bytes()
    return {'path': str(path), 'sha256': x.digest(data), 'bytes': len(data)}


class Fixture:
    def __init__(self, root):
        self.root = root
        self.jobs = []
        for ordinal in range(63):
            episode = ('S02', 'S80', 'S08')[ordinal] if ordinal < 3 else 'SYN' + str(ordinal)
            pop = 'COMMON_SUPPLEMENT6' if ordinal == 1 else 'COMMON9'
            for cfg in x.CONFIGS[ordinal % 7:] + x.CONFIGS[:ordinal % 7]:
                self.jobs.append(dict(block_id=pop + '-' + episode, population=pop, episode=episode, rep=1, configuration=cfg))
        common = {e: {'responses': [{'event_id': 'U1', 'response_id': 'R1', 'required_content': 'oracle distinct'}],
                         'goals': [{'id': 'G1', 'start_event': 'U1', 'member_events': ['U1']}], 'bindings': []} for e in ('S02', 'S08')}
        supplement = {'S80': copy.deepcopy(common['S02'])}
        self.sources = []
        files = {x.ORACLES['COMMON9']: common, x.ORACLES['COMMON_SUPPLEMENT6']: supplement,
                 'benchmark/architecture/common_supplement6_20261019/functional-oracles.json': {'S80': {'required': 'functional separate'}}}
        files.update({p: 'synthetic source, never imported' for p in x.REFERENCE_SOURCES})
        for relative, value in files.items():
            path = root / 'frozen/source' / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(value) if relative.endswith('.json') else value)
            self.sources.append(dict(relativePath=relative, frozen_path=str(path), sha256=x.digest(path.read_bytes())))
        self.freeze = dict(version=x.GENERATION, total_episode_runs=441, jobs=self.jobs, sources=self.sources, binary={'sha256': 'synthetic-binary'})
        self.freeze_ref = dump(root / 'freeze.json', self.freeze)
        self.block = self.block_at(1)
        old = dump(root / 'old-closure.json', {'status': 'FIRST_FIXED7_CONFIGURATION_BLOCK_CLOSED_NOT_CAMPAIGN_COMPLETE'})
        bundle = dump(root / 'old-bundle.json', {'version': 'COMMON9_CLOSED_BLOCK_EVIDENCE_v2'})
        review = dump(root / 'old-review.json', {'status': 'SCOPED_REVIEW_NOT_FULL_PASS'})
        auth = dump(root / 'root-authority.json', {'scope': 'ACCEPTED_PRE_V3_S02_BUNDLE_ONLY', 'next_block_ordinal': 1,
                'allowed_bootstrap_sha256': {k: v['sha256'] for k, v in [('historical_closure', old), ('previous_bundle', bundle), ('previous_review', review)]}})
        self.cursor = dict(version=schema.CURSOR_VERSION, freeze_sha256=self.freeze_ref['sha256'], next_block_ordinal=1,
                           bootstrap_kind='ACCEPTED_PRE_V3_S02_BUNDLE', previous_block=self.block_at(0),
                           previous_index={'status': 'NOT_AVAILABLE_PRE_V3', 'ref': None}, historical_closure=old,
                           previous_bundle=bundle, previous_review=review, root_authorization=auth)
        self.cursor_ref = dump(root / 'cursor.json', self.cursor)
        self.lines = []
        for job in self.jobs[7:14]:
            rid = job['configuration'] + '-S80-r1-synthetic'
            directory = root / 'raw/runs' / rid
            initial = directory / 'candidate-initial.json'
            dump(initial, {'text': 'actual authored initial not oracle', 'image_url': 'data:image/png;base64,QUJD'})
            trace = dict(run_id=rid, configuration=job['configuration'], episode=job['episode'], rep=1, phase='FORMAL', binary_sha256='synthetic-binary', end_ns=100, returncode=0,
                         candidate_input_file=str(initial), events=[{'event_id': 'a', 'kind': 'SEMANTIC_ADOPTION', 'observed_ns': 2, 'data': {'stimulus_id': 'U1', 'plan': {'answer': 'actual'}}},
                         {'event_id': 'd', 'kind': 'DISPLAY_PROXY', 'observed_ns': 5, 'data': {'stimulus_id': 'U1', 'text': 'actual answer'}}], calls=[], provided_sources=[], source_nodes=[])
            dump(directory / 'trace.json', trace)
            record = dict(job, state='TERMINATED', returncode=0, stdout=json.dumps({'run_id': rid}) + '\n', stderr='', technical_failure=False, budget_stop=False, at_ns=100)
            self.lines.append((json.dumps(record) + '\n').encode())
        self.progress = root / 'campaign-progress.jsonl'
        self.progress.write_bytes(b''.join(self.lines))

    def block_at(self, ordinal):
        j = self.jobs[ordinal * 7]
        return {**{k: j[k] for k in ('block_id', 'population', 'episode', 'rep')}, 'ordinal_in_frozen_block_order': ordinal,
                'job_indexes': list(range(ordinal * 7, ordinal * 7 + 7))}

    def prepare(self, **kwargs):
        return prep.prepare(self.root, self.freeze_ref['sha256'], self.block, self.cursor_ref, self.root / 'index', **kwargs)

    def build(self, **kwargs):
        p = self.root / 'index/index.json'
        return x.build_bundle(self.root, p, x.digest(p.read_bytes()), self.freeze_ref['sha256'], **kwargs)

    def change_trace(self, fn):
        rid = self.jobs[7]['configuration'] + '-S80-r1-synthetic'
        p = self.root / 'raw/runs' / rid / 'trace.json'
        t = json.loads(p.read_text()); fn(t); dump(p, t)
        return p


class Tests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='v3-tiny-only-')
        self.f = Fixture(Path(self.tmp.name))

    def tearDown(self):
        self.tmp.cleanup()

    def test_valid_supplement_and_order_and_separation(self):
        self.f.prepare(); m, e, w = self.f.build()
        self.assertEqual(m['planned_configurations'], [j['configuration'] for j in self.f.jobs[7:14]])
        self.assertEqual(m['declared_obligations'], {'response': 7, 'goal_relation': 7})
        self.assertIn('functional separate', str(m['additional_supplement_functional_expectation']))
        self.assertIn('actual authored initial', str(e)); self.assertNotIn('oracle distinct', str(e))
        self.assertTrue(all(v['status'] == 'UNREVIEWED' for row in w for v in row['review'].values()))

    def test_wrong_freeze(self):
        with self.assertRaises(ValueError): prep.prepare(self.f.root, 'bad', self.f.block, self.f.cursor_ref, self.f.root / 'index')

    def test_cursor_skip_wrong_freeze_and_bootstrap(self):
        for mutate in [lambda c: c.update(next_block_ordinal=2), lambda c: c.update(freeze_sha256='bad'), lambda c: c.update(bootstrap_kind='synthetic-index')]:
            c=copy.deepcopy(self.f.cursor);mutate(c); cr=dump(self.f.root/'bad-cursor.json',c)
            with self.assertRaises(ValueError): schema.validate_cursor(x.Reader(self.f.root),cr,self.f.freeze,self.f.freeze_ref['sha256'],self.f.block)

    def test_wrong_block_indexes_population_and_ordinal(self):
        for change in [{'job_indexes': [0]*7}, {'population': 'COMMON9'}, {'rep': True}, {'ordinal_in_frozen_block_order': True}]:
            with self.assertRaises(ValueError): schema.block_jobs(self.f.freeze,{**self.f.block,**change})

    def test_duplicate_terminal(self):
        self.f.progress.write_bytes(b''.join(self.f.lines+[self.f.lines[0]]))
        with self.assertRaises(ValueError): self.f.prepare()

    def test_wrong_terminal_state_is_missing_not_closed(self):
        self.f.progress.write_bytes(self.f.progress.read_bytes().replace(b'TERMINATED',b'NOT_TERMINATED'))
        d=self.f.prepare(); self.assertEqual(d['status'],'REQUESTED_BLOCK_NOT_FULLY_IN_CAPTURED_PREFIX')
        self.assertEqual(len(d['planned_jobs']),7)

    def test_partial_utf8_and_unrelated_later_block(self):
        extra=json.dumps({'block_id':'UNRELATED','state':'TERMINATED','configuration':'NOT_SELECTED'}).encode()+b'\n'
        self.f.progress.write_bytes(self.f.progress.read_bytes()+extra+b'\xe3\x81')
        i=self.f.prepare();self.assertEqual(i['source_progress']['trailing_partial_bytes'],2);self.f.build()

    def test_prefix_cap_does_not_fetch_more(self):
        old=prep.MAX_PREFIX;prep.MAX_PREFIX=len(self.f.lines[0])+2
        try:self.assertEqual(self.f.prepare()['status'],'REQUESTED_BLOCK_NOT_FULLY_IN_CAPTURED_PREFIX')
        finally:prep.MAX_PREFIX=old

    def test_append_after_capture_is_allowed(self):
        self.f.prepare(after_capture=lambda p:p.write_bytes(p.read_bytes()+b'{"unrelated":1}\n'));self.f.build()

    def test_selected_mutation_rejected(self):
        with self.assertRaises(ValueError): self.f.prepare(after_capture=lambda p:p.write_bytes(p.read_bytes().replace(b'100',b'101',1)))

    def test_replacement_rejected(self):
        def replace(p):
            q=p.with_suffix('.new');q.write_bytes(p.read_bytes());q.replace(p)
        with self.assertRaises(ValueError):self.f.prepare(after_capture=replace)

    def test_nonzero_and_technical_preserved(self):
        row=json.loads(self.f.lines[0]);row.update(returncode=2,technical_failure=True,budget_stop=True,stderr='retained error');self.f.lines[0]=(json.dumps(row)+'\n').encode();self.f.progress.write_bytes(b''.join(self.f.lines))
        i=self.f.prepare();m,e,w=self.f.build();raw=Path(i['rows'][0]['termination_record']['copied_path']).read_text();self.assertIn('retained error',raw);self.assertEqual(len(w),14)

    def test_missing_trace_keeps_obligations(self):
        p=self.f.change_trace(lambda t:None);p.unlink();self.f.prepare();m,e,w=self.f.build();self.assertEqual(m['status'],'INCOMPLETE_EVIDENCE_UNREVIEWED');self.assertEqual(len(w),14)

    def test_open_bool_and_identity_rejected(self):
        for change in [{'end_ns':True},{'returncode':False},{'phase':'OFFLINE'},{'binary_sha256':'wrong'},{'run_id':'other'}]:
            with tempfile.TemporaryDirectory(prefix='v3-sub-') as td:
                f=Fixture(Path(td));f.change_trace(lambda t:t.update(change));f.prepare()
                with self.assertRaises(ValueError):f.build()

    def test_duplicate_runid_and_bad_stdout(self):
        for stdout in ['not json',json.dumps({'run_id':'../escape'})]:
            with self.assertRaises(ValueError):schema.terminal_run_id({'state':'TERMINATED','stdout':stdout})
        with self.assertRaises(ValueError):schema.terminal_run_id({'state':'NOT_CLOSED','stdout':'{}'})

    def test_hash_and_range_changes_rejected(self):
        self.f.prepare();p=self.f.root/'index/index.json';index=json.loads(p.read_text());index['rows'][0]['termination_record']['original_byte_offset']=1;dump(p,index)
        with self.assertRaises(ValueError):self.f.build()

    def test_saved_eval_unsupported_and_unknown_clock(self):
        self.f.change_trace(lambda t:t['events'].append({'event_id':'unknown','kind':'DISPLAY_PROXY','observed_ns':True,'data':{'stimulus_id':'U1'}}));self.f.prepare()
        with self.assertRaises(ValueError):self.f.build(saved_index={})
        m,e,w=self.f.build();self.assertEqual(w[0]['raw_callback_state'],'AMBIGUOUS_RAW_CALLBACK');self.assertIsNone(w[0]['first_raw_callback'])
        self.assertEqual(x.pointer(()),'')

    def test_caps_and_escape(self):
        self.f.prepare()
        with self.assertRaises(ValueError):self.f.build(max_total=10)
        with self.assertRaises(ValueError):schema.safe_path('../outside',self.f.root)
        with tempfile.TemporaryDirectory(prefix='v3-outside-') as td:
            p=Path(td)/'external';p.write_text('x');link=self.f.root/'escape';link.symlink_to(p)
            with self.assertRaises(ValueError):schema.safe_path(link,self.f.root)

    def test_hash_all_outputs_and_immutable(self):
        self.f.prepare();bundle=self.f.build();out=self.f.root/'bundle';x.write_bundle(out,bundle)
        m=json.loads((out/'manifest.json').read_text())
        for name,expected in m['outputs'].items():self.assertEqual(reference(out/name)['sha256'],expected['sha256'])
        with self.assertRaises(ValueError):x.write_bundle(out,bundle)

    def test_valid_common9_after_actual_previous_v3_index(self):
        self.f.prepare();bundle=self.f.build();x.write_bundle(self.f.root/'bundle',bundle)
        prior=reference(self.f.root/'index/index.json');prior_bundle=reference(self.f.root/'bundle/manifest.json')
        cursor=dict(version=schema.CURSOR_VERSION,freeze_sha256=self.f.freeze_ref['sha256'],next_block_ordinal=2,
                    previous_block=self.f.block,previous_index=prior,previous_bundle=prior_bundle,
                    previous_review=self.f.cursor['previous_review'],root_authorization=self.f.cursor['root_authorization'])
        block=self.f.block_at(2)
        cursor_ref=dump(self.f.root/'cursor2-bootstrap-authority-invalid.json',cursor)
        with self.assertRaises(ValueError):schema.validate_cursor(x.Reader(self.f.root),cursor_ref,self.f.freeze,self.f.freeze_ref['sha256'],block)
        authority={'scope':'ADVANCE_EXACT_PREVIOUS_V3_REVIEW','freeze_sha256':self.f.freeze_ref['sha256'],'next_block_ordinal':2,'previous_block':self.f.block,
                   'allowed_previous_sha256':{k:cursor[k]['sha256'] for k in ('previous_index','previous_bundle','previous_review')}}
        cursor['root_authorization']=dump(self.f.root/'root-authority2.json',authority)
        cursor_ref=dump(self.f.root/'cursor2.json',cursor)
        extra=[]
        for job in self.f.jobs[14:21]:
            rid=job['configuration']+'-S08-r1-synthetic';d=self.f.root/'raw/runs'/rid
            initial=d/'candidate-initial.json';dump(initial,{'text':'common actual'})
            dump(d/'trace.json',dict(run_id=rid,configuration=job['configuration'],episode='S08',rep=1,phase='FORMAL',binary_sha256='synthetic-binary',end_ns=200,returncode=0,candidate_input_file=str(initial),events=[],calls=[]))
            extra.append((json.dumps(dict(job,state='TERMINATED',returncode=0,stdout=json.dumps({'run_id':rid}),stderr='',technical_failure=False,budget_stop=False,at_ns=200))+'\n').encode())
        self.f.progress.write_bytes(self.f.progress.read_bytes()+b''.join(extra))
        prep.prepare(self.f.root,self.f.freeze_ref['sha256'],block,cursor_ref,self.f.root/'index2')
        ip=self.f.root/'index2/index.json';m,e,w=x.build_bundle(self.f.root,ip,x.digest(ip.read_bytes()),self.f.freeze_ref['sha256'])
        self.assertEqual(m['block']['population'],'COMMON9');self.assertIsNone(m['additional_supplement_functional_expectation']);self.assertEqual(len(w),14)
        cursor['previous_bundle']=self.f.cursor['previous_bundle'];bad=dump(self.f.root/'cursor2bad.json',cursor)
        with self.assertRaises(ValueError):schema.validate_cursor(x.Reader(self.f.root),bad,self.f.freeze,self.f.freeze_ref['sha256'],block)

    def test_explicit_stdout_mismatch_and_trace_hash(self):
        self.f.prepare();p=self.f.root/'index/index.json';i=json.loads(p.read_text());i['rows'][0]['run_id']='different';dump(p,i)
        with self.assertRaises(ValueError):self.f.build()
        i['rows'][0]['run_id']=self.f.jobs[7]['configuration']+'-S80-r1-synthetic';i['rows'][0]['trace']['sha256']='bad';dump(p,i)
        with self.assertRaises(ValueError):self.f.build()

    def test_unsupported_origin_mixed_index_and_missing_cfg(self):
        self.f.prepare();p=self.f.root/'index/index.json';original=json.loads(p.read_text())
        for mutate in [lambda i:i['rows'][0].update(run_id_origin={'method':'filename guess'}),lambda i:i['rows'][0].update(population='COMMON9'),lambda i:i['rows'].pop(),lambda i:i['rows'][1].update(configuration=i['rows'][0]['configuration'])]:
            i=copy.deepcopy(original);mutate(i);dump(p,i)
            with self.assertRaises(ValueError):self.f.build()

    def test_manual_closure_is_not_index(self):
        p=self.f.root/'old-closure.json'
        with self.assertRaises(ValueError):x.build_bundle(self.f.root,p,x.digest(p.read_bytes()),self.f.freeze_ref['sha256'])

    def test_wrong_index_root_and_binary(self):
        self.f.prepare();p=self.f.root/'index/index.json';original=json.loads(p.read_text())
        for change in [{'formal_root':'/not-synthetic-root'},{'binary_sha256':'bad'},{'version':'NOT_CLOSED'}]:
            dump(p,{**original,**change})
            with self.assertRaises(ValueError):self.f.build()

    def test_no_false_missing_after_late_file_appearance(self):
        p=self.f.change_trace(lambda t:None);raw=p.read_bytes();p.unlink();self.f.prepare();p.write_bytes(raw)
        m,e,w=self.f.build();self.assertEqual(m['executions'][0]['execution_state'],'MISSING_TRACE');self.assertEqual(len(e),6)

    def test_unmatched_fact_context_and_literal_no_normalization(self):
        self.f.change_trace(lambda t:t['events'].append({'event_id':'r','kind':'PUBLICATION_REJECTED','observed_ns':9,'data':{'reason':'literal rejected'}}));self.f.prepare()
        m,e,w=self.f.build();self.assertIn('literal rejected',x.readable_review(m,e,w));self.assertEqual(w[0]['saved_QA01_selection']['selected'],None)
        self.assertEqual(x.pointer(('a/b','~x')),'/a~1b/~0x')

    def test_selected_copy_mutation_and_incomplete_line(self):
        self.f.prepare();p=self.f.root/'index/terminal-0.jsonl';p.write_bytes(p.read_bytes().rstrip(b'\n'))
        with self.assertRaises(ValueError):self.f.build()

    def test_regular_file_and_protocol_scheme(self):
        with self.assertRaises(ValueError):schema.safe_path('https://example.invalid/file',self.f.root)
        with self.assertRaises(ValueError):schema.safe_path(self.f.root/'raw',self.f.root)

    def test_ties_and_duplicate_ids_stay_ambiguous(self):
        self.f.change_trace(lambda t:t['events'].append(copy.deepcopy(t['events'][1])))
        self.f.prepare();m,e,w=self.f.build();self.assertEqual(w[0]['raw_callback_state'],'AMBIGUOUS_RAW_CALLBACK');self.assertIsNone(w[0]['first_raw_callback'])

    def test_source_checkpoint_hash_dependency_and_caps(self):
        source=self.f.root/'tool.py';source.write_text('synthetic tool')
        checkpoint=self.f.root/'checkpoint.json';dump(checkpoint,{'files':[{'path':str(source),'sha256':x.digest(source.read_bytes())}]})
        refs,total=schema.checkpoint_guard(checkpoint,source);self.assertEqual(total,sum(i['bytes'] for i in refs))
        source.write_text('changed')
        with self.assertRaises(ValueError):schema.checkpoint_guard(checkpoint,source)

    def test_no_execution_imports_or_dynamic_execution(self):
        import ast
        forbidden={'subprocess','socket','urllib','requests','http','evaluation_v3','evaluate','causal_timing','torch','PIL'}
        for path in [Path(x.__file__),Path(prep.__file__),Path(schema.__file__)]:
            tree=ast.parse(path.read_text())
            for node in ast.walk(tree):
                if isinstance(node,ast.Import):self.assertFalse(any(a.name.split('.')[0] in forbidden for a in node.names))
                if isinstance(node,ast.ImportFrom):self.assertNotIn((node.module or '').split('.')[0],forbidden)
                if isinstance(node,ast.Call) and isinstance(node.func,ast.Name):self.assertNotIn(node.func.id,{'eval','exec','__import__'})

    def test_exact_controller_nonzero_null_flags_remain_unknown(self):
        row=json.loads(self.f.lines[0]);row.update(returncode=3,technical_failure=None,budget_stop=None,stderr='actual nonzero host evidence')
        self.f.lines[0]=(json.dumps(row)+'\n').encode();self.f.progress.write_bytes(b''.join(self.f.lines));index=self.f.prepare();m,e,w=self.f.build()
        retained=json.loads(Path(index['rows'][0]['termination_record']['copied_path']).read_bytes())
        self.assertIsNone(retained['technical_failure']);self.assertIsNone(retained['budget_stop']);self.assertEqual(retained['returncode'],3);self.assertEqual(len(w),14)
        del retained['budget_stop']
        self.f.progress.write_bytes((json.dumps(retained)+'\n').encode()+b''.join(self.f.lines[1:]))
        # A new immutable index captures this malformed terminal; extractor rejects missing evidence.
        prep.prepare(self.f.root,self.f.freeze_ref['sha256'],self.f.block,self.f.cursor_ref,self.f.root/'index-missing-flag')
        ip=self.f.root/'index-missing-flag/index.json'
        with self.assertRaises(ValueError):x.build_bundle(self.f.root,ip,x.digest(ip.read_bytes()),self.f.freeze_ref['sha256'])

    def test_prefix_and_selected_range_reserve_before_read(self):
        reader=x.Reader(self.f.root,max_total=9);reader.total_bytes=8
        for requested in (2,schema.MAX_PREFIX):
            handle=io.BytesIO(b'0123456789')
            with self.assertRaises(ValueError):prep.read_reserved(reader,handle,requested,'synthetic range or prefix')
            self.assertEqual(handle.tell(),0);self.assertEqual(reader.total_bytes,8)
        handle=io.BytesIO(b'x');self.assertEqual(prep.read_reserved(reader,handle,1,'synthetic'),b'x');self.assertEqual(reader.total_bytes,9)

    def test_reader_growth_does_not_read_limit_plus_one(self):
        from unittest import mock
        path=self.f.root/'tiny-growing.bin';path.write_bytes(b'{}');original=Path.open;requested=[]
        def opening(p,*args,**kwargs):
            if Path(p)==path and args and args[0]=='rb':
                with original(path,'ab') as out:out.write(b'changed-after-stat')
                handle=original(p,*args,**kwargs)
                class Captured:
                    def __enter__(self):return self
                    def __exit__(self,*unused):handle.close()
                    def read(self,n):requested.append(n);return handle.read(n)
                return Captured()
            return original(p,*args,**kwargs)
        reader=x.Reader(self.f.root,max_total=2)
        with mock.patch.object(Path,'open',opening):
            with self.assertRaises(ValueError) as caught:reader.read(path,limit=1000,parse=False)
        self.assertEqual(requested,[2]);self.assertEqual(reader.total_bytes,2)
        self.assertEqual(caught.exception.evidence_read_attempts[-1]['returned_bytes'],2)


if __name__=='__main__':
    started=time.perf_counter();stream=io.StringIO();result=unittest.TextTestRunner(stream=stream,verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(Tests));elapsed=time.perf_counter()-started
    record={'status':'PASS' if result.wasSuccessful() else 'FAIL','tests':result.testsRun,'elapsed_seconds':elapsed,'scope':'Tiny temporary synthetic fixtures only; no actual progress/index/trace/evaluator/model/parser/candidate/network/build execution','stdout':stream.getvalue(),'source_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),Path(x.__file__),Path(prep.__file__),Path(schema.__file__)]}}
    target=Path(__file__).parent/'v3_preparation';target.mkdir(exist_ok=True);stamp=str(time.monotonic_ns())
    (target/('synthetic-'+stamp+'.json')).write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:v for k,v in record.items() if k not in ['stdout','source_sha256']}))
    if not result.wasSuccessful():print(stream.getvalue());raise SystemExit(1)
