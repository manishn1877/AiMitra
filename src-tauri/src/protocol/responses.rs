//! Responses API protocol conversion (ported from deepseek-recipe).

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Responses API request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesRequest {
    pub model: String,
    pub input: ResponsesInput,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default)]
    pub stream: Option<bool>,
    #[serde(default)]
    pub tools: Option<Vec<ResponsesTool>>,
    #[serde(default)]
    pub tool_choice: Option<ResponsesToolChoice>,
    #[serde(default)]
    pub text: Option<ResponsesTextConfig>,
    #[serde(default)]
    pub reasoning: Option<ResponsesReasoningConfig>,
    #[serde(default)]
    pub parallel_tool_calls: Option<bool>,
    #[serde(default)]
    pub previous_response_id: Option<String>,
    #[serde(default)]
    pub store: Option<bool>,
    #[serde(default)]
    pub background: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ResponsesInput {
    Text(String),
    Items(Vec<ResponsesItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ResponsesItem {
    Message {
        role: String,
        content: ResponsesContent,
    },
    Reasoning {
        id: String,
        summary: Option<Vec<ResponsesSummary>>,
    },
    FunctionCall {
        id: String,
        call_id: String,
        name: String,
        arguments: String,
    },
    FunctionCallOutput {
        call_id: String,
        output: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ResponsesContent {
    Text(String),
    Parts(Vec<ResponsesContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ResponsesContentPart {
    InputText { text: String },
    OutputText { text: String },
    InputImage { image_url: String },
    InputFile { file_data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesSummary {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ResponsesTool {
    Function {
        name: String,
        description: Option<String>,
        parameters: Option<serde_json::Value>,
        strict: Option<bool>,
    },
    FileSearch {
        vector_store_ids: Vec<String>,
    },
    Computer {
        display_width: u32,
        display_height: u32,
    },
    WebSearch {
        #[serde(default)]
        user_location: Option<serde_json::Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ResponsesToolChoice {
    Auto(String),
    Disabled(String),
    Specific { function: ResponsesFunctionChoice },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesFunctionChoice {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesTextConfig {
    #[serde(default)]
    pub format: Option<ResponsesTextFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ResponsesTextFormat {
    Text,
    JsonSchema {
        #[serde(default)]
        schema: Option<serde_json::Value>,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        strict: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesReasoningConfig {
    #[serde(default)]
    pub effort: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

/// Responses API response
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub output: Vec<ResponsesItem>,
    pub usage: Option<ResponsesUsage>,
    pub status: Option<String>,
    pub previous_response_id: Option<String>,
    pub instructions: Option<String>,
    pub reasoning: Option<ResponsesReasoning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
    #[serde(default)]
    pub input_tokens_details: Option<ResponsesInputTokenDetails>,
    #[serde(default)]
    pub output_tokens_details: Option<ResponsesOutputTokenDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesInputTokenDetails {
    pub cached_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesOutputTokenDetails {
    pub reasoning_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesReasoning {
    pub effort: Option<String>,
    pub summary: Option<Vec<ResponsesSummary>>,
}

/// Responses API streaming event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum ResponsesStreamEvent {
    ResponseCreated { response: ResponsesResponse },
    ResponseInProgress { response: ResponsesResponse },
    ResponseCompleted { response: ResponsesResponse },
    OutputItemAdded { item: ResponsesItem },
    OutputItemDone { item: ResponsesItem },
    ContentPartAdded { item_id: String, part: ResponsesContentPart },
    ContentPartDone { item_id: String, part: ResponsesContentPart },
    OutputTextDelta { item_id: String, delta: String },
    OutputTextDone { item_id: String, text: String },
    ReasoningSummaryTextDelta { item_id: String, delta: String },
    ReasoningSummaryTextDone { item_id: String, text: String },
    ReasoningTextDelta { item_id: String, delta: String },
    ReasoningTextDone { item_id: String, text: String },
    FunctionCallArgumentsDelta { item_id: String, call_id: String, delta: String },
    FunctionCallArgumentsDone { item_id: String, call_id: String, arguments: String },
    ResponseFailed { error: ResponsesError },
    ResponseCanceled,
    Error { error: ResponsesError },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResponsesError {
    pub code: String,
    pub message: String,
    pub param: Option<String>,
    pub type: Option<String>,
}