use serde_json::Value;
use std::io::Read;
use std::time::Duration;

pub(crate) fn post_json(
    endpoint: &str,
    headers: &[(&str, String)],
    body: &Value,
    timeout: Duration,
    max_response: usize,
) -> Result<Value, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let mut request = agent
        .post(endpoint)
        .set("Content-Type", "application/json")
        .set("User-Agent", concat!("mote/", env!("CARGO_PKG_VERSION")));
    for (name, value) in headers {
        request = request.set(name, value);
    }
    let response = match request.send_json(body.clone()) {
        Ok(response) => response,
        Err(ureq::Error::Status(status, _)) => return Err(format!("http status {status}")),
        Err(ureq::Error::Transport(error)) => return Err(format!("http transport error: {error}")),
    };
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take((max_response + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("response read failed: {error}"))?;
    if bytes.len() > max_response {
        return Err("response too large".to_owned());
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid json response: {error}"))
}
