import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {Presentation, PresentationFile, FileBlob} from '@oai/artifact-tool';

// Run a copy in a private build directory linked to the bundled node_modules.
// The existing draw.io/SVG scenes remain the content and geometry source.
const [repoArg,buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_dp_editable_pptx.mjs REPO BUILD_DIR');
const repo=await fs.realpath(repoArg), build=await fs.realpath(buildArg);
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const python=process.env.VIA_RUNTIME_PYTHON;
const modules=process.env.RUNTIME_NODE_MODULES;
if (![skill,python,modules].every(value=>path.isAbsolute(value??''))) {
  throw new Error('Set absolute VIA_PRESENTATION_SKILL_DIR, VIA_RUNTIME_PYTHON and RUNTIME_NODE_MODULES paths.');
}
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const font='Apple SD Gothic Neo';
const extraction=String.raw`
import sys,json,hashlib,re
from pathlib import Path
root=Path(sys.argv[1]);sys.path.insert(0,str(root/'scripts/architecture'))
import generate_dp_background_slides as bg
import generate_dp_comparison_slides as cmp
scenes=[]
# Python 3.12's compensated sum changes last-bit annotation widths only.
def normalized_svg(value):
    return re.sub(r'-?\d+\.\d+',lambda m:format(float(m[0]),'.8f').rstrip('0').rstrip('.'),value)
for n in range(41,46):
    for kind in ['background','comparison']:
        s=getattr(bg,'slide'+str(n))() if kind=='background' else cmp.Comparison(n)
        if kind=='comparison':
            for side,x in enumerate([140,1010]):getattr(cmp,'graph'+str(n))(s,x,side)
        prefix='docs/architecture/12-decisions/decision-packages/diagrams' if kind=='background' else 'docs/presentations_files/dp-comparison'
        source=prefix+'/'+s.slug+'.svg'
        assert normalized_svg((root/source).read_text())==normalized_svg(s.svg()),source+' is out of sync'
        scenes.append(dict(number=n,kind=kind,slug=s.slug,title=s.caption,items=s.items,source=source,sha256=hashlib.sha256((root/source).read_bytes()).hexdigest()))
print(json.dumps(scenes,ensure_ascii=False))
`;
const scenes=JSON.parse(execFileSync(python,['-c',extraction,repo],{encoding:'utf8',maxBuffer:8*1024*1024}));
await fs.writeFile(path.join(build,'source-scenes.json'),JSON.stringify(scenes,null,2));

const stroke=(color,width=1.7,dashed=false)=>({fill:color==='none'?'none':color,width:color==='none'?0:width,style:dashed?'dashed':'solid'});
function polyline(slide,name,pts,color,width,dashed=false) {
  const xs=pts.map(p=>p[0]),ys=pts.map(p=>p[1]);
  const left=Math.min(...xs),top=Math.min(...ys),w=Math.max(.1,Math.max(...xs)-left),h=Math.max(.1,Math.max(...ys)-top);
  return slide.shapes.add({name,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',line:stroke(color,width,dashed),
    customPaths:[{width:w,height:h,commands:pts.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
}
function arrowhead(slide,name,pts,color,width) {
  const [ex,ey]=pts.at(-1);
  let previous=pts.length-2;
  while(previous>=0 && pts[previous][0]===ex && pts[previous][1]===ey) previous--;
  if(previous<0)return;
  const [px,py]=pts[previous],length=Math.hypot(ex-px,ey-py),u=(ex-px)/length,v=(ey-py)/length;
  const back=width*6.4,side=width*3.2;
  polyline(slide,name,[[ex-u*back-v*side,ey-v*back+u*side],[ex,ey],[ex-u*back+v*side,ey-v*back-u*side]],color,width*1.28);
}
function nativeScene(presentation,scene) {
  const slide=presentation.slides.add();slide.background.fill='#FFFFFF';
  for(const item of scene.items) {
    const name=`${scene.slug} / ${item.id}`;
    if(item.kind==='line') {
      polyline(slide,name+' / path',item.points,item.color,item.width,item.dashed);
      if(item.arrow)arrowhead(slide,name+' / arrow',item.points,item.color,item.width);
      continue;
    }
    if(item.kind==='text') {
      const left=item.x-(item.align==='center'?item.w/2:item.align==='right'?item.w:0);
      // Separate lines keep the exact authored line breaks and baseline spacing.
      for(const [index,content] of item.lines.entries()) {
        const shape=slide.shapes.add({name:name+` / text ${index+1}`,geometry:'textbox',
          position:{left,top:item.y+index*item.leading,width:item.w,height:item.size*1.45},fill:'none',line:stroke('none',0)});
        shape.text=content;
        shape.text.style={typeface:font,fontSize:item.size,bold:item.bold,color:item.color,
          alignment:item.align,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
      }
      continue;
    }
    const position={left:item.x,top:item.y,width:item.w,height:item.h};
    if(item.kind==='note') {
      const w=item.w,h=item.h;
      slide.shapes.add({name,geometry:'custom',position,fill:item.fill,line:stroke(item.stroke),
        customPaths:[{width:w,height:h,commands:[{moveTo:{x:0,y:0}},{lineTo:{x:w-18,y:0}},
          {lineTo:{x:w,y:18}},{lineTo:{x:w,y:h}},{lineTo:{x:0,y:h}},{close:{}}]}]});
      polyline(slide,name+' / fold',[[item.x+item.w-18,item.y],[item.x+item.w-18,item.y+18],[item.x+item.w,item.y+18]],item.stroke,1.7);
    } else if(item.kind==='store') {
      slide.shapes.add({name,geometry:'can',position,fill:item.fill,line:stroke(item.stroke),
        adjustmentList:[{name:'adj',formula:'val 20000'}]});
    } else if(item.kind==='rect') {
      slide.shapes.add({name,geometry:'rect',position,fill:item.fill,line:stroke(item.stroke,1.7,item.dashed)});
    } else throw new Error(`Unsupported scene object: ${item.kind}`);
  }
  const base='https://github.com/cbyche/via-sw-architecture/blob/main/';
  const documents={41:'04-41-request-resolution-control.md',42:'04-42-lifecycle-ownership.md',
    43:'04-43-request-interpretation.md',44:'04-44-continuous-interaction.md',45:'04-45-memory-and-context.md'};
  slide.speakerNotes.textFrame.setText(`${scene.title}\n${base}docs/architecture/12-decisions/decision-packages/${documents[scene.number]}\n${base}${scene.source}\n`+
    (scene.kind==='comparison'?'수치와 원형 점수는 형식 검토용 예상 예시이며 실측 또는 대안 선정 결과가 아니다. 양안 조건은 본문과 그림의 비교 예시 조건을 따른다.':'공통 문제와 설계 고려 사항의 배경이며 특정 설계안의 선택을 뜻하지 않는다.'));
  return slide;
}

const plans=[
  {slug:'VIA-DP-background-and-comparison-41-45',scenes,destination:'docs/presentations_files/VIA-DP-background-and-comparison-41-45.pptx'},
  {slug:'VIA-DP-background-41-45',scenes:scenes.filter(s=>s.kind==='background'),destination:'docs/presentations_files/dp-background/VIA-DP-background-41-45.pptx'},
  {slug:'VIA-DP-comparison-41-45',scenes:scenes.filter(s=>s.kind==='comparison'),destination:'docs/presentations_files/dp-comparison/VIA-DP-comparison-41-45.pptx'},
];
const checkPackage=String.raw`
import sys,json,zipfile,re
import xml.etree.ElementTree as E
scenes=json.loads(open(sys.argv[2]).read());ns={'p':'http://schemas.openxmlformats.org/presentationml/2006/main','a':'http://schemas.openxmlformats.org/drawingml/2006/main'}
with zipfile.ZipFile(sys.argv[1]) as z:
    files=sorted([n for n in z.namelist() if re.fullmatch(r'ppt/slides/slide\d+\.xml',n)],key=lambda n:int(re.search(r'(\d+)\.xml',n)[1]))
    assert len(files)==len(scenes),(len(files),len(scenes))
    stats=[]
    for file,scene in zip(files,scenes):
        root=E.fromstring(z.read(file));texts=[n.text or '' for n in root.findall('.//a:t',ns)]
        required=[line for item in scene['items'] if item['kind']=='text' for line in item['lines']]
        assert texts==required,(scene['slug'],'text order or content changed')
        assert not root.findall('.//p:pic',ns),scene['slug']+' contains a flattened image'
        native=len(root.findall('.//p:sp',ns)); paths=len(root.findall('.//a:custGeom',ns))
        assert paths>0 and native>50,(scene['slug'],native,paths)
        stats.append(dict(slug=scene['slug'],nativeShapes=native,nativeTextLines=len(texts),nativePaths=paths,images=0))
    print(json.dumps(stats))
`;
const stats=[];
for(const plan of plans) {
  const presentation=Presentation.create({slideSize:{width:1920,height:1080}});
  for(const scene of plan.scenes)nativeScene(presentation,scene);
  const draft=path.join(build,`${plan.slug}.candidate.pptx`),final=path.join(build,'final',`${plan.slug}.pptx`);
  await fs.mkdir(path.dirname(final),{recursive:true});
  await (await PresentationFile.exportPptx(presentation)).save(draft);
  await finalizePresentation({workspaceDir:build,candidatePath:draft,finalPath:final,explicitTotalSlideCount:plan.scenes.length,
    pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
    layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
    layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-heading-fit'],
    requiredNativeTableOwnerSlides:[],fontPolicy:{basis:'design',families:[font]},verifyArtifactToolImport:true,
    receiptPath:path.join(build,`${plan.slug}.validation.json`)});
  const selected=path.join(build,`${plan.slug}.scenes.json`);await fs.writeFile(selected,JSON.stringify(plan.scenes));
  const editability=JSON.parse(execFileSync(python,['-c',checkPackage,final,selected],{encoding:'utf8'}));
  const imported=await PresentationFile.importPptx(await FileBlob.load(final));
  for(const [index,slide] of imported.slides.items.entries()) {
    const png=await imported.export({slide,format:'png',scale:1});
    const preview=path.join(build,'renders',plan.slug,`${String(index+1).padStart(2,'0')}-${plan.scenes[index].slug}.png`);
    await fs.mkdir(path.dirname(preview),{recursive:true});
    await fs.writeFile(preview,new Uint8Array(await png.arrayBuffer()));
  }
  const destination=path.join(repo,plan.destination);
  await fs.copyFile(final,destination);
  stats.push({file:plan.destination,slides:plan.scenes.length,editability});
  console.log(`PASS: ${plan.slug}, ${plan.scenes.length} editable slides, no slide images`);
}
await fs.writeFile(path.join(build,'editability.json'),JSON.stringify(stats,null,2));
