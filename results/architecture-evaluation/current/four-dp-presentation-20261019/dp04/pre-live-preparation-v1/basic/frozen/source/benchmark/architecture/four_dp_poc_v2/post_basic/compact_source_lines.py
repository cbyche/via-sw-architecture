"""Diagnostic model-input COPY formatter. Original host evidence/guards are never mutated."""
import copy
VERSION='COMPACT_ORIGINAL_LINES_V1_20261018'
EXPLANATION='Source-passage text is displayed as ORIGINAL_LINE_NUMBER| original text. These are original one-based source line numbers, including blank lines. Cite the original numbers; copy fact quote text WITHOUT the numeric prefix. The prefix is display notation, not source content. Summaries and statements are not original numbered lines.'
class FormatError(ValueError):pass

def transform(value):
 result=copy.deepcopy(value);changes=[]
 def visit(v,path):
  if isinstance(v,list):
   for i,x in enumerate(v):visit(x,path+[str(i)])
  elif isinstance(v,dict):
   for k,x in list(v.items()):visit(x,path+[str(k)])
   if not all(k in v for k in ['source_id','revision','start_line','end_line','text']):return
   # Delivered summary/fact text has provenance but is not the contiguous original source.
   if v.get('kind') in ('ACTUALLY_DELIVERED_FACT','ACTUALLY_DELIVERED_ANSWER'):return
   lo,hi,text=v['start_line'],v['end_line'],v['text']
   if not isinstance(text,str):raise FormatError('Non-string source passage at /'+'/'.join(path))
   if not isinstance(lo,int)or isinstance(lo,bool)or not isinstance(hi,int)or isinstance(hi,bool)or lo<1 or hi<lo:raise FormatError('Invalid original range at /'+'/'.join(path))
   if v.get('_original_line_format')==VERSION:
    lines=text.split('\n')
    if len(lines)!=hi-lo+1 or any(not line.startswith(str(lo+i)+'| ')for i,line in enumerate(lines)):raise FormatError('Invalid existing line notation at /'+'/'.join(path))
    return
   lines=text.split('\n')
   if len(lines)!=hi-lo+1:raise FormatError('Original text line count/range mismatch at /'+'/'.join(path))
   formatted='\n'.join(str(lo+i)+'| '+line for i,line in enumerate(lines))
   v['text']=formatted;v['_original_line_format']=VERSION
   changes.append({'path':'/'+'/'.join(path),'source_id':v['source_id'],'revision':v['revision'],'start_line':lo,'end_line':hi,'line_count':len(lines),'before_utf8_bytes':len(text.encode()),'after_utf8_bytes':len(formatted.encode()),'roundtrip_exact':all(line[len(str(lo+i)+'| '):]==lines[i]for i,line in enumerate(formatted.split('\n')))})
 visit(result,[])
 return result,changes
