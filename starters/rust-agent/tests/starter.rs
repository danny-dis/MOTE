use mote::action::Action;
use mote::config::Manifest;
use mote::model::Model;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

struct Script(Mutex<std::collections::VecDeque<Action>>);
impl Model for Script {
    fn infer(&self, _prompt: &str) -> Result<Action, String> {
        self.0
            .lock()
            .unwrap()
            .pop_front()
            .ok_or("script exhausted".into())
    }
}

fn scenario(actions: Vec<Action>) -> Result<(), String> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "mote-starter-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut manifest = Manifest::from_yaml(
        "name: starter\ncapabilities: [uppercase, write_file]\noutput_file: report.txt\n",
    )
    .unwrap();
    manifest.workspace = root.clone();
    let result = mote_rust_starter::run(&manifest, &Script(Mutex::new(actions.into())));
    std::fs::remove_dir_all(root).unwrap();
    result
}

#[test]
fn custom_tool_and_verified_artifact_complete_the_starter() {
    let custom = mote::model::parse_action(
        r#"{"action":"custom","name":"uppercase","input":{"text":"MOTE works"}}"#,
    )
    .unwrap();
    assert!(scenario(vec![
        custom,
        Action::WriteFile {
            path: "report.txt".into(),
            content: "MOTE WORKS".into()
        }
    ])
    .is_ok());
}

#[test]
fn correct_text_without_the_required_callback_is_not_success() {
    assert!(scenario(vec![Action::WriteFile {
        path: "report.txt".into(),
        content: "MOTE WORKS".into()
    }])
    .is_err());
}

#[test]
fn completed_but_wrong_artifact_is_not_success() {
    let custom = mote::model::parse_action(
        r#"{"action":"custom","name":"uppercase","input":{"text":"MOTE works"}}"#,
    )
    .unwrap();
    assert!(scenario(vec![
        custom,
        Action::Complete {
            summary: "not the requested text".into()
        }
    ])
    .is_err());
}
