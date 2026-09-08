use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
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
}

fn default_max_iters() -> usize { 20 }
fn default_model() -> String { "~deepseek/deepseek-v4-flash-latest".into() }
fn default_endpoint() -> String { "http://127.0.0.1:47113/v1/chat/completions".into() }

#[derive(Debug)]
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

// ── Raw TCP HTTP client for DMR-X ────────────────────────────────────
struct DmrxModel {
    host: String,
    port: u16,
    path: String,
    model: String,
    auth: String,
}

impl DmrxModel {
    fn new(endpoint: &str, model: &str, auth: &str) -> Self {
        let without_scheme = endpoint.strip_prefix("http://").unwrap_or(endpoint);
        let (host_port, path) = without_scheme.split_once('/').unwrap_or((without_scheme, ""));
        let (host, port) = host_port.split_once(':').unwrap_or((host_port, "80"));
        Self {
            host: host.into(),
            port: port.parse().unwrap_or(80),
            path: format!("/{}", path),
            model: model.into(),
            auth: auth.into(),
        }
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
        let req = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nAuthorization: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.path, self.host, self.auth, body.len(), body
        );
        stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;
        let mut resp = String::new();
        stream.read_to_string(&mut resp).map_err(|e| e.to_string())?;
        if let Some(idx) = resp.find("\r\n\r\n") {
            Ok(resp[idx + 4..].to_string())
        } else {
            Ok(resp)
        }
    }
}

impl Model for DmrxModel {
    fn infer(&self, prompt: &str) -> Action {
        let system = "You are MOTE. Respond with EXACTLY ONE line in one of these formats:\n\
                      shell: <command>\n\
                      read_file: <path>\n\
                      write_file: <path> | <content>\n\
                      list_dir: <path>\n\
                      git: <args>\n\
                      complete: <summary>\n\
                      Nothing else. No explanations.";
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
            Err(e) => Action::Complete { summary: format!("model error: {}", e) },
        }
    }
}

// ── EchoModel stub (for tests) ───────────────────────────────────────
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

    fn run(&mut self, task: &str) -> State {
        let mut context = task.to_string();
        while self.state == State::Running && !self.budget.exhausted() {
            self.budget.tick();
            let action = self.model.infer(&context);
            let action_desc = format!("{:?}", action);
            self.events.push(Event::ActionProposed(action_desc));
            match action {
                Action::Shell { command } => {
                    let cap = self.capabilities.iter().find(|c| c.name() == "shell");
                    match cap {
                        Some(cap) => match cap.invoke(&command) {
                            Ok(obs) => {
                                self.events.push(Event::ActionExecuted(command));
                                self.events.push(Event::ObservationReceived(obs.clone()));
                                context = format!("{}\nObservation: {}", context, obs);
                            }
                            Err(e) => {
                                self.state = State::Failed;
                                self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                                break;
                            }
                        },
                        None => {
                            self.events.push(Event::ObservationReceived("error: shell capability not available".into()));
                        }
                    }
                }
                Action::ReadFile { path } => {
                    let cap = self.capabilities.iter().find(|c| c.name() == "read_file");
                    match cap {
                        Some(cap) => match cap.invoke(&path) {
                            Ok(content) => {
                                self.events.push(Event::ActionExecuted(format!("read_file {}", path)));
                                self.events.push(Event::ObservationReceived(content.clone()));
                                context = format!("{}\nFile {}:\n{}", context, path, content);
                            }
                            Err(e) => {
                                self.state = State::Failed;
                                self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                                break;
                            }
                        },
                        None => {
                            self.events.push(Event::ObservationReceived("error: read_file capability not available".into()));
                        }
                    }
                }
                Action::WriteFile { path, content } => {
                    let cap = self.capabilities.iter().find(|c| c.name() == "write_file");
                    match cap {
                        Some(cap) => {
                            let input = format!("{}|{}", path, content);
                            match cap.invoke(&input) {
                                Ok(obs) => {
                                    self.events.push(Event::ActionExecuted(format!("write_file {}", path)));
                                    self.events.push(Event::ObservationReceived(obs));
                                }
                                Err(e) => {
                                    self.state = State::Failed;
                                    self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                                    break;
                                }
                            }
                        },
                        None => {
                            self.events.push(Event::ObservationReceived("error: write_file capability not available".into()));
                        }
                    }
                }
                Action::ListDir { path } => {
                    let cap = self.capabilities.iter().find(|c| c.name() == "list_dir");
                    match cap {
                        Some(cap) => match cap.invoke(&path) {
                            Ok(entries) => {
                                self.events.push(Event::ActionExecuted(format!("list_dir {}", path)));
                                self.events.push(Event::ObservationReceived(entries.clone()));
                                context = format!("{}\nDir {}:\n{}", context, path, entries);
                            }
                            Err(e) => {
                                self.state = State::Failed;
                                self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                                break;
                            }
                        },
                        None => {
                            self.events.push(Event::ObservationReceived("error: list_dir capability not available".into()));
                        }
                    }
                }
                Action::Git { args } => {
                    let cap = self.capabilities.iter().find(|c| c.name() == "git");
                    match cap {
                        Some(cap) => match cap.invoke(&args) {
                            Ok(output) => {
                                self.events.push(Event::ActionExecuted(format!("git {}", args)));
                                self.events.push(Event::ObservationReceived(output.clone()));
                                context = format!("{}\nGit output: {}", context, output);
                            }
                            Err(e) => {
                                self.state = State::Failed;
                                self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                                break;
                            }
                        },
                        None => {
                            self.events.push(Event::ObservationReceived("error: git capability not available".into()));
                        }
                    }
                }
                Action::Complete { summary } => {
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
    println!("MOTE v0.3 — {}", manifest.name);
    println!("Task: {}", task);
    println!("Model: {}", manifest.model);
    println!("Capabilities: {:?}", manifest.capabilities);
    println!("---");
    let model = DmrxModel::new(&manifest.endpoint, &manifest.model, "Bearer g");
    let mut rt = Runtime::new(&model, &manifest);
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
        let m = MockModel;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 5,
            model: "test".into(), endpoint: "http://localhost".into(),
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("test task");
        assert_eq!(state, State::Completed);
        assert_eq!(rt.budget.iterations, 1);
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
            model: "test".into(), endpoint: "http://localhost".into(),
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
        };
        let rt = Runtime::new(&m, &manifest);
        assert_eq!(rt.capabilities.len(), 3);
    }
}
