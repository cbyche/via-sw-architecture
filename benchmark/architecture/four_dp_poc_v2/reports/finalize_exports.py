"""Final derived exports only, after exact120 BASIC and27 authorized tactic trials plus9 input-representation diagnostics."""
import datetime,hashlib,json,shutil,subprocess,sys
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018';ANALYSIS=ROOT/'analysis';SCRIPTS=Path(__file__).resolve().parent
def read(p):return json.loads(Path(p).read_text())
def run(args):subprocess.run([sys.executable,*map(str,args)],check=True)
def pin_analysis_sources():
 stamp=datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ');snapshot=ANALYSIS/'source-snapshots'/stamp;files={};base=REPO/'benchmark/architecture/four_dp_poc_v2'
 for source in sorted(p for p in base.rglob('*')if p.is_file()and p.suffix in ('.py','.js','.html')and'__pycache__'not in p.parts):
  relative=source.relative_to(REPO);dest=snapshot/relative;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest);files[str(relative)]={'sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'snapshot':str(dest.resolve())}
 manifest={'version':'EXPLICIT_FINAL_ANALYSIS_SOURCE_BUNDLE','generated_utc':stamp,'canonical_root':str(base),'files':files,'scope':'Actual current canonical analysis/report/runner sources; candidate/evaluator input versions remain pinned independently per phase. Historical unknown provenance is not reconstructed.'}
 (ANALYSIS/'final-analysis-source-manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n');return manifest

def main():
 expected={'basic-v1':120,'tactic-dp04-A4':12,'tactic-dp04-B4':12,'tactic-dp01-A2':3,'diagnostic-dp05-numbered-lines':9,'basic-dp05-reuse-union-corrected-v2':36}
 for name,n in expected.items():
  p=ROOT/name;records=[json.loads(x)for x in(p/'runs.jsonl').read_text().splitlines()if x];jobs=read(p/'MODEL-CALLS-AUTHORIZED.json')['allowed_jobs'];key=lambda r:(r['dp'],r['episode'],r['variant'],r['repetition'])
  assert len(records)==n and len(jobs)==n and {key(r)for r in records}=={key(r)for r in jobs},name
 stopped=ROOT/'basic-dp05-reuse-corrected-v1';stop=read(stopped/'PHASE-STOPPED.json');assert stop['completed_trials']==11 and stop['unexecuted_trials']==25 and hashlib.sha256((stopped/'runs.jsonl').read_bytes()).hexdigest()==stop['runs_sha256']
 ledger=read(ROOT/'budget.json')['calls'];assert not any(c['state']=='RESERVED'for c in ledger.values()),'Actual provider call still in flight'
 run([SCRIPTS/'publish_corrected_source.py']);source_bundle=pin_analysis_sources()
 phases=[]
 for marker in sorted(ROOT.glob('*/MODEL-CALLS-AUTHORIZED.json')):
  p=marker.parent
  if not(p/'runs.jsonl').exists():continue
  run([SCRIPTS/'export_details.py',p]);phases.append(p)
 run([SCRIPTS/'memory_option_summary.py']);run([SCRIPTS/'dp05_phase_report.py']);run([SCRIPTS/'compose_effective_comparison.py']);run([SCRIPTS/'campaign_totals.py']);run([SCRIPTS/'tactic_comparison.py']);run([SCRIPTS/'input_representation_comparison.py'])
 run([SCRIPTS/'dp05_correction_comparison.py']);run([SCRIPTS/'first_callback_scope_diagnostics.py'])
 run([SCRIPTS/'publication_summary.py']);run([SCRIPTS/'build_replay.py',ROOT,'--include-calibration'])
 for relative,pin in source_bundle['files'].items():assert hashlib.sha256((REPO/relative).read_bytes()).hexdigest()==pin['sha256'],'Canonical source changed during final derivation '+relative
 inventory={'generation':'CORRECTED_REVIEW_V2','basic_episode_trials':120,'post_basic_episode_trials':27,'calibration_episode_trials':26,'additional_input_representation_episode_trials':9,'corrected_dp05_episode_trials':36,'all_actual_episode_trials':229,'boundary_stopped_intermediate_trials':11,'scope':'No additional model calls; final derived analysis/replay. Recorded callback proxy, not physical GUI. Per-phase immutable candidate/evaluator provenance preserved.','files':{str(p.relative_to(ROOT)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size}for p in [ROOT/'README.md',ROOT/'final-measurement-summary.json',ROOT/'final-measurement-summary.md',ROOT/'campaign-totals.json',ROOT/'campaign-totals.md',ROOT/'tactic-comparison.json',ROOT/'tactic-comparison.md',ROOT/'demo.html',ANALYSIS/'memory-option-peaks.json',ANALYSIS/'memory-option-peaks.csv',ANALYSIS/'memory-option-peaks.md',ROOT/'basic-v1/summary.json',ROOT/'basic-v1/case-summary.csv',ROOT/'basic-v1/individual-trials.md',ROOT/'basic-v1/episode-metrics.csv',ROOT/'effective-main-selection.json',ROOT/'effective-main-comparison/report.md',ROOT/'effective-main-comparison/summary.json',ROOT/'effective-main-comparison/episode-metrics.csv',ROOT/'input-representation-comparison.json',ROOT/'dp05-correction-comparison.json',ROOT/'dp05-correction-comparison.md',ROOT/'qa04/final-effective-cross-review.md',ROOT/'qa04/final-effective-cross-review.json',ANALYSIS/'qa01-callback-proxy-review.md',ANALYSIS/'first-callback-scope-diagnostics.json']},'actual_phase_roots':[str(p)for p in phases],'durable_corrected_source':str((REPO/'prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/EFFECTIVE-SOURCE.json').resolve()),'reproduction_entry':str((REPO/'benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md').resolve()),'analysis_source_manifest':str(ANALYSIS/'final-analysis-source-manifest.json'),'analysis_source_manifest_sha256':hashlib.sha256((ANALYSIS/'final-analysis-source-manifest.json').read_bytes()).hexdigest()}
 (ROOT/'artifact-inventory.json').write_text(json.dumps(inventory,ensure_ascii=False,indent=2)+'\n');print(json.dumps({'final_exports':True,'files':len(inventory['files']),'actual_phases':len(phases)}))
if __name__=='__main__':main()
