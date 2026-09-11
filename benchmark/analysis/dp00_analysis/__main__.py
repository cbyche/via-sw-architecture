"""DP-00 analysis command-line interface."""

from __future__ import annotations

import argparse
import json

from .diagnostics import derive_summary
from .loader import load_evidence
from .serialization import read_summary, write_summary


def main() -> int:
    parser = argparse.ArgumentParser(prog="dp00_analysis")
    sub = parser.add_subparsers(dest="command", required=True)
    validate = sub.add_parser("validate")
    validate.add_argument("raw_root")
    derive = sub.add_parser("derive")
    derive.add_argument("raw_root")
    derive.add_argument("--output", required=True)
    summary = sub.add_parser("summary")
    summary.add_argument("derived_root")
    args = parser.parse_args()
    if args.command == "summary":
        print(json.dumps(read_summary(args.derived_root), indent=2, sort_keys=True))
        return 0
    evidence = load_evidence(args.raw_root)
    result = derive_summary(evidence)
    if args.command == "validate":
        print(json.dumps(result["validation"], indent=2, sort_keys=True))
    else:
        destination = write_summary(args.output, result)
        print(destination)
    return 0 if evidence.validation.valid else 1


if __name__ == "__main__":
    raise SystemExit(main())
