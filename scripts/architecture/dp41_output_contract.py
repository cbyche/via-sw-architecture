"""Validate DP41 documentation examples, not an executable resolution engine."""
from pathlib import Path
import copy
import json

ROOT = Path(__file__).resolve().parents[2]
CONTRACTS = ROOT / 'docs/architecture/12-decisions/decision-packages/contracts'
STRUCTURER = '요청 구조화기'
TOOLS = {'task_retrieve', 'context_retrieve', 'interaction_retrieve'}


def load():
    return (json.loads((CONTRACTS/'dp41-request-frame-v1.schema.json').read_text()),
            json.loads((CONTRACTS/'dp41-output-examples.json').read_text()))


def shape(value, rule, root):
    """Check only the JSON Schema vocabulary used by this document contract."""
    allowed={'$schema','$id','title','$defs','$ref','oneOf','const','enum','type','required','additionalProperties','properties','items','minItems','maxItems','minLength','maxLength','minimum','maximum'}
    if set(rule)-allowed: raise ValueError('unsupported schema vocabulary in documentation checker')
    if '$ref' in rule:
        return shape(value, root['$defs'][rule['$ref'].split('/')[-1]], root)
    if 'oneOf' in rule:
        matches=0
        for branch in rule['oneOf']:
            try: shape(value,branch,root); matches+=1
            except ValueError: pass
        if matches!=1: raise ValueError('oneOf must match exactly one branch')
    if 'const' in rule and (type(value)!=type(rule['const']) or value!=rule['const']):
        raise ValueError('wrong constant')
    if 'enum' in rule and value not in rule['enum']: raise ValueError('unknown operation/value')
    types={'object':lambda v:isinstance(v,dict),'array':lambda v:isinstance(v,list),
           'string':lambda v:isinstance(v,str),'integer':lambda v:type(v)==int,'null':lambda v:v is None}
    expected=rule.get('type')
    if expected and not any(types[t](value) for t in ([expected] if isinstance(expected,str) else expected)):
        raise ValueError('wrong type')
    if isinstance(value,dict) and expected=='object':
        if set(rule.get('required',[]))-value.keys(): raise ValueError('missing required field')
        if rule.get('additionalProperties') is False and value.keys()-rule['properties'].keys():
            raise ValueError('unknown field')
        for k,v in value.items(): shape(v,rule['properties'][k],root)
    if isinstance(value,list) and expected=='array':
        if not rule.get('minItems',0)<=len(value)<=rule.get('maxItems',10**9): raise ValueError('array bounds')
        for v in value: shape(v,rule['items'],root)
    if isinstance(value,str) and not rule.get('minLength',0)<=len(value)<=rule.get('maxLength',10**9):
        raise ValueError('string bounds')
    if type(value)==int and not rule.get('minimum',-10**9)<=value<=rule.get('maximum',10**9):
        raise ValueError('number bounds')


def check_frame(frame, schema, input_ref, utterance):
    shape(frame,schema,schema)
    if frame['input_ref']!=input_ref: raise ValueError('wrong input')
    for item in frame['goals']+frame['relations']+frame['constraints']:
        if item['source_text'] not in utterance: raise ValueError('source text outside input')
    goals={g['id']:g for g in frame['goals']}; refs={r['id']:r for r in frame['references']}
    if len(goals)!=len(frame['goals']) or len(refs)!=len(frame['references']): raise ValueError('duplicate ID')
    sources={'INPUT_SELECTION':'CONTEXT','RECENT_TASK':'TASK','RECENT_INTERACTION':'INTERACTION'}
    for r in refs.values():
        if r['selector']['op'] in sources and r['source']!=sources[r['selector']['op']]:
            raise ValueError('selector/source mismatch')
    for g in goals.values():
        if g['target_ref'] is not None and g['target_ref'] not in refs: raise ValueError('dangling target')
        if set(g['input_refs'])-refs.keys(): raise ValueError('dangling input')
        if g['kind'] in ('UPDATE_TASK','CANCEL_TASK') and (g['target_ref'] is None or refs[g['target_ref']]['source']!='TASK'):
            raise ValueError('Task target required')
    edges={k:set() for k in goals}
    for r in frame['relations']:
        if r['from_goal'] not in goals or r['to_goal'] not in goals: raise ValueError('dangling relation')
        if r['from_goal']==r['to_goal']: raise ValueError('self relation')
        if r['op']!='INDEPENDENT': edges[r['from_goal']].add(r['to_goal'])
        if r['op']=='WHEN' and r['predicate']['reference_id'] not in refs: raise ValueError('dangling predicate')
    def walk(k,path):
        if k in path: raise ValueError('dependency cycle')
        for next_id in edges[k]: walk(next_id,path|{k})
    for k in goals: walk(k,set())
    for c in frame['constraints']:
        if not c['scope'] or set(c['scope'])-goals.keys(): raise ValueError('invalid constraint scope')


def check_partial(result, request):
    if set(result)!={'field_ref','candidate_ref','evidence_refs','uncertain'}:
        raise ValueError('partial result cannot replace the frame')
    if result['field_ref']!=request['field_ref'] or result['candidate_ref'] not in request['candidate_refs']:
        raise ValueError('partial result outside supplied field/candidates')
    if type(result['uncertain']) is not bool or not isinstance(result['evidence_refs'],list):
        raise ValueError('invalid partial types')
    if not result['evidence_refs'] or set(result['evidence_refs'])-set(request['evidence_refs']):
        raise ValueError('partial result outside supplied evidence')


def check_read(packet):
    if set(packet)!={'kind','tool_calls'} or packet['kind']!='READ' or not 1<=len(packet['tool_calls'])<=32:
        raise ValueError('invalid read packet')
    for call in packet['tool_calls']:
        if set(call)!={'name','arguments'} or call['name'] not in TOOLS:
            raise ValueError('unknown read tool')
        args=call['arguments']
        if call['name']=='context_retrieve':
            if args!={'selector':'INPUT_SELECTION'}: raise ValueError('invalid context selector')
        elif set(args)!={'query','limit'} or not isinstance(args['query'],str) or not 1<=len(args['query'])<=512 or type(args['limit'])!=int or not 1<=args['limit']<=10:
            raise ValueError('invalid bounded read arguments')


def main():
    schema,e=load();check_frame(e['b_frame'],schema,e['input_ref'],e['utterance'])
    for packet in (e['a_read'],e['a_followup_read']):check_read(packet)
    forbidden=copy.deepcopy(e['a_read']);forbidden['tool_calls'][0]['arguments']['policy_override']=True
    try:check_read(forbidden)
    except ValueError:pass
    else:raise AssertionError('invented authority argument accepted')
    invalid=[]
    def reject(change):
        f=copy.deepcopy(e['b_frame']);change(f);invalid.append(f)
    reject(lambda f:f.update(arbitrary_new_field='invented'))
    reject(lambda f:f['goals'][0].update(kind='RUN_MODEL_CODE'))
    reject(lambda f:f['relations'][0].update(to_goal='missing'))
    reject(lambda f:f['references'][0]['selector'].update(op='EXECUTE_EXPRESSION'))
    reject(lambda f:f['relations'].append({'op':'AFTER','from_goal':'G2','to_goal':'G1','source_text':'결과를 바탕으로'}))
    reject(lambda f:f.update(input_ref='wrong input'))
    reject(lambda f:f['constraints'][0].update(source_text='임의로 지어낸 금지'))
    for f in invalid:
        try:check_frame(f,schema,e['input_ref'],e['utterance'])
        except ValueError:continue
        raise AssertionError('invalid documentation example accepted')
    check_partial(e['b_partial_result'],e['b_partial_input'])
    for change in (lambda r:r.update(relations=[]),lambda r:r.update(candidate_ref='Task3')):
        r=copy.deepcopy(e['b_partial_result']);change(r)
        try:check_partial(r,e['b_partial_input'])
        except ValueError:continue
        raise AssertionError('invalid partial result accepted')
    print('PASS: fixed RequestFrame fields/types/operations/references; valid example + 7 frame/2 partial rejection cases; bounded read examples')
    print('Documentation consistency only: no model run, resolution engine or semantic-accuracy test.')


if __name__=='__main__':main()
