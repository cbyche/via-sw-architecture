// Render documentation only. No model calls or DP quality measurements.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const {chromium} = require('playwright');
const dir = path.resolve(__dirname, '../../docs/architecture/12-decisions/decision-packages/diagrams');
const figures = [
  {stem:'choice47-structure',width:2560,height:1440,pngDir:dir},
  {stem:'screen-voice-grounding-background',width:1920,height:1080,
   pngDir:path.resolve(__dirname,'../../docs/presentations_files/dp-background')}
];
const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
(async () => {
  const browser = await chromium.launch({headless:true,
    executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'});
  try {
    for (const {stem,width,height,pngDir} of figures) {
    const page = await browser.newPage({viewport:{width,height},deviceScaleFactor:1});
    const svg = fs.readFileSync(path.join(dir, stem+'.svg'));
    await page.setContent(`<html lang="ko"><style>body{margin:0;background:white}</style>${svg.toString()}</html>`);
    await page.evaluate(() => document.fonts.ready);
    const issues = await page.evaluate(({width,height}) => {
      const txt = [...document.querySelectorAll('svg text')].map(e => ({t:e.textContent,b:e.getBBox(),w:Number(e.dataset.width)}));
      const found=[];
      for (const e of txt) {
        if (e.b.width>e.w+1) found.push({kind:'width',text:e.t,width:e.b.width,allowed:e.w});
        if (e.b.x<0||e.b.y<0||e.b.x+e.b.width>width||e.b.y+e.b.height>height) found.push({kind:'canvas',text:e.t});
      }
      for(let a=0;a<txt.length;a++) for(let b=a+1;b<txt.length;b++) {
        const x=txt[a].b,y=txt[b].b;
        if(x.x+1<y.x+y.width&&y.x+1<x.x+x.width&&x.y+1<y.y+y.height&&y.y+1<x.y+x.height)
          found.push({kind:'text-overlap',a:txt[a].t,b:txt[b].t});
      }
      return found;
    },{width,height});
    const pngPath = path.join(pngDir,stem+'.png');
    await page.screenshot({path:pngPath});
    if(issues.length) throw new Error(JSON.stringify(issues,null,2));
    fs.writeFileSync(path.join(dir,stem+'-render.json'), JSON.stringify({
      stem,width,height,text_geometry_issues:0,
      svg_sha256:digest(svg),png_sha256:digest(fs.readFileSync(pngPath)),
      scope:'Documentation rendering only; not API capability, architecture execution or QA evidence'
    },null,2)+'\n');
    process.stdout.write(`PASS: ${stem} PNG ${width}x${height}; text width/overlap/bounds; SVG and PNG hashes\n`);
    await page.close();
    }
  } finally { await browser.close(); }
})().catch(e => {process.stderr.write(e.message+'\n');process.exitCode=1;});
