import asyncio
import copy
import json
import time
import unittest
from benchmark.architecture.dp41_45_poc.worker import AnswerSource
from benchmark.architecture.dp41_45_poc.fixtures import load_cases, public_inputs
from prototypes.candidates.dp41_45_poc.shared.runtime import LocalVIA, Ports
from prototypes.candidates.dp41_45_poc.dp41.contract import FRAME_VERSION, MEANING_VERSION


class AnswerSourceTests(unittest.TestCase):
    def test_related_question_uses_actual_scope_and_allows_paraphrase(self):
        source=AnswerSource(next(c for c in load_cases() if c['case_id']=='S6-01'))
        self.assertTrue(source.report_scope_question('3분기와 4분기 중 어느 업무를 어떤 형식으로 수정할까요?',None))
        self.assertTrue(source.report_scope_question('어떤 형식으로 바꿀까요?',{'target':'T3분기'}))
        self.assertFalse(source.report_scope_question('메일을 보내도 될까요?',{'target':'T메일'}))
        self.assertFalse(source.report_scope_question('연락처 주소를 알려주세요.',{'target':'민수'}))

    def test_information_consent_uses_actual_question_subject_and_both_uses(self):
        scope = {'source_id': '민수', 'task_id': 'T메일', 'uses': ['READ_CONTACT', 'PROVIDE_TO_TASK']}
        for text in ('민수의 연락처를 조회해 메일 업무에 제공할까요?',
                     '민수님의 이메일 주소를 확인하고 이 메일 업무에 전달해도 될까요?',
                     '민수 연락처 조회 후 메일 업무에 공유해도 괜찮을까요? 발송은 별도 승인입니다.',
                     '민수 연락처 조회와 이 메일 업무에 주소 제공에 동의하나요? 발송 승인은 별개예요.',
                     '민수 연락처 조회와 이 메일 업무에 주소 제공에 동의하나요? 발송 승인과는 별개예요.',
                     '민수 연락처 조회와 이 메일 업무에 주소 제공에 동의하나요? 발송의 승인은 별도예요.',
                     '민수 연락처 조회와 이 메일 업무에 주소 제공에 동의하나요? 발 송 승 인 은 별 개 예요.',
                     '민수 연락처를 조회해 메일 업무에 제공할까요? 발송은 하지 않겠습니다.'):
            self.assertTrue(AnswerSource.information_scope_question(text, scope), text)
        for text in ('민수 연락처를 조회할까요?', '민수 메일을 발송할까요?',
                     '영수 연락처를 조회해 메일 업무에 제공할까요?',
                     '민수 연락처를 조회해 메일 업무에 제공하고 발송할까요?',
                     '민수 연락처를 조회해 메일 업무에 제공하고 바로 발송해도 될까요?',
                     '민수 연락처를 조회해 메일 업무에 제공할까요? 발송 승인은 별개예요. 지금 보내도 될까요?',
                     '민수 연락처를 조회해 메일 업무에 제공했습니다.'):
            self.assertFalse(AnswerSource.information_scope_question(text, scope), text)


class ReplayEvidence:
    def __init__(self):
        self.origin, self.rows = time.monotonic(), []

    def seconds(self):
        return time.monotonic() - self.origin

    def write(self, table, row):
        self.rows.append((table, copy.deepcopy(row)))
        return row


class ScriptedProvider:
    """Specified outputs test wiring only, not actual model accuracy."""
    def __init__(self, variant, *, answer=True, question_text=None, main_presentation=False):
        self.variant, self.answer, self.question_text = variant, answer, question_text
        self.main_presentation = main_presentation
        self.calls = []

    async def call(self, messages, role, **kwargs):
        self.calls.append((copy.deepcopy(messages), role, copy.deepcopy({k: v for k, v in kwargs.items()
                                                                       if not k.startswith('causal_')})))
        if role == 'CRESPONSE':
            facts = json.loads(messages[-1]['content'])
            return {'text': self.question_text or facts.get('question', {}).get('text', '실제 접수 사실')}
        proposal = {'version': MEANING_VERSION, 'goals': [], 'relations': [], 'constraints': [],
                    'preservations': [], 'response': '답변을 확인했습니다.', 'evidence_refs': [], 'unresolved': []}
        if self.main_presentation and kwargs.get('request_id') == 'root-request':
            proposal['response'] = None
            proposal['goals'] = [QuestionFollowupRuntimeTests.present_goal()]
            if self.variant == 'A':
                return {'action': 'PROPOSE', 'meaning': proposal}
            proposal['version'] = FRAME_VERSION
            proposal['references'] = [{'id': 'pending-question', 'selector': {'op': 'KNOWN_REF', 'source_ref': 'Q1'}}]
            proposal['goals'][0]['arguments']['question_ref'] = 'pending-question'
            return proposal
        if self.answer:
            proposal['goals'] = [{'id': 'answer', 'kind': 'ANSWER_QUESTION', 'operation': 'ANSWER_QUESTION',
                'target': 'T메일', 'arguments': {'question_ref': 'Q1', 'question_revision': 2,
                    'answer': '동의', 'properties': {}, 'consent': {'read': True, 'share': True, 'send': False}},
                'instruction': '실제 사용자 답을 현재 질문에 전달'}]
        if self.variant == 'A':
            return {'action': 'PROPOSE', 'meaning': proposal}
        proposal['version'], proposal['references'] = FRAME_VERSION, []
        if self.answer:
            proposal['references'] = [
                {'id': 'mail-task', 'selector': {'op': 'KNOWN_REF', 'source_ref': 'T메일'}},
                {'id': 'consent-question', 'selector': {'op': 'KNOWN_REF', 'source_ref': 'Q1'}}]
            proposal['goals'][0]['target'] = 'mail-task'
            proposal['goals'][0]['arguments']['question_ref'] = 'consent-question'
        return proposal


class QuestionFollowupRuntimeTests(unittest.IsolatedAsyncioTestCase):
    def build(self, variant='A', *, case_id='S6-02', answer=True, question_text=None, voice_on=True, wait=.002,
              main_presentation=False):
        case = copy.deepcopy(next(c for c in load_cases() if c['case_id'] == case_id))
        for plan in case['followups']:
            plan['wait_seconds'], plan['speech_seconds'] = wait, .002
        case['unexpected_question_reply']['wait_seconds'] = .002
        case['unexpected_question_reply']['speech_seconds'] = .002
        initial = public_inputs(case)['initial']
        initial['voice']['on'] = voice_on
        evidence, model = ReplayEvidence(), ScriptedProvider(variant, answer=answer, question_text=question_text,
                                                           main_presentation=main_presentation)
        runtime = LocalVIA(initial, model, evidence, variant, answer_source=AnswerSource(case),
            voice_profile={'transcript_delay_seconds': 0., 'output_onset_seconds': .001,
                           'characters_per_second': 10000., 'stop_seconds': .001},
            stub_profile={'acceptance_receipt_seconds': .001, 'bounded_result_seconds': .001,
                          'summary_result_seconds': .001, 'final_result_seconds': None})
        self.runtimes = [*getattr(self, 'runtimes', []), runtime]
        return case, runtime, model, evidence

    async def asyncTearDown(self):
        for runtime in getattr(self, 'runtimes', []):
            pending = list(runtime.jobs)
            for job in pending:
                job.cancel()
            await asyncio.gather(*pending, return_exceptions=True)

    @staticmethod
    async def register_replacement(case, runtime):
        for event in case['events']:
            await runtime.external_event(event)

    @staticmethod
    def present_goal():
        return {'id': 'present', 'kind': 'DIRECT_RESPONSE', 'operation': 'QUESTION_PRESENT', 'target': None,
                'arguments': {'question_ref': 'Q1'}, 'instruction': '현재 owner 질문을 제시'}

    @staticmethod
    def followups(evidence):
        return [row for table, row in evidence.rows if table == 'inputs-and-source' and row['kind'] == 'FOLLOWUP_INPUT']

    async def test_actual_question_present_new_request_reaches_both_candidate_binders_and_consent_owner(self):
        for variant in ('A', 'B'):
            with self.subTest(variant=variant):
                case, runtime, model, evidence = self.build(variant, main_presentation=True,
                    question_text='민수 연락처 조회와 이 메일 업무에 주소 제공에 동의하나요? 발송 승인은 별개예요.' if variant == 'B' else None)
                await self.register_replacement(case, runtime)
                result = await runtime.resolve('응.', request_id='root-request', input_id='root-input')
                await asyncio.wait_for(runtime.drain(), .7)
                self.assertEqual('ADOPTED', result['status'])
                delivery = next(d for d in runtime.deliveries if d['kind'] == 'QUESTION')
                followups = self.followups(evidence)
                self.assertTrue(delivery['delivered_receipt'])
                self.assertEqual(1, len(followups))
                self.assertNotEqual('root-request', followups[0]['request_id'])
                self.assertEqual(1, runtime.current_input_generation)
                self.assertEqual(['ANSWER_QUESTION'], [e['operation'] for e in runtime.effects])
                self.assertEqual('Q1', runtime.effects[0]['payload']['question']['question_id'])
                self.assertEqual(2, runtime.effects[0]['payload']['question']['revision'])
                self.assertEqual('ANSWERED', runtime.owners.questions['Q1']['status'])
                self.assertEqual('minsu@example.invalid', runtime.owners.lookup('민수')['text'])
                self.assertFalse(runtime.policy['send_allowed'])
                contexts = [json.loads(messages[1]['content']) for messages, role, kwargs in model.calls if role == 'CINTERPRET']
                self.assertEqual(2, len(contexts))
                self.assertEqual(followups[0]['transcript'], contexts[1]['transcript'])
                focus = contexts[1]['metadata']['question_focus']
                self.assertEqual(('Q1', 2, 'T메일', 'X메일'),
                    (focus['question_id'], focus['question_revision'], focus['task_id'], focus['execution_id']))
                self.assertNotIn('eligible_planned_reply', json.dumps(contexts[1]))
                self.assertNotIn('minsu@example.invalid', json.dumps(contexts[1]))

    async def test_external_question_delivery_receipt_starts_only_one_new_request(self):
        case, runtime, model, evidence = self.build(answer=False, voice_on=False, wait=.01)
        await self.register_replacement(case, runtime)
        event = copy.deepcopy(case['events'][1])
        event['event_id'], event['type'] = 'actual-question-presentation', 'QUESTION'
        event['payload']['presentation_requested'] = True
        await runtime.external_event(event)
        duplicate = copy.deepcopy(event)
        duplicate['event_id'] = 'same-question-new-publication'
        await runtime.external_event(duplicate)
        await asyncio.wait_for(runtime.drain(), .7)
        self.assertEqual(1, len(self.followups(evidence)))
        self.assertEqual(1, len([1 for _, role, _ in model.calls if role == 'CINTERPRET']))
        self.assertTrue(any(row['kind'] == 'QUESTION_FOLLOWUP_SUPPRESSED' for table, row in evidence.rows if table == 'events'))

    async def test_ineligible_full_question_releases_reservation_for_same_version_eligible_presentation(self):
        case, runtime, model, evidence = self.build(question_text='이 메일을 발송할까요?', voice_on=False)
        await self.register_replacement(case, runtime)
        goal = self.present_goal()
        first = await runtime.perform_goal(goal, {'goals': [], 'constraints': []},
            Ports(runtime, 'first-display', 'input-one', '질문'))
        await runtime.drain()
        self.assertTrue(first['delivered_receipt'])
        self.assertEqual([], self.followups(evidence))
        self.assertNotIn(('Q1', 2), runtime.question_followup_keys)
        model.question_text = '민수 연락처 조회와 이 메일 업무에 주소 제공에 동의하나요? 발송 승인은 별개예요.'
        second = await runtime.perform_goal(goal, {'goals': [], 'constraints': []},
            Ports(runtime, 'second-display', 'input-two', '다시 질문'))
        await asyncio.wait_for(runtime.drain(), .5)
        self.assertTrue(second['delivered_receipt'])
        self.assertEqual(1, len(self.followups(evidence)))
        self.assertEqual([True], [p.get('used') for p in runtime.answer_source.plans])
        self.assertIn(('Q1', 2), runtime.question_followup_keys)
        self.assertEqual(['ANSWER_QUESTION'], [e['operation'] for e in runtime.effects])
        self.assertFalse(runtime.policy['send_allowed'])

    async def test_concurrent_question_presentations_reserve_one_callback_until_actual_answer(self):
        case, runtime, model, evidence = self.build(voice_on=False)
        await self.register_replacement(case, runtime)
        original, started, release = runtime.answer_source, asyncio.Event(), asyncio.Event()
        class HeldAnswerSource:
            def __init__(self): self.calls = 0
            async def on_question_presented(self, delivery, via):
                self.calls += 1
                started.set()
                await release.wait()
                return await original.on_question_presented(delivery, via)
        held = runtime.answer_source = HeldAnswerSource()
        await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'first-display', 'input-one', '질문'))
        await asyncio.wait_for(started.wait(), .3)
        await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'second-display', 'input-two', '같은 질문'))
        await asyncio.sleep(0)
        self.assertEqual(1, held.calls)
        self.assertEqual([], self.followups(evidence))
        self.assertIn(('Q1', 2), runtime.question_followup_keys)
        release.set()
        await asyncio.wait_for(runtime.drain(), .5)
        self.assertEqual(1, held.calls)
        self.assertEqual(1, len(self.followups(evidence)))
        self.assertEqual(['ANSWER_QUESTION'], [e['operation'] for e in runtime.effects])

    async def test_callback_exception_releases_reservation_without_consuming_planned_user_input(self):
        case, runtime, model, evidence = self.build(voice_on=False)
        await self.register_replacement(case, runtime)
        original = runtime.answer_source
        class FailFirstAnswerSource:
            def __init__(self): self.calls = 0
            async def on_question_presented(self, delivery, via):
                self.calls += 1
                if self.calls == 1:
                    raise RuntimeError('scripted callback transport failure')
                return await original.on_question_presented(delivery, via)
        failing = runtime.answer_source = FailFirstAnswerSource()
        await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'first-display', 'input-one', '질문'))
        await runtime.drain()
        self.assertNotIn(('Q1', 2), runtime.question_followup_keys)
        self.assertEqual([], self.followups(evidence))
        self.assertFalse(original.plans[0].get('used'))
        await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'second-display', 'input-two', '같은 질문'))
        await asyncio.wait_for(runtime.drain(), .5)
        self.assertEqual(2, failing.calls)
        self.assertEqual(1, len(self.followups(evidence)))
        self.assertIn(('Q1', 2), runtime.question_followup_keys)
        self.assertTrue(any(row['kind'] == 'ASYNC_JOB_FAILED' for table, row in evidence.rows if table == 'events'))

    async def test_clarify_returns_same_request_refinement_without_new_request_callback(self):
        case, runtime, model, evidence = self.build()
        await self.register_replacement(case, runtime)
        ports = Ports(runtime, 'same-request', 'root-input', '응.')
        answer = await ports.clarify('unused proposed text', {'question_refs': [{'question_ref': 'Q1', 'revision': 2}]})
        await asyncio.wait_for(runtime.drain(), .5)
        self.assertEqual(case['followups'][0]['transcript'], answer['transcript'])
        self.assertEqual(('Q1', 2), (answer['question_id'], answer['question_revision']))
        self.assertEqual(0, runtime.current_input_generation)
        self.assertEqual({}, runtime.requests)
        self.assertFalse([1 for _, role, _ in model.calls if role == 'CINTERPRET'])
        self.assertEqual(1, len(self.followups(evidence)))
        self.assertNotIn('request_id', self.followups(evidence)[0])

    async def test_unplanned_task_question_receives_no_neutral_reply_even_via_clarify(self):
        case, runtime, model, evidence = self.build(case_id='S5-01', answer=False, voice_on=False)
        question = next(iter(runtime.owners.questions.values()), None)
        if question is None:
            await runtime.external_event(case['events'][0])
            question = next(iter(runtime.owners.questions.values()))
        ports = Ports(runtime, 'task-question-request', 'input', '현재 질문')
        result = await ports.clarify(question['text'], {'question_refs': [{'question_ref': question['question_id'],
                                                                         'revision': question['revision']}]})
        await asyncio.wait_for(runtime.drain(), .5)
        self.assertTrue(result['no_answer'])
        self.assertEqual([], self.followups(evidence))
        self.assertEqual(0, runtime.answer_source.unexpected_count)

    async def test_generated_via_clarification_preserves_planned_and_neutral_user_inputs(self):
        for case_id, text, scope, planned in (
            ('S6-01', '3분기와 4분기 중 어느 보고서를 어떤 형식으로 수정할까요?', {'task_refs': ['T3분기', 'T4분기']}, True),
            ('S1-01', '어떤 내용을 확인할까요?', None, False),
        ):
            with self.subTest(case_id=case_id):
                case, runtime, model, evidence = self.build(case_id=case_id, answer=False, voice_on=False)
                answer = await Ports(runtime, 'same-request', 'input', case['transcript']).clarify(text, scope)
                await runtime.drain()
                expected = case['followups'][0] if planned else case['unexpected_question_reply']
                self.assertEqual(expected['transcript'], answer['transcript'])
                self.assertEqual(0, runtime.current_input_generation)
                self.assertEqual({}, runtime.requests)
                self.assertEqual('VIA_CLARIFICATION', runtime.owners.questions[answer['question_id']]['origin'])
                self.assertEqual(planned, self.followups(evidence)[0]['eligible_planned_reply'])

    async def test_wrong_or_partial_question_text_and_nonmatching_scope_never_receive_planned_answer(self):
        for text, changed_scope in [('민수 연락처를 조회할까요?', False), ('메일을 발송할까요?', False),
                                    ('민수의 연락처를 조회해 메일 업무에 제공할까요?', True)]:
            with self.subTest(text=text, changed_scope=changed_scope):
                case, runtime, model, evidence = self.build(question_text=text, voice_on=False)
                await self.register_replacement(case, runtime)
                if changed_scope:
                    runtime.owners.questions['Q1']['scope']['uses'] = ['READ_CONTACT']
                await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
                    Ports(runtime, 'question-request', 'input', '질문'))
                await asyncio.wait_for(runtime.drain(), .5)
                self.assertEqual([], self.followups(evidence))
                self.assertEqual(0, runtime.current_input_generation)
                self.assertFalse(hasattr(runtime, 'pending_consent_source'))

    async def test_task_execution_or_revision_change_rejects_question_before_presentation(self):
        for key, value in (('execution_id', 'new-execution'), ('request_revision', 2)):
            with self.subTest(field=key):
                case, runtime, model, evidence = self.build(voice_on=False)
                await self.register_replacement(case, runtime)
                self.assertEqual(1, runtime.owners.questions['Q1']['request_revision'])
                runtime.owners.tasks['T메일'][key] = value
                delivery = await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
                    Ports(runtime, 'question-request', 'input', '질문'))
                await runtime.drain()
                self.assertFalse(delivery['delivered_receipt'])
                self.assertEqual('QUESTION_OWNER_OR_REVISION_MISMATCH', delivery['discard_reason'])
                self.assertEqual([], self.followups(evidence))

    async def test_partial_voice_and_withdrawal_before_receipt_never_start_followup(self):
        case, runtime, model, evidence = self.build()
        await self.register_replacement(case, runtime)
        runtime.profile['characters_per_second'] = 20.
        pending = asyncio.create_task(runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'question-request', 'input', '질문')))
        while not runtime.playing:
            await asyncio.sleep(.001)
        await runtime.input_started('interrupt')
        await runtime.speech_end('interrupt')
        delivery = await asyncio.wait_for(pending, .5)
        await runtime.drain()
        self.assertFalse(delivery['delivered_receipt'])
        self.assertNotEqual(delivery['text'], delivery['voice_text'])
        self.assertEqual([], self.followups(evidence))
        # A separate actual held output is withdrawn before either channel.
        case, runtime, model, evidence = self.build(voice_on=False)
        await self.register_replacement(case, runtime)
        await runtime.input_started('speaking')
        pending = asyncio.create_task(runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'question-request', 'input', '질문')))
        await asyncio.sleep(.005)
        await runtime.external_event({'event_id': 'withdraw-v2', 'type': 'QUESTION_WITHDRAWN',
            'payload': {'question_id': 'Q1', 'revision': 2}})
        await runtime.speech_end('speaking')
        delivery = await asyncio.wait_for(pending, .5)
        await runtime.drain()
        self.assertFalse(delivery['delivered_receipt'])
        self.assertIsNone(delivery['text_presented_s'])
        self.assertEqual([], self.followups(evidence))

    async def test_withdrawal_during_user_wait_and_reverse_revision_events_cannot_answer_stale_question(self):
        case, runtime, model, evidence = self.build(answer=False, voice_on=False, wait=.03)
        await self.register_replacement(case, runtime)
        old = copy.deepcopy(case['events'][1])
        old['event_id'] = 'late-old-question'
        old['payload']['question']['revision'] = 1
        await runtime.external_event(old)
        self.assertEqual(2, runtime.owners.questions['Q1']['revision'])
        await runtime.perform_goal(self.present_goal(), {'goals': [], 'constraints': []},
            Ports(runtime, 'question-request', 'input', '질문'))
        await asyncio.sleep(.005)
        await runtime.external_event({'event_id': 'late-old-withdraw', 'type': 'QUESTION_WITHDRAWN',
            'payload': {'question_id': 'Q1', 'revision': 1}})
        self.assertEqual('VALID', runtime.owners.questions['Q1']['status'])
        await runtime.external_event({'event_id': 'current-withdraw', 'type': 'QUESTION_WITHDRAWN',
            'payload': {'question_id': 'Q1', 'revision': 2}})
        await asyncio.wait_for(runtime.drain(), .5)
        self.assertEqual([], self.followups(evidence))
        self.assertFalse(hasattr(runtime, 'pending_consent_source'))
        self.assertEqual(0, runtime.current_input_generation)
        self.assertIsNone(runtime.question_focus)
        self.assertTrue(any(row['kind'] == 'FOLLOWUP_INPUT_ABORTED' for table, row in evidence.rows if table == 'events'))
