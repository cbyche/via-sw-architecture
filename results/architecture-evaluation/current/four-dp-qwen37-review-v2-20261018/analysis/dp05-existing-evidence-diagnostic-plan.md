# DP05 기확보 공통 근거 진단 계획 — 미실행

2026-10-18. 읽기와 계획만. 현재 BASIC source/binary/입력/원시는 변경하지 않았고 추가 API 호출은 없다.

## 문서와 구현의 구별

설계 `05-agent-handoff.md` §2.2(현재 line52)는 A도01에서 이미 얻은 유효근거를 쓰며 인계비교를 위해 숨기지 않는다고 한다. §4.1(line94–99)은 현재목표/이미확보근거/공통원본에서 목적별 선택하고, 즉석구성이 기존확보근거를 버리는 뜻은 아니라고 명시한다. 별도 미래인계용 의미record와 capsule cache/native session은 기본A에 없다. 따라서 **이미 실제읽은 원본passage를 다음목적에서 사용 가능하게 하는 것은 capsule cache가 아니며 기본A의 허용기능**이다.

현재 frozen `dp05.rs`는 `common_evidence`를 만들고 명시적 acquire event일 때만 실제read결과를 추가한다(line23). source/revoke시 현재판본/권한으로 거른다. `handoff::build`에 제공하며 prefix도 본다(line28/40). 하지만 builder가실제로읽은passage는반환capsule과보낸handoff기록에는남는데 `common_evidence`로 합쳐지지 않는다(line40–43). 세목적의후속build에서이전VIA_COMPOSER읽기가새로활용가능한근거로제공되지않는다. 현재4fixture에는acquire event도없다.

따라서A가처음에원문목록만받아목적별검토하는것은기본A의정당한cold조건이지만, **같은episode에서이미읽은원문을다음목적에다시이용하지못하는부분은기본설계가의도적으로버린선택이라고정당화하기어렵다. 현재최소구현/fixture의공통획득근거연결누락 또는범위제약**으로기록해야한다. 이관측만으로강한A의재사용비용을비교완료했다고하지않는다.

## 최소 별도 진단 (승인 전, 실행하지 않음)

- 기존E3의실제3source와3위임질문·조건·oracle를그대로보존한다.
- 마지막source/adopt활동뒤, 첫delegate앞에 기존지원 `acquire` event하나를추가한파생fixture를새로동결한다.
- reads는그시점현재source catalog의모든공개허용범위를동일하게사용한다. `public-comparison`1–4, `public-quote`1–3, `delivered-guidance`1–3이다. 미래목적/기대답/선호제품을보아고르지않으며원문을실제로ctx.source로읽는다.
- ABC모두동일하게획득한다. 준비된B record는기존동일생산조건/비용을그대로두고, C는이공통획득상태를알수있지만원래receiver선택책임과manifest경로를유지한다. 구조별실제사용여부를보고한다.
- E3×ABC각1회=3episode/9인계. 태그예: COMMON_ACQUIRED_EVIDENCE. BASIC3회평균과혼합하거나대체하지않는다. 이는branch기능/준비조건민감도진단이며정밀성능우열실험이아니다.
- common acquisition의실제파일read/time/RSS/retainedbytes를준비비용으로따로보고하고전체episodewall에포함한다. 앞선획득이있다는조건을명시하며모델에무료기대사실/요약을주지않는다. LLM비용이0인rawread라도I/O/RAM/전송token은0이라고하지않는다.
- A가실제로DIRECT를선택하면선택한source spans와새compose/원본read생략을기록한다. 이미충분한근거가있어도실제모델이READ를선택하면그대로0회재사용으로남기며DIRECT를강제하거나튜닝하지않는다. B directrecord/extraction/fallback, C receiver reads와전체비용도같이보고한다.
- 실제최종9문안은동일사전rubric으로별도검토하며BASIC54본문점수를바꾸지않는다.

## 이 진단이 답하지 못하는 것

현재지원acquire를이용하면candidate/binary변경없이충분한기확보근거DIRECT경로의실제이용가능성을시험할수있다. 그러나기본흐름내이전read결과를다음build에연결하지않는구현제약자체를수정한것은아니다. 그수정을검증하려면후속별도승인하에공통원본획득원장에실제read결과를일관되게기록하고현재판본/권한무효화를지키는경로를ABC동일기능으로수정한별도freeze가필요하다. 현진단을그수정의완료증거로사용하지않는다.
