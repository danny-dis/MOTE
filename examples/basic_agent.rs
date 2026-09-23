//! Run with `cargo run --example basic_agent` (no API key required).
use mote::action::Action;
use mote::config::Manifest;
use mote::model::Model;
use mote::runtime::{Runtime, State};
use mote::WorkspaceFs;
use std::sync::atomic::{AtomicUsize, Ordering};

struct DemoModel(AtomicUsize);

impl Model for DemoModel {
    fn infer(&self, _prompt: &str) -> Result<Action, String> {
        if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
            Ok(Action::ListDir { path: ".".into() })
        } else {
            Ok(Action::Complete {
                summary: "Workspace listed".into(),
            })
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = Manifest::from_yaml("name: basic-agent\ncapabilities: [list_dir]\n")?;
    let workspace = WorkspaceFs::new(std::env::current_dir()?)?;
    let mut runtime = Runtime::new(&manifest, workspace).map_err(std::io::Error::other)?;
    let state = runtime.run(&DemoModel(AtomicUsize::new(0)), "List this workspace");
    println!("State: {state:?}; events: {}", runtime.events.len());
    if state != State::Completed {
        return Err(std::io::Error::other("agent did not complete").into());
    }
    Ok(())
}
