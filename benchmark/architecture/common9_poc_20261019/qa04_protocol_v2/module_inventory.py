"""Audited mapping for this crate, not a general files==modules rule."""
import re
PATHS={'src/main.rs':'crate','src/api.rs':'crate::api','src/context.rs':'crate::context','src/model_wire.rs':'crate::model_wire','src/sources.rs':'crate::sources','src/evidence.rs':'crate::evidence','src/grounding.rs':'crate::grounding','src/handoff.rs':'crate::handoff','src/runtime.rs':'crate::runtime','src/agent_wire.rs':'crate::agent_wire','src/grounding/wire.rs':'crate::grounding::wire','src/grounding/joint.rs':'crate::grounding::joint','src/grounding/producers.rs':'crate::grounding::producers','src/grounding/binding.rs':'crate::grounding::binding','src/input_format.rs':'crate::input_format'}
LIVE_PATHS={
'C01':'Every selected configuration Context.call -> model_wire.encode/decode; actual semantic/producer/receiver calls use migrated wire.',
'C02':'Every selected configuration runtime Agent/control ingress and receipt -> agent_wire; downstream receive decodes the actual submitted request.',
'C03':'Every selected configuration main initial ingestion and Context streamed input -> input_format.normalize. Parent mod registration and ingestion adapters are operational changes.',
'C04':'Every selected configuration actual runtime publication/section edit -> guarded registered displayed source. No evidence/grounding/handoff alternative branch edited.',
'C05':'Every selected configuration actual semantic adoption/publication -> delivered-only provenance panel query. No DP-specific branch edited.',
'C06':'Every selected configuration streamed range withdrawal -> SourceRegistry authority -> existing actual raw/record/read/publication checks. No optional cache-residency edits or dormant DP branches edited.'}

def tokens(text,with_spans=False):
 """Rust lexical tokens sufficient for this audited source: strings keep braces opaque."""
 out=[];i=0;n=len(text)
 def emit(value,start,end):out.append((value,start,end)if with_spans else value)
 while i<n:
  if text[i].isspace():i+=1;continue
  if text.startswith('//',i):
   j=text.find('\n',i);i=n if j<0 else j+1;continue
  if text.startswith('/*',i):
   depth=1;i+=2
   while depth and i<n:
    if text.startswith('/*',i):depth+=1;i+=2
    elif text.startswith('*/',i):depth-=1;i+=2
    else:i+=1
   if depth:raise ValueError('Unclosed Rust block comment')
   continue
  raw=re.match(r'(?:br|r)(#*)"',text[i:])
  if raw:
   end='"'+raw.group(1);j=text.find(end,i+len(raw.group(0)))
   if j<0:raise ValueError('Unclosed Rust raw string')
   emit(text[i:j+len(end)],i,j+len(end));i=j+len(end);continue
  if text[i]=='"'or(text[i]=="'"and re.match(r"'(?:\\.|[^'\\])'",text[i:])):
   quote=text[i];j=i+1
   while j<n:
    if text[j]=='\\':j+=2;continue
    if text[j]==quote:j+=1;break
    j+=1
   emit(text[i:j],i,j);i=j;continue
  word=re.match(r'[A-Za-z_][A-Za-z_0-9]*|[0-9]+',text[i:])
  if word:emit(word.group(0),i,i+len(word.group(0)));i+=len(word.group(0));continue
  emit(text[i],i,i+1);i+=1
 return out

def operational_tokens(text):
 t=tokens(text);out=[];i=0
 while i<len(t):
  if t[i:i+7]==['#','[','cfg','(','test',')',']']:
   j=i+7
   if t[j:j+1]==['pub']:j+=1
   if t[j:j+1]!=['mod']:raise ValueError('Unreviewed cfg(test) item; inspect before operational counting')
   j+=2
   if t[j:j+1]!=['{']:raise ValueError('Unreviewed external cfg(test) module')
   depth=1;j+=1
   while depth and j<len(t):
    depth+=int(t[j]=='{')-int(t[j]=='}');j+=1
   if depth:raise ValueError('Unclosed test module')
   i=j;continue
  out.append(t[i]);i+=1
 return out

def inventory(crate):
 rows=[]
 for p in sorted((crate/'src').rglob('*.rs')):
  rel=str(p.relative_to(crate))
  if rel not in PATHS:raise ValueError('Unreviewed Rust module/file mapping: '+rel)
  # This campaign contains only file modules plus excluded inline test modules.
  live=operational_tokens(p.read_text())
  for i,x in enumerate(live):
   if x=='mod'and i+2<len(live)and live[i+2]=='{':raise ValueError('New inline operational module requires explicit mapping: '+rel)
  if rel!='src/main.rs':
   name=PATHS[rel].rsplit('::',1)[-1];parent=crate/('src/grounding.rs'if rel.startswith('src/grounding/')else'src/main.rs');pt=operational_tokens(parent.read_text())
   if not any(pt[i:i+3]==['mod',name,';']for i in range(len(pt)-2)):raise ValueError('Module lacks actual parent registration: '+rel)
  rows.append({'file':rel,'operational_module_path':PATHS[rel],'inline_test_modules_excluded':True,'mapping_scope':'This audited common9 crate only'})
 return rows
