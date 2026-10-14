"""Insert all five v12 necessity options into v11, preserving native resources."""
from pathlib import Path
import copy
import json
import posixpath
import re
import sys
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named, patch_node

BASE = 'docs/presentations_files/VIA-architecture-presentation_v11.pptx'
OPTIONS = 'docs/presentations_files/VIA-necessity-options_v12.pptx'
HASHES = ['176d5f006a0bfa7f1c0dc28a4db7137e081438d3f7a52a223fee063f725d853f',
          '1fe80604d8e50647756584fc00122e3834f92c171761d7f07fad9832ff66ed66']


def field(data):
    return ''.join(t.text or '' for t in named(data, 'v6-header-page').findall('.//a:t', F.NS))


def signature(data):
    root = E.fromstring(data)
    st = root.find('p:cSld/p:spTree', F.NS)
    for node in list(st):
        prop = node.find('.//p:cNvPr', F.NS)
        if prop is not None and prop.get('name') == 'v6-header-page':
            st.remove(node)
    return E.tostring(root)


def inputs(repo):
    result = []
    for file, sha in zip([BASE, OPTIONS], HASHES, strict=True):
        assert F.digest((repo/file).read_bytes()) == sha, file
        result.append(F.read(repo/file))
    return result


def prepare(repo, b):
    b.mkdir(parents=True, exist_ok=True)
    src, opts = inputs(repo)
    variants = json.loads((repo/'docs/presentations_files/necessity-options-v12/manifest.json').read_text())['variants']
    sources = []
    for key, file, parts in [('base', BASE, src), ('options', OPTIONS, opts)]:
        edits = []
        for page, (_, part) in enumerate(F.ordered(parts), 1):
            if key == 'base' and page == 2:
                continue
            after = f'{page if page == 1 else page+4} / 37' if key == 'base' else f'{page+1} / 37   {page}안: {variants[page-1]["title"]}'
            edits.append(dict(page=page, before=field(parts[part]), after=after))
        sources.append(dict(key=key, path=file, edits=edits))
    (b/'plan.json').write_text(json.dumps(dict(sources=sources), ensure_ascii=False, indent=2)+'\n')


def package(repo, b):
    src, opts = inputs(repo)
    parts = dict(src)
    order, options_order = F.ordered(src), F.ordered(opts)
    assert len(order) == 33 and len(options_order) == 5
    previous = json.loads((repo/'docs/presentations_files/full-presentation/manifest-v11.json').read_text())
    variants = json.loads((repo/'docs/presentations_files/necessity-options-v12/manifest.json').read_text())['variants']
    plan = json.loads((b/'plan.json').read_text())
    authored = {s['key']:F.read(b/(s['key']+'-authored.pptx')) for s in plan['sources']}

    def page_field(data, source_key, source_page):
        ap = F.ordered(authored[source_key])[source_page-1][1]
        value = field(authored[source_key][ap])
        expected = next(e['after'] for s in plan['sources'] if s['key'] == source_key for e in s['edits'] if e['page'] == source_page)
        assert value == expected, (value, expected)
        node = named(data, 'v6-header-page')
        ts = node.findall('.//a:t', F.NS)
        assert len(ts) == 1
        ts[0].text = value
        return patch_node(data, 'v6-header-page', node)

    ct = E.fromstring(parts['[Content_Types].xml'])
    option_types = F.content_types(opts)
    mapped = {}
    original = order[1][1]
    layout = F.related(src, original, 'slideLayout')
    old_note = F.related(src, original, 'notesSlide')
    note_master = F.related(src, old_note, 'notesMaster')
    for _, part in options_order:
        mapped[F.related(opts, part, 'slideLayout')] = layout
        mapped[F.related(opts, F.related(opts, part, 'notesSlide'), 'notesMaster')] = note_master
    next_part = max(int(re.search(r'slide(\d+)\.xml$', p)[1]) for _, p in order)+1
    next_id = max(int(i) for i, _ in order)+1
    for i, (_, part) in enumerate(options_order):
        mapped[part] = f'ppt/slides/slide{next_part+i}.xml'
        mapped[F.related(opts, part, 'notesSlide')] = f'ppt/notesSlides/necessityV13_{i+1}.xml'

    copied = set()

    def dependency(part):
        if part not in mapped:
            folder, name = posixpath.split(part)
            mapped[part] = folder+'/necessity_v13_'+name
        dest = mapped[part]
        if dest in src or dest in copied:
            return dest
        assert dest not in parts, dest
        copied.add(dest)
        parts[dest] = opts[part]
        E.SubElement(ct, '{'+F.CT+'}Override', PartName='/'+dest, ContentType=option_types(part))
        rels = F.relations(opts, part)
        if rels:
            rs = E.Element('{'+F.PR+'}Relationships')
            for r in rels:
                r = copy.deepcopy(r)
                if r.get('TargetMode') != 'External':
                    r.set('Target', '/'+dependency(F.target(part, r.get('Target'))))
                rs.append(r)
            parts[F.rels_path(dest)] = F.xml(rs)
        return dest

    new_rows = []
    for i, (_, part) in enumerate(options_order):
        dest = dependency(part)
        parts[dest] = page_field(parts[dest], 'options', i+1)
        assert signature(parts[dest]) == signature(opts[part]), i
        assert parts[F.related(parts, dest, 'notesSlide')] == opts[F.related(opts, part, 'notesSlide')]
        new_rows.append(dict(page=i+2, part=dest, slide_id=next_id+i, key=f'necessity-option-v12-{i+1}',
                             title='과제 필요성 / '+variants[i]['title'], kind='inserted-necessity-option',
                             source_path=OPTIONS, source_page=i+1, section='본 발표'))
    rows = []
    for pg, row in enumerate(previous['slides'], 1):
        if pg == 2:
            rows.extend(new_rows)
            continue
        row = copy.deepcopy(row)
        row.update(page=pg if pg == 1 else pg+4, source_page=pg, source_path=BASE, kind='retained-v11')
        part = row['part']
        parts[part] = page_field(parts[part], 'base', pg)
        assert signature(parts[part]) == signature(src[part]), pg
        for n in [F.related(src, part, 'notesSlide'), F.rels_path(part)]:
            assert parts[n] == src[n], n
        rows.append(row)
    root = E.fromstring(parts['ppt/presentation.xml'])
    rs = E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for r in list(rs):
        if r.get('Type').endswith('/slide'):
            rs.remove(r)
    lst = root.find('p:sldIdLst', F.NS)
    lst.clear()
    for row in rows:
        rid = 'V13Slide'+str(row['page'])
        E.SubElement(lst, '{'+F.P+'}sldId', {'id':str(row['slide_id']), '{'+F.R+'}id':rid})
        E.SubElement(rs, '{'+F.PR+'}Relationship', Id=rid, Type=F.R+'/slide', Target='/'+row['part'])
    for section in root.findall('.//{'+F.P14+'}section'):
        ids = section.find('{'+F.P14+'}sldIdLst')
        ids.clear()
        for row in rows[:28] if section.get('name') == '본 발표' else rows[28:]:
            E.SubElement(ids, '{'+F.P14+'}sldId', id=str(row['slide_id']))
    parts['ppt/presentation.xml'] = F.xml(root)
    parts['ppt/_rels/presentation.xml.rels'] = F.xml(rs)
    for n in [original, F.rels_path(original), old_note, F.rels_path(old_note)]:
        del parts[n]
    for n in list(ct):
        if n.tag.endswith('Override') and n.get('PartName').lstrip('/') not in parts:
            ct.remove(n)
    parts['[Content_Types].xml'] = F.xml(ct)
    app = E.fromstring(parts['docProps/app.xml'])
    for n in app:
        if n.tag.split('}')[-1] in ('Slides', 'Notes'):
            n.text = '37'
    parts['docProps/app.xml'] = F.xml(app)
    for row in rows:
        part = row['part']
        row['content_group'] = row.get('section')
        row['section'] = '본 발표' if row['page'] <= 28 else 'Appendix / QA 지표'
        row.update(native_tables=len(E.fromstring(parts[part]).findall('.//a:tbl', F.NS)),
                   slide_sha256=F.digest(parts[part]), notes_sha256=F.digest(parts[F.related(parts, part, 'notesSlide')]))
    preserved_resources = [n for n in src if n.startswith(('ppt/media/','ppt/embeddings/','ppt/charts/','ppt/slideMasters/','ppt/slideLayouts/','ppt/notesMasters/','ppt/theme/'))]
    for n in preserved_resources:
        assert parts[n] == src[n], n
    assert len(F.ordered(parts)) == 37
    assert len([n for n in parts if re.fullmatch(r'ppt/slides/[^/]+\.xml', n)]) == 37
    F.verify(parts)
    policy = W.policy_report(parts)
    assert not policy['violations'], policy
    out = b/'candidate.pptx'
    F.write(out, parts)
    manifest = dict(version=13, published_date='2026-10-15', source_path=BASE, source_sha256=HASHES[0],
                    options_path=OPTIONS, options_sha256=HASHES[1], output_path='docs/presentations_files/VIA-architecture-presentation_v13.pptx',
                    output_sha256=F.digest(out.read_bytes()), total_slides=37, main_slides=28, appendix_first_page=29, appendix_last_page=37,
                    necessity_option_pages=[2,3,4,5,6], retained_v11_slides=32, all_five_option_bodies_and_notes_preserved=True,
                    preserved_source_bodies_and_notes=[i for i in range(1,34) if i != 2],
                    preserved_resource_parts=len(preserved_resources), resources_byte_identical=True,
                    authoring_policy=policy, slides=rows)
    (b/'manifest-v13.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2)+'\n')
    print(json.dumps(dict(slides=37, retained=32, inserted=5, policy_violations=policy['violations'])))


if __name__ == '__main__':
    mode, repo, build = sys.argv[1:]
    {'prepare':prepare, 'package':package}[mode](Path(repo), Path(build))
