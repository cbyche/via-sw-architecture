"""Project current P21 criteria and calculated references into editable QA areas.

No candidate result or grade is manufactured from a regional calculation.
The v4 snapshot is historical and is never written by this generator.
"""
from pathlib import Path
from fractions import Fraction as F
import hashlib
import json
import sys

from build_dp_v4_qa_scenes import Scene, INK, MUTED, TEAL

ROOT = Path(__file__).resolve().parents[2]
QA = ROOT / 'docs/presentations_files/quality-attributes/quality-attributes.json'
REFERENCE = ROOT / 'docs/architecture/12-decisions/decision-packages/03-02-poc-reference-calculations.json'
DETAILS = {
    41: dict(title='모델 판단 반복과 코드 해결 반복',
             options=['모델 주도 해석', '모델 요청 틀과 코드 주도 완성'],
             cases=['S2-01', 'S2-02', 'S2-03', 'S2-05', 'S3-02', 'S3-03', 'S4-01', 'S6-01', 'S6-02'],
             reasons=[
                 ['전체 의미 결합과 부분 갱신', '요청 틀의 표현 범위와 부분 결합'],
                 ['조회 후 재판단과 실제 첫 반응', '지원 항목의 코드 해결과 추가 해석'],
                 ['조회와 전체 판단의 반복 경로', '항목별 해결과 코드 결합 경로'],
                 ['실제 재판단 token과 폐기 청구', '부분 해석 호출과 전체 청구'],
                 ['모델 지침과 검증의 실제 수정', '요청 틀과 Engine 규칙의 실제 수정'],
                 ['Context와 조회 buffer의 동시 할당', '요청 틀과 해결 상태의 동시 할당']],
             condition='충분한 typed 근거에서만 A 2호출/B 1호출. B의 추가 해석 또는 A의 cache로 차이가 줄어든다.'),
    44: dict(title='중앙 후속 지시와 단계별 실행 연결',
             options=['중앙에서 다음 실행 지시', '연결된 단계가 조건으로 실행'],
             cases=['S3-02', 'S4-01', 'S4-02', 'S5-01', 'S5-02', 'S5-04'],
             reasons=[
                 ['현재성 검사와 올바른 후속 지시', 'window와 Join의 현재성 검사'],
                 ['중앙 queue와 회수 대기', 'credit와 Stage 및 Join 대기'],
                 ['완료 회수와 다음 실행의 연결', 'Stage 활성화와 publication Join'],
                 ['실제 호출과 retry 및 폐기 청구', '실제 호출과 retry 및 폐기 청구'],
                 ['후속 규칙과 사건 계약의 수정', '단계와 Join 계약의 실제 수정'],
                 ['중앙 진행 상태와 실행 대기열', 'window와 credit 및 envelope copy']],
             condition='양안의 async, 우선순위와 병렬 실행을 허용한다. S5-04의 실제 queue와 Join 경합을 확인한다.'),
    45: dict(title='원본 조합과 공통 가공 정보 조회',
             options=['Request별 원본 조합', '공통 가공 정보 생산과 조회'],
             cases=['S2-02', 'S2-03', 'S2-04', 'S2-05', 'S6-01'],
             reasons=[
                 ['원문과 유효 cache의 조건 보존', '게시 관계의 coverage와 예외 보존'],
                 ['최초 조합과 재사용 cache 대기', '준비된 조회와 미게시 backfill 대기'],
                 ['첫 사용과 반복 및 갱신 후 조합', '조회와 원문검증 및 동기 backfill'],
                 ['cache miss와 실제 추가 해석 청구', '초기와 미사용 갱신 및 철회 청구'],
                 ['조합과 cache의 철회 처리 수정', '생산자와 조회자의 철회 처리 수정'],
                 ['원문 Context와 선택 cache', '관계 index와 생산 상태 및 원문']],
             condition='A의 유효 cache와 부분 갱신을 인정한다. B의 초기/미사용 생산과 backfill도 같은 P21에 포함한다.'),
    42: dict(title='공동 상태 확정과 독립 상태 접수',
             options=['통합 Core의 공동 확정', '독립 대화와 Task 서비스의 접수'],
             cases=['S6-04', 'S4-03', 'S5-01', 'S6-02', 'S5-03', 'S5-04'],
             reasons=[
                 ['의미와 접수 및 현재성 검사', '동일 검사와 receipt의 올바른 반영'],
                 ['공동 commit과 대화 반영 대기', '독립 접수와 receipt 반영 대기'],
                 ['관련 상태의 공동 확정 경로', '인계와 reconciliation의 전체 경로'],
                 ['코드 저장 외 실제 모델 청구', 'IPC 외 실제 모델 청구'],
                 ['공동 상태 계약의 실제 수정', '독립 인계 계약의 실제 수정'],
                 ['통합 실행체의 상태와 대기열', '추가 process와 receipt 상태']],
             condition='독립 process만으로 정확성이나 모델 비용 이익을 부여하지 않는다. 정상 접수와 경합을 함께 본다.'),
}


def load_current():
    qa = json.loads(QA.read_text())
    ref = json.loads(REFERENCE.read_text())
    source = ROOT / qa['source']
    sha = hashlib.sha256(source.read_bytes()).hexdigest()
    assert qa['contract_version'] == ref['contract_version'] == 'ASR-QA-v6'
    assert qa['source_sha256'] == ref['source_sha256'] == sha, 'Stale QA/reference input'
    assert ref['not_candidate_results'] is True
    assert [r['id'] for r in qa['rows']] == [f'ASR-QA-0{i}' for i in range(1, 7)]
    # Independently reproduce the displayed cost subtotal and remaining budget.
    pricing, voice = ref['llm_pricing'], ref['voice_pricing']
    semantic = (124000 * F(pricing['input_per_million']) + 12400 * F(pricing['output_per_million'])) / 1_000_000
    audio = F(50, 60) * F(voice['audio_input_per_minute']) + F(179, 60) * F(voice['audio_output_per_minute'])
    text = (10000 * F(voice['text_input_per_million']) + 2000 * F(voice['text_output_per_million'])) / 1_000_000
    total = semantic + audio + text
    assert total == F(ref['arithmetic']['activity_total_USD'])
    assert F('.50') - total == F(ref['arithmetic']['activity_headroom_USD'])
    budget = dict(semantic_USD=str(float(semantic)), fresh_audio_USD=str(float(audio)),
                  voice_text_USD=str(float(text)), subtotal_USD=str(float(total)),
                  remaining_USD=str(float(F('.50') - total)), target_USD='0.50',
                  evidence_level='CALCULATED_REFERENCE_NOT_CANDIDATE_RESULT')
    snapshot = dict(contract_version='ASR-QA-v6', source=qa['source'], source_sha256=sha,
                    projection_sha256=hashlib.sha256(QA.read_bytes()).hexdigest(),
                    rows=[{k: r[k] for k in ('id', 'name', 'method', 'target', 'guards', 'score')} for r in qa['rows']])
    data = dict(status='CURRENT_P21_CRITERIA_WITH_CALCULATED_REFERENCES_NOT_MEASURED', qa_snapshot=snapshot,
                workload=ref['workload_summary'], reference_source=str(REFERENCE.relative_to(ROOT)),
                reference_sha256=hashlib.sha256(REFERENCE.read_bytes()).hexdigest(), common_budget=budget,
                regional_references={k: ref[k] for k in ('dp41_sufficient_typed_evidence', 'dp42_valid_answer_seconds',
                                                       'dp44_six_normal_transitions_seconds', 'dp45_past_processing_USD', 'memory_examples')},
                candidate_values_and_grades='NOT_EVALUATED',
                dps={str(dp): {**d, 'results': [{'qa_id': q['id'], 'A': None, 'B': None,
                                                'A_grade': None, 'B_grade': None} for q in snapshot['rows']]}
                     for dp, d in DETAILS.items()})
    return data


def regional_line(dp, ref):
    if dp == 41:
        row = next(r for r in ref['dp41_sufficient_typed_evidence'] if r['profile'] == 'base-assumption')
        return f"지역 계산 S2-01: A {float(row['A_seconds']):.3f}초 / B {float(row['B_seconds']):.3f}초. 충분한 typed 근거와 2/1호출 가정이며 전체 평균은 아니다."
    if dp == 44:
        row = ref['dp44_six_normal_transitions_seconds']
        return f"지역 계산 S5-04: 전이 6회 A {float(row['A']):.3f}초 / B {float(row['B']):.3f}초. 실제 queue와 음성 차례 비용은 별도다."
    if dp == 45:
        row = ref['dp45_past_processing_USD']
        return f"과거 처리만 계산: A 3miss ${float(row['A_3_misses']):.5f}, cache 1miss ${float(row['A_1_miss']):.5f} / B 4생산 ${float(row['B_4_productions']):.5f}. 중복 청구는 제거한다."
    row = ref['dp42_valid_answer_seconds']
    return f"지역 계산 S6-04: 확정/접수 A {float(row['A']):.3f}초 / B {float(row['B']):.3f}초. 모델과 외부 접수는 공통이며 전체 평균은 아니다."


def scenes(data):
    qa, ref = data['qa_snapshot']['rows'], data['regional_references']
    memory = {int(r['dp']): r for r in ref['memory_examples']}
    targets = ['≥98%, 618/630', '평균 ≤1초, stop ≤0.1초', '평균 ≤10초', '≤$0.50/P21', '평균 ≤3개, 과제별 ≤5개', 'commit 최대 ≤4,096MiB']
    thresholds = ['1%p', '0.10초', '0.50초', '$0.03/P21', '0.50개', '128MiB']
    result = []
    for dp in (41, 44, 45, 42):
        d = DETAILS[dp]
        for kind in ('summary', 'qa-detail'):
            s = Scene(dp, kind)
            s.value['slug'] = f'dp{dp}-v5-{kind}'
            s.value['title'] = f"04-{dp}. {d['title']} / ASR-QA 평가"
            s.value['qa_region_start'] = 934 if dp == 41 else 871
            notes = ['ASR-QA-v6, 동일 P21 전체 5회. 후보 전체 원값/등급은 평가 전이며 0점으로 해석하지 않는다.',
                     '지역 call-plan과 메모리 예산은 CALCULATED_REFERENCE다. 지역 차이를 전체 평균/목표 충족/승자로 확대하지 않는다.',
                     '원문: '+data['qa_snapshot']['source'], '계산: '+data['reference_source'],
                     '공통 모델 배치: 로컬 VIA/로컬 VAD, 별도 클라우드 음성 모델과 의미 LLM. 04-40 공통 실행 계약을 따른다.',
                     'PoC 비용 reference: Qwen3-30B-A3B-Instruct-2507 Singapore/International non-thinking 및 gpt-realtime-2025-08-28. 제품 모델 선택을 뜻하지 않는다.',
                     d['condition'], '활성화 사례: '+', '.join(d['cases']),
                     regional_line(dp, ref), json.dumps(data['common_budget'], ensure_ascii=False),
                     '신규 audio와 예산 Text만의 소계이며 이력 audio 재입력, 별도 전사/vision/helper, 재시도/폐기/배경 생산은 실제 청구로 추가한다.',
                     '62호출은 공통 예산 reference로 양안 호출수를 강제하지 않는다. 지역 과거 처리와 중복되면 교체하고 별도면 추가한다.',
                     'QA-01/05의 정답률과 실제 변경 단위는 새 입력/변경 patch 없이 생성하지 않는다. 모든 QA의 보호 조건은 별도 판정한다.']
            if kind == 'qa-detail':
                s.text(40,22,1840,'VIA SOFTWARE ARCHITECTURE / ASR-QA-v6',19,TEAL,True)
                s.text(40,65,1840,s.value['title'],37,INK,True)
                s.line(40,121,1880,121)
                s.text(40,142,1840,'P21 전체 5회: 630개 채점 조건, 135반응, 95요청 처리. 비용과 메모리는 전체 활동을 포함한다.',24,MUTED)
                s.text(40,185,1840,'A/B 전체 값과 0–6점 등급은 평가 전. 아래 지역 계산과 메모리 예산은 작성자 가정이다.',22,MUTED)
                s.add('rect',x=40,y=225,w=1840,h=48,fill='#D5E7C7')
                for x,w,value in [(56,264,'QA / 목표'),(363,610,'A '+d['options'][0]),(1013,610,'B '+d['options'][1]),(1659,205,'차이 판단')]:
                    s.text(x,235,w,value,23,INK,True)
            for index,q in enumerate(qa):
                reasons = d['reasons'][index]
                labels = ['평가 전', '평가 전']
                if index == 5:
                    labels = [f"예산 {int(memory[dp][key]):,}MiB" for key in ('A_mib','B_mib')]
                notes.extend([f"{q['id']} {q['name']}: {q['method']}. 목표 {q['target']}. 보호 조건 {q['guards']}.",
                              f"A: {reasons[0]}. B: {reasons[1]}. 후보값/등급 NOT_EVALUATED."])
                if kind == 'summary':
                    y = 936+index*19 if dp==41 else 877+index*25
                    font = 14 if dp==41 else 16
                    s.line(40,y-2 if dp==41 else y-6,1880,y-2 if dp==41 else y-6)
                    s.text(42,y,245,q['name'],font,INK,True)
                    for side,x,w in [(0,300,700),(1,1050,825)]:
                        s.text(x,y,w,labels[side]+'  ['+reasons[side]+']',font)
                else:
                    y=273+index*105
                    s.line(40,y+104,1880,y+104)
                    s.text(56,y+10,264,f'{index+1:02d}  '+q['name'],22,INK,True)
                    s.text(56,y+48,264,targets[index],18,MUTED)
                    for side,x in [(0,363),(1,1013)]:
                        s.text(x,y+9,612,labels[side]+(' (가정)' if index==5 else ''),26,INK,True)
                        s.text(x,y+52,612,reasons[side],21)
                    s.text(1659,y+13,205,'실측 차이 평가 전',20,INK,True)
                    s.text(1659,y+53,205,'기준 '+thresholds[index],18,MUTED)
                    s.text(1659,y+78,205,'보호 조건 별도',17,MUTED)
            if kind == 'summary':
                if dp!=41:s.text(40,1030,1840,'P21 비용 예산 소계 $0.35184, 잔여 $0.14816. 후보별 전체 청구와 목표 판정은 평가 전.',15,MUTED)
                s.text(40,1053 if dp==41 else 1057,1840,'ASR-QA-v6 / P21 기준 / 미실측은 0점이 아님 / 메모리는 예산 가정 / 지역 계산은 다음 장',14,MUTED)
            else:
                for x in (340,990,1640):s.line(x,225,x,903)
                s.text(40,931,1840,regional_line(dp,ref),21)
                s.text(40,969,1840,'P21 공통 비용 예산: 의미 $0.03472 + 신규 audio $0.24512 + 음성 Text $0.07200 = $0.35184. 잔여 $0.14816.',21,MUTED)
                s.text(40,1035,1840,'이력 audio, 추가 생산, 재시도와 폐기는 실제 청구로 추가한다. 전체 후보값·보호 조건 확인 후 등급을 부여한다.',19,MUTED)
            s.value['notes']='\n'.join(notes)
            result.append(s.value)
    return dict(status=data['status'],scenes=result)


if __name__=='__main__':
    assert len(sys.argv)==3, 'Usage: build_dp_v5_qa_scenes.py SCENES_JSON SNAPSHOT_JSON'
    data=load_current()
    Path(sys.argv[1]).write_text(json.dumps(scenes(data),ensure_ascii=False,indent=2)+'\n')
    Path(sys.argv[2]).parent.mkdir(parents=True,exist_ok=True)
    Path(sys.argv[2]).write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n')
    print('PASS: current P21 criteria and exact budget projected; candidate grades remain unevaluated')
