//! Versioned, one-run stdio host-tool bridge. See docs/BRIDGE_PROTOCOL.md.
use mote::config::Manifest;
use mote::model::ModelChain;
use mote::runtime::{CustomTool, Runtime, State};
use mote::WorkspaceFs;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, Read, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const MAX_FRAME: usize = 1_048_576;
type Frames = mpsc::Receiver<Result<Value, String>>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolSpec {
    name: String,
    description: String,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Message {
    Run {
        protocol: u32,
        manifest: Box<Manifest>,
        task: String,
        #[serde(default)]
        tools: Vec<ToolSpec>,
    },
    ToolResult {
        id: u64,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        error: Option<String>,
    },
}

fn send(value: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    if bytes.len() > MAX_FRAME {
        return Err("outgoing bridge frame exceeds byte limit".into());
    }
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(&bytes)
        .and_then(|_| stdout.flush())
        .map_err(|e| e.to_string())
}

fn input_frames() -> Frames {
    // Bounded channel + bounded reads prevent a pipelining host from queuing
    // unlimited data. A blocked stdin reader dies with this one-run process.
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut input = std::io::stdin().lock();
        loop {
            let mut bytes = Vec::new();
            let frame = (&mut input)
                .take((MAX_FRAME + 1) as u64)
                .read_until(b'\n', &mut bytes)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    if bytes.len() > MAX_FRAME {
                        return Err("incoming bridge frame exceeds byte limit".into());
                    }
                    if bytes.last() != Some(&b'\n') {
                        return Err("bridge stdin closed or unterminated frame".into());
                    }
                    serde_json::from_slice(&bytes).map_err(|e| format!("invalid bridge JSON: {e}"))
                });
            let failed = frame.is_err();
            if sender.send(frame).is_err() || failed {
                break;
            }
        }
    });
    receiver
}

fn receive(receiver: &Frames, timeout: Duration) -> Result<Message, String> {
    let value = receiver
        .recv_timeout(timeout)
        .map_err(|_| "bridge input closed or deadline exceeded".to_owned())??;
    // Option<T> would otherwise treat an explicit null as an absent field.
    // Protocol v1 requires exactly one present, string-valued result field.
    if value.get("type").and_then(Value::as_str) == Some("tool_result")
        && !matches!(
            (value.get("output"), value.get("error")),
            (Some(Value::String(_)), None) | (None, Some(Value::String(_)))
        )
    {
        return Err("tool result requires exactly one string output or error".into());
    }
    serde_json::from_value(value).map_err(|e| format!("invalid bridge message: {e}"))
}

fn run() -> Result<i32, String> {
    let receiver = input_frames();
    let Message::Run {
        protocol,
        manifest,
        task,
        tools,
    } = receive(&receiver, Duration::from_secs(30))?
    else {
        return Err("expected run startup message".into());
    };
    if protocol != 1 {
        return Err("unsupported bridge protocol version".into());
    }
    let started = Instant::now();
    let deadline = Duration::from_secs(manifest.max_runtime_seconds);
    let receiver = Arc::new(Mutex::new(receiver));
    let next_id = Arc::new(std::sync::atomic::AtomicU64::new(1));
    let contracts: Vec<_> = tools
        .iter()
        .map(|tool| json!({"name":tool.name,"description":tool.description}))
        .collect();
    let handlers = tools
        .into_iter()
        .map(|tool| {
            let receiver = Arc::clone(&receiver);
            let next_id = Arc::clone(&next_id);
            let name = tool.name.clone();
            CustomTool::new(tool.name, move |input| {
                let receiver = receiver
                    .lock()
                    .map_err(|_| "bridge transport unavailable")?;
                let id = next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                send(&json!({"type":"tool_call","id":id,"name":name,"input":input}))?;
                let reply = receive(&receiver, deadline.saturating_sub(started.elapsed()))?;
                match reply {
                    Message::ToolResult {
                        id: returned,
                        output: Some(output),
                        error: None,
                    } if returned == id => Ok(output),
                    Message::ToolResult {
                        id: returned,
                        output: None,
                        error: Some(_),
                    } if returned == id => Err("host tool callback failed".into()),
                    _ => Err("unexpected or mismatched tool result".into()),
                }
            })
        })
        .collect();
    // Registration never grants permission. Reuse the public runtime's checks.
    let workspace = WorkspaceFs::new(&manifest.workspace).map_err(|e| e.to_string())?;
    let mut runtime = Runtime::with_tools(&manifest, workspace, handlers)?;
    let model = ModelChain::from_manifest(&manifest)?;
    let task = format!(
        "{task}\nHost tool contracts (registration is not permission): {}",
        Value::Array(contracts)
    );
    let state = runtime.run(&model, &task);
    let name = match state {
        State::Completed => "completed",
        State::Failed => "failed",
        State::Cancelled => "cancelled",
        State::Running => return Err("runtime returned nonterminal state".into()),
    };
    send(&json!({"type":"run_result","state":name,"events":runtime.events}))?;
    Ok(if state == State::Completed { 0 } else { 1 })
}

fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(message) => {
            let _ = send(&json!({"type":"error","message":message}));
            2
        }
    };
    std::process::exit(code);
}
