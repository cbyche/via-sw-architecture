import fs from 'node:fs/promises';
import path from 'node:path';
import {PresentationFile, FileBlob} from '@oai/artifact-tool';

const [repo, build] = process.argv.slice(2);
const plan = JSON.parse(await fs.readFile(path.join(build, 'plan.json'), 'utf8'));
for (const source of plan.sources) {
  const p = await PresentationFile.importPptx(await FileBlob.load(path.join(repo, source.path)));
  const snapshot = await p.inspect({kind:'slide,textbox,shape', maxChars:4000000});
  const records = snapshot.ndjson.split('\n').filter(Boolean).map(JSON.parse);
  for (const edit of source.edits) {
    const matches = records.filter(r => r.slideIndex === edit.page-1 && r.name === 'v6-header-page' && r.id?.startsWith('sh/'));
    if (matches.length !== 1) throw new Error(`Page field missing: ${source.key} ${edit.page}`);
    p.resolve(matches[0].id).text.replace(edit.before, edit.after);
  }
  await (await PresentationFile.exportPptx(p)).save(path.join(build, `${source.key}-authored.pptx`));
  console.log(`Updated ${source.edits.length} page fields in ${source.key}`);
}
