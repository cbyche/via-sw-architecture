// Documentation render only. No model call or Architecture measurement.
const fs=require('fs'), path=require('path'), crypto=require('crypto');
const {chromium}=require(process.env.VIA_PLAYWRIGHT_MODULE || 'playwright');
const [repo,build]=process.argv.slice(2);
const scenes=JSON.parse(fs.readFileSync(path.join(build,'scenes.json')));
const hash=b=>crypto.createHash('sha256').update(b).digest('hex');
(async()=>{
  const browser=await chromium.launch({headless:true,executablePath:'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'});
  const page=await browser.newPage({viewport:{width:1920,height:1080},deviceScaleFactor:1});
  const report=[];fs.mkdirSync(path.join(build,'figure-renders'),{recursive:true});
  try{
    for(const s of scenes){
      const svg=fs.readFileSync(path.join(repo,'docs/architecture/12-decisions/decision-packages/diagrams',s.slug+'.svg'));
      await page.setContent(`<html lang="ko"><style>body{margin:0}</style>${svg}</html>`);
      await page.evaluate(()=>document.fonts.ready);
      const issues=await page.evaluate(()=>{
        const ts=[...document.querySelectorAll('svg text')].map(e=>({text:e.textContent,b:e.getBBox(),limit:Number(e.dataset.width)}));const issues=[];
        for(const t of ts){if(t.b.width>t.limit+2)issues.push({kind:'width',text:t.text,width:t.b.width,limit:t.limit});if(t.b.x<0||t.b.y<0||t.b.x+t.b.width>1920||t.b.y+t.b.height>1080)issues.push({kind:'canvas',text:t.text});}
        for(let a=0;a<ts.length;a++)for(let b=a+1;b<ts.length;b++){
          const x=ts[a].b,y=ts[b].b;if(x.x+2<y.x+y.width&&y.x+2<x.x+x.width&&x.y+2<y.y+y.height&&y.y+2<x.y+x.height)issues.push({kind:'text-overlap',a:ts[a].text,b:ts[b].text});
        }return issues;
      });
      const png=path.join(build,'figure-renders',s.slug+'.png');await page.screenshot({path:png});
      report.push({slug:s.slug,svg_sha256:hash(svg),png_sha256:hash(fs.readFileSync(png)),issues});
      console.log(s.slug+': '+JSON.stringify(issues));
    }
  }finally{await browser.close();}
  fs.writeFileSync(path.join(build,'figure-render-report.json'),JSON.stringify(report,null,2));
  if(report.some(r=>r.issues.length))process.exitCode=1;
})().catch(e=>{console.error(e);process.exitCode=1;});
