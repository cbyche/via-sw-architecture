"""Explicit authorized API probe; not a QA sample."""
import argparse
import asyncio
from pathlib import Path
from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence
from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient
from prototypes.candidates.dp41_45_poc.shared.memory import task_memory


async def main(directory):
    evidence = Evidence(directory)
    try:
        print({"memory": task_memory()})
        for provider in ("qwen", "openai"):
            client = ModelClient(evidence, provider, max_cost_usd=.10)
            result = await client.call([{"role": "system", "content": "Return JSON only."},
                                        {"role": "user", "content": 'Return {"ok":true}.'}],
                                       "SETUP_PROBE", request_id="probe-" + provider, max_tokens=64)
            print({"provider": provider, "model": client.model_id, "response": result,
                   "standard_price_usage_usd": client.spent})
    finally:
        evidence.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()
    asyncio.run(main(arguments.output))
