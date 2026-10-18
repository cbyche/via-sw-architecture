"""Evaluator-only DP04 identity normalization from visible source labels, never target oracles."""
import json,re,xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path

_VISIBLE=re.compile(r'^([A-Z][0-9]{1,2})(?:\s*/\s*.+)?$')
def _text(value):return ' '.join(value.split()) if isinstance(value,str) else None

def _inventory(record):
    fixture=json.loads(Path(record['input']).read_text())
    inventories={}; errors=[]
    for rev in fixture.get('revisions',[]):
        records=[];seen=set()
        for frame in rev.get('frames',[]):
            key=(frame.get('frame_id'),frame.get('source_id'),frame.get('source_revision'),frame.get('sha256'))
            if key in seen:continue
            seen.add(key)
            # Evaluator reads the authored render source solely to inventory visible labels.
            source=Path(frame['image_path']).with_suffix('.svg')
            try:tree=ET.parse(source)
            except (OSError,ET.ParseError) as error:
                errors.append({'revision':rev.get('revision'),'frame_id':frame.get('frame_id'),'reason':'VISIBLE_SOURCE_INVENTORY_UNAVAILABLE','detail':str(error)});continue
            for element_index,element in enumerate(tree.getroot().iter()):
                if element.tag.rsplit('}',1)[-1]!='text':continue
                label=_text(''.join(element.itertext()));match=_VISIBLE.fullmatch(label or '')
                if match:records.append({'visible_id':match[1],'label':label,'frame_id':key[0],'source_id':key[1],'source_revision':key[2],'sha256':key[3],'inventory_source':str(source),'visible_text_index':element_index})
        inventories[str(rev.get('revision'))]=records
    return inventories,errors

def _screen_state(record,adoption):
    # Publication events survive a later episode failure; only already-published same-revision
    # state is eligible. No future producer output or different-run output is consulted.
    state=None;revision=adoption.get('revision')
    for event in record.get('events',[]):
        data=event.get('data',{})
        if event.get('kind')=='SCREEN_CANDIDATES_PUBLISHED' and data.get('revision')==revision:state=data.get('state')
        if event.get('kind')in ('ADOPTION','SEMANTIC_ADOPTION')and data.get('expression_id')==adoption.get('expression_id')and data.get('revision')==revision and data.get('targets')==adoption.get('targets'):break
    return state

def canonicalize_adoptions(record,adoptions):
    inventories,errors=_inventory(record);normalized=[];mappings=[];unresolved=list(errors)
    for adoption in adoptions:
        new=dict(adoption);targets=adoption.get('targets');revision=adoption.get('revision');inventory=inventories.get(str(revision),[])
        if not isinstance(targets,list):normalized.append(new);continue
        state=_screen_state(record,adoption);candidates=state.get('candidates',[])if isinstance(state,dict)else[]
        new_targets=[];identity_unresolved=False
        for target in targets:
            raw=_text(target);matches=[];method=None
            # A bare source-visible ID or the exact visible header denotes the same item.
            if raw is not None:
                matches=[x for x in inventory if raw in (x['visible_id'],x['label'])];method='EXACT_VISIBLE_SOURCE_ID_OR_LABEL'
            # An implementation-specific B ID is interpreted only through its actual
            # published candidate's visible header and verified source-frame identity.
            if not matches and raw is not None:
                aliases=[c for c in candidates if isinstance(c,dict)and c.get('target_id')==target]
                if len(aliases)==1:
                    c=aliases[0];label=c.get('label');header=_text(label.splitlines()[0])if isinstance(label,str)and label.splitlines()else None
                    snapshot=state.get('source_snapshot',[])
                    for item in inventory:
                        source_ok=any(all(s.get(k)==item[k]for k in ('frame_id','source_id','source_revision','sha256'))for s in snapshot if isinstance(s,dict))
                        if source_ok and c.get('frame_id')==item['frame_id'] and header==item['label']:matches.append(item)
                    method='PUBLISHED_CANDIDATE_EXACT_VISIBLE_LABEL_AND_FRAME'
                elif len(aliases)>1:
                    unresolved.append({'expression_id':adoption.get('expression_id'),'revision':revision,'raw_target':target,'reason':'AMBIGUOUS_DUPLICATE_PRODUCER_ALIAS'})
            # Duplicate source labels across distinct frames/versions are unresolved even
            # when their strings share an ID. Normalization cannot guess source intent.
            unique={(x['visible_id'],x['frame_id'],x['source_id'],x['source_revision'],x['sha256'],x['visible_text_index']):x for x in matches}
            if len(unique)==1:
                item=next(iter(unique.values()));canonical=item['visible_id'];new_targets.append(canonical)
                mappings.append({'expression_id':adoption.get('expression_id'),'revision':revision,'raw_target':target,'canonical_target':canonical,'method':method,'source':item})
            else:
                identity_unresolved=True
                new_targets.append(target)
                unresolved.append({'expression_id':adoption.get('expression_id'),'revision':revision,'raw_target':target,'reason':'NO_SOURCE_BACKED_EXACT_IDENTITY'if not unique else'AMBIGUOUS_SOURCE_IDENTITY'})
        new['targets']=new_targets;new['_target_identity_unresolved']=identity_unresolved;normalized.append(new)
    return {'adoptions':normalized,'mappings':mappings,'unresolved':unresolved}
