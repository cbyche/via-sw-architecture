// One responsibility/geometry model for native PowerPoint, SVG and draw.io.
export const C={black:'#111111',gray:'#98999B',grid:'#B7B9B9',green:'#C9E1B3',cream:'#FFF6DD',blue:'#4676CB',cyan:'#35A8CD',pale:'#F1F6FC',red:'#C00000',muted:'#50555B',white:'#FFFFFF'};
export class Scene {
  constructor(name,page,title,subtitle){this.name=name;this.page=page;this.items=[];this.nodes={};this.notes='';this.title=title;this.subtitle=subtitle;}
  node(kind,id,text,x,y,w,h,size,parent,extra={}){if(this.nodes[id])throw Error('Duplicate '+id);const n={kind,id,text,x,y,w,h,size,parent,...extra};this.nodes[id]=n;this.items.push(n);return n;}
  comp(id,t,x,y,w,h,size=25,parent=null){return this.node('comp',id,t,x,y,w,h,size,parent);}
  mod(id,t,x,y,w,h,size=24,parent=null){return this.node('mod',id,t,x,y,w,h,size,parent);}
  store(id,t,x,y,w,h,size=22,parent=null){return this.node('store',id,t,x,y,w,h,size,parent);}
  proc(id,t,x,y,w,h,parent=null){return this.node('proc',id,t,x,y,w,h,24,parent);}
  ext(id,t,x,y,w,h,size=24){return this.node('ext',id,t,x,y,w,h,size,null);}
  doc(id,t,x,y,w,h,size=22,parent=null){return this.node('doc',id,t,x,y,w,h,size,parent);}
  text(t,x,y,w,h,size=22,opt={}){this.items.push({kind:'text',text:t,x,y,w,h,size,...opt});}
  edge(id,from,to,points,opt={}){this.items.push({kind:'edge',id,from,to,points,...opt});}
  rule(points,opt={}){this.items.push({kind:'rule',points,...opt});}
  table(rows,x,y,widths,heights,size=24){this.items.push({kind:'table',rows,x,y,widths,heights,size});}
  band(text,x,y,w,h=48){this.items.push({kind:'band',text,x,y,w,h,size:27});}
  note(text){this.notes+=text+'\n';}
}
export function validateScene(d){
 const failures=[];
 const onBoundary=(n,[x,y])=>x>=n.x-.1&&x<=n.x+n.w+.1&&y>=n.y-.1&&y<=n.y+n.h+.1&&(Math.abs(x-n.x)<.1||Math.abs(x-n.x-n.w)<.1||Math.abs(y-n.y)<.1||Math.abs(y-n.y-n.h)<.1);
 for(const n of Object.values(d.nodes)){
  if(n.parent){const p=d.nodes[n.parent];if(!p||n.x<p.x||n.y<p.y||n.x+n.w>p.x+p.w+.1||n.y+n.h>p.y+p.h+.1)failures.push('Ownership bounds: '+n.id);}
  if(n.kind==='mod'&&(!n.parent||d.nodes[n.parent]?.kind!=='comp'))failures.push('Module without Component: '+n.id);
 }
 for(const e of d.items.filter(x=>x.kind==='edge')){
  const a=d.nodes[e.from],b=d.nodes[e.to];
  if(!a||!b){failures.push('Missing endpoint '+e.id);continue;}
  if(['comp','proc'].includes(a.kind)||['comp','proc'].includes(b.kind))failures.push('Boundary instead of endpoint '+e.id);
  if(!onBoundary(a,e.points[0])||!onBoundary(b,e.points.at(-1)))failures.push('Detached endpoint '+e.id);
  for(let i=1;i<e.points.length;i++)if(e.points[i][0]!==e.points[i-1][0]&&e.points[i][1]!==e.points[i-1][1])failures.push('Nonorthogonal '+e.id);
  for(const n of Object.values(d.nodes)){
   if([e.from,e.to].includes(n.id)||['comp','proc'].includes(n.kind))continue;
   for(let i=1;i<e.points.length;i++){const a=e.points[i-1],b=e.points[i],{x,y,w,h}=n;
    if(a[0]===b[0]&&x+1<a[0]&&a[0]<x+w-1&&Math.max(Math.min(a[1],b[1]),y+1)<Math.min(Math.max(a[1],b[1]),y+h-1))failures.push('Path through '+n.id+': '+e.id);
    if(a[1]===b[1]&&y+1<a[1]&&a[1]<y+h-1&&Math.max(Math.min(a[0],b[0]),x+1)<Math.min(Math.max(a[0],b[0]),x+w-1))failures.push('Path through '+n.id+': '+e.id);
   }
  }

 }
 for(const o of d.items)if(/[·•]/.test(o.text||''))failures.push('Forbidden punctuation: '+o.text);
 if(failures.length)throw Error(d.name+'\n'+failures.join('\n'));
 return {page:d.page,nodes:Object.keys(d.nodes).length,edges:d.items.filter(x=>x.kind==='edge').length,endpoint_and_ownership_checks:'pass'};
}
