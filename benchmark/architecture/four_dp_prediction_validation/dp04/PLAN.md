# DP3 (repository DP04) conditional prediction validation

2026-10-19. `results/architecture-evaluation/current/four-dp-prediction-validation-20261019/dp04/plan.json` preregisters the following before any new cloud outcome. Existing pixel and normalized campaigns remain separate. This source generation uses one shared normalized output wire for every vision owner: integer0..1000 in original-frame axes, converted to original pixels with ties-even rounding, no clamping or oracle geometry.

## Conditions and hypotheses

Six freshly authored PNG-only episodes cover repeated dense labels with edge-near points, small text, a free scatter region, a language-defined pair of inner graphs, a historical target before navigation, and partial correction with stale proposal replay. There are7 input revisions,10 referent-set judgments and6 final goals per configuration/repetition. Three repetitions give QA01 scheduled21, QA02 scheduled30 and QA03 scheduled18 per configuration. The same finite fixtures, raw event overlays, rights, recovery bounds and total two cloud slots apply to all alternatives. Every repetition rotates alternative order by case; even repetition reverses it. SVG authoring sources and oracle IDs are evaluator-only. Real native object data does not exist for these PNG sources; unsupported native availability is explicit.

| QA | Preregistered direction and condition | Countercondition / limitation |
| --- | --- | --- |
| QA01 | A can benefit from fewer dependent stages when its first joint proposal passes. B4 can overlap independent screen and speech work. | A repairs, complete B candidates and typed binding can reverse the observation. Missing endpoints remain missing. Local cold helper startup is a deployment cost, not inherent cloud B latency. |
| QA02 | B's dedicated visual observations may help dense/small text if accurate and sufficient. No overall strict ranking. | OCR glyph boxes are not item extents or sample centers. Candidate omission, free groups and correction semantics can harm B. A4 tests whether representation alone can recover an advantage. |
| QA03 | A avoids mandatory helper/producer/binder dependencies; B4 may overlap part of them. | Joint rejudgment, actual typed binding and per-trial parser reuse may offset this. Compare fixed goals and conditional means separately. |
| QA04 | No strict ranking: source-local changes and schema propagation conflict. | Fresh independent C01/C03/C05/C06 Rust patches and actual acceptance determine counts. Previous2.5 is not reused. Third-party/Python helper changes remain separately disclosed. |
| QA05 | A avoids mandatory candidate production/output and binding calls. | Raw rejudgment can be costly; B code binding may omit calls. All unused/repair/producer calls count. Local compute is not monetized. Original cloud parser price remains NOT_MEASURED. |
| QA06 | A4/B/B4's local full helper likely dominates A's RSS; B4 adds simultaneous buffers. | This local CPU diagnostic is a changed deployment. Cloud helper weights would not belong to VIA local RAM. Sampled RSS and logical bytes differ; shared-page deduplication is unverified. |

## Configurations

- A: original PNGs + full speech/events → joint target-extent and binding owner → common gate/display.
- B: actual dedicated detector/OCR/caption observations + preserved original pixels → Screen Target Producer; separate Speech Reference Producer → typed/model binding. Only the screen owner expands insufficient candidate evidence.
- A4: exactly the same real helper and trigger → joint owner's auxiliary input. A4 does not invoke Qwen screen production first. The joint owner retains final extent/binding authority.
- B4: same B with independent speech and screen paths overlapping under the same two cloud slots. No producer/binding authority change.
- BQ: explicitly Qwen-only separated-producer diagnostic; no dedicated parser claim. A matched dense/free-region subset with three repetitions may run separately if budget permits. It is not pooled into full-parser B.

B's original design §5.3 proposed a separate cloud parser dependency. This experiment attempts an actual **task-local CPU deployment** of the complete public detector/OCR/caption stack. Its measured initialization, inference and local weights belong to this diagnostic deployment. Original cloud-service latency, monetary charges and deployment memory are NOT_MEASURED. If the full stack cannot run, preserve concrete attempts and keep full-parser B unmeasured; a Qwen proxy or text-only OCR does not fill that cell.

Each trial starts a fresh helper on its first actual parser request, inside the stimulus-to-display interval. Startup/load time and per-frame inference are recorded separately. The helper stays resident until that trial ends. A4/B/B4 use identical cold-start policy, two CPU inference threads and cache keyed by actual frame ID, source revision and content hash. Repeated same-source expansion/correction reuses the actual parse; cache survives no trial. Every helper PID enters the same50ms Python+Rust RSS sample registry. Failures and incomplete samples are preserved.

## Dependency and reproduction

Official source revision: `microsoft/OmniParser` commit `354021201345a96178360b28733573e27269f2de`. Runtime path `/private/tmp/via-dp04-omniparser-venv` uses bundled Python3.12 and task-local packages. Data/cache path `/private/tmp/via-dp04-omniparser`. The public v2 legacy detector is40,623,819 bytes and Florence caption checkpoint1,083,916,964 bytes. Actual checkpoint SHA256, package lock, upstream source and adapter diff are recorded in the result parser inventory before freeze. No checkpoint, cache or compiled helper belongs in Git.

The official `util/utils.py` eagerly imports Azure/Paddle and initializes Paddle even when `use_paddleocr=False`. This task adapter removes unused Azure import and unused Paddle import/initialization, explicitly selects EasyOCR CPU and a task-local OCR model directory. It preserves detector, EasyOCR and Florence caption algorithms. Any further compatibility change is recorded in the adapter diff and reviewed before formal freeze. Official entrypoint chooses CUDA if available, CPU otherwise; this host uses CPU.

Commands and attempts live in the result logs and `parser/reproduce.py`. The actual service is `parser/service.py`; it reads original PNGs only. `parser/vision_ocr.swift` is a separately labeled genuine text-only Apple Vision diagnostic. It has no icon detector/caption functionality and is never relabeled full OmniParser.

## Execution boundaries

`campaign.py prepare` snapshots source, fixture/oracle, model settings, exact schedule, parser inventory and executable hash. Formal cloud calls require root's `API-WINDOW-GRANTED.json` naming the exact freeze hash. API budget ceiling is$1.40 including reservations for unknown usage. Expected72 trajectories and approximately170–230 cloud calls cost roughly$0.35–0.60 at fixed comparison rates input$0.40/output$1.60 per million actual tokens; reservation accounting can stop earlier. Unknown cost never becomes zero.

Report case×configuration×repeat raw values and all failures. Observed QA01/03 means retain scheduled denominators; common-success comparisons are supplemental. QA04 runs four actual independent change exercises on this new source. QA05 distinguishes cloud provider usage, real local helper computation and unmeasured cloud helper price. QA06 includes actual local helper state. No old measurements are pooled and no product-wide or on-device Omni performance claim follows.
