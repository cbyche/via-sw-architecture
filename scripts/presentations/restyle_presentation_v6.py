"""Preserve the native v5 package while applying the Artifact Tool v6 edit.

The reference consists of PNGs, so the header is authored as editable objects.
This pass attaches existing labels to their own shapes, retains the original
diagram geometry and media, and repairs the imported master registries.
"""
import copy
import json
import re
import sys
from pathlib import Path
import xml.etree.ElementTree as E

import full_presentation_package as F
import presentation_windows_policy as W

A, P = W.A, W.P
EMU = 9525
BLACK, BLUE, GREEN, GRAY = '000000', '4472C4', 'C6E0B4', 'D9D9D9'

TITLES = [
    '과제 배경 | 관련 동향', '과제 필요성', '과제 소개 | VIA의 역할', '과제 소개 | 시스템 구조',
    '대표 시나리오', '업무 위임과 대화 연결', '요구사항 도출', '기능 요구사항 및 제약사항',
    '품질 요구사항 (1/2)', '품질 요구사항 (2/2)', '상호작용 응답시간과 요청 처리시간',
    '요구사항과 설계 질문', '설계 Overview', '진행 중 업무와 새 입력',
    'DP41. 요청 해석의 진행 책임 (배경)', 'DP41. 정확성을 위한 요청 해석 설계',
    'DP44. 사건의 겹침과 응답 시점 (배경)', 'DP44. 응답시간을 위한 지속 대화 설계',
    'DP45. 과거 근거의 연결과 재사용 (배경)', 'DP45. 정확성을 위한 과거 맥락 설계',
    'DP42. 대화와 업무의 수명 (배경)', 'DP42. 변경 용이성을 위한 상태 관리 설계',
    '기능 범위와 공통 계약', '설계 검토 요약', 'Appendix',
    '공통 평가 활동 | P21', 'P21 사례 대응 (1/3)', 'P21 사례 대응 (2/3)', 'P21 사례 대응 (3/3)',
    'QA01. 정확성 지표와 등급', 'QA02. 상호작용 응답시간 지표', 'QA03. VIA 요청 처리시간 지표',
    'QA04. 모델 사용 비용 지표', 'QA05. 변경 용이성 지표', 'QA06. 로컬 메모리 사용량 지표',
    '과제 배경 | 관련 동향 (원본)', 'VIA 역할과 범위 (원본)', '과제 필요성 (원본)',
    '대표 시나리오 (원본)', '기능 요구사항과 제약 (원본)', '품질 요구사항 (원본 1/2)',
    '품질 요구사항 (원본 2/2)', '설계 Overview (원본)', '업무 위임과 대화 연결 (원본)',
    '진행 중 업무와 새 입력 (원본)', 'DP41. 요청 해석 배경 (원본)', 'DP41. 요청 해석 설계 비교 (원본)',
    'DP41. 품질 평가 상세', 'DP44. 지속 대화 배경 (원본)', 'DP44. 지속 대화 설계 비교 (원본)',
    'DP44. 품질 평가 상세', 'DP45. 과거 맥락 배경 (원본)', 'DP45. 과거 맥락 설계 비교 (원본)',
    'DP45. 품질 평가 상세', 'DP42. 상태 수명 배경 (원본)', 'DP42. 상태 관리 설계 비교 (원본)',
    'DP42. 품질 평가 상세', 'DP41. 요청 해석 배경 (v2)', 'DP44. 지속 대화 배경 (v2)',
    'DP45. 과거 맥락 배경 (v2)', 'DP42. 상태 수명 배경 (v2)',
]
REPLACEMENTS = {
    1: {'화면을 보며 말로 지시하는 상호작용': '말로 업무를 맡기고 이어가는 상호작용',
        '화면의 대상을 가리키며 음성으로 요청': '앞선 결과를 바탕으로 음성으로 수정 요청',
        '화면과 대화의 맥락을 기기 경험과 연결하고,': '대화와 업무의 맥락을 기기 경험과 연결하고,'},
    3: {'음성 또는 텍스트 + 당시 화면 + 앞선 대화와 진행 업무': '음성 또는 텍스트 + 앞선 대화와 진행 업무 + 필요한 자료',
        '요청의 의미와 대상 확정': '요청의 목표와 적용 범위 확정'},
    4: {'Voice Interaction Agent: 음성, 텍스트와 당시 화면을 함께 받아 PC 업무를 이어주는 프로그램':
        'Voice Interaction Agent: 음성과 텍스트로 PC 업무를 맡기고, 대화와 진행 상태를 이어주는 프로그램',
        '입력과 화면': '입력과 관련 맥락', '화면과 과거 자료': '자료와 과거 기록',
        '화면에 표가 있고,': '매출 표가 준비되어 있고,'},
    5: {'① 화면을 짚어 질문': '① 자료의 핵심을 질문', '당시 화면의 표를 지정': '매출 표에 대해 질문'},
    6: {'이 표와 지난달 매출보고서를 바탕으로': '매출 자료와 지난달 매출보고서를 바탕으로',
        '음성 요청 + 선택한 표': '음성 요청 + 매출 자료'},
    8: {'화면, 자료와 대화 맥락을 이용해': '자료와 대화 및 업무 맥락을 이용해'},
    12: {'FR2 / FR3  대상, 조건과 의존 관계': 'FR2 / FR3  목표, 조건과 의존 관계',
         '요청의 대상과 순서, 조건을 어떻게 정확히 확정할까?': '요청의 적용 범위와 순서, 조건을 어떻게 정확히 확정할까?'},
    15: {'이 표를 아까 보고서에 넣고,': '매출 자료를 아까 보고서에 넣고,', '이 표': '이번 자료',
         '당시 선택 화면과 시점': '앞선 자료 참조와 대화 기록', '실제 표와 근거 연결': '사용할 자료와 근거 연결',
         '대상이 모호하면 확인합니다.': '업무 연결이나 조건이 모호하면 확인합니다.',
         '요청의 대상과 순서 및 조건을 어떻게 정확히 확정할까?': '요청의 적용 범위, 순서와 조건을 어떻게 정확히 확정할까?'},
    16: {'이 표를 아까 보고서에 넣고,': '매출 자료를 아까 보고서에 넣고,'},
    25: {'원본 파일은 변경하지 않았고, 아래 페이지는 원래 문구와 도형을 그대로 유지합니다.':
         '이전 버전과 원본 문구, 발표자 노트는 보존했고, 아래 페이지에도 v6의 서식을 적용했습니다.',
         'v2의 실제 위치는 README와 manifest에서 확인할 수 있습니다.':
         '현재 슬라이드 위치는 README와 manifest에서 확인할 수 있습니다.'},
}


def text(node):
    return '\n'.join(''.join(t.text or '' for t in p.iter(A+'t')) for p in node.findall('p:txBody/a:p', F.NS))


def name(node):
    return node.find('p:nvSpPr/p:cNvPr', F.NS).get('name', '')


def bbox(node):
    x = node.find('p:spPr/a:xfrm', F.NS)
    if x is None: return None
    off, ext = x.find(A+'off'), x.find(A+'ext')
    if off is None or ext is None: return None
    return tuple(int(n) for n in (off.get('x'), off.get('y'), ext.get('cx'), ext.get('cy')))


def is_header(node):
    box = bbox(node)
    if box is None: return False
    x, y, w, h = [v/EMU for v in box]
    n = name(node)
    if '/ title /' in n and y < 190: return True
    if y < 110 and h < 120: return True
    if h < 3 and y < 150 and w > 1400: return True
    return False


def revise_paragraphs(root, replacements):
    for p in root.iter(A+'p'):
        nodes = list(p.iter(A+'t'))
        old = ''.join(n.text or '' for n in nodes)
        new = old
        for before, after in replacements.items(): new = new.replace(before, after)
        if new != old:
            nodes[0].text = new
            for node in nodes[1:]: node.text = ''


def black_text(root):
    for props in root.iter():
        if props.tag not in W.PROPERTIES: continue
        for child in list(props):
            if child.tag in {A+n for n in ('solidFill','gradFill','pattFill','noFill','blipFill','grpFill')}:
                props.remove(child)
        fill = E.Element(A+'solidFill')
        E.SubElement(fill,A+'srgbClr',{'val':BLACK})
        # Fill precedes effects, highlighting, underline and script fonts.
        index = next((i for i,n in enumerate(props) if n.tag not in {A+'ln'}),len(props))
        props.insert(index,fill)


def has_closed_geometry(node):
    props = node.find(P+'spPr')
    line = props.find(A+'ln')
    visible_fill = any(props.find(A+n) is not None for n in ('solidFill','gradFill','pattFill','blipFill'))
    visible_line = line is not None and line.find(A+'noFill') is None
    if not (visible_fill or visible_line): return False
    prst = node.find('p:spPr/a:prstGeom',F.NS)
    if prst is not None: return prst.get('prst') not in ('line','arc')
    return node.find('p:spPr/a:custGeom/a:pathLst/a:path/a:close',F.NS) is not None


def label_pairs(root):
    shapes = root.findall('p:cSld/p:spTree/p:sp',F.NS)
    targets = [s for s in shapes if not text(s) and bbox(s) and has_closed_geometry(s)]
    groups = {}
    for label in shapes:
        value = text(label); box = bbox(label)
        if not value or not box or is_header(label): continue
        x,y,w,h = box
        choices=[]
        for target in targets:
            a,b,c,d=bbox(target)
            if c < 20*EMU or d < 20*EMU: continue
            if not (x>=a-EMU and y>=b-EMU and x+w<=a+c+EMU and y+h<=b+d+EMU): continue
            ratio = w*h/(c*d)
            centered = ratio >= 0.15
            primitive_label = d <= 160*EMU
            container_title = (len(value)<55 and '\n' not in value and
                               x-a<55*EMU and y-b<35*EMU and
                               h<60*EMU and c<1400*EMU)
            if not (centered or container_title or primitive_label): continue
            # Canvas backgrounds and broad prose panels are not diagram labels.
            if c>1600*EMU and d>160*EMU: continue
            choices.append((c*d,target))
        if choices:
            target=min(choices,key=lambda a:a[0])[1]
            groups.setdefault(target,[]).append(label)
    return groups


def attach_labels(root):
    tree=root.find('p:cSld/p:spTree',F.NS)
    groups=label_pairs(root); removed={}; records=[]
    for target,labels in groups.items():
        # Attach the shape's own label; keep distant flow annotations at their
        # original positions instead of collapsing them into a container title.
        labels.sort(key=lambda n:(bbox(n)[1],bbox(n)[0]))
        # These wide rows have separate native cells authored by Artifact Tool.
        if is_split_row(target,labels):continue
        cluster=[labels[0]]
        for label in labels[1:]:
            _,previous_y,_,previous_h=bbox(cluster[-1])
            x,y,_,_=bbox(label)
            if abs(x-bbox(cluster[0])[0])>30*EMU or y-previous_y-previous_h>12*EMU:
                break
            cluster.append(label)
        labels=cluster
        a,b,c,d=bbox(target)
        boxes=[bbox(n) for n in labels]
        left=min(x for x,y,w,h in boxes); top=min(y for x,y,w,h in boxes)
        right=max(x+w for x,y,w,h in boxes); bottom=max(y+h for x,y,w,h in boxes)
        body=copy.deepcopy(labels[0].find(P+'txBody'))
        props=body.find(A+'bodyPr')
        original={k:int(props.get(k,'0')) for k in ('lIns','rIns','tIns','bIns')}
        props.set('lIns',str(max(0,left-a+original['lIns'])))
        props.set('rIns',str(max(0,a+c-right+original['rIns'])))
        props.set('tIns',str(max(0,top-b+original['tIns'])))
        props.set('bIns',str(max(0,b+d-bottom+original['bIns'])))
        if len(labels)>1:
            props.set('anchor','t')
            for p in list(body.findall(A+'p')): body.remove(p)
            for label in labels:
                for p in label.findall('p:txBody/a:p',F.NS): body.append(copy.deepcopy(p))
        geometry=target.find('p:spPr/a:prstGeom',F.NS)
        if geometry is not None and geometry.get('prst')=='diamond':
            # A diamond already restricts its text rectangle to the center.
            # Reusing the overlay's outer insets would clip its own label.
            for key in ('lIns','rIns','tIns','bIns'):props.set(key,'0')
            props.set('anchor','ctr')
        old=target.find(P+'txBody')
        if old is not None: target.remove(old)
        target.append(body)
        target.find('p:nvSpPr/p:cNvSpPr',F.NS).attrib.pop('txBox',None)
        target_id=target.find('p:nvSpPr/p:cNvPr',F.NS).get('id')
        for label in labels:
            removed[label.find('p:nvSpPr/p:cNvPr',F.NS).get('id')]=target_id
            tree.remove(label)
        records.append({'shape':name(target),'labels':[name(n) for n in labels],'text':text(target)})
    for n in root.iter():
        if n.tag in (A+'stCxn',A+'endCxn') and n.get('id') in removed:n.set('id',removed[n.get('id')])
    return records


def is_split_row(target,labels):
    if len(labels)<2:return False
    _,_,width,height=bbox(target)
    ys=[bbox(n)[1] for n in labels]
    xs=[bbox(n)[0] for n in labels]
    return (width>1000*EMU and height<100*EMU and max(ys)-min(ys)<10*EMU
            and max(xs)-min(xs)>100*EMU)


def color_shapes(root):
    for sp in root.findall('p:cSld/p:spTree/p:sp',F.NS):
        pr=sp.find(P+'spPr')
        fill=pr.find(A+'solidFill')
        if fill is None: continue
        rgb=fill.find(A+'srgbClr')
        if rgb is None: continue
        value=rgb.get('val').upper()
        if value in ('FFFFFF','000000'): continue
        r,g,b=[int(value[i:i+2],16) for i in (0,2,4)]
        if max(r,g,b)<150:
            rgb.set('val','EEEEEE')
        elif g>r and g>=b:
            rgb.set('val',GREEN)
        elif r>g and r>b:
            rgb.set('val','FFF6DE')
        else: rgb.set('val','E8EEF7')


def style_tables(root, page):
    for table in root.iter(A+'tbl'):
        for i,row in enumerate(table.findall(A+'tr')):
            for cell in row.findall(A+'tc'):
                props=cell.find(A+'tcPr')
                if props is None: props=E.SubElement(cell,A+'tcPr')
                # First rows are the source's table headers, with one exception:
                # the DP41 background has a genuine header row too.
                fill=props.find(A+'solidFill')
                if fill is None: fill=E.SubElement(props,A+'solidFill')
                fill.clear();E.SubElement(fill,A+'srgbClr',{'val':GREEN if i==0 else 'FFF6DE' if page in (9,10,41,42) else 'FFFFFF'})
                for line in props:
                    if line.tag not in {A+n for n in ('lnL','lnR','lnT','lnB','lnTlToBr','lnBlToTr')}: continue
                    for old in list(line):
                        if old.tag.endswith('Fill'): line.remove(old)
                    lf=E.Element(A+'solidFill');E.SubElement(lf,A+'srgbClr',{'val':BLUE if page in (9,10,41,42) else 'B7B7B7'})
                    line.insert(0,lf)
        # The SDK requires line children before fill, and cell3D after fill.
        for props in table.iter(A+'tcPr'):
            children=list(props)
            children.sort(key=lambda n:0 if n.tag in {A+x for x in ('lnL','lnR','lnT','lnB','lnTlToBr','lnBlToTr')} else 1 if n.tag.endswith('Fill') else 2)
            props[:]=children


def repair_masters(parts):
    presentation=E.fromstring(parts['ppt/presentation.xml'])
    for n in presentation.find(P+'sldMasterIdLst'):
        if n.tag==P+'slideMasterId': n.tag=P+'sldMasterId'
    notes=presentation.find(P+'notesMasterIdLst')
    keep_id=notes[0].get('{'+F.R+'}id')
    for n in list(notes)[1:]: notes.remove(n)
    rels=E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    keep_target=next(n.get('Target') for n in rels if n.get('Id')==keep_id)
    for n in list(rels):
        if n.get('Type').endswith('/notesMaster') and n.get('Id')!=keep_id: rels.remove(n)
    parts['ppt/presentation.xml']=F.xml(presentation)
    parts['ppt/_rels/presentation.xml.rels']=F.xml(rels)
    for name,data in list(parts.items()):
        if name.startswith('ppt/notesSlides/_rels/') and name.endswith('.rels'):
            root=E.fromstring(data)
            for rel in root:
                if rel.get('Type').endswith('/notesMaster'): rel.set('Target',keep_target)
            parts[name]=F.xml(root)


def prepare(source, output):
    parts=F.read(Path(source)); pages=[]
    for page,(_,part) in enumerate(F.ordered(parts),1):
        root=E.fromstring(parts[part]); updates=[]
        for sp in root.findall('p:cSld/p:spTree/p:sp',F.NS):
            old=text(sp); new=old
            for before,after in REPLACEMENTS.get(page,{}).items():new=new.replace(before,after)
            if new!=old:updates.append({'name':name(sp),'old':old,'new':new})
        tables=[]
        for i,tbl in enumerate(root.iter(A+'tbl')):
            for r,row in enumerate(tbl.findall(A+'tr')):
                for c,cell in enumerate(row.findall(A+'tc')):
                    old='\n'.join(''.join(n.text or '' for n in p.iter(A+'t')) for p in cell.findall('a:txBody/a:p',F.NS))
                    new=old
                    for before,after in REPLACEMENTS.get(page,{}).items():new=new.replace(before,after)
                    if new!=old:tables.append({'table':i,'row':r,'col':c,'old':old,'new':new})
        split_rows=[]
        for target,labels in label_pairs(root).items():
            if not is_split_row(target,labels):continue
            labels.sort(key=lambda n:bbox(n)[0])
            x,y,w,h=bbox(target)
            edges=[x]+[int((bbox(left)[0]+bbox(left)[2]+bbox(right)[0])/2)
                         for left,right in zip(labels,labels[1:])]+[x+w]
            for i,label in enumerate(labels):
                props=label.find('p:txBody/a:p/a:r/a:rPr',F.NS)
                lx,ly,lw,lh=bbox(label)
                split_rows.append({'name':f'v6-native-header-cell-{page}-{len(split_rows)}',
                    'label_name':name(label),'text':text(label),'font_size_pt':int(props.get('sz'))/100,
                    'bold':props.get('b')=='1','align':label.find('p:txBody/a:p/a:pPr',F.NS).get('algn','l'),
                    'box':[edges[i]/EMU,y/EMU,(edges[i+1]-edges[i])/EMU,h/EMU],
                    'text_box':[(lx-edges[i])/EMU,(ly-y)/EMU,(edges[i+1]-lx-lw)/EMU,(y+h-ly-lh)/EMU]})
        pages.append({'page':page,'title':TITLES[page-1],'shape_updates':updates,'table_updates':tables,
                      'native_header_cells':split_rows})
    Path(output).write_text(json.dumps({'source_sha256':F.digest(Path(source).read_bytes()),'pages':pages},ensure_ascii=False,indent=2)+'\n')


def finish(source, authored, plan_path, previous, output, manifest_path):
    parts=F.read(Path(source)); original=copy.deepcopy(parts); authored_parts=F.read(Path(authored))
    plan=json.loads(Path(plan_path).read_text()); old_manifest=json.loads(Path(previous).read_text())
    assert F.digest(Path(source).read_bytes())==plan['source_sha256']==old_manifest['output_sha256']
    authored_order=F.ordered(authored_parts); merge_log=[]
    for info,(_,part),(_,authored_part) in zip(plan['pages'],F.ordered(parts),authored_order):
        page=info['page'];root=E.fromstring(parts[part]);tree=root.find('p:cSld/p:spTree',F.NS)
        revise_paragraphs(root,REPLACEMENTS.get(page,{}));black_text(root)
        if page<=24:revise_paragraphs(root,{' / v2 ':' / v6 '})
        for sp in list(tree.findall(P+'sp')):
            if is_header(sp):tree.remove(sp)
        merge_log.append({'page':page,'attached':attach_labels(root)})
        color_shapes(root);style_tables(root,page)
        headers=[s for s in E.fromstring(authored_parts[authored_part]).findall('p:cSld/p:spTree/p:sp',F.NS) if name(s).startswith('v6-header-')]
        assert len(headers)==8,(page,len(headers))
        next_id=max(int(n.get('id')) for n in root.iter(P+'cNvPr'))+1
        for s in headers:
            node=copy.deepcopy(s);node.find('p:nvSpPr/p:cNvPr',F.NS).set('id',str(next_id));next_id+=1
            tree.append(node)
        native_cells=[s for s in E.fromstring(authored_parts[authored_part]).findall('p:cSld/p:spTree/p:sp',F.NS)
                      if name(s).startswith('v6-native-header-cell-')]
        assert len(native_cells)==len(info['native_header_cells'])
        for cell in info['native_header_cells']:
            label=next(s for s in tree.findall(P+'sp') if name(s)==cell['label_name'])
            tree.remove(label)
        for cell in native_cells:
            node=copy.deepcopy(cell);node.find('p:nvSpPr/p:cNvPr',F.NS).set('id',str(next_id));next_id+=1
            tree.append(node)
        # Only the design-question label uses red. Its sentence stays black.
        for sp in tree.findall(P+'sp'):
            if text(sp)=='설계 질문':
                for props in sp.iter():
                    if props.tag in W.PROPERTIES:
                        rgb=props.find(A+'solidFill/'+A+'srgbClr')
                        if rgb is not None:rgb.set('val','C00000')
        parts[part]=W.normalize_xml(part,F.xml(root))
    # Language and Calibri defaults also apply to notes, layouts and masters.
    for filename,data in list(parts.items()):
        if filename.endswith('.xml') and filename not in {p for _,p in F.ordered(parts)}:
            root=E.fromstring(data);black_text(root);parts[filename]=W.normalize_xml(filename,F.xml(root))
    repair_masters(parts)
    assert F.ordered(parts)==F.ordered(original)
    for filename,data in original.items():
        if filename.startswith('ppt/media/'):assert parts[filename]==data,filename
        if filename.startswith('ppt/notesSlides/') and filename.endswith('.xml'):
            note_text=lambda value:[n.text for n in E.fromstring(value).iter(A+'t')]
            assert note_text(parts[filename])==note_text(data),filename
    # All fixed QA cell content remains exact, including the preserved appendix.
    for page in (9,10,*range(26,36),41,42):
        part=F.ordered(parts)[page-1][1]
        get=lambda data:[n.text for tbl in E.fromstring(data).iter(A+'tbl') for n in tbl.iter(A+'t')]
        assert get(parts[part])==get(original[part]),page
    assert W.policy_report(parts)['violations']==0
    F.write(Path(output),parts)
    slides=copy.deepcopy(old_manifest['slides'])
    for row in slides:
        root=E.fromstring(parts[row['part']])
        row.update(source_page=row['page'],source_key=row['key'],kind='reference-style-and-native-shape-text',
                   source_slide_sha256=F.digest(original[row['part']]),slide_sha256=F.digest(parts[row['part']]),
                   title=TITLES[row['page']-1],native_tables=len(list(root.iter(A+'tbl'))))
    manifest={'version':6,'source_path':old_manifest['output_path'],'source_sha256':plan['source_sha256'],
              'output_path':'docs/presentations_files/VIA-architecture-presentation_v6.pptx',
              'output_sha256':F.digest(Path(output).read_bytes()),'main_slides':24,'appendix_divider_page':25,
              'preserved_v2_background_pages':[58,59,60,61],'slide_order_ids_and_keys_preserved':True,
              'media_byte_identical':True,'notes_text_preserved':True,'fixed_qa_tables_text_preserved':True,
              'native_header_cell_count':sum(len(p['native_header_cells']) for p in plan['pages']),
              'reference_path':'docs/presentations_files/reference_presentaion/presentation_kjh',
              'reference_kind':'PNG images, rebuilt header and styles as editable objects',
              'master_registry_repairs':['slideMasterId -> sldMasterId','one presentation notesMaster relationship','notes slides use the shared notes master'],
              'authoring_policy':{'target_application':'PowerPoint on Windows','main_text_color':'000000',
                                  'emphasis_colors':['C00000'],'shape_labels':'native shape text, no label overlays',**W.policy_report(parts)},
              'label_attachment_log':merge_log,'slides':slides,'published_date':'2026-10-12'}
    Path(manifest_path).write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'slides':len(slides),'labels_attached':sum(len(x['attached']) for x in merge_log),'sha256':manifest['output_sha256']}))


if __name__=='__main__':
    command,*args=sys.argv[1:]
    if command=='prepare':prepare(*args)
    elif command=='finish':finish(*args)
    else:raise ValueError(command)
