import {Client} from './dist/src/index.js';
const model = {
  provider: 'openai-compatible', model: process.env.MOTE_MODEL ?? 'auto',
  endpoint: process.env.MOTE_MODEL_ENDPOINT ?? 'http://127.0.0.1:47113/v1/chat/completions',
  ...(process.env.MODEL_API_KEY ? {auth_env:'MODEL_API_KEY'} : {}),
};
const calls = [];
const result = await new Client({binary:process.env.MOTE_BRIDGE ?? 'mote-bridge'}).run({
  manifest:{name:'typescript-example',capabilities:['uppercase'],models:[model]},
  task:'Call uppercase with {"text":"hello"}, then complete with its returned text.',
  tools:{uppercase:{description:'Input: text string. Returns uppercase text.',handler:input=>{
    if(typeof input.text !== 'string') throw new Error('Expected text');
    calls.push(input); return input.text.toUpperCase();
  }}},
});
const observed = result.events.some(e=>e.type==='ObservationReceived' && e.data?.text==='HELLO');
if(!result.success || !calls.length || !observed) throw new Error('Example failed callback/observation checks');
console.log(JSON.stringify({state:result.state,observation:'HELLO',callback_used:!!calls.length}));
