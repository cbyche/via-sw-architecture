// Lightweight DOM harness only: no browser, networking, models, or rendering claim.
const fs=require('fs'),vm=require('vm'),assert=require('assert');
class Element{
 constructor(tag,text=''){this.tagName=tag;this.children=[];this._text=String(text);this._value='';this.dataset={};this.events={};this.classList={toggle(){}};}
 set textContent(v){this._text=v==null?'':String(v);this.children=[]}get textContent(){return this._text+this.children.map(x=>x.textContent??String(x)).join('')}
 append(...xs){this.children.push(...xs)}replaceChildren(...xs){this.children=[];this._text='';this._value='';this.append(...xs)}
 get options(){return this.children}set value(v){this._value=v}get value(){return this.children.some(x=>x.value===this._value)?this._value:(this.children[0]?.value||'')}
 get selectedIndex(){return this.children.findIndex(x=>x.value===this.value)}set selectedIndex(v){this._value=this.children[v]?.value||''}
 addEventListener(kind,fn){this.events[kind]=fn}
}
const results=[];
for(const filename of process.argv.slice(2)){
 const html=fs.readFileSync(filename,'utf8'),matches=[...html.matchAll(/<script([^>]*)>([\s\S]*?)<\/script>/g)];
 const raw=matches.find(x=>x[1].includes('application/json'))[2],evidence=JSON.parse(raw),script=matches.find(x=>!x[1].includes('application/json'))[2];
 const nodes=new Map();for(const id of ['phase','dp','variant','run','overview','selection','interaction','calls','metrics','raw','summary','prev','next','evidence'])nodes.set(id,new Element(['phase','dp','variant','run'].includes(id)?'select':'div'));
 nodes.get('evidence').textContent=raw;const tabs=['interaction','calls','metrics','raw'].map(id=>{const e=new Element('button');e.dataset.tab=id;return e});
 const document={getElementById:id=>nodes.get(id),createElement:tag=>new Element(tag),querySelectorAll:()=>tabs};
 function Option(text,value){const e=new Element('option',text);Object.defineProperty(e,'value',{value,writable:true});return e}
 const context=vm.createContext({document,Option,console});new vm.Script(script).runInContext(context);
 const initialPhase=nodes.get('phase').value,effectiveIds=new Set((evidence.effective_selection?.selection||[]).map(x=>x.run_id));const initiallySelected=evidence.runs.filter(x=>!initialPhase||(initialPhase==='@EFFECTIVE_MAIN_120'?effectiveIds.has(x.record.run_id):x.phase_directory===initialPhase));assert.strictEqual(nodes.get('run').options.length,initiallySelected.length);if(effectiveIds.size){assert.strictEqual(initialPhase,'@EFFECTIVE_MAIN_120');assert(initiallySelected.every(x=>effectiveIds.has(x.record.run_id)));assert(initiallySelected.filter(x=>x.record.dp==='05').every(x=>x.record.phase==='BASIC_CORRECTED'))}else if(evidence.runs.some(x=>x.record.phase==='BASIC'))assert(initiallySelected.every(x=>x.record.phase==='BASIC'));
 assert.strictEqual(nodes.get('overview').textContent.includes('미완료 미리보기'),evidence.limited_preview!==null&&evidence.limited_preview!==undefined);
 assert(nodes.get('raw').textContent.includes('실제 input.json'));
 assert(nodes.get('calls').textContent.includes('실제 provider response JSON'));assert(nodes.get('calls').textContent.includes('실제 model request: system/user 원문'));
 assert(nodes.get('summary').textContent.includes('평행 동치 내용 진단'));
 const wanted=evidence.runs[evidence.runs.length-1];nodes.get('phase').value=wanted.phase_directory;nodes.get('dp').value=wanted.record.dp;nodes.get('variant').value=wanted.record.variant;vm.runInContext('choose()',context);assert(nodes.get('selection').textContent.includes(wanted.record.variant));
 const images=[];function visit(n){if(n.tagName==='img')images.push(n);for(const c of n.children||[])visit(c)}visit(nodes.get('interaction'));
 if(wanted.frames.length){assert(images.length===wanted.frames.length);assert(images.every(i=>i.src.startsWith('data:image/png;base64,')))}
 const fail=evidence.runs.find(x=>x.record.events.some(e=>e.kind==='GOAL_FAILED'));
 if(fail){nodes.get('phase').value=fail.phase_directory;nodes.get('dp').value=fail.record.dp;nodes.get('variant').value=fail.record.variant;vm.runInContext('choose()',context);nodes.get('run').value=fail.record.run_id;vm.runInContext('render()',context);assert(nodes.get('interaction').textContent.includes('GOAL_FAILED'));assert(nodes.get('raw').textContent.includes('GOAL_FAILED'))}
 tabs[1].onclick();nodes.get('next').onclick();nodes.get('prev').onclick();
 results.push({file:filename,runs:evidence.runs.length,png_assets:Object.keys(evidence.assets).length,selection_and_raw_response:true,failure_visible:!!fail,status:'PASS',scope:'STATIC_DOM_HARNESS_NOT_RENDERED_BROWSER'});
}
process.stdout.write(JSON.stringify({checks:results,provider_calls:0,candidate_runs:0},null,2)+'\n');
