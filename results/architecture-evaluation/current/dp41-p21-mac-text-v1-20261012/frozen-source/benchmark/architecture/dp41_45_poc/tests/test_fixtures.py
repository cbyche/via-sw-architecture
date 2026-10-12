"""Checks for the frozen-source fixture/oracle boundary, not model quality."""

import copy
import json
import unittest

from benchmark.architecture.dp41_45_poc.fixtures import (
    build_activity_baseline,
    build_observation_buffers,
    load_cases,
    load_manifest,
    load_oracles,
    public_inputs,
    observation_buffer_records,
    validate_fixtures,
)
from prototypes.candidates.dp41_45_poc.shared.owners import OwnerRepository, identity, reference


class FixtureContractTests(unittest.TestCase):
    def setUp(self):
        self.cases = {case["case_id"]: case for case in load_cases()}

    def test_canonical_membership_provenance_and_observation_population(self):
        self.assertEqual([], validate_fixtures())
        self.assertEqual(21, len(self.cases))
        self.assertEqual(27, sum(len(case["reactions"]) for case in self.cases.values()))
        self.assertEqual(19, sum(case["cycle"] is not None for case in self.cases.values()))
        self.assertEqual(126, sum(len(o["conditions"]) for o in load_oracles().values()))

    def test_public_allowlist_hides_oracles_future_events_and_case_selection(self):
        case = copy.deepcopy(self.cases["S3-02"])
        case["oracle"] = {"secret_answer": "SECRET_ORACLE_SENTINEL"}
        case["events"].append({"type": "FUTURE_SENTINEL", "payload": "SECRET_FUTURE_SENTINEL"})
        public = public_inputs(case)
        encoded = json.dumps(public, ensure_ascii=False)
        self.assertNotIn("SECRET_ORACLE_SENTINEL", encoded)
        self.assertNotIn("SECRET_FUTURE_SENTINEL", encoded)
        self.assertNotIn("case_id", public)
        self.assertNotIn("events", public)
        self.assertNotIn("stub_profile", public)
        self.assertNotIn("canonical_doc_excerpt", encoded)
        public["initial"]["sources"][0]["text"] = "changed in candidate"
        self.assertNotEqual("changed in candidate", case["initial"]["sources"][0]["text"])

    def test_denied_amount_and_unconsented_contact_are_not_candidate_source_data(self):
        denied = public_inputs(self.cases["S6-03"])
        self.assertNotIn("500", json.dumps(denied, ensure_ascii=False))
        self.assertEqual("", denied["initial"]["sources"][0]["text"])
        self.assertEqual(500, load_oracles()["S6-03"]["private_source"]["contract_amount_million_krw"])
        contact = public_inputs(self.cases["S6-02"])
        self.assertNotIn("minsu@example.invalid", json.dumps(contact))
        self.assertNotIn("consent_source_release", contact)

    def test_unpresented_replacement_question_is_registered_without_presentation(self):
        event = self.cases["S6-02"]["events"][1]
        self.assertEqual("QUESTION_REGISTER", event["type"])
        self.assertFalse(event["payload"]["presentation_requested"])
        question = event["payload"]["question"]
        self.assertEqual("PRESENTATION_PENDING", question["presentation_status"])
        self.assertFalse(question["delivered_receipt"])
        self.assertEqual(-.1, event["at_seconds"])

    def test_actual_dependency_preparation_is_not_replaced_by_ready_snapshot(self):
        first = self.cases["S2-02"]
        second = self.cases["S2-04"]
        self.assertEqual([], first["initial"]["sources"])
        self.assertEqual(["P0", "P1", "P2", "P3", "P4"], [e["event_id"] for e in first["preparation"]["events"]])
        self.assertEqual(5, first["preparation"]["settle_seconds"])
        self.assertFalse(second["preparation"]["restore"])
        self.assertEqual(["S2-02"], second["preparation"]["depends_on"])
        self.assertEqual([], second["initial"]["sources"])

    def test_fixed_duplicate_and_stale_results_keep_identity_and_revision(self):
        events = self.cases["S5-02"]["events"]
        self.assertEqual(events[0]["event_id"], events[2]["event_id"])
        self.assertEqual(events[0]["payload"], events[2]["payload"])
        self.assertEqual(2, events[1]["payload"]["request_revision"])
        self.assertEqual(1, events[3]["payload"]["request_revision"])
        self.assertEqual([0, .02, .04, .06], [e["at_seconds"] for e in events])

    def test_clarification_and_reconnect_inputs_require_actual_delivery(self):
        for case_id in ("S6-01", "S6-02", "S5-03"):
            followup = self.cases[case_id]["followups"][0]
            self.assertTrue(followup["only_after_actual_receipt"])
            self.assertEqual(1, followup["wait_seconds"])
        self.assertIsNone(self.cases["S5-03"]["transcript"])
        self.assertEqual(2, self.cases["S5-03"]["followups"][0]["speech_seconds"])
        self.assertEqual(3, self.cases["S6-02"]["followups"][0]["speech_seconds"])

    def test_external_agent_questions_do_not_receive_invented_answers(self):
        for case_id in ("S5-01", "S5-02", "S5-04"):
            case = self.cases[case_id]
            self.assertEqual([], case["followups"])
            self.assertTrue(case["unexpected_question_reply"]["never_answer_agent_questions"])
            self.assertEqual("VIA_CLARIFICATION", case["unexpected_question_reply"]["only_for_origin"])
        self.assertGreater(self.cases["S1-01"]["unexpected_question_reply"]["speech_seconds"], 0)

    def test_old_interpretation_is_real_job_release_not_injected_meaning(self):
        case = self.cases["S4-01"]
        held = case["preparation"]["pending_interpretation"]
        self.assertTrue(held["require_actual_candidate_job"])
        self.assertEqual(2.1, held["hold_provider_return_until_seconds"])
        self.assertEqual("RELEASE_HELD_JOB", case["events"][0]["type"])
        self.assertTrue(case["events"][0]["payload"]["only_if_actual_job_exists"])
        self.assertNotIn("meaning", case["events"][0]["payload"])

    def test_final_tail_oracle_is_separate_from_stub_payload(self):
        case = self.cases["S3-02"]
        self.assertEqual(8, case["stub_profile"]["final_result_seconds"])
        self.assertNotIn("K현재", json.dumps(case["stub_profile"], ensure_ascii=False))
        self.assertNotIn("MailCurrent", json.dumps(public_inputs(case), ensure_ascii=False))
        self.assertEqual("MailCurrent@v1:SENT", load_oracles()["S3-02"]["final_tail"]["semantic_artifact_label"])

    def test_background_payload_uses_original_records_not_derived_relations(self):
        data = build_activity_baseline()
        spec = load_manifest()["initial_workload"]
        self.assertEqual(spec["history_turns"], len(data["interactions"]))
        self.assertEqual(spec["completed_tasks"], len(data["tasks"]))
        self.assertEqual(spec["context_sources"], len(data["sources"]))
        size = sum(len(s["text"].encode("utf-8")) for s in data["sources"])
        size += sum(len(i["text"].encode("utf-8")) for i in data["interactions"])
        size += sum(len(t["instruction"].encode("utf-8")) + len(t["result"]["text"].encode("utf-8")) for t in data["tasks"])
        self.assertEqual(20 * 1024 * 1024, size)
        self.assertTrue(all(s["metadata"].get("unpopulated") for s in data["sources"][20:]))

    def test_common_raw_observation_buffers_are_actual_distinct_touched_allocations(self):
        frames = build_observation_buffers()
        records = observation_buffer_records(frames)
        self.assertEqual(15, len(frames))
        self.assertEqual(15, len({id(frame) for frame in frames}))
        self.assertEqual(15, len({record["sha256"] for record in records}))
        self.assertEqual(15 * 2880 * 1800 * 4, sum(record["byte_count"] for record in records))
        self.assertTrue(all(record["contents_touched"] for record in records))
        self.assertTrue(all(record["width"] == 2880 and record["height"] == 1800 for record in records))
        self.assertEqual(bytes(records[4]["pixel_rgba"]), bytes(frames[4][:4]))
        self.assertEqual(bytes(records[4]["pixel_rgba"]), bytes(frames[4][-4:]))
        frames[4][1] ^= 1
        with self.assertRaises(ValueError):
            observation_buffer_records(frames)
        with self.assertRaises(ValueError):
            observation_buffer_records([frames[0]] * 15)


class OwnerReadContractTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self):
        self.cases = {case["case_id"]: case for case in load_cases()}
        self.events = []

    def owner(self, initial):
        return OwnerRepository(initial, lambda kind, payload: self.events.append((kind, payload)))

    async def read(self, owner, op, **selector):
        return await owner.read({"selector": {"op": op, **selector}, "limit": 10})

    async def test_question_identity_and_versioned_wire_aliases(self):
        owner = self.owner(self.cases["S6-04"]["initial"])
        question = owner.questions["Q기간"]
        self.assertEqual("Q기간", identity(question))
        self.assertEqual("Q기간", reference(question))
        self.assertEqual(question, owner.lookup({"question_id": "Q기간", "revision": 7}))
        self.assertIsNone(owner.lookup("Q기간@v6"))
        for alias in ("question_ref", "source_ref", "ref"):
            found = await self.read(owner, "CURRENT_QUESTION", **{alias: "Q기간@v7"})
            self.assertEqual(["Q기간"], [item["ref"] for item in found["items"]])
        denied_revision = await self.read(owner, "CURRENT_QUESTION", question_ref="Q기간@v6")
        self.assertEqual("NO_MATCH", denied_revision["status"])

    async def test_known_source_and_explicit_revision_aliases_preserve_old_version(self):
        case = self.cases["S2-03"]
        owner = self.owner(case["initial"])
        owner.apply_source(case["events"][0]["payload"]["source"])
        self.assertEqual(2, owner.lookup("D1")["revision"])
        for op, selector in (
            ("KNOWN_REF", {"source_ref": {"source_id": "D1", "revision": 1}}),
            ("KNOWN_REF", {"source_ref": "D1@v1"}),
            ("SOURCE_AT_REV", {"source_ref": "D1@v1"}),
            ("SOURCE_AT_REV", {"source_id": "D1", "revision": 1}),
        ):
            found = await self.read(owner, op, **selector)
            self.assertEqual("OK", found["status"])
            self.assertEqual(["D1@v1"], [item["ref"] for item in found["items"]])
            self.assertIn("100", found["items"][0]["content"])
        self.assertEqual("NO_MATCH", (await self.read(owner, "SOURCE_AT_REV", source_ref="D1@v3"))["status"])

    async def test_actual_current_task_result_dict_ref_and_unknown_ref(self):
        initial = copy.deepcopy(self.cases["S2-02"]["initial"])
        records = self.cases["S2-02"]["preparation"]["events"]
        initial["sources"] = [event["payload"]["source"] for event in records if "source" in event["payload"]]
        tasks = {event["payload"]["task"]["task_id"]: event["payload"]["task"]
                 for event in records if "task" in event["payload"]}
        initial["tasks"].extend(tasks.values())
        owner = self.owner(initial)
        for selector in ({"task_id": "T비교"}, {"source_ref": "T비교"}, {"source_ref": "D2@v2"},
                         {"source_ref": {"source_id": "D2", "revision": 2}}):
            found = await self.read(owner, "TASK_RESULT", **selector)
            self.assertEqual(["D2@v2"], [item["ref"] for item in found["items"]])
            self.assertEqual("A: 설치 2일, B: 설치 4일", found["items"][0]["content"])
        for ref in ("missing", "D2@v1", "T비교@v1"):
            self.assertEqual("NO_MATCH", (await self.read(owner, "TASK_RESULT", source_ref=ref))["status"])
        # An actual current Task artifact remains readable when no duplicate source record exists.
        old_owner = self.owner(self.cases["S3-02"]["initial"])
        old_result = await self.read(old_owner, "TASK_RESULT", task_id="T옛")
        self.assertEqual("K옛@v0", old_result["items"][0]["ref"])
        self.assertIn("10월 1일", old_result["items"][0]["content"])

    async def test_completed_task_input_is_not_fabricated_as_missing_result(self):
        initial = copy.deepcopy(self.cases["S1-01"]["initial"])
        initial["tasks"].append({"task_id": "Tmissing", "status": "COMPLETED",
                                 "source_refs": [{"source_id": "D1", "revision": 1}]})
        self.assertEqual("NO_MATCH", (await self.read(self.owner(initial), "TASK_RESULT", task_id="Tmissing"))["status"])

    async def test_known_artifact_cannot_bypass_task_permission_and_task_read_has_no_result_body(self):
        result = {"artifact_id": "private-result", "revision": 1, "text": "SECRET_RESULT_BODY"}
        task = {"task_id": "T", "status": "COMPLETED", "instruction": "actual prior instruction",
                "result": result, "permission": {"read": True, "model": False, "agent": True}}
        owner = self.owner({"tasks": [task]})
        denied = await self.read(owner, "KNOWN_REF", source_ref="private-result@v1")
        self.assertEqual("DENIED", denied["status"])
        self.assertNotIn("SECRET_RESULT_BODY", json.dumps(denied))
        owner.tasks["T"]["permission"]["model"] = True
        task_read = await self.read(owner, "KNOWN_REF", source_ref="T")
        self.assertEqual("actual prior instruction", task_read["items"][0]["content"])
        self.assertNotIn("SECRET_RESULT_BODY", json.dumps(task_read))
        result_read = await self.read(owner, "TASK_RESULT", source_ref="T")
        self.assertEqual("SECRET_RESULT_BODY", result_read["items"][0]["content"])

    async def test_past_delivery_reference_filters_and_retains_actual_audible_text(self):
        owner = self.owner(self.cases["S2-03"]["initial"])
        for ref in ("P1", "P1@v1", {"source_id": "D1", "revision": 1}, "D1@v1"):
            result = await self.read(owner, "PAST_DELIVERED", source_ref=ref)
            self.assertEqual(["P1@v1"], [item["ref"] for item in result["items"]])
            self.assertTrue(result["items"][0]["content"]["delivery_receipt"])
        self.assertEqual("NO_MATCH", (await self.read(owner, "PAST_DELIVERED", source_ref="unrelated"))["status"])

    async def test_metadata_is_bounded_and_does_not_supply_history_or_result_bodies(self):
        initial = build_activity_baseline()
        initial["sources"][0]["text"] = "SECRET_SOURCE_BODY"
        initial["tasks"][0]["result"]["text"] = "SECRET_RESULT_BODY"
        initial["interactions"][-1]["text"] = "SECRET_TEXT_BODY"
        initial["interactions"][-1]["delivered_voice_text"] = "SECRET_AUDIBLE_BODY"
        owner = self.owner(initial)
        metadata = owner.metadata()
        encoded = json.dumps(metadata)
        self.assertNotIn("SECRET_", encoded)
        self.assertEqual(10, len(metadata["interactions"]))
        self.assertEqual(10, len(metadata["tasks"]))
        self.assertEqual(2000, metadata["metadata_scope"]["interactions_total"])
        self.assertEqual(200, metadata["metadata_scope"]["tasks_total"])
        self.assertEqual(2000, len(owner.interactions))
        self.assertIn("B01@v1", owner.provided)
        self.assertNotIn("B01@v1", owner.body_read_refs)
        result = await owner.read({"selector": {"op": "RECENT_INTERACTION"}, "limit": 2})
        self.assertEqual(2, len(result["items"]))
        self.assertEqual(2000, result["candidate_count"])

    async def test_model_and_agent_permission_are_distinct_and_wire_cannot_override_role(self):
        source = {"source_id": "S", "revision": 1, "text": "PRIVATE_AGENT_BODY",
                  "permission": {"read": True, "model": False, "agent": True},
                  "metadata": {"email": "PRIVATE_METADATA_EMAIL"}}
        owner = self.owner({"sources": [source]})
        self.assertNotIn("PRIVATE_METADATA_EMAIL", json.dumps(owner.metadata()))
        spec = {"selector": {"op": "KNOWN_REF", "source_ref": "S"}, "consumer_role": "agent"}
        model_read = await owner.read(spec)
        self.assertEqual("DENIED", model_read["status"])
        self.assertNotIn("PRIVATE_AGENT_BODY", json.dumps(model_read))
        agent_read = await owner.read(spec, consumer_role="agent")
        self.assertEqual("PRIVATE_AGENT_BODY", agent_read["items"][0]["content"])
        self.assertEqual("agent", agent_read["consumer_role"])
        owner.sources[0]["permission"] = {"read": True, "model": True, "agent": False}
        self.assertEqual("OK", (await owner.read(spec))["status"])
        self.assertEqual("DENIED", (await owner.read(spec, consumer_role="agent"))["status"])

    async def test_selection_read_supplies_all_timed_original_observations_without_binding(self):
        case = self.cases["S2-01"]
        initial = copy.deepcopy(case["initial"])
        initial["observations"] = [{"at_seconds": event["at_seconds"], **event["payload"]}
                                   for event in case["events"]]
        owner = self.owner(initial)
        result = await self.read(owner, "INPUT_SELECTION")
        self.assertEqual(["A@v1", "B@v1", "C@v1"], [item["ref"] for item in result["items"]])
        self.assertEqual([0, .9, 1.7], [item["metadata"]["metadata"]["observation"]["at_seconds"]
                                       for item in result["items"]])
        self.assertNotIn("bound_span", json.dumps(result))
        self.assertIn("900", result["items"][2]["content"])

    async def test_revocation_invalidates_cached_model_and_agent_reads(self):
        owner = self.owner(self.cases["S1-01"]["initial"])
        first = await self.read(owner, "KNOWN_REF", source_ref="D1")
        second = await self.read(owner, "KNOWN_REF", source_ref="D1")
        self.assertFalse(first["items"][0]["cache_hit"])
        self.assertTrue(second["items"][0]["cache_hit"])
        owner.revoke("D1")
        for role in ("model", "agent"):
            result = await owner.read({"selector": {"op": "KNOWN_REF", "source_ref": "D1"}}, consumer_role=role)
            self.assertEqual("DENIED", result["status"])
            self.assertEqual([], result["items"])
        self.assertEqual(2, owner.policy_epoch)

    async def test_consent_materialization_replaces_actual_placeholder_without_broadening_roles(self):
        case = self.cases["S6-02"]
        owner = self.owner(case["initial"])
        self.assertEqual("DENIED", (await self.read(owner, "KNOWN_REF", source_ref="민수"))["status"])
        actual = copy.deepcopy(case["consent_source_release"]["source"])
        actual["permission"]["agent"] = "DENIED"
        result = owner.materialize_consent_source(actual)
        self.assertEqual("minsu@example.invalid", owner.lookup("민수@v1")["text"])
        self.assertEqual(1, sum(identity(s) == "민수" for s in owner.sources))
        self.assertEqual(2, owner.policy_epoch)
        self.assertNotIn("민수@v1", owner.body_read_refs)
        self.assertEqual("DENIED", result["permission"]["agent"])
        source_read = await self.read(owner, "KNOWN_REF", source_ref="민수")
        self.assertEqual("minsu@example.invalid", source_read["items"][0]["content"])
        denied_agent = await owner.read({"selector": {"op": "KNOWN_REF", "source_ref": "민수"}}, consumer_role="agent")
        self.assertEqual("DENIED", denied_agent["status"])
        materialization = next(payload for kind, payload in self.events if kind == "CONSENT_SOURCE_MATERIALIZED")
        self.assertFalse(materialization["body_read_evidence"])
        self.assertTrue(materialization["previous_record_sha256"])
        owner.materialize_consent_source(actual)
        self.assertEqual(2, owner.policy_epoch)
        self.assertEqual("CONSENT_SOURCE_MATERIALIZATION_REUSED", self.events[-1][0])

    async def test_consent_materialization_clears_empty_cache_and_rejects_new_source_or_same_revision_rewrite(self):
        owner = self.owner({"sources": [{"source_id": "S", "revision": 1, "text": "",
                                       "permission": {"read": True, "model": True, "agent": False}}]})
        await self.read(owner, "KNOWN_REF", source_ref="S")
        self.assertTrue(owner.cache)
        actual = {"source_id": "S", "revision": 1, "text": "actual received source content",
                  "permission": {"read": True, "model": True, "agent": False}}
        owner.materialize_consent_source(actual)
        self.assertEqual({}, owner.cache)
        self.assertFalse((await self.read(owner, "KNOWN_REF", source_ref="S"))["items"][0]["cache_hit"])
        with self.assertRaises(ValueError):
            owner.materialize_consent_source({**actual, "source_id": "unrelated"})
        with self.assertRaises(ValueError):
            owner.materialize_consent_source({**actual, "text": "replacement answer at unchanged revision"})

    async def test_new_sources_consume_slots_and_revisions_preserve_history_without_more_slots(self):
        owner = self.owner(build_activity_baseline())
        source = copy.deepcopy(self.cases["S1-01"]["initial"]["sources"][0])
        owner.apply_source(source)
        self.assertEqual("C01", self.events[-1][1]["consumed_slot_id"])
        self.assertEqual(30, len({identity(s) for s in owner.sources}))
        self.assertIsNone(owner.lookup("C01"))
        self.assertNotIn("C01@v1", owner.provided)
        self.assertEqual("D1", owner.source_slot_assignments["C01"])
        owner.apply_source({**source, "revision": 2, "text": "actual revised source"})
        self.assertEqual(30, len({identity(s) for s in owner.sources}))
        self.assertEqual(31, len(owner.sources))
        self.assertEqual(source["text"], owner.lookup("D1@v1")["text"])
        self.assertEqual("actual revised source", owner.lookup("D1")["text"])
        self.assertIsNone(self.events[-1][1]["consumed_slot_id"])
        self.assertEqual(9, owner.metadata()["metadata_scope"]["unpopulated_source_slots"])

    async def test_exhausted_slot_capacity_rejects_new_id_atomically_but_allows_same_id_revision(self):
        owner = self.owner(build_activity_baseline())
        source = copy.deepcopy(self.cases["S1-01"]["initial"]["sources"][0])
        for index in range(10):
            owner.apply_source({**source, "source_id": f"actual-{index}"})
        before = copy.deepcopy(owner.sources)
        with self.assertRaises(ValueError):
            owner.apply_source({**source, "source_id": "eleventh"})
        self.assertEqual(before, owner.sources)
        self.assertEqual("SOURCE_CAPACITY_REJECTED", self.events[-1][0])
        self.assertIsNone(owner.lookup("eleventh"))
        owner.apply_source({**source, "source_id": "actual-0", "revision": 2})
        self.assertEqual(30, len({identity(s) for s in owner.sources}))

    async def test_small_fixture_without_slots_accepts_new_source_within_default_capacity(self):
        source = {"source_id": "existing", "revision": 1, "text": "actual original"}
        owner = self.owner({"sources": [source]})
        owner.apply_source({**source, "source_id": "published", "text": "actual publication"})
        self.assertEqual("actual publication", owner.lookup("published")["text"])
        self.assertEqual(2, len({identity(s) for s in owner.sources}))
        self.assertEqual(30, owner.metadata()["metadata_scope"]["source_capacity"])
        self.assertIsNone(self.events[-1][1]["consumed_slot_id"])
        owner.apply_source({**source, "revision": 2, "text": "actual revision"})
        self.assertEqual(2, len({identity(s) for s in owner.sources}))
        self.assertEqual("actual original", owner.lookup("existing@v1")["text"])

    async def test_non_slot_capacity_limit_rejects_new_logical_id_but_not_revision(self):
        source = {"source_id": "only", "revision": 1, "text": "original"}
        owner = self.owner({"sources": [source], "max_sources": 1})
        with self.assertRaises(ValueError):
            owner.apply_source({**source, "source_id": "extra"})
        self.assertEqual([source], owner.sources)
        self.assertEqual("LOGICAL_SOURCE_CAPACITY_EXHAUSTED", self.events[-1][1]["reason"])
        owner.apply_source({**source, "revision": 2, "text": "revised"})
        self.assertEqual("revised", owner.lookup("only")["text"])
        self.assertEqual(1, len({identity(s) for s in owner.sources}))

    async def test_consent_replaces_same_logical_source_without_consuming_another_slot(self):
        owner = self.owner(build_activity_baseline())
        case = self.cases["S6-02"]
        owner.apply_source(case["initial"]["sources"][0])
        slots = owner.metadata()["metadata_scope"]["unpopulated_source_slots"]
        owner.materialize_consent_source(case["consent_source_release"]["source"])
        self.assertEqual(slots, owner.metadata()["metadata_scope"]["unpopulated_source_slots"])
        self.assertEqual(30, len({identity(s) for s in owner.sources}))
        self.assertEqual("minsu@example.invalid", owner.lookup("민수")["text"])


if __name__ == "__main__":
    unittest.main()
