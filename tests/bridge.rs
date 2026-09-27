use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::time::Duration;

fn model_server(actions: Vec<Value>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for action in actions {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 4096];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                assert!(count > 0);
                request.extend_from_slice(&buffer[..count]);
                if let Some(end) = request.windows(4).position(|p| p == b"\r\n\r\n") {
                    let length: usize = String::from_utf8_lossy(&request[..end])
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let body = json!({"choices":[{"finish_reason":"stop","message":{"content":action.to_string()}}]}).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        }
    });
    format!("http://{address}/model")
}

fn request(endpoint: &str, granted: bool) -> Value {
    json!({"type":"run","protocol":1,
        "manifest":{"name":"bridge-test","workspace":std::env::temp_dir(),
            "capabilities": if granted { vec!["uppercase"] } else { vec![] },
            "max_runtime_seconds":2,
            "models":[{"provider":"openai-compatible","model":"test","endpoint":endpoint,"timeout_seconds":2}]},
        "task":"Use uppercase with text hello then complete.",
        "tools":[{"name":"uppercase","description":"Input: text string. Returns uppercase text."}]})
}

fn run_input(input: &[u8]) -> (i32, Vec<Value>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mote-bridge"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input);
    let output = child.wait_with_output().unwrap();
    let frames = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (output.status.code().unwrap(), frames)
}

#[test]
fn custom_callback_round_trip_uses_the_real_runtime() {
    let endpoint = model_server(vec![
        json!({"action":"custom","name":"uppercase","input":{"text":"hello"}}),
        json!({"action":"complete","summary":"done"}),
    ]);
    let mut child = Command::new(env!("CARGO_BIN_EXE_mote-bridge"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    writeln!(input, "{}", request(&endpoint, true)).unwrap();
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let call: Value = serde_json::from_str(&line).expect("bridge must request host callback");
    assert_eq!(
        call,
        json!({"type":"tool_call","id":1,"name":"uppercase","input":{"text":"hello"}})
    );
    let text = call["input"]["text"].as_str().unwrap().to_uppercase();
    writeln!(
        input,
        "{}",
        json!({"type":"tool_result","id":1,"output":text})
    )
    .unwrap();
    line.clear();
    output.read_line(&mut line).unwrap();
    let result: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(result["state"], "completed");
    assert!(result["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|event| event["type"] == "ObservationReceived" && event["data"]["text"] == "HELLO"));
    drop(input);
    assert!(child.wait().unwrap().success());
}

#[test]
fn startup_and_framing_errors_fail_closed() {
    let oversized = vec![b'x'; 1_048_577];
    for bytes in [
        b"{}\n".as_slice(),
        b"{bad}\n",
        b"{}",
        b"\xff\n",
        b"\n",
        oversized.as_slice(),
    ] {
        let (status, frames) = run_input(bytes);
        assert_eq!(status, 2, "{frames:?}");
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0]["type"], "error");
    }
    let mut invalid = request("http://127.0.0.1:1", false);
    invalid["protocol"] = json!(2);
    let (status, frames) = run_input(format!("{invalid}\n").as_bytes());
    assert_eq!(status, 2);
    assert_eq!(frames[0]["type"], "error");
}

#[test]
fn denied_tool_never_crosses_the_bridge() {
    let endpoint = model_server(vec![
        json!({"action":"custom","name":"uppercase","input":{"text":"hello"}}),
    ]);
    let (status, frames) = run_input(format!("{}\n", request(&endpoint, false)).as_bytes());
    assert_eq!(status, 1);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["state"], "failed");
}

#[test]
fn invalid_callback_results_cannot_complete_the_run() {
    for reply in [
        json!({"type":"tool_result","id":2,"output":"HELLO"}),
        json!({"type":"tool_result","id":1,"output":"HELLO","error":"oops"}),
        json!({"type":"tool_result","id":1}),
        json!({"type":"tool_result","id":1,"output":42}),
        json!({"type":"tool_result","id":1,"output":"HELLO","error":null}),
        json!({"type":"tool_result","id":1,"output":null,"error":"failure"}),
    ] {
        let endpoint = model_server(vec![
            json!({"action":"custom","name":"uppercase","input":{"text":"hello"}}),
        ]);
        let (status, frames) =
            run_input(format!("{}\n{reply}\n", request(&endpoint, true)).as_bytes());
        assert_eq!(status, 1);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[1]["state"], "failed");
        assert!(!frames[1]["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["type"] == "ActionExecuted"));
    }
}

#[test]
fn callback_wait_respects_manifest_deadline() {
    let endpoint = model_server(vec![
        json!({"action":"custom","name":"uppercase","input":{"text":"hello"}}),
    ]);
    let mut req = request(&endpoint, true);
    req["manifest"]["max_runtime_seconds"] = json!(1);
    let mut child = Command::new(env!("CARGO_BIN_EXE_mote-bridge"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(input, "{req}").unwrap();
    // Keep stdin open but never answer; a hung host must not hang this run.
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(1));
    let frames: Vec<Value> = String::from_utf8(result.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(frames.last().unwrap()["state"], "failed");
}
