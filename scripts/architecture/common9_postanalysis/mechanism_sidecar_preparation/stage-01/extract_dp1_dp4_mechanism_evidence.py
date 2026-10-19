"""Read-only descriptive sidecar; explicit quiescent inputs, no runtime imports.

Preparation is not execution authorization. Root must separately confirm controller
quiescence and issue an exact GO before this CLI is used on real campaign inputs.
"""
import argparse
from collections import Counter, defaultdict
from datetime import datetime, timezone
import hashlib
import json
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
    'mandatory','denials','contract','event','public_state','actual_public_state','observed_activity'))
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
        self.root, self.total, self.inputs = root, 0, []
        self.aggregate_exhausted = False

    def read(self, ref, limit, jsonl=False, raw=False):
        path = validate_ref(ref, self.root)
        try:
            before = path.stat()
        except FileNotFoundError as error:
            raise EvidenceError('MISSING','Named input missing: '+str(path)) from error
        require(path.is_file(), 'Input is not a regular file')
        require(before.st_size <= limit, 'Per-file byte cap exceeded', 'LIMIT_EXCEEDED')
        if self.aggregate_exhausted or self.total + before.st_size > CAPS['aggregate_input_bytes']:
            self.aggregate_exhausted = True
            raise EvidenceError('LIMIT_EXCEEDED', 'Aggregate input cap reached')
        with path.open('rb') as handle:
            data = handle.read(limit+1)
        self.total += len(data)
        require(len(data) <= limit, 'Input grew beyond cap', 'LIMIT_EXCEEDED')
        after = path.stat()
        require((before.st_size,before.st_mtime_ns,before.st_ino)==
                (after.st_size,after.st_mtime_ns,after.st_ino) and len(data)==before.st_size,
                'Input changed while reading')
        actual = {'path':str(path),'sha256':digest(data),'bytes':len(data)}
        self.inputs.append(actual)
        require(all(actual[k]==ref[k] for k in actual), 'Input hash/bytes mismatch')
        if raw:
            return data
        try:
            return [strict_json(line) for line in data.splitlines() if line.strip()] if jsonl else strict_json(data)
        except (ValueError,UnicodeError,RecursionError) as error:
            if isinstance(error,EvidenceError):
                raise
            raise EvidenceError('UNSUPPORTED','Invalid JSON source') from error


def identity(row):
    require(isinstance(row,dict) and type(row.get('rep')) is int, 'Invalid planned identity')
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
            'collections':{name:{'status':status,'items':[]} for name in COLLECTIONS}}


def issue(row, status, detail, ref=None, collection=None):
    row['collections']['join_issues']['items'].append({'status':status,'detail':detail,'source_ref':ref})
    row['extraction_status'] = status if row['extraction_status']==COMPLETE else row['extraction_status']
    if collection:
        row['collections'][collection]['status'] = status
    row['collections']['join_issues']['status'] = status


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


def extract_run(trace, trace_ref, row, reader, accounting_ref):
    """One in-memory bounded trace; each request released before the next."""
    require(isinstance(trace,dict),'Trace object required','UNSUPPORTED')
    events,calls = trace.get('events'),trace.get('calls')
    require(isinstance(events,list) and isinstance(calls,list),'Trace arrays missing','UNSUPPORTED')
    trace_complete = type(trace.get('end_ns')) is int and type(trace.get('returncode')) is int
    status = COMPLETE if trace_complete else 'MISSING'
    row['extraction_status']=status
    for collection in row['collections'].values():
        collection['status']=status
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
                                reference(trace_ref,p),None,trace_ref,i))
        if kind=='AGENT_INGRESS':
            request=d.get('request')
            if not isinstance(request,dict) or not isinstance(request.get('contract'),dict):
                issue(row,'UNSUPPORTED','Actual ingress contract missing',reference(trace_ref,p),'record_occurrences')
            else:
                occurrences.append((request['contract'].get('capsule'),p+'/data/request/contract/capsule',
                    'PRESENT_AT_ACTUAL_INGRESS',reference(trace_ref,p),request,trace_ref,i))
    # Acquisition nodes are recorded provenance, not automatically local-read nodes.
    def acquisition_links(passage):
        if not isinstance(passage,dict): return {'status':'UNSUPPORTED','candidates':[]}
        node=passage.get('acquisition_node')
        matches=[item for _,original,item in reads if node is not None
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
        item['acquisition_link']=acquisition_links(passage)
        producers=[]
        if isinstance(passage,dict) and passage.get('producer_node') is not None:
            for _,record,g in generations:
                if g['producer_node']==passage['producer_node'] and any(span_matches(passage,p)
                        for p in record.get('supporting_passages',[]) if isinstance(p,dict)):
                    producers.append({'commit_event_ref':g['commit_event_ref'],
                        'record_pointer':g['record_pointer'],'owner':g['owner']})
        item['supporting_producer_link']={'status':COMPLETE if len(producers)==1 else
            ('CONFLICT' if len(producers)>1 else 'MISSING'),'candidates':producers}
        owners={x['owner'] for x in producers}
        acquisition_owner=item['acquisition_link'].get('provenance_owner')
        if acquisition_owner: owners.add(acquisition_owner)
        item['provenance_owner']=next(iter(owners)) if len(owners)==1 else None
        item['provenance_owner_status']=COMPLETE if len(owners)==1 else ('CONFLICT' if owners else 'MISSING')
    for _,record,g in generations:
        g['supporting_acquisition_links']=[acquisition_links(p) for p in record.get('supporting_passages',[]) if isinstance(p,dict)]
        g['producer_node_event_refs']=[reference(trace_ref,'/events/'+str(n))
            for n,_ in nodes.get(g['producer_node'],[])]
    selected_names=set()
    request_refs={}
    for i,call in enumerate(calls):
        cp='/calls/'+str(i)
        if not isinstance(call,dict):
            issue(row,'UNSUPPORTED','Nonobject call',reference(trace_ref,cp),'call_links');continue
        causal=call.get('causal_trace') if isinstance(call.get('causal_trace'),dict) else {}
        stage=call.get('stage',causal.get('stage'))
        link={k:project(v,trace_ref,child(cp,k),k) for k,v in call.items()
              if k not in ('request_file','response_file','raw_response','response','request')}
        link.update(source_ref=reference(trace_ref,cp),
                    request_file_ref={'recorded_path':call.get('request_file'),'content_inspected':False},
                    response_file_ref={'recorded_path':call.get('response_file'),'content_inspected':False},
                    accepted_accounting_link={'calls_ref':accounting_ref,'call_uuid':call.get('call_uuid'),
                        'linkage':'REFERENCE_ONLY_NOT_INDEPENDENT_ACCOUNTING_VERIFICATION'},
                    explicit_node_events=[reference(trace_ref,'/events/'+str(n)) for n,_ in nodes.get(causal.get('node_id'),[])],
                    explicit_parent_node_events={parent:[reference(trace_ref,'/events/'+str(n))
                        for n,_ in nodes.get(parent,[])] for parent in causal.get('parent_ids',[]) if isinstance(parent,str)})
        data_items('call_links').append(link)
        if stage not in STAGES: continue
        name=call.get('request_file')
        if not isinstance(name,str):
            issue(row,'MISSING','Selected-stage request file not recorded',reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences']['status']='MISSING';continue
        if name not in selected_names and len(selected_names)>=CAPS['selected_requests_per_run']:
            issue(row,'LIMIT_EXCEEDED','Selected-request count cap exceeded',reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences']['status']='LIMIT_EXCEEDED';continue
        selected_names.add(name)
        try:
            path=safe_path(name,reader.root)
            require(path.parent==reader.root/'raw/model' and path.name==str(call.get('call_uuid'))+'-request.json',
                    'Request path/UUID association mismatch')
            # Bridge trace records paths, not request hashes. Bind bytes on first bounded read.
            # Reader's dedicated capture is used rather than inventing an upstream hash.
            body,rr=capture_request(reader,path)
            previous=request_refs.get(name)
            require(previous is None or previous==rr,'Conflicting repeated request file')
            request_refs[name]=rr
            value,outer=decoded_input(body)
            require(isinstance(value,dict),'Decoded input must be object','UNSUPPORTED')
            chosen={k:decoded_project(v,rr,outer,child('',k)) for k,v in value.items() if k in INPUT_FIELDS}
            # Decoded-input pointers name the JSON document inside the user text.
            link['request_file_ref']=dict(reference(rr),content_inspected=True,
                binding_kind='POSTANALYSIS_CAPTURE_NOT_UPSTREAM_HASH_CERTIFICATION')
            link['input_exposure']={'outer_text_ref':reference(rr,outer),
                'decoded_document_sha256':digest(encoded(value)), 'decoded_pointer_namespace':'DECODED_USER_TEXT_JSON',
                'decoded_input_fields':chosen,'other_input_fields_not_projected':sorted(set(value)-INPUT_FIELDS)}
            for field in ('published_relations','work_records'):
                if isinstance(value.get(field),list):
                    occurrences.append(({'records':value[field]},'/'+field,'AVAILABLE_IN_INPUT',
                        reference(trace_ref,cp),value.get('mandatory'),rr,None))
            contract=value.get('contract')
            if isinstance(contract,dict):
                occurrences.append((contract.get('capsule'),'/contract/capsule',
                    'SUPPLIED_TO_RECEIVER' if stage.startswith('receiver_') and call.get('actual_api_sent') is True else 'AVAILABLE_IN_INPUT',
                    reference(trace_ref,cp),contract.get('mandatory'),rr,None))
            del body,value
        except (EvidenceError,OSError,ValueError,RecursionError) as error:
            issue(row,getattr(error,'status','UNSUPPORTED'),str(error),reference(trace_ref,cp),'call_links')
            row['collections']['record_occurrences']['status']=getattr(error,'status','UNSUPPORTED')
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
            candidates=[reference(trace_ref,'/events/'+str(n)) for n,request in ingresses
                if n<int(p.split('/')[2]) and receipt.get('stimulus_id') is not None
                and request.get('stimulus_id')==receipt['stimulus_id']
                and contract is not None and request.get('contract')==contract]
            item['actual_contract_ingress_join']={'status':COMPLETE if len(candidates)==1 else
                ('CONFLICT' if candidates else 'MISSING'),'candidates':candidates,
                'not_proof_of_record_prose_use':True}
    for capsule,p,phase,consumer,association,ref,event_index in occurrences:
        if capsule is None: continue
        if not isinstance(capsule,dict) or not isinstance(capsule.get('records',[]),list):
            issue(row,'UNSUPPORTED','Capsule/records shape unsupported',consumer,'record_occurrences');continue
        for j,record in enumerate(capsule.get('records',[])):
            rp=child(p,j) if p in ('/published_relations','/work_records') else child(child(p,'records'),j)
            candidates=[]
            if isinstance(record,dict) and isinstance(record.get('producer_node'),str):
                candidates=[g for n,original,g in generations
                    if original==record and (event_index is None or n<event_index)]
            status=COMPLETE if len(candidates)==1 else ('CONFLICT' if len(candidates)>1 else 'MISSING')
            occurrence={'phase':phase,'source_ref':reference(ref,rp),'consumer_ref':consumer,
                'consumer_operation_key':consumer,'record':project(record,ref,rp),
                'association':project(association,ref,p),'generation_join_status':status,
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
                    occurrence['association']=decoded_project(association,ref,outer,p)
                    occurrence['consumer_operation_key']={'call_uuid':linked.get('call_uuid'),
                        'causal_trace':linked.get('causal_trace')}
                    occurrence['actual_dispatch']=linked.get('actual_api_sent')
            elif phase=='PRESENT_AT_ACTUAL_INGRESS':
                ingress_event=events[event_index]
                occurrence['consumer_operation_key']={'event_id':ingress_event.get('event_id'),
                    'source_ref_if_id_absent':consumer if ingress_event.get('event_id') is None else None}
            data_items('record_occurrences').append(occurrence)
            if status!=COMPLETE:
                issue(row,status,'Record generation is not uniquely established',consumer,'record_occurrences')
    # Group actual operations, so duplicate references to one call/ingress are not consumers.
    groups=defaultdict(lambda:defaultdict(set))
    for item in data_items('record_occurrences'):
        if item['generation_ref'] and item['phase'] in ('PRESENT_AT_ACTUAL_INGRESS','SUPPLIED_TO_RECEIVER'):
            groups[json.dumps(item['generation_ref'],sort_keys=True)][item['phase']].add(json.dumps(item['consumer_operation_key'],sort_keys=True))
    row['observed_generation_exposures']=[{'generation_ref':json.loads(g),
        'distinct_actual_ingresses':len(phases['PRESENT_AT_ACTUAL_INGRESS']),
        'distinct_receiver_operations':len(phases['SUPPLIED_TO_RECEIVER']),
        'not_semantic_use_or_causal_savings':True} for g,phases in groups.items()]
    return row


def capture_request(reader,path):
    before=path.stat()
    require(path.is_file(),'Request is not a regular file')
    require(before.st_size<=CAPS['request_bytes'],'Request file cap exceeded','LIMIT_EXCEEDED')
    if reader.aggregate_exhausted or reader.total+before.st_size>CAPS['aggregate_input_bytes']:
        reader.aggregate_exhausted=True
        raise EvidenceError('LIMIT_EXCEEDED','Aggregate input cap reached')
    with path.open('rb') as handle: data=handle.read(CAPS['request_bytes']+1)
    reader.total+=len(data)
    after=path.stat()
    require(len(data)<=CAPS['request_bytes'],'Request grew beyond cap','LIMIT_EXCEEDED')
    require((before.st_size,before.st_mtime_ns,before.st_ino)==(after.st_size,after.st_mtime_ns,after.st_ino)
            and len(data)==before.st_size,'Request changed while reading')
    ref={'path':str(path),'sha256':digest(data),'bytes':len(data)}
    reader.inputs.append(ref)
    return strict_json(data),ref


def pinned_file(path,sha,limit):
    path=safe_path(path)
    require(isinstance(sha,str) and re.fullmatch('[0-9a-f]{64}',sha),'Expected SHA256 required')
    before=path.stat();require(path.is_file() and before.st_size<=limit,'Metadata byte cap exceeded','LIMIT_EXCEEDED')
    with path.open('rb') as handle: data=handle.read(limit+1)
    after=path.stat()
    require(len(data)<=limit and (before.st_size,before.st_mtime_ns,before.st_ino)==
            (after.st_size,after.st_mtime_ns,after.st_ino) and len(data)==before.st_size,'Metadata changed/cap exceeded')
    require(digest(data)==sha,'Pinned metadata hash mismatch')
    return strict_json(data),{'path':str(path),'sha256':sha,'bytes':len(data)}


def load_boundary(args):
    supplied,mref=pinned_file(args.input_manifest,args.input_sha256,CAPS['metadata_bytes'])
    require(supplied.get('version')=='JOINT441_QUANTITATIVE_INPUT_v1','Accepted input ABI required')
    root=safe_path(supplied.get('formal_root')).resolve();safe_path(mref['path'],root)
    reader=Reader(root);reader.total+=mref['bytes'];reader.inputs.append(mref)
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
    contract_refs=[]
    for suffix in ('19-common9-evaluation-contract.md','20-common-supplement-evaluation-contract.md'):
        matches=[s for s in freeze['sources'] if s.get('relativePath','').endswith('/'+suffix)]
        require(len(matches)==1,'Frozen contract reference missing/conflicting')
        s=matches[0];p=safe_path(s['frozen_path'],root)
        require(p==root/'frozen/source'/s['relativePath'],'Frozen contract path mismatch')
        ref={'path':str(p),'sha256':s['sha256'],'bytes':p.stat().st_size}
        reader.read(ref,CAPS['metadata_bytes'],raw=True);contract_refs.append(reference(ref))
    safe_path(args.accepted_report_manifest,root)
    report,rref=pinned_file(args.accepted_report_manifest,args.accepted_report_sha256,CAPS['metadata_bytes'])
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
    reader.inputs.append(rref);reader.total+=rref['bytes']
    index=reader.read(refs['result_index'],CAPS['metadata_bytes'],jsonl=True)
    require(len(index)<=882,'Result index exceeds plan/conflict bound')
    return root,reader,keys,freeze,index,{'input_manifest':reference(mref),'accepted_report_manifest':reference(rref),
        'accepted_calls_ref':reference(calls),'accepted_accounting_linkage':'REFERENCE_ONLY_NOT_INDEPENDENT_ACCOUNTING_VERIFICATION',
        'freeze':reference(refs['freeze']),'approval':reference(refs['approval']),
        'quiescence':reference(refs['quiescence']),'contracts':contract_refs}


def execute(args):
    start=time.monotonic();out=safe_path(args.output)
    # Establish destination confinement from a pinned small manifest before any write.
    preliminary,_=pinned_file(args.input_manifest,args.input_sha256,CAPS['metadata_bytes'])
    root=safe_path(preliminary.get('formal_root')).resolve()
    safe_path(args.input_manifest,root);safe_path(out,root)
    require(out.is_relative_to(root/'postanalysis'),'Output must be inside formal postanalysis root')
    require(not out.exists(),'Immutable output already exists')
    out.mkdir(parents=True,exist_ok=False)
    outputs={};receipt={'version':VERSION,'started_at_utc':datetime.now(timezone.utc).isoformat(),
        'caps':dict(CAPS),'plan_sha256':PLAN_SHA,'mode':'POSTANALYSIS_DESCRIPTIVE_ONLY',
        'no_new_metric_eligibility_or_oracle':True,'source':None,'inputs':[],'outputs':outputs}
    reader=None
    try:
        source=Path(__file__).resolve();data=source.read_bytes()
        receipt['source']={'path':str(source),'sha256':digest(data),'bytes':len(data)}
        root,reader,keys,freeze,index,boundary=load_boundary(args)
        safe_path(out,root)
        require(out.is_relative_to(root/'postanalysis'),'Output must be inside formal postanalysis root')
        receipt['boundary']=boundary
        by_key=defaultdict(list);unknown=[]
        for number,item in enumerate(index):
            try: k=identity(item.get('identity'))
            except (EvidenceError,AttributeError): unknown.append({'index_pointer':'/'+str(number),'status':'CONFLICT'});continue
            if k not in set(keys): unknown.append({'identity':item['identity'],'index_pointer':'/'+str(number),'status':'CONFLICT'})
            else: by_key[k].append(item)
        receipt['unplanned_index_conflicts']=unknown
        counts=Counter();row_path=out/'mechanism-evidence.jsonl';file_sha=hashlib.sha256();file_bytes=0
        with row_path.open('xb') as handle:
            for k in keys:
                matches=by_key[k];row=empty_row(k)
                if len(matches)!=1:
                    row['extraction_status']='CONFLICT' if matches else 'MISSING'
                    issue(row,row['extraction_status'],'Duplicate/missing planned trace identity')
                    row['index_candidates']=matches
                else:
                    item=matches[0];row['run_id']=item.get('run_id');row['accepted_endpoint_and_result_refs']={
                        name:item.get(name) for name in ('result_ref','association_ref')}
                    ref=item.get('trace_ref')
                    try:
                        require(isinstance(row['run_id'],str) and re.fullmatch('[A-Za-z0-9_.-]+',row['run_id'])
                                and row['run_id'] not in ('.','..'),'Invalid run identity')
                        require(isinstance(ref,dict),'Planned trace reference missing','MISSING')
                        require(ref.get('path')==str(root/'raw/runs'/row['run_id']/'trace.json'),'Trace path/run association mismatch')
                        counts['attempted']+=1
                        trace=reader.read(ref,CAPS['trace_bytes'])
                        require(type(trace.get('rep')) is int and all(trace.get(f)==v for f,v in
                            [('configuration',k[1]),('episode',k[2]),('rep',k[3]),('run_id',row['run_id']),('phase','FORMAL')]),
                            'Trace identity/phase mismatch')
                        require(trace.get('binary_sha256')==freeze['binary']['sha256'],'Trace binary generation mismatch')
                        extract_run(trace,ref,row,reader,boundary['accepted_calls_ref']);del trace
                    except (EvidenceError,OSError,ValueError,TypeError,AttributeError,RecursionError) as error:
                        status=getattr(error,'status','UNSUPPORTED')
                        issue(row,status,str(error),reference(ref) if isinstance(ref,dict) and all(x in ref for x in ('path','sha256','bytes')) else None)
                        row['extraction_status']=status
                data=encoded(row)
                if len(data)>CAPS['projected_bytes_per_run']:
                    row=empty_row(k,'LIMIT_EXCEEDED',row.get('run_id'))
                    issue(row,'LIMIT_EXCEEDED','Projected row cap exceeded; no partial payload silently truncated')
                    data=encoded(row)
                handle.write(data);file_sha.update(data);file_bytes+=len(data)
                outputs[row_path.name]={'path':str(row_path),'sha256':file_sha.hexdigest(),
                    'bytes':file_bytes,'hash_scope':'SUCCESSFULLY_WRITTEN_ROWS'}
                counts[row['extraction_status']]+=1
        outputs[row_path.name]={'path':str(row_path),'sha256':file_sha.hexdigest(),'bytes':file_bytes}
        receipt['counts']={'planned':441,'attempted':counts['attempted'],'complete':counts[COMPLETE],
            'partial':sum(counts[s] for s in ('MISSING','UNSUPPORTED','LIMIT_EXCEEDED','CONFLICT')),
            'unavailable':counts['MISSING']+counts['UNSUPPORTED']+counts['LIMIT_EXCEEDED'],
            'conflict':counts['CONFLICT'],'by_extraction_status':{s:counts[s] for s in (COMPLETE,'MISSING','UNSUPPORTED','LIMIT_EXCEEDED','CONFLICT')}}
        receipt['status']='COMPLETE' if counts[COMPLETE]==441 and not unknown else 'PARTIAL'
    except (EvidenceError,OSError,ValueError,KeyError,TypeError,AttributeError,RecursionError) as error:
        receipt['status']='REJECTED_INPUT';receipt['error']={'status':getattr(error,'status','UNSUPPORTED'),'detail':str(error)}
    finally:
        if reader is not None: receipt['inputs']=reader.inputs;receipt['input_bytes_read']=reader.total
        receipt['elapsed_seconds']=time.monotonic()-start
        receipt['finished_at_utc']=datetime.now(timezone.utc).isoformat()
        data=encoded(receipt)
        with (out/'execution-receipt.json').open('xb') as handle: handle.write(data)
        # Receipt is written once; its digest is separate to avoid a self-hash cycle.
        with (out/'execution-receipt.sha256').open('xb') as handle:
            handle.write((digest(data)+'  execution-receipt.json\n').encode())
    return receipt


def main():
    parser=argparse.ArgumentParser(description=__doc__)
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
