// Render proposed code diagrams only; no model calls or architecture measurements.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { chromium } = require('playwright');
const dir = path.resolve(__dirname, '../../docs/architecture/12-decisions/decision-packages/diagrams');
const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

(async () => {
  const browser = await chromium.launch({ headless: true,
    executablePath: process.env.DP44_CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' });
  try {
    const records = [];
    for (const side of ['a', 'b']) {
      const stem = 'dp44-implementation-' + side;
      const page = await browser.newPage({ viewport: { width: 2560, height: 1440 }, deviceScaleFactor: 1 });
      const svg = fs.readFileSync(path.join(dir, stem + '.svg'));
      await page.setContent(`<html lang="ko"><style>body{margin:0;background:white}</style>${svg.toString()}</html>`);
      await page.evaluate(() => document.fonts.ready);
      const issues = await page.evaluate(() => {
        const txt = [...document.querySelectorAll('svg text')].map(e => ({
          t: e.textContent, b: e.getBBox(), parent: e.parentElement
        }));
        const found = [];
        for (const e of txt) {
          if (e.b.x < 0 || e.b.y < 0 || e.b.x + e.b.width > 2560 || e.b.y + e.b.height > 1440)
            found.push({ kind: 'canvas', text: e.t });
          if (e.parent.dataset.node) {
            const x = Number(e.parent.dataset.x), y = Number(e.parent.dataset.y);
            const w = Number(e.parent.dataset.w), h = Number(e.parent.dataset.h);
            if (e.b.x < x + 6 || e.b.x + e.b.width > x + w - 6 || e.b.y < y + 8 || e.b.y + e.b.height > y + h - 7)
              found.push({ kind: 'node-bounds', text: e.t, node: e.parent.id });
          }
        }
        for (let a = 0; a < txt.length; a++) for (let b = a + 1; b < txt.length; b++) {
          const x = txt[a].b, y = txt[b].b;
          if (x.x + 1 < y.x + y.width && y.x + 1 < x.x + x.width && x.y + 1 < y.y + y.height && y.y + 1 < x.y + x.height)
            found.push({ kind: 'text-overlap', a: txt[a].t, b: txt[b].t });
        }
        for (const line of document.querySelectorAll('svg polyline')) {
          const points = [...line.points].map(p => ({ x: p.x, y: p.y }));
          for (const e of txt) {
            if (e.parent.hasAttribute('paint-order')) continue; // Intentional edge labels.
            const b = e.b;
            for (let k = 1; k < points.length; k++) {
              const a = points[k - 1], z = points[k];
              const vertical = a.x === z.x && b.x + 1 < a.x && a.x < b.x + b.width - 1 &&
                Math.max(Math.min(a.y, z.y), b.y + 1) < Math.min(Math.max(a.y, z.y), b.y + b.height - 1);
              const horizontal = a.y === z.y && b.y + 1 < a.y && a.y < b.y + b.height - 1 &&
                Math.max(Math.min(a.x, z.x), b.x + 1) < Math.min(Math.max(a.x, z.x), b.x + b.width - 1);
              if (vertical || horizontal) found.push({ kind: 'line-through-text', line: line.id, text: e.t });
            }
          }
        }
        return found;
      });
      await page.screenshot({ path: path.join(dir, stem + '.png') });
      await page.close();
      if (issues.length) throw new Error(stem + ':\n' + JSON.stringify(issues, null, 2));
      records.push({ stem, width: 2560, height: 1440, text_geometry_issues: 0,
        svg_sha256: digest(svg), png_sha256: digest(fs.readFileSync(path.join(dir, stem + '.png'))) });
    }
    fs.writeFileSync(path.join(dir, 'dp44-implementation-render.json'), JSON.stringify({
      scope: 'Documentation rendering only; not VIA runtime execution, API validation or QA evidence',
      records
    }, null, 2) + '\n');
    process.stdout.write('PASS: DP44 A/B PNG, text bounds/overlap and SVG/PNG hashes\n');
  } finally { await browser.close(); }
})().catch(e => { process.stderr.write(e.message + '\n'); process.exitCode = 1; });
