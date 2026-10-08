"""42 MAIN and comparison share the same owners, contracts and example data.

The MAIN view enlarges the structure without the presentation's assumed QA rows.
This is documentation generation, not a candidate implementation.
"""
import copy
import xml.etree.ElementTree as ET
from generate_dp_comparison_slides import Comparison, DATA
from dp_comparison_structures import graph42


class MainSlide(Comparison):
    def __init__(self):
        super().__init__(42)
        self.items=[]
        self.slug='choice42-structure'
        self.title='대화와 업무 상태의 소유 및 확정'
        self.caption='04-42. '+self.title
        self.rect(0,0,1920,1080,'white','none')
        self.text(40,25,1500,['VIA / 04-42 / 같은 기능과 다른 상태 확정 경계'],20,'#000000',True)
        self.text(1880,25,300,['A/B 미선정 / 미측정'],19,'#000000',align='right')
        self.text(40,70,1840,[self.caption],39,'#000000',True)
        self.text(40,131,1840,['같은 답변: 실제 제시된 보고서 질문 Q1에 “응, 상반기로”라고 답한다'],24,'#000000')
        self.rect(40,188,1840,55,'#D5E7C7','#000000');self.items[-1]['line_width']=1.0
        for side,x in enumerate((140,1010)):
            self.text(x+435,203,838,[DATA[42]['options'][side]],27,'#000000',True,'center')
        self.text(90,260,98,['구조'],20,'#000000',True,'center')
        first=len(self.items)
        for side,x in enumerate((140,1010)):graph42(self,x,side)
        # Expand the structure vertically on the standalone plate.
        for item in self.items[first:]:
            if 'y' in item:item['y']=250+(item['y']-243)*1.25
            if 'h' in item:item['h']*=1.25
            if 'points' in item:item['points']=[(x,250+(y-243)*1.25) for x,y in item['points']]
        self.text(40,984,1840,['u2: 답변 입력 / R2: 답변 요청 / Q1: 보고서 질문 / T1: 업무 / X1: 외부 실행 / P1·P2: 게시 / K1: 명령'],20,'#000000')
        self.text(40,1019,1840,['Legend와 같은 표기 / P1은 이미 제시됨 / 내부 접수와 외부 접수는 별도 사실'],19,'#000000')
        # Author at comparison resolution, export the architecture's 2560×1440.
        self.items=copy.deepcopy(self.items)
        for item in self.items:
            for key in ('x','y','w','h','size','leading','width','line_width'):
                if key in item:item[key]*=4/3
            if 'points' in item:item['points']=[(x*4/3,y*4/3) for x,y in item['points']]

    def svg(self):
        return super().svg().replace('width="1920" height="1080" viewBox="0 0 1920 1080"',
                                     'width="2560" height="1440" viewBox="0 0 2560 1440"')

    def diagram(self):
        d=super().diagram();m=d.find('mxGraphModel')
        m.set('pageWidth','2560');m.set('pageHeight','1440')
        return d

    def validate(self):
        ids=[i['id'] for i in self.items]
        assert len(ids)==len(set(ids)),'Duplicate IDs'
        for item in self.items:
            if item['kind'] in ('rect','store'):
                assert 0<=item['x']<=item['x']+item['w']<=2560
                assert 0<=item['y']<=item['y']+item['h']<=1440
            if item['kind']=='line':
                for a,b in zip(item['points'],item['points'][1:]):
                    assert a[0]==b[0] or a[1]==b[1],('Non-orthogonal',item['id'],a,b)
                    assert 0<=a[0]<=2560 and 0<=a[1]<=1440
        text=' '.join(line for i in self.items if i['kind']=='text' for line in i['lines'])
        for name in ('Interaction Manager','Request Controller','Request Interpreter','Response Manager','Task Manager','Agent Gateway','Model Access'):
            assert text.count(name)>=2,name+' missing in one option'
        for obsolete in ('대화 처리기','업무 관리기','Agent 연동기'):
            assert obsolete not in text,obsolete
        assert 'P1' in text and 'P2' in text and 'K1' in text
        ET.fromstring(self.svg())


def structure():
    return MainSlide()
