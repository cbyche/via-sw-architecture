import fs from 'node:fs/promises';
import path from 'node:path';

const QA = [
  ['qa01','QA01 첫 응답 지연시간','초','낮을수록 좋음','입력 제출부터 첫 구체적 내용 표시까지. 오답의 첫 표시시간도 유지한다.'],
  ['qa02','QA02 지칭 대상 식별 정확도','%','높을수록 좋음','표현과 revision별 최초 채택 집합의 완전 일치율이다.'],
  ['qa03','QA03 VIA 처리시간','초','낮을수록 좋음','원래 목표 시작부터 최종 revision 완료까지의 VIA 경로다. 외부 업무와 사용자 대기 구간을 제거한다.'],
  ['qa04','QA04 변경 모듈 수','개/과제','낮을수록 좋음','공통 C01~C06을 적용할 때 수정할 것으로 설정한 운영 모듈 수의 평균이다.'],
  ['qa05','QA05 VIA 모델 사용 비용','USD/묶음','낮을수록 좋음','같은 활동 묶음의 VIA 사용량에 공통 단가를 적용한다. 준비와 미사용 생산 및 갱신도 포함한다.'],
  ['qa06','QA06 최대 로컬 RAM','MB','낮을수록 좋음','같은 순간의 로컬 상태 합계 중 최고값이다. 원격 모델 가중치는 이 profile에 없다.'],
  ['qa07','QA07 요청 의미 해석 정확도','%','높을수록 좋음','입력과 revision별 최초 확정 의미의 필수 항목 전체 일치율이다.']
];
const clean = x => String(x??'').replaceAll('·',' / ');
const esc = x => clean(x).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const cell = x => clean(x).replaceAll('|','\\|').replaceAll('\n','<br>');
const num = (x,n=3) => Number.isFinite(Number(x))?Number(x).toFixed(n):'—';
const percent = x => `${num(100*x,2)}%`;
const metric = (key,x) => ['qa02','qa07'].includes(key)?percent(x):key==='qa05'?`$${num(x,6)}`:num(x,key==='qa06'?2:3);
const sum = xs => xs.reduce((s,x)=>s+x,0);
const avg = xs => xs.length?sum(xs)/xs.length:NaN;
const mdTable = (heads,rows) => `| ${heads.map(cell).join(' | ')} |\n| ${heads.map(()=>'---').join(' | ')} |\n${rows.map(r=>`| ${r.map(cell).join(' | ')} |`).join('\n')}`;
const htmlTable = (heads,rows) => `<div class="table-scroll"><table><thead><tr>${heads.map(h=>`<th>${esc(h)}</th>`).join('')}</tr></thead><tbody>${rows.map(r=>`<tr>${r.map(c=>`<td>${esc(c)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
const relative = (from,to) => path.relative(from,to).split(path.sep).join('/');
const fieldNames={goal:'처리 목표',targets:'대상 집합',conditions:'필수 조건',prior_task_action:'기존 목표 관계',data_source:'내용 출처'};
const prettyMeaning = m => Object.entries(m).map(([k,v])=>`${fieldNames[k]||k}: ${Array.isArray(v)?v.join('; '):v}`).join('\n');
const diffMeaning=(a,b)=>Object.keys(a).filter(k=>JSON.stringify(a[k])!==JSON.stringify(b[k])).map(k=>`${fieldNames[k]||k}: ${JSON.stringify(a[k])} → ${JSON.stringify(b[k])}`).join('\n')||'필수 의미 전체 일치';

function makeView(d) {
  const ids=d.configs.map(c=>c.id), index=Object.fromEntries(d.configs.map(c=>[c.id,c]));
  const population={episodes:d.episodes.length,inputs:d.episodes.flatMap(e=>e.inputs).length,expressions:d.episodes.flatMap(e=>e.inputs.flatMap(i=>i.referents)).length,goals:new Set(d.episodes.flatMap(e=>e.inputs.map(i=>i.goal_id))).size};
  const comparisons=QA.map(([key,name,unit,direction])=>[name,unit,metric(key,d.summary.BASE[key]),metric(key,d.summary.EXEC[key]),metric(key,d.summary.BASE[key]),metric(key,d.summary.REL[key]),direction]);
  const familyRows=d.families.map(f=>{
    const rr=ids.map(c=>d.family_summary.find(x=>x.family===f.id&&x.config===c));
    return [f.id+' '+f.name,f.common_mapping.join(', '),f.overlap?'진행 중 U02':'완료 후 U02',f.second_kind,rr.map(x=>num(x.qa01)).join(' / '),rr.map(x=>percent(x.qa02)).join(' / '),rr.map(x=>num(x.qa03)).join(' / '),rr.map(x=>percent(x.qa07)).join(' / '),`${rr[0].response_n}입력 / ${rr[0].referent_n}표현 / ${rr[0].goal_n}목표`];
  });
  const subgroups=[];
  for(const overlap of [true,false]) for(const config of ids){
    const familyIds=new Set(d.families.filter(f=>f.overlap===overlap).map(f=>f.id));
    const t=d.times.filter(x=>x.config===config&&familyIds.has(x.family_id)&&x.input_id.endsWith('U02'));
    const r=d.responses.filter(x=>x.config===config&&familyIds.has(x.family_id)&&x.input_id.endsWith('U02'));
    const q=d.referents.filter(x=>x.config===config&&familyIds.has(x.family_id)&&x.input_id.endsWith('U02'));
    subgroups.push([overlap?'진행 중 도착 U02':'완료 후 도착 U02',config,t.length,num(avg(t.map(x=>x.qa01_s))),`${q.length?percent(avg(q.map(x=>x.correct))):'비적용'} (${sum(q.map(x=>x.correct))}/${q.length})`,`${percent(avg(r.map(x=>x.qa07)))} (${sum(r.map(x=>x.qa07))}/${r.length})`]);
  }
  const costs=ids.map(config=>{
    const rows=d.costs.filter(c=>c.config===config),via=rows.filter(c=>c.role==='VIA');
    const prepare=via.filter(c=>/사전 생산/.test(c.stage)),update=via.filter(c=>/관계 갱신/.test(c.stage));
    return [config,`$${num(sum(prepare.map(c=>c.cost_usd)),6)}`,`$${num(sum(update.map(c=>c.cost_usd)),6)}`,`$${num(sum(via.filter(c=>!prepare.includes(c)&&!update.includes(c)).map(c=>c.cost_usd)),6)}`,`$${num(d.summary[config].qa05,6)}`,`$${num(d.summary[config].receiver_cost,6)}`,`$${num(d.summary[config].total_cost,6)}`];
  });
  const peaks=ids.map(config=>d.memory.filter(m=>m.config===config).reduce((a,b)=>!a||b.total_mb>a.total_mb?b:a,null));
  const memoryRows=peaks.map(m=>[m.config,m.episode_id,`${num(m.time_s)}초`,num(m.core_mb,2),num(m.state_mb,2),num(m.model_buffers_mb,2),num(m.read_mb,2),num(m.relation_mb+m.producer_mb,2),num(m.total_mb,2)]);
  const examples=['SYN-F06-01','SYN-F11-01','SYN-F12-03'].map(id=>d.episodes.find(e=>e.id===id)).filter(Boolean);
  return {ids,index,population,comparisons,familyRows,subgroups,costs,memoryRows,examples};
}

function exampleMarkdown(d,e) {
  const u=e.inputs[1], family=d.families.find(f=>f.id===e.family_id);
  const rows=d.configs.map(c=>{
    const r=d.responses.find(x=>x.config===c.id&&x.input_id===u.id),t=d.times.find(x=>x.config===c.id&&x.input_id===u.id),refs=d.referents.filter(x=>x.config===c.id&&x.input_id===u.id),g=d.goals.find(x=>x.config===c.id&&x.goal_id===u.goal_id);
    return [c.id,r.first_content,num(t.qa01_s),num(r.full_meaning_adopted_s-t.start_s),refs.length?`${sum(refs.map(x=>x.correct))}/${refs.length}`:'비적용',r.qa07?'일치':'불일치',num(g.via_s),diffMeaning(r.expected_meaning,r.adopted_meaning)];
  });
  const blocks=d.configs.map(c=>{
    const r=d.responses.find(x=>x.config===c.id&&x.input_id===u.id);
    return `<details><summary>${c.id}의 최초 확정 의미와 근거</summary>\n\n\`\`\`json\n${JSON.stringify(r.adopted_meaning,null,2)}\n\`\`\`\n\n${r.error_reason}.\n\n</details>`;
  });
  const wanted=e.family_id==='F11'?['D1V1','D2V1']:e.family_id==='F12'?['KNOWLEDGE','TASK']:['DISPLAY','FEEDBACK1'];
  const sourceText=wanted.map(suffix=>d.sources.find(s=>s.id===`${e.id}-SRC-${suffix}`)).filter(Boolean).map(s=>`- **${s.id}**: ${s.content}`).join('\n');
  const interpretation=e.family_id==='F06'?'대상 집합이 같더라도 현재 목적과 기존 목표의 정정 관계는 별도 필수 의미다. QA02의 성공을 QA07의 성공으로 바꿔 읽을 수 없다.':e.family_id==='F11'?'첫 안내에서 낮은 비용을 선택하겠다는 규칙을 구체적으로 알릴 수 있다. 실제 어느 견적이 낮은지는 읽기 반환 뒤 확정하므로 첫 표시와 전체 의미 확정의 시각을 나누어 읽는다.':'후속 일반 질문에는 채점할 지칭 표현이 없어 QA02 분모를 늘리지 않는다. 답변 내용과 함께 원래 자료 작성 유지가 QA07의 필수 의미이며 두 목표의 완료도 각각 남는다.';
  return `### ${e.id}: ${family.name}\n\n**U01** “${e.inputs[0].text}”\n\n**U02** “${u.text}”\n\n${interpretation}\n\n**U02 공통 정답**\n\n\`\`\`text\n${prettyMeaning(u.meaning)}\n\`\`\`\n\n${mdTable(['구성','첫 구체적 내용','QA01 초','전체 의미 확정까지 초','QA02','QA07','관련 목표 QA03 초','최초 채택의 차이'],rows)}\n\n시간 열은 입력별 QA01과 목표별 QA03을 구별한다. 정정의 QA03에는 U01부터 남은 경로가 포함되며, 위 시각은 실제 계측 timestamp가 아니다.\n\n${blocks.join('\n\n')}\n\n**대상과 조건을 확인하는 원자료**\n\n${sourceText}\n`;
}

function browserApp() {
  const d=JSON.parse(document.getElementById('dataset').textContent),byId=Object.fromEntries(d.episodes.map(e=>[e.id,e])),sources=Object.fromEntries(d.sources.map(s=>[s.id,s]));
  const $=id=>document.getElementById(id),fmt=(v,n=3)=>Number(v).toFixed(n),esc=s=>String(s??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])),array=a=>a.length?a.join(', '):'해당 없음';
  let page=0,filtered=[],selected=null;
  for(const f of d.families){const o=document.createElement('option');o.value=f.id;o.textContent=f.id+' '+f.name;$('family').append(o);}
  for(const c of d.configs){const o=document.createElement('option');o.value=c.id;o.textContent=c.id+' / '+c.label;$('config').append(o);}
  function matches(r){
    const q=$('search').value.trim().toLocaleLowerCase(),e=byId[r.episode_id],u=e.inputs.find(i=>i.id===r.input_id),mode=$('error').value;
    return (!$('family').value||r.family_id===$('family').value)&&(!$('config').value||r.config===$('config').value)&&(!mode||(mode==='ERROR'?r.error!=='NONE':r.error===mode))&&(!q||[r.input_id,e.project,u.text,r.first_content,r.error_reason].join(' ').toLocaleLowerCase().includes(q));
  }
  function renderRows(){
    filtered=d.responses.filter(matches);page=Math.min(page,Math.max(0,Math.ceil(filtered.length/30)-1));
    $('count').textContent=filtered.length+'개 기록 / '+(filtered.length?page+1:0)+'페이지';$('previous').disabled=page===0;$('next').disabled=(page+1)*30>=filtered.length;
    $('records').innerHTML=filtered.slice(page*30,(page+1)*30).map(r=>{const t=d.times.find(t=>t.input_id===r.input_id&&t.config===r.config),e=byId[r.episode_id];return '<tr><td><button class="case-link" data-key="'+esc(r.config+'|'+r.input_id)+'">'+esc(r.input_id)+'</button></td><td>'+esc(r.config)+'</td><td>'+esc(e.project)+'</td><td>'+fmt(t.qa01_s)+'</td><td><span class="badge '+(r.error==='NONE'?'ok':'fail')+'">'+(r.error==='NONE'?'일치':r.error==='TARGET'?'대상 오류':'의미 오류')+'</span></td><td>'+esc(r.first_content)+'</td></tr>';}).join('');
    if(!filtered.length)$('records').innerHTML='<tr><td colspan="6">조건에 맞는 기록이 없습니다.</td></tr>';
  }
  function show(key){
    selected=key;const [config,inputId]=key.split('|'),r=d.responses.find(x=>x.config===config&&x.input_id===inputId),e=byId[r.episode_id],u=e.inputs.find(i=>i.id===inputId),t=d.times.find(x=>x.config===config&&x.input_id===inputId),refs=d.referents.filter(x=>x.config===config&&x.input_id===inputId),goals=d.goals.filter(x=>x.config===config&&x.episode_id===e.id);
    const fields={goal:'처리 목표',targets:'대상 집합',conditions:'필수 조건',prior_task_action:'기존 목표 관계',data_source:'내용 출처'};
    const meaning=Object.keys(fields).map(k=>'<tr><th>'+fields[k]+'</th><td>'+esc(Array.isArray(r.expected_meaning[k])?r.expected_meaning[k].join('\n'):r.expected_meaning[k])+'</td><td class="'+(JSON.stringify(r.expected_meaning[k])!==JSON.stringify(r.adopted_meaning[k])?'changed':'')+'">'+esc(Array.isArray(r.adopted_meaning[k])?r.adopted_meaning[k].join('\n'):r.adopted_meaning[k])+'</td></tr>').join('');
    $('detail').innerHTML='<div class="detail-head"><div><p class="eyebrow">선택 기록</p><h3>'+esc(config+' / '+inputId)+'</h3></div><span class="badge '+(r.error==='NONE'?'ok':'fail')+'">QA07 '+(r.qa07?'일치':'불일치')+'</span></div><p>'+esc(e.history)+'</p><div class="dialogue">'+e.inputs.map(i=>'<div class="utterance '+(i.id===u.id?'current':'')+'"><strong>'+esc(i.id.split('-').pop())+'</strong><p>'+esc(i.text)+'</p></div>').join('')+'</div><p class="response"><strong>첫 구체적 내용</strong><br>'+esc(r.first_content)+'</p><div class="mini-grid"><div>입력 도착<strong>'+fmt(t.start_s)+'초</strong></div><div>첫 표시<strong>'+fmt(t.first_s)+'초</strong></div><div>QA01<strong>'+fmt(t.qa01_s)+'초</strong></div><div>전체 의미 확정<strong>'+fmt(r.full_meaning_adopted_s)+'초</strong></div></div><p class="muted">절대 시각은 episode 시작을 0으로 둔 작성 경로입니다. QA01은 이 입력의 도착부터 계산하며 첫 표시가 전체 의미 확정보다 빠를 수 있습니다.</p><h4>정답과 최초 확정 의미</h4><div class="table-scroll"><table class="meaning"><thead><tr><th>항목</th><th>공통 정답</th><th>최초 채택</th></tr></thead><tbody>'+meaning+'</tbody></table></div><p>'+esc(r.error_reason)+'</p><h4>QA02 표현별 판정</h4>'+(refs.length?'<div class="table-scroll"><table><thead><tr><th>표현</th><th>정답 집합</th><th>최초 집합</th><th>판정</th></tr></thead><tbody>'+refs.map(x=>'<tr><td>'+esc(x.phrase)+'</td><td>'+esc(array(x.expected))+'</td><td>'+esc(array(x.adopted))+'</td><td>'+(x.correct?'일치':'불일치')+'</td></tr>').join('')+'</tbody></table></div>':'<p>이 입력은 채점할 지칭 표현이 없습니다. QA07 입력 분모에는 포함됩니다.</p>')+'<h4>같은 episode의 목표 완료 경로</h4><div class="table-scroll"><table><thead><tr><th>목표</th><th>포함 입력</th><th>전체 경과</th><th>QA03 VIA 경로</th></tr></thead><tbody>'+goals.map(g=>'<tr><td>'+esc(g.goal_id)+'</td><td>'+esc(array(g.members))+'</td><td>'+fmt(g.wall_s)+'초</td><td>'+fmt(g.via_s)+'초</td></tr>').join('')+'</tbody></table></div><h4>이 입력에서 사용할 수 있는 원자료</h4>'+u.source_ids.map(id=>{const s=sources[id];return s?'<details><summary>'+esc(s.id+' / '+s.kind)+'</summary><p>'+esc(s.content)+'</p></details>':'';}).join('')+'<p class="muted">'+esc(e.evidence_notes)+'</p>';
    $('detail').hidden=false;
  }
  ['family','config','error','search'].forEach(id=>$(id).addEventListener(id==='search'?'input':'change',()=>{page=0;renderRows();}));
  $('previous').addEventListener('click',()=>{page--;renderRows();});$('next').addEventListener('click',()=>{page++;renderRows();});
  $('records').addEventListener('click',event=>{const btn=event.target.closest('button[data-key]');if(btn){show(btn.dataset.key);$('detail').scrollIntoView({behavior:'smooth',block:'start'});}});
  document.querySelectorAll('[data-example]').forEach(btn=>btn.addEventListener('click',()=>{show(btn.dataset.example);$('detail').scrollIntoView({behavior:'smooth',block:'start'});}));
  renderRows();show('BASE|SYN-F06-01-U02');
}

export async function renderReport(dataset,{docDir,deliveryDir}) {
  const d=dataset,v=makeView(d),p=v.population;
  await fs.mkdir(docDir,{recursive:true});await fs.mkdir(deliveryDir,{recursive:true});
  const summaryHeads=['QA','단위','DP2 A / BASE','DP2 B / EXEC','DP5 A / BASE','DP5 B / REL','방향'];
  const familyHeads=['사례군','COMMON 주제','U02 시점','관계','QA01 초','QA02','QA03 초','QA07','구성별 분모'];
  const subgroupHeads=['조건','구성','입력 수','QA01 초','QA02 일치/표현','QA07 일치/입력'];
  const costHeads=['구성','사전 생산','관계 갱신','그 밖의 VIA','QA05 VIA 합계','수신 Agent','전체 합계'];
  const memoryHeads=['구성','최고값 episode','시점','공통 MB','요청 상태 MB','모델 버퍼 MB','읽기 MB','관계와 생산 MB','동시 합 MB'];
  const b=d.summary.BASE,x=d.summary.EXEC,r=d.summary.REL;
  const changesRows=d.changes.map(c=>[c.id,c.title,...v.ids.map(id=>c.configs[id].modules.length),c.acceptance]);
  const relHtml=relative(docDir,path.join(deliveryDir,'VIA-QA-common-guide.html'));
  const relWorkbook=relative(docDir,path.join(deliveryDir,'VIA-QA-common-synthetic.xlsx'));
  const assumptionsRows=[
    ['의미 모델',d.assumptions.model_name,'특정 실제 모델의 성능이나 가격을 대표하지 않음'],
    ['동시 호출 슬롯',d.assumptions.max_concurrent_semantic_calls,'대기 중인 읽기와 새 해석의 겹침을 허용하는 공통 자원'],
    ['대화 실행 직렬 구간','BASE와 REL의 대화형 의미 판단 및 상세 준비 호출은 같은 실행 순서를 유지','EXEC는 입력별 실행 상태와 공통 호출 슬롯을 함께 사용'],
    ['REL 요청 전 준비',`episode 시작 ${d.assumptions.production_start_s??-8}초부터 첫 입력 0초 전까지`,'이 준비 여유를 갖는 profile이며 생산 비용과 동시 RAM은 포함'],
    ['REL 갱신 시점',d.assumptions.update_policy||'두 입력의 모든 목표 종료 후 유휴 구간','현재 요청 지연을 줄이는 대신 종료 후 비용과 RAM을 함께 부담'],
    ['모델 호출 시간',`${d.assumptions.call_startup_s}초 + 입력/${d.assumptions.prefill_tokens_per_second} + 출력/${d.assumptions.decode_tokens_per_second}`,'같은 작업량에는 같은 계산식'],
    ['입력과 출력 단가',`입력 $${d.assumptions.input_usd_per_million}/M, 출력 $${d.assumptions.output_usd_per_million}/M`,'비용 원장의 입력 및 출력 토큰에 적용'],
    ['음성 요약 생성',`$${d.assumptions.audio_usd_per_second}/초`,'음성 준비 비용이며 실제 듣기 시작시간이 아님'],
    ['후속 입력 trigger',`진행 중 첫 표시 +${d.assumptions.overlap_input_gap_s}초 / 완료 후 +${d.assumptions.sequential_input_gap_s}초`,'같은 상대 trigger이고 구성별 절대 시각은 다를 수 있음'],
    ['외부 Agent 업무',`${d.assumptions.external_agent_s}초`,'QA03에서 외부 구간만 0으로 투영하고 비용은 별도 열'],
    ['처리 경로','initial/correction은 Agent 위임, addition/independent는 VIA 직접 답변','추가 조회와 일반 질문에 존재하지 않는 외부 Agent 업무를 넣지 않음'],
    ['EXEC 요청 상태 RAM',`실행 상태당 ${d.assumptions.request_state_mb}MB`,'동시에 살아 있는 실행 상태 수를 반영하며 입력 수를 일괄 최고값으로 더하지 않음'],
    ['정확도 오류 배정',`${d.error_rules.length}개 구성별 오류 규칙`,'자료와 잘못된 채택 내용을 연결한 작성 가정이며 빈도의 실증 근거는 아님']
  ];
  const interpretation=[
    `DP2 B는 BASE 대비 QA01이 ${num(b.qa01-x.qa01)}초, QA03이 ${num(b.qa03-x.qa03)}초 짧다. 이 묶음에서는 별도 판단과 진행 중 읽기의 겹침이 시간을 줄인다. QA07은 ${percent(b.qa07)}에서 ${percent(x.qa07)}로 달라지고 VIA 비용은 $${num(x.qa05-b.qa05,6)}, 최고 RAM은 ${num(x.qa06-b.qa06,2)}MB 증가한다.`,
    `DP5 B는 BASE 대비 QA01이 ${num(b.qa01-r.qa01)}초, QA03이 ${num(b.qa03-r.qa03)}초 짧다. 준비된 관계의 조회가 원문 재구성을 줄이는 가정이다. QA02는 ${percent(b.qa02)}에서 ${percent(r.qa02)}로 바뀌지만 QA07은 ${percent(b.qa07)}에서 ${percent(r.qa07)}로 낮아진다. 대상 연결과 현재 목적 적용을 같은 정확성으로 취급하지 않는다.`,
    `QA04는 DP2에서 ${num(b.qa04)}개와 ${num(x.qa04)}개로 같고, DP5 B는 ${num(r.qa04)}개다. 차이는 구간 철회 때 관계 의존성을 바꾸는 C06 가정에서 발생한다. 공통 모듈로 해결할 수 있으면 이 차이는 없어질 수 있다.`
  ];
  const migrationRows=[
    ['모집단','DP2 200입력 / DP5 100입력',`${p.episodes} episode, ${p.inputs}입력, ${p.expressions}표현, ${p.goals}목표`,'공통 입력과 목표를 새로 구성했으므로 이전 평균과 직접 개선율을 계산하지 않음'],
    ['공통 A','각 DP에서 서로 다른 사례와 A값','BASE 한 벌을 두 비교에 동일 참조','DP2 A와 DP5 A는 같은 행이며 독립 표본 두 벌이 아님'],
    ['시간','DP2 독립 질문 후속 생성 1.2초 / DP5 후속 준비 0.4초 차이 가정','작업량과 의존 그래프의 공통 계산식','기존 고정 격차를 계승하지 않음'],
    ['비용','DP2 별도 COMMON-28 USD / DP5 수신자 포함 정규화 단위','새 공통 묶음의 VIA USD, 수신자 USD와 전체 USD','이전 정규화 비용을 USD에 환산하거나 합산하지 않음'],
    ['변경과 메모리','DP별 작성 수준과 profile 차이','C01~C06, 같은 시각 상태 합계의 최고값','실제 diff나 RSS 검사 완료를 뜻하지 않음'],
    ['새 QA07','입력 의미 필드의 범위와 표현이 달랐음','대상 포함 필수 의미 전체의 최초 확정 일치','우선순위와 점수는 새로 만들지 않음']
  ];
  const markdown=`# DP2와 DP5의 공통 QA 결과와 사례 해설\n\n> **설명용 합성 비교 / ${d.version}.** 모든 사례, 채택 결과, 시간, 사용량과 상태 크기는 작성 가정이다. 실제 후보 또는 모델 실행 결과가 아니며 실제 시험을 수행했다는 근거로 제시하지 않는다.\n\n공통 QA를 같은 사용자 활동에 적용한 결과다. **DP2 A와 DP5 A는 동일한 BASE 기록**을 사용한다. [대화형 설명서](${relHtml})에서 사례와 구성별 오류를 살펴보고, [워크북](${relWorkbook})에서 계산 원장을 확인할 수 있다. 정의는 [공통 계약](01-common-contract.md), 변경과 자원 가정은 [설계 근거](02-change-and-resource-rationale.md), 이전 자료와의 관계는 [이관 및 확장 규칙](03-migration-and-extension.md)을 따른다.\n\n## 1. 같은 기준으로 읽는 대표값\n\n${mdTable(summaryHeads,v.comparisons)}\n\n한 구성은 ${p.episodes}개 episode와 ${p.inputs}개 입력이다. QA02는 ${p.expressions}개 지칭 표현, QA03은 ${p.goals}개 목표, QA04는 ${d.changes.length}개 변경 과제를 사용한다. 비용은 같은 ${p.episodes}개 episode의 준비부터 종료 후 처리까지를 합산한다. BASE를 두 번 합산하지 않으므로 고유 구성은 ${v.ids.length}개다.\n\n${interpretation.map(t=>'- '+t).join('\n')}\n\n오류율은 처리 구조에서 자동으로 도출된 값이 아니라 작성한 채택 사례의 집계다. 이 결과로 구조의 필연적 정확도 우위나 통계적 유의성을 주장하지 않는다. 기존 우선순위와 새 QA07의 미합의 상태를 유지하며, 여기서 새 대안 채택이나 점수를 부여하지 않는다.\n\n## 2. 무엇을 평가했는가\n\n${mdTable(['QA','단위와 경계'],QA.map(([,name,,,definition])=>[name,definition]))}\n\nQA02와 QA07은 같은 대상 오류를 함께 반영할 수 있다. 이를 독립적인 두 번의 정확성 개선으로 합산하지 않는다. 첫 구체적 안내, 전체 의미 최초 확정과 최종 목표 완료는 서로 다른 사건이다. QA01의 빠른 안내가 QA07 성공이나 QA03 완료를 대신하지 않는다.\n\n## 3. 사례군과 조건별 결과\n\n각 값의 순서는 **BASE / EXEC / REL**이다. 사례군당 10개 변형과 20개 입력이며 표현 및 목표 수는 장면에 따라 다르다. COMMON ID는 기존 시나리오와의 주제 연결이고 동일 실행이나 coverage 실적을 뜻하지 않는다.\n\n${mdTable(familyHeads,v.familyRows)}\n\n다음은 후속 입력 U02만 고른 진단이다. 진행 중 도착과 완료 후 도착을 구분해 DP2의 차이가 실제로 가정된 구간을 확인한다. QA03은 U02 입력 수와 목표 수가 일치하지 않으므로 이 표에 섞지 않았다.\n\n${mdTable(subgroupHeads,v.subgroups)}\n\n명시 ID와 이미 확인한 관계는 양안의 재사용 대조다. 준비된 관계는 미리 생산한 내용을 활용하고, dirty와 unproduced에서는 최신 원본 보완을 수행한다. F09의 새 검토 메모는 외부 원자료 갱신이며 별도 미채점 사용자 발화를 추가하지 않는다. F11은 읽기 결과에 의존하고, F12의 독립 질문에는 지칭 표현이 없다.\n\n## 4. 세 가지 입력을 따라가기\n\n${v.examples.map(e=>exampleMarkdown(d,e)).join('\n')}\n\n## 5. 시간과 정확도 차이의 가정\n\n${mdTable(['항목','공통 설정','해석'],assumptionsRows)}\n\n시간의 인과는 입력량과 읽기, 모델 호출, 의존 및 자원 순서에 있다. DP2 B는 새 의미 판단을 먼저 시작해도 최종 대상에 필요한 조회 반환을 기다린다. DP5 B는 준비된 관계를 쓸 수 있는 경우 재구성을 줄이지만 갱신과 미사용 생산을 부담한다. 독립 설명 생성에 대안 이름만으로 고정 시간을 더하거나 빼지 않는다.\n\nU01에서 오답을 채택한 뒤 U02가 정답으로 작성된 경우는 같은 접근 가능 원본을 다시 판단한 가정이다. 이전 오답은 최초 판정에 그대로 남고 이후 정답으로 소급 삭제하지 않는다.\n\n정확도 차이의 가정은 오류 원장에 별도로 공개한다. TARGET은 대상 연결 오류로 QA02와 QA07에 함께 영향을 줄 수 있고, MEANING_ONLY는 대상이 맞아도 조건 또는 기존 목표 관계가 달라 QA07만 실패하는 경우다. 인계 이후 누락이나 외부 Agent의 업무 품질을 이 두 QA에 넣지 않는다.\n\n## 6. 비용, 변경과 최대 메모리\n\n${mdTable(costHeads,v.costs)}\n\n관계 사전 생산에는 끝내 사용되지 않은 생산도 포함한다. 전체 비용은 VIA와 수신자 비용의 합이고 QA05 대표값은 VIA다. 수신자에게 설정한 공통 업무 비용이 같다고 외부 업무가 무료라는 뜻은 아니다.\n\n${mdTable(['과제','같은 변경 요구','BASE','EXEC','REL','공통 완료 조건'],changesRows)}\n\n${mdTable(memoryHeads,v.memoryRows)}\n\n메모리는 구성요소별 최고값을 더한 수치가 아니다. 위 episode와 시점에 동시에 생존한 상태를 합한 전체 최고값이다. cloud profile의 로컬 VIA 상태이며 목표 온디바이스 Omni 전체 메모리나 RSS 실측과 직접 비교하지 않는다.\n\n## 7. 이전 자료에서 무엇이 바뀌었는가\n\n${mdTable(['항목','이전 DP별 자료','공통판','비교 시 유의점'],migrationRows)}\n\n원래 DP2와 DP5 자료는 보존한다. 이번 결과는 이전 수치에 비율을 곱해 맞춘 보정이 아니라 공통 입력, 목표, 사용량과 계산 경계로 다시 작성한 세대다. 두 세대의 차이를 성능 개선으로 표시하지 않는다.\n\n## 8. 다른 DP를 추가하는 방법\n\n1. 새 DP의 A/B가 바꾸는 책임과 상태 및 실행 의존을 구성 registry에 등록한다. 기존 DP와 독립적인 축인지 확인한다.\n2. 같은 BASE를 쓸 수 있으면 원본 행을 공유한다. 주변 구조가 달라지면 별도 context와 비교 쌍을 둔다.\n3. 기존 QA의 의미와 사건 경계를 유지하며 내부 단계를 연결한다. 부가 진단이 QA를 대체하지 않는다.\n4. 새 기능을 다룰 시나리오가 필요하면 corpus 판본을 올리고 기존 DP도 동일 새 묶음으로 다시 산출한다. 새 DP만 사례를 추가해 평균을 바꾸지 않는다.\n5. 기본안과 tactic 및 복수 DP 조합은 별도 구성으로 둔다. 두 개별 개선량을 더해 2B5B 결과를 만들지 않는다.\n6. 시나리오, 정답, 가정과 계산 원장을 함께 버전 관리하고 JSON, 워크북과 설명서를 다시 생성한다. 실제 실행 자료는 별도 근거 종류와 dataset으로 추가한다.\n`;
  await fs.writeFile(path.join(docDir,'04-results-and-walkthrough.md'),clean(markdown).replace(/[ \t]+$/gm,''));
  const embed={version:d.version,configs:d.configs,families:d.families,episodes:d.episodes,sources:d.sources,responses:d.responses,referents:d.referents,times:d.times,goals:d.goals};
  const scriptData=JSON.stringify(embed).replaceAll('·',' / ').replaceAll('<','\\u003c').replaceAll('\u2028','\\u2028').replaceAll('\u2029','\\u2029');
  const html=`<!doctype html><html lang="ko"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>VIA 공통 QA 비교 | DP2와 DP5</title><style>
  :root{--ink:#183247;--muted:#5e7180;--line:#dbe4e8;--navy:#193c53;--teal:#167a79;--paper:#f4f7f8;--warn:#9b4f1c}*{box-sizing:border-box}body{margin:0;background:var(--paper);color:var(--ink);font:15px/1.65 -apple-system,BlinkMacSystemFont,"Apple SD Gothic Neo","Noto Sans KR",sans-serif}a{color:#146a80}header{background:var(--navy);color:white;padding:48px max(24px,calc((100vw - 1360px)/2)) 36px}header p{max-width:1050px;color:#d9e5ed}.eyebrow{font-size:12px;text-transform:uppercase;letter-spacing:.12em;font-weight:700;margin:0 0 8px;color:var(--teal)}header .eyebrow{color:#83d2cc}h1{font-size:36px;letter-spacing:-.04em;line-height:1.2;margin:8px 0 18px}h2{font-size:24px;letter-spacing:-.03em;line-height:1.35;margin:0 0 18px}h3{font-size:19px;margin:4px 0 12px}h4{font-size:16px;margin:24px 0 12px}p{margin:10px 0 16px}.notice{border-left:3px solid #efbf85;padding:10px 16px;background:#ffffff10;color:#fff1db;max-width:1100px}.tags{display:flex;gap:10px;flex-wrap:wrap;margin-top:22px}.tags span{background:#ffffff12;border:1px solid #ffffff30;border-radius:100px;padding:6px 14px;font-size:13px}nav{position:sticky;top:0;background:#fffefcef;backdrop-filter:blur(8px);padding:12px max(24px,calc((100vw - 1360px)/2));border-bottom:1px solid var(--line);z-index:4;display:flex;gap:24px;flex-wrap:wrap}nav a{text-decoration:none;font-weight:650;font-size:13px}main{max-width:1408px;margin:auto;padding:28px 24px 64px}section{background:white;border:1px solid var(--line);border-radius:14px;margin:0 0 24px;padding:28px;scroll-margin-top:70px}.muted{color:var(--muted);font-size:13px}.reading{max-width:1100px}.table-scroll{overflow:auto;border:1px solid var(--line);border-radius:8px;margin:18px 0}table{width:100%;border-collapse:collapse;font-size:13px}th{text-align:left;background:#eaf0f3;font-weight:700;color:#29495e;padding:12px 13px;white-space:nowrap}td{padding:12px 13px;border-top:1px solid var(--line);vertical-align:top;min-width:80px}tbody tr:nth-child(even){background:#f9fbfc}#summary td:first-child{font-weight:700;min-width:240px}#summary td:nth-child(n+3):nth-child(-n+6){font-variant-numeric:tabular-nums;white-space:nowrap;font-weight:650}.insights{display:grid;grid-template-columns:repeat(3,1fr);gap:16px}.insights article{background:#f2f7f8;padding:20px;border-radius:9px}.insights p{font-size:14px;margin-bottom:0}.controls{display:grid;grid-template-columns:1.4fr 1fr 1fr 1.6fr;gap:12px;margin:20px 0}label{font-size:12px;font-weight:700;color:var(--muted)}select,input{display:block;width:100%;margin-top:6px;font:inherit;color:var(--ink);border:1px solid #b6c8d1;border-radius:7px;padding:10px 12px;background:white}button{font:inherit;cursor:pointer;border:1px solid #abc2ce;border-radius:7px;background:white;padding:8px 12px;color:var(--navy)}button:hover{background:#eaf3f5}button:focus-visible,input:focus-visible,select:focus-visible,a:focus-visible{outline:3px solid #43aaa2;outline-offset:3px}button:disabled{opacity:.45;cursor:default}.pager{display:flex;align-items:center;justify-content:space-between;gap:12px}.pager-buttons{display:flex;gap:8px}.case-link{border:0;padding:0;background:none;color:#11667a;text-align:left;font-size:12px;text-decoration:underline;text-underline-offset:3px;white-space:nowrap}.badge{display:inline-block;border-radius:100px;padding:3px 9px;font-size:12px;white-space:nowrap}.ok{background:#e5f4ee;color:#267354}.fail{background:#fff0e3;color:#9b4f1c}.detail-head{display:flex;justify-content:space-between;align-items:center}.dialogue{display:grid;gap:10px}.utterance{padding:14px 16px;border-left:3px solid #c7d4dc;background:#f4f7f9;border-radius:4px}.utterance.current{border-left-color:var(--teal);background:#edf7f5}.utterance p{margin:4px 0}.response{padding:16px;background:#eef2f8;border-radius:7px}.mini-grid{display:grid;grid-template-columns:repeat(4,1fr);gap:10px}.mini-grid>div{padding:14px;border:1px solid var(--line);border-radius:8px;font-size:12px;color:var(--muted)}.mini-grid strong{display:block;font-size:21px;color:var(--ink);font-variant-numeric:tabular-nums;margin-top:4px}.meaning td{white-space:pre-line;min-width:260px}.changed{background:#fff0e3!important;color:#7d3b16}.examples{display:flex;gap:10px;flex-wrap:wrap}.examples button{text-align:left;padding:12px 16px;flex:1;min-width:220px}details{border-bottom:1px solid var(--line);padding:12px 0}summary{cursor:pointer;color:#25546e;font-size:13px;overflow-wrap:anywhere}details p{font-size:13px;white-space:pre-wrap;padding:10px 0 0}footer{font-size:12px;color:var(--muted);padding:12px 4px}code{background:#edf1f3;padding:2px 5px;border-radius:3px;font-size:12px}@media(max-width:900px){h1{font-size:29px}.insights{grid-template-columns:1fr}.controls{grid-template-columns:1fr 1fr}.mini-grid{grid-template-columns:1fr 1fr}section{padding:20px}nav{gap:15px}}@media print{nav,.controls,.pager,.examples,#explorer{display:none}header{background:white;color:#183247;padding:20px}header p{color:#334}.notice{color:#654;background:#fff4e8}main{max-width:none;padding:0}section{break-inside:avoid;border-radius:0}table{font-size:10px}.table-scroll{overflow:visible}}
  </style></head><body><header><p class="eyebrow">VIA Architecture / ${esc(d.version)}</p><h1>같은 장면, 같은 QA로 읽는<br>DP2와 DP5의 구조 선택</h1><p>새 입력을 어느 실행에서 해석할지, 과거 관계를 언제 생산할지 바꿔 보았습니다. 같은 사용자 의도와 원자료에서 시간, 정확도와 자원 부담이 어떻게 연결되는지 살펴봅니다.</p><div class="notice"><strong>설명용 합성 데이터.</strong> 사례와 채택 결과, 시간, 비용과 메모리는 작성 가정으로 계산했습니다. 실제 후보 또는 모델 시험 결과가 아닙니다.</div><div class="tags"><span>${p.episodes}개 episode</span><span>${p.inputs}개 입력</span><span>${p.expressions}개 지칭 표현</span><span>${p.goals}개 목표</span><span>${v.ids.length}개 고유 구성</span></div></header><nav><a href="#summary">대표값</a><a href="#families">공통 장면</a><a href="#conditions">조건별 결과</a><a href="#explorer">사례 원장</a><a href="#cost">비용과 자원</a><a href="#assumptions">가정과 확장</a></nav><main>
  <section id="summary"><p class="eyebrow">01 / Shared comparison</p><h2>공통 A는 한 번만 계산합니다</h2><p class="reading">BASE는 DP2 A + DP5 A, EXEC는 DP2 B + DP5 A, REL은 DP2 A + DP5 B입니다. 두 DP의 A 열은 같은 원본 기록을 참조합니다. 단위와 분모가 다른 QA는 각각의 공통 원장으로 집계합니다.</p>${htmlTable(summaryHeads,v.comparisons)}<div class="insights">${interpretation.map((t,i)=>`<article><strong>${['DP2: 실행 분리','DP5: 관계 사전 생산','변경 범위의 대조'][i]}</strong><p>${esc(t)}</p></article>`).join('')}</div><p class="muted">오류 빈도는 작성한 채택 사례의 집계입니다. 통계적 우위나 필연적 정확도 차이를 뜻하지 않습니다. QA02와 QA07의 같은 대상 오류를 독립 효과로 합산하지 않으며, 새 QA07 우선순위와 점수는 여기서 정하지 않습니다.</p><p><a href="VIA-QA-common-synthetic.xlsx">전체 워크북</a> <span aria-hidden="true"> / </span> <a href="${esc(relative(deliveryDir,path.join(docDir,'04-results-and-walkthrough.md')))}">상세 해설 문서</a></p></section>
  <section id="families"><p class="eyebrow">02 / Common scenes</p><h2>한 QA에 서로 다른 분모가 생기는 이유</h2><p>각 행은 10개 변형입니다. 값은 BASE / EXEC / REL 순서입니다. 정정은 같은 목표를 갱신하고, 추가 요청과 독립 질문은 새 목표를 만듭니다.</p>${htmlTable(familyHeads,v.familyRows)}<p class="muted">COMMON ID는 주제 대응입니다. 기존 COMMON-28 또는 common9의 실행 건수와 결과를 계승하지 않습니다. F12 후속 질문에는 지칭이 없지만 최초 의미와 원래 업무 유지 판정은 있습니다.</p></section>
  <section id="conditions"><p class="eyebrow">03 / Conditions</p><h2>진행 중인 입력과 순차 입력을 나눠 봅니다</h2><p>두 번째 입력 U02만 비교한 진단입니다. 원래 목표의 경로가 포함되는 QA03을 입력별 평균에 섞지 않았습니다.</p>${htmlTable(subgroupHeads,v.subgroups)}<div class="examples"><button data-example="REL|SYN-F06-01-U02"><strong>목적 변경</strong><br>대상은 맞아도 의미가 다른 경우</button><button data-example="EXEC|SYN-F11-01-U02"><strong>결과 의존 정정</strong><br>첫 안내와 최종 대상 확정의 간격</button><button data-example="BASE|SYN-F12-03-U02"><strong>독립 질문</strong><br>답변과 기존 목표 유지의 분리</button></div></section>
  <section id="explorer"><p class="eyebrow">04 / Trace an input</p><h2>입력에서 정답과 최초 채택까지</h2><p>사례군, 구성과 오류 종류로 좁힌 뒤 입력 ID를 선택하면 원자료, 첫 응답, 최초 의미와 시간 경로를 확인할 수 있습니다.</p><div class="controls"><label>사례군<select id="family"><option value="">모든 사례군</option></select></label><label>구성<select id="config"><option value="">세 구성 모두</option></select></label><label>판정<select id="error"><option value="">전체</option><option value="ERROR">오류만</option><option value="TARGET">대상 오류</option><option value="MEANING_ONLY">의미만 오류</option><option value="NONE">일치</option></select></label><label>입력 또는 프로젝트 검색<input id="search" type="search" placeholder="예: 설치비, F06, 오로라"></label></div><div class="pager"><span id="count" class="muted" aria-live="polite"></span><div class="pager-buttons"><button id="previous">이전</button><button id="next">다음</button></div></div><div class="table-scroll"><table><thead><tr><th>입력 ID</th><th>구성</th><th>프로젝트</th><th>QA01 초</th><th>최초 의미</th><th>첫 구체적 내용</th></tr></thead><tbody id="records"></tbody></table></div></section>
  <section id="detail" hidden aria-live="polite"></section>
  <section id="cost"><p class="eyebrow">05 / Cost and state</p><h2>준비와 미사용 생산까지 같은 묶음에 포함합니다</h2>${htmlTable(costHeads,v.costs)}<p class="muted">QA05는 VIA 비용이며 수신 Agent와 전체 비용을 함께 표시합니다. 생산해 놓고 쓰지 않은 관계도 비용 원장에 남습니다.</p><h3>최고 메모리를 만든 같은 시점의 상태</h3>${htmlTable(memoryHeads,v.memoryRows)}<p class="muted">공통 기반, 요청 상태, 버퍼와 관계 상태가 동시에 생존한 합계입니다. 원격 모델 가중치를 포함한 온디바이스 제품 메모리나 RSS 실측이 아닙니다.</p><h3>같은 변경 과제</h3>${htmlTable(['과제','변경 요구','BASE','EXEC','REL','공통 완료 조건'],changesRows)}</section>
  <section id="assumptions"><p class="eyebrow">06 / Assumptions and extension</p><h2>가정을 바꾸면 모든 DP를 함께 다시 계산합니다</h2>${htmlTable(['항목','공통 설정','해석'],assumptionsRows)}<p class="reading">DP2 B는 현재 읽기와 새 판단을 겹칠 수 있지만 결과가 필요한 최종 대상은 반환까지 기다립니다. DP5 B의 준비된 관계는 조회로 재사용하고, 무효하거나 생산되지 않은 관계는 원본으로 보완합니다. 이미 확인한 사실은 A에서도 재사용합니다.</p><p class="reading">U01 오답 뒤 U02를 정답으로 작성한 경우는 동일 원본을 다시 판단한 가정입니다. 이전 최초 오답을 이후 정답으로 소급 삭제하지 않습니다.</p><h3>이전 자료와의 차이</h3>${htmlTable(['항목','이전 DP별 자료','공통판','비교 시 유의점'],migrationRows)}<h3>후속 DP 등록</h3><ol><li>새 A/B의 책임, 상태와 실행 의존을 구성 registry에 등록합니다.</li><li>같은 주변 구조이면 BASE를 공유하고, 조건이 달라지면 별도 비교 context를 둡니다.</li><li>새 시나리오가 필요하면 corpus 판본을 올려 DP2와 DP5도 같은 묶음으로 다시 산출합니다.</li><li>기본안, tactic과 복수 DP 동시 변경은 구별합니다. 개별 개선량을 합산하지 않습니다.</li><li>원자료와 정답, profile과 계산 원장을 함께 버전 관리하고 모든 표를 다시 생성합니다.</li></ol></section>
  <footer>${esc(d.version)} / ${esc(d.created_on)} / 입력, 최초 채택, 시간과 비용 원장에서 생성한 설명서. BASE는 두 비교가 공유합니다.</footer></main><script type="application/json" id="dataset">${scriptData}</script><script>(${browserApp.toString()})();</script></body></html>`;
  await fs.writeFile(path.join(deliveryDir,'VIA-QA-common-guide.html'),clean(html));
  return {markdown:path.join(docDir,'04-results-and-walkthrough.md'),html:path.join(deliveryDir,'VIA-QA-common-guide.html'),records:d.responses.length,examples:v.examples.map(e=>e.id)};
}
