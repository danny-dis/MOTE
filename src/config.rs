use serde::Deserialize;
use std::path::{Path, PathBuf};

fn default_iterations() -> usize {
    20
}
fn default_max_per_tool() -> usize {
    5
}
fn default_runtime_seconds() -> u64 {
    120
}
fn default_command_seconds() -> u64 {
    30
}
fn default_http_seconds() -> u64 {
    30
}
fn default_max_output_bytes() -> usize {
    65_536
}
fn default_workspace() -> PathBuf {
    PathBuf::from(".")
}
fn default_jev_model() -> String {
    "jev-latest".to_owned()
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    #[serde(default)]
    pub auth_env: Option<String>,
    #[serde(default = "default_http_seconds")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ShellConfig {
    #[serde(default)]
    pub allowed_programs: Vec<String>,
    #[serde(default)]
    pub unsafe_shell: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DecisionConfig {
    pub provider: String,
    pub endpoint: String,
    #[serde(default = "default_jev_model")]
    pub model: String,
    #[serde(default)]
    pub auth_env: Option<String>,
    #[serde(default)]
    pub fallback: Option<String>,
    #[serde(default = "default_http_seconds")]
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    #[serde(default = "default_workspace")]
    pub workspace: PathBuf,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default = "default_iterations")]
    pub max_iterations: usize,
    #[serde(default = "default_max_per_tool")]
    pub max_per_tool: usize,
    #[serde(default = "default_runtime_seconds")]
    pub max_runtime_seconds: u64,
    #[serde(default = "default_command_seconds")]
    pub command_timeout_seconds: u64,
    #[serde(default = "default_max_output_bytes")]
    pub max_output_bytes: usize,
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub output_file: Option<PathBuf>,
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    #[serde(default)]
    pub decision: Option<DecisionConfig>,
}

impl Manifest {
    pub fn from_yaml(input: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(input)
    }

    pub fn unsafe_shell_enabled(&self) -> bool {
        self.shell.unsafe_shell
    }

    pub fn workspace_from(&self, manifest_path: &Path) -> PathBuf {
        if self.workspace.is_absolute() {
            self.workspace.clone()
        } else {
            manifest_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&self.workspace)
        }
    }
}
