use std::fs;
use std::process::Command;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    name: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default = "default_max_iters")]
    max_iterations: usize,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_endpoint")]
    endpoint: String,
    #[serde(default = "default_auth")]
    auth: String,
    #[serde(default = "default_provider")]
    provider: String,
}

fn default_provider() -> String { "openai".into() }
fn default_max_iters() -> usize { 20 }
fn default_model() -> String { "groq/compound-mini".into() }
fn default_endpoint() -> String { "https://api.groq.com/openai/v1/chat/completions".into() }
fn default_auth() -> String { String::new() }

const MOTE_SYSTEM_PROMPT: &str = "\
You are MOTE. Use the available capabilities to gather information, then complete.
CRITICAL RULES:
1. On the first turn, you MUST use a capability (shell, read_file, write_file, list_dir, git). NEVER use 'complete:' on turn 1.
2. Respond with EXACTLY ONE line. No explanations, no markdown, no extra text.
3. Valid formats:
   shell: <command>
   read_file: <path>
   write_file: <path> | <content>
   list_dir: <path>
   git: <args>
   complete: <summary>
4. YAGNI: pick the SINGLE simplest action that moves toward the goal. Do not chain speculative reads, writes, or commands. One precise action beats a multi-step guess.
Examples:
User: Show current date
shell: date
User: List files in current directory
list_dir: .
User: Read the README file
read_file: README.md
User: Create a file hello.py that prints hi
write_file: hello.py | print(\"hi\")
User: Show last 3 git commits
git: log -3
After observing results, you may use complete: <summary> to finish.";

#[derive(Debug, PartialEq, Clone)]
enum Action {
    Shell { command: String },
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    ListDir { path: String },
    Git { args: String },
    Complete { summary: String },
}

trait Model {
    fn infer(&self, prompt: &str) -> Action;
}

struct CurlModel {
    endpoint: String,
    model: String,
    auth: String,
}

impl CurlModel {
    fn new(endpoint: &str, model: &str, auth: &str) -> Self {
        Self { endpoint: endpoint.into(), model: model.into(), auth: auth.into() }
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
            .arg("-X").arg("POST")
            .arg("-H").arg("Content-Type: application/json")
            .arg("-H").arg(format!("Authorization: Bearer {}", self.auth))
            .arg("-d").arg(body)
            .arg(&self.endpoint);
        let output = cmd.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

impl Model for CurlModel {
    fn infer(&self, prompt: &str) -> Action {
        let system = MOTE_SYSTEM_PROMPT;
        let body = format!(
            r#"{{"model":"{}","messages":[{{"role":"system","content":"{}"}},{{"role":"user","content":"{}"}}],"max_tokens":200}}"#,
            self.model,
            system.replace('"', "\\\"").replace('\n', " "),
            prompt.replace('"', "\\\"").replace('\n', " ")
        );
        match self.post(&body) {
            Ok(text) => {
                let content = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.pointer("/choices/0/message/content").cloned())
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                parse_action(&content)
            }
            Err(e) => Action::Complete { summary: format!("model error: {}", e) },
        }
    }
}

// ── Google Gemini adapter ────────────────────────────────────────────
struct GoogleModel {
    model: String,
    auth: String,
}

impl GoogleModel {
    fn new(model: &str, auth: &str) -> Self {
        Self { model: model.into(), auth: auth.into() }
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.auth
        );
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
            .arg("-X").arg("POST")
            .arg("-H").arg("Content-Type: application/json")
            .arg("-d").arg(body)
            .arg(&url);
        let output = cmd.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

impl Model for GoogleModel {
    fn infer(&self, prompt: &str) -> Action {
        let system = MOTE_SYSTEM_PROMPT;
        let body = format!(
            r#"{{"contents":[{{"parts":[{{"text":"{}"}}]}},{{"parts":[{{"text":"{}"}}]}}]}}"#,
            system.replace('"', "\\\"").replace('\n', " "),
            prompt.replace('"', "\\\"").replace('\n', " ")
        );
        match self.post(&body) {
            Ok(text) => {
                let content = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.pointer("/candidates/0/content/parts/0/text").cloned())
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                parse_action(&content)
            }
            Err(e) => Action::Complete { summary: format!("model error: {}", e) },
        }
    }
}

// ── Anthropic adapter ────────────────────────────────────────────────
struct AnthropicModel {
    model: String,
    auth: String,
}

impl AnthropicModel {
    fn new(model: &str, auth: &str) -> Self {
        Self { model: model.into(), auth: auth.into() }
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
            .arg("-X").arg("POST")
            .arg("-H").arg("Content-Type: application/json")
            .arg("-H").arg("anthropic-version: 2023-06-01")
            .arg("-H").arg(format!("x-api-key: {}", self.auth))
            .arg("-d").arg(body)
            .arg("https://api.anthropic.com/v1/messages");
        let output = cmd.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

impl Model for AnthropicModel {
    fn infer(&self, prompt: &str) -> Action {
        let system = MOTE_SYSTEM_PROMPT;
        let body = format!(
            r#"{{"model":"{}","max_tokens":200,"system":"{}","messages":[{{"role":"user","content":"{}"}}]}}"#,
            self.model,
            system.replace('"', "\\\"").replace('\n', " "),
            prompt.replace('"', "\\\"").replace('\n', " ")
        );
        match self.post(&body) {
            Ok(text) => {
                let content = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.pointer("/content/0/text").cloned())
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                parse_action(&content)
            }
            Err(e) => Action::Complete { summary: format!("model error: {}", e) },
        }
    }
}

fn parse_action(content: &str) -> Action {
    let lower = content.to_lowercase();
    if lower.starts_with("complete:") {
        Action::Complete { summary: content[9..].trim().into() }
    } else if lower.starts_with("shell:") {
        Action::Shell { command: content[6..].trim().into() }
    } else if lower.starts_with("read_file:") {
        Action::ReadFile { path: content[10..].trim().into() }
    } else if lower.starts_with("write_file:") {
        let rest = &content[11..];
        if let Some((path, content_text)) = rest.split_once('|') {
            Action::WriteFile { path: path.trim().into(), content: content_text.trim().into() }
        } else {
            Action::Complete { summary: "write_file format: path | content".into() }
        }
    } else if lower.starts_with("list_dir:") {
        Action::ListDir { path: content[9..].trim().into() }
    } else if lower.starts_with("git:") {
        Action::Git { args: content[4..].trim().into() }
    } else {
        Action::Complete { summary: content.into() }
    }
}

struct EchoModel;
impl Model for EchoModel {
    fn infer(&self, prompt: &str) -> Action {
        if prompt.contains("done") {
            Action::Complete { summary: "task finished".into() }
        } else {
            Action::Shell { command: "echo hello from MOTE".into() }
        }
    }
}

trait Capability: Send {
    fn name(&self) -> &str;
    fn invoke(&self, input: &str) -> Result<String, String>;
}

struct ShellCap;
impl Capability for ShellCap {
    fn name(&self) -> &str { "shell" }
    fn invoke(&self, cmd: &str) -> Result<String, String> {
        let output = std::process::Command::new("sh").arg("-c").arg(cmd)
            .output().map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

struct ReadFileCap;
impl Capability for ReadFileCap {
    fn name(&self) -> &str { "read_file" }
    fn invoke(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| e.to_string())
    }
}

struct WriteFileCap;
impl Capability for WriteFileCap {
    fn name(&self) -> &str { "write_file" }
    fn invoke(&self, input: &str) -> Result<String, String> {
        if let Some((path, content)) = input.split_once('|') {
            fs::write(path.trim(), content.trim()).map_err(|e| e.to_string())?;
            Ok(format!("wrote {}", path.trim()))
        } else {
            Err("format: path | content".into())
        }
    }
}

struct ListDirCap;
impl Capability for ListDirCap {
    fn name(&self) -> &str { "list_dir" }
    fn invoke(&self, path: &str) -> Result<String, String> {
        let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
        let mut result = String::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let meta = entry.metadata().ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            result.push_str(&format!("{}{}\n", name, if is_dir { "/" } else { "" }));
        }
        Ok(result.trim().to_string())
    }
}

struct GitCap;
impl Capability for GitCap {
    fn name(&self) -> &str { "git" }
    fn invoke(&self, args: &str) -> Result<String, String> {
        let output = std::process::Command::new("git").args(args.split_whitespace())
            .output().map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() {
            Ok(stdout.trim().to_string())
        } else {
            Err(stderr.trim().to_string())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum State { Running, Completed, Failed, Cancelled }

#[derive(Debug)]
enum Event {
    RunStarted,
    ActionProposed(String),
    ActionExecuted(String),
    ObservationReceived(String),
    RunCompleted,
}

struct Budgets {
    max_iterations: usize,
    iterations: usize,
}

impl Budgets {
    fn new(max: usize) -> Self { Self { max_iterations: max, iterations: 0 } }
    fn exhausted(&self) -> bool { self.iterations >= self.max_iterations }
    fn tick(&mut self) { self.iterations += 1; }
}

struct Runtime<'a> {
    model: &'a dyn Model,
    capabilities: Vec<Box<dyn Capability>>,
    budget: Budgets,
    state: State,
    events: Vec<Event>,
}

impl<'a> Runtime<'a> {
    fn new(model: &'a dyn Model, manifest: &Manifest) -> Self {
        let mut capabilities: Vec<Box<dyn Capability>> = Vec::new();
        for cap_name in &manifest.capabilities {
            match cap_name.as_str() {
                "shell" => capabilities.push(Box::new(ShellCap)),
                "read_file" => capabilities.push(Box::new(ReadFileCap)),
                "write_file" => capabilities.push(Box::new(WriteFileCap)),
                "list_dir" => capabilities.push(Box::new(ListDirCap)),
                "git" => capabilities.push(Box::new(GitCap)),
                _ => {}
            }
        }
        if capabilities.is_empty() {
            capabilities.push(Box::new(ShellCap));
        }
        Self {
            model,
            capabilities,
            budget: Budgets::new(manifest.max_iterations),
            state: State::Running,
            events: vec![Event::RunStarted],
        }
    }

    fn invoke_cap(&mut self, name: &str, input: &str, action_desc: &str, _ctx_prefix: &str, capability_used: &mut bool) -> bool {
        let cap = self.capabilities.iter().find(|c| c.name() == name);
        match cap {
            Some(cap) => match cap.invoke(input) {
                Ok(obs) => {
                    *capability_used = true;
                    self.events.push(Event::ActionExecuted(action_desc.into()));
                    self.events.push(Event::ObservationReceived(obs.clone()));
                    return true;
                }
                Err(e) => {
                    self.state = State::Failed;
                    self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                    return false;
                }
            },
            None => {
                self.events.push(Event::ObservationReceived(format!("error: {} capability not available", name)));
                return true; // not a fatal error, just unavailable
            }
        }
    }

    fn run(&mut self, task: &str) -> State {
        let mut context = task.to_string();
        let mut last_action: Option<Action> = None;
        let mut repeat_count = 0;
        let mut capability_used = false;
        while self.state == State::Running && !self.budget.exhausted() {
            self.budget.tick();
            let action = self.model.infer(&context);
            let action_desc = format!("{:?}", action);
            self.events.push(Event::ActionProposed(action_desc.clone()));
            // Repetition detection
            if last_action.as_ref() == Some(&action) {
                repeat_count += 1;
                if repeat_count >= 3 {
                    self.events.push(Event::ObservationReceived("error: action repeated 3 times, stopping".into()));
                    self.state = State::Failed;
                    break;
                }
            } else {
                repeat_count = 0;
            }
            last_action = Some(action.clone());
            match action {
                Action::Shell { command } => {
                    let desc = format!("shell {}", command);
                    if self.invoke_cap("shell", &command, &desc, "", &mut capability_used) {
                        // observation already pushed; update context
                        let obs = self.events.last().map(|e| match e {
                            Event::ObservationReceived(s) => s.clone(),
                            _ => String::new(),
                        }).unwrap_or_default();
                        context = format!("{}\nObservation: {}", context, obs);
                    } else {
                        break;
                    }
                }
                Action::ReadFile { path } => {
                    let desc = format!("read_file {}", path);
                    if self.invoke_cap("read_file", &path, &desc, &format!("File {}:", path), &mut capability_used) {
                        let obs = self.events.last().map(|e| match e {
                            Event::ObservationReceived(s) => s.clone(),
                            _ => String::new(),
                        }).unwrap_or_default();
                        context = format!("{}\nFile {}:\n{}", context, path, obs);
                    } else {
                        break;
                    }
                }
                Action::WriteFile { path, content } => {
                    let desc = format!("write_file {}", path);
                    let input = format!("{}|{}", path, content);
                    if self.invoke_cap("write_file", &input, &desc, "", &mut capability_used) {
                        // observation already pushed
                    } else {
                        break;
                    }
                }
                Action::ListDir { path } => {
                    let desc = format!("list_dir {}", path);
                    if self.invoke_cap("list_dir", &path, &desc, &format!("Dir {}:", path), &mut capability_used) {
                        let obs = self.events.last().map(|e| match e {
                            Event::ObservationReceived(s) => s.clone(),
                            _ => String::new(),
                        }).unwrap_or_default();
                        context = format!("{}\nDir {}:\n{}", context, path, obs);
                    } else {
                        break;
                    }
                }
                Action::Git { args } => {
                    let desc = format!("git {}", args);
                    if self.invoke_cap("git", &args, &desc, "Git output:", &mut capability_used) {
                        let obs = self.events.last().map(|e| match e {
                            Event::ObservationReceived(s) => s.clone(),
                            _ => String::new(),
                        }).unwrap_or_default();
                        context = format!("{}\nGit output: {}", context, obs);
                    } else {
                        break;
                    }
                }
                Action::Complete { summary } => {
                    // Must use at least one capability before completing
                    if !capability_used {
                        self.events.push(Event::ObservationReceived("error: use at least one capability before completing".into()));
                        continue;
                    }
                    self.events.push(Event::ObservationReceived(summary));
                    self.state = State::Completed;
                    self.events.push(Event::RunCompleted);
                    break;
                }
            }
        }
        if self.budget.exhausted() && self.state == State::Running {
            self.state = State::Failed;
        }
        self.state.clone()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spec_path = args.get(1).map(|s| s.as_str()).unwrap_or("specs/demo.yaml");
    let task = args.get(2).map(|s| s.as_str()).unwrap_or("Say hello");
    let yaml = fs::read_to_string(spec_path)
        .unwrap_or_else(|_| "name: demo\ncapabilities: [shell]\nmax_iterations: 10".into());
    let manifest: Manifest = serde_yaml::from_str(&yaml).expect("invalid manifest");
    println!("MOTE v0.7 — {}", manifest.name);
    println!("Task: {}", task);
    println!("Model: {}", manifest.model);
    println!("Capabilities: {:?}", manifest.capabilities);
    println!("---");
    let model: Box<dyn Model> = match manifest.provider.as_str() {
        "google" => Box::new(GoogleModel::new(&manifest.model, &manifest.auth)),
        "anthropic" => Box::new(AnthropicModel::new(&manifest.model, &manifest.auth)),
        _ => Box::new(CurlModel::new(&manifest.endpoint, &manifest.model, &manifest.auth)),
    };
    let mut rt = Runtime::new(&*model, &manifest);
    let final_state = rt.run(task);
    println!("---");
    for ev in &rt.events { println!("{:?}", ev); }
    println!("---");
    println!("Final state: {:?}", final_state);
    println!("Iterations used: {}/{}", rt.budget.iterations, rt.budget.max_iterations);
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockModel;
    impl Model for MockModel {
        fn infer(&self, _: &str) -> Action {
            Action::Complete { summary: "mock done".into() }
        }
    }
    #[test]
    fn test_runtime_completes() {
        struct TwoStepModel;
        impl Model for TwoStepModel {
            fn infer(&self, prompt: &str) -> Action {
                if prompt.contains("Observation") {
                    Action::Complete { summary: "mock done".into() }
                } else {
                    Action::Shell { command: "echo step1".into() }
                }
            }
        }
        let m = TwoStepModel;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 5,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(),
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("test task");
        assert_eq!(state, State::Completed);
        assert_eq!(rt.budget.iterations, 2);
    }
    #[test]
    fn test_budget_exhaustion() {
        struct LoopForever;
        impl Model for LoopForever {
            fn infer(&self, _: &str) -> Action {
                Action::Shell { command: "echo loop".into() }
            }
        }
        let m = LoopForever;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 3,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(),
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("loop");
        assert_eq!(state, State::Failed);
        assert_eq!(rt.budget.iterations, 3);
    }
    #[test]
    fn test_capability_registry() {
        let m = MockModel;
        let manifest = Manifest {
            name: "t".into(),
            capabilities: vec!["shell".into(), "read_file".into(), "git".into()],
            max_iterations: 5,
            model: "test".into(),
            endpoint: "http://localhost".into(),
            auth: String::new(),
            provider: String::new(),
        };
        let rt = Runtime::new(&m, &manifest);
        assert_eq!(rt.capabilities.len(), 3);
    }
    #[test]
    fn test_repetition_detection() {
        struct RepeatModel;
        impl Model for RepeatModel {
            fn infer(&self, _: &str) -> Action {
                Action::Shell { command: "echo same".into() }
            }
        }
        let m = RepeatModel;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 10,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(),
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("repeat");
        assert_eq!(state, State::Failed);
        assert_eq!(rt.budget.iterations, 4); // 3rd repeat = 4th iteration
    }
}
