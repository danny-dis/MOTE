use mote::config::Manifest;
use mote::model::ModelChain;
use mote::runtime::{Runtime, State};
use mote::WorkspaceFs;
use std::path::PathBuf;

fn usage() -> &'static str {
    "Usage: mote [--jsonl] <manifest.yaml> <task>\n       mote --version"
}

fn run() -> Result<i32, String> {
    let mut jsonl = false;
    let mut positional = Vec::new();
    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--jsonl" => jsonl = true,
            "--version" => {
                println!("mote {}", env!("CARGO_PKG_VERSION"));
                return Ok(0);
            }
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(0);
            }
            _ => positional.push(argument),
        }
    }
    if positional.len() < 2 {
        return Err(usage().to_owned());
    }

    let manifest_path = PathBuf::from(&positional[0]);
    let task = positional[1..].join(" ");
    let yaml = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("unable to read manifest: {error}"))?;
    let manifest =
        Manifest::from_yaml(&yaml).map_err(|error| format!("invalid manifest: {error}"))?;
    let workspace_path = manifest.workspace_from(&manifest_path);
    let workspace = WorkspaceFs::new(&workspace_path)
        .map_err(|error| format!("invalid workspace {}: {error}", workspace_path.display()))?;
    let model = ModelChain::from_manifest(&manifest)?;
    let mut runtime = Runtime::new(&manifest, workspace)?;
    let state = runtime.run(&model, &task);

    if jsonl {
        for event in &runtime.events {
            println!("{}", event.json_line().map_err(|error| error.to_string())?);
        }
    } else {
        println!("MOTE {} — {}", env!("CARGO_PKG_VERSION"), manifest.name);
        println!("State: {state:?}");
        println!("Events: {}", runtime.events.len());
    }

    Ok(if state == State::Completed { 0 } else { 1 })
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
