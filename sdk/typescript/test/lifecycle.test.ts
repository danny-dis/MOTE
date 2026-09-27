import test from 'node:test';
import assert from 'node:assert/strict';
import {Client, MoteError, type RunRequest} from '../src/index.js';

const result = `console.log(JSON.stringify({type:'run_result',state:'completed',events:[]}));`;
const startup = `import{createInterface}from'node:readline';const rl=createInterface({input:process.stdin});let n=0;rl.on('line',line=>{if(++n!==1)return;`;
function client(body:string, timeoutMs=3000):Client {
  return new Client({binary:process.execPath,args:['--input-type=module','-e',startup+body+'});'],timeoutMs});
}
function request(handler: (input:Record<string,unknown>)=>string|Promise<string>=()=> 'OK'):RunRequest {
  return {manifest:{name:'test',workspace:'.',capabilities:['x'],models:[]},task:'go',tools:{x:{description:'x',handler}}};
}
const call = `console.log(JSON.stringify({type:'tool_call',id:1,name:'x',input:{}}));`;

test('pending async callback cannot defeat deadline',{timeout:5000},async()=>{
  let called=false;
  const started=Date.now();
  await assert.rejects(client(call,1000).run(request(()=>{called=true;return new Promise<string>(()=>{});})),MoteError);
  assert.equal(called,true);
  assert.ok(Date.now()-started<3000);
});

test('registration without a grant never invokes callback',async()=>{
  let called=false;
  const r=request(()=>{called=true;return 'BAD';});r.manifest.capabilities=[];
  await assert.rejects(client(call+`rl.on('line',()=>{${result}process.exit(0)});`).run(r),MoteError);
  assert.equal(called,false);
});

test('trailing unterminated output is rejected after valid terminal',async()=>{
  await assert.rejects(client(result+`process.stdout.write('unfinished');process.exit(0);`).run(request()),MoteError);
});

test('malformed frame fails promptly instead of waiting for child timeout',{timeout:7000},async()=>{
  const started=Date.now();
  await assert.rejects(client(`console.log('{bad}');`,4000).run(request()),MoteError);
  assert.ok(Date.now()-started<3000);
});

test('outgoing startup and callback frames are bounded',async()=>{
  const r=request();r.task='x'.repeat(1048576);
  await assert.rejects(client(result+`process.exit(0);`).run(r),MoteError);
  await assert.rejects(client(call+`rl.on('line',()=>{${result}process.exit(0)});`).run(request(()=>'x'.repeat(1048576))),MoteError);
});

test('finite positive deadline required',()=>{
  for(const timeoutMs of [0,-1,Infinity,NaN,2147483648]) assert.throws(()=>new Client({timeoutMs}),MoteError);
});

test('stderr floods are drained and missing executable fails cleanly',async()=>{
  assert.equal((await client(`process.stderr.write('x'.repeat(200000));${result}rl.close();`).run(request())).success,true);
  await assert.rejects(new Client({binary:'mote-bridge-does-not-exist-5985',timeoutMs:1000}).run(request()),MoteError);
});
