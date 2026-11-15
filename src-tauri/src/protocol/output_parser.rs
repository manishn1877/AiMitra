//! Output parsing (ported from deepseek-recipe).
//!
//! Parses llama.cpp's `/completion` streaming and non-streaming responses and
//! converts them into the protocol-neutral `UnifiedStreamEvent` / `UnifiedResponse`
//! types produced by [`converter`].

use serde::{Deserialize, Serialize};

use crate::protocol::converter::{UnifiedResponse, UnifiedStreamEvent, UnifiedUsage};

/// Raw non-streaming completion response from llama.cpp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaCompletionResponse {
    pub content: String,
    pub model: Option<String>,
    #[serde(default)]
    pub tokens_predicted: Option<u32>,
    #[serde(default)]
    pub tokens_evaluated: Option<u32>,
    #[serde(default)]
    pub generation_time: Option<f32>,
    #[serde(default, rename = "timings")]
    pub timings: Option<LlamaTimings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaTimings {
    #[serde(default, rename = "predicted_ms")]
    pub predicted_ms: Option<f32>,
    #[serde(default, rename = "predicted_seconds")]
    pub predicted_seconds: Option<f32>,
    #[serde(default, rename = "evaluated_ms")]
    pub evaluated_ms: Option<f32>,
    #[serde(default, rename = "evaluated_seconds")]
    pub evaluated_seconds: Option<f32>,
    #[serde(default, rename = "per_second")]
    pub per_second: Option<f32>,
    #[serde(default, rename = "n_prompt_tokens")]
    pub n_prompt_tokens: Option<u32>,
    #[serde(default, rename = "n_predicted_tokens")]
    pub n_predicted_tokens: Option<u32>,
}

/// A single token fragment from a streaming llama.cpp response (SSE `data:` line).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaStreamData {
    pub content: Option<String>,
    #[serde(default)]
    pub stop: Option<bool>,
    #[serde(default)]
    pub generation_settings: Option<serde_json::Value>,
    #[serde(rename = "model")]
    pub model: Option<String>,
    #[serde(default, rename = "stop")]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub timings: Option<LlamaTimings>,
    #[serde(default, rename = "pool_perf")]
    pub pool_perf: Option<serde_json::Value>,
    #[serde(default)]
    pub index: Option<u32>,
    #[serde(default)]
    pub timestamp: Option<f64>,
}

/// Parse a complete (non-streaming) llama.cpp response into a `UnifiedResponse`.
pub fn parse_completion(resp: LlamaCompletionResponse) -> UnifiedResponse {
    let LlamaCompletionResponse {
        content,
        model,
        tokens_predicted,
        tokens_evaluated,
        timings,
        ..
    } = resp;

    let usage = match (tokens_evaluated, tokens_predicted) {
        (Some(prompt), Some(completion)) => Some(UnifiedUsage::new(prompt, completion)),
        _ => timings.as_ref().and_then(|t| {
            match (t.n_prompt_tokens, t.n_predicted_tokens) {
                (Some(p), Some(c)) => Some(UnifiedUsage::new(p, c)),
                _ => None,
            }
        }),
    };

    UnifiedResponse {
        text: content,
        model: model.or_else(|| Some("local".to_string())),
        usage,
        tool_calls: None,
    }
}

/// Parse a single SSE `data:` line from a streaming llama.cpp response into a
/// `UnifiedStreamEvent`.
///
/// Returns `None` when the event carries no textual token (e.g. a keep-alive or
/// a `slot` update that we don't surface). Returns an event with
/// `reason = Some("stop")` when the stream signals completion.
pub fn parse_stream_event(data: &str) -> Option<UnifiedStreamEvent> {
    // llama.cpp emits JSON lines on `data:` prefixed lines, but the prefix is
    // already stripped by the caller (we receive the JSON body).
    let parsed: LlamaStreamData = match serde_json::from_str(data) {
        Ok(v) => v,
        Err(_) => {
            // Some llama.cpp builds emit a bare JSON string for the final
            // "stop" sentinel. Treat any non-JSON line as a no-op.
            return None;
        }
    };

    if parsed.stop == Some(true) {
        return Some(UnifiedStreamEvent {
            delta: String::new(),
            reason: Some("stop".to_string()),
            tool_call_delta: None,
        });
    }

    // Extract the token text. llama.cpp may put it under `content` or
    // `content_parts`. Handle both.
    let text = parsed
        .content
        .as_deref()
        .unwrap_or("")
        .to_string();

    // Detect tool-call fragments. Some llama.cpp forks emit function calls
    // inline as special tokens; we surface them as tool-call deltas.
    let tool_call_delta = if text.starts_with("[func:") && text.ends_with(']') {
        let inner = &text["[func:".len()..text.len() - 1];
        if let Some((name, args)) = inner.split_once('(') {
            let args = args.trim_end_matches(')').trim();
            Some(crate::protocol::converter::UnifiedToolCall {
                id: uuid::Uuid::new_v4().to_string(),
                name: name.trim().to_string(),
                arguments: args.to_string(),
            })
        } else {
            None
        }
    } else {
        None
    };

    // If we have a tool-call delta, don't also emit it as text.
    let delta_text = if tool_call_delta.is_some() {
        String::new()
    } else {
        text
    };

    Some(UnifiedStreamEvent {
        delta: delta_text,
        reason: None,
        tool_call_delta,
    })
}

/// Parse a full SSE stream body (the concatenated `data:` payloads) into a list
/// of unified events. This is a convenience for non-streaming fallback logic.
pub fn parse_stream_body(body: &str) -> Vec<UnifiedStreamEvent> {
    let mut events = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line == "data: [DONE]" {
            continue;
        }
        let data = line
            .strip_prefix("data: ")
            .or_else(|| line.strip_prefix("data:"))
            .unwrap_or(line);
        if let Some(event) = parse_stream_event(data.trim()) {
            events.push(event);
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_non_streaming() {
        let resp = LlamaCompletionResponse {
            content: "Hello world".into(),
            model: Some("codeqwen".into()),
            tokens_predicted: Some(3),
            tokens_evaluated: Some(5),
            generation_time: None,
            timings: None,
        };
        let unified = parse_completion(resp);
        assert_eq!(unified.text, "Hello world");
        assert!(unified.usage.is_some());
        assert_eq!(unified.usage.as_ref().unwrap().prompt_tokens, 5);
        assert_eq!(unified.usage.as_ref().unwrap().completion_tokens, 3);
    }

    #[test]
    fn parse_stream_token() {
        let data = r#"{"content":"Hello"}"#;
        let event = parse_stream_event(data).unwrap();
        assert_eq!(event.delta, "Hello");
        assert!(event.reason.is_none());
    }

    #[test]
    fn parse_stream_stop() {
        let data = r#"{"content":"","stop":true}"#;
        let event = parse_stream_event(data).unwrap();
        assert_eq!(event.delta, "");
        assert_eq!(event.reason.as_deref(), Some("stop"));
    }

    #[test]
    fn parse_stream_body_aggregates() {
        let body = "data: {\"content\":\"Hi\"}\ndata: {\"content\":\" there\"}\ndata: {\"content\":\"\",\"stop\":true}\ndata: [DONE]";
        let events = parse_stream_body(body);
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].delta, "Hi");
        assert_eq!(events[1].delta, " there");
        assert_eq!(events[2].reason.as_deref(), Some("stop"));
    }
}
