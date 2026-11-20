//! AI Gateway for the M5 conversation module.
//!
//! OpenAI-compatible Chat Completions over HTTPS with explicit timeout and a
//! single retry for transient failures (transport errors and 5xx). Provider
//! credentials live in the local database only and are never exposed to the
//! frontend build.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LlmConfig {
    /// e.g. https://api.openai.com/v1 or https://api.deepseek.com/v1
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmReply {
    pub content: String,
    pub model: String,
    pub latency_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LlmTestResult {
    pub ok: bool,
    pub latency_ms: i64,
    pub message: String,
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const READ_TIMEOUT: Duration = Duration::from_secs(45);

fn build_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(CONNECT_TIMEOUT)
        .timeout_read(READ_TIMEOUT)
        .timeout_write(READ_TIMEOUT)
        .build()
}

/// Whether an attempt failure is worth one retry (transport errors, 429, 5xx).
#[derive(Debug)]
enum AttemptError {
    Transient(String),
    Permanent(String),
}

fn map_http_error(code: u16, detail: &str) -> String {
    let detail = if detail.trim().is_empty() {
        String::new()
    } else {
        format!("（{detail}）")
    };
    match code {
        400 => format!("请求参数错误（400）{detail}，请检查模型名称或配置。"),
        401 | 403 => format!("API Key 无效或无权限（{code}），请在设置中检查后重试。"),
        404 => format!("接口地址不正确（404）{detail}，请在设置中检查 Base URL。"),
        429 => "请求过于频繁（429），请稍后重试。".to_string(),
        500..=599 => format!("AI 服务暂时不可用（{code}），请稍后重试。"),
        _ => format!("AI 服务返回异常（{code}）{detail}，请稍后重试。"),
    }
}

fn parse_reply(body: &serde_json::Value, model: &str) -> Result<LlmReply, String> {
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "AI 返回内容为空，请重试。".to_string())?
        .trim()
        .to_string();
    if content.is_empty() {
        return Err("AI 返回内容为空，请重试。".to_string());
    }
    let reply_model = body["model"].as_str().unwrap_or(model).to_string();
    Ok(LlmReply {
        content,
        model: reply_model,
        latency_ms: 0,
    })
}

/// Run one chat completion attempt against the configured provider.
fn attempt(
    config: &LlmConfig,
    messages: &[ChatMessage],
    max_tokens: u32,
) -> Result<LlmReply, AttemptError> {
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": config.model,
        "messages": messages
            .iter()
            .map(|message| serde_json::json!({ "role": message.role, "content": message.content }))
            .collect::<Vec<_>>(),
        "temperature": 0.7,
        "max_tokens": max_tokens,
    });
    let response = build_agent()
        .post(&url)
        .set("Content-Type", "application/json")
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .send_json(payload)
        .map_err(|error| match error {
            ureq::Error::Status(code, response) => {
                let detail = response
                    .into_string()
                    .unwrap_or_default()
                    .chars()
                    .take(160)
                    .collect::<String>();
                let message = map_http_error(code, &detail);
                if matches!(code, 429 | 500..=599) {
                    AttemptError::Transient(message)
                } else {
                    AttemptError::Permanent(message)
                }
            }
            ureq::Error::Transport(transport) => {
                let detail = transport.to_string();
                AttemptError::Transient(format!(
                    "网络连接失败，请检查网络与配置后重试。（{detail}）"
                ))
            }
        })?;
    let body: serde_json::Value = response
        .into_json()
        .map_err(|error| AttemptError::Permanent(format!("AI 返回格式无法解析：{error}")))?;
    parse_reply(&body, &config.model).map_err(AttemptError::Permanent)
}

/// Chat completion with one retry on transient failures.
pub fn chat_completion(
    config: &LlmConfig,
    messages: &[ChatMessage],
    max_tokens: u32,
) -> Result<LlmReply, String> {
    let started = Instant::now();
    for attempt_index in 0..2 {
        match attempt(config, messages, max_tokens) {
            Ok(mut reply) => {
                reply.latency_ms = started.elapsed().as_millis() as i64;
                return Ok(reply);
            }
            Err(AttemptError::Permanent(message)) => return Err(message),
            Err(AttemptError::Transient(_message)) if attempt_index == 0 => continue,
            Err(AttemptError::Transient(message)) => return Err(message),
        }
    }
    Err("AI 请求失败，请稍后重试。".to_string())
}

/// Send a tiny prompt to verify the configured provider works.
pub fn test_connection(config: &LlmConfig) -> LlmTestResult {
    let started = Instant::now();
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a connectivity probe. Reply with exactly OK.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Ping".to_string(),
        },
    ];
    match chat_completion(config, &messages, 16) {
        Ok(reply) => LlmTestResult {
            ok: true,
            latency_ms: reply.latency_ms,
            message: format!("连接成功（{}，{} ms）。", reply.model, reply.latency_ms),
        },
        Err(message) => LlmTestResult {
            ok: false,
            latency_ms: started.elapsed().as_millis() as i64,
            message,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};
    use std::thread;

    /// Minimal mock OpenAI server. Each accepted connection pops one response
    /// from the queue; the final response is reused for any extra connections.
    fn spawn_mock(responses: Vec<(u16, &'static str)>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://127.0.0.1:{}", addr.port());
        let queue = Arc::new(Mutex::new(responses));
        thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut stream) = stream {
                    read_full_request(&mut stream);
                    let mut queue = queue.lock().unwrap();
                    let (status, body) = if queue.is_empty() {
                        (500, "{}")
                    } else if queue.len() == 1 {
                        queue[0]
                    } else {
                        queue.remove(0)
                    };
                    let reason = if status == 200 { "OK" } else { "Error" };
                    let header = format!(
                        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(header.as_bytes());
                    let _ = stream.write_all(body.as_bytes());
                }
            }
        });
        base_url
    }


    /// Read until the full HTTP request (headers + Content-Length body) is
    /// consumed so the connection is not closed with unread data (which would
    /// make the client see a connection reset instead of a clean response).
    fn read_full_request(stream: &mut TcpStream) {
        let mut data = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => data.extend_from_slice(&buffer[..n]),
                Err(_) => break,
            }
            let text = String::from_utf8_lossy(&data);
            if let Some(end) = text.find("\r\n\r\n") {
                let head = &text[..end];
                let mut length = None;
                for line in head.split("\r\n") {
                    let lower = line.to_ascii_lowercase();
                    if let Some(rest) = lower.strip_prefix("content-length:") {
                        length = rest.trim().parse::<usize>().ok();
                    }
                }
                if let Some(len) = length {
                    if data.len() >= end + 4 + len {
                        break;
                    }
                }
            }
        }
    }

    const OK_BODY: &str = r#"{"id":"chatcmpl-1","model":"mock-model","choices":[{"index":0,"message":{"role":"assistant","content":"Hello from the mock."},"finish_reason":"stop"}]}"#;

    fn config(base_url: &str) -> LlmConfig {
        LlmConfig {
            base_url: base_url.to_string(),
            api_key: "test-key".to_string(),
            model: "mock-model".to_string(),
        }
    }

    #[test]
    fn chat_completion_parses_reply() {
        let base = spawn_mock(vec![(200, OK_BODY)]);
        let reply = chat_completion(&config(&base), &[], 64).unwrap();
        assert_eq!(reply.content, "Hello from the mock.");
        assert_eq!(reply.model, "mock-model");
        assert!(reply.latency_ms >= 0);
    }

    #[test]
    fn chat_completion_retries_once_on_5xx() {
        let base = spawn_mock(vec![(500, r#"{"error":{"message":"boom"}}"#), (200, OK_BODY)]);
        let reply = chat_completion(&config(&base), &[], 64).unwrap();
        assert_eq!(reply.content, "Hello from the mock.");
    }

    #[test]
    fn chat_completion_fails_after_two_5xx() {
        let base = spawn_mock(vec![(500, "{}"), (503, "{}")]);
        let error = chat_completion(&config(&base), &[], 64).unwrap_err();
        assert!(error.contains("AI 服务暂时不可用"));
    }

    #[test]
    fn chat_completion_maps_401_to_clear_message() {
        let base = spawn_mock(vec![(401, r#"{"error":{"message":"bad key"}}"#)]);
        let error = chat_completion(&config(&base), &[], 64).unwrap_err();
        assert!(error.contains("API Key 无效或无权限"));
        assert!(!error.contains("bad key"));
    }

    #[test]
    fn chat_completion_maps_404_to_base_url_message() {
        let base = spawn_mock(vec![(404, "{}")]);
        let error = chat_completion(&config(&base), &[], 64).unwrap_err();
        assert!(error.contains("接口地址不正确"));
    }

    #[test]
    fn empty_reply_is_reported() {
        let base = spawn_mock(vec![(
            200,
            r#"{"id":"1","model":"mock-model","choices":[{"index":0,"message":{"role":"assistant","content":"   "},"finish_reason":"stop"}]}"#,
        )]);
        let error = chat_completion(&config(&base), &[], 64).unwrap_err();
        assert!(error.contains("AI 返回内容为空"));
    }
}