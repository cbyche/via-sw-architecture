"""Apply/check the user's Windows PPTX typography and Korean language policy.

Artifact Tool authors the font revision. This scoped OOXML pass preserves the
original editable objects and media, and also covers defaults omitted by the
authoring API: masters, layouts, notes, themes and new-text language.
"""
from pathlib import Path
from collections import Counter
import argparse
import copy
import json
import re
import xml.etree.ElementTree as E

import full_presentation_package as F

FONT = 'Calibri'
LANG = 'ko-KR'
A = '{' + F.A + '}'
P = '{' + F.P + '}'
DC = '{http://purl.org/dc/elements/1.1/}'
PROPERTIES = {A + n for n in ('rPr', 'defRPr', 'endParaRPr')}
FONT_TAGS = {A + n for n in ('latin', 'ea', 'cs', 'sym', 'buFont', 'font')}


def repair_qa_identifiers(root):
    """Keep the twelve 19pt QA IDs on one line after the Calibri revision."""
    count = 0
    for table in root.iter(A + 'tbl'):
        widths = [int(n.get('w')) for n in table.findall(A + 'tblGrid/' + A + 'gridCol')]
        for row in table.findall(A + 'tr'):
            for column, cell in enumerate(row.findall(A + 'tc')):
                text = ''.join(n.text or '' for n in cell.iter(A + 't'))
                if widths[column] != 1476375 or re.fullmatch(r'ASR-QA-\d{2}', text) is None:
                    continue
                changed = False
                for props in cell.iter():
                    if props.tag in PROPERTIES and props.get('sz') == '1900':
                        props.set('sz', '1700')
                        changed = True
                count += int(changed)
    return count


def character_defaults(node):
    node.set('lang', LANG)
    node.attrib.pop('altLang', None)
    for name in ('latin', 'ea', 'cs'):
        font = node.find(A + name)
        if font is None:
            font = E.Element(A + name)
            # Character font elements precede hyperlink/extension elements.
            index = next((i for i, child in enumerate(node)
                          if child.tag in {A + n for n in ('hlinkClick', 'hlinkMouseOver', 'rtl', 'extLst')}), len(node))
            node.insert(index, font)
        font.set('typeface', FONT)


def normalize_xml(name, data):
    root = E.fromstring(data)
    repair_qa_identifiers(root)
    if name == 'ppt/presentation.xml':
        defaults = root.find(P + 'defaultTextStyle')
        if defaults is None:
            defaults = E.Element(P + 'defaultTextStyle')
            # defaultTextStyle is before modifyVerifier/extLst in CT_Presentation.
            index = next((i for i, child in enumerate(root)
                          if child.tag in (P + 'modifyVerifier', P + 'extLst')), len(root))
            root.insert(index, defaults)
        for level in ('defPPr', *(f'lvl{i}pPr' for i in range(1, 10))):
            props = defaults.find(A + level)
            if props is None:
                props = E.SubElement(defaults, A + level)
            run = props.find(A + 'defRPr')
            if run is None:
                run = E.SubElement(props, A + 'defRPr')
            character_defaults(run)
    if name == 'docProps/core.xml':
        language = root.find(DC + 'language')
        if language is None:
            language = E.SubElement(root, DC + 'language')
        language.text = LANG
    for node in list(root.iter()):
        if node.tag in (A + 'r', A + 'fld', A + 'br'):
            props = node.find(A + 'rPr')
            if props is None:
                props = E.Element(A + 'rPr')
                node.insert(0, props)
            character_defaults(props)
        if node.tag == A + 'p':
            props = node.find(A + 'pPr')
            if props is None:
                props = E.Element(A + 'pPr')
                node.insert(0, props)
            default = props.find(A + 'defRPr')
            if default is None:
                default = E.SubElement(props, A + 'defRPr')
            character_defaults(default)
            end = node.find(A + 'endParaRPr')
            if end is None:
                end = E.SubElement(node, A + 'endParaRPr')
            character_defaults(end)
        if node.tag in PROPERTIES:
            character_defaults(node)
            if node.get('sz') is not None:
                size = int(node.get('sz'))
                node.set('sz', str(max(100, ((size + 50) // 100) * 100)))
        if node.tag == A + 'buSzPts':
            node.set('val', str(max(100, ((int(node.get('val')) + 50) // 100) * 100)))
        if node.tag in FONT_TAGS and 'typeface' in node.attrib:
            node.set('typeface', FONT)
    return F.xml(root)


def policy_report(parts):
    counts = Counter()
    errors = []
    for name, data in parts.items():
        if not name.endswith('.xml'):
            continue
        root = E.fromstring(data)
        for node in root.iter():
            if 'typeface' in node.attrib:
                counts['font_references'] += 1
                if node.get('typeface') != FONT:
                    errors.append((name, 'font', node.get('typeface')))
            if node.tag in PROPERTIES:
                counts['text_language_properties'] += 1
                if node.get('lang') != LANG or node.get('altLang') not in (None, LANG):
                    errors.append((name, 'language', node.attrib))
                for face in ('latin', 'ea', 'cs'):
                    if node.find(A + face) is None or node.find(A + face).get('typeface') != FONT:
                        errors.append((name, 'script_font', face))
                if node.get('sz') is not None:
                    counts['font_size_properties'] += 1
                    if int(node.get('sz')) <= 0 or int(node.get('sz')) % 100:
                        errors.append((name, 'fractional_size', node.get('sz')))
            if node.tag == A + 'buSzPts':
                counts['absolute_bullet_size_properties'] += 1
                if int(node.get('val')) <= 0 or int(node.get('val')) % 100:
                    errors.append((name, 'fractional_bullet_size', node.get('val')))
            if node.tag in (A + 'r', A + 'fld', A + 'br') and node.find(A + 'rPr') is None:
                errors.append((name, 'missing_run_language', node.tag))
            if node.tag == A + 'p' and (node.find(A + 'pPr/' + A + 'defRPr') is None
                                      or node.find(A + 'endParaRPr') is None):
                errors.append((name, 'missing_new_text_defaults', node.tag))
            if node.tag == A + 'normAutofit' and node.get('fontScale', '100000') != '100000':
                errors.append((name, 'automatic_fractional_font_scaling', node.attrib))
    defaults = E.fromstring(parts['ppt/presentation.xml']).find(P + 'defaultTextStyle')
    if defaults is None or len(defaults) != 10:
        errors.append(('ppt/presentation.xml', 'missing_default_text_levels'))
    if E.fromstring(parts['docProps/core.xml']).findtext(DC + 'language') != LANG:
        errors.append(('docProps/core.xml', 'missing_document_language'))
    if errors:
        raise ValueError(json.dumps(errors[:30], ensure_ascii=False))
    return {'font_family': FONT, 'font_size_unit': 'integer pt',
            'language': LANG, 'default_text_levels': 10, **counts, 'violations': 0}


def content_signature(root):
    """Ignore only policy-owned properties, retaining text, geometry and style."""
    root = copy.deepcopy(root)
    # Allow only the identified 19->17pt ID fit repair; compare other sizes too.
    repair_qa_identifiers(root)
    for node in list(root.iter())[::-1]:
        node.attrib.pop('typeface', None)
        node.attrib.pop('lang', None)
        node.attrib.pop('altLang', None)
        for child in list(node):
            if child.tag in FONT_TAGS or child.tag in (P + 'defaultTextStyle', DC + 'language'):
                node.remove(child)
            elif child.tag in PROPERTIES | {A + 'pPr'} and not child.attrib and not len(child) and not child.text:
                node.remove(child)
    def signature(node):
        return (node.tag, tuple(sorted(node.attrib.items())), node.text or '',
                tuple(signature(child) for child in node))
    return signature(root)


def normalize(source_path, authored_path, previous_path, output_path, manifest_path):
    source_path, authored_path, previous_path, output_path, manifest_path = map(
        Path, (source_path, authored_path, previous_path, output_path, manifest_path))
    source, authored = F.read(source_path), F.read(authored_path)
    previous = json.loads(previous_path.read_text())
    assert F.digest(source_path.read_bytes()) == previous['output_sha256']
    order, authored_order = F.ordered(source), F.ordered(authored)
    assert len(order) == len(authored_order) == len(previous['slides']) == 61
    for _, part in authored_order:
        fonts = [n.get('typeface') for n in E.fromstring(authored[part]).iter() if n.get('typeface')]
        assert FONT in fonts, ('Artifact Tool font edit missing', part)
    parts = dict(source)
    for name, data in source.items():
        if name.endswith('.xml'):
            parts[name] = normalize_xml(name, data)
            assert content_signature(E.fromstring(data)) == content_signature(E.fromstring(parts[name])), name
    assert F.ordered(parts) == order
    for name, data in source.items():
        if not name.endswith('.xml'):
            assert parts[name] == data, name
    report = policy_report(parts)
    F.write(output_path, parts)
    rows = copy.deepcopy(previous['slides'])
    for row in rows:
        row.update(source_page=row['page'], source_key=row['key'], kind='windows-typography-and-korean-language',
                   source_slide_sha256=F.digest(source[row['part']]), slide_sha256=F.digest(parts[row['part']]))
    manifest = {
        'version': 5, 'source_path': 'docs/presentations_files/' + source_path.name,
        'source_sha256': previous['output_sha256'], 'output_path': 'docs/presentations_files/VIA-architecture-presentation_v5.pptx',
        'output_sha256': F.digest(output_path.read_bytes()), 'main_slides': 24, 'appendix_divider_page': 25,
        'preserved_v2_background_pages': [58, 59, 60, 61],
        'slide_order_ids_and_keys_preserved': True, 'text_notes_geometry_preserved': True,
        'other_styles_preserved_except_declared_typography_changes': True,
        'font_fit_repairs': {'pages': [9, 10, 41, 42], 'qa_identifier_cells': 12,
                            'from_pt': 19, 'to_pt': 17, 'reason': 'Keep Calibri QA IDs on one line'},
        'media_and_relationships_byte_identical': True,
        'authoring_policy': {'target_application': 'PowerPoint on Windows', 'basis': 'user_request',
                             'scope': 'all slides, tables, notes, masters, layouts, themes and new-text defaults', **report},
        'slides': rows, 'published_date': '2026-10-12'}
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    norm = sub.add_parser('normalize')
    for arg in ('source', 'authored', 'previous', 'output', 'manifest'):
        norm.add_argument(arg)
    check = sub.add_parser('check')
    check.add_argument('pptx')
    args = parser.parse_args()
    if args.command == 'normalize':
        normalize(args.source, args.authored, args.previous, args.output, args.manifest)
    else:
        print(json.dumps(policy_report(F.read(Path(args.pptx)))))
