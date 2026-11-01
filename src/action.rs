use crate::decision::DecisionRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    Shell {
        command: String,
    },
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    ListDir {
        path: String,
    },
    Git {
        args: String,
    },
    Decision {
        request: DecisionRequest,
    },
    Custom {
        name: String,
        input: serde_json::Value,
    },
    Complete {
        summary: String,
    },
}

impl Action {
    pub(crate) fn capability(&self) -> Option<&str> {
        match self {
            Self::Shell { .. } => Some("shell"),
            Self::ReadFile { .. } => Some("read_file"),
            Self::WriteFile { .. } => Some("write_file"),
            Self::ListDir { .. } => Some("list_dir"),
            Self::Git { .. } => Some("git"),
            Self::Decision { .. } => Some("decision"),
            Self::Custom { name, .. } => Some(name),
            Self::Complete { .. } => None,
        }
    }
}
