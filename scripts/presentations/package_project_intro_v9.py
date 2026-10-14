"""Package two Artifact Tool authored intros; preserve all other v8 bytes."""
from pathlib import Path
import copy
import json
import posixpath
import shutil
import sys
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named, revise_notes

TITLES = ['과제 소개 | 관련 동향', '과제 소개 | 과제 필요성']


def prepare(repo, b):
    assets = b/'assets'
    assets.mkdir(parents=True, exist_ok=True)
    for name in ['continuity.png', 'context.png', 'efficiency.png']:
        shutil.copy2(repo/'docs/presentations_files/project-introduction-v9/assets'/name, assets/name)
    source = F.read(repo/'docs/presentations_files/VIA-architecture-presentation_v8.pptx')
    for name in ['v7_claude_cowork_browser_20260826.gif', 'initial01_image2.gif']:
        (assets/name).write_bytes(source['ppt/media/'+name])


def add_headers(data, source, page, total):
    root = E.fromstring(data)
    st = root.find('p:cSld/p:spTree', F.NS)
    for old in list(st):
        prop = old.find('.//p:cNvPr', F.NS)
        if prop is not None and prop.get('name', '').startswith('v6-header-'):
            st.remove(old)
    for name in ['title', 'rule', *['stage-'+str(i) for i in range(5)], 'page']:
        n = copy.deepcopy(named(source, 'v6-header-'+name))
        if name in ('title', 'page'):
            ts = n.findall('.//a:t', F.NS)
            assert len(ts) == 1
            ts[0].text = TITLES[page-1] if name == 'title' else f'{page} / {total}'
        st.append(n)
    for i, prop in enumerate(st.findall('.//p:cNvPr', F.NS), 2):
        prop.set('id', str(i))
    root.find('p:cSld', F.NS).set('name', 'project-intro-v9-'+str(page))
    return W.normalize_xml('slide.xml', F.xml(root))


def standalone(repo, b):
    src = F.read(repo/'docs/presentations_files/VIA-architecture-presentation_v8.pptx')
    parts = F.read(b/'authored.pptx')
    for i, (_, slide) in enumerate(F.ordered(parts), 1):
        parts[slide] = add_headers(parts[slide], src[F.ordered(src)[i-1][1]], i, 2)
    for n in list(parts):
        if n.endswith('.xml'):
            parts[n] = W.normalize_xml(n, parts[n])
    report = W.policy_report(parts)
    assert not report['violations'], report
    F.write(b/'standalone-candidate.pptx', parts)
    print(json.dumps({'standalone_policy':report}, ensure_ascii=False))


def integrated(repo, b):
    source_path = repo/'docs/presentations_files/VIA-architecture-presentation_v8.pptx'
    src = F.read(source_path)
    native = F.read(b/'final/VIA-project-introduction_v9.pptx')
    parts = dict(src)
    types = F.content_types(native)
    ct = E.fromstring(parts['[Content_Types].xml'])
    mapped = {}

    def dependency(part):
        if part in mapped:
            return mapped[part]
        folder, name = posixpath.split(part)
        dest = folder+'/intro_v9_'+name
        assert dest not in parts, dest
        mapped[part] = dest
        parts[dest] = native[part]
        E.SubElement(ct, '{'+F.CT+'}Override', PartName='/'+dest, ContentType=types(part))
        rels = F.relations(native, part)
        if rels:
            root = E.Element('{'+F.PR+'}Relationships')
            for r in rels:
                r = copy.deepcopy(r)
                if r.get('TargetMode') != 'External':
                    r.set('Target', '/'+dependency(F.target(part, r.get('Target'))))
                root.append(r)
            parts[F.rels_path(dest)] = F.xml(root)
        return dest

    for i, (_, authored) in enumerate(F.ordered(native), 1):
        original = F.ordered(src)[i-1][1]
        parts[original] = add_headers(native[authored], src[original], i, 63)
        note = F.related(src, original, 'notesSlide')
        parts[note] = revise_notes(src[note], native[F.related(native, authored, 'notesSlide')])
        rs = E.Element('{'+F.PR+'}Relationships')
        E.SubElement(rs, '{'+F.PR+'}Relationship', Id='V9SourceLayout', Type=F.R+'/slideLayout', Target='/'+F.related(src, original, 'slideLayout'))
        E.SubElement(rs, '{'+F.PR+'}Relationship', Id='V9SourceNote', Type=F.R+'/notesSlide', Target='/'+note)
        for r in F.relations(native, authored):
            if r.get('Type').endswith(('/slideLayout', '/notesSlide')):
                continue
            r = copy.deepcopy(r)
            if r.get('TargetMode') != 'External':
                r.set('Target', '/'+dependency(F.target(authored, r.get('Target'))))
            rs.append(r)
        parts[F.rels_path(original)] = F.xml(rs)
    parts['[Content_Types].xml'] = F.xml(ct)
    preserved = []
    for i, (_, part) in enumerate(F.ordered(src), 1):
        if i > 2:
            assert src[part] == parts[part]
            note = F.related(src, part, 'notesSlide')
            assert src[note] == parts[note]
            assert src[F.rels_path(part)] == parts[F.rels_path(part)]
            preserved.append(i)
    media = [n for n in src if n.startswith('ppt/media/')]
    system = [n for n in src if n.startswith(('ppt/slideMasters/', 'ppt/slideLayouts/', 'ppt/notesMasters/', 'ppt/theme/'))]
    for name in media+system:
        assert src[name] == parts[name], name
    # Both animated sources must remain referenced and byte-identical in slide 1.
    first = F.ordered(parts)[0][1]
    images = [parts[F.target(first, r.get('Target'))] for r in F.relations(parts, first) if r.get('Type').endswith('/image')]
    for name in ['ppt/media/v7_claude_cowork_browser_20260826.gif', 'ppt/media/initial01_image2.gif']:
        assert src[name] in images, 'Animated GIF replaced or flattened: '+name
    report = W.policy_report(parts)
    assert not report['violations'], report
    candidate = b/'integrated-candidate.pptx'
    F.write(candidate, parts)
    previous = json.loads((repo/'docs/presentations_files/full-presentation/manifest-v8.json').read_text())
    rows = copy.deepcopy(previous['slides'])
    for i, row in enumerate(rows, 1):
        if i <= 2:
            row.update(key='project-intro-v9-'+str(i), title=TITLES[i-1], kind='authored-project-intro')
        row['slide_sha256'] = F.digest(parts[row['part']])
        row['notes_sha256'] = F.digest(parts[F.related(parts,row['part'],'notesSlide')])
    manifest = {'version':9,'published_date':'2026-10-14','source_path':str(source_path.relative_to(repo)),
        'source_sha256':F.digest(source_path.read_bytes()),'output_path':'docs/presentations_files/VIA-architecture-presentation_v9.pptx',
        'output_sha256':F.digest(candidate.read_bytes()),'total_slides':63,'revised_pages':[1,2],
        'preserved_source_bodies_and_notes':preserved,'all_original_media_byte_identical':True,
        'original_media_parts':len(media),'masters_layouts_themes_byte_identical':True,
        'animated_sources_referenced_and_byte_identical':True,'authoring_policy':report,'slides':rows}
    (b/'manifest-v9.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'preserved_slide_bodies_and_notes':len(preserved),'policy_violations':report['violations']}))


if __name__ == '__main__':
    mode, repo, b = sys.argv[1:]
    {'prepare':prepare,'standalone':standalone,'integrated':integrated}[mode](Path(repo),Path(b))
