import unittest
from benchmark.architecture.dp41_45_poc.normalize import normalize_trial
from prototypes.candidates.dp41_45_poc.shared.timing import CausalTimeline


class CoreOnsetTests(unittest.TestCase):
    def test_core_after_greeting_uses_observed_character_not_first_output(self):
        t=CausalTimeline(clock=lambda:0)
        start=t.point('INPUT_STARTED',0)
        speech=t.span('EXTERNAL_USER_SPEECH','USER',0,2,parents=[start])
        text=t.point('TEXT_PRESENTED',3,parents=[speech])
        voice=t.point('VOICE_ONSET',3.25,parents=[text])
        core=t.point('VOICE_CHARACTER',3.75,parents=[voice])
        delivery={'publication_id':'P1','text':'네. 마감일은 15일입니다.','voice_text':'네. 마감일은 15일입니다.',
                  'voice_on':True,'text_presented_s':3,'voice_onset_s':3.25,'text_node_id':text,'causal_node_id':voice,
                  'voice_char_events':[{'char_index':0,'onset_s':3.25,'causal_node_id':voice},
                                       {'char_index':3,'onset_s':3.75,'causal_node_id':core}]}
        trial={'variant':'A','repeat':1,'case_id':'S1-01','causal_timeline':t.snapshot(),
               'scene_origin_evidence_seconds':0,'events':[], 'observed_inputs':[{'speech_end_s':2}],
               'deliveries':[delivery],'reaction_blueprint':[{'id':'I1','endpoint_kind':'MEANINGFUL_RESPONSE','start_kind':'SPEECH_END'}],
               'cycle_blueprint':{'id':'T1'},'evidence_level':'MEASURED_MODEL_WITH_SIMULATED_SPEECH'}
        judged={'final_judgement':{'conditions':{},'reactions':[{'id':'I1','valid_publication_id':'P1','first_actual_publication_id':'P1',
                    'valid_core_quote':'마감일은 15일입니다.','first_actual_core_quote':'마감일은 15일입니다.',
                    'wrong_first':False,'evidence':['P1'],'reason':'actual core clause'}],
                 'cycle':{'valid_publication_id':'P1','valid_core_quote':'마감일은 15일입니다.','evidence':['P1'],'reason':'actual result'},'critical_violations':[]},'method':'TEST'}
        result,_=normalize_trial(trial,judged)
        self.assertAlmostEqual(result['reactions'][0]['attributed_seconds'],1.75)
        self.assertAlmostEqual(result['cycle']['attributed_seconds'],1.75)
        delivery['voice_text']='네. '
        with self.assertRaisesRegex(ValueError,'actual speech-sink'):
            normalize_trial(trial,judged)
