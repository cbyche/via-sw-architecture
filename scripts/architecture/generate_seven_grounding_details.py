"""Generate the two DP-47 integration detail figures.

Documentation assets only. No product, model, or performance execution.
The shared Figure renderer is owned by the package coordinator.
"""
from PIL import Image, ImageDraw, ImageFont

from generate_seven_problem_figures import Figure, FONT, BLUE, ORANGE, INK


def common(side, title):
    f = Figure(
        f'dp47-seven-{side}-detail',
        f'DP-47 상세 {side}: {title}',
        '같은 장면: D1의 두 대상을 가리키고 페이지를 바꾼 뒤 “이 둘과 방금 그래프를 비교해줘.” 이후 일부 정정.',
        width=2400, height=1800,
    )
    f.labels[2]['text'] = '파랑: 역할과 처리  |  주황: 보존 상태  |  실선: 입력/결과  |  점선: 조건부 호출/제어/재조회  |  하단: 선택 후 보완'
    f.labels[2]['size'] = 25
    f.node('capture', 55, 235, 550, 245, [
        '1 Interaction Manager: 실제 관측',
        '발화 중 원음 + 전사 revision',
        'frame + source 시각 / 수신 시각',
        '유효한 발화 전 selection / pre-roll',
        'pointer path / 버튼 / page / 창',
        'circle / page 이동을 함께 기록',
    ])
    f.node('control', 1795, 235, 550, 245, [
        '정정과 권한의 공통 제어',
        'r2 등록 즉시 r1 후속 사용 hold',
        'BeginRevision: expected 값 교체',
        'TryApply / TryPublish와 같은 Gate',
        '취소와 채택 차단은 별도 처리',
        '의미 worker 완료를 기다리지 않음',
    ])
    f.node('raw', 55, 555, 550, 420, [
        'Raw Evidence Window + Reader',
        '원음 / encoded frame / native',
        'source ID / 판본 / locator / 시각 오차',
        '순서 / gap / 권한 / 만료를 보존',
        '시간 대응은 의도 판단의 근거',
        'timestamp만으로 의도 확정 안 함',
        'worker가 범위 read lease로 읽음',
        'callback을 막지 않는 유한 Queue',
        '포화: payload 누락과 gap 기록',
        '없던 원본은 생성하지 않음',
        '현재 화면으로 과거 원본 대체 금지',
    ], state=True)
    f.node('retention', 55, 1040, 550, 430, [
        '원본과 임시 상태의 수명',
        'bytes 안전 복사 후 OS frame 반납',
        'raw: 시간 / byte / lease 한도 적용',
        '필요 정정 구간 보존, 철회 우선',
        '후보가 생겨도 raw 삭제 안 함',
        'decoded / staging: reader 종료 후',
        'SDK 참조 종료도 확인 후 회수',
        '임시 상태는 채택 / 종료 후 정리',
        '채택된 허용 artifact만 지속 저장',
        'raw 해제 뒤 불가능한 재판단 명시',
        '영속 파일 크기와 local RAM 구별',
    ], state=True)
    f.node('failure', 1795, 1040, 550, 430, [
        '정정 / 오류 / unknown 경로',
        'r2: 현재 요청 binding을 재판단',
        '늦은 r1: writer가 stale 게시 거절',
        '권한 철회: raw / 파생 사용 차단',
        'source: 역사 / 현재 selector 구별',
        '관측 gap: 한도 대기 또는 확인',
        '모델 실패: 예산 내 재시도 / 보류',
        'coverage 불명은 정답이 아님',
        '불완전 binding으로 업무 위임 금지',
        '제어 / 완료 inbox와 job Queue 분리',
        'state lock 안에서 I/O await 금지',
    ])
    final_step = 6 if side == 'A' else 7
    f.node('adoption', 665, 1315, 1070, 155, [
        f'{final_step} 공통 Currentness + Adoption Gate → Request Controller / State Store',
        'input / source / policy / base 및 A 원본 의존 / B 후보 read-set의 현재성 검사',
        '통과한 binding만 요청 해석 / 응답 / 업무에 사용. 불일치는 거절, 모호함은 확인',
        'QA02 최초 채택 오류 보존. 후속 수정으로 최초 실패를 지우지 않음',
    ])
    f.edge('capture', 'raw', [(330, 480), (330, 555)])
    f.edge('capture', 'trigger', [(605, 355), (665, 355)])
    f.edge('control', 'trigger', [(1795, 355), (1735, 355)], dashed=True)
    f.edge('raw', 'retention', [(330, 975), (330, 1040)])
    f.edge('state', 'decision', [(1200, 1035), (1200, 1085)])
    f.edge('decision', 'adoption', [(1200, 1275), (1200, 1315)])
    f.edge('failure', 'adoption', [(1795, 1390), (1735, 1390)], dashed=True)
    return f


def check_document_layout(f):
    """Fail on clipped text rather than letting the renderer reduce its font."""
    draw = ImageDraw.Draw(Image.new('RGB', (1, 1)))
    for node in f.nodes:
        size = 28 if node['panel'] else 25
        font = ImageFont.truetype(FONT, size)
        for line in node['lines']:
            if draw.textlength(line, font=font) > node['w'] - 36:
                raise ValueError(f'{f.slug}/{node["key"]}: line is too wide: {line}')
        if 16 + (len(node['lines']) - 1) * 35 + size + 8 > node['h']:
            raise ValueError(f'{f.slug}/{node["key"]}: text is too tall')
        if node['y'] < 235 or node['y'] + node['h'] > f.height - 90:
            raise ValueError(f'{f.slug}/{node["key"]}: node outside content area')
    for label in f.labels:
        if '\u00b7' in label['text']:
            raise ValueError('Middle-dot punctuation is not allowed')
    for node in f.nodes:
        if any('\u00b7' in line for line in node['lines']):
            raise ValueError('Middle-dot punctuation is not allowed')


def raw_joint():
    f = common('A', '원본에서 target extent와 binding을 함께 생산')
    f.node('trigger', 665, 235, 1070, 245, [
        '2 Request Controller → Grounding Attempt Control',
        '완성 입력 revision 등록 후 의미 판단 시작. 발화 중 원본 수집은 계속',
        'GroundingInput: rawRefs / 전사 / prior / 권한 / revision / deadline',
        'bounded Joint Work Queue에는 참조 job. worker가 Reader를 직접 읽음',
        '정정 시 현재 요청의 전체 binding 집합 재검토. 전체 대화 재실행은 아님',
        '제어 / 완료 inbox 분리. 현재 시도 안의 유효 binding은 재사용',
    ])
    f.node('joint', 665, 555, 1070, 270, [
        '3 Joint Grounding Interpreter: extent와 binding의 의미 writer',
        'native 선택과 표현이 명백하면 code. 의미 판단이 필요하면 M 공통 모델',
        '원음 / 전사 + frame / path / native를 함께 읽어 지칭 집합과 관계를 생산',
        '한 call에서 두 graph와 제목 등 다수 지칭 가능. 표현 수만큼 call 강제 없음',
        '같은 판단에서 읽은 원본과 binding으로 충분한 검사에는 재입력을 강제 안 함',
        '부족한 발화 / 화면 범위는 5에서 원본 재조회. 별도 후보 게시 관문 없음',
        '기본에는 semantic crop/cache, 부분 binding patch, 증분 의미 판단 미포함',
    ])
    f.node('state', 665, 880, 1070, 155, [
        '4 Joint Grounding State: 현재 시도의 불변 결과와 근거 의존',
        'bindings / source read-set / input revision / base version / 불확실성',
        'TryApply 현재성 통과 시 게시. 같은 판단 안의 후속 검사에 재사용',
        '요청 채택 / 종료 뒤 필요한 binding과 source만 남기고 임시 상태 회수',
    ], state=True)
    f.node('decision', 665, 1085, 1070, 190, [
        '5 Ready / NeedEvidence / Ambiguous',
        'NeedEvidence → 같은 Joint Grounding Interpreter가 bounded 원본 재조회',
        '관측이 충분해지면 3 재판단. 원본이 없으면 unknown / 질문 / 보류',
        'source 오류나 r2 정정은 새 근거 판본으로 현재 요청 binding을 재검토',
        'Ready도 직접 사용하지 않음. 6 공통 gate에서 최초 채택 확정',
    ])
    f.node('model', 1795, 555, 550, 420, [
        'M 조건부 cloud 의미 의존',
        'Model Access / 같은 capability',
        'native 충분 → 모델 생략 가능',
        '3: 원본 의미 판단에만 요청',
        '발화 + 관련 media + prior 입력',
        '다수 지칭도 한 번에 처리 가능',
        'worker가 결과 / usage를 회수',
        '입력 / 출력 / 재시도 청구 포함',
        '취소 뒤 청구와 staging도 포함',
        'cloud weights / KV는 RAM 제외',
        '기본 A에 별도 screen parser 없음',
    ])
    f.edge('trigger', 'joint', [(1200, 480), (1200, 555)])
    f.edge('raw', 'joint', [(605, 700), (665, 700)])
    f.edge('joint', 'state', [(1200, 825), (1200, 880)])
    f.edge('joint', 'model', [(1735, 700), (1795, 700)], dashed=True)
    f.edge('decision', 'joint', [(665, 1170), (630, 1170), (630, 760), (665, 760)], dashed=True)
    f.node('tactics', 55, 1510, 2290, 190, [
        '선택 후 보완: 각각 기본 A + 개별 기법. 함께 쓰면 조합을 명시',
        'A1 crop/cache: 실제 read와 청구 감소 확인.  A2 부분 binding patch: 의미 영향 범위 판단과 전체 재검토 fallback 필요',
        'A3 발화 중 증분 의미 판단: 양쪽에 가능한 보완.  A4 동일 parser를 advisory 입력으로 사용: 최종 의미 writer는 Joint',
        '현재 기본 추천 A. A1/A2는 필요 시 선택. 큰 원본 입력과 재판단 비용은 남으며, 정확도 우위나 비용 승자는 미측정',
    ], state=True)
    check_document_layout(f)
    f.save()
    return f.slug


def candidate_binding():
    f = common('B', 'producer 소유 후보를 생산하고 호환 revision으로 결합')
    f.node('trigger', 665, 235, 1070, 245, [
        '2 Request Controller → Grounding Coordinator',
        '완성 입력 revision 등록 후 의미 판단 시작. 발화 중 원본 수집은 계속',
        'GroundingInput: rawRefs / 전사 / prior / 권한 / revision / deadline',
        '기본: 단계 3 → 4 순차 dispatch. 두 생산 사이 데이터 선후 의존은 없음',
        '결합에는 단계 3/4 호환 결과 모두 필요. Coordinator는 의미 값 수정 금지',
        'job은 raw 참조만 전달. 각 worker가 Evidence Reader를 직접 읽음',
    ])
    f.node('screen', 665, 555, 505, 270, [
        '4 Screen Target Producer',
        'frame / path / selection / native',
        'native 충분 → code target 생산',
        '부족 → M 별도 parser 생산',
        '자유 extent / pointer 의도는 M 보완',
        '누락 / 경쟁 / coverage 함께 반환',
        'Screen 후보 의미의 단독 writer',
    ])
    f.node('speech', 1230, 555, 505, 270, [
        '3 Speech Reference Producer',
        '원음 / 전사 / 시각 / prior 읽기',
        '표현 / 수 / 종류 / 관계 / 정정 가설',
        '표현의 미해결 항목도 유지',
        '의미 해석 필요 시 M 공통 모델',
        '불명 / 경쟁 해석과 source 보존',
        'Speech 후보 의미의 단독 writer',
    ])
    f.node('state', 665, 880, 1070, 155, [
        '5 Speech Candidate Revisions + Screen Candidate Revisions',
        '각 producer만 자신의 immutable 후보 게시. source / peer / extent / coverage',
        '호환 snapshot read-set으로 결합. 같은 판단 안의 복수 표현에 후보 재사용',
        '새 완성 입력 사이 cache와 일부 재생산은 B1/B2. 기본 재사용에 섞지 않음',
    ], state=True)
    f.node('decision', 665, 1085, 1070, 190, [
        '6 Multimodal Binding Resolver → Binding Revision State',
        '관계 / type / 수 / source-time 호환 + 유일한 해석 + 필수 unknown 없음 → code',
        'coverage 주장은 정답 증명이 아님. 자유 의미 / 모호함은 M 조건부 판단',
        '후보 부족 → Coordinator가 원 producer에 raw 확장 요청 → 단계 3/4 재생산',
        'raw에서 새 target 직접 생산은 hybrid. 읽은 후보 판본이 바뀌면 재결합',
    ])
    f.node('model', 1795, 555, 550, 420, [
        'M 조건부 cloud 의존',
        '공통 모델: 단계 3/4/6 의미 판단',
        '같은 capability / 총 호출 예산',
        '별도 screen parser: 단계 4만 호출',
        'native 부족 → detector / OCR',
        '+ caption local semantics 생산',
        '단계 4가 source 후보 의미를 소유',
        'helper / network / 의미 보완 비용',
        '가격 / 서비스 적합성 미확인',
        'cloud weights는 local RAM 제외',
        'response / candidate RAM은 포함',
    ])
    f.edge('trigger', 'screen', [(917, 480), (917, 555)])
    f.edge('trigger', 'speech', [(1482, 480), (1482, 555)])
    f.edge('raw', 'screen', [(605, 645), (665, 645)])
    f.edge('raw', 'speech', [(605, 765), (620, 765), (620, 850), (1510, 850), (1510, 825)])
    f.edge('screen', 'state', [(917, 825), (917, 880)])
    f.edge('speech', 'state', [(1482, 825), (1482, 880)])
    f.edge('screen', 'model', [(1060, 555), (1060, 515), (1770, 515), (1770, 650), (1795, 650)], dashed=True)
    f.edge('speech', 'model', [(1735, 720), (1795, 720)], dashed=True)
    f.edge('decision', 'model', [(1735, 1170), (1770, 1170), (1770, 930), (1795, 930)], dashed=True)
    f.edge('decision', 'trigger', [(665, 1170), (645, 1170), (645, 440), (665, 440)], dashed=True)
    f.node('tactics', 55, 1510, 2290, 190, [
        '선택 후 보완: 각각 기본 B + 개별 기법. 함께 쓰면 조합을 명시',
        'B1 cross-revision 안정 후보 재사용.  B2 부분 후보 재생산: 유효성 / 의미 영향 범위 판단과 무효화 비용 포함',
        'B3 발화 중 증분 의미 판단.  B4 단계 3/4 병행: 같은 예산의 정책 보완. 순차 정책만으로 B 구조의 필연적 열세 판정 금지',
        '교차 확인: A4 동일 parser / parser 없는 B. 시각 표현의 정확도 가능성과 후보 소유 구조 자체의 효과를 분리',
    ], state=True)
    check_document_layout(f)
    f.save()
    return f.slug


if __name__ == '__main__':
    print('Generated:', raw_joint(), candidate_binding())
