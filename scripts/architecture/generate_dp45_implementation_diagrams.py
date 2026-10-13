"""Generate and check DP45 implementation-design SVG/draw.io pairs."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET
from dp45_implementation_scene import a, b

ROOT=Path(__file__).resolve().parents[2]
DOC=ROOT/'docs/architecture/12-decisions/decision-packages/04-45-memory-and-context.md'
OUT=DOC.parent/'diagrams'

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    scenes=(a(),b())
    for scene in scenes:
        scene.validate()
        for ext,data in [('svg',scene.svg()),('drawio',scene.drawio())]:
            root=ET.fromstring(data)
            if ext=='drawio':
                cells=root.findall('.//mxCell'); ids=[c.get('id') for c in cells]
                assert len(ids)==len(set(ids)), 'duplicate cell ID'
                for c in cells:
                    for k in ('source','target','parent'):
                        assert not c.get(k) or c.get(k) in ids, (scene.slug,k,c.get(k))
            dest=OUT/(scene.slug+'.'+ext)
            if args.check:
                assert dest.exists() and dest.read_text()==data, 'regenerate '+str(dest)
            else:
                dest.write_text(data)
                print(dest.relative_to(ROOT))
    if args.check:
        body=DOC.read_text()
        for scene in scenes:
            assert f'](./diagrams/{scene.slug}.svg)' in body and f'![{scene.side} 구현 설계' in body
            for ext in ('drawio','png'): assert f'](./diagrams/{scene.slug}.{ext})' in body
        for p in OUT.glob('*.svg'):
            if p.name.startswith(('dp45-','choice45-')):
                assert f'](./diagrams/{p.name})' in body, 'missing body preview '+p.name
        manifest=json.loads((OUT/'dp45-implementation-render.json').read_text())
        assert {r['stem'] for r in manifest['records']}=={s.slug for s in scenes}
        for r in manifest['records']:
            assert r['text_geometry_issues']==0
            for ext in ('svg','png'):
                assert hashlib.sha256((OUT/(r['stem']+'.'+ext)).read_bytes()).hexdigest()==r[ext+'_sha256'], 'render drift '+r['stem']
    print('PASS: DP45 ownership, bounds, orthogonal routes, XML and artifact parity'+('; body previews and render hashes' if args.check else ''))

if __name__=='__main__': main()
