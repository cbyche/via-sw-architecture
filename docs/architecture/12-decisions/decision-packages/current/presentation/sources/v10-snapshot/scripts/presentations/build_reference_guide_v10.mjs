import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import sharp from 'sharp';
import {Presentation,PresentationFile,FileBlob} from '@oai/artifact-tool';
const [ROOT,BUILD,SOURCE,SCRIPTS=path.join(ROOT,'scripts/presentations')]=process.argv.slice(2);
const SKILL='/Users/changbae/.codex/plugins/cache/openai-primary-runtime/presentations/26.1017.22135/skills/presentations';
const PY='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3';
process.env.RUNTIME_NODE_MODULES='/Users/changbae/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules';
const {Scene,validateScene}=await import(pathToFileURL(path.join(SCRIPTS,'dp3_v8_scene.mjs')));
const {authorScene,svg,drawio,standaloneFigure}=await import(pathToFileURL(path.join(SCRIPTS,'whole_deck_v10_scene_io.mjs')));
const {dp1Structure,dp2Structure,dp4Structure}=await import(pathToFileURL(path.join(SCRIPTS,'legacy_dp_structure_scenes.mjs')));
const {dp5GroundingLoopsPage,dp5TacticFragmentsPage}=await import(pathToFileURL(path.join(SCRIPTS,'dp5_v10_grounding_loops.mjs')));
let scenes=[dp1Structure(Scene,false),dp1Structure(Scene,true),dp2Structure(Scene,false),dp2Structure(Scene,true),dp4Structure(Scene,false),dp4Structure(Scene,true),dp5GroundingLoopsPage(Scene),dp5TacticFragmentsPage(Scene)];
try{const {tacticDetailScenes}=await import(pathToFileURL(path.join(SCRIPTS,'legacy_tactic_detail_fragments.mjs')));scenes.push(...tacticDetailScenes(Scene));}catch(e){if(e.code!=='ERR_MODULE_NOT_FOUND')throw e;if(!process.env.V10_ALLOW_PARTIAL)throw e;}
scenes.sort((a,b)=>a.page-b.page);
for(const d of scenes){
 const r=d.fragment?.replaceBounds;
 d.replacement_regions=r?(Array.isArray(r)?r:[r]).map(r=>[r.x,r.y,r.w,r.h]):d.replacement_regions||({2:[[196,226,1695,367]],3:[[196,226,1695,367]],6:[[196,226,1695,420]],7:[[196,226,1695,420]],17:[[196,220,1695,425]],18:[[196,220,1695,425]]}[d.page]);
 if(!d.replacement_regions)throw Error('Missing replacement region '+d.page);
 // Only the figure region is authored. Original heading, native QA tables and notes remain.
 d.items=d.items.filter(o=>o.kind!=='table'&&(!(o.kind==='text')||d.replacement_regions.some(([x,y,w,h])=>o.x>=x-.3&&o.y>=y-.3&&o.x+o.w<=x+w+.3&&o.y+o.h<=y+h+.3)));
}
await fs.mkdir(BUILD,{recursive:true});await fs.mkdir(path.join(BUILD,'output'),{recursive:true});await fs.mkdir(path.join(BUILD,'diagrams'),{recursive:true});await fs.mkdir(path.join(BUILD,'previews'),{recursive:true});
await fs.copyFile(SOURCE,path.join(BUILD,'source.pptx'));
const source=await PresentationFile.importPptx(await FileBlob.load(path.join(BUILD,'source.pptx')));
const total=source.slides.items.length;
if(total!==36&&!process.env.V10_ALLOW_PARTIAL)throw Error('Expected DP44-completed 36-slide v9');
const checks=scenes.map(validateScene);
await fs.writeFile(path.join(BUILD,'scene-model.json'),JSON.stringify(scenes,null,2));await fs.writeFile(path.join(BUILD,'scene-checks.json'),JSON.stringify(checks,null,2));
const p=Presentation.create({slideSize:{width:1920,height:1080}});
for(const d of scenes){authorScene(p,d);const f=standaloneFigure(d);const [xx,yy,ww,hh]=f.viewBox;await fs.writeFile(path.join(BUILD,'diagrams',d.name+'.svg'),svg(f.scene).replace('width="1920" height="1080" viewBox="0 0 1920 1080"',`width="${ww}" height="${hh}" viewBox="${xx} ${yy} ${ww} ${hh}"`));await fs.writeFile(path.join(BUILD,'diagrams',d.name+'.drawio'),drawio(f.scene));await sharp(await fs.readFile(path.join(BUILD,'diagrams',d.name+'.svg')),{density:144}).png().toFile(path.join(BUILD,'diagrams',d.name+'.png'));}
await(await PresentationFile.exportPptx(p)).save(path.join(BUILD,'authored.pptx'));
execFileSync(PY,[path.join(SCRIPTS,'package_whole_deck_v10.py'),path.join(BUILD,'source.pptx'),path.join(BUILD,'authored.pptx'),path.join(BUILD,'scene-model.json'),path.join(SCRIPTS,'whole_deck_v10_label_map.json'),path.join(BUILD,'candidate.pptx'),path.join(BUILD,'preservation.json')],{env:{...process.env,PYTHONPATH:path.join(ROOT,'scripts/presentations')},stdio:'inherit'});
const {finalizePresentation}=await import(pathToFileURL(path.join(SKILL,'container_tools/artifact_tool_utils.mjs')));
const final=path.join(BUILD,'output',`VIA-DP1-DP5-DP44-reference-style-${total}slides-v10-guide-reviewed-windows.pptx`);
await finalizePresentation({workspaceDir:BUILD,candidatePath:path.join(BUILD,'candidate.pptx'),finalPath:final,explicitTotalSlideCount:total,pythonExecutable:PY,integrityValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_package_integrity.py'),layoutValidatorPath:path.join(SKILL,'container_tools/inspect_presentation_layout_geometry.py'),layoutArgs:['--expected-slide-size-emu','18288000,10287000','--validate-bullet-geometry','--validate-heading-fit'],fontPolicy:{basis:'reference',families:['Calibri'],referencePath:path.join(BUILD,'source.pptx'),referenceSha256:createHash('sha256').update(await fs.readFile(path.join(BUILD,'source.pptx'))).digest('hex')},verifyArtifactToolImport:true,receiptPath:path.join(BUILD,'validation.json')});
const result=await PresentationFile.importPptx(await FileBlob.load(final));
for(let i=0;i<total;i++){const sl=result.slides.items[i];await fs.writeFile(path.join(BUILD,'previews',`slide-${String(i+1).padStart(2,'0')}.png`),new Uint8Array(await(await result.export({slide:sl,format:'png',scale:1})).arrayBuffer()));console.log('Rendered '+(i+1));}
console.log(final);
