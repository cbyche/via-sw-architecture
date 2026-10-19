"""Same-sample process RSS and separate logical buffer lifecycle diagnostics."""
import os,subprocess,threading,time

class LogicalBuffers:
    def __init__(self):self.lock=threading.Lock();self.live={};self.events=[]
    def set(self,name,size,category='serialized',owner='python_runner_bridge'):
        with self.lock:
            self.live[name]={'bytes':int(size),'category':category,'owner':owner}
            self.events.append({'at_ns':time.monotonic_ns(),'name':name,'bytes':int(size),'action':'ALLOCATE_OR_REPLACE','category':category,'owner':owner,'not_rss':True})
    def release(self,name):
        with self.lock:
            old=self.live.pop(name,None);self.events.append({'at_ns':time.monotonic_ns(),'name':name,'action':'RELEASE','prior':old,'not_rss':True})
    def snapshot(self):
        with self.lock:return {k:dict(v) for k,v in self.live.items()}

def sample_processes(roles,stage,buffers):
    roles=dict(roles)
    start=time.monotonic_ns()
    query=subprocess.run(['ps','-o','pid=,ppid=,rss=','-p',','.join(str(p) for p in roles)],capture_output=True,text=True)
    rows=[]
    for line in query.stdout.splitlines():
        cells=line.split()
        if len(cells)!=3:continue
        pid,ppid,rss=map(int,cells)
        rows.append({'pid':pid,'ppid':ppid,'role':roles.get(pid,'UNREGISTERED'),'rss_bytes':rss*1024})
    return {'at_ns':start,'sample_end_ns':time.monotonic_ns(),'stage':stage,'processes':rows,'registered_processes':[{'pid':p,'role':r} for p,r in roles.items()],
            'rss_bytes_by_pid':{str(r['pid']):r['rss_bytes'] for r in rows},'rss_bytes_by_role':{r['role']:r['rss_bytes'] for r in rows},
            'concurrent_sum_bytes':sum(r['rss_bytes'] for r in rows),'all_registered_observed':len(rows)==len(roles),'logical_buffers':buffers.snapshot(),
            'method':'macOS ps RSS; decimal MB; one ps invocation; sample start/end bracket recorded; shared-page deduplication unverified',
            'scope':'Fresh Python runner/bridge + Rust candidate + actual registered local detector/OCR/caption helper. Browser/replay controller, cloud weights, outer orchestrator excluded. Logical bytes are not RSS.'}
