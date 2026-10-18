"""Detailed evidence and handoff design figures, 2026-10-18.

Documentation rendering only. No product implementation, model calls or benchmark.
The shared Figure renderer emits editable draw.io, SVG and PNG from the same data.
"""
import json

from PIL import ImageFont

from generate_seven_problem_figures import Figure, OUT, FONT, BLUE, ORANGE


WIDTH, HEIGHT = 2200, 1700
LEFT, MAIN, RIGHT = 70, 780, 1510
LW, MW, RW = 630, 650, 650
ROWS = [270, 455, 640, 825, 1010, 1195]
GENERATED = []


def node(f, key, col, row, lines, state=False, height=150):
    x, w = {'l': (LEFT, LW), 'm': (MAIN, MW), 'r': (RIGHT, RW)}[col]
    f.node(key, x, ROWS[row], w, height, lines, state=state)


def down(f, a, b, x=1105):
    na = next(n for n in f.nodes if n['key'] == a)
    nb = next(n for n in f.nodes if n['key'] == b)
    f.edge(a, b, [(x, na['y'] + na['h']), (x, nb['y'])])


def across(f, a, b, y, dashed=False):
    na = next(n for n in f.nodes if n['key'] == a)
    nb = next(n for n in f.nodes if n['key'] == b)
    if na['x'] < nb['x']:
        points = [(na['x']+na['w'], y), (nb['x'], y)]
    else:
        points = [(na['x'], y), (nb['x']+nb['w'], y)]
    f.edge(a, b, points, dashed=dashed)


def model_pair(f, main_key, model_key, row):
    y = ROWS[row]
    across(f, main_key, model_key, y+55)
    across(f, model_key, main_key, y+115, dashed=True)


def start(slug, title, scenario, evidence=True):
    f = Figure(slug, title, scenario, width=WIDTH, height=HEIGHT)
    f.label(LEFT, 220, '원본과 상태: 각 owner가 보존', 27, BLUE, True)
    f.label(MAIN, 220, 'VIA 기본 처리: 번호 순서', 27, BLUE, True)
    f.label(RIGHT, 220, 'Cloud 호출과 변경 / 실패' if evidence else '모델 / 외부 Agent / 변경', 27, BLUE, True)
    return f


def finish(f, baseline, tactics):
    f.label(70, 1370, baseline, 25, BLUE)
    f.label(70, 1410, '선택 후 보완: 두 기법은 독립 적용. 기본 동작과 추가 비용을 구별하고 결합은 별도로 비교한다.', 25, ORANGE)
    f.node('tactic1', 70, 1450, 1010, 155, tactics[0], state=True)
    f.node('tactic2', 1150, 1450, 1010, 155, tactics[1], state=True)
    # Layout checks concern only documentation. They are not product tests.
    font = ImageFont.truetype(FONT, 25)
    overflow, intersections = [], []
    for n in f.nodes:
        if n['x'] < 0 or n['y'] < 230 or n['x']+n['w'] > WIDTH or n['y']+n['h'] > HEIGHT-65:
            overflow.append(n['key']+':bounds')
        if 16+(len(n['lines'])-1)*35+25 > n['h']:
            overflow.append(n['key']+':height')
        for line in n['lines']:
            if len(line) > 35:
                overflow.append(n['key']+':line_length:'+line)
            if font.getlength(line) > n['w']-36:
                overflow.append(n['key']+':text:'+line)
    for i, a in enumerate(f.nodes):
        for b in f.nodes[i+1:]:
            if min(a['x']+a['w'], b['x']+b['w']) > max(a['x'], b['x']) and min(a['y']+a['h'], b['y']+b['h']) > max(a['y'], b['y']):
                intersections.append([a['key'], b['key']])
    if overflow or intersections:
        raise ValueError((f.slug, overflow, intersections))
    f.save()
    GENERATED.append({'figure': f.slug, 'size': [WIDTH, HEIGHT],
                      'nodes': len(f.nodes), 'edges': len(f.edges),
                      'node_bounds_or_text_overflow': len(overflow),
                      'node_intersections': len(intersections),
                      'visual_review': 'reported separately after viewing PNG'})


ORIGINALS = ['원본의 권위는 각 owner에 있음',
             '입력 / 정정: Request Controller',
             '실제 전달: Response Manager',
             '결과 / 현재 업무: Task Manager']
EINPUT = ['1 Interaction Manager가 입력 수신',
          'Request Controller: 입력 revision',
          '최근 대화 / 관측 참조 / 허용 범위',
          '정답 Task 연결은 아직 주어지지 않음']
EADOPT = ['5 현재성 검사 뒤 의미 채택',
          'Request Controller / Policy Manager',
          '입력 / source / 권한 epoch 재확인',
          'UseCheck와 사용 등록을 철회와 직렬화']
ECLOUD = ['4 자연어 해석이 필요할 때 호출',
          'Model Access → cloud 의미 모델',
          '지칭 / 조건 / Task 관계를 제안',
          '최종 채택 권위는 VIA에 남음']
ECHANGE = ['정정 / 삭제 / 권한 철회 사건',
           '현재 revision / epoch를 먼저 변경',
           '의존 자료와 미전송 효과 즉시 차단',
           '늦은 완료로 이전 조건을 되살리지 않음']


def evidence_a():
    f = start('problem01-A-detail', '근거 공급 A: 대화 작업 문맥을 유지하고 부족한 원본을 보완한다',
              '같은 입력: “아까 실제로 설명한 기준과 비교 결과를 이번 요청에 써줘.” 실제 전달 범위와 source 판본은 공통이다.')
    node(f, 'raw', 'l', 0, ORIGINALS, True)
    node(f, 'in', 'm', 0, EINPUT)
    node(f, 'scope', 'r', 0, ['기본 A의 고유 선택', '대화 수명의 source-backed 작업 문맥', '다음 발화가 앞서 읽은 근거를 재사용', '원본 소유권 / 현재 의미 권위는 공통'])
    node(f, 'view', 'l', 1, ['지속 상태: Context Manager의 view', '근거 발췌 / 조건 / 미해결 사항', 'source ID / revision / 적용 범위', '6에서 갱신하고 다음 요청의 2에서 읽음'], True)
    node(f, 'context', 'm', 1, ['2 유효 작업 문맥을 읽어 준비', 'Context Manager: 대화별 view revision', '현재 source / 권한을 확인', '예산 초과 / 영향 변경이면 재구성'])
    node(f, 'condense', 'r', 1, ['2 압축 / 재구성 필요 때만 호출', 'Model Access → cloud 의미 모델', '출처와 실제 전달 범위를 함께 보존', '기본 재사용은 prefix cache와 다름'])
    node(f, 'bundle', 'l', 2, ['요청 수명: EvidenceBundle / Receipt', '발췌 / 참조 / 별도 source vector', '실제 읽은 범위와 누락 / 실패 상태', '전역 atomic snapshot을 주장하지 않음'], True)
    node(f, 'read', 'm', 2, ['3 부족 / 변경된 근거만 원본 보완', 'Context Manager: owner 범위 읽기', '충분하면 유지 근거를 그대로 통과', '필수 원문 / 정확한 인용은 원문 확인'])
    node(f, 'dispute', 'r', 2, ['원본이 같은 파생 오류도 신고 가능', 'Request Controller: 오류와 view 판본', 'Context Manager가 EvidenceDispute 수신', '해당 판본 차단 후 원본으로 재검증'])
    node(f, 'receipt', 'l', 3, ['원본 / 실제 전달의 구별', '준비한 설명과 표시 / 재생 구간 별도', '참조 보관이 원본 전체 보관은 아님', '과거 동의가 현재 권한을 대신하지 않음'], True)
    node(f, 'interpret', 'm', 3, ['4 근거를 사용해 현재 의미 제안', 'Request Interpreter', '부족 범위는 보존하고 필요하면 재조회', '같은 충분 근거이면 정확도 우위 미정'])
    node(f, 'cloud', 'r', 3, ECLOUD)
    node(f, 'used', 'l', 4, ['확정 상태: 채택 의미와 사용 근거', 'Request Controller: 입력 / 요청 판본', '원본 source spans와 실제 사용 참조', '잠정 제안은 확정 조건과 분리'], True)
    node(f, 'adopt', 'm', 4, EADOPT)
    node(f, 'change', 'r', 4, ECHANGE)
    node(f, 'recovery', 'l', 5, ['재시작: 삭제 원장을 먼저 확인', '유효 view 복원 또는 원본 재구성', '재사용 불명 자료는 최신으로 표시 안 함', '외부 Action을 자동 재실행하지 않음'])
    node(f, 'out', 'm', 5, ['6 근거 참조와 view revision을 보존', '실제 응답 준비 또는 업무 인계로 전달', 'Response Manager의 실제 전달은 별도', '다음 발화가 유효 작업 문맥을 재사용'])
    node(f, 'failure', 'r', 5, ['실패: read 거절 / hard 압축 실패', 'bounded 원본 재구성 또는 PARTIAL', '필수 근거 없으면 보류 / 확인', '취소 / 재시도 / 늦은 cloud 청구 보존'])
    for a, b in [('in','context'),('context','read'),('read','interpret'),('interpret','adopt'),('adopt','out')]: down(f,a,b)
    across(f,'view','context',525)
    f.edge('raw','read',[(700,345),(740,345),(740,705),(780,705)])
    across(f,'read','bundle',750)
    across(f,'adopt','used',1120)
    model_pair(f,'context','condense',1)
    model_pair(f,'interpret','cloud',3)
    across(f,'dispute','read',715,dashed=True)
    across(f,'change','adopt',1085,dashed=True)
    across(f,'failure','out',1270,dashed=True)
    f.edge('out','view',[(1105,1345),(1105,1355),(35,1355),(35,590),(70,590)])
    finish(f, '기본의 이익과 비용: 반복 발견 / 조회 감소 가능 ↔ 유지 입력, 압축과 정정 뒤 재구성 비용.',
           [['A1 입력 prefix cache', '같은 유효 prefix를 provider 지원 아래 재사용',
             '추가: cache 수명 / 권한 재확인 / 요금', 'A1+A2는 patch 뒤 prefix 적중 변화도 확인'],
            ['A2 변경 조각 갱신', '확인된 dependency의 부분만 재구성',
             '추가: index / patch 정합성 검사 비용', '영향 범위가 불명이면 기본 전체 재구성']])


def evidence_b():
    f = start('problem01-B-detail', '근거 공급 B: 현재 요청에 필요한 원본으로 요청별 근거를 만든다',
              '같은 입력: “아까 실제로 설명한 기준과 비교 결과를 이번 요청에 써줘.” 최근 대화와 채택 상태를 없애지 않는다.')
    node(f,'raw','l',0,ORIGINALS,True)
    node(f,'in','m',0,EINPUT)
    node(f,'scope','r',0,['기본 B의 고유 선택', '요청마다 필요한 근거를 원본에서 구성', '확장한 해석 문맥은 요청 종료 때 종료', '공통 원본 / 채택 상태 / 최근 대화 보존'])
    node(f,'state','l',1,['공통 시작 상태와 source 탐색 정보', '최근 대화 / 채택 조건 / 관측 참조', 'metadata / 명시 ID / 정정 링크', '추가 조회 결과의 요청 간 cache는 없음'],True)
    node(f,'context','m',1,['2 현재 목적의 읽기 범위 정하기', 'Request Interpreter가 읽기 범위 제안', '후보 source를 발견하고 범위를 좁힘', '매 요청마다 전체 transcript를 읽지 않음'])
    node(f,'querymodel','r',1,['2 탐색 의미가 모호할 때만 호출', 'Model Access → cloud 의미 모델', '명시 ID 연결은 코드로 처리', '모든 read에 추가 모델 호출을 강제 안 함'])
    node(f,'bundle','l',2,['요청 수명: EvidenceBundle / Receipt', '현재 필요한 발췌 / 참조 / 관계 후보', '읽은 source 판본 / 범위 / 누락', '다음 요청의 확장 view로 자동 유지 안 함'],True)
    node(f,'read','m',2,['3 owner의 허용 원본을 조합', 'Context Manager → bounded read', '필수 원문과 실제 전달 구간 확인', '부족하면 같은 예산 안에서 추가 조회'])
    node(f,'dispute','r',2,['파생 오류 / source 변경', '문제된 묶음 revision을 무효화', '원본 재조회 후 현재 관계를 다시 판단', '이전 완료는 새 read ID에 합치지 않음'])
    node(f,'receipt','l',3,['원본 / 실제 전달의 구별', '생성 원문과 실제 제시 구간 별도', '지연 수신 사실을 과거에 안 것으로 안 함', 'source vector는 별도 관측 판본의 목록'],True)
    node(f,'interpret','m',3,['4 요청 근거에서 현재 의미 제안', 'Request Interpreter', '지칭 / 조건 / Task 관계의 후보를 제출', '현재 해석의 최종 채택은 5에서 수행'])
    node(f,'cloud','r',3,ECLOUD)
    node(f,'used','l',4,['확정 상태: 채택 의미와 사용 근거', 'Request Controller: 현재 요청 판본', '사용 source spans / 결과 / 실제 전달', '확장 해석 문맥 종료와 원본 삭제는 다름'],True)
    node(f,'adopt','m',4,EADOPT)
    node(f,'change','r',4,ECHANGE)
    node(f,'recovery','l',5,['재시작: 삭제 / 권한 원장 먼저 확인', '새 read ID로 현재 원본을 다시 구성', '명시 원본과 사용 참조는 그대로 보존', '외부 Action을 자동 재실행하지 않음'])
    node(f,'out','m',5,['6 채택 근거를 보존하고 요청 묶음 종료', '실제 응답 준비 또는 업무 인계로 전달', '다음 요청은 공통 최소 문맥에서 시작', '같은 근거 필요 시 원본을 다시 구성'])
    node(f,'failure','r',5,['실패: DENIED / UNAVAILABLE / PARTIAL', 'NO_MATCH는 실제 확인한 범위의 부재', '필수 근거 없으면 보류 / 확인', '반환 유실 / 재시도 비용을 숨기지 않음'])
    for a,b in [('in','context'),('context','read'),('read','interpret'),('interpret','adopt'),('adopt','out')]:down(f,a,b)
    across(f,'state','context',525)
    f.edge('raw','read',[(700,345),(740,345),(740,705),(780,705)])
    across(f,'read','bundle',750)
    across(f,'adopt','used',1120)
    model_pair(f,'context','querymodel',1)
    model_pair(f,'interpret','cloud',3)
    across(f,'dispute','read',715,dashed=True)
    across(f,'change','adopt',1085,dashed=True)
    across(f,'failure','out',1270,dashed=True)
    finish(f,'기본의 이익과 비용: 새 목적의 좁은 입력 / 유지 비용 감소 ↔ 같은 근거의 반복 발견과 원본 구성.',
           [['B1 조회 결과 cache', '같은 목적 / source 판본의 읽기 결과 재사용',
             '추가: 저장 / 무효화 / 현재 권한 확인', 'cache miss에서도 기본 원본 구성이 가능'],
            ['B2 독립 근거 병렬 읽기', '실제 독립 source를 같은 총 예산 안에서 조회',
             '추가: 동시 buffer / 실패 결합 처리', '현재 초기 추천은 A. B 선택 시의 보완 후보']])


def evidence_c():
    f=start('problem01-C-detail','근거 공급 C: 과거 관계를 게시하고 현재 요청의 빈 곳은 원본으로 보완한다',
            '같은 입력과 과거 원본. 미래 요청의 정답 Task를 미리 생산하지 않는다. 게시 생산과 현재 조회는 별도 수명이다.')
    node(f,'raw','l',0,ORIGINALS,True)
    node(f,'in','m',0,EINPUT)
    node(f,'scope','r',0,['기본 C의 고유 선택', 'Context Manager: 생산과 조회 책임 분리', '요청은 B와 같은 최소 문맥에서 시작', 'A+C도 가능하나 별도 구조 조합임'])
    node(f,'producer','l',1,['독립 생산: Context Manager', '확정 변경 → 원본 읽기 → 관계 생산', '명시 관계는 코드, 자유문 관계만 모델', 'job / source vector / 생산자 판본'],True)
    node(f,'context','m',1,['2 게시된 관계와 coverage 조회', 'Context Manager: 목적별 EvidenceQuery', 'PRESENTS / HAS_RESULT / REVISES 등', '현재 권한 / source 판본 확인은 유지'])
    node(f,'prodmodel','r',1,['생산 중 의미 추출 필요 때만 호출', 'Model Access → cloud 의미 모델', '예: 실제 설명의 항목 / 예외 추출', '현재 요청 채택이나 domain 계획은 안 함'])
    node(f,'repo','l',2,['게시 상태: record / coverage / cursor', 'job 결과 / 알림 outbox와 일관 확정', '검증 전 candidate는 일반 조회에 안 보임', '게시 성공이 의미의 완전함은 아님'],True)
    node(f,'read','m',2,['3 누락 / 오류 / 미지원은 원본 보완', 'owner bounded read로 필요한 부분 구성', '지원 범위만 생산 대기 / 재조회 가능', '원본 보완과 게시 대기 비용을 모두 기록'])
    node(f,'dispute','r',2,['EvidenceDispute / source 변경', '정확한 게시 revision을 즉시 차단', '원본 재검증 / 재생산 또는 직접 보완', '미지원 관계를 무한 backfill하지 않음'])
    node(f,'receipt','l',3,['요청 수명: 묶음 / 영수증 / waiter', '실제 읽은 범위 / gaps / source vector', '대기 등록 뒤 게시 revision 재조회', 'deadline / 중복 알림 / 취소를 구분'],True)
    node(f,'interpret','m',3,['4 충분한 근거로 현재 의미 제안', 'Request Interpreter', '게시 관계가 이번 Task를 정하지 않음', '정확한 인용은 원문 locator로 확인'])
    node(f,'cloud','r',3,ECLOUD)
    node(f,'used','l',4,['확정 상태: 현재 의미 / 사용 근거', '원본 source와 게시 revision 함께 기록', '수신한 원본들은 전역 snapshot이 아님', '이미 가진 공통 ID index도 사용 가능'],True)
    node(f,'adopt','m',4,EADOPT)
    node(f,'change','r',4,ECHANGE)
    node(f,'recovery','l',5,['재시작: 삭제 원장과 게시 commit 확인', '마지막 게시 cursor에서 gap을 복구', '알림 outbox 재전송 / 조회 재확인', '늦은 생산으로 삭제 자료 재게시 금지'])
    node(f,'out','m',5,['6 채택 근거를 보존하고 요청은 종료', '게시 기록은 허용 목적에서 계속 유지', '생산은 유효 목적 / 예산에서만 지속', '미소비 생산 / 폐기 / 재시도까지 계상'])
    node(f,'failure','r',5,['실패: 저장소 / 생산 / read 오류', '원본 보완 또는 명시적 제한으로 반환', '필수 근거 없으면 보류 / 확인', '중요 입력과 다른 요청을 함께 막지 않음'])
    for a,b in [('in','context'),('context','read'),('read','interpret'),('interpret','adopt'),('adopt','out')]:down(f,a,b)
    down(f,'raw','producer',385);down(f,'producer','repo',385)
    f.edge('repo','context',[(700,700),(740,700),(740,525),(780,525)])
    f.edge('producer','prodmodel',[(385,455),(385,437),(1835,437),(1835,455)])
    f.edge('prodmodel','producer',[(1835,605),(1835,622),(385,622),(385,605)],dashed=True)
    across(f,'dispute','read',715,dashed=True)
    across(f,'interpret','receipt',940)
    across(f,'adopt','used',1120)
    model_pair(f,'interpret','cloud',3)
    across(f,'change','adopt',1085,dashed=True)
    across(f,'failure','out',1270,dashed=True)
    finish(f,'기본의 이익과 비용: 교차 대화의 반복 관계 조합 감소 ↔ 선생산, 미사용 기록, 갱신과 원본 보완 비용.',
           [['C1 선택적 기억 생산', '재사용 목적이 있는 허용 범위부터 생산',
             '추가: 선별 / 미생산 fallback / 오분류 비용', '업무 인계와 공유한 생산 비용은 한 번만 계상'],
            ['C2 변경 묶음 갱신', '여러 허용 변경을 묶어 관계를 재생산',
             '추가: 게시 대기 / stale 범위의 원본 보완', '삭제와 권한 철회의 사용 차단은 즉시 유지']])


HINPUT=['1 현재 인계 요청과 채택 상태', 'Request Controller / Task Manager', '목표 / 조건 / 제외 / 허용 Action', 'Agent 선택은 공통으로 고정한 별도 결정']
HGATE=['4 외부 실행 허가 계약 확정', '요청 / attempt / delegation 판본', 'source / 권한 / 필수 조건 재확인', '불명인 동일 Action의 중복 실행 금지']
HAGENT=['외부 Downstream Agent 경계', '받은 계약 안에서 domain 판단 / 실행', '비공개 추론 / memory 이전 가정 없음', '수신 확인과 실제 수행은 다른 사실']
HCHANGE=['정정 / 철회 / 실행 상태 불명', '원본 revision / 권한 epoch를 먼저 변경', '미전송 계약과 늦은 이전 완료 차단', '이미 보낸 외부 자료의 회수는 별도']


def handoff_common(f):
    node(f,'raw','l',0,ORIGINALS,True)
    node(f,'in','m',0,HINPUT)
    node(f,'scope','r',0,['공통 실제 근거 / 권한 계약', '원본 + 공개 결과 + 실제 전달만 사용', '필수 조건은 수신자 추측에 맡기지 않음', '공통 생산은 01과 한 번만 비용 계상'])
    node(f,'gate','m',3,HGATE)
    node(f,'sent','l',3,['내구 원장: 계약 / 송신 / 수신 사실', '4 계약 확정, 5 실제 송신 등록', 'Agent Gateway: 수신 확인은 별도 기록', 'Task Manager: 공개 수행 사실만 기록'],True)
    node(f,'change','r',3,HCHANGE)
    node(f,'send','m',4,['5 Agent Gateway가 확정 계약 전송', 'adapter의 입력 형식 / 예산을 준수', '실제 전송 등록과 철회 경쟁을 차단', '정정 적용 완료는 외부 확인과 구별'])
    node(f,'agent','r',4,HAGENT)
    for a,b in [('in','prepare'),('prepare','candidate'),('candidate','gate'),('gate','send')]:down(f,a,b)
    across(f,'gate','sent',935)
    across(f,'change','gate',900,dashed=True)
    across(f,'send','agent',1085)


def handoff_a():
    f=start('problem05-A-detail','업무 인계 A: 실제 위임이 생길 때 목적별 자료 묶음을 구성한다',
            '같은 업무: 수정한 비교 결과를 새 Agent의 발표자료 / 메일 작성에 사용한다. Agent 내부 실행 상태는 옮기지 않는다.',False)
    node(f,'work','l',1,['공통 상태만 지속 보존', '현재 Task 조건 / 공개 결과 / 전달 구간', '이미 확보한 01의 유효 근거도 사용', '미래 인계용 의미 record 생산은 없음'],True)
    node(f,'prepare','m',1,['2 현재 위임 목적의 자료를 선택', 'Context Manager가 유효 원본을 구성', '명시 ID / 조건 / 상태는 코드로 읽음', '필요한 발췌 / 설명 / 결과 참조 선택'])
    node(f,'model','r',1,['2 자유문 선택 / 요약 필요 때만 호출', 'Model Access → cloud 의미 모델', '공개 원본과 현재 조건만 사용', '조건 / source 판본 검사 자체는 코드'])
    node(f,'cap','l',2,['요청 수명: 준비 중 capsule', '목표 / 필수 조건 / 실제 전달 근거', 'source identity / revision / 범위', '유효 artifact 참조도 기본에서 허용'],True)
    node(f,'candidate','m',2,['3 목적별 인계 묶음 후보 준비', '큰 자료는 선택한 유효 참조로 전달', '부족한 원본은 bounded read로 보완', '필수 항목이 없으면 4로 진행하지 않음'])
    node(f,'failure','r',2,['read 실패 / 만료 / source 변경', '동일 판본 보완 또는 명시적 보류', 'capsule source와 목적을 다시 확인', '준비 실패를 외부 성공으로 표시 안 함'])
    node(f,'ledger','l',4,['보존 원장: 조건 / 삭제 / 수행 범위', '재시작 시 현재 권한과 source 재확인', '같은 불명 Action은 다시 보내지 않음', '별도로 허용된 업무는 계속 가능'],True)
    node(f,'out','m',5,['6 공개 결과 / 질문 / 실패를 수신', 'Task Manager가 외부 사실을 확인', 'Request Controller가 같은 업무에 연결', '새 목적이면 1에서 새 묶음을 구성'])
    node(f,'resume','r',5,['외부 실패 / native session 유실', '공개 원본에서 portable 묶음 재구성', 'fresh 성공이 이전 session 복원은 아님', '외부 비용 / 취소 후 청구는 따로 기록'])
    handoff_common(f)
    across(f,'work','prepare',525);across(f,'candidate','cap',750)
    model_pair(f,'prepare','model',1)
    across(f,'failure','candidate',715,dashed=True)
    f.edge('agent','out',[(1835,1160),(1835,1177),(1105,1177),(1105,1195)])
    across(f,'resume','out',1270,dashed=True)
    finish(f,'기본의 이익과 비용: 실제 위임 목적에만 구성 ↔ 같은 자료의 반복 인계에서 선택 / 조합 반복.',
           [['A1 인계 묶음 cache', '같은 목적 / 조건 / source 판본의 묶음 재사용',
             '추가: 저장 / 무효화 / 권한 재확인', '유효 artifact 참조는 원래 기본 기능'],
            ['A2 같은 Agent의 native session 재사용', '재개와 유효성을 확인한 본문만 생략',
             '추가: capability / 재개 실패 처리', '필수 조건은 명시. fresh / 새 Agent는 기본 A']])


def handoff_b():
    f=start('problem05-B-detail','업무 인계 B: 공개 업무 기록을 유지하고 위임 목적에 맞춰 추출한다',
            '같은 원본과 현재 조건. 실제 인계가 없는 사건도 생산 비용에 포함하며, 공통 Task / 01 근거 복제는 새 이익이 아니다.',False)
    node(f,'work','l',1,['지속 업무 기록: Context Manager', '공개 결과의 확인 범위 / 미확인 설명', '조건과 결과의 source-backed 관계', 'record / 의존성 / coverage / job'],True)
    node(f,'prepare','m',1,['2 목적에 맞는 업무 기록 조회', 'Context Manager: 유효 record에서 추출', '공통 기록에 없는 현재 목적은 추가 구성', '현재 Task 조건은 원본과 대조'])
    node(f,'model','r',1,['별도 생산: 자유문 관계 필요 때 호출', 'Model Access → cloud 의미 모델', '공개 결과 / 실제 전달 변경으로 갱신', '구조화 필드는 코드, domain 계획 제외'])
    node(f,'cap','l',2,['지속 게시와 요청별 추출을 구별', 'record / coverage / job 일관 확정', '목적별 capsule은 요청 수명', '원본과 같은 내용이면 추가 이익 없음'],True)
    node(f,'candidate','m',2,['3 추출 자료와 부족한 원본을 결합', 'source 판본 / 실제 전달 범위 확인', '새 목적의 원본 보완 / 재구성도 계상', '준비가 끝난 인계 후보만 4로 전달'])
    node(f,'failure','r',2,['게시 / 추출 오류 또는 source 변경', '해당 record revision을 사용 차단', '재생산 또는 허용 원본에서 보완', '필수 근거 없으면 보류 / 확인'])
    node(f,'ledger','l',4,['복구 원장: record / 게시 cursor', '삭제 원장 먼저, 확정 게시부터 복원', '오래된 job의 재게시를 거절', '수행 불명 Action은 다시 보내지 않음'],True)
    node(f,'out','m',5,['6 공개 사건을 보존하고 기록 갱신', 'Task Manager / Response Manager 원본', '다음 인계는 갱신된 record를 재사용', '미소비 생산 / 폐기 / 재시도도 계상'])
    node(f,'resume','r',5,['B의 실제 이익이 없는 경우', '공통 Task / 01이 같은 설명을 이미 제공', 'A도 그 근거를 사용하므로 추가 절약 0', '목적마다 재구성이면 상각 이익 감소'])
    handoff_common(f)
    down(f,'raw','work',385)
    across(f,'work','prepare',525);across(f,'candidate','cap',750)
    f.edge('work','model',[(385,455),(385,437),(1835,437),(1835,455)])
    f.edge('model','work',[(1835,605),(1835,622),(385,622),(385,605)],dashed=True)
    across(f,'failure','candidate',715,dashed=True)
    f.edge('agent','out',[(1835,1160),(1835,1177),(1105,1177),(1105,1195)])
    f.edge('out','work',[(1105,1345),(1105,1355),(35,1355),(35,590),(70,590)])
    finish(f,'기본의 이익과 비용: 반복 위임 준비 대기 감소 ↔ 선행 생산 / 갱신 / 미소비 비용. N-H 자동 우위는 없음.',
           [['B1 변경 묶음 갱신', '여러 공개 사건을 묶어 기록 생산',
             '추가: 최신 record 대기 / 원본 보완', '철회 사용 차단은 갱신을 기다리지 않음'],
            ['B2 변경 조각 갱신', '확인된 dependency의 설명 블록만 재생산',
             '추가: index / 부분 정합성 검사 / fallback', 'B+C 수신자 읽기는 tactic이 아닌 구조 조합']])


def handoff_c():
    f=start('problem05-C-detail','업무 인계 C: 필수 명세를 보내고 수신 Agent가 세부 자료를 읽는다',
            '같은 목표와 필수 조건. 실제 읽기 capability를 가진 Agent에서만 적용하며, 수신자 비용을 VIA 절약과 혼동하지 않는다.',False)
    node(f,'work','l',1,['공통 상태와 adapter capability', '현재 목표 / 조건 / 결과 locator', 'protocol / 형식 / namespace / 권한', '읽기 미지원이면 A/B 전환 또는 보류'],True)
    node(f,'prepare','m',1,['2 필수 manifest와 자료 목록 구성', 'Context Manager: 목적과 원본 범위 연결', '필수 조건은 명시하고 세부 선택만 위임', '모든 원문을 미리 골라 요약하지 않음'])
    node(f,'model','r',1,['2 필수 설명에 의미 가공 필요 때 호출', 'Model Access → cloud 의미 모델', '명시 조건 / 판본 / 접근 검사는 코드', '수신자의 domain 자료 선택과 구별'])
    node(f,'cap','l',2,['지속 접근 상태: manifest / lease', '수신자 / source revision / 허용 범위', '만료 / 철회 epoch / read receipt', 'session ID만으로 권한을 주지 않음'],True)
    node(f,'candidate','m',2,['3 유효 자료 접근을 준비', '실제 읽기 방법과 범위 제한을 확인', '허용 자료의 판본 / lease / 만료 명시', '필수 자료 접근이 없으면 허가하지 않음'])
    node(f,'failure','r',2,['접근 실패 / 판본 변경 / 기능 미지원', '묵시적 최신판 교체나 성공 처리 금지', '허용 계약 갱신 또는 A/B로 명시 전환', '지원 가능한 일부만 전체 성공으로 안 셈'])
    node(f,'ledger','l',4,['복구 / 철회 원장과 read receipt', 'lease 복원 전에 현재 권한을 재검사', '철회 즉시 신규 읽기와 늦은 반환 차단', '외부에 이미 전달한 자료 회수는 별도'],True)
    node(f,'out','m',5,['6 요청된 범위마다 자료 읽기 제공', 'Context Manager: 권한 / 판본 재확인', '유효 구간 / 누락 / read receipt 반환', '자료 읽기 자체가 외부 실행 허가는 아님'])
    node(f,'pull','r',5,['수신 Agent가 필요한 상세를 선택', 'manifest를 받은 뒤 허용 범위를 요청', '읽은 자료로 domain 업무 수행', '공개 결과는 Task Manager로 별도 수신'])
    handoff_common(f)
    # C's sixth step is entered by an external read, not automatically after send.
    across(f,'work','prepare',525);across(f,'candidate','cap',750)
    model_pair(f,'prepare','model',1)
    across(f,'failure','candidate',715,dashed=True)
    down(f,'agent','pull',1835)
    across(f,'pull','out',1245)
    across(f,'out','pull',1310,dashed=True)
    finish(f,'기본의 이익과 비용: VIA의 선행 세부 선택 감소 ↔ 읽기 지원 / 왕복 / 접근 관리와 외부 선택 비용.',
           [['C1 특정 자료 prefetch', '사용 가능성이 높은 원본을 미리 준비',
             '추가: 미사용 I/O / buffer / 만료 처리', '작은 반환 범위가 작은 backend RAM은 아님'],
            ['C2 읽기 결과 cache', '같은 허용 판본 / 범위의 결과를 재사용',
             '추가: 저장 / 무효화 / 수신자 권한 확인', 'QA03 종료 뒤 serving / 외부 비용은 별도']])


if __name__ == '__main__':
    for build in [evidence_a,evidence_b,evidence_c,handoff_a,handoff_b,handoff_c]:
        build()
    report={'purpose':'documentation layout, not performance measurement', 'figures':GENERATED}
    (OUT/'evidence-handoff-detail-layout.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(report,ensure_ascii=False,indent=2))
