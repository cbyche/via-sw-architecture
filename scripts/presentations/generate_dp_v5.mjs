import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { FileBlob, PresentationFile } from '@oai/artifact-tool';

// Run a private copy with node_modules linked to the bundled dependency runtime.
const [repoArg, buildArg, revision = '01'] = process.argv.slice(2);
const repo = await fs.realpath(repoArg), build = await fs.realpath(buildArg);
const skill = process.env.VIA_PRESENTATION_SKILL_DIR;
const python = process.env.VIA_RUNTIME_PYTHON;
if (![skill, python].every(value => path.isAbsolute(value ?? ''))) throw new Error('Absolute skill and Python paths required.');
const source = path.join(repo, 'docs/presentations_files/VIA-DP-background-and-comparison-41-45_v4.pptx');
const comparison = path.join(repo, 'docs/presentations_files/dp-comparison/VIA-DP-comparison-41-45_v4.pptx');
const candidate = path.join(build, `integrated-v5-candidate-${revision}.pptx`);
const filename = 'VIA-DP-background-and-comparison-41-45_v5.pptx';
const final = path.join(build, 'final', filename);
const referenceSha256 = createHash('sha256').update(await fs.readFile(source)).digest('hex');
await fs.mkdir(path.dirname(final), { recursive: true });

async function renderDeck(file, label) {
  const deck = await PresentationFile.importPptx(await FileBlob.load(file));
  const directory = path.join(build, 'renders', label);
  await fs.mkdir(directory, { recursive: true });
  const snapshot = await deck.inspect({ kind: 'slide,layout', maxChars: 20000 });
  await fs.writeFile(path.join(directory, 'inspection.ndjson'), snapshot.ndjson);
  for (const [index, slide] of deck.slides.items.entries()) {
    const blob = await deck.export({ slide, format: 'png', scale: 1 });
    await fs.writeFile(path.join(directory, `${String(index + 1).padStart(2, '0')}.png`), new Uint8Array(await blob.arrayBuffer()));
    console.log(`Rendered ${label}: ${index + 1}/${deck.slides.items.length}`);
  }
  return deck;
}

await renderDeck(source, 'source-v4');
execFileSync(python, [path.join(repo, 'scripts/presentations/dp_v5_package.py'), source, comparison, candidate,
  path.join(build, 'v5-preservation.json')], { stdio: 'inherit' });
const { finalizePresentation } = await import(pathToFileURL(path.join(skill, 'container_tools/artifact_tool_utils.mjs')).href);
await finalizePresentation({
  workspaceDir: build, candidatePath: candidate, finalPath: final, explicitTotalSlideCount: 14,
  pythonExecutable: python,
  integrityValidatorPath: path.join(skill, 'container_tools/inspect_presentation_package_integrity.py'),
  layoutValidatorPath: path.join(skill, 'container_tools/inspect_presentation_layout_geometry.py'),
  layoutArgs: ['--expected-slide-size-emu', '18288000,10287000', '--validate-heading-fit'],
  requiredNativeTableOwnerSlides: [],
  fontPolicy: { basis: 'reference', families: ['Apple SD Gothic Neo'], referencePath: source, referenceSha256 },
  verifyArtifactToolImport: true, receiptPath: path.join(build, 'integrated-v5.validation.json'),
});
await renderDeck(final, 'final-v5');
await fs.copyFile(final, path.join(repo, 'docs/presentations_files', filename));
console.log('PASS: single integrated v5 exported, 14 slides. Comparison-only v5 is not generated.');
