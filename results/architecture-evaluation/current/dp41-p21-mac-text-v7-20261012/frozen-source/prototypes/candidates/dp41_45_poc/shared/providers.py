"""Actual Qwen and OpenAI requests with durable wire, usage and budget records."""
from __future__ import annotations

import asyncio
import json
import os
import subprocess
import time
import urllib.error
import urllib.request
import uuid
import fcntl
from pathlib import Path
from collections import defaultdict

from .evidence import canonical, digest
from .adapters import ProviderWireAdapter, AdapterError


PRICES = {
    "qwen": {"input": .375, "output": 2.25, "cached": .375,
             "source": "https://www.alibabacloud.com/help/en/model-studio/model-pricing",
             "date": "2026-10-12", "region": "Singapore/International"},
    "openai": {"input": .10, "output": .50, "cached": .01, "cache_write": .125,
               "source": "https://developers.openai.com/api/docs/models/gpt-6-luna",
               "date": "2026-10-12", "tier": "default/standard"},
}


def credential(environment_name, keychain_service):
    value = os.environ.get(environment_name)
    if value:
        return value.strip()
    result = subprocess.run(["/usr/bin/security", "find-generic-password", "-s", keychain_service, "-w"],
                            capture_output=True, timeout=15)
    if result.returncode:
        raise RuntimeError(f"Credential unavailable: {keychain_service} (no secret printed)")
    return result.stdout.decode().strip()


def parse_json(text):
    text = text.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1].rsplit("```", 1)[0].strip()
    value = json.loads(text)
    if not isinstance(value, dict):
        raise ValueError("Model output must be a JSON object")
    return value


class ModelClient:
    def __init__(self, evidence, provider="qwen", *, max_cost_usd=100., max_concurrency=4, budget_file=None):
        self.evidence = evidence
        self.provider = provider
        self.key = credential("DASHSCOPE_API_KEY" if provider == "qwen" else "OPENAI_API_KEY",
                              "via-dashscope-api-key" if provider == "qwen" else "via-openai-api-key")
        if provider == "qwen":
            workspace = credential("DASHSCOPE_WORKSPACE_ID", "via-dashscope-workspace-id")
            base = os.environ.get("VIA_QWEN_BASE_URL", f"https://{workspace}.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1")
            self.url, self.model_id = base.rstrip("/") + "/chat/completions", "qwen3.6-35b-a3b"
        else:
            self.url, self.model_id = "https://api.openai.com/v1/responses", "gpt-6-luna"
        self.limit, self.spent, self.reserved = max_cost_usd, 0., 0.
        self.semaphore = asyncio.Semaphore(max_concurrency)
        self.request_calls = defaultdict(int)
        self.budget_file = Path(budget_file) if budget_file else None
        self.adapter = ProviderWireAdapter(provider, self.model_id)
        self.free_slots = list(range(max_concurrency))
        self.slot_parents = {}

    def global_budget(self, call_id, amount, *, terminal=False):
        if self.budget_file is None:
            return
        self.budget_file.parent.mkdir(parents=True, exist_ok=True)
        with self.budget_file.open("a+", encoding="utf-8") as handle:
            fcntl.flock(handle, fcntl.LOCK_EX)
            handle.seek(0)
            content = handle.read()
            budget = json.loads(content) if content else {"provider": self.provider, "limit_usd": 100., "calls": {}}
            if not terminal and sum(budget["calls"].values()) + amount > budget["limit_usd"]:
                raise RuntimeError(f"{self.provider} campaign $100 hard cap would be exceeded")
            budget["calls"][call_id] = amount
            handle.seek(0)
            handle.truncate()
            handle.write(canonical(budget))
            handle.flush()
            os.fsync(handle.fileno())
            fcntl.flock(handle, fcntl.LOCK_UN)

    def _request(self, body):
        request = urllib.request.Request(self.url, data=canonical(body).encode(), method="POST",
                                         headers={"Authorization": "Bearer " + self.key,
                                                  "Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(request, timeout=90) as response:
                return json.load(response), response.status, dict(response.headers)
        except urllib.error.HTTPError as error:
            payload = error.read().decode(errors="replace")
            return {"http_error": error.code, "body": payload}, error.code, {}

    async def call(self, messages, role="CINTERPRET", *, request_id="", case_id="", max_tokens=4096,
                   purpose="FOREGROUND", enforce_request_budget=True,
                   causal_timeline=None, causal_parents=None, causal_callback=None, reasoning_effort="none"):
        body = self.adapter.encode(messages, max_tokens=max_tokens)
        if self.provider == "openai":
            body["reasoning"] = {"effort": reasoning_effort}
        price = PRICES[self.provider]
        next_parents = list(causal_parents or [])
        # UTF-8 bytes conservatively bound ordinary token counts; max output bounds the remaining charge.
        for attempt in range(3):
            reserve = (len(canonical(body).encode()) * price["input"] + max_tokens * price["output"]) / 1e6
            if enforce_request_budget:
                self.request_calls[request_id] += 1
                if self.request_calls[request_id] > 8:
                    raise RuntimeError("Logical Request model-call budget exhausted")
            if self.spent + self.reserved + reserve > self.limit:
                raise RuntimeError(f"{self.provider} hard budget exhausted before request")
            call_id = str(uuid.uuid4())
            self.global_budget(call_id, reserve)
            self.reserved += reserve
            enqueued = time.monotonic_ns()
            context = self.evidence.artifact("model-contexts", call_id, body)
            record = {"call_id": call_id, "request_id": request_id, "case_id": case_id,
                      "provider": self.provider, "model": self.model_id, "role": role,
                      "purpose": purpose, "attempt": attempt + 1, "enqueue_ns": enqueued,
                      "context": context, "context_sha256": digest(messages), "price": price,
                      "endpoint_host_redacted": self.provider, "billing_item_id": call_id}
            self.evidence.write("model-call-starts", record)
            enqueue_node = causal_timeline.point("MODEL_ENQUEUE", at_s=(enqueued-self.evidence.origin)/1e9,
                parents=next_parents, evidence=[call_id], request_id=request_id) if causal_timeline else None
            charged = False
            cancelled = False
            sent = None
            try:
                async with self.semaphore:
                    slot = self.free_slots.pop(0)
                    sent = time.monotonic_ns()
                    network_job = asyncio.create_task(asyncio.to_thread(self._request, body))
                    try:
                        try:
                            response, status, headers = await asyncio.shield(network_job)
                        except asyncio.CancelledError:
                            # A local cancellation cannot cancel an HTTP request.
                            cancelled = True
                            response, status, headers = await asyncio.shield(network_job)
                    finally:
                        terminal = time.monotonic_ns()
                        if causal_timeline:
                            parents = [enqueue_node]
                            previous = self.slot_parents.get(slot)
                            if previous and previous[0] is causal_timeline:
                                parents.append(previous[1])
                            network_node = causal_timeline.span("MODEL_NETWORK", "CLOUD_MODEL",
                                (sent-self.evidence.origin)/1e9, (terminal-self.evidence.origin)/1e9,
                                parents=parents, evidence=[call_id], request_id=request_id,
                                metadata={"role": role, "slot": slot, "attempt": attempt+1})
                            self.slot_parents[slot] = (causal_timeline, network_node)
                            next_parents = [network_node]
                            if causal_callback:
                                causal_callback(network_node)
                        self.free_slots.append(slot)
                output = self.evidence.artifact("model-outputs", call_id, response)
                decoded = None
                decode_error = None
                if status == 200:
                    try:
                        decoded = self.adapter.decode(response, headers)
                    except AdapterError as error:
                        decode_error = str(error)
                usage = decoded["usage"] if decoded is not None else None
                cost = None
                if usage is not None:
                    input_tokens = usage["input_tokens"]
                    output_tokens = usage["output_tokens"]
                    details = usage["input_tokens_details"]
                    cached = details.get("cached_tokens", 0)
                    cache_writes = details.get("cache_write_tokens", 0)
                    cost = ((input_tokens - cached - cache_writes) * price["input"] + cached * price["cached"]
                            + cache_writes * price.get("cache_write", price["input"]) + output_tokens * price["output"]) / 1e6
                    self.spent += cost
                else:
                    # An ambiguous request remains provisioned at its conservative bound.
                    self.spent += reserve
                self.reserved -= reserve
                charged = True
                self.global_budget(call_id, cost if cost is not None else reserve, terminal=True)
                terminal_record = {**record, "send_ns": sent, "terminal_ns": terminal,
                                   "first_token_ns": None, "first_token_observation": "NON_STREAMING_NOT_OBSERVED",
                                   "http_status": status, "provider_request_id": response.get("id") or headers.get("x-request-id"),
                                   "usage": usage, "raw_usage": response.get("usage"),
                                   "usage_state": "PROVIDER_REPORTED" if usage is not None else "UNKNOWN",
                                   "cost_usd": cost, "cost_basis": "PROVIDER_USAGE_X_STANDARD_PRICE",
                                   "billed_usd": None, "invoice_state": "NOT_INVOICE_RECONCILED",
                                   "status": "DISCARDED_AFTER_LOCAL_CANCEL" if cancelled else "COMPLETED" if status == 200 else "HTTP_ERROR",
                                   "discard_reason": "LOCAL_CONTINUATION_CANCELLED" if cancelled else None,
                                   "causal_node_id": next_parents[-1] if causal_timeline and next_parents else None,
                                   "decode_error": decode_error,
                                   "output": output, "evidence_level": "MEASURED_MODEL"}
                self.evidence.write("model-calls", terminal_record)
                self.evidence.write("billing", terminal_record)
                if cancelled:
                    raise asyncio.CancelledError
                if decode_error:
                    raise AdapterError(decode_error)
                if status != 200:
                    if status in (429, 500, 502, 503) and attempt < 2:
                        await asyncio.sleep(.5 * (attempt + 1))
                        continue
                    raise RuntimeError(f"{self.provider} HTTP {status}; saved response {output['path']}")
                text = decoded["text"]
                try:
                    return parse_json(text)
                except (ValueError, json.JSONDecodeError) as error:
                    self.evidence.write("model-validation", {"call_id": call_id, "error": str(error),
                                                            "disposition": "RETRY" if attempt < 2 else "FAILED"})
                    if attempt == 2:
                        raise
                    body = {**body}
                    # The identical semantic inputs are retained; repair only asks for the required encoding.
                    if self.provider == "qwen":
                        body["messages"] = [*messages, {"role": "assistant", "content": text},
                                            {"role": "user", "content": "Return the same answer as a valid JSON object only."}]
                    else:
                        body["input"] = [*messages, {"role": "assistant", "content": text},
                                        {"role": "user", "content": "Return the same answer as a valid JSON object only."}]
            except (urllib.error.URLError, TimeoutError, OSError) as error:
                if not charged:
                    self.reserved -= reserve
                    self.spent += reserve
                error_record = {**record, "status": "TRANSPORT_ERROR", "send_ns": sent, "terminal_ns": time.monotonic_ns(),
                                                   "usage": None, "usage_state": "UNKNOWN", "cost_usd": None,
                                                   "reserved_upper_bound_usd": reserve, "error": type(error).__name__}
                self.evidence.write("model-calls", error_record)
                self.evidence.write("billing", error_record)
                if attempt == 2:
                    raise
            except asyncio.CancelledError:
                if not charged:
                    # No request was dispatched if cancellation happened in the queue.
                    self.reserved -= reserve
                    self.global_budget(call_id, 0., terminal=True)
                    self.evidence.write("model-calls", {**record, "send_ns": None,
                        "terminal_ns": time.monotonic_ns(), "status": "CANCELLED_BEFORE_SEND",
                        "usage": {"input_tokens": 0, "output_tokens": 0}, "cost_usd": 0.,
                        "evidence_level": "MEASURED_MODEL", "discard_reason": "QUEUE_CANCELLED"})
                raise
