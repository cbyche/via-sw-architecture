"""Authored synthetic documents -> SVG; PNG rendered with sharp, not captured user data."""
from pathlib import Path
import json, html, hashlib, subprocess
ROOT=Path(__file__).resolve().parents[5]
BASE=ROOT/'benchmark/architecture/four_dp_poc_v2'
ASSET=BASE/'assets/screen'
def svg(name, body):
    text=f'<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="650" viewBox="0 0 1000 650"><rect width="1000" height="650" fill="#f1f5f9"/><g font-family="DejaVu Sans,Arial,sans-serif">{body}</g></svg>'
    (ASSET/f'{name}.svg').write_text(text)
def txt(x,y,t,size=18,color='#172033'):
    return f'<text x="{x}" y="{y}" font-size="{size}" fill="{color}">{html.escape(t)}</text>'
def rect(x,y,w,h,color='white'):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="7" fill="{color}" stroke="#94a3b8"/>'
s=txt(35,45,'Regional Performance / similar active project cards',25)
for i in range(12):
    x=35+(i%4)*242;y=90+(i//4)*165
    s+=rect(x,y,225,140)+txt(x+14,y+29,f'D{i+1:02} / Orion',20)+txt(x+14,y+60,['North','South','East'][i//4]+' / Q'+str(i%4+1))+txt(x+14,y+90,f'Revenue {85+(i*7)%29} / active')+txt(x+14,y+118,'Forecast review',15)
s+=txt(35,625,'Visible IDs label document items; pointer event positions are provided separately.',15)
svg('dense',s)
s=txt(35,45,'Sensor readings / irregular plot',25)+rect(65,80,870,480)
s+='<path d="M100 520 L900 520 M100 520 L100 110" fill="none" stroke="#475569" stroke-width="2"/>'
points=[('P1',140,350),('P2',240,160),('P3',350,230),('P4',500,320),('P5',410,380),('P6',650,220),('P7',760,370),('P8',850,140)]
for name,x,y in points:s+=f'<circle cx="{x}" cy="{y}" r="8" fill="#4338ca"/>'+txt(x+12,y-10,name)
s+=txt(35,610,'No native object selection is available. Samples have visible point labels.',16)
svg('graph',s)
for page in ['A','B']:
    s=txt(35,45,f'Quarterly review / page {page}',26)
    for i in range(3):
        y=100+i*160;s+=rect(60,y,830,135)+txt(90,y+34,f'{page}{i+1} / '+['Revenue table','Conversion chart','Retention table'][i],23)+txt(90,y+77,f'Page {page} source material / section {i+1}')+txt(90,y+108,'Quarterly actuals and comparison notes',16)
    s+=rect(950,90,16,470,'#cbd5e1')+rect(950,100 if page=='A' else 380,16,150,'#64748b')
    svg('page_'+page.lower(),s)
s=txt(35,45,'Scenario comparison / three charts',26)
for i in range(3):
    x=40+i*320;s+=rect(x,110,295,420)+txt(x+20,150,f'C{i+1} / '+['Baseline','Expansion','Recovery'][i],22)
    s+=f'<path d="M{x+30} 450 L{x+255} 450 M{x+30} 450 L{x+30} 200" fill="none" stroke="#64748b"/>'
    for j in range(4):s+=f'<rect x="{x+50+j*48}" y="{400-j*35+i*10}" width="28" height="{50+j*35-i*10}" fill="{["#2563eb","#d97706","#059669"][i]}"/>'
svg('correction',s)
node='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node'
sharp='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/sharp'
for f in ASSET.glob('*.svg'):
    subprocess.run([node,'-e','require(process.argv[1])(process.argv[2]).png().toFile(process.argv[3])',sharp,str(f),str(f.with_suffix('.png'))],check=True)
def frame(name,t,rev):
    p=ASSET/f'{name}.png';return {'frame_id':name,'source_id':'document','source_revision':rev,'observed_at_ms':t,'width':1000,'height':650,'coordinate_space':'frame_pixels','image_path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
def exp(i,text,start,end):return {'expression_id':i,'text':text,'start_ms':start,'end_ms':end,'time_uncertainty_ms':20}
def rev(n,text,expressions,events,frames):return {'revision':n,'transcript':text,'expressions':expressions,'events':events,'frames':frames,'native_selection':{'status':'unsupported'},'capture_gaps':[],'permission':'allowed','transcript_ready_ms':max([e['end_ms'] for e in expressions])+100}
def fix(i,description,revisions,late=False):return {'id':i,'dp':'04','description':description,'provenance':'AUTHORED_SYNTHETIC_SVG_RENDERED_TO_PNG; timestamped authored transcript/pointer ledger, not a recorded user study','revisions':revisions,'late_result_replay':late}
dense=rev(1,'이 항목과 이 항목의 분기 실적을 비교해줘.',[exp('dense-first','이 항목',100,500),exp('dense-second','이 항목',900,1300)],[{'event_id':'pointer-1','type':'pointer_move','t_ms':300,'frame_id':'dense','x':375,'y':315,'button':'none'},{'event_id':'pointer-2','type':'pointer_move','t_ms':1100,'frame_id':'dense','x':630,'y':480,'button':'none'}],[frame('dense',0,1)])
graph=rev(1,'이렇게 둘러싼 세 점만 평균을 내줘.',[exp('graph-region','이렇게 둘러싼 세 점',100,1200)],[{'event_id':'loop-1','type':'pointer_path','start_ms':200,'end_ms':1100,'frame_id':'graph','button':'none','points':[[300,180],[580,180],[600,410],[300,420],[300,180]]}],[frame('graph',0,1)])
page=rev(1,'방금 가리킨 이 항목을 다음 작업의 자료로 써줘.',[exp('page-target','방금 가리킨 이 항목',100,600)],[{'event_id':'point-old','type':'pointer_move','t_ms':300,'frame_id':'page_a','x':470,'y':320,'button':'none'},{'event_id':'drag','type':'pointer_drag','start_ms':680,'end_ms':850,'frame_id':'page_a','points':[[958,150],[958,445]],'button':'left'},{'event_id':'page-change','type':'source_change','t_ms':800,'from_frame':'page_a','to_frame':'page_b','source_revision':2}],[frame('page_a',0,1),frame('page_b',800,2)])
r1=rev(1,'왼쪽 그래프와 가운데 그래프를 비교해줘.',[exp('keep-first','왼쪽 그래프',100,500),exp('change-second','가운데 그래프',700,1200)],[],[frame('correction',0,1)])
r2=rev(2,'첫 그래프는 그대로 두고, 두 번째는 가운데 말고 오른쪽 그래프로 바꿔줘.',[exp('keep-first','첫 그래프는 그대로',2000,2500),exp('change-second','두 번째는 가운데 말고 오른쪽 그래프',2700,3600)],[],[frame('correction',0,1)])
fixtures=[fix('dp04-dense','Dense similar item cards with two timed pointer referents',[dense]),fix('dp04-free-region','Free pointer loop encloses exactly three plotted samples',[graph]),fix('dp04-page-change','Pointing precedes a page navigation before transcript finalization',[page]),fix('dp04-partial-correction','Partial target correction retains first binding; delayed old result replay',[r1,r2],True)]
# Common observation presentation: raw event geometry only, never semantic targets.
for fixture in fixtures:
    fixture['observation_encoding']='original PNG plus RAW_EVENT_OVERLAY_V1 from observed event coordinates, identical for A/B'
    for revision in fixture['revisions']:
        for f in revision['frames']:
            parts=[]
            for event in revision['events']:
                if event.get('frame_id')!=f['frame_id']: continue
                color='#c026d3' if event['type']=='pointer_path' else '#dc2626'
                label=html.escape(event['event_id'])
                if event['type']=='pointer_move':
                    x,y=event['x'],event['y']
                    parts.append(f'<g fill="none" stroke="{color}" stroke-width="3"><circle cx="{x}" cy="{y}" r="11"/><path d="M{x-15} {y} H{x+15} M{x} {y-15} V{y+15}"/></g>')
                    parts.append(txt(min(x+18,850),max(y-15,18),label,14,color))
                elif event['type'] in ('pointer_path','pointer_drag'):
                    points=event['points'];d=' '.join(('M' if n==0 else 'L')+f'{x} {y}' for n,(x,y) in enumerate(points))
                    parts.append(f'<path d="{d}" fill="none" stroke="{color}" stroke-width="3" stroke-dasharray="7 4"/>')
                    parts.append(txt(points[0][0]+5,max(points[0][1]-8,18),label,14,color))
            if parts:
                original=Path(f['image_path']).with_suffix('.svg').read_text()
                body='<g aria-label="raw event overlay">'+''.join(parts)+'</g>'
                out=ASSET/(fixture['id']+'-r'+str(revision['revision'])+'-'+f['frame_id']+'-events.svg')
                out.write_text(original.replace('</svg>',body+'</svg>'))
                subprocess.run([node,'-e','require(process.argv[1])(process.argv[2]).png().toFile(process.argv[3])',sharp,str(out),str(out.with_suffix('.png'))],check=True)
                image=out.with_suffix('.png')
                f['overlay_image_path']=str(image);f['overlay_sha256']=hashlib.sha256(image.read_bytes()).hexdigest()
                f['overlay_encoding']='RAW_EVENT_OVERLAY_V1; crosses/path and labels are original events, no semantic region/target annotations'
oracles=[]
for f,expected in zip(fixtures,[[('dense-first',1,['D06']),('dense-second',1,['D11'])],[('graph-region',1,['P3','P4','P5'])],[('page-target',1,['A2'])],[('keep-first',1,['C1']),('change-second',1,['C2']),('keep-first',2,['C1']),('change-second',2,['C3'])]]):
    oracles.append({'id':f['id'],'adoptions':[{'expression_id':e,'revision':r,'targets':ts} for e,r,ts in expected],'rationale':{'dp04-dense':'Pointer-1 falls inside D06 and pointer-2 D11 during the respective expression spans.','dp04-free-region':'The authored closed path geometrically contains P3 P4 P5 and excludes all other plotted points.','dp04-page-change':'The expression and pointing occur on page A; scrollbar drag is later and does not replace the target.','dp04-partial-correction':'Revision 2 explicitly preserves C1 and replaces C2 with C3; a subsequent old-revision proposal must be rejected.'}[f['id']]})
(BASE/'fixtures/dp04.json').write_text(json.dumps(fixtures,ensure_ascii=False,indent=2)+'\n')
(BASE/'oracles/dp04.json').write_text(json.dumps(oracles,ensure_ascii=False,indent=2)+'\n')
