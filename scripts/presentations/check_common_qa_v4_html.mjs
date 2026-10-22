import fs from 'node:fs/promises';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../docs/architecture/12-decisions/decision-packages/current/evaluation/common-qa-v4');
const output=path.join(root,'artifacts/outputs/01a15dfe-v4');
const html=await fs.readFile(path.join(output,'guide.html'),'utf8');
const ledger=JSON.parse(await fs.readFile(path.join(root,'artifacts/ledger.json')));
const data=JSON.parse(html.match(/<script id="qa-data" type="application\/json">([\s\S]*?)<\/script>/)[1]);
const script=html.match(/<script>([\s\S]*?)<\/script>/)[1];
assert.deepEqual(data.summary,ledger.summary);assert.deepEqual(data.deltas,ledger.deltas);
class Element{constructor(value=''){this.value=value;this.innerHTML='';this.textContent='';this.handlers={};}addEventListener(event,handler){this.handlers[event]=handler;}}
const elements={'qa-data':new Element(),case:new Element('U01'),config:new Element('DP2-B'),'case-detail':new Element()};elements['qa-data'].textContent=JSON.stringify(data);
const context=vm.createContext({document:{getElementById:id=>elements[id]}});new vm.Script(script).runInContext(context);
let comparisons=0;for(const config of data.configs.slice(1))for(const s of data.scenarios){elements.config.value=config.id;elements.case.value=s.id;elements.case.handlers.change();const rendered=elements['case-detail'].innerHTML;assert(rendered.includes(s.text));assert(rendered.includes(config.id));assert(rendered.includes('BASE'));assert(rendered.includes('후보 출력은 아직 없음'));for(const cid of ['BASE',config.id]){const r=ledger.cases.find(c=>c.config_id===cid&&c.case_id===s.id);for(const q of ['qa01','qa03','qa05','qa06','first_voice'])assert(rendered.includes(r[q].toLocaleString('en-US',{minimumFractionDigits:q==='qa05'?6:3,maximumFractionDigits:q==='qa05'?6:3,useGrouping:false})));}comparisons++;}
elements.config.handlers.change();assert.equal(comparisons,144);
assert.equal((html.match(/data-config=/g)||[]).length,10);assert(html.includes('실측 아님'));assert(html.includes('33개 지칭'));assert(!html.includes('NaN'));
for(const match of html.matchAll(/href="([^"]+)"/g)){assert(!match[1].includes('://'));await fs.access(path.resolve(output,match[1]));}
const report={status:'PASS',html_sha256:crypto.createHash('sha256').update(html).digest('hex'),syntax:'PASS',embedded_summary:'PASS',matrix_rows:10,offline_dom_comparisons:comparisons,links:'PASS',physical_browser_visual:'NOT_COMPLETED',limitation:'Minimal offline DOM handler/string harness; no actual browser layout or native DOM verification.',actual_model_calls:0};
await fs.writeFile(path.join(root,'artifacts/html-validation.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
