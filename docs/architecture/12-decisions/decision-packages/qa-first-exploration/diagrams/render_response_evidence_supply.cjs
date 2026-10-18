// Render documentation only. No model calls or VIA quality measurement.
const fs = require('fs');
const path = require('path');
const {chromium} = require(process.env.VIA_PLAYWRIGHT_MODULE || 'playwright');

(async () => {
  const browser = await chromium.launch({headless:true,
    executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'});
  try {
    const page = await browser.newPage({viewport:{width:1800,height:1400},deviceScaleFactor:1});
    const svg = fs.readFileSync(path.join(__dirname,'response-evidence-supply.svg'),'utf8');
    await page.setContent(`<html lang="ko"><style>body{margin:0;background:white}</style>${svg}</html>`);
    await page.evaluate(() => document.fonts.ready);
    const issues = await page.evaluate(() => {
      const items = [...document.querySelectorAll('svg text')].map(el => ({
        text:el.textContent,box:el.getBBox(),width:Number(el.dataset.width)
      }));
      const found=[];
      for(const item of items) {
        const b=item.box;
        if(b.width>item.width+1) found.push({type:'width',text:item.text});
        if(b.x<0||b.y<0||b.x+b.width>1800||b.y+b.height>1400)
          found.push({type:'canvas',text:item.text});
      }
      for(let i=0;i<items.length;i++) for(let j=i+1;j<items.length;j++) {
        const a=items[i].box,b=items[j].box;
        if(a.x+1<b.x+b.width&&b.x+1<a.x+a.width&&a.y+1<b.y+b.height&&b.y+1<a.y+a.height)
          found.push({type:'text-overlap',a:items[i].text,b:items[j].text});
      }
      return found;
    });
    await page.screenshot({path:path.join(__dirname,'response-evidence-supply.png')});
    if(issues.length) throw new Error(JSON.stringify(issues,null,2));
    process.stdout.write('Rendered PNG; no text width, overlap or canvas issues. Documentation only.\n');
  } finally {await browser.close();}
})().catch(error => {process.stderr.write(error.message+'\n');process.exitCode=1;});
