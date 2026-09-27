// Verify what a consumer actually installs, not just source imports.
import {execFileSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';

const sdk = fileURLToPath(new URL('../', import.meta.url));
const npm = process.env.npm_execpath;
if (!npm) throw new Error('Run via npm run pack:check');
const temp = mkdtempSync(join(tmpdir(), 'mote-sdk-consumer-'));
try {
  const packed = JSON.parse(execFileSync(process.execPath,
    [npm, 'pack', '--json', '--pack-destination', temp], {cwd:sdk, encoding:'utf8'}))[0];
  const consumer = join(temp, 'consumer with spaces'); mkdirSync(consumer);
  writeFileSync(join(consumer, 'package.json'), JSON.stringify({private:true,type:'module'}));
  execFileSync(process.execPath, [npm,'install','--offline','--ignore-scripts','--no-audit','--no-fund',join(temp,packed.filename)], {cwd:consumer,stdio:'inherit'});
  execFileSync(process.execPath, ['--input-type=module','-e',
    "import {Client,MoteError} from '@mote-agent/sdk';if(typeof new Client().run!=='function'||!(new MoteError('test') instanceof Error))throw Error('Invalid exports');"], {cwd:consumer,stdio:'inherit'});
  writeFileSync(join(consumer,'check.ts'), "import {Client,type Manifest} from '@mote-agent/sdk';const manifest:Manifest={name:'consumer',max_iterations:3};void new Client({env:{TEST:'ok'}}).run({manifest,task:'go'});\n");
  execFileSync(process.execPath, [join(sdk,'node_modules/typescript/bin/tsc'),'--strict','--noEmit','--module','NodeNext','--moduleResolution','NodeNext','--target','ES2022','check.ts'], {cwd:consumer,stdio:'inherit'});
  console.log('Installed tarball exports and consumer TypeScript declarations verified.');
} finally {
  rmSync(temp,{recursive:true,force:true});
}
