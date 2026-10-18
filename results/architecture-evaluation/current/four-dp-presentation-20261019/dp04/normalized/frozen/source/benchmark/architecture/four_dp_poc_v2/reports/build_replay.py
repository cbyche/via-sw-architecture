"""Build a standalone recorded-evidence viewer. No models, candidates, or scorers run."""
import argparse,base64,csv,datetime,hashlib,json,os,shutil,struct
from pathlib import Path
from urllib.parse import quote

HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if (p/'benchmark/architecture/four_dp_poc_v2').is_dir())
DEFAULT=REPO/'results/architecture-evaluation/current/four-dp-qwen37-review-v2-20261018'
def read(path,default=None):
    return json.loads(path.read_text()) if path.exists() else default
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def safe_json(value):return json.dumps(value,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c').replace('\u2028','\\u2028').replace('\u2029','\\u2029')
def has_allowed_record(path,allowed):
    with path.open() as f:
        for line in f:
            try:r=json.loads(line)
            except json.JSONDecodeError:continue
            if r.get('phase') in allowed and not r.get('offline_responder'):return True
    return False
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('campaign',type=Path,nargs='?',default=DEFAULT);parser.add_argument('--output',type=Path);parser.add_argument('--limit',type=int);parser.add_argument('--limit-per-phase',type=int);parser.add_argument('--phase',action='append');parser.add_argument('--include-calibration',action='store_true');a=parser.parse_args()
    root=a.campaign.resolve();output=(a.output or root/'demo.html').resolve();repo=next(p for p in root.parents if (p/'benchmark/architecture').exists());warnings=[];runs=[];summaries=[];assets={};source_files={}
    # A final review supersedes its partial progress snapshot in presentation only;
    # preserve the historical file, avoid displaying the same judgment twice.
    manual_reviews=[{'source':str(p),'sha256':digest(p),'value':read(p)} for p in sorted((root/'analysis').glob('*content-review.json')) if not ('-partial-' in p.name and p.with_name(p.name.replace('-partial-','-')).is_file())]
    href=lambda p:quote(os.path.relpath(p,output.parent),safe='/')
    allowed={'BASIC','BASIC_CORRECTED','TACTIC','DIAGNOSTIC'}
    if a.include_calibration:allowed.update({'CALIBRATION','CALIBRATION_V2','CALIBRATION_V3'})
    paths=sorted(p for p in root.rglob('runs.jsonl') if not any(x in p.relative_to(root).parts for x in ['frozen','raw','qa04','offline-dp04-contract-checks']))
    for path in paths:
        phase=path.parent
        if not (phase/'freeze.json').exists():continue
        if a.phase and str(phase.relative_to(root)) not in a.phase:continue
        if not has_allowed_record(path,allowed):continue
        metrics={m['run_id']:m for m in read(phase/'episode-metrics.json',[])}
        summary=read(phase/'summary.json')
        if summary:summaries.append({'phase_directory':str(phase.relative_to(root)),'source':str(phase/'summary.json'),'value':summary})
        source_files[str(path.relative_to(root))]=digest(path);phase_kept=0
        for index,line in enumerate(path.read_text().splitlines()):
            if not line.strip():continue
            try:r=json.loads(line)
            except json.JSONDecodeError:warnings.append({'path':str(path),'line':index+1,'reason':'Partial/invalid JSONL line omitted, not counted as a run.'});continue
            if r.get('phase') not in allowed or r.get('offline_responder'):continue
            if a.limit is not None and len(runs)>=a.limit:continue
            if a.limit_per_phase is not None and phase_kept>=a.limit_per_phase:continue
            input_path=Path(r.get('input',''));actual_input=read(input_path) if input_path.is_file() else None
            if actual_input is None:warnings.append({'run_id':r.get('run_id'),'reason':'Actual input file unavailable; raw trace remains.'})
            providers=[];requests=[]
            for call in r.get('calls',[]):
                p=Path(call.get('response_file',''))
                providers.append({'call_uuid':call.get('call_uuid'),'source':str(p),'sha256':digest(p) if p.is_file() else None,'actual_provider_response':read(p) if p.is_file() else None})
                request_path=Path(call.get('request_file',''));body=read(request_path) if request_path.is_file() else None;image_refs=[]
                if body is not None:
                    for mi,message in enumerate(body.get('messages',[])):
                        if not isinstance(message.get('content'),list):continue
                        for ci,part in enumerate(message['content']):
                            if part.get('type')!='image_url':continue
                            url=part.get('image_url',{}).get('url','')
                            if not url.startswith('data:'):continue
                            header,encoded=url.split(',',1);data=base64.b64decode(encoded,validate=True);sha=hashlib.sha256(data).hexdigest();width,height=struct.unpack('>II',data[16:24]) if data.startswith(b'\x89PNG\r\n\x1a\n') else (None,None)
                            if sha not in assets:assets[sha]={'data_uri':url,'sha256':sha,'width':width,'height':height,'source':str(request_path)+f'#messages/{mi}/content/{ci}/image_url'}
                            image_refs.append({'message_index':mi,'content_index':ci,'asset_sha256':sha,'width':width,'height':height})
                            part['image_url']['url']='EMBEDDED_ASSET_SHA256:'+sha
                else:warnings.append({'run_id':r['run_id'],'call_uuid':call.get('call_uuid'),'reason':'Actual model request file unavailable; do not infer its prompt from fixtures.'})
                requests.append({'call_uuid':call.get('call_uuid'),'source':str(request_path),'href':href(request_path),'original_request_sha256':digest(request_path) if request_path.is_file() else None,'actual_request_with_image_references':body,'image_references':image_refs,'presentation_note':'All actual system/user text and request settings are unchanged. Only image_url data bytes are deduplicated into embedded SHA256 assets. Original exact JSON hash/path retained; no server/fetch needed for text or images.'})
            samples=r.get('memory',[]);peak=max(samples,key=lambda x:x['concurrent_sum_bytes']) if samples else None
            memory_summary={'sample_count':len(samples),'simultaneous_python_rust_sample_count':sum(len(x.get('rss_bytes_by_pid',{}))>=2 for x in samples),'peak_same_sample':{k:v for k,v in (peak or {}).items() if k!='logical_buffers'},'max_sample_gap_ms':max(((b['at_ns']-a['at_ns'])/1e6 for a,b in zip(samples,samples[1:])),default=None),'scope':'Actual same-sample Python/Rust RSS estimate, no shared-page dedup; browser/UI/cloud weights excluded. This export does not measure browser memory.'}
            csv_path=root/'analysis/memory-samples'/(r['run_id']+'.csv');csv_path.parent.mkdir(parents=True,exist_ok=True)
            with csv_path.open('w',newline='') as f:
                writer=csv.DictWriter(f,fieldnames=['at_ns','sample_end_ns','stage','concurrent_sum_bytes','rss_bytes_by_role','rss_bytes_by_pid','all_registered_observed']);writer.writeheader()
                for sample in samples:writer.writerow({k:json.dumps(sample.get(k),ensure_ascii=False) if isinstance(sample.get(k),(dict,list)) else sample.get(k) for k in writer.fieldnames})
            live={};category_peak={};logical_peak=0
            for event in r.get('logical_buffer_events',[]):
                if event.get('action')=='RELEASE':live.pop(event['name'],None)
                elif event.get('bytes') is not None:live[event['name']]={'category':event.get('category','unknown'),'bytes':event['bytes'],'owner':event.get('owner')}
                totals={}
                for value in live.values():totals[value['category']]=totals.get(value['category'],0)+value['bytes']
                for category,n in totals.items():category_peak[category]=max(category_peak.get(category,0),n)
                logical_peak=max(logical_peak,sum(totals.values()))
            logical_summary={'actual_lifecycle_event_count':len(r.get('logical_buffer_events',[])),'max_concurrent_serialized_logical_bytes':logical_peak,'category_peaks_not_additive':category_peak,'last_live_records':live,'scope':'Logical serialized estimates include harness retained events/call evidence. Category peaks may occur at different times; do not add to RSS or equate with product live heap.'}
            projection={k:v for k,v in r.items() if k not in ['memory','memory_baseline','logical_buffer_events']};projection['events']=[e for e in r.get('events',[]) if e.get('kind')!='LOGICAL_BUFFER']
            trace=input_path.with_name('trace.json')
            run={'phase_directory':str(phase.relative_to(root)),'record':projection,'actual_input':actual_input,'metrics':metrics.get(r['run_id']),'model_requests':requests,'provider_responses':providers,'input_sha256':digest(input_path) if input_path.is_file() else None,'frames':[],'memory_summary':memory_summary,'logical_summary':logical_summary,'raw_links':{'trace':{'path':str(trace),'href':href(trace),'sha256':digest(trace) if trace.is_file() else None},'memory_csv':{'path':str(csv_path),'href':href(csv_path),'sha256':digest(csv_path)}},'projection_note':{'original_events':len(r.get('events',[])),'embedded_events':len(projection['events']),'omitted_event_kind':'LOGICAL_BUFFER','not_embedded_fields':['memory','memory_baseline','logical_buffer_events'],'reason':'All input, actual system/user request text, provider responses and substantive events retained; full50ms samples and logical lifecycle remain in actual raw trace, physical samples also in CSV.'}}
            run['author_manual_reviews']=[]
            run['phase_stop']=read(phase/'PHASE-STOPPED.json')
            for review in manual_reviews:
                value=review['value'];details=[d for d in value.get('details',value.get('rows',[])) if d.get('run_id')==r['run_id']]
                if details:run['author_manual_reviews'].append({'source':review['source'],'sha256':review['sha256'],'scope':value.get('scope'),'reviewer':value.get('reviewer'),'reviewer_role':value.get('reviewer_role','AUTHOR'),'method':value.get('method'),'phase_status':value.get('phase_status'),'details':details,'label':value.get('review_label','POST_HOC_AUTHOR_CONTENT_REVIEW_NOT_QA02_OR_AUTOMATIC_SCORE')})
            for revision in (actual_input or {}).get('revisions',[]):
                for frame in revision.get('frames',[]):
                    for field in ['image_path','overlay_image_path']:
                        if not frame.get(field):continue
                        original=Path(frame[field]);image=original
                        try:
                            candidate=phase/'frozen/source'/original.relative_to(repo)
                            if candidate.is_file():image=candidate
                        except ValueError:pass
                        if not image.is_file():warnings.append({'run_id':r['run_id'],'reason':'Frame pixels unavailable','path':str(original)});continue
                        data=image.read_bytes();sha=hashlib.sha256(data).hexdigest()
                        if sha not in assets:
                            width,height=struct.unpack('>II',data[16:24]) if data.startswith(b'\x89PNG\r\n\x1a\n') else (None,None)
                            assets[sha]={'data_uri':'data:image/png;base64,'+base64.b64encode(data).decode(),'sha256':sha,'width':width,'height':height,'source':str(image)}
                        expected=frame.get('overlay_sha256' if field.startswith('overlay') else 'sha256')
                        if expected and expected!=sha:warnings.append({'run_id':r['run_id'],'reason':'Frame expected/source hash mismatch','path':str(original)})
                        run['frames'].append({'revision':revision.get('revision'),'frame_id':frame.get('frame_id'),'kind':'RAW EVENT OVERLAY' if field.startswith('overlay') else 'ORIGINAL PIXELS','asset_sha256':sha,'expected_sha256':expected})
            runs.append(run);phase_kept+=1
    ids=[x['record']['run_id'] for x in runs]
    if len(set(ids))!=len(ids):warnings.append({'reason':'Duplicate run IDs retained for visibility, not silently merged.'})
    parallel_content=[{'source':str(p),'sha256':digest(p),'value':read(p)} for p in sorted((root/'analysis').glob('*bounded*equivalence*.json'))]
    preview_limit=a.limit if a.limit is not None else ('phase별 '+str(a.limit_per_phase) if a.limit_per_phase is not None else None)
    effective_path=root/'effective-main-selection.json';effective=read(effective_path);effective_summary=read(root/'effective-main-comparison/summary.json')
    if effective and (effective.get('selected_trials')!=120 or len(effective.get('selection',[]))!=120 or len({x['run_id']for x in effective.get('selection',[])})!=120):warnings.append({'reason':'Effective main selection does not contain exactly120 unique explicitly selected runs; no successful completeness claim.'})
    source_pins={str(HERE/name):digest(HERE/name) for name in ['build_replay.py','replay_template.html','check_replay_static.js','audit_campaign.py']}
    data={'schema_version':1,'title':'VIA four-DP recorded evidence replay','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'campaign':str(root),'scope':'RECORDED_REPLAY / DISPLAY_PROXY callbacks. QA01 is actual substantive callback latency proxy, not fully automatic first semantic-correctness validation. GOAL_COMPLETE is the frozen host contract, not full content success. No live model, physical UI display, ASR or audio endpoint. No source/output repair. QA tables are embedded existing derived artifacts, not recomputed by this builder.','included_phases':sorted(allowed),'limited_preview':preview_limit,'runs':runs,'summaries':summaries,'effective_selection':effective,'effective_summary':effective_summary,'effective_selection_provenance':{'path':str(effective_path),'sha256':digest(effective_path)} if effective_path.is_file() else None,'report_tool_sources':source_pins,'assets':assets,'qa04':read(root/'qa04/summary.json'),'parallel_content_diagnostics':parallel_content,'manual_review_sources':[{'source':x['source'],'sha256':x['sha256'],'scope':x['value'].get('scope'),'reviewer':x['value'].get('reviewer'),'by_variant':x['value'].get('by_variant')} for x in manual_reviews],'budget':read(root/'budget.json'),'source_runs_sha256':source_files,'warnings':warnings}
    template=(HERE/'replay_template.html').read_text();assert template.count('__EMBEDDED_EVIDENCE__')==1
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(template.replace('__EMBEDDED_EVIDENCE__',safe_json(data)))
    snapshot=output.with_suffix('.build-sources');snapshot.mkdir(exist_ok=True)
    for name in ['build_replay.py','replay_template.html','check_replay_static.js','audit_campaign.py']:shutil.copy2(HERE/name,snapshot/name)
    metadata={'output':str(output),'sha256':digest(output),'bytes':output.stat().st_size,'runs':len(runs),'assets':len(assets),'provider_responses':sum(len(x['provider_responses']) for x in runs),'warnings':warnings,'limited_preview':preview_limit,'source_runs_sha256':source_files,'canonical_report_tool_sources':source_pins,'report_tool_snapshot':str(snapshot),'effective_selection':data['effective_selection_provenance'],'provider_calls_created':0,'candidate_runs_created':0}
    output.with_suffix('.build.json').write_text(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in metadata.items() if k not in ['source_runs_sha256','warnings']},ensure_ascii=False))
if __name__=='__main__':main()
