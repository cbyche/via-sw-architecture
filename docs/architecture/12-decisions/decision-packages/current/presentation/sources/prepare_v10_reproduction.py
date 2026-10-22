"""Prepare an isolated replay of the historical v9-to-v10 presentation build.

Only runtime paths in an execution copy are adapted. The preserved snapshot and
the current presentation are never edited. By default no export or render runs.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import shlex
import shutil
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--workdir', required=True, type=Path)
    parser.add_argument('--runtime-root', required=True, type=Path)
    parser.add_argument('--skill-dir', required=True, type=Path)
    parser.add_argument('--run', action='store_true', help='Export and render after preparing the isolated build')
    args = parser.parse_args()
    stage = args.workdir.expanduser().resolve()
    runtime = args.runtime_root.expanduser().resolve()
    skill = args.skill_dir.expanduser().resolve()
    snapshot = Path(__file__).resolve().parent / 'v10-snapshot'
    node = runtime / 'node/bin/node'
    python = runtime / 'python/bin/python3'
    modules = runtime / 'node/node_modules'
    for item in (node, python, modules, skill / 'container_tools/artifact_tool_utils.mjs'):
        if not item.exists():
            parser.error(f'Required runtime input does not exist: {item}')
    if stage.exists():
        parser.error(f'Refusing to overwrite an existing work directory: {stage}')
    scripts = stage / 'scripts/presentations'
    shutil.copytree(snapshot / 'scripts/presentations', scripts)
    shutil.copyfile(snapshot / 'input-v9.pptx', stage / 'input-v9.pptx')
    (stage / 'node_modules').symlink_to(modules, target_is_directory=True)
    builder = scripts / 'build_reference_guide_v10.mjs'
    code = builder.read_text()
    replacements = {
        "const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1017.22135/skills/presentations';":
            'const SKILL=' + json.dumps(str(skill)) + ';',
        "const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';":
            'const PY=' + json.dumps(str(python)) + ';',
        "process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';":
            'process.env.RUNTIME_NODE_MODULES=' + json.dumps(str(modules)) + ';',
    }
    for original, adapted in replacements.items():
        if code.count(original) != 1:
            raise ValueError('Snapshot runtime assignment changed; review the adapter before replay')
        code = code.replace(original, adapted)
    builder.write_text(code)
    command = [str(node), str(builder), str(stage), str(stage / 'build'), str(stage / 'input-v9.pptx'), str(scripts)]
    record = {
        'operation': 'PREPARE_HISTORICAL_V10_REPLAY',
        'snapshot': str(snapshot),
        'workdir': str(stage),
        'runtime_root': str(runtime),
        'skill_dir': str(skill),
        'adapted_execution_copy_only': True,
        'command': command,
        'expected_output_directory': str(stage / 'build/output'),
        'generation_requested': args.run,
    }
    (stage / 'reproduction-inputs.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n')
    print(shlex.join(command))
    if args.run:
        subprocess.run(command, check=True)


if __name__ == '__main__':
    main()
