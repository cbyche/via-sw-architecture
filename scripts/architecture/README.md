# Architecture Repository Checks

이 디렉터리는 active Architecture 문서와 저장소 구조의 정합성을 검사하는 유지보수 도구를 둔다. Benchmark campaign runner나 result generator의 active 위치가 아니다.

## Current checks

```bash
.venv/bin/python scripts/architecture/check_active_markdown_links.py
.venv/bin/python scripts/architecture/check_active_terminology.py
```

- `check_active_markdown_links.py`는 active Markdown의 local link와 archive 경계 문제를 찾는다.
- `check_active_terminology.py`는 legacy metric ID과 과거 lifecycle 번호가 active 문서에 다시 섞이지 않는지 확인한다.

새 active README나 contract directory를 추가하면 link checker의 scan scope도 함께 검토한다. 이전 W12-G1 measurement/review scripts는 [script archive](../archive/README.md)에 보존한다.
