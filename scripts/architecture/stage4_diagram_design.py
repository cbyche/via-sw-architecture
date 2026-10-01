"""Architecture review plates: shared geometry for SVG and editable draw.io.

This renderer is deliberately separate from the reviewed target's renderer.
Responsibility boundaries are not deployment boundaries. Text remains editable.
"""
from __future__ import annotations
import html
import xml.etree.ElementTree as ET

W, H = 2560, 1660
INK = '#233247'
MUTED = '#5E6D80'
LINE = '#C9D2DE'
BLUE = '#195FBD'
PALE = '#EDF4FF'
AMBER = '#956016'
FONT = 'Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'


class Plate:
    def __init__(self, slug, number, title, thesis, height=H):
        self.slug, self.title, self.height = slug, title, height
        self.items = []
        self.nodes = {}
        self.serial = 0
        self.box(0, 0, W, 10, fill=INK, stroke='none')
        self.text(64, 45, 'VIA  /  ARCHITECTURE REVIEW', 18, MUTED, bold=True)
        self.text(2496, 45, f'{number}   /   STAGE 04', 18, MUTED, align='right', bold=True)
        self.text(64, 92, title, 42, INK, bold=True)
        self.text(64, 159, thesis, 24, MUTED)
        self.line([(64, 211), (2496, 211)], LINE, arrow=False)

    def _add(self, kind, **data):
        self.serial += 1
        data.update(kind=kind, id=data.pop('id', None) or f'e{self.serial}')
        self.items.append(data)
        return data['id']

    def text(self, x, y, value, size=22, color=INK, align='left', bold=False, width=None, leading=None):
        lines = value.split('\n') if isinstance(value, str) else value
        return self._add('text', x=x, y=y, lines=lines, size=size, color=color,
                         align=align, bold=bold, width=width, leading=leading or size*1.35)

    def box(self, x, y, w, h, fill='white', stroke=LINE, radius=8, dashed=False, thick=1.5, id=None):
        return self._add('box', x=x, y=y, w=w, h=h, fill=fill, stroke=stroke,
                         radius=radius, dashed=dashed, thick=thick, id=id)

    def boundary(self, key, x, y, w, h, title='VIA SOFTWARE', subtitle='논리적 책임 경계', focus=False):
        self.box(x, y, w, h, '#F7F9FC' if not focus else PALE,
                 BLUE if focus else LINE, 10, thick=1.7, id=key)
        self.text(x+24, y+18, title, 18, BLUE if focus else INK, bold=True)
        if subtitle:
            self.text(x+w-24, y+19, subtitle, 16, MUTED, align='right')

    def node(self, key, x, y, w, h, name, kind='component', focus=False, tag=None):
        self.nodes[key] = (x, y, w, h)
        self._add('node', id=key, x=x, y=y, w=w, h=h, name=name.split('\n'),
                  type=kind, focus=focus, tag=tag)
        return key

    def line(self, points, color=INK, dashed=False, arrow=True, width=2, id=None):
        return self._add('line', points=points, color=color, dashed=dashed,
                         arrow=arrow, width=width, id=id)

    def port(self, key, side, delta=0):
        x,y,w,h = self.nodes[key]
        return {'L':(x,y+h/2+delta), 'R':(x+w,y+h/2+delta),
                'T':(x+w/2+delta,y), 'B':(x+w/2+delta,y+h)}[side]

    def edge(self, source, target, sp='B', tp='T', via=(), label='', at=None,
             focus=False, ret=False, sd=0, td=0):
        pts=[self.port(source,sp,sd), *via, self.port(target,tp,td)]
        self.line(pts, BLUE if focus else INK, ret)
        self.items[-1].update(source=source,target=target)
        if label:
            self.label(at[0],at[1],label,BLUE if focus else INK)

    def label(self, x, y, value, color=INK, size=18):
        self._add('label', x=x, y=y, lines=value.split('\n'), size=size, color=color)

    def panel(self, x, w, letter, name, description):
        self.box(x, 238, 48, 48, INK if letter=='T' else BLUE, 'none', 6)
        self.text(x+24, 246, letter, 27, 'white', 'center', True)
        self.text(x+68, 237, name, 27, INK, bold=True)
        self.text(x+68, 276, description, 18, MUTED)

    def focus(self, x, y, w, title, lines, note):
        self.box(x,y,w,220,'white',LINE,8)
        self.box(x,y,5,220,BLUE,'none',0)
        self.text(x+25,y+19,'DESIGN FOCUS  /  '+title,18,BLUE,bold=True)
        self.text(x+25,y+57,lines,22,INK,leading=33)
        self.text(x+25,y+169,note,18,AMBER)

    def footer(self, note):
        y=self.height-132
        self.line([(64,y),(2496,y)],LINE,arrow=False)
        self.text(64,y+24,'범례',17,MUTED,bold=True)
        self.line([(130,y+35),(178,y+35)],INK)
        self.text(190,y+23,'호출 / 전달',17,MUTED)
        self.line([(345,y+35),(393,y+35)],INK,True)
        self.text(405,y+23,'결과 / 복원',17,MUTED)
        self.box(565,y+24,20,20,PALE,BLUE,2)
        self.text(597,y+23,'구조 차이 또는 확대 대상',17,MUTED)
        self.text(1020,y+23,'외부 책임 경계 ≠ 원격 배치  |  모델은 on-device 의존성',17,MUTED)
        self.text(64,y+69,note,18,MUTED)
        self.text(2496,y+70,'설계 탐색  /  미채택 대안  /  미측정',17,MUTED,'right')

    def validate(self):
        ids=[i['id'] for i in self.items]
        assert len(ids)==len(set(ids)), (self.slug,'duplicate ID')
        for i in self.items:
            if i['kind'] in ('box','node'):
                assert 0<=i['x'] and 0<=i['y'] and i['x']+i['w']<=W and i['y']+i['h']<=self.height,(self.slug,i)
            elif i['kind']=='line':
                for a,b in zip(i['points'],i['points'][1:]):
                    assert a[0]==b[0] or a[1]==b[1],(self.slug,i['id'],'non-orthogonal',a,b)
                for x,y in i['points']:
                    assert 0<=x<=W and 0<=y<=self.height,(self.slug,i['id'],'bounds')
                for a,b in zip(i['points'],i['points'][1:]):
                    for key,(x,y,w,h) in self.nodes.items():
                        endpoints=(i['points'][0],i['points'][-1])
                        if any(x<=px<=x+w and y<=py<=y+h for px,py in endpoints):continue
                        vertical=a[0]==b[0] and x+1<a[0]<x+w-1 and max(min(a[1],b[1]),y+1)<min(max(a[1],b[1]),y+h-1)
                        horizontal=a[1]==b[1] and y+1<a[1]<y+h-1 and max(min(a[0],b[0]),x+1)<min(max(a[0],b[0]),x+w-1)
                        assert not (vertical or horizontal),(self.slug,i['id'],'route crosses',key)

    @staticmethod
    def node_type(i):
        return i['tag'] or {'component':'COMPONENT','module':'MODULE','data':'DATA','store':'STORE','model':'MODEL DEPENDENCY','process':'PROCESS'}[i['type']]

    @staticmethod
    def node_text(i):
        size=23 if i['w']>=310 else 20
        leading=size*1.25
        # Center the visible glyph block below the 26 px stereotype strip.
        top=i['y']+26+(i['h']-26-(len(i['name'])-1)*leading-size)/2-4
        return top,size,leading

    def layers(self):
        order={'box':0,'line':1,'node':2,'text':3,'label':3}
        return sorted(self.items,key=lambda i:order[i['kind']])

    def svg(self):
        e=html.escape
        out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{self.height}" viewBox="0 0 {W} {self.height}" role="img" aria-labelledby="title">',
             f'<title id="title">{e(self.title)}</title>',
             '<defs>']
        for color in (INK,BLUE):
            out.append(f'<marker id="a{color[1:]}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="{color}" stroke-width="1.6"/></marker>')
        out += ['</defs>',f'<rect width="{W}" height="{self.height}" fill="white"/>',f'<g font-family="{FONT}">']
        def tx(x,y,lines,size,color,align='left',bold=False,leading=None):
            anchor={'left':'start','center':'middle','right':'end'}[align]
            return ''.join(f'<text x="{x}" y="{y+j*(leading or size*1.35)+size}" text-anchor="{anchor}" font-size="{size}" font-weight="{700 if bold else 400}" fill="{color}">{e(s)}</text>' for j,s in enumerate(lines))
        for i in self.layers():
            k=i['kind']; ident=i['id']
            if k=='box':
                out.append(f'<rect id="{ident}" x="{i["x"]}" y="{i["y"]}" width="{i["w"]}" height="{i["h"]}" rx="{i["radius"]}" fill="{i["fill"]}" stroke="{i["stroke"]}" stroke-width="{i["thick"]}"'+(' stroke-dasharray="7 5"' if i['dashed'] else '')+'/>')
            elif k=='text':
                out.append(f'<g id="{ident}">'+tx(i['x'],i['y'],i['lines'],i['size'],i['color'],i['align'],i['bold'],i['leading'])+'</g>')
            elif k=='line':
                pts=' '.join(f'{x},{y}' for x,y in i['points'])
                out.append(f'<polyline id="{ident}" points="{pts}" fill="none" stroke="{i["color"]}" stroke-width="{i["width"]}" stroke-linejoin="round"'+(' stroke-dasharray="7 5"' if i['dashed'] else '')+(f' marker-end="url(#a{i["color"][1:]})"' if i['arrow'] else '')+'/>')
            elif k=='label':
                # Stroke creates a small opaque label backplate without guessing font widths.
                out.append(f'<g id="{ident}" paint-order="stroke" stroke="white" stroke-width="7" stroke-linejoin="round">'+tx(i['x'],i['y'],i['lines'],i['size'],i['color'])+'</g>')
            elif k=='node':
                x,y,w,h=i['x'],i['y'],i['w'],i['h'];c=BLUE if i['focus'] else INK
                fill=PALE if i['focus'] else 'white'
                out.append(f'<g id="{ident}" data-node="true"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="5" fill="{fill}" stroke="{c}" stroke-width="1.8"/>')
                tag=self.node_type(i)
                out.append(tx(x+14,y+9,[tag],12,c,bold=True))
                if i['type']=='component':
                    out.append(f'<path d="M{x+w-23} {y+10}h10v8h-10z M{x+w-23} {y+23}h10v8h-10z" fill="{fill}" stroke="{c}" stroke-width="1.3"/>')
                elif i['type']=='store':
                    out.append(f'<path d="M{x+w-31} {y+13}c0-6 20-6 20 0v16c0 6-20 6-20 0z M{x+w-31} {y+13}c0 6 20 6 20 0" fill="none" stroke="{c}" stroke-width="1.3"/>')
                top,size,leading=self.node_text(i)
                out.append(tx(x+w/2,top,i['name'],size,c,'center',True,leading))
                out.append('</g>')
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
                st=f'edgeStyle=none;rounded=0;html=1;strokeColor={i["color"]};strokeWidth={i["width"]};endArrow={"open" if i["arrow"] else "none"};endFill=0;'+('dashed=1;dashPattern=7 5;' if i['dashed'] else '')
                connections={}
                if 'source' in i:
                    for role,point,prefix in [('source',i['points'][0],'exit'),('target',i['points'][-1],'entry')]:
                        key=i[role];x,y,w,h=self.nodes[key]
                        connections[role]=key
                        st+=f'{prefix}X={(point[0]-x)/w};{prefix}Y={(point[1]-y)/h};{prefix}Perimeter=0;'
                c=ET.SubElement(r,'mxCell',id=id,style=st,edge='1',parent='1',**connections)
                g=ET.SubElement(c,'mxGeometry',relative='1',attrib={'as':'geometry'})
                for p,key in ((i['points'][0],'sourcePoint'),(i['points'][-1],'targetPoint')):
                    ET.SubElement(g,'mxPoint',x=str(p[0]),y=str(p[1]),attrib={'as':key})
                ar=ET.SubElement(g,'Array',attrib={'as':'points'})
                for x,y in i['points'][1:-1]:ET.SubElement(ar,'mxPoint',x=str(x),y=str(y))
            elif k=='box':
                vertex(id,i['x'],i['y'],i['w'],i['h'],'',f'rounded={int(i["radius"]>0)};arcSize=5;fillColor={i["fill"]};strokeColor={i["stroke"]};strokeWidth={i["thick"]};'+('dashed=1;' if i['dashed'] else ''))
            elif k=='node':
                c=BLUE if i['focus'] else INK; fill=PALE if i['focus'] else 'white'
                vertex(id,i['x'],i['y'],i['w'],i['h'],'',f'rounded=1;arcSize=5;html=1;fillColor={fill};strokeColor={c};strokeWidth=1.8;container=1;collapsible=0;')
                base=f'text;html=1;whiteSpace=nowrap;overflow=visible;strokeColor=none;fillColor=none;fontColor={c};fontFamily=Arial;fontStyle=1;verticalAlign=top;spacing=0;'
                vertex(id+'-tag',14,9,i['w']-28,18,html.escape(self.node_type(i)),base+'fontSize=12;align=left;',id)
                top,size,leading=self.node_text(i)
                for j,line in enumerate(i['name']):vertex(id+f'-name{j}',8,top-i['y']+j*leading,i['w']-16,leading,html.escape(line),base+f'fontSize={size};align=center;',id)
                if i['type']=='component':
                    for j in range(2):vertex(id+f'-icon{j}',i['w']-23,10+j*13,10,8,'',f'fillColor={fill};strokeColor={c};strokeWidth=1.3;',id)
                elif i['type']=='store':
                    vertex(id+'-icon',i['w']-31,9,20,24,'',f'shape=cylinder;fillColor={fill};strokeColor={c};strokeWidth=1.3;',id)
            else:
                size=i['size'];lines=i['lines']; align=i.get('align','left')
                width=i.get('width') or max(sum(size*(1 if ord(c)>=0x2E80 else .62) for c in line) for line in lines)+8
                x=i['x']-(width/2 if align=='center' else width if align=='right' else 0)
                vertex(id,x,i['y'],width,len(lines)*(i.get('leading') or size*1.35)+6,'<br>'.join(html.escape(s) for s in lines),f'text;html=1;whiteSpace=nowrap;overflow=visible;strokeColor=none;fillColor=none;labelBackgroundColor={"white" if k=="label" else "none"};fontColor={i["color"]};fontSize={size};fontFamily=Arial;fontStyle={int(i.get("bold",False))};align={align};verticalAlign=top;spacing=0;')
        return ET.tostring(root,encoding='unicode')+'\n'
