#!/usr/bin/env python3
"""Static QA proposal checks only. This does not run VIA or score a PoC."""

from fractions import Fraction
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit
import qa_candidate_population as population

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / 'docs/architecture/12-decisions/decision-packages/qa-candidate-package'
NUMBER = r'\d+(?:\.\d+)?(?:/\d+)?'
GLYPHS = ['○○○', '◐○○', '●○○', '●◐○', '●●○', '●●◐', '●●●']


def interval(raw):
    text = raw.replace(',', '').replace('`', '')
    text = re.sub(r'MiB|MB|ms|초|%', '', text)
    chain = re.search(rf'({NUMBER})\s*([<≤])\s*[A-Za-z]+\s*([<≤])\s*({NUMBER})', text)
    if chain:
        lo, left, right, hi = chain.groups()
        return Fraction(lo), left == '≤', Fraction(hi), right == '≤'
    single = re.search(rf'[A-Za-z]+\s*([<≤>=])\s*({NUMBER})', text)
    assert single, f'Unsupported score expression: {raw}'
    op, bound = single.groups()
    value = Fraction(bound)
    if op == '=':
        return value, True, value, True
    if op in ('<', '≤'):
        return Fraction(0), True, value, op == '≤'
    return value, False, None, False


def contains(bounds, value):
    lo, lo_closed, hi, hi_closed = bounds
    return (value > lo or lo_closed and value == lo) and (
        hi is None or value < hi or hi_closed and value == hi)


def check_scores(path):
    body = path.read_text()
    assert 'SCORE_BANDS_PENDING' not in body, path.name
    rows = {}
    for line in body.splitlines():
        if not re.match(r'^\|\s*[0-6]\s*\|', line):
            continue
        cells = [part.strip() for part in line.split('|')[1:-1]]
        score = int(cells[0])
        assert score not in rows, f'{path.name}: duplicate score {score}'
        assert cells[1] == GLYPHS[score], f'{path.name}: incorrect glyph {score}'
        rows[score] = interval(cells[2])
    assert set(rows) == set(range(7)), f'{path.name}: missing score rows'
    boundaries = {Fraction(0)}
    for lo, _, hi, _ in rows.values():
        boundaries.add(lo)
        if hi is not None:
            boundaries.add(hi)
    probes = set(boundaries)
    for value in boundaries:
        for offset in (Fraction(-1, 10000), Fraction(1, 10000)):
            if value + offset >= 0:
                probes.add(value + offset)
    ordered = sorted(boundaries)
    probes.update((a + b) / 2 for a, b in zip(ordered, ordered[1:]))
    probes.add(ordered[-1] * 2 + 1)
    scores = []
    for value in sorted(probes):
        if path.name.startswith('qa02-') and value > 100:
            continue  # A percentage above 100 is invalid input, not an eighth band.
        matches = [score for score, bounds in rows.items() if contains(bounds, value)]
        assert len(matches) == 1, f'{path.name}: {value} has scores {matches}'
        scores.append(matches[0])
    increasing = path.name.startswith('qa02-')
    assert all(a <= b if increasing else a >= b for a, b in zip(scores, scores[1:])), path.name
    return len(scores)


def check_links(path):
    count = 0
    for raw in re.findall(r'\]\(([^\s)]+)', path.read_text()):
        ref = raw.strip('<>')
        if urlsplit(ref).scheme or ref.startswith('#'):
            continue
        target = unquote(ref.split('#', 1)[0])
        if not target:
            continue
        assert (path.parent / target).resolve().exists(), f'{path}: missing link {target}'
        count += 1
    return count


def check_common_population():
    data = population.load()
    counts = population.validate(data)
    expected = dict(scenarios=28, user=84, agent=36, responses=120,
                    goals=76, bindings=44, voices=109)
    assert all(counts[key] == value for key, value in expected.items())
    body = (PACKAGE / 'common-scenarios.md').read_text()
    for label, render in [('COUNTS', population.counts_md),
                          ('CATALOG', population.catalog_md),
                          ('COVERAGE', population.coverage_md)]:
        assert population.replace_block(body, label, render(data)) == body
    assert (PACKAGE / 'common-model-cost-quote.md').read_text() == population.quote_md(data)
    _, realtime, luna, total, budget = population.quote(data)
    assert Fraction(realtime) == Fraction('2.04632')
    assert Fraction(luna) == Fraction('.49972')
    assert Fraction(total) == Fraction('2.54604')
    assert Fraction(budget) == Fraction('2.55')
    assert data['execution_profile']['response_timeout_seconds'] == 30
    assert len(data['qualification_variants']) == 18
    # Both requested tables must use the same population, even after prose edits.
    table = (PACKAGE / 'quality-requirements-table.md').read_text()
    projected = {}
    for line in table.splitlines():
        match = re.match(r'^\| (?:\*\*)?\[(QA\d\d)\]\(\./qa', line)
        if match:
            projected.setdefault(match[1], []).append(line)
    expected_tokens = {
        'QA01': ['28', '120', '5'], 'QA02': ['28', '44', '5'],
        'QA03': ['28', '76', '5'], 'QA04': ['3', '28'],
        'QA05': ['28', '5', '$2.55', '$2.54604'],
        'QA06': ['28', '5', '512 MB'], 'QA07': ['S05/S11/S18/S19', '5'],
        'QA08': ['6'], 'QA09': ['28', '109', '5'],
    }
    for qa, tokens in expected_tokens.items():
        assert len(projected.get(qa, [])) == 2, qa
        for row in projected[qa]:
            for token in tokens:
                assert token in row, f'{qa}: missing table projection {token}'
    for qa, count in [('QA01', 600), ('QA02', 220), ('QA03', 380), ('QA09', 545)]:
        assert str(count) in projected[qa][0], qa
    print('PASS: common population, generated catalogs, cost arithmetic and both QA tables.')


def score_cells(body):
    rows = {}
    for line in body.splitlines():
        if re.match(r'^\|\s*[0-6]\s*\|', line):
            cells = [part.strip() for part in line.split('|')[1:-1]][:3]
            assert int(cells[0]) not in rows, 'Duplicate score in page projection'
            rows[int(cells[0])] = cells
    return rows


def check_page_projection(docs):
    pages = (PACKAGE / 'slide-page-plan.md').read_text()
    compact = (PACKAGE / 'quality-requirements-table.md').read_text().split(
        '<a id="presentation-table"></a>')[1]
    for index, path in enumerate(docs, 1):
        anchor = f'<a id="qa{index:02}"></a>'
        assert pages.count(anchor) == 1, path.name
        page = pages.split(anchor)[1].split('<a id=')[0]
        assert score_cells(page) == score_cells(path.read_text()), path.name
        row = next(line for line in compact.splitlines()
                   if line.startswith(f'| [QA{index:02}]'))
        cells = [part.strip() for part in row.split('|')[1:-1]]
        expected = '| ' + ' | '.join(cells[1:4]) + ' |'
        assert expected in page, f'{path.name}: compact requirement mismatch'
        for label in ['측정 산식', '설명용 예시', '구간 도출', '출처 각주', '발표자 노트', 'DP 설명']:
            assert label in page, f'{path.name}: missing page field {label}'
    print('PASS: 9 self-contained page projections match requirement and score tables.')


def main():
    check_common_population()
    docs = sorted(PACKAGE.glob('qa[0-9][0-9]-*.md'))
    assert len(docs) == 9, f'Expected nine QA specs; found {len(docs)}'
    # The new decimal-MB target is user-defined, not inferred from older studies.
    ram = PACKAGE / 'qa06-local-memory.md'
    ram_body = ram.read_text()
    assert 'USER_DEFINED_SYSTEM_REQUIREMENT' in ram_body
    assert 'MEASUREMENT_METHOD_PENDING' in ram_body
    assert 'RAM-v3-MB' in ram_body and '/ 1,000,000  [MB]' in ram_body
    source = (PACKAGE / 'targets-source-qa06-platform.md').read_text()
    assert 'SR-PC-MEM-01' in source and '512,000,000바이트' in source
    assert Fraction(512000000, 2**20) == Fraction('488.28125')
    ram_scores = score_cells(ram_body)
    for value, expected in [(256, 6), (400, 3), (512, 1), (Fraction('512.000001'), 0)]:
        actual = [n for n, cells in ram_scores.items() if contains(interval(cells[2]), value)]
        assert actual == [expected], (value, actual)
    probes = sum(check_scores(path) for path in docs)
    check_page_projection(docs)
    links = sum(check_links(path) for path in PACKAGE.rglob('*.md'))
    # Verify illustrative arithmetic independently of any future measured data.
    assert max(Fraction('.8'), Fraction('.6'), 1, Fraction('.8')) == 1
    assert Fraction('1.2') + max(0, 3) + Fraction('.6') == Fraction('4.8')
    assert Fraction('5.4') * Fraction('.6') == Fraction('3.24')
    assert (Fraction('5.4') - 3) / Fraction('5.4') == Fraction(4, 9)
    assert Fraction(2 + 2 + 3 + 4 + 3 + 4, 6) == 3
    # Simultaneous RAM example in decimal MB; these are illustrative, not measured.
    assert 120 + 140 + 30 + 90 == 380
    assert 110 + 180 + 30 + 80 == 400
    assert max(380, 400) == 400
    assert max(120, 110) + max(140, 180) + 30 + max(90, 80) == 420
    print(f'PASS: 9 QA specs, 9 score tables, {probes} score probes, {links} local file links.')
    print('QA06: user-defined 512 MB target; Windows measurement method remains unverified.')
    print('Scope: documented population, projection arithmetic, score intervals/glyphs/monotonicity, local file existence and examples.')
    print('Not checked: external URLs, heading anchors, empirical targets, DP superiority or PoC results.')


if __name__ == '__main__':
    main()
