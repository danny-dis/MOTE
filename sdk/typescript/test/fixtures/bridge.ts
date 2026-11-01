import { createInterface } from "node:readline";
import assert from "node:assert/strict";
const mode = process.env.TEST_BRIDGE_MODE ?? "success";
const rl = createInterface({ input: process.stdin });
rl.on("line", (line: string) => {
  const r = JSON.parse(line) as { type: string };
  if (mode === "success" || mode === "callbackerror") {
    if (r.type === "run") process.stdout.write(JSON.stringify({ type: "tool_call", id: 1, name: "x", input: { value: "hello" } }) + "\n");
    else if (r.type === "tool_result") {
      if (mode === "callbackerror") assert.deepEqual(r, { type: "tool_result", id: 1, error: "tool handler failed" });
      else assert.deepEqual(r, { type: "tool_result", id: 1, output: "HELLO" });
      process.stdout.write(JSON.stringify({ type: "run_result", state: "completed", events: [{ type: "hello" }] }) + "\n", () => rl.close());
    }
    return;
  }
  if (mode === "malformed") process.stdout.write("{bad}\n");
  else if (mode === "oversized") process.stdout.write("x".repeat(1048576) + "\n");
  else if (mode === "invalidutf8") process.stdout.write(Buffer.from([0xff, 0xfe, 0x0a]));
  else if (mode === "wrongid") process.stdout.write(JSON.stringify({ type: "tool_call", id: 2, name: "x", input: {} }) + "\n");
  else if (mode === "duplicate") process.stdout.write(JSON.stringify({ type: "run_result", state: "completed", events: [] }) + "\n" + JSON.stringify({ type: "run_result", state: "completed", events: [] }) + "\n");
  else if (mode === "mismatch") {
    process.exitCode = 1;
    process.stdout.write(JSON.stringify({ type: "run_result", state: "completed", events: [] }) + "\n", () => rl.close());
  }
});
