use crate::config::DecisionConfig;
use crate::http::post_json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub id: String,
    pub state: Value,
    pub question: DecisionQuestion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DecisionQuestion {
    Choice {
        instructions: String,
        criteria: BTreeMap<String, String>,
    },
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
    Noul {
        instructions: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DecisionAnswer {
    Choice {
        choice: String,
        #[serde(default)]
        confidence: Option<f64>,
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
    },
    Score {
        score: f64,
        #[serde(default)]
        confidence: Option<f64>,
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
    },
    Noul {
        noul: f64,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionResult {
    pub model: String,
    pub answer: DecisionAnswer,
    pub source: String,
    #[serde(default)]
    pub usage: DecisionUsage,
}

pub trait DecisionProvider: Send + Sync {
    fn decide(&self, request: &DecisionRequest) -> Result<DecisionResult, String>;
}

pub fn deterministic_fallback(
    request: &DecisionRequest,
    preferred_choice: Option<&str>,
) -> DecisionAnswer {
    match &request.question {
        DecisionQuestion::Choice { criteria, .. } => {
            let preferred = preferred_choice.filter(|choice| criteria.contains_key(*choice));
            let choice = preferred
                .map(str::to_owned)
                .or_else(|| {
                    ["deny", "reject", "block", "escalate"]
                        .iter()
                        .find(|choice| criteria.contains_key(**choice))
                        .map(|choice| (*choice).to_owned())
                })
                .or_else(|| criteria.keys().next().cloned())
                .unwrap_or_default();
            DecisionAnswer::Choice {
                choice,
                confidence: Some(0.0),
                probabilities: BTreeMap::new(),
            }
        }
        DecisionQuestion::Score { .. } => DecisionAnswer::Score {
            score: 0.0,
            confidence: Some(0.0),
            probabilities: BTreeMap::new(),
        },
        DecisionQuestion::Noul { .. } => DecisionAnswer::Noul { noul: 0.0 },
    }
}

pub(crate) fn validate_answer(
    request: &DecisionRequest,
    answer: &DecisionAnswer,
) -> Result<(), String> {
    match (&request.question, answer) {
        (DecisionQuestion::Choice { criteria, .. }, DecisionAnswer::Choice { choice, .. })
            if criteria.contains_key(choice) =>
        {
            Ok(())
        }
        (DecisionQuestion::Score { criteria, .. }, DecisionAnswer::Score { score, .. })
            if !criteria.is_empty() && *score >= 0.0 && *score <= (criteria.len() - 1) as f64 =>
        {
            Ok(())
        }
        (DecisionQuestion::Noul { .. }, DecisionAnswer::Noul { noul })
            if (0.0..=1.0).contains(noul) =>
        {
            Ok(())
        }
        _ => Err("decision answer does not match the requested bounded domain".to_owned()),
    }
}

#[derive(Debug, Deserialize)]
struct JevResponse {
    model: String,
    answers: BTreeMap<String, DecisionAnswer>,
    #[serde(default)]
    usage: DecisionUsage,
}

pub struct JevDecisionAdapter {
    pub endpoint: String,
    pub model: String,
    pub auth_env: Option<String>,
    pub timeout: Duration,
    pub max_response: usize,
}

impl JevDecisionAdapter {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        auth_env: Option<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
            auth_env,
            timeout: Duration::from_secs(30),
            max_response: 1_048_576,
        }
    }

    fn authentication(&self) -> Result<Option<String>, String> {
        self.auth_env
            .as_ref()
            .map(|name| {
                std::env::var(name)
                    .map_err(|_| format!("missing auth environment variable: {name}"))
            })
            .transpose()
    }
}

impl DecisionProvider for JevDecisionAdapter {
    fn decide(&self, request: &DecisionRequest) -> Result<DecisionResult, String> {
        let mut headers = Vec::new();
        if let Some(secret) = self.authentication()? {
            headers.push(("Authorization", format!("Bearer {secret}")));
        }
        let body = json!({
            "model": self.model,
            "state": request.state,
            "questions": {request.id.clone(): request.question.clone()}
        });
        let value = post_json(
            &self.endpoint,
            &headers,
            &body,
            self.timeout,
            self.max_response,
        )?;
        let mut response: JevResponse = serde_json::from_value(value)
            .map_err(|error| format!("invalid Jev response: {error}"))?;
        let answer = response
            .answers
            .remove(&request.id)
            .ok_or_else(|| format!("Jev response missing answer: {}", request.id))?;
        validate_answer(request, &answer)?;
        Ok(DecisionResult {
            model: response.model,
            answer,
            source: "jev".to_owned(),
            usage: response.usage,
        })
    }
}

pub struct DmrXDecisionAdapter {
    pub endpoint: String,
    pub model: String,
    pub auth_env: Option<String>,
    pub timeout: Duration,
    pub max_response: usize,
}

impl DmrXDecisionAdapter {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        auth_env: Option<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            model: model.into(),
            auth_env,
            timeout: Duration::from_secs(30),
            max_response: 1_048_576,
        }
    }
}

impl DecisionProvider for DmrXDecisionAdapter {
    fn decide(&self, request: &DecisionRequest) -> Result<DecisionResult, String> {
        let mut headers = Vec::new();
        if let Some(name) = &self.auth_env {
            let secret = std::env::var(name)
                .map_err(|_| format!("missing auth environment variable: {name}"))?;
            headers.push(("Authorization", format!("Bearer {secret}")));
        }
        let schema = serde_json::to_string(request)
            .map_err(|error| format!("decision serialization failed: {error}"))?;
        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": "Return only one JSON DecisionAnswer matching the requested type and bounded criteria."},
                {"role": "user", "content": schema}
            ],
            "max_tokens": 256,
            "stream": false
        });
        let value = post_json(
            &self.endpoint,
            &headers,
            &body,
            self.timeout,
            self.max_response,
        )?;
        let content = value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .ok_or("DMR-X decision response missing content")?;
        let answer: DecisionAnswer = serde_json::from_str(content)
            .map_err(|error| format!("invalid DMR-X decision response: {error}"))?;
        validate_answer(request, &answer)?;
        Ok(DecisionResult {
            model: self.model.clone(),
            answer,
            source: "dmr-x".to_owned(),
            usage: DecisionUsage::default(),
        })
    }
}

pub fn from_config(config: &DecisionConfig) -> Result<Box<dyn DecisionProvider>, String> {
    match config.provider.trim().to_ascii_lowercase().as_str() {
        "jev" | "typesafe" | "systemone" => {
            let mut adapter =
                JevDecisionAdapter::new(&config.endpoint, &config.model, config.auth_env.clone());
            adapter.timeout = Duration::from_secs(config.timeout_seconds);
            Ok(Box::new(adapter))
        }
        "dmr-x" | "dmrx" => {
            let mut adapter =
                DmrXDecisionAdapter::new(&config.endpoint, &config.model, config.auth_env.clone());
            adapter.timeout = Duration::from_secs(config.timeout_seconds);
            Ok(Box::new(adapter))
        }
        provider => Err(format!("unsupported decision provider: {provider}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn native_jev_adapter_returns_typed_choice_and_fallback_is_allowed() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 8192];
            let _ = stream.read(&mut request);
            let body = serde_json::json!({
                "model": "jev-1.13.0",
                "answers": {
                    "route": {
                        "type": "choice",
                        "choice": "safe",
                        "confidence": 0.9,
                        "probabilities": {"safe": 0.9, "escalate": 0.1}
                    }
                },
                "usage": {"input_tokens": 10, "output_tokens": 4}
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let request = DecisionRequest {
            id: "route".to_owned(),
            state: serde_json::json!({"task": "inspect"}),
            question: DecisionQuestion::Choice {
                instructions: "Choose the route".to_owned(),
                criteria: BTreeMap::from([
                    ("safe".to_owned(), "continue".to_owned()),
                    ("escalate".to_owned(), "ask a human".to_owned()),
                ]),
            },
        };
        let adapter =
            JevDecisionAdapter::new(format!("http://{address}/v1/systemone"), "jev-latest", None);
        let result = adapter.decide(&request).unwrap();
        assert!(matches!(
            result.answer,
            DecisionAnswer::Choice { ref choice, .. } if choice == "safe"
        ));
        assert!(matches!(
            deterministic_fallback(&request, Some("escalate")),
            DecisionAnswer::Choice { ref choice, .. } if choice == "escalate"
        ));
    }
}
