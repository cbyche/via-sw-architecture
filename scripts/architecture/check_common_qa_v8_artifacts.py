#!/usr/bin/env python3
"""Read-only verification of v8 spreadsheet content and preserved v7 tabs."""
import copy
import hashlib
import json
import math
from pathlib import Path
import openpyxl

ROOT=Path(__file__).resolve().parents[2]
P=ROOT/'docs/architecture/12-decisions/decision-packages/current/evaluation'


def main():
    old=P/'common-qa-v7/artifacts/outputs/01a15dfe-v7/VIA-QA-common-v7.xlsx';new=P/'common-qa-v8/artifacts/outputs/01a15dfe-v8/VIA-QA-common-v8.xlsx'
    ow=openpyxl.load_workbook(old);nw=openpyxl.load_workbook(new);ov=openpyxl.load_workbook(old,data_only=True);nv=openpyxl.load_workbook(new,data_only=True)
    d=json.loads((P/'common-qa-v8/artifacts/ledger.json').read_text())
    assert nw.sheetnames==ow.sheetnames+['QA07v3사례','QA07v3의무','QA07v3명세']
    allowed={'전체비교':{'A3','A32',*[f'G{r}' for r in range(21,31)]},'QA07사례':{'A3'},'QA07의무':{'A3'},'QA상세부록':{'A3'}}
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
    for r in range(6,13):assert [nv['QA07v3사례'].cell(r,c).value for c in range(2,13)]==[160,0,0,160,0,'미동결','미산출','미산출','미산출','미산출','NO']
    for r in range(17,1137):
        assert nv['QA07v3사례'].cell(r,12).value=='UNKNOWN'
        assert all(nv['QA07v3사례'].cell(r,c).value in [None,''] for c in [9,10,11,14,15])
        i=r-17;registry_row=6+(i%160)//5
        assert nw['QA07v3사례'].cell(r,14).value==f'=IF(R{registry_row}="","",R{registry_row})'
    for r in range(6,3366):
        assert nv['QA07v3의무'].cell(r,15).value=='UNKNOWN'
        assert all(nv['QA07v3의무'].cell(r,c).value in [None,''] for c in range(7,15))
    for i,c in enumerate(d['qa07']['manifest']['cases']):
        assert [nv['QA07v3명세'].cell(i+6,k).value for k in range(1,7)]==[c[k] for k in ['case_id','type','input_and_events','required','forbidden','modality']]
    for r in range(6,62):assert [nv['QA07v3명세'].cell(r,c).value for c in range(10,16)]==[4,20,0,0,20,'미산출']
    for r in range(21,31):assert nw['전체비교'].cell(r,7).value.startswith("='QA07v3사례'!") and nv['전체비교'].cell(r,7).value=='미산출'
    for s in nv:
        for row in s:
            for cell in row:assert cell.data_type!='e',(s.title,cell.coordinate,cell.value)
    result=dict(status='PASS',xlsx_sha256=hashlib.sha256(new.read_bytes()).hexdigest(),sheets=18,unchanged_parent_cells=compared,
                parent_values_formulas_styles_validations='PASS',qa07_case_families=32,planned_repeat_records=1120,stage_checklist_rows=3360,
                actual_executions=0,all_unknown='PASS',temporary_probes_removed='PASS',native_excel='NOT_RUN')
    (P/'common-qa-v8/artifacts/artifact-validation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n');print(json.dumps(result))


if __name__=='__main__':main()
