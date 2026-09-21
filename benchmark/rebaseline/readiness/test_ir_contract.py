import copy
import json
import unittest
from ir_contract import clean_input, json_validate, merge_stages, prepare, schemas, validate_result


def fixture():
    return {'case_id':'SMOKE-SEMANTIC-NOT-SCORED', 'request_revision':2,
            'original_request':'아까 문서에 결론 한 장 추가해줘', 'interaction_events':[],
            'visible_context':[], 'conversation':[], 'task_views':[{'task_id':'T1','revision':4}],
            'pending_interactions':[{'id':'P1','task_id':'T1'}],
            'capabilities':['document.update'], 'policy_constraints':[]}


def outputs():
    common={'status':'READY','request_revision':2,'needed_context':[],'clarification':None}
    ground={**common,'requests':[{'id':'r1','text':'결론 한 장 추가', 'referents':[], 'constraints':['한 장']}], 'relations':[]}
    association={**common,'associations':[{'request_id':'r1','task_relation':'existing_task','task_id':'T1','task_view_revision':4,'pending_interaction_id':None}], 'correction_reason':None}
    handling={**common,'decisions':[{'request_id':'r1','handling':'downstream_agent','required_capability':'document.update'}], 'correction_stage':None,'correction_reason':None}
    return ground, association, handling


class IrContractTests(unittest.TestCase):
    def setUp(self):
        self.case=fixture()
        self.g,self.a,self.h=outputs()

    def test_task_relation_is_exactly_three_values(self):
        s=schemas()['integrated']['properties']['requests']['items']['properties']
        self.assertEqual(s['task_relation']['enum'], ['no_tracked_task','new_task','existing_task'])
        self.assertIn('pending_interaction_id',s)

    def test_correction_enum_is_not_accidentally_shared(self):
        s=schemas()
        self.assertNotIn('CORRECT_PRIOR_STAGE',s['grounding']['properties']['status']['enum'])
        self.assertNotIn('CORRECT_PRIOR_STAGE',s['integrated']['properties']['status']['enum'])
        self.assertIn('CORRECT_PRIOR_STAGE',s['association']['properties']['status']['enum'])

    def test_grounding_and_integrated_share_provenance_shape(self):
        a=schemas()['grounding']['properties']['requests']['items']['properties']
        b=schemas()['integrated']['properties']['requests']['items']['properties']
        for key in a:
            self.assertEqual(a[key],b[key])

    def test_oracle_top_level_rejected(self):
        self.case['oracle']={'answer':'T1'}
        with self.assertRaises(ValueError): clean_input(self.case)

    def test_oracle_nested_rejected(self):
        self.case['visible_context']=[{'source_id':'s','expected_output':'T1'}]
        with self.assertRaises(ValueError): clean_input(self.case)

    def test_model_is_not_called_by_prepare(self):
        result=prepare('integrated',self.case,'reference-model')
        self.assertEqual(result['model_execution'],'NOT_RUN')
        self.assertIsNone(result['input_tokens'])
        self.assertEqual(len(result['request_sha256']),64)

    def test_payload_fingerprint_stable(self):
        self.assertEqual(prepare('integrated',self.case,'m'),prepare('integrated',copy.deepcopy(self.case),'m'))

    def test_payload_preserves_original_request(self):
        result=prepare('association',self.case,'m',{'grounding':self.g})
        payload=json.loads(result['request']['messages'][1]['content'])
        self.assertEqual(payload['input']['original_request'],self.case['original_request'])
        self.assertEqual(payload['prior']['grounding'],self.g)


    def test_grounding_input_excludes_task_and_agent_evidence(self):
        payload=json.loads(prepare('grounding',self.case,'m')['request']['messages'][1]['content'])
        self.assertEqual(set(payload['input']), {'case_id','request_revision','original_request','interaction_events','visible_context','conversation'})
        self.assertNotIn('task_views',payload['input'])
        self.assertNotIn('capabilities',payload['input'])

    def test_association_input_excludes_raw_visible_context_and_capabilities(self):
        payload=json.loads(prepare('association',self.case,'m',{'grounding':self.g})['request']['messages'][1]['content'])
        self.assertEqual(set(payload['input']), {'case_id','request_revision','original_request','conversation','task_views','pending_interactions'})
        self.assertEqual(payload['prior']['grounding'],self.g)

    def test_handling_input_exposes_capabilities_policy_not_raw_source(self):
        payload=json.loads(prepare('handling',self.case,'m',{'grounding':self.g,'association':self.a})['request']['messages'][1]['content'])
        self.assertEqual(set(payload['input']), {'case_id','request_revision','original_request','capabilities','policy_constraints'})
        self.assertNotIn('visible_context',payload['input'])

    def test_controller_feedback_is_explicit_and_not_part_of_initial_case(self):
        payload=json.loads(prepare('grounding',self.case,'m',controller_feedback='referent version mismatch')['request']['messages'][1]['content'])
        self.assertEqual(payload['controller_feedback'],'referent version mismatch')
        self.assertNotIn('controller_feedback',payload['input'])


    def test_sampling_contract_and_seed_are_explicit(self):
        request=prepare('integrated',self.case,'m',seed=123)['request']
        self.assertEqual(request['temperature'],0.7)
        self.assertEqual(request['top_p'],0.8)
        self.assertEqual(request['top_k'],20)
        self.assertEqual(request['min_p'],0.0)
        self.assertEqual(request['presence_penalty'],1.5)
        self.assertEqual(request['seed'],123)
        self.assertFalse(request['chat_template_kwargs']['enable_thinking'])

    def test_seed_changes_request_fingerprint_without_changing_input(self):
        a=prepare('integrated',self.case,'m',seed=1)
        b=prepare('integrated',self.case,'m',seed=2)
        self.assertNotEqual(a['request_sha256'],b['request_sha256'])
        pa=json.loads(a['request']['messages'][1]['content'])
        pb=json.loads(b['request']['messages'][1]['content'])
        self.assertEqual(pa['input'],pb['input'])

    def test_invented_pending_task_relation_rejected(self):
        self.a['associations'][0]['task_relation']='pending_interaction'
        with self.assertRaises(ValueError): validate_result('association',self.a,self.case,{'grounding':self.g})

    def test_pending_binding_is_independent(self):
        self.a['associations'][0]['pending_interaction_id']='P1'
        validate_result('association',self.a,self.case,{'grounding':self.g})
        self.assertEqual(self.a['associations'][0]['task_relation'],'existing_task')

    def test_wrong_task_revision_rejected(self):
        self.a['associations'][0]['task_view_revision']=3
        with self.assertRaises(ValueError): validate_result('association',self.a,self.case,{'grounding':self.g})

    def test_missing_request_in_stage_output_rejected(self):
        self.a['associations']=[]
        with self.assertRaises(ValueError): validate_result('association',self.a,self.case,{'grounding':self.g})

    def test_duplicate_request_in_stage_output_rejected(self):
        self.a['associations']*=2
        with self.assertRaises(ValueError): validate_result('association',self.a,self.case,{'grounding':self.g})

    def test_unknown_agent_capability_rejected(self):
        self.h['decisions'][0]['required_capability']='magic.execute'
        with self.assertRaises(ValueError): validate_result('handling',self.h,self.case,{'grounding':self.g,'association':self.a})

    def test_conditional_edge_requires_condition(self):
        second=copy.deepcopy(self.g['requests'][0]);second['id']='r2';self.g['requests'].append(second)
        self.g['relations']=[{'from':'r1','to':'r2','type':'conditional','condition':None}]
        with self.assertRaises(ValueError): validate_result('grounding',self.g,self.case)

    def test_cyclic_relation_rejected(self):
        second=copy.deepcopy(self.g['requests'][0]);second['id']='r2';self.g['requests'].append(second)
        self.g['relations']=[{'from':'r1','to':'r2','type':'sequential','condition':None},{'from':'r2','to':'r1','type':'data_dependent','condition':None}]
        with self.assertRaises(ValueError): validate_result('grounding',self.g,self.case)

    def test_ready_cannot_request_more_context(self):
        self.g['needed_context']=[{'source_id':'s','scope':'page1','reason':'missing'}]
        with self.assertRaises(ValueError): validate_result('grounding',self.g,self.case)

    def test_prepare_does_not_invent_later_stage_outputs(self):
        with self.assertRaises(ValueError): prepare('handling',self.case,'m')

    def test_stages_merge_without_losing_constraints(self):
        result=merge_stages(self.case,self.g,self.a,self.h)
        self.assertEqual(result['requests'][0]['constraints'],['한 장'])
        self.assertEqual(result['requests'][0]['task_id'],'T1')
        self.assertEqual(result['requests'][0]['required_capability'],'document.update')

    def test_boolean_revision_is_not_integer(self):
        self.g['request_revision']=True
        with self.assertRaises(ValueError): json_validate(self.g,schemas()['grounding'])


if __name__=='__main__': unittest.main()
