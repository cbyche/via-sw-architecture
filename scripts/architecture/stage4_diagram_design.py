"""Vector architecture notation shared by SVG and editable draw.io.

Components are responsibility containers; modules have an explicit component owner.
Colors encode option membership, never architectural rank or measured superiority.
"""
from __future__ import annotations
import html
import xml.etree.ElementTree as ET

W, H = 2560, 2200
INK = '#263445'
MUTED = '#5B697B'
LINE = '#C7D1DD'
BLUE = '#195FBD'
GREEN = '#087A53'
PALE = '#EEF4FF'
MINT = '#EDF9F2'
AMBER = '#895914'
FONT = 'Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'


def tint(color):
    return PALE if color == BLUE else MINT if color == GREEN else '#F7F9FC'


class Plate:
    def __init__(self, slug, number, title, thesis, height=H):
        self.slug, self.title, self.height = slug, title, height
        self.items, self.nodes, self.meta = [], {}, {}
        self.serial = 0
        self.box(0, 0, W, 9, INK, 'none', 0)
        self.text(64, 42, 'VIA  /  ARCHITECTURE REVIEW', 18, MUTED, bold=True)
        self.text(2496, 42, number + '  /  STAGE 04', 18, MUTED, align='right', bold=True)
        self.text(64, 87, title, 42, bold=True)
        self.text(64, 151, thesis, 23, MUTED)
        self.line([(64, 202), (2496, 202)], LINE, arrow=False)

    def _add(self, kind, **kw):
        self.serial += 1
        kw.update(kind=kind, id=kw.pop('id', None) or f'e{self.serial}')
        self.items.append(kw)
        return kw['id']

    def text(self, x, y, value, size=22, color=INK, align='left', bold=False, width=None, leading=None):
        return self._add('text', x=x, y=y, lines=value.split('\n') if isinstance(value, str) else value,
                         size=size, color=color, align=align, bold=bold, width=width, leading=leading or size*1.35)

    def box(self, x, y, w, h, fill='white', stroke=LINE, radius=6, dashed=False, thick=1.5, id=None):
        return self._add('box', x=x, y=y, w=w, h=h, fill=fill, stroke=stroke,
                         radius=radius, dashed=dashed, thick=thick, id=id)

    def boundary(self, key, x, y, w, h, title='VIA SOFTWARE', subtitle='비교에 필요한 책임만 표시', color=INK, dashed=False):
        self.box(x, y, w, h, '#FAFBFD', LINE if color==INK else color, 10, dashed=dashed, id=key)
        self.text(x+22, y+16, title, 18, color, bold=True)
        if subtitle:self.text(x+w-22, y+18, subtitle, 16, MUTED, align='right')

    def component(self, key, x, y, w, h, name, color=INK, role='', parent=None):
        self.nodes[key]=(x,y,w,h)
        self.meta[key]={'kind':'component','parent':parent}
        self._add('component', id=key,x=x,y=y,w=w,h=h,name=name.split('\n'),color=color,role=role,parent=parent)
        return key

    def node(self, key, x, y, w, h, name, kind='module', color=INK, owner=None, focus=False, tag=None):
        # All internal implementation modules must name their owning Component.
        if kind=='module':assert owner, (self.slug,key,'module without owning Component')
        self.nodes[key]=(x,y,w,h)
        self.meta[key]={'kind':kind,'parent':owner}
        self._add('node',id=key,x=x,y=y,w=w,h=h,name=name.split('\n'),type=kind,color=color,parent=owner)
        return key

    def port(self, key, side, delta=0):
        x,y,w,h=self.nodes[key]
        return {'L':(x,y+h/2+delta),'R':(x+w,y+h/2+delta),'T':(x+w/2+delta,y),'B':(x+w/2+delta,y+h)}[side]

    def line(self, points, color=INK, dashed=False, arrow=True, width=2, id=None):
        return self._add('line',points=points,color=color,dashed=dashed,arrow=arrow,width=width,id=id)

    def edge(self, source, target, sp='B', tp='T', via=(), label='', at=None, color=INK, ret=False, sd=0, td=0):
        self.line([self.port(source,sp,sd),*via,self.port(target,tp,td)],color,ret)
        self.items[-1].update(source=source,target=target)
        if label:self.label(*at,label,color)

    def label(self, x, y, value, color=INK, size=18):
        self._add('label',x=x,y=y,lines=value.split('\n'),size=size,color=color)

    def panel(self, x, w, letter, name, description, y=237):
        c=BLUE if letter=='T' else GREEN
        self.box(x,y,48,48,c,'none',5)
        self.text(x+24,y+7,letter,27,'white','center',True)
        self.text(x+68,y-1,name,28,bold=True)
        self.text(x+68,y+39,description,18,MUTED)

    def focus(self, x, y, w, title, lines, note, color=INK):
        self.box(x,y,w,212,'white',LINE)
        self.box(x,y,5,212,color,'none',0)
        self.text(x+24,y+17,title,21,color,bold=True)
        self.text(x+24,y+58,lines,21,leading=31)
        self.text(x+24,y+164,note,18,AMBER)

    def footer(self, note):
        y=self.height-146
        self.line([(64,y),(2496,y)],LINE,arrow=False)
        self.text(64,y+18,'색:  검정 = 공통     파랑 = T 전용     초록 = A/B 전용',19,bold=True)
        self.text(2496,y+18,'실선 → 요청/전달     점선 → 결과/복원',18,MUTED,'right')
        self.text(64,y+56,'모양:  제목 칸이 있는 박스 = Component   |   내부 둥근 박스 = Module   |   원통 = 저장소   |   접힌 종이 = 자료   |   육각형 = 연동 대상/모델',17,MUTED)
        self.text(64,y+96,note,17,MUTED)
        self.text(2496,y+96,'탐색안 / 미채택 / 미측정',17,MUTED,'right')

    def ancestors(self, key):
        out=set()
        while self.meta[key]['parent']:
            key=self.meta[key]['parent'];out.add(key)
        return out

    def validate(self):
        ids=[i['id'] for i in self.items]
        assert len(ids)==len(set(ids)),(self.slug,'duplicate ID')
        for key,meta in self.meta.items():
            if meta['parent']:
                owner=meta['parent'];assert owner in self.nodes,(self.slug,key,owner)
                x,y,w,h=self.nodes[key];ox,oy,ow,oh=self.nodes[owner]
                assert ox<=x and oy+62<=y and x+w<=ox+ow and y+h<=oy+oh,(self.slug,key,'outside owner',owner)
                if meta['kind']=='module':assert self.meta[owner]['kind']=='component'
        for i in self.items:
            if i['kind'] in ('box','component','node'):
                assert 0<=i['x'] and 0<=i['y'] and i['x']+i['w']<=W and i['y']+i['h']<=self.height,(self.slug,i['id'],'bounds')
            if i['kind']!='line':continue
            for a,b in zip(i['points'],i['points'][1:]):
                assert a[0]==b[0] or a[1]==b[1],(self.slug,i['id'],'non-orthogonal',a,b)
                assert 0<=a[0]<=W and 0<=a[1]<=self.height,(self.slug,i['id'],'bounds')
                exempt=set()
                for role in ('source','target'):
                    if role in i:exempt|={i[role]}|self.ancestors(i[role])
                for key,(x,y,w,h) in self.nodes.items():
                    if key in exempt:continue
                    if any(x<=px<=x+w and y<=py<=y+h for px,py in (i['points'][0],i['points'][-1])):continue
                    v=a[0]==b[0] and x+1<a[0]<x+w-1 and max(min(a[1],b[1]),y+1)<min(max(a[1],b[1]),y+h-1)
                    z=a[1]==b[1] and y+1<a[1]<y+h-1 and max(min(a[0],b[0]),x+1)<min(max(a[0],b[0]),x+w-1)
                    assert not (v or z),(self.slug,i['id'],'route crosses',key)

    def layers(self):
        order={'box':0,'component':1,'line':2,'node':3,'text':4,'label':4}
        def layer(i):
            if i['kind']=='box' and any(n['kind']=='component' and n['x']<=i['x'] and n['y']+62<=i['y'] and i['x']+i['w']<=n['x']+n['w'] and i['y']+i['h']<=n['y']+n['h'] for n in self.items):return 1.5
            return order[i['kind']]
        return sorted(self.items,key=layer)

    @staticmethod
    def text_geometry(i):
        size=22 if i['w']>=290 else 19
        leading=size*1.25
        return i['y']+(i['h']-len(i['name'])*leading)/2-2+(10 if i['type']=='store' else 0),size,leading

    def svg(self):
        esc=html.escape
        out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{self.height}" viewBox="0 0 {W} {self.height}" role="img" aria-labelledby="title"><title id="title">{esc(self.title)}</title><defs>']
        for c in (INK,BLUE,GREEN):out.append(f'<marker id="a{c[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="{c}" stroke-width="1.6"/></marker>')
        out+=['</defs>',f'<rect width="{W}" height="{self.height}" fill="white"/><g font-family="{FONT}">']
        def tx(x,y,lines,size,c,align='left',bold=False,leading=None):
            anchor={'left':'start','center':'middle','right':'end'}[align]
            return ''.join(f'<text x="{x}" y="{y+j*(leading or size*1.35)+size}" text-anchor="{anchor}" font-size="{size}" font-weight="{700 if bold else 400}" fill="{c}">{esc(s)}</text>' for j,s in enumerate(lines))
        for i in self.layers():
            k=i['kind'];ident=i['id']
            if k=='box':out.append(f'<rect id="{ident}" x="{i["x"]}" y="{i["y"]}" width="{i["w"]}" height="{i["h"]}" rx="{i["radius"]}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="{i["thick"]}"'+(' stroke-dasharray="8 6"' if i['dashed'] else '')+'/>')
            elif k=='component':
                x,y,w,h,c=i['x'],i['y'],i['w'],i['h'],i['color']
                out.append(f'<g id="{ident}" data-component="true"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="white" stroke="{c}" stroke-width="2.2"/>')
                out.append(f'<path d="M{x} {y+62}H{x+w}" stroke="{c}" stroke-width="1.4"/>')
                out.append(tx(x+18,y+15,i['name'],17 if w<250 else 23,c,bold=True))
                if i['role']:out.append(tx(x+18,y+76,[i['role']],17,MUTED))
                out.append('</g>')
            elif k=='node':
                x,y,w,h,c=i['x'],i['y'],i['w'],i['h'],i['color'];fill=tint(c);typ=i['type']
                out.append(f'<g id="{ident}" data-node="true" data-x="{x}" data-y="{y}" data-w="{w}" data-h="{h}">')
                if typ=='store':
                    out.append(f'<path d="M{x} {y+14}C{x} {y-5} {x+w} {y-5} {x+w} {y+14}V{y+h-14}C{x+w} {y+h+5} {x} {y+h+5} {x} {y+h-14}Z M{x} {y+14}C{x} {y+33} {x+w} {y+33} {x+w} {y+14}" fill="{fill}" stroke="{c}" stroke-width="1.7"/>')
                elif typ=='data':
                    out.append(f'<path d="M{x} {y}H{x+w-20}L{x+w} {y+20}V{y+h}H{x}Z M{x+w-20} {y}V{y+20}H{x+w}" fill="{fill}" stroke="{c}" stroke-width="1.7"/>')
                elif typ in ('external','model'):
                    out.append(f'<path d="M{x+24} {y}H{x+w-24}L{x+w} {y+h/2}L{x+w-24} {y+h}H{x+24}L{x} {y+h/2}Z" fill="{fill}" stroke="{c}" stroke-width="1.7"/>')
                else:out.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="12" fill="{fill}" stroke="{c}" stroke-width="1.6"/>')
                top,size,leading=self.text_geometry(i)
                out.append(tx(x+w/2,top,i['name'],size,c,'center',True,leading));out.append('</g>')
            elif k=='line':
                pts=' '.join(f'{x},{y}' for x,y in i['points'])
                out.append(f'<polyline id="{ident}" points="{pts}" fill="none" stroke="{i["color"]}" stroke-width="{i["width"]}"'+(' stroke-dasharray="7 5"' if i['dashed'] else '')+(f' marker-end="url(#a{i["color"][1:]})"' if i['arrow'] else '')+'/>')
            else:
                out.append(f'<g id="{ident}"'+(' paint-order="stroke" stroke="white" stroke-width="7" stroke-linejoin="round"' if k=='label' else '')+'>'+tx(i['x'],i['y'],i['lines'],i['size'],i['color'],i.get('align','left'),i.get('bold',False),i.get('leading'))+'</g>')
        return '\n'.join(out+['</g></svg>'])+'\n'

    def drawio(self):
        root=ET.Element('mxfile',host='app.diagrams.net',type='device')
        d=ET.SubElement(root,'diagram',id=self.slug,name=self.title)
        m=ET.SubElement(d,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(self.height),grid='1',gridSize='8')
        r=ET.SubElement(m,'root');ET.SubElement(r,'mxCell',id='0');ET.SubElement(r,'mxCell',id='1',parent='0')
        def vertex(id,x,y,w,h,value,style,parent='1'):
            c=ET.SubElement(r,'mxCell',id=id,value=value,style=style,vertex='1',parent=parent)
            ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
        for i in self.layers():
            k=i['kind'];id=i['id']
            if k=='line':
                st=f'edgeStyle=none;rounded=0;strokeColor={i["color"]};strokeWidth={i["width"]};endArrow={"open" if i["arrow"] else "none"};endFill=0;'+('dashed=1;' if i['dashed'] else '')
                con={}
                if 'source' in i:
                    for role,point,prefix in [('source',i['points'][0],'exit'),('target',i['points'][-1],'entry')]:
                        key=i[role];x,y,w,h=self.nodes[key];con[role]=key
                        st+=f'{prefix}X={(point[0]-x)/w};{prefix}Y={(point[1]-y)/h};{prefix}Perimeter=0;'
                c=ET.SubElement(r,'mxCell',id=id,style=st,edge='1',parent='1',**con)
                g=ET.SubElement(c,'mxGeometry',relative='1',attrib={'as':'geometry'})
                for pt,key in ((i['points'][0],'sourcePoint'),(i['points'][-1],'targetPoint')):ET.SubElement(g,'mxPoint',x=str(pt[0]),y=str(pt[1]),attrib={'as':key})
                ar=ET.SubElement(g,'Array',attrib={'as':'points'})
                for x,y in i['points'][1:-1]:ET.SubElement(ar,'mxPoint',x=str(x),y=str(y))
            elif k=='box':vertex(id,i['x'],i['y'],i['w'],i['h'],'',f'rounded=1;arcSize=4;fillColor={i["fill"]};strokeColor={i["stroke"]};'+('dashed=1;' if i['dashed'] else ''))
            elif k in ('node','component'):
                c=i['color'];parent=i.get('parent') or '1';x,y=i['x'],i['y']
                if parent!='1':px,py,_,_=self.nodes[parent];x-=px;y-=py
                if k=='component':
                    value='<br>'.join(html.escape(n) for n in i['name']);style=f'swimlane;html=1;startSize=62;horizontal=1;align=left;spacingLeft=18;fontSize={17 if i["w"]<250 else 23};fontStyle=1;fillColor=white;swimlaneFillColor=white;strokeColor={c};fontColor={c};strokeWidth=2.2;collapsible=0;'
                    vertex(id,x,y,i['w'],i['h'],value,style,parent)
                    if i['role']:vertex(id+'-role',18,76,i['w']-36,25,html.escape(i['role']),f'text;html=1;strokeColor=none;fillColor=none;fontSize=17;fontColor={MUTED};align=left;spacing=0;',id)
                else:
                    shape={'module':'rounded=1;arcSize=15;','store':'shape=cylinder;size=14;','data':'shape=note;size=20;','external':'shape=hexagon;','model':'shape=hexagon;'}[i['type']]
                    top,size,leading=self.text_geometry(i)
                    vertex(id,x,y,i['w'],i['h'],'<br>'.join(html.escape(s) for s in i['name']),shape+f'html=1;whiteSpace=wrap;align=center;verticalAlign=middle;fontSize={size};fontStyle=1;fillColor={tint(c)};strokeColor={c};fontColor={c};strokeWidth=1.7;',parent)
            else:
                size=i['size'];lines=i['lines'];align=i.get('align','left')
                width=i.get('width') or max(sum(100 if ord(c)>=0x2E80 else 62 for c in line)*size/100 for line in lines)+8
                x=i['x']-(width/2 if align=='center' else width if align=='right' else 0)
                vertex(id,x,i['y'],width,len(lines)*(i.get('leading') or size*1.35)+6,'<br>'.join(html.escape(s) for s in lines),f'text;html=1;whiteSpace=nowrap;overflow=visible;strokeColor=none;fillColor=none;labelBackgroundColor={"white" if k=="label" else "none"};fontColor={i["color"]};fontSize={size};fontStyle={int(i.get("bold",False))};align={align};verticalAlign=top;spacing=0;')
        return ET.tostring(root,encoding='unicode')+'\n'
