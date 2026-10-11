"""Editable v4 QA layouts from an isolated thought-experiment snapshot.

No Architecture, canonical comparison scene, QA contract or v3 file is written.
Fractions preserve exact score boundaries before presentation rounding.
"""
from fractions import Fraction as F
from pathlib import Path
import json
import sys

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT/'docs/presentations_files/dp-comparison/v4/qa-thought-experiment.json'
INK, MUTED, TEAL, LINE = '#18232E', '#5F6B77', '#087E8B', '#CDD6DE'


def score(value, qa):
    for index, bound in enumerate(qa['score']['bounds']):
        if (value >= F(bound) if qa['score']['direction'] == 'high' else value <= F(bound)):
            return 6-index
    return 0


def number(value, index):
    if index == 0: return f'{float(value):.2f}%'
    if index in (1, 2): return f'{float(value):.3f}초'
    if index == 3: return f'${float(value):.4f}'
    if index == 4: return f'{float(value):.3f}개'
    return f'{int(value):,}MiB'


def difference(value, index):
    if index == 0: return f'{float(value):.2f}%p'
    if index in (1, 2): return f'{float(value):.3f}초'
    if index == 3: return f'{float(value):.4f}USD'
    if index == 4: return f'{float(value):.3f}개'
    return f'{int(value)}MiB'


class Scene:
    def __init__(self, dp, kind):
        self.value = dict(number=dp, kind=kind, slug=f'dp{dp}-v4-{kind}', items=[])

    def add(self, kind, **attrs):
        self.value['items'].append(dict(kind=kind, id=f'n{len(self.value["items"])+1}', **attrs))

    def text(self, x, y, w, content, size=22, color=INK, bold=False, h=None):
        self.add('text', x=x, y=y, w=w, h=h or size*1.5, content=content,
                 size=size, color=color, bold=bold)

    def line(self, x1, y1, x2, y2):
        self.add('line', x=x1, y=y1, w=max(.1,x2-x1), h=max(.1,y2-y1), color=LINE)

    def circles(self, x, y, points, size):
        for index in range(3):
            fill = max(0, min(2, points-index*2))
            self.add('score-circle', x=x+index*(size+4), y=y, w=size, h=size, value=fill)


def scenes():
    data = json.loads(DATA.read_text())
    assert data['status'] == 'THOUGHT_EXPERIMENT_LAYOUT_DRAFT_NOT_MEASURED'
    qa = data['qa_snapshot']['rows']
    assert [r['id'] for r in qa] == [f'ASR-QA-0{i}' for i in range(1,7)]
    assumptions=data['assumptions']
    common=F(assumptions['voice_cost_usd'])+assumptions['common_semantic_calls']*F(assumptions['semantic_cost_per_call_usd'])
    a_cost=common+assumptions['dp41_extra_calls_A']*F(assumptions['semantic_cost_per_call_usd'])+assumptions['dp45_misses_A']*F(assumptions['dp45_cost_per_miss_A_usd'])
    assert all(F(d['rows'][3]['A'])==a_cost for d in data['dps'].values())
    assert F(data['dps']['41']['rows'][3]['B'])==common+F('.160')
    assert F(data['dps']['45']['rows'][3]['B'])==common+F('.168')+F('.128')
    result=[]
    for dp in (41,44,45,42):
        d=data['dps'][str(dp)]
        assert len(d['rows'])==6
        for kind in ('summary','qa-detail'):
            s=Scene(dp,kind)
            s.value['title']=f'04-{dp}. '+d['title']+' / ASR-QA 비교'
            s.value['qa_region_start']=934 if dp==41 else 871
            notes=['사고 실험 가정값 / 측정 결과와 최종 대안 선정 아님',
                   'QA 조정 전 '+data['qa_snapshot']['contract_version']+' 스냅샷 / 조정 후 값과 등급 재계산',
                   *data['notes'],*d['conditions'],
                   '데이터: docs/presentations_files/dp-comparison/v4/qa-thought-experiment.json',
                   'QA 원본: '+data['qa_snapshot']['source']]
            if kind=='qa-detail':
                s.text(40,22,1400,'VIA SOFTWARE ARCHITECTURE / ASR-QA 사고 실험',19,TEAL,True)
                s.text(40,65,1840,s.value['title'],37,INK,True)
                s.line(40,121,1880,121)
                s.text(40,142,1840,'동일 활동 기준의 A/B 가정값과 0–6점 비교 / 보호 조건 별도',24,MUTED)
                x=40
                for points in range(6,-1,-1):
                    s.circles(x,188,points,17)
                    s.text(x+69,183,85,f'{points}점',20)
                    x+=164
                s.text(1240,185,640,'원값 차이와 점수 차이는 별도 확인',20,MUTED)
                s.add('rect',x=40,y=225,w=1840,h=48,fill='#D5E7C7')
                s.text(56,235,260,'QA / 목표',23,INK,True)
                s.text(363,235,610,'A '+d['options'][0],23,INK,True)
                s.text(1013,235,610,'B '+d['options'][1],23,INK,True)
                s.text(1659,235,205,'원값 차이',23,INK,True)
            for index,row in enumerate(d['rows']):
                q=qa[index]
                assert row['qa_id']==q['id']
                values=[F(row['A']),F(row['B'])]
                scores=[score(v,q) for v in values]
                notes.append(f'{q["id"]}: A={row["A"]} ({scores[0]}점), B={row["B"]} ({scores[1]}점). A {row["reasons"][0]}. B {row["reasons"][1]}.')
                if kind=='summary':
                    y=(936+index*19) if dp==41 else (877+index*25)
                    font=14 if dp==41 else 16
                    s.line(40,y-2 if dp==41 else y-6,1880,y-2 if dp==41 else y-6)
                    s.text(42,y,245,q['name'],font,INK,True)
                    for side,x in enumerate((300,1050)):
                        s.circles(x,y+2,scores[side],12 if dp==41 else 14)
                        text=f'{scores[side]}점  {number(values[side],index)}  [{row["compact_reasons"][side]}]'
                        s.text(x+54,y,665 if side==0 else 775,text,font)
                else:
                    y=273+index*105
                    s.line(40,y+104,1880,y+104)
                    s.text(56,y+10,264,f'{index+1:02d}  '+q['name'],22,INK,True)
                    s.text(56,y+48,264,'목표 '+q['display_target'],19,MUTED)
                    for side,x in enumerate((363,1013)):
                        s.circles(x,y+16,scores[side],18)
                        s.text(x+76,y+9,85,f'{scores[side]}점',25,INK,True)
                        s.text(x+170,y+7,420,number(values[side],index),28,INK,True)
                        reason=row['reasons'][side].split(' / ')
                        for line,value in enumerate(reason):
                            s.text(x,y+52+line*25,612,value,20)
                    delta=abs(values[0]-values[1])
                    significant=delta>=F(data['difference_thresholds'][index])
                    advantage='A' if (values[0]>values[1] if index==0 else values[0]<values[1]) else 'B'
                    unit=['%p','초','초','USD','개','MiB'][index]
                    s.text(1659,y+15,205,difference(delta,index),23,INK,True)
                    label='같은 가정값' if delta==0 else f'가정상 {advantage} 유리' if significant else '차이 기준 미만'
                    s.text(1659,y+55,205,label,19,MUTED)
                    s.text(1659,y+79,205,'기준 '+data['difference_thresholds'][index]+unit,16,MUTED)
            if kind=='summary':
                if dp!=41:
                    s.text(40,1030,1840,d['conditions'][0],15,MUTED)
                s.text(40,1053 if dp==41 else 1057,1840,
                       '사고 실험 가정값 / '+data['qa_snapshot']['contract_version']+' 기준 / 실제 측정 전 / 보호 조건 별도 / 상세 비교 다음 장',14,MUTED)
            else:
                for x in (340,990,1640):s.line(x,225,x,903)
                s.text(40,931,1840,d['conditions'][0],23)
                s.text(40,968,1840,d['conditions'][1],22,MUTED)
                s.text(40,1035,1840,'사고 실험 / '+data['qa_snapshot']['contract_version']+' 스냅샷 / 실측 전 / 보호 조건 별도 / 점수 합산 없음',19,MUTED)
            s.value['notes']='\n'.join(notes)
            result.append(s.value)
    return dict(status=data['status'],scenes=result)


if __name__=='__main__':
    value=scenes()
    if len(sys.argv)>1:Path(sys.argv[1]).write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
    else:print(json.dumps(value,ensure_ascii=False))
