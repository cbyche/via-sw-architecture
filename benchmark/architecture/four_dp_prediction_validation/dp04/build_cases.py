"""Fresh authored heldout PNGs. SVG geometry is evaluator-only, never parser input."""
from pathlib import Path
import json,html,hashlib,subprocess
BASE=Path(__file__).resolve().parent;ASSET=BASE/'assets';ASSET.mkdir(exist_ok=True)
NODE='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node';SHARP='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp'
def text(x,y,s,size=17):return f'<text x="{x}" y="{y}" font-size="{size}" fill="#14243c">{html.escape(s)}</text>'
def rect(x,y,w,h):return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="#fff" stroke="#8793a3" stroke-width="2"/>'
def render(name,body):
 p=ASSET/(name+'.svg');p.write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="700"><rect width="1000" height="700" fill="#eef3f7"/><g font-family="Arial,sans-serif">{body}</g></svg>');subprocess.run([NODE,'-e','require(process.argv[1])(process.argv[2]).png().toFile(process.argv[3])',SHARP,str(p),str(p.with_suffix('.png'))],check=True)
def frame(name,rev=1,t=0):
 p=ASSET/(name+'.png');return dict(frame_id=name,source_id='authored-document',source_revision=rev,observed_at_ms=t,width=1000,height=700,coordinate_space='frame_pixels',image_path=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def expr(i,s,a,b):return dict(expression_id=i,text=s,start_ms=a,end_ms=b,time_uncertainty_ms=20)
def revision(n,s,es,events,frames):return dict(revision=n,transcript=s,expressions=es,events=events,frames=frames,native_selection={'status':'unsupported','reason':'PNG-only authored evidence; no native object source exists'},capture_gaps=[],permission='allowed',transcript_ready_ms=max(e['end_ms'] for e in es)+100)
def point(i,f,x,y,t):return dict(event_id=i,type='pointer_move',frame_id=f,x=x,y=y,t_ms=t,button='none')
fixtures=[];oracle=[]
def case(i,description,rs,expected,late=False,condition=''):
 fixtures.append(dict(id=i,dp='04',description=description,revisions=rs,late_result_replay=late,provenance='FRESH_AUTHORED_HELDOUT_20261019; no recorded user/native API data',prediction_condition=condition));oracle.append(dict(id=i,adoptions=[dict(expression_id=e,revision=r,targets=t) for e,r,t in expected]))
body=text(25,35,'Regional projects / repeated names and neighboring cards',24)
for i in range(12):
 x=25+i%4*244;y=70+i//4*194
 body+=rect(x,y,228,178)+text(x+12,y+28,f'D{i+1:02} / Vega',18)+text(x+12,y+61,['West','Central','North'][i//4]+' phase '+str(i%4+1),16)+text(x+12,y+100,'Review pending',14)+text(x+12,y+145,'Metric '+str(72+i*3),15)
render('dense_new',body)
r=revision(1,'이 항목과 이 항목의 실적을 비교해줘.',[expr('one','이 항목',100,500),expr('two','이 항목',900,1300)],[point('p1','dense_new',464,390,300),point('p2','dense_new',781,594,1100)],[frame('dense_new')]);case('dp04-held-dense','Repeated labels and edge-near pointer extents',[r],[('one',1,['D06']),('two',1,['D12'])],condition='B may gain OCR candidate legibility; inaccurate item extents can defeat code binding')
body=text(25,35,'Maintenance board / closely spaced small text',24)
for i in range(16):
 x=35+i%4*240;y=70+i//4*150
 body+=rect(x,y,220,130)+text(x+10,y+23,f'T{i+1:02} / '+('Cedar' if i%2 else 'Birch'),13)+text(x+10,y+52,'Status '+('HOLD' if i in [6,13] else 'READY'),12)+text(x+10,y+84,'Owner '+['Mira','Niko','Lena','Tari'][i%4],12)+text(x+10,y+113,'Priority '+str(20-i),12)
render('small_text',body)
r=revision(1,'Cedar 항목 중에서 HOLD 상태인 T14만 설명해줘.',[expr('item','HOLD 상태인 T14',100,900)],[],[frame('small_text')]);case('dp04-held-small-text','Small repeated text and explicit unique condition',[r],[('item',1,['T14'])],condition='Actual OCR text may reduce small-label confusion; both may directly read IDs without helper benefit')
body=text(25,35,'Measured samples / irregular scatter',24)+rect(55,70,890,540)
pts=[('P1',145,430),('P2',235,220),('P3',360,280),('P4',495,390),('P5',570,245),('P6',685,440),('P7',810,190),('P8',870,505)]
for n,x,y in pts:body+=f'<circle cx="{x}" cy="{y}" r="7" fill="#453bc1"/>'+text(x+12,y-10,n)
render('graph_new',body)
e=dict(event_id='loop',type='pointer_path',start_ms=200,end_ms=1100,frame_id='graph_new',button='none',points=[[300,180],[620,180],[620,470],[310,470],[300,180]])
r=revision(1,'이렇게 둘러싼 세 점만 평균을 내줘.',[expr('group','둘러싼 세 점',100,1200)],[e],[frame('graph_new')]);case('dp04-held-region','Free region containing three irregular samples',[r],[('group',1,['P3','P4','P5'])],condition='OCR boxes cover labels, not sample centers; joint raw interpretation may avoid parser granularity mismatch')
body=text(25,35,'Four throughput plots / shared axes',24)
for i in range(4):
 x=30+i*244;body+=rect(x,100,220,470)+text(x+15,137,f'G{i+1} / '+['Input','Encode','Decode','Output'][i],18)
 for j in range(4):body+=f'<rect x="{x+25+j*44}" y="{440-j*45}" width="23" height="{80+j*45}" fill="#2478b0"/>'
render('nested_graphs',body)
r=revision(1,'바깥쪽 두 개는 빼고 안쪽 두 그래프만 비교해줘.',[expr('inner','안쪽 두 그래프',100,1000)],[],[frame('nested_graphs')]);case('dp04-held-language-region','Language defines pair, excluding outer plots',[r],[('inner',1,['G2','G3'])],condition='Joint access to original language and plot grouping may help; well-complete candidates can also bind correctly')
for page in ['H','J']:
 body=text(25,35,'Operations review / page '+page,24)
 for i in range(3):body+=rect(65,85+i*185,830,160)+text(90,126+i*185,f'{page}{i+1} / '+['Capacity','Reliability','Costs'][i],23)+text(90,180+i*185,'Current observed metrics and notes',17)
 body+=rect(950,90,18,500);render('page_'+page.lower(),body)
events=[point('old-point','page_h',765,488,300),dict(event_id='scroll',type='pointer_drag',start_ms=680,end_ms=850,frame_id='page_h',points=[[960,150],[960,500]],button='left'),dict(event_id='nav',type='source_change',t_ms=800,from_frame='page_h',to_frame='page_j',source_revision=2)]
r=revision(1,'방금 가리켰던 항목을 다음 작업 자료로 써줘.',[expr('historic','방금 가리켰던 항목',100,600)],events,[frame('page_h'),frame('page_j',2,800)]);case('dp04-held-page','Historic target before navigation',[r],[('historic',1,['H3'])],condition='Frame-owned candidates may protect history; extra production of unused new page costs both parser branches')
r1=revision(1,'왼쪽 첫 그래프와 오른쪽 끝 그래프를 비교해줘.',[expr('keep','왼쪽 첫 그래프',100,500),expr('replace','오른쪽 끝 그래프',700,1200)],[],[frame('nested_graphs')]);r2=revision(2,'첫 번째는 그대로 두고 두 번째는 오른쪽 끝 말고 왼쪽에서 두 번째 그래프로 바꿔줘.',[expr('keep','첫 번째는 그대로',2000,2400),expr('replace','두 번째는 왼쪽에서 두 번째 그래프',2600,3500)],[],[frame('nested_graphs')]);case('dp04-held-correction','Partial correction across same frame and delayed stale replay',[r1,r2],[('keep',1,['G1']),('replace',1,['G4']),('keep',2,['G1']),('replace',2,['G2'])],True,condition='Explicit producer revision/retention may help; semantic split can lose correction scope and costs repeated screen production')
# Raw event overlays render only input event geometry. No expected targets are read.
for f in fixtures:
 for r in f['revisions']:
  for fr in r['frames']:
   parts=[]
   for e in r['events']:
    if e.get('frame_id')!=fr['frame_id']:continue
    if e['type']=='pointer_move':
     x,y=e['x'],e['y'];parts.append(f'<path d="M{x-12} {y} H{x+12} M{x} {y-12} V{y+12}" stroke="#d72d69" stroke-width="3"/>'+text(x+14,y-15,e['event_id'],12))
    elif e.get('points'):
     parts.append('<polyline points="'+' '.join(f'{x},{y}' for x,y in e['points'])+'" fill="none" stroke="#d72d69" stroke-width="3"/>')
   if parts:
    name=f['id']+'-r'+str(r['revision'])+'-'+fr['frame_id']+'-events';original=(ASSET/(fr['frame_id']+'.svg')).read_text();p=ASSET/(name+'.svg');p.write_text(original.replace('</svg>',''.join(parts)+'</svg>'));subprocess.run([NODE,'-e','require(process.argv[1])(process.argv[2]).png().toFile(process.argv[3])',SHARP,str(p),str(p.with_suffix('.png'))],check=True);fr['overlay_image_path']=str(p.with_suffix('.png'));fr['overlay_sha256']=hashlib.sha256(p.with_suffix('.png').read_bytes()).hexdigest();fr['overlay_encoding']='RAW_EVENT_OVERLAY_V1 no inferred targets'
for folder,value in [('fixtures',fixtures),('oracles',oracle)]:
 (BASE/folder).mkdir(exist_ok=True);(BASE/folder/'dp04.json').write_text(json.dumps(value,ensure_ascii=False,indent=2)+'\n')
manifest={f['id']:{'qa01_stimuli':[f['id']+':r'+str(r['revision']) for r in f['revisions']],'qa03_goals':[{'goal_id':f['id']+':grounding','origin_stimulus_id':f['id']+':r1','final_revision':f['revisions'][-1]['revision'],'scope':'GROUNDING_ADOPTION'}]} for f in fixtures};(BASE/'goal_manifests').mkdir(exist_ok=True);(BASE/'goal_manifests/dp04.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'episodes':len(fixtures),'stimuli':sum(len(f['revisions']) for f in fixtures),'referent_sets':sum(len(o['adoptions']) for o in oracle),'native':'Unavailable: actual source is PNG-only'}))
