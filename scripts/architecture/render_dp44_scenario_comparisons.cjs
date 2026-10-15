// Render and inspect the four focused DP44 SVG scenes.
const fs=require('fs');
const path=require('path');
const crypto=require('crypto');
const {chromium}=require(process.env.VIA_PLAYWRIGHT_MODULE||'playwright');
const folder=path.resolve(__dirname,'../../docs/architecture/12-decisions/decision-packages/diagrams');
const slugs=['input','agent','onset','correction'].map(x=>'dp44-scenario-'+x);
const digest=data=>crypto.createHash('sha256').update(data).digest('hex');

(async()=>{
  const browser=await chromium.launch({headless:true,executablePath:process.env.VIA_CHROMIUM_PATH||
    (process.platform==='darwin'?'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome':undefined)});
  try {
    const page=await browser.newPage({viewport:{width:1920,height:1080},deviceScaleFactor:1});
    for(const stem of slugs){
      const svg=fs.readFileSync(path.join(folder,stem+'.svg'));
      await page.setContent(`<html lang="ko"><style>body{margin:0}</style>${svg}</html>`);
      await page.evaluate(()=>document.fonts.ready);
      const errors=await page.evaluate(()=>{
        const rows=[...document.querySelectorAll('svg text')].map(el=>({text:el.textContent,box:el.getBBox(),allowed:Number(el.dataset.width)}));
        const errors=[];
        for(const r of rows){const b=r.box;
          if(b.x<0||b.y<0||b.x+b.width>1920||b.y+b.height>1080)errors.push(['canvas',r.text]);
          if(b.width>r.allowed+2)errors.push(['width',r.text,b.width,r.allowed]);
        }
        for(let i=0;i<rows.length;i++)for(let j=i+1;j<rows.length;j++){
          const a=rows[i].box,b=rows[j].box;
          if(a.x+1<b.x+b.width&&b.x+1<a.x+a.width&&a.y+1<b.y+b.height&&b.y+1<a.y+a.height)
            errors.push(['overlap',rows[i].text,rows[j].text]);
        }
        return errors;
      });
      if(errors.length)throw Error(stem+': '+JSON.stringify(errors,null,2));
      const png=path.join(folder,stem+'.png');
      await page.screenshot({path:png});
      fs.writeFileSync(path.join(folder,stem+'-render.json'),JSON.stringify({stem,width:1920,height:1080,
        text_geometry_issues:0,svg_sha256:digest(svg),png_sha256:digest(fs.readFileSync(png))},null,2)+'\n');
      process.stdout.write('PASS: '+stem+'\n');
    }
  } finally {await browser.close();}
})().catch(e=>{process.stderr.write(e.message+'\n');process.exitCode=1;});
