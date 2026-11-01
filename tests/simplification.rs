use mote::action::Action;
use mote::config::Manifest;
use mote::decision::{DecisionQuestion, DecisionRequest};
use mote::event::Event;
use mote::runtime::{CustomTool, Runtime, State};
use mote::WorkspaceFs;
use serde_json::json;

#[test]
fn runtime_guards_report_the_failure_they_are_meant_to_detect() {
    for (iterations, paths, expected) in [
        (4, vec![".", ".", "."], "repeated action guard"),
        (2, vec![".", "./"], "iteration budget exhausted"),
    ] {
        let manifest = Manifest::from_yaml(&format!(
            "name: guards\ncapabilities: [list_dir]\nmax_iterations: {iterations}\n"
        ))
        .unwrap();
        let mut runtime = Runtime::new(
            &manifest,
            WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap(),
        )
        .unwrap();
        let actions: Vec<_> = paths
            .into_iter()
            .map(|path| Action::ListDir { path: path.into() })
            .collect();
        assert_eq!(runtime.run_actions(&actions), State::Failed);
        assert!(
            matches!(runtime.events.last(), Some(Event::Error { message }) if message == expected)
        );
        assert_eq!(
            runtime
                .events
                .iter()
                .filter(|event| matches!(event, Event::ActionExecuted { .. }))
                .count(),
            2
        );
    }
}

#[test]
fn every_action_keeps_its_own_capability_grant() {
    let manifest = Manifest::from_yaml("name: grants\n").unwrap();
    let runtime = Runtime::with_tools(
        &manifest,
        WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap(),
        vec![CustomTool::new("lookup", |_| Ok("ok".into()))],
    )
    .unwrap();
    for (action, capability) in [
        (
            Action::Shell {
                command: "echo ok".into(),
            },
            "shell",
        ),
        (
            Action::ReadFile {
                path: "input.txt".into(),
            },
            "read_file",
        ),
        (
            Action::WriteFile {
                path: "output.txt".into(),
                content: "ok".into(),
            },
            "write_file",
        ),
        (Action::ListDir { path: ".".into() }, "list_dir"),
        (
            Action::Git {
                args: "status".into(),
            },
            "git",
        ),
        (
            Action::Decision {
                request: DecisionRequest {
                    id: "gate".into(),
                    state: json!({}),
                    question: DecisionQuestion::Noul {
                        instructions: "score".into(),
                    },
                },
            },
            "decision",
        ),
        (
            Action::Custom {
                name: "lookup".into(),
                input: json!({}),
            },
            "lookup",
        ),
    ] {
        assert_eq!(
            runtime.validate_action(&action).unwrap_err(),
            format!("capability not granted: {capability}")
        );
    }
    assert!(runtime
        .validate_action(&Action::Complete {
            summary: "done".into()
        })
        .is_ok());
    assert_eq!(
        runtime
            .validate_action(&Action::Custom {
                name: "missing".into(),
                input: json!([])
            })
            .unwrap_err(),
        "custom tool input must be a JSON object"
    );
    assert_eq!(
        runtime
            .validate_action(&Action::Custom {
                name: "missing".into(),
                input: json!({})
            })
            .unwrap_err(),
        "custom tool not registered: missing"
    );
}

#[test]
fn read_and_list_preserve_errors_and_zero_budgets() {
    for (action, capability) in [
        (
            Action::ReadFile {
                path: "__mote_missing_read__".into(),
            },
            "read_file",
        ),
        (
            Action::ListDir {
                path: "__mote_missing_list__".into(),
            },
            "list_dir",
        ),
    ] {
        for limit in [0, 1] {
            let manifest = Manifest::from_yaml(&format!(
                "name: reads\ncapabilities: [{capability}]\nmax_per_tool: {limit}\n"
            ))
            .unwrap();
            let mut runtime = Runtime::new(
                &manifest,
                WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap(),
            )
            .unwrap();
            assert_eq!(
                runtime.run_actions(std::slice::from_ref(&action)),
                State::Failed
            );
            assert!(!runtime
                .events
                .iter()
                .any(|event| matches!(event, Event::ActionExecuted { .. })));
            if limit == 0 {
                assert!(runtime.events.iter().any(|event| matches!(event, Event::ObservationReceived { text } if text == &format!("error: {capability} tool call limit (0) reached"))));
            } else {
                assert!(runtime.events.iter().any(|event| matches!(event, Event::Error { message } if message == "path does not exist")));
            }
        }
    }
}
