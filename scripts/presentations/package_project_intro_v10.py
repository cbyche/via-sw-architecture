"""Apply scoped v9 review edits and replace the necessity slide with native diagrams."""
from pathlib import Path
import copy
import json
import posixpath
import sys
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named, revise_notes, notes_body, patch_node, normalized_node
from package_project_intro_v9 import add_headers, TITLES

PATCHES = ['user-via-link', 'via-agent-link', 'demo-two-source', 'market-title', 'market-source', 'cases-source']


def prepare(repo, b):
    b.mkdir(parents=True, exist_ok=True)
    src = F.read(repo/'docs/presentations_files/VIA-project-introduction_v9.pptx')
    part = F.ordered(src)[0][1]
    body = notes_body(src[F.related(src, part, 'notesSlide')])
    text = '\n'.join(''.join(t.text or '' for t in para.findall('.//a:t', F.NS))
                     for para in body.findall('p:txBody/a:p', F.NS))
    text = text.replace('자료 확인일: 2026-10-14', '자료 확인일: 2026-10-15')
    text += '''\n\n시장 범위 확인\nVIA는 음성 기반 상호작용 소프트웨어라는 점에서 이 시장과 관련. 다만 보고서는 PC 전용 VIA 시장을 별도로 산정하지 않으며, 자동차, 고객상담 등 여러 산업의 음성 비서 소프트웨어와 관련 서비스를 포함. 따라서 상위 관련 시장의 성장 방향을 보여주는 근거로 사용. PC용 VIA의 직접 TAM/SAM이나 삼성의 확보 가능한 매출 규모로 해석 불가. 보고서 수치와 포함 관계를 근거로 한 분류 판단이며 조사사의 VIA 제품 분류 결과가 아님.\n\nOpenAI 날짜 근거\n공식 발표일 2026-09-03을 시연 캡션에 명시. 영상 촬영일을 뜻하지 않음.\nhttps://help.openai.com/en/articles/6825453-chatgpt-release-notes\nSeptember 3, 2026: Introducing GPT-6 Astra\nhttps://www.youtube.com/watch?v=1QNsdr-Qx_I\n기존 시연 GIF의 원본 바이트와 발췌 조건 보존.\n'''
    (b/'slide1-notes.txt').write_text(text)


def replace_slide(parts, src, native, page, total, ct, mapped):
    original = F.ordered(src)[page-1][1]
    authored = F.ordered(native)[page-1][1]
    types = F.content_types(native)
    hashes = {F.digest(data):name for name,data in src.items()
              if not F.relations(src,name) and name.startswith(('ppt/media/','ppt/embeddings/'))}

    def dependency(part):
        if part in mapped:
            return mapped[part]
        match = hashes.get(F.digest(native[part]))
        if match is not None and types(part) == F.content_types(src)(match):
            mapped[part] = match
            return match
        folder, name = posixpath.split(part)
        dest = folder+'/intro_v10_'+name
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

    data = add_headers(native[authored], src[original], page, total)
    parts[original] = data.replace(b'project-intro-v9-', b'project-intro-v10-')
    note = F.related(src, original, 'notesSlide')
    parts[note] = revise_notes(src[note], native[F.related(native, authored, 'notesSlide')])
    rs = E.Element('{'+F.PR+'}Relationships')
    E.SubElement(rs, '{'+F.PR+'}Relationship', Id='V10SourceLayout', Type=F.R+'/slideLayout', Target='/'+F.related(src, original, 'slideLayout'))
    E.SubElement(rs, '{'+F.PR+'}Relationship', Id='V10SourceNote', Type=F.R+'/notesSlide', Target='/'+note)
    for r in F.relations(native, authored):
        if r.get('Type').endswith(('/slideLayout', '/notesSlide')):
            continue
        r = copy.deepcopy(r)
        if r.get('TargetMode') != 'External':
            r.set('Target', '/'+dependency(F.target(authored, r.get('Target'))))
        rs.append(r)
    parts[F.rels_path(original)] = F.xml(rs)


def standalone(repo, b):
    src = F.read(repo/'docs/presentations_files/VIA-project-introduction_v9.pptx')
    native = F.read(b/'authored.pptx')
    parts = dict(src)
    part, patch = F.ordered(src)[0][1], F.ordered(native)[0][1]
    for name in PATCHES:
        old = named(parts[part], name)
        new = copy.deepcopy(named(native[patch], name))
        new.find('.//p:cNvPr', F.NS).set('id', old.find('.//p:cNvPr', F.NS).get('id'))
        parts[part] = patch_node(parts[part], name, normalized_node(new))
    parts[part] = parts[part].replace(b'project-intro-v9-', b'project-intro-v10-')
    note = F.related(src,part,'notesSlide')
    parts[note] = revise_notes(src[note],native[F.related(native,patch,'notesSlide')])
    ct = E.fromstring(parts['[Content_Types].xml'])
    replace_slide(parts,src,native,2,2,ct,{})
    parts['[Content_Types].xml'] = F.xml(ct)
    # Chart and embedded workbook remain byte-identical to v9.
    for name in src:
        if name.startswith(('ppt/charts/','ppt/embeddings/','ppt/media/')):
            assert src[name] == parts[name], name
    report = W.policy_report(parts)
    assert not report['violations'], report
    F.write(b/'standalone-candidate.pptx',parts)
    print(json.dumps(report,ensure_ascii=False))


def integrated(repo, b):
    source_path = repo/'docs/presentations_files/VIA-architecture-presentation_v9.pptx'
    src = F.read(source_path)
    native = F.read(b/'final/VIA-project-introduction_v10.pptx')
    parts = dict(src)
    ct = E.fromstring(parts['[Content_Types].xml'])
    mapped = {}
    for page in (1,2):
        replace_slide(parts,src,native,page,63,ct,mapped)
    parts['[Content_Types].xml'] = F.xml(ct)
    preserved=[]
    for i,(_,part) in enumerate(F.ordered(src),1):
        if i>2:
            for name in [part,F.related(src,part,'notesSlide'),F.rels_path(part)]:
                assert src[name] == parts[name], name
            preserved.append(i)
    media=[n for n in src if n.startswith('ppt/media/')]
    system=[n for n in src if n.startswith(('ppt/slideMasters/','ppt/slideLayouts/','ppt/notesMasters/','ppt/theme/'))]
    for name in media+system:
        assert src[name] == parts[name], name
    first=F.ordered(parts)[0][1]
    images=[parts[F.target(first,r.get('Target'))] for r in F.relations(parts,first) if r.get('Type').endswith('/image')]
    for name in ['ppt/media/v7_claude_cowork_browser_20260826.gif','ppt/media/initial01_image2.gif']:
        assert src[name] in images, name
    report=W.policy_report(parts)
    assert not report['violations'],report
    candidate=b/'integrated-candidate.pptx'
    F.write(candidate,parts)
    previous=json.loads((repo/'docs/presentations_files/full-presentation/manifest-v9.json').read_text())
    rows=copy.deepcopy(previous['slides'])
    for i,row in enumerate(rows,1):
        if i<=2:
            row.update(key='project-intro-v10-'+str(i),title=TITLES[i-1],kind='authored-project-intro')
        row['slide_sha256']=F.digest(parts[row['part']])
        row['notes_sha256']=F.digest(parts[F.related(parts,row['part'],'notesSlide')])
    manifest={'version':10,'published_date':'2026-10-15','source_path':str(source_path.relative_to(repo)),
        'source_sha256':F.digest(source_path.read_bytes()),'output_path':'docs/presentations_files/VIA-architecture-presentation_v10.pptx',
        'output_sha256':F.digest(candidate.read_bytes()),'total_slides':63,'revised_pages':[1,2],
        'preserved_source_bodies_and_notes':preserved,'all_original_media_byte_identical':True,
        'original_media_parts':len(media),'masters_layouts_themes_byte_identical':True,
        'animated_sources_referenced_and_byte_identical':True,'authoring_policy':report,'slides':rows}
    (b/'manifest-v10.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'preserved_slide_bodies_and_notes':len(preserved),'policy_violations':report['violations']}))


if __name__=='__main__':
    mode,repo,b=sys.argv[1:]
    {'prepare':prepare,'standalone':standalone,'integrated':integrated}[mode](Path(repo),Path(b))
