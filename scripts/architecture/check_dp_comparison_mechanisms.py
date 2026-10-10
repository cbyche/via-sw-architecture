"""Focused figure contract: geometry, state ownership and completion endpoints.

No VIA execution, model calls or quality measurements are performed.
"""
import argparse
from generate_dp_comparison_slides import Comparison
import dp44_comparison_scene as execution
import dp45_comparison_scene as memory
import dp42_comparison_scene as lifecycle


def connections(g):
    for item in g.s.items[g.first:]:
        if item['kind']=='line' and item.get('arrow'):
            yield [(px-g.x,py) for px,py in item['points']]


def inside(point,box):
    px,py=point;x,y,w,h=box
    return x<=px<=x+w and y<=py<=y+h


def endpoint_bounds(g,key):
    x,y,w,h=g.nodes[key]
    # Some owner ports are routed relative to their heading, while the actual
    # Component boundary includes its state below that heading.
    for item in g.s.items[g.first:]:
        if item['kind'] not in ('rect','store','note'):continue
        if item.get('x')==g.x+x and item.get('y')==y and item.get('w')==w:
            h=max(h,item['h'])
    return x,y,w,h


def required_edge(g,a,b):
    assert any(inside(p[0],endpoint_bounds(g,a)) and inside(p[-1],endpoint_bounds(g,b)) for p in connections(g)),('Missing path',a,b)


def geometry(g,n,side):
    for item in g.s.items[g.first:]:
        if item['kind']!='line':continue
        points=[(px-g.x,py) for px,py in item['points']]
        for a,b in zip(points,points[1:]):
            assert a[0]==b[0] or a[1]==b[1],('Non-orthogonal',n,side,a,b)
            for key,box in g.nodes.items():
                if inside(points[0],box) or inside(points[-1],box):continue
                nx,ny,nw,nh=box
                vertical=a[0]==b[0] and nx+1<a[0]<nx+nw-1 and max(min(a[1],b[1]),ny+1)<min(max(a[1],b[1]),ny+nh-1)
                horizontal=a[1]==b[1] and ny+1<a[1]<ny+nh-1 and max(min(a[0],b[0]),nx+1)<min(max(a[0],b[0]),nx+nw-1)
                assert not(vertical or horizontal),('Unrelated box crossing',n,side,key,a,b)


def owner(g,key,bounds):
    x,y,w,h=g.nodes[key];ox,oy,ow,oh=bounds
    assert ox<=x and oy<=y and x+w<=ox+ow and y+h<=oy+oh,('Wrong state owner',key)


def check(n,side):
    mod={42:lifecycle,44:execution,45:memory}[n]
    g=mod.draw(Comparison(n),140,side)
    geometry(g,n,side)
    if n==44:
        receiver='dispatch' if side==0 else 'input'
        required_edge(g,'ri',receiver)
        if side==0:
            for a,b in [('compose','dispatch'),('dispatch','compose'),('dispatch','publish')]:required_edge(g,a,b)
            owner(g,'wait',(25,323,810,148))
        else:
            for a,b in [('input','compose'),('notice','compose'),('compose','join'),('join','publish')]:required_edge(g,a,b)
            for key in ['iw','nw']:owner(g,key,(25,323,810,148))
            owner(g,'pw',(310,494,525,157))
    elif n==45:
        receiver='composer' if side==0 else 'reader'
        required_edge(g,'ri',receiver);required_edge(g,receiver,'ri')
        if side==0:
            for a,b in [('composer','rm'),('composer','tm'),('rm','composer'),('tm','composer')]:required_edge(g,a,b)
            for key in ['cache','set']:owner(g,key,(25,344,810,284))
        else:
            for a,b in [('reader','publisher'),('publisher','reader'),('reader','repo'),('repo','reader'),('publisher','coverage')]:required_edge(g,a,b)
            for key in ['repo','coverage']:owner(g,key,(25,344,810,284))
            assert not any(inside(p[0],g.nodes['publisher']) and inside(p[-1],g.nodes['ri']) for p in connections(g)), 'Publisher bypasses Reader'
        producer='composer' if side==0 else 'publisher'
        required_edge(g,producer,'ma');required_edge(g,'ma','cloud')
    else:
        lifecycle.validate(g,side)
        required_edge(g,'gate','gateway')
        if side==0:
            for a,b in [('controller','commit'),('task','commit'),('commit','controller'),('commit','task')]:required_edge(g,a,b)
        else:
            for a,b in [('controller','task'),('task','controller'),('controller','dialogue_store'),('task','work_store')]:required_edge(g,a,b)
    print(f'PASS: {n}{"A" if side==0 else "B"} orthogonal paths, owner states and completion endpoints')


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--only',type=int,choices=[42,44,45]);args=p.parse_args()
    for n in ([args.only] if args.only else [44,45,42]):
        for side in (0,1):check(n,side)
    print('Figure consistency only; no runtime, semantic or measured QA verification.')
