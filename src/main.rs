use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    name: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default = "default_max_iters")]
    max_iterations: usize,
}

fn default_max_iters() -> usize { 20 }

#[derive(Debug)]
enum Action {
    Shell { command: String },
    Complete { summary: String },
}

trait Model {
    fn infer(&self, prompt: &str) -> Action;
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
        Self {
            model,
            capabilities: vec![Box::new(ShellCap)],
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
                    match cap.unwrap().invoke(&command) {
                        Ok(obs) => {
                            self.events.push(Event::ActionExecuted(command));
                            self.events.push(Event::ObservationReceived(obs.clone()));
                            context = format!("{}\nObservation: {}", context, obs);
                            if obs.contains("hello") {
                                self.state = State::Completed;
                                self.events.push(Event::RunCompleted);
                                break;
                            }
                        }
                        Err(e) => {
                            self.state = State::Failed;
                            self.events.push(Event::ObservationReceived(format!("error: {}", e)));
                            break;
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
    println!("MOTE v0.1 — {}", manifest.name);
    println!("Task: {}", task);
    println!("---");
    let model = EchoModel;
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
        let manifest = Manifest { name: "t".into(), capabilities: vec![], max_iterations: 5 };
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
        let manifest = Manifest { name: "t".into(), capabilities: vec![], max_iterations: 3 };
        let mut rt = Runtime::new(&m, &manifest);
        let state = rt.run("loop");
        assert_eq!(state, State::Failed);
        assert_eq!(rt.budget.iterations, 3);
    }
}
