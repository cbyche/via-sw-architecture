// Documentation render and geometry audit only; no VIA runtime or model execution.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { chromium } = require('playwright');
const dir = path.resolve(__dirname, '../../docs/architecture/12-decisions/decision-packages/diagrams');
const stems = ['dp42-implementation-a', 'dp42-implementation-b', 'dp42-flow-a', 'dp42-flow-b'];
const digest = b => crypto.createHash('sha256').update(b).digest('hex');
(async () => {
  const browser = await chromium.launch({ headless: true,
    executablePath: process.env.DP42_CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' });
  const records = [];
  try {
    for (const stem of stems) {
      const svg = fs.readFileSync(path.join(dir, stem + '.svg'));
      const page = await browser.newPage({ viewport: { width: 3840, height: 2160 }, deviceScaleFactor: 1 });
      await page.setContent(`<html lang="ko"><style>body{margin:0;background:white}</style>${svg}</html>`);
      await page.evaluate(() => document.fonts.ready);
      const issues = await page.evaluate(() => {
        const nodes = [...document.querySelectorAll('svg text')].map(e => ({e, b:e.getBBox(), t:e.textContent}));
        const errors = [];
        for (const {e,b,t} of nodes) {
          if (b.x < 0 || b.y < 0 || b.x+b.width > 3840 || b.y+b.height > 2160)
            errors.push({kind:'canvas', text:t});
          const p=e.parentElement;
          if (p.dataset.node) {
            const x=+p.dataset.x,y=+p.dataset.y,w=+p.dataset.w,h=+p.dataset.h;
            if (b.x < x+4 || b.x+b.width > x+w-4 || b.y < y+3 || b.y+b.height > y+h-3)
              errors.push({kind:'node-bounds', node:p.id, text:t});
          }
        }
        for(let i=0;i<nodes.length;i++) for(let j=i+1;j<nodes.length;j++) {
          const a=nodes[i].b,b=nodes[j].b;
          if(a.x+1<b.x+b.width && b.x+1<a.x+a.width && a.y+1<b.y+b.height && b.y+1<a.y+a.height)
            errors.push({kind:'text-overlap', a:nodes[i].t,b:nodes[j].t});
        }
        for (const wire of document.querySelectorAll('svg polyline')) {
          const pts=[...wire.points];
          for (const {e,b,t} of nodes) {
            if(e.hasAttribute('paint-order')) continue;
            for(let i=1;i<pts.length;i++) {
              const a=pts[i-1],z=pts[i];
              const v=a.x===z.x && b.x+1<a.x && a.x<b.x+b.width-1 && Math.max(Math.min(a.y,z.y),b.y+1)<Math.min(Math.max(a.y,z.y),b.y+b.height-1);
              const h=a.y===z.y && b.y+1<a.y && a.y<b.y+b.height-1 && Math.max(Math.min(a.x,z.x),b.x+1)<Math.min(Math.max(a.x,z.x),b.x+b.width-1);
              if(v||h)errors.push({kind:'line-through-text',wire:wire.id,text:t});
            }
          }
        }
        return errors;
      });
      const png=path.join(dir,stem+'.png');
      await page.screenshot({path:png});
      await page.close();
      if(issues.length) throw new Error(stem+'\n'+JSON.stringify(issues,null,2));
      records.push({stem,width:3840,height:2160,text_geometry_issues:0,svg_sha256:digest(svg),png_sha256:digest(fs.readFileSync(png))});
      process.stdout.write('PASS: '+stem+'\n');
    }
    fs.writeFileSync(path.join(dir,'dp42-implementation-render.json'),JSON.stringify({
      scope:'Author documentation rendering review; not product implementation, independent review or QA measurement',records
    },null,2)+'\n');
  } finally {await browser.close();}
})().catch(e=>{process.stderr.write(e.message+'\n');process.exitCode=1;});
