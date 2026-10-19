"""TEST-ONLY deterministic structural responder, never a model-quality result.
Reads authored SVG pixels' source as a mock vision service. No target oracle is read.
Real candidate/model input contains PNGs, not these SVG-derived extents.
Optional meta.dp04_probe_bad_initial exercises host rejection and bounded owner repair.
"""
import copy
import re
import xml.etree.ElementTree as ET
from pathlib import Path


def _inside(p, ps):
    hit = False
    for a,b in zip(ps,ps[1:]+ps[:1]):
        if (a[1]>p[1]) != (b[1]>p[1]) and p[0] < (b[0]-a[0])*(p[1]-a[1])/(b[1]-a[1])+a[0]: hit=not hit
    return hit


def _source_candidates(frames):
    output=[]
    for frame in frames:
        tree=ET.parse(Path(frame['image_path']).with_suffix('.svg'))
        nodes=list(tree.getroot().iter());rects=[n for n in nodes if n.tag.endswith('rect') and 'rx' in n.attrib]
        circles=[n for n in nodes if n.tag.endswith('circle')]
        for n in nodes:
            if not n.tag.endswith('text'): continue
            label=''.join(n.itertext())
            if not re.fullmatch(r'[A-Z][0-9]{1,2}(?: / .+)?',label):continue
            target=label.split(' / ')[0];x=float(n.attrib['x']);y=float(n.attrib['y'])
            if target.startswith('P'):
                circle=min(circles,key=lambda c:abs(float(c.attrib['cx'])+12-x)+abs(float(c.attrib['cy'])-10-y))
                cx,cy=float(circle.attrib['cx']),float(circle.attrib['cy']);box=[cx-8,cy-8,cx+8,cy+8];kind='sample'
            else:
                r=next(r for r in rects if float(r.attrib['x'])<=x<=float(r.attrib['x'])+float(r.attrib['width']) and float(r.attrib['y'])<=y<=float(r.attrib['y'])+float(r.attrib['height']))
                rx,ry,w,h=(float(r.attrib[k]) for k in ['x','y','width','height']);box=[rx,ry,rx+w,ry+h];kind='card'
            output.append({'target_id':target,'frame_id':frame['frame_id'],'kind':kind,'label':label,'bbox':box})
    return output


def _cue_targets(event,candidates):
    cs=[c for c in candidates if c['frame_id']==event['frame_id']]
    if event['type']=='pointer_move':
        return [c['target_id'] for c in cs if c['bbox'][0]<=event['x']<=c['bbox'][2] and c['bbox'][1]<=event['y']<=c['bbox'][3]]
    if event['type']=='pointer_path':
        return [c['target_id'] for c in cs if c['kind']=='sample' and _inside([(c['bbox'][0]+c['bbox'][2])/2,(c['bbox'][1]+c['bbox'][3])/2],event['points'])]
    return []


def _speech(r):
    expressions=[]
    for e in r['expressions']:
        cues=[ev['event_id'] for ev in r['events'] if ev['type'] in ['pointer_move','pointer_path'] and e['start_ms']<=ev.get('t_ms',ev.get('start_ms',-1))<=e['end_ms']]
        is_loop=any(ev['event_id'] in cues and ev['type']=='pointer_path' for ev in r['events'])
        expressions.append({'expression_id':e['expression_id'],'kind':'sample' if is_loop else 'item','count':3 if '세 점' in e['text'] else 2 if '두 그래프' in e['text'] else 1,'relation':'within_region' if is_loop else ('designated_by_cue' if cues else ('retain_prior' if '그대로' in e['text'] else 'relative_position')),'cue_ids':cues,'explicit_target_ids':[],'unresolved':[],'meaning':e['text']})
    return {'intent':'지칭한 대상의 비교 또는 자료 사용 요청을 현재 판본에 연결','expressions':expressions,'coverage':{'complete':True,'unknown':[]}}


def _bind(screen,speech,events,prior):
    out=[]
    for e in speech['expressions']:
        ids=[]
        if e['cue_ids']:
            for cue in e['cue_ids']:
                relation=next((x for x in screen['cue_relations'] if x['cue_id']==cue),None)
                if relation:ids.extend(relation['target_ids'])
        else:
            old=next((a for a in (prior or {}).get('adoptions',[]) if a['expression_id']==e['expression_id']),None)
            meaning=e['meaning']
            if e['relation']=='retain_prior' and old and old['targets']:ids=old['targets']
            else:
                candidates=sorted(screen['candidates'],key=lambda c:c['bbox'][0])
                explicit=re.findall(r'\b[A-Z][0-9]{1,2}\b',meaning)
                if explicit:ids=explicit
                elif '안쪽 두' in meaning:ids=[c['target_id'] for c in candidates[1:-1]]
                else:
                    pos=1 if '왼쪽에서 두 번째' in meaning else -1 if '오른쪽' in meaning else (len(candidates)//2 if '가운데' in meaning else 0)
                    ids=[candidates[pos]['target_id']]
        out.append({'expression_id':e['expression_id'],'targets':ids,'status':'ready','count':e['count'],'relation':e['relation'],'cue_ids':e['cue_ids'],'extents':[c for c in screen['candidates'] if c['target_id'] in ids],'evidence':'Offline structural source geometry; not a model inference result'})
    return {'intent':speech['intent'],'adoptions':out,'need_expansion':False,'need_evidence':False}


def respond(payload, meta):
    stage=meta['stage']; value=payload['input']
    if stage=='dp04.local_screen_parser':return {'frames':[],'scope':'offline structural double; not actual parser'}
    if 'speech' in stage:
        r=value.get('original',value)
        return _speech(r)
    if 'candidate_binding' in stage:
        return _bind(value['screen'],value['speech'],[],value.get('prior'))
    joint='joint' in stage
    if joint:
        value=value['original'];r=value['observations'];prior=value.get('prior')
    else:r=value;prior=None
    candidates=_source_candidates(r['frames'])
    relations=[]
    for event in r['events']:
        if event['type'] not in ['pointer_move','pointer_path']:continue
        targets=_cue_targets(event,candidates)
        relations.append({'cue_id':event['event_id'],'target_ids':targets,'relation':'within_region' if event['type']=='pointer_path' else 'point_inside','complete':True,'unique':True,'navigation_alternative':False})
    screen={'candidates':candidates,'cue_relations':relations,'coverage':{'complete':True,'unknown':[],'competing':[]},'source_refs':[f['frame_id'] for f in r['frames']]}
    result=_bind(screen,_speech(r),r['events'],prior) if joint else screen
    initial=stage in ['dp04.joint_grounding','dp04.screen_producer']
    if (meta.get('dp04_probe_bad_initial') and initial) or (meta.get('dp04_probe_fail_revision_one') and r.get('revision')==1):
        if joint: result['adoptions'][0]['extents'][0]['bbox']=[-1,-1,2000,2000]
        else:
            selected=[t for rel in relations for t in rel['target_ids']]
            chosen=next((c for c in result['candidates'] if c['target_id'] in selected),result['candidates'][0])
            chosen['bbox']=[-1,-1,2000,2000]
    result=copy.deepcopy(result)
    extents=result.get('candidates',[]) if not joint else [c for a in result.get('adoptions',[]) for c in a.get('extents',[])]
    for c in extents:
        f=next(f for f in r['frames'] if f['frame_id']==c['frame_id']);b=c.pop('bbox');c['bbox_2d']=[round(b[0]/f['width']*1000),round(b[1]/f['height']*1000),round(b[2]/f['width']*1000),round(b[3]/f['height']*1000)];c['coordinate_space']='qwen_normalized_0_1000'
    return result
