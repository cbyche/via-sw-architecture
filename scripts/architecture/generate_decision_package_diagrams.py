#!/usr/bin/env python3
"""Editable, paired presentation diagrams for target-derived decision alternatives.

This is documentation tooling; it does not execute or measure VIA candidates.
Coordinates, text and routes have one source for SVG and draw.io.
"""
from __future__ import annotations
import argparse
import html
from pathlib import Path
import xml.etree.ElementTree as ET

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'docs/architecture/12-decisions/decision-packages/diagrams'
INK='#171717'; BLUE='#1458C0'; WHITE='#FFFFFF'; PALE='#F3F7FD'; GRAY='#F4F4F4'
FONT='Arial, Apple SD Gothic Neo, Malgun Gothic, sans-serif'
W,H=2560,1440

class Page:
 def __init__(self,slug,title,subtitle):
  self.slug=slug; self.title=title; self.subtitle=subtitle; self.shapes=[]; self.edges=[]; self.texts=[]
  self.text('title',48,58,title,38,bold=True)
  self.text('subtitle',48,104,subtitle,22)
  self.text('footer',48,1414,'VIA  /  Decision Reconstruction  /  설계 비교 · 구현 및 측정 결과 아님',18)
 def text(self,id,x,y,lines,size=22,color=INK,bold=False):
  if isinstance(lines,str):lines=[lines]
  self.texts.append(dict(id=id,x=x,y=y,lines=lines,size=size,color=color,bold=bold))
 def rect(self,id,x,y,w,h,label='',blue=False,kind='box',lines=()):
  self.shapes.append(dict(id=id,x=x,y=y,w=w,h=h,label=label,blue=blue,kind=kind,lines=list(lines)))
  return id
 def node(self,id,x,y,w,h,title,*lines,blue=False,kind='component'):
  return self.rect(id,x,y,w,h,title,blue,kind,lines)
 def group(self,id,x,y,w,h,title,blue=False,kind='group'):
  return self.rect(id,x,y,w,h,title,blue,kind)
 def port(self,id,p):
  s=next(a for a in self.shapes if a['id']==id)
  x,y,w,h=(s[a] for a in ('x','y','w','h'))
  return {'L':(x,y+h/2),'R':(x+w,y+h/2),'T':(x+w/2,y),'B':(x+w/2,y+h)}[p]
 def edge(self,id,source,target,sp='R',tp='L',via=(),label='',at=None,blue=False,dash=False):
  pts=[self.port(source,sp),*via,self.port(target,tp)]
  self.edges.append(dict(id=id,source=source,target=target,points=pts,label=label,at=at,blue=blue,dash=dash))
 def rawedge(self,id,pts,label='',at=None,blue=False,dash=False):
  self.edges.append(dict(id=id,source=None,target=None,points=pts,label=label,at=at,blue=blue,dash=dash))
 def validate(self):
  ids=[s['id'] for s in self.shapes]+[s['id'] for s in self.texts]+[s['id'] for s in self.edges]
  assert len(ids)==len(set(ids)),(self.slug,'duplicate id')
  for s in self.shapes:
   assert 0<=s['x'] and 0<=s['y'] and s['x']+s['w']<=W and s['y']+s['h']<=H,(self.slug,s['id'],'bounds')
   if s['kind'] not in ('group','process'):
    assert (42+len(s['lines'])*23 if s['kind']=='module' else 49+len(s['lines'])*27)<=s['h']+9,(self.slug,s['id'],'text height')
  for e in self.edges:
   for terminal,point in [('source',e['points'][0]),('target',e['points'][-1])]:
    if e[terminal]:
     n=next(n for n in self.shapes if n['id']==e[terminal])
     assert n['x']<=point[0]<=n['x']+n['w'] and n['y']<=point[1]<=n['y']+n['h'],(self.slug,e['id'],terminal,'attachment outside node')
   for x,y in e['points']:assert 0<=x<=W and 0<=y<=H,(self.slug,e['id'],'edge bounds')
   for a,b in zip(e['points'],e['points'][1:]):
    assert a[0]==b[0] or a[1]==b[1],(self.slug,e['id'],'non-orthogonal route')
    for n in self.shapes:
     if n['kind'] in ('group','process') or n['id'] in (e['source'],e['target']):continue
     x,y,w,h=(n[k] for k in ('x','y','w','h'))
     vertical=a[0]==b[0] and x+1<a[0]<x+w-1 and max(min(a[1],b[1]),y+1)<min(max(a[1],b[1]),y+h-1)
     horizontal=a[1]==b[1] and y+1<a[1]<y+h-1 and max(min(a[0],b[0]),x+1)<min(max(a[0],b[0]),x+w-1)
     assert not (vertical or horizontal),(self.slug,e['id'],'route crosses',n['id'])
 def svg(self):
  esc=html.escape
  a=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" aria-labelledby="title desc">',f'<title id="title">{esc(self.title)}</title><desc id="desc">{esc(self.subtitle)}</desc>',f'<defs><marker id="k" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L10 5L0 10Z" fill="{INK}"/></marker><marker id="b" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto"><path d="M0 0L10 5L0 10Z" fill="{BLUE}"/></marker></defs>',f'<rect width="{W}" height="{H}" fill="white"/><g font-family="{FONT}">']
  def shape(s):
   x,y,w,h=(s[k] for k in ('x','y','w','h')); c=BLUE if s['blue'] else INK; kind=s['kind']
   process_node=self.slug=='speech-evidence-source-comparison' and s['id'] in ('a-recognize','a-omni')
   fill=PALE if s['blue'] else WHITE
   if kind in ('group','process'):fill=WHITE
   dash=' stroke-dasharray="10 7"' if kind in ('process','external') or process_node else ''
   out=[f'<g id="{s["id"]}"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{0 if kind in ("group","process") else 8}" fill="{fill}" stroke="{c}" stroke-width="{2 if kind in ("group","process") else 2.5}"{dash}/>']
   if kind=='store':
    out.append(f'<path d="M{x} {y+20} Q{x+w/2} {y+48} {x+w} {y+20}" fill="none" stroke="{c}" stroke-width="2"/>')
   if s['label']:
    ty=y+28 if kind!='store' else y+46
    out.append(f'<text x="{x+16}" y="{ty}" font-size="{18 if kind=='module' else 20}" font-weight="700" fill="{c}">{esc(s["label"])}</text>')
   if kind=='component':
    out.append(f'<path d="M{x} {y+45}H{x+w}" stroke="{c}" fill="none"/>')
   for i,line in enumerate(s['lines']):
    yy=(y+54+i*23 if kind=='module' else y+64+i*25+(15 if kind=='store' else 0))
    out.append(f'<text x="{x+16}" y="{yy}" font-size="{18 if kind=='module' else 20}" fill="{c}">{esc(line)}</text>')
   return ''.join(out)+ '</g>'
  for s in self.shapes:
   if s['kind'] in ('group','process'):a.append(shape(s))
  for e in self.edges:
   c=BLUE if e['blue'] else INK; dash=' stroke-dasharray="8 6"' if e['dash'] else ''
   pts=' '.join(f'{x},{y}' for x,y in e['points'])
   a.append(f'<polyline id="{e["id"]}" points="{pts}" fill="none" stroke="{c}" stroke-width="2.5" stroke-linejoin="round" marker-end="url(#{"b" if e["blue"] else "k"})"{dash}/>')
  for s in self.shapes:
   if s['kind'] not in ('group','process'):a.append(shape(s))
  for e in self.edges:
   if e['label']:
    x,y=e['at']; c=BLUE if e['blue'] else INK
    a.append(f'<text x="{x}" y="{y}" font-size="19" fill="{c}" stroke="white" stroke-width="7" paint-order="stroke">{esc(e["label"])}</text>')
  for t in self.texts:
   for i,line in enumerate(t['lines']):
    a.append(f'<text x="{t["x"]}" y="{t["y"]+i*(t["size"]+10)}" font-size="{t["size"]}" font-weight="{700 if t["bold"] else 400}" fill="{t["color"]}">{esc(line)}</text>')
  return '\n'.join(a+['</g></svg>'])+'\n'
 def drawio(self):
  f=ET.Element('mxfile',host='app.diagrams.net',type='device')
  d=ET.SubElement(f,'diagram',name=self.title,id=self.slug)
  m=ET.SubElement(d,'mxGraphModel',page='1',pageWidth=str(W),pageHeight=str(H),grid='1',gridSize='10')
  r=ET.SubElement(m,'root');ET.SubElement(r,'mxCell',id='0');ET.SubElement(r,'mxCell',id='1',parent='0')
  shapes={s['id']:s for s in self.shapes}
  def parent(s):
   containers=[g for g in self.shapes if g['id']!=s['id'] and g['kind'] in ('group','process') and g['x']<=s['x'] and g['y']<=s['y'] and g['x']+g['w']>=s['x']+s['w'] and g['y']+g['h']>=s['y']+s['h']]
   return min(containers,key=lambda g:g['w']*g['h']) if containers else None
  def cell(id,x,y,w,h,value,style,pid='1'):
   c=ET.SubElement(r,'mxCell',id=id,value=value,style=style,vertex='1',parent=pid)
   ET.SubElement(c,'mxGeometry',x=str(x),y=str(y),width=str(w),height=str(h),attrib={'as':'geometry'})
  def txt(id,x,y,w,size,value,color,bold=False,pid='1'):
   cell(id,x,y-size,w,size+7,html.escape(value),f'text;html=1;whiteSpace=nowrap;overflow=visible;strokeColor=none;fillColor=none;fontColor={color};fontSize={size};fontStyle={1 if bold else 0};fontFamily=Arial;align=left;verticalAlign=top;spacing=0;connectable=0;',pid)
  # Parent cells preserve genuine containment; labels remain individually editable.
  for s in sorted(self.shapes,key=lambda z:-(z['w']*z['h'])):
   c=BLUE if s['blue'] else INK;kind=s['kind'];fill=PALE if s['blue'] else WHITE
   if kind in ('group','process'):fill=WHITE
   g=parent(s);x=s['x']-(g['x'] if g else 0);y=s['y']-(g['y'] if g else 0)
   style=f'rounded={0 if kind in ("group","process") else 1};arcSize=8;html=1;fillColor={fill};strokeColor={c};strokeWidth=2.5;container=1;collapsible=0;recursiveResize=0;'
   if kind in ('process','external') or (self.slug=='speech-evidence-source-comparison' and s['id'] in ('a-recognize','a-omni')):style+='dashed=1;dashPattern=10 7;'
   cell(s['id'],x,y,s['w'],s['h'],'',style,g['id'] if g else '1')
   size=18 if kind=='module' else 20
   txt(s['id']+'-title',16,46 if kind=='store' else 28,s['w']-32,size,s['label'],c,True,s['id'])
   for i,line in enumerate(s['lines']):
    yy=54+i*23 if kind=='module' else 64+i*25+(15 if kind=='store' else 0)
    txt(s['id']+'-text-'+str(i),16,yy,s['w']-32,size,line,c,False,s['id'])
   if kind=='component':cell(s['id']+'-divider',0,45,s['w'],0,'',f'shape=line;strokeColor={c};strokeWidth=1;connectable=0;',s['id'])
  for e in self.edges:
   c=BLUE if e['blue'] else INK
   style=f'endArrow=block;endFill=1;strokeColor={c};strokeWidth=2.5;rounded=0;noEdgeStyle=1;'+('dashed=1;' if e['dash'] else '')
   attrs={'id':e['id'],'edge':'1','parent':'1'}
   for endpoint,point,prefix in [('source',e['points'][0],'exit'),('target',e['points'][-1],'entry')]:
    if e[endpoint]:
     node=shapes[e[endpoint]];attrs[endpoint]=node['id']
     style+=f'{prefix}X={(point[0]-node["x"])/node["w"]};{prefix}Y={(point[1]-node["y"])/node["h"]};{prefix}Perimeter=0;'
   attrs['style']=style
   ce=ET.SubElement(r,'mxCell',**attrs);g=ET.SubElement(ce,'mxGeometry',relative='1',attrib={'as':'geometry'})
   pts=e['points']
   for point,tag in [(pts[0],'sourcePoint'),(pts[-1],'targetPoint')]:ET.SubElement(g,'mxPoint',x=str(point[0]),y=str(point[1]),attrib={'as':tag})
   ar=ET.SubElement(g,'Array',attrib={'as':'points'})
   for x,y in pts[1:-1]:ET.SubElement(ar,'mxPoint',x=str(x),y=str(y))
   if e['label']:
    x,y=e['at'];txt(e['id']+'-label',x,y,620,19,e['label'],c)
  for t in self.texts:
   for i,line in enumerate(t['lines']):txt(t['id']+'-'+str(i),t['x'],t['y']+i*(t['size']+10),W-t['x']-20,t['size'],line,t['color'],t['bold'])
  return ET.tostring(f,encoding='unicode')+'\n'

PAGES=[]
def comparison(slug,title,a,b):
 p=Page(slug+'-comparison',title+'  |  설계 비교','검정: 양안 공통 책임·경로   /   파랑: 양안에서 달라지는 모듈·상태·계약   /   점선 화살표: 보완·무효화·서비스 호출')
 for side,x,label in [('a',48,a),('b',1312,b)]:
  p.text(side+'-title',x+16,161,label,29,bold=True)
  p.group(side+'-boundary',x,187,1200,885,'VIA · 논리 Component / 내부 모듈 확대 · 외부 dependency는 명시')
 return p

DEPENDENCY_STRIPS={
 'request-interpretation-topology':('근거 → 통합 proposal → host 확정','지칭 vR → 업무 vT → 경로 vH → host 확정'),
 'context-acquisition-strategy':('발화 ∥ 기본 준비 → 통합 해석 → 한정 보완','확정 입력 → 조회 계획 → source read → 통합 해석'),
 'agent-state-observation':('progress → reducer → projection → host','hint → query → snapshot 검증 → host'),
 'direct-response-generation':('제안 ∥ 선생성 → content admission → release','분류 → permit → 생성 → content admission → release'),
 'speech-evidence-source':('capture → 독립 ASR → 근거 · Omni는 별도 진행','capture → 공유 Omni native 경로 → 근거'),
 'conversation-context-maintenance':('요청 → cache 검사 / 필요한 범위 재구성 → package','변경 → view 갱신 ∥ 요청 → revision barrier → package'),
 'context-representation-pipeline':('원문 package → 현재 목적에서 의미 해석','source → 사실 view 생성 → 목적별 재사용·해석'),
 'recovery-state-source':('transaction → current records + intent → load','transaction → events + intent → checkpoint / tail replay'),
}
def costs(p,a,b,claim):
 strips=DEPENDENCY_STRIPS[p.slug.removesuffix('-comparison')]
 for i,(side,x,lines) in enumerate([('a',48,a),('b',1312,b)]):
  p.node(side+'-cost',x,1102,1200,190,'선택할 이유와 감수할 비용',*lines,kind='box')
  p.text(side+'-dependency',x+16,1261,strips[i],21,BLUE,True)
 p.text('takeaway',64,1345,claim,26,bold=True)
 PAGES.append(p)

def background(slug,title,quote,uc,rows,pressures,challenge):
 p=Page(slug+'-background',title+'  |  배경',uc+'  ·  아래 사건은 설계를 설명하는 시나리오이며 측정 trace가 아님')
 p.node('scenario',48,142,2464,104,'사용자 상황',quote,kind='box')
 p.text('time-heading',330,294,'시간의 진행 →',23,bold=True)
 times=['① 시작 / 기존 상태','② 변화 / 교차 사건','③ 확인할 근거','④ 사용자에게 필요한 결과']
 for j,t in enumerate(times):p.text('t'+str(j),330+j*540,334,t,24,bold=True)
 for i,(label,events) in enumerate(rows):
  y=380+i*162
  p.node('lane'+str(i),48,y,246,130,label,kind='box')
  for j,(t,l1,l2) in enumerate(events):
   id=f'e{i}{j}';p.node(id,330+j*540,y,490,130,t,l1,l2,kind='box')
   if j:p.edge(f'flow{i}{j}',f'e{i}{j-1}',id,label='',blue=False)
 for i,(title,lines) in enumerate(pressures):
  p.node('pressure'+str(i),48+i*836,925,792,205,title,*lines,kind='box')
 p.node('challenge',48,1170,2464,170,'Architecture Challenge',challenge,kind='box')
 PAGES.append(p)

def interpretation():
 slug='request-interpretation-topology'
 background(slug,'요청 해석 · 서로 의존하는 의미를 어디서 결합할까?',
  '“이 표를 아까 보고서에 넣어줘. 아니, 새 보고서로 만들어줘.”','UC-03 · UC-06 · UC-10 · UC-14',[
  ('사용자 입력', [('최초 지칭','“이 표” + 선택 영역','당시 화면·원문 연결'),('업무 연결','“아까 보고서”','기존 Task 후보 2개'),('사용자 정정','“새 보고서로”','기존 Task 연결 취소'),('기대 결과','같은 표 + 새 목표','새 Task로 위임')]),
  ('근거와 의미', [('대상 후보','표 A / 표 B','시점·범위·용도 확인'),('상호 의존','보고서 용도 → 대상 구분','대상 내용 → 업무 기능'),('영향 범위','목표·Task·Agent 후보 변경','대상도 의존했으면 재검토'),('확정할 묶음','대상·목표·제약·처리 경로','같은 revision으로 일치')]),
  ('VIA 책임', [('입력·근거 확보','원문·후보·불확실성 보존','모델 판단 전 사실'),('의미 제안','모델이 후보 관계 해석','아직 실행 권한 없음'),('정정 처리','영향받은 결과 무효화','늦은 결과 거절'),('업무 admission','host가 근거·현재성 확인','잘못된 Task 전송 방지')])],
  [('결합 판단의 이점',['관련 의미를 함께 조정할 수 있다.','큰 입력·출력과 결합 오류를 감수한다.','한 번의 호출이 무오류를 뜻하지 않는다.']),('단계화의 이점',['좁은 계약·부분 재사용이 가능하다.','중간 정보 손실과 순차 대기가 생긴다.','정정 시 되돌림 경로가 필요하다.']),('구조로 해결할 난점',['중간 의미를 누가 변경할 수 있는가?','앞 판단을 바꾸면 무엇을 폐기하는가?','최종 확정은 어떤 version에 결합되는가?'])],
  '통합 proposal 안에서 관계를 조정할 것인가, versioned 중간 계약과 되돌림을 가진 파이프라인으로 연결할 것인가?')


def acquisition():
 slug='context-acquisition-strategy'
 background(slug,'Context 획득 · 요청을 알려면 자료가, 자료를 고르려면 요청이 필요하다',
 '“아까 그 보고서 어디까지 됐어?”와 “지난주 견적 메일을 요약해줘”는 필요한 source가 다르다.','UC-02 · UC-05 · UC-06 · UC-10',[
 ('입력', [('발화 시작','화면·포인터 당시 상태','최근 대화가 관련될 수 있음'),('말하는 동안','문서·선택이 바뀔 수 있음','최종 목표는 아직 미확정'),('입력 확정','“지난주 견적 메일”','읽을 source·범위가 드러남'),('의미 해석','근거 + 사용자 목표','설명 또는 확인 질문')]),
 ('필요한 자료', [('기본 근거','현재 선택 · 최근 대화','Task 요약 · 연결 metadata'),('추가 근거 후보','파일 / 메일 / 일정 / Task','모두 읽을 수는 없음'),('실제 bounded read','source + filter + page 한도','실패·누락·revision 기록'),('사용 가능한 근거','원문 · typed record','coverage · 현재 권한')]),
 ('설계의 긴장', [('사전 준비','발화와 조회를 겹칠 여지','사용되지 않을 수도 있음'),('계획 선행','선택적 조회로 낭비 감소','계획 자체에 근거가 필요'),('source 대기','API / 파일 / owner read','병렬 query도 결과 대기 필요'),('동일 완료 조건','당시 화면·후보 누락 보존','빠른 잘못된 응답은 실패')])],
 [('입력 전에 할 수 있는 일',['당시 화면 관측은 두 안 모두 필요하다.','의미 기반 자료 읽기와 구분해야 한다.','현재 화면으로 과거 지칭을 복원할 수 없다.']),('응답 경로의 비용',['선행 준비의 적중과 폐기를 함께 본다.','조회 계획·query·해석의 의존이 생긴다.','cache·병렬 읽기는 두 안 모두 허용한다.']),('구조로 해결할 난점',['첫 읽기를 무엇이 trigger하는가?','첫 모델 입력에 어떤 근거가 있는가?','누락·source 실패를 어디서 종료하는가?'])],
 '기본 Context를 발화 중 준비할 것인가, 확정 입력에서 조회 계획을 만든 뒤 필요한 source를 읽을 것인가?')


def observation():
 slug='agent-state-observation'
 background(slug,'Agent 상태 · 원격 업무의 현재 모습을 무엇으로 구성할까?',
 '보고서와 메일 업무가 동시에 진행된다. 사용자는 “보고서 어디까지 됐어?”라고 묻고, 다른 Agent는 승인을 요청한다.','UC-10 · UC-13 · UC-14 · UC-18',[
 ('외부 Agent', [('업무 실행','report run-A / mail run-B','각각 독립 identity'),('교차 event','progress r12 → r14 → r13','승인 question-Q 도착'),('현재 snapshot','phase / revision / artifact','대기 질문·terminal 확인'),('원격의 사실','완료와 취소 접수는 다름','실행 내부는 VIA 밖')]),
 ('연결·도착', [('정상 연결','event 구독 + query API','동일 capability 전제'),('지연·단절','중복 / gap / 재연결','과거 메시지가 늦게 도착'),('확인된 최신성','마지막 확인 시각 표시','필요하면 source 재조회'),('필수 사건 전달','질문·완료·실패 내구 보존','사용자 재질문 없이 안내')]),
 ('사용자 창구', [('같은 Conversation','Task identity 유지','Agent thread 선택 불필요'),('상태 질문','기존 업무만 선택','다른 Task와 혼동 금지'),('알림 구성','현재 사실 + 불확실성','낡은 상태 덮어쓰기 금지'),('동일한 결과','Text 기록 / Voice 차례','승인 응답을 정확한 Q에 결합')])],
 [('로컬 projection 유지',['query 없이 확인한 상태를 읽을 수 있다.','event 의미·순서·gap 보정 책임이 생긴다.','로컬 값이 원격의 지금을 보장하지 않는다.']),('snapshot 중심 구성',['원격이 만든 현재 상태를 활용한다.','API 지연·한도·snapshot 완전성에 의존한다.','cache·query 합치기로 과도한 조회를 줄인다.']),('공통으로 보존할 사실',['질문·승인·완료를 일시 hint로 버리지 않는다.','Task·Execution·command 결합은 VIA 책임이다.','새 snapshot이 확인된 terminal을 지우면 안 된다.'])],
 '일반 progress를 event reducer로 계속 구성할 것인가, 변경 신호와 사용자 조회에 따라 snapshot을 읽어 구성할 것인가?')


def direct():
 slug='direct-response-generation'
 background(slug,'직접 응답 · 먼저 만들면 빠르지만 버릴 답변일 수 있다',
 '“광합성이 뭐야?”는 직접 응답 후보지만, “아까 표의 두 번째는?”은 Context를 사용하는 Core 요청이다.','UC-01 · UC-11 · 공유 Omni 자원',[
 ('현재 입력', [('독립 질문 또는 지칭','같은 Voice 입력 창구','최종 전사·revision 확보'),('경로 판단','history / screen / task 등','dependency가 남으면 Core'),('직접 경로 허용','현재 revision + host 조건','분류는 게시 허가와 구분'),('사용자 전달','첫 유효 문장 + Text','새 발화 때 즉시 중단')]),
 ('생성 자원', [('단일 공유 Omni','VOICE / SEMANTIC session','가중치 ×1, KV는 별도'),('선생성 선택','답변 text/audio 준비','host 검사와 겹칠 여지'),('기각되면 폐기','이미 사용한 계산·KV','buffer와 취소 대기 비용'),('순차 생성 선택','분류 후 필요한 답변 생성','첫 문장까지 순차 경계 추가')]),
 ('정정·예외', [('입력 아직 provisional','확정 전에 재생 금지','handle은 권한이 아님'),('새 발화 시작','옛 output epoch 무효화','미승인 결과 사용 차단'),('backend 취소','다음 safe point에서 정리','즉시 GPU 중단 가정 금지'),('후속 처리','같은 Request로 Core 인계','중복 답변·중복 기록 금지')])],
 [('선생성이 줄일 수 있는 대기',['허용 시점에 유효 문장이 준비되어 있다.','실제 겹친 구간만 이득이다.','Core임을 이미 알면 조기 중지한다.']),('선생성이 늘리는 부담',['버릴 답변도 공유 모델을 점유한다.','KV·audio buffer·취소 처리가 필요하다.','다른 입력·해석의 대기로 번질 수 있다.']),('대안도 감수할 비용',['분류 job과 후행 generation의 두 경계','재입력·허용 record·stale 검사','별도 분류가 정확도를 보장하지 않는다.'])],
 '경로 허용과 답변 생성을 겹칠 것인가, 내용 없는 경로 허용을 먼저 확정한 뒤 필요한 답변만 생성할 것인가?')


def speech():
 slug='speech-evidence-source'
 background(slug,'음성 입력 근거 · 의미 추론 중에도 계속 듣고 당시 화면에 연결해야 한다',
 '이전 요청을 해석하는 동안 사용자가 그래프에서 표로 포인터를 옮기며 “이거, 아니 저걸 넣어줘”라고 말한다.','UC-03 · UC-04 · UC-11 · UC-18',[
 ('발화·화면', [('기존 semantic 실행','이전 자료 설명 중','Omni 계산·KV 사용'),('새 정정 발화','“이거” → “아니 저걸”','화면 A → 화면 B'),('전사 늦게 도착','최종 revision / 대체 구간','도착 시각 ≠ 발화 시각'),('올바른 지칭','표 B / 정정 관계','당시 sample 구간과 연결')]),
 ('필요 근거', [('연속 capture','sample sequence / clock','유한 ring + local stop'),('연속 recognition','partial / final / revision','녹음 대기만으로는 부족'),('시간 관계','span + 오차 + watermark','겹치는 후보 / gap 보존'),('확정 입력','SEALED + evidence 참조','근거 없으면 재지칭 질문')]),
 ('dependency', [('독립 인식 경로','경량 ASR + Omni','두 전사의 충돌 가능'),('통합 인식 경로','Omni native evidence','한 runtime 자원 공유'),('Omni crash','독립 ASR: 전사 유지 가능','native: 전사도 중단'),('사용자에게 알릴 사실','입력 접수와 이해는 다름','미처리·유실을 숨기지 않음')])],
 [('독립성의 가치와 비용',['semantic과 별도로 입력 근거가 진행한다.','별도 weights·CPU·worker가 필요하다.','중복 인식과 전사 충돌을 관리한다.']),('통합의 가치와 비용',['중복 dependency를 줄일 수 있다.','native 시간·revision 계약이 필요하다.','같은 오류·자원·장애에 함께 노출된다.']),('필수 runtime 계약',['source sample 시각·uncertainty·gap','semantic 중에도 recognition 진행','모델 기능은 요구 계약이며 지원 미검증'])],
 'SpeechEvidence를 독립 recognizer에서 생산할 것인가, 동일 Omni의 native stream에 입력 근거까지 맡길 것인가?')


def maintenance():
 slug='conversation-context-maintenance'
 background(slug,'대화 Context · 다음 발화 사이에도 업무·질문·기억은 변한다',
 '보고서와 메일 업무를 오가며 “아까 형식으로 정리해줘”라고 한다. 그 사이 취소·질문·선호 삭제가 발생한다.','UC-06 · UC-10 · UC-15 · UC-17',[
 ('대화', [('첫 요청','보고서 요약 + 표 우선 선호','허용된 기억 사용'),('다른 업무 대화','메일 질문에 답하는 중','보고서 상태 계속 변경'),('기억 변경','“그 선호는 잊어줘”','memory epoch 증가'),('후속 요청','“보고서 결과도 정리해줘”','삭제된 선호 재사용 금지')]),
 ('여러 owner', [('대화·전달 기록','Request Controller','Response Manager'),('업무 상태','Task Manager revision 증가','질문 / 결과 / 취소'),('기억·권한','Context Manager 삭제','Policy Manager 철회'),('필요한 입력 묶음','각 owner의 근거·revision','동시 snapshot으로 위장 금지')]),
 ('파생 Context', [('요청별 조합','필요 범위 read + cache','원문·typed field 보존'),('지속 view 갱신','변경 delta → projector','관심 없는 갱신 비용도 존재'),('lag 또는 누락','dirty / revision gap','read barrier 또는 rebuild'),('확정 전 검증','read set 최신성 확인','view는 운영 원본이 아님')])],
 [('요청 시 구성',['쓰는 범위만 읽고 조합한다.','유효 cache·summary는 재사용한다.','반복 조합 비용이 요청 경로에 남는다.']),('지속 갱신 view',['다음 요청 전에 준비할 수 있다.','변경 전달·적용 위치·회수 책임이 생긴다.','요청이 없어도 메모리·계산을 사용한다.']),('삭제·정정의 어려움',['epoch로 즉시 사용 차단해야 한다.','projector가 늦다고 사용을 허용할 수 없다.','늦은 summary·옛 checkpoint도 차단한다.'])],
 '여러 owner 기록을 요청 시 조합할 것인가, 활성 대화의 working view를 지속 갱신하고 revision barrier 뒤 소비할 것인가?')


def representation():
 slug='context-representation-pipeline'
 background(slug,'자료 표현 · 같은 표를 여러 목적에 쓸 때 무엇을 재사용할까?',
 '“이 표를 설명해줘” 다음에 “방금 수치로 보고서를 만들어줘”라고 한다. 단위·기간·각주도 함께 전달되어야 한다.','UC-02 · UC-03 · UC-05 · UC-07',[
 ('선택된 원문', [('PDF 표 선택','제품 A: 12 / 제품 B: 8','단위: 백만원'),('각주와 범위','* 해외 매출 제외','2025년 4분기 / 연결 기준'),('source 변경','표 revision r7 → r8','값·각주가 바뀔 수 있음'),('근거 보존','값 + 단위 + 범위 + 출처','숫자만 복사하면 의미 손실')]),
 ('소비 목적', [('어느 표인가','지칭·선택 범위 연결','Request Interpreter'),('무엇을 뜻하나','자료와 현재 질문의 해석','필수 사실·불확실성 유지'),('업무로 이어가기','확정 목표 + 허용 근거','Agent 내부 분석은 외부'),('재사용 판단','동일 source revision인가','새 목적의 필드가 충분한가')]),
 ('구조적 긴장', [('원문 중심','선행 사실 추출 없이 소비','자료와 목적을 함께 해석'),('사실 view 선행','구조·출처·누락 추출','별도 job·저장·검증'),('정보 손실','각주 누락이 반복 전파','view 일치 ≠ 사실 정확성'),('필요한 복귀','원문 확인 / view 보완','없으면 질문·실패 명시')])],
 [('원문 중심이 유리한 상황',['일회성·다양한 자료, 목적 의존 해석','parse cache·이전 유효 결과 재사용 가능','모든 소비가 전체 PDF를 다시 읽지는 않는다.']),('사실 view가 유리한 상황',['안정된 자료를 여러 목적에서 반복 소비','자료 추출 책임·출처 구조 집중','첫 추출·schema·무효화 비용을 감수한다.']),('구조로 해결할 난점',['변환 산출물의 범위·권위는 무엇인가?','빠진 각주·불확실성을 어떻게 전달하는가?','source 삭제가 모든 파생 view에 닿는가?'])],
 '원문과 요청을 함께 해석할 것인가, source별 사실·구조·출처 view를 먼저 만들어 여러 소비 경로에 제공할 것인가?')


def recovery():
 slug='recovery-state-source'
 background(slug,'복구 원본 · 다시 켜지는 것과 업무를 올바르게 이어가는 것은 다르다',
 '취소 command 전송 직후 ACK가 사라지고 VIA가 종료된다. 다른 Task의 결과는 저장됐지만 사용자에게 아직 전달되지 않았다.','UC-12 · UC-13 · UC-16 · UC-18',[
 ('업무 제어', [('사용자 취소','Task A / command C17','현재 승인·revision 검사'),('전송 시작','DISPATCHING commit','외부 호출은 transaction 밖'),('ACK 미수신 + crash','Agent가 받았는지 불명','취소 완료라고 단정 금지'),('재연결','같은 command key로 query','미확인은 UNKNOWN 유지')]),
 ('결과 전달', [('Task B 결과 확정','result r9 / publication P3','source-confirmed 완료'),('인계 중단','domain outbox는 남음','UI receipt 아직 없음'),('재시작','원래 대화·질문 관계 복구','새 Task로 중복 생성 금지'),('사용자 결과','Text는 같은 ID upsert','불명 음성 자동 재생 금지')]),
 ('저장·삭제', [('현재 state 또는 이력','owner별 확정 전이','같은 local DB 내구성'),('memory 삭제','epoch·tombstone commit','payload purge 예약'),('옛 checkpoint','삭제 전 데이터 포함 가능','그대로 복원하면 기억 부활'),('운영 가능 상태','삭제 fence 먼저 적용','schema·effect identity 검증')])],
 [('현재 상태 중심 복원',['owner row·pending 기록을 직접 읽는다.','복구에 필요한 관계의 원자성 유지','audit·outbox가 없는 안이 아니다.']),('확정 이력 중심 재구성',['checkpoint + tail로 projection을 만든다.','과거 schema·reducer·삭제 호환 필요','모델 판단·외부 Action은 재실행하지 않는다.']),('두 안 모두 못하는 일',['잃어버린 외부 ACK를 추측할 수 없다.','실제 들은 음성을 로그만으로 증명 못한다.','권위 저장 자체 손상은 별도 복구가 필요하다.'])],
 '운영 상태를 현재 owner 기록에서 복원할 것인가, 확정 domain 이력과 checkpoint를 권위 원본으로 재구성할 것인가?')


class Pane:
 """Expanded component view; named ports and nested editable modules."""
 def __init__(self,p,side):self.p=p;self.s=side;self.x=48 if side=='a' else 1312
 def node(self,id,x,y,w,h,title,*lines,blue=False,kind='module'):
  return self.p.node(self.s+'-'+id,self.x+x,y,w,h,title,*lines,blue=blue,kind=kind)
 def group(self,id,x,y,w,h,title,blue=False,kind='group'):
  return self.p.group(self.s+'-'+id,self.x+x,y,w,h,title,blue,kind)
 def line(self,id,src,dst,points,label='',at=None,blue=False,dash=False):
  pts=[(self.x+x,y) for x,y in points]
  self.p.edges.append(dict(id=self.s+'-edge-'+id,source=self.s+'-'+src,target=self.s+'-'+dst,points=pts,label=label,at=(self.x+at[0],at[1]) if at else None,blue=blue,dash=dash))
 def text(self,id,x,y,text,blue=False,size=18):self.p.text(self.s+'-'+id,self.x+x,y,text,size,BLUE if blue else INK)
 def top(self,a,b,c):
  for i,(title,l1,l2) in enumerate([a,b,c]):self.node('top'+str(i),24+i*410,259,356,108,title,l1,l2,kind='component')
 def groups(self,left,right,bl=False,br=False):
  self.group('left',24,437,550,383,left,bl);self.group('right',650,437,526,383,right,br)
 def module(self,id,side,row,col,title,*lines,blue=False):
  x=(44 if side=='l' else 670)+col*264;y=500+row*190
  return self.node(id,x,y,230,105,title,*lines,blue=blue)
 def supports(self,left,right,bl=False,br=False):
  for id,x,w,data,blue in [('supportL',24,550,left,bl),('supportR',650,526,right,br)]:
   self.node(id,x,908,w,118,*data,blue=blue,kind='component')
 def footer(self,text):self.text('panenote',32,1052,text,size=19)

def rich_interpretation():
 p=comparison('request-interpretation-topology','요청 의미의 결합·중간 계약·정정 경로','방안 1  통합 SemanticProposal','방안 2  versioned 단계 결과 + 제한된 되돌림')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Interaction Manager','InputFinal · evidence refs','InputStarted → hold'),('Policy Manager','scope · policy revision','사용 직전 현재 권한'),('Context Manager','후보 / 원문 / receipt','bounded read · source revision'))
  q.groups('Request Controller','Request Interpreter',True,True)
  q.module('workspace','l',0,0,'Turn Workspace','input · context revision','candidate dependencies')
  q.module('orchestrate','l',0,1,'통합 job 제어' if s=='a' else 'Stage Coordinator','read / deadline' if s=='a' else 'stage / retry budget','proposal revision' if s=='a' else 'expected input versions',blue=True)
  q.module('invalidate','l',1,0,'정정·hold 처리','영향 read set 무효화','미전송 admission 보류')
  q.module('commit','l',1,1,'의미 확정·admission','field / coverage / policy','Semantic Commit')
  if s=='a':
   q.module('prompt','r',0,0,'통합 입력 조립','지칭·목표·Task·경로','원문·후보·제약',blue=True)
   q.module('job','r',0,1,'통합 semantic job','서로 의존하는 field','동일 proposal에서 조정',blue=True)
   q.module('parse','r',1,0,'결과·근거 해석','경쟁 후보 / 미해결','Next Evidence Request',blue=True)
   q.module('intermediate','r',1,1,'SemanticProposal','field origin / deps','임시 결과 · 권한 없음',blue=True)
   q.line('unify','prompt','job',[(900,552),(934,552)],blue=True)
   q.line('job-result','job','intermediate',[(1049,605),(1049,690)],'구조화 결과',(1060,653),True)
   q.line('parse','intermediate','parse',[(934,742),(900,742)],blue=True)
  else:
   q.module('prompt','r',0,0,'1. 지칭·근거 해석','ReferentSet vR','후보·불확실성·원문',blue=True)
   q.module('job','r',0,1,'2. 목표·Task 연결','GoalTaskSet vT','vR + Task 근거 의존',blue=True)
   q.module('parse','r',1,0,'3. 처리 경로 제안','HandlingProposal vH','vR / vT / capability',blue=True)
   q.module('intermediate','r',1,1,'Correction Request','원인 field + evidence','앞 stage 새 version 요청',blue=True)
   q.line('stage1','prompt','job',[(900,552),(934,552)],blue=True)
   q.line('stage2','job','parse',[(1049,605),(1049,644),(785,644),(785,690)],'vT → 경로 판단',(839,633),True)
   q.line('correction','intermediate','prompt',[(1164,742),(1188,742),(1188,478),(785,478),(785,500)],'정정 / 후속 무효화',(941,485),True,True)
   q.line('issue','parse','intermediate',[(900,742),(934,742)],blue=True,dash=True)
  q.supports(('State Store','Conversation · Request · Semantic Commit','중간 결과는 Request attempt 임시 상태; crash 시 재해석'),('Model Access','단일 Omni · role/session/KV 분리','모든 stage 호출·취소·재처리 비용 포함'))
  q.line('input','top0','workspace',[(202,367),(202,407),(159,407),(159,500)],'① 입력',(213,407))
  q.line('read','orchestrate','top2',[(423,500),(423,397),(1022,397),(1022,367)],'② read / receipt (host 경유)',(652,389),True)
  q.line('run','orchestrate','prompt',[(538,552),(670,552)],'③ job',(583,542),True)
  q.line('evidence','workspace','orchestrate',[(274,552),(308,552)])
  q.line('hold','workspace','invalidate',[(159,605),(159,690)],'revision',(171,654))
  q.line('guard','invalidate','commit',[(274,742),(308,742)])
  q.line('proposal','parse','commit',[(670,742),(538,742)],'④ proposal',(558,731),True)
  q.line('model','job','supportR',[(1049,605),(1200,605),(1200,964),(1176,964)],'InferenceJob',(1058,870),False,True)
  q.line('persist','commit','supportL',[(423,795),(423,908)],'⑤ durable commit',(434,875),False,True)
  q.line('policy','top1','commit',[(612,367),(612,856),(557,856),(557,770),(538,770)],'policy',(552,846),False,True)
  q.footer('확정 뒤 Task Manager / Response Manager 인계 · 모델 결과 자체에는 dispatch·게시 권한 없음')
 costs(p,['선택 근거: 지칭·목표·Task가 서로 바뀌는 요청을 한 의미 묶음에서 조정','대가: 큰 입력·출력과 결합 오류; 관련 근거 변경 시 재해석','반증: 부분 재사용보다 전면 조정 부담이 반복적으로 큼'],['선택 근거: 안정된 중간 계약을 진단·재사용하고 영향 stage부터 처리','대가: 추가 호출·중간 정보 손실·version propagation·되돌림','반증: 매번 모든 stage를 다시 돌거나 마지막 통합 판단으로 수렴'],'핵심 차이: 의미를 한 proposal 안에서 함께 조정하는가, 앞 stage의 새 version과 후속 무효화를 거쳐 바꾸는가.')

def rich_acquisition():
 p=comparison('context-acquisition-strategy','Context 획득의 선행 작업·조회 계약·대기 구조','방안 1  InputStarted 기본 준비 → 통합 해석','방안 2  InputFinal 계획 → 선택적 조회 → 통합 해석')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Interaction Manager','당시 화면·음성·선택 capture','Started / Final / revision'),('Policy Manager','source / recipient / purpose','현재 scope · 읽기 허용'),('Request Interpreter','통합 해석 · 추가 근거 제안' if s=='a' else 'ReadPlan · 후행 통합 해석','모델 제안은 host를 경유'))
  q.groups('Request Controller','Context Manager',True,True)
  q.module('trigger','l',0,0,'기본 준비 trigger' if s=='a' else 'ReadPlan 승인','InputStarted / 관련 변경' if s=='a' else 'Final + 최소 식별 근거','현재 scope / 예산',blue=True)
  q.module('workspace','l',0,1,'Provisional Context' if s=='a' else 'Query Plan Registry','기본 package revision' if s=='a' else 'source / filter / 한도','Final에서 유효 범위 선택' if s=='a' else 'query dependency / 상태',blue=True)
  q.module('join','l',1,0,'해석·보완 제어','모델 제안 → 한정 read','deadline 내 재해석',blue=True)
  q.module('commit','l',1,1,'근거·의미 확정','read set / coverage 검사','Commit / 질문 / 실패')
  q.module('select','r',0,0,'기본 source 선택' if s=='a' else 'Plan Dispatcher','최근 대화·Task·metadata' if s=='a' else '승인 query만 fan-out','cache key / source rev',blue=True)
  q.module('read','r',0,1,'Read Adapters','owner / file / mail 等','독립 query 병렬 실행')
  q.module('receipt','r',1,0,'Receipt / Package','반환·누락·실패·범위','원문 / typed 근거')
  q.module('cache','r',1,1,'Revision Cache','기본 준비·추가 읽기' if s=='a' else '계획별 선택적 재사용','dirty / invalid / expiry',blue=True)
  q.supports(('Owner / source ports','Request Controller · Task Manager · Response Manager','자료 source: 허용된 file / mail / calendar / browser'),('Model Access','동일 Omni · 동일 최종 통합 해석','plan·추가 해석·KV·입력 중 작업 비용 모두 포함'))
  q.line('input','top0','trigger',[(202,367),(202,405),(159,405),(159,500)],'① Started' if s=='a' else '① Final',(217,405),True)
  q.line('plan','trigger','top2',[(159,500),(159,418),(1022,418),(1022,367)],'② 기본 근거 후 해석' if s=='a' else '② 최소 입력 → ReadPlan 반환',(650,411),True)
  q.line('state','trigger','workspace',[(274,552),(308,552)],blue=True)
  q.line('dispatch','workspace','select',[(538,552),(670,552)],'③ read',(578,539),True)
  q.line('fanout','select','read',[(900,552),(934,552)])
  q.line('source','read','supportL',[(1049,605),(1200,605),(1200,866),(300,866),(300,908)],'④ bounded query / source revision',(665,852))
  q.line('cache','read','cache',[(1049,605),(1049,690)],'result',(1064,650))
  q.line('package','cache','receipt',[(934,742),(900,742)])
  q.line('join','receipt','join',[(670,770),(600,770),(600,838),(159,838),(159,795)],'⑤ package → host → 해석',(210,829),True)
  q.line('final','join','commit',[(274,742),(308,742)])
  q.line('policy','top1','trigger',[(612,367),(612,455),(286,455),(286,527),(274,527)],dash=True)
  q.line('inference','top2','supportR',[(1022,367),(1188,367),(1188,893),(914,893),(914,908)],'model job',(1041,884),False,True)
  q.footer('capture는 두 안에서 계속된다 · 조회 계획은 bounded read이며 업무 planning·외부 Action이 아님')
 costs(p,['선택 근거: 반복되는 최근 Context를 발화와 겹쳐 준비','대가: 실제로 쓰지 않은 읽기·cache와 변경 갱신','반증: 기본 준비의 낭비·경합이 중첩 이득보다 큼'],['선택 근거: source가 다양하고 조회가 비쌀 때 필요한 범위만 선택','대가: 계획 → query → 해석의 순차 의존·계획 오류','반증: 계획에도 기본 Context 전체가 필요해 같은 구조로 수렴'],'핵심 차이: Context Manager의 존재가 아니라 첫 read의 trigger·첫 모델 입력·중간 query 상태가 달라진다.')

def rich_observation():
 p=comparison('agent-state-observation','Agent 상태의 수신·조정·진행 view 구성','방안 1  일반 progress event 적용','방안 2  일반 progress는 hint / snapshot 교체')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Downstream Agent','event + versioned snapshot','같은 capability profile'),('Request Controller','사용자 상태 조회 / 최신성','질문 binding · 게시 admission'),('Response Manager','Text / Voice 상태·결과 전달','publication ID 중복 방지'))
  q.groups('Agent Gateway','Task Manager',True,True)
  q.module('ingress','l',0,0,'Event Adapter','identity / source revision','inbox durable 수신')
  q.module('query','l',0,1,'Gap Query' if s=='a' else 'Query Coordinator','gap / 명시 최신 조회' if s=='a' else 'hint / user / poll trigger','source snapshot 확인' if s=='a' else 'single-flight / 유한 retry',blue=True)
  q.module('critical','l',1,0,'필수 사건 inbox','question / approval','terminal / failure 보존')
  q.module('progress','l',1,1,'Ordered Progress' if s=='a' else 'Dirty Hint Registry','중복·순서·cursor' if s=='a' else 'execution별 hint 합치기','유효 event 전달' if s=='a' else 'query demand 유지',blue=True)
  q.module('binding','r',0,0,'Execution Binding','Task / run / command','known terminal 단조성')
  q.module('reduce','r',0,1,'Event Reducer' if s=='a' else 'Snapshot Validator','projection + cursor CAS' if s=='a' else 'source rev / scope 검사','gap → reconcile' if s=='a' else '늦은 응답 교체 금지',blue=True)
  q.module('criticalbind','r',1,0,'질문·결과 결합','question / artifact version','상태 query와 독립 보존')
  q.module('view','r',1,1,'Progress Projection' if s=='a' else 'Snapshot View','phase / freshness / rev','확인됨·재조회·불명',blue=True)
  q.supports(('State Store','공통: Task·command·질문·terminal·publication intent','event cursor · gap · progress projection' if s=='a' else 'query identity · dirty · snapshot'),('상태 전달 계약','TaskUpdate(task, run, source_revision, observed_at)','사용자 최신성 요구 동일 · 확인 불가를 성공으로 만들지 않음'))
  q.line('event','top0','ingress',[(202,367),(202,407),(159,407),(159,500)],'① event',(216,407))
  q.line('query-source','query','top0',[(423,500),(423,396),(380,396),(380,315)],'② query / snapshot',(426,414),True)
  q.line('critical-route','ingress','critical',[(159,605),(159,690)],'필수 사건',(169,653))
  q.line('progress-route','ingress','progress',[(274,577),(286,577),(286,660),(423,660),(423,690)],'일반 progress',(292,649),True)
  if s=='a':
   q.line('progress-target','progress','reduce',[(538,742),(612,742),(612,481),(1049,481),(1049,500)],'③ event 적용',(934,472),True)
   q.line('gap-query','reduce','query',[(1049,605),(1049,633),(423,633),(423,605)],'gap / reconcile',(584,624),True,True)
  else:
   q.line('hint-query','progress','query',[(423,690),(423,605)],'dirty → query',(430,643),True)
   q.line('query-result','query','reduce',[(538,552),(612,552),(612,481),(1049,481),(1049,500)],'③ snapshot 검증',(934,472),True)
  q.line('query-demand','binding','query',[(785,605),(785,617),(556,617),(556,579),(538,579)],'최신성 요구',(623,611),False,True)
  q.line('critical-bind','critical','criticalbind',[(159,795),(159,832),(637,832),(637,742),(670,742)],'필수 질문·결과',(318,822))
  q.line('binding','binding','reduce',[(900,552),(934,552)])
  q.line('view','reduce','view',[(1049,605),(1049,690)],'④ 현재 view',(1060,651),True)
  q.line('persist','critical','supportL',[(159,795),(159,908)],'내구 기록',(170,875),False,True)
  q.line('deliver','view','top1',[(1164,742),(1193,742),(1193,393),(612,393),(612,367)],'⑤ 현재 상태 → host',(899,383))
  q.line('publish','top1','top2',[(790,315),(844,315)])
  q.line('read','top1','binding',[(612,367),(612,414),(785,414),(785,500)],'사용자 read',(674,402))
  q.footer('일반 progress만 비교 · 질문·승인·완료를 hint로 폐기하지 않음 · snapshot은 과거 terminal을 지우지 못함')
 costs(p,['선택 근거: event가 충실하고 상태 조회가 잦으면 로컬 view 활용','대가: 전이 schema·event reducer·cursor·gap 복구','반증: 대부분 query로 보정하여 중복 유지 비용만 증가'],['선택 근거: 충실하고 저렴한 snapshot으로 전이 재구성 축소','대가: API 왕복·호출 한도·query 병합·freshness 관리','반증: 필수 알림 기한·snapshot 완전성을 충족하지 못함'],'핵심 차이: 외부 상태를 받아 현재 view를 만드는 방식이다. 전체 VIA event sourcing이나 Task writer 변경과 구분한다.')

def rich_direct():
 p=comparison('direct-response-generation','직접 응답의 job 그래프·보류 상태·게시 경계','방안 1  제안과 답변 선생성 / host 허용 후 release','방안 2  분류 → route permit → 답변 생성')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Interaction Manager','capture / InputFinal / stop','현재 Turn · ASR final'),('Request Controller','좁은 direct 검사 · Core 인계','입력·질문·policy·route owner'),('Policy Manager','현재 권한 / output epoch','과거 Context는 direct에 미제공'))
  q.groups('Model Access · 공유 Omni 실행 계약','Interaction Manager · generation 수명',True,True)
  q.module('classify','l',0,0,'VoiceProposal Job' if s=='a' else 'Classification Job','current Turn only','dependency flags / echo',blue=True)
  q.module('generate','l',0,1,'Speculative Generation' if s=='a' else 'Permitted Generation','제안과 답변 생성 중첩' if s=='a' else '유효 permit 뒤 시작','Text/audio segment',blue=True)
  q.module('kv','l',1,0,'Shared Runtime','Omni weights ×1','VOICE / SEMANTIC 격리')
  q.module('cancel','l',1,1,'Job Cancel / Reclaim','safe point cancel','KV 반환 확인 · deadline')
  q.module('handle','r',0,0,'Held Generation' if s=='a' else 'Route Permit Binding','request / generation ID' if s=='a' else 'request / input / policy','미승인 Text/audio buffer' if s=='a' else '허용 전 답변 없음',blue=True)
  q.module('fence','r',0,1,'Input / Output Fence','늦은 결과 사용 차단','새 발화 → local stop')
  q.module('release','r',1,0,'Release / Discard','held buffer 선택' if s=='a' else '생성 결과 수신','유효 content / epoch 검사',blue=True)
  q.module('device','r',1,1,'Channel I/O','Text 표시 / Voice 재생','실제 receipt / 중단')
  q.supports(('Response Manager','Request Controller admission → content·epoch 검증','내구 publication intent → release / cancel'),('State Store','route owner / publication / delivery receipt','content 포함 admission · 보류 handle' if s=='a' else 'route permit · 후행 content admission'),False,True)
  q.line('input','top0','classify',[(202,367),(202,407),(159,407),(159,500)],'① 입력',(218,407))
  q.line('proposal','classify','top1',[(159,500),(159,416),(612,416),(612,367)],'② proposal → host',(298,404),True)
  if s=='a':q.line('gen','top0','generate',[(202,367),(202,392),(423,392),(423,500)],'동일 VOICE 작업의 선생성',(286,382),True)
  if s=='b':
   q.line('permit','top1','generate',[(612,367),(612,472),(423,472),(423,500)],'③ permit 뒤 job',(327,461),True)
  q.line('buffer','generate','handle',[(538,552),(670,552)],'④ handle',(570,539),True)
  q.line('state','handle','fence',[(900,552),(934,552)])
  q.line('ready','handle','release',[(785,605),(785,690)],'held / ready',(796,649),True)
  q.line('audio','release','device',[(900,742),(934,742)])
  q.line('cancel','cancel','generate',[(423,690),(423,605)],'취소',(433,650),False,True)
  q.line('runtime','kv','cancel',[(274,742),(308,742)])
  q.line('admission','top1','supportL',[(612,367),(612,865),(299,865),(299,908)],'⑤ admission / 기각',(320,854))
  q.line('release-command','supportL','release',[(574,963),(629,963),(629,866),(785,866),(785,795)],'⑥ release / cancel',(680,855))
  q.line('receipt','device','supportR',[(1049,795),(1049,908)],'⑦ receipt',(1060,875),False,True)
  q.line('epoch','top2','fence',[(1022,367),(1022,410),(1049,410),(1049,500)],'현재 epoch',(1061,409),False,True)
  q.footer('기각/unknown이면 같은 Request의 Core 경로 · 생성 완료와 실제 사용자 전달은 별도 상태')
 costs(p,['선택 근거: 허용 순간 유효 문장이 준비되면 생성 대기를 중첩','대가: 기각된 답변 계산·KV·buffer·cancel 지연','반증: 폐기·공유 경합이 중첩 이득을 소모'],['선택 근거: Core 요청에 불필요한 직접 답변을 만들지 않음','대가: 분류·permit·generation 순차 경계와 재입력','반증: 분류 자체가 답변만큼 비싸거나 직접 경로 대기가 증가'],'핵심 차이: host 허용 전 생성된 content의 존재와 job 의존성. 어느 안도 승인 전 음성을 재생하지 않는다.')

def rich_speech():
 p=comparison('speech-evidence-source','음성 근거의 process·dependency·clock·확정 경로','방안 1  독립 ASR worker + 공유 Omni','방안 2  공유 Omni의 native evidence stream')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Interaction Manager','Voice: AEC · capture · local stop','sample sequence · bounded ring'),('Interaction Manager','UI: 화면·선택 timeline','capture time / clock uncertainty'),('Request Interpreter','기존 semantic job 진행 중','동일 Omni · 다른 session'))
  q.group('left',24,437,550,383,'모델 실행 · Model Access adapter',True)
  q.group('right',650,437,526,383,'Interaction Manager · 입력 근거 결합',True)
  if s=='a':
   q.node('recognize',44,500,230,130,'Speech Input Worker','Streaming ASR','독립 CPU / weights','partial / final / span',blue=True,kind='module')
   q.node('omni',291,500,265,130,'Omni Runtime','Shared Inference Service','weights ×1 / session 격리','VOICE + SEMANTIC',blue=True,kind='module')
  else:
   q.group('omni-boundary',40,487,510,181,'Shared Inference Service',True,kind='process')
   q.node('recognize',56,541,218,108,'Native Evidence Path','transcript / span / gap','semantic 중에도 진행',blue=True)
   q.node('omni',308,541,218,108,'VOICE / SEMANTIC','Omni weights ×1','session / KV 격리',blue=True)
  q.module('adapter','l',1,0,'Model Access','ASR + Omni adapter' if s=='a' else 'native evidence adapter','source build / incarnation',blue=True)
  q.module('scheduling','l',1,1,'자원 admission','독립 입력 + Omni 예약' if s=='a' else '공유 runtime 입력 예약','queue / gap / 유한 backlog',blue=True)
  q.module('normalize','r',0,0,'Evidence Normalizer','ASR canonical revision' if s=='a' else 'native canonical revision','sample span / 불확실성',blue=True)
  q.module('timeline','r',0,1,'Timeline Join','screen / pointer interval','producer watermark / gap')
  q.module('conflict','r',1,0,'Transcript Conflict' if s=='a' else 'Evidence Revision','Omni echo와 충돌 보존' if s=='a' else '정정·대체 구간 보존','원음 한정 확인 / 질문' if s=='a' else '자기 echo는 독립 검증 아님',blue=True)
  q.module('seal','r',1,1,'입력 근거 묶음','final / clock / gap','현재 화면 대체 금지')
  q.supports(('Request Controller','SEALED 입력 → 해석 · 늦은 정정은 admission 무효화','핵심 불일치·gap이면 한정 재확인 또는 clarification'),('Omni crash 시 기능','a: capture·ASR·stop 유지, 의미·음성 생성 중단' if s=='a' else 'b: capture·stop 유지, 인식·의미·음성 생성 중단','미처리·overflow 명시 · 입력 접수와 이해 완료 구별'),False,True)
  q.line('audio','top0','recognize',[(202,367),(202,407),(159,407),(159,500 if s=='a' else 541)],'① audio',(215,407),True)
  q.line('native','recognize','normalize',[(159,630 if s=='a' else 649),(159,661),(588,661),(588,552),(670,552)],'② SpeechEvidence',(283,653),True)
  if s=='a':
   q.line('voice-audio','top0','omni',[(380,315),(402,315),(402,480),(423,480),(423,500)],'audio fan-out',(427,481),True)
   q.line('echo','omni','conflict',[(556,575),(580,575),(580,679),(785,679),(785,690)],'Omni echo / conflict',(680,670),True,True)
  q.line('screen','top1','timeline',[(612,367),(612,414),(1049,414),(1049,500)],'시점별 화면 근거',(821,402))
  q.line('semantic','top2','omni',[(1022,367),(1022,389),(423,389),(423,500 if s=='a' else 541)],'semantic job',(669,381),False,True)
  q.line('adapter-control','adapter','recognize',[(44,742),(32,742),(32,566),(44 if s=='a' else 56,566)],blue=True,dash=True)
  q.line('schedule-control','scheduling','omni',[(538,742),(563,742),(563,596),(556 if s=='a' else 526,596)],blue=True,dash=True)
  q.line('normalize','normalize','timeline',[(900,552),(934,552)])
  q.line('conflict','normalize','conflict',[(785,605),(785,690)],'revision / echo',(791,656),True)
  q.line('seal','timeline','seal',[(1049,605),(1049,690)],'watermark',(1054,655))
  q.line('combine','conflict','seal',[(900,742),(934,742)])
  q.line('input-record','seal','supportL',[(1049,795),(1049,866),(299,866),(299,908)],'③ final + evidence → host 입력 확정',(424,855))
  q.footer('모델 내부 학습은 외부 · 두 안 모두 source-time·revision·동시 recognition 필수 · 실제 native 지원은 미검증')
 costs(p,['선택 근거: semantic 부하·Omni 재시작에서 입력 근거 진행을 분리','대가: ASR weights·CPU·worker·중복 인식·전사 충돌','반증: 독립 인식의 효용보다 전체 자원·오류 조정 부담이 큼'],['선택 근거: native 계약이 충족되면 중복 recognizer 제거 가능','대가: 인식까지 공유 계산·runtime 장애에 결합','반증: semantic 완료를 기다리거나 source-time·gap을 제공 못함'],'핵심 차이: dependency와 근거 생산 경로가 바뀐다. capture만 계속되는 상태를 동시 인식 성공으로 세지 않는다.')

def rich_maintenance():
 p=comparison('conversation-context-maintenance','대화 working view의 갱신·삭제·현재성 검사','방안 1  요청 시 구성 / 유효 cache 재사용','방안 2  변경마다 증분 view / revision barrier')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Request Controller','Conversation / 질문 / Request','Context 요청·required revisions'),('Task Manager','확인된 Task / 결과 / 질문','versioned owner read port'),('Response Manager','실제 게시 Text / audible 범위','versioned delivery read port'))
  q.groups('Context Manager','State Store · Context Manager 소유 파생 상태',True,True)
  q.module('ingress','l',0,0,'Request Assembler' if s=='a' else 'Delta Consumer','필요 owner / 범위 선택' if s=='a' else 'owner event / cursor 검사','cache 현재성 확인' if s=='a' else 'gap → dirty / snapshot',blue=True)
  q.module('update','l',0,1,'Partial Rebuild' if s=='a' else 'Working View Projector','사용 범위만 read·조합' if s=='a' else '활성 Conversation만 갱신','원문 / typed / summary',blue=True)
  q.module('read','l',1,0,'Cache Validator' if s=='a' else 'Revision Read Barrier','source rev / epoch 확인','미충족 → 재조회' if s=='a' else 'lag → catch-up / rebuild',blue=True)
  q.module('package','l',1,1,'Evidence Package','selected fields + read set','요약만으로 대상 확정 금지')
  q.module('view','r',0,0,'Revision Cache' if s=='a' else 'Conversation View','source IDs / revisions','재구성 가능한 파생 입력',blue=True)
  q.module('cursor','r',0,1,'Dependency Metadata' if s=='a' else 'Applied Revision Vector','dirty / expiry / source key' if s=='a' else 'owner별 적용 위치·누락','권한·memory epoch',blue=True)
  q.module('delete','r',1,0,'Deletion / Use Fence','tombstone / epoch','사용 즉시 차단 → purge')
  q.module('bound','r',1,1,'Lifetime / Rebuild','cache LRU + TTL' if s=='a' else '활성 view 회수·checkpoint','원본 owner 상태는 별도',blue=True)
  q.supports(('Request Controller','package 수신 → 해석 → 최종 read-set 관련 revision 재검증','view의 최신성 검사 뒤에도 새 변경은 가능; commit 검사 유지'),('Model Access','같은 Omni · 유휴 summary 작업 · 임시 KV','typed 변경은 즉시 반영 · 늦은 summary는 CAS 폐기'))
  q.line('req','top0','ingress',[(202,367),(202,409),(159,409),(159,500)],'① 요청' if s=='a' else '① owner 변경',(215,408),True)
  q.line('owner','top1','update',[(612,367),(612,409),(423,409),(423,500)],'versioned read' if s=='a' else 'delta / snapshot',(626,405),True)
  q.line('delivery','top2','cursor',[(1022,367),(1022,413),(1049,413),(1049,500)],'delivery rev',(1057,409))
  q.line('assemble','ingress','update',[(274,552),(308,552)],blue=True)
  q.line('materialize','update','view',[(538,552),(670,552)],'② 유지',(585,540),True)
  q.line('vector','view','cursor',[(900,552),(934,552)],blue=True)
  q.line('read','ingress','read',[(159,605),(159,690)],'③ 요청 시 확인',(169,654),True)
  q.line('fence','delete','read',[(670,742),(600,742),(600,656),(159,656),(159,690)],'삭제 epoch: projector 지연과 무관하게 차단',(295,643),False,True)
  q.line('emit','read','package',[(274,742),(308,742)])
  q.line('result','package','supportL',[(423,795),(423,908)],'④ 근거·read set',(433,875))
  q.line('current-view','view','read',[(785,605),(785,625),(286,625),(286,720),(274,720)],'revision / cache read',(420,616),True,True)
  q.line('summary','bound','supportR',[(1049,795),(1049,908)],'유휴 작업 / 재구성',(1059,875),False,True)
  q.footer('owner별 revision은 전역 원자 snapshot이 아님 · 삭제된 선호를 옛 summary·KV·checkpoint에서 복원 금지')
 costs(p,['선택 근거: 관심 업무가 자주 바뀌거나 요청이 뜸하면 필요한 범위만 구성','대가: 반복 요청에서 owner 조회·조합이 대기 경로에 남음','반증: 같은 조합 비용이 반복되고 cache로 충분히 줄지 않음'],['선택 근거: 같은 대화·업무를 자주 읽으면 준비를 앞당길 수 있음','대가: delta 전달·cursor·view 메모리·미사용 갱신·gap rebuild','반증: lag 때문에 매번 rebuild하거나 기존 cache와 동일 계약으로 수렴'],'핵심 차이: 원본 저장 방식이 아니라 파생 입력의 정상 유지 경로다. 삭제·원본 소유권·최종 의미 확정은 공통이다.')

def rich_representation():
 p=comparison('context-representation-pipeline','자료의 변환·출처·재사용·소비 계약','방안 1  원문 중심 package / 소비 시 의미 해석','방안 2  source fact view 선행 생성 / 목적별 소비')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Source / parse cache','같은 identity·revision·선택 범위','원문 / 이미지 / typed field'),('Request Controller','source scope·소비·보완 제어','의미 확정 / 업무·게시 admission'),('Policy Manager','접근·보관·recipient 허용','변경·철회 시 사용 차단'))
  q.groups('Context Manager','소비 Component · 공통 책임',True,False)
  q.module('read','l',0,0,'Evidence Reader','원문·parse·typed record','coverage / receipt 보존')
  q.module('transform','l',0,1,'Package Assembler' if s=='a' else 'Materialization Job','필요 원문·출처 선택' if s=='a' else '구조·사실·각주 추출','기존 summary는 보조' if s=='a' else 'parser + 필요 시 Omni',blue=True)
  q.module('view','l',1,0,'Raw Evidence Package' if s=='a' else 'Versioned FactView','원문·이미지·typed data' if s=='a' else 'value / unit / range','source revision / refs' if s=='a' else 'provenance / omissions',blue=True)
  q.module('repair','l',1,1,'Evidence Completion' if s=='a' else 'View Repair / 원문 복귀','부족 범위 추가 읽기','누락·불확실성 유지',blue=True)
  q.module('interpret','r',0,0,'Request Interpreter','자료 + 요청 의미 해석','Task·handling 후보 제안')
  q.module('respond','r',0,1,'Response Manager','허용 근거로 설명 구성','중요 수치·실패 보존')
  q.module('task','r',1,0,'Task Manager','확정 목표·제약·자료','업무·command 구성')
  q.module('gateway','r',1,1,'Agent Gateway','허용 자료·참조 제공','업무 reasoning은 외부')
  q.supports(('State Store','원문 / parse cache · source revision' if s=='a' else 'FactView · 변환 version · source dependency','source 삭제 → 관련 payload·파생 view 무효화·purge'),('Model Access','소비 시 원문 해석 · 선행 추출 job 없음' if s=='a' else '선행 추출 job·KV·취소 비용 추가','같은 Omni ×1 · 각 소비 작업의 비용도 포함'),True,True)
  q.line('source','top0','read',[(202,367),(202,405),(159,405),(159,500)],'① 확보',(216,405))
  q.line('transform','read','transform',[(274,552),(308,552)],blue=True)
  q.line('view','transform','view',[(423,605),(423,652),(159,652),(159,690)],'② 소비 근거 구성',(201,642),True)
  q.line('storage','view','supportL',[(159,795),(159,908)],'source key / 재사용',(171,875),True,True)
  q.line('host','view','top1',[(44,742),(12,742),(12,416),(612,416),(612,367)],'③ package / view → host',(299,406),True)
  q.line('interpret','top1','interpret',[(612,367),(612,473),(785,473),(785,500)],'④ 해석 호출',(801,494))
  q.line('respond','top1','respond',[(612,367),(612,390),(1194,390),(1194,482),(1049,482),(1049,500)],'⑤ 확정 후 구성',(935,409))
  q.line('task','top1','task',[(612,367),(612,853),(785,853),(785,795)],'⑤ 확정 후 업무',(642,841))
  q.line('gateway','task','gateway',[(900,742),(934,742)])
  q.line('repair','repair','read',[(423,690),(423,672),(30,672),(30,553),(44,553)],'보완 / 원문 복귀',(46,663),True,True)
  if s=='a':q.line('model','interpret','supportR',[(785,605),(785,646),(915,646),(915,876),(914,876),(914,908)],'소비 시 model job',(682,865),True,True)
  else:q.line('model','transform','supportR',[(538,580),(589,580),(589,876),(914,876),(914,908)],'선행 extraction job',(682,865),True,True)
  q.footer('소비자가 source view를 진실로 확정하지 않음 · 최종 요청 의미는 host 확정 · Action·업무 계획은 Agent 책임')
 costs(p,['선택 근거: 일회성·다양한 자료를 현재 목적과 함께 해석','대가: 반복 자료에서 구조 해석·입력 비용이 반복될 수 있음','반증: 유효 cache를 써도 동일 추출·불일치가 반복됨'],['선택 근거: 동일 source 구조·출처를 여러 소비 목적에서 재사용','대가: 추출 job·schema·dependency·첫 사용 대기·오류 전파','반증: 새 목적마다 view를 확장하거나 항상 전체 원문으로 복귀'],'핵심 차이: 자료를 소비하기 전에 별도 사실 산출물·재사용 계약을 만드는가. 보기 좋은 JSON만 만드는 차이는 제외한다.')

def rich_recovery():
 p=comparison('recovery-state-source','복구 원본의 commit·load/replay·외부 효과 조정','방안 1  현재 owner 상태 + 미완료 원장','방안 2  확정 event batch + checkpoint / tail')
 for s in ('a','b'):
  q=Pane(p,s)
  q.top(('Request Controller','Request·질문·admission 변경','현재 revision / policy 검사'),('Task Manager','Task·command·effect identity','expected revision 검사'),('Response Manager','publication intent·실제 receipt','DELIVERY_UNKNOWN 보존'))
  q.groups('State Store · 같은 embedded transactional DB','복구 모듈 · 각 owner가 자기 상태 재구성',True,True)
  q.module('uow','l',0,0,'Owner Unit of Work','검증된 변경 집합 결합','외부 호출은 transaction 밖')
  q.module('authority','l',0,1,'Current Records' if s=='a' else 'Domain Event Batch','row / revision / 관계' if s=='a' else 'event schema / log pos','운영 상태의 권위 원본',blue=True)
  q.module('effects','l',1,0,'Effect / Pending Ledger','inbox / command outbox','publication ID / attempt')
  q.module('projection','l',1,1,'Pending Index' if s=='a' else 'Checkpoint / Projection','미완료·전달·질문 조회' if s=='a' else '적용 위치 / schema version','audit는 임의 덮어쓰기 불가' if s=='a' else '빠른 현재 읽기 · 재생성 가능',blue=True)
  q.module('fence','r',0,0,'Incarnation / Use Fence','새 전송 차단 / schema','현재 deletion epoch 먼저')
  q.module('load','r',0,1,'State Loader' if s=='a' else 'Checkpoint / Tail','row migration·무결성' if s=='a' else 'version reader·reducer','owner state load' if s=='a' else 'side-effect 없는 replay',blue=True)
  q.module('reconcile','r',1,0,'Effect Reconciliation','전송 시작 뒤 불명','동일 command key 조회')
  q.module('restore','r',1,1,'Relation Restoration','질문·결과·publication','복구 scope admission 개방')
  q.supports(('Agent Gateway','source query / command-key 확인 / capability별 처리','UNKNOWN: 확인 전 재전송·새 Agent start 금지'),('State Store · 삭제·보관','현재 tombstone 우선 · 삭제된 기록의 재사용 차단','row migration·삭제된 관계의 복원 검사' if s=='a' else 'event redaction·호환 reader 유지'),True,True)
  q.line('request','top0','uow',[(202,367),(202,410),(159,410),(159,500)],'① 변경',(217,408))
  q.line('task','top1','uow',[(612,367),(612,401),(290,401),(290,526),(274,526)])
  q.line('response','top2','uow',[(1022,367),(1022,388),(296,388),(296,573),(274,573)])
  q.line('commit','uow','authority',[(274,552),(308,552)],blue=True)
  q.line('intent','uow','effects',[(159,605),(159,690)],'동일 transaction',(169,654))
  q.line('project','authority','projection',[(423,605),(423,690)],'② current/index' if s=='a' else '② append + projection',(432,654),True)
  q.line('start','authority','fence',[(538,552),(670,552)],'③ restart read',(576,537),True)
  q.line('load','fence','load',[(900,552),(934,552)],blue=True)
  q.line('read-source','projection','load',[(538,742),(600,742),(600,632),(1178,632),(1178,578),(1164,578)],'원본·미완료 기록 읽기' if s=='a' else 'checkpoint + committed tail',(637,623),True,True)
  q.line('apply','load','reconcile',[(1049,605),(1049,650),(785,650),(785,690)],'④ load / replay → pending 調整'.replace('調整','조정'),(838,678),True)
  q.line('query','reconcile','supportL',[(785,795),(785,860),(299,860),(299,908)],'⑤ source 상태 확인',(380,849))
  q.line('restore','reconcile','restore',[(900,742),(934,742)])
  q.line('delete','supportR','fence',[(914,908),(914,843),(637,843),(637,473),(785,473),(785,500)],'삭제 fence / payload 참조',(953,870),False,True)
  q.footer('동일 DB·내구성·owner·Agent capability · replay로 모델 판단·메일 발송·과거 음성 재생을 실행하지 않음')
 costs(p,['선택 근거: 현재 관계·미완료 원장으로 필요한 재연결을 직접 수행','대가: row migration·원장 일관성·손상 repair 유지','반증: 상태·전송·전달 관계 조정 로직이 반복적으로 복잡해짐'],['선택 근거: 확정 이력에서 projection과 전이 관계를 재구성','대가: event schema·reducer·checkpoint·삭제 redaction·tail 비용','반증: 필요한 복구가 단순 조회로 충분하거나 권위 이력 자체 손상'],'핵심 차이: 현재 view와 기록이 충돌할 때 무엇이 권위 원본인가. 외부 ACK·실제 청취의 불명은 두 안 모두 남는다.')

RICH_BUILDERS=[rich_interpretation,rich_acquisition,rich_observation,rich_direct,rich_speech,rich_maintenance,rich_representation,rich_recovery]

BUILDERS=[interpretation,acquisition,observation,direct,speech,maintenance,representation,recovery]

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
 if not args.check:OUT.mkdir(parents=True,exist_ok=True)
 for bg,compare in zip(BUILDERS,RICH_BUILDERS):
  bg();compare()
 errors=[]
 for p in PAGES:
  p.validate()
  for ext,data in [('svg',p.svg()),('drawio',p.drawio())]:
   ET.fromstring(data)
   path=OUT/(p.slug+'.'+ext)
   if args.check:
    if not path.exists() or path.read_text()!=data:errors.append(str(path.relative_to(ROOT)))
   else:path.write_text(data)
 slides='\n'.join(f'<section><img src="{p.slug}.svg" alt="{html.escape(p.title)}"><p><a href="{p.slug}.drawio">draw.io 원본</a></p></section>' for p in PAGES)
 review='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 설계 비교 · 16페이지</title><style>body{margin:0;background:#ececec;font-family:Arial}nav{padding:18px;position:sticky;top:0;background:white;border-bottom:1px solid #aaa}section{max-width:1600px;margin:24px auto;background:white;break-after:page}img{display:block;width:100%;height:auto}p{padding:0 20px 16px}@media print{nav,p{display:none}section{margin:0;max-width:none}@page{size:16in 9in;margin:0}}</style><nav>VIA · 배경 / 설계 비교 16페이지 — 브라우저 확대 또는 SVG 원본으로 보기</nav>'+slides+'</html>'
 path=OUT/'review.html'
 if args.check:
  if not path.exists() or path.read_text()!=review:errors.append(str(path.relative_to(ROOT)))
 else:path.write_text(review)
 if errors:raise SystemExit('Diagram drift: '+', '.join(errors))
 print(f'PASS: {len(PAGES)} diagram pairs; XML, identity, bounds and route geometry checked')
if __name__=='__main__':main()
