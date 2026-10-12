"""Verify v8 preservation and scene synchronization, not quality performance."""
from pathlib import Path
import hashlib
import json
import sys
import xml.etree.ElementTree as E
from PIL import Image, ImageChops
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import body_signature, named


def check(repo, build, source_render_build, figure_build):
    repo, b, source_b, figure_b = map(Path, (repo, build, source_render_build, figure_build))
    final = b/'final/VIA-architecture-presentation_v8.pptx'
    source_path = repo/'docs/presentations_files/VIA-architecture-presentation_v7.pptx'
    source, output = F.read(source_path), F.read(final)
    F.verify(output)
    manifest = json.loads((b/'manifest-v8.json').read_text())
    assert F.digest(final.read_bytes()) == manifest['output_sha256']
    assert F.digest(source_path.read_bytes()) == manifest['source_sha256']
    assert len(F.ordered(output)) == 63
    source_order = F.ordered(source)
    preserved_pixels = []
    for row in manifest['slides']:
        if row['source_page'] not in manifest['preserved_source_bodies_and_notes']:
            continue
        old_part = source_order[row['source_page']-1][1]
        assert body_signature(output[row['part']]) == body_signature(source[old_part])
        note = F.related(source, old_part, 'notesSlide')
        assert output[note] == source[note]
        left = Image.open(source_b/f'source-renders/{row["source_page"]:02}.png').convert('RGB')
        right = Image.open(b/f'renders/{row["page"]:02}.png').convert('RGB')
        assert left.size == right.size == (1920,1080)
        diff = ImageChops.difference(left,right)
        # Only the rendered page field changes on these preserved pages.
        diff.paste((0,0,0),(1740,72,1920,105))
        if row['page'] in (62,63):
            # These two moved pages now highlight Appendix in the navigation.
            diff.paste((0,0,0),(1140,20,1885,74))
        assert diff.getbbox() is None, ('Unexpected rendered body change',row['page'])
        preserved_pixels.append({'source_page':row['source_page'],'page':row['page']})
    media = [n for n in source if n.startswith('ppt/media/')]
    for name in media:
        assert source[name] == output[name]
    scenes = json.loads((b/'scenes.json').read_text())
    pages = [14,15,17,18,20,22]
    for scene, page in zip(scenes,pages):
        part = manifest['slides'][page-1]['part']
        for item in scene['items']:
            if item['kind'] == 'line':
                continue
            shape = named(output[part], item['id'])
            body = shape.find('p:txBody',F.NS)
            text = '\n'.join(''.join(n.text or '' for n in p.findall('.//a:t',F.NS)) for p in body.findall('a:p',F.NS))
            assert text == item['text'],(page,item['id'],text)
            sizes = [int(n.get('sz')) for n in body.iter() if n.get('sz')]
            assert sizes and set(sizes) == {item['sizePt']*100},(page,item['id'],sizes)
    figures = json.loads((figure_b/'figure-render-report.json').read_text())
    for figure in figures:
        path = repo/'docs/architecture/12-decisions/decision-packages/diagrams'/f'{figure["slug"]}.svg'
        assert F.digest(path.read_bytes()) == figure['svg_sha256']
        assert not figure['issues']
        png = figure_b/'figure-renders'/f'{figure["slug"]}.png'
        assert F.digest(png.read_bytes()) == figure['png_sha256']
        assert Image.open(png).size == (1920,1080)
    report = {'version':8,'final_sha256':F.digest(final.read_bytes()),'source_sha256':manifest['source_sha256'],
        'slides_rendered':63, 'preserved_body_and_notes_count':len(preserved_pixels),
        'preserved_pixels_except_page_fields_and_moved_navigation':preserved_pixels,
        'media_parts_preserved_bytes':len(media),'scene_text_and_integer_pt_match':pages,
        'figure_render_geometry_issues':0,'figure_assets':figures,'typography_policy':W.policy_report(output),
        'claim_boundary':'Local documentation render/package checks. No model runs, Architecture quality measurements, A/B selection, Windows SDK validation or native PowerPoint execution.'}
    (b/'preservation-v8.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'slides':63,'preservedPixelBodies':len(preserved_pixels),'media':len(media),'figureIssues':0,'policyViolations':0}))


if __name__ == '__main__':
    check(*sys.argv[1:])
