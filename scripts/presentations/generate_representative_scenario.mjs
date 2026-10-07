import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { Presentation, PresentationFile, FileBlob } from '@oai/artifact-tool';

// Run a copy in a fresh private build directory with bundled node_modules.
const [repoArg, buildArg] = process.argv.slice(2);
if (!repoArg || !buildArg) throw new Error('Usage: generate_representative_scenario.mjs REPO BUILD_DIR');
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR, python = process.env.VIA_RUNTIME_PYTHON;
if (!path.isAbsolute(skill ?? '') || !path.isAbsolute(python ?? '')) throw new Error('Set absolute skill and Python paths.');
const dir = path.join(repo, 'docs/presentations_files');
const source = JSON.parse(await fs.readFile(path.join(dir, 'representative-scenario/scenario.json'), 'utf8'));
if (source.scenes.length !== 5 || source.taskFlow.length !== 3 || source.taskFlow.some(r => r.length !== 5)) throw new Error('Expected five scenes and a 3 × 5 task table.');
const font = 'Apple SD Gothic Neo', ink = '#20262C', navy = '#122445', teal = '#258FAF';
const reference = path.join(dir, 'VIA_과제배경_필요성_검토반영_v3.pptx');
const referenceHash = crypto.createHash('sha256').update(await fs.readFile(reference)).digest('hex');
const p = Presentation.create({ slideSize: { width: 1280, height: 720 } });
const slide = p.slides.add();
slide.background.fill = '#FFFFFF';
function text(name, value, x, y, w, h, size, color = ink, bold = false, alignment = 'left') {
  const s = slide.shapes.add({ name, geometry: 'textbox', position: { left: x, top: y, width: w, height: h }, fill: 'none', line: { fill: 'none', width: 0 } });
  s.text = value;
  s.text.style = { typeface: font, fontSize: size, color, bold, alignment, wrap: 'none', autoFit: 'none', insets: { left: 0, right: 0, top: 0, bottom: 0 } };
}
text('title', source.title, 40, 15, 704, 47, 32, ink, true);
// Same section navigation, geometry, dimensions, font and colors as the intro deck.
for (const [i, label] of ['과제 소개', '요구사항 분석', '설계', '구현 및 검증', '결론'].entries()) {
  slide.shapes.add({ name: `section-${i}`, geometry: 'chevron', position: { left: 765 + i * 94, top: 20, width: 104, height: 29 }, fill: i === 0 ? teal : '#60666B', line: { fill: '#FFFFFF', width: 1 } });
  text(`section-label-${i}`, label, 773 + i * 94, 23, 83, 23, 12, '#FFFFFF', true, 'center');
}
slide.shapes.add({ name: 'header-rule', geometry: 'line', position: { left: 30, top: 68, width: 1220, height: 0 }, fill: 'none', line: { fill: '#6C747C', width: 1 } });
text('story-subtitle', source.subtitle, 42, 95, 1196, 42, 28, navy, true);
for (const [i, scene] of source.scenes.entries()) {
  const x = 40 + i * 244;
  text(`scene-${i + 1}-heading`, scene.heading, x, 155, 224, 32, 23, teal, true);
  slide.images.add({ blob: await fs.readFile(path.join(dir, 'requirements-assets', scene.image)), contentType: 'image/png', alt: scene.heading, fit: 'contain', position: { left: x, top: 188, width: 224, height: 168 } });
  text(`scene-${i + 1}-quote`, scene.quote, x, 373, 224, 110, 21, navy, true);
  text(`scene-${i + 1}-behavior`, scene.behavior, x, 506, 224, 58, 18.5, '#657079');
}
const table = slide.tables.add({ rows: 3, columns: 5, left: 40, top: 584, width: 1200, height: 90, columnWidths: [244, 239, 239, 239, 239], values: source.taskFlow });
table.styleOptions = { headerRow: false, bandedRows: false };
table.borders.assign({ style: 'solid', fill: '#CAD2D9', width: 0.7 });
for (let row = 0; row < 3; row++) {
  table.rows[row].height = 30;
  for (let col = 0; col < 5; col++) table.cells.block({ row, column: col, rowCount: 1, columnCount: 1 }).assign({
    fill: row === 0 ? navy : '#FFFFFF', margins: { left: 9, right: 9, top: 1, bottom: 1 }, anchor: 'center',
    textStyle: { typeface: font, fontSize: 17, bold: row === 0 || col === 0, color: row === 0 ? '#FFFFFF' : ink, alignment: 'left' }
  });
}
text('role-boundary', source.roleBoundary, 47, 690, 1160, 22, 15, '#657079');
text('page-number', '3', 1234, 690, 16, 22, 13, '#657079');
slide.speakerNotes.textFrame.setText(source.notes.join('\n\n'));
const { finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
const candidate = path.join(build, 'candidate.pptx'), final = path.join(build, 'final', 'VIA-representative-scenario.pptx');
await fs.mkdir(path.dirname(final), { recursive: true });
await (await PresentationFile.exportPptx(p)).save(candidate);
await finalizePresentation({ workspaceDir: build, candidatePath: candidate, finalPath: final, explicitTotalSlideCount: 1,
  pythonExecutable: python, integrityValidatorPath: path.join(skill, 'container_tools/inspect_presentation_package_integrity.py'), layoutValidatorPath: path.join(skill, 'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs: ['--expected-slide-size-emu', '12192000,6858000', '--validate-heading-fit', '--validate-bullet-geometry', '--require-native-table-slide', '1'],
  requiredNativeTableOwnerSlides: [1], fontPolicy: { basis: 'reference', families: [font], referencePath: reference, referenceSha256: referenceHash }, verifyArtifactToolImport: true, receiptPath: path.join(build, 'validation.json') });
const loaded = await PresentationFile.importPptx(await FileBlob.load(final));
const png = await loaded.export({ slide: loaded.slides.items[0], format: 'png', scale: 1.5 });
await fs.writeFile(path.join(build, 'preview.png'), new Uint8Array(await png.arrayBuffer()));
const layout = await loaded.slides.items[0].export({ format: 'layout' });
await fs.writeFile(path.join(build, 'layout.json'), await layout.text());
await fs.copyFile(final, path.join(dir, path.basename(final)));
await fs.copyFile(path.join(build, 'preview.png'), path.join(dir, 'representative-scenario/representative-scenario.png'));
console.log(JSON.stringify({ slides: 1, scenes: 5, nativeTables: 1, final }, null, 2));
