use mote::config::Manifest;
use mote::event::Event;
use mote::model::Model;
use mote::runtime::{CustomTool, Runtime, State};
use mote::WorkspaceFs;

/// Application-owned acceptance criteria, not just the runtime's exit state.
pub fn run(manifest: &Manifest, model: &dyn Model) -> Result<(), String> {
    if manifest.output_file.as_deref() != Some(std::path::Path::new("report.txt")) {
        return Err("this starter requires output_file: report.txt".into());
    }
    let workspace = WorkspaceFs::new(&manifest.workspace).map_err(|e| e.to_string())?;
    let uppercase = CustomTool::new("uppercase", |input| {
        input
            .get("text")
            .and_then(|v| v.as_str())
            .map(str::to_uppercase)
            .ok_or_else(|| "text must be a string".into())
    });
    let mut runtime = Runtime::with_tools(manifest, workspace, vec![uppercase])?;
    let task = "Call the uppercase tool with JSON input {\"text\":\"MOTE works\"}. \
        It accepts a text string and returns uppercase text. Write its exact returned \
        text to report.txt with no added newline. Use the tool, do not compute the result yourself.";
    let state = runtime.run(model, task);
    if state != State::Completed {
        return Err(format!("agent did not complete: {state:?}"));
    }
    let tool_used = runtime.events.iter().any(
        |event| matches!(event, Event::ActionExecuted { capability } if capability == "uppercase"),
    );
    let actual = runtime
        .workspace
        .read(std::path::Path::new("report.txt"))
        .map_err(|e| e.to_string())?;
    if !tool_used || actual != "MOTE WORKS" {
        return Err(
            "acceptance check failed: tool use and exact report content are required".into(),
        );
    }
    Ok(())
}
