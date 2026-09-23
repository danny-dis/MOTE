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
    #[serde(default = "default_shell_backend")]
    shell_backend: String,
    #[serde(default)]
    output_file: Option<String>,
}

fn default_provider() -> String { "openai".into() }
fn default_shell_backend() -> String { "auto".into() }
fn default_max_iters() -> usize { 20 }
fn default_model() -> String { "groq/compound-mini".into() }
fn default_endpoint() -> String { "https://api.groq.com/openai/v1/chat/completions".into() }
fn default_auth() -> String { String::new() }

const MOTE_SYSTEM_PROMPT: &str = "\
You are MOTE. Use capabilities to gather info, then complete.
RULES:
1. First turn: MUST use a capability (shell, read_file, write_file, list_dir, git). NEVER complete: on turn 1.
2. Respond with EXACTLY ONE line. No markdown, no extra text.
3. Formats: shell: <cmd> | read_file: <path> | write_file: <path> | <content> | list_dir: <path> | git: <args> | complete: <summary>
4. YAGNI: pick the SINGLE simplest action. One precise action beats a multi-step guess.
After observing results, use complete: <summary> to finish.";

#[derive(Debug, PartialEq, Clone)]
enum Action {
    Shell { command: String },
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    ListDir { path: String },
    Git { args: String },
    Complete { summary: String },
    Retry,
}

trait Model {
    fn infer(&self, prompt: &str) -> Action;
}

#[derive(Clone, Copy)]
enum RequestFormat { OpenAI, Gemini, Anthropic }

struct HttpModel {
    endpoint: String,
    model: String,
    auth: String,
    auth_in_url: bool,
    format: RequestFormat,
    response_path: String,
}

impl HttpModel {
    fn new(endpoint: &str, model: &str, auth: &str, auth_in_url: bool, format: RequestFormat, response_path: &str) -> Self {
        Self { endpoint: endpoint.into(), model: model.into(), auth: auth.into(), auth_in_url, format, response_path: response_path.into() }
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let mut cmd = Command::new("curl");
        cmd.arg("-s").arg("-X").arg("POST").arg("-H").arg("Content-Type: application/json");
        if self.auth_in_url {
            let url = self.endpoint.replace("{model}", &self.model).replace("{auth}", &self.auth);
            cmd.arg("-d").arg(body).arg(&url);
        } else {
            cmd.arg("-H").arg(format!("Authorization: Bearer {}", self.auth)).arg("-d").arg(body).arg(&self.endpoint);
        }
        let output = cmd.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

impl Model for HttpModel {
    fn infer(&self, prompt: &str) -> Action {
        let system = MOTE_SYSTEM_PROMPT;
        let body = match self.format {
            RequestFormat::OpenAI => format!(
                r#"{{"model":"{}","messages":[{{"role":"system","content":"{}"}},{{"role":"user","content":"{}"}}],"max_tokens":200}}"#,
                self.model, system.replace('"', "\\\"").replace('\n', " "), prompt.replace('"', "\\\"").replace('\n', " ")
            ),
            RequestFormat::Gemini => format!(
                r#"{{"contents":[{{"parts":[{{"text":"{}"}}]}},{{"parts":[{{"text":"{}"}}]}}]}}"#,
                system.replace('"', "\\\"").replace('\n', " "), prompt.replace('"', "\\\"").replace('\n', " ")
            ),
            RequestFormat::Anthropic => format!(
                r#"{{"model":"{}","max_tokens":200,"system":"{}","messages":[{{"role":"user","content":"{}"}}]}}"#,
                self.model, system.replace('"', "\\\"").replace('\n', " "), prompt.replace('"', "\\\"").replace('\n', " ")
            ),
        };
        match self.post(&body) {
            Ok(text) => {
                let content = serde_json::from_str::<serde_json::Value>(&text)
                    .ok()
                    .and_then(|v| v.pointer(&self.response_path).cloned())
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default();
                parse_action(&content)
            }
            Err(e) => Action::Complete { summary: format!("model error: {}", e) },
        }
    }
}

fn parse_action(content: &str) -> Action {
    let trimmed = content.trim();
    if trimmed.is_empty() { return Action::Retry; }
    let lower = trimmed.to_lowercase();
    if lower.starts_with("complete:") {
        let summary = trimmed[9..].trim();
        if summary.is_empty() { return Action::Retry; }
        Action::Complete { summary: summary.into() }
    } else if lower.starts_with("shell:") {
        Action::Shell { command: trimmed[6..].trim().into() }
    } else if lower.starts_with("read_file:") {
        Action::ReadFile { path: trimmed[10..].trim().into() }
    } else if lower.starts_with("write_file:") {
        let rest = &trimmed[11..];
        if let Some((path, content_text)) = rest.split_once('|') {
            Action::WriteFile { path: path.trim().into(), content: content_text.trim().into() }
        } else {
            Action::Complete { summary: "write_file format: path | content".into() }
        }
    } else if lower.starts_with("list_dir:") {
        Action::ListDir { path: trimmed[9..].trim().into() }
    } else if lower.starts_with("git:") {
        Action::Git { args: trimmed[4..].trim().into() }
    } else {
        Action::Complete { summary: trimmed.into() }
    }
}

trait Capability: Send {
    fn name(&self) -> &str;
    fn invoke(&self, input: &str) -> Result<String, String>;
}

struct ShellCap { backend: String }

impl ShellCap {
    fn new(backend: &str) -> Self {
        let backend = match backend {
            "auto" => if cfg!(windows) { "cmd".into() } else { "sh".into() },
            other => other.into(),
        };
        Self { backend }
    }
}

impl Capability for ShellCap {
    fn name(&self) -> &str { "shell" }
    fn invoke(&self, cmd: &str) -> Result<String, String> {
        let mut command = match self.backend.as_str() {
            "cmd" => { let mut c = Command::new("cmd"); c.arg("/c"); c }
            "powershell" => { let mut c = Command::new("powershell"); c.arg("-Command"); c }
            _ => { let mut c = Command::new("sh"); c.arg("-c"); c }
        };
        command.arg(cmd);
        let output = command.output().map_err(|e| e.to_string())?;
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
        let output = Command::new("git").args(args.split_whitespace()).output().map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.success() { Ok(stdout.trim().to_string()) } else { Err(stderr.trim().to_string()) }
    }
}

#[derive(Debug, PartialEq, Clone)]
enum State { Running, Completed, Failed }

struct Budgets { max_iterations: usize, iterations: usize }

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
    trace: Vec<String>,
    last_obs: String,
    output_file: Option<String>,
}

impl<'a> Runtime<'a> {
    fn new(model: &'a dyn Model, manifest: &Manifest) -> Self {
        let mut capabilities: Vec<Box<dyn Capability>> = Vec::new();
        for cap_name in &manifest.capabilities {
            match cap_name.as_str() {
                "shell" => capabilities.push(Box::new(ShellCap::new(&manifest.shell_backend))),
                "read_file" => capabilities.push(Box::new(ReadFileCap)),
                "write_file" => capabilities.push(Box::new(WriteFileCap)),
                "list_dir" => capabilities.push(Box::new(ListDirCap)),
                "git" => capabilities.push(Box::new(GitCap)),
                _ => {}
            }
        }
        if capabilities.is_empty() {
            capabilities.push(Box::new(ShellCap::new(&manifest.shell_backend)));
        }
        Self {
            model, capabilities,
            budget: Budgets::new(manifest.max_iterations),
            state: State::Running,
            trace: vec!["run started".into()],
            last_obs: String::new(),
            output_file: manifest.output_file.clone(),
        }
    }

    fn invoke_cap(&mut self, name: &str, input: &str, action_desc: &str, capability_used: &mut bool) -> bool {
        let cap = self.capabilities.iter().find(|c| c.name() == name);
        match cap {
            Some(cap) => match cap.invoke(input) {
                Ok(obs) => {
                    *capability_used = true;
                    self.last_obs = obs.clone();
                    self.trace.push(format!("executed: {}", action_desc));
                    self.trace.push(format!("observation: {}", obs));
                    true
                }
                Err(e) => {
                    self.state = State::Failed;
                    self.last_obs = format!("error: {}", e);
                    self.trace.push(self.last_obs.clone());
                    false
                }
            },
            None => {
                self.last_obs = format!("error: {} capability not available", name);
                self.trace.push(self.last_obs.clone());
                true
            }
        }
    }

    fn maybe_hint_complete(&self, context: &mut String) {
        if !context.contains("HINT: You have used a capability. You may now call complete: <summary> to finish.") {
            context.push_str("\nHINT: You have used a capability. You may now call complete: <summary> to finish.");
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
            self.trace.push(format!("proposed: {:?}", action));
            if last_action.as_ref() == Some(&action) {
                repeat_count += 1;
                if repeat_count >= 3 {
                    self.trace.push("error: action repeated 3 times, stopping".into());
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
                    if self.invoke_cap("shell", &command, &desc, &mut capability_used) {
                        context = format!("{}\nObservation: {}", context, self.last_obs);
                        self.maybe_hint_complete(&mut context);
                    } else { break; }
                }
                Action::ReadFile { path } => {
                    let desc = format!("read_file {}", path);
                    if self.invoke_cap("read_file", &path, &desc, &mut capability_used) {
                        context = format!("{}\nFile {}:\n{}", context, path, self.last_obs);
                        self.maybe_hint_complete(&mut context);
                    } else { break; }
                }
                Action::WriteFile { path, content } => {
                    let desc = format!("write_file {}", path);
                    let input = format!("{}|{}", path, content);
                    if !self.invoke_cap("write_file", &input, &desc, &mut capability_used) { break; }
                }
                Action::ListDir { path } => {
                    let desc = format!("list_dir {}", path);
                    if self.invoke_cap("list_dir", &path, &desc, &mut capability_used) {
                        context = format!("{}\nDir {}:\n{}", context, path, self.last_obs);
                        self.maybe_hint_complete(&mut context);
                    } else { break; }
                }
                Action::Git { args } => {
                    let desc = format!("git {}", args);
                    if self.invoke_cap("git", &args, &desc, &mut capability_used) {
                        context = format!("{}\nGit output: {}", context, self.last_obs);
                        self.maybe_hint_complete(&mut context);
                    } else { break; }
                }
                Action::Complete { summary } => {
                    if !capability_used {
                        self.trace.push("error: use at least one capability before completing".into());
                        continue;
                    }
                    self.trace.push(format!("observation: {}", summary));
                    if let Some(path) = &self.output_file {
                        if let Err(e) = fs::write(path, &summary) {
                            self.trace.push(format!("error writing output_file: {}", e));
                        } else {
                            self.trace.push(format!("wrote output_file: {}", path));
                        }
                    }
                    self.state = State::Completed;
                    self.trace.push("run completed".into());
                    break;
                }
                Action::Retry => {
                    self.trace.push("error: empty response from model, retrying".into());
                    continue;
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
    println!("MOTE v0.9 — {}", manifest.name);
    println!("Task: {}", task);
    println!("Model: {}", manifest.model);
    println!("Capabilities: {:?}", manifest.capabilities);
    println!("---");
    let model: Box<dyn Model> = match manifest.provider.as_str() {
        "google" => Box::new(HttpModel::new(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={auth}",
            &manifest.model, &manifest.auth, true, RequestFormat::Gemini, "/candidates/0/content/parts/0/text",
        )),
        "anthropic" => Box::new(HttpModel::new(
            "https://api.anthropic.com/v1/messages",
            &manifest.model, &manifest.auth, false, RequestFormat::Anthropic, "/content/0/text",
        )),
        _ => Box::new(HttpModel::new(
            &manifest.endpoint, &manifest.model, &manifest.auth, false, RequestFormat::OpenAI, "/choices/0/message/content",
        )),
    };
    let mut rt = Runtime::new(&*model, &manifest);
    let final_state = rt.run(task);
    println!("---");
    for line in &rt.trace { println!("{}", line); }
    println!("---");
    println!("Final state: {:?}", final_state);
    println!("Iterations used: {}/{}", rt.budget.iterations, rt.budget.max_iterations);
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MockModel;
    impl Model for MockModel {
        fn infer(&self, _: &str) -> Action { Action::Complete { summary: "mock done".into() } }
    }
    #[test]
    fn test_runtime_completes() {
        struct TwoStepModel;
        impl Model for TwoStepModel {
            fn infer(&self, prompt: &str) -> Action {
                if prompt.contains("Observation") { Action::Complete { summary: "mock done".into() } }
                else { Action::Shell { command: "echo step1".into() } }
            }
        }
        let m = TwoStepModel;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 5,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(), shell_backend: "sh".into(), output_file: None,
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
            fn infer(&self, _: &str) -> Action { Action::Shell { command: "echo loop".into() } }
        }
        let m = LoopForever;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 3,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(), shell_backend: "sh".into(), output_file: None,
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
            name: "t".into(), capabilities: vec!["shell".into(), "read_file".into(), "git".into()],
            max_iterations: 5, model: "test".into(), endpoint: "http://localhost".into(),
            auth: String::new(), provider: String::new(), shell_backend: "sh".into(), output_file: None,
        };
        let rt = Runtime::new(&m, &manifest);
        assert_eq!(rt.capabilities.len(), 3);
    }
    #[test]
    fn test_repetition_detection() {
        struct RepeatModel;
        impl Model for RepeatModel {
            fn infer(&self, _: &str) -> Action { Action::Shell { command: "echo same".into() } }
        }
        let m = RepeatModel;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 10,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(), shell_backend: "sh".into(), output_file: None,
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("repeat");
        assert_eq!(state, State::Failed);
        assert_eq!(rt.budget.iterations, 4);
    }
    #[test]
    fn test_empty_content_retries() {
        struct EmptyModel;
        impl Model for EmptyModel {
            fn infer(&self, prompt: &str) -> Action {
                if prompt.contains("Observation") { Action::Complete { summary: "done".into() } }
                else { Action::Shell { command: "echo step".into() } }
            }
        }
        let m = EmptyModel;
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 5,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(), shell_backend: "sh".into(), output_file: None,
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("test empty");
        assert_eq!(state, State::Completed);
        assert_eq!(rt.budget.iterations, 2);
    }
    #[test]
    fn test_output_file_written() {
        struct WriteModel;
        impl Model for WriteModel {
            fn infer(&self, prompt: &str) -> Action {
                if prompt.contains("Observation") { Action::Complete { summary: "report content here".into() } }
                else { Action::Shell { command: "echo hi".into() } }
            }
        }
        let m = WriteModel;
        let out_path = "test_output_file.txt".to_string();
        let manifest = Manifest {
            name: "t".into(), capabilities: vec!["shell".into()], max_iterations: 5,
            model: "test".into(), endpoint: "http://localhost".into(), auth: String::new(),
            provider: String::new(), shell_backend: "sh".into(), output_file: Some(out_path.clone()),
        };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("test output");
        assert_eq!(state, State::Completed);
        let written = fs::read_to_string(&out_path).expect("output file not written");
        assert_eq!(written, "report content here");
        fs::remove_file(&out_path).ok();
    }
}
