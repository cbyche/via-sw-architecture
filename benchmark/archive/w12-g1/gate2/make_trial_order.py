"""Generate deterministic balanced A/B order without running candidates."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def blocks(count_each: int) -> list[str]:
    if count_each % 2:
        raise ValueError("count_each must be even so four-trial blocks stay balanced")
    out: list[str] = []
    patterns = ("ABBA", "BAAB")
    for index in range(count_each // 2):
        out.extend(patterns[index % 2])
    assert out.count("A") == count_each
    assert out.count("B") == count_each
    return out


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--warmup-each", type=int, default=10)
    parser.add_argument("--scored-each", type=int, default=100)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    payload = {
        "status": "ORDER_ONLY_BENCHMARK_NOT_RUN",
        "warmup": blocks(args.warmup_each),
        "scored": blocks(args.scored_each),
    }
    text = json.dumps(payload, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(text, encoding="utf-8")
    print(text, end="")


if __name__ == "__main__":
    main()
