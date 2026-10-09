"""46 documentation-only SVG/draw.io serializer, independent of all 45/41 scenes."""
import html
import xml.etree.ElementTree as ET
W, H = 2560, 1440
FONT = "Apple SD Gothic Neo, Arial, sans-serif"


class Scene:
    def __init__(self, title, slug):
        self.title = title
        self.slug = slug
        self.nodes = []
        self.edges = []
        self.texts = []

    def node(self, ident, x, y, w, h, name, kind='component', owner=None, different=False):
        self.nodes.append(dict(id=ident, x=x, y=y, w=w, h=h, name=name,
                               kind=kind, owner=owner, different=different))

    def text(self, x, y, w, lines, size=22, bold=False):
        self.texts.append(dict(x=x, y=y, w=w, lines=lines, size=size, bold=bold))

    def edge(self, source, target, points, both=False, dashed=False):
        self.edges.append(dict(source=source, target=target, points=points, both=both, dashed=dashed))

    def render(self):
        byid = {n['id']: n for n in self.nodes}
        assert len(byid) == len(self.nodes)
        containers = {n['owner'] for n in self.nodes if n['owner']}
        svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title">',
               f'<title id="title">{html.escape(self.title)}</title>',
               '<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1L9 5L1 9" fill="none" stroke="#333" stroke-width="1.2"/></marker></defs>',
               f'<rect width="{W}" height="{H}" fill="white"/>', f'<g font-family="{FONT}" fill="#111">']
        mx = ET.Element('mxfile', host='app.diagrams.net', type='device')
        d = ET.SubElement(mx, 'diagram', id=self.slug, name=self.title)
        model = ET.SubElement(d, 'mxGraphModel', page='1', pageWidth=str(W), pageHeight=str(H), background='#FFFFFF')
        root = ET.SubElement(model, 'root')
        ET.SubElement(root, 'mxCell', id='0')
        ET.SubElement(root, 'mxCell', id='1', parent='0')

        def svg_text(x, y, width, lines, size, bold=False, anchor='start'):
            for k, line in enumerate(lines):
                svg.append(f'<text x="{x}" y="{y+size+k*size*1.3}" data-width="{width}" font-size="{size}" font-weight="{700 if bold else 400}" text-anchor="{anchor}">{html.escape(line)}</text>')

        for n in self.nodes:
            x, y, w, h = (n[k] for k in ('x', 'y', 'w', 'h'))
            assert x >= 0 and y >= 0 and x+w <= W and y+h <= H
            parent = n['owner'] or '1'
            if parent != '1':
                p = byid[parent]
                assert p['kind'] in ('component', 'module', 'boundary')
                assert p['x'] <= x and p['y'] <= y and x+w <= p['x']+p['w'] and y+h <= p['y']+p['h']
            fill = '#FCE4D6' if n['different'] else '#FFFFFF'
            stroke = '#9B502B' if n['different'] else '#222222'
            sw = 2.4 if n['different'] else 1.6
            kind = n['kind']
            dash = ' stroke-dasharray="9 6"' if kind == 'boundary' else ''
            style = f'html=1;fillColor={fill};strokeColor={stroke};strokeWidth={sw};fontFamily=Apple SD Gothic Neo;fontColor=#111111;'
            if kind == 'lifeline':
                svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="85" fill="white" stroke="{stroke}" stroke-width="{sw}"/>')
                svg.append(f'<path d="M{x+w/2},{y+85}V{y+h}" fill="none" stroke="#777" stroke-dasharray="7 6"/>')
                style += 'shape=umlLifeline;size=85;'
            elif kind == 'external':
                svg.append(f'<polygon points="{x+22},{y} {x+w-22},{y} {x+w},{y+h/2} {x+w-22},{y+h} {x+22},{y+h} {x},{y+h/2}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>')
                style += 'shape=hexagon;'
            elif kind == 'data':
                svg.append(f'<path d="M{x},{y}H{x+w-18}L{x+w},{y+18}V{y+h}H{x}Z M{x+w-18},{y}V{y+18}H{x+w}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>')
                style += 'shape=note;size=18;'
            elif kind == 'state':
                r = 9
                svg.append(f'<path d="M{x},{y+r}C{x},{y-r} {x+w},{y-r} {x+w},{y+r}V{y+h-r}C{x+w},{y+h+r} {x},{y+h+r} {x},{y+h-r}Z M{x},{y+r}C{x},{y+3*r} {x+w},{y+3*r} {x+w},{y+r}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>')
                style += 'shape=cylinder;size=10;'
            else:
                svg.append(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{9 if kind=="module" else 0}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"{dash}/>')
                style += f'rounded={int(kind=="module")};arcSize=10;'
                if kind == 'boundary':
                    style += 'dashed=1;'
            lines = n['name'].split('\n')
            top = n['id'] in containers or kind in ('component', 'boundary')
            size = 25 if kind == 'component' else 21
            if kind == 'lifeline':
                svg_text(x+w/2, y+12, w-24, lines, 25, True, 'middle')
            elif top:
                svg_text(x+15, y+8, w-30, lines, size, kind=='component')
            else:
                svg_text(x+w/2, y+h/2-len(lines)*size*0.65+(8 if kind=='state' else 0), w-24, lines, size, False, 'middle')
            if kind == 'lifeline':
                style += 'fontSize=25;fontStyle=1;align=center;verticalAlign=top;spacingTop=12;'
            else:
                style += f'fontSize={size};fontStyle={int(kind=="component")};align={"left" if top else "center"};verticalAlign={"top" if top else "middle"};spacingLeft={15 if top else 0};spacingTop={8 if top else 0};'
            cell = ET.SubElement(root, 'mxCell', id=n['id'], value='<br>'.join(html.escape(s) for s in lines), parent=parent, vertex='1', style=style)
            if parent != '1':
                p = byid[parent]
                x, y = x-p['x'], y-p['y']
            ET.SubElement(cell, 'mxGeometry', x=str(x), y=str(y), width=str(w), height=str(h), attrib={'as':'geometry'})
        for k, e in enumerate(self.edges):
            points = e['points']
            assert e['source'] in byid and e['target'] in byid
            allowed = {e['source'], e['target']}
            for nid in (e['source'], e['target']):
                parent = byid[nid]['owner']
                while parent:
                    allowed.add(parent)
                    parent = byid[parent]['owner']
            for nid, point in [(e['source'], points[0]), (e['target'], points[-1])]:
                n = byid[nid]
                px, py = point
                assert n['x'] <= px <= n['x']+n['w'] and n['y'] <= py <= n['y']+n['h'], (nid, point)
                assert n['kind'] == 'lifeline' or px in (n['x'], n['x']+n['w']) or py in (n['y'], n['y']+n['h']), (nid, point)
            for a, b in zip(points, points[1:]):
                assert a[0] == b[0] or a[1] == b[1]
                for n in self.nodes:
                    if n['id'] in allowed or n['kind'] == 'lifeline':
                        continue
                    x, y, w, h = (n[q] for q in ('x', 'y', 'w', 'h'))
                    crossing = (
                        a[0] == b[0] and x < a[0] < x+w
                        and max(min(a[1], b[1]), y) < min(max(a[1], b[1]), y+h)
                    ) or (
                        a[1] == b[1] and y < a[1] < y+h
                        and max(min(a[0], b[0]), x) < min(max(a[0], b[0]), x+w)
                    )
                    assert not crossing, (e['source'], e['target'], n['id'], a, b)
            svg.append(f'<polyline points="{" ".join(f"{x},{y}" for x,y in points)}" fill="none" stroke="#333" stroke-width="1.6" marker-end="url(#arrow)"'+(' marker-start="url(#arrow)"' if e['both'] else '')+(' stroke-dasharray="7 5"' if e['dashed'] else '')+'/>')
            # Explicit normalized terminals keep editable draw.io edges on the same ports.
            s, t = byid[e['source']], byid[e['target']]
            sx, sy = points[0]
            tx, ty = points[-1]
            style = f'edgeStyle=none;rounded=0;strokeColor=#333333;strokeWidth=1.6;endArrow=open;endFill=0;exitX={(sx-s["x"])/s["w"]};exitY={(sy-s["y"])/s["h"]};entryX={(tx-t["x"])/t["w"]};entryY={(ty-t["y"])/t["h"]};exitPerimeter=0;entryPerimeter=0;'
            if e['both']:
                style += 'startArrow=open;startFill=0;'
            if e['dashed']:
                style += 'dashed=1;dashPattern=7 5;'
            c = ET.SubElement(root, 'mxCell', id=f'e{k}', edge='1', parent='1', source=e['source'], target=e['target'], style=style)
            g = ET.SubElement(c, 'mxGeometry', relative='1', attrib={'as':'geometry'})
            a = ET.SubElement(g, 'Array', attrib={'as':'points'})
            for x, y in points[1:-1]:
                ET.SubElement(a, 'mxPoint', x=str(x), y=str(y))
        for k, t in enumerate(self.texts):
            if t.get('erase', True):
                svg.append(f'<rect x="{t["x"]-3}" y="{t["y"]}" width="{t["w"]+6}" height="{len(t["lines"])*t["size"]*1.3}" fill="white"/>')
            svg_text(t['x'], t['y'], t['w'], t['lines'], t['size'], t['bold'])
            fill = '#FFFFFF' if t.get('erase', True) else 'none'
            c = ET.SubElement(root, 'mxCell', id=f't{k}', vertex='1', parent='1', value='<br>'.join(html.escape(s) for s in t['lines']), style=f'text;html=1;align=left;verticalAlign=top;spacing=0;whiteSpace=nowrap;overflow=visible;fillColor={fill};strokeColor=none;fontSize={t["size"]};fontStyle={int(t["bold"])};fontFamily=Apple SD Gothic Neo;')
            ET.SubElement(c, 'mxGeometry', x=str(t['x']), y=str(t['y']), width=str(t['w']), height=str(len(t['lines'])*t['size']*1.3), attrib={'as':'geometry'})
        svg += ['</g></svg>']
        values = {'svg':'\n'.join(svg)+'\n', 'drawio':ET.tostring(mx, encoding='unicode')+'\n'}
        for v in values.values():
            ET.fromstring(v)
        return values
