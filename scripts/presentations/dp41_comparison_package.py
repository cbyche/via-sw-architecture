"""Update the native DP41 comparison only in v2 and comparison decks.

Resolve the slide through its authored name and presentation relationships.
The two v2 introductions, background images and all non-DP41 parts survive
byte-for-byte. No physical slide numbers or flattened replacements are used.
"""
from pathlib import Path
import hashlib
import json
import sys
import zipfile
from dp_pptx_package import related, replace_related
from dp_v2_intro_package import read, slides


def graft(candidate, source, report):
    before, generated = read(source), read(candidate)
    parts = dict(before)
    slug = 'dp41-comparison'
    def find(package):
        matches = [s for s in slides(package)
                   if any(n.startswith(slug+' / ') for n in s['names'])]
        assert len(matches) == 1, matches
        return matches[0]
    old, new = find(before), find(generated)
    replace_related(parts, generated, old['part'], new['part'])
    mutable = {old['part'], related(before, old['part'], 'notesSlide')}
    changed = {n for n in before if parts[n] != before[n]}
    assert changed <= mutable, changed
    assert set(parts) == set(before)
    assert len(slides(parts)) == len(slides(before))
    tmp = candidate.with_suffix('.scoped.pptx')
    with zipfile.ZipFile(source) as original, zipfile.ZipFile(tmp, 'w') as out:
        for info in original.infolist():out.writestr(info, parts[info.filename])
    tmp.replace(candidate)
    report.write_text(json.dumps(dict(
        source=str(source), source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
        slide_count=len(slides(parts)), selected_slug=slug,
        changed_parts=sorted(changed), all_other_parts_preserved=True),indent=2)+'\n')


if __name__ == '__main__':
    assert len(sys.argv)==4
    graft(*map(Path,sys.argv[1:]))
