"""After all authorized trials, persist the exact corrected crate in active prototypes without builds."""
import hashlib,json,shutil
from pathlib import Path
REPO=next(p for p in Path(__file__).resolve().parents if(p/'benchmark/architecture/four_dp_poc_v2').is_dir());R=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(p):return json.loads(p.read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 expected={'basic-v1':120,'tactic-dp04-A4':12,'tactic-dp04-B4':12,'tactic-dp01-A2':3,'basic-dp05-reuse-union-corrected-v2':36,'diagnostic-dp05-numbered-lines':9}
 for name,n in expected.items():assert len((R/name/'runs.jsonl').read_text().splitlines())==n
 phase=R/'basic-dp05-reuse-union-corrected-v2';m=read(phase/'freeze.json');pin=m['effective_candidate_source'];src=Path(pin['snapshot']);target=REPO/'prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected';target.mkdir(exist_ok=True)
 for name,wanted in pin['sha256'].items():
  original=src/name;assert sha(original)==wanted;out=target/name;out.parent.mkdir(parents=True,exist_ok=True)
  if out.exists():assert sha(out)==wanted,'Do not overwrite divergent active corrected source '+name
  else:shutil.copy2(original,out)
 proof={'scope':'Exact durable source for effective DP05 ABC36 and its line-number diagnostic; no build/cache copied. Other effective DPs retain original BASIC source.','actual_phase':str(phase.resolve()),'freeze_sha256':sha(phase/'freeze.json'),'candidate_binary':m['binary'],'source_sha256':pin['sha256'],'only_baseline_source_change':pin['only_baseline_source_change'],'preserved_original_active_crate':str((REPO/'prototypes/candidates/four_dp_poc_v2').resolve()),'copy_code_sha256':sha(Path(__file__))};(target/'EFFECTIVE-SOURCE.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n')
 (target/'README.md').write_text('# DP05 공통 원문 재사용 교정의 실행 소스\n\n이 crate는 원래 BASIC 전체 소스에서 승인된 DP05 세 파일만 바꾼 실제 교정 source의 영구 복사본이다. DP01/02/04의 발표 수치는 원래 BASIC 동결본을 사용한다. 이 복사본을 새 전체120 결과로 해석하지 않는다.\n\n정확한 source/binary/freeze SHA와 원래 baseline 관계는 EFFECTIVE-SOURCE.json에 있다. 기존 BASIC 소스·원시 결과·보완 binary는 덮지 않았다. build/cache는 복사하지 않았다.\n\n`cargo build --offline --manifest-path prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/Cargo.toml`로 재빌드할 수 있다. 실제 측정 binary는 결과 freeze에 별도로 보존돼 있으며 새 빌드의 경로/메타데이터에 따라 binary SHA는 달라질 수 있다. API 재실행은 기존 결과를 덮지 않는 새 승인 phase가 필요하다. 전체 재현은 [FINAL-REPRODUCTION.md](../../../benchmark/architecture/four_dp_poc_v2/FINAL-REPRODUCTION.md)에 있다.\n')
 bench=REPO/'benchmark/architecture/four_dp_poc_v2';p=bench/'FINAL-REPRODUCTION.md';p.write_text('''# 최종 evidence 재현 진입점

발표용 비교는 `results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018/effective-main-selection.json`의 명시적120선택이다. 원래01/02/04 84회는 `basic-v1`, DP05 전체36회는 `basic-dp05-reuse-union-corrected-v2`이다. 원래 DP05 36회와 calibration26·보완27·중간교정11·번호진단9를 포함한 actual229은 모두 별도로 보존한다.

## 정확한 후보 소스와 binary

- 원래 BASIC: `prototypes/candidates/four_dp_poc_v2/`; exact measured source/binary는 `basic-v1/frozen/`.
- 교정 DP05: `prototypes/candidates/four_dp_poc_v2_dp05_reuse_corrected/`; exact source pin/binary는 `basic-dp05-reuse-union-corrected-v2/freeze.json`의 effective_candidate_source와 binary.
- DP04 A4/B4: BASIC과 동일한 frozen binary, 실제 input.variant는 A4/B4다. `tactic-dp04-A4`, `tactic-dp04-B4`의 plan/freeze 확인. --tactic만으로 이 경로가 활성화되지 않는다.
- DP01 A2: `tactic-dp01-A2/effective-candidate-source/` 및 frozen binary. exact BASIC+dp01.rs만의 승인변경, variant A+tactic A2.
- DP05 번호진단: 교정 DP05 binary 그대로, `diagnostic-dp05-numbered-lines/frozen/harness/`의 deep-copy 입력 formatter/worker. guard/oracle/caps는 유지되고 actual original-line 표기+공통설명만 추가.

## API 호출 없는 분석과 재생

아래는 저장된 원시 결과만 읽는다. API 키가 필요하지 않다.

```sh
python3 benchmark/architecture/four_dp_poc_v2/reports/finalize_exports.py
python3 benchmark/architecture/four_dp_poc_v2/reports/audit_campaign.py --final
```

재생은 campaign `demo.html`에 실제 requests/responses/PNG를 내장한 recorded replay다. 서버나 추가 API 호출이 필요 없고 actual display timing을 재측정하지 않는다. 원시50ms RSS 전수 표본은 별도 CSV/raw SHA 링크로 제공한다. 브라우저 로컬 파일 정책이 막으면 읽기 전용 HTTP 서버로 같은 파일을 제공할 수 있다.

최종 파생 계산기 정본은 `reports/`, 후속 측정 정본은 `post_basic/`다. 실제 실행 시 사용한 snapshot/SHA는 각 phase의 freeze와 final-analysis-source-manifest.json에 있다. results/analysis의 과거 준비본은 provenance이며 실행 정본이 아니다.

## 보존된 실제 실행 명령

`post_basic/tactics_control.py run dp04-A4`, `run dp04-B4`, `run dp01-A2`; `post_basic/corrected_dp05_union_control.py run`; `post_basic/line_number_diagnostic.py run --root <campaign>`를 사용했다. 준비 단계가 생성한 각 MODEL-CALLS-AUTHORIZED.json 및 plan의 고정 job만 실행했다. 이 명령은 완료 job을 건너뛰지만 새 모델 재평가는 새 결과 디렉터리·명시적 승인·전역 비용한도를 요구한다. 기존 실패행을 지우거나 같은 freeze로 조용히 재실행하면 안 된다.

calibration-v1의 당시 파생분석 provenance는 UNKNOWN으로 보존했다. 후속 CURRENT-REANALYSIS는 새이름/현재 계산기 hash/원시 hash를 가진 별도 재분석이며 당시 버전을 추정하지 않는다.
''')
 (bench/'README-EFFECTIVE-FINAL.md').write_text('# 최종 effective120 재현\n\n원래 동결 README는 당시 계약과 재현을 위해 보존한다. 현재 최종 두 freeze, 후보 source, 보완별 actual variant와 API 없는 분석·재생은 [FINAL-REPRODUCTION.md](FINAL-REPRODUCTION.md)를 따른다. 새 교정 crate README/EFFECTIVE-SOURCE.json에도 동일 source와 binary를 연결했다.\n')
 print(json.dumps({'durable_corrected_crate':str(target.resolve()),'source_files':len(pin['sha256']),'reproduction':str(p.resolve())}))
if __name__=='__main__':main()
