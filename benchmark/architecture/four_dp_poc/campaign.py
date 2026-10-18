"""Serial paired run orchestration; fresh Python bridge+Rust per trial avoids RSS carryover."""
import argparse,json,subprocess,sys
from pathlib import Path
BASE=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--results',required=True);p.add_argument('--phase',default='BASIC');p.add_argument('--dp');p.add_argument('--variant');p.add_argument('--tactic');a=p.parse_args()
completed=0
for dp in ([a.dp] if a.dp else ['01','02','04','05']):
    episodes=json.loads((BASE/'fixtures'/f'dp{dp}.json').read_text());variants=[a.variant] if a.variant else list('ABC' if dp in ('01','05') else 'AB')
    for rep in range(1,4):
        order=variants[(rep-1)%len(variants):]+variants[:(rep-1)%len(variants)]
        for episode in episodes:
            for variant in order:
                cmd=[sys.executable,str(BASE/'run.py'),'--skip-build','--dp',dp,'--variant',variant,'--episode',episode['id'],'--reps','1','--single-repetition',str(rep),'--phase',a.phase,'--results',a.results]
                if a.tactic:cmd+=['--tactic',a.tactic]
                subprocess.run(cmd,check=True);completed+=1
                if completed%10==0:print('PROGRESS '+str(completed)+' fresh-process episode runs completed',flush=True)
