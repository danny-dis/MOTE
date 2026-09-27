import test from "node:test";
import assert from "node:assert/strict";
import { Client, MoteError } from "../src/index.js";
import { join } from "node:path";

const fixture = join(process.cwd(), "dist/test/fixtures/bridge.js");
const request = (tools = { x: { description: "x", handler: async (i: Record<string, unknown>) => String(i.value).toUpperCase() } }) => ({ manifest: { name: "t", workspace: ".", capabilities: ["x"], models: [] }, task: "t", tools });
async function run(mode: string, tools = request().tools, timeoutMs = 3000) { return new Client({ binary: process.execPath, args: [fixture], timeoutMs, env: { TEST_BRIDGE_MODE: mode } }).run({ ...request(tools), manifest: { ...request(tools).manifest, models: [{ provider: "fixture", model: "fixture", endpoint: "http://unused.invalid" }] } }); }

test("runs callback and returns completed result", async () => { const result = await run("success"); assert.equal(result.success, true); assert.deepEqual(result.events, [{ type: "hello" }]); });
test("rejects malformed protocol frames", async () => { await assert.rejects(run("malformed"), (e: unknown) => e instanceof MoteError); });
test("rejects wrong ids and oversized frames", async () => { await assert.rejects(run("wrongid")); await assert.rejects(run("oversized")); });
test("rejects invalid utf8, duplicate result and exit mismatch", async () => { await assert.rejects(run("invalidutf8")); await assert.rejects(run("duplicate")); await assert.rejects(run("mismatch")); });
test("redacts callback failure", async () => { const secret = "top-secret-input"; const result = await run("success", { x: { description: "x", handler: () => { throw new Error(secret); } } }); assert.equal(result.success, true); });
test("timeout rejects", async () => { await assert.rejects(run("timeout", request().tools, 50)); });
