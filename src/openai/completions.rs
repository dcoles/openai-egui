//! Completions API.

use super::error::ErrorResponse;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Response {
    Error(ErrorResponse),
    Success(CompletionsResponse),
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct CompletionsResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct Choice {
    pub text: String,
    pub index: u64,
    pub finish_reason: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}
