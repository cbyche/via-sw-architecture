#!/usr/bin/env bash
set -euo pipefail
OUT="${1:-results/gate2-local/environment.json}"
mkdir -p "$(dirname "$OUT")"
export G2_ENV_OUT="$OUT"
python3 - <<'PY'
import json, os, platform, shutil, subprocess
from pathlib import Path

def run(*args):
    try:
        return subprocess.check_output(args, text=True, stderr=subprocess.STDOUT).strip()
    except Exception as exc:
        return f'UNAVAILABLE: {type(exc).__name__}: {exc}'

def sysctl(key):
    return run('sysctl', '-n', key)

data = {
    'evidence': 'ENVIRONMENT_MANIFEST_ONLY',
    'benchmark': 'NOT_RUN',
    'platform': {
        'uname': run('uname', '-a'),
        'macos': run('sw_vers'),
        'machine': platform.machine(),
        'model': sysctl('hw.model'),
        'cpu_brand': sysctl('machdep.cpu.brand_string'),
        'logical_cpu': sysctl('hw.logicalcpu'),
        'physical_memory_bytes': sysctl('hw.memsize'),
    },
    'thermal_snapshot': run('pmset', '-g', 'therm'),
    'toolchain': {
        'git': run('git', '--version'),
        'git_head': run('git', 'rev-parse', 'HEAD'),
        'rustc': run('rustc', '--version', '--verbose') if shutil.which('rustc') else 'MISSING',
        'cargo': run('cargo', '--version') if shutil.which('cargo') else 'MISSING',
        'python': run('python3', '--version'),
        'llama_server': run('llama-server', '--version') if shutil.which('llama-server') else 'NOT_INSTALLED',
    },
}
Path(os.environ['G2_ENV_OUT']).write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(os.environ['G2_ENV_OUT'])
PY
