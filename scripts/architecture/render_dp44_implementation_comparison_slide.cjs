// Render and check only the added DP44 page; earlier comparison pages stay intact.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const {chromium} = require(process.env.VIA_PLAYWRIGHT_MODULE || 'playwright');

const folder = path.resolve(__dirname, '../../docs/presentations_files/dp-comparison');
const stem = 'dp44-implementation-comparison';
const digest = data => crypto.createHash('sha256').update(data).digest('hex');

(async () => {
  const executablePath = process.env.VIA_CHROMIUM_PATH ||
    (process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : undefined);
  const browser = await chromium.launch({headless:true, executablePath});
  try {
    const page = await browser.newPage({viewport:{width:1920,height:1080},deviceScaleFactor:1});
    const svg = fs.readFileSync(path.join(folder,stem+'.svg'));
    await page.setContent(`<html lang="ko"><style>body{margin:0}</style>${svg.toString()}</html>`);
    await page.evaluate(() => document.fonts.ready);
    const issues = await page.evaluate(() => {
      const rows = [...document.querySelectorAll('svg text')].map(el => ({
        text:el.textContent, box:el.getBBox(), width:Number(el.dataset.width)
      }));
      const errors=[];
      for(const row of rows) {
        const b=row.box;
        if(b.x<0 || b.y<0 || b.x+b.width>1920 || b.y+b.height>1080)
          errors.push({kind:'canvas',text:row.text});
        if(b.width>row.width+2) errors.push({kind:'width',text:row.text,actual:b.width,allowed:row.width});
      }
      for(let i=0;i<rows.length;i++) for(let j=i+1;j<rows.length;j++) {
        const a=rows[i].box,b=rows[j].box;
        if(a.x+1<b.x+b.width && b.x+1<a.x+a.width && a.y+1<b.y+b.height && b.y+1<a.y+a.height)
          errors.push({kind:'text-overlap',a:rows[i].text,b:rows[j].text});
      }
      return errors;
    });
    if(issues.length) throw new Error(JSON.stringify(issues,null,2));
    const png=path.join(folder,stem+'.png');
    await page.screenshot({path:png});
    fs.writeFileSync(path.join(folder,stem+'-render.json'),JSON.stringify({
      stem,width:1920,height:1080,text_geometry_issues:0,
      svg_sha256:digest(svg),png_sha256:digest(fs.readFileSync(png))
    },null,2)+'\n');
    process.stdout.write('PASS: new DP44 comparison page rendered, text geometry verified\n');
  } finally { await browser.close(); }
})().catch(error => {process.stderr.write(error.message+'\n');process.exitCode=1;});
