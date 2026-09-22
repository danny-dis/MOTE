use crate::decision::DecisionRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Shell { command: String },
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    ListDir { path: String },
    Git { args: String },
    Decision { request: DecisionRequest },
    Complete { summary: String },
}
