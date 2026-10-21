"""Apply editable figure regions and shape-owned text to an existing deck.

Keep native QA tables, page order, relationships, masters, notes and all resources.
No original version is overwritten. New figures have no external relationships.
"""
from pathlib import Path
import copy,json,sys,xml.etree.ElementTree as E
sys.path.insert(0,str(Path(__file__).resolve().parent))
import full_presentation_package as F
import presentation_windows_policy as W
from migrate_native_shape_labels import migrate_parts


def sig(n):
    return (n.tag,tuple(sorted(n.attrib.items())),n.text or '',tuple(sig(c) for c in n))


def bounds(s):
    q=s.find('p:spPr/a:xfrm',F.NS)
    if q is None:return None
    a,b=q.find('a:off',F.NS),q.find('a:ext',F.NS)
    return [int(a.get('x'))/9525,int(a.get('y'))/9525,int(b.get('cx'))/9525,int(b.get('cy'))/9525]


def replace_text(shape, text, size_pt=None, rect=None):
    body=shape.find('p:txBody',F.NS); assert body is not None
    template=copy.deepcopy(body.find('a:p',F.NS)); assert template is not None
    props=template.find('a:pPr',F.NS)
    rprops=template.find('.//a:rPr',F.NS)
    for paragraph in list(body.findall('a:p',F.NS)): body.remove(paragraph)
    for line in text.split('\n'):
        p=E.SubElement(body,'{'+F.A+'}p')
        if props is not None:p.append(copy.deepcopy(props))
        r=E.SubElement(p,'{'+F.A+'}r')
        rp=copy.deepcopy(rprops) if rprops is not None else E.Element('{'+F.A+'}rPr')
        rp.set('lang','ko-KR')
        if size_pt:rp.set('sz',str(size_pt*100))
        r.append(rp);E.SubElement(r,'{'+F.A+'}t').text=line
    if rect:
        x,y,w,h=rect;tf=shape.find('p:spPr/a:xfrm',F.NS)
        for elem,values in [('a:off',dict(x=x,y=y)),('a:ext',dict(cx=w,cy=h))]:
            for key,val in values.items():tf.find(elem,F.NS).set(key,str(round(val*9525)))


def update_legends(parts, order):
    changes=[]
    patches={
      2:[('22','검정 공통\n파랑 A / 초록 B/C\n실선 요청 / 점선 결과',13,None),('23','같은 이름: 같은 책임',12,[35,540,155,35])],
      3:[('22','검정 공통\n파랑 A / 초록 B/C\n실선 요청 / 점선 결과',13,None)],
      4:[('36','같은 Component 반복 / 실선 요청 / 점선 반환\n채택 후 갱신 → 다음 요청',13,[47,542,568,49])],
      6:[('25','실선 요청 / 점선 결과\n검정 공통 / 파랑 A\n초록 B',13,[35,560,151,52])],
      7:[('25','실선 요청 / 점선 결과\n검정 공통 / 파랑 A\n초록 B / 파랑 보완 B1',12,[35,560,151,52])],
      8:[('44','검정 기본 B / 파랑 B1\n실선 갱신 / 점선 조회. 위/아래는 대안 비교\n공유 입력을 빌려 payload 직렬화',13,[43,920,575,54])],
      17:[('22','실선 요청 / 전달\n점선 결과 / 반환',14,None),('23','검정 공통 / 파랑 A\n초록 B/C\nTask와 원본 권위는\n기존 owner 유지',12,[32,550,154,85])],
      18:[('22','실선 요청 / 전달\n점선 결과 / 반환',14,None),('23','검정 공통\n파랑 A / 보완 C1\n초록 B/C\nTask와 원본 권위는\n기존 owner 유지',12,[32,550,154,90])],
    }
    for page,items in patches.items():
        part=order[page-1][1];root=E.fromstring(parts[part])
        by_id={s.find('p:nvSpPr/p:cNvPr',F.NS).get('id'):s for s in root.findall('p:cSld/p:spTree/p:sp',F.NS)}
        for id,text,size,rect in items:
            shape=by_id[id];old=''.join(n.text or '' for n in shape.findall('.//a:t',F.NS))
            old_bounds=[round(v*9525) for v in bounds(shape)]
            replace_text(shape,text,size,rect)
            changes.append(dict(page=page,part=part,id=id,old_text=old,new_text=text,old_bounds_emu=old_bounds,new_bounds_emu=[round(v*9525) for v in bounds(shape)],size_pt=size))
        parts[part]=W.normalize_xml(part,F.xml(root))
    return changes


def append_scene_notes(parts,order,scenes):
    added=[]
    for scene in scenes:
        note=F.related(parts,order[scene['page']-1][1],'notesSlide')
        root=E.fromstring(parts[note]);body=None
        for shape in root.findall('.//p:sp',F.NS):
            ph=shape.find('p:nvSpPr/p:nvPr/p:ph',F.NS)
            if ph is not None and ph.get('type')=='body':body=shape.find('p:txBody',F.NS);break
        assert body is not None,note
        text='v10 가이드 검토에 따른 도식 설명 (기존 노트와 QA 근거 보존)\n'+scene['notes'].strip()
        for line in text.split('\n'):
            p=E.SubElement(body,'{'+F.A+'}p');r=E.SubElement(p,'{'+F.A+'}r')
            rp=E.SubElement(r,'{'+F.A+'}rPr',{'lang':'ko-KR','sz':'1200'})
            for script in ['latin','ea','cs']:E.SubElement(rp,'{'+F.A+'}'+script,{'typeface':'Calibri'})
            E.SubElement(r,'{'+F.A+'}t').text=line
        parts[note]=W.normalize_xml(note,F.xml(root));added.append(dict(page=scene['page'],part=note,added_text=text))
    return added


def pack(source_path,authored_path,scene_path,mapping_path,output_path,report_path):
    source,authored=F.read(source_path),F.read(authored_path)
    scenes=json.loads(Path(scene_path).read_text())
    order=F.ordered(source);aorder=F.ordered(authored)
    assert len(aorder)==len(scenes)
    parts=dict(source); patches=[]
    for scene,(_,a_part) in zip(scenes,aorder):
        page=scene['page'];part=order[page-1][1]
        root=E.fromstring(parts[part]);tree=root.find('p:cSld/p:spTree',F.NS)
        before_tables=[sig(n) for n in root.findall('.//a:tbl',F.NS)]
        original_max_id=max(int(n.get('id')) for n in root.findall('.//p:cNvPr',F.NS))
        regions=scene['replacement_regions'];removed=[]
        remove_ids=set(map(str,scene.get('fragment',{}).get('removeShapeIds',[])))
        for s in list(tree):
            b=bounds(s)
            if b is None:continue
            x,y,w,h=b
            if (s.find('p:nvSpPr/p:cNvPr',F.NS).get('id') in remove_ids) if remove_ids else any(x>=r[0]-.3 and y>=r[1]-.3 and x+w<=r[0]+r[2]+.3 and y+h<=r[1]+r[3]+.3 for r in regions):
                removed.append({'id':s.find('p:nvSpPr/p:cNvPr',F.NS).get('id'),'text':''.join(n.text or '' for n in s.findall('.//a:t',F.NS)),'bounds':b});tree.remove(s)
        assert removed,('No shapes selected',page)
        aroot=E.fromstring(W.normalize_xml(a_part,authored[a_part]));atree=aroot.find('p:cSld/p:spTree',F.NS)
        highest=original_max_id+1000
        inserted=[]
        for s in atree:
            if s.tag!='{'+F.P+'}sp':continue
            q=copy.deepcopy(s)
            assert not any(k.startswith('{'+F.R+'}') for n in q.iter() for k in n.attrib),'Unexpected dependency'
            n=q.find('p:nvSpPr/p:cNvPr',F.NS); highest+=1;n.set('id',str(highest));n.set('name','v10 '+n.get('name',''))
            tree.append(q);inserted.append({'id':highest,'name':n.get('name')})
        assert [sig(n) for n in root.findall('.//a:tbl',F.NS)]==before_tables,'QA table changed'
        parts[part]=F.xml(root)
        patches.append({'page':page,'part':part,'regions':regions,'removed':removed,'inserted':inserted,'native_tables_preserved':True})
    parts,label_report=migrate_parts(parts,mapping_path)
    legend_changes=update_legends(parts,order)
    notes_appended=append_scene_notes(parts,order,scenes)
    # A figure repair does not alter the deck's QA tables or footers.
    for page,(_,part) in enumerate(order,1):
        old,new=E.fromstring(source[part]),E.fromstring(parts[part])
        assert [sig(n) for n in old.findall('.//a:tbl',F.NS)]==[sig(n) for n in new.findall('.//a:tbl',F.NS)]
        footer=f'{page} / {len(order)}'
        text=[''.join(n.text or '' for n in s.findall('.//a:t',F.NS)) for s in new.findall('.//p:sp',F.NS)]
        assert text.count(footer)==1,(page,footer)
    changed=[n for n in parts if parts[n]!=source[n]]
    assert all(n in {p for _,p in order}|{n['part'] for n in notes_appended} for n in changed),'Unplanned shared part changed'
    assert F.ordered(parts)==order
    F.verify(parts);policy=W.policy_report(parts)
    F.write(Path(output_path),parts)
    report={'source':str(source_path),'source_sha256':F.digest(Path(source_path).read_bytes()),'output':str(output_path),'output_sha256':F.digest(Path(output_path).read_bytes()),'slide_count':len(order),'patches':patches,'labels':label_report,'legend_changes':legend_changes,'notes_appended':notes_appended,'changed_parts':changed,'all_native_tables_preserved':True,'original_notes_text_preserved_with_scene_additions':True,'relationships_masters_resources_preserved':True,'page_order_and_footers_preserved':True,'typography':policy}
    Path(report_path).write_text(json.dumps(report,ensure_ascii=False,indent=2))
    return report

if __name__=='__main__':
    print(json.dumps({k:v for k,v in pack(*sys.argv[1:]).items() if k not in ('patches','labels')},ensure_ascii=False,indent=2))
