"""Small API-free preparation checks. Does not import or run a candidate."""
import copy
import unittest
from patches import migrate_c03
from protocol import Controls, CONTROL_TYPES
from module_inventory import operational_tokens


class ProtocolChecks(unittest.TestCase):
    def setup_control(self, armed=True):
        self.now = 100
        self.sent = []
        def send(value):
            self.sent.append(copy.deepcopy(value))
            self.now += 1
            return {'send_begin_ns': self.now, 'send_end_ns': self.now, 'provided_at_ns': self.now}
        c = Controls(send, now=lambda: self.now)
        c.start()
        c.observe({'kind': 'HOST_CLOCK_PROBE_ACK', 'event_id': 'clock', 'observed_ns': 104,
                   'host_monotonic_ns': 103, 'data': {'probe_id': 'qa04-feature-clock', 'host_monotonic_ns': 102}})
        self.now = 105
        if armed:
            c.arm(100, seconds=.000001)
            self.ack(c)
        return c

    def ack(self, c, **overrides):
        row = c.updates[-1]
        message = {'kind': 'OBSERVATION_DEADLINE_ACK', 'observed_ns': self.now + 2,
            'data': {'accepted': True, 'update_admitted_ns': self.now + 1,
                     'control': {k: row[k] for k in ('update_id', 'deadline_ns', 'last_actual_source_ns', 'previous_deadline_ns')}}}
        message.update(overrides)
        c.observe(message)

    def display(self, c, publication='pub', observed_ns=None):
        c.observe({'kind': 'DISPLAY_PROXY', 'event_id': 'display-'+publication,
                   'observed_ns': observed_ns if observed_ns is not None else self.now,
                   'data': {'publication_id': publication}})

    def test_clock_and_deadline_are_required(self):
        c = self.setup_control(False)
        with self.assertRaises(AssertionError): c.check()
        c.arm(100, seconds=.000001)
        with self.assertRaises(AssertionError): c.check()
        self.ack(c)
        c.check()

    def test_missing_clock_proof_disallows_dispatch(self):
        c = Controls(lambda value: {}, now=lambda: 100)
        with self.assertRaises(AssertionError): c.check(require_deadline=False)

    def test_receipt_uses_actual_observation_and_send_brackets(self):
        c = self.setup_control()
        self.display(c, observed_ns=107)
        r = c.receipts[0]
        self.assertEqual(r['observed_ns'], 107)
        self.assertEqual(r['publication_id'], 'pub')
        self.assertEqual(r['display_event_id'], 'display-pub')
        self.assertTrue(r['accepted'])
        self.assertEqual(r['send_begin_ns'], r['provided_at_ns'])

    def test_late_receipt_remains_raw_and_rejected(self):
        c = self.setup_control()
        self.now = c.deadline_ns + 1
        self.display(c)
        self.assertFalse(c.receipts[0]['accepted'])
        with self.assertRaises(AssertionError): c.check()

    def test_duplicate_publication_is_not_receipted_twice(self):
        c = self.setup_control()
        self.display(c)
        self.display(c)
        self.assertEqual(len(c.receipts), 1)
        self.assertEqual(c.failures[-1]['reason'], 'DUPLICATE_TECHNICAL_PUBLICATION_ID')

    def test_cancel_blocks_receipt_admission_and_does_not_erase_contract(self):
        c = self.setup_control()
        original = c.deadline_ns
        c.cancel('TEST_ABORT')
        self.display(c)
        self.assertFalse(c.receipts[0]['accepted'])
        self.assertEqual(c.snapshot()['contract_observation_deadline_ns'], original)
        self.assertLess(c.effective_stop_ns, original)
        self.assertEqual(c.settlement_deadline_ns, c.effective_stop_ns + int(120e9))
        with self.assertRaises(AssertionError): c.check()

    def test_late_deadline_ack_cannot_extend_old_cutoff(self):
        c = self.setup_control()
        old = c.deadline_ns
        c.arm(200, seconds=.000001)
        self.ack(c, observed_ns=old + 1)
        self.assertEqual(c.updates[-1]['ack_status'], 'REJECTED_OR_LATE_OR_MISMATCHED')
        self.assertFalse(c.deadline_ready.is_set())

    def test_cancelled_deadline_cannot_be_revived_by_ack(self):
        c = self.setup_control(False)
        c.arm(100, seconds=.000001)
        c.cancel('TEST_ABORT')
        self.ack(c)
        self.assertFalse(c.deadline_ready.is_set())
        with self.assertRaises(AssertionError): c.check()

    def test_mismatched_deadline_ack_is_rejected(self):
        c = self.setup_control(False)
        c.arm(100, seconds=.000001)
        self.ack(c, data={'accepted': True, 'update_admitted_ns': 108,
            'control': {'update_id': 1, 'deadline_ns': 9999, 'last_actual_source_ns': 100, 'previous_deadline_ns': None}})
        self.assertFalse(c.deadline_ready.is_set())
        self.assertEqual(c.failures[-1]['reason'], 'DEADLINE_ACK_REJECTED_OR_LATE_OR_MISMATCHED')

    def test_technical_controls_are_unchanged_by_c03(self):
        for kind in CONTROL_TYPES:
            value = {'type': kind, 'deadline_ns': 123456789, 'observed_ns': 123450000,
                'control': {'source_id': 'DOC', 'revision': 2, 'start_line': 1},
                'publication_id': 'pub', 'accepted': True}
            self.assertEqual(migrate_c03(value), value)
            self.assertIsNot(migrate_c03(value), value)

    def test_c03_changes_source_fields_but_keeps_request_revision(self):
        value = {'type': 'input_event', 'event': {'kind': 'user', 'revision': 3,
            'frames': [{'frame_id': 'F', 'source_id': 'DOC', 'source_revision': 2,
                'width': 2880, 'height': 1800, 'observed_at_ms': 123}],
            'references': [{'source_id': 'DOC', 'revision': 2, 'start_line': 1, 'end_line': 2}]}}
        result = migrate_c03(value)
        self.assertEqual(result['event']['revision'], 3)
        self.assertEqual(result['event']['frames'][0]['version'], {'number': 2})
        self.assertEqual(result['event']['frames'][0]['captured_at']['value'], 123000)
        self.assertEqual(result['event']['references'][0]['version'], {'number': 2})
        self.assertIn('source_revision', value['event']['frames'][0])

    def test_audited_test_only_gate_helper_does_not_count(self):
        before = 'impl Context { pub fn operational() { actual(); } }'
        after = 'impl Context { #[cfg(test)] pub fn test_deadline(&self, at:u64) { helper(at); } pub fn operational() { actual(); } }'
        self.assertEqual(operational_tokens(before), operational_tokens(after))


if __name__ == '__main__':
    unittest.main(verbosity=2)
