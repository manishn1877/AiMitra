//! Anthropic Messages API protocol conversion.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Anthropic Messages API request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessagesRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(default)]
    pub system: Option<AnthropicSystemPrompt>,
    #[serde(default)]
    pub max_tokens: u32,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(default)]
    pub tool_choice: Option<AnthropicToolChoice>,
    #[serde(default)]
    pub metadata: Option<MessagesMetadata>,
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AnthropicSystemPrompt {
    Text(String),
    Blocks(Vec<AnthropicSystemBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnthropicSystemBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
    Text { text: String },
    Image { source: ImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: AnthropicContent },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AnthropicTool {
    Function {
        name: String,
        description: Option<String>,
        input_schema: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AnthropicToolChoice {
    Auto(ToolChoiceAuto),
    Any(ToolChoiceAny),
    Specific(ToolChoiceSpecific),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolChoiceAuto {
    #[serde(rename = "type")]
    pub choice_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolChoiceAny {
    #[serde(rename = "type")]
    pub choice_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolChoiceSpecific {
    #[serde(rename = "type")]
    pub choice_type: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessagesMetadata {
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThinkingConfig {
    #[serde(default)]
    pub thinking: bool,
    #[serde(default)]
    pub budget_tokens: Option<u32>,
}

/// Anthropic Messages API response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessagesResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<AnthropicResponseBlock>,
    pub stop_reason: Option<String>,
    pub usage: MessagesUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AnthropicResponseBlock {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessagesUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u32>,
}

/// Anthropic Messages streaming event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum MessagesStreamEvent {
    MessageStart { message: MessagesResponse },
    ContentBlockStart {
        index: u32,
        content_block: AnthropicResponseBlock,
    },
    ContentBlockDelta {
        index: u32,
        delta: ContentBlockDelta,
    },
    ContentBlockStop { index: u32 },
    MessageDelta {
        delta: MessageDelta,
        usage: MessagesUsage,
    },
    MessageStop,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ContentBlockDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    SignatureDelta { signature: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}