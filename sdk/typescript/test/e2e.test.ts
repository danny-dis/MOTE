import test from "node:test";
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { Client } from "../src/index.js";

const bridge = process.env.MOTE_BRIDGE;
test("real bridge requests uppercase and returns HELLO event", { skip: !bridge }, async () => {
  let calls = 0;
  const server = createServer((req, res) => {
    let body = "";
    req.on("data", chunk => { body += chunk; });
    req.on("end", () => {
      calls++;
      const response = calls === 1
        ? { choices: [{ finish_reason: "stop", message: { role: "assistant", content: JSON.stringify({action:"custom",name:"uppercase",input:{text:"hello"}}) } }] }
        : { choices: [{ finish_reason: "stop", message: { role: "assistant", content: JSON.stringify({action:"complete",summary:"HELLO"}) } }] };
      res.writeHead(200, { "content-type": "application/json" }); res.end(JSON.stringify(response));
      void body;
    });
  });
  await new Promise<void>(resolve => server.listen(0, "127.0.0.1", resolve));
  const address = server.address(); assert.ok(address && typeof address !== "string");
  try {
    const result = await new Client({ binary: bridge, timeoutMs: 15_000 }).run({
      manifest: { name: "typescript-e2e", workspace: process.cwd(), capabilities: ["uppercase"], models: [{ provider: "openai-compatible", model: "fixture", endpoint: `http://127.0.0.1:${address.port}/v1/chat/completions` }] },
      task: "Use uppercase with input hello, then finish.",
      tools: { uppercase: { description: "Returns uppercase text.", handler: ({ text }) => String(text).toUpperCase() } }
    });
    assert.equal(result.success, true);
    assert.ok(result.events.some(event => JSON.stringify(event).includes("HELLO")));
  } finally { server.close(); }
});
