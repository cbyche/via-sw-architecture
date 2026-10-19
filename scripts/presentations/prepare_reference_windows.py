"""Apply the repository Windows text policy to an editable reference deck.

The Artifact Tool supplies Calibri text objects. This OOXML pass completes
theme, master, layout, notes and Korean insertion defaults without changing
slide text, object geometry or media.
"""
from pathlib import Path
import argparse
import json
import xml.etree.ElementTree as ET

import full_presentation_package as package
from presentation_windows_policy import normalize_xml, policy_report

A = '{' + package.A + '}'
P = '{' + package.P + '}'


def slide_fingerprint(data):
    root = ET.fromstring(data)
    return {
        'text': [''.join(node.itertext()) for node in root.iter(A + 't')],
        'shapes': len(list(root.iter(P + 'sp'))),
        'tables': len(list(root.iter(A + 'tbl'))),
        'images': len(list(root.iter(P + 'pic'))),
        'positions': [
            (node.get('x'), node.get('y'), node.get('cx'), node.get('cy'))
            for node in root.iter()
            if node.tag in (A + 'off', A + 'ext')
        ],
    }


def prepare(source, output, report):
    source, output, report = map(Path, (source, output, report))
    before = package.read(source)
    after = {
        name: normalize_xml(name, data) if name.endswith('.xml') else data
        for name, data in before.items()
    }
    assert package.ordered(before) == package.ordered(after)
    slides = [part for _, part in package.ordered(before)]
    for part in slides:
        assert slide_fingerprint(before[part]) == slide_fingerprint(after[part]), part
    for name, data in before.items():
        if not name.endswith('.xml'):
            assert after[name] == data, name
    result = policy_report(after)
    output.parent.mkdir(parents=True, exist_ok=True)
    package.write(output, after)
    report.write_text(json.dumps({
        'source': str(source),
        'output': str(output),
        'slide_count': len(slides),
        'editable_slide_objects_preserved': True,
        'slide_text_geometry_media_preserved': True,
        'policy': result,
    }, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(result, ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('source')
    parser.add_argument('output')
    parser.add_argument('report')
    args = parser.parse_args()
    prepare(args.source, args.output, args.report)
