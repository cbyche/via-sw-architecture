"""Keep v10's main slides and assets, replace QA prose, replace all appendix slides.

The new tables/text are authored with Artifact Tool. This package step reuses
the existing blank layout and notes master so unaffected drawings stay intact.
"""
from pathlib import Path
import copy
import json
import re
import sys
import uuid
import xml.etree.ElementTree as E
import full_presentation_package as F
import presentation_windows_policy as W
from package_problem_intros_v8 import named, patch_node, revise_notes, tree, body_signature


def set_text(node, value):
    tx = node.find('p:txBody', F.NS)
    if tx is None:
        tx = node.find('a:txBody', F.NS)
    assert tx is not None
    old = tx.find('a:p', F.NS)
    pp = old.find('a:pPr', F.NS)
    rp = old.find('a:r/a:rPr', F.NS)
    ep = old.find('a:endParaRPr', F.NS)
    for child in list(tx):
        if child.tag == '{'+F.A+'}p':
            tx.remove(child)
    for line in value.split('\n'):
        para = E.SubElement(tx, '{'+F.A+'}p')
        if pp is not None:
            para.append(copy.deepcopy(pp))
        run = E.SubElement(para, '{'+F.A+'}r')
        if rp is not None:
            run.append(copy.deepcopy(rp))
        E.SubElement(run, '{'+F.A+'}t').text = line
        if ep is not None:
            para.append(copy.deepcopy(ep))


def run(repo, build):
    repo, b = Path(repo), Path(build)
    source_path = repo/'docs/presentations_files/VIA-architecture-presentation_v10.pptx'
    source, authored = F.read(source_path), F.read(b/'qa-authored.pptx')
    previous = json.loads((repo/'docs/presentations_files/full-presentation/manifest-v10.json').read_text())
    assert F.digest(source_path.read_bytes()) == previous['output_sha256']
    order, new_order = F.ordered(source), F.ordered(authored)
    assert len(order) == 63 and len(new_order) == 12
    data = json.loads((repo/'docs/presentations_files/qa-metrics-v11/qa-pages.json').read_text())
    parts = dict(source)
    rows = copy.deepcopy(previous['slides'][:24])
    for row in rows:
        row.update(kind='retained-main', source_page=row['page'])

    def shape(page, name, value):
        part = order[page-1][1]
        node = named(parts[part], name)
        set_text(node, value)
        parts[part] = patch_node(parts[part], name, node)

    def native_table(page, values):
        part = order[page-1][1]
        frames = [n for n in tree(parts[part]) if n.find('.//a:tbl', F.NS) is not None]
        assert len(frames) == 1
        frame = frames[0]
        for row, strings in zip(frame.findall('.//a:tr', F.NS), values, strict=True):
            for cell, value in zip(row.findall('a:tc', F.NS), strings, strict=True):
                set_text(cell, value)
        parts[part] = patch_node(parts[part], frame.find('.//p:cNvPr', F.NS).get('name'), frame)

    def read_table(page):
        root = E.fromstring(parts[order[page-1][1]])
        return [[''.join(t.text or '' for t in c.findall('.//a:t', F.NS)) for c in r.findall('a:tc', F.NS)] for r in root.findall('.//a:tr', F.NS)]

    def replace_notes(page, value):
        note = F.related(parts, order[page-1][1], 'notesSlide')
        root = E.fromstring(parts[note])
        node = next(n for n in root.findall('.//p:sp', F.NS) if n.find('p:nvSpPr/p:nvPr/p:ph', F.NS) is not None and n.find('p:nvSpPr/p:nvPr/p:ph', F.NS).get('type') == 'body')
        set_text(node, value)
        parts[note] = W.normalize_xml(note, F.xml(root))

    shape(13,'overview-quality-contract','COMMON-28-v1 / QA01~06')
    overview=[
        '01 첫 응답 지연시간\n평균 1초 이하',
        '02 지칭 대상 식별 정확도\n집합 일치율 93% 이상',
        '03 VIA 처리시간\n평균 5초 이하',
        '04 변경 범위\n평균 Rust 모듈 3개 이하',
        '05 모델 사용 비용\n$2.55 / 공통 28개 묶음',
        '06 최대 RAM 사용량\n512 MB 이하']
    for i, value in enumerate(overview,1):
        shape(13,f'overview-goal-ASR-QA-{i:02d}',value)
    # Remove the previous QA-number-to-DP connectors after reprioritization.
    for node in list(tree(parts[order[12][1]])):
        prop=node.find('.//p:cNvPr',F.NS)
        if prop is not None and re.match(r'overview-goal-[1-6]-',prop.get('name','')):
            parts[order[12][1]]=patch_node(parts[order[12][1]],prop.get('name'))

    for page, note_name in [(16,'v2-314-note'),(19,'v2-381-note'),(21,'v2-447-note')]:
        old = read_table(page)
        values = [old[0]] + [old[r] for r in [2,1,3,5,4,6]]
        values[0][0]='품질 지표 / 공통 28개'
        for i,q in enumerate(data[:6],1):
            values[i][0]=q['id']+'  '+q['title']
        if page in [16,21]:
            values[2][1]='공통 검증 / 같은 지칭 결과 전달'
            values[2][2]='공통 검증 / 같은 지칭 결과 전달'
        else:
            values[2][1]='평가 전 / 원문에서 대상 집합 구성'
            values[2][2]='평가 전 / 게시 관계와 원문 확인'
        for c in [1,2]:
            values[6][c]=re.sub(r'예산 [\d,]+MiB','측정 전',values[6][c])
            values[4][c]=values[4][c].replace('계약','규칙과 형식')
        if page == 21:
            values[1][1]='동점 예상 / 업무 접수 전 수행 안내'
            values[1][2]='동점 예상 / 업무 접수 전 수행 안내'
        native_table(page, values)
        shape(page,note_name,'미측정. 공통 28개를 사용하며 지표와 목표 및 0~6점 기준은 Appendix QA01~QA09에서 확인한다.')

    vals=read_table(12)
    vals[1][1]='지칭 대상 식별 정확도 / VIA 처리시간 / 모델 비용'
    vals[2][1]='첫 응답 지연시간 / VIA 처리시간 / 변경 범위'
    vals[3][1]='지칭 대상 식별 정확도 / VIA 처리시간 / 모델 비용'
    vals[4][1]='첫 응답 지연시간 / 변경 범위 / 최대 RAM 사용량'
    native_table(12,vals)
    shape(12,'v2-100-drivers-qa','핵심 QA01~QA06과 추가 후보 QA07~QA09를 정리했습니다. 표는 구조별 주요 영향 경로를 보여줍니다.\n대안별 영향과 실제 차이는 같은 입력과 외부 조건에서 확인합니다.')
    shape(23,'v2-452-scope-qa','첫 응답 지연시간 / 지칭 대상 식별 정확도 / VIA 처리시간 / 변경 범위 / 모델 사용 비용 / 최대 RAM 사용량')
    shape(24,'v2-459-next-contract','공통 28개 시나리오와 같은 외부 조건에서 핵심 QA 6개와 추가 후보 3개를 평가합니다.')
    shape(24,'v2-460-next-status','현재 숫자는 품질 목표와 설명용 계산입니다. QA별 정의와 목표 근거 및 점수는 Appendix에 정리했습니다.\nA/B 선정과 PoC 실측 결과는 아직 없습니다.')
    vals=read_table(24);vals[-1][2]='실제 수정 모듈 수와 접수 대기 및 RAM 사용량';native_table(24,vals)
    common_notes='\n\nv11 QA 개정: qa-candidate-package의 QA01~QA09, COMMON-28-v1과 0~6점 기준을 적용했다. 이전 P21과 과거 QA 번호 및 지역 예산 계산은 이 버전에서 사용하지 않는다. 기존 Appendix는 모두 삭제했다. 숫자는 목표 또는 설명용 계산이며 실측 결과가 아니다.\n출처: https://github.com/cbyche/via-sw-architecture/tree/main/docs/architecture/12-decisions/decision-packages/qa-candidate-package'
    custom_notes={
        12:'기능의 차이가 아닌 구조의 차이로 품질에 영향을 주는 경로를 설명한다. 표가 모든 QA에서 실제 성능 차이가 발생함을 뜻하지 않는다.',
        13:'VIA의 공통 책임과 기존 DP44/45/42 비교 위치를 보존한다. 왼쪽 품질 목표만 최신 QA01~QA06으로 갱신했다. 기존 QA 우선순위에 연결된 선은 제거했다. Component 그림은 참조 위치이며 최종 채택안이 아니다. 모델은 현재 평가 패키지의 클라우드 조건을 따른다.',
        16:'B의 의미 제안은 Input Resolution Stage/Request Controller에서 채택한다. 채택 의미 또는 유효 Notice로 응답 후보를 만들고 게시 조건과 Join한다. A의 중앙 완료 회수와 B의 단계별 진행을 비교하며 두 방안 모두 비동기 병렬 실행이 가능하다. QA02는 동일 지칭 결과의 공통 검증이다.\n출처: docs/architecture/12-decisions/decision-packages/04-44-continuous-interaction.md',
        19:'A는 현재 Query에 맞춰 원본 근거를 조합하고 B는 공통 관계를 생산하여 별도 Repository와 Coverage를 관리한다. 원본 owner와 권한 및 개정 확인은 양안 공통이다. QA02는 대상 집합 전체의 일치를 평가한다.\n출처: docs/architecture/12-decisions/decision-packages/04-45-memory-and-context.md',
        21:'A는 모듈형 Core에서 대화와 업무의 관련 로컬 상태를 함께 확정한다. B는 독립 상태 owner 사이에 명령과 접수 확인 및 재확인 비용이 발생한다. 논리 수명 분리는 양안 공통이며 B가 자동으로 의미 정확도나 변경 범위를 개선하지 않는다. 내부 접수와 외부 Agent 실행 확인은 구별한다.\n출처: docs/architecture/12-decisions/decision-packages/04-42-lifecycle-ownership.md',
        23:'기능과 공통 외부 조건은 유지하고 최신 QA 순서를 반영한다. Architecture 최종 선정이나 실제 성능 결론을 뜻하지 않는다.',
        24:'각 QA의 정의, 목표 수치와 출처, 계산 예시 및 등급을 Appendix에 제공한다. PoC 실행과 A/B 최종 선정은 별도 작업이다.'}
    for pg, value in custom_notes.items():
        replace_notes(pg,value+common_notes)

    next_part=max(int(re.search(r'slide(\d+)\.xml$',p)[1]) for _,p in order)+1
    next_id=max(int(i)for i,_ in order)+1
    for i,q in enumerate(data):
        rows.append(dict(page=25+i,part=f'ppt/slides/slide{next_part+i}.xml',slide_id=next_id+i,key=q['id'].lower()+'-metric-v11',title=q['title'],kind='new-qa-detail',source_page=None,section='Appendix / QA 지표'))
    ct=E.fromstring(parts['[Content_Types].xml'])
    # Copy native authored bodies, notes and the exact existing header design.
    configs=[(9,10,'품질 요구사항 01~03'),(10,11,'품질 요구사항 04~06'),(11,12,'응답 시점과 시간 지표')]+[(25+i,1+i,q['id']+'  '+q['title'])for i,q in enumerate(data)]
    for page,apg,title in configs:
        row=rows[page-1];part=row['part'];ap=new_order[apg-1][1]
        template=order[29 if page>=25 else 8][1]
        root=E.fromstring(authored[ap]);st=root.find('p:cSld/p:spTree',F.NS)
        for node in tree(source[template]):
            prop=node.find('.//p:cNvPr',F.NS)
            if prop is not None and prop.get('name','').startswith('v6-header-'):
                node=copy.deepcopy(node)
                if prop.get('name')=='v6-header-title':set_text(node,title)
                st.append(node)
        for i, prop in enumerate(st.findall('.//p:cNvPr',F.NS),2):prop.set('id',str(i))
        root.find('p:cSld',F.NS).set('name',row['key'])
        parts[part]=W.normalize_xml(part,F.xml(root))
        template_note=F.related(source,template,'notesSlide')
        note=F.related(source,part,'notesSlide') if page<=24 else f'ppt/notesSlides/qaV11{page}.xml'
        parts[note]=revise_notes(source[template_note],authored[F.related(authored,ap,'notesSlide')])
        rs=E.Element('{'+F.PR+'}Relationships')
        E.SubElement(rs,'{'+F.PR+'}Relationship',Id='rId1',Type=F.R+'/slideLayout',Target='/'+F.related(source,template,'slideLayout'))
        E.SubElement(rs,'{'+F.PR+'}Relationship',Id='rId2',Type=F.R+'/notesSlide',Target='/'+note)
        parts[F.rels_path(part)]=F.xml(rs)
        nrs=E.Element('{'+F.PR+'}Relationships')
        E.SubElement(nrs,'{'+F.PR+'}Relationship',Id='rId1',Type=F.R+'/slide',Target='/'+part)
        E.SubElement(nrs,'{'+F.PR+'}Relationship',Id='rId2',Type=F.R+'/notesMaster',Target='/'+F.related(source,template_note,'notesMaster'))
        parts[F.rels_path(note)]=F.xml(nrs)
        if page>24:
            for name,kind in [(part,'slide'),(note,'notesSlide')]:
                E.SubElement(ct,'{'+F.CT+'}Override',PartName='/'+name,ContentType='application/vnd.openxmlformats-officedocument.presentationml.'+kind+'+xml')
        row.update(title=title,kind='new-qa-detail'if page>24 else'replaced-qa-main')

    # Rebuild the slide registry and remove the old appendix slide/notes parts.
    root=E.fromstring(parts['ppt/presentation.xml']);rs=E.fromstring(parts['ppt/_rels/presentation.xml.rels'])
    for rel in list(rs):
        if rel.get('Type').endswith('/slide'):rs.remove(rel)
    lst=root.find('p:sldIdLst',F.NS);lst.clear()
    for row in rows:
        rid='V11Slide'+str(row['page'])
        E.SubElement(lst,'{'+F.P+'}sldId',{'id':str(row['slide_id']),'{'+F.R+'}id':rid})
        E.SubElement(rs,'{'+F.PR+'}Relationship',Id=rid,Type=F.R+'/slide',Target='/'+row['part'])
    sl=root.find('.//{'+F.P14+'}sectionLst');assert sl is not None;sl.clear()
    for name,group in [('본 발표',rows[:24]),('Appendix / QA 지표',rows[24:])]:
        section=E.SubElement(sl,'{'+F.P14+'}section',name=name,id='{'+str(uuid.uuid5(uuid.NAMESPACE_URL,'via-v11:'+name)).upper()+'}')
        ids=E.SubElement(section,'{'+F.P14+'}sldIdLst')
        for row in group:E.SubElement(ids,'{'+F.P14+'}sldId',id=str(row['slide_id']))
    parts['ppt/presentation.xml']=F.xml(root);parts['ppt/_rels/presentation.xml.rels']=F.xml(rs)
    removed=[]
    for pg,(_,part)in enumerate(order[24:],25):
        note=F.related(source,part,'notesSlide')
        removed.append(dict(page=pg,part=part,key=E.fromstring(source[part]).find('p:cSld',F.NS).get('name')))
        for n in [part,F.rels_path(part),note,F.rels_path(note)]:parts.pop(n,None)
    # Discard assets that were referenced only by deleted appendix pages.
    keep=F.closure(parts,[F.target('',r.get('Target'))for r in E.fromstring(parts['_rels/.rels'])if r.get('TargetMode')!='External'])
    keep|={F.rels_path(n)for n in keep}|{'_rels/.rels','[Content_Types].xml'}
    for n in list(parts):
        if n not in keep:del parts[n]
    for node in list(ct):
        if node.tag.endswith('Override')and node.get('PartName').lstrip('/')not in parts:ct.remove(node)
    parts['[Content_Types].xml']=F.xml(ct)
    for row in rows:
        part=row['part'];header=named(parts[part],'v6-header-page');set_text(header,f'{row["page"]} / 33')
        parts[part]=patch_node(parts[part],'v6-header-page',header)
        if row['page']in custom_notes:row['kind']='qa-labels-updated'
        row.update(native_tables=len(E.fromstring(parts[part]).findall('.//a:tbl',F.NS)),slide_sha256=F.digest(parts[part]),notes_sha256=F.digest(parts[F.related(parts,part,'notesSlide')]))
    app=E.fromstring(parts['docProps/app.xml'])
    for n in app:
        if n.tag.split('}')[-1]in('Slides','Notes'):n.text='33'
    parts['docProps/app.xml']=F.xml(app)
    changed={9,10,11,12,13,16,19,21,23,24}
    preserved=[]
    for pg,(_,part)in enumerate(order[:24],1):
        if pg not in changed:
            assert body_signature(parts[part])==body_signature(source[part]),pg
            note=F.related(parts,part,'notesSlide');assert parts[note]==source[note],pg
            preserved.append(pg)
    for n in parts:
        if n.startswith('ppt/media/')and n in source:assert parts[n]==source[n],n
    assert len(F.ordered(parts))==33
    assert len([n for n in parts if re.fullmatch('ppt/slides/[^/]+.xml',n)])==33
    report=W.policy_report(parts);assert not report['violations'],report
    out=b/'candidate-v11.pptx';F.write(out,parts)
    manifest=dict(version=11,published_date='2026-10-15',source_path=str(source_path.relative_to(repo)),source_sha256=F.digest(source_path.read_bytes()),output_path='docs/presentations_files/VIA-architecture-presentation_v11.pptx',output_sha256=F.digest(out.read_bytes()),total_slides=33,main_slides=24,appendix_first_page=25,appendix_last_page=33,removed_source_appendix=removed,revised_main_pages=sorted(changed),preserved_source_bodies_and_notes=preserved,retained_media_byte_identical=True,authoring_policy=report,slides=rows)
    (b/'manifest-v11.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps(dict(slides=33,removedAppendix=len(removed),retainedMain=len(preserved),policyViolations=report['violations'])))


if __name__=='__main__':run(*sys.argv[1:])
