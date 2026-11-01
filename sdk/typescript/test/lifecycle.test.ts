import test from 'node:test';
import assert from 'node:assert/strict';
import {Client, MoteError, type RunRequest} from '../src/index.js';

const result = `console.log(JSON.stringify({type:'run_result',state:'completed',events:[]}));`;
const startup = `import{createInterface}from'node:readline';const rl=createInterface({input:process.stdin});let n=0;rl.on('line',line=>{if(++n!==1)return;`;
// Give ordinary fixtures startup time without changing explicit deadline tests.
function client(body:string, timeoutMs=15000):Client {
  return new Client({binary:process.execPath,args:['--input-type=module','-e',startup+body+'});'],timeoutMs});
}
function request(handler: (input:Record<string,unknown>)=>string|Promise<string>=()=> 'OK'):RunRequest {
  return {manifest:{name:'test',workspace:'.',capabilities:['x'],models:[]},task:'go',tools:{x:{description:'x',handler}}};
}
const call = `console.log(JSON.stringify({type:'tool_call',id:1,name:'x',input:{}}));`;

test('pending async callback cannot defeat deadline',{timeout:15000},async(t)=>{
  // Freeze the SDK timer until the real child reaches the pending callback.
  t.mock.timers.enable({apis:['setTimeout']});
  t.after(()=>t.mock.timers.tick(1000));
  let entered!:()=>void;
  const ready=new Promise<void>(resolve=>{entered=resolve;});
  const timedOut=assert.rejects(
    client(`setTimeout(()=>{${call}},1500);`,1000).run(request(()=>{
      entered();return new Promise<string>(()=>{});
    })),
    (error:unknown)=>error instanceof MoteError&&error.code==='TIMEOUT'
  );
  await Promise.race([ready,timedOut.then(()=>{throw new Error('callback was not reached');})]);
  t.mock.timers.tick(1000);
  await timedOut;
});

test('registration without a grant never invokes callback',async()=>{
  let called=false;
  const r=request(()=>{called=true;return 'BAD';});r.manifest.capabilities=[];
  await assert.rejects(client(call+`rl.on('line',()=>{${result}process.exit(0)});`).run(r),
    (error:unknown)=>error instanceof MoteError&&error.code==='PROTOCOL');
  assert.equal(called,false);
});

test('trailing unterminated output is rejected after valid terminal',async()=>{
  await assert.rejects(client(result+`process.stdout.write('unfinished');process.exit(0);`).run(request()),
    (error:unknown)=>error instanceof MoteError&&error.code==='PROTOCOL');
});

test('malformed frame fails promptly instead of waiting for child timeout',{timeout:20000},async()=>{
  let started=0;
  // The callback proves startup finished; only protocol rejection is timed.
  const child=client(call+`rl.on('line',()=>console.log('{bad}'));`);
  await assert.rejects(child.run(request(()=>{started=Date.now();return 'OK';})),
    (error:unknown)=>error instanceof MoteError&&error.code==='PROTOCOL');
  assert.ok(started>0,'callback must be reached before measuring rejection');
  assert.ok(Date.now()-started<3000);
});

test('outgoing startup and callback frames are bounded',async()=>{
  const r=request();r.task='x'.repeat(1048576);
  await assert.rejects(client(result+`process.exit(0);`).run(r),
    (error:unknown)=>error instanceof MoteError&&error.code==='PROTOCOL');
  await assert.rejects(client(call+`rl.on('line',()=>{${result}process.exit(0)});`).run(request(()=>'x'.repeat(1048576))),
    (error:unknown)=>error instanceof MoteError&&error.code==='PROTOCOL');
});

test('finite positive deadline required',()=>{
  for(const timeoutMs of [0,-1,Infinity,NaN,2147483648]) assert.throws(()=>new Client({timeoutMs}),MoteError);
});

test('stderr floods are drained and missing executable fails cleanly',async()=>{
  assert.equal((await client(`process.stderr.write('x'.repeat(200000));${result}rl.close();`).run(request())).success,true);
  await assert.rejects(new Client({binary:'mote-bridge-does-not-exist-5985',timeoutMs:1000}).run(request()),MoteError);
});
