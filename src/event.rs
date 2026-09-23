use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    RunStarted,
    ActionProposed { action: String },
    ActionExecuted { capability: String },
    ObservationReceived { text: String },
    RunCompleted { state: String },
    Error { message: String },
}

impl Event {
    pub fn json_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
