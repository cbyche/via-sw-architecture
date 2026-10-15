import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {Presentation,PresentationFile} from '@oai/artifact-tool';

const [repo,build]=process.argv.slice(2);
if(!repo||!build)throw new Error('Usage: generate_dp44_scenario_comparisons.mjs REPO BUILD');
const skill=process.env.VIA_PRESENTATION_SKILL_DIR;
const python=process.env.VIA_RUNTIME_PYTHON;
if(!skill||!python)throw new Error('Missing bundled presentation runtime paths');
const {finalizePresentation}=await import(pathToFileURL(path.join(skill,'container_tools/artifact_tool_utils.mjs')).href);
const source=path.join(repo,'docs/architecture/12-decisions/decision-packages/diagrams');
const v14=JSON.parse(await fs.readFile(path.join(repo,'docs/presentations_files/full-presentation/manifest-v14.json'),'utf8'));
const names=['input','agent','onset','correction'].map(value=>'dp44-scenario-'+value);
const presentation=Presentation.create({slideSize:{width:1920,height:1080}});
const stroke=(color,width=1.7,dashed=false)=>({fill:color==='none'?'none':color,width:color==='none'?0:width,style:dashed?'dashed':'solid'});

function line(slide,name,pts,color,width,dashed=false){
  const xx=pts.map(p=>p[0]),yy=pts.map(p=>p[1]);
  const left=Math.min(...xx),top=Math.min(...yy),w=Math.max(.1,Math.max(...xx)-left),h=Math.max(.1,Math.max(...yy)-top);
  slide.shapes.add({name,geometry:'custom',position:{left,top,width:w,height:h},fill:'none',line:stroke(color,width,dashed),
    customPaths:[{width:w,height:h,commands:pts.map(([x,y],i)=>({[i?'lineTo':'moveTo']:{x:x-left,y:y-top}}))}]});
}

for(const stem of names){
  const scene=JSON.parse(await fs.readFile(path.join(source,stem+'.scene.json'),'utf8'));
  const slide=presentation.slides.add();slide.background.fill='#FFFFFF';
  for(const item of scene.items){
    const name=`${stem} / ${item.id}`;
    if(item.kind==='line'){
      line(slide,name+' / path',item.points,item.color,item.width,item.dashed);
      if(item.arrow){const [x,y]=item.points.at(-1);let j=item.points.length-2;
        while(j>=0&&item.points[j][0]===x&&item.points[j][1]===y)j--;
        if(j>=0){const [px,py]=item.points[j],len=Math.hypot(x-px,y-py),u=(x-px)/len,v=(y-py)/len,b=item.width*6.4,s=item.width*3.2;
          line(slide,name+' / arrow',[[x-u*b-v*s,y-v*b+u*s],[x,y],[x-u*b+v*s,y-v*b-u*s]],item.color,item.width*1.28);}}
    }else if(item.kind==='text'){
      const left=item.x-(item.align==='center'?item.w/2:item.align==='right'?item.w:0);
      item.lines.forEach((content,index)=>{
        const shape=slide.shapes.add({name:name+` / text ${index+1}`,geometry:'textbox',
          position:{left,top:item.y+index*item.leading,width:item.w,height:item.size*1.45},fill:'none',line:stroke('none',0)});
        shape.text=content;
        shape.text.style={typeface:'Calibri',fontSize:item.size,bold:item.bold,color:item.color,
          alignment:item.align,verticalAlignment:'top',wrap:'none',autoFit:'none',insets:{left:0,right:0,top:0,bottom:0}};
      });
    }else if(item.kind==='rect'||item.kind==='store'){
      slide.shapes.add({name,geometry:item.kind==='store'?'can':'rect',position:{left:item.x,top:item.y,width:item.w,height:item.h},
        fill:item.fill,line:stroke(item.stroke,item.line_width??1.7,item.dashed)});
    }else throw new Error('Unsupported scene kind '+item.kind);
  }
  slide.speakerNotes.textFrame.setText(scene.title+'\n'+scene.notes+'\n'+
    '기존 종합 그림과 상세 A/B 구조는 04-44 본문에 보존된다. 이 장은 한 사건의 경로만 확대한다. 설계안은 미선정, 미구현, 미측정이다.\n'+
    'https://github.com/cbyche/via-sw-architecture/blob/main/docs/architecture/12-decisions/decision-packages/04-44-continuous-interaction.md');
}
await fs.mkdir(build,{recursive:true});
const candidate=path.join(build,'candidate.pptx');
const final=path.join(build,'final','VIA-DP44-scenario-comparisons_v1.pptx');
await fs.mkdir(path.dirname(final),{recursive:true});
await(await PresentationFile.exportPptx(presentation)).save(candidate);
await finalizePresentation({workspaceDir:build,candidatePath:candidate,finalPath:final,explicitTotalSlideCount:4,
  pythonExecutable:python,integrityValidatorPath:path.join(skill,'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath:path.join(skill,'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs:['--expected-slide-size-emu','18288000,10287000'],requiredNativeTableOwnerSlides:[],
  fontPolicy:{basis:'reference',families:['Calibri'],referencePath:path.join(repo,'docs/presentations_files/VIA-architecture-presentation_v14.pptx'),
    referenceSha256:v14.output_sha256},
  verifyArtifactToolImport:true,receiptPath:path.join(build,'validation.json')});
console.log(final);
