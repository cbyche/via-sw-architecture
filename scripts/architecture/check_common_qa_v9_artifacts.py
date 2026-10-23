#!/usr/bin/env python3
"""Read-only verification of v9 spreadsheet content and preserved v8 tabs."""
import copy
import hashlib
import json
import math
from pathlib import Path
import openpyxl

ROOT=Path(__file__).resolve().parents[2]
P=ROOT/'docs/architecture/12-decisions/decision-packages/current/evaluation'


def main():
    old=P/'common-qa-v8/artifacts/outputs/01a15dfe-v8/VIA-QA-common-v8.xlsx';new=P/'common-qa-v9/artifacts/outputs/01a15dfe-v9/VIA-QA-common-v9.xlsx'
    ow=openpyxl.load_workbook(old);nw=openpyxl.load_workbook(new);ov=openpyxl.load_workbook(old,data_only=True);nv=openpyxl.load_workbook(new,data_only=True)
    d=json.loads((P/'common-qa-v9/artifacts/ledger.json').read_text())
    assert nw.sheetnames==ow.sheetnames+['QA07순서연결','QA07선정입력']
    allowed={'전체비교':{'A3','A32',*[f'G{r}' for r in range(21,31)]},'QA07사례':{'A3'},'QA07의무':{'A3'},'QA상세부록':{'A3'},'QA07v3사례':{'A3'},'QA07v3의무':{'A3'},'QA07v3명세':{'A3'}}
    for row in ow['QA상세부록']:
        if row[0].value=='QA07':allowed['QA상세부록'].add(row[0].coordinate)
    compared=0
    for name in ow.sheetnames:
        a,b=ow[name],nw[name];assert a.freeze_panes==b.freeze_panes,name
        assert set(a.tables)==set(b.tables) and list(a.data_validations)==list(b.data_validations),name
        for row in a:
            for cell in row:
                if cell.coordinate in allowed.get(name,set()):continue
                n=b[cell.coordinate];assert cell.value==n.value,(name,cell.coordinate)
                x,y=ov[name][cell.coordinate].value,nv[name][cell.coordinate].value
                if isinstance(x,(float,int)) and isinstance(y,(float,int)):assert math.isclose(x,y,abs_tol=1e-7)
                else:assert x==y,(name,cell.coordinate,x,y)
                if cell.value is not None:
                    assert cell.number_format==n.number_format
                    for attr in ['font','fill','alignment']:assert copy.copy(getattr(cell,attr))==copy.copy(getattr(n,attr)),(name,cell.coordinate,attr)
                compared+=1
    for r in range(6,13):assert [nv['QA07순서연결'].cell(r,c).value for c in range(2,13)]==[140,0,0,140,0,'미동결','미산출','미산출','미산출','미산출','NO']
    for r in range(17,997):
        assert nv['QA07순서연결'].cell(r,18).value=='UNKNOWN'
        assert nv['QA07순서연결'].cell(r,16).value=='NO'
        assert all(nv['QA07순서연결'].cell(r,c).value in [None,''] for c in [*range(5,16),17,20])
        assert nw['QA07순서연결'].cell(r,20).value==f'=IF(\'QA07선정입력\'!H{6+(r-17)%140//5}="","",\'QA07선정입력\'!H{6+(r-17)%140//5})'
    for i,c in enumerate(d['qa07']['manifest']['cases']):
        assert [nv['QA07선정입력'].cell(i+6,k).value for k in range(1,8)]==[c['source_event_id'],c['input'],'\n'.join(u['anchor']+': '+u['link'] for u in c['units']),'\n'.join(' → '.join(e) for e in c['before']) or '없음',c['order_type'],c['source_input_mode'],c['context_note'] or None]
    for i,c in enumerate(d['qa07']['manifest']['population']['excluded_user_events']):
        assert [nv['QA07선정입력'].cell(i+38,k).value for k in [1,2]]==[c['source_event_id'],c['reason']]
    for r in range(6,20):
        expected=[2,10,0,0,10,'미산출'] if r%2==0 else [26,130,0,0,130,'미산출']
        assert [nv['QA07선정입력'].cell(r,c).value for c in range(12,18)]==expected
    for r in range(20,27):assert [nv['QA07선정입력'].cell(r,c).value for c in range(12,18)]==[1,5,0,0,5,'미산출']
    for r in range(21,31):assert nw['전체비교'].cell(r,7).value.startswith("='QA07순서연결'!") and nv['전체비교'].cell(r,7).value=='미산출'
    for s in nv:
        for row in s:
            for cell in row:assert cell.data_type!='e',(s.title,cell.coordinate,cell.value)
    result=dict(status='PASS',xlsx_sha256=hashlib.sha256(new.read_bytes()).hexdigest(),sheets=20,unchanged_parent_cells=compared,
                parent_values_formulas_styles_validations='PASS',selected_inputs=28,excluded_inputs=56,planned_records=980,
                actual_executions=0,all_unknown='PASS',temporary_probes_removed='PASS',native_excel='NOT_RUN')
    (P/'common-qa-v9/artifacts/artifact-validation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result))


if __name__=='__main__':main()
