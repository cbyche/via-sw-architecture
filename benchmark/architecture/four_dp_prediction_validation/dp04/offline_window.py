"""Run only in root-granted isolated offline window. Actual local parser and Rust acceptance."""
from pathlib import Path
import subprocess,json,sys,time
BASE=Path(__file__).resolve().parent;REPO=BASE.parents[3];OUT=REPO/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04';PYTHON='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';commands=[]
def command(args,label,timeout=300):
 t=time.time();p=subprocess.run(args,capture_output=True,text=True,timeout=timeout);entry={'label':label,'command':args,'returncode':p.returncode,'wall_s':time.time()-t,'stdout':p.stdout,'stderr':p.stderr};commands.append(entry);(OUT/'offline-window-commands.json').write_text(json.dumps(commands,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'label':label,'returncode':p.returncode,'wall_s':entry['wall_s']}),flush=True)
 if p.returncode:raise RuntimeError(label+' failed; preserved log')
if sys.argv[-1]=='parser':command([PYTHON,str(BASE/'parser_preflight.py')],'actual-complete-parser-protocol',360)
elif sys.argv[-1]=='accept':
 command(['cargo','test','--offline','--manifest-path',str(REPO/'prototypes/candidates/four_dp_prediction_validation/dp04/Cargo.toml')],'baseline-Rust-regressions')
 command([PYTHON,str(BASE/'offline_accept.py')],'new-heldout-24-structural-paths')
 command([PYTHON,str(BASE/'qa04_controls_accept.py')],'new-C01-C03-actual-acceptance')
 for task in ['C05','C06']:
  for variant in ['A','B']:
   crate=OUT/'qa04'/task/variant/'candidate';command(['cargo','build','--offline','--manifest-path',str(crate/'Cargo.toml')],task+'-'+variant+'-build');command([PYTHON,str(BASE/'qa04_feature_accept.py'),task,'--variant',variant],task+'-'+variant+'-actual-acceptance')
else:raise SystemExit('parser or accept')
