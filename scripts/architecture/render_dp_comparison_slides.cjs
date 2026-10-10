// Render the editable SVG scenes without changing the Architecture source figures.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const {chromium} = require(process.env.VIA_PLAYWRIGHT_MODULE || 'playwright');
const folder = path.resolve(__dirname, '../../docs/presentations_files/dp-comparison');
const scope=process.argv[2];
if(scope && !['--dp41-only','--dp42-only','--dp41-42-only','--dp44-45-only','--dp44-only','--dp45-only'].includes(scope)) throw new Error('Optional scope: --dp41-42-only, --dp44-only or --dp45-only');
const digest = data => crypto.createHash('sha256').update(data).digest('hex');

(async () => {
  const executablePath = process.env.VIA_CHROMIUM_PATH ||
    (process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : undefined);
  const browser = await chromium.launch({headless: true, executablePath});
  const page = await browser.newPage({viewport: {width: 1920, height: 1080}, deviceScaleFactor: 1});
  const records = [];
  try {
    for (const n of [41,42,43,44,45]) {
      const slug = `dp${n}-comparison`;
      if((scope==='--dp44-45-only' && n<44) || (scope==='--dp41-only' && n!==41) || (scope==='--dp42-only' && n!==42) || (scope==='--dp41-42-only' && n>42) || (scope==='--dp44-only' && n!==44) || (scope==='--dp45-only' && n!==45)) {
        const old=JSON.parse(fs.readFileSync(path.join(folder,'render-manifest.json'))).slides.find(r=>r.slug===slug);
        for(const ext of ['svg','png']) if(digest(fs.readFileSync(path.join(folder,`${slug}.${ext}`)))!==old[`${ext}_sha256`])
          throw new Error(`Unselected preview is stale: ${slug}.${ext}`);
        records.push(old);continue;
      }
      const svg = fs.readFileSync(path.join(folder, `${slug}.svg`));
      await page.setContent(`<html lang="ko"><style>body{margin:0}</style>${svg.toString()}</html>`);
      await page.evaluate(() => document.fonts.ready);
      const issues = await page.evaluate(() => {
        const rs = [...document.querySelectorAll('svg text')].map(e => ({
          text: e.textContent, b: e.getBBox(), w: parseFloat(e.dataset.width)
        }));
        const problems = [];
        for (const r of rs) {
          const b = r.b;
          if (b.x<0 || b.y<0 || b.x+b.width>1920 || b.y+b.height>1080)
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
      if (issues.length) throw new Error(`${slug}: ${JSON.stringify(issues)}`);
      const pngPath = path.join(folder, `${slug}.png`);
      await page.screenshot({path: pngPath});
      records.push({slug, svg_sha256: digest(svg), png_sha256: digest(fs.readFileSync(pngPath)),
        width:1920, height:1080, text_geometry_issues:0});
    }
    fs.writeFileSync(path.join(folder, 'render-manifest.json'), JSON.stringify({slides:records},null,2)+'\n');
    process.stdout.write(`PASS: ${['--dp41-only','--dp42-only','--dp44-only','--dp45-only'].includes(scope)?'one selected':scope?'two selected':'five'} 1920x1080 PNGs; text geometry issues 0; hashes recorded\n`);
  } finally {
    await browser.close();
  }
})().catch(error => {process.stderr.write(error.message+'\n');process.exitCode=1;});
