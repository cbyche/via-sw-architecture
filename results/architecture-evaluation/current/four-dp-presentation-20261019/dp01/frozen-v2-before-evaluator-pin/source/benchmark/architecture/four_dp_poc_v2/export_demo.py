"""Export self-contained offline evidence replay, including actual input/result/timeline."""
import argparse,base64,json
from pathlib import Path
BASE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('results',type=Path);a=p.parse_args()
summary=json.loads((a.results/'summary.json').read_text());records=[json.loads(l) for l in (a.results/'runs.jsonl').read_text().splitlines() if l]
traces={};images={};index=[]
conditions=json.loads((a.results/'semantic-conditions.json').read_text()).get('details',[]) if (a.results/'semantic-conditions.json').exists() else []
for r in records:
    input_value=json.loads(Path(r['input']).read_text());trace={k:r[k] for k in ['run_id','phase','dp','variant','episode','repetition','tactic','start_ns','events','calls']};trace['input_value']=input_value;trace['content_diagnostics']=[x for x in conditions if x['run_id']==r['run_id']]
    for c in trace.get('calls',[]):
        if c.get('response_file') and Path(c['response_file']).exists():c['response_value']=json.loads(Path(c['response_file']).read_text())
    traces[r['run_id']]=trace
    index.append({k:r[k] for k in ['run_id','phase','dp','variant','episode','repetition','tactic']})
    for revision in input_value.get('revisions',[]):
        for frame in revision.get('frames',[]):
            path=frame['image_path']
            if path not in images:images[path]='data:image/png;base64,'+base64.b64encode(Path(path).read_bytes()).decode()
evidence={'summary':summary,'runs':index,'traces':traces,'images':images}
html=(BASE/'demo.html').read_text();a_script=html.index('<script>');b_script=html.index('</script>',a_script)
script=html[a_script+8:b_script]
script=script.replace("data=await(await fetch('/api/data')).json();","data=window.__EVIDENCE__;")
script=script.replace("let r=await(await fetch('/api/trace?id='+encodeURIComponent(id))).json();","let r=data.traces[id];")
script=script.replace("img.src='/api/asset?path='+encodeURIComponent(f.image_path);","img.src=data.images[f.image_path];")
encoded=json.dumps(evidence,ensure_ascii=False,separators=(',',':')).replace('<','\\u003c')
html=html[:a_script]+'<script>window.__EVIDENCE__='+encoded+';</script><script>'+script+'</script>'+html[b_script+9:]
(a.results/'evidence-browser.html').write_text(html)
print(json.dumps({'path':str(a.results/'evidence-browser.html'),'bytes':len(html.encode()),'run_count':len(records),'embedded_images':len(images)}))
