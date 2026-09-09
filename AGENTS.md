# VIA Engineering Rules

## Source of truth
- The Git repository is the persistent engineering source of truth.
- Codex sessions are disposable execution contexts.

## Architecture
- Do not modify an approved requirements baseline unless explicitly requested.
- Record architecture decisions under docs/architecture/ and docs/adr/.
- Store prototypes under prototypes/.
- Store benchmark implementation under benchmark/.
- Store experiment results under results/.

## Development
- Inspect git status before modifying files.
- Use the project Python virtual environment in .venv/.
- Run relevant tests or benchmarks after implementation.
- Review git diff before considering work complete.

## Git safety
- Never commit secrets, API keys, credentials, or .env files.
- Do not commit or push unless explicitly requested.
