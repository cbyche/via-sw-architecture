import fs from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v8');
const output=path.join(root,'artifacts/outputs/01a15dfe-v8');
const html=await fs.readFile(path.join(output,'guide.html'),'utf8'),ledger=JSON.parse(await fs.readFile(path.join(root,'artifacts/ledger.json')));
const data=JSON.parse(html.match(/<script id="qa-data" type="application\/json">([\s\S]*?)<\/script>/)[1]);
assert.deepEqual(data,ledger);
const script=html.match(/<script>([\s\S]*?)<\/script>/)[1];
class Element{constructor(value=''){this.value=value;this.innerHTML='';this.textContent='';this.handlers={};}addEventListener(event,handler){this.handlers[event]=handler;}}
const elements={'qa-data':new Element(),case:new Element('U01'),config:new Element('DP2-B'),detail:new Element(),'qa7-case':new Element('Q7-C01'),'qa7-detail':new Element()};elements['qa-data'].textContent=JSON.stringify(data);
new vm.Script(script).runInContext(vm.createContext({document:{getElementById:id=>elements[id]}}));
let comparisons=0;for(const cfg of data.configs.slice(1))for(const s of data.scenarios){elements.config.value=cfg.id;elements.case.value=s.id;elements.case.handlers.change();const rendered=elements.detail.innerHTML;assert(rendered.includes(s.text)&&rendered.includes(s.routing)&&rendered.includes('BASE')&&rendered.includes(cfg.id));for(const cid of ['BASE',cfg.id]){const c=data.cases.find(c=>c.config_id===cid&&c.case_id===s.id);for(const q of ['qa03','qa06','qa08'])assert(rendered.includes(c[q].toFixed(3)));for(const e of c.events){assert(rendered.includes(e.id)&&rendered.includes(e.profile)&&rendered.includes(e.status));if(e.L!=null)assert(rendered.includes(e.L.toFixed(3)));}}comparisons++;}
elements.config.handlers.change();assert.equal(comparisons,144);assert(html.includes('WAIT_BOTH')&&html.includes('실제 측정값 아님'));assert(!html.includes('NaN'));
for(const m of html.matchAll(/href="([^"]+)"/g)){assert(!m[1].includes('://'));await fs.access(path.resolve(output,m[1]));}
let qa07Cases=0;for(const c of data.qa07.manifest.cases){elements['qa7-case'].value=c.case_id;elements['qa7-case'].handlers.change();const rendered=elements['qa7-detail'].innerHTML;assert(rendered.includes(c.case_id)&&rendered.includes('UNKNOWN'));for(const o of c.obligations)assert(rendered.includes(o.id)&&rendered.includes(o.stage));qa07Cases++;}assert.equal(qa07Cases,32);
const main=html.match(/<section id="main-summary">([\s\S]*?)<\/section>/)[1];
assert(main.includes('QA08 GB')&&main.includes('QA04 개/과제'));
const mainRows=[...main.matchAll(/<tr>([\s\S]*?)<\/tr>/g)].slice(1);
assert.equal(mainRows.length,10);
const cmp=['BASE','DP2-B','BASE','DP3-B-R1','DP3-B-R2','DP3-B-R3','BASE','DP5-B','BASE','DP44-B'];
mainRows.forEach((match,i)=>{const cells=[...match[1].matchAll(/<td>([\s\S]*?)<\/td>/g)].map(x=>x[1]);assert.equal(cells[10],'미측정');assert.equal(cells[14],data.summary[cmp[i]].qa08_gb.toFixed(3));});
assert(html.includes('QA04 같은 8개 변경 과제')&&html.includes('후속 청구 종료 관측'));
for(const t of data.qa04.tasks)assert(html.includes(t.task_id));
assert(html.includes('계획 160회')&&html.includes('목표 성공 144회')&&html.includes('작성자 제안'));
const report={qa07Cases,status:'PASS',html_sha256:crypto.createHash('sha256').update(html).digest('hex'),embedded_ledger:'PASS',offline_dom_comparisons:comparisons,links:'PASS',physical_browser_visual:'NOT_COMPLETED',limitation:'Offline minimal DOM handler/string test; no native browser layout validation.',actual_model_calls:0};
await fs.writeFile(path.join(root,'artifacts/html-validation.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
