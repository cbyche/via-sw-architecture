import argparse,json
from pathlib import Path
from bridge import Client
p=argparse.ArgumentParser();p.add_argument('--results',required=True);a=p.parse_args()
c=Client(a.results);value,record=c.call({'instruction':'Return {"ok":true,"model_access":"verified"}.','input':{'purpose':'minimal snapshot access check'},'max_tokens':80},{'phase':'ACCESS_SMOKE','dp':None,'variant':None})
print(json.dumps({'value':value,'model':record['model'],'usage':record.get('raw_usage'),'cost_usd':record.get('cost_usd'),'status':record['status']}))
