"""Read-only descriptive sidecar; explicit quiescent inputs, no runtime imports.

Preparation is not execution authorization. Root must separately confirm controller
quiescence and issue an exact GO before this CLI is used on real campaign inputs.
"""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
import math
from pathlib import Path
import re
import time

VERSION = 'JOINT441_DP1_DP4_MECHANISM_EVIDENCE_v1'
PLAN_SHA = '071f5872010214aca45f168c585ac1e323f7c303187693f69c45d8e0796e85ea'
FREEZE_SHA = '0753a648e97dbb831c70c127bce383de747333ee0737f1d5b75010cc1f510ebd'
APPROVAL_SHA = 'd3f78606f7a8bad161ee58a0c0fb5e7f7bf83e287140dbc9a4bbe2c028ce93bd'
ADAPTER_SHA = 'c0e369bab675d96cd2cbeb15c79523d89841cdc0a3e91bc2b12a477b94fe77a0'
CAPS = {'trace_bytes': 16*1024*1024, 'request_bytes': 1024*1024,
        'selected_requests_per_run': 100, 'aggregate_input_bytes': 16*1024**3,
        'projected_bytes_per_run': 2*1024*1024, 'metadata_bytes': 2*1024*1024}
CONFIGS = ('AAAA', 'BAAA', 'CAAA', 'ABAA', 'AABA', 'AAAB', 'AAAC')
POPULATIONS = {'COMMON9': (('S02','S08','S11','S12','S15','S18','S19','S20','S26'), 5),
               'COMMON_SUPPLEMENT6': (tuple('S'+str(i) for i in range(80,86)), 3)}
COLLECTIONS = ('source_reads', 'source_reuses', 'record_generations', 'record_occurrences',
               'handoffs', 'receiver_support', 'call_links', 'preparation_and_failures',
               'logical_state_observations', 'join_issues')
COMPLETE = 'COMPLETE_WITHIN_DECLARED_SCOPE'
STAGES = frozenset(('evidence_publish_relations','evidence_select_ranges',
    'handoff_publish_work_record','handoff_select_details','handoff_repair_selection',
    'handoff_compose_capsule','receiver_select_or_complete','receiver_repair_read_selection',
    'receiver_complete','receiver_repair_result'))
BUFFERS = frozenset(('common_evidence_records','common_evidence_working_view',
                    'common_handoff_work_record','common_handoff_acquired_originals'))
PREPARATION = frozenset(('PREPARATION_START','PREPARATION_COMPLETE','PREPARATION_DISCARDED',
    'PREPARATION_FAILED','PRODUCTION_INPUT_COVERAGE','HANDOFF_RECORD_COVERAGE',
    'EVIDENCE_VIEW_EVICTED','SOURCE_RANGE_DENIED','EVIDENCE_REQUEST_READY','GOAL_FAILED'))
HANDOFF = frozenset(('HANDOFF_BUILD_BRANCH','HANDOFF_CONTRACT_BUILT','STALE_HANDOFF_DISCARDED',
                     'AGENT_INGRESS'))
RECEIVER = frozenset(('RECEIVER_RESULT_AVAILABLE','RECEIVER_BRANCH','RECEIVER_READ_RECEIPT',
                      'RECEIVER_RESULT_REJECTED','SERVING_START','SERVING_END'))
ASSOCIATION = frozenset(('SEMANTIC_ADOPTION','ADOPTION_COMMITTED','PERMISSION_OPERATION_APPLIED'))
OPERATIONS = frozenset(('OPERATION_START','OPERATION_END','LOCAL_OPERATION_START','LOCAL_OPERATION_END'))
# Only these scalar string fields are copied. All other prose is digest/ref only.
IDENTIFIERS = frozenset(('event_id','kind','type','node_id','parent_ids','goal_ids','goal_id',
    'stimulus_id','request_id','request_revision','revision','task_id','source_id','record_id',
    'producer_node','acquisition_node','owner','activity_seq','consumer','variant','branch',
    'stage','role','call_uuid','name','unit','action','status','recorded_work_disposition','disposition',
    'start_line','end_line','accepted','external','not_rss','mode','content_type',
    'source_revisions','required_source_ids','used_source_ids','record_ids','selected_record_ids',
    'read_allowed','external_allowed','read_permission','external_share_permission','allowed_actions',
    'completed_actions','unknown_actions','clock_semantics','host_clock_semantics'))
INPUT_FIELDS = frozenset(('actual_evidence','available_originals','published_relations',
    'actually_acquired_originals','work_records','actual_supplied_originals','catalog','manifest',
    'mandatory','denials','contract','event','public_state','actual_public_state','observed_activity',
    'observed_evidence','actual_delivered'))
OWNERS = {'EVIDENCE_RELATION_PRODUCER':'DP1_EVIDENCE',
          'REQUEST_EVIDENCE_ACQUISITION':'DP1_EVIDENCE',
          'HANDOFF_WORK_RECORD_PRODUCER':'DP4_HANDOFF_PREFIX',
          'VIA_HANDOFF_COMPOSER':'DP4_HANDOFF_PREFIX', 'DOWNSTREAM_RECEIVER':'RECEIVER'}


class EvidenceError(ValueError):
    def __init__(self, status, detail):
        super().__init__(detail)
        self.status = status


def require(ok, detail, status='CONFLICT'):
    if not ok:
        raise EvidenceError(status, detail)


def encoded(value):
    return (json.dumps(value, ensure_ascii=False, sort_keys=True,
                       separators=(',', ':'), allow_nan=False)+'\n').encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def strict_json(data):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'Duplicate JSON object key')
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=pairs,
                      parse_constant=lambda _: require(False, 'Nonfinite JSON constant'))


def safe_path(name, root=None):
    require(isinstance(name, (str, Path)), 'Path must be explicit')
    p = Path(name)
    require(p.is_absolute() and '..' not in p.parts, 'Absolute path without traversal required')
    require(not any(x.is_symlink() for x in (p, *p.parents)), 'Symlink path/ancestor unsupported')
    require(root is None or p.resolve().is_relative_to(root), 'Path outside exact formal root')
    return p


def validate_ref(ref, root):
    require(isinstance(ref, dict) and type(ref.get('bytes')) is int and ref['bytes'] >= 0
            and isinstance(ref.get('sha256'), str) and re.fullmatch('[0-9a-f]{64}', ref['sha256']),
            'Explicit SHA256/byte-count reference required')
    require(not ref.get('json_pointer'), 'Whole-file reference required')
    return safe_path(ref.get('path'), root)


def reference(ref, pointer=''):
    return {'recorded_path':ref['path'], 'sha256':ref['sha256'], 'bytes':ref['bytes'], 'pointer':pointer}


def child(pointer, key):
    return pointer+'/'+str(key).replace('~','~0').replace('/','~1')


def project(value, ref, pointer='', key=''):
    """Hash prose and opaque media; never interpret prose or decode media."""
    if isinstance(value, dict):
        return {k: project(v, ref, child(pointer,k), k) for k,v in value.items()}
    if isinstance(value, list):
        return [project(v, ref, child(pointer,i), key) for i,v in enumerate(value)]
    if isinstance(value, str) and key not in IDENTIFIERS:
        data = value.encode()
        return {'sha256':digest(data), 'utf8_bytes':len(data), 'raw_ref':reference(ref,pointer)}
    return value


def decoded_project(value, ref, outer, pointer=''):
    result=project(value,ref,pointer)
    def visit(obj):
        if isinstance(obj,dict):
            if 'raw_ref' in obj:
                obj['decoded_input_pointer']=obj['raw_ref']['pointer']
                obj['raw_ref']=reference(ref,outer)
            for item in obj.values(): visit(item)
        elif isinstance(obj,list):
            for item in obj: visit(item)
    visit(result)
    return result


class Reader:
    def __init__(self, root):
        self.root, self.total, self.inputs, self.attempts = root, 0, [], []
        self.aggregate_exhausted = False

    def capture(self, name, limit, *, expected=None, source_self=False):
        """Every byte, including a failed/growing read, passes the same hard gate."""
        attempt={'recorded_path':str(name),'bytes_read':0,'status':'ATTEMPTED',
                 'expected':expected,'source_self':source_self}
        self.attempts.append(attempt)
        try:
            path=safe_path(name,None if source_self else self.root)
            if source_self:
                require(path==Path(__file__).resolve(),'Only exact source-self may leave formal root')
            before = path.stat()
            require(path.is_file(), 'Input is not a regular file')
            require(before.st_size<=limit,'Per-file byte cap exceeded','LIMIT_EXCEEDED')
            remaining=CAPS['aggregate_input_bytes']-self.total
            if self.aggregate_exhausted or remaining<=0 or before.st_size>remaining:
                self.aggregate_exhausted=True
                raise EvidenceError('LIMIT_EXCEEDED','Aggregate input cap reached')
            allowance=min(limit,remaining)
            attempt['read_allowance_bytes']=allowance
            # No +1 sentinel: even concurrent file growth cannot exceed remaining bytes.
            with path.open('rb') as handle: data=handle.read(allowance)
            self.total+=len(data);attempt['bytes_read']=len(data)
            actual={'path':str(path),'sha256':digest(data),'bytes':len(data)}
            attempt['read_bytes_ref']=actual
            attempt['hash_scope']='BYTES_ACTUALLY_READ_NOT_YET_FULL_FILE_VERIFIED'
            self.inputs.append(actual)
            if self.total>=CAPS['aggregate_input_bytes']: self.aggregate_exhausted=True
            after=path.stat()
            require(after.st_size<=allowance,'Input grew beyond available read allowance','LIMIT_EXCEEDED')
            require((before.st_size,before.st_mtime_ns,before.st_ino)==
                    (after.st_size,after.st_mtime_ns,after.st_ino) and len(data)==before.st_size,
                    'Input changed while reading')
            attempt['hash_scope']='STABLE_FULL_FILE_BYTES'
            if expected is not None:
                require(all(actual[k]==expected[k] for k in actual),'Input hash/bytes mismatch')
            attempt['status']=COMPLETE
            return data,actual
        except (EvidenceError,OSError) as error:
            attempt['status']=getattr(error,'status','MISSING' if isinstance(error,FileNotFoundError) else 'UNSUPPORTED')
            attempt['detail']=str(error)
            if isinstance(error,EvidenceError): raise
            raise EvidenceError(attempt['status'],'Named input unavailable: '+str(name)) from error

    def pinned(self, name, sha, limit):
        require(isinstance(sha,str) and re.fullmatch('[0-9a-f]{64}',sha),'Expected SHA256 required')
        data,actual=self.capture(name,limit)
        try:
            require(actual['sha256']==sha,'Pinned metadata hash mismatch')
            return strict_json(data),actual
        except (ValueError,UnicodeError,RecursionError) as error:
            self.attempts[-1]['status']=getattr(error,'status','UNSUPPORTED')
            self.attempts[-1]['detail']=str(error)
            raise

    def read(self, ref, limit, jsonl=False, raw=False):
        try:
            path=validate_ref(ref,self.root)
        except EvidenceError as error:
            self.attempts.append({'recorded_path':ref.get('path') if isinstance(ref,dict) else None,
                'expected':ref,'bytes_read':0,'status':error.status,'detail':str(error)})
            raise
        data,_=self.capture(path,limit,expected=ref)
        if raw:
            return data
        try:
            return [strict_json(line) for line in data.splitlines() if line.strip()] if jsonl else strict_json(data)
        except (ValueError,UnicodeError,RecursionError) as error:
            self.attempts[-1]['status']=getattr(error,'status','UNSUPPORTED')
            self.attempts[-1]['detail']=str(error)
            if isinstance(error,EvidenceError):
                raise
            raise EvidenceError('UNSUPPORTED','Invalid JSON source') from error


def identity(row):
    require(isinstance(row,dict) and type(row.get('rep')) is int and all(
        isinstance(row.get(k),str) and bool(row[k]) for k in ('population','configuration','episode')),
        'Invalid identity scalar types')
    return tuple(row.get(k) for k in ('population','configuration','episode','rep'))


def key_object(key):
    return dict(zip(('population','configuration','episode','rep'),key))


def validate_plan(jobs):
    keys = [identity(row) for row in jobs]
    expected = {(p,c,e,r) for p,(cases,reps) in POPULATIONS.items() for c in CONFIGS
                for e in cases for r in range(1,reps+1)}
    require(len(keys)==len(set(keys))==441 and set(keys)==expected,
            'Exact unique 315 COMMON9 +126 COMMON_SUPPLEMENT6 plan required')
    return keys


def empty_row(key, status='MISSING', run_id=None):
    return {'identity':key_object(key),'run_id':run_id,'extraction_status':status,
            'scope':'POSTANALYSIS_DESCRIPTIVE_ONLY', 'technical_certification':'NOT_ASSESSED',
            'semantic_certification':'NOT_ASSESSED',
            'collections':{name:{'status':status,'processing_state':'PENDING','items':[]} for name in COLLECTIONS}}


def finish_collection(row,name,status):
    collection=row['collections'][name]
    if collection['processing_state']=='PENDING' and not collection.get('coverage_issue'):
        collection['status']=status
    collection['processing_state']='FINISHED'


def interrupt_row(row,status,detail,ref=None,*,all_collections=False):
    for collection in row['collections'].values():
        if all_collections or collection.get('processing_state')!='FINISHED':
            collection['status']=status;collection['processing_state']='INTERRUPTED'
    row['extraction_status']=status
    issue(row,status,detail,ref)


def issue(row, status, detail, ref=None, collection=None):
    row['collections']['join_issues']['items'].append({'status':status,'detail':detail,'source_ref':ref})
    row['extraction_status'] = status if row['extraction_status']==COMPLETE else row['extraction_status']
    if collection:
        row['collections'][collection]['status'] = status
        row['collections'][collection]['coverage_issue'] = True
    row['collections']['join_issues']['status'] = status
    row['collections']['join_issues']['coverage_issue'] = True


def event_info(event, ref, pointer):
    return {'source_ref':reference(ref,pointer),
            'event':{k:project(v,ref,child(pointer,k),k) for k,v in event.items() if k!='data'},
            'data':project(event.get('data'),ref,child(pointer,'data'))}


def span_matches(a,b):
    return (isinstance(a,dict) and isinstance(b,dict)
            and all(k in a and k in b for k in ('source_id','revision','start_line','end_line'))
            and a['source_id']==b['source_id'] and a['revision']==b['revision']
            and type(a['start_line']) is int and type(a['end_line']) is int
            and type(b['start_line']) is int and type(b['end_line']) is int
            and b['start_line'] <= a['start_line'] <= a['end_line'] <= b['end_line'])


def decoded_input(body):
    require(isinstance(body,dict), 'Request body must be object','UNSUPPORTED')
    messages = body.get('messages')
    require(isinstance(messages,list) and len(messages)>1 and isinstance(messages[1],dict)
            and messages[1].get('role')=='user','Unsupported user message envelope','UNSUPPORTED')
    content = messages[1].get('content')
    if isinstance(content,str):
        return strict_json(content), '/messages/1/content'
    require(isinstance(content,list),'Unsupported content shape','UNSUPPORTED')
    texts = [(i,x) for i,x in enumerate(content) if isinstance(x,dict) and x.get('type')=='text']
    require(len(texts)==1 and isinstance(texts[0][1].get('text'),str),
            'Nonunique/missing request text','UNSUPPORTED')
    i,item = texts[0]
    return strict_json(item['text']), '/messages/1/content/'+str(i)+'/text'


def valid_name(value):
    return isinstance(value,str) and bool(value)


def canonical_call_identities(calls,run_id):
    """Raw occurrences, dispatch UUIDs and operation nodes are different identities."""
    by_uuid=defaultdict(set);by_node=defaultdict(set);rows=[]
    for call in calls:
        if not isinstance(call,dict): rows.append(None);continue
        causal=call.get('causal_trace')
        causal=causal if isinstance(causal,dict) else {}
        uuid,node=call.get('call_uuid'),causal.get('node_id')
        signature=encoded({'causal_trace':causal,'stage':call.get('stage'),
                           'role':call.get('role'),'actual_api_sent':call.get('actual_api_sent')})
        if valid_name(uuid): by_uuid[uuid].add(signature)
        if valid_name(node) and valid_name(uuid): by_node[node].add(uuid)
        rows.append((uuid,node,causal))
    result=[]
    for entry in rows:
        if entry is None: result.append({'status':'UNSUPPORTED','operation_key':None,'dispatch_key':None});continue
        uuid,node,causal=entry
        parents=causal.get('parent_ids',[])
        malformed=not isinstance(parents,list) or not all(valid_name(x) for x in parents)
        conflict=(valid_name(uuid) and len(by_uuid[uuid])!=1) or (valid_name(node) and len(by_node[node])>1)
        status='CONFLICT' if conflict else 'UNSUPPORTED' if malformed else COMPLETE if valid_name(uuid) and valid_name(node) else 'MISSING'
        result.append({'status':status,'recorded_dispatch_uuid':uuid if valid_name(uuid) else None,
            'recorded_operation_node':node if valid_name(node) else None,
            'operation_key':{'run_id':run_id,'node_id':node} if status==COMPLETE else None,
            'dispatch_key':{'run_id':run_id,'call_uuid':uuid} if status==COMPLETE else None})
    return result


def canonical_ingress_identities(events,run_id):
    by_id=defaultdict(set)
    for event in events:
        if isinstance(event,dict) and event.get('kind')=='AGENT_INGRESS' and valid_name(event.get('event_id')):
            by_id[event['event_id']].add(encoded(event))
    result={}
    for i,event in enumerate(events):
        if not isinstance(event,dict) or event.get('kind')!='AGENT_INGRESS': continue
        name=event.get('event_id')
        status='MISSING' if not valid_name(name) else 'CONFLICT' if len(by_id[name])!=1 else COMPLETE
        result[i]={'status':status,'ingress_key':{'run_id':run_id,'event_id':name} if status==COMPLETE else None}
    return result


def extract_run(trace, trace_ref, row, reader, accounting_ref):
    try:
        return _extract_run(trace,trace_ref,row,reader,accounting_ref)
    except (EvidenceError,OSError,ValueError,TypeError,KeyError,AttributeError,RecursionError) as error:
        interrupt_row(row,getattr(error,'status','UNSUPPORTED'),str(error),reference(trace_ref))
        return row


def _extract_run(trace, trace_ref, row, reader, accounting_ref):
    """One in-memory bounded trace; each request released before the next."""
    require(isinstance(trace,dict),'Trace object required','UNSUPPORTED')
    events,calls = trace.get('events'),trace.get('calls')
    require(isinstance(events,list) and isinstance(calls,list),'Trace arrays missing','UNSUPPORTED')
    trace_complete = type(trace.get('end_ns')) is int and type(trace.get('returncode')) is int
    status = COMPLETE if trace_complete else 'MISSING'
    row['extraction_status']=status
    row['trace_ref']=reference(trace_ref)
    row['trace_terminal_facts']=project({k:trace[k] for k in ('end_ns','returncode','host_failures',
        'technical_failures','technical_abort_reason','unprovided_mandatory_events') if k in trace},trace_ref)
    row['trace_complete_for_absence_claims']=False  # Extraction never certifies mechanism inactivity.
    if not trace_complete:
        issue(row,'MISSING','Trace terminal closure absent; zero matches are unknown',reference(trace_ref))
    data_items = lambda name: row['collections'][name]['items']
    nodes=defaultdict(list)
    reads=[]
    generations=[]
    occurrences=[]
    for i,event in enumerate(events):
        if not isinstance(event,dict) or not isinstance(event.get('data'),dict):
            issue(row,'UNSUPPORTED','Invalid event/data shape',reference(trace_ref,'/events/'+str(i)))
            for collection in row['collections'].values(): collection['status']='UNSUPPORTED'
            continue
        p='/events/'+str(i); d=event['data']; kind=event.get('kind')
        if kind in OPERATIONS and isinstance(d.get('node_id'),str):
            nodes[d['node_id']].append((i,event))
        if kind=='SOURCE_RANGE_READ':
            item=event_info(event,trace_ref,p);item['provenance_owner']=OWNERS.get(d.get('consumer'))
            data_items('source_reads').append(item)
            reads.append((i,d.get('passage'),item))
        if kind=='EVIDENCE_ORIGINAL_REUSED':
            data_items('source_reuses').append(event_info(event,trace_ref,p))
        if kind in ('EVIDENCE_RELATIONS_COMMITTED','HANDOFF_RECORD_COMMITTED'):
            owner='DP1_EVIDENCE' if kind=='EVIDENCE_RELATIONS_COMMITTED' else 'DP4_HANDOFF_PREFIX'
            records=d.get('records')
            if not isinstance(records,list):
                issue(row,'UNSUPPORTED','Commit records missing',reference(trace_ref,p),'record_generations')
            else:
                for j,record in enumerate(records):
                    rp=p+'/data/records/'+str(j)
                    if not isinstance(record,dict):
                        issue(row,'UNSUPPORTED','Nonobject committed record',reference(trace_ref,rp),'record_generations');continue
                    g={'run_id':row['run_id'],'commit_event_ref':reference(trace_ref,p),
                       'record_pointer':rp,'owner':owner,'activity_seq':d.get('activity_seq'),
                       'producer_node':record.get('producer_node'),'record_id':record.get('record_id'),
                       'source_spans':project(record.get('source_spans'),trace_ref,rp+'/source_spans'),
                       'supporting_passages':project(record.get('supporting_passages'),trace_ref,rp+'/supporting_passages'),
                       'payload_sha256':digest(encoded(record)),
                       'record':project(record,trace_ref,rp)}
                    data_items('record_generations').append(g);generations.append((i,record,g))
        if kind in PREPARATION: data_items('preparation_and_failures').append(event_info(event,trace_ref,p))
        if kind in HANDOFF: data_items('handoffs').append(event_info(event,trace_ref,p))
        if kind in RECEIVER: data_items('receiver_support').append(event_info(event,trace_ref,p))
        if kind in ASSOCIATION:
            data_items('handoffs').append(event_info(event,trace_ref,p))
        if kind=='LOGICAL_BUFFER' and d.get('name') in BUFFERS:
            data_items('logical_state_observations').append(event_info(event,trace_ref,p))
        # The branch plan's record_ids never become capsule/ingress occurrences.
        if kind=='HANDOFF_CONTRACT_BUILT':
            occurrences.append((d.get('capsule'),p+'/data/capsule','SELECTED_IN_BUILT_CAPSULE',
                                reference(trace_ref,p),None,None,trace_ref,i))
        if kind=='AGENT_INGRESS':
            request=d.get('request')
            if not isinstance(request,dict) or not isinstance(request.get('contract'),dict):
                issue(row,'UNSUPPORTED','Actual ingress contract missing',reference(trace_ref,p),'record_occurrences')
            else:
                occurrences.append((request['contract'].get('capsule'),p+'/data/request/contract/capsule',
                    'PRESENT_AT_ACTUAL_INGRESS',reference(trace_ref,p),request,p+'/data/request',trace_ref,i))
    for name in ('source_reads','preparation_and_failures','handoffs','logical_state_observations'):
        finish_collection(row,name,status)
    call_identities=canonical_call_identities(calls,row['run_id'])
    ingress_identities=canonical_ingress_identities(events,row['run_id'])
    # Acquisition nodes are recorded provenance, not automatically local-read nodes.
    def acquisition_links(passage,consumer_index):
        if not isinstance(passage,dict): return {'status':'UNSUPPORTED','candidates':[]}
        node=passage.get('acquisition_node')
        matches=[item for n,original,item in reads if n<consumer_index and node is not None
                 and isinstance(original,dict) and original.get('acquisition_node')==node
                 and span_matches(passage,original)]
        return {'recorded_acquisition_node':node,
                'status':COMPLETE if len(matches)==1 else ('CONFLICT' if len(matches)>1 else 'MISSING'),
                'candidates':[x['source_ref'] for x in matches],
                'provenance_owner':matches[0]['provenance_owner'] if len(matches)==1 else None}
    for item in data_items('source_reuses'):
        # Retrieve only through our canonical event pointer, never timestamps.
        event=events[int(item['source_ref']['pointer'].split('/')[2])]
        passage=event['data'].get('passage')
        item['acquisition_link']=acquisition_links(passage,int(item['source_ref']['pointer'].split('/')[2]))
        producers=[]
        if isinstance(passage,dict) and passage.get('producer_node') is not None:
            for _,record,g in generations:
                if g['producer_node']==passage['producer_node'] and any(span_matches(passage,p)
                        for p in (record.get('supporting_passages') or []) if isinstance(p,dict)):
                    producers.append({'commit_event_ref':g['commit_event_ref'],
                        'record_pointer':g['record_pointer'],'owner':g['owner']})
        item['supporting_producer_link']={'status':COMPLETE if len(producers)==1 else
            ('CONFLICT' if len(producers)>1 else 'MISSING'),'candidates':producers}
        owners={x['owner'] for x in producers}
        acquisition_owner=item['acquisition_link'].get('provenance_owner')
        if acquisition_owner: owners.add(acquisition_owner)
        item['provenance_owner']=next(iter(owners)) if len(owners)==1 else None
        item['provenance_owner_status']=COMPLETE if len(owners)==1 else ('CONFLICT' if owners else 'MISSING')
    finish_collection(row,'source_reuses',status)
    for consumer_index,record,g in generations:
        passages=record.get('supporting_passages',[])
        if not isinstance(passages,list):
            issue(row,'UNSUPPORTED','Committed supporting_passages must be an array',g['commit_event_ref'],'record_generations')
            passages=[]
        g['supporting_acquisition_links']=[acquisition_links(p,consumer_index) for p in passages if isinstance(p,dict)]
        g['producer_node_event_refs']=[reference(trace_ref,'/events/'+str(n))
            for n,_ in nodes.get(g['producer_node'],[])]
    finish_collection(row,'record_generations',status)
    selected_names=set()
    request_refs={}
    for i,call in enumerate(calls):
        cp='/calls/'+str(i)
        if not isinstance(call,dict):
            issue(row,'UNSUPPORTED','Nonobject call',reference(trace_ref,cp),'call_links');continue
        causal=call.get('causal_trace') if isinstance(call.get('causal_trace'),dict) else {}
        stage=call.get('stage',causal.get('stage'))
        parents=causal.get('parent_ids',[])
        if not isinstance(parents,list):
            issue(row,'UNSUPPORTED','Call parent_ids must be an array',reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences'].update(status='UNSUPPORTED',coverage_issue=True)
            parents=[]
        node=causal.get('node_id');node=node if valid_name(node) else None
        link={k:project(v,trace_ref,child(cp,k),k) for k,v in call.items()
              if k not in ('request_file','response_file','raw_response','response','request')}
        link.update(source_ref=reference(trace_ref,cp),
                    request_file_ref={'recorded_path':call.get('request_file'),'content_inspected':False},
                    response_file_ref={'recorded_path':call.get('response_file'),'content_inspected':False},
                    accepted_accounting_link={'calls_ref':accounting_ref,'call_uuid':call.get('call_uuid'),
                        'linkage':'REFERENCE_ONLY_NOT_INDEPENDENT_ACCOUNTING_VERIFICATION'},
                    consumer_identity=call_identities[i],
                    explicit_node_events=[reference(trace_ref,'/events/'+str(n)) for n,_ in nodes.get(node,[])],
                    explicit_parent_node_events={parent:[reference(trace_ref,'/events/'+str(n))
                        for n,_ in nodes.get(parent,[])] for parent in parents if isinstance(parent,str)})
        data_items('call_links').append(link)
        if not isinstance(stage,str):
            issue(row,'UNSUPPORTED','Call stage must be a string',reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences'].update(status='UNSUPPORTED',coverage_issue=True)
            continue
        if stage.startswith('receiver_') and call.get('actual_api_sent') is True and call_identities[i]['status']!=COMPLETE:
            issue(row,call_identities[i]['status'],'Receiver consumer identity is unresolved',reference(trace_ref,cp),'record_occurrences')
        if stage not in STAGES: continue
        name=call.get('request_file')
        if not isinstance(name,str):
            issue(row,'MISSING','Selected-stage request file not recorded',reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences'].update(status='MISSING',coverage_issue=True);continue
        if name not in selected_names and len(selected_names)>=CAPS['selected_requests_per_run']:
            issue(row,'LIMIT_EXCEEDED','Selected-request count cap exceeded',reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences'].update(status='LIMIT_EXCEEDED',coverage_issue=True);continue
        selected_names.add(name)
        try:
            path=safe_path(name,reader.root)
            require(path.parent==reader.root/'raw/model' and path.name==str(call.get('call_uuid'))+'-request.json',
                    'Request path/UUID association mismatch')
            # Bridge trace records paths, not request hashes. Bind bytes on first bounded read.
            # Reader's dedicated capture is used rather than inventing an upstream hash.
            body,rr=capture_request(reader,path)
            link['request_file_ref']=dict(reference(rr),content_inspected=True,
                binding_kind='POSTANALYSIS_CAPTURE_NOT_UPSTREAM_HASH_CERTIFICATION')
            link['input_decode_status']='UNSUPPORTED'
            previous=request_refs.get(name)
            require(previous is None or previous==rr,'Conflicting repeated request file')
            request_refs[name]=rr
            value,outer=decoded_input(body)
            require(isinstance(value,dict),'Decoded input must be object','UNSUPPORTED')
            chosen={k:decoded_project(v,rr,outer,child('',k)) for k,v in value.items() if k in INPUT_FIELDS}
            # Decoded-input pointers name the JSON document inside the user text.
            link['input_decode_status']=COMPLETE
            link['input_exposure']={'outer_text_ref':reference(rr,outer),
                'decoded_document_sha256':digest(encoded(value)), 'decoded_pointer_namespace':'DECODED_USER_TEXT_JSON',
                'decoded_input_fields':chosen,'other_input_fields_not_projected':sorted(set(value)-INPUT_FIELDS)}
            for field in ('published_relations','work_records'):
                if isinstance(value.get(field),list):
                    occurrences.append(({'records':value[field]},'/'+field,'AVAILABLE_IN_INPUT',
                        reference(trace_ref,cp),value.get('mandatory'),'/mandatory',rr,None))
            contract=value.get('contract')
            if isinstance(contract,dict):
                occurrences.append((contract.get('capsule'),'/contract/capsule',
                    'SUPPLIED_TO_RECEIVER' if stage.startswith('receiver_') and call.get('actual_api_sent') is True else 'AVAILABLE_IN_INPUT',
                    reference(trace_ref,cp),contract.get('mandatory'),'/contract/mandatory',rr,None))
            del body,value
        except (EvidenceError,OSError,ValueError,RecursionError) as error:
            issue(row,getattr(error,'status','UNSUPPORTED'),str(error),reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences'].update(status=getattr(error,'status','UNSUPPORTED'),coverage_issue=True)
    finish_collection(row,'call_links',status)
    ingresses=[]
    for i,event in enumerate(events):
        if isinstance(event,dict) and event.get('kind')=='AGENT_INGRESS':
            d=event.get('data',{});request=d.get('request') if isinstance(d,dict) else None
            if isinstance(request,dict): ingresses.append((i,request))
    for item in data_items('receiver_support'):
        p=item['source_ref']['pointer'];event=events[int(p.split('/')[2])];d=event['data']
        receipt=d.get('receipt')
        if event.get('kind')=='RECEIVER_RESULT_AVAILABLE' and isinstance(receipt,dict):
            actual=receipt.get('actual_result');contract=actual.get('contract') if isinstance(actual,dict) else None
            matches=[n for n,request in ingresses
                if n<int(p.split('/')[2]) and receipt.get('stimulus_id') is not None
                and request.get('stimulus_id')==receipt['stimulus_id']
                and contract is not None and request.get('contract')==contract]
            unique={encoded(ingress_identities[n]['ingress_key']) for n in matches
                    if ingress_identities[n]['status']==COMPLETE}
            unresolved=any(ingress_identities[n]['status']!=COMPLETE for n in matches)
            item['actual_contract_ingress_join']={'status':COMPLETE if len(unique)==1 and not unresolved else
                ('CONFLICT' if matches else 'MISSING'),
                'candidates':[reference(trace_ref,'/events/'+str(n)) for n in matches],
                'established_ingress_keys':[strict_json(k) for k in sorted(unique)],
                'not_proof_of_record_prose_use':True}
    finish_collection(row,'receiver_support',status)
    for capsule,p,phase,consumer,association,association_pointer,ref,event_index in occurrences:
        if capsule is None: continue
        if not isinstance(capsule,dict) or not isinstance(capsule.get('records',[]),list):
            issue(row,'UNSUPPORTED','Capsule/records shape unsupported',consumer,'record_occurrences');continue
        for j,record in enumerate(capsule.get('records',[])):
            rp=child(p,j) if p in ('/published_relations','/work_records') else child(child(p,'records'),j)
            candidates=[]
            if isinstance(record,dict) and isinstance(record.get('producer_node'),str):
                candidates=[g for n,original,g in generations
                    if original==record]
            status=COMPLETE if len(candidates)==1 else ('CONFLICT' if len(candidates)>1 else 'MISSING')
            occurrence={'phase':phase,'source_ref':reference(ref,rp),'consumer_ref':consumer,
                'consumer_operation_key':None,'consumer_identity_status':'MISSING',
                'record':project(record,ref,rp),
                'association':project(association,ref,association_pointer or ''),
                'association_pointer':association_pointer,'generation_join_status':status,
                'generation_ref':candidates[0]['commit_event_ref']|{'record_pointer':candidates[0]['record_pointer']} if len(candidates)==1 else None,
                'generation_candidates':[g['commit_event_ref']|{'record_pointer':g['record_pointer']} for g in candidates],
                'interpretation':'OBSERVED_EXPOSURE_NOT_PROOF_OF_SEMANTIC_USE_OR_SAVINGS'}
            if ref!=trace_ref:
                occurrence['decoded_input_pointer']=rp
                occurrence['source_ref']=reference(ref,'/messages/1/content')
                # Exact outer pointer is available in call input_exposure; never guess array shape.
                linked=next((x for x in data_items('call_links') if x['source_ref']==consumer),None)
                occurrence['source_ref']=linked['input_exposure']['outer_text_ref'] if linked else None
                if linked:
                    outer=linked['input_exposure']['outer_text_ref']['pointer']
                    occurrence['record']=decoded_project(record,ref,outer,rp)
                    occurrence['association']=decoded_project(association,ref,outer,association_pointer or '')
                    canonical=linked['consumer_identity']
                    occurrence['consumer_identity_status']=canonical['status']
                    occurrence['consumer_operation_key']=canonical['operation_key']
                    occurrence['dispatch_key']=canonical['dispatch_key']
                    occurrence['recorded_dispatch_uuid']=canonical['recorded_dispatch_uuid']
                    occurrence['recorded_operation_node']=canonical['recorded_operation_node']
                    occurrence['actual_dispatch']=linked.get('actual_api_sent')
            elif phase=='PRESENT_AT_ACTUAL_INGRESS':
                canonical=ingress_identities[event_index]
                occurrence['consumer_identity_status']=canonical['status']
                occurrence['consumer_operation_key']=canonical['ingress_key']
                if canonical['status']!=COMPLETE:
                    issue(row,canonical['status'],'Actual ingress identity is unresolved',consumer,'record_occurrences')
            data_items('record_occurrences').append(occurrence)
            if status!=COMPLETE:
                issue(row,status,'Record generation is not uniquely established',consumer,'record_occurrences')
    # Group actual operations, so duplicate references to one call/ingress are not consumers.
    groups=defaultdict(list)
    for item in data_items('record_occurrences'):
        if item['generation_ref'] and item['phase'] in ('PRESENT_AT_ACTUAL_INGRESS','SUPPLIED_TO_RECEIVER'):
            groups[json.dumps(item['generation_ref'],sort_keys=True)].append(item)
    exposures=[]
    for g,observations in groups.items():
        ingress=[x for x in observations if x['phase']=='PRESENT_AT_ACTUAL_INGRESS']
        receiver=[x for x in observations if x['phase']=='SUPPLIED_TO_RECEIVER']
        established=lambda values:{encoded(x['consumer_operation_key']) for x in values
            if x['consumer_identity_status']==COMPLETE}
        recorded_uuids={x['recorded_dispatch_uuid'] for x in receiver if valid_name(x.get('recorded_dispatch_uuid'))}
        exposures.append({'generation_ref':json.loads(g),'distinct_actual_ingresses':len(established(ingress)),
            'distinct_receiver_operations':len(established(receiver)),
            'raw_ingress_record_occurrences':len(ingress),'raw_receiver_record_occurrences':len(receiver),
            'recorded_distinct_dispatch_uuids':sorted(recorded_uuids),
            'established_distinct_dispatch_uuids':sorted({x['recorded_dispatch_uuid'] for x in receiver
                if x['consumer_identity_status']==COMPLETE}),
            'unresolved_consumer_occurrences':[{'consumer_ref':x['consumer_ref'],'status':x['consumer_identity_status']}
                for x in observations if x['consumer_identity_status']!=COMPLETE],
            'count_scope':'ONLY_ESTABLISHED_CANONICAL_IDENTITIES_WITH_RAW_UNRESOLVED_COVERAGE',
            'not_semantic_use_or_causal_savings':True})
    row['observed_generation_exposures']=exposures
    finish_collection(row,'record_occurrences',COMPLETE if trace_complete else 'MISSING')
    finish_collection(row,'join_issues',COMPLETE if trace_complete else 'MISSING')
    return row


def capture_request(reader,path):
    data,ref=reader.capture(path,CAPS['request_bytes'])
    return strict_json(data),ref


def load_boundary(args,reader,context):
    supplied,mref=reader.pinned(args.input_manifest,args.input_sha256,CAPS['metadata_bytes'])
    require(supplied.get('version')=='JOINT441_QUANTITATIVE_INPUT_v1','Accepted input ABI required')
    root=reader.root
    require(safe_path(supplied.get('formal_root')).resolve()==root,'Explicit formal-root/manifest mismatch')
    refs=supplied['artifacts']
    require(refs['freeze']['path']==str(root/'freeze.json') and refs['freeze']['sha256']==FREEZE_SHA,'Wrong frozen campaign')
    require(refs['approval']['path']==str(root/'approval/APPROVAL-BINDING.json')
            and refs['approval']['sha256']==APPROVAL_SHA,'Wrong campaign approval')
    freeze=reader.read(refs['freeze'],CAPS['metadata_bytes']);approval=reader.read(refs['approval'],CAPS['metadata_bytes'])
    require(freeze.get('version')=='COMMON9_PLUS_SUPPLEMENT6_JOINT441_v1' and
            approval.get('formal_freeze_sha256')==FREEZE_SHA,'Freeze/approval association mismatch')
    q=reader.read(refs['quiescence'],CAPS['metadata_bytes'])
    require(q.get('version')=='JOINT441_QUIESCENT_SNAPSHOT_v1' and q.get('controller_state') in ('STOPPED','COMPLETED')
            and q.get('no_active_episode_processes') is True and q.get('evidence_refs'),
            'Independent quiescence is required before traces may be read')
    for ref in q['evidence_refs']:
        reader.read(ref,CAPS['metadata_bytes'],raw=True)
    keys=validate_plan(freeze['jobs'])
    context.update(keys=keys,freeze=freeze)
    contract_refs=[]
    for suffix in ('19-common9-evaluation-contract.md','20-common-supplement-evaluation-contract.md'):
        matches=[s for s in freeze['sources'] if s.get('relativePath','').endswith('/'+suffix)]
        require(len(matches)==1,'Frozen contract reference missing/conflicting')
        s=matches[0];p=safe_path(s['frozen_path'],root)
        require(p==root/'frozen/source'/s['relativePath'],'Frozen contract path mismatch')
        ref={'path':str(p),'sha256':s['sha256'],'bytes':p.stat().st_size}
        reader.read(ref,CAPS['metadata_bytes'],raw=True);contract_refs.append(reference(ref))
    safe_path(args.accepted_report_manifest,root)
    report,rref=reader.pinned(args.accepted_report_manifest,args.accepted_report_sha256,CAPS['metadata_bytes'])
    require(report.get('version')=='JOINT441_QUANTITATIVE_MANIFEST_v1' and report.get('report_version')=='JOINT441_QUANTITATIVE_REPORT_v1',
            'Accepted report manifest ABI required')
    require(report.get('input_manifest')==mref,'Accepted report/input manifest association differs')
    outputs=report.get('outputs')
    require(isinstance(outputs,dict) and 'calls.jsonl' in outputs,'Unique calls.jsonl role required')
    # Accepted adapter manifest keys are relative filenames; output members contain
    # hashes/sizes only. Resolve the exact unique role beside that manifest.
    require(all(isinstance(k,str) and not Path(k).is_absolute() and '..' not in Path(k).parts
                for k in outputs),'Unsafe accepted output member')
    calls=dict(outputs['calls.jsonl'],path=str(Path(rref['path']).parent/'calls.jsonl'))
    p=validate_ref(calls,root)
    require(sum(1 for name in outputs if Path(rref['path']).parent/name==p)==1,'Conflicting calls output roles')
    source_refs=report.get('sources')
    require(isinstance(source_refs,list) and sum(1 for s in source_refs if isinstance(s,dict)
        and s.get('sha256')==ADAPTER_SHA and Path(s.get('path','')).name=='report_adapter.py')==1,
        'Accepted adapter source binding required')
    index=reader.read(refs['result_index'],CAPS['metadata_bytes'],jsonl=True)
    require(len(index)<=882,'Result index exceeds plan/conflict bound')
    return keys,freeze,index,{'input_manifest':reference(mref),'accepted_report_manifest':reference(rref),
        'accepted_calls_ref':reference(calls),'accepted_accounting_linkage':'REFERENCE_ONLY_NOT_INDEPENDENT_ACCOUNTING_VERIFICATION',
        'freeze':reference(refs['freeze']),'approval':reference(refs['approval']),
        'quiescence':reference(refs['quiescence']),'contracts':contract_refs}


CAUGHT=(EvidenceError,OSError,ValueError,KeyError,TypeError,AttributeError,RecursionError)


def partition_index(index,keys):
    by_key=defaultdict(list);conflicts=[];planned=set(keys)
    for number,item in enumerate(index):
        try:
            k=identity(item.get('identity'))
            require(k in planned,'Unplanned index identity')
            by_key[k].append(item)
        except CAUGHT as error:
            conflicts.append({'index_pointer':'/'+str(number),'status':'CONFLICT','detail':str(error)})
    return by_key,conflicts


def run_row(k,matches,reader,freeze,boundary,counts):
    row=empty_row(k)
    try:
        if len(matches)!=1:
            status='CONFLICT' if matches else 'MISSING'
            interrupt_row(row,status,'Duplicate/missing planned trace identity')
            row['index_candidates']=matches
            return row
        item=matches[0];row['run_id']=item.get('run_id')
        row['accepted_endpoint_and_result_refs']={name:item.get(name) for name in ('result_ref','association_ref')}
        ref=item.get('trace_ref')
        require(isinstance(row['run_id'],str) and re.fullmatch('[A-Za-z0-9_.-]+',row['run_id'])
                and row['run_id'] not in ('.','..'),'Invalid run identity')
        require(isinstance(ref,dict),'Planned trace reference missing','MISSING')
        require(ref.get('path')==str(reader.root/'raw/runs'/row['run_id']/'trace.json'),'Trace path/run association mismatch')
        counts['attempted']+=1
        trace=reader.read(ref,CAPS['trace_bytes'])
        require(isinstance(trace,dict),'Trace must be an object','UNSUPPORTED')
        require(type(trace.get('rep')) is int and all(trace.get(f)==v for f,v in
            [('configuration',k[1]),('episode',k[2]),('rep',k[3]),('run_id',row['run_id']),('phase','FORMAL')]),
            'Trace identity/phase mismatch')
        require(trace.get('binary_sha256')==freeze['binary']['sha256'],'Trace binary generation mismatch')
        extract_run(trace,ref,row,reader,boundary['accepted_calls_ref'])
    except CAUGHT as error:
        interrupt_row(row,getattr(error,'status','UNSUPPORTED'),str(error))
    return row


def serialization_safe(value):
    if isinstance(value,dict): return {str(k):serialization_safe(v) for k,v in value.items()}
    if isinstance(value,(list,tuple)): return [serialization_safe(v) for v in value]
    if value is None or type(value) in (bool,int): return value
    if type(value) is float and math.isfinite(value): return value
    if isinstance(value,str):
        try: value.encode();return value
        except UnicodeError:
            return {'invalid_unicode_sha256':digest(value.encode(errors='surrogatepass'))}
    return {'unsupported_projection_type':type(value).__name__}


def serialize_row(row,k):
    try:
        data=encoded(row)
    except (ValueError,TypeError,UnicodeError,RecursionError) as error:
        try:
            row=serialization_safe(row)
            interrupt_row(row,'UNSUPPORTED','Row serialization failed; prior facts retained with explicit unsupported-value markers',all_collections=True)
            data=encoded(row)
        except (ValueError,TypeError,UnicodeError,RecursionError,KeyError):
            row=empty_row(k,'UNSUPPORTED')
            interrupt_row(row,'UNSUPPORTED','Row serialization failed; projection could not be retained')
            data=encoded(row)
    if len(data)>CAPS['projected_bytes_per_run']:
        row=empty_row(k,'LIMIT_EXCEEDED',row.get('run_id'))
        interrupt_row(row,'LIMIT_EXCEEDED','Projected row cap exceeded; full trace/request references remain in read receipt')
        data=encoded(row)
    return data,row['extraction_status']


def write_rows(out,keys,reader,receipt,*,index=(),freeze=None,boundary=None,rejection=None):
    by_key,conflicts=partition_index(index,keys)
    receipt['unplanned_index_conflicts']=conflicts
    counts=Counter();p=out/'mechanism-evidence.jsonl';h=hashlib.sha256();written=0
    with p.open('xb') as handle:
        for k in keys:
            if rejection is not None:
                row=empty_row(k,rejection['status'])
                interrupt_row(row,rejection['status'],'Boundary rejected; planned identity retained without extraction')
            elif reader.aggregate_exhausted:
                row=empty_row(k,'LIMIT_EXCEEDED')
                interrupt_row(row,'LIMIT_EXCEEDED','Aggregate input cap reached; planned identity retained')
            else:
                row=run_row(k,by_key[k],reader,freeze,boundary,counts)
            data,status=serialize_row(row,k)
            handle.write(data);h.update(data);written+=len(data);counts[status]+=1
            receipt['outputs'][p.name]={'path':str(p),'sha256':h.hexdigest(),'bytes':written,
                'hash_scope':'SUCCESSFULLY_WRITTEN_ROWS'}
    receipt['counts']={'planned':len(keys),'attempted':counts['attempted'],'complete':counts[COMPLETE],
        'partial':len(keys)-counts[COMPLETE],
        'unavailable':counts['MISSING']+counts['UNSUPPORTED']+counts['LIMIT_EXCEEDED'],
        'conflict':counts['CONFLICT'],
        'by_extraction_status':{s:counts[s] for s in (COMPLETE,'MISSING','UNSUPPORTED','LIMIT_EXCEEDED','CONFLICT')}}
    return counts[COMPLETE]==len(keys) and not conflicts


def execute(args):
    start=time.monotonic();root=safe_path(args.formal_root).resolve()
    out=safe_path(args.output,root)
    require(out.is_relative_to(root/'postanalysis'),'Output must be inside explicit formal postanalysis root')
    require(not out.exists(),'Immutable output already exists')
    out.mkdir(parents=True,exist_ok=False)
    reader=Reader(root);context={}
    receipt={'version':VERSION,'started_at_utc':datetime.now(timezone.utc).isoformat(),
        'formal_root':str(root),'caps':dict(CAPS),'plan_sha256':PLAN_SHA,
        'mode':'POSTANALYSIS_DESCRIPTIVE_ONLY','no_new_metric_eligibility_or_oracle':True,
        'source':None,'inputs':[],'outputs':{}}
    try:
        _,receipt['source']=reader.capture(Path(__file__).resolve(),CAPS['metadata_bytes'],source_self=True)
        keys,freeze,index,boundary=load_boundary(args,reader,context)
        receipt['boundary']=boundary
        complete=write_rows(out,keys,reader,receipt,index=index,freeze=freeze,boundary=boundary)
        receipt['status']='COMPLETE' if complete else 'PARTIAL'
    except CAUGHT as error:
        receipt['status']='REJECTED_INPUT'
        receipt['error']={'status':getattr(error,'status','UNSUPPORTED'),'detail':str(error)}
        if context.get('keys') and not (out/'mechanism-evidence.jsonl').exists():
            write_rows(out,context['keys'],reader,receipt,rejection=receipt['error'])
    finally:
        receipt.update(inputs=reader.inputs,input_read_attempts=reader.attempts,input_bytes_read=reader.total,
            elapsed_seconds=time.monotonic()-start,finished_at_utc=datetime.now(timezone.utc).isoformat())
        data=encoded(receipt)
        with (out/'execution-receipt.json').open('xb') as handle: handle.write(data)
        with (out/'execution-receipt.sha256').open('xb') as handle:
            handle.write((digest(data)+'  execution-receipt.json\n').encode())
    return receipt


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--formal-root',type=Path,required=True)
    parser.add_argument('--input-manifest',type=Path,required=True)
    parser.add_argument('--input-sha256',required=True)
    parser.add_argument('--accepted-report-manifest',type=Path,required=True)
    parser.add_argument('--accepted-report-sha256',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    receipt=execute(args)
    print(json.dumps({'status':receipt['status'],'output':str(args.output)}))
    return 0 if receipt['status']!='REJECTED_INPUT' else 2


if __name__=='__main__':
    raise SystemExit(main())
