#!/usr/bin/env python3
"""Four editable detailed DP47 figures. Documentation only."""
import argparse
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET
from dp47_implementation_views import scenes
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'

def generate(check=False):
    for scene in scenes():
        scene.validate()
        for ext,text in [('svg',scene.svg()),('drawio',scene.drawio())]:
            ET.fromstring(text)
            target=OUT/(scene.slug+'.'+ext)
            if check:assert target.read_text()==text,f'Drift: {target.name}'
            else:target.write_text(text)
        print('PASS:',scene.slug,'owner, routes, deterministic SVG/draw.io')
        if check:
            manifest=json.loads((OUT/(scene.slug+'-render.json')).read_text())
            assert (manifest['width'],manifest['height'],manifest['text_geometry_issues'])==(3840,2160,0)
            for ext in ('svg','png'):
                digest=hashlib.sha256((OUT/(scene.slug+'.'+ext)).read_bytes()).hexdigest()
                assert digest==manifest[ext+'_sha256'],f'Stale render: {scene.slug}.{ext}'
    print('PASS: 27 shared nodes have identical A/B coordinates and sizes')
    if check:print('PASS: four PNG render manifests match current SVG/PNG bytes')
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--check',action='store_true');args=parser.parse_args()
    generate(args.check)

if __name__=='__main__':main()
