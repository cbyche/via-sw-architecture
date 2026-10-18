"""Read-only QA04 effective-comparison provenance/count summary; no Rust/models."""
import hashlib,json
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    qa=ROOT/'qa04';history=qa/'history-before-union-final-review';history.mkdir(exist_ok=True)
    for name in ['final-effective-cross-review.json','final-effective-cross-review.md']:
        old=qa/name;archived=history/name
        if old.is_file() and not archived.is_file():archived.write_bytes(old.read_bytes())
    original=read(qa/'summary.json');corrected=read(qa/'reuse-union-corrected-dp05-four-task-summary.json');options=[x for x in original['options'] if x['dp']!='05']+corrected['options']
    base=ROOT/'basic-v1/frozen/source/prototypes/candidates/four_dp_poc_v2';corrected_freeze=read(ROOT/'basic-dp05-reuse-union-corrected-v2/freeze.json');effective=corrected_freeze['effective_candidate_source'];new=Path(effective['snapshot']);changes=[]
    assert all(sha(new/relative)==h for relative,h in effective['sha256'].items())
    for p in sorted(base.rglob('*.rs')):
        relative=p.relative_to(base);q=new/relative
        if not q.is_file() or sha(p)!=sha(q):changes.append(str(relative))
    assert changes==['src/dp05.rs','src/dp05_handoff.rs','src/dp05_sources.rs']
    controls=qa/'controls-reuse-union-corrected-v2';inventory=read(controls/'baseline-source-sha256.json');mismatches=[relative for relative,h in inventory.items() if not (new/relative).is_file() or sha(new/relative)!=h];assert not mismatches
    refs=[qa/'summary.json',qa/'reuse-union-corrected-dp05-four-task-summary.json',qa/'controls/acceptance-summary.json',qa/'common-dp02-final-baseline-transition.json',qa/'baseline-v2/dp01-C05/summary.json',qa/'baseline-v2/dp01-C06/summary.json',qa/'dp04-rebased-calibration-v3/summary.json',qa/'dp04-rebased-calibration-v3/basic-v1-applicability.json',controls/'acceptance-summary.json',qa/'baseline-dp05-reuse-union-corrected-v2/dp05-C04/summary.json',qa/'baseline-dp05-reuse-union-corrected-v2/dp05-C06/summary.json',ROOT/'analysis/qa04-cross-review-payload-checks.json',ROOT/'analysis/final-review/dp05-common-retention-final-prefix-source/review.json']
    for o in options:
        assert len(o['tasks'])==4 and all(t['status']=='PASS' and t['count']==len(t['modules']) for t in o['tasks'])
        assert sum(t['count'] for t in o['tasks'])/4==o['mean_changed_operational_modules']
    out={'scope':'QA04 for effective main120: original DP01/02/04 plus corrected DP05 only. Four predeclared equal-weight tasks. No tactic/diagnostic QA04 inheritance and no architecture-wide/semantic/performance claim.','options':options,'baseline_relationship':{'original_freeze_sha256':sha(ROOT/'basic-v1/freeze.json'),'corrected_freeze_sha256':sha(ROOT/'basic-dp05-reuse-union-corrected-v2/freeze.json'),'actual_changed_rust_files':changes,'corrected_control_baseline_inventory_matches_actual_effective_candidate':not mismatches,'supplied_union_guard_sha256':sha(new/'src/dp05_sources.rs'),'corrected_effective_candidate_source':str(new),'inventory_sha256':sha(controls/'baseline-source-sha256.json'),'original_01_02_04_unchanged':True},'evidence':[{'path':str(p.resolve()),'sha256':sha(p)} for p in refs],'independent_review_scope':'DP04 independently implemented/executed functional scratch changes; corrected C01/C03 actual six patched Rust runs independently executed. DP01/05 initial ancillary-text issue independently read/rechecked then owner rebased; owner feature acceptance results are linked, not claimed as newly independently rerun here. Final corrected source3 was independently read/hash-checked.','historical_review':'analysis/qa04-cross-review.md is calibration-v1 provisional history, not final accepted-baseline evidence. Its original finding remains visible.','generator':{'path':str(Path(__file__).resolve()),'sha256':sha(Path(__file__))},'provider_calls_created':0,'candidate_runs_created':0}
    (qa/'final-effective-cross-review.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n')
    lines=['# 최종 effective120 QA04 근거·독립 검토','',out['scope'],'','|DP/안|C01|C03|새 기능1|새 기능2|네 과제 주평균|','|---|---:|---:|---:|---:|---:|']
    for o in options:
        ts=o['tasks'];lines.append('|'+o['dp']+'/'+o['variant']+'|'+'|'.join(str(t['count'])+((' '+t['id']) if i>1 else '') for i,t in enumerate(ts))+f"|{o['mean_changed_operational_modules']:.2f}|")
    lines+=['','## 실제 baseline 관계','',f"- 원래01/02/04: basic-v1 freeze `{out['baseline_relationship']['original_freeze_sha256']}`.",f"- 교정05: basic-dp05-reuse-union-corrected-v2 freeze `{out['baseline_relationship']['corrected_freeze_sha256']}`.",'- 교정 crate와 원래 crate의 Rust 차이는 dp05.rs/dp05_handoff.rs/dp05_sources.rs뿐이며, controls의 실제 고정 baseline inventory도 교정freeze의 effective_candidate_source와 일치했다. manifest.source는 그대로인 원래 baseline guard이며 실제 교정 코드는 별도 effective map으로 고정된다.','- DP04는 최종 relation 계약 위 독립 C05/C06 A/B scratch를 재base·실행했고, 기본freeze 동일성은 applicability JSON으로 보존했다.','- DP01은 guard-v2 scratch와 최종 운영모듈 동일성/전이 근거를 연결한다. DP02 common adapter/feature 전이 근거도 별도 보존한다.','- 교정05는 C01/C03 actual6 offline, C04/C06 actual6 offline가 별도 source/diff/원시로 보존된다. 초기 ps 차단은 후보 시작 전 인프라 실패로 별도 보존했으며 수용 PASS로 바꾸지 않았다.','','## 독립 검토 범위','',out['independent_review_scope'],'','개별안에 실제 필요한 변경 Rust 모듈만 세었고 새 test-only probe/harness는 제외했다. 초기 cal-v1 ancillary 문구 재주입 finding과 그 수정/검사 이력은 삭제하지 않는다. 새 기능만의 pair 평균이 네 과제 주평균을 대신하지 않는다. 보완 A2/A4/B4 및 번호 진단에는 이 QA04를 자동 상속하지 않는다.','','## 상세 근거','']
    for p in refs:lines.append(f'- [{p.relative_to(ROOT)}]({Path(__import__("os").path.relpath(p,qa))})')
    lines+=['','구체적 source SHA·task별모듈·수용 링크는 final-effective-cross-review.json. 기존수치/원시/모델호출 변경0.']
    (qa/'final-effective-cross-review.md').write_text('\n'.join(lines)+'\n');print(json.dumps({'options':len(options),'counts':[o['mean_changed_operational_modules']for o in options],'corrected_source_inventory_match':not mismatches,'provider_calls':0}))
if __name__=='__main__':main()
