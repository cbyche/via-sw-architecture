import fs from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v5');
const output=path.join(root,'artifacts/outputs/01a15dfe-v5');
const html=await fs.readFile(path.join(output,'guide.html'),'utf8'),ledger=JSON.parse(await fs.readFile(path.join(root,'artifacts/ledger.json')));
const data=JSON.parse(html.match(/<script id="qa-data" type="application\/json">([\s\S]*?)<\/script>/)[1]);
assert.deepEqual(data,ledger);
const script=html.match(/<script>([\s\S]*?)<\/script>/)[1];
class Element{constructor(value=''){this.value=value;this.innerHTML='';this.textContent='';this.handlers={};}addEventListener(event,handler){this.handlers[event]=handler;}}
const elements={'qa-data':new Element(),case:new Element('U01'),config:new Element('DP2-B'),detail:new Element()};elements['qa-data'].textContent=JSON.stringify(data);
new vm.Script(script).runInContext(vm.createContext({document:{getElementById:id=>elements[id]}}));
let comparisons=0;for(const cfg of data.configs.slice(1))for(const s of data.scenarios){elements.config.value=cfg.id;elements.case.value=s.id;elements.case.handlers.change();const rendered=elements.detail.innerHTML;assert(rendered.includes(s.text)&&rendered.includes(s.routing)&&rendered.includes('BASE')&&rendered.includes(cfg.id));for(const cid of ['BASE',cfg.id]){const c=data.cases.find(c=>c.config_id===cid&&c.case_id===s.id);for(const q of ['qa03','qa06','qa08'])assert(rendered.includes(c[q].toFixed(3)));for(const e of c.events){assert(rendered.includes(e.id)&&rendered.includes(e.profile)&&rendered.includes(e.status));if(e.L!=null)assert(rendered.includes(e.L.toFixed(3)));}}comparisons++;}
elements.config.handlers.change();assert.equal(comparisons,144);assert(html.includes('WAIT_BOTH')&&html.includes('실제 측정값 아님'));assert(!html.includes('NaN'));
for(const m of html.matchAll(/href="([^"]+)"/g)){assert(!m[1].includes('://'));await fs.access(path.resolve(output,m[1]));}
const report={status:'PASS',html_sha256:crypto.createHash('sha256').update(html).digest('hex'),embedded_ledger:'PASS',offline_dom_comparisons:comparisons,links:'PASS',physical_browser_visual:'NOT_COMPLETED',limitation:'Offline minimal DOM handler/string test; no native browser layout validation.',actual_model_calls:0};
await fs.writeFile(path.join(root,'artifacts/html-validation.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
