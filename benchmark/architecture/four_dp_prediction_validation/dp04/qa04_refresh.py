"""Reapply four actual independent features onto this generation. No provider API."""
from pathlib import Path
import json,shutil,subprocess,difflib,hashlib,sys,copy
BASE=Path(__file__).resolve().parent;REPO=BASE.parents[3];OUT=REPO/'results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04/qa04';ORIG=REPO/'prototypes/candidates/four_dp_prediction_validation/dp04';OLD=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/qa04'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(p,v):p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def prepare():
 if OUT.exists():raise RuntimeError('Preserve earlier QA04 attempt')
 constants=json.loads((OLD/'controls-corrected-v1/control-constants.json').read_text())
 for task in ['C01','C03','C05','C06']:
  for variant in ['A','B']:
   folder=OUT/task/variant;crate=folder/'candidate';shutil.copytree(ORIG,crate,ignore=shutil.ignore_patterns('target'))
   if task=='C01':(crate/'src/model_wire.rs').write_text(constants['WIRE'])
   elif task=='C03':
    p=crate/'src/source_contract.rs';s=p.read_text();p.write_text(s[:s.index('pub fn normalize(')]+constants['NORMALIZER']);p=crate/'src/main.rs';s=p.read_text();needle='let input:Value=serde_json::from_str(&line).unwrap();';assert s.count(needle)==1;p.write_text(s.replace(needle,needle+'\n    let input=source_contract::normalize_input(&input);'))
   else:
    # Reapply reviewed old feature diff. Its baseline generation remains preserved.
    patch=(OLD/'dp04-rebased-calibration-v3'/task/variant/'actual.patch').read_text()
    # New baseline has an actual parser mode and a changed diagnostic output string.
    patch=patch.replace('let observed = observations::read(revision, ctx)?;','let mut observed = observations::read(revision, ctx)?;').replace('let observed = observations::read(&permitted_revision, ctx)?;','let mut observed = observations::read(&permitted_revision, ctx)?;')
    # Defer final output metadata hunk to exact single-key insertion below.
    lines=patch.splitlines(True);blocks=[];block=[]
    for line in lines:
     if line.startswith('--- ') and block:blocks.append(block);block=[]
     block.append(line)
    if block:blocks.append(block)
    adjusted=[]
    for block in blocks:
     current=[];chunks=[]
     for line in block:
      if line.startswith('@@ ') and current:chunks.append(current);current=[]
      current.append(line)
     if current:chunks.append(current)
     for chunk in chunks:
      if any(x.startswith('-        json!({"adoptions":all') for x in chunk):
       # This hunk includes only final output-key addition in prior generation.
       p=crate/'src/dp04.rs';s=p.read_text();key='explanations'if task=='C05'else'region_reports';s=s.replace('json!({"adoptions":all',f'json!({{"{key}":{key},"adoptions":all');p.write_text(s)
       # Keep any preceding insertion lines in this hunk (query loop C05).
       insert=[x[1:] for x in chunk if x.startswith('+') and not x.startswith('+++') and not x.startswith('+        json!')]
       if task=='C06' and insert:
        s=p.read_text();s=s.replace('        prior_nodes=ctx.last_node().into_iter().collect();',''.join(insert)+'        prior_nodes=ctx.last_node().into_iter().collect();');p.write_text(s)
       if task=='C05' and insert:
        # query loop lines immediately precede Ok(; exclude closing formatting line.
        s=p.read_text();query=''.join(x for x in insert if 'explanations' in x or 'for query' in x);s=s.replace('    Ok(\n',query+'    Ok(\n');p.write_text(s)
       continue
      adjusted.extend(chunk)
    pf=folder/'reapplied.patch';pf.write_text(''.join(adjusted));p=subprocess.run(['patch','--batch','--forward','-p1','-i',str(pf)],cwd=crate,capture_output=True,text=True);(folder/'patch.log').write_text(p.stdout+p.stderr)
    # The mode stamp is an intervening line in the new source. Preserve it while
    # mechanically applying the reviewed region-authorization insertion.
    rejects=list(crate.rglob('*.rej'))
    if rejects:
     for reject in rejects:
      if reject.name!='dp04.rs.rej':raise RuntimeError('Unexpected patch reject '+str(reject))
      s=(crate/'src/dp04.rs').read_text();needle='        let mut observed = observations::read(revision, ctx)?;';assert task=='C06' and needle in s
      s=s.replace(needle,'        let mut permitted_revision=revision.clone();permitted_revision["region_withdrawals"]=region_policy.regions();permitted_revision["policy_generation"]=json!(region_policy.epoch());\n        let mut observed = observations::read(&permitted_revision, ctx)?;\n        revision_gate.set_policy_epoch(region_policy.epoch());');(crate/'src/dp04.rs').write_text(s)
     save(folder/'manual-rebase.json',{'reason':'Actual parser-mode stamp intervenes in old hunk context','rejected_hunks':[p.read_text()for p in rejects],'new_feature_semantics':'same region-policy application; parser enablement retained'})
   names=sorted(set(str(p.relative_to(ORIG))for p in ORIG.rglob('*.rs')if'target'not in p.parts)|set(str(p.relative_to(crate))for p in crate.rglob('*.rs')if'target'not in p.parts));changes=[];diff=[]
   for name in names:
    a=(ORIG/name).read_text()if(ORIG/name).exists()else'';b=(crate/name).read_text()if(crate/name).exists()else''
    if a!=b:changes.append(name);diff.extend(difflib.unified_diff(a.splitlines(True),b.splitlines(True),fromfile='new-frozen/'+name,tofile=task+'/'+variant+'/'+name))
   (folder/'actual.patch').write_text(''.join(diff));save(folder/'manifest.json',{'task':task,'variant':variant,'changes':changes,'count':len(changes),'baseline_source':{str(p.relative_to(ORIG)):sha(p)for p in(ORIG/'src').rglob('*.rs')},'not_old_count_reused':True,'API_calls':0})
 print('Prepared8 independent patched crates; execution pending isolatedofflinewindow')
if __name__=='__main__':prepare()
