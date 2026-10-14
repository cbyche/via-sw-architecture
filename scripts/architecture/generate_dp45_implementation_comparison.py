"""One-page DP45 comparison derived from the detailed implementation designs."""
from pathlib import Path
import sys, json, argparse, hashlib
import xml.etree.ElementTree as ET
from dp45_implementation_scene import MemoryScene, INK, BLUE, GRAY, RED
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'

class Comparison(MemoryScene):
    def __init__(self):
        super().__init__('A');self.nodes={};self.edges=[];self.notes=[]
        self.slug='dp45-implementation-comparison';self.title='DP-45 과거 이력의 근거 공급 구조'
        self.text(64,82,self.title,64,bold=True)
        self.text(64,147,'같은 요청: “지난번 설명한 기준에 제품 비교와 견적을 맞춰 제안서를 만들어줘.”',40)
        self.text(64,201,'설명 원문과 실제 전달 범위, 제품 결과와 견적이 원본이다. 현재 Task 연결과 의미 채택은 양안 공통이다.',32)
    def texts(self,n):
        x,y,w,h=(n[k] for k in ('x','y','w','h'))
        if n['kind'] in ('component','group'):
            return [(x+18,y+36,n['title'],36,True,'left')]
        r=[(x+w/2,y+46,n['title'],32,True,'center')]
        for i,t in enumerate(n['lines']):r.append((x+18,y+94+i*40,t,32,False,'left'))
        return r
    def drawio(self):
        xml=super().drawio()
        return xml.replace('font-size:27px','font-size:32px').replace('font-size:23px','font-size:32px').replace('fontSize=30;','fontSize=36;')

def build():
    p=Comparison()
    for side,x in [('A',64),('B',1984)]:
        def n(key,dx,y,w,h,title,lines=(),kind='code',owner=None,different=False):
            return p.node(side+key,x+dx,y,w,h,title,lines,kind,side+owner if owner else None,different)
        def e(s,t,sp='R',tp='L',via=(),kind='post',sd=0,td=0):
            p.edge(side+s,side+t,sp,tp,[(x+a,b) for a,b in via],kind,sd,td)
        def label(dx,y,t):p.text(x+dx,y,t,32,color=BLUE,mask=True)
        p.text(x,272,('A  요청별 원본 조회와 조합' if side=='A' else 'B  별도 관계 생산과 게시본 조회'),48,bold=True)
        n('ri',24,324,520,180,'Request Interpreter',kind='component')
        n('caller',44,390,480,100,'Read Tool Runner',owner='ri')
        n('rm',664,324,520,180,'Response Manager',kind='component')
        n('response',684,390,480,100,'원문 / 실제 전달 read port',owner='rm')
        n('tm',1264,324,520,180,'Task Manager',kind='component')
        n('task',1284,390,480,100,'Task / 결과 참조 read port',owner='tm')
        n('cm',24,574,1760,1206,'Context Manager',kind='component')
        n('model',24,1844,520,180,'Model Access',kind='component')
        n('api',44,1910,480,100,'Semantic API Client',owner='model')
        n('rc',664,1844,520,180,'Request Controller',kind='component')
        n('adoption',684,1910,480,100,'입력 이력 read / 현재 채택',owner='rc')
        n('policy',1264,1844,520,180,'Policy Manager',kind='component')
        n('use',1284,1910,480,100,'현재 권한 검사 / 사용 등록',owner='policy')
        if side=='A':
            n('composer',64,650,1680,680,'Context Composer',kind='group',owner='cm',different=True)
            n('planner',104,748,460,180,'Read Pump / Planner',['접수, 범위와 조회 계획','필요한 읽기 병렬 시작'],owner='composer',different=True)
            n('broker',704,748,460,180,'Owner Read Broker',['search / read / validate','원본 버전과 누락 반환'],owner='composer',different=True)
            n('bundle',1304,748,400,180,'Bundle Builder',['실제 전달 범위로 조합','사용 검사 후 1회 반환'],owner='composer',different=True)
            n('evidence',704,1080,460,180,'Request Evidence Set',['RAM / 이번 read의 발췌','원본 버전과 pending read'],kind='store',owner='composer',different=True)
            n('jobs',1304,1080,400,180,'Read Job Index',['RAM / job과 cancel','늦은 완료 회수'],kind='store',owner='composer',different=True)
            n('cache',104,1440,460,200,'Source View Cache',['재사용 발췌와 관계','검증한 cache / 부분 갱신'],kind='store',owner='cm',different=True)
            n('invalid',704,1440,460,200,'Invalidation Handler',['변경 dependency 차단','cache와 활성 read 정리'],owner='cm',different=True)
            n('inbox',1304,1440,400,200,'Change Inbox',['원본 변경, 철회와 이의','정리 queue / 사용 차단 별도'],kind='queue',owner='cm',different=True)
            e('caller','planner','L','L',via=[(4,440),(4,838)])
            label(34,722,'1 현재 Query')
            e('planner','broker');label(580,822,'2 읽기')
            e('broker','response','T','B',sd=-80,td=-70,via=[(854,540),(854,540)],kind='call')
            e('response','broker','B','T',sd=70,td=60,via=[(994,554),(994,554)],kind='done')
            e('broker','task','T','B',sd=170,td=-70,via=[(1104,528),(1454,528)],kind='call')
            e('task','broker','B','T',sd=70,td=200,via=[(1594,562),(1134,562)],kind='done')
            e('broker','evidence','B','T',kind='done');label(720,1024,'3 완료를 read에 반영')
            e('broker','jobs','R','T',sd=50,via=[(1214,888),(1214,1026),(1504,1026)],kind='state')
            e('evidence','bundle','R','L',via=[(1240,1170),(1240,838)],kind='state');label(1174,1065,'4 조합')
            e('bundle','caller','T','R',sd=130,td=0,via=[(1634,622),(600,622),(600,440)],kind='done');label(634,611,'5 Bundle + 읽은 범위 / 버전')
            e('planner','cache','B','T',sd=-60,td=-60,kind='state')
            e('cache','planner','T','B',sd=60,td=60,kind='done');label(120,1384,'검증한 cache 재사용 / 저장')
            e('inbox','invalid','L','R',kind='cancel');e('invalid','cache','L','R',kind='cancel')
            e('invalid','jobs','T','B',sd=160,td=-100,via=[(1094,1376),(1404,1376)],kind='cancel')
            e('task','inbox','R','R',sd=20,td=20,via=[(1812,460),(1812,1560)],kind='done')
            e('bundle','use','R','R',sd=50,td=0,via=[(1828,888),(1828,1960)],kind='call')
            e('use','bundle','R','R',sd=20,td=70,via=[(1844,1980),(1844,908)],kind='done')
            e('planner','api','L','L',sd=20,td=0,via=[(80,858),(80,1816),(32,1816),(32,1960)],kind='call')
            e('api','planner','R','L',sd=20,td=40,via=[(592,1980),(592,1832),(64,1832),(64,878)],kind='done')
            e('broker','adoption','R','R',sd=30,td=0,via=[(1220,868),(1220,1960)],kind='call')
            e('adoption','broker','R','R',sd=20,td=50,via=[(1232,1980),(1232,888)],kind='done')
            # Secondary owner reads keep the same source authority.
            p.text(x+704,1310,'S: 입력 이력도 같은 Broker로 읽기',28,color=GRAY)
            p.text(x+104,1712,'누락은 이번 read에서 해결. 반복 요청은 cache를 재사용한다.',32,bold=True)
        else:
            n('publisher',64,650,1680,490,'Memory Publisher',kind='group',owner='cm',different=True)
            n('admit',104,730,460,160,'Change Admission',['확정 변경 / backfill 접수','범위, 예산과 생산 job 등록'],owner='publisher',different=True)
            n('worker',704,730,460,160,'Source Read Worker',['job claim / 원본 읽기','S 입력 이력도 확인'],owner='publisher',different=True)
            n('derive',1304,730,400,160,'Relation Builder',['명시 관계는 코드','의미 추출은 조건부 모델'],owner='publisher',different=True)
            n('journal',104,958,460,160,'Production Journal',['DURABLE / job, retry','게시 통지 outbox'],kind='store',owner='publisher',different=True)
            n('candidate',704,958,460,160,'Candidate Batch',['RAM / 게시 전 후보','출처, 구간과 uncertainty'],kind='store',owner='publisher',different=True)
            n('commit',1304,958,400,160,'Publish Guard',['source / 권한 / 개정 검사','조건부 transaction'],owner='publisher',different=True)
            n('wait',104,1214,460,172,'Read Wait Registry',['RAM / 현재 read 대기','deadline / 취소 / 재조회'],kind='store',owner='cm',different=True)
            n('repo',704,1214,1000,172,'Episodic Memory Repository',['DURABLE / 관계 + Coverage + 게시 개정 + cursor','지원한 범위와 누락 / dependency 역색인'],kind='store',owner='cm',different=True)
            n('reader',64,1450,1680,276,'Evidence Reader',kind='group',owner='cm',different=True)
            n('read',104,1524,480,174,'Read Pump / Bundle Builder',['현재 read 접수와 반환','검증한 근거와 누락 조합'],owner='reader',different=True)
            n('coverage',704,1524,460,174,'Index / Coverage Resolver',['게시된 관계와 범위 읽기','미생산이면 생산 후 재조회'],owner='reader',different=True)
            n('origin',1304,1524,400,174,'Origin Verifier',['S 원본 버전 / 인용 확인','현재 권한 검사'],owner='reader',different=True)
            e('task','admit','B','T',sd=70,td=210,via=[(1594,526),(544,526)],kind='done');label(400,548,'1 원본 변경 또는 지원 범위 생산 요청')
            e('admit','journal','B','T',kind='state')
            e('worker','journal','L','R',sd=20,td=-25,via=[(620,830),(620,1013)],kind='state')
            e('journal','worker','R','L',sd=20,td=45,via=[(648,1058),(648,855)],kind='done')
            e('worker','response','T','B',sd=-80,td=-70,kind='call')
            e('response','worker','B','T',sd=70,td=60,kind='done')
            e('worker','task','T','B',sd=170,td=-70,via=[(1104,562),(1454,562)],kind='call')
            e('task','worker','B','T',sd=110,td=200,via=[(1634,604),(1134,604)],kind='done')
            e('worker','derive');label(1178,790,'2 생산')
            e('derive','candidate','B','T',sd=-80,via=[(1424,924),(934,924)],kind='done')
            e('candidate','commit');label(1176,1020,'3 검증')
            e('commit','repo','B','T',sd=0,td=300,kind='state');label(1350,1190,'4 관계와 Coverage 함께 게시')
            e('commit','journal','B','B',sd=-120,td=0,via=[(1384,1170),(334,1170)],kind='state')
            e('caller','read','L','L',via=[(4,440),(4,1611)]);label(34,1580,'5')
            e('read','coverage');label(580,1594,'6 조회')
            e('coverage','repo','T','B',sd=-60,td=-330,kind='call')
            e('repo','coverage','B','T',sd=-210,td=60,kind='done')
            e('coverage','origin');label(1180,1594,'7 확인')
            e('origin','response','R','R',sd=-40,td=0,via=[(1812,1571),(1812,516),(1200,516),(1200,440)],kind='call')
            e('response','origin','R','R',sd=20,td=-20,via=[(1216,460),(1216,508),(1824,508),(1824,1591)],kind='done')
            e('origin','task','R','R',sd=0,td=25,via=[(1836,1611),(1836,465)],kind='call')
            e('task','origin','R','R',sd=35,td=20,via=[(1848,475),(1848,1631)],kind='done')
            e('origin','read','B','B',sd=-80,td=80,via=[(1424,1748),(424,1748)],kind='done')
            e('read','caller','L','R',sd=30,td=0,via=[(20,1641),(20,534),(600,534),(600,440)],kind='done');label(80,1810,'8 Bundle + 읽은 범위 / 버전')
            e('coverage','wait','L','R',sd=40,td=30,via=[(628,1651),(628,1330)],kind='state');label(106,1430,'9 미생산이면 대기 등록')
            e('coverage','admit','L','L',sd=60,td=-20,via=[(652,1671),(652,1740),(80,1740),(80,790)],kind='post')
            e('commit','wait','R','T',sd=30,td=-140,via=[(1728,1068),(1728,1196),(194,1196)],kind='done')
            e('wait','read','B','T',sd=210,td=200,kind='done')
            e('read','wait','T','B',sd=160,td=170,kind='state')
            p.text(x+704,1430,'게시 알림 후 6 재조회 / 미지원은 제한 반환',28,color=BLUE,mask=True)
            e('derive','api','R','R',sd=0,td=0,via=[(1758,810),(1758,1818),(580,1818),(580,1960)],kind='call')
            e('api','derive','R','R',sd=20,td=20,via=[(592,1980),(592,1832),(1770,1832),(1770,830)],kind='done')
            e('read','use','B','L',sd=140,td=0,via=[(484,1800),(1248,1800),(1248,1960)],kind='call')
            e('use','read','L','B',sd=20,td=180,via=[(1236,1980),(1236,1824),(524,1824)],kind='done')
            # Input-history reads use the named S port, identical to A.
            p.text(x+704,1810,'S: 원본 확인에는 입력 이력도 포함',28,color=GRAY,mask=True)
        p.text(x+24,2070,'M  cloud 의미 API',28,color=GRAY)
        p.text(x+664,2070,'S  입력 이력 / 현재 의미 채택',28,color=GRAY)
        p.text(x+1264,2070,'U  철회 이후 사용 차단',28,color=GRAY)
    p.text(64,2140,'실선: 요청 / 점선: 완료   둥근 상자: 코드   원통: 상태   살구색: 변경 구조   소유 Component 안의 비동기 작업이며 process 수를 뜻하지 않음',28)
    p.text(3776,2140,'설계안 / 미선정 / 미측정',28,align='right')
    return p

def main():
    a=argparse.ArgumentParser();a.add_argument('--check',action='store_true');a.add_argument('--json',type=Path);args=a.parse_args()
    p=build();p.validate()
    for ext,value in [('svg',p.svg()),('drawio',p.drawio())]:
        ET.fromstring(value);path=OUT/(p.slug+'.'+ext)
        if args.check:assert path.read_text()==value,path
        else:path.write_text(value)
    if args.check:
        body=(OUT.parent/'04-45-memory-and-context.md').read_text()
        for ext in ('svg','drawio','png'):assert f'](./diagrams/{p.slug}.{ext})' in body
        record=json.loads((OUT/(p.slug+'-render.json')).read_text())['records'][0]
        assert record['text_geometry_issues']==0
        for ext in ('svg','png'):assert hashlib.sha256((OUT/(p.slug+'.'+ext)).read_bytes()).hexdigest()==record[ext+'_sha256']
    if args.json:args.json.write_text(json.dumps(dict(width=3840,height=2160,title=p.title,nodes=list(p.nodes.values()),edges=p.edges,notes=p.notes,texts={k:p.texts(v) for k,v in p.nodes.items()}),ensure_ascii=False))
    print('PASS: DP45 one-page ownership, geometry, routes and source pairs')
if __name__=='__main__':main()
