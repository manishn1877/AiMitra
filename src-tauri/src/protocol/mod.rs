//! Protocol conversion module (ported from deepseek-recipe).
//!
//! Converts API requests in different formats (OpenAI Chat Completions,
//! Anthropic Messages, DeepSeek Responses) into a unified internal
//! representation, and converts model output back into the requested format.

pub mod chat_completions;
pub mod messages;
pub mod responses;
pub mod converter;
pub mod output_parser;