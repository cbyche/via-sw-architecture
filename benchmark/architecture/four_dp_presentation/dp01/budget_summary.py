"""Read-only ledger snapshot -> small derived index; never changes budget entries."""
import argparse,datetime,fcntl,hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=next(p for p in HERE.parents if(p/'results/architecture-evaluation/current').is_dir())
ROOT=REPO/'results/architecture-evaluation/current/four-dp-presentation-20261019'
def summary(path):
 with path.open()as f:
  fcntl.flock(f,fcntl.LOCK_SH);raw=f.read();data=json.loads(raw)
 groups={}
 for item in data['calls'].values():
  group=groups.setdefault(item['state'],{'calls':0,'amount_usd':0});group['calls']+=1;group['amount_usd']+=item['amount_usd']
 return {'ledger':str(path.relative_to(REPO)),'ledger_sha256':hashlib.sha256(raw.encode()).hexdigest(),'limit_usd':data['limit_usd'],'calls':len(data['calls']),'by_state':groups,'known_usage_calls':groups.get('USAGE_RECORDED',{}).get('calls',0),'known_usage_usd':groups.get('USAGE_RECORDED',{}).get('amount_usd',0),'reserved_calls':groups.get('RESERVED',{}).get('calls',0),'reserved_usd':groups.get('RESERVED',{}).get('amount_usd',0),'unknown_calls':groups.get('UNKNOWN',{}).get('calls',0),'unknown_retained_reservation_usd':groups.get('UNKNOWN',{}).get('amount_usd',0),'total_usage_or_retained_reservation_usd':sum(g['amount_usd']for g in groups.values()),'cost_semantics':'Provider usage x frozen standard prices, not invoice; unknowns retain reservations'}
def main():
 p=argparse.ArgumentParser();p.add_argument('--status',choices=['INTERIM','FINAL'],default='INTERIM');a=p.parse_args();out={'status':a.status,'as_of_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'global':summary(ROOT/'budget.json'),'dp01_dp05':summary(ROOT/'dp01-dp05-budget.json'),'native_replay_new_API_calls':0};(ROOT/'budget-summary.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n');print(json.dumps(out,ensure_ascii=False))
if __name__=='__main__':main()
