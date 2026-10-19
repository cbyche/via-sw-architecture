import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {adjustWindowsLayout} from './reference_windows_layout.mjs';

// Build the versioned Windows typography variant from the same editable
// source objects as the 16-slide reference deck.
const sourcePath=path.join(path.dirname(new URL(import.meta.url).pathname),'combine_dp1_dp4_reference_style.mjs');
const source=await fs.readFile(sourcePath,'utf8');
let revised=source
 .replace("const FONT='Apple SD Gothic Neo'", "const FONT='Calibri'")
 .replace("fontPolicy:{basis:'reference',families:[FONT],referencePath:inputRecords[0].pptxPath,referenceSha256:inputRecords[0].pptxSha256}", "fontPolicy:{basis:'user_request',families:[FONT]}")
 .replace("'VIA-DP1-DP4-reference-style-16slides.pptx'", "'VIA-DP1-DP4-reference-style-16slides-v2-windows.pptx'");
revised=revised.replace(
 "const moduleText=\`export function build(pres,common)",
 "body=adjustWindowsLayout(body,spec.dp);\n const moduleText=\`export function build(pres,common)"
).replace(
 "import {createHash} from 'node:crypto';",
 `import {createHash} from 'node:crypto';\nimport {adjustWindowsLayout} from ${JSON.stringify(pathToFileURL(path.join(path.dirname(sourcePath),'reference_windows_layout.mjs')).href)};`
);
if(revised===source || !revised.includes("const FONT='Calibri'"))throw new Error('Source pattern changed');
const build=process.argv[2];
if(!build || !path.isAbsolute(build))throw new Error('Pass absolute build directory');
await fs.mkdir(build,{recursive:true});
const generated=path.join(build,'combined-windows-authoring.mjs');
await fs.writeFile(generated,revised);
await import(pathToFileURL(generated).href);
