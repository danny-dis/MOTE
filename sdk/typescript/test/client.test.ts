import test from "node:test";
import assert from "node:assert/strict";
import { Client, MoteError } from "../src/index.js";
import { fileURLToPath } from "node:url";

const fixture = fileURLToPath(new URL("./fixtures/bridge.js", import.meta.url));
const request = (tools = { x: { description: "x", handler: async (i: Record<string, unknown>) => String(i.value).toUpperCase() } }) => ({ manifest: { name: "t", workspace: ".", capabilities: ["x"], models: [] }, task: "t", tools });
// General protocol tests allow child startup; deadline tests use explicit short limits.
async function run(mode: string, tools = request().tools, timeoutMs = 15000) { return new Client({ binary: process.execPath, args: [fixture], timeoutMs, env: { TEST_BRIDGE_MODE: mode } }).run({ ...request(tools), manifest: { ...request(tools).manifest, models: [{ provider: "fixture", model: "fixture", endpoint: "http://unused.invalid" }] } }); }

test("runs callback and returns completed result", async () => { const result = await run("success"); assert.equal(result.success, true); assert.deepEqual(result.events, [{ type: "hello" }]); });
test("rejects malformed protocol frames", async () => { await assert.rejects(run("malformed"), (e: unknown) => e instanceof MoteError && e.code === "PROTOCOL"); });
test("rejects wrong ids and oversized frames", async () => { for (const mode of ["wrongid", "oversized"]) await assert.rejects(run(mode), (e: unknown) => e instanceof MoteError && e.code === "PROTOCOL"); });
test("rejects invalid utf8, duplicate result and exit mismatch", async () => { for (const mode of ["invalidutf8", "duplicate", "mismatch"]) await assert.rejects(run(mode), (e: unknown) => e instanceof MoteError && e.code === "PROTOCOL"); });
test("redacts callback failure", async () => { const secret = "top-secret-input"; const result = await run("callbackerror", { x: { description: "x", handler: () => { throw new Error(secret); } } }); assert.equal(result.success, true); });
test("timeout rejects", async () => { await assert.rejects(run("timeout", request().tools, 50), (e: unknown) => e instanceof MoteError && e.code === "TIMEOUT"); });
