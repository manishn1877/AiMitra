//! OpenAI Chat Completions protocol conversion (ported from deepseek-recipe).

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Chat Completions request (OpenAI-compatible)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    #[serde(default)]
    pub response_format: Option<ResponseFormat>,
    #[serde(default)]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(default)]
    pub tool_choice: Option<ToolChoice>,
    #[serde(default)]
    pub logprobs: Option<bool>,
    #[serde(default)]
    pub top_logprobs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<ChatMessageContent>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ChatMessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ContentPart {
    TextPart { text: String },
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ToolChoice {
    Auto(String),
    Specific { function: FunctionChoice },
    Disabled { tool: ToolChoiceDisabled },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FunctionChoice {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolChoiceDisabled {
    pub tool: ToolDisabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolDisabled {
    #[serde(rename = "type")]
    pub tool_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(default)]
    pub json_schema: Option<serde_json::Value>,
}

/// Chat Completions response (OpenAI-compatible)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatResponseMessage,
    pub finish_reason: Option<String>,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ChatToolCall>>,
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ChatFunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub completion_tokens_details: Option<CompletionTokenDetails>,
    #[serde(default)]
    pub prompt_tokens_details: Option<PromptTokenDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CompletionTokenDetails {
    pub reasoning_tokens: u32,
    pub accepted_prediction_tokens: u32,
    pub rejected_prediction_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PromptTokenDetails {
    pub cached_tokens: u32,
}

/// Chat Completions streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatStreamChoice {
    pub index: u32,
    pub delta: ChatStreamDelta,
    pub finish_reason: Option<String>,
    pub logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatStreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub reasoning_content: Option<String>,
    pub tool_calls: Option<Vec<ChatStreamToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatStreamToolCall {
    pub index: u32,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub tool_type: Option<String>,
    pub function: Option<ChatStreamFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatStreamFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}