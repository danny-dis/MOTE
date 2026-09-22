use crate::action::Action;
use crate::capability::{CapabilityRegistry, WorkspaceFs};
use crate::config::Manifest;
use crate::decision::{
    self, deterministic_fallback, DecisionProvider, DecisionRequest, DecisionResult, DecisionUsage,
};
use crate::event::Event;
use crate::model::Model;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub struct ShellExecutor {
    pub workspace: std::path::PathBuf,
    pub allowed: std::collections::BTreeSet<String>,
    pub timeout: Duration,
    pub max_output: usize,
    pub unsafe_shell: bool,
}

fn drain_limited<R: std::io::Read + Send + 'static>(
    mut reader: R,
    limit: usize,
) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut retained = Vec::with_capacity(limit.min(8_192));
        let mut buffer = [0_u8; 8_192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(read) => {
                    let remaining = limit.saturating_sub(retained.len());
                    retained.extend_from_slice(&buffer[..read.min(remaining)]);
                }
            }
        }
        retained
    })
}

impl ShellExecutor {
    pub fn run(&self, command: &str, token: &CancellationToken) -> Result<String, String> {
        if token.is_cancelled() {
            return Err("cancelled".into());
        }
        let has_operator = ["&&", "||", ";", "|", ">", "<", "$", "`"]
            .iter()
            .any(|operator| command.contains(operator));
        if has_operator && !self.unsafe_shell {
            return Err("shell operators require unsafe_shell".into());
        }

        let words = shell_words::split(command).map_err(|error| error.to_string())?;
        let executable = words.first().ok_or("empty command")?;
        let executable_name = std::path::Path::new(executable)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(executable);
        let is_bare_name = !executable.contains(['/', '\\', ':']);
        let normalize = |value: &str| {
            if cfg!(windows) {
                value.to_ascii_lowercase()
            } else {
                value.to_owned()
            }
        };
        if !(is_bare_name && self.allowed.contains(&normalize(executable_name)))
            && !self.allowed.contains(&normalize(executable))
        {
            return Err("executable is not allowlisted".into());
        }
        if !self.unsafe_shell && matches!(normalize(executable_name).as_str(), "git" | "git.exe") {
            if !matches!(
                words.get(1).map(String::as_str),
                Some("status" | "log" | "diff" | "show" | "rev-parse" | "ls-files")
            ) {
                return Err("git subcommand not permitted".into());
            }
            if words.iter().skip(2).any(|argument| {
                if argument.contains('!') {
                    return true;
                }
                if !argument.starts_with('-') {
                    return false;
                }
                match words[1].as_str() {
                    "status" => !matches!(
                        argument.as_str(),
                        "--short"
                            | "--porcelain"
                            | "--branch"
                            | "--untracked-files=no"
                            | "--untracked-files=normal"
                            | "--untracked-files=all"
                    ),
                    "log" => {
                        !(matches!(
                            argument.as_str(),
                            "--oneline" | "--stat" | "--no-patch" | "--name-only" | "--name-status"
                        ) || argument.strip_prefix('-').is_some_and(|n| {
                            !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())
                        }))
                    }
                    "diff" => !matches!(
                        argument.as_str(),
                        "--stat"
                            | "--name-only"
                            | "--name-status"
                            | "--cached"
                            | "--check"
                            | "--quiet"
                    ),
                    "show" => !matches!(
                        argument.as_str(),
                        "--stat" | "--oneline" | "--no-patch" | "--name-only"
                    ),
                    "rev-parse" => !matches!(
                        argument.as_str(),
                        "--show-toplevel" | "--is-inside-work-tree" | "--abbrev-ref"
                    ),
                    "ls-files" => !matches!(
                        argument.as_str(),
                        "--cached" | "--others" | "--exclude-standard"
                    ),
                    _ => true,
                }
            }) {
                return Err("git option not permitted".into());
            }
        }

        let mut process = if self.unsafe_shell {
            if cfg!(windows) {
                let mut process = Command::new("cmd");
                process.args(["/C", command]);
                process
            } else {
                let mut process = Command::new("sh");
                process.args(["-c", command]);
                process
            }
        } else {
            let mut process = Command::new(executable);
            if !self.unsafe_shell
                && matches!(normalize(executable_name).as_str(), "git" | "git.exe")
            {
                process.env_clear();
                for key in ["PATH", "SystemRoot", "WINDIR", "TEMP", "TMP"] {
                    if let Some(value) = std::env::var_os(key) {
                        process.env(key, value);
                    }
                }
                process.env("GIT_CONFIG_NOSYSTEM", "1");
                process.env(
                    "GIT_CONFIG_GLOBAL",
                    if cfg!(windows) { "NUL" } else { "/dev/null" },
                );
                process.env("GIT_OPTIONAL_LOCKS", "0");
                process.env("GIT_PAGER", "");
                process.env("GIT_TERMINAL_PROMPT", "0");
                process.args([
                    "-c",
                    "core.fsmonitor=false",
                    "-c",
                    "core.hooksPath=/dev/null",
                    "-c",
                    "log.showSignature=false",
                    "-c",
                    "show.signature=false",
                ]);
                process.arg(&words[1]);
                if matches!(words[1].as_str(), "log" | "diff" | "show") {
                    process.args(["--no-ext-diff", "--no-textconv"]);
                }
                process.args(&words[2..]);
            } else {
                process.args(&words[1..]);
            }
            process
        };
        let mut child = process
            .current_dir(&self.workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        let stdout = child.stdout.take().ok_or("stdout pipe unavailable")?;
        let stderr = child.stderr.take().ok_or("stderr pipe unavailable")?;
        let stdout_reader = drain_limited(stdout, self.max_output);
        let stderr_reader = drain_limited(stderr, self.max_output);

        let deadline = Instant::now() + self.timeout;
        let status = loop {
            if token.is_cancelled() || Instant::now() >= deadline {
                let cancelled = token.is_cancelled();
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(if cancelled {
                    "cancelled"
                } else {
                    "command timed out"
                }
                .into());
            }
            if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                break status;
            }
            std::thread::sleep(Duration::from_millis(5));
        };

        let mut bytes = stdout_reader
            .join()
            .map_err(|_| "stdout reader failed".to_owned())?;
        bytes.extend(
            stderr_reader
                .join()
                .map_err(|_| "stderr reader failed".to_owned())?,
        );
        bytes.truncate(self.max_output);
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if status.success() {
            Ok(text)
        } else {
            Err(format!("command failed: {text}"))
        }
    }
}

pub struct Runtime {
    pub registry: CapabilityRegistry,
    pub workspace: WorkspaceFs,
    pub events: Vec<Event>,
    pub max_iterations: usize,
    pub max_runtime: Duration,
    pub cancellation: CancellationToken,
    pub decision: Option<Box<dyn DecisionProvider>>,
    pub decision_fallback: Option<String>,
    pub shell: Option<ShellExecutor>,
    pub output_file: Option<std::path::PathBuf>,
}
impl Runtime {
    pub fn new(manifest: &Manifest, workspace: WorkspaceFs) -> Result<Self, String> {
        let registry = CapabilityRegistry::from_names(&manifest.capabilities)
            .map_err(|error| error.to_string())?;
        if registry.allows("git") && !registry.allows("shell") {
            return Err("git capability requires shell capability".to_owned());
        }
        let decision = manifest
            .decision
            .as_ref()
            .map(decision::from_config)
            .transpose()?;
        let allowed = manifest
            .shell
            .allowed_programs
            .iter()
            .map(|program| {
                if cfg!(windows) {
                    program.to_ascii_lowercase()
                } else {
                    program.clone()
                }
            })
            .collect::<std::collections::BTreeSet<_>>();
        if registry.allows("git") && !allowed.contains("git") {
            return Err("git capability requires git in shell.allowed_programs".to_owned());
        }
        if registry.allows("shell") && allowed.is_empty() {
            return Err("shell capability requires shell.allowed_programs".to_owned());
        }
        let shell = if registry.allows("shell") || registry.allows("git") {
            Some(ShellExecutor {
                workspace: workspace.root().to_owned(),
                allowed,
                timeout: Duration::from_secs(manifest.command_timeout_seconds),
                max_output: manifest.max_output_bytes,
                unsafe_shell: manifest.shell.unsafe_shell,
            })
        } else {
            None
        };
        Ok(Self {
            registry,
            workspace,
            events: vec![Event::RunStarted],
            max_iterations: manifest.max_iterations,
            max_runtime: Duration::from_secs(manifest.max_runtime_seconds),
            cancellation: CancellationToken::default(),
            decision,
            decision_fallback: manifest
                .decision
                .as_ref()
                .and_then(|config| config.fallback.clone()),
            shell,
            output_file: manifest.output_file.clone(),
        })
    }

    pub fn decide(&self, request: &DecisionRequest) -> Result<DecisionResult, String> {
        if let Some(provider) = &self.decision {
            if let Ok(result) = provider.decide(request) {
                decision::validate_answer(request, &result.answer)?;
                return Ok(result);
            }
        }
        if let crate::decision::DecisionQuestion::Choice { criteria, .. } = &request.question {
            if !self
                .decision_fallback
                .as_deref()
                .is_some_and(|choice| criteria.contains_key(choice))
            {
                return Err(
                    "decision provider failed and no valid fail-closed fallback is configured"
                        .to_owned(),
                );
            }
        }
        Ok(DecisionResult {
            model: "deterministic-fallback".to_owned(),
            answer: deterministic_fallback(request, self.decision_fallback.as_deref()),
            source: "fallback".to_owned(),
            usage: DecisionUsage::default(),
        })
    }
    pub fn validate_action(&self, action: &Action) -> Result<(), String> {
        let cap = match action {
            Action::Shell { .. } => "shell",
            Action::ReadFile { .. } => "read_file",
            Action::WriteFile { .. } => "write_file",
            Action::ListDir { .. } => "list_dir",
            Action::Git { .. } => "git",
            Action::Decision { .. } => "decision",
            Action::Complete { .. } => return Ok(()),
        };
        if self.registry.allows(cap) {
            Ok(())
        } else {
            Err(format!("capability not granted: {cap}"))
        }
    }

    pub fn run(&mut self, model: &dyn Model, task: &str) -> State {
        let started = Instant::now();
        let mut context = task.to_owned();
        let mut previous: Option<Action> = None;
        let mut repeats = 0_usize;
        let mut capability_used = false;

        for _iteration in 0..self.max_iterations {
            if self.cancellation.is_cancelled() {
                self.events.push(Event::RunCompleted {
                    state: "cancelled".to_owned(),
                });
                return State::Cancelled;
            }
            if started.elapsed() >= self.max_runtime {
                self.events.push(Event::Error {
                    message: "runtime deadline exceeded".to_owned(),
                });
                return State::Failed;
            }

            let action = match model.infer(&context) {
                Ok(action) => action,
                Err(error) => {
                    self.events.push(Event::Error { message: error });
                    return State::Failed;
                }
            };
            if previous.as_ref() == Some(&action) {
                repeats += 1;
                if repeats >= 2 {
                    self.events.push(Event::Error {
                        message: "repeated action guard".to_owned(),
                    });
                    return State::Failed;
                }
            } else {
                repeats = 0;
            }
            previous = Some(action.clone());
            if let Err(error) = self.validate_action(&action) {
                self.events.push(Event::Error { message: error });
                return State::Failed;
            }
            self.events.push(Event::ActionProposed {
                action: format!("{action:?}"),
            });

            let (capability, observation) = match action {
                Action::Shell { command } => {
                    let shell = match &self.shell {
                        Some(shell) => shell,
                        None => {
                            self.events.push(Event::Error {
                                message: "shell executor unavailable".to_owned(),
                            });
                            return State::Failed;
                        }
                    };
                    match shell.run(&command, &self.cancellation) {
                        Ok(output) => ("shell", output),
                        Err(error) => {
                            self.events.push(Event::Error { message: error });
                            return if self.cancellation.is_cancelled() {
                                State::Cancelled
                            } else {
                                State::Failed
                            };
                        }
                    }
                }
                Action::ReadFile { path } => match self.workspace.read(&path) {
                    Ok(content) => ("read_file", content),
                    Err(error) => {
                        self.events.push(Event::Error {
                            message: error.to_string(),
                        });
                        return State::Failed;
                    }
                },
                Action::WriteFile { path, content } => {
                    match self.workspace.write(&path, &content) {
                        Ok(()) => ("write_file", format!("wrote {path}")),
                        Err(error) => {
                            self.events.push(Event::Error {
                                message: error.to_string(),
                            });
                            return State::Failed;
                        }
                    }
                }
                Action::ListDir { path } => match self.workspace.list(&path) {
                    Ok(entries) => ("list_dir", entries.join("\n")),
                    Err(error) => {
                        self.events.push(Event::Error {
                            message: error.to_string(),
                        });
                        return State::Failed;
                    }
                },
                Action::Git { args } => {
                    let shell = match &self.shell {
                        Some(shell) => shell,
                        None => {
                            self.events.push(Event::Error {
                                message: "git executor unavailable".to_owned(),
                            });
                            return State::Failed;
                        }
                    };
                    match shell.run(&format!("git {args}"), &self.cancellation) {
                        Ok(output) => ("git", output),
                        Err(error) => {
                            self.events.push(Event::Error { message: error });
                            return State::Failed;
                        }
                    }
                }
                Action::Decision { request } => {
                    let result = match self.decide(&request) {
                        Ok(result) => result,
                        Err(error) => {
                            self.events.push(Event::Error { message: error });
                            return State::Failed;
                        }
                    };
                    let observation = serde_json::to_string(&result)
                        .unwrap_or_else(|_| "decision serialization failed".to_owned());
                    ("decision", observation)
                }
                Action::Complete { summary } => {
                    if !capability_used {
                        context.push_str("\nObservation: use at least one permitted capability before completing");
                        continue;
                    }
                    if let Some(path) = &self.output_file {
                        if !self.registry.allows("write_file") {
                            self.events.push(Event::Error {
                                message: "output_file requires write_file capability".to_owned(),
                            });
                            return State::Failed;
                        }
                        if let Err(error) = self.workspace.write_if_absent(path, &summary) {
                            self.events.push(Event::Error {
                                message: format!("output file write failed: {error}"),
                            });
                            return State::Failed;
                        }
                    }
                    self.events
                        .push(Event::ObservationReceived { text: summary });
                    self.events.push(Event::RunCompleted {
                        state: "completed".to_owned(),
                    });
                    return State::Completed;
                }
            };
            capability_used = true;
            self.events.push(Event::ActionExecuted {
                capability: capability.to_owned(),
            });
            self.events.push(Event::ObservationReceived {
                text: observation.clone(),
            });
            context.push_str("\nObservation: ");
            context.push_str(&observation);
        }

        self.events.push(Event::Error {
            message: "iteration budget exhausted".to_owned(),
        });
        State::Failed
    }

    pub fn run_actions(&mut self, actions: &[Action]) -> State {
        struct SequenceModel(std::sync::Mutex<std::collections::VecDeque<Action>>);
        impl Model for SequenceModel {
            fn infer(&self, _prompt: &str) -> Result<Action, String> {
                self.0
                    .lock()
                    .map_err(|_| "action queue unavailable".to_owned())?
                    .pop_front()
                    .ok_or_else(|| "script exhausted without completion".to_owned())
            }
        }
        let model = SequenceModel(std::sync::Mutex::new(actions.iter().cloned().collect()));
        self.run(&model, "execute scripted actions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Model;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct ScriptedModel(Mutex<VecDeque<Action>>);

    impl Model for ScriptedModel {
        fn infer(&self, _prompt: &str) -> Result<Action, String> {
            self.0
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| "script exhausted".to_owned())
        }
    }

    #[test]
    fn decision_failure_never_defaults_to_allow() {
        use crate::decision::{DecisionAnswer, DecisionQuestion, DecisionRequest};
        use std::collections::BTreeMap;
        let manifest = Manifest::from_yaml("name: gate\ncapabilities: [decision]\n").unwrap();
        let workspace = WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap();
        let mut runtime = Runtime::new(&manifest, workspace).unwrap();
        let request = DecisionRequest {
            id: "safety".to_owned(),
            state: serde_json::json!({}),
            question: DecisionQuestion::Choice {
                instructions: "Choose".to_owned(),
                criteria: BTreeMap::from([
                    ("allow".to_owned(), "allow".to_owned()),
                    ("deny".to_owned(), "deny".to_owned()),
                ]),
            },
        };
        assert!(runtime.decide(&request).is_err());
        runtime.decision_fallback = Some("deny".to_owned());
        assert!(matches!(runtime.decide(&request).unwrap().answer,
            DecisionAnswer::Choice { choice, .. } if choice == "deny"));
    }

    #[test]
    fn decision_provider_cannot_return_unlisted_choice() {
        use crate::decision::{DecisionAnswer, DecisionQuestion, DecisionRequest};
        use std::collections::BTreeMap;
        struct OutOfDomain;
        impl DecisionProvider for OutOfDomain {
            fn decide(&self, _request: &DecisionRequest) -> Result<DecisionResult, String> {
                Ok(DecisionResult {
                    model: "invalid".to_owned(),
                    answer: DecisionAnswer::Choice {
                        choice: "allow".to_owned(),
                        confidence: None,
                        probabilities: BTreeMap::new(),
                    },
                    source: "invalid".to_owned(),
                    usage: DecisionUsage::default(),
                })
            }
        }
        let manifest = Manifest::from_yaml("name: gate\ncapabilities: [decision]\n").unwrap();
        let workspace = WorkspaceFs::new(std::env::current_dir().unwrap()).unwrap();
        let mut runtime = Runtime::new(&manifest, workspace).unwrap();
        runtime.decision = Some(Box::new(OutOfDomain));
        let request = DecisionRequest {
            id: "safety".to_owned(),
            state: serde_json::json!({}),
            question: DecisionQuestion::Choice {
                instructions: "Choose".to_owned(),
                criteria: BTreeMap::from([("deny".to_owned(), "deny".to_owned())]),
            },
        };
        assert!(runtime.decide(&request).is_err());
    }

    #[test]
    fn completion_does_not_write_without_write_file_grant() {
        let root = std::env::temp_dir().join(format!("mote-output-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let manifest = Manifest::from_yaml(
            "name: output\ncapabilities: [list_dir]\noutput_file: report.txt\n",
        )
        .unwrap();
        let mut runtime = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
        let state = runtime.run_actions(&[
            Action::ListDir { path: ".".into() },
            Action::Complete {
                summary: "done".into(),
            },
        ]);
        assert_eq!(state, State::Failed);
        assert!(!root.join("report.txt").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn completion_preserves_existing_report() {
        let root = std::env::temp_dir().join(format!("mote-report-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let manifest = Manifest::from_yaml(
            "name: report\ncapabilities: [write_file]\noutput_file: report.txt\n",
        )
        .unwrap();
        let mut runtime = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
        let state = runtime.run_actions(&[
            Action::WriteFile {
                path: "report.txt".into(),
                content: "full report".into(),
            },
            Action::Complete {
                summary: "done".into(),
            },
        ]);
        assert_eq!(state, State::Completed);
        assert_eq!(
            std::fs::read_to_string(root.join("report.txt")).unwrap(),
            "full report"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn run_actions_executes_writes_and_rejects_empty_scripts() {
        let root = std::env::temp_dir().join(format!("mote-actions-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let manifest = Manifest::from_yaml("name: scripted\ncapabilities: [write_file]\n").unwrap();
        let mut runtime = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
        assert_eq!(runtime.run_actions(&[]), State::Failed);
        let actions = [
            Action::WriteFile {
                path: "result.txt".into(),
                content: "saved".into(),
            },
            Action::Complete {
                summary: "done".into(),
            },
        ];
        assert_eq!(runtime.run_actions(&actions), State::Completed);
        assert_eq!(
            std::fs::read_to_string(root.join("result.txt")).unwrap(),
            "saved"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn runtime_executes_capabilities_emits_events_and_honors_cancellation() {
        let root = std::env::temp_dir().join(format!("mote-runtime-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let manifest = Manifest::from_yaml(
            "name: runtime\ncapabilities: [write_file, read_file]\nmax_iterations: 5\n",
        )
        .unwrap();
        let mut runtime = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
        let model = ScriptedModel(Mutex::new(VecDeque::from([
            Action::WriteFile {
                path: "result.txt".to_owned(),
                content: "verified".to_owned(),
            },
            Action::ReadFile {
                path: "result.txt".to_owned(),
            },
            Action::Complete {
                summary: "done".to_owned(),
            },
        ])));
        assert_eq!(runtime.run(&model, "write and verify"), State::Completed);
        assert_eq!(
            std::fs::read_to_string(root.join("result.txt")).unwrap(),
            "verified"
        );
        assert!(runtime.events.iter().all(|event| event.json_line().is_ok()));

        let mut cancelled = Runtime::new(&manifest, WorkspaceFs::new(&root).unwrap()).unwrap();
        cancelled.cancellation.cancel();
        assert_eq!(cancelled.run(&model, "cancel"), State::Cancelled);
        let _ = std::fs::remove_dir_all(&root);
    }
}
