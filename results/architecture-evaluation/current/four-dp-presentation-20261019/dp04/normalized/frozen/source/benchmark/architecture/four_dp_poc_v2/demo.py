"""Local read-only replay UI. DISPLAY_PROXY is not browser physical timing."""
import argparse,json,mimetypes
from pathlib import Path
from http.server import ThreadingHTTPServer,BaseHTTPRequestHandler
from urllib.parse import urlparse,parse_qs
BASE=Path(__file__).resolve().parent;REPO=BASE.parents[2]
p=argparse.ArgumentParser();p.add_argument('--results',type=Path,default=REPO/'results/architecture-evaluation/current/four-dp-qwen37-20261018');p.add_argument('--port',type=int,default=8768);a=p.parse_args()
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        url=urlparse(self.path);q=parse_qs(url.query)
        try:
            if url.path=='/':content=(BASE/'demo.html').read_bytes();mime='text/html; charset=utf-8'
            elif url.path=='/api/data':
                runs=[json.loads(l) for l in (a.results/'runs.jsonl').read_text().splitlines() if l] if (a.results/'runs.jsonl').exists() else []
                summary=json.loads((a.results/'summary.json').read_text()) if (a.results/'summary.json').exists() else {}
                content=json.dumps({'summary':summary,'runs':[{k:r[k] for k in ['run_id','phase','dp','variant','episode','repetition','tactic']} for r in runs]}).encode();mime='application/json'
            elif url.path=='/api/trace':
                identifier=q['id'][0]
                if '/' in identifier or '..' in identifier:raise ValueError('Bad run id')
                data=next(json.loads(line) for line in (a.results/'runs.jsonl').read_text().splitlines() if json.loads(line)['run_id']==identifier);data['input_value']=json.loads(Path(data['input']).read_text())
                condition_file=a.results/'semantic-conditions.json'
                data['content_diagnostics']=[x for x in json.loads(condition_file.read_text()).get('details',[]) if x['run_id']==identifier] if condition_file.exists() else []
                for c in data.get('calls',[]):
                    if c.get('response_file') and Path(c['response_file']).exists():c['response_value']=json.loads(Path(c['response_file']).read_text())
                content=json.dumps(data).encode();mime='application/json'
            elif url.path=='/api/asset':
                path=Path(q['path'][0]).resolve()
                if not path.is_relative_to(BASE) or path.suffix.lower() not in ['.png','.jpg','.jpeg']:raise ValueError('Unsupported asset')
                content=path.read_bytes();mime=mimetypes.guess_type(path)[0]
            else:self.send_error(404);return
            self.send_response(200);self.send_header('Content-Type',mime);self.send_header('Content-Length',str(len(content)));self.end_headers();self.wfile.write(content)
        except Exception as e:self.send_error(404,str(e))
    def log_message(self,*args):pass
print(f'VIA replay demo http://127.0.0.1:{a.port}',flush=True);ThreadingHTTPServer(('127.0.0.1',a.port),Handler).serve_forever()
