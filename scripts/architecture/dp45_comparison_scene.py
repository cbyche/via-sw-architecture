"""DP45 focused comparison: request composition versus publish/read lifetimes.

Editable illustration only. It does not execute an evidence producer or model.
Contract examples are in contracts/dp45-evidence-examples.json; the deliberately
partial example contains one delivered exception clause, not every criterion.
"""
from dp_comparison_structures import ResolutionGraph, APRICOT, DIFFERENCE_STROKE
from dp41_comparison_scene import document
from dp44_comparison_scene import legend


def state(g, key, x, y, w, h, name, different=True, size=16):
    before = len(g.s.items)
    g.store(key, x, y, w, h, name, size=size)
    if different:
        g.s.items[before].update(fill=APRICOT, stroke=DIFFERENCE_STROKE)


def draw(s, x, side):
    g = ResolutionGraph(s, x)
    if side == 0:
        legend(s)
    # The raw owners and the current request consumer are common Components.
    g.box('rm', 35, 253, 290, 31, 'Response Manager', size=18)
    g.box('tm', 525, 253, 300, 31, 'Task Manager', size=18)
    g.text(35, 291, 425, 'P20: 실제 전달 / E20: 설명 원문', size=14)
    g.text(525, 291, 315, 'T21 → D21 개정 2 / T22 → D22 개정 1', size=12)
    g.text(35, 310, 790, '같은 원본: 전달한 예외 문구 + 제품 조사 결과 + 견적 결과', size=14)
    g.group(25, 344, 810, 318, 'Context Manager')

    if side == 0:
        provider = 'composer'
        g.box(provider, 55, 392, 290, 37, 'Context Composer', True, True, size=19)
        state(g, 'cache', 55, 550, 290, 43, 'Source View Cache', False, 17)
        state(g, 'set', 485, 550, 310, 43, 'Request Evidence Set', size=17)
        for owner, offset in [('rm', -12), ('tm', 12)]:
            g.edge(provider, owner, 'T', 'B', sd=offset+100, td=offset,
                   via=[(300+offset, 331), (g.port(owner, 'B', offset)[0], 331)])
            g.edge(owner, provider, 'B', 'T', sd=offset+6, td=offset+106,
                   via=[(g.port(owner, 'B', offset+6)[0], 337), (306+offset, 337)], ret=True)
        g.text(390, 354, 410, '② 이번 요청에 맞춘 원본 조회 / 반환', size=14)
        g.edge(provider, 'cache', 'B', 'T', sd=-28, td=-28)
        g.edge('cache', provider, 'T', 'B', sd=28, td=28, ret=True)
        g.text(65, 458, 270, '③ 원본 확인 / 유효 cache 재사용', size=13)
        g.text(65, 481, 270, '부족하면 추가 조회 / 부분 갱신', size=14)
        g.edge(provider, 'set', 'R', 'T', via=[(365, 410.5), (365, 539), (640, 539)])
        g.edge('set', provider, 'T', 'R', sd=15, td=10,
               via=[(655, 533), (375, 533), (375, 420.5)], ret=True)
        document(g, 'data', 405, 389, 390, 132, [
            '이번 요청의 Bundle1 / Receipt1: PARTIAL',
            '원본 발췌 E20[27,52), 실제 전달 P20:',
            '“가격이 같은 경우 유지보수 기간을',
            '우선합니다.”',
            'T21 → D21 개정 2 / T22 → D22 개정 1',
            '읽은 범위 / source revision / 누락 포함'], 12)
        g.text(55, 603, 290, 'cache는 선택 tactic / 이번 조회가 구성 책임', size=12)
        g.text(485, 603, 310, '④ 요청 수명: 발췌 / 결과 참조 / 읽은 범위', size=12)
        g.text(55, 630, 735, '정정 / 삭제 → 파생 버전 차단 → 원본 재검증 / 재조회', size=14)
    else:
        provider = 'reader'
        g.box('publisher', 55, 392, 290, 37, 'Memory Publisher', True, True, size=19)
        state(g, 'repo', 485, 392, 310, 48, 'Episodic Memory Repository', size=15)
        g.box(provider, 55, 550, 290, 37, 'Evidence Reader', True, True, size=19)
        state(g, 'coverage', 485, 550, 310, 43, 'Coverage State', size=17)
        for owner, offset in [('rm', -12), ('tm', 12)]:
            g.edge(owner, 'publisher', 'B', 'T', sd=offset, td=offset+100,
                   via=[(g.port(owner, 'B', offset)[0], 331), (300+offset, 331)])
            g.edge('publisher', owner, 'T', 'B', sd=offset+106, td=offset+6,
                   via=[(306+offset, 337), (g.port(owner, 'B', offset+6)[0], 337)], ret=True)
        g.text(390, 354, 410, '① 원본 변경 알림 → 범위 읽기 / 생산', size=14)
        g.edge('publisher', 'repo', 'R', 'L', sd=-7, td=-12)
        g.text(355, 382, 120, '같은 개정 게시', size=11)
        g.edge('publisher', 'coverage', 'R', 'L', sd=9, td=-9,
               via=[(465, 419.5), (465, 562.5)])
        document(g, 'data', 485, 453, 310, 81, [
            'Memory1: EXPLAINS_ITEM(P20 → E20)',
            'MODEL_DERIVED / source[27,52)',
            'Memory2/3: HAS_RESULT(T21/22 → D21/22)',
            'EXPLICIT / 확인된 개정 참조'], 10.5)
        g.edge(provider, 'repo', 'R', 'B', sd=-10,
               via=[(415, 558.5), (415, 446), (640, 446)])
        g.edge('repo', provider, 'B', 'R', sd=15, td=3,
               via=[(655, 449), (425, 449), (425, 571.5)], ret=True)
        g.edge('coverage', provider, 'L', 'R', sd=9, td=13, ret=True)
        # Real retry graph: a missing supported range enters the producer; its
        # publication receipt returns to the Reader, which reads again above.
        g.edge(provider, 'publisher', 'T', 'B', sd=-28, td=-28)
        g.edge('publisher', provider, 'B', 'T', sd=28, td=28, ret=True)
        g.text(65, 460, 275, 'NOT_COVERED ↑ 지원 범위 생산', size=12)
        g.text(65, 483, 270, '게시 revision ↓ 확인 뒤 Reader 재조회', size=12)
        g.text(55, 603, 350, 'UNSUPPORTED_RELATION: 지원 밖 / 생산 안 함', size=12)
        g.text(485, 603, 310, 'Coverage1[27,52) / gap[0,27) / 개정 1', size=12)
        # Reader verifies raw owner revisions and requested original excerpts.
        # Bottom/right corridors avoid Repository, Coverage and producer nodes.
        for owner, xx, yy, offset in [('tm', 848, 621, 70), ('rm', 840, 625, 90)]:
            target_side = 'R' if owner == 'tm' else 'B'
            target_offset = 9 if owner == 'tm' else 100
            target = g.port(owner, target_side, target_offset)
            turn = [(xx, target[1])] if owner == 'tm' else [(xx, 323), (target[0], 323)]
            g.edge(provider, owner, 'B', target_side, sd=offset, td=target_offset,
                   via=[(200+offset, yy), (xx, yy), *turn])
            back = [(xx-4, target[1]+5)] if owner == 'tm' else [(target[0]+5, 327), (xx-4, 327)]
            g.edge(owner, provider, target_side, 'B', sd=target_offset+5, td=offset+5,
                   via=[*back, (xx-4, yy+2), (205+offset, yy+2)], ret=True)
        g.text(365, 620, 425, 'Reader → owner: 현재 개정 / 요청한 원문 확인', size=12)
        g.text(55, 640, 735, '정정 / 삭제 → 사용 fence → 부분 재생산 / 옛 job 게시 거부', size=12)

    # One common model-access dependency, with a conditional context-production
    # call. Current request interpretation and final adoption stay outside CM.
    g.box('ma', 35, 684, 290, 28, 'Model Access', size=17)
    g.box('cloud', 525, 684, 300, 28, '클라우드 의미 LLM', size=17)
    s.items[-2]['external'] = True
    g.edge(provider if side == 0 else 'publisher', 'ma', 'L', 'T',
           via=[(20, 410.5), (20, 675), (180, 675)])
    g.edge('ma', provider if side == 0 else 'publisher', 'T', 'L', sd=12, td=10,
           via=[(192, 670), (12, 670), (12, 420.5)], ret=True)
    g.edge('ma', 'cloud', 'R', 'L', sd=-4, td=-4)
    g.edge('cloud', 'ma', 'L', 'R', sd=5, td=5, ret=True)
    g.text(355, 683, 155, '조건부 C-CONTEXT', size=12)
    g.text(35, 717, 790, 'Query1 {purpose:REQUEST_UNDERSTANDING, relation_kinds:[EXPLAINS_ITEM,HAS_RESULT], original_required:true}', size=10.5)
    g.box('ri', 35, 744, 290, 30, 'Request Interpreter', size=17)
    g.box('rc', 525, 744, 300, 30, 'Request Controller', size=17)
    g.edge('ri', provider, 'L', 'L', td=10,
           via=[(8, 759), (8, 420.5 if side == 0 else 578.5)])
    g.edge(provider, 'ri', 'L', 'T', sd=-9,
           via=[(30, 401.5 if side == 0 else 559.5), (30, 737), (180, 737)], ret=True)
    g.text(365, 657, 440, 'Query1 → / ← Bundle1 + Receipt1 (PARTIAL)', size=12)
    g.edge('ri', 'rc', 'R', 'L')
    g.text(352, 751, 161, '의미 제안 → 검증 / 채택', size=11)
    g.finish()
