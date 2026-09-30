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
   process_node=False
   fill=PALE if s['blue'] else WHITE
   if kind in ('group','process'):fill=WHITE
   dash=' stroke-dasharray="10 7"' if kind in ('process','external') or process_node else ''
   out=[f'<g id="{s["id"]}"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{0 if kind in ("group","process") else 8}" fill="{fill}" stroke="{c}" stroke-width="{2 if kind in ("group","process") else 2.5}"{dash}/>']
   if kind=='store':
    out.append(f'<ellipse cx="{x+w/2}" cy="{y+16}" rx="{w/2}" ry="16" fill="{fill}" stroke="{c}" stroke-width="2"/>')
   if s['label']:
    ty=y+28 if kind!='store' else y+46
    out.append(f'<text x="{x+16}" y="{ty}" font-size="{18 if kind=='module' else 20}" font-weight="700" fill="{c}">{esc(s["label"])}</text>')
   if kind=='component':
    out.append(f'<path d="M{x} {y+45}H{x+w}" stroke="{c}" fill="none"/>')
   for i,line in enumerate(s['lines']):
    yy=(y+54+i*23 if kind=='module' else y+64+i*25+(15 if kind=='store' else 0))
    lc=INK if i in s.get('common_lines',[]) else c
    out.append(f'<text x="{x+16}" y="{yy}" font-size="{18 if kind=='module' else 20}" fill="{lc}">{esc(line)}</text>')
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
   if kind in ('process','external'):style+='dashed=1;dashPattern=10 7;'
   if kind=='store':style+='shape=cylinder3;size=16;'
   cell(s['id'],x,y,s['w'],s['h'],'',style,g['id'] if g else '1')
   size=18 if kind=='module' else 20
   txt(s['id']+'-title',16,46 if kind=='store' else 28,s['w']-32,size,s['label'],c,True,s['id'])
   for i,line in enumerate(s['lines']):
    yy=54+i*23 if kind=='module' else 64+i*25+(15 if kind=='store' else 0)
    txt(s['id']+'-text-'+str(i),16,yy,s['w']-32,size,line,INK if i in s.get('common_lines',[]) else c,False,s['id'])
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


class Canvas:
 """Coordinate freedom per alternative. No shared box count or grid."""
 def __init__(self,p,s):self.p=p;self.s=s;self.x=48 if s=='a' else 1312
 def n(self,id,x,y,w,h,title,*lines,blue=False,kind='component'):
  return self.p.node(self.s+'-'+id,self.x+x,y,w,h,title,*lines,blue=blue,kind=kind)
 def g(self,id,x,y,w,h,title,blue=False,process=False):
  return self.p.group(self.s+'-'+id,self.x+x,y,w,h,title,blue,'process' if process else 'group')
 def e(self,id,src,dst,sp='B',tp='T',via=(),label='',at=None,blue=False,dash=False):
  self.p.edge(self.s+'-edge-'+id,self.s+'-'+src,self.s+'-'+dst,sp,tp,[(self.x+x,y) for x,y in via],label,(self.x+at[0],at[1]) if at else None,blue,dash)
 def t(self,id,x,y,txt,size=20,blue=False):self.p.text(self.s+'-'+id,self.x+x,y,txt,size,BLUE if blue else INK)

def pair(slug,title,one,two):
 p=Page(slug+'-comparison',title+'  |  SW Architecture 비교','검정: 공통 구성·책임  /  파랑: 양안의 대체·추가·제거 대상  /  실선 경계: 논리 Component  /  점선 경계: process  /  원통: 저장소')
 for s,x,t in [('a',48,one),('b',1312,two)]:
  p.text(s+'-heading',x+8,161,t,27,bold=True)
 p.rawedge('divider',[(1280,195),(1280,1310)],dash=True)
 return p,Canvas(p,'a'),Canvas(p,'b')

def conclusion(p,a,b,claim):
 for s,x,lines in [('a',48,a),('b',1312,b)]:
  p.node(s+'-cost',x,1110,1200,186,'구조 선택의 이득과 대가',*lines,kind='box')
 p.text('claim',55,1354,claim,25,bold=True);PAGES.append(p)

def retrieval():
 p,a,b=pair('semantic-retrieval-subsystem','자료·대화·Task의 후보 검색','방안 1 · target  |  owner 조회 중심','방안 2 · 대안  |  지속 색인 + 검색 서브시스템')
 a.n('request',330,220,460,95,'Request Controller','허용 scope · 현재 입력 · 후보 요청')
 a.g('cm',40,375,770,655,'Context Manager',True)
 a.n('search',70,425,540,110,'Metadata / Keyword Search','이름 · 시간 · artifact · 최근 대화','bounded source 선택',blue=True)
 a.n('cache',70,640,240,140,'Revision Cache','summary / index','기존 cache 재사용',blue=False,kind='store')
 a.n('ports',355,640,255,140,'Owner Read Adapters','Task / 대화 / 자료','revision · receipt',blue=True)
 a.n('source',840,630,310,150,'Owner / source ports','Task · 대화 · 게시 기록','허용된 자료 원본','source revision')
 a.n('policy',840,390,310,110,'Policy Manager','read / 보관 / recipient','현재 epoch')
 a.n('package',330,885,460,115,'Evidence Package Assembly','원문 · 출처 · 후보 coverage','근거 package → 의미 해석')
 a.t('absent',62,1060,'별도 embedding runtime · vector index · 색인 worker 없음',20,True)
 a.e('q','request','search',via=[(560,347),(340,347)])
 a.e('cache','search','cache',via=[(340,582),(190,582)])
 a.e('read','search','ports',via=[(340,565),(482.5,565)],blue=True)
 a.e('source','ports','source','R','L',via=[(748,710),(748,705)],label='bounded read',at=(636,694),blue=True)
 a.e('policy','policy','ports','L','R',via=[(700,445),(700,710)],dash=True)
 a.e('package','ports','package',via=[(482.5,855),(560,855)],label='검증한 원문',at=(574,853))
 a.e('reuse','cache','package',via=[(190,860),(445,860),(445,885)],tp='T',blue=True) # replaced below to keep exact port
 # Owner/cache meet at the package port without inventing a persistent indexer.
 p.edges[-1]['blue']=False
 p.edges[-1]['points']=[(a.x+190,780),(a.x+190,869),(a.x+560,869),(a.x+560,885)]
 b.n('sources',35,220,740,95,'Owner / source ports','같은 허용 자료 · source revision / 변경·삭제 알림')
 b.n('policy',850,220,310,95,'Policy Manager','같은 read·보관 epoch')
 b.g('indexer',35,350,1125,282,'Indexing Worker · local background process',True,True)
 b.n('feed',65,410,290,125,'Change Collector','등록 collection만 수집','변경 feed / 유한 scan',blue=True)
 b.n('chunk',412,410,295,125,'Chunk / Index Builder','source revision · span','유한 queue / generation',blue=True)
 b.n('embedding',785,410,340,125,'Embedding Runtime','추가 helper weights','document / query encode',blue=True)
 b.n('index',785,705,340,155,'Vector Index + Manifest','chunk · source revision','build / epoch / coverage','원본에서 재생성 가능',blue=True,kind='store')
 b.g('service',35,667,680,355,'Semantic Retrieval Service',True)
 b.n('query',65,730,280,120,'Query API / Use Gate','scope · lexical / exact','현재 policy · data epoch',blue=True)
 b.n('resolve',390,870,295,120,'Evidence Resolver','후보 → 원문 재검증','stale / 누락 → 보완',blue=True)
 b.n('request',35,1050,335,48,'Request Controller',kind='module')
 b.n('cm',750,948,410,140,'Context Manager','Consume Gate / 현재 두 epoch','기억·조립·revision cache 유지','후보 생산 위임 / 삭제 전파',blue=True)
 p.shapes[-1]['common_lines']=[1]
 b.e('feed','sources','feed',via=[(405,335),(210,335)],blue=True)
 b.e('split','feed','chunk','R','L',blue=True)
 b.e('encode','chunk','embedding','R','L',blue=True)
 b.e('publish','embedding','index',label='원자 generation 게시',at=(827,659),blue=True)
 b.e('request','request','cm','R','L',via=[(700,1074),(700,1018)])
 b.e('query','cm','query','L','L',via=[(725,1018),(725,1034),(20,1034),(20,790)],blue=True,label='query + 현재 data epoch',at=(48,1026))
 b.e('query-encode','query','embedding','R','L',via=[(365,790),(365,650),(755,650),(755,472.5)],label='query / 같은 embedding build',at=(382,644),blue=True)
 b.e('encoded','embedding','query','B','T',via=[(955,612),(380,612),(380,714),(205,714)],blue=True,dash=True)
 b.e('lookup','query','index','B','L',via=[(205,858),(735,858),(735,782.5)],label='검색',at=(739,820),blue=True)
 b.e('candidates','index','resolve','B','R',via=[(955,908),(685,908),(685,930)],blue=True)
 b.e('raw','resolve','sources','L','L',via=[(15,930),(15,267.5)],label='원문 revision 확인',at=(48,646),dash=True)
 b.e('result','resolve','cm','R','T',via=[(715,930),(955,930)])
 b.e('revoke','policy','index','R','T',via=[(1183,267.5),(1183,683),(955,683)],label='비동기 purge',at=(1000,675),dash=True)
 b.e('query-policy','policy','query','L','L',via=[(795,267.5),(795,328),(5,328),(5,790)],label='현재 Use Envelope',at=(45,344),dash=True)
 b.e('consume-policy','policy','cm','R','R',via=[(1200,267.5),(1200,1018)],dash=True)
 b.t('new',50,590,'변경 생산 경로가 요청과 독립적으로 유지됨',19,True)
 conclusion(p,['이득: 기존 source·cache 활용, 별도 helper·색인 자산 없이 시작','대가: 표현이 다른 과거 자료의 후보 누락·반복 source 조회','QA: 정확한 후보·응답 대기·source 변경 파급'],['이득: 허용 corpus의 의미 후보를 여러 요청에서 재사용','대가: helper 메모리·지속 색인·최신성·삭제·재구축','조건: 같은 자료의 반복 의미 검색 가치가 유지 비용보다 큰가?'],'핵심 구조 차이: 요청별 원본 조회  ↔  독립 색인 생산체 + 검색 서비스 + 파생 검색 저장소')

def workflow():
 p,a,b=pair('durable-request-orchestration','사용자 질문·선행 결과의 대기와 재개','방안 1 · target  |  domain controller가 진행 소유','방안 2 · 대안  |  workflow runtime이 continuation 소유')
 for q in (a,b):q.n('input',50,220,1100,90,'Interaction Manager / Task Manager','입력·사용자 답변 / 확인된 Task 질문·결과 — 같은 사건과 기능')
 a.g('controller',160,385,880,330,'Request Controller',True)
 a.n('semantic',190,435,360,115,'의미 / admission 검증','현재 input · policy','Semantic Commit')
 a.n('resume',630,435,370,115,'Graph / Question Controller','Request Graph · 질문 lifecycle','대기·timer·재개·정정 제어',blue=True)
 a.n('store',95,810,460,180,'State Store · 권위 current state','Request / 질문 / graph revision','inbox · domain outbox · intent','target도 이미 durable',blue=True,kind='store')
 a.n('services',725,810,430,180,'기존 Component ports','Context Manager · Request Interpreter','Task Manager · Response Manager','Agent Gateway: 외부 command 전송')
 a.e('input','input','semantic',via=[(600,350),(370,350)])
 a.e('check','semantic','resume','R','L')
 a.e('save','resume','store',via=[(815,760),(325,760)],label='owner UoW / CAS',at=(357,750),blue=True)
 a.e('call','resume','services',via=[(815,735),(940,735)],label='직접 실행·재개',at=(952,761),blue=True)
 a.t('target',200,616,'domain 전이 코드가 각 대기와 재개를 구현',22,True)
 a.t('target2',200,658,'미완료 원장·교차 owner transaction·재시작 복구 포함',20)
 b.n('controller',295,355,620,120,'Request Controller','입력·의미·질문 binding 검사 / admission','graph·질문 진행 writer는 runtime으로 이동',blue=True)
 b.g('runtime',35,535,1125,315,'Interaction Workflow Runtime · Core 내부 논리 서브시스템',True)
 b.n('inbox',65,590,260,115,'Signal Inbox','question / result key','중복 제거 · revision',blue=True)
 b.n('engine',412,590,310,115,'Continuation Engine','step / waiting / version','Request 진행 단일 writer',blue=True)
 b.n('timer',815,590,310,115,'Timer Service','내구 timer identity','만료 ≠ 사용자 승인',blue=True)
 b.n('activity',412,745,310,90,'Activity Dispatcher','intent / attempt / 완료 CAS',blue=True)
 b.n('store',40,910,460,170,'Continuation Store · State Store','workflow / signal / activity intent','기존 owner DB와 같은 transaction','새 event sourcing은 도입하지 않음',blue=True,kind='store')
 b.n('services',715,910,445,170,'기존 Component ports','Context Manager · Request Interpreter','Task Manager · Response Manager','Agent Gateway: 외부 command 전송')
 b.e('input','input','controller',via=[(600,333),(605,333)])
 b.e('signal','controller','inbox',via=[(605,510),(195,510)],label='검증된 signal',at=(210,500),blue=True)
 b.e('resume','inbox','engine','R','L',blue=True)
 b.e('timer','timer','engine','L','R',blue=True)
 b.e('dispatch','engine','activity',blue=True)
 b.e('persist','engine','store','L','T',via=[(380,647.5),(380,880),(270,880)],blue=True)
 b.e('call','activity','services',via=[(567,882),(937.5,882)],label='고정 VIA activity',at=(787,873),blue=True)
 b.e('admit','activity','controller','R','R',via=[(1183,790),(1183,415)],label='실행·게시 전 admission',at=(830,512),dash=True)
 conclusion(p,['이득: domain 상태·검증·교차 transaction에 가까운 직접 제어','대가: 대기 종류마다 전이·재개·복구 코드 유지','존재: Request Graph + 질문 + 내구 원장 / 별도 실행 엔진 없음'],['이득: 내구 signal·timer·activity로 재개 계약을 공통화','대가: engine 상태·queue·definition migration·adapter','대체: Request Controller의 진행 책임 → workflow runtime'],'핵심 구조 차이: domain 상태기계  ↔  별도 실행 substrate · Agent 업무 planning과 Task 소유권은 유지')

def speech():
 p,a,b=pair('speech-evidence-source','새 발화를 계속 인식하는 실행 구성','방안 1 · target  |  독립 ASR + Omni의 두 실행 경계','방안 2 · 대안  |  Omni native evidence · ASR worker 제거')
 for q in (a,b):q.n('capture',235,220,730,115,'Interaction Manager','capture / local stop / sample clock / 화면 timeline','모델 계산·재시작과 별도로 동작')
 a.g('asr',30,425,470,280,'Speech Input Worker',True,True)
 a.n('recognizer',60,482,410,155,'Streaming ASR · 별도 dependency','ASR weights / decoder / CPU 예약','partial · final · span · revision','ASR adapter / producer incarnation',blue=True)
 a.g('omni',660,425,500,280,'Shared Inference Service',True,True)
 a.n('model',690,482,440,155,'Omni Runtime · weights ×1','VOICE / SEMANTIC session 격리','공유 scheduler / KV 예산','Omni adapter / echo / generation')
 a.g('join',150,790,900,225,'Interaction Manager · 입력 근거 결합',True)
 a.n('merge',180,840,385,125,'Evidence Join / Conflict','ASR 전사 + Omni echo','핵심 불일치 → 원음/질문',blue=True)
 a.n('timeline',620,840,400,125,'Timeline / Input Record','당시 화면 · watermark / gap','canonical revision → host')
 a.e('asr','capture','recognizer',via=[(600,378),(265,378)],label='audio',at=(279,401),blue=True)
 a.e('omni','capture','model',via=[(600,395),(910,395)],label='같은 audio',at=(925,411),blue=True)
 a.e('evidence','recognizer','merge',via=[(265,747),(372.5,747)],label='SpeechEvidence',at=(49,759),blue=True)
 a.e('echo','model','merge',via=[(910,737),(587,737),(587,902.5)],tp='R',label='echo / conflict',at=(674,727),blue=True)
 a.e('combine','merge','timeline','R','L')
 a.t('fault',53,1060,'Omni 중단: capture·ASR·stop 유지 / 의미 이해·음성 생성 불가',20,True)
 b.g('shared',120,415,960,410,'Shared Inference Service · 하나의 process',True,True)
 b.n('omni',400,470,590,120,'Omni Runtime · weights ×1','공유 encoder / scheduler / session별 KV','인식·VOICE·SEMANTIC 진행 예산')
 b.n('native',155,650,405,125,'Native Evidence Adapter','transcript / span / gap / final','Omni incarnation · input revision',blue=True)
 b.n('roles',640,650,350,125,'VOICE / SEMANTIC Sessions','제안·생성 / 요청 의미 해석','역할별 Context·권한 분리')
 b.n('timeline',235,910,730,115,'Interaction Manager','native revision + 당시 화면 → input record','단일 producer의 자기 동의는 독립 검증이 아님',blue=True)
 b.e('audio','capture','omni',via=[(600,377),(695,377)],label='audio',at=(720,397),blue=True)
 b.e('evidence','omni','native',via=[(695,618),(357.5,618)],blue=True)
 b.e('role','omni','roles','R','R',via=[(1021,530),(1021,712.5)])
 b.e('native','native','timeline',via=[(357.5,870),(600,870)],label='SpeechEvidence',at=(384,856),blue=True)
 b.t('fault',83,1070,'Omni 중단: capture·stop만 유지 / 인식·이해·음성 생성 모두 불가',20,True)
 conclusion(p,['이득: Omni 부하·재기동과 독립된 입력 근거 진행','대가: ASR 전체 메모리·CPU·IPC·중복 인식·전사 충돌','필수: canonical 근거 → Request Controller 확정은 공통'],['이득: 별도 ASR worker·weights·연동·복구 경로 제거','대가: 인식까지 공유 계산·장애 경계에 결합','조건: native timestamp·revision·동시 recognition 실제 지원'],'핵심 구조 차이: 독립 recognizer 서브시스템의 존재 / 부재 · capture 지속과 recognition 지속을 구별')

def recovery():
 p,a,b=pair('recovery-state-source','확정 상태를 보존하고 복원하는 저장 구조','방안 1 · target  |  current records에서 복원','방안 2 · 대안  |  domain journal에서 projection 재구성')
 for q in (a,b):q.n('owners',40,220,1120,95,'Request Controller / Task Manager / Response Manager','각 owner가 의미·revision을 검증한 변경 집합 · 외부 호출은 transaction 밖')
 a.g('store',95,395,980,285,'State Store · local embedded DB / 동일 transaction',True)
 a.n('current',135,452,470,170,'Current Records · 권위 원본','Conversation / Request / Task','질문 · relation · publication','현재 row / schema / revision',blue=True,kind='store')
 a.n('ledger',665,452,365,170,'Pending / Effect Ledger','inbox · command outbox','domain outbox · receipt','Audit 존재 ≠ 권위 replay',kind='store')
 a.n('loader',135,770,470,125,'State Loader / Migration','현재 row · 관계 · 미완료 읽기','source 확인과 owner 복원',blue=True)
 a.n('reconcile',725,770,350,125,'Agent Gateway','동일 command key 조회','UNKNOWN 재실행 금지')
 a.n('open',135,960,940,100,'각 owner의 복구 개방','질문·Task·publication·현재 policy 검증 / 옛 음성 자동 재생 금지')
 a.e('commit','owners','current',via=[(600,360),(370,360)],label='current + intent 원자 기록',at=(397,351),blue=True)
 a.e('effects','current','ledger','R','L')
 a.e('load','current','loader',label='restart load',at=(385,724),blue=True)
 a.e('scan','ledger','reconcile',via=[(847.5,723),(900,723)])
 a.e('restore','loader','open',via=[(370,932),(605,932)])
 a.e('source','reconcile','open',via=[(900,929),(605,929)])
 b.g('db',35,380,1125,670,'State Store · 같은 embedded DB / event authority',True)
 b.n('journal',65,445,380,170,'Domain Journal · 권위 원본','committed batch / event schema','owner revision / log position','effect intent와 원자 commit',blue=True,kind='store')
 b.n('projector',535,445,270,125,'Projection Engine','versioned reducer','정상 commit에 적용',blue=True)
 b.n('projection',860,445,265,170,'Current Projection','재생성 가능한 view','applied log position','원본과 다르면 재구축',blue=True,kind='store')
 b.n('replay',65,710,380,125,'Replay Engine','schema reader registry','checkpoint + tail / prefix',blue=True)
 b.n('checkpoint',535,710,270,140,'Checkpoint Manager','position / digest','삭제 epoch 검증',blue=True)
 b.n('snap',860,710,265,140,'Checkpoint Store','가속용 파생물','권위 원본 아님',blue=True,kind='store')
 b.n('ledger',65,915,1060,100,'공통 Effect / Pending Ledger · 삭제 fence','원장·현재 deletion epoch 우선 / replay에서 모델·외부 Action·음성 재실행 금지')
 b.e('append','owners','journal',via=[(600,352),(255,352)],label='batch + projection + intent',at=(625,351),blue=True)
 b.e('reduce','journal','projector','R','L',via=[(489,530),(489,507.5)],blue=True)
 b.e('view','projector','projection','R','L',via=[(832,507.5),(832,530)],blue=True)
 b.e('replay','journal','replay',label='restart read',at=(269,672),blue=True)
 b.e('rebuild','replay','projection','R','B',via=[(484,772.5),(484,665),(992.5,665)],label='再구성',at=(633,656),blue=True,dash=True)
 p.edges[-1]['label']='projection 재구성'
 b.e('checkpoint','projection','snap',blue=True,dash=True)
 b.e('validate','snap','checkpoint','L','R',blue=True)
 b.e('base','checkpoint','replay','L','R',via=[(470,780),(470,772.5)],blue=True,dash=True)
 b.t('outside',50,1080,'복구 후 공통: Agent Gateway source 확인 → 각 owner admission 개방',20)
 conclusion(p,['이득: 현재 상태·미완료 원장을 직접 읽는 복구','대가: current 관계 손상을 모든 audit로 재구성할 수는 없음','target도 inbox·outbox·domain event·transaction을 보유'],['이득: intact journal로 projection 손상·새 view 재구축','대가: reducer·reader·checkpoint·redaction·긴 replay','동일 DB / 별도 서버 도입 효과나 외부 ACK 복구를 주장하지 않음'],'핵심 구조 차이: 권위 current row + loader  ↔  권위 journal + projection / replay / checkpoint 체계')

def bg_speech():
 slug='speech-evidence-source'
 background(slug,'음성 입력 근거 · 의미 추론 중에도 계속 듣고 당시 화면에 연결해야 한다',
 '이전 요청을 해석하는 동안 사용자가 그래프에서 표로 포인터를 옮기며 “이거, 아니 저걸 넣어줘”라고 말한다.','UC-03 · UC-04 · UC-11 · UC-18',[
 ('발화·화면', [('기존 semantic 실행','이전 자료 설명 중','Omni 계산·KV 사용'),('새 정정 발화','“이거” → “아니 저걸”','화면 A → 화면 B'),('전사 늦게 도착','최종 revision / 대체 구간','도착 시각 ≠ 발화 시각'),('올바른 지칭','표 B / 정정 관계','당시 sample 구간과 연결')]),
 ('필요 근거', [('연속 capture','sample sequence / clock','유한 ring + local stop'),('연속 recognition','partial / final / revision','녹음 대기만으로는 부족'),('시간 관계','span + 오차 + watermark','겹치는 후보 / gap 보존'),('확정 입력','SEALED + evidence 참조','근거 없으면 재지칭 질문')]),
 ('dependency', [('독립 인식 경로','경량 ASR + Omni','두 전사의 충돌 가능'),('통합 인식 경로','Omni native evidence','한 runtime 자원 공유'),('Omni crash','독립 ASR: 전사 유지 가능','native: 전사도 중단'),('사용자에게 알릴 사실','입력 접수와 이해는 다름','미처리·유실을 숨기지 않음')])],
 [('독립성의 가치와 비용',['semantic과 별도로 입력 근거가 진행한다.','별도 weights·CPU·worker가 필요하다.','중복 인식과 전사 충돌을 관리한다.']),('통합의 가치와 비용',['중복 dependency를 줄일 수 있다.','native 시간·revision 계약이 필요하다.','같은 오류·자원·장애에 함께 노출된다.']),('필수 runtime 계약',['source sample 시각·uncertainty·gap','semantic 중에도 recognition 진행','모델 기능은 요구 계약이며 지원 미검증'])],
 'SpeechEvidence를 독립 recognizer에서 생산할 것인가, 동일 Omni의 native stream에 입력 근거까지 맡길 것인가?')



def bg_recovery():
 slug='recovery-state-source'
 background(slug,'복구 원본 · 다시 켜지는 것과 업무를 올바르게 이어가는 것은 다르다',
 '취소 command 전송 직후 ACK가 사라지고 VIA가 종료된다. 다른 Task의 결과는 저장됐지만 사용자에게 아직 전달되지 않았다.','UC-12 · UC-13 · UC-16 · UC-18',[
 ('업무 제어', [('사용자 취소','Task A / command C17','현재 승인·revision 검사'),('전송 시작','DISPATCHING commit','외부 호출은 transaction 밖'),('ACK 미수신 + crash','Agent가 받았는지 불명','취소 완료라고 단정 금지'),('재연결','같은 command key로 query','미확인은 UNKNOWN 유지')]),
 ('결과 전달', [('Task B 결과 확정','result r9 / publication P3','source-confirmed 완료'),('인계 중단','domain outbox는 남음','UI receipt 아직 없음'),('재시작','원래 대화·질문 관계 복구','새 Task로 중복 생성 금지'),('사용자 결과','Text는 같은 ID upsert','불명 음성 자동 재생 금지')]),
 ('저장·삭제', [('현재 state 또는 이력','owner별 확정 전이','같은 local DB 내구성'),('memory 삭제','epoch·tombstone commit','payload purge 예약'),('옛 checkpoint','삭제 전 데이터 포함 가능','그대로 복원하면 기억 부활'),('운영 가능 상태','삭제 fence 먼저 적용','schema·effect identity 검증')])],
 [('현재 상태 중심 복원',['owner row·pending 기록을 직접 읽는다.','복구에 필요한 관계의 원자성 유지','audit·outbox가 없는 안이 아니다.']),('확정 이력 중심 재구성',['checkpoint + tail로 projection을 만든다.','과거 schema·reducer·삭제 호환 필요','모델 판단·외부 Action은 재실행하지 않는다.']),('두 안 모두 못하는 일',['잃어버린 외부 ACK를 추측할 수 없다.','실제 들은 음성을 로그만으로 증명 못한다.','권위 저장 자체 손상은 별도 복구가 필요하다.'])],
 '운영 상태를 현재 owner 기록에서 복원할 것인가, 확정 domain 이력에서 검증된 checkpoint와 tail로 재구성할 것인가?')



def bg_retrieval():
 background('semantic-retrieval-subsystem','자료 검색 · 현재 표현과 과거 자료의 이름이 다를 때',
 '“지난번 비용을 줄였던 제안서 찾아서 이번 보고서에 써줘.” 파일명·대화·Task 기록에는 서로 다른 표현이 남아 있다.','UC-05 중심 · UC-02 · UC-06 · UC-07 · UC-10 · UC-17',[
 ('자료와 요청',[('기존 자료','보고서·대화·Task 결과','허용된 collection만 대상'),('새로운 표현','“비용을 줄였던 제안”','파일명은 운영 개선 r3'),('후보 발견','이름·기간·의미 근접 후보','찾음 ≠ 대상 확정'),('업무에 사용','현재 원문·고유 대상 확인','동명이면 사용자 선택')]),
 ('변경과 권한',[('원본 revision','자료와 owner 기록 존재','동일 기능·scope 유지'),('수정·삭제·철회','r3 → r4 / 기억 삭제','파생 검색 자산도 영향'),('조회 시 검증','current policy + 삭제 epoch','오래된 후보 사용 차단'),('근거 package','원문 span·출처·coverage','후보 순위는 실행 권한 아님')]),
 ('구조 선택',[('요청별 조회','Context Manager / owner port','기존 cache·summary 사용'),('지속 색인','별도 수집·embedding·index','요청이 없어도 관리 필요'),('후보 생산','source read 또는 index search','최신성·누락 확인 필요'),('실제 차이','검색 생산체·helper·저장소','조립·기억 owner는 유지')])],
 [('왜 VIA에 중요한가',['대화 속 표현만으로 과거 자료를 다시 찾는다.','후보 누락은 재질문·잘못된 자료 전달로 이어진다.','검색 점수로 Action 대상을 확정할 수는 없다.']),('왜 Architecture 문제인가',['source 조회를 반복할지 검색 자산을 유지할지 결정한다.','추가 helper·worker·저장소·삭제 경로가 생긴다.','요청 속도와 상시 유지 비용을 교환한다.']),('공정한 대안의 조건',['동일 자료·권한·사용자 목표를 유지한다.','target의 기존 cache·summary를 제거하지 않는다.','embedding 지원·효과는 구현 전 설계 가정이다.'])],
 '요청마다 owner의 자료를 조회할 것인가, 별도 색인 생산체와 검색 서비스를 유지하고 현재 원문을 다시 검증할 것인가?')

def bg_workflow():
 background('durable-request-orchestration','대기와 재개 · 대화가 끊겨도 요청의 진행 관계는 이어져야 한다',
 '“이 표로 보고서를 만들고 끝나면 메일을 준비해줘.” 수신인 질문에 답하기 전에 다른 업무로 전환하고 VIA가 재시작된다.','UC-06 · UC-09 · UC-11 · UC-14 · UC-16 · UC-17 · UC-18',[
 ('사용자 대화',[('복합 요청','보고서 결과 → 메일 준비','VIA는 상호작용 진행만 소유'),('사용자 질문','수신인을 질문 / WAIT_USER','Omni KV 점유는 해제'),('교차 사건','다른 대화·정정·취소','현재 Request revision 증가'),('답변 후 재개','맞는 질문·Request에 연결','옛 조건의 실행은 차단')]),
 ('진행 상태',[('graph 진행','선행 결과·질문·admission','Task 실행 자체는 Task Manager'),('durable wait','대기 이유·timer·effect intent','target도 이미 기록한다'),('crash / 재시작','signal 중복·timer 만료','같은 activity ID로 확인'),('계속할 위치','현재 revision의 continuation','모델을 replay해서 복원하지 않음')]),
 ('책임의 위치',[('target controller','Request Controller가 graph 진행','domain UoW + outbox'),('대안 실행체','Interaction Workflow Runtime','진행·질문·대기 상태 소유'),('owner 협력','의미·Task·기억의 owner 유지','고정 VIA activity만 호출'),('구조적 대가','signal / timer / continuation','버전 호환·중복 계약 유지')])],
 [('왜 VIA에 중요한가',['긴 Agent 업무와 짧은 사용자 대화가 겹친다.','잘못된 재개는 옛 요청·다른 질문을 실행한다.','재시작 후도 동일 대화와 업무 관계를 이어야 한다.']),('왜 Architecture 문제인가',['진행 상태를 domain controller가 직접 소유할지,','별도 실행체에 continuation을 맡길지 결정한다.','상태·수명·호출·버전 계약의 주체가 바뀐다.']),('공정한 대안의 조건',['양안 모두 durable wait·중복 방지·KV 해제를 갖춘다.','별도 서버나 자동 병렬화의 효과를 주장하지 않는다.','Agent의 업무 planning을 VIA로 옮기지 않는다.'])],
 'Request Controller가 진행·대기·재개를 직접 관리할 것인가, 별도 workflow 실행체가 continuation을 소유하고 owner들을 호출할 것인가?')

BUILDERS=[(bg_retrieval,retrieval),(bg_workflow,workflow),(bg_speech,speech),(bg_recovery,recovery)]

def main():
 ap=argparse.ArgumentParser();ap.add_argument('--check',action='store_true');args=ap.parse_args()
 if not args.check:OUT.mkdir(parents=True,exist_ok=True)
 for bg,compare in BUILDERS:
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
 review='<!doctype html><html lang="ko"><meta charset="utf-8"><title>VIA 설계 비교 · 8페이지</title><style>body{margin:0;background:#ececec;font-family:Arial}nav{padding:18px;position:sticky;top:0;background:white;border-bottom:1px solid #aaa}section{max-width:1600px;margin:24px auto;background:white;break-after:page}img{display:block;width:100%;height:auto}p{padding:0 20px 16px}@media print{nav,p{display:none}section{margin:0;max-width:none}@page{size:16in 9in;margin:0}}</style><nav>VIA · 배경 / 설계 비교 8페이지 — 브라우저 확대 또는 SVG 원본으로 보기</nav>'+slides+'</html>'
 path=OUT/'review.html'
 if args.check:
  if not path.exists() or path.read_text()!=review:errors.append(str(path.relative_to(ROOT)))
 else:path.write_text(review)
 if errors:raise SystemExit('Diagram drift: '+', '.join(errors))
 print(f'PASS: {len(PAGES)} diagram pairs; XML, identity, bounds and route geometry checked')
if __name__=='__main__':main()
