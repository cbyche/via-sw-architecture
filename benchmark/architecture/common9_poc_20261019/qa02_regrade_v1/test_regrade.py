import unittest

from regrade import associate, canonical, intended_targets


class ReferentRegressionTests(unittest.TestCase):
    def setUp(self):
        self.catalog = [{'id': 'DOC', 'revision': 1, 'is_current': False, 'line_count': 5,
                         'extents': {'P1': [4, 4], 'all': [1, 5]}},
                        {'id': 'DOC', 'revision': 2, 'is_current': True, 'line_count': 5,
                         'extents': {'P1': [4, 4], 'all': [1, 5]}},
                        {'id': 'HISTORY', 'revision': 1, 'is_current': True, 'line_count': 1,
                         'extents': {'delivered': [1, 1]}}]

    def test_whole_source_alias_is_same_extent(self):
        self.assertEqual(canonical('HISTORY', self.catalog), canonical('HISTORY:v1:delivered', self.catalog))

    def test_original_is_not_delivered_explanation(self):
        self.assertNotEqual(canonical('DOC:v1:P1', self.catalog), canonical('HISTORY', self.catalog))

    def test_required_revision_is_not_erased(self):
        self.assertNotEqual(canonical('DOC:v1:P1', self.catalog), canonical('DOC:v2:P1', self.catalog))
        self.assertEqual(canonical('DOC', self.catalog), ('DOC', 2, 1, 5))

    def test_unknown_source_cannot_be_invented(self):
        with self.assertRaises(ValueError):
            canonical('GUESS:v1:P1', self.catalog)

    def test_longer_expression_keeps_its_actual_target(self):
        row = {'binding_id': 'B', 'input_id': 'U', 'plan': {'bindings': [
            {'expression': '아까 설명했던 원본 부분을 보여줘.', 'targets': ['DOC:v1:P1']}], 'targets': []}}
        self.assertEqual(associate(row, {'B': ['아까 설명했던 원본 부분을 보여줘.']}, {'U': 1})[0], ['DOC:v1:P1'])

    def test_explicit_wrong_binding_not_repaired_by_top_level(self):
        row = {'binding_id': 'B', 'input_id': 'U', 'plan': {'bindings': [
            {'expression': '이것', 'targets': ['DOC:v1:P1']}], 'targets': ['HISTORY']}}
        self.assertEqual(associate(row, {'B': ['이것']}, {'U': 1})[0], ['DOC:v1:P1'])

    def test_unassigned_union_cannot_fill_two_references(self):
        row = {'binding_id': 'B', 'input_id': 'U', 'plan': {'bindings': [], 'targets': ['A', 'B']}}
        self.assertEqual(associate(row, {'B': ['첫째']}, {'U': 2})[:2], ([], 'ATTRIBUTION_REVIEW'))

    def test_no_adoption_cannot_receive_credit(self):
        self.assertEqual(associate({'plan': None}, {}, {})[:2], ([], 'NO_ADOPTION'))

    def test_revised_result_needs_actual_prior_delivery(self):
        row = {'requests': [{'origin_event_id': 'S18-U02', 'task_id': 'TASK-5', 'goal_id': 'G'}],
               'tasks': [{'task_id': 'TASK-5', 'result_source_id': 'RESULT'}], 'delivered': [],
               'source_catalog': [{'id': 'RESULT', 'revision': 1, 'is_current': True, 'line_count': 1, 'extents': {'result': [1, 1]}}]}
        with self.assertRaisesRegex(ValueError, 'NOT_AVAILABLE'):
            intended_targets(row, {'expected_set': ['TA:revised-result']})
        row['delivered'] = [{'stimulus_id': 'S18-A01', 'task_id': 'TASK-5', 'goal_id': 'G', 'phase': 'AGENT_RESULT'}]
        self.assertEqual(intended_targets(row, {'expected_set': ['TA:revised-result']})[0], [('RESULT', 1, 1, 1)])


if __name__ == '__main__':
    unittest.main()
