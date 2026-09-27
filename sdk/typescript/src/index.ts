import { ChildProcess, spawn } from "node:child_process";
import { Buffer } from "node:buffer";

export interface ModelConfig {
  provider: string;
  model: string;
  endpoint: string;
  auth_env?: string | null;
  timeout_seconds?: number;
}

export interface Manifest {
  name: string;
  workspace?: string;
  capabilities?: string[];
  models?: ModelConfig[];
  max_iterations?: number;
  max_per_tool?: number;
  max_runtime_seconds?: number;
  command_timeout_seconds?: number;
  max_output_bytes?: number;
  output_file?: string | null;
  shell?: { allowed_programs?: string[]; unsafe_shell?: boolean };
  decision?: { provider: string; endpoint: string; model?: string;
    auth_env?: string | null; fallback?: string | null; timeout_seconds?: number } | null;
}

export interface Tool {
  description: string;
  handler: (input: Record<string, unknown>) => string | Promise<string>;
}

export interface RunRequest {
  manifest: Manifest;
  task: string;
  tools?: Record<string, Tool>;
}

export type RunState = "completed" | "failed" | "cancelled";
export interface RunResult { state: RunState; events: Record<string, unknown>[]; returncode: number; success: boolean; }

export interface ClientOptions {
  binary?: string;
  /** Primarily useful for embedding/test launchers; normal bridge binaries need no arguments. */
  args?: string[];
  timeoutMs?: number;
  cwd?: string;
  env?: Record<string, string | undefined>;
}

export class MoteError extends Error {
  readonly code: string;
  constructor(message: string, code = "MOTE_ERROR") { super(message); this.name = "MoteError"; this.code = code; }
}

const MAX_FRAME = 1_048_576;
const MAX_STDERR = 65_536;
const states = new Set<RunState>(["completed", "failed", "cancelled"]);
const isObject = (v: unknown): v is Record<string, unknown> => typeof v === "object" && v !== null && !Array.isArray(v);

class Frames {
  private buf = Buffer.alloc(0);
  private decoder = new TextDecoder("utf-8", { fatal: true });
  push(chunk: Buffer): Record<string, unknown>[] {
    this.buf = Buffer.concat([this.buf, chunk]);
    if (this.buf.length > MAX_FRAME && !this.buf.includes(10)) throw new MoteError("stdout frame exceeds limit", "PROTOCOL");
    const out: Record<string, unknown>[] = [];
    let nl: number;
    while ((nl = this.buf.indexOf(10)) !== -1) {
      const line = this.buf.subarray(0, nl + 1); this.buf = this.buf.subarray(nl + 1);
      if (line.length > MAX_FRAME) throw new MoteError("stdout frame exceeds limit", "PROTOCOL");
      const text = this.decoder.decode(line.subarray(0, line.length - 1));
      if (text.length === 0) throw new MoteError("blank stdout frame", "PROTOCOL");
      let value: unknown; try { value = JSON.parse(text); } catch { throw new MoteError("malformed stdout frame", "PROTOCOL"); }
      if (!isObject(value)) throw new MoteError("stdout frame is not an object", "PROTOCOL");
      out.push(value);
    }
    if (this.buf.length >= MAX_FRAME) throw new MoteError("unterminated stdout frame exceeds limit", "PROTOCOL");
    return out;
  }
  finish(): void { if (this.buf.length) throw new MoteError("unterminated stdout frame", "PROTOCOL"); }
}

function stderrCollector(child: ChildProcess): { text: () => string } {
  let data = Buffer.alloc(0);
  child.stderr?.on("data", (chunk: Buffer) => { data = Buffer.concat([data, chunk]); if (data.length > MAX_STDERR) data = data.subarray(data.length - MAX_STDERR); });
  return { text: () => data.toString("utf8") };
}

export class Client {
  private readonly options: Required<Pick<ClientOptions, "binary" | "timeoutMs">> & Omit<ClientOptions, "binary" | "timeoutMs">;
  constructor(options: ClientOptions = {}) {
    const timeoutMs = options.timeoutMs ?? 150_000;
    if (!Number.isFinite(timeoutMs) || timeoutMs <= 0 || timeoutMs > 2_147_483_647)
      throw new MoteError("timeoutMs must be a positive finite timer duration", "SETUP");
    this.options = { ...options, binary: options.binary ?? "mote-bridge", timeoutMs };
  }

  async run(request: RunRequest): Promise<RunResult> {
    if (!isObject(request) || !isObject(request.manifest) || typeof request.task !== "string")
      throw new MoteError("invalid run request", "SETUP");
    const tools = request.tools ?? {};
    const capabilities = request.manifest.capabilities ?? [];
    if (!isObject(tools) || !Array.isArray(capabilities) || !capabilities.every(v => typeof v === "string"))
      throw new MoteError("invalid tools or capabilities", "SETUP");
    const grants = new Set(capabilities);
    const toolNames = Object.keys(tools);
    for (const name of toolNames) {
      if (!tools[name] || typeof tools[name].description !== "string" || typeof tools[name].handler !== "function")
        throw new MoteError("invalid tool registration", "SETUP");
    }
    const encode = (value: unknown): Buffer => {
      let text: string;
      try { text = JSON.stringify(value); }
      catch { throw new MoteError("frame is not JSON serializable", "PROTOCOL"); }
      const bytes = Buffer.from(text + "\n", "utf8");
      if (bytes.length > MAX_FRAME) throw new MoteError("outgoing frame exceeds limit", "PROTOCOL");
      return bytes;
    };
    const initial = encode({ type: "run", protocol: 1, manifest: request.manifest, task: request.task,
      tools: toolNames.map(name => ({ name, description: tools[name].description })) });
    let child: ReturnType<typeof spawn>;
    try { child = spawn(this.options.binary, this.options.args ?? [], { shell: false, cwd: this.options.cwd,
      env: { ...process.env, ...this.options.env }, stdio: ["pipe", "pipe", "pipe"] }); }
    catch { throw new MoteError("could not start bridge", "TRANSPORT"); }
    const closed = new Promise<number | null>(resolve => child.once("close", code => resolve(code)));
    let stopped: MoteError | undefined;
    let rejectFailure!: (error: MoteError) => void;
    const failure = new Promise<never>((_, reject) => { rejectFailure = reject; });
    const shutdown = (): void => {
      if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
      child.stdin?.destroy(); child.stdout?.destroy(); child.stderr?.destroy();
    };
    const abort = (error: MoteError): void => {
      if (!stopped) { stopped = error; rejectFailure(error); shutdown(); }
    };
    child.on("error", () => abort(new MoteError("bridge process error", "TRANSPORT")));
    child.stdin!.on("error", () => abort(new MoteError("bridge stdin failed", "TRANSPORT")));
    child.stderr!.on("error", () => abort(new MoteError("bridge stderr failed", "TRANSPORT")));
    stderrCollector(child);
    const timer = setTimeout(() => abort(new MoteError("bridge timed out", "TIMEOUT")), this.options.timeoutMs);
    const write = (bytes: Buffer): Promise<void> => {
      if (stopped) return Promise.reject(stopped);
      return new Promise((resolve, reject) => child.stdin!.write(bytes, error => error
        ? reject(new MoteError("bridge write failed", "TRANSPORT")) : resolve()));
    };
    const execute = async (): Promise<RunResult> => {
      await write(initial);
      const frames = new Frames();
      let expectedId = 1;
      let terminal: {state: RunState; events: Record<string, unknown>[]} | undefined;
      for await (const chunk of child.stdout!) {
        let messages: Record<string, unknown>[];
        try { messages = frames.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)); }
        catch { throw new MoteError("invalid stdout frame", "PROTOCOL"); }
        for (const frame of messages) {
          if (terminal) throw new MoteError("frame after terminal result", "PROTOCOL");
          if (frame.type === "tool_call") {
            if (Object.keys(frame).some(k => !["type", "id", "name", "input"].includes(k)) ||
              !Number.isSafeInteger(frame.id) || frame.id !== expectedId || typeof frame.name !== "string" ||
              !Object.hasOwn(tools, frame.name) || !grants.has(frame.name) || !isObject(frame.input))
              throw new MoteError("invalid or ungranted tool call", "PROTOCOL");
            expectedId++;
            let reply: Record<string, unknown>;
            try {
              const output = await tools[frame.name].handler(frame.input);
              if (typeof output !== "string") throw new Error("non-string result");
              reply = { type: "tool_result", id: frame.id, output };
            } catch {
              reply = { type: "tool_result", id: frame.id, error: "tool handler failed" };
            }
            await write(encode(reply));
          } else if (frame.type === "run_result") {
            if (Object.keys(frame).some(k => !["type", "state", "events"].includes(k)) ||
              typeof frame.state !== "string" || !states.has(frame.state as RunState) ||
              !Array.isArray(frame.events) || !frame.events.every(isObject))
              throw new MoteError("invalid run result", "PROTOCOL");
            terminal = {state: frame.state as RunState, events: frame.events};
            child.stdin!.end();
          } else {
            throw new MoteError("unexpected bridge message", "PROTOCOL");
          }
        }
      }
      frames.finish();
      if (!terminal) throw new MoteError("bridge closed before result", "PROTOCOL");
      const code = await closed;
      if (code === null || code !== (terminal.state === "completed" ? 0 : 1))
        throw new MoteError("exit status disagrees with result", "PROTOCOL");
      return {...terminal, returncode: code, success: terminal.state === "completed"};
    };
    try {
      // Do not await a stalled host Promise after timeout: closing the child is
      // independent of settling that Promise. No later reply can be written.
      return await Promise.race([execute(), failure]);
    } catch (error) {
      const problem = stopped ?? (error instanceof MoteError ? error : new MoteError("bridge transport failed", "TRANSPORT"));
      stopped = problem;
      throw problem;
    } finally {
      clearTimeout(timer);
      shutdown();
      await closed;
    }
  }
}
