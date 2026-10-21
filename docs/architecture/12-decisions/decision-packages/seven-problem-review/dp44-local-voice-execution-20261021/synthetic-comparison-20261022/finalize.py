"""Finalize the authored package after ledger, report and workbook exports."""
from pathlib import Path
from datetime import datetime, timezone
from urllib.parse import unquote
import ast
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET

HERE = Path(__file__).resolve().parent
ROOT = next(p for p in HERE.parents if (p / 'INTENT.md').exists())
DESIGN = HERE.parent
OUT = ROOT / 'docs/presentations_files/dp44-synthetic-backdata-20261022'
NODE = os.environ.get('VIA_NODE', str(Path.home() / '.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node'))
NAV = [ROOT / 'docs/architecture/12-decisions/decision-packages/00-workplan.md',
       DESIGN.parent / 'README.md', DESIGN.parent / 'dp44-conversation-delivery-reset-20261021.md']


def run(args):
    p = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, timeout=60)
    return {'exit_code': p.returncode, 'output': (p.stdout + p.stderr).strip()}


def main():
    validation = json.loads((OUT / 'validation.json').read_text())
    workbook = json.loads((OUT / 'workbook-validation.json').read_text())
    assert validation['status'] == workbook['status'] == 'PASS'
    markdown = sorted(set([*DESIGN.rglob('*.md'), OUT / 'README.md', *NAV]))
    missing = []
    pending = {OUT / 'manifest.json', OUT / 'publication-validation.json'}
    for document in markdown:
        for match in re.findall(r'!?\[[^\]]*\]\(([^)]+)\)', document.read_text()):
            target = match.split('#', 1)[0].strip('<>')
            if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:', target):
                continue
            path = (document.parent / unquote(target)).resolve()
            if path not in pending and not path.exists():
                missing.append({'document': str(document.relative_to(ROOT)), 'target': target})
    assert not missing, missing
    xml_files = sorted([*DESIGN.rglob('*.svg'), *DESIGN.rglob('*.drawio')])
    for path in xml_files:
        ET.parse(path)
    for path in HERE.glob('*.py'):
        ast.parse(path.read_text(), filename=str(path))
    html = (OUT / 'outputs/01a154de/guide.html').read_text()
    scripts = re.findall(r'<script[^>]*>(.*?)</script>', html, flags=re.S)
    assert scripts
    with tempfile.TemporaryDirectory(prefix='dp44-html-check-') as temp:
        source = Path(temp) / 'guide.js'
        source.write_text('\n'.join(scripts))
        html_check = run([NODE, '--check', str(source)])
    assert html_check['exit_code'] == 0, html_check
    diff = run(['git', 'diff', '--check', '--', *[str(p.relative_to(ROOT)) for p in NAV]])
    assert diff['exit_code'] == 0, diff
    # Materialize generated link targets before the repository-wide check.
    for path in pending:
        if not path.exists():
            path.write_text('{"status": "FINALIZATION_IN_PROGRESS"}\n')
    record = {
        'version': 'DP44-DELIVERY-SYN-1.0',
        'evidence_kind': 'SYNTHETIC_AUTHORED_NOT_MEASURED',
        'checked_at_utc': datetime.now(timezone.utc).isoformat(),
        'scope_status': 'PASS_WITH_RECORDED_LIMITS',
        'ledger': validation['status'],
        'workbook': workbook['status'],
        'scoped_markdown': {'files': len(markdown), 'missing_paths': missing, 'anchors_checked': False},
        'diagram_xml': {'files': len(xml_files), 'status': 'PASS'},
        'python_syntax': 'PASS',
        'html_javascript_syntax': html_check,
        'tracked_scope_diff_check': diff,
        'active_terminology': run([sys.executable, 'scripts/architecture/check_active_terminology.py']),
        'repository_link_snapshot': run([sys.executable, 'scripts/architecture/check_active_markdown_links.py']),
        'repository_link_checker_sha256': hashlib.sha256((ROOT / 'scripts/architecture/check_active_markdown_links.py').read_bytes()).hexdigest(),
        'repository_link_checker_has_uncommitted_changes': bool(run(['git', 'diff', '--', 'scripts/architecture/check_active_markdown_links.py'])['output']),
        'repository_snapshot_note': 'Whole-repository state includes concurrent work; failures outside DP44 are not scoped failures.',
        'manual_visual_review': {'workbook_sheets': workbook['rendered_sheets'], 'status': 'PASS', 'observed_on': '2026-10-22'},
        'html_browser_visual': {'status': 'NOT_COMPLETED', 'reason': 'Local listener permission denied; browser file URL security policy rejected access. No alternate route attempted after the browser policy rejection.'},
        'native_windows_excel': 'NOT_RUN',
        'independent_review': 'NOT_PERFORMED_IN_THIS_PACKAGE',
        'actual_model_calls': 0,
        'actual_product_runs': 0,
        'remote_ci': 'NOT_CHECKED_BY_THIS_SCRIPT; report after publishing separately',
    }
    assert record['active_terminology']['exit_code'] == 0
    (OUT / 'publication-validation.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
    artifacts = sorted(set([p for folder in (DESIGN, OUT) for p in folder.rglob('*') if p.is_file()] + NAV))
    artifacts = [p for p in artifacts if p.name != 'manifest.json' and '__pycache__' not in p.parts and not p.name.endswith('.inspect.ndjson')]
    manifest = {'version': record['version'], 'evidence_kind': record['evidence_kind'],
        'path_base': 'repository root', 'files': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in artifacts}}
    (OUT / 'manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'scope_status': record['scope_status'], 'hashed_files': len(artifacts),
        'scoped_markdown_files': len(markdown), 'diagram_xml_files': len(xml_files),
        'repository_links': record['repository_link_snapshot']}, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
