use mote::action::Action;
use mote::config::Manifest;
use mote::event::Event;
use mote::runtime::{CancellationToken, CustomTool, Runtime, State};
use mote::WorkspaceFs;
use serde_json::json;
use std::sync::{Arc, Mutex};

fn workspace() -> WorkspaceFs {
    WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap()
}

fn manifest(capabilities: &str) -> Manifest {
    Manifest::from_yaml(&format!(
        "name: host-tools\ncapabilities: [{capabilities}]\nmax_per_tool: 1\nmax_output_bytes: 16\n"
    ))
    .unwrap()
}

#[test]
fn registered_and_granted_tool_executes_with_typed_input_and_events() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&seen);
    let tool = CustomTool::new("lookup", move |input| {
        captured.lock().unwrap().push(input.clone());
        Ok("found".to_owned())
    });
    let mut runtime = Runtime::with_tools(&manifest("lookup"), workspace(), vec![tool]).unwrap();
    let state = runtime.run_actions(&[
        Action::Custom {
            name: "lookup".into(),
            input: json!({"key": "SECRET-INPUT"}),
        },
        Action::Complete {
            summary: "done".into(),
        },
    ]);
    assert_eq!(state, State::Completed);
    assert_eq!(*seen.lock().unwrap(), vec![json!({"key": "SECRET-INPUT"})]);
    assert!(runtime.events.iter().any(|event| matches!(event,
        Event::ActionExecuted { capability } if capability == "lookup")));
    assert!(runtime.events.iter().any(|event| matches!(event,
        Event::ObservationReceived { text } if text == "found")));
    assert!(!runtime.events.iter().any(|event| matches!(event,
        Event::ActionProposed { action } if action.contains("SECRET-INPUT"))));
}

#[test]
fn callback_error_does_not_publish_secret_input_in_events() {
    let tool = CustomTool::new("lookup", |_| Err("SECRET-CALLBACK-ERROR".into()));
    let mut runtime = Runtime::with_tools(&manifest("lookup"), workspace(), vec![tool]).unwrap();
    assert_eq!(
        runtime.run_actions(&[Action::Custom {
            name: "lookup".into(),
            input: json!({"token": "SECRET-INPUT"}),
        }]),
        State::Failed
    );
    assert!(!format!("{:?}", runtime.events).contains("SECRET-"));
}

#[test]
fn callback_cancellation_is_observed_before_publishing_output() {
    let token = CancellationToken::default();
    let inside = token.clone();
    let tool = CustomTool::new("lookup", move |_| {
        inside.cancel();
        Ok("should not publish".into())
    });
    let mut runtime = Runtime::with_tools(&manifest("lookup"), workspace(), vec![tool]).unwrap();
    runtime.cancellation = token;
    assert_eq!(
        runtime.run_actions(&[Action::Custom {
            name: "lookup".into(),
            input: json!({}),
        }]),
        State::Cancelled
    );
    assert!(!format!("{:?}", runtime.events).contains("should not publish"));
}

#[test]
fn oversized_custom_input_never_reaches_handler() {
    let called = Arc::new(Mutex::new(false));
    let marker = Arc::clone(&called);
    let tool = CustomTool::new("lookup", move |_| {
        *marker.lock().unwrap() = true;
        Ok("found".into())
    });
    let mut runtime = Runtime::with_tools(&manifest("lookup"), workspace(), vec![tool]).unwrap();
    assert_eq!(
        runtime.run_actions(&[Action::Custom {
            name: "lookup".into(),
            input: json!({"payload": "x".repeat(1_048_576)}),
        }]),
        State::Failed
    );
    assert!(!*called.lock().unwrap());
    assert!(!format!("{:?}", runtime.events).contains(&"x".repeat(128)));
}

#[test]
fn ungranted_and_unregistered_tools_never_invoke_handlers() {
    let called = Arc::new(Mutex::new(false));
    let marker = Arc::clone(&called);
    let tool = CustomTool::new("lookup", move |_| {
        *marker.lock().unwrap() = true;
        Ok("found".into())
    });
    let mut runtime = Runtime::with_tools(&manifest(""), workspace(), vec![tool]).unwrap();
    assert_eq!(
        runtime.run_actions(&[Action::Custom {
            name: "lookup".into(),
            input: json!({}),
        }]),
        State::Failed
    );
    assert!(!*called.lock().unwrap());
    assert!(Runtime::new(&manifest("lookup"), workspace()).is_err());
    assert!(Runtime::with_tools(&manifest("missing"), workspace(), vec![]).is_err());
}

#[test]
fn duplicate_and_reserved_tool_names_are_rejected() {
    let tool = || CustomTool::new("lookup", |_| Ok("found".into()));
    assert!(Runtime::with_tools(&manifest("lookup"), workspace(), vec![tool(), tool()]).is_err());
    assert!(Runtime::with_tools(
        &manifest(""),
        workspace(),
        vec![CustomTool::new("shell", |_| Ok("bad".into()))]
    )
    .is_err());
}

#[test]
fn custom_tool_budget_is_independent_per_name() {
    let mut runtime = Runtime::with_tools(
        &manifest("lookup, uppercase"),
        workspace(),
        vec![
            CustomTool::new("lookup", |_| Ok("found".into())),
            CustomTool::new("uppercase", |_| Ok("CAPS".into())),
        ],
    )
    .unwrap();
    assert_eq!(
        runtime.run_actions(&[
            Action::Custom {
                name: "lookup".into(),
                input: json!({"n":1})
            },
            Action::Custom {
                name: "uppercase".into(),
                input: json!({"n":2})
            },
            Action::Custom {
                name: "lookup".into(),
                input: json!({"n":3})
            },
            Action::Complete {
                summary: "done".into()
            },
        ]),
        State::Completed
    );
    assert!(runtime.events.iter().any(|event| matches!(event,
        Event::ObservationReceived { text } if text.contains("lookup tool call limit"))));
    assert!(runtime.events.iter().any(|event| matches!(event,
        Event::ActionExecuted { capability } if capability == "uppercase")));
}

#[test]
fn oversized_custom_output_never_enters_events_or_context() {
    let tool = CustomTool::new("lookup", |_| Ok("SENSITIVE-OUTPUT".repeat(20)));
    let mut runtime = Runtime::with_tools(&manifest("lookup"), workspace(), vec![tool]).unwrap();
    assert_eq!(
        runtime.run_actions(&[Action::Custom {
            name: "lookup".into(),
            input: json!({}),
        }]),
        State::Failed
    );
    assert!(runtime.events.iter().any(|event| matches!(event,
        Event::Error { message } if message.contains("max_output_bytes"))));
    assert!(!runtime.events.iter().any(|event| matches!(
        event,
        Event::ObservationReceived { .. } | Event::ActionExecuted { .. }
    )));
    assert!(!format!("{:?}", runtime.events).contains("SENSITIVE-OUTPUT"));
}
