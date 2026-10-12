"""Stable local ports for replaceable external wire and question ownership.

Adapters preserve values and authority. They never repair candidate meaning or
infer a consent/answer from a question that was not actually presented.
"""
from __future__ import annotations

import copy
from datetime import date, datetime
from zoneinfo import ZoneInfo


class AdapterError(ValueError):
    pass


def _positive_revision(value):
    if isinstance(value, bool) or not isinstance(value, int) or value < 1:
        raise AdapterError("revision must be a positive integer")
    return value


class ProviderWireAdapter:
    """Only API serialization/normalization; network, price and usage ledger stay outside."""

    def __init__(self, provider, model_id):
        if provider not in ("qwen", "openai"):
            raise AdapterError("unsupported provider wire")
        self.provider, self.model_id = provider, model_id

    def encode(self, messages, *, max_tokens=4096):
        messages = copy.deepcopy(messages)
        if self.provider == "qwen":
            return {"model": self.model_id, "messages": messages, "temperature": 0,
                    "max_tokens": max_tokens, "enable_thinking": False,
                    "response_format": {"type": "json_object"}}
        return {"model": self.model_id, "input": messages, "max_output_tokens": max_tokens,
                "reasoning": {"effort": "none"}, "store": False,
                "text": {"format": {"type": "json_object"}}}

    def decode(self, response, headers=None):
        if not isinstance(response, dict):
            raise AdapterError("provider response must be an object")
        raw_usage = response.get("usage")
        usage = None
        if raw_usage is not None:
            input_tokens = raw_usage.get("prompt_tokens", raw_usage.get("input_tokens"))
            output_tokens = raw_usage.get("completion_tokens", raw_usage.get("output_tokens"))
            if input_tokens is None or output_tokens is None:
                raise AdapterError("provider usage omits input/output token counts")
            details = raw_usage.get("prompt_tokens_details", raw_usage.get("input_tokens_details", {})) or {}
            usage = {"input_tokens": input_tokens, "output_tokens": output_tokens,
                     "input_tokens_details": {"cached_tokens": details.get("cached_tokens", 0),
                                              "cache_write_tokens": details.get("cache_write_tokens", 0)}}
        if self.provider == "qwen":
            try:
                text = response["choices"][0]["message"].get("content") or ""
            except (KeyError, IndexError, TypeError) as error:
                raise AdapterError("provider response omits choice content") from error
        else:
            if not isinstance(response.get("output"), list):
                raise AdapterError("provider response omits output records")
            text = "".join(content.get("text", "") for item in response["output"]
                           for content in item.get("content", []) if content.get("type") == "output_text")
        if not isinstance(text, str):
            raise AdapterError("provider text is not a string")
        return {"text": text, "usage": usage, "raw_usage": copy.deepcopy(raw_usage),
                "provider_request_id": response.get("id") or (headers or {}).get("x-request-id")}


class AgentWireAdapter:
    """Lossless baseline command/receipt transport, including cancellation uncertainty."""

    protocol = "agent-wire-v1"

    def _validate(self, value, *, command):
        if not isinstance(value, dict):
            raise AdapterError("normalized Agent envelope must be an object")
        for key in ("command_id", "task_id", "execution_id"):
            if not isinstance(value.get(key), str) or not value[key]:
                raise AdapterError(f"Agent envelope requires {key}")
        _positive_revision(value.get("request_revision"))
        if command:
            if not isinstance(value.get("operation"), str) or not isinstance(value.get("payload"), dict):
                raise AdapterError("Agent command requires original operation/payload")
        elif not isinstance(value.get("status"), str) or not value["status"]:
            raise AdapterError("Agent receipt requires original status")
        return copy.deepcopy(value)

    def encode_command(self, normalized):
        return {"protocol": self.protocol, "command": self._validate(normalized, command=True)}

    def decode_command(self, wire):
        if not isinstance(wire, dict) or wire.get("protocol") != self.protocol:
            raise AdapterError("Agent command protocol mismatch")
        return self._validate(wire.get("command"), command=True)

    def encode_receipt(self, normalized):
        return {"protocol": self.protocol, "receipt": self._validate(normalized, command=False)}

    def decode_receipt(self, wire):
        if not isinstance(wire, dict) or wire.get("protocol") != self.protocol:
            raise AdapterError("Agent receipt protocol mismatch")
        return self._validate(wire.get("receipt"), command=False)


class CalendarSourceAdapter:
    """Normalize declared date/time fields without interpreting free-form calendar text."""

    def __init__(self, default_timezone="Asia/Seoul"):
        self.default_timezone = default_timezone

    @staticmethod
    def _time(value, day, timezone):
        if value is None:
            return None
        if not isinstance(value, str):
            raise AdapterError("calendar time must be a string")
        try:
            instant = datetime.fromisoformat(value) if "T" in value else datetime.fromisoformat(f"{day}T{value}")
        except ValueError as error:
            raise AdapterError("invalid calendar date/time") from error
        if instant.tzinfo is None:
            instant = instant.replace(tzinfo=ZoneInfo(timezone))
        return instant.astimezone(ZoneInfo(timezone)).isoformat()

    def normalize(self, raw_source):
        result = copy.deepcopy(raw_source)
        if result.get("kind") != "CALENDAR":
            return result
        if not isinstance(result.get("source_id"), str) or not result["source_id"]:
            raise AdapterError("calendar requires stable source ID")
        _positive_revision(result.get("revision"))
        if not isinstance(result.get("text"), str):
            raise AdapterError("calendar original text is required")
        metadata = result.setdefault("metadata", {})
        if not isinstance(metadata, dict):
            raise AdapterError("calendar metadata must be an object")
        day = metadata.get("date")
        try:
            date.fromisoformat(day)
        except (ValueError, TypeError) as error:
            raise AdapterError("calendar date must be explicit ISO date") from error
        timezone = metadata.get("timezone", metadata.get("time_zone", self.default_timezone))
        ZoneInfo(timezone)
        metadata["timezone"] = timezone
        metadata["start"] = self._time(metadata.get("start"), day, timezone)
        metadata["end"] = self._time(metadata.get("end"), day, timezone)
        metadata["state"] = metadata.get("state", metadata.get("interval_state"))
        for event in metadata.get("events", []):
            event["start"] = self._time(event.get("start"), day, timezone)
            event["end"] = self._time(event.get("end"), day, timezone)
            if event["start"] is not None and event["end"] is not None and event["end"] < event["start"]:
                raise AdapterError("calendar event ends before it starts")
        return result


class QuestionAuthority:
    """Task owner baseline: identity/revision/scope checks, without the future M05 token."""

    def issue(self, question, *, task_id, execution_id=None):
        result = copy.deepcopy(question)
        if result.get("task_id", task_id) != task_id:
            raise AdapterError("question belongs to a different Task")
        if not isinstance(result.get("question_id"), str) or not result["question_id"]:
            raise AdapterError("question ID is required")
        _positive_revision(result.get("revision"))
        result["task_id"] = task_id
        if execution_id is not None:
            if result.get("execution_id", execution_id) != execution_id:
                raise AdapterError("question belongs to a different Execution")
            result["execution_id"] = execution_id
        return result

    def validate_answer(self, question, answer, *, task_id, execution_id=None, consume=False):
        reason = "ACCEPTED"
        if not isinstance(question, dict) or not isinstance(answer, dict):
            return {"accepted": False, "reason": "INVALID_QUESTION_OR_ANSWER"}
        qid = question.get("question_id")
        if question.get("task_id") != task_id:
            reason = "WRONG_TASK"
        elif execution_id is not None and question.get("execution_id") != execution_id:
            reason = "WRONG_EXECUTION"
        elif answer.get("question_ref", answer.get("question_id")) != qid:
            reason = "WRONG_QUESTION"
        elif answer.get("question_revision") != question.get("revision"):
            reason = "STALE_REVISION"
        elif question.get("status") != "VALID" or question.get("answer_status") == "ACCEPTED":
            reason = "NOT_CURRENT_OR_ALREADY_ANSWERED"
        elif question.get("delivered_receipt") is not True:
            reason = "NOT_ACTUALLY_PRESENTED"
        else:
            scope = question.get("scope", {})
            allowed = set(scope.get("allowed_properties", []))
            if scope.get("property"):
                allowed.add(scope["property"])
            properties = answer.get("properties", {})
            if not isinstance(properties, dict) or set(properties) - allowed:
                reason = "PROPERTY_OUTSIDE_QUESTION_SCOPE"
            elif "answer" not in answer:
                reason = "ANSWER_MISSING"
        accepted = reason == "ACCEPTED"
        if accepted and consume:
            question["answer_status"] = "ACCEPTED"
            question["status"] = "ANSWERED"
            question["answer"] = copy.deepcopy(answer["answer"])
        return {"accepted": accepted, "reason": reason, "question_id": qid,
                "question_revision": question.get("revision"), "task_id": task_id}


class PresentedQuestionBinding:
    """Dialogue owner returns only an actually presented question identity/scope."""

    def __init__(self):
        self.presented = {}

    def present(self, question, *, receipt):
        if not question.get("question_id") or not question.get("text"):
            raise AdapterError("presented question requires identity and text")
        _positive_revision(question.get("revision"))
        text_presented = receipt.get("text_presented") is True or receipt.get("text_presented_s") is not None
        voice_presented = receipt.get("voice_presented") is True or receipt.get("voice_onset_s") is not None
        if not text_presented or receipt.get("voice_on", True) and not voice_presented:
            raise AdapterError("question lacks actual required presentation receipt")
        if not receipt.get("publication_id"):
            raise AdapterError("presentation requires a publication receipt ID")
        key = question["question_id"]
        self.presented[key] = {"question": copy.deepcopy(question), "receipt": copy.deepcopy(receipt)}
        return copy.deepcopy(self.presented[key])

    def reflect_answer(self, question_id, revision, *, answer, task_id, properties=None):
        record = self.presented.get(question_id)
        if not record:
            raise AdapterError("question was not actually presented")
        question = record["question"]
        if question.get("revision") != revision or question.get("task_id") != task_id:
            raise AdapterError("presented question revision/Task mismatch")
        return {"question_ref": question_id, "question_revision": revision,
                "answer": copy.deepcopy(answer), "properties": copy.deepcopy(properties or {})}

    def withdraw(self, question_id, revision):
        current = self.presented.get(question_id)
        if current and current["question"].get("revision") == revision:
            del self.presented[question_id]

    def snapshot(self):
        return copy.deepcopy(self.presented)

    def restore(self, snapshot):
        restored = PresentedQuestionBinding()
        for key, record in snapshot.items():
            if key != record.get("question", {}).get("question_id"):
                raise AdapterError("restored presentation identity mismatch")
            restored.present(record["question"], receipt=record["receipt"])
        self.presented = restored.presented
