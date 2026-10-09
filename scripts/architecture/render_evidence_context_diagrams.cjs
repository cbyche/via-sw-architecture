// Render only the independent 46 review; never rewrites shared DP slides.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const {chromium} = require(process.env.VIA_PLAYWRIGHT_MODULE || 'playwright');
const folder = path.resolve(__dirname, '../../docs/architecture/12-decisions/decision-packages/diagrams');
const digest = data => crypto.createHash('sha256').update(data).digest('hex');

(async () => {
  const browser = await chromium.launch({headless: true,
    executablePath: process.env.VIA_CHROMIUM_PATH ||
      (process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : undefined)});
  try {
    const page = await browser.newPage({viewport: {width:2560, height:1440}, deviceScaleFactor:1});
    for (const stem of ['dp46-background', 'choice46-structure', 'choice46-timeline', 'choice46-lifecycle', 'choice46-quality']) {
      const svg = fs.readFileSync(path.join(folder, `${stem}.svg`));
      await page.setContent(`<html lang="ko"><style>body{margin:0}</style>${svg.toString()}</html>`);
      await page.evaluate(() => document.fonts.ready);
      const issues = await page.evaluate(() => {
        const rs = [...document.querySelectorAll('svg text')].map(e => ({text:e.textContent, b:e.getBBox(), w:parseFloat(e.dataset.width)}));
        const problems = [];
        for (const r of rs) {
          const b=r.b;
          if (b.x<0 || b.y<0 || b.x+b.width>2560 || b.y+b.height>1440)
            problems.push({kind:'canvas', text:r.text});
          if (b.width>r.w+2) problems.push({kind:'width', text:r.text});
        }
        for (let i=0;i<rs.length;i++) for (let j=i+1;j<rs.length;j++) {
          const a=rs[i].b, b=rs[j].b;
          if (a.x+1<b.x+b.width && b.x+1<a.x+a.width && a.y+1<b.y+b.height && b.y+1<a.y+a.height)
            problems.push({kind:'text-overlap', a:rs[i].text, b:rs[j].text});
        }
        return problems;
      });
      if (issues.length) throw new Error(`${stem}: ${JSON.stringify(issues)}`);
      const png = path.join(folder, `${stem}.png`);
      await page.screenshot({path:png});
      fs.writeFileSync(path.join(folder, `${stem}-render.json`), JSON.stringify({
        slug:stem, width:2560, height:1440, text_geometry_issues:0,
        svg_sha256:digest(svg), png_sha256:digest(fs.readFileSync(png)),
        scope:'Documentation render only; no model/architecture execution or quality measurement'
      }, null, 2)+'\n');
      process.stdout.write(`PASS: ${stem} 2560x1440 PNG; text geometry issues 0; hashes recorded\n`);
    }
    const remoteRequests = [];
    page.on('request', r => { if (/^https?:/.test(r.url())) remoteRequests.push(r.url()); });
    await page.goto('file://' + path.join(folder, 'choice46-review.html'));
    for (const stem of ['dp46-background', 'choice46-structure', 'choice46-timeline', 'choice46-lifecycle', 'choice46-quality']) {
      await page.locator(`[data-stem="${stem}"]`).click();
      await page.waitForFunction(expected => {
        const f=document.getElementById('figure');
        return f.src.endsWith(expected+'.svg') && f.complete && f.naturalWidth===2560;
      }, stem);
      const links=await page.evaluate(() => [document.getElementById('svg').href,document.getElementById('drawio').href]);
      if (!links[0].endsWith(stem+'.svg') || !links[1].endsWith(stem+'.drawio')) throw new Error('Review links out of sync');
    }
    await page.locator('#zoom').click();
    if (await page.locator('#figure').evaluate(e => e.getBoundingClientRect().width) !== 2560) throw new Error('Original-size zoom failed');
    await page.locator('#zoom').click();
    if (remoteRequests.length) throw new Error('Unexpected network request in local review');
    process.stdout.write('PASS: local review HTML five tabs, image loading, source links and zoom; no remote requests\n');
  } finally { await browser.close(); }
})().catch(e => { process.stderr.write(e.message+'\n'); process.exitCode=1; });
