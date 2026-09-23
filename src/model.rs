use crate::action::Action;
use crate::config::{Manifest, ModelConfig};
use crate::http::post_json;
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::Duration;

const SYSTEM_PROMPT: &str = "You are MOTE. Respond with exactly one action line: shell: <command>, read_file: <path>, write_file: <path> | <content>, list_dir: <path>, git: <args>, decision: <json>, custom: <registered-name> | <JSON object>, or complete: <summary>. Use custom only when the task names a host-registered, granted tool. Choose the single simplest permitted action.";

pub trait Model: Send + Sync {
    fn infer(&self, prompt: &str) -> Result<Action, String>;
}

pub struct ModelChain {
    models: Vec<Box<dyn Model>>,
}

impl ModelChain {
    pub fn new(models: Vec<Box<dyn Model>>) -> Result<Self, String> {
        if models.is_empty() {
            return Err("at least one model is required".to_owned());
        }
        Ok(Self { models })
    }

    pub fn from_manifest(manifest: &Manifest) -> Result<Self, String> {
        let models = manifest
            .models
            .iter()
            .map(HttpModel::from_config)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|model| Box::new(model) as Box<dyn Model>)
            .collect();
        Self::new(models)
    }
}

impl Model for ModelChain {
    fn infer(&self, prompt: &str) -> Result<Action, String> {
        let mut errors = Vec::new();
        for model in &self.models {
            let mut empty_retries = 0;
            loop {
                match model.infer(prompt) {
                    Ok(action) => return Ok(action),
                    Err(error)
                        if empty_retries == 0
                            && matches!(
                                error.as_str(),
                                "provider response missing content" | "action payload is empty"
                            ) =>
                    {
                        empty_retries += 1;
                    }
                    Err(error) => {
                        errors.push(error);
                        break;
                    }
                }
            }
        }
        Err(format!("all models failed: {}", errors.join("; ")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAiCompatible,
    DmrX,
    Gemini,
    Anthropic,
}

impl FromStr for Provider {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" | "openai-compatible" => Ok(Self::OpenAiCompatible),
            "dmr-x" | "dmrx" => Ok(Self::DmrX),
            "google" | "gemini" => Ok(Self::Gemini),
            "anthropic" => Ok(Self::Anthropic),
            _ => Err(format!("unsupported model provider: {value}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpModel {
    pub provider: Provider,
    pub endpoint: String,
    pub model: String,
    pub auth_env: Option<String>,
    pub timeout: Duration,
    pub max_response: usize,
}

impl HttpModel {
    pub fn new(
        provider: Provider,
        endpoint: impl Into<String>,
        model: impl Into<String>,
        auth_env: Option<String>,
    ) -> Self {
        Self {
            provider,
            endpoint: endpoint.into(),
            model: model.into(),
            auth_env,
            timeout: Duration::from_secs(30),
            max_response: 1_048_576,
        }
    }

    pub fn from_config(config: &ModelConfig) -> Result<Self, String> {
        let mut model = Self::new(
            Provider::from_str(&config.provider)?,
            &config.endpoint,
            &config.model,
            config.auth_env.clone(),
        );
        model.timeout = Duration::from_secs(config.timeout_seconds);
        Ok(model)
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

    fn request(&self, prompt: &str) -> Result<Value, String> {
        let authentication = self.authentication()?;
        let mut headers = Vec::new();
        let body = match self.provider {
            Provider::OpenAiCompatible | Provider::DmrX => {
                if let Some(secret) = authentication {
                    headers.push(("Authorization", format!("Bearer {secret}")));
                }
                json!({
                    "model": self.model,
                    "messages": [
                        {"role": "system", "content": SYSTEM_PROMPT},
                        {"role": "user", "content": prompt}
                    ],
                    "max_tokens": 512,
                    "stream": false
                })
            }
            Provider::Gemini => {
                if let Some(secret) = authentication {
                    headers.push(("x-goog-api-key", secret));
                }
                json!({
                    "system_instruction": {"parts": [{"text": SYSTEM_PROMPT}]},
                    "contents": [{"role": "user", "parts": [{"text": prompt}]}]
                })
            }
            Provider::Anthropic => {
                if let Some(secret) = authentication {
                    headers.push(("x-api-key", secret));
                }
                headers.push(("anthropic-version", "2023-06-01".to_owned()));
                json!({
                    "model": self.model,
                    "max_tokens": 512,
                    "system": SYSTEM_PROMPT,
                    "messages": [{"role": "user", "content": prompt}]
                })
            }
        };
        post_json(
            &self.endpoint,
            &headers,
            &body,
            self.timeout,
            self.max_response,
        )
    }
}

impl Model for HttpModel {
    fn infer(&self, prompt: &str) -> Result<Action, String> {
        let value = self.request(prompt)?;
        let content = match self.provider {
            Provider::Gemini => value.pointer("/candidates/0/content/parts/0/text"),
            Provider::Anthropic => value.pointer("/content/0/text"),
            Provider::OpenAiCompatible | Provider::DmrX => {
                value.pointer("/choices/0/message/content")
            }
        }
        .and_then(Value::as_str)
        .ok_or("provider response missing content")?;
        parse_action(content)
    }
}

pub fn parse_action(content: &str) -> Result<Action, String> {
    let text = content.trim();
    if text.is_empty() {
        return Err("action payload is empty".to_owned());
    }
    let (kind, rest) = text.split_once(':').ok_or("invalid action")?;
    let rest = rest.trim();
    if rest.is_empty() {
        return Err("action payload is empty".to_owned());
    }
    match kind.trim().to_ascii_lowercase().as_str() {
        "complete" => Ok(Action::Complete {
            summary: rest.to_owned(),
        }),
        "shell" => Ok(Action::Shell {
            command: rest.to_owned(),
        }),
        "read_file" => Ok(Action::ReadFile {
            path: rest.to_owned(),
        }),
        "write_file" => {
            let (path, content) = rest
                .split_once('|')
                .ok_or("write_file requires: path | content")?;
            Ok(Action::WriteFile {
                path: path.trim().to_owned(),
                content: content.trim().to_owned(),
            })
        }
        "list_dir" => Ok(Action::ListDir {
            path: rest.to_owned(),
        }),
        "git" => Ok(Action::Git {
            args: rest.to_owned(),
        }),
        "decision" => serde_json::from_str(rest)
            .map(|request| Action::Decision { request })
            .map_err(|error| format!("invalid decision action: {error}")),
        "custom" => {
            let (name, input) = rest
                .split_once('|')
                .ok_or("custom requires: name | JSON object")?;
            let name = name.trim();
            if name.is_empty() {
                return Err("custom tool name is empty".to_owned());
            }
            let input: Value = serde_json::from_str(input.trim())
                .map_err(|error| format!("invalid custom tool input: {error}"))?;
            if !input.is_object() {
                return Err("custom tool input must be a JSON object".to_owned());
            }
            Ok(Action::Custom {
                name: name.to_owned(),
                input,
            })
        }
        _ => Err("unknown action".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn custom_action_parses_name_and_json_object() {
        assert_eq!(
            parse_action("custom: lookup | {\"query\":\"status\"}").unwrap(),
            Action::Custom {
                name: "lookup".into(),
                input: json!({"query":"status"}),
            }
        );
        assert!(parse_action("custom: lookup | [1,2]").is_err());
        assert!(parse_action("custom: lookup | not-json").is_err());
        assert!(parse_action("custom: | {}").is_err());
    }

    fn chunked_server(body: String, status: u16) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 8192];
            loop {
                let read = stream.read(&mut buffer).unwrap();
                assert!(read > 0, "client disconnected before sending request");
                request.extend_from_slice(&buffer[..read]);
                if let Some(end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]);
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            line.split_once(':').and_then(|(name, value)| {
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().ok())
                                    .flatten()
                            })
                        })
                        .expect("request must have Content-Length");
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let reason = if status == 200 { "OK" } else { "Unauthorized" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\nTransfer-Encoding: chunked\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{:x}\r\n{}\r\n0\r\n\r\n",
                body.len(), body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        format!("http://{address}/v1/chat/completions")
    }

    #[test]
    fn whitespace_only_model_content_is_retryable() {
        assert_eq!(
            parse_action(" \n\t").unwrap_err(),
            "action payload is empty"
        );
    }

    #[test]
    fn empty_response_retries_once_before_falling_back() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        struct EmptyOnce(Arc<AtomicUsize>);
        impl Model for EmptyOnce {
            fn infer(&self, _: &str) -> Result<Action, String> {
                if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                    Err("provider response missing content".to_owned())
                } else {
                    Ok(Action::Complete {
                        summary: "first recovered".to_owned(),
                    })
                }
            }
        }
        struct Fallback;
        impl Model for Fallback {
            fn infer(&self, _: &str) -> Result<Action, String> {
                Ok(Action::Complete {
                    summary: "fallback".to_owned(),
                })
            }
        }
        let calls = Arc::new(AtomicUsize::new(0));
        let chain =
            ModelChain::new(vec![Box::new(EmptyOnce(calls.clone())), Box::new(Fallback)]).unwrap();
        assert_eq!(
            chain.infer("task").unwrap(),
            Action::Complete {
                summary: "first recovered".to_owned()
            }
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn http_model_decodes_chunked_success_and_rejects_non_success() {
        let success = serde_json::json!({
            "choices": [{"message": {"content": "complete: verified"}}]
        })
        .to_string();
        let model = HttpModel::new(
            Provider::OpenAiCompatible,
            chunked_server(success, 200),
            "test",
            None,
        );
        assert_eq!(
            model.infer("task").unwrap(),
            Action::Complete {
                summary: "verified".to_owned()
            }
        );

        let model = HttpModel::new(
            Provider::OpenAiCompatible,
            chunked_server("{\"error\":\"denied\"}".to_owned(), 401),
            "test",
            None,
        );
        assert!(model.infer("task").unwrap_err().contains("401"));
    }
}
