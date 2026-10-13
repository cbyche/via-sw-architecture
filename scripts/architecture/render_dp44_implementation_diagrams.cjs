// Render documentation diagrams only; no model calls or architecture measurements.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const { chromium } = require('playwright');
const dir = path.resolve(__dirname, '../../docs/architecture/12-decisions/decision-packages/diagrams');
const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const all = process.argv.includes('--all');
const implementationsOnly = process.argv.includes('--implementations-only');
const diagrams = (implementationsOnly ? ['dp44-implementation-a', 'dp44-implementation-b'] :
  ['dp44-common-path', 'dp44-implementation-a', 'dp44-implementation-b'])
  .map(stem => ({ stem, dir, width: 2560, height: 1440 }));
if (all) diagrams.push(
  ...['choice44-structure', 'choice44-event'].map(stem => ({ stem, dir, width: 2560, height: 1440 })),
  { stem: 'dp44-background', dir, pngDir: path.resolve(dir, '../../../../presentations_files/dp-background'), width: 1920, height: 1080 },
  { stem: 'dp44-comparison', dir: path.resolve(dir, '../../../../presentations_files/dp-comparison'), width: 1920, height: 1080 }
);

(async () => {
  const browser = await chromium.launch({ headless: true,
    executablePath: process.env.DP44_CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' });
  try {
    const records = [];
    for (const spec of diagrams) {
      const { stem } = spec;
      const pngPath = path.join(spec.pngDir || spec.dir, stem + '.png');
      const svg = fs.readFileSync(path.join(spec.dir, stem + '.svg'));
      const dimensions = svg.toString().match(/<svg[^>]*width="(\d+)"[^>]*height="(\d+)"/);
      if (!dimensions) throw new Error('Missing SVG dimensions: ' + stem);
      const width = Number(dimensions[1]), height = Number(dimensions[2]);
      const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: 1 });
      await page.setContent(`<html lang="ko"><style>body{margin:0;background:white}</style>${svg.toString()}</html>`);
      await page.evaluate(() => document.fonts.ready);
      const issues = await page.evaluate(({ width, height }) => {
        const txt = [...document.querySelectorAll('svg text')].map(e => ({
          t: e.textContent, b: e.getBBox(), parent: e.parentElement, element: e
        }));
        const found = [];
        for (const e of txt) {
          if (e.b.x < 0 || e.b.y < 0 || e.b.x + e.b.width > width || e.b.y + e.b.height > height)
            found.push({ kind: 'canvas', text: e.t });
          if (e.element.dataset.width && e.b.width > Number(e.element.dataset.width) + 1)
            found.push({ kind: 'text-width', text: e.t, width: e.b.width, allowed: Number(e.element.dataset.width) });
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
            const masks = [...document.querySelectorAll('svg rect')].filter(m => {
              const fill = m.getAttribute('fill');
              return fill && fill !== 'none' && (line.compareDocumentPosition(m) & 4) &&
                (m.compareDocumentPosition(e.element) & 4);
            });
            if (e.parent.hasAttribute('paint-order') || masks.some(m => {
              const b = m.getBBox();
              return b.x <= e.b.x && b.y <= e.b.y && b.x + b.width >= e.b.x + e.b.width && b.y + b.height >= e.b.y + e.b.height;
            })) continue; // Intentional edge labels with a white background.
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
      }, { width, height });
      await page.screenshot({ path: pngPath });
      await page.close();
      if (issues.length) throw new Error(stem + ':\n' + JSON.stringify(issues, null, 2));
      records.push({ stem, width, height, text_geometry_issues: 0,
        svg_sha256: digest(svg), png_sha256: digest(fs.readFileSync(pngPath)) });
      process.stdout.write('PASS: ' + stem + '\n');
    }
    const mergeRecords = filename => {
      const prior = JSON.parse(fs.readFileSync(path.join(dir, filename))).records;
      const byStem = new Map(records.map(row => [row.stem, row]));
      return prior.map(row => byStem.get(row.stem) || row);
    };
    fs.writeFileSync(path.join(dir, all ? 'dp44-document-render.json' : 'dp44-implementation-render.json'), JSON.stringify({
      scope: 'Documentation rendering only; not VIA runtime execution, API validation or QA evidence',
      records: implementationsOnly ? mergeRecords('dp44-implementation-render.json') : records
    }, null, 2) + '\n');
    if (all) fs.writeFileSync(path.join(dir, 'dp44-implementation-render.json'), JSON.stringify({
      scope: 'Documentation rendering only; not VIA runtime execution, API validation or QA evidence',
      records: records.slice(0, 3)
    }, null, 2) + '\n');
    if (implementationsOnly) fs.writeFileSync(path.join(dir, 'dp44-document-render.json'), JSON.stringify({
      scope: 'Documentation rendering only; not VIA runtime execution, API validation or QA evidence',
      records: mergeRecords('dp44-document-render.json')
    }, null, 2) + '\n');
    process.stdout.write('PASS: DP44 PNG, text bounds/overlap and SVG/PNG hashes\n');
  } finally { await browser.close(); }
})().catch(e => { process.stderr.write(e.message + '\n'); process.exitCode = 1; });
