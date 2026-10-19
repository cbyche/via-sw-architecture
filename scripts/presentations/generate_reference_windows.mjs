import fs from 'node:fs/promises';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {adjustWindowsLayout} from './reference_windows_layout.mjs';

const [sourceName,outputName,build]=process.argv.slice(2);
const allowed=new Set([
 'generate_dp1_reference_style_v3.mjs',
 'generate_dp2_reference_style_v2.mjs',
 'generate_dp3_reference_style.mjs',
 'generate_dp4_reference_style.mjs',
]);
if(!allowed.has(sourceName) || !outputName?.endsWith('.pptx') || !path.isAbsolute(build)){
 throw new Error('Pass a supported source, versioned PPTX filename and absolute build directory');
}
const sourcePath=path.join(path.dirname(new URL(import.meta.url).pathname),sourceName);
const source=await fs.readFile(sourcePath,'utf8');
const originalName=source.match(/path\.join\(BUILD,'output','([^']+\.pptx)'\)/)?.[1];
if(!originalName)throw new Error('Cannot find source output filename');
const dp=Number(sourceName.match(/generate_dp([1-4])_/)?.[1]);
const revised=adjustWindowsLayout(source,dp)
 .replace("const FONT='Apple SD Gothic Neo';","const FONT='Calibri';")
 .replaceAll(`'output','${originalName}'`,`'output','${outputName}'`)
 .replace("fontPolicy:{basis:'design',families:[FONT]}","fontPolicy:{basis:'user_request',families:[FONT]}");
if(!revised.includes("const FONT='Calibri';") || !revised.includes(`'output','${outputName}'`)){
 throw new Error('Source pattern changed');
}
await fs.mkdir(build,{recursive:true});
const generated=path.join(build,'windows-authoring.mjs');
await fs.writeFile(generated,revised);
process.argv[2]=build;
await import(pathToFileURL(generated).href);
