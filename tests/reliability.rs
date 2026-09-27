use mote::action::Action;
use mote::model::{parse_action, HttpModel, Model, Provider};

#[test]
fn write_file_payload_is_lossless() {
    let expected = Action::WriteFile {
        path: "script.py".into(),
        content: "    indented\n\n".into(),
    };
    assert_eq!(
        parse_action("write_file: script.py |     indented\n\n").unwrap(),
        expected
    );
    assert_eq!(
        parse_action(&serde_json::to_string(&expected).unwrap()).unwrap(),
        expected
    );
    let empty = Action::WriteFile {
        path: "empty.txt".into(),
        content: String::new(),
    };
    assert_eq!(
        parse_action(&serde_json::to_string(&empty).unwrap()).unwrap(),
        empty
    );
    assert_eq!(parse_action("write_file: empty.txt | ").unwrap(), empty);
}

use mote::config::Manifest;
use mote::event::Event;
use mote::runtime::{Runtime, State};
use mote::WorkspaceFs;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

#[test]
fn completion_uses_the_normal_write_budget_and_events() {
    let root = std::env::temp_dir().join(format!("mote-completion-budget-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for limit in [1, 2] {
        let report = root.join("report.txt");
        std::fs::write(&report, "old report").unwrap();
        let manifest = Manifest::from_yaml(&format!("name: budget\ncapabilities: [write_file]\nmax_per_tool: {limit}\noutput_file: report.txt\n")).unwrap();
        let mut runtime = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
        let state = runtime.run_actions(&[
            Action::WriteFile {
                path: "intermediate.txt".into(),
                content: "first write".into(),
            },
            Action::Complete {
                summary: "fresh report".into(),
            },
        ]);
        if limit == 1 {
            assert_eq!(state, State::Failed);
            assert_eq!(std::fs::read_to_string(&report).unwrap(), "old report");
        } else {
            assert_eq!(state, State::Completed);
            assert_eq!(std::fs::read_to_string(&report).unwrap(), "fresh report");
        }
        let writes = runtime.events.iter().filter(|event| matches!(event, Event::ActionExecuted {capability} if capability == "write_file")).count();
        assert_eq!(writes, limit);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn model_receives_grants_and_labeled_completed_actions() {
    use std::sync::Mutex;
    struct Inspect(Mutex<Vec<String>>);
    impl Model for Inspect {
        fn infer(&self, prompt: &str) -> Result<Action, String> {
            let mut prompts = self.0.lock().unwrap();
            prompts.push(prompt.to_owned());
            Ok(if prompts.len() == 1 {
                Action::ReadFile {
                    path: "input.txt".into(),
                }
            } else {
                Action::Complete {
                    summary: "done".into(),
                }
            })
        }
    }
    let root = std::env::temp_dir().join(format!("mote-context-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("input.txt"), "payload").unwrap();
    let manifest =
        Manifest::from_yaml("name: context\ncapabilities: [read_file]\nmax_per_tool: 2\n").unwrap();
    let mut runtime = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
    let model = Inspect(Mutex::new(Vec::new()));
    assert_eq!(
        runtime.run(&model, "Read input.txt and report."),
        State::Completed
    );
    let prompts = model.0.lock().unwrap();
    assert!(prompts[0].contains("\"capabilities\":[\"read_file\"]"));
    assert!(prompts[0].contains("\"max_per_tool\":2"));
    assert!(prompts[1].contains("ReadFile") && prompts[1].contains("input.txt"));
    assert!(prompts[1].contains("\"observation\":\"payload\""));
    assert!(prompts[1].contains("already executed"));
    std::fs::remove_dir_all(root).unwrap();
}

fn json_server(body: Value) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 8192];
        loop {
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
            if let Some(end) = request.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]);
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse().unwrap())
                    })
                    .unwrap();
                if request.len() >= end + 4 + length {
                    break;
                }
            }
        }
        let body = body.to_string();
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
    });
    format!("http://{address}/model")
}

#[test]
fn incomplete_provider_responses_never_become_actions() {
    let action = "write_file: result.json | {\"total\":";
    let responses = [
        (
            Provider::OpenAiCompatible,
            json!({"choices":[{"finish_reason":"length","message":{"content":action}}]}),
        ),
        (
            Provider::DmrX,
            json!({"choices":[{"finish_reason":"content_filter","message":{"content":action}}]}),
        ),
        (
            Provider::Anthropic,
            json!({"stop_reason":"max_tokens","content":[{"text":action}]}),
        ),
        (
            Provider::Gemini,
            json!({"candidates":[{"finishReason":"MAX_TOKENS","content":{"parts":[{"text":action}]}}]}),
        ),
    ];
    for (provider, response) in responses {
        let model = HttpModel::new(provider, json_server(response), "test", None);
        let error = model
            .infer("write the report")
            .expect_err("must reject incomplete output");
        assert!(error.contains("incomplete"), "{error}");
    }
}
