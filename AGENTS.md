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
- For repository-scoped change/build tasks on a non-default feature branch, commit and push the completed work by default when relevant checks pass, the diff is scoped, and no unrelated user changes would be included.
- Do not commit or push review-only, diagnosis-only, exploratory, incomplete, or failing work unless the user explicitly asks for that exact state.
- Do not commit or push when the worktree contains unrelated user changes that cannot be safely separated.
- Never force-push, rewrite published history, amend another author's commit, or push directly to a protected/default branch unless explicitly requested.
- Honor an explicit user request to leave changes uncommitted or not push.
- After pushing, report the commit SHA and CI status when CI is available.
