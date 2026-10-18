"""Render problem 02's two detailed design figures.

Documentation assets only. Imports the shared renderer without changing it.
No candidate execution, model calls, or performance measurement.
"""

from generate_seven_problem_figures import Figure, BLUE, GRAY, ORANGE


def common_inputs(f):
    f.node('user', 55, 235, 650, 165, [
        '1a 사용자 입력: Interaction Manager',
        '수신 / VAD에서 로컬 음성 즉시 중단 [코드]',
        '원문 / 전사 revision / 화면 근거 참조',
        '새 질문, 정정, 실제 표시한 질문의 답변',
    ])
    f.node('agent', 775, 235, 650, 165, [
        '1b 외부 Agent 통지: Task Manager',
        '진행 / 질문 / 승인 필요 / 결과 / 실패',
        '접수 ID와 외부에서 확인된 사실 보존',
        '취소 요청과 확인된 종료는 별도 사실',
    ])
    f.node('delivered', 1495, 235, 650, 165, [
        '1c 실제 전달 receipt: 표시 / 재생 결과',
        'Interaction Manager: UI 표시 / 실제 재생',
        'Response Manager: 전달 범위 / Q1 기록',
        '생성 문장과 실제 전달 / 미전달을 구별',
    ])
    f.node('ingress', 775, 450, 650, 165, [
        '2 입력 등록 / hold: Request Controller [코드]',
        '입력 / revision 보존, 관련 미전송 효과 hold',
        '명시 ID 제어 / 단순 상태는 코드로 6에 전달',
        '관계가 모호한 입력은 의미 판단에 전달',
    ])
    f.node('records', 55, 450, 650, 165, [
        '원본 상태: 각 owner의 내구 원장',
        'Request Controller: 입력 / 목표 / revision',
        'Task Manager: Task / execution / 외부 사실',
        'Response Manager: 실제 전달 / 질문 receipt',
    ], state=True)
    f.node('cloud', 1495, 450, 650, 190, [
        'cloud 모델 호출 경계: 공통 상한 K',
        '모든 [모델] 단계는 같은 예산에서 admission',
        '시작한 호출은 실제 종료까지 slot 점유',
        '외부 Agent 업무 대기는 VIA slot 미점유',
        'API 송신 시각은 provider 계산 시작 아님',
    ])
    f.edge('user', 'ingress', [(380, 400), (380, 425), (930, 425), (930, 450)])
    f.edge('agent', 'ingress', [(1100, 400), (1100, 450)])
    f.edge('delivered', 'ingress', [(1820, 400), (1820, 425), (1270, 425), (1270, 450)])
    f.edge('ingress', 'records', [(775, 530), (705, 530)])


def basic_a():
    f = Figure(
        'problem02-A-detail',
        '문제 2 / 기본 A: 하나의 대화 해석 실행에서 다음 판단으로 이어 간다',
        'D1 판단 중 새 질문, D2 요청, “비용만” 정정과 Agent의 Q1이 들어온다. 앞선 요청 전체 완료를 기다리지 않는다.',
        width=2200, height=1700,
    )
    common_inputs(f)
    f.node('context', 55, 665, 650, 165, [
        '대화 해석 문맥: Request Interpreter',
        '대화 수명 동안 읽은 근거와 잠정 판단 유지',
        'R1 / R2의 미해결 목표와 사용 근거를 구별',
        '모델 문맥은 왼쪽 위 내구 원장의 대체물 아님',
    ], state=True)
    f.node('boundary', 775, 665, 650, 130, [
        '3 다음 판단 경계에서 입력 반영 [코드]',
        '현재 VIA 호출 / 읽기 step의 종료를 기다림',
        '원래 목표 전체 완료나 외부 업무 종료는 안 기다림',
    ])
    f.node('active', 1495, 690, 650, 140, [
        '진행 중인 step: Request Interpreter',
        '필요한 원본 조회 [코드 / I/O]',
        '자유 의미 판단이 필요할 때만 [모델]',
    ])
    f.node('evidence', 55, 880, 650, 165, [
        '필요 근거 공급: 문제 1의 같은 정책',
        '새 입력 + 최근 실제 전달 + 활성 목표',
        '유지 문맥의 유효 근거를 계속 사용',
        '부족한 범위는 허용 원본을 bounded read',
    ])
    f.node('interpret', 775, 880, 650, 165, [
        '4 관계와 내용 판단: Request Interpreter',
        '모호한 의미 / 대상 / 목표 연결이면 [모델]',
        '독립 질문, 정정, Q1 답변의 관계를 제안',
        '새 입력 전에 읽은 유효 문맥도 함께 사용',
    ])
    f.node('branch', 1495, 880, 650, 165, [
        '5 관계에 따른 다음 처리',
        '독립 질문: 원래 목표를 남기고 새 답 준비',
        '정정: 해당 요청의 변경 / 보존 범위 반영',
        'Q1 답변: 실제 제시한 질문과 연결',
    ])
    f.node('display', 55, 1100, 650, 165, [
        '7 Response Manager: 응답 준비 / 게시 원장',
        'Interaction Manager: 실제 UI 표시 / 음성 재생',
        '표시 / 재생 receipt로 실제 전달 범위 기록',
        '실제 표시한 질문의 답변은 새 입력 1a로',
    ])
    f.node('adopt', 775, 1100, 650, 165, [
        '6 현재성 확인 / 채택: Request Controller',
        '입력 / source revision, 관계, 권한 재확인',
        '미해결 효과는 hold, 재판단이 필요하면 4로',
        '늦은 결과는 보관해도 최신 답으로 채택 안 함',
    ])
    f.node('failure', 1495, 1100, 650, 165, [
        '실패 / 늦은 완료: 같은 실행에서 복구',
        '실패 step과 유효 목표를 보존해 다시 진행',
        '취소 요청과 실제 종료를 구별해 slot 추적',
        '외부 수행 불명이면 새 Action으로 재시작 금지',
    ])
    f.node('resume', 55, 1330, 2090, 100, [
        '8 남은 유효 목표 / 입력을 같은 실행에서 계속 처리 → 다음 판단 경계 3',
        '외부 Agent 결과나 사용자 답변을 기다릴 때는 VIA 호출 자원을 놓는다. 내구 원본과 실제 전달 기록은 해석 문맥보다 오래 보존한다.',
    ])
    f.edge('ingress', 'boundary', [(1100, 615), (1100, 665)])
    f.edge('records', 'context', [(380, 615), (380, 665)], dashed=True)
    f.edge('cloud', 'active', [(1820, 640), (1820, 690)], dashed=True)
    f.edge('active', 'boundary', [(1495, 735), (1425, 735)])
    f.edge('boundary', 'interpret', [(1100, 795), (1100, 880)])
    f.edge('context', 'evidence', [(380, 830), (380, 880)])
    f.edge('evidence', 'interpret', [(705, 960), (775, 960)])
    f.edge('interpret', 'branch', [(1425, 960), (1495, 960)])
    f.edge('branch', 'adopt', [(1820, 1045), (1820, 1070), (1100, 1070), (1100, 1100)])
    f.edge('ingress', 'adopt', [(1425, 565), (1460, 565), (1460, 1180), (1425, 1180)], dashed=True)
    f.edge('adopt', 'display', [(775, 1180), (705, 1180)])
    f.edge('display', 'records', [(55, 1180), (25, 1180), (25, 530), (55, 530)], dashed=True)
    f.edge('adopt', 'resume', [(1100, 1265), (1100, 1330)])
    f.edge('failure', 'resume', [(1820, 1265), (1820, 1330)])
    f.label(55, 1450, '선택 후 보완: 아래 기법은 기본 A의 정상 경로에 포함하지 않는다. A2는 A1을 자동 포함하지 않는다.', 25, ORANGE)
    f.node('tactic_a1', 55, 1495, 1010, 115, [
        'A1 제한된 별도 질문: 도구 없는 side 답변 경로',
        '짧은 질문의 대기를 줄이되 실제 전달 이력은 보존',
        '추가 근거 필요 / 분류 오류이면 본 실행 복귀와 중복 비용',
    ])
    f.node('tactic_a2', 1135, 1495, 1010, 115, [
        'A2 현재 호출 중단 / 재개: 최신 정정으로 다시 판단',
        '취소 요청 뒤 실제 종료까지 slot 유지, 폐기 / 재입력 비용',
        '독립 질문 때문에 중단했다면 원래 목표의 재개 비용도 기록',
    ])
    f.save()


def basic_b():
    f = Figure(
        'problem02-B-detail',
        '문제 2 / 기본 B: 공통 대화를 유지하고 긴 판단을 요청별 실행으로 나눈다',
        'A와 같은 원본 / 실제 전달 / 호출 상한 K. 짧은 답은 바로 반환하며 R1과 R2는 자원 여유가 있을 때만 함께 진행한다.',
        width=2200, height=1700,
    )
    common_inputs(f)
    f.node('common_context', 55, 665, 650, 165, [
        '공통 최근 문맥: Request Interpreter',
        '최근 실제 전달, 활성 요청 / 질문, 원문 참조',
        '문제 1의 유효한 공통 view도 그대로 사용 가능',
        '대화 수명 유지. 원본은 각 owner 원장에 보존',
    ], state=True)
    f.node('common_judgment', 775, 665, 650, 165, [
        '3 공통 대화 해석: Request Interpreter',
        '관계 / 내용의 자유 의미가 필요할 때 [모델]',
        '짧은 답은 같은 판단에서 생성해 4a로 반환',
        '긴 판단만 목표 / revision / 근거 참조로 분리',
    ])
    f.node('short', 1495, 690, 650, 140, [
        '4a 짧은 답 / 확인 질문 → 6으로 바로',
        '별도 요청 worker 호출을 강제하지 않음',
        '모호하면 질문을 실제 표시하고 답변은 1a로',
    ])
    f.node('run1', 55, 885, 650, 235, [
        '4b R1 실행: Request Interpreter',
        'D1 목적에 필요한 근거를 골라 원본 조회',
        '자유 의미 판단 필요 시 [모델], 매 호출 admission',
        '상태: R1 revision / 근거 / 잠정 결과 / 대기',
        '수명: R1의 수정 / 재개 / 완료까지',
        '유효한 공통 근거도 사용하며 원본 참조 보존',
    ], state=True)
    f.node('run2', 775, 885, 650, 235, [
        '4c R2 실행: Request Interpreter',
        'D2 목적에 필요한 근거를 골라 원본 조회',
        '자유 의미 판단 필요 시 [모델], 매 호출 admission',
        '상태: R2 revision / 근거 / 잠정 결과 / 대기',
        '수명: R2의 수정 / 재개 / 완료까지',
        '전체 대화 복제나 선별 정답을 가정하지 않음',
    ], state=True)
    f.node('correction', 1495, 885, 650, 235, [
        '5 정정 / 확인 답변 / Agent 질문 연결',
        '관계는 3에서 판단하고 해당 Request에 전달',
        '정정: 해당 R1 또는 R2의 revision / 목표 갱신',
        '미전송 효과 hold, 모호하면 4a에서 확인',
        'Q1 답변: 실제 질문 receipt를 보고 연결',
        '대응 / 변경 / 완료 ID: Request Controller 소유',
    ])
    f.node('failure', 55, 1170, 650, 165, [
        '실패 / 포화: 실행 상태와 목표 보존',
        '실패 run 재개, 무관한 다른 목표는 유지',
        '취소 요청 ≠ 실제 종료: 종료까지 slot 점유',
        '외부 수행 불명은 재시작하지 않고 상태 확인',
    ])
    f.node('adopt', 775, 1170, 650, 190, [
        '6 각 반환 확인 / 채택: Request Controller',
        '실행 ID / revision / 관계 / 권한을 재확인',
        '재결합 필요 시 Request Interpreter [모델]',
        '필요한 결과만 최신 대화와 결합해 채택',
        '늦은 결과 보관, 현재 답으로는 채택하지 않음',
    ])
    f.node('display', 1495, 1170, 650, 165, [
        '7 Response Manager: 응답 준비 / 게시 원장',
        'Interaction Manager: 실제 UI 표시 / 음성 재생',
        '표시 / 재생 receipt로 실제 전달 범위 기록',
        '이 기록이 다음 입력 관계 판단의 근거가 됨',
    ])
    f.node('resume', 55, 1380, 2090, 80, [
        '8 공통 대화는 다음 입력을 받고 각 실행은 남은 목표를 계속한다. 모든 run의 완료를 한꺼번에 기다리지 않는다.',
        '원본 취득 정책은 A와 같고 실행별 문맥만 선별한다. 완료 상태 보존 후 실행 문맥을 해제하며 내구 원장 수명은 별도다.',
    ])
    f.edge('ingress', 'common_judgment', [(1100, 615), (1100, 665)])
    f.edge('records', 'common_context', [(380, 615), (380, 665)], dashed=True)
    f.edge('common_context', 'common_judgment', [(705, 745), (775, 745)])
    f.edge('cloud', 'common_judgment', [(1495, 570), (1460, 570), (1460, 745), (1425, 745)], dashed=True)
    f.edge('common_judgment', 'short', [(1425, 785), (1495, 785)])
    f.edge('common_judgment', 'run1', [(970, 830), (970, 855), (380, 855), (380, 885)])
    f.edge('common_judgment', 'run2', [(1100, 830), (1100, 885)])
    f.edge('common_judgment', 'correction', [(1250, 830), (1250, 855), (1820, 855), (1820, 885)])
    f.edge('correction', 'run2', [(1495, 1000), (1425, 1000)], dashed=True)
    f.edge('correction', 'run1', [(1495, 1060), (1450, 1060), (1450, 1135), (740, 1135), (740, 1060), (705, 1060)], dashed=True)
    f.edge('run1', 'adopt', [(380, 1120), (380, 1145), (970, 1145), (970, 1170)])
    f.edge('run2', 'adopt', [(1100, 1120), (1100, 1170)])
    f.edge('short', 'adopt', [(2145, 765), (2170, 765), (2170, 1145), (1250, 1145), (1250, 1170)])
    f.edge('ingress', 'adopt', [(1425, 530), (1475, 530), (1475, 1250), (1425, 1250)], dashed=True)
    f.edge('adopt', 'display', [(1425, 1290), (1495, 1290)])
    f.edge('display', 'delivered', [(2145, 1250), (2190, 1250), (2190, 315), (2145, 315)], dashed=True)
    f.edge('adopt', 'resume', [(1100, 1360), (1100, 1380)])
    f.edge('failure', 'resume', [(380, 1335), (380, 1380)])
    f.label(55, 1470, '선택 후 보완: 아래 기법은 기본 B의 정상 경로에 포함하지 않는다. B2는 B1을 자동 포함하지 않는다.', 25, ORANGE)
    f.node('tactic_b1', 55, 1510, 1010, 100, [
        'B1 불변 문맥 참조 공유: 같은 근거 buffer의 로컬 중복 감소',
        'version / 권한 철회 / 해제 관리 추가. cloud 토큰 절감과 구별',
    ])
    f.node('tactic_b2', 1135, 1510, 1010, 100, [
        'B2 foreground 호출 여유 예약: 같은 총 K 안에서 새 입력 보호',
        '원래 요청의 시작 / 완료 지연 비용. K를 늘려 얻은 이익이 아님',
    ])
    f.save()


if __name__ == '__main__':
    basic_a()
    basic_b()
    print('Generated problem02-A-detail and problem02-B-detail as SVG, draw.io, and PNG.')
