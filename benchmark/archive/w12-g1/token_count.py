"""공식 Qwen tokenizer + chat template로 비용 원장을 만든다. 모델 가중치 불필요.
사용: python token_count.py --tokenizer-dir PATH
PATH에는 pinned tokenizer.json 및 tokenizer_config.json이 있어야 한다.
설치: pip install tokenizers jinja2 (실제 사용 버전은 원장에 기록)
"""
from __future__ import annotations
import argparse, hashlib, importlib.metadata, json
from pathlib import Path
TOKENIZER_SHA256='aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4'

def main() -> None:
    ap=argparse.ArgumentParser();ap.add_argument('--tokenizer-dir',required=True,type=Path);args=ap.parse_args()
    try:
        from tokenizers import Tokenizer
        from jinja2 import Environment, StrictUndefined
    except ImportError as e:
        raise SystemExit('NOT_COUNTED: install tokenizers and jinja2; no approximate token counts will be emitted') from e
    f=args.tokenizer_dir/'tokenizer.json';cfg=args.tokenizer_dir/'tokenizer_config.json'
    if not f.is_file() or not cfg.is_file():raise SystemExit('NOT_COUNTED: both official tokenizer files required')
    sha=hashlib.sha256(f.read_bytes()).hexdigest()
    if sha!=TOKENIZER_SHA256:raise SystemExit('NOT_COUNTED: tokenizer hash mismatch; review the evidence lock')
    config=json.loads(cfg.read_text(encoding='utf-8'));template=config['chat_template']
    if not isinstance(template,str):raise SystemExit('NOT_COUNTED: this runner expects the single official template')
    env=Environment(undefined=StrictUndefined)
    def fail(message):raise ValueError(message)
    env.globals['raise_exception']=fail
    tok=Tokenizer.from_file(str(f));t=env.from_string(template)
    def encode(s):return tok.encode(s,add_special_tokens=False).ids
    root=Path(__file__).parent;ps=json.loads((root/'prompts.json').read_text(encoding='utf-8'));out=[]
    for p in ps:
        # Actual source context is serialized; example output is NOT given to the model.
        context=json.dumps(p['context'],ensure_ascii=False,separators=(',',':'))
        system=p['rules']+'\n'+p['output_schema_instruction']
        user='Context:\n'+context+'\nRequest:\n'+p['user_request']
        messages=[{'role':'system','content':system},{'role':'user','content':user}]
        rendered=t.render(messages=messages,tools=None,add_generation_prompt=True,enable_thinking=False)
        answer=json.dumps(p['example_output'],ensure_ascii=False,separators=(',',':'))
        count=len(encode(rendered));expected_out=len(encode(answer+'<|im_end|>'))
        fields={k:len(encode(v)) for k,v in [('rules',p['rules']),('schema_instruction',p['output_schema_instruction']),('context',context),('request',p['user_request'])]}
        out.append({'id':p['id'],'input_tokens_authoritative':count,'illustrative_output_tokens_with_end':expected_out,
          'field_tokens_separately_counted':fields,'framing_and_join_delta':count-sum(fields.values()),
          'rendered_prompt_sha256':hashlib.sha256(rendered.encode()).hexdigest(),'actual_generated_output_tokens':None,
          'within_reference_4096':count+expected_out<=4096})
        (root/f'{p["id"]}.rendered.txt').write_text(rendered,encoding='utf-8')
    record={'model':'Qwen/Qwen3-8B','tokenizer_sha256':sha,'tokenizer_config_sha256':hashlib.sha256(cfg.read_bytes()).hexdigest(),
     'versions':{x:importlib.metadata.version(x) for x in ['tokenizers','jinja2']},'mode':'non-thinking','ledger':out,
     'caution':'field counts are explanatory; full rendered prompt count is authoritative. Example output is not a model result.'}
    (root/'token-ledger.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('token-ledger.json written')
if __name__=='__main__':main()
