//! Run with `cargo run --locked --example custom_tool` (no model account required).
use mote::action::Action;
use mote::config::Manifest;
use mote::event::Event;
use mote::model::Model;
use mote::runtime::{CustomTool, Runtime, State};
use mote::WorkspaceFs;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};

struct DemoModel(AtomicUsize);

impl Model for DemoModel {
    fn infer(&self, _prompt: &str) -> Result<Action, String> {
        if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
            Ok(Action::Custom {
                name: "uppercase".into(),
                input: json!({"text": "hello"}),
            })
        } else {
            Ok(Action::Complete {
                summary: "Custom tool completed".into(),
            })
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_yaml("name: custom-tool\ncapabilities: [uppercase]\n")?;
    let workspace = WorkspaceFs::new(std::env::current_dir()?)?;
    let handler = CustomTool::new("uppercase", |input| {
        input["text"]
            .as_str()
            .map(str::to_uppercase)
            .ok_or_else(|| "text must be a string".to_owned())
    });
    let mut runtime =
        Runtime::with_tools(&manifest, workspace, vec![handler]).map_err(std::io::Error::other)?;
    let state = runtime.run(&DemoModel(AtomicUsize::new(0)), "Uppercase hello");
    let saw_uppercase = runtime
        .events
        .iter()
        .any(|event| matches!(event, Event::ObservationReceived { text } if text == "HELLO"));
    println!("State: {state:?}; uppercase observation: {saw_uppercase}");
    if state != State::Completed || !saw_uppercase {
        return Err(std::io::Error::other("custom tool run did not complete").into());
    }
    Ok(())
}
