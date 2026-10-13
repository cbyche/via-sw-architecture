"""DP47 detailed documentation scenes derived from the repository diagram notation. Documentation only, not VIA runtime code.

Queue, mutable state, immutable messages, code and long operations have different
symbols. One scene supplies native draw.io geometry and the SVG preview.
"""
from __future__ import annotations

import html
import xml.etree.ElementTree as ET

W, H = 3840, 2160
INK, BLUE, RED, GRAY = '#263445', '#195FBD', '#B63732', '#758294'
PALE, ORANGE = '#FFF3E5', '#B8753F'
FONT = 'Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'


class GroundingScene:
    def __init__(self, slug, title, subtitle):
        self.slug, self.title = slug, title
        self.nodes, self.edges, self.notes = {}, [], []
        self.text(60, 48, 'VIA / DP-47 / 구현 수준 상세 설계', 26, bold=True)
        self.text(3780, 48, '설계 가정 / 제품 미구현 / 모델 호출과 측정 없음', 23, align='right')
        self.text(60, 110, title, 43, bold=True)
        self.text(60, 166, subtitle, 27)

    def text(self, x, y, value, size=24, color=INK, bold=False, align='left', mask=True):
        self.notes.append(dict(x=x, y=y, lines=value.split('\n'), size=size, color=color,
                               bold=bold, align=align, mask=mask))

    def node(self, key, x, y, w, h, title, lines=(), kind='code', owner=None, different=False):
        assert key not in self.nodes
        self.nodes[key] = dict(id=key, x=x, y=y, w=w, h=h, title=title,
                               lines=list(lines), kind=kind, owner=owner, different=different)
        return key

    def port(self, key, side, offset=0):
        n = self.nodes[key]; x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
        return {'L': (x, y+h/2+offset), 'R': (x+w, y+h/2+offset),
                'T': (x+w/2+offset, y), 'B': (x+w/2+offset, y+h)}[side]

    def edge(self, source, target, sp='R', tp='L', via=(), kind='post', sd=0, td=0, label='', at=None):
        points = [self.port(source, sp, sd), *via, self.port(target, tp, td)]
        self.edges.append(dict(id='wire'+str(len(self.edges)+1), source=source, target=target,
                               points=points, kind=kind))
        if label: self.text(*at, label, 21, color={'post':BLUE,'done':BLUE,'cancel':RED,'state':GRAY,'call':INK}[kind], mask=True)

    def ancestors(self, key):
        result = set()
        while self.nodes[key]['owner']:
            key = self.nodes[key]['owner']; result.add(key)
        return result

    def validate(self):
        for key, n in self.nodes.items():
            x,y,w,h = (n[k] for k in ('x','y','w','h'))
            assert 0 <= x and 0 <= y and x+w <= W and y+h <= H, (self.slug,key,'bounds')
            if n['owner']:
                p = self.nodes[n['owner']]
                assert p['x'] <= x and p['y']+48 <= y and x+w <= p['x']+p['w'] and y+h <= p['y']+p['h'], (self.slug,key,'containment')
                assert any(self.nodes[a]['kind']=='component' for a in self.ancestors(key)), (key,'component owner')
            elif n['kind'] not in ('component','trace','external','life'):
                raise AssertionError((key,'missing owner'))
        for e in self.edges:
            exempt=self.ancestors(e['source']) | self.ancestors(e['target'])
            for a,b in zip(e['points'],e['points'][1:]):
                assert a[0]==b[0] or a[1]==b[1], (self.slug,e['id'],'orthogonal',a,b)
                assert 0<=a[0]<=W and 0<=a[1]<=H
                for key,n in self.nodes.items():
                    if key in exempt or n['kind'] in ('component','group','life'): continue
                    x,y,w,h=(n[k] for k in ('x','y','w','h'))
                    v=a[0]==b[0] and x+1<a[0]<x+w-1 and max(min(a[1],b[1]),y+1)<min(max(a[1],b[1]),y+h-1)
                    z=a[1]==b[1] and y+1<a[1]<y+h-1 and max(min(a[0],b[0]),x+1)<min(max(a[0],b[0]),x+w-1)
                    assert not (v or z), (self.slug,e['id'],'crosses node',key)
        assert not any(chr(c) in self.svg() for c in (0xB7,0x2022,0x2027,0x318D,0x30FB,0x22C5,0x2219))
    def texts(self, n):
        x,y,w,h=(n[k] for k in ('x','y','w','h')); k=n['kind']
        if k=='life': return []
        if k in ('component','group'):
            return [(x+18,y+34,n['title'],30 if k=='component' else 27,True,'left')]
        title_size=25 if w<480 else 27
        top=y+(42 if k=='store' else 32)
        result=[(x+w/2,top,n['title'],title_size,True,'center')]
        for j,line in enumerate(n['lines']): result.append((x+18,top+35+j*29,line,23,False,'left'))
        return result

    def svg(self):
        esc=html.escape
        out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img"><title>{esc(self.title)}</title><defs>']
        for c in (INK,BLUE,RED,GRAY):
            out.append(f'<marker id="a{c[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M1 1L9 5L1 9" fill="none" stroke="{c}" stroke-width="1.6"/></marker>')
        out += ['</defs>',f'<rect width="{W}" height="{H}" fill="white"/>',f'<g font-family="{FONT}">']
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'));kind=n['kind']
            fill=PALE if n['different'] else 'white';stroke=ORANGE if n['different'] else INK
            attrs=f'id="{key}" data-kind="{kind}" data-owner="{n["owner"] or ""}"'
            if kind not in ('component','group'):
                attrs+=f' data-node="true" data-x="{x}" data-y="{y}" data-w="{w}" data-h="{h}"'
            out.append('<g '+attrs+'>')
            if kind=='life':
                out.append(f'<path d="M{x+w/2} {y}V{y+h}" fill="none" stroke="{GRAY}" stroke-dasharray="6 6"/>')
            elif kind=='external':
                out.append(f'<polygon points="{x+20},{y} {x+w-20},{y} {x+w},{y+h/2} {x+w-20},{y+h} {x+20},{y+h} {x},{y+h/2}" fill="white" stroke="{stroke}" stroke-width="2"/>')
            elif kind=='store':
                out.append(f'<path d="M{x} {y+13}C{x} {y-3} {x+w} {y-3} {x+w} {y+13}V{y+h-13}C{x+w} {y+h+3} {x} {y+h+3} {x} {y+h-13}Z M{x} {y+13}C{x} {y+29} {x+w} {y+29} {x+w} {y+13}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            elif kind in ('data','trace'):
                out.append(f'<path d="M{x} {y}H{x+w-20}L{x+w} {y+20}V{y+h}H{x}Z M{x+w-20} {y}V{y+20}H{x+w}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>')
            else:
                out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{9 if kind=="code" else 0}" fill="{fill}" stroke="{stroke}" stroke-width="{2.5 if kind=="component" else 2}"/>')
                if kind in ('component','group'):
                    out.append(f'<path d="M{x} {y+48}H{x+w}" stroke="{stroke}" stroke-width="1.5"/>')
                if kind=='queue':
                    start=x+w-190
                    out.append(f'<path d="M{start-20} {y+h-22}H{start+165}" stroke="{BLUE}" marker-end="url(#a{BLUE[1:]})"/>')
                    for j in range(4): out.append(f'<rect x="{start+j*38}" y="{y+h-38}" width="30" height="30" fill="white" stroke="{stroke}"/>')
            for xx,yy,t,size,bold,align in self.texts(n):
                out.append(f'<text x="{xx}" y="{yy}" data-width="{w-36}" font-size="{size}" font-weight="{700 if bold else 400}" text-anchor="{"middle" if align=="center" else "start"}" fill="{INK}">{esc(t)}</text>')
            out.append('</g>')
        for e in self.edges:
            c={'post':BLUE,'done':BLUE,'cancel':RED,'state':GRAY,'call':INK}[e['kind']]
            dash=' stroke-dasharray="9 6"' if e['kind'] in ('done','cancel','state') else ''
            pts=' '.join(f'{x},{y}' for x,y in e['points'])
            out.append(f'<polyline id="{e["id"]}" data-flow="{e["kind"]}" data-source="{e["source"]}" data-target="{e["target"]}" points="{pts}" fill="none" stroke="{c}" stroke-width="2" marker-end="url(#a{c[1:]})"{dash}/>')
        for i,n in enumerate(self.notes):
            attr=' paint-order="stroke" stroke="white" stroke-width="8" stroke-linejoin="round"' if n['mask'] else ''
            out.append(f'<g id="note{i}"{attr}>')
            for j,t in enumerate(n['lines']):
                anchor={'left':'start','right':'end','center':'middle'}[n['align']]
                out.append(f'<text x="{n["x"]}" y="{n["y"]+j*n["size"]*1.3}" font-size="{n["size"]}" font-weight="{700 if n["bold"] else 400}" text-anchor="{anchor}" fill="{n["color"]}">{esc(t)}</text>')
            out.append('</g>')
        return '\n'.join(out+['</g></svg>'])+'\n'

    def drawio(self):
        mx=ET.Element('mxfile',host='app.diagrams.net',type='device')
        d=ET.SubElement(mx,'diagram',id=self.slug,name=self.title)
        g=ET.SubElement(d,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(H))
        root=ET.SubElement(g,'root');ET.SubElement(root,'mxCell',id='0');ET.SubElement(root,'mxCell',id='1',parent='0')
        def vertex(key,x,y,w,h,value,style,parent='1'):
            c=ET.SubElement(root,'mxCell',id=key,parent=parent,vertex='1',value=value,style=style)
            ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        for key,n in self.nodes.items():
            x,y,w,h=(n[k] for k in ('x','y','w','h'));parent=n['owner'] or '1'
            if parent!='1':x-=self.nodes[parent]['x'];y-=self.nodes[parent]['y']
            kind=n['kind'];fill=PALE if n['different'] else 'white';stroke=ORANGE if n['different'] else INK
            shape={'component':'swimlane;startSize=48;collapsible=0;', 'group':'swimlane;startSize=48;collapsible=0;',
                   'store':'shape=cylinder;size=13;', 'data':'shape=note;size=20;', 'trace':'shape=note;size=20;',
                   'life':'shape=line;direction=south;dashed=1;', 'external':'shape=hexagon;', 'queue':'rounded=0;', 'code':'rounded=1;arcSize=10;'}[kind]
            value=html.escape(n['title']) if kind in ('component','group') else ''
            vertex(key,x,y,w,h,value,shape+f'html=1;whiteSpace=wrap;fillColor={fill};swimlaneFillColor={fill};strokeColor={stroke};fontColor={INK};fontFamily=Arial;fontSize={30 if kind=="component" else 27};fontStyle=1;align=left;spacingLeft=18;',parent)
            if kind not in ('component','group'):
                for j,(xx,yy,t,size,bold,align) in enumerate(self.texts(n)):
                    vertex(key+f'-text{j}',18,yy-n['y']-size,w-36,size*1.3,html.escape(t),f'text;html=1;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;fontColor={INK};fontSize={size};fontStyle={int(bold)};align={align};spacing=0;',key)
            if kind=='queue':
                for j in range(4):vertex(key+f'-slot{j}',w-190+j*38,h-38,30,30,'',f'fillColor=white;strokeColor={stroke};',key)
        for e in self.edges:
            c={'post':BLUE,'done':BLUE,'cancel':RED,'state':GRAY,'call':INK}[e['kind']]
            st=f'edgeStyle=none;rounded=0;strokeColor={c};strokeWidth=2;endArrow=open;endFill=0;'+('dashed=1;' if e['kind'] in ('done','cancel','state') else '')
            for role,p,prefix in (('source',e['points'][0],'exit'),('target',e['points'][-1],'entry')):
                n=self.nodes[e[role]];st+=f'{prefix}X={(p[0]-n["x"])/n["w"]};{prefix}Y={(p[1]-n["y"])/n["h"]};{prefix}Perimeter=0;'
            cell=ET.SubElement(root,'mxCell',id=e['id'],source=e['source'],target=e['target'],edge='1',parent='1',style=st)
            geo=ET.SubElement(cell,'mxGeometry',relative='1',attrib={'as':'geometry'})
            arr=ET.SubElement(geo,'Array',attrib={'as':'points'})
            for x,y in e['points'][1:-1]:ET.SubElement(arr,'mxPoint',x=str(x),y=str(y),attrib={'as':'point'})
        for i,n in enumerate(self.notes):
            value='<br>'.join(html.escape(t) for t in n['lines'])
            width=max(len(t)*n['size'] for t in n['lines'])
            if n['align']=='right':x=n['x']-width
            elif n['align']=='center':x=n['x']-width/2
            else:x=n['x']
            vertex('note'+str(i),x,n['y']-n['size'],width,len(n['lines'])*n['size']*1.3,value,f'text;html=1;whiteSpace=nowrap;overflow=visible;fillColor=none;strokeColor=none;labelBackgroundColor={"white" if n["mask"] else "none"};fontColor={n["color"]};fontSize={n["size"]};fontStyle={int(n["bold"])};align={n["align"]};spacing=0;')
        return ET.tostring(mx,encoding='unicode')+'\n'
