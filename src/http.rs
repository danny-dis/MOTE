use serde_json::Value;
use std::io::Read;
use std::time::Duration;

pub(crate) fn authentication(auth_env: Option<&str>) -> Result<Option<String>, String> {
    auth_env
        .map(|name| {
            std::env::var(name).map_err(|_| format!("missing auth environment variable: {name}"))
        })
        .transpose()
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn authentication_preserves_keyless_missing_and_configured_values() {
        assert_eq!(authentication(None).unwrap(), None);
        let name = format!("MOTE_AUTH_TEST_{}", std::process::id());
        assert!(std::env::var_os(&name).is_none());
        assert_eq!(
            authentication(Some(&name)).unwrap_err(),
            format!("missing auth environment variable: {name}")
        );
        for value in ["test-only-key", ""] {
            std::env::set_var(&name, value);
            assert_eq!(authentication(Some(&name)).unwrap(), Some(value.to_owned()));
        }
        std::env::remove_var(&name);
    }

    fn read_request(stream: &mut impl BufRead) -> bool {
        let mut line = String::new();
        if stream.read_line(&mut line).unwrap() == 0 {
            return false;
        }
        let mut length = 0;
        loop {
            line.clear();
            assert!(stream.read_line(&mut line).unwrap() > 0);
            if line.as_bytes() == [13, 10] {
                break;
            }
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") {
                    length = value.trim().parse::<usize>().unwrap();
                }
            }
        }
        let mut body = vec![0; length];
        stream.read_exact(&mut body).unwrap();
        assert_eq!(body, b"{}");
        true
    }

    #[test]
    fn repeated_requests_keep_header_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut stream = BufReader::new(stream);
            for request in 0..2 {
                if !read_request(&mut stream) {
                    let (next, _) = listener.accept().unwrap();
                    next.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                    stream = BufReader::new(next);
                    assert!(read_request(&mut stream));
                }
                if request == 1 {
                    thread::sleep(Duration::from_secs(2));
                }
                let _ = stream
                    .get_mut()
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}");
            }
        });
        let timeout = Duration::from_millis(300);
        assert_eq!(
            post_json(&endpoint, &[], &serde_json::json!({}), timeout, 32).unwrap(),
            serde_json::json!({})
        );
        let started = std::time::Instant::now();
        let result = post_json(&endpoint, &[], &serde_json::json!({}), timeout, 32);
        let elapsed = started.elapsed();
        server.join().unwrap();
        assert!(
            result.is_err(),
            "second request ignored its timeout: {result:?} after {elapsed:?}"
        );
        assert!(
            elapsed < Duration::from_secs(1),
            "timeout was not prompt: {elapsed:?}"
        );
    }

    #[test]
    fn cached_client_honors_timeout_changes_and_response_limit() {
        for (timeout, limit, expected) in [
            (Duration::from_secs(5), 32, None),
            (Duration::from_millis(30), 32, Some("http transport error")),
            (Duration::from_secs(5), 1, Some("response too large")),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}/", listener.local_addr().unwrap());
            let server = thread::spawn(move || {
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut stream = BufReader::new(stream);
                assert!(read_request(&mut stream));
                thread::sleep(Duration::from_millis(300));
                let _ = stream.get_mut().write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                );
            });
            let result = post_json(&endpoint, &[], &serde_json::json!({}), timeout, limit);
            match expected {
                Some(message) => assert!(result.unwrap_err().contains(message)),
                None => assert_eq!(result.unwrap(), serde_json::json!({})),
            }
            server.join().unwrap();
        }
    }
}
