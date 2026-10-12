import unittest
from benchmark.architecture.dp41_45_poc.worker import AnswerSource
from benchmark.architecture.dp41_45_poc.fixtures import load_cases


class AnswerSourceTests(unittest.TestCase):
    def test_related_question_uses_actual_scope_and_allows_paraphrase(self):
        source=AnswerSource(next(c for c in load_cases() if c['case_id']=='S6-01'))
        self.assertTrue(source.report_scope_question('3분기와 4분기 중 어느 업무를 어떤 형식으로 수정할까요?',None))
        self.assertTrue(source.report_scope_question('어떤 형식으로 바꿀까요?',{'target':'T3분기'}))
        self.assertFalse(source.report_scope_question('메일을 보내도 될까요?',{'target':'T메일'}))
        self.assertFalse(source.report_scope_question('연락처 주소를 알려주세요.',{'target':'민수'}))
