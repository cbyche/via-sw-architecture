"""Offline campaign identity/authorization/OS-lock guards. No harness/model dispatch."""
import hashlib,json,subprocess,sys,time
from pathlib import Path
from unittest.mock import patch
import campaign
OUT=campaign.RESULT/'development'/('campaign-guards-'+str(time.monotonic_ns()));OUT.mkdir(parents=True)
JOB={'configuration':'AAAA','episode':'S02','rep':1}
def folder(name,progress,allowed=None):
 root=OUT/name;root.mkdir();(root/'freeze.json').write_text('{}');(root/'MODEL-CALLS-AUTHORIZED.json').write_text(json.dumps({'freeze_sha256':hashlib.sha256(b'{}').hexdigest(),'allowed_jobs':[JOB]if allowed is None else allowed}));(root/'campaign-progress.jsonl').write_text(''.join(json.dumps({**JOB,**p})+'\n'for p in progress));return root
checks=[]
with patch.object(campaign,'validate',return_value={'purpose':'FORMAL_REVIEW_CANDIDATE','jobs':[JOB]}),patch.object(campaign,'seed_budget',return_value={}),patch.object(campaign.subprocess,'Popen',side_effect=AssertionError('No dispatch permitted in offline guard')):
 root=folder('orphan',[{'state':'STARTED'}]);before=campaign.sha(root/'campaign-progress.jsonl')
 try:campaign.run(root)
 except RuntimeError as error:assert'Orphaned'in str(error)
 else:raise AssertionError('Orphan silently rerun')
 assert(root/'ORPHANED-STARTED-JOBS.json').exists()and before==campaign.sha(root/'campaign-progress.jsonl');checks.append('orphanSTARTEDpreservedNoAutomaticRetry')
 root=folder('terminated-failure',[{'state':'STARTED'},{'state':'TERMINATED','returncode':99}]);before=campaign.sha(root/'campaign-progress.jsonl');campaign.run(root);assert before==campaign.sha(root/'campaign-progress.jsonl');checks.append('terminatedFailureIdentityNotReexecuted')
 root=folder('unauthorized',[],[])
 try:campaign.run(root)
 except RuntimeError as error:assert'authorization'in str(error)
 else:raise AssertionError('Unauthorized job dispatched')
 assert not(root/'campaign-progress.jsonl').read_text();checks.append('outsideWindowJobNotDispatched')
# Real OS contention between independent Python controllers; the controller body
# is replaced only inside these worker processes, so no authorization is created
# and no harness/model/credential access can occur.
root=OUT/'exclusive-controller-root';root.mkdir()
worker=r"""
import sys
from pathlib import Path
import campaign
root=Path(sys.argv[1]);mode=sys.argv[2]
def offline_body(actual_root):
 assert actual_root==root.resolve()
 print('ENTERED',flush=True)
 if mode=='hold':sys.stdin.readline()
 if mode=='fail':raise RuntimeError('INJECTED_OFFLINE_CONTROLLER_FAILURE')
campaign._run_locked=offline_body
campaign.run(root)
"""
def start(mode):return subprocess.Popen([sys.executable,'-c',worker,str(root),mode],cwd=campaign.H,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
first=start('hold')
try:
 assert first.stdout.readline().strip()=='ENTERED'
 second=start('return');stdout,stderr=second.communicate(timeout=10)
 assert second.returncode!=0 and 'already active'in stderr and 'ENTERED'not in stdout
 campaign.dump(OUT/'second-controller-rejection.json',{'returncode':second.returncode,'stdout':stdout,'stderr':stderr})
 assert first.poll()is None
 checks.append('twoActualControllersSecondRejectedBeforeBody')
finally:
 first.stdin.write('release\n');first.stdin.flush();stdout,stderr=first.communicate(timeout=10)
 campaign.dump(OUT/'first-controller-release.json',{'returncode':first.returncode,'stdout':stdout,'stderr':stderr})
assert first.returncode==0
before=(root/'controller-lock-events.jsonl').read_bytes()
third=start('fail');stdout,stderr=third.communicate(timeout=10)
assert third.returncode!=0 and 'ENTERED'in stdout and 'INJECTED_OFFLINE_CONTROLLER_FAILURE'in stderr
campaign.dump(OUT/'injected-controller-failure.json',{'returncode':third.returncode,'stdout':stdout,'stderr':stderr})
checks.append('afterControllerExitLockReacquired')
assert(root/'controller-lock-events.jsonl').read_bytes().startswith(before)
fourth=start('return');stdout,stderr=fourth.communicate(timeout=10)
assert fourth.returncode==0 and 'ENTERED'in stdout
campaign.dump(OUT/'post-failure-controller-reacquisition.json',{'returncode':fourth.returncode,'stdout':stdout,'stderr':stderr})
records=[json.loads(line)for line in(root/'controller-lock-events.jsonl').read_text().splitlines()]
assert sum(r['state']=='CONTROLLER_LOCK_REJECTED'for r in records)==1
assert sum(r['state']=='CONTROLLER_LOCK_ACQUIRED'for r in records)==3
assert sum(r['state']=='CONTROLLER_LOCK_RELEASED'for r in records)==3
assert any('INJECTED_OFFLINE_CONTROLLER_FAILURE'in r.get('outcome','')for r in records)
assert not(root/'MODEL-CALLS-AUTHORIZED.json').exists()
checks.append('failedControllerRawEvidencePreservedAndLockReleased')
campaign.dump(OUT/'summary.json',{'status':'PASS','checks':checks,'actual_API_calls':0,'harness_dispatches':0,'offline_controller_processes':4});print(json.dumps({'status':'PASS','checks':checks,'evidence':str(OUT)}))
