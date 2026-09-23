import unittest
from score import WORKING_IDS,aggregate_change_rows,load_baseline,score_metrics,score_value
class T(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.b=load_baseline()
 def test_contract(self):self.assertEqual(tuple(self.b["metrics"]),WORKING_IDS);self.assertFalse(self.b["not_run_is_score"])
 def test_not_run(self):self.assertTrue(all(r["score"] is None for r in score_metrics({})["rows"]))
 def test_bands(self):
  self.assertEqual(3,score_value(2000,self.b["metrics"]["W-01"]));self.assertEqual(5,score_value(99,self.b["metrics"]["W-05"]))
  self.assertEqual(3,score_value(2,self.b["metrics"]["W-07"]));self.assertEqual(3,score_value(3,self.b["metrics"]["W-08"]))
  self.assertEqual(5,score_value(100,self.b["metrics"]["W-10"]));self.assertEqual(4,score_value(100/24,self.b["metrics"]["W-12"]))
 def test_change_aggregation(self):
  rows=[]
  for c in ("AGENT-DP01/A","AGENT-DP01/B"):
   rows += [{"candidate_id":c,"family":"A","unique_count":1 if c.endswith("/A") else 2} for _ in range(9)]
   rows += [{"candidate_id":c,"family":"M","unique_count":3} for _ in range(9)]
   rows += [{"candidate_id":c,"family":"C","unique_count":3} for _ in range(6)]
  r=aggregate_change_rows(rows);self.assertEqual(1,r["AGENT-DP01/A"]["W-07"]);self.assertEqual(2,r["AGENT-DP01/B"]["W-07"]);self.assertEqual(3,r["AGENT-DP01/A"]["W-08"])
if __name__=="__main__":unittest.main()
