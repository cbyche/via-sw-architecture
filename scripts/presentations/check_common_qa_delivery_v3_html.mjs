import fs from 'node:fs/promises';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../docs/presentations_files/qa-common-synthetic-v3-20261022');
const html=await fs.readFile(root+'/outputs/01a15dfe/guide.html','utf8');
const script=html.match(/<script>([\s\S]*)<\/script>/)[1];
new vm.Script(script);
// Minimal offline DOM contract harness, not a browser or visual renderer.
class Element {
 constructor(id){this.id=id;this.value=id==='profile'?'TEXT':'';this.checked=false;this._html='';this.textContent='';this.dataset={};this.classes=new Set();this.classList={toggle:(c,b)=>b?this.classes.add(c):this.classes.delete(c)};}
 set innerHTML(v){this._html=v;if(['config','family'].includes(this.id)){const m=v.match(/<option(?: value="([^"]*)")?>(.*?)<\/option>/);this.value=m?(m[1]??m[2]):'';}if(this.id==='list')buttons=[...v.matchAll(/<button class="case" data-id="([^"]+)"/g)].map(m=>{const e=new Element('button');e.dataset.id=m[1];return e;});}
 get innerHTML(){return this._html;}
}
let buttons=[];const els=Object.fromEntries(['profile','config','family','query','errors','count','list','detail'].map(n=>[n,new Element(n)]));
const document={getElementById:id=>els[id],querySelectorAll:s=>{assert.equal(s,'.case');return buttons;}};
const context=vm.createContext({document});vm.runInContext(script,context);
const data=JSON.parse(vm.runInContext('JSON.stringify(D)',context));
const roundDisplay=(n,places)=>vm.runInContext(`roundDisplay(${JSON.stringify(n)},${places})`,context);
assert.equal(roundDisplay(1.1675,3),'1.168');
assert.equal(roundDisplay(-1.1675,3),'-1.168');
const baselineRange=data.range_review.configs['T-BASE'].qa01;
assert(html.includes(roundDisplay(baselineRange.min,3)+'–'+roundDisplay(baselineRange.max,3)));
const checks=[];
for(const profile of ['TEXT','VOICE','DELIVERY']){
 els.profile.value=profile;els.profile.onchange();
 const configs=data.configs.filter(c=>c.stratum===profile);
 for(const c of configs){
  els.config.value=c.id;els.config.onchange();const rows=data.responses.filter(r=>r.config_id===c.id);
  assert.equal(buttons.length,rows.length);buttons[0].onclick();assert(els.detail.innerHTML.includes(rows[0].input_id));
  const last=buttons.at(-1);last.onclick();assert(els.detail.innerHTML.includes(last.dataset.id));assert(last.classes.has('active'));
  els.errors.checked=true;els.errors.onchange();assert.equal(buttons.length,rows.filter(r=>!r.qa07_success).length);
  if(profile==='DELIVERY')assert(els.detail.innerHTML.includes('조건에 맞는 입력이 없습니다'));
  els.errors.checked=false;els.errors.onchange();
  els.family.value=rows[0].family_id;els.family.onchange();assert.equal(buttons.length,rows.filter(r=>r.family_id===rows[0].family_id).length);
  els.family.value='';els.family.onchange();
  els.query.value=rows[0].input_id;els.query.oninput();assert(buttons.some(b=>b.dataset.id===rows[0].input_id));
  els.query.value='zzzz-no-match-739118';els.query.oninput();assert.equal(buttons.length,0);
  els.query.value='';els.query.oninput();
  if(profile==='DELIVERY'){assert(els.detail.innerHTML.includes('고정 정답 제어 fixture'));assert(els.detail.innerHTML.includes('첫 음성 지연 (별도 진단)'));assert(els.detail.innerHTML.includes('응답 소스와 생성 및 전달 이력'));assert(els.detail.innerHTML.includes('terminal'));assert(els.detail.innerHTML.includes('의존'));const par=data.stages.find(x=>x.config_id===c.id&&x.case_id===rows[0].case_id&&x.parents?.length);assert(par);assert(els.detail.innerHTML.includes(par.parents[0]));}
  else assert(els.detail.innerHTML.includes('첫 음성 지연 (별도 진단): 미모델링'));
  if(profile==='VOICE'){assert(els.detail.innerHTML.includes('전체 의미 확정 구간'));assert(els.detail.innerHTML.includes('정밀한 순간은 미작성'));}
  if(c.id==='T-EXEC'){const partial=rows.find(r=>r.first_is_partial);assert(partial);buttons.find(b=>b.dataset.id===partial.input_id).onclick();assert(partial.first_display_s<partial.adopted_at_s);assert(els.detail.innerHTML.includes('부분 안내'));assert(els.detail.innerHTML.includes('첫 표시 t='+roundDisplay(partial.first_display_s,3)));assert(els.detail.innerHTML.includes('전체 의미 확정 t='+roundDisplay(partial.adopted_at_s,3)));assert(els.detail.innerHTML.includes(partial.first_display_node_id));assert(els.detail.innerHTML.includes(partial.full_meaning_adoption_node_id));}
  if(profile==='DELIVERY'){const s03=buttons.find(b=>b.dataset.id.includes('S03-01'));assert(s03);s03.onclick();const r=data.responses.find(x=>x.config_id===c.id&&x.input_id===s03.dataset.id);const par=data.stages.find(x=>x.config_id===c.id&&x.case_id===r.case_id&&x.parents?.length);assert(els.detail.innerHTML.includes(par.parents[0]));assert(els.detail.innerHTML.includes('시작 -6'));assert(!els.detail.innerHTML.includes('&quot;config_id&quot;: &quot;'+(c.id==='D-A'?'D-B':'D-A')+'&quot;'));}
  checks.push({profile,config:c.id,input_count:rows.length,error_count:rows.filter(r=>!r.qa07_success).length,configuration_family_search_error_filters:true,selected_detail:true});
 }
}
assert.equal(data.configs.length,9);assert.equal(html.match(/<section class="profile">/g).length,3);
assert(html.includes('QA01 첫 응답 지연시간 (화면)'));assert(html.includes('미모델링'));assert(html.includes('SYNTHETIC_AUTHORED_NOT_MEASURED'));
assert(html.includes('기준 원장의 수치 범위'));assert(html.includes('512 MB 참고'));assert.equal(data.responses.filter(r=>r.first_is_partial).length,70);
assert(html.includes('560 MB 기반은 512 MB 요구를 이미 48 MB 초과'));
assert(html.includes('상대 비교이며 제품 적합안이 아니다'));
for(const c of data.configs){const r=data.range_review.configs[c.id].qa04;assert(html.includes(`${r.min}–${r.max}<br><small>n=${r.n}</small>`));}
const result={dataset:'COMMON-QA-SYN-3.0',review_revision:'COMMON-QA-SYN-3.0-REVIEW-2',evidence_kind:'SYNTHETIC_AUTHORED_NOT_MEASURED',html_sha256:crypto.createHash('sha256').update(html).digest('hex'),javascript_syntax:'PASS',offline_dom_contract_harness:'PASS',decimal_half_up_display_check:{positive_tie:'1.1675 → 1.168',negative_tie:'-1.1675 → -1.168',status:'PASS'},checks,missing_delivery_receiver_total:null,live_browser_visual:'NOT_COMPLETED',live_browser_reason:'Codex in-app browser rejected file:// URL by security policy (http/https only). No alternate browser or listener workaround attempted.',native_windows_excel_verified:false,actual_model_calls:0,limitations:['Offline minimal DOM harness checks event handlers and rendered HTML strings; it does not verify browser layout or native DOM parsing.']};
await fs.writeFile(root+'/html-validation.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify({configs:checks.length,offline_dom:'PASS',live_browser:'NOT_COMPLETED'}));
