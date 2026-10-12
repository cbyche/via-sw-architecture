"""Replay identical benign API input to classify a preparation provider rejection."""
import asyncio, hashlib, json, sys
from pathlib import Path
PREPARATION=Path(__file__).resolve().parents[1]
SOURCE=PREPARATION.parent/'dp41-p21-mac-text-v6-20261012/frozen-source'
sys.path.insert(0,str(SOURCE))
from prototypes.candidates.dp41_45_poc.shared.evidence import Evidence, canonical
from prototypes.candidates.dp41_45_poc.shared.providers import ModelClient
ORIGINAL=PREPARATION/'v6-question-followup-preflight-B/model-contexts/8815576d-ba00-4dc8-b055-bfbf87f5de16.json'
async def run():
 request=json.loads(ORIGINAL.read_text())
 evidence=Evidence(Path(__file__).resolve().parent/'raw','B',0)
 client=ModelClient(evidence,'qwen',max_cost_usd=1,budget_file=PREPARATION/'global-budgets/qwen.json')
 outcomes=[]
 for attempt in (1,2):
  try:
   await client.call(request['messages'],'CINTERPRET',request_id='SERVICE_HEALTH_IDENTICAL_CONTEXT',case_id='S6-01',max_tokens=request['max_tokens'])
   outcomes.append({'attempt':attempt,'status':'RETURNED'})
   break
  except Exception as e:
   outcomes.append({'attempt':attempt,'status':'REJECTED','error_type':type(e).__name__})
 contexts=list((Path(__file__).resolve().parent/'raw/model-contexts').glob('*.json'))
 assert all(json.loads(p.read_text())==request for p in contexts)
 record={'evidence_level':'SERVICE_HEALTH_REPLAY_NOT_CANDIDATE_TRIAL','included_in_final_population':False,'original_context_sha256':hashlib.sha256(ORIGINAL.read_bytes()).hexdigest(),'context_values_identical':True,'outcomes':outcomes,'original_rejection_preserved':True}
 (Path(__file__).resolve().parent/'summary.json').write_text(json.dumps(record,indent=2)+'\n')
 print(json.dumps(record),flush=True)
asyncio.run(run())
