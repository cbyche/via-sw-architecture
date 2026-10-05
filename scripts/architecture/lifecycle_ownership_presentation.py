"""42 main: per-option lifetime, ownership and collaboration in one 16:9 scene.

Normal E4 answer handling is expanded; auxiliary plates retain their contracts.
This is presentation code, not an implementation or evaluation of either option.
"""
from stage4_diagram_design import Plate, INK, MUTED, BLUE, GREEN, LINE


class MainSlide(Plate):
    def __init__(self):
        super().__init__('choice42-structure', '04-42', '대화가 바뀌어도 남는 업무, 누가 관리하고 연결하는가?', '', height=1440)
        self.items=[]
        self.serial=0

    @staticmethod
    def text_geometry(i):
        size=27 if i['w']>=290 else 25
        leading=size*1.25
        return i['y']+(i['h']-len(i['name'])*leading)/2-2+(7 if i['type']=='store' else 0),size,leading

    def svg(self):
        return super().svg().replace('font-size="23"','font-size="27"')

    def drawio(self):
        return super().drawio().replace('fontSize=23;','fontSize=27;')

    def arrow(self,a,b,label='',at=None,sp='B',tp='T',via=(),color=INK,ret=False,sd=0,td=0,size=23,main=False):
        self.edge(a,b,sp=sp,tp=tp,via=via,color=color,ret=ret,sd=sd,td=td)
        self.items[-1]['width']=3.3 if main else 1.8
        if label:
            self.label(*at,label,color,size=size)

    def validate(self):
        super().validate()
        limits={'A':(64,1248),'B':(1312,2496)}
        for key,(x,y,w,h) in self.nodes.items():
            lo,hi=limits[key[0]]
            assert lo<=x and x+w<=hi,(key,'outside option column')
        for i in self.items:
            if i['kind']=='line' and 'source' in i:
                assert i['source'][0]==i['target'][0],(i['id'],'cross-option edge')
                lo,hi=limits[i['source'][0]]
                assert all(lo<=x<=hi for x,y in i['points']),(i['id'],'edge leaves column')


def structure():
    p=MainSlide()
    p.box(0,0,2560,8,INK,'none',0)
    p.text(64,26,'VIA / 04-42 / 수명 · 상태 소유 · 협력',23,MUTED,bold=True)
    p.text(2496,26,'설계 비교 · 미선정 · 미측정',23,MUTED,'right')
    p.text(64,70,p.title,43,bold=True)
    p.text(64,135,'같은 정상 장면  다른 대화에서 돌아와, 보고서 Agent의 “상반기로 할까요?”에 “응, 상반기로 해줘.”',28)

    for opt,dx,c,title in [('A',0,BLUE,'A  하나의 Core 안에서 대화와 업무 관리'),
                           ('B',1248,GREEN,'B  독립된 대화·업무 서비스가 협력')]:
        p.box(64+dx,211,1184,1133,'white',LINE,0)
        p.box(64+dx,211,1184,48,'#EEF4FF' if opt=='A' else '#EDF9F2','none',0)
        p.text(80+dx,218,title,30,c,bold=True)
        p.text(82+dx,274,'보고서 요청 → 질문 도착 → 다른 대화 → 복귀·답변 → 결과·수정',25,bold=True)
        # Compact common lifetimes are repeated in each option, not a shared path.
        for yy,label in [(325,'대화 기록 C1'),(362,'최초 요청 R1'),(399,'대기 질문 Q1'),(436,'보고서 업무 T1'),(473,'외부 실행')]:
            p.text(84+dx,yy,label,23,bold=True)
        def life(x1,x2,y,label,tx,ongoing=False):
            p.line([(x1+dx,y+23),(x2+dx,y+23)],INK,width=2,arrow=ongoing)
            p.line([(x1+dx,y+16),(x1+dx,y+30)],INK,arrow=False,width=1.5)
            if not ongoing:
                p.line([(x2+dx,y+16),(x2+dx,y+30)],INK,arrow=False,width=1.5)
            p.label(tx+dx,y-1,label,INK,size=22)
        life(360,1208,325,'다른 대화로 이동해도 기록·참조 유지',520,True)
        life(360,405,362,'위임 접수 후 요청 처리는 종료',440)
        life(590,1010,399,'대화 전환 중에도 답변 대기',620)
        life(360,1208,436,'결과를 보존하고 같은 보고서를 수정',515,True)
        life(405,1040,473,'X1  질문 대기 → 재개 → 완료',480)
        life(1110,1208,473,'X2',1130,True)
        p.line([(80+dx,521),(1232+dx,521)],LINE,arrow=False)
        p.text(82+dx,531,'위의 대화·요청과 업무·질문을 아래 담당자가 소유한다',24,c,bold=True)

        for key,x,w,name,kind in [('ui',120,310,'사용자 입출력','external'),
                                  ('omni',485,310,'공유 Omni','model'),
                                  ('agent',850,310,'Downstream Agent','external')]:
            p.node(opt+key,x+dx,582,w,62,name,kind=kind)

        if opt=='A':
            p.text(96,654,'하나의 Core process',23,c,bold=True)
            p.box(72,686,1146,640,'none',c,8,dashed=True,thick=2.5)
            p.component('Acore',80,695,1130,620,'VIA Core',c)
            d_owner=t_owner='Acore'
        else:
            p.text(1344,654,'독립 process',23,c,bold=True)
            p.text(2052,654,'독립 process',23,c,bold=True)
            for key,x in [('dialogue',1330),('work',2040)]:
                p.box(x-8,686,436,640,'none',c,8,dashed=True,thick=2.5)
                p.component('B'+key,x,695,420,620,'대화 서비스' if key=='dialogue' else '업무 서비스',c)
            d_owner,t_owner='Bdialogue','Bwork'
        p.node(opt+'d',120+dx,795,310,90,'대화 처리기',owner=d_owner)
        p.node(opt+'t',850+dx,795,310,90,'업무 관리기',owner=t_owner)

        def arrow(a,b,label='',at=None,**kw):
            if at:
                at=(at[0]+dx,at[1])
            if 'via' in kw:
                kw['via']=[(x+dx,y) for x,y in kw['via']]
            p.arrow(opt+a,opt+b,label,at,**kw)
        arrow('ui','d','1  “상반기로” 답변',(128,755),sd=-90,td=-90,
              via=[(185,650),(90,650),(90,786),(185,786)],main=True,size=22)
        arrow('d','ui','6  확인된 상태 전달',(290,718),sp='T',tp='B',sd=90,td=90,main=True,size=22)
        arrow('ui','d','실제 게시 기록',(294,681),sd=50,td=50,ret=True,size=21)
        arrow('d','omni','같은 의미 해석',(465,712),sp='T',tp='B',sd=115,td=-60,
              via=[(390,750),(580,750)],size=22)
        arrow('omni','d','모델 결과',(700,715),sd=60,td=135,
              via=[(700,780),(410,780)],ret=True,size=21)
        arrow('t','agent','확정 후 답변 전달',(962,715),sp='T',tp='B',sd=65,td=65,main=True,size=22)
        arrow('agent','t','5  외부 접수·진행·질문·결과',(863,753),sd=-65,td=-65,
              via=[(940,658),(1190,658),(1190,782),(940,782)],ret=True,size=21)
        # Thin bounded reads are separate from the thick normal answer proposal.
        arrow('d','t','질문·업무 조회 / 상태 반환',(478,767),sp='R',tp='L',sd=-25,td=-25,size=22)
        arrow('t','d',sp='L',tp='R',sd=-14,td=-14,ret=True)
        arrow('d','t','2  답변 변경 제안' if opt=='A' else '2  조건부 답변 명령',
              (484,825),sp='R',tp='L',sd=10,td=10,color=c,main=True,size=24)
        arrow('t','d','5  확인된 외부 Agent 접수',(477,884),sp='L',tp='R',sd=34,td=34,ret=True,size=22)

        if opt=='A':
            p.node('Acommit',490,1010,300,62,'트랜잭션 관리자',owner='Acore',color=c)
            p.node('Asaved',120,1140,1040,78,'대화·업무 상태',kind='store',owner='Acore',color=c)
            arrow('d','commit','3  답변·제시 연결 변경',(160,931),td=-80,
                  via=[(275,977),(560,977)],color=c,main=True,size=22)
            arrow('t','commit','4  질문·전송 명령 변경',(854,931),td=80,
                  via=[(1005,977),(720,977)],color=c,main=True,size=22)
            arrow('commit','d','로컬 저장 결과',(222,1016),sp='L',tp='B',td=95,
                  via=[(450,1041),(450,993),(370,993)],color=c,ret=True,size=22)
            arrow('commit','t','로컬 저장 결과',(846,1016),sp='R',tp='B',td=-95,
                  via=[(825,1041),(825,993),(910,993)],color=c,ret=True,size=22)
            arrow('commit','saved','검증한 관련 변경과 전송 준비를 함께 저장',(486,1089),td=0,
                  via=[(640,1125)],color=c,main=True,size=22)
            arrow('saved','commit',sp='T',tp='B',sd=80,td=80,color=c,ret=True)
        else:
            for role,x,owner,title in [('d',1368,'Bdialogue','대화 상태'),('t',2098,'Bwork','업무 상태')]:
                p.node('B'+role+'saved',x,1140,310,78,title,kind='store',owner=owner,color=c)
                label='3  답변 전달 대기 저장\n내부 접수 결과로 다시 갱신' if role=='d' else '4  질문·전송 명령과\nVIA 내부 접수 결과 저장'
                p.arrow('B'+role,'B'+role+'saved',label,(x+8,1039),sd=-90,td=-90,color=c,main=True,size=22)
                p.arrow('B'+role+'saved','B'+role,sp='T',tp='B',sd=90,td=90,color=c,ret=True)
            arrow('t','d','4 저장 후 VIA 내부 접수 결과',(490,953),sp='B',tp='B',sd=-135,td=135,
                  via=[(870,1001),(410,1001)],color=c,ret=True,size=22)
            p.text(1790,1065,'같은 명령 ID로\n접수 결과 재조회',22,MUTED,leading=31)

        p.text(130+dx,1232,['대화 기록 C1 · 최초 요청 R1','답변 요청 · VIA 질문 · 실제 제시 기록'],22,INK,leading=32)
        p.text(860+dx,1232,['보고서 업무 T1 · 외부 실행 X1/X2','대기 질문 Q1 · 전송 명령'],22,INK,leading=32)

    p.text(64,1362,'점선 테두리: process  ·  굵은 선: 답변 처리  ·  가는 선: 조회/추론/반환  ·  점선 화살표: 반환·외부 알림',22,MUTED)
    p.text(2496,1402,'각 칸은 독립 대안 · 한 벌의 Omni / 같은 해석·권한 조건 · 수명 선은 사건 순서, 종료 ≠ 기록 삭제 · 내부 접수 ≠ 외부 접수',20,MUTED,'right')
    return p
