"""Technical controls for deterministic QA04 features, never provider execution.

Host timestamps use the same actual monotonic clock as the common harness.
This feature protocol is not a substitute for strict common9 regression.
"""
import copy
import threading
import time


CONTROL_TYPES = frozenset(('clock_probe', 'observation_deadline',
                           'cancel_work', 'publication_receipt', 'end_input'))


def is_control(value):
    return isinstance(value, dict) and value.get('type') in CONTROL_TYPES


class Controls:
    def __init__(self, send, now=time.monotonic_ns):
        self.send = send
        self.now = now
        self.lock = threading.RLock()
        self.clock_ready = threading.Event()
        self.deadline_ready = threading.Event()
        self.cancel_ready = threading.Event()
        self.clock_send = None
        self.clock_proof = None
        self.updates = []
        self.receipts = []
        self.publications = set()
        self.sends = []
        self.failures = []
        self.deadline_ns = None
        self.cancelled_ns = None
        self.effective_stop_ns = None
        self.settlement_deadline_ns = None

    def start(self):
        # Hold the lock through send-bracket assignment so the stdout reader
        # cannot process an ACK before the actual send record exists.
        with self.lock:
            self.clock_send = self.send({'type': 'clock_probe', 'host_probe_id': 'qa04-feature-clock'})

    def _fail(self, reason):
        self.failures.append({'reason': reason, 'at_ns': self.now()})
        return False

    def observe(self, message):
        with self.lock:
            kind = message.get('kind')
            data = message.get('data', {})
            at = message['observed_ns']
            if kind == 'HOST_CLOCK_PROBE_ACK':
                rust = data.get('host_monotonic_ns')
                event = message.get('host_monotonic_ns')
                valid = (self.clock_send is not None and data.get('probe_id') == 'qa04-feature-clock'
                         and type(rust) is int and type(event) is int
                         and self.clock_send['send_begin_ns'] <= rust <= event <= at)
                self.clock_proof = {'status': 'VERIFIED' if valid else 'FAILED',
                    'probe_send': copy.deepcopy(self.clock_send), 'rust_probe_ns': rust,
                    'rust_event_ns': event, 'host_observed_ns': at, 'event_id': message.get('event_id')}
                if valid:
                    self.clock_ready.set()
                else:
                    self._fail('HOST_CLOCK_PROOF_INVALID')
            elif kind == 'OBSERVATION_DEADLINE_ACK':
                control = data.get('control', {})
                row = next((u for u in self.updates if u['update_id'] == control.get('update_id')), None)
                admitted = data.get('update_admitted_ns')
                valid = (row is not None and row['ack_status'] == 'PENDING' and self.cancelled_ns is None and data.get('accepted') is True
                         and type(admitted) is int and row['control_send']['send_begin_ns'] <= admitted <= at
                         and all(control.get(k) == row[k] for k in ('deadline_ns', 'last_actual_source_ns', 'previous_deadline_ns'))
                         and at < row['deadline_ns']
                         and (row['previous_deadline_ns'] is None or at < row['previous_deadline_ns']))
                if row is not None:
                    row['ack'] = copy.deepcopy(message)
                    row['ack_status'] = 'ACCEPTED' if valid else 'REJECTED_OR_LATE_OR_MISMATCHED'
                if valid:
                    self.deadline_ready.set()
                else:
                    self._fail('DEADLINE_ACK_REJECTED_OR_LATE_OR_MISMATCHED')
            elif kind == 'WORK_CANCELLATION_ACK':
                sent = next((r for r in self.sends if r['type'] == 'cancel_work'), None)
                if (self.cancelled_ns is not None and sent is not None
                    and all(data.get('control', {}).get(k) == sent[k] for k in ('observation_deadline_ns', 'settlement_deadline_ns', 'reason'))):
                    self.cancel_ready.set()
                else:
                    self._fail('UNREQUESTED_CANCELLATION_ACK')
            elif kind in ('DISPLAY_PROXY', 'QA04_SECTION_DISPLAY_PROXY'):
                publication = data.get('publication_id')
                if not publication or not message.get('event_id'):
                    self._fail('DISPLAY_WITHOUT_TECHNICAL_PUBLICATION_ID')
                    return
                if publication in self.publications:
                    self._fail('DUPLICATE_TECHNICAL_PUBLICATION_ID')
                    return
                self.publications.add(publication)
                accepted = (self.clock_ready.is_set() and self.deadline_ready.is_set()
                            and self.now() < self.deadline_ns and at <= self.deadline_ns
                            and self.cancelled_ns is None)
                receipt = {'type': 'publication_receipt', 'publication_id': publication,
                    'display_event_id': message['event_id'], 'observed_ns': at,
                    'observation_deadline_ns': self.deadline_ns, 'accepted': accepted}
                self.receipts.append({**receipt, **self.send(receipt)})

    def arm(self, last_actual_source_ns, seconds=60):
        with self.lock:
            self.check(require_deadline=False)
            previous = self.deadline_ns
            deadline = last_actual_source_ns + int(seconds * 1e9)
            if self.now() >= deadline or previous is not None and self.now() >= previous:
                raise AssertionError('Cannot revive an expired feature window')
            self.deadline_ready.clear()
            row = {'update_id': len(self.updates) + 1, 'at_ns': self.now(),
                'last_actual_source_ns': last_actual_source_ns, 'deadline_ns': deadline,
                'previous_deadline_ns': previous, 'reason': 'QA04_DETERMINISTIC_FEATURE_WINDOW', 'ack_status': 'PENDING'}
            self.deadline_ns = deadline  # Conservative proposed bound applies while awaiting ACK.
            self.updates.append(row)
            row['control_send'] = self.send({'type': 'observation_deadline', **copy.deepcopy(row)})

    def check(self, require_deadline=True):
        with self.lock:
            if self.failures or not self.clock_ready.is_set():
                raise AssertionError('QA04 host clock or technical protocol not verified')
            if self.cancelled_ns is not None or self.deadline_ns is not None and self.now() >= self.deadline_ns:
                raise AssertionError('QA04 observation admission closed')
            if require_deadline and not self.deadline_ready.is_set():
                raise AssertionError('QA04 observation deadline ACK not verified')
            return self.now()

    def check_locked(self):
        # Same small gate interface used by the unchanged bridge bookkeeping.
        return self.check()

    def cancel(self, reason):
        with self.lock:
            if self.cancelled_ns is not None:
                return
            self.cancelled_ns = self.now()
            bounds = [self.cancelled_ns, self.deadline_ns]
            bounds.extend(u['previous_deadline_ns'] for u in self.updates if u['ack_status'] != 'ACCEPTED')
            self.effective_stop_ns = min(b for b in bounds if b is not None)
            self.settlement_deadline_ns = self.effective_stop_ns + int(120e9)
            control = {'type': 'cancel_work', 'reason': reason,
                'observation_deadline_ns': self.effective_stop_ns,
                'settlement_deadline_ns': self.settlement_deadline_ns}
            self.sends.append({**control, **self.send(control)})

    def snapshot(self):
        with self.lock:
            return copy.deepcopy({'clock_probe_send': self.clock_send, 'host_clock_proof': self.clock_proof,
                'host_display_receipts': self.receipts, 'observation_deadline_updates': self.updates,
                'contract_observation_deadline_ns': self.deadline_ns,
                'proposed_observation_deadline_ns': self.deadline_ns,
                'last_accepted_observation_deadline_ns': next((u['deadline_ns'] for u in reversed(self.updates) if u['ack_status'] == 'ACCEPTED'), None),
                'cancellation_requested_ns': self.cancelled_ns, 'effective_admission_stop_ns': self.effective_stop_ns,
                'settlement_deadline_ns': self.settlement_deadline_ns,
                'technical_failures': self.failures, 'control_sends': self.sends})
