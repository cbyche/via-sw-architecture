"""Wire/owner regression fixtures, without API calls or performance measurements."""
import copy
import unittest

from prototypes.candidates.dp41_45_poc.shared.adapters import (
    AdapterError, AgentWireAdapter, CalendarSourceAdapter, PresentedQuestionBinding,
    ProviderWireAdapter, QuestionAuthority,
)


def command(**changes):
    record = {"command_id": "CMD1", "task_id": "T1", "execution_id": "X1", "request_revision": 3,
              "operation": "SEND_MAIL", "payload": {"recipient": "wrong@example.invalid", "body": "wrong body"}}
    record.update(changes)
    return record


def question(**changes):
    record = {"question_id": "Q1", "revision": 7, "task_id": "T1", "execution_id": "X1",
              "status": "VALID", "text": "기간을 상반기로 할까요?", "delivered_receipt": True,
              "scope": {"property": "PERIOD", "proposed_value": "2026-H1"}}
    record.update(changes)
    return record


class AdapterTests(unittest.TestCase):
    def test_qwen_wire_round_trip_preserves_context_output_and_usage(self):
        adapter = ProviderWireAdapter("qwen", "qwen-fixture")
        messages = [{"role": "user", "content": [{"type": "text", "text": "자료"},
                                                   {"type": "image_url", "image_url": {"url": "fixture://pixels"}}]}]
        wire = adapter.encode(messages, max_tokens=700)
        self.assertEqual(wire["messages"], messages)
        self.assertEqual(wire["response_format"], {"type": "json_object"})
        self.assertEqual(wire["max_tokens"], 700)
        wire["messages"][0]["content"][0]["text"] = "changed"
        self.assertEqual(messages[0]["content"][0]["text"], "자료")
        response = {"id": "response1", "choices": [{"message": {"content": '{"target":"wrong"}'}}],
                    "usage": {"prompt_tokens": 100, "completion_tokens": 20,
                              "prompt_tokens_details": {"cached_tokens": 30}}}
        result = adapter.decode(response)
        self.assertEqual(result["text"], '{"target":"wrong"}')
        self.assertEqual(result["usage"]["input_tokens"], 100)
        self.assertEqual(result["usage"]["output_tokens"], 20)
        self.assertEqual(result["usage"]["input_tokens_details"]["cached_tokens"], 30)
        self.assertEqual(result["raw_usage"], response["usage"])

    def test_openai_wire_uses_actual_response_segments_and_unknown_usage_stays_unknown(self):
        adapter = ProviderWireAdapter("openai", "openai-fixture")
        wire = adapter.encode([{"role": "user", "content": "hello"}], max_tokens=64)
        self.assertFalse(wire["store"])
        self.assertEqual(wire["max_output_tokens"], 64)
        result = adapter.decode({"output": [{"content": [{"type": "output_text", "text": "a"},
                                                          {"type": "refusal", "refusal": "unrelated"},
                                                          {"type": "output_text", "text": "b"}]}]},
                                {"x-request-id": "transport1"})
        self.assertEqual(result["text"], "ab")
        self.assertIsNone(result["usage"])
        self.assertEqual(result["provider_request_id"], "transport1")

    def test_invalid_provider_usage_is_not_normalized_to_zero(self):
        adapter = ProviderWireAdapter("qwen", "fixture")
        with self.assertRaisesRegex(AdapterError, "omits input/output"):
            adapter.decode({"usage": {}, "choices": [{"message": {"content": "x"}}]})
        with self.assertRaises(AdapterError):
            adapter.decode({"usage": None, "choices": []})

    def test_agent_wire_preserves_wrong_candidate_payload_and_uncertain_cancel(self):
        adapter = AgentWireAdapter()
        candidate = command()
        wire = adapter.encode_command(candidate)
        self.assertEqual(adapter.decode_command(wire), candidate)
        wire["command"]["payload"]["recipient"] = "modified@example.invalid"
        self.assertEqual(candidate["payload"]["recipient"], "wrong@example.invalid")
        receipt = {"command_id": "CMD1", "task_id": "T1", "execution_id": "X1", "request_revision": 3,
                   "status": "CANCEL_ACCEPTED_TERMINATION_UNKNOWN", "terminated": False}
        self.assertEqual(adapter.decode_receipt(adapter.encode_receipt(receipt)), receipt)
        with self.assertRaises(AdapterError):
            adapter.decode_receipt({"protocol": "wrong", "receipt": receipt})

    def test_agent_requires_original_routing_and_payload(self):
        adapter = AgentWireAdapter()
        for field in ("command_id", "task_id", "execution_id", "request_revision", "payload", "operation"):
            missing = command()
            del missing[field]
            with self.assertRaises(AdapterError):
                adapter.encode_command(missing)

    def test_calendar_preserves_text_permission_version_and_normalizes_declared_times(self):
        raw = {"source_id": "Cal13", "revision": 2, "kind": "CALENDAR", "text": "original unmodified statement",
               "permission": {"read": "DENIED"}, "metadata": {"date": "2026-10-13", "interval_state": "BUSY",
                                                          "events": [{"start": "09:00", "end": "09:30", "title": "회의"}]}}
        normalized = CalendarSourceAdapter().normalize(raw)
        self.assertEqual(normalized["source_id"], "Cal13")
        self.assertEqual(normalized["revision"], 2)
        self.assertEqual(normalized["text"], raw["text"])
        self.assertEqual(normalized["permission"], raw["permission"])
        self.assertEqual(normalized["metadata"]["events"][0]["start"], "2026-10-13T09:00:00+09:00")
        self.assertEqual(normalized["metadata"]["state"], "BUSY")
        self.assertIsNone(normalized["metadata"]["start"])
        self.assertEqual(raw["metadata"]["events"][0]["start"], "09:00")

    def test_calendar_converts_explicit_timezone_without_inventing_state_from_text(self):
        raw = {"source_id": "C", "revision": 1, "kind": "CALENDAR", "text": "FREE",
               "metadata": {"date": "2026-10-13", "timezone": "Asia/Seoul",
                            "start": "2026-10-13T00:00:00+00:00", "end": "2026-10-13T01:00:00+00:00"}}
        result = CalendarSourceAdapter().normalize(raw)
        self.assertEqual(result["metadata"]["start"], "2026-10-13T09:00:00+09:00")
        self.assertIsNone(result["metadata"]["state"])

    def test_question_authority_checks_task_revision_scope_and_duplicate(self):
        authority = QuestionAuthority()
        q = authority.issue(question(), task_id="T1", execution_id="X1")
        self.assertNotIn("scope_token", q)
        answer = {"question_ref": "Q1", "question_revision": 7, "answer": "네", "properties": {"PERIOD": "2026-H1"}}
        self.assertTrue(authority.validate_answer(q, answer, task_id="T1", execution_id="X1")["accepted"])
        for update, task in [({"question_revision": 6}, "T1"), ({}, "T2"), ({"properties": {"FORMAT": "TABLE"}}, "T1")]:
            self.assertFalse(authority.validate_answer(q, {**answer, **update}, task_id=task)["accepted"])
        self.assertTrue(authority.validate_answer(q, answer, task_id="T1", consume=True)["accepted"])
        self.assertFalse(authority.validate_answer(q, answer, task_id="T1")["accepted"])

    def test_question_authority_rejects_unpresented_or_wrong_owner(self):
        authority = QuestionAuthority()
        args = {"question_ref": "Q1", "question_revision": 7, "answer": "yes"}
        self.assertFalse(authority.validate_answer(question(delivered_receipt=False), args, task_id="T1")["accepted"])
        with self.assertRaises(AdapterError):
            authority.issue(question(), task_id="T2")

    def test_presented_binding_only_actual_required_delivery_and_restore(self):
        binding = PresentedQuestionBinding()
        receipt = {"publication_id": "P1", "text_presented_s": 0, "voice_on": True, "voice_onset_s": .3}
        with self.assertRaises(AdapterError):
            binding.present(question(), receipt={"publication_id": "pending", "text_presented_s": 0, "voice_on": True})
        binding.present(question(), receipt=receipt)
        args = binding.reflect_answer("Q1", 7, task_id="T1", answer="yes", properties={"PERIOD": "2026-H1"})
        self.assertEqual(args["question_ref"], "Q1")
        self.assertNotIn("scope_token", args)
        for revision, task in [(6, "T1"), (7, "T2")]:
            with self.assertRaises(AdapterError):
                binding.reflect_answer("Q1", revision, task_id=task, answer="yes")
        restored = PresentedQuestionBinding()
        restored.restore(binding.snapshot())
        self.assertEqual(restored.reflect_answer("Q1", 7, task_id="T1", answer="yes", properties={"PERIOD": "2026-H1"}), args)
        restored.withdraw("Q1", 7)
        with self.assertRaises(AdapterError):
            restored.reflect_answer("Q1", 7, task_id="T1", answer="yes")

    def test_voice_off_binding_needs_text_and_publication_identity(self):
        binding = PresentedQuestionBinding()
        binding.present(question(), receipt={"publication_id": "P1", "text_presented_s": 1, "voice_on": False})
        with self.assertRaises(AdapterError):
            binding.present(question(), receipt={"text_presented_s": 1, "voice_on": False})


if __name__ == "__main__":
    unittest.main()
