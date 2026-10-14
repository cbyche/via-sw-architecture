"""Apply the v11 necessity slide's native header to five standalone visual alternatives."""
from pathlib import Path
import copy
import json
import sys
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named

repo,b=map(Path,sys.argv[1:])
source=repo/'docs/presentations_files/VIA-architecture-presentation_v11.pptx'
assert F.digest(source.read_bytes())=='176d5f006a0bfa7f1c0dc28a4db7137e081438d3f7a52a223fee063f725d853f'
src=F.read(source)
template=src[F.ordered(src)[1][1]]
parts=F.read(b/'authored.pptx')
variants=json.loads((repo/'docs/presentations_files/necessity-options-v12/prompts.json').read_text())['variants']
for page,(_,part) in enumerate(F.ordered(parts),1):
    root=E.fromstring(parts[part]);st=root.find('p:cSld/p:spTree',F.NS)
    for name in ['title','rule',*['stage-'+str(i) for i in range(5)],'page']:
        node=copy.deepcopy(named(template,'v6-header-'+name))
        if name=='page':
            ts=node.findall('.//a:t',F.NS);assert len(ts)==1
            ts[0].text=f'{page}안 / 5   {variants[page-1]["title"]}'
            xfrm=node.find('p:spPr/a:xfrm',F.NS)
            xfrm.find('a:off',F.NS).set('x',str(1450*9525))
            xfrm.find('a:ext',F.NS).set('cx',str(420*9525))
        st.append(node)
    for i,prop in enumerate(st.findall('.//p:cNvPr',F.NS),2):prop.set('id',str(i))
    root.find('p:cSld',F.NS).set('name','necessity-option-v12-'+str(page))
    parts[part]=F.xml(root)
for name in list(parts):
    if name.endswith('.xml'):parts[name]=W.normalize_xml(name,parts[name])
report=W.policy_report(parts)
assert not report['violations'],report
F.write(b/'candidate.pptx',parts)
manifest={'base':str(source.relative_to(repo)),'base_sha256':F.digest(source.read_bytes()),
          'base_slide_count':33,'base_page':2,'base_unchanged':True,'output_slide_count':5,
          'output':'docs/presentations_files/VIA-necessity-options_v12.pptx',
          'variants':[{'page':i+1,'key':v['key'],'title':v['title'],'image':'assets/'+v['key']+'.png'} for i,v in enumerate(variants)],
          'font_policy':report,'selection_status':'USER_REVIEW_OPTIONS','native_powerpoint_verified':False}
(b/'manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(report,ensure_ascii=False))
