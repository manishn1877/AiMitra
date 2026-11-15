//! Protocol conversion layer (ported from deepseek-recipe).
//!
//! Converts incoming requests in different API formats (OpenAI Chat Completions,
//! Anthropic Messages, OpenAI Responses) into a single unified `UnifiedRequest`,
//! and converts model output back into the format requested by the caller.
//!
//! This is the glue between the externally-facing `/v1/chat/completions`,
//! `/v1/messages`, and `/v1/responses` endpoints and the llama.cpp `/completion`
//! endpoint, which always speaks its own single format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::protocol::chat_completions::{
    ChatCompletionRequest, ChatCompletionChunk, ChatCompletionResponse, ChatChoice, ChatResponseMessage,
    ChatStreamChoice, ChatStreamDelta, Usage,
};
use crate::protocol::messages::{
    MessagesRequest, MessagesResponse, MessagesStreamEvent, AnthropicMessage, AnthropicContentBlock,
    MessagesUsage,
};
use crate::protocol::responses::{
    ResponsesRequest, ResponsesResponse, ResponsesStreamEvent, ResponsesUsage,
};

/// Unified internal representation of a completion request.
///
/// Every incoming protocol format is flattened into this before being sent to
/// llama.cpp's `/completion` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedRequest {
    /// Concatenated prompt in llama.cpp format.
    pub prompt: String,
    /// Model identifier (unused by llama.cpp, kept for parity).
    pub model: Option<String>,
    /// Generation parameters.
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop: Option<Vec<String>>,
    pub stream: bool,
    /// Tools the caller wants the model to be able to invoke.
    pub tools: Option<Vec<UnifiedTool>>,
    /// Optional system prompt pulled out of the message list.
    pub system: Option<String>,
}

/// Tool definition normalised across protocols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

/// Unified internal representation of a completion response (non-streaming).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    pub text: String,
    pub model: Option<String>,
    pub usage: Option<UnifiedUsage>,
    pub tool_calls: Option<Vec<UnifiedToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Unified streaming event. A single textual delta (may be a function-call
/// fragment) ready to be emitted over an SSE stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedStreamEvent {
    pub delta: String,
    pub reason: Option<String>,
    pub tool_call_delta: Option<UnifiedToolCall>,
}

impl UnifiedResponse {
    pub fn new(text: String) -> Self {
        Self {
            text,
            model: None,
            usage: None,
            tool_calls: None,
        }
    }
}

impl UnifiedUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

// ---------------------------------------------------------------------------
// Converters: incoming -> UnifiedRequest
// ---------------------------------------------------------------------------

/// Convert an OpenAI Chat Completions request into a `UnifiedRequest`.
pub fn from_chat_completion(req: &ChatCompletionRequest) -> UnifiedRequest {
    let (prompt, system) = render_chat_messages(&req.messages);

    UnifiedRequest {
        prompt,
        model: Some(req.model.clone()),
        temperature: req.temperature,
        top_p: req.top_p,
        max_tokens: req.max_tokens,
        stop: req.stop.clone(),
        stream: req.stream.unwrap_or(false),
        tools: req.tools.as_ref().map(|t| {
            t.iter()
                .filter_map(|def| {
                    if def.tool_type.as_deref() == Some("function") {
                        Some(UnifiedTool {
                            name: def.function.name.clone(),
                            description: def.function.description.clone(),
                            parameters: def.function.parameters.clone(),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }),
        system,
    }
}

/// Convert an Anthropic Messages request into a `UnifiedRequest`.
pub fn from_messages(req: &MessagesRequest) -> UnifiedRequest {
    // Anthropic messages are role/content pairs; render as a chat transcript.
    let mut messages: Vec<crate::protocol::chat_completions::ChatMessage> = Vec::new();

    if let Some(sys) = &req.system {
        messages.push(crate::protocol::chat_completions::ChatMessage {
            role: "system".to_string(),
            content: Some(match sys {
                crate::protocol::messages::AnthropicSystemPrompt::Text(t) => {
                    crate::protocol::chat_completions::ChatMessageContent::Text(t.clone())
                }
                crate::protocol::messages::AnthropicSystemPrompt::Blocks(blocks) => {
                    let text = blocks
                        .iter()
                        .filter_map(|b| b.text.clone())
                        .collect::<Vec<_>>()
                        .join("\n");
                    crate::protocol::chat_completions::ChatMessageContent::Text(text)
                }
            }),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });
    }

    for msg in &req.messages {
        let content = match &msg.content {
            crate::protocol::messages::AnthropicContent::Text(t) => {
                crate::protocol::chat_completions::ChatMessageContent::Text(t.clone())
            }
            crate::protocol::messages::AnthropicContent::Blocks(blocks) => {
                let text = blocks
                    .iter()
                    .filter_map(|b| match b {
                        crate::protocol::messages::AnthropicContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                crate::protocol::chat_completions::ChatMessageContent::Text(text)
            }
        };
        messages.push(crate::protocol::chat_completions::ChatMessage {
            role: msg.role.clone(),
            content: Some(content),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        });
    }

    let (prompt, system) = render_chat_messages(&messages);

    UnifiedRequest {
        prompt,
        model: Some(req.model.clone()),
        temperature: req.temperature,
        top_p: req.top_p,
        max_tokens: Some(req.max_tokens),
        stop: req.stop_sequences.clone(),
        stream: req.stream.unwrap_or(false),
        tools: req.tools.as_ref().map(|t| {
            t.iter()
                .filter_map(|tool| match tool {
                    crate::protocol::messages::AnthropicTool::Function { name, description, input_schema } => {
                        Some(UnifiedTool {
                            name: name.clone(),
                            description: description.clone(),
                            parameters: Some(input_schema.clone()),
                        })
                    }
                })
                .collect()
        }),
        system,
    }
}

/// Convert an OpenAI Responses API request into a `UnifiedRequest`.
pub fn from_responses(req: &ResponsesRequest) -> UnifiedRequest {
    let (prompt, system) = match &req.input {
        crate::protocol::responses::ResponsesInput::Text(t) => (t.clone(), req.instructions.clone()),
        crate::protocol::responses::ResponsesInput::Items(items) => {
            // Render structured items into a chat-style prompt.
            let mut messages: Vec<crate::protocol::chat_completions::ChatMessage> = Vec::new();
            for item in items {
                match item {
                    crate::protocol::responses::ResponsesItem::Message { role, content } => {
                        let text = match content {
                            crate::protocol::responses::ResponsesContent::Text(t) => t.clone(),
                            crate::protocol::responses::ResponsesContent::Parts(parts) => {
                                parts
                                    .iter()
                                    .filter_map(|p| match p {
                                        crate::protocol::responses::ResponsesContentPart::InputText { text } => Some(text.clone()),
                                        crate::protocol::responses::ResponsesContentPart::OutputText { text } => Some(text.clone()),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        };
                        messages.push(crate::protocol::chat_completions::ChatMessage {
                            role: role.clone(),
                            content: Some(crate::protocol::chat_completions::ChatMessageContent::Text(text)),
                            name: None,
                            tool_call_id: None,
                            tool_calls: None,
                        });
                    }
                    crate::protocol::responses::ResponsesItem::FunctionCall { name, arguments, .. } => {
                        messages.push(crate::protocol::chat_completions::ChatMessage {
                            role: "assistant".to_string(),
                            content: Some(crate::protocol::chat_completions::ChatMessageContent::Text(format!("{}({})", name, arguments))),
                            name: None,
                            tool_call_id: None,
                            tool_calls: None,
                        });
                    }
                    crate::protocol::responses::ResponsesItem::FunctionCallOutput { output, .. } => {
                        messages.push(crate::protocol::chat_completions::ChatMessage {
                            role: "tool".to_string(),
                            content: Some(crate::protocol::chat_completions::ChatMessageContent::Text(output.clone())),
                            name: None,
                            tool_call_id: None,
                            tool_calls: None,
                        });
                    }
                    _ => {}
                }
            }
            let (p, s) = render_chat_messages(&messages);
            if let Some(instructions) = &req.instructions {
                (format!("{}\n\n{}", instructions, p), Some(instructions.clone()))
            } else {
                (p, s)
            }
        }
    };

    UnifiedRequest {
        prompt,
        model: Some(req.model.clone()),
        temperature: req.temperature,
        top_p: req.top_p,
        max_tokens: req.max_output_tokens,
        stop: req.stop.clone(),
        stream: req.stream.unwrap_or(false),
        tools: req.tools.as_ref().map(|t| {
            t.iter()
                .filter_map(|tool| match tool {
                    crate::protocol::responses::ResponsesTool::Function { name, description, parameters, .. } => {
                        Some(UnifiedTool {
                            name: name.clone(),
                            description: description.clone(),
                            parameters: parameters.clone(),
                        })
                    }
                    _ => None,
                })
                .collect()
        }),
        system,
    }
}

// ---------------------------------------------------------------------------
// Converters: UnifiedResponse -> outgoing protocol formats
// ---------------------------------------------------------------------------

/// Convert a `UnifiedResponse` back into an OpenAI Chat Completions response.
pub fn to_chat_completion(resp: UnifiedResponse) -> ChatCompletionResponse {
    let id = uuid::Uuid::new_v4().to_string();
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let tool_calls = resp.tool_calls.as_ref().map(|calls| {
        calls
            .iter()
            .map(|tc| crate::protocol::chat_completions::ChatToolCall {
                id: tc.id.clone(),
                tool_type: "function".to_string(),
                function: crate::protocol::chat_completions::ChatFunctionCall {
                    name: tc.name.clone(),
                    arguments: tc.arguments.clone(),
                },
            })
            .collect()
    });

    ChatCompletionResponse {
        id,
        object: "chat.completion".to_string(),
        created,
        model: resp.model.unwrap_or_else(|| "local".to_string()),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatResponseMessage {
                role: "assistant".to_string(),
                content: Some(resp.text),
                tool_calls,
                reasoning_content: None,
            },
            finish_reason: Some("stop".to_string()),
            logprobs: None,
        }],
        usage: resp.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            completion_tokens_details: None,
            prompt_tokens_details: None,
        }),
    }
}

/// Convert a `UnifiedResponse` back into an Anthropic Messages response.
pub fn to_messages(resp: UnifiedResponse) -> MessagesResponse {
    let usage = resp.usage.map(|u| MessagesUsage {
        input_tokens: u.prompt_tokens,
        output_tokens: u.completion_tokens,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
    });

    MessagesResponse {
        id: uuid::Uuid::new_v4().to_string(),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        model: resp.model.unwrap_or_else(|| "local".to_string()),
        content: vec![AnthropicContentBlock::Text { text: resp.text }],
        stop_reason: Some("end_turn".to_string()),
        usage,
    }
}

/// Convert a `UnifiedResponse` back into an OpenAI Responses API response.
pub fn to_responses(resp: UnifiedResponse) -> ResponsesResponse {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    ResponsesResponse {
        id: format!("resp_{}", uuid::Uuid::new_v4()),
        object: "response".to_string(),
        created: now,
        model: resp.model.unwrap_or_else(|| "local".to_string()),
        output: vec![],
        usage: resp.usage.map(|u| ResponsesUsage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            input_tokens_details: None,
            output_tokens_details: None,
        }),
        status: Some("completed".to_string()),
        previous_response_id: None,
        instructions: resp.system,
        reasoning: None,
    }
}

/// Convert a streaming `UnifiedStreamEvent` into a Chat Completions chunk.
pub fn to_chat_stream_chunk(event: UnifiedStreamEvent) -> ChatCompletionChunk {
    let id = uuid::Uuid::new_v4().to_string();
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let tool_calls = event.tool_call_delta.as_ref().map(|tc| {
        vec![crate::protocol::chat_completions::ChatStreamToolCall {
            index: 0,
            id: Some(tc.id.clone()),
            tool_type: Some("function".to_string()),
            function: Some(crate::protocol::chat_completions::ChatStreamFunction {
                name: Some(tc.name.clone()),
                arguments: Some(tc.arguments.clone()),
            }),
        }]
    });

    ChatCompletionChunk {
        id,
        object: "chat.completion.chunk".to_string(),
        created,
        model: "local".to_string(),
        choices: vec![ChatStreamChoice {
            index: 0,
            delta: ChatStreamDelta {
                role: None,
                content: if event.delta.is_empty() { None } else { Some(event.delta) },
                reasoning_content: None,
                tool_calls,
            },
            finish_reason: event.reason,
            logprobs: None,
        }],
    }
}

/// Convert a streaming `UnifiedStreamEvent` into an Anthropic Messages stream event.
pub fn to_messages_stream_event(event: UnifiedStreamEvent) -> MessagesStreamEvent {
    if let Some(reason) = &event.reason {
        let _ = reason;
        MessagesStreamEvent::MessageStop
    } else if let Some(tc) = event.tool_call_delta {
        MessagesStreamEvent::ContentBlockStart {
            index: 0,
            content_block: AnthropicContentBlock::ToolUse {
                id: tc.id,
                name: tc.name,
                input: serde_json::json!({ "arguments": tc.arguments }),
            },
        }
    } else {
        MessagesStreamEvent::ContentBlockDelta {
            index: 0,
            delta: crate::protocol::messages::ContentBlockDelta::TextDelta { text: event.delta },
        }
    }
}

/// Convert a streaming `UnifiedStreamEvent` into an OpenAI Responses stream event.
pub fn to_responses_stream_event(event: UnifiedStreamEvent) -> ResponsesStreamEvent {
    if let Some(reason) = &event.reason {
        if reason == "stop" {
            return ResponsesStreamEvent::ResponseCompleted {
                response: ResponsesResponse {
                    id: format!("resp_{}", uuid::Uuid::new_v4()),
                    object: "response".to_string(),
                    created: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                    model: "local".to_string(),
                    output: vec![],
                    usage: None,
                    status: Some("completed".to_string()),
                    previous_response_id: None,
                    instructions: None,
                    reasoning: None,
                },
            };
        }
    }

    if !event.delta.is_empty() {
        ResponsesStreamEvent::OutputTextDelta {
            item_id: uuid::Uuid::new_v4().to_string(),
            delta: event.delta,
        }
    } else {
        ResponsesStreamEvent::ResponseCreated {
            response: ResponsesResponse {
                id: format!("resp_{}", uuid::Uuid::new_v4()),
                object: "response".to_string(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                model: "local".to_string(),
                output: vec![],
                usage: None,
                status: Some("in_progress".to_string()),
                previous_response_id: None,
                instructions: None,
                reasoning: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render a list of chat messages into a single prompt string compatible with
/// llama.cpp's `/completion` endpoint format.
///
/// Returns `(prompt, system_prompt)`.
pub fn render_chat_messages(
    messages: &[crate::protocol::chat_completions::ChatMessage],
) -> (String, Option<String>) {
    let mut system = None;
    let mut prompt = String::new();

    for msg in messages {
        match msg.role.as_str() {
            "system" => {
                if let Some(content) = &msg.content {
                    let text = content_to_text(content);
                    system = Some(text.clone());
                    prompt.push_str(&format!("### System:\n{}\n\n", text));
                }
            }
            "user" => {
                if let Some(content) = &msg.content {
                    let text = content_to_text(content);
                    prompt.push_str(&format!("### User:\n{}\n\n", text));
                }
            }
            "assistant" => {
                if let Some(content) = &msg.content {
                    let text = content_to_text(content);
                    prompt.push_str(&format!("### Assistant:\n{}\n\n", text));
                }
                // Render tool calls
                if let Some(tool_calls) = &msg.tool_calls {
                    for tc in tool_calls {
                        prompt.push_str(&format!(
                            "### FunctionCall: {}({})\n\n",
                            tc.function.name, tc.function.arguments
                        ));
                    }
                }
            }
            "tool" | "function" => {
                if let Some(content) = &msg.content {
                    let text = content_to_text(content);
                    if let Some(id) = &msg.tool_call_id {
                        prompt.push_str(&format!("[tool result {}]\n{}\n\n", id, text));
                    } else {
                        prompt.push_str(&format!("### Tool:\n{}\n\n", text));
                    }
                }
            }
            _ => {
                prompt.push_str(&format!("### {}:\n", msg.role));
                if let Some(content) = &msg.content {
                    prompt.push_str(&content_to_text(content));
                }
                prompt.push_str("\n\n");
            }
        }
    }

    prompt.push_str("### Assistant:");
    (prompt, system)
}

fn content_to_text(content: &crate::protocol::chat_completions::ChatMessageContent) -> String {
    match content {
        crate::protocol::chat_completions::ChatMessageContent::Text(t) => t.clone(),
        crate::protocol::chat_completions::ChatMessageContent::Parts(parts) => {
            parts
                .iter()
                .filter_map(|p| match p {
                    crate::protocol::chat_completions::ContentPart::TextPart { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

/// Determine which protocol format a given path + content-type corresponds to.
pub fn detect_protocol(path: &str, content_type: Option<&str>) -> ProtocolFormat {
    let path_lower = path.to_lowercase();
    if path_lower.contains("/v1/chat/completions") {
        ProtocolFormat::ChatCompletions
    } else if path_lower.contains("/v1/messages") {
        ProtocolFormat::Messages
    } else if path_lower.contains("/v1/responses") {
        ProtocolFormat::Responses
    } else if path_lower.contains("/completion") {
        ProtocolFormat::LlamaCpp
    } else if let Some(ct) = content_type {
        if ct.contains("application/vnd.anthropic") {
            ProtocolFormat::Messages
        } else if ct.contains("openai") || ct.contains("completion") {
            ProtocolFormat::ChatCompletions
        } else {
            ProtocolFormat::Unknown
        }
    } else {
        ProtocolFormat::Unknown
    }
}

/// The external API format in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolFormat {
    ChatCompletions,
    Messages,
    Responses,
    LlamaCpp,
    Unknown,
}

/// Map a model display name to a short identifier used in responses.
pub fn model_id_from_name(name: &str) -> String {
    // Strip path/extension if a file path was passed.
    let stem = name
        .rsplit('/')
        .next()
        .unwrap_or(name)
        .rsplit('\\')
        .next()
        .unwrap_or(name);
    let stem = stem.split('.').next().unwrap_or(stem);
    stem.to_string()
}

/// Convert a `HashMap` of request headers into a `Vec` of (key, value) strings.
pub fn headers_to_vec(headers: &HashMap<String, String>) -> Vec<(String, String)> {
    headers.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_simple_chat() {
        let msgs = vec![
            crate::protocol::chat_completions::ChatMessage {
                role: "system".into(),
                content: Some(crate::protocol::chat_completions::ChatMessageContent::Text("You are helpful.".into())),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
            crate::protocol::chat_completions::ChatMessage {
                role: "user".into(),
                content: Some(crate::protocol::chat_completions::ChatMessageContent::Text("Hello".into())),
                name: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];
        let (prompt, system) = render_chat_messages(&msgs);
        assert!(prompt.contains("### User:\nHello"));
        assert_eq!(system, Some("You are helpful.".to_string()));
    }

    #[test]
    fn detect_protocol_routes() {
        assert_eq!(detect_protocol("/v1/chat/completions", None), ProtocolFormat::ChatCompletions);
        assert_eq!(detect_protocol("/v1/messages", None), ProtocolFormat::Messages);
        assert_eq!(detect_protocol("/v1/responses", None), ProtocolFormat::Responses);
        assert_eq!(detect_protocol("/completion", None), ProtocolFormat::LlamaCpp);
    }

    #[test]
    fn model_id_from_name_strips_extension() {
        assert_eq!(model_id_from_name("codeqwen-1.5b-q4_k_m.gguf"), "codeqwen-1.5b-q4_k_m");
        assert_eq!(model_id_from_name("./models/foo.gguf"), "foo");
    }
}
