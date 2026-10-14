#!/usr/bin/env python3
"""Static QA proposal checks only. This does not run VIA or score a PoC."""

from fractions import Fraction
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / 'docs/architecture/12-decisions/decision-packages/qa-candidate-package'
NUMBER = r'\d+(?:\.\d+)?(?:/\d+)?'
GLYPHS = ['○○○', '◐○○', '●○○', '●◐○', '●●○', '●●◐', '●●●']


def interval(raw):
    text = raw.replace(',', '').replace('`', '')
    text = re.sub(r'MiB|ms|초|%', '', text)
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


def main():
    docs = sorted(PACKAGE.glob('qa[0-9][0-9]-*.md'))
    assert len(docs) == 9, f'Expected nine QA specs; found {len(docs)}'
    probes = sum(check_scores(path) for path in docs)
    links = sum(check_links(path) for path in PACKAGE.rglob('*.md'))
    # Verify illustrative arithmetic independently of any future measured data.
    assert max(Fraction('.8'), Fraction('.6'), 1, Fraction('.8')) == 1
    assert Fraction('1.2') + max(0, 3) + Fraction('.6') == Fraction('4.8')
    assert Fraction(5, 100) * 5 + Fraction(26 * 20000 + 90 * 3000, 1000000) * Fraction('.25') + Fraction(26 * 500, 1000000) * Fraction('1.2') == Fraction('.4631')
    assert Fraction(100) - Fraction('1.3') == Fraction('98.7')
    assert Fraction(2 + 2 + 3 + 4 + 3 + 4, 6) == 3
    # MEM-v3: simultaneous RAM example, not a product-target derivation.
    assert 120 + 140 + 30 + 90 == 380
    assert 110 + 180 + 30 + 80 == 400
    assert max(380, 400) == 400
    assert max(120, 110) + max(140, 180) + 30 + max(90, 80) == 420
    print(f'PASS: 9 QA specs, 9 score tables, {probes} score probes, {links} local file links.')
    print('Scope: score intervals/glyphs/monotonicity, local file existence and examples only.')
    print('Not checked: external URLs, heading anchors, empirical targets, DP superiority or PoC results.')


if __name__ == '__main__':
    main()
